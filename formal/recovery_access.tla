------------------------ MODULE recovery_access ------------------------
EXTENDS Naturals, FiniteSets, TLC

CONSTANTS MaxRootGeneration, MaxShareSetGeneration

Domains == {"D", "U", "N", "A"}
Approvers == {"A1", "A2", "A3"}
Nodes == {"N1", "N2", "N3", "N4", "N5"}
RecoveryKeys == {1, 2}
OperationIDs == {1, 2, 3}

VARIABLES
    compromisedDomains,
    ticketActive,
    ticketFresh,
    ticketRootGeneration,
    ticketShareSetGeneration,
    ticketPk,
    releasedPk,
    opID,
    usedOpIDs,
    approvalSet,
    nodeResponses,
    networkShareReleased,
    releaseAuthorized,
    rootGeneration,
    shareSetGeneration,
    recoveryCount,
    lastRecoveryOldShareSet,
    lastRecoveryNewShareSet,
    lastRecoveryRootBefore,
    lastRecoveryRootAfter,
    lastRecoveryWasCompromised,
    lastRecoveryLeftCompromised,
    rootCompromised,
    rootCompromiseGeneration,
    rootRepairCount,
    lastRepairRootBefore,
    lastRepairRootAfter

vars == <<compromisedDomains, ticketActive, ticketFresh,
          ticketRootGeneration, ticketShareSetGeneration, ticketPk,
          releasedPk, opID, usedOpIDs, approvalSet, nodeResponses,
          networkShareReleased, releaseAuthorized, rootGeneration,
          shareSetGeneration, recoveryCount, lastRecoveryOldShareSet,
          lastRecoveryNewShareSet, lastRecoveryRootBefore,
          lastRecoveryRootAfter, lastRecoveryWasCompromised,
          lastRecoveryLeftCompromised, rootCompromised,
          rootCompromiseGeneration, rootRepairCount,
          lastRepairRootBefore, lastRepairRootAfter>>

Init ==
    /\ compromisedDomains = {}
    /\ ticketActive = FALSE
    /\ ticketFresh = FALSE
    /\ ticketRootGeneration = 1
    /\ ticketShareSetGeneration = 1
    /\ ticketPk = 1
    /\ releasedPk = 1
    /\ opID = 1
    /\ usedOpIDs = {}
    /\ approvalSet = {}
    /\ nodeResponses = {}
    /\ networkShareReleased = FALSE
    /\ releaseAuthorized = FALSE
    /\ rootGeneration = 1
    /\ shareSetGeneration = 1
    /\ recoveryCount = 0
    /\ lastRecoveryOldShareSet = 1
    /\ lastRecoveryNewShareSet = 1
    /\ lastRecoveryRootBefore = 1
    /\ lastRecoveryRootAfter = 1
    /\ lastRecoveryWasCompromised = FALSE
    /\ lastRecoveryLeftCompromised = FALSE
    /\ rootCompromised = FALSE
    /\ rootCompromiseGeneration = 1
    /\ rootRepairCount = 0
    /\ lastRepairRootBefore = 1
    /\ lastRepairRootAfter = 1

Compromise(domain) ==
    /\ domain \in Domains
    /\ compromisedDomains' = compromisedDomains \cup {domain}
    /\ UNCHANGED <<ticketActive, ticketFresh, ticketRootGeneration,
                    ticketShareSetGeneration, ticketPk, releasedPk, opID,
                    usedOpIDs, approvalSet, nodeResponses,
                    networkShareReleased, releaseAuthorized, rootGeneration,
                    shareSetGeneration, recoveryCount,
                    lastRecoveryOldShareSet, lastRecoveryNewShareSet,
                    lastRecoveryRootBefore, lastRecoveryRootAfter,
                    lastRecoveryWasCompromised, lastRecoveryLeftCompromised,
                    rootCompromised, rootCompromiseGeneration,
                    rootRepairCount, lastRepairRootBefore,
                    lastRepairRootAfter>>

