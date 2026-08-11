# ASTER protocol specification v0.1

## 1. Research question

Can a legacy-password credential manager simultaneously provide:

- exact policy compliance;
- deterministic reconstruction;
- strict same-lineage non-repetition;
- bounded authorization blast radius;
- no endpoint Root-Key reconstruction during normal use;
- failure-safe remote rotation; and
- genuine recovery after Root-Key compromise?

The key distinction is between **share refresh** and **Root-Epoch replacement**.
Refreshing shares of the same Root Key may protect against a mobile adversary
that has not learned the root, but it does not heal a root that has already been
exposed. ASTER creates an independent new Root-Epoch key and migrates each
credential to it.

## 2. Entities

- **Client C**: endpoint that requests a credential.
- **Approval Authority A**: issues narrowly scoped, short-lived capabilities.
- **Evaluator domains E1..En**: independently controlled threshold/MPC parties.
- **Legacy target R**: accepts only textual passwords and exposes some
  password-change/verification interface.
- **Freshness service F (optional but recommended)**: compare-and-set anchor for
  current Root-Epoch and operation digest.
- **Recovery trustees**: offline break-glass holders used only to reconstitute
  evaluator infrastructure; they are not callable by the normal endpoint.

## 3. Persistent public metadata

For a credential c:

```
C = (
  vaultID,
  serviceID,
  accountID,
  lineageID,
  credentialSalt,
  policyID,
  policyHash,
  policyEpoch
)
```

Lifecycle state adds:

```
rootEpoch
committedGeneration
pendingOperation?
historyDescriptors[]
freshnessGeneration
```

No service-password value is persisted.

## 4. Exact accepted domain

The existing EPSCD compiler is retained.

For policy P:

```
L_P = finite accepted password language
N_P = |L_P|
Rank_P   : L_P -> [0, N_P)
Unrank_P : [0, N_P) -> L_P
```

Rank/Unrank are mutual inverses.

## 5. Distributed exact-domain permutation

ASTER requires a threshold service implementing the ideal functionality:

```
DPRP.Eval(
    rootEpoch = e,
    context   = C,
    generation= g,
    domain    = N_P,
    capability= cap
) -> r_g in [0, N_P)
```

with:

1. `g1 != g2  => r_g1 != r_g2` for one `(e,C,P)` domain.
2. fewer than threshold evaluator compromises reveal no reusable Root-Epoch key;
3. the client learns only the authorized output;
4. evaluators refuse an authorization whose complete canonical context does not
   match;
5. the Root-Epoch key is never reconstructed at the client.

The production instantiation should use secure MPC around an established PRP/FPE
construction (or another reviewed threshold exact-domain PRP construction).
The research contribution is **not** a claim that MPC, threshold cryptography,
FF1, or cycle walking are new primitives.

Password derivation is:

```
r_g = DPRP.Eval(e, C, g, N_P, cap)
pwd_g = Unrank_P(r_g)
```

## 6. Capability format

Each capability signs/MACs an unambiguous canonical encoding of:

```
protocolVersion
operation
vaultID
serviceID
accountID
lineageID
credentialSalt
policyHash
policyEpoch
rootEpoch
generation
freshnessGeneration
expiry
nonce
useBudget
```

Every evaluator validates the *same complete scope* before participating.

A capability for `(C,e,g)` must not authorize `(C',e,g)`, `(C,e,g')`, another
operation, a stale freshness generation, or use after expiry/revocation.

### Security target: authorization non-amplification

Under the modeled deployment boundary and below the unrestricted-derivation
threshold, a capability budget q should expose at most q distinct authorized
credential outputs. A projection/wildcard capability is a negative control and
must violate this bound in proportion to the number of contexts it aliases.

## 7. Root-Epoch replacement

Suppose Root Epoch `e` is suspected compromised.

### 7.1 Create a fresh root

Evaluator domains run a fresh distributed key generation for an **independent**
Root-Epoch key `K_(e+1)`.

This is not resharing `K_e`.

### 7.2 Per-credential migration

For credential c currently at `(e,g)`:

1. persist `Prepared(c, old=(e,g), candidateEpoch=e+1)`;
2. inside MPC, derive the recent history under old epochs/generations;
3. search candidate generations `j=0,1,...` under `e+1`;
4. select the first candidate whose exact rank/password is outside the required
   authenticated history window;
5. persist the candidate descriptor **before** remote submission;
6. submit the candidate password to the target;
7. collect adapter-specific evidence.

The client need not learn rejected candidate passwords during history exclusion.

### 7.3 Evidence classification

- `NewOnly` -> commit `(e+1,j)`.
- `OldOnly` -> abort candidate and keep `(e,g)`.
- timeout / lost response / contradictory evidence / unsafe verification ->
  `UnknownOutcome`, retaining both descriptors.
