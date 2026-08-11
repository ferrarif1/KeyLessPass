# Extended lifecycle and recovery model-checking record

Date: 2026-08-09
TLC: 2.19 (08 August 2024, revision `5a47802`)
Jar: `tmp/tla2tools-v1.7.4.jar`
SHA-256: `936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88`

The existing `lifecycle.tla` credential-rotation model and its recorded
1,006,128 distinct states are retained unchanged. This revision adds two
separate abstractions rather than replacing that result.

## Recovery access model

Files: `recovery_access.tla`, `recovery_access.cfg`

The model covers D/U/N/A compromise domains, two-of-three independent
approvals, three-of-five node responses, session public-key binding, ticket
freshness, one-time operation identifiers, share-set generation, ordinary
re-sharing, Root-Key compromise, and Root-Key replacement. Cryptographic
operations are atomic assumptions.

Bounds:

```text
MaxRootGeneration = 2
MaxShareSetGeneration = 3
3 approvers, 5 nodes, 2 recovery public keys, 3 operation IDs
```

Checked invariants:

- `NoNetworkShareWithoutAuthorization`
- `TicketBoundToRecoverySession`
- `NoReuseOfExpiredTicket`
- `NoSingleDomainRootRecovery`
- `RecoveryIncrementsShareSetGeneration`
- `RootGenerationStableOnOrdinaryReshare`
- `ShareSetRotationDoesNotRepairCompromisedRoot`
- `RootCompromiseRequiresRootGenerationAdvance`

TLC result:

```text
6,145,889 states generated
852,704 distinct states found
0 states left on queue
complete graph depth 32
no invariant violation reported
fingerprint seed -1575291533255633256
```

## Integrated freshness/compromise model

Files: `integrated_model.tla`, `integrated_model.cfg`

The model covers credential generation, policy epoch, credential-key lineage,
Root-Key generation, share-set generation, freshness publication, complete
credential snapshot rollback, credential-key compromise/rekey, ordinary
re-sharing, and Root-Key replacement.

Bounds:

```text
MaxCredentialGeneration = 2
MaxPolicyEpoch = 1
MaxRootGeneration = 2
MaxShareSetGeneration = 3
MaxCheckpoint = 3
```

Checked invariants:

- `NoSilentGenerationRollback`
- `CommittedGenerationNotBelowFreshnessAnchor`
- `ShareSetRotationDoesNotRepairCompromisedRoot`
- `RootCompromiseRequiresRootGenerationAdvance`

The rollback transition restores the complete
`(policyEpoch,credentialGeneration)` tuple and requires it to be
lexicographically older than the anchor.

Final TLC result:

```text
287,151 states generated
40,292 distinct states found
0 states left on queue
complete graph depth 16
no invariant violation reported
fingerprint seed 7517020762989663758
```

## Claim boundary

These are finite-state checks of the listed abstractions. They do not verify
Ed25519, X25519, AES-GCM, Shamir, HKDF, FF1, database durability, clock
correctness, administrator judgment, network delivery, Byzantine nodes, or
unbounded executions. The result supports only the stated transition guards and
invariants within the configured bounds.
