------------------------- MODULE PasswordRotation -------------------------
EXTENDS Naturals, TLC

VARIABLES contract, state, phase, possible, localGeneration

vars == <<contract, state, phase, possible, localGeneration>>

Contracts == {"ATOMIC_REPLACEMENT", "OVERLAP_THEN_REVOKE", "OPAQUE_REPLACEMENT"}

States == {
    "STABLE", "PREPARED", "UPDATE_SENT", "UNKNOWN_OUTCOME",
    "RECONCILIATION_REQUIRED", "EVIDENCE_INSUFFICIENT",
    "REMOTE_CONFIRMED", "LOCAL_COMMITTED", "ABORTED",
    "ROLLBACK_REQUIRED", "AMBIGUOUS_REMOTE_STATE",
    "OVERLAP_ESTABLISHED", "OLD_REVOCATION_SENT",
    "OLD_REVOCATION_UNKNOWN"
}

RemoteStates == {"OLD_ONLY", "NEW_ONLY", "BOTH", "NEITHER"}
Phases == {"NONE", "UPDATE", "REVOKE"}

Init ==
    /\ contract \in Contracts
    /\ state = "STABLE"
    /\ phase = "NONE"
    /\ possible = RemoteStates
    /\ localGeneration = 0

Prepare ==
    /\ state = "STABLE"
    /\ localGeneration = 0
    /\ state' = "PREPARED"
    /\ phase' = "UPDATE"
    /\ possible' = RemoteStates
    /\ UNCHANGED <<contract, localGeneration>>

Send ==
    /\ state = "PREPARED"
    /\ state' = "UPDATE_SENT"
    /\ UNCHANGED <<contract, phase, possible, localGeneration>>

LoseUpdateResponse ==
    /\ state = "UPDATE_SENT"
    /\ state' = "UNKNOWN_OUTCOME"
    /\ UNCHANGED <<contract, phase, possible, localGeneration>>

BeginReconciliation ==
    /\ state = "UNKNOWN_OUTCOME"
    /\ state' = "RECONCILIATION_REQUIRED"
    /\ UNCHANGED <<contract, phase, possible, localGeneration>>

CanProbe == state \in {
    "RECONCILIATION_REQUIRED", "OLD_REVOCATION_SENT", "OLD_REVOCATION_UNKNOWN"
}

ProbeNewSuccess ==
    /\ CanProbe
    /\ possible \cap {"NEW_ONLY", "BOTH"} # {}
    /\ possible' = possible \cap {"NEW_ONLY", "BOTH"}
    /\ UNCHANGED <<contract, state, phase, localGeneration>>

ProbeNewFailure ==
    /\ CanProbe
    /\ possible \cap {"OLD_ONLY", "NEITHER"} # {}
    /\ possible' = possible \cap {"OLD_ONLY", "NEITHER"}
    /\ UNCHANGED <<contract, state, phase, localGeneration>>

ProbeOldSuccess ==
    /\ CanProbe
    /\ possible \cap {"OLD_ONLY", "BOTH"} # {}
    /\ possible' = possible \cap {"OLD_ONLY", "BOTH"}
    /\ UNCHANGED <<contract, state, phase, localGeneration>>

ProbeOldFailure ==
    /\ CanProbe
    /\ possible \cap {"NEW_ONLY", "NEITHER"} # {}
    /\ possible' = possible \cap {"NEW_ONLY", "NEITHER"}
    /\ UNCHANGED <<contract, state, phase, localGeneration>>

ClassifyNewOnly ==
    /\ CanProbe
    /\ possible = {"NEW_ONLY"}
    /\ state' = IF contract = "OPAQUE_REPLACEMENT"
                 THEN "EVIDENCE_INSUFFICIENT"
                 ELSE "REMOTE_CONFIRMED"
    /\ UNCHANGED <<contract, phase, possible, localGeneration>>

ClassifyOldOnly ==
    /\ CanProbe
    /\ possible = {"OLD_ONLY"}
    /\ state' = IF phase = "UPDATE" THEN "ABORTED" ELSE "ROLLBACK_REQUIRED"
    /\ UNCHANGED <<contract, phase, possible, localGeneration>>

ClassifyBoth ==
    /\ CanProbe
    /\ possible = {"BOTH"}
    /\ state' = IF contract = "OVERLAP_THEN_REVOKE"
                 THEN "OVERLAP_ESTABLISHED"
                 ELSE IF contract = "OPAQUE_REPLACEMENT"
                 THEN "EVIDENCE_INSUFFICIENT"
                 ELSE "AMBIGUOUS_REMOTE_STATE"
    /\ UNCHANGED <<contract, phase, possible, localGeneration>>

ClassifyNeither ==
    /\ CanProbe
    /\ possible = {"NEITHER"}
    /\ state' = IF contract = "OPAQUE_REPLACEMENT"
                 THEN "EVIDENCE_INSUFFICIENT"
                 ELSE "ROLLBACK_REQUIRED"
    /\ UNCHANGED <<contract, phase, possible, localGeneration>>

RequestOldRevocation ==
    /\ state = "OVERLAP_ESTABLISHED"
    /\ contract = "OVERLAP_THEN_REVOKE"
    /\ state' = "OLD_REVOCATION_SENT"
    /\ phase' = "REVOKE"
    /\ possible' = RemoteStates
    /\ UNCHANGED <<contract, localGeneration>>

LoseRevocationResponse ==
    /\ state = "OLD_REVOCATION_SENT"
    /\ state' = "OLD_REVOCATION_UNKNOWN"
    /\ UNCHANGED <<contract, phase, possible, localGeneration>>

CommitLocal ==
    /\ state = "REMOTE_CONFIRMED"
    /\ possible = {"NEW_ONLY"}
    /\ contract # "OPAQUE_REPLACEMENT"
    /\ state' = "LOCAL_COMMITTED"
    /\ localGeneration' = 1
    /\ UNCHANGED <<contract, phase, possible>>

Finalize ==
    /\ state = "LOCAL_COMMITTED"
    /\ state' = "STABLE"
    /\ UNCHANGED <<contract, phase, possible, localGeneration>>

Next ==
    Prepare \/ Send \/ LoseUpdateResponse \/ BeginReconciliation
    \/ ProbeNewSuccess \/ ProbeNewFailure \/ ProbeOldSuccess \/ ProbeOldFailure
    \/ ClassifyNewOnly \/ ClassifyOldOnly \/ ClassifyBoth \/ ClassifyNeither
    \/ RequestOldRevocation \/ LoseRevocationResponse \/ CommitLocal \/ Finalize

TypeInvariant ==
    /\ contract \in Contracts
    /\ state \in States
    /\ phase \in Phases
    /\ possible \subseteq RemoteStates
    /\ possible # {}
    /\ localGeneration \in 0..1

NoUnconfirmedCommit ==
    localGeneration = 1 => possible = {"NEW_ONLY"}

OpaqueTargetNeverCommits ==
    contract = "OPAQUE_REPLACEMENT" => localGeneration = 0

OverlapIsContractBound ==
    state = "OVERLAP_ESTABLISHED" =>
        /\ contract = "OVERLAP_THEN_REVOKE"
        /\ possible = {"BOTH"}

AtomicBothNeverCommits ==
    /\ contract = "ATOMIC_REPLACEMENT"
    /\ possible = {"BOTH"}
    => localGeneration = 0

Spec == Init /\ [][Next]_vars

=============================================================================
