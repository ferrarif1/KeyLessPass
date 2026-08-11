------------------------ MODULE epscd_rotation ------------------------
EXTENDS Naturals, FiniteSets, TLC

CONSTANTS MaxGeneration, AllowHttp200Commit, DropCandidateOnUnknown

Phases == {"Active", "Prepared", "Submitted", "Verifying",
           "Committed", "Aborted", "UnknownOutcome"}
RemoteStates == {"OldOnly", "NewOnly", "Both", "Neither"}
Requirements == {"NewAcceptance", "NewOnly", "AuthoritativeVersion", "UnknownOnly"}
EvidenceKinds == {"None", "NewAcceptance", "NewOnly", "RemoteVersion"}

AcceptsNew(remote) == remote \in {"NewOnly", "Both"}
EvidenceSufficient(requirement, kind) ==
    \/ kind = "NewOnly"
    \/ kind = "RemoteVersion"
    \/ /\ requirement = "NewAcceptance"
       /\ kind = "NewAcceptance"

VARIABLES phase,
          committedGeneration,
          candidateGeneration,
          operationPersisted,
          adapterRequirement,
          remoteState,
          evidence,
          transportSuccess,
          oldReconstructible,
          newReconstructible,
          committedGenerations,
          lastCommitEvidence,
          lastCommitRemoteState

vars == <<phase, committedGeneration, candidateGeneration,
          operationPersisted, adapterRequirement, remoteState, evidence,
          transportSuccess, oldReconstructible, newReconstructible,
          committedGenerations, lastCommitEvidence, lastCommitRemoteState>>

Init ==
    /\ phase = "Active"
    /\ committedGeneration = 0
    /\ candidateGeneration = 0
    /\ operationPersisted = FALSE
    /\ adapterRequirement = "UnknownOnly"
    /\ remoteState = "OldOnly"
    /\ evidence = "None"
    /\ transportSuccess = FALSE
    /\ oldReconstructible = TRUE
    /\ newReconstructible = FALSE
    /\ committedGenerations = {0}
    /\ lastCommitEvidence = "None"
    /\ lastCommitRemoteState = "OldOnly"

Prepare(requirement) ==
    /\ phase \in {"Active", "Committed", "Aborted"}
    /\ committedGeneration < MaxGeneration
    /\ requirement \in Requirements
    /\ phase' = "Prepared"
    /\ candidateGeneration' = committedGeneration + 1
    /\ operationPersisted' = TRUE
    /\ adapterRequirement' = requirement
    /\ remoteState' = "OldOnly"
    /\ evidence' = "None"
    /\ transportSuccess' = FALSE
    /\ oldReconstructible' = TRUE
    /\ newReconstructible' = TRUE
    /\ UNCHANGED <<committedGeneration, committedGenerations,
                    lastCommitEvidence, lastCommitRemoteState>>

Submit(remote) ==
    /\ phase = "Prepared"
    /\ operationPersisted
    /\ remote \in RemoteStates
    /\ phase' = "Submitted"
    /\ remoteState' = remote
    /\ transportSuccess' \in BOOLEAN
    /\ UNCHANGED <<committedGeneration, candidateGeneration,
                    operationPersisted, adapterRequirement, evidence,
                    oldReconstructible, newReconstructible,
                    committedGenerations, lastCommitEvidence,
                    lastCommitRemoteState>>

BeginVerify ==
    /\ phase \in {"Submitted", "UnknownOutcome"}
    /\ phase' = "Verifying"
    /\ evidence' = "None"
    /\ UNCHANGED <<committedGeneration, candidateGeneration,
                    operationPersisted, adapterRequirement, remoteState,
                    transportSuccess, oldReconstructible,
                    newReconstructible, committedGenerations,
                    lastCommitEvidence, lastCommitRemoteState>>

ObserveNewAcceptance ==
    /\ phase = "Verifying"
    /\ adapterRequirement = "NewAcceptance"
    /\ AcceptsNew(remoteState)
    /\ evidence' = "NewAcceptance"
    /\ UNCHANGED <<phase, committedGeneration, candidateGeneration,
                    operationPersisted, adapterRequirement, remoteState,
                    transportSuccess, oldReconstructible,
                    newReconstructible, committedGenerations,
                    lastCommitEvidence, lastCommitRemoteState>>

ObserveNewOnly ==
    /\ phase = "Verifying"
    /\ adapterRequirement = "NewOnly"
    /\ remoteState = "NewOnly"
    /\ evidence' = "NewOnly"
    /\ UNCHANGED <<phase, committedGeneration, candidateGeneration,
                    operationPersisted, adapterRequirement, remoteState,
                    transportSuccess, oldReconstructible,
                    newReconstructible, committedGenerations,
                    lastCommitEvidence, lastCommitRemoteState>>

ObserveVersion ==
    /\ phase = "Verifying"
    /\ adapterRequirement = "AuthoritativeVersion"
    /\ remoteState = "NewOnly"
    /\ evidence' = "RemoteVersion"
    /\ UNCHANGED <<phase, committedGeneration, candidateGeneration,
                    operationPersisted, adapterRequirement, remoteState,
                    transportSuccess, oldReconstructible,
                    newReconstructible, committedGenerations,
                    lastCommitEvidence, lastCommitRemoteState>>