BeginRecovery(pk, operation) ==
    /\ ~ticketActive
    /\ pk \in RecoveryKeys
    /\ operation \in OperationIDs \ usedOpIDs
    /\ ticketActive' = TRUE
    /\ ticketFresh' = TRUE
    /\ ticketRootGeneration' = rootGeneration
    /\ ticketShareSetGeneration' = shareSetGeneration
    /\ ticketPk' = pk
    /\ releasedPk' = pk
    /\ opID' = operation
    /\ approvalSet' = {}
    /\ nodeResponses' = {}
    /\ networkShareReleased' = FALSE
    /\ releaseAuthorized' = FALSE
    /\ UNCHANGED <<compromisedDomains, usedOpIDs, rootGeneration,
                    shareSetGeneration, recoveryCount,
                    lastRecoveryOldShareSet, lastRecoveryNewShareSet,
                    lastRecoveryRootBefore, lastRecoveryRootAfter,
                    lastRecoveryWasCompromised, lastRecoveryLeftCompromised,
                    rootCompromised, rootCompromiseGeneration,
                    rootRepairCount, lastRepairRootBefore,
                    lastRepairRootAfter>>

Approve(approver) ==
    /\ ticketActive /\ ticketFresh
    /\ approver \in Approvers \ approvalSet
    /\ approvalSet' = approvalSet \cup {approver}
    /\ UNCHANGED <<compromisedDomains, ticketActive, ticketFresh,
                    ticketRootGeneration, ticketShareSetGeneration, ticketPk,
                    releasedPk, opID, usedOpIDs, nodeResponses,
                    networkShareReleased, releaseAuthorized, rootGeneration,
                    shareSetGeneration, recoveryCount,
                    lastRecoveryOldShareSet, lastRecoveryNewShareSet,
                    lastRecoveryRootBefore, lastRecoveryRootAfter,
                    lastRecoveryWasCompromised, lastRecoveryLeftCompromised,
                    rootCompromised, rootCompromiseGeneration,
                    rootRepairCount, lastRepairRootBefore,
                    lastRepairRootAfter>>

NodeRespond(node) ==
    /\ ticketActive /\ ticketFresh
    /\ Cardinality(approvalSet) >= 2
    /\ ticketRootGeneration = rootGeneration
    /\ ticketShareSetGeneration = shareSetGeneration
    /\ opID \notin usedOpIDs
    /\ node \in Nodes \ nodeResponses
    /\ nodeResponses' = nodeResponses \cup {node}
    /\ UNCHANGED <<compromisedDomains, ticketActive, ticketFresh,
                    ticketRootGeneration, ticketShareSetGeneration, ticketPk,
                    releasedPk, opID, usedOpIDs, approvalSet,
                    networkShareReleased, releaseAuthorized, rootGeneration,
                    shareSetGeneration, recoveryCount,
                    lastRecoveryOldShareSet, lastRecoveryNewShareSet,
                    lastRecoveryRootBefore, lastRecoveryRootAfter,
                    lastRecoveryWasCompromised, lastRecoveryLeftCompromised,
                    rootCompromised, rootCompromiseGeneration,
                    rootRepairCount, lastRepairRootBefore,
                    lastRepairRootAfter>>

ReleaseNetworkShare ==
    /\ ticketActive /\ ticketFresh
    /\ Cardinality(approvalSet) >= 2
    /\ Cardinality(nodeResponses) >= 3
    /\ ticketRootGeneration = rootGeneration
    /\ ticketShareSetGeneration = shareSetGeneration
    /\ opID \notin usedOpIDs
    /\ networkShareReleased' = TRUE
    /\ releaseAuthorized' = TRUE
    /\ releasedPk' = ticketPk
    /\ UNCHANGED <<compromisedDomains, ticketActive, ticketFresh,
                    ticketRootGeneration, ticketShareSetGeneration, ticketPk,
                    opID, usedOpIDs, approvalSet, nodeResponses,
                    rootGeneration, shareSetGeneration, recoveryCount,
                    lastRecoveryOldShareSet, lastRecoveryNewShareSet,
                    lastRecoveryRootBefore, lastRecoveryRootAfter,
                    lastRecoveryWasCompromised, lastRecoveryLeftCompromised,
                    rootCompromised, rootCompromiseGeneration,
                    rootRepairCount, lastRepairRootBefore,
                    lastRepairRootAfter>>

