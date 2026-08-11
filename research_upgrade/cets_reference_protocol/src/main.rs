use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Instant;
use voprf::{PoprfClient, PoprfServer, Ristretto255};

const PROTOCOL: &str = "cets-two-server-poprf-v1";
const OPERATION: &str = "derive-credential";
const NOW_EPOCH: u64 = 100;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize)]
struct Context {
    service: String,
    account: String,
    lineage: String,
    root_generation: String,
    policy: String,
}

impl Context {
    fn canonical(&self) -> Vec<u8> {
        encode_fields(&[
            ("service", &self.service),
            ("account", &self.account),
            ("lineage", &self.lineage),
            ("root-generation", &self.root_generation),
            ("policy", &self.policy),
        ])
    }

    fn service_account_projection(&self) -> Vec<u8> {
        encode_fields(&[("service", &self.service), ("account", &self.account)])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TicketScope {
    Exact,
    ServiceAccount,
    Wildcard,
}

impl TicketScope {
    fn tag(self) -> u8 {
        match self {
            Self::Exact => 1,
            Self::ServiceAccount => 2,
            Self::Wildcard => 3,
        }
    }

    fn canonical_projection(self, context: &Context) -> Vec<u8> {
        match self {
            Self::Exact => context.canonical(),
            Self::ServiceAccount => context.service_account_projection(),
            Self::Wildcard => encode_fields(&[]),
        }
    }
}

#[derive(Clone)]
struct Ticket {
    scope: TicketScope,
    binding: [u8; 32],
    issued_epoch: u64,
    expires_epoch: u64,
    freshness_generation: u64,
    nonce: [u8; 16],
    signature: Signature,
}

impl Ticket {
    fn issue(scope: TicketScope, context: &Context, signer: &SigningKey) -> Self {
        let binding = binding_digest(scope, context);
        let issued_epoch = NOW_EPOCH - 1;
        let expires_epoch = NOW_EPOCH + 1;
        let freshness_generation = 7;
        let mut nonce = [0u8; 16];
        OsRng.fill_bytes(&mut nonce);
        let payload = ticket_payload(
            scope,
            &binding,
            issued_epoch,
            expires_epoch,
            freshness_generation,
            &nonce,
        );
        Self {
            scope,
            binding,
            issued_epoch,
            expires_epoch,
            freshness_generation,
            nonce,
            signature: signer.sign(&payload),
        }
    }

    fn payload(&self) -> Vec<u8> {
        ticket_payload(
            self.scope,
            &self.binding,
            self.issued_epoch,
            self.expires_epoch,
            self.freshness_generation,
            &self.nonce,
        )
    }

    fn authorizes(
        &self,
        context: &Context,
        configured_scope: TicketScope,
        verifier: &VerifyingKey,
    ) -> bool {
        self.scope == configured_scope
            && verifier.verify(&self.payload(), &self.signature).is_ok()
            && self.issued_epoch <= NOW_EPOCH
            && NOW_EPOCH < self.expires_epoch
            && self.freshness_generation == 7
            && self.binding == binding_digest(configured_scope, context)
    }
}

fn encode_fields(fields: &[(&str, &String)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(PROTOCOL.as_bytes());
    for (name, value) in fields {
        append_len_prefixed(&mut out, name.as_bytes());
        append_len_prefixed(&mut out, value.as_bytes());
    }
    out
}

fn append_len_prefixed(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(bytes);
}

fn binding_digest(scope: TicketScope, context: &Context) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"cets-ticket-binding-v1");
    hash.update(scope.canonical_projection(context));
    hash.finalize().into()
}

fn ticket_payload(
    scope: TicketScope,
    binding: &[u8; 32],
    issued_epoch: u64,
    expires_epoch: u64,
    freshness_generation: u64,
    nonce: &[u8; 16],
) -> Vec<u8> {
    let mut out = Vec::new();
    append_len_prefixed(&mut out, PROTOCOL.as_bytes());
    append_len_prefixed(&mut out, OPERATION.as_bytes());
    out.push(scope.tag());
    out.extend_from_slice(binding);
    out.extend_from_slice(&issued_epoch.to_be_bytes());
    out.extend_from_slice(&expires_epoch.to_be_bytes());
    out.extend_from_slice(&freshness_generation.to_be_bytes());
    out.extend_from_slice(nonce);
    out
}

struct ReferenceProtocol {
    evaluator_a: PoprfServer<Ristretto255>,
    evaluator_b: PoprfServer<Ristretto255>,
    approval_verifier: VerifyingKey,
    client_input: [u8; 32],
}

impl ReferenceProtocol {
    fn new(approval_verifier: VerifyingKey) -> Result<Self, Box<dyn Error>> {
        let mut rng = OsRng;
        let mut client_input = [0u8; 32];
        rng.fill_bytes(&mut client_input);
        Ok(Self {
            evaluator_a: PoprfServer::new(&mut rng).map_err(protocol_error)?,
            evaluator_b: PoprfServer::new(&mut rng).map_err(protocol_error)?,
            approval_verifier,
            client_input,
        })
    }

