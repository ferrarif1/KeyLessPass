---------------------------- MODULE lifecycle ----------------------------
EXTENDS Naturals, FiniteSets, TLC

(***************************************************************************)
(* Standalone EPSCD scheme-version-1 lifecycle abstraction. Cryptographic   *)
(* derivation is represented by the injective (epoch,generation) index.     *)
(***************************************************************************)

CONSTANTS MaxEpoch, MaxGeneration

Phases == {"Active", "Selecting", "Pending", "Submitted", "UnknownOutcome",
           "NewOnly", "OldOnly", "Both", "Neither", "Recovering"}
CandidateKinds == {"None", "SamePolicy", "PolicyChange"}
EvidenceStates == {"None", "NewOnly", "OldOnly", "Both", "Neither"}
GenerationPairs == (0..MaxEpoch) \X (0..MaxGeneration)

VARIABLES
    phase,
    activeEpoch,
    activeGeneration,
    activeSalt,
    candidateKind,
    candidateEpoch,
    candidateGeneration,
    candidateSalt,
    excludedGenerations,
    pendingPersisted,
    evidence,
    committedPairs,
    commitCount,
    baseEpoch,
    baseGeneration,
    baseCommitCount,
    lastCommitEvidence,
    lastCommittedCandidate,
    lastCommittedExclusions

vars == <<phase, activeEpoch, activeGeneration, activeSalt,
          candidateKind, candidateEpoch, candidateGeneration, candidateSalt,
          excludedGenerations, pendingPersisted, evidence, committedPairs,
          commitCount, baseEpoch, baseGeneration, baseCommitCount,
          lastCommitEvidence, lastCommittedCandidate,
          lastCommittedExclusions>>

Init ==
    /\ phase = "Active"
    /\ activeEpoch = 0
    /\ activeGeneration = 0
    /\ activeSalt = "credential-salt"
    /\ candidateKind = "None"
    /\ candidateEpoch = 0
    /\ candidateGeneration = 0
    /\ candidateSalt = "credential-salt"
    /\ excludedGenerations = {}
    /\ pendingPersisted = FALSE
    /\ evidence = "None"
    /\ committedPairs = {<<0, 0>>}
    /\ commitCount = 0
    /\ baseEpoch = 0
    /\ baseGeneration = 0
    /\ baseCommitCount = 0
    /\ lastCommitEvidence = "None"
    /\ lastCommittedCandidate = <<0, 0>>
    /\ lastCommittedExclusions = {}

PrepareSamePolicy ==
    /\ phase = "Active"
    /\ activeGeneration < MaxGeneration
    /\ phase' = "Selecting"
    /\ candidateKind' = "SamePolicy"
    /\ candidateEpoch' = activeEpoch
    /\ candidateGeneration' = activeGeneration + 1
    /\ candidateSalt' = activeSalt
    /\ excludedGenerations' \in SUBSET (0..MaxGeneration)
    /\ pendingPersisted' = FALSE
    /\ evidence' = "None"
    /\ baseEpoch' = activeEpoch
    /\ baseGeneration' = activeGeneration
    /\ baseCommitCount' = commitCount
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt, committedPairs,
                    commitCount, lastCommitEvidence, lastCommittedCandidate,
                    lastCommittedExclusions>>

PreparePolicyChange ==
    /\ phase = "Active"
    /\ activeEpoch < MaxEpoch
    /\ phase' = "Selecting"
    /\ candidateKind' = "PolicyChange"
    /\ candidateEpoch' = activeEpoch + 1
    /\ candidateGeneration' = 0
    /\ candidateSalt' = activeSalt
    /\ excludedGenerations' \in SUBSET (0..MaxGeneration)
    /\ pendingPersisted' = FALSE
    /\ evidence' = "None"
    /\ baseEpoch' = activeEpoch
    /\ baseGeneration' = activeGeneration
    /\ baseCommitCount' = commitCount
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt, committedPairs,
                    commitCount, lastCommitEvidence, lastCommittedCandidate,
                    lastCommittedExclusions>>

