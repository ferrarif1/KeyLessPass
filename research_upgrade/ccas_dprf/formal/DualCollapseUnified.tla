---------------------- MODULE DualCollapseUnified ----------------------
EXTENDS Naturals

CONSTANT Mode

Contexts == {"credential-A", "credential-B"}

VARIABLES phase, approved, requested, partialD, partialU, root, exposed

vars == <<phase, approved, requested, partialD, partialU, root, exposed>>

Init ==
    /\ phase = "Idle"
    /\ approved = {}
    /\ requested = {}
    /\ partialD = {}
    /\ partialU = {}
    /\ root = FALSE
    /\ exposed = {}

Begin ==
    /\ phase = "Idle"
    /\ phase' = "Routine"
    /\ approved' = {"credential-A"}
    /\ UNCHANGED <<requested, partialD, partialU, root, exposed>>

Request(c) ==
    /\ phase = "Routine"
    /\ c \in Contexts
    /\ requested' = requested \cup {c}
    /\ partialD' = partialD \cup {c}
    /\ UNCHANGED <<phase, approved, partialU, root, exposed>>

TokenEval(c) ==
    /\ phase = "Routine"
    /\ c \in requested
    /\ (Mode = "Wildcard" \/ c \in approved)
    /\ partialU' = partialU \cup {c}
    /\ UNCHANGED <<phase, approved, requested, partialD, root, exposed>>

Combine(c) ==
    /\ phase = "Routine"
    /\ c \in partialD \cap partialU
    /\ exposed' = exposed \cup {c}
    /\ UNCHANGED <<phase, approved, requested, partialD, partialU, root>>

AcquireRoot ==
    /\ phase = "Routine"
    /\ Mode = "Root"
    /\ root' = TRUE
    /\ exposed' = Contexts
    /\ UNCHANGED <<phase, approved, requested, partialD, partialU>>

Next ==
    \/ Begin
    \/ \E c \in Contexts : Request(c)
    \/ \E c \in Contexts : TokenEval(c)
    \/ \E c \in Contexts : Combine(c)
    \/ AcquireRoot

TypeOK ==
    /\ Mode \in {"Exact", "Wildcard", "Root"}
    /\ phase \in {"Idle", "Routine"}
    /\ approved \subseteq Contexts
    /\ requested \subseteq Contexts
    /\ partialD \subseteq Contexts
    /\ partialU \subseteq Contexts
    /\ root \in BOOLEAN
    /\ exposed \subseteq Contexts

RootDominance == root => exposed = Contexts
NoAuthorizationAmplification == exposed \subseteq approved
ExactScopeBelowRoot == (~root /\ Mode = "Exact") => exposed \subseteq approved

Spec == Init /\ [][Next]_vars

=============================================================================
