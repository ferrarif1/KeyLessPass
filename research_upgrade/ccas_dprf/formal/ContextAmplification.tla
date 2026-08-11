---------------------- MODULE ContextAmplification ----------------------
EXTENDS Naturals

CONSTANT TokenScope

Contexts == {"credential-A", "credential-B"}

VARIABLES phase, approved, requested, partialD, partialU, derived

vars == <<phase, approved, requested, partialD, partialU, derived>>

Init ==
    /\ phase = "Idle"
    /\ approved = {}
    /\ requested = {}
    /\ partialD = {}
    /\ partialU = {}
    /\ derived = {}

Begin ==
    /\ phase = "Idle"
    /\ phase' = "Routine"
    /\ approved' = {"credential-A"}
    /\ UNCHANGED <<requested, partialD, partialU, derived>>

Request(c) ==
    /\ phase = "Routine"
    /\ c \in Contexts
    /\ requested' = requested \cup {c}
    /\ partialD' = partialD \cup {c}
    /\ UNCHANGED <<phase, approved, partialU, derived>>

TokenEval(c) ==
    /\ phase = "Routine"
    /\ c \in requested
    /\ (TokenScope = "Wildcard" \/ c \in approved)
    /\ partialU' = partialU \cup {c}
    /\ UNCHANGED <<phase, approved, requested, partialD, derived>>

Combine(c) ==
    /\ phase = "Routine"
    /\ c \in partialD \cap partialU
    /\ derived' = derived \cup {c}
    /\ UNCHANGED <<phase, approved, requested, partialD, partialU>>

Next ==
    \/ Begin
    \/ \E c \in Contexts : Request(c)
    \/ \E c \in Contexts : TokenEval(c)
    \/ \E c \in Contexts : Combine(c)

TypeOK ==
    /\ phase \in {"Idle", "Routine"}
    /\ approved \subseteq Contexts
    /\ requested \subseteq Contexts
    /\ partialD \subseteq Contexts
    /\ partialU \subseteq Contexts
    /\ derived \subseteq Contexts

NoAuthorizationAmplification == derived \subseteq approved
RawTokenShareNeverLeaves == TRUE
RootNeverMaterializes == TRUE

Spec == Init /\ [][Next]_vars

=============================================================================
