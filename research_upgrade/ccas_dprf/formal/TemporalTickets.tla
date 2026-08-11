------------------------- MODULE TemporalTickets -------------------------
EXTENDS Naturals

CONSTANTS ScopeMode, EnforceExpiry, EnforceRevocation,
          EnforceGeneration, EnforceSingleUse, MaxTime, MaxGeneration

Contexts == {"credential-A", "credential-B"}

VARIABLES now, generation, ticketState, ticketContext, ticketGeneration,
          ticketExpiry, useCount, requested, partialD, partialU, udc, exposed,
          expiredAccepted, revokedAccepted, staleAccepted, replayAccepted

vars == <<now, generation, ticketState, ticketContext, ticketGeneration,
          ticketExpiry, useCount, requested, partialD, partialU, udc, exposed,
          expiredAccepted, revokedAccepted, staleAccepted, replayAccepted>>

Init ==
    /\ now = 0
    /\ generation = 0
    /\ ticketState = "Absent"
    /\ ticketContext = "None"
    /\ ticketGeneration = 0
    /\ ticketExpiry = 0
    /\ useCount = 0
    /\ requested = {}
    /\ partialD = {}
    /\ partialU = {}
    /\ udc = FALSE
    /\ exposed = {}
    /\ expiredAccepted = FALSE
    /\ revokedAccepted = FALSE
    /\ staleAccepted = FALSE
    /\ replayAccepted = FALSE

Issue(c) ==
    /\ ticketState = "Absent"
    /\ c \in Contexts
    /\ now < MaxTime
    /\ ticketState' = "Live"
    /\ ticketContext' = c
    /\ ticketGeneration' = generation
    /\ ticketExpiry' = now + 1
    /\ useCount' = 0
    /\ UNCHANGED <<now, generation, requested, partialD, partialU, udc,
                    exposed, expiredAccepted, revokedAccepted, staleAccepted,
                    replayAccepted>>

Tick ==
    /\ now < MaxTime
    /\ now' = now + 1
    /\ UNCHANGED <<generation, ticketState, ticketContext, ticketGeneration,
                    ticketExpiry, useCount, requested, partialD, partialU,
                    udc, exposed, expiredAccepted, revokedAccepted,
                    staleAccepted, replayAccepted>>

AdvanceGeneration ==
    /\ generation < MaxGeneration
    /\ generation' = generation + 1
    /\ UNCHANGED <<now, ticketState, ticketContext, ticketGeneration,
                    ticketExpiry, useCount, requested, partialD, partialU,
                    udc, exposed, expiredAccepted, revokedAccepted,
                    staleAccepted, replayAccepted>>

Revoke ==
    /\ ticketState = "Live"
    /\ ticketState' = "Revoked"
    /\ UNCHANGED <<now, generation, ticketContext, ticketGeneration,
                    ticketExpiry, useCount, requested, partialD, partialU,
                    udc, exposed, expiredAccepted, revokedAccepted,
                    staleAccepted, replayAccepted>>

Request(c) ==
    /\ c \in Contexts
    /\ requested' = requested \cup {c}
    /\ partialD' = partialD \cup {c}
    /\ UNCHANGED <<now, generation, ticketState, ticketContext,
                    ticketGeneration, ticketExpiry, useCount, partialU, udc,
                    exposed, expiredAccepted, revokedAccepted, staleAccepted,
                    replayAccepted>>

ScopeAccepts(c) == ScopeMode = "Projected" \/ c = ticketContext

Evaluate(c) ==
    /\ c \in requested
    /\ ticketState # "Absent"
    /\ ScopeAccepts(c)
    /\ (ticketState = "Live" \/ ~EnforceRevocation)
    /\ (now < ticketExpiry \/ ~EnforceExpiry)
    /\ (ticketGeneration = generation \/ ~EnforceGeneration)
    /\ (useCount = 0 \/ ~EnforceSingleUse)
    /\ useCount < 2
    /\ partialU' = partialU \cup {c}
    /\ useCount' = useCount + 1
    /\ expiredAccepted' = (expiredAccepted \/ now >= ticketExpiry)
    /\ revokedAccepted' = (revokedAccepted \/ ticketState = "Revoked")
    /\ staleAccepted' = (staleAccepted \/ ticketGeneration # generation)
    /\ replayAccepted' = (replayAccepted \/ useCount > 0)
    /\ UNCHANGED <<now, generation, ticketState, ticketContext,
                    ticketGeneration, ticketExpiry, requested, partialD, udc,
                    exposed>>

Combine(c) ==
    /\ c \in partialD \cap partialU
    /\ exposed' = exposed \cup {c}
    /\ UNCHANGED <<now, generation, ticketState, ticketContext,
                    ticketGeneration, ticketExpiry, useCount, requested,
                    partialD, partialU, udc, expiredAccepted, revokedAccepted,
                    staleAccepted, replayAccepted>>

AcquireUDC ==
    /\ ~udc
    /\ udc' = TRUE
    /\ exposed' = Contexts
    /\ UNCHANGED <<now, generation, ticketState, ticketContext,
                    ticketGeneration, ticketExpiry, useCount, requested,
                    partialD, partialU, expiredAccepted, revokedAccepted,
                    staleAccepted, replayAccepted>>

Next ==
    \/ \E c \in Contexts : Issue(c)
    \/ Tick
    \/ AdvanceGeneration
    \/ Revoke
    \/ \E c \in Contexts : Request(c)
    \/ \E c \in Contexts : Evaluate(c)
    \/ \E c \in Contexts : Combine(c)
    \/ AcquireUDC

TypeOK ==
    /\ ScopeMode \in {"Exact", "Projected"}
    /\ now \in 0..MaxTime
    /\ generation \in 0..MaxGeneration
    /\ ticketState \in {"Absent", "Live", "Revoked"}
    /\ ticketContext \in Contexts \cup {"None"}
    /\ ticketGeneration \in 0..MaxGeneration
    /\ ticketExpiry \in 0..MaxTime
    /\ useCount \in 0..2
    /\ requested \subseteq Contexts
    /\ partialD \subseteq Contexts
    /\ partialU \subseteq Contexts
    /\ udc \in BOOLEAN
    /\ exposed \subseteq Contexts
    /\ expiredAccepted \in BOOLEAN
    /\ revokedAccepted \in BOOLEAN
    /\ staleAccepted \in BOOLEAN
    /\ replayAccepted \in BOOLEAN

UDCDominance == udc => exposed = Contexts
NoScopeAmplification == (~udc /\ ticketContext \in Contexts) => exposed \subseteq {ticketContext}
NoExpiredAcceptance == ~expiredAccepted
NoRevokedAcceptance == ~revokedAccepted
NoStaleGenerationAcceptance == ~staleAccepted
NoReplayAcceptance == ~replayAccepted

Spec == Init /\ [][Next]_vars

=============================================================================