SkipExcluded ==
    /\ phase = "Selecting"
    /\ candidateGeneration \in excludedGenerations
    /\ candidateGeneration < MaxGeneration
    /\ candidateGeneration' = candidateGeneration + 1
    /\ UNCHANGED <<phase, activeEpoch, activeGeneration, activeSalt,
                    candidateKind, candidateEpoch, candidateSalt,
                    excludedGenerations, pendingPersisted, evidence,
                    committedPairs, commitCount, baseEpoch, baseGeneration,
                    baseCommitCount, lastCommitEvidence,
                    lastCommittedCandidate, lastCommittedExclusions>>

PersistCandidate ==
    /\ phase = "Selecting"
    /\ candidateGeneration \notin excludedGenerations
    /\ <<candidateEpoch, candidateGeneration>> \notin committedPairs
    /\ phase' = "Pending"
    /\ pendingPersisted' = TRUE
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt,
                    candidateKind, candidateEpoch, candidateGeneration,
                    candidateSalt, excludedGenerations, evidence,
                    committedPairs, commitCount, baseEpoch, baseGeneration,
                    baseCommitCount, lastCommitEvidence,
                    lastCommittedCandidate, lastCommittedExclusions>>

Submit ==
    /\ phase = "Pending"
    /\ pendingPersisted
    /\ phase' = "Submitted"
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt,
                    candidateKind, candidateEpoch, candidateGeneration,
                    candidateSalt, excludedGenerations, pendingPersisted,
                    evidence, committedPairs, commitCount, baseEpoch,
                    baseGeneration, baseCommitCount, lastCommitEvidence,
                    lastCommittedCandidate, lastCommittedExclusions>>

LoseResponse ==
    /\ phase = "Submitted"
    /\ phase' = "UnknownOutcome"
    /\ evidence' = "None"
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt,
                    candidateKind, candidateEpoch, candidateGeneration,
                    candidateSalt, excludedGenerations, pendingPersisted,
                    committedPairs, commitCount, baseEpoch, baseGeneration,
                    baseCommitCount, lastCommitEvidence,
                    lastCommittedCandidate, lastCommittedExclusions>>

Observe(remoteState) ==
    /\ phase \in {"Submitted", "UnknownOutcome"}
    /\ remoteState \in {"NewOnly", "OldOnly", "Both", "Neither"}
    /\ phase' = remoteState
    /\ evidence' = remoteState
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt,
                    candidateKind, candidateEpoch, candidateGeneration,
                    candidateSalt, excludedGenerations, pendingPersisted,
                    committedPairs, commitCount, baseEpoch, baseGeneration,
                    baseCommitCount, lastCommitEvidence,
                    lastCommittedCandidate, lastCommittedExclusions>>

Commit ==
    /\ phase = "NewOnly"
    /\ evidence = "NewOnly"
    /\ pendingPersisted
    /\ candidateGeneration \notin excludedGenerations
    /\ <<candidateEpoch, candidateGeneration>> \notin committedPairs
    /\ phase' = "Active"
    /\ activeEpoch' = candidateEpoch
    /\ activeGeneration' = candidateGeneration
    /\ activeSalt' = candidateSalt
    /\ candidateKind' = "None"
    /\ pendingPersisted' = FALSE
    /\ committedPairs' = committedPairs \cup
                            {<<candidateEpoch, candidateGeneration>>}
    /\ commitCount' = commitCount + 1
    /\ lastCommitEvidence' = evidence
    /\ lastCommittedCandidate' = <<candidateEpoch, candidateGeneration>>
    /\ lastCommittedExclusions' = excludedGenerations
    /\ UNCHANGED <<candidateEpoch, candidateGeneration, candidateSalt,
                    excludedGenerations, evidence, baseEpoch, baseGeneration,
                    baseCommitCount>>

Abort ==
    /\ phase = "OldOnly"
    /\ evidence = "OldOnly"
    /\ phase' = "Active"
    /\ candidateKind' = "None"
    /\ pendingPersisted' = FALSE
    /\ evidence' = "None"
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt,
                    candidateEpoch, candidateGeneration, candidateSalt,
                    excludedGenerations, committedPairs, commitCount,
                    baseEpoch, baseGeneration, baseCommitCount,
                    lastCommitEvidence, lastCommittedCandidate,
                    lastCommittedExclusions>>