ExpireTicket ==
    /\ ticketActive /\ ticketFresh /\ ~networkShareReleased
    /\ ticketFresh' = FALSE
    /\ UNCHANGED <<compromisedDomains, ticketActive, ticketRootGeneration,
                    ticketShareSetGeneration, ticketPk, releasedPk, opID,
                    usedOpIDs, approvalSet, nodeResponses,
                    networkShareReleased, releaseAuthorized, rootGeneration,
                    shareSetGeneration, recoveryCount,
                    lastRecoveryOldShareSet, lastRecoveryNewShareSet,
                    lastRecoveryRootBefore, lastRecoveryRootAfter,
                    lastRecoveryWasCompromised, lastRecoveryLeftCompromised,
                    rootCompromised, rootCompromiseGeneration,
                    rootRepairCount, lastRepairRootBefore,
                    lastRepairRootAfter>>

OrdinaryRecovery ==
    /\ networkShareReleased /\ releaseAuthorized
    /\ shareSetGeneration < MaxShareSetGeneration
    /\ shareSetGeneration' = shareSetGeneration + 1
    /\ recoveryCount' = recoveryCount + 1
    /\ lastRecoveryOldShareSet' = shareSetGeneration
    /\ lastRecoveryNewShareSet' = shareSetGeneration + 1
    /\ lastRecoveryRootBefore' = rootGeneration
    /\ lastRecoveryRootAfter' = rootGeneration
    /\ lastRecoveryWasCompromised' = rootCompromised
    /\ lastRecoveryLeftCompromised' = rootCompromised
    /\ usedOpIDs' = usedOpIDs \cup {opID}
    /\ ticketActive' = FALSE
    /\ ticketFresh' = FALSE
    /\ approvalSet' = {}
    /\ nodeResponses' = {}
    /\ networkShareReleased' = FALSE
    /\ releaseAuthorized' = FALSE
    /\ UNCHANGED <<compromisedDomains, ticketRootGeneration,
                    ticketShareSetGeneration, ticketPk, releasedPk, opID,
                    rootGeneration, rootCompromised,
                    rootCompromiseGeneration, rootRepairCount,
                    lastRepairRootBefore, lastRepairRootAfter>>

CompromiseRoot ==
    /\ ~rootCompromised
    /\ rootCompromised' = TRUE
    /\ rootCompromiseGeneration' = rootGeneration
    /\ UNCHANGED <<compromisedDomains, ticketActive, ticketFresh,
                    ticketRootGeneration, ticketShareSetGeneration, ticketPk,
                    releasedPk, opID, usedOpIDs, approvalSet, nodeResponses,
                    networkShareReleased, releaseAuthorized, rootGeneration,
                    shareSetGeneration, recoveryCount,
                    lastRecoveryOldShareSet, lastRecoveryNewShareSet,
                    lastRecoveryRootBefore, lastRecoveryRootAfter,
                    lastRecoveryWasCompromised, lastRecoveryLeftCompromised,
                    rootRepairCount, lastRepairRootBefore,
                    lastRepairRootAfter>>

