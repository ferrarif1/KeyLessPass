-------------------- MODULE FactorPreservingRecovery --------------------
EXTENDS Naturals, FiniteSets, TLC

CONSTANT Nodes, Approvers, StorageNodes

VARIABLES device, usb, fragments, approvals, responses,
          currentEpoch, requestedEpoch, networkReleased,
          rootRecovered, refreshed

vars == <<device, usb, fragments, approvals, responses,
          currentEpoch, requestedEpoch, networkReleased,
          rootRecovered, refreshed>>

CapabilityAvailable == device \/ usb
CurrentRequest == requestedEpoch = currentEpoch
StorageAvailable == Cardinality(fragments) >= 3
AuthorizationAvailable == Cardinality(approvals) >= 2
ThresholdReleaseAvailable == Cardinality(responses) >= 3
NetworkFactor == networkReleased
LogicalShareCount ==
    (IF device THEN 1 ELSE 0)
    + (IF usb THEN 1 ELSE 0)
    + (IF NetworkFactor THEN 1 ELSE 0)

Init ==
    /\ device = FALSE
    /\ usb = FALSE
    /\ fragments = {}
    /\ approvals = {}
    /\ responses = {}
    /\ currentEpoch = 1
    /\ requestedEpoch \in {0, 1}
    /\ networkReleased = FALSE
    /\ rootRecovered = FALSE
    /\ refreshed = FALSE

AcquireDevice ==
    /\ ~device
    /\ device' = TRUE
    /\ UNCHANGED <<usb, fragments, approvals, responses,
                    currentEpoch, requestedEpoch, networkReleased,
                    rootRecovered, refreshed>>

AcquireUsb ==
    /\ ~usb
    /\ usb' = TRUE
    /\ UNCHANGED <<device, fragments, approvals, responses,
                    currentEpoch, requestedEpoch, networkReleased,
                    rootRecovered, refreshed>>

AcquireFragment(fragment) ==
    /\ fragment \in StorageNodes \ fragments
    /\ fragments' = fragments \cup {fragment}
    /\ UNCHANGED <<device, usb, approvals, responses,
                    currentEpoch, requestedEpoch, networkReleased,
                    rootRecovered, refreshed>>

Approve(approver) ==
    /\ approver \in Approvers \ approvals
    /\ approvals' = approvals \cup {approver}
    /\ UNCHANGED <<device, usb, fragments, responses,
                    currentEpoch, requestedEpoch, networkReleased,
                    rootRecovered, refreshed>>

NodeResponse(node) ==
    /\ node \in Nodes \ responses
    /\ CapabilityAvailable
    /\ CurrentRequest
    /\ AuthorizationAvailable
    /\ responses' = responses \cup {node}
    /\ UNCHANGED <<device, usb, fragments, approvals,
                    currentEpoch, requestedEpoch, networkReleased,
                    rootRecovered, refreshed>>

ReleaseNetworkShare ==
    /\ ~networkReleased
    /\ CapabilityAvailable
    /\ CurrentRequest
    /\ StorageAvailable
    /\ AuthorizationAvailable
    /\ ThresholdReleaseAvailable
    /\ networkReleased' = TRUE
    /\ UNCHANGED <<device, usb, fragments, approvals, responses,
                    currentEpoch, requestedEpoch, rootRecovered, refreshed>>

RecoverRoot ==
    /\ ~rootRecovered
    /\ LogicalShareCount >= 2
    /\ rootRecovered' = TRUE
    /\ UNCHANGED <<device, usb, fragments, approvals, responses,
                    currentEpoch, requestedEpoch, networkReleased, refreshed>>

RefreshAfterRecovery ==
    /\ rootRecovered
    /\ networkReleased
    /\ ~refreshed
    /\ currentEpoch' = currentEpoch + 1
    /\ networkReleased' = FALSE
    /\ responses' = {}
    /\ approvals' = {}
    /\ fragments' = {}
    /\ rootRecovered' = FALSE
    /\ refreshed' = TRUE
    /\ UNCHANGED <<device, usb, requestedEpoch>>

Next ==
    AcquireDevice \/ AcquireUsb
    \/ \E fragment \in StorageNodes : AcquireFragment(fragment)
    \/ \E approver \in Approvers : Approve(approver)
    \/ \E node \in Nodes : NodeResponse(node)
    \/ ReleaseNetworkShare \/ RecoverRoot \/ RefreshAfterRecovery

TypeInvariant ==
    /\ device \in BOOLEAN
    /\ usb \in BOOLEAN
    /\ fragments \subseteq StorageNodes
    /\ approvals \subseteq Approvers
    /\ responses \subseteq Nodes
    /\ currentEpoch \in 1..2
    /\ requestedEpoch \in {0, 1}
    /\ networkReleased \in BOOLEAN
    /\ rootRecovered \in BOOLEAN
    /\ refreshed \in BOOLEAN

NoUnauthorizedNetworkRelease ==
    networkReleased =>
        /\ CapabilityAvailable
        /\ StorageAvailable
        /\ AuthorizationAvailable
        /\ ThresholdReleaseAvailable
        /\ CurrentRequest

NoSingleShareRecovery == rootRecovered => LogicalShareCount >= 2

NetworkAloneCannotRecover == (~device /\ ~usb) => ~rootRecovered

StaleEpochCannotRelease == (~CurrentRequest) => ~networkReleased

NoDuplicateNodeContribution == Cardinality(responses) <= Cardinality(Nodes)

Spec == Init /\ [][Next]_vars

=============================================================================
