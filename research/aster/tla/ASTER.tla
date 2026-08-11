----------------------------- MODULE ASTER -----------------------------
EXTENDS Naturals, FiniteSets

CONSTANTS AllowHttpCommit,
          DropCandidateOnTimeout,
          DisableReplayLimit,
          DisableExpiry,
          DisableFreshnessBinding,
          DisableRootBinding,
          DisableGenerationBinding,
          AllowUnsafeRetirement

Phases == {"Committed", "Prepared", "Submitted", "Verifying", "Unknown"}
Evidence == {"None", "NewOnly", "OldOnly", "Both", "Neither", "Unavailable"}
Epochs == {1, 2}

VARIABLES phase,
          committedEpoch, committedGeneration,
          oldEpoch, oldGeneration,
          candidateEpoch, candidateGeneration,
          oldPresent, candidatePresent,
          evidence, lastEvent, advanced,
          ticketExpired, ticketRevoked, ticketUses,
          requestFresh, requestRootMatches, requestGenerationMatches,
          evaluatedCount, availableEpochs

vars == <<phase, committedEpoch, committedGeneration,
          oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
          oldPresent, candidatePresent, evidence, lastEvent, advanced,
          ticketExpired, ticketRevoked, ticketUses,
          requestFresh, requestRootMatches, requestGenerationMatches,
          evaluatedCount, availableEpochs>>

Init ==
  /\ phase = "Committed"
  /\ committedEpoch = 1
  /\ committedGeneration = 0
  /\ oldEpoch = 1
  /\ oldGeneration = 0
  /\ candidateEpoch = 2
  /\ candidateGeneration = 1
  /\ oldPresent = FALSE
  /\ candidatePresent = FALSE
  /\ evidence = "None"
  /\ lastEvent = "Init"
  /\ advanced = FALSE
  /\ ticketExpired = FALSE
  /\ ticketRevoked = FALSE
  /\ ticketUses = 0
  /\ requestFresh = TRUE
  /\ requestRootMatches = TRUE
  /\ requestGenerationMatches = TRUE
  /\ evaluatedCount = 0
  /\ availableEpochs = Epochs

Prepare ==
  /\ phase = "Committed"
  /\ candidateEpoch # committedEpoch
  /\ phase' = "Prepared"
  /\ oldEpoch' = committedEpoch
  /\ oldGeneration' = committedGeneration
  /\ oldPresent' = TRUE
  /\ candidatePresent' = TRUE
  /\ lastEvent' = "PreparedFsync"
  /\ UNCHANGED <<committedEpoch, committedGeneration,
                  candidateEpoch, candidateGeneration, evidence, advanced,
                  ticketExpired, ticketRevoked, ticketUses,
                  requestFresh, requestRootMatches, requestGenerationMatches,
                  evaluatedCount, availableEpochs>>

Submit ==
  /\ phase = "Prepared"
  /\ phase' = "Submitted"
  /\ lastEvent' = "RequestSent"
  /\ UNCHANGED <<committedEpoch, committedGeneration,
                  oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  oldPresent, candidatePresent, evidence, advanced,
                  ticketExpired, ticketRevoked, ticketUses,
                  requestFresh, requestRootMatches, requestGenerationMatches,
                  evaluatedCount, availableEpochs>>

BeginVerify ==
  /\ phase \in {"Submitted", "Unknown"}
  /\ phase' = "Verifying"
  /\ lastEvent' = "Verify"
  /\ UNCHANGED <<committedEpoch, committedGeneration,
                  oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  oldPresent, candidatePresent, evidence, advanced,
                  ticketExpired, ticketRevoked, ticketUses,
                  requestFresh, requestRootMatches, requestGenerationMatches,
                  evaluatedCount, availableEpochs>>

ConclusiveNew ==
  /\ phase = "Verifying"
  /\ evidence' = "NewOnly"
  /\ committedEpoch' = candidateEpoch
  /\ committedGeneration' = candidateGeneration
  /\ phase' = "Committed"
  /\ oldPresent' = FALSE
  /\ candidatePresent' = FALSE
  /\ advanced' = TRUE
  /\ lastEvent' = "AdapterEvidence"
  /\ UNCHANGED <<oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  ticketExpired, ticketRevoked, ticketUses,
                  requestFresh, requestRootMatches, requestGenerationMatches,
                  evaluatedCount, availableEpochs>>

Http200 ==
  /\ phase = "Submitted"
  /\ IF AllowHttpCommit
        THEN /\ phase' = "Committed"
             /\ committedEpoch' = candidateEpoch
             /\ committedGeneration' = candidateGeneration
             /\ oldPresent' = FALSE
             /\ candidatePresent' = FALSE
             /\ advanced' = TRUE
        ELSE /\ phase' = "Verifying"
             /\ UNCHANGED <<committedEpoch, committedGeneration,
                             oldPresent, candidatePresent, advanced>>
  /\ evidence' = "None"
  /\ lastEvent' = "HTTP200"
  /\ UNCHANGED <<oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  ticketExpired, ticketRevoked, ticketUses,
                  requestFresh, requestRootMatches, requestGenerationMatches,
                  evaluatedCount, availableEpochs>>