Commit ==
    /\ phase = "Verifying"
    /\ EvidenceSufficient(adapterRequirement, evidence)
    /\ AcceptsNew(remoteState)
    /\ phase' = "Committed"
    /\ committedGeneration' = candidateGeneration
    /\ operationPersisted' = FALSE
    /\ oldReconstructible' = FALSE
    /\ newReconstructible' = TRUE
    /\ committedGenerations' = committedGenerations \cup {candidateGeneration}
    /\ lastCommitEvidence' = evidence
    /\ lastCommitRemoteState' = remoteState
    /\ UNCHANGED <<candidateGeneration, adapterRequirement, remoteState,
                    evidence, transportSuccess>>

Abort ==
    /\ phase \in {"Prepared", "Verifying"}
    /\ remoteState = "OldOnly"
    /\ phase' = "Aborted"
    /\ operationPersisted' = FALSE
    /\ oldReconstructible' = TRUE
    /\ newReconstructible' = FALSE
    /\ UNCHANGED <<committedGeneration, candidateGeneration,
                    adapterRequirement, remoteState, evidence,
                    transportSuccess, committedGenerations,
                    lastCommitEvidence, lastCommitRemoteState>>

BecomeUnknown ==
    /\ phase \in {"Submitted", "Verifying"}
    /\ phase' = "UnknownOutcome"
    /\ evidence' = "None"
    /\ oldReconstructible' = TRUE
    /\ newReconstructible' = IF DropCandidateOnUnknown THEN FALSE ELSE TRUE
    /\ UNCHANGED <<committedGeneration, candidateGeneration,
                    operationPersisted, adapterRequirement, remoteState,
                    transportSuccess, committedGenerations,
                    lastCommitEvidence, lastCommitRemoteState>>

Crash ==
    /\ phase \in {"Prepared", "Submitted", "Verifying"}
    /\ operationPersisted
    /\ phase' = "UnknownOutcome"
    /\ evidence' = "None"
    /\ oldReconstructible' = TRUE
    /\ newReconstructible' = IF DropCandidateOnUnknown THEN FALSE ELSE TRUE
    /\ UNCHANGED <<committedGeneration, candidateGeneration,
                    operationPersisted, adapterRequirement, remoteState,
                    transportSuccess, committedGenerations,
                    lastCommitEvidence, lastCommitRemoteState>>

BadHttpCommit ==
    /\ AllowHttp200Commit
    /\ phase = "Submitted"
    /\ transportSuccess
    /\ phase' = "Committed"
    /\ committedGeneration' = candidateGeneration
    /\ operationPersisted' = FALSE
    /\ oldReconstructible' = FALSE
    /\ newReconstructible' = TRUE
    /\ committedGenerations' = committedGenerations \cup {candidateGeneration}
    /\ lastCommitEvidence' = "None"
    /\ lastCommitRemoteState' = remoteState
    /\ UNCHANGED <<candidateGeneration, adapterRequirement, remoteState,
                    evidence, transportSuccess>>

Next ==
    \/ \E requirement \in Requirements : Prepare(requirement)
    \/ \E remote \in RemoteStates : Submit(remote)
    \/ BeginVerify
    \/ ObserveNewAcceptance
    \/ ObserveNewOnly
    \/ ObserveVersion
    \/ Commit
    \/ Abort
    \/ BecomeUnknown
    \/ Crash
    \/ BadHttpCommit

Spec == Init /\ [][Next]_vars

TypeInvariant ==
    /\ phase \in Phases
    /\ committedGeneration \in 0..MaxGeneration
    /\ candidateGeneration \in 0..MaxGeneration
    /\ operationPersisted \in BOOLEAN
    /\ adapterRequirement \in Requirements
    /\ remoteState \in RemoteStates
    /\ evidence \in EvidenceKinds
    /\ transportSuccess \in BOOLEAN
    /\ oldReconstructible \in BOOLEAN
    /\ newReconstructible \in BOOLEAN
    /\ committedGenerations \subseteq (0..MaxGeneration)

CommitRequiresEvidence ==
    phase = "Committed" => EvidenceSufficient(adapterRequirement, lastCommitEvidence)

CommitMatchesRemoteAcceptance ==
    phase = "Committed" => AcceptsNew(lastCommitRemoteState)

UncertaintyKeepsBoth ==
    phase = "UnknownOutcome" => oldReconstructible /\ newReconstructible

PreparedBeforeSubmission ==
    phase \in {"Submitted", "Verifying", "UnknownOutcome"} => operationPersisted

UnknownDoesNotAdvance ==
    phase = "UnknownOutcome" => committedGeneration < candidateGeneration

SequentialUniqueGenerations ==
    /\ Cardinality(committedGenerations) = committedGeneration + 1
    /\ committedGenerations = 0..committedGeneration

=============================================================================