ReplaceCompromisedRoot ==
    /\ rootCompromised
    /\ rootGeneration < MaxRootGeneration
    /\ shareSetGeneration < MaxShareSetGeneration
    /\ lastRepairRootBefore' = rootGeneration
    /\ rootGeneration' = rootGeneration + 1
    /\ lastRepairRootAfter' = rootGeneration + 1
    /\ shareSetGeneration' = shareSetGeneration + 1
    /\ rootCompromised' = FALSE
    /\ rootRepairCount' = rootRepairCount + 1
    /\ ticketActive' = FALSE
    /\ ticketFresh' = FALSE
    /\ approvalSet' = {}
    /\ nodeResponses' = {}
    /\ networkShareReleased' = FALSE
    /\ releaseAuthorized' = FALSE
    /\ UNCHANGED <<compromisedDomains, ticketRootGeneration,
                    ticketShareSetGeneration, ticketPk, releasedPk, opID,
                    usedOpIDs, recoveryCount, lastRecoveryOldShareSet,
                    lastRecoveryNewShareSet, lastRecoveryRootBefore,
                    lastRecoveryRootAfter, lastRecoveryWasCompromised,
                    lastRecoveryLeftCompromised, rootCompromiseGeneration>>

Next ==
    \/ \E domain \in Domains : Compromise(domain)
    \/ \E pk \in RecoveryKeys, operation \in OperationIDs : BeginRecovery(pk, operation)
    \/ \E approver \in Approvers : Approve(approver)
    \/ \E node \in Nodes : NodeRespond(node)
    \/ ReleaseNetworkShare
    \/ ExpireTicket
    \/ OrdinaryRecovery
    \/ CompromiseRoot
    \/ ReplaceCompromisedRoot

Spec == Init /\ [][Next]_vars

AttackerShares ==
    (IF "D" \in compromisedDomains THEN {"SD"} ELSE {}) \cup
    (IF "U" \in compromisedDomains THEN {"SU"} ELSE {}) \cup
    (IF ("N" \in compromisedDomains) \/
        ({"D", "A"} \subseteq compromisedDomains)
     THEN {"SN"} ELSE {})

TypeInvariant ==
    /\ compromisedDomains \subseteq Domains
    /\ ticketActive \in BOOLEAN /\ ticketFresh \in BOOLEAN
    /\ ticketRootGeneration \in 1..MaxRootGeneration
    /\ ticketShareSetGeneration \in 1..MaxShareSetGeneration
    /\ ticketPk \in RecoveryKeys /\ releasedPk \in RecoveryKeys
    /\ opID \in OperationIDs /\ usedOpIDs \subseteq OperationIDs
    /\ approvalSet \subseteq Approvers /\ nodeResponses \subseteq Nodes
    /\ networkShareReleased \in BOOLEAN /\ releaseAuthorized \in BOOLEAN
    /\ rootGeneration \in 1..MaxRootGeneration
    /\ shareSetGeneration \in 1..MaxShareSetGeneration
    /\ rootCompromised \in BOOLEAN

NoNetworkShareWithoutAuthorization ==
    networkShareReleased => releaseAuthorized

TicketBoundToRecoverySession ==
    networkShareReleased =>
        /\ releasedPk = ticketPk
        /\ ticketRootGeneration = rootGeneration
        /\ ticketShareSetGeneration = shareSetGeneration

NoReuseOfExpiredTicket ==
    networkShareReleased => ticketFresh /\ opID \notin usedOpIDs

NoSingleDomainRootRecovery ==
    Cardinality(compromisedDomains) = 1 => Cardinality(AttackerShares) < 2

RecoveryIncrementsShareSetGeneration ==
    recoveryCount > 0 => lastRecoveryNewShareSet = lastRecoveryOldShareSet + 1

RootGenerationStableOnOrdinaryReshare ==
    recoveryCount > 0 => lastRecoveryRootAfter = lastRecoveryRootBefore

ShareSetRotationDoesNotRepairCompromisedRoot ==
    lastRecoveryWasCompromised => lastRecoveryLeftCompromised

RootCompromiseRequiresRootGenerationAdvance ==
    rootRepairCount > 0 => lastRepairRootAfter = lastRepairRootBefore + 1

==========================================================================
