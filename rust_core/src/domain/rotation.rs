use crate::domain::{CredentialDescriptionRecord, CredentialState, RotationState};
use crate::error::{KeylessPassError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RotationEvent {
    RequestSent,
    RemoteNewPasswordVerified,
    RemoteRejected,
    TransportOutcomeUnknown,
    BeginReconciliation,
    NewPasswordAuthenticated,
    OldPasswordAuthenticated,
    BothPasswordsAuthenticated,
    NeitherPasswordAuthenticated,
    CommitLocal,
    Finalize,
    Abort,
    Supersede,
}

pub fn transition_rotation(
    record: &mut CredentialDescriptionRecord,
    event: RotationEvent,
) -> Result<RotationState> {
    use RotationEvent::*;
    use RotationState::*;
    let next = match (&record.rotation_state, event) {
        (Prepared, RequestSent) => UpdateSent,
        (Prepared, Abort) => Aborted,
        (UpdateSent, RemoteNewPasswordVerified) => RemoteConfirmed,
        (UpdateSent, RemoteRejected) => RollbackRequired,
        (UpdateSent, TransportOutcomeUnknown) => UnknownOutcome,
        (UnknownOutcome, BeginReconciliation) => ReconciliationRequired,
        (ReconciliationRequired, NewPasswordAuthenticated) => RemoteConfirmed,
        (ReconciliationRequired, OldPasswordAuthenticated) => Aborted,
        (ReconciliationRequired, BothPasswordsAuthenticated) => AmbiguousRemoteState,
        (ReconciliationRequired, NeitherPasswordAuthenticated) => RollbackRequired,
        (RemoteConfirmed, CommitLocal) => LocalCommitted,
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
    use RotationEvent::*;
    use RotationState::*;

    fn pending() -> CredentialDescriptionRecord {
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
        CredentialDescriptionRecord::rotation_from(&active, EncodingDescriptor::default())
    }

    #[test]
    fn unknown_outcome_reconciles_to_all_observable_remote_states() {
        for (event, expected) in [
            (NewPasswordAuthenticated, RemoteConfirmed),
            (OldPasswordAuthenticated, Aborted),
            (BothPasswordsAuthenticated, AmbiguousRemoteState),
            (NeitherPasswordAuthenticated, RollbackRequired),
        ] {
            let mut record = pending();
            transition_rotation(&mut record, RequestSent).unwrap();
            transition_rotation(&mut record, TransportOutcomeUnknown).unwrap();
            transition_rotation(&mut record, BeginReconciliation).unwrap();
            assert_eq!(transition_rotation(&mut record, event).unwrap(), expected);
        }
    }

    #[test]
    fn illegal_and_replayed_transitions_are_rejected() {
        let mut record = pending();
        assert!(transition_rotation(&mut record, CommitLocal).is_err());
        transition_rotation(&mut record, RequestSent).unwrap();
        assert!(transition_rotation(&mut record, RequestSent).is_err());
    }

    #[test]
    fn exhaustive_event_exploration_preserves_state_invariants() {
        let events = [
            RequestSent,
            RemoteNewPasswordVerified,
            RemoteRejected,
            TransportOutcomeUnknown,
            BeginReconciliation,
            NewPasswordAuthenticated,
            OldPasswordAuthenticated,
            BothPasswordsAuthenticated,
            NeitherPasswordAuthenticated,
            CommitLocal,
            Finalize,
            Abort,
            Supersede,
        ];
        let mut frontier = vec![pending()];
        for _depth in 0..12 {
            let mut next = Vec::new();
            for record in frontier {
                for event in events {
                    let mut candidate = record.clone();
                    if transition_rotation(&mut candidate, event).is_ok() {
                        match candidate.rotation_state {
                            Stable | LocalCommitted => {
                                assert_eq!(candidate.state, CredentialState::Active)
                            }
                            Aborted | Superseded => {
                                assert_eq!(candidate.state, CredentialState::Retired)
                            }
                            _ => assert_eq!(candidate.state, CredentialState::PendingRotation),
                        }
                        next.push(candidate);
                    }
                }
            }
            frontier = next;
        }
    }
}