    fn derive(
        &self,
        context: &Context,
        ticket: &Ticket,
        configured_scope: TicketScope,
    ) -> Result<[u8; 32], Box<dyn Error>> {
        if !ticket.authorizes(context, configured_scope, &self.approval_verifier) {
            return Err(
                "ticket does not authorize the requested context under its binding scope".into(),
            );
        }

        let info = context.canonical();
        let output_a = evaluate_poprf(&self.evaluator_a, &self.client_input, &info)?;
        let output_b = evaluate_poprf(&self.evaluator_b, &self.client_input, &info)?;

        let mut hash = Sha256::new();
        hash.update(b"cets-combined-credential-output-v1");
        hash.update(output_a);
        hash.update(output_b);
        hash.update(&info);
        Ok(hash.finalize().into())
    }
}

fn evaluate_poprf(
    server: &PoprfServer<Ristretto255>,
    private_input: &[u8],
    public_info: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut rng = OsRng;
    let blind =
        PoprfClient::<Ristretto255>::blind(private_input, &mut rng).map_err(protocol_error)?;
    let evaluation = server
        .blind_evaluate(&mut rng, &blind.message, Some(public_info))
        .map_err(protocol_error)?;
    let output = blind
        .state
        .finalize(
            private_input,
            &evaluation.message,
            &evaluation.proof,
            server.get_public_key(),
            Some(public_info),
        )
        .map_err(protocol_error)?;
    Ok(output.to_vec())
}

fn protocol_error(error: voprf::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::Other,
        format!("POPRF protocol error: {error:?}"),
    )
}

fn all_contexts() -> Vec<Context> {
    let mut contexts = Vec::new();
    for service in ["svc0", "svc1"] {
        for account in ["acct0", "acct1"] {
            for lineage in ["lin0", "lin1"] {
                for root_generation in ["root0", "root1"] {
                    for policy in ["pol0", "pol1"] {
                        contexts.push(Context {
                            service: service.into(),
                            account: account.into(),
                            lineage: lineage.into(),
                            root_generation: root_generation.into(),
                            policy: policy.into(),
                        });
                    }
                }
            }
        }
    }
    contexts
}

fn percentile(values: &[u128], percentile: usize) -> u128 {
    let rank = ((percentile * values.len()) + 99) / 100;
    values[rank.saturating_sub(1).min(values.len() - 1)]
}

#[derive(Serialize)]
struct ScopeResult {
    scope: TicketScope,
    exposed_contexts: usize,
    unauthorized_spill: usize,
    distinct_outputs: usize,
    cardinality_spectrum: Vec<usize>,
}

#[derive(Serialize)]
struct BenchmarkResult {
    repetitions: usize,
    median_microseconds: u128,
    p95_microseconds: u128,
}