Timeout ==
  /\ phase \in {"Submitted", "Verifying"}
  /\ phase' = "Unknown"
  /\ evidence' = "Unavailable"
  /\ candidatePresent' = IF DropCandidateOnTimeout THEN FALSE ELSE candidatePresent
  /\ lastEvent' = "Timeout"
  /\ UNCHANGED <<committedEpoch, committedGeneration,
                  oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  oldPresent, advanced, ticketExpired, ticketRevoked, ticketUses,
                  requestFresh, requestRootMatches, requestGenerationMatches,
                  evaluatedCount, availableEpochs>>

ExpireTicket ==
  /\ ticketExpired = FALSE
  /\ ticketExpired' = TRUE
  /\ lastEvent' = "Expired"
  /\ UNCHANGED <<phase, committedEpoch, committedGeneration,
                  oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  oldPresent, candidatePresent, evidence, advanced,
                  ticketRevoked, ticketUses, requestFresh,
                  requestRootMatches, requestGenerationMatches,
                  evaluatedCount, availableEpochs>>

StaleFreshness ==
  /\ requestFresh = TRUE
  /\ requestFresh' = FALSE
  /\ lastEvent' = "StaleFreshness"
  /\ UNCHANGED <<phase, committedEpoch, committedGeneration,
                  oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  oldPresent, candidatePresent, evidence, advanced,
                  ticketExpired, ticketRevoked, ticketUses,
                  requestRootMatches, requestGenerationMatches,
                  evaluatedCount, availableEpochs>>

WrongRoot ==
  /\ requestRootMatches = TRUE
  /\ requestRootMatches' = FALSE
  /\ lastEvent' = "WrongRoot"
  /\ UNCHANGED <<phase, committedEpoch, committedGeneration,
                  oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  oldPresent, candidatePresent, evidence, advanced,
                  ticketExpired, ticketRevoked, ticketUses, requestFresh,
                  requestGenerationMatches, evaluatedCount, availableEpochs>>

WrongGeneration ==
  /\ requestGenerationMatches = TRUE
  /\ requestGenerationMatches' = FALSE
  /\ lastEvent' = "WrongGeneration"
  /\ UNCHANGED <<phase, committedEpoch, committedGeneration,
                  oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  oldPresent, candidatePresent, evidence, advanced,
                  ticketExpired, ticketRevoked, ticketUses, requestFresh,
                  requestRootMatches, evaluatedCount, availableEpochs>>

Evaluate ==
  /\ ~ticketRevoked
  /\ (DisableExpiry \/ ~ticketExpired)
  /\ (DisableFreshnessBinding \/ requestFresh)
  /\ (DisableRootBinding \/ requestRootMatches)
  /\ (DisableGenerationBinding \/ requestGenerationMatches)
  /\ (DisableReplayLimit \/ ticketUses = 0)
  /\ ticketUses' = ticketUses + 1
  /\ evaluatedCount' = evaluatedCount + 1
  /\ lastEvent' = "Evaluate"
  /\ UNCHANGED <<phase, committedEpoch, committedGeneration,
                  oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  oldPresent, candidatePresent, evidence, advanced,
                  ticketExpired, ticketRevoked, requestFresh,
                  requestRootMatches, requestGenerationMatches, availableEpochs>>

RetireOld ==
  /\ oldEpoch \in availableEpochs
  /\ (AllowUnsafeRetirement \/ ~(oldPresent \/ committedEpoch = oldEpoch))
  /\ availableEpochs' = availableEpochs \ {oldEpoch}
  /\ lastEvent' = "RetireOld"
  /\ UNCHANGED <<phase, committedEpoch, committedGeneration,
                  oldEpoch, oldGeneration, candidateEpoch, candidateGeneration,
                  oldPresent, candidatePresent, evidence, advanced,
                  ticketExpired, ticketRevoked, ticketUses, requestFresh,
                  requestRootMatches, requestGenerationMatches, evaluatedCount>>

Next ==
  \/ Prepare
  \/ Submit
  \/ BeginVerify
  \/ ConclusiveNew
  \/ Http200
  \/ Timeout
  \/ ExpireTicket
  \/ StaleFreshness
  \/ WrongRoot
  \/ WrongGeneration
  \/ Evaluate
  \/ RetireOld

Spec == Init /\ [][Next]_vars

TypeOK ==
  /\ phase \in Phases
  /\ committedEpoch \in Epochs
  /\ oldEpoch \in Epochs
  /\ candidateEpoch \in Epochs
  /\ evidence \in Evidence
  /\ availableEpochs \subseteq Epochs

CommitRequiresEvidence == advanced => evidence = "NewOnly"
UnknownPreservesBoth == phase = "Unknown" => oldPresent /\ candidatePresent
ReferencedEpochsAvailable ==
  /\ committedEpoch \in availableEpochs
  /\ (oldPresent => oldEpoch \in availableEpochs)
  /\ (candidatePresent => candidateEpoch \in availableEpochs)
EvaluatedNotExpired == lastEvent = "Evaluate" => ~ticketExpired
EvaluatedFresh == lastEvent = "Evaluate" => requestFresh
EvaluatedRootBound == lastEvent = "Evaluate" => requestRootMatches
EvaluatedGenerationBound == lastEvent = "Evaluate" => requestGenerationMatches
ReplayLimited == evaluatedCount <= 1

=============================================================================
