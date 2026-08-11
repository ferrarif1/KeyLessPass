------------------------- MODULE integrated_model -------------------------
EXTENDS Naturals, TLC

CONSTANTS MaxCredentialGeneration, MaxPolicyEpoch, MaxRootGeneration,
          MaxShareSetGeneration, MaxCheckpoint

VARIABLES
    localCredentialGeneration,
    localPolicyEpoch,
    localRootGeneration,
    localShareSetGeneration,
    localLineage,
    anchorCredentialGeneration,
    anchorPolicyEpoch,
    anchorRootGeneration,
    anchorShareSetGeneration,
    anchorLineage,
    checkpoint,
    rollbackAttempt,
    readAllowed,
    credentialKeyCompromised,
    rootCompromised,
    rootCompromiseGeneration,
    reshareCount,
    lastReshareRootCompromisedBefore,
    lastReshareRootCompromisedAfter,
    rootRepairCount,
    lastRepairRootBefore,
    lastRepairRootAfter

vars == <<localCredentialGeneration, localPolicyEpoch, localRootGeneration,
          localShareSetGeneration, localLineage, anchorCredentialGeneration,
          anchorPolicyEpoch, anchorRootGeneration, anchorShareSetGeneration,
          anchorLineage, checkpoint, rollbackAttempt, readAllowed,
          credentialKeyCompromised, rootCompromised,
          rootCompromiseGeneration, reshareCount,
          lastReshareRootCompromisedBefore,
          lastReshareRootCompromisedAfter, rootRepairCount,
          lastRepairRootBefore, lastRepairRootAfter>>

Init ==
    /\ localCredentialGeneration = 0 /\ localPolicyEpoch = 0
    /\ localRootGeneration = 1 /\ localShareSetGeneration = 1
    /\ localLineage = 0
    /\ anchorCredentialGeneration = 0 /\ anchorPolicyEpoch = 0
    /\ anchorRootGeneration = 1 /\ anchorShareSetGeneration = 1
    /\ anchorLineage = 0 /\ checkpoint = 0
    /\ rollbackAttempt = FALSE /\ readAllowed = TRUE
    /\ credentialKeyCompromised = FALSE /\ rootCompromised = FALSE
    /\ rootCompromiseGeneration = 1
    /\ reshareCount = 0
    /\ lastReshareRootCompromisedBefore = FALSE
    /\ lastReshareRootCompromisedAfter = FALSE
    /\ rootRepairCount = 0
    /\ lastRepairRootBefore = 1 /\ lastRepairRootAfter = 1

SamePolicyCommit ==
    /\ localCredentialGeneration < MaxCredentialGeneration
    /\ localCredentialGeneration' = localCredentialGeneration + 1
    /\ readAllowed' = FALSE
    /\ rollbackAttempt' = FALSE
    /\ UNCHANGED <<localPolicyEpoch, localRootGeneration,
                    localShareSetGeneration, localLineage,
                    anchorCredentialGeneration, anchorPolicyEpoch,
                    anchorRootGeneration, anchorShareSetGeneration,
                    anchorLineage, checkpoint, credentialKeyCompromised,
                    rootCompromised, rootCompromiseGeneration, reshareCount,
                    lastReshareRootCompromisedBefore,
                    lastReshareRootCompromisedAfter, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

PolicyChange ==
    /\ localPolicyEpoch < MaxPolicyEpoch
    /\ localPolicyEpoch' = localPolicyEpoch + 1
    /\ localCredentialGeneration' = 0
    /\ readAllowed' = FALSE
    /\ rollbackAttempt' = FALSE
    /\ UNCHANGED <<localRootGeneration, localShareSetGeneration,
                    localLineage, anchorCredentialGeneration,
                    anchorPolicyEpoch, anchorRootGeneration,
                    anchorShareSetGeneration, anchorLineage, checkpoint,
                    credentialKeyCompromised, rootCompromised,
                    rootCompromiseGeneration, reshareCount,
                    lastReshareRootCompromisedBefore,
                    lastReshareRootCompromisedAfter, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