- `Both` is ambiguous unless an adapter has an explicit overlap-then-revoke
  contract.

### 7.4 Retire old epoch

`K_e` shares may be cryptographically erased only when no credential is
committed to `e` and no pending/unknown operation still requires `e`.

## 8. Post-compromise healing claim

Let the adversary learn the complete old Root-Epoch key `K_e`.

After credential c is conclusively migrated to independently generated
`K_(e+1)`, exposure of `K_e` alone must not enable derivation of c's new
credential sequence.

After all credentials and pending states leave epoch e and all honest evaluator
domains erase their shares of `K_e`, later compromise of the new system does not
recreate the erased old root from ASTER state.

This is a scoped post-compromise recovery statement, not a claim that passwords
already observed by malware become secret again.

## 9. Break-glass recovery

Normal operation must not give the endpoint credentials that automatically
release enough network shares to reconstruct a root.

Offline recovery material is used to reconstitute evaluator infrastructure on a
clean recovery workstation under explicit operator procedure. It is not the
normal password-derivation path.

## 10. Main invariants

**I1 — Exact compliance**
Every released password belongs to `L_P`.

**I2 — Same-lineage non-repetition**
For fixed `(rootEpoch,C,P)`, different generations produce different passwords.

**I3 — Endpoint non-materialization**
Normal endpoint state never contains a Root-Epoch key or a reusable lineage key.

**I4 — Exact-scope authorization**
A valid capability for one canonical request cannot be replayed for another
context/generation/operation.

**I5 — Failure-safe migration**
Unknown remote outcomes preserve reconstructibility of both the old and candidate
credential descriptors.

**I6 — Safe epoch retirement**
An old Root-Epoch cannot be erased while any committed or unresolved credential
still depends on it.

**I7 — Healing separation**
Knowledge of `K_e` does not determine outputs under independently generated
`K_(e+1)`.

## 11. Candidate theorem statements for the paper

### Theorem A — Exact-domain injectivity
Assuming the distributed evaluator correctly realizes a permutation over
`[0,N_P)`, `Unrank_P(DPRP(C,g))` is injective in generation g.

### Theorem B — Capability confinement
Assuming signature/MAC unforgeability, canonical encoding, replay enforcement,
and honest validation by the required evaluator threshold, a capability bound to
request x cannot authorize evaluation at x' != x.

### Theorem C — q-capability non-amplification
Under exact request binding, single-use enforcement, and no unrestricted
derivation capability, q valid capabilities yield at most q distinct authorized
request outputs. The theorem is intentionally conditional on the deployment
access structure.

### Theorem D — Root-Epoch healing
If `K_(e+1)` is independently generated and the adversary does not compromise
its threshold, compromise of `K_e` does not computationally determine outputs in
epoch `e+1`.

### Theorem E — migration safety
If local commit occurs only on a singleton evidence state identifying the
candidate as authoritative, ASTER never advances its committed descriptor solely
from an ambiguous transport event.

## 12. Evaluation plan

### RQ1 — Does exact scope prevent authorization amplification?
Run complete, projected, and wildcard capability negative controls over
10^1..10^5 contexts and q=1..32.

### RQ2 — Does endpoint compromise remain bounded?
Compromise the client before, during, and after a legitimate derivation.
Measure the number of unauthorized contexts/generations derivable without
additional evaluator/approval compromise.

### RQ3 — Does Root-Epoch replacement actually heal?
Inject compromise of `K_e`, create `K_(e+1)`, migrate subsets of credentials, and
verify the exact boundary between still-exposed and healed records.

### RQ4 — Is strict non-repetition preserved?
For each translated policy, derive up to min(100000,N_P) generations per lineage
and check exact membership, replay determinism, and duplicates.

### RQ5 — Does migration survive remote ambiguity?
Fault-inject crash, timeout, lost response, delayed replication, contradictory
verification, and restart at every persistence/network boundary.

### RQ6 — What is the cost?
Benchmark 2-of-3 and 3-of-5 evaluator layouts over loopback, LAN, and WAN:
median/P95 derivation, migration candidate search, bytes transferred, MPC rounds,
and server CPU.

## 13. Baselines

- current local EPSCD;
- single-server exact-domain derivation;
- threshold PRF/POPRF with exact tickets but no exact-domain permutation;
- local deterministic generator;
- optional password-vault baseline for lifecycle cost only.

## 14. What ASTER does *not* claim

- It does not make a weak target password policy strong.
- It does not protect a plaintext password after malware legitimately observes
  that password on a compromised endpoint.
- It does not claim a new MPC, OPRF, threshold-sharing, FPE, or signature
  primitive.
- It does not claim availability against malicious evaluator denial-of-service.
- It does not equate proactive share refresh with recovery from a leaked root.
