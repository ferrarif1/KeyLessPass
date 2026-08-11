# Capability-Closed Credential Exposure: mathematical design note

Date: 2026-08-09
Status: candidate innovation method; research branch only

## 1. Problem not captured by a Root-Key threshold

A threshold or distributed PRF can prevent routine materialization of its
master key while still allowing a compromised endpoint to request an output for
every credential context. Root secrecy and credential compartmentalization are
therefore different properties.

The method below extends capability-closure analysis from a Boolean question—
“can these domains recover the Root Key?”—to a set-valued question—“which
credential contexts can these domains cause honest components to derive after
the user authorized a stated set of contexts?”

## 2. Model

Let:

- `D` be a finite set of deployment compromise domains;
- `C` be a finite set of canonical credential contexts;
- `K` be a finite set of parameterized capabilities;
- `R` be automatically exercisable monotone protocol rules;
- `C0(X,T)` be the initial capabilities available when domains `X subseteq D`
  are compromised and tickets for contexts `T subseteq C` have legitimately
  been issued.

Capabilities include `Request(c)`, `Approval(c)`, `Partial_i(c)`, `Derived(c)`,
raw shares, and master-reconstruction authority. Define the least fixed point:

```text
Cl_R(X,T) = lfp(S -> C0(X,T) union consequences_R(S)).
```

The **credential-exposure map** is:

```text
E_R(X,T) = { c in C | Derived(c) in Cl_R(X,T) }.
```

The Root-Key compromise predicate remains separate:

```text
M_R(X,T) = 1 iff RootCapability in Cl_R(X,T).
```

This produces two reported failure predicates:

1. **factor collapse:** `M_R(X,T)=1` for a domain set forbidden by the
   configured deployment threshold;
2. **authorization amplification:** `E_R(X,T)` contains credential contexts
   outside the legitimately approved set `T`; when `M_R(X,T)=0`, this isolates
   an over-broad callable interface rather than Root compromise.

The pair

```text
B_R(X,T) = (M_R(X,T), E_R(X,T))
```

is the **dual-collapse signature**.  It preserves a Boolean master-access
result and a set-valued output-access result instead of reducing both to one
threshold.

### Proposition 0: Root dominance

Assume every modeled credential is deterministically derivable from the Root
Key and public context, and the attacker can run the public derivation
algorithm. Then:

```text
M_R(X,T)=1 implies E_R(X,T)=C.
```

Consequently, for a proper approved subset `T`, Root compromise also violates
authorization non-amplification. The state "factor collapse without output
exposure" is not reachable in this profile.

### Proposition 1: Root-only analysis is incomplete below the threshold

For every finite context set `C` with at least two elements, there are two
protocol rule sets with identical `M_R(X,T)` for every `X,T` but different
credential-exposure maps.  Let both protocols keep the master unavailable.  In
the first, an honest evaluator accepts only a ticket's exact context, giving
`E_R(X,T)=T`; in the second, the same evaluator accepts that ticket for every
context, giving `E_R(X,T)=C` whenever `T` is nonempty.  Hence no analysis whose
only output is the master access structure can decide authorization
non-amplification.

Two protocols can expose the same requested outputs while one reconstructs the
Root Key and the other uses an unscoped DPRF. The output coordinate alone
therefore also fails to identify the cause and required repair. The coordinates
are not fully independent: under Root dominance, `M=1` forces maximal `E`.

## 3. Authorization non-amplification

For a protected family of compromised domain sets `F`, define:

```text
ACNA(F,q) iff
    for every X in F and T subseteq C with |T| <= q,
    E_R(X,T) subseteq T.
```

`ACNA` means **Authorized-Context Non-Amplification**. It does not promise that
an approved credential remains secret from a compromised endpoint; it promises
that approval for one set of contexts does not authorize additional contexts.

Define spill and normalized amplification:

```text
spill(X,T) = |E_R(X,T) \ T|

beta_q = max over X in F, |T| <= q of
         spill(X,T) / max(1, |C \ T|).
```

`beta_q=0` is equivalent to `ACNA(F,q)` in the finite model. A system can have a
safe master threshold while having `beta_1=1`, meaning one approval can expose
the rest of the modeled vault.

For a more interpretable curve, define the maximum unauthorized exposure under
an approval budget:

```text
A_R(X,q) = max over T subseteq C, |T| <= q of |E_R(X,T) \ T|.
```

`A_R(X,1)` is the one-approval blast radius; the complete sequence over `q`
distinguishes an exact per-context interface from a broad reusable capability.

To relate exposure to the deployment threshold, define the **credential
exposure threshold spectrum** for `k >= 1`:

```text
tau_E(k,q) = min |X| such that there exists T, |T| <= q,
             with |E_R(X,T) \ T| >= k.
```

The weighted form `rho_E(k,q)` replaces `|X|` with the sum of domain compromise
costs.  If no such `X,T` exists, the value is infinity.  Unlike a single Root
threshold, the spectrum states how many deployment domains must be compromised
to expose at least `k` unauthorized credential contexts under an approval
budget.

The spectrum is monotone in its exposure target:

```text
tau_E(k+1,q) >= tau_E(k,q).
```

Because the feasible approved sets for budget `q` are included in those for
`q+1`, it is non-increasing in the budget.  Under Root dominance, if `tau_M` is
the effective Root threshold, then:

```text
tau_E(k,q) <= tau_M for every 1 <= k <= |C|.
```

The inequality may be strict: an unscoped DPRF can expose many credential
outputs with one compromised endpoint while its Root threshold remains two or
infinite.

## 4. Projection-bound tickets

Let a canonical context have fields:

```text
c = (serviceID, accountID, credentialLineage,
     rootGeneration, policyID, ...).
```