#[derive(Serialize)]
struct ExperimentResult {
    protocol: &'static str,
    primitive: &'static str,
    client_input_model: &'static str,
    ticket_binding_model: &'static str,
    context_fields: [&'static str; 5],
    contexts: usize,
    udc_domain_threshold: usize,
    minimum_udc_witness: [&'static str; 2],
    alternate_udc_witness: [&'static str; 3],
    scopes: Vec<ScopeResult>,
    complete_derivation_benchmark: BenchmarkResult,
    properties: Properties,
}

#[derive(Serialize)]
struct Properties {
    repeated_context_is_deterministic: bool,
    all_context_outputs_are_distinct_in_sample: bool,
    tampered_ticket_is_rejected: bool,
}

fn analyze_scope(
    protocol: &ReferenceProtocol,
    signer: &SigningKey,
    contexts: &[Context],
    scope: TicketScope,
) -> Result<ScopeResult, Box<dyn Error>> {
    let ticket = Ticket::issue(scope, &contexts[0], signer);
    let mut outputs = HashSet::new();
    let mut exposed: usize = 0;
    for context in contexts {
        if let Ok(output) = protocol.derive(context, &ticket, scope) {
            exposed += 1;
            outputs.insert(output);
        }
    }
    let unauthorized_spill = exposed.saturating_sub(1);
    let mut spectrum = vec![1; unauthorized_spill];
    spectrum.extend(std::iter::repeat(2).take(contexts.len() - unauthorized_spill));
    Ok(ScopeResult {
        scope,
        exposed_contexts: exposed,
        unauthorized_spill,
        distinct_outputs: outputs.len(),
        cardinality_spectrum: spectrum,
    })
}

fn run_experiment() -> Result<ExperimentResult, Box<dyn Error>> {
    let signer = fresh_signing_key();
    let protocol = ReferenceProtocol::new(signer.verifying_key())?;
    let contexts = all_contexts();
    let scopes = [
        TicketScope::Exact,
        TicketScope::ServiceAccount,
        TicketScope::Wildcard,
    ]
    .into_iter()
    .map(|scope| analyze_scope(&protocol, &signer, &contexts, scope))
    .collect::<Result<Vec<_>, _>>()?;

    let exact_ticket = Ticket::issue(TicketScope::Exact, &contexts[0], &signer);
    let first = protocol.derive(&contexts[0], &exact_ticket, TicketScope::Exact)?;
    let second = protocol.derive(&contexts[0], &exact_ticket, TicketScope::Exact)?;

    let wildcard_ticket = Ticket::issue(TicketScope::Wildcard, &contexts[0], &signer);
    let mut all_outputs = HashSet::new();
    for context in &contexts {
        all_outputs.insert(protocol.derive(context, &wildcard_ticket, TicketScope::Wildcard)?);
    }

    let mut tampered = exact_ticket.clone();
    tampered.expires_epoch += 10;
    let tampered_ticket_is_rejected = protocol
        .derive(&contexts[0], &tampered, TicketScope::Exact)
        .is_err();

    let repetitions = 1_000;
    let mut timings = Vec::with_capacity(repetitions);
    for _ in 0..repetitions {
        let start = Instant::now();
        protocol.derive(&contexts[0], &exact_ticket, TicketScope::Exact)?;
        timings.push(start.elapsed().as_micros());
    }
    timings.sort_unstable();

    Ok(ExperimentResult {
        protocol: PROTOCOL,
        primitive: "two independent RFC 9497 POPRF evaluations (ristretto255-SHA512)",
        client_input_model: "one stable 32-byte endpoint-held input shared by all configured contexts",
        ticket_binding_model: "H(cets-ticket-binding-v1 || Canon(project_scope(context))); the signed scope tag must equal the evaluator configuration",
        context_fields: [
            "service",
            "account",
            "credential_lineage",
            "root_generation",
            "policy_identity",
        ],
        contexts: contexts.len(),
        udc_domain_threshold: 2,
        minimum_udc_witness: ["endpoint", "approval_authority"],
        alternate_udc_witness: ["endpoint", "evaluator_a", "evaluator_b"],
        scopes,
        complete_derivation_benchmark: BenchmarkResult {
            repetitions,
            median_microseconds: percentile(&timings, 50),
            p95_microseconds: percentile(&timings, 95),
        },
        properties: Properties {
            repeated_context_is_deterministic: first == second,
            all_context_outputs_are_distinct_in_sample: all_outputs.len() == contexts.len(),
            tampered_ticket_is_rejected,
        },
    })
}

fn fresh_signing_key() -> SigningKey {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    SigningKey::from_bytes(&secret)
}

fn main() -> Result<(), Box<dyn Error>> {
    let result = run_experiment()?;
    let json = serde_json::to_string_pretty(&result)?;
    let mut args = env::args().skip(1);
    if args.next().as_deref() == Some("--output") {
        let path = PathBuf::from(args.next().ok_or("--output requires a path")?);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, format!("{json}\n"))?;
    } else {
        println!("{json}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (SigningKey, ReferenceProtocol, Vec<Context>) {
        let signer = fresh_signing_key();
        let protocol = ReferenceProtocol::new(signer.verifying_key()).unwrap();
        (signer, protocol, all_contexts())
    }

    #[test]
    fn scope_cardinalities_match_projection_classes() {
        let (signer, protocol, contexts) = setup();
        let expected = [
            (TicketScope::Exact, 1),
            (TicketScope::ServiceAccount, 8),
            (TicketScope::Wildcard, 32),
        ];
        for (scope, count) in expected {
            let result = analyze_scope(&protocol, &signer, &contexts, scope).unwrap();
            assert_eq!(result.exposed_contexts, count);
            assert_eq!(result.distinct_outputs, count);
        }
    }

    #[test]
    fn same_context_is_deterministic_and_contexts_are_separated() {
        let (signer, protocol, contexts) = setup();
        let ticket = Ticket::issue(TicketScope::Wildcard, &contexts[0], &signer);
        let first = protocol
            .derive(&contexts[0], &ticket, TicketScope::Wildcard)
            .unwrap();
        let repeated = protocol
            .derive(&contexts[0], &ticket, TicketScope::Wildcard)
            .unwrap();
        let other = protocol
            .derive(&contexts[1], &ticket, TicketScope::Wildcard)
            .unwrap();
        assert_eq!(first, repeated);
        assert_ne!(first, other);
    }

    #[test]
    fn signature_and_lifetime_are_enforced() {
        let (signer, protocol, contexts) = setup();
        let mut ticket = Ticket::issue(TicketScope::Exact, &contexts[0], &signer);
        assert!(protocol
            .derive(&contexts[0], &ticket, TicketScope::Exact)
            .is_ok());
        assert!(protocol
            .derive(&contexts[0], &ticket, TicketScope::ServiceAccount)
            .is_err());
        ticket.expires_epoch += 1;
        assert!(protocol
            .derive(&contexts[0], &ticket, TicketScope::Exact)
            .is_err());
    }

    #[test]
    fn canonical_encoding_is_unambiguous_for_reference_contexts() {
        let contexts = all_contexts();
        let encodings: HashSet<_> = contexts.iter().map(Context::canonical).collect();
        assert_eq!(encodings.len(), contexts.len());

        assert_ne!(
            binding_digest(TicketScope::Exact, &contexts[0]),
            binding_digest(TicketScope::Exact, &contexts[1])
        );
        assert_eq!(
            binding_digest(TicketScope::ServiceAccount, &contexts[0]),
            binding_digest(TicketScope::ServiceAccount, &contexts[1])
        );
        assert_eq!(
            binding_digest(TicketScope::Wildcard, &contexts[0]),
            binding_digest(TicketScope::Wildcard, &contexts[1])
        );
    }
}
