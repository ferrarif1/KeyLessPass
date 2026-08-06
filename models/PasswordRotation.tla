------------------------- MODULE PasswordRotation -------------------------
EXTENDS Naturals, TLC

VARIABLES state, localGeneration, remoteEvidence

vars == <<state, localGeneration, remoteEvidence>>

States == {
    "STABLE", "PREPARED", "UPDATE_SENT", "UNKNOWN_OUTCOME",
    "RECONCILIATION_REQUIRED", "REMOTE_CONFIRMED", "LOCAL_COMMITTED",
    "ABORTED", "ROLLBACK_REQUIRED", "AMBIGUOUS_REMOTE_STATE"
}

Evidence == {"NONE", "NEW_ONLY", "OLD_ONLY", "BOTH", "NEITHER"}

Init ==
    /\ state = "STABLE"
    /\ localGeneration = 0
    /\ remoteEvidence = "NONE"

Prepare ==
    /\ state = "STABLE"
    /\ localGeneration = 0
    /\ state' = "PREPARED"
    /\ UNCHANGED <<localGeneration, remoteEvidence>>

Send ==
    /\ state = "PREPARED"
    /\ state' = "UPDATE_SENT"
    /\ UNCHANGED <<localGeneration, remoteEvidence>>

VerifiedResponse ==
    /\ state = "UPDATE_SENT"
    /\ state' = "REMOTE_CONFIRMED"
    /\ remoteEvidence' = "NEW_ONLY"
    /\ UNCHANGED localGeneration

LostResponse ==
    /\ state = "UPDATE_SENT"
    /\ state' = "UNKNOWN_OUTCOME"
    /\ remoteEvidence' \in {"NEW_ONLY", "OLD_ONLY", "BOTH", "NEITHER"}
    /\ UNCHANGED localGeneration

BeginReconciliation ==
    /\ state = "UNKNOWN_OUTCOME"
    /\ state' = "RECONCILIATION_REQUIRED"
    /\ UNCHANGED <<localGeneration, remoteEvidence>>

ReconcileNewOnly ==
    /\ state = "RECONCILIATION_REQUIRED"
    /\ remoteEvidence = "NEW_ONLY"
    /\ state' = "REMOTE_CONFIRMED"
    /\ UNCHANGED <<localGeneration, remoteEvidence>>

ReconcileOldOnly ==
    /\ state = "RECONCILIATION_REQUIRED"
    /\ remoteEvidence = "OLD_ONLY"
    /\ state' = "ABORTED"
    /\ UNCHANGED <<localGeneration, remoteEvidence>>

ReconcileBoth ==
    /\ state = "RECONCILIATION_REQUIRED"
    /\ remoteEvidence = "BOTH"
    /\ state' = "AMBIGUOUS_REMOTE_STATE"
    /\ UNCHANGED <<localGeneration, remoteEvidence>>

ReconcileNeither ==
    /\ state = "RECONCILIATION_REQUIRED"
    /\ remoteEvidence = "NEITHER"
    /\ state' = "ROLLBACK_REQUIRED"
    /\ UNCHANGED <<localGeneration, remoteEvidence>>

CommitLocal ==
    /\ state = "REMOTE_CONFIRMED"
    /\ remoteEvidence = "NEW_ONLY"
    /\ state' = "LOCAL_COMMITTED"
    /\ localGeneration' = 1
    /\ UNCHANGED remoteEvidence

Next ==
    Prepare \/ Send \/ VerifiedResponse \/ LostResponse \/ BeginReconciliation
    \/ ReconcileNewOnly \/ ReconcileOldOnly \/ ReconcileBoth \/ ReconcileNeither
    \/ CommitLocal

TypeInvariant ==
    /\ state \in States
    /\ localGeneration \in 0..1
    /\ remoteEvidence \in Evidence

NoUnconfirmedCommit ==
    localGeneration = 1 => remoteEvidence = "NEW_ONLY"

AmbiguityNeverCommits ==
    state = "AMBIGUOUS_REMOTE_STATE" => localGeneration = 0

AbortNeverCommits ==
    state \in {"ABORTED", "ROLLBACK_REQUIRED"} => localGeneration = 0

Spec == Init /\ [][Next]_vars

=============================================================================