CredentialRekey ==
    /\ localLineage = 0
    /\ localLineage' = 1
    /\ credentialKeyCompromised' = FALSE
    /\ readAllowed' = FALSE
    /\ rollbackAttempt' = FALSE
    /\ UNCHANGED <<localCredentialGeneration, localPolicyEpoch,
                    localRootGeneration, localShareSetGeneration,
                    anchorCredentialGeneration, anchorPolicyEpoch,
                    anchorRootGeneration, anchorShareSetGeneration,
                    anchorLineage, checkpoint, rootCompromised,
                    rootCompromiseGeneration, reshareCount,
                    lastReshareRootCompromisedBefore,
                    lastReshareRootCompromisedAfter, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

PublishFreshness ==
    /\ checkpoint < MaxCheckpoint
    /\ anchorCredentialGeneration' = localCredentialGeneration
    /\ anchorPolicyEpoch' = localPolicyEpoch
    /\ anchorRootGeneration' = localRootGeneration
    /\ anchorShareSetGeneration' = localShareSetGeneration
    /\ anchorLineage' = localLineage
    /\ checkpoint' = checkpoint + 1
    /\ rollbackAttempt' = FALSE
    /\ readAllowed' = TRUE
    /\ UNCHANGED <<localCredentialGeneration, localPolicyEpoch,
                    localRootGeneration, localShareSetGeneration,
                    localLineage, credentialKeyCompromised, rootCompromised,
                    rootCompromiseGeneration, reshareCount,
                    lastReshareRootCompromisedBefore,
                    lastReshareRootCompromisedAfter, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

RestoreCredentialSnapshot(epoch, generation) ==
    /\ epoch \in 0..MaxPolicyEpoch
    /\ generation \in 0..MaxCredentialGeneration
    /\ (epoch < anchorPolicyEpoch \/
        (epoch = anchorPolicyEpoch /\ generation < anchorCredentialGeneration))
    /\ localPolicyEpoch' = epoch
    /\ localCredentialGeneration' = generation
    /\ rollbackAttempt' = TRUE
    /\ readAllowed' = FALSE
    /\ UNCHANGED <<localRootGeneration, localShareSetGeneration, localLineage,
                    anchorCredentialGeneration, anchorPolicyEpoch,
                    anchorRootGeneration, anchorShareSetGeneration,
                    anchorLineage, checkpoint, credentialKeyCompromised,
                    rootCompromised, rootCompromiseGeneration, reshareCount,
                    lastReshareRootCompromisedBefore,
                    lastReshareRootCompromisedAfter, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

CheckFreshness ==
    /\ readAllowed' =
        (localRootGeneration >= anchorRootGeneration /\
         localShareSetGeneration >= anchorShareSetGeneration /\
         localPolicyEpoch >= anchorPolicyEpoch /\
         (localPolicyEpoch > anchorPolicyEpoch \/
          localCredentialGeneration >= anchorCredentialGeneration) /\
         localLineage = anchorLineage)
    /\ UNCHANGED <<localCredentialGeneration, localPolicyEpoch,
                    localRootGeneration, localShareSetGeneration,
                    localLineage, anchorCredentialGeneration,
                    anchorPolicyEpoch, anchorRootGeneration,
                    anchorShareSetGeneration, anchorLineage, checkpoint,
                    rollbackAttempt, credentialKeyCompromised,
                    rootCompromised, rootCompromiseGeneration, reshareCount,
                    lastReshareRootCompromisedBefore,
                    lastReshareRootCompromisedAfter, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

CompromiseCredentialKey ==
    /\ ~credentialKeyCompromised
    /\ credentialKeyCompromised' = TRUE
    /\ UNCHANGED <<localCredentialGeneration, localPolicyEpoch,
                    localRootGeneration, localShareSetGeneration,
                    localLineage, anchorCredentialGeneration,
                    anchorPolicyEpoch, anchorRootGeneration,
                    anchorShareSetGeneration, anchorLineage, checkpoint,
                    rollbackAttempt, readAllowed, rootCompromised,
                    rootCompromiseGeneration, reshareCount,
                    lastReshareRootCompromisedBefore,
                    lastReshareRootCompromisedAfter, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

CompromiseRoot ==
    /\ ~rootCompromised
    /\ rootCompromised' = TRUE
    /\ rootCompromiseGeneration' = localRootGeneration
    /\ UNCHANGED <<localCredentialGeneration, localPolicyEpoch,
                    localRootGeneration, localShareSetGeneration,
                    localLineage, anchorCredentialGeneration,
                    anchorPolicyEpoch, anchorRootGeneration,
                    anchorShareSetGeneration, anchorLineage, checkpoint,
                    rollbackAttempt, readAllowed, credentialKeyCompromised,
                    reshareCount, lastReshareRootCompromisedBefore,
                    lastReshareRootCompromisedAfter, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