EscalateAmbiguous ==
    /\ phase \in {"Both", "Neither"}
    /\ phase' = "Recovering"
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt,
                    candidateKind, candidateEpoch, candidateGeneration,
                    candidateSalt, excludedGenerations, pendingPersisted,
                    evidence, committedPairs, commitCount, baseEpoch,
                    baseGeneration, baseCommitCount, lastCommitEvidence,
                    lastCommittedCandidate, lastCommittedExclusions>>

Crash ==
    /\ pendingPersisted
    /\ phase \in {"Pending", "Submitted", "UnknownOutcome", "NewOnly",
                    "OldOnly", "Both", "Neither"}
    /\ phase' = "Recovering"
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt,
                    candidateKind, candidateEpoch, candidateGeneration,
                    candidateSalt, excludedGenerations, pendingPersisted,
                    evidence, committedPairs, commitCount, baseEpoch,
                    baseGeneration, baseCommitCount, lastCommitEvidence,
                    lastCommittedCandidate, lastCommittedExclusions>>

Recover ==
    /\ phase = "Recovering"
    /\ pendingPersisted
    /\ phase' = "UnknownOutcome"
    /\ evidence' = "None"
    /\ UNCHANGED <<activeEpoch, activeGeneration, activeSalt,
                    candidateKind, candidateEpoch, candidateGeneration,
                    candidateSalt, excludedGenerations, pendingPersisted,
                    committedPairs, commitCount, baseEpoch, baseGeneration,
                    baseCommitCount, lastCommitEvidence,
                    lastCommittedCandidate, lastCommittedExclusions>>

Next ==
    \/ PrepareSamePolicy
    \/ PreparePolicyChange
    \/ SkipExcluded
    \/ PersistCandidate
    \/ Submit
    \/ LoseResponse
    \/ \E remoteState \in {"NewOnly", "OldOnly", "Both", "Neither"} :
          Observe(remoteState)
    \/ Commit
    \/ Abort
    \/ EscalateAmbiguous
    \/ Crash
    \/ Recover

Spec == Init /\ [][Next]_vars

TypeInvariant ==
    /\ phase \in Phases
    /\ activeEpoch \in 0..MaxEpoch
    /\ activeGeneration \in 0..MaxGeneration
    /\ activeSalt = "credential-salt"
    /\ candidateKind \in CandidateKinds
    /\ candidateEpoch \in 0..MaxEpoch
    /\ candidateGeneration \in 0..MaxGeneration
    /\ candidateSalt = "credential-salt"
    /\ excludedGenerations \subseteq (0..MaxGeneration)
    /\ pendingPersisted \in BOOLEAN
    /\ evidence \in EvidenceStates
    /\ committedPairs \subseteq GenerationPairs
    /\ commitCount \in Nat
    /\ baseEpoch \in 0..MaxEpoch
    /\ baseGeneration \in 0..MaxGeneration
    /\ baseCommitCount \in Nat
    /\ lastCommitEvidence \in EvidenceStates
    /\ lastCommittedCandidate \in GenerationPairs
    /\ lastCommittedExclusions \subseteq (0..MaxGeneration)

NoCommitWithoutNewOnly ==
    commitCount > 0 => lastCommitEvidence = "NewOnly"

CommittedGenerationMatchesCandidate ==
    commitCount > 0 =>
        <<activeEpoch, activeGeneration>> = lastCommittedCandidate

CredentialSaltStableWithinLineage ==
    candidateKind # "None" => candidateSalt = activeSalt

PolicyChangeAdvancesEpoch ==
    candidateKind = "PolicyChange" => candidateEpoch = baseEpoch + 1

SamePolicyRotationPreservesEpoch ==
    candidateKind = "SamePolicy" => candidateEpoch = baseEpoch

NoCommittedHistoryReuse ==
    commitCount > 0 => activeGeneration \notin lastCommittedExclusions

UnknownOutcomeDoesNotCommit ==
    phase = "UnknownOutcome" =>
        /\ activeEpoch = baseEpoch
        /\ activeGeneration = baseGeneration
        /\ commitCount = baseCommitCount

PendingBeforeSubmit ==
    phase \in {"Submitted", "UnknownOutcome", "NewOnly", "OldOnly",
               "Both", "Neither", "Recovering"} => pendingPersisted

=============================================================================
