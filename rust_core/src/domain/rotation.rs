use crate::domain::{CredentialDescriptionRecord, CredentialState, RotationState};
use crate::error::{KeylessPassError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const MAX_PERSISTED_PROBES: usize = 64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRequirement {
    /// The target contract only requires proof that the new credential works.
    NewAcceptance,
    /// The target contract requires proof that new works and old no longer works.
    NewOnly,
    /// The adapter exposes an authoritative remote version/readback value.
    AuthoritativeVersion,
    /// The adapter cannot safely establish a commit predicate.
    UnknownOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdapterCapabilities {
    pub can_verify_new: bool,
    pub can_verify_old_safely: bool,
    pub has_atomic_success_evidence: bool,
    pub has_remote_version: bool,
    pub supports_idempotency_key: bool,
    pub evidence_requirement: EvidenceRequirement,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdapterObservation {
    /// A transport/application success response alone is never commit evidence.
    pub update_response_success: bool,
    pub new_probe: Option<ProbeVerdict>,
    pub old_probe: Option<ProbeVerdict>,
    pub atomic_success_evidence: bool,
    pub observed_remote_version: Option<u64>,
    pub expected_remote_version: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommitEvidence {
    SufficientNewAcceptance,
    SufficientNewOnly,
    SufficientAtomicEvidence,
    SufficientRemoteVersion,
    Insufficient,
    Contradictory,
}

impl CommitEvidence {
    pub fn permits_commit(self) -> bool {
        matches!(
            self,
            Self::SufficientNewAcceptance
                | Self::SufficientNewOnly
                | Self::SufficientAtomicEvidence
                | Self::SufficientRemoteVersion
        )
    }
}

/// Adapter-specific commit predicate. Network delivery and a generic success
/// response are intentionally ignored unless accompanied by declared evidence.
pub fn classify_commit_evidence(
    capabilities: AdapterCapabilities,
    observation: AdapterObservation,
) -> CommitEvidence {
    use CommitEvidence::*;
    use EvidenceRequirement::*;
    use ProbeVerdict::{ConclusiveFailure, Success};

    if observation.new_probe == Some(ConclusiveFailure)
        && observation.old_probe == Some(ConclusiveFailure)
    {
        return Contradictory;
    }
    if capabilities.has_atomic_success_evidence && observation.atomic_success_evidence {
        return SufficientAtomicEvidence;
    }
    if capabilities.has_remote_version
        && observation.expected_remote_version.is_some()
        && observation.observed_remote_version == observation.expected_remote_version
    {
        return SufficientRemoteVersion;
    }

    match capabilities.evidence_requirement {
        NewAcceptance if capabilities.can_verify_new && observation.new_probe == Some(Success) => {
            SufficientNewAcceptance
        }
        NewOnly
            if capabilities.can_verify_new
                && capabilities.can_verify_old_safely
                && observation.new_probe == Some(Success)
                && observation.old_probe == Some(ConclusiveFailure) =>
        {
            SufficientNewOnly
        }
        AuthoritativeVersion | UnknownOnly | NewAcceptance | NewOnly => Insufficient,
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RotationContract {
    AtomicReplacement,
    OverlapThenRevoke,
    #[default]
    OpaqueReplacement,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RemoteCredentialState {
    OldOnly,
    NewOnly,
    Both,
    Neither,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProbedCredential {
    Old,
    New,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeVerdict {
    Success,
    ConclusiveFailure,
    Indeterminate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationProbe {
    pub credential: ProbedCredential,
    pub verdict: ProbeVerdict,
    pub endpoint_id: String,
    pub observed_at: DateTime<Utc>,
}

impl AuthenticationProbe {
    pub fn now(
        credential: ProbedCredential,
        verdict: ProbeVerdict,
        endpoint_id: impl Into<String>,
    ) -> Self {
        Self {
            credential,
            verdict,
            endpoint_id: endpoint_id.into(),
            observed_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RotationEvidence {
    pub possible_states: Vec<RemoteCredentialState>,
    pub probes: Vec<AuthenticationProbe>,
}

impl Default for RotationEvidence {
    fn default() -> Self {
        Self {
            possible_states: vec![
                RemoteCredentialState::OldOnly,
                RemoteCredentialState::NewOnly,
                RemoteCredentialState::Both,
                RemoteCredentialState::Neither,
            ],
            probes: Vec::new(),
        }
    }
}

impl RotationEvidence {
    pub fn observe(&mut self, probe: AuthenticationProbe) -> Result<()> {
        if probe.endpoint_id.trim().is_empty() {
            return Err(KeylessPassError::Validation(
                "rotation evidence requires an endpoint identity".to_string(),
            ));
        }
        if self.probes.len() >= MAX_PERSISTED_PROBES {
            return Err(KeylessPassError::Validation(
                "rotation evidence probe limit reached".to_string(),
            ));
        }

        let mut next = self.possible_states.clone();
        next.retain(|state| probe_consistent(*state, probe.credential, probe.verdict));
        if next.is_empty() {
            return Err(KeylessPassError::Integrity(
                "rotation probes are mutually inconsistent".to_string(),
            ));
        }
        self.possible_states = next;
        self.probes.push(probe);
        Ok(())
    }

    pub fn singleton(&self) -> Option<RemoteCredentialState> {
        (self.possible_states.len() == 1).then_some(self.possible_states[0])
    }

    pub fn reset_after_remote_mutation(&mut self) {
        *self = Self::default();
    }
}

fn probe_consistent(
    state: RemoteCredentialState,
    credential: ProbedCredential,
    verdict: ProbeVerdict,
) -> bool {
    use ProbeVerdict::*;
    use ProbedCredential::*;
    use RemoteCredentialState::*;
    match (credential, verdict) {
        (_, Indeterminate) => true,
        (Old, Success) => matches!(state, OldOnly | Both),
        (Old, ConclusiveFailure) => matches!(state, NewOnly | Neither),
        (New, Success) => matches!(state, NewOnly | Both),
        (New, ConclusiveFailure) => matches!(state, OldOnly | Neither),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RotationEvent {
    RequestSent,
    RemoteNewPasswordVerified,
    RemoteRejected,
    TransportOutcomeUnknown,
    BeginReconciliation,
    EvidenceNewOnly,
    EvidenceOldOnly,
    EvidenceBoth,
    EvidenceNeither,
    EvidenceInsufficient,
    RequestOldRevocation,
    OldRevocationOutcomeUnknown,
    CommitLocal,
    Finalize,
    Abort,
    Supersede,
}

pub fn apply_authentication_probe(
    record: &mut CredentialDescriptionRecord,
    probe: AuthenticationProbe,
) -> Result<RotationState> {
    use RotationEvent::*;
    use RotationState::*;

    match record.rotation_state {
        Prepared => {
            transition_rotation(record, RequestSent)?;
            transition_rotation(record, TransportOutcomeUnknown)?;
            transition_rotation(record, BeginReconciliation)?;
        }
        UpdateSent => {
            transition_rotation(record, TransportOutcomeUnknown)?;
            transition_rotation(record, BeginReconciliation)?;
        }
        UnknownOutcome => {
            transition_rotation(record, BeginReconciliation)?;
        }
        ReconciliationRequired | OldRevocationSent | OldRevocationUnknown => {}
        _ => {
            return Err(KeylessPassError::Validation(format!(
                "rotation state {:?} does not accept authentication probes",
                record.rotation_state
            )))
        }
    }

    let evidence = record
        .rotation_evidence
        .get_or_insert_with(Default::default);
    evidence.observe(probe)?;
    let Some(remote_state) = evidence.singleton() else {
        record.updated_at = Utc::now();
        return Ok(record.rotation_state.clone());
    };

    let contract = record.rotation_contract.unwrap_or_default();
    if contract == RotationContract::OpaqueReplacement {
        return transition_rotation(record, RotationEvent::EvidenceInsufficient);
    }

    let event = match remote_state {
        RemoteCredentialState::NewOnly => EvidenceNewOnly,
        RemoteCredentialState::OldOnly => EvidenceOldOnly,
        RemoteCredentialState::Both => EvidenceBoth,
        RemoteCredentialState::Neither => EvidenceNeither,
    };
    transition_rotation(record, event)
}

pub fn transition_rotation(
    record: &mut CredentialDescriptionRecord,
    event: RotationEvent,
) -> Result<RotationState> {
    use RotationContract::*;
    use RotationEvent::*;
    use RotationState::*;

    let contract = record.rotation_contract.unwrap_or_default();
    let next = match (&record.rotation_state, event) {
        (Prepared, RequestSent) => UpdateSent,
        (Prepared, Abort) => Aborted,
        (UpdateSent, RemoteNewPasswordVerified)
            if record
                .rotation_evidence
                .as_ref()
                .and_then(RotationEvidence::singleton)
                == Some(RemoteCredentialState::NewOnly)
                && contract != OpaqueReplacement =>
        {
            RemoteConfirmed
        }
        (UpdateSent, RemoteRejected) => RollbackRequired,
        (UpdateSent, TransportOutcomeUnknown) => UnknownOutcome,
        (UnknownOutcome, BeginReconciliation) => ReconciliationRequired,
        (ReconciliationRequired, EvidenceNewOnly) => RemoteConfirmed,
        (ReconciliationRequired, EvidenceOldOnly) => Aborted,
        (ReconciliationRequired, EvidenceBoth) if contract == OverlapThenRevoke => {
            OverlapEstablished
        }
        (ReconciliationRequired, EvidenceBoth) => AmbiguousRemoteState,
        (ReconciliationRequired, EvidenceNeither) => RollbackRequired,
        (ReconciliationRequired, RotationEvent::EvidenceInsufficient) => {
            RotationState::EvidenceInsufficient
        }
        (OverlapEstablished, RequestOldRevocation) => OldRevocationSent,
        (OldRevocationSent, OldRevocationOutcomeUnknown)
        | (OldRevocationSent, TransportOutcomeUnknown) => OldRevocationUnknown,
        (OldRevocationSent, EvidenceNewOnly) | (OldRevocationUnknown, EvidenceNewOnly) => {
            RemoteConfirmed
        }
        (OldRevocationSent, EvidenceBoth) | (OldRevocationUnknown, EvidenceBoth) => {
            OverlapEstablished
        }
        (OldRevocationSent, EvidenceOldOnly)
        | (OldRevocationUnknown, EvidenceOldOnly)
        | (OldRevocationSent, EvidenceNeither)
        | (OldRevocationUnknown, EvidenceNeither) => RollbackRequired,
        (RemoteConfirmed, CommitLocal)
            if record
                .rotation_evidence
                .as_ref()
                .and_then(RotationEvidence::singleton)
                == Some(RemoteCredentialState::NewOnly) =>
        {
            LocalCommitted
        }
        (LocalCommitted, Finalize) => Stable,
        (RollbackRequired, Abort) => Aborted,
        (Aborted, Supersede) => Superseded,
        _ => {
            return Err(KeylessPassError::Validation(format!(
                "illegal rotation transition from {:?} using {:?}",
                record.rotation_state, event
            )))
        }
    };

    if next == OldRevocationSent {
        record
            .rotation_evidence
            .get_or_insert_with(Default::default)
            .reset_after_remote_mutation();
    }
    record.rotation_state = next.clone();
    record.updated_at = Utc::now();
    record.state = match next {
        Stable | LocalCommitted => CredentialState::Active,
        Superseded | Aborted => CredentialState::Retired,
        _ => CredentialState::PendingRotation,
    };
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EncodingDescriptor;
    use uuid::Uuid;

    fn pending(contract: RotationContract) -> CredentialDescriptionRecord {
        let active = CredentialDescriptionRecord::new(
            Uuid::new_v4(),
            1,
            1,
            "service",
            "service.example",
            "account",
            "",
            EncodingDescriptor::default(),
        );
        CredentialDescriptionRecord::rotation_from_with_contract(
            &active,
            EncodingDescriptor::default(),
            contract,
        )
    }

    fn probe(credential: ProbedCredential, verdict: ProbeVerdict) -> AuthenticationProbe {
        AuthenticationProbe::now(credential, verdict, "node-a")
    }

    #[test]
    fn evidence_refines_to_new_only_without_boolean_guessing() {
        let mut evidence = RotationEvidence::default();
        evidence
            .observe(probe(ProbedCredential::New, ProbeVerdict::Success))
            .unwrap();
        assert_eq!(
            evidence.possible_states,
            vec![RemoteCredentialState::NewOnly, RemoteCredentialState::Both]
        );
        evidence
            .observe(probe(
                ProbedCredential::Old,
                ProbeVerdict::ConclusiveFailure,
            ))
            .unwrap();
        assert_eq!(evidence.singleton(), Some(RemoteCredentialState::NewOnly));
    }

    #[test]
    fn contradictory_probes_are_rejected() {
        let mut evidence = RotationEvidence::default();
        evidence
            .observe(probe(ProbedCredential::New, ProbeVerdict::Success))
            .unwrap();
        assert!(evidence
            .observe(probe(
                ProbedCredential::New,
                ProbeVerdict::ConclusiveFailure,
            ))
            .is_err());
    }

    #[test]
    fn atomic_replacement_requires_new_only_evidence() {
        let mut record = pending(RotationContract::AtomicReplacement);
        apply_authentication_probe(
            &mut record,
            probe(ProbedCredential::New, ProbeVerdict::Success),
        )
        .unwrap();
        assert_eq!(record.rotation_state, RotationState::ReconciliationRequired);
        apply_authentication_probe(
            &mut record,
            probe(ProbedCredential::Old, ProbeVerdict::ConclusiveFailure),
        )
        .unwrap();
        assert_eq!(record.rotation_state, RotationState::RemoteConfirmed);
        assert_eq!(
            transition_rotation(&mut record, RotationEvent::CommitLocal).unwrap(),
            RotationState::LocalCommitted
        );
    }

    #[test]
    fn overlap_contract_treats_both_valid_as_a_planned_stage() {
        let mut record = pending(RotationContract::OverlapThenRevoke);
        apply_authentication_probe(
            &mut record,
            probe(ProbedCredential::New, ProbeVerdict::Success),
        )
        .unwrap();
        apply_authentication_probe(
            &mut record,
            probe(ProbedCredential::Old, ProbeVerdict::Success),
        )
        .unwrap();
        assert_eq!(record.rotation_state, RotationState::OverlapEstablished);

        transition_rotation(&mut record, RotationEvent::RequestOldRevocation).unwrap();
        apply_authentication_probe(
            &mut record,
            probe(ProbedCredential::New, ProbeVerdict::Success),
        )
        .unwrap();
        apply_authentication_probe(
            &mut record,
            probe(ProbedCredential::Old, ProbeVerdict::ConclusiveFailure),
        )
        .unwrap();
        assert_eq!(record.rotation_state, RotationState::RemoteConfirmed);
    }

    #[test]
    fn opaque_target_never_auto_commits_from_local_probes() {
        let mut record = pending(RotationContract::OpaqueReplacement);
        apply_authentication_probe(
            &mut record,
            probe(ProbedCredential::New, ProbeVerdict::Success),
        )
        .unwrap();
        apply_authentication_probe(
            &mut record,
            probe(ProbedCredential::Old, ProbeVerdict::ConclusiveFailure),
        )
        .unwrap();
        assert_eq!(record.rotation_state, RotationState::EvidenceInsufficient);
        assert!(transition_rotation(&mut record, RotationEvent::CommitLocal).is_err());
    }

    #[test]
    fn http_success_alone_never_satisfies_commit_predicate() {
        let evidence = classify_commit_evidence(
            AdapterCapabilities {
                can_verify_new: true,
                can_verify_old_safely: false,
                has_atomic_success_evidence: false,
                has_remote_version: false,
                supports_idempotency_key: false,
                evidence_requirement: EvidenceRequirement::NewAcceptance,
            },
            AdapterObservation {
                update_response_success: true,
                ..Default::default()
            },
        );
        assert_eq!(evidence, CommitEvidence::Insufficient);
        assert!(!evidence.permits_commit());
    }

    #[test]
    fn unsafe_old_probe_can_be_omitted_for_acceptance_only_adapter() {
        let evidence = classify_commit_evidence(
            AdapterCapabilities {
                can_verify_new: true,
                can_verify_old_safely: false,
                has_atomic_success_evidence: false,
                has_remote_version: false,
                supports_idempotency_key: true,
                evidence_requirement: EvidenceRequirement::NewAcceptance,
            },
            AdapterObservation {
                new_probe: Some(ProbeVerdict::Success),
                ..Default::default()
            },
        );
        assert_eq!(evidence, CommitEvidence::SufficientNewAcceptance);
    }

    #[test]
    fn new_only_adapter_requires_both_safe_probes() {
        let capabilities = AdapterCapabilities {
            can_verify_new: true,
            can_verify_old_safely: true,
            has_atomic_success_evidence: false,
            has_remote_version: false,
            supports_idempotency_key: false,
            evidence_requirement: EvidenceRequirement::NewOnly,
        };
        let partial = classify_commit_evidence(
            capabilities,
            AdapterObservation {
                new_probe: Some(ProbeVerdict::Success),
                ..Default::default()
            },
        );
        assert_eq!(partial, CommitEvidence::Insufficient);
        let complete = classify_commit_evidence(
            capabilities,
            AdapterObservation {
                new_probe: Some(ProbeVerdict::Success),
                old_probe: Some(ProbeVerdict::ConclusiveFailure),
                ..Default::default()
            },
        );
        assert_eq!(complete, CommitEvidence::SufficientNewOnly);
    }
}
