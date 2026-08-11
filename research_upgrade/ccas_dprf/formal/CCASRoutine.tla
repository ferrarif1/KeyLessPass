-------------------------- MODULE CCASRoutine --------------------------
EXTENDS Naturals, FiniteSets

\* A bounded abstract model. Cryptographic computation is represented only by
\* the capabilities it releases; no DPRF algebra is modeled here.

Domains == {"D", "U", "A", "N"}
Contexts == {"credential-A", "credential-B"}

ShareD == "S_D"
ShareU == "S_U"
ShareN == "S_N"
ClientToken == "ClientToken"
Request == "NetworkRequest"
Approval == "Approval_A"
RootCapability == "K_root_capability"

InitialCaps(d) ==
    CASE d = "D" -> {ShareD, ClientToken}
      [] d = "U" -> {ShareU}
      [] d = "A" -> {Approval}
      [] d = "N" -> {ShareN}
      [] OTHER -> {}

VARIABLES compromised,
          caps,
          phase,
          authorized,
          authorizedHistory,
          partialD,
          partialU,
          combined,
          derived,
          credentialKnowledge,
          routineRootMaterialized,
          recoveryRootMaterialized,
          rawUAtEndpoint

vars == <<compromised, caps, phase, authorized, authorizedHistory,
          partialD, partialU, combined, derived, credentialKnowledge,
          routineRootMaterialized, recoveryRootMaterialized, rawUAtEndpoint>>

Init ==
    /\ compromised = {}
    /\ caps = {}
    /\ phase = "Idle"
    /\ authorized = {}
    /\ authorizedHistory = {}
    /\ partialD = {}
    /\ partialU = {}
    /\ combined = {}
    /\ derived = {}
    /\ credentialKnowledge = {}
    /\ routineRootMaterialized = FALSE
    /\ recoveryRootMaterialized = FALSE
    /\ rawUAtEndpoint = FALSE

Compromise(d) ==
    /\ d \in Domains \ compromised
    /\ compromised' = compromised \cup {d}
    /\ caps' = caps \cup InitialCaps(d)
    /\ UNCHANGED <<phase, authorized, authorizedHistory, partialD, partialU,
                    combined, derived, credentialKnowledge,
                    routineRootMaterialized, recoveryRootMaterialized,
                    rawUAtEndpoint>>

CloseRequest ==
    /\ ClientToken \in caps
    /\ Request \notin caps
    /\ caps' = caps \cup {Request}
    /\ UNCHANGED <<compromised, phase, authorized, authorizedHistory,
                    partialD, partialU, combined, derived,
                    credentialKnowledge, routineRootMaterialized,
                    recoveryRootMaterialized, rawUAtEndpoint>>

CloseNetworkRelease ==
    /\ {Request, Approval} \subseteq caps
    /\ ShareN \notin caps
    /\ caps' = caps \cup {ShareN}
    /\ UNCHANGED <<compromised, phase, authorized, authorizedHistory,
                    partialD, partialU, combined, derived,
                    credentialKnowledge, routineRootMaterialized,
                    recoveryRootMaterialized, rawUAtEndpoint>>

CloseRootCapability ==
    /\ RootCapability \notin caps
    /\ \/ {ShareD, ShareU} \subseteq caps
       \/ {ShareD, ShareN} \subseteq caps
       \/ {ShareU, ShareN} \subseteq caps
    /\ caps' = caps \cup {RootCapability}
    /\ UNCHANGED <<compromised, phase, authorized, authorizedHistory,
                    partialD, partialU, combined, derived,
                    credentialKnowledge, routineRootMaterialized,
                    recoveryRootMaterialized, rawUAtEndpoint>>

BeginRoutine(c) ==
    /\ phase = "Idle"
    /\ c \in Contexts
    /\ phase' = "Routine"
    /\ authorized' = {c}
    /\ authorizedHistory' = authorizedHistory \cup {c}
    /\ partialD' = {}
    /\ partialU' = {}
    /\ combined' = {}
    /\ derived' = {}
    /\ routineRootMaterialized' = FALSE
    /\ recoveryRootMaterialized' = FALSE
    /\ rawUAtEndpoint' = FALSE
    /\ UNCHANGED <<compromised, caps, credentialKnowledge>>

EvalD(c) ==
    /\ phase = "Routine"
    /\ c \in authorized
    /\ partialD' = partialD \cup {c}
    /\ UNCHANGED <<compromised, caps, phase, authorized, authorizedHistory,
                    partialU, combined, derived, credentialKnowledge,
                    routineRootMaterialized, recoveryRootMaterialized,
                    rawUAtEndpoint>>