Suppose a token checks only fields `B`, represented by projection `pi_B(c)`. A
ticket issued for target `c` is accepted for candidate `c'` exactly when:

```text
pi_B(c') = pi_B(c).
```

This induces an equivalence class:

```text
[c]_B = { c' in C | pi_B(c') = pi_B(c) }.
```

### Proposition 2: projection-exposure identity

Assume a compromised endpoint can request every context, can compute its own
partial evaluation for every context, and an honest token returns its partial
evaluation exactly when the ticket projection matches. Then:

```text
E_R({D},{c}) = [c]_B.
```

**Reason.** Every member of `[c]_B` passes the token predicate and completes the
two partial evaluations. Every context outside the class fails the only rule
that can produce the token partial. This is equality, not merely an upper bound.

### Corollary 1: collision criterion

In that profile, single-context non-amplification holds for the modeled context
space if and only if `pi_B` is injective on `C`.

For open-ended future contexts, empirical injectivity is insufficient. The
deployable repair is to bind the ticket to a collision-resistant digest of the
complete canonical derivation context and to its operation identifier, expiry,
and freshness generation—not to rely on fields that happen to be unique in the
current database.

### Proposition 3: exact approval-budget profile for projection tickets

Let the projection partition `C` into classes of sizes
`s_1 >= s_2 >= ... >= s_p`.  If a ticket for any member authorizes its complete
projection class, then:

```text
A_R(X,q) = sum from i=1 to min(q,p) of (s_i - 1).
```

**Reason.** A first ticket in a class exposes its other `s_i-1` members.
Further tickets in the same class add no exposed context and reduce the set
counted as unauthorized, so an optimum uses at most one ticket per class and
chooses the largest classes first.  The analyzer computes this exact profile,
not a sampling estimate.

## 5. Separation is insufficient

### Proposition 4: hidden master, full-vault exposure

If the endpoint can invoke all threshold participants on arbitrary contexts,
then a protocol may satisfy `M_R({D},empty)=0` while satisfying:

```text
E_R({D},empty) = C.
```

Therefore, “the master key is never reconstructed” does not imply a smaller
worst-case credential blast radius.

This proposition is operational rather than cryptographic: the DPRF can remain
fully secure while its legitimate evaluation interface is over-authorized.

## 6. Composition with a reviewed DPRF

Assume:

1. the chosen threshold/distributed PRF is secure for the stated corruption
   model under adaptive evaluation queries;
2. fewer than the threshold parties do not reveal the master key;
3. the protocol satisfies `ACNA(F,q)`;
4. tickets are unforgeable and bind the complete canonical context; and
5. no other rule releases raw token shares or a master capability.

Then an adversary compromising `X in F` and observing at most the authorized
set `T` obtains no protocol interface for a fresh `c* notin T`. Its ability to
distinguish or compute the fresh result is bounded by the selected DPRF's
security advantage plus ticket-forgery and context-digest collision terms.

This is a composition argument, not a new DPRF theorem. Its contribution is to
make the missing authorization premise explicit and mechanically checkable.

## 7. Analyzer and binding synthesis

The prototype:

- expands a finite canonical context space;
- computes the projection equivalence class exposed by each ticket policy;
- reports master materialization separately from credential exposure;
- reports unauthorized spill and exposure fraction; and
- computes the cardinality- and cost-based credential exposure threshold
  spectrum from the same `X,T` closure used for Root reachability; and
- enumerates inclusion-minimal collision-free field sets for the corpus.

For `m` context fields and `n` contexts, exhaustive field synthesis costs
`O(2^m n m)`, which is suitable for the small canonical schemas in this system.
No generic optimal-authorization or NP-hardness claim is made.

In the 32-context factorial experiment, authorizing one context produced the
exposure curve:

```text
root reconstruction                 32
unscoped DPRF token                 32
bind service only                   16
bind service + account               8
+ credential lineage                 4
+ root generation                     2
+ policy identity                     1
```

Thus master non-materialization alone gave no exposure reduction; exact
canonical binding reduced spill from 31 contexts to zero.

In the corrected joint model, exact binding with a two-domain Root threshold
has spectrum `[2,...,2]` over exposure targets 1--32.  An unscoped evaluator
with the same Root threshold has `[1,...,1,2]`: one endpoint exposes up to 31
unauthorized contexts, while exposing all 32 still requires Root compromise.
If the deployment threshold itself collapses to one domain, the spectrum is
`[1,...,1]`.  These three curves distinguish a safe interface, scope-only
amplification, and Root compromise.

The exact ticket-budget profile adds information that the one-ticket row does
not show.  For service-only binding, the projection has two classes of size 16,
so budgets one through four have maximum spill `[15, 30, 30, 30]`.  Full
canonical binding has 32 singleton classes and profile `[0, 0, 0, 0]`.

The standalone reference implementation remains linear in the number of
enumerated contexts for a fixed projection.  Across five runs, median/P95
latencies were 1.40/1.55 ms for 32 contexts, 34.82/36.88 ms for 1,024,
335.01/340.12 ms for 10,000, and 3,555.38/3,863.90 ms for 100,000.  These are
local analysis-tool measurements, not online derivation latency.

## 8. Candidate research contribution

The defensible candidate is not a new token, PRF, capability system, or access
structure. It is:

> a dual-collapse analysis for deterministic credential derivation that jointly
> computes (i) deployment-domain access to the master and (ii) the set of
> credential contexts reachable from scoped approvals, with an automated test
> for authorization amplification, an exact approval-budget exposure profile,
> a credential exposure threshold spectrum, and binding collisions.

The combination is useful because conventional recovery analysis detects
factor collapse but misses an evaluation oracle that leaks the whole vault one
context at a time, while DPRF security proofs assume the application controls
which evaluations are authorized.