ReshareRoot ==
    /\ localShareSetGeneration < MaxShareSetGeneration
    /\ localShareSetGeneration' = localShareSetGeneration + 1
    /\ reshareCount' = reshareCount + 1
    /\ lastReshareRootCompromisedBefore' = rootCompromised
    /\ lastReshareRootCompromisedAfter' = rootCompromised
    /\ readAllowed' = FALSE
    /\ UNCHANGED <<localCredentialGeneration, localPolicyEpoch,
                    localRootGeneration, localLineage,
                    anchorCredentialGeneration, anchorPolicyEpoch,
                    anchorRootGeneration, anchorShareSetGeneration,
                    anchorLineage, checkpoint, rollbackAttempt,
                    credentialKeyCompromised, rootCompromised,
                    rootCompromiseGeneration, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

ReplaceRoot ==
    /\ rootCompromised
    /\ localRootGeneration < MaxRootGeneration
    /\ localShareSetGeneration < MaxShareSetGeneration
    /\ lastRepairRootBefore' = localRootGeneration
    /\ localRootGeneration' = localRootGeneration + 1
    /\ lastRepairRootAfter' = localRootGeneration + 1
    /\ localShareSetGeneration' = localShareSetGeneration + 1
    /\ rootCompromised' = FALSE
    /\ credentialKeyCompromised' = FALSE
    /\ rootRepairCount' = rootRepairCount + 1
    /\ readAllowed' = FALSE
    /\ UNCHANGED <<localCredentialGeneration, localPolicyEpoch, localLineage,
                    anchorCredentialGeneration, anchorPolicyEpoch,
                    anchorRootGeneration, anchorShareSetGeneration,
                    anchorLineage, checkpoint, rollbackAttempt,
                    rootCompromiseGeneration, reshareCount,
                    lastReshareRootCompromisedBefore,
                    lastReshareRootCompromisedAfter>>

Next ==
    \/ SamePolicyCommit
    \/ PolicyChange
    \/ CredentialRekey
    \/ PublishFreshness
    \/ \E epoch \in 0..MaxPolicyEpoch,
             generation \in 0..MaxCredentialGeneration :
          RestoreCredentialSnapshot(epoch, generation)
    \/ CheckFreshness
    \/ CompromiseCredentialKey
    \/ CompromiseRoot
    \/ ReshareRoot
    \/ ReplaceRoot

Spec == Init /\ [][Next]_vars

TypeInvariant ==
    /\ localCredentialGeneration \in 0..MaxCredentialGeneration
    /\ anchorCredentialGeneration \in 0..MaxCredentialGeneration
    /\ localPolicyEpoch \in 0..MaxPolicyEpoch
    /\ anchorPolicyEpoch \in 0..MaxPolicyEpoch
    /\ localRootGeneration \in 1..MaxRootGeneration
    /\ anchorRootGeneration \in 1..MaxRootGeneration
    /\ localShareSetGeneration \in 1..MaxShareSetGeneration
    /\ anchorShareSetGeneration \in 1..MaxShareSetGeneration
    /\ localLineage \in {0, 1} /\ anchorLineage \in {0, 1}
    /\ checkpoint \in 0..MaxCheckpoint
    /\ rollbackAttempt \in BOOLEAN /\ readAllowed \in BOOLEAN
    /\ credentialKeyCompromised \in BOOLEAN /\ rootCompromised \in BOOLEAN

NoSilentGenerationRollback == rollbackAttempt => ~readAllowed

CommittedGenerationNotBelowFreshnessAnchor ==
    readAllowed =>
        /\ localRootGeneration >= anchorRootGeneration
        /\ localShareSetGeneration >= anchorShareSetGeneration
        /\ localPolicyEpoch >= anchorPolicyEpoch
        /\ (localPolicyEpoch > anchorPolicyEpoch \/
            localCredentialGeneration >= anchorCredentialGeneration)
        /\ localLineage = anchorLineage

ShareSetRotationDoesNotRepairCompromisedRoot ==
    lastReshareRootCompromisedBefore => lastReshareRootCompromisedAfter

RootCompromiseRequiresRootGenerationAdvance ==
    rootRepairCount > 0 => lastRepairRootAfter = lastRepairRootBefore + 1

==========================================================================