EvalU(c) ==
    /\ phase = "Routine"
    /\ c \in authorized
    /\ partialU' = partialU \cup {c}
    /\ rawUAtEndpoint' = FALSE
    /\ UNCHANGED <<compromised, caps, phase, authorized, authorizedHistory,
                    partialD, combined, derived, credentialKnowledge,
                    routineRootMaterialized, recoveryRootMaterialized>>

Combine(c) ==
    /\ phase = "Routine"
    /\ c \in partialD \cap partialU
    /\ combined' = combined \cup {c}
    /\ UNCHANGED <<compromised, caps, phase, authorized, authorizedHistory,
                    partialD, partialU, derived, credentialKnowledge,
                    routineRootMaterialized, recoveryRootMaterialized,
                    rawUAtEndpoint>>

Derive(c) ==
    /\ phase = "Routine"
    /\ c \in combined \cap authorized
    /\ derived' = derived \cup {c}
    /\ credentialKnowledge' = credentialKnowledge \cup {c}
    /\ UNCHANGED <<compromised, caps, phase, authorized, authorizedHistory,
                    partialD, partialU, combined, routineRootMaterialized,
                    recoveryRootMaterialized, rawUAtEndpoint>>

EndRoutine ==
    /\ phase = "Routine"
    /\ phase' = "Idle"
    /\ authorized' = {}
    /\ partialD' = {}
    /\ partialU' = {}
    /\ combined' = {}
    /\ derived' = {}
    /\ routineRootMaterialized' = FALSE
    /\ rawUAtEndpoint' = FALSE
    /\ UNCHANGED <<compromised, caps, authorizedHistory,
                    credentialKnowledge, recoveryRootMaterialized>>

BeginRecovery ==
    /\ phase = "Idle"
    /\ phase' = "Recovery"
    /\ recoveryRootMaterialized' = TRUE
    /\ UNCHANGED <<compromised, caps, authorized, authorizedHistory,
                    partialD, partialU, combined, derived,
                    credentialKnowledge, routineRootMaterialized,
                    rawUAtEndpoint>>

EndRecovery ==
    /\ phase = "Recovery"
    /\ phase' = "Idle"
    /\ recoveryRootMaterialized' = FALSE
    /\ UNCHANGED <<compromised, caps, authorized, authorizedHistory,
                    partialD, partialU, combined, derived,
                    credentialKnowledge, routineRootMaterialized,
                    rawUAtEndpoint>>

Next ==
    \/ \E d \in Domains : Compromise(d)
    \/ CloseRequest
    \/ CloseNetworkRelease
    \/ CloseRootCapability
    \/ \E c \in Contexts : BeginRoutine(c)
    \/ \E c \in Contexts : EvalD(c)
    \/ \E c \in Contexts : EvalU(c)
    \/ \E c \in Contexts : Combine(c)
    \/ \E c \in Contexts : Derive(c)
    \/ EndRoutine
    \/ BeginRecovery
    \/ EndRecovery

TypeOK ==
    /\ compromised \subseteq Domains
    /\ phase \in {"Idle", "Routine", "Recovery"}
    /\ authorized \subseteq Contexts
    /\ authorizedHistory \subseteq Contexts
    /\ partialD \subseteq Contexts
    /\ partialU \subseteq Contexts
    /\ combined \subseteq Contexts
    /\ derived \subseteq Contexts
    /\ credentialKnowledge \subseteq Contexts
    /\ routineRootMaterialized \in BOOLEAN
    /\ recoveryRootMaterialized \in BOOLEAN
    /\ rawUAtEndpoint \in BOOLEAN

NoRoutineRootMaterialization == ~routineRootMaterialized
NoCrossCredentialDerivationFromSingleAuthorizedEvaluation == derived \subseteq authorized
NoSingleDomainMasterCapability == Cardinality(compromised) < 2 => RootCapability \notin caps
EffectiveThresholdNotBelowConfiguredThreshold == RootCapability \in caps => Cardinality(compromised) >= 2
UnauthorizedContextCannotCompleteEvaluation == combined \subseteq authorized
RawShareNeverLeavesTokenDomain == ~rawUAtEndpoint
CurrentCredentialExposureDoesNotAuthorizeOtherContexts == credentialKnowledge \subseteq authorizedHistory
RecoveryIsExplicitlySeparate == recoveryRootMaterialized => phase = "Recovery"

Spec == Init /\ [][Next]_vars

=============================================================================
