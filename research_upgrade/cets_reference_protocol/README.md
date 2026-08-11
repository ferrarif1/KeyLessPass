# CETS two-server POPRF reference protocol

This artifact instantiates the CETS context analysis with real cryptographic
operations. It composes two independent RFC 9497 POPRF evaluators: a client
must verify and combine both outputs, so neither evaluator alone derives the
final credential value. An independent Ed25519 authority signs a short-lived
ticket that both evaluators verify before answering.

The client samples one stable 32-byte derivation input for the reference run.
It is available to the modeled compromised endpoint for every configured
context and is not an additional authorization factor; POPRF blinding keeps it
private from either evaluator in the protocol transcript.

The minimum UDC witness is compromise of the endpoint and approval authority:
the endpoint supplies the stable client input and arbitrary contexts, while the
authority can mint valid tickets that the two honest evaluators accept. A
separate UDC path compromises the endpoint and both evaluator domains. The
evaluator pair alone is not a UDC witness because neither evaluator learns the
client input.

The artifact is deliberately a reference case, not a new OPRF or threshold-PRF
construction. For binding-field set `B`, the ticket digest is the
domain-separated hash of the canonical projection `project_B(context)`; the
signed scope identifier must also match the evaluator configuration. The correct mode selects
all five fields. Two negative controls select only service/account or the empty
projection, while POPRF evaluation still receives the complete context as RFC
9497 public information.

Run:

```sh
cargo test --locked
cargo run --release --locked -- --output results/reference_protocol.json
```

The 32-context experiment varies five binary fields: service, account,
credential lineage, Root generation, and policy identity. The expected
single-ticket exposed-set sizes are 1, 8, and 32 for exact,
service/account-projected, and wildcard tickets respectively.
