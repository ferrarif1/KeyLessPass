----------------------------- MODULE ASTER -----------------------------
EXTENDS Naturals, Sequences

CONSTANTS Records, Epochs

States == {"Committed", "Prepared", "Unknown"}

VARIABLES epoch, state, oldEpoch, candEpoch

Init ==
  /\ epoch \in [Records -> Epochs]
  /\ state = [r \in Records |-> "Committed"]
  /\ oldEpoch = [r \in Records |-> epoch[r]]
  /\ candEpoch = [r \in Records |-> epoch[r]]

Prepare(r, e2) ==
  /\ state[r] = "Committed"
  /\ e2 # epoch[r]
  /\ oldEpoch' = [oldEpoch EXCEPT ![r] = epoch[r]]
  /\ candEpoch' = [candEpoch EXCEPT ![r] = e2]
  /\ state' = [state EXCEPT ![r] = "Prepared"]
  /\ UNCHANGED epoch

Ambiguous(r) ==
  /\ state[r] \in {"Prepared", "Unknown"}
  /\ state' = [state EXCEPT ![r] = "Unknown"]
  /\ UNCHANGED <<epoch, oldEpoch, candEpoch>>

CommitNew(r) ==
  /\ state[r] \in {"Prepared", "Unknown"}
  /\ epoch' = [epoch EXCEPT ![r] = candEpoch[r]]
  /\ state' = [state EXCEPT ![r] = "Committed"]
  /\ UNCHANGED <<oldEpoch, candEpoch>>

AbortOld(r) ==
  /\ state[r] \in {"Prepared", "Unknown"}
  /\ epoch' = [epoch EXCEPT ![r] = oldEpoch[r]]
  /\ state' = [state EXCEPT ![r] = "Committed"]
  /\ UNCHANGED <<oldEpoch, candEpoch>>

Next ==
  \/ \E r \in Records, e2 \in Epochs: Prepare(r, e2)
  \/ \E r \in Records: Ambiguous(r)
  \/ \E r \in Records: CommitNew(r)
  \/ \E r \in Records: AbortOld(r)

TypeOK ==
  /\ epoch \in [Records -> Epochs]
  /\ state \in [Records -> States]

UnknownPreservesOld ==
  \A r \in Records:
    state[r] = "Unknown" => epoch[r] = oldEpoch[r]

NoTransportCommit ==
  \A r \in Records:
    state[r] = "Unknown" => epoch[r] # candEpoch[r]

Spec == Init /\ [][Next]_<<epoch, state, oldEpoch, candEpoch>>

=============================================================================
