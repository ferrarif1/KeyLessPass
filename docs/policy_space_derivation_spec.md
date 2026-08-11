# Exact Policy-Space Credential Derivation Specification

Status: public algorithm specification for `EPSCD`, scheme version 1
Cryptographic boundary: EPSCD composes published primitives; it does not define a new cipher.

## 1. Finite policy language

For a canonical policy `P`, let `Σ_P` be its ordered finite alphabet and let

\[
\mathcal L_P \subseteq \bigcup_{\ell=L_{min}}^{L_{max}}\Sigma_P^\ell
\]

be the accepted password set. Strings use shortlex order: shorter accepted strings precede longer strings, and equal-length strings follow the declared alphabet order.

Scheme version 1 supports finite length ranges, allowed and forbidden characters, character-class minima and maxima, fixed positions/prefix/suffix, first/last-character restrictions, bounded identical or ASCII-sequential runs, per-character occurrence caps, and finite forbidden-substring sets. Unsupported semantics—including back-references, look-around, dictionary or breach-list queries, username similarity, and server-side history not represented by authenticated metadata—must be rejected rather than approximated.

## 2. Canonical policy representation

The canonical representation fixes UTF-8 encoding, unique alphabet order, class identifiers and alphabets, integer semantics, the distinction between absent and zero values, forbidden-substring ordering, and RFC 8785 JSON serialization. Define

\[
policyHash=\operatorname{SHA256}(canonicalPolicyBytes).
\]

Any semantic or canonicalization change requires a new scheme or policy-IR version. A dependency upgrade must not silently reinterpret an existing policy.

## 3. Deterministic finite-state compilation

The compiler constructs a deterministic partial automaton

\[
A_P=(Q,\Sigma_P,\delta,q_0,F).
\]

Each reachable state is the product of only the enabled finite components: saturated class counters, optional per-character counters, prefix/suffix progress, edge-position state, run state, and forbidden-substring matcher state. Invalid transitions are absent. Acceptance at length `ell` requires `L_min <= ell <= L_max`, all lower bounds, and every terminal condition.

Worst-case product growth is exponential in the number and bounds of enabled counters. The implementation exposes state count and rejects compilation above a configured limit (`250,000` states by default). EPSCD therefore supports a documented regular subset; it does not claim efficient compilation of every regular expression.

## 4. Exact counting

For target length `ell`, let `C_ell(i,q)` count accepted completions when `i` symbols have been emitted and the automaton is in state `q`:

\[
C_\ell(\ell,q)=\begin{cases}1&q\in F_\ell\\0&q\notin F_\ell\end{cases}
\]

and

\[
C_\ell(i,q)=\sum_{a\in\Sigma_P:\delta(q,a)\downarrow}
C_\ell(i+1,\delta(q,a)).
\]

All cells are arbitrary-precision non-negative integers. Consequently,

\[
N_P=|\mathcal L_P|=\sum_{\ell=L_{min}}^{L_{max}}C_\ell(0,q_0),
\qquad H_P=\log_2N_P.
\]

`N_P` remains exact; only its displayed logarithm is rounded.

## 5. Ranking and unranking

`Unrank_P(r)`, for `0 <= r < N_P`, first selects the length interval and then selects each character by the exact completion-count intervals behind outgoing transitions. `Rank_P(w)` adds all shorter-length counts and all preceding transition intervals while scanning an accepted string. Both algorithms use the same canonical alphabet order.

The recurrence partitions accepted suffixes into disjoint ordered intervals, yielding

\[
Rank_P(Unrank_P(r))=r,
\qquad
Unrank_P(Rank_P(w))=w.
\]

## 6. Credential context

The public context is:

```text
schemeVersion = 1
vaultID
serviceID
accountID
credentialSalt (128 bits)
rootGeneration
policyID
policyVersion
policyHash
policyEpoch
```

Let `K_root` be a uniformly sampled 256-bit key. EPSCD derives

\[
K_{cred}=\operatorname{HKDF-SHA256}
(K_{root},credentialSalt,JCS(keyContext)).
\]

The key context is domain-separated by `EPSCD/credential-key/scheme-v1` and binds the scheme, vault, service, account, and root generation. The permutation tweak is RFC-8785-encoded and domain-separated by `EPSCD/policy-space-permutation/scheme-v1`; it binds every public context field above, including `policyHash`.

`credentialGeneration` is intentionally absent from both key context and tweak. It is the input to one fixed permutation during an epoch.

## 7. Finite-domain permutation

For domain size `N`, EPSCD assumes a reviewed keyed family

\[
\pi_{K,T,N}:[0,N)\rightarrow[0,N)
\]

that is a bijection and provides an inverse. Pseudorandomness claims require the selected family to meet its stated PRP security bounds. The abstraction is not presented as a new primitive.

The reference backend applies FF1-AES-256 to the smallest binary superset `M=2^ceil(log2 N)` and cycle-walks until the output is below `N`. It accepts domains of at least `1,000,000`, supports at most `512` bits, and fails closed after `1,024` primitive calls. For an ideal permutation and non-power-of-two `N`, `N>M/2` and

\[
\Pr[W>k]\le\left(\frac{M-N}{M}\right)^k<2^{-k}.
\]

This is an ideal-model availability bound, not a proof that the bounded concrete backend is total.

## 8. Password derivation

For `g=credentialGeneration`:

\[
0\le g<N_P,
\quad r_g=\pi_{K_{cred},T,N_P}(g),
\quad Password_{P,g}=Unrank_P(r_g).
\]

If `g >= N_P`, derivation fails. The implementation never reduces a generation modulo `N_P`.

## 9. Policy epochs and history exclusion

A substantive remote-policy change increments `policyEpoch` and starts a new local generation sequence. Because different epochs may have overlapping languages, cross-epoch non-repetition is enforced by authenticated metadata for the required history window:

```text
g := 0
repeat:
    candidate := Derive(newEpoch, g)
    history := ReDeriveAuthenticatedPredecessors(h)
    if candidate not in history: return (g, candidate)
    g := g + 1
until g == N_P
```

Skipped indices remain consumed. Password strings and ranks need not be stored. If the exclusion set has size `e < N_P`, injectivity guarantees termination after at most `e+1` distinct candidates.

## 10. Proven properties

1. **Policy correctness.** `Unrank_P(r)` is in `L_P` for every valid rank.
2. **Exact count.** The dynamic-programming total equals `|L_P|`.
3. **Rank/unrank bijection.** Ranking and unranking are mutual inverses.
4. **Uniform marginal.** For a fixed generation and an ideal uniformly keyed permutation, every password in `L_P` has probability `1/N_P`.
5. **Intra-epoch collision freedom.** Distinct valid generations under one key, tweak, and policy produce distinct passwords.
6. **Computational context separation.** Distinct credential contexts are separated under HKDF and tweakable-PRP assumptions; this is not an information-theoretic promise of no string collision across contexts.
7. **History-window termination.** With an authenticated exclusion set smaller than `N_P`, deterministic scanning finds a non-excluded password within `e+1` attempts.

Functional properties 1–3 and 5 follow from automaton determinism, exact interval partitioning, and permutation injectivity. Property 4 is an ideal-permutation statement: fixed-key outputs are deterministic and samples across generations are without replacement, not independent.

## 11. Reproducibility requirements

An implementation claiming scheme-version-1 compatibility must publish the canonical policy bytes, `policyHash`, key context, tweak bytes, domain size, generation, permuted rank, and resulting password for fixed test vectors. It must also report compiler state limits, finite-domain backend limits, and fail-closed behavior.
