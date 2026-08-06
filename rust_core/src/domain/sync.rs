use crate::domain::CredentialDescriptionRecord;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReplicaRelation {
    Identical,
    LeftDescendsFromRight,
    RightDescendsFromLeft,
    LeftStale,
    RightStale,
    ConcurrentModification,
    ForkedHistory,
    DuplicateOperationConflict,
    CannotMerge,
}

pub fn compare_replicas(
    left: &CredentialDescriptionRecord,
    right: &CredentialDescriptionRecord,
) -> Result<ReplicaRelation> {
    if left.canonical_bytes()? == right.canonical_bytes()? {
        return Ok(ReplicaRelation::Identical);
    }
    if left.vault_id != right.vault_id || left.record_id != right.record_id {
        return Ok(ReplicaRelation::CannotMerge);
    }
    if left.operation_id.is_some() && left.operation_id == right.operation_id {
        return Ok(ReplicaRelation::DuplicateOperationConflict);
    }
    let left_hash = left.record_hash()?;
    let right_hash = right.record_hash()?;
    if left.parent_record_hash == right_hash {
        return Ok(ReplicaRelation::LeftDescendsFromRight);
    }
    if right.parent_record_hash == left_hash {
        return Ok(ReplicaRelation::RightDescendsFromLeft);
    }
    if left.credential_generation == right.credential_generation {
        return Ok(if left.parent_record_hash == right.parent_record_hash {
            ReplicaRelation::ConcurrentModification
        } else {
            ReplicaRelation::ForkedHistory
        });
    }
    Ok(
        if left.credential_generation < right.credential_generation {
            ReplicaRelation::LeftStale
        } else {
            ReplicaRelation::RightStale
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EncodingDescriptor;
    use uuid::Uuid;

    fn active() -> CredentialDescriptionRecord {
        CredentialDescriptionRecord::new(
            Uuid::new_v4(),
            1,
            1,
            "service",
            "service.example",
            "account",
            "",
            EncodingDescriptor::default(),
        )
    }

    #[test]
    fn distinguishes_descendant_concurrent_and_cross_vault_records() {
        let parent = active();
        let child =
            CredentialDescriptionRecord::rotation_from(&parent, EncodingDescriptor::default());
        assert_eq!(
            compare_replicas(&child, &parent).unwrap(),
            ReplicaRelation::LeftDescendsFromRight
        );

        let mut concurrent = child.clone();
        concurrent.operation_id = Some(Uuid::new_v4());
        concurrent.salt = crate::crypto::random_base64(16);
        assert_eq!(
            compare_replicas(&child, &concurrent).unwrap(),
            ReplicaRelation::ConcurrentModification
        );

        let unrelated = active();
        assert_eq!(
            compare_replicas(&parent, &unrelated).unwrap(),
            ReplicaRelation::CannotMerge
        );
    }
}
