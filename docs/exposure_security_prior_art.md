# Known-credential exposure and credential-space prior-art audit

Audit date: 2026-08-09  
Scope: PRP security under observed pairs, small-domain/FPE attacks, key exposure,
deterministic password exposure, server-verifier compromise, rekey, and rollback.

## 1. Novelty boundary

The following are standard facts and must not be presented as new mathematics:

- a random permutation conditioned on q consistent input/output pairs is uniform
  over permutations consistent with those pairs;
- an unseen input is therefore uniform over the `N-q` unused outputs;
- PRP security is defined against adversaries that observe or choose permutation
  inputs and outputs;
- rank-then-encipher transfers the finite-domain cipher to a bijectively indexed
  format;
- small FPE domains admit generic/codebook and construction-specific attacks;
- key exposure requires rekeying and does not disappear because a public tweak
  changes.

The manuscript contribution can only be the specialization of these facts to a
generation-indexed credential sequence and its lifecycle state.

## 2. Observed-credential model

All of the following are public:

```text
P, L_P, N_P, Rank_P, Unrank_P,
schemeVersion, vaultID, serviceID, accountID,
credentialSalt, rootGeneration,
policyID, policyVersion, policyHash, policyEpoch,
credentialGeneration and authenticated history descriptors.
```

If `password_g` is exposed, the attacker computes
`r_g = Rank_P(password_g)` and learns exactly one permutation pair `(g,r_g)`.
This observation does not become harmless because the scheme is injective:

> Injectivity provides non-repetition; PRP security provides unpredictability
> under observed input/output pairs.

For q distinct consistent pairs and an ideal random permutation on N ranks, a
fixed unseen input has probability

```text
(N-q-1)! / (N-q)! = 1/(N-q)
```

for each unused rank. The result should be called **Observed-Credential Residual
Uniformity** only as an EPSCD-specific proposition. Its proof is standard
random-permutation symmetry, not a newly discovered theorem.

For a real backend, replace equality by a computational game: an adversary that
distinguishes the unseen EPSCD rank from the corresponding without-replacement
ideal game yields a distinguisher against the finite-domain PRP, plus any HKDF
context-separation advantage. The statement must include the backend's domain,
query, tweak, and cycle-walk bounds.

## 3. Published work that constrains the claim

| Work | Established result | Consequence for EPSCD |
|---|---|---|
| Black and Rogaway, CT-RSA 2002 | Ciphers on arbitrary finite domains and cycle walking | EPSCD does not invent a finite-domain permutation or cycle walking. |
| Bellare et al., SAC 2009 | Formal FPE and rank-then-encipher, including arbitrary formats through ranking | The policy-space PRP composition is prior art; only credential sequencing/lifecycle specialization is claimable. |
| NIST SP 800-38G Rev. 1 draft | FF1 requirements and minimum domain response to small-domain vulnerabilities | Backend eligibility is not a password-strength claim. The current prototype's one-million minimum follows a construction requirement. [NIST](https://csrc.nist.gov/pubs/sp/800/38/g/r1/2pd) |
| Durak and Vaudenay, CRYPTO 2017 | Practical attacks on FF3 over small domains and tweak/domain-separation weaknesses | Public tweaks and minimum domain do not automatically provide adequate concrete security. |
| Hoang, Tessaro, Trieu, CRYPTO 2018 | Known-plaintext message-recovery attacks against FPE, including multi-target settings | A manuscript theorem in the ideal permutation model cannot be advertised as a concrete FF1 exposure guarantee. [ePrint copy](https://eprint.iacr.org/2018/556.pdf) |
| Bellare and Hoang, ACM CCS 2017 | Identity-based FPE to localize damage from key exposure | Key-exposure localization is established prior art. EPSCD must state rather than overclaim its per-credential key hierarchy and remediation. [CCS record](https://doi.org/10.1145/3133956.3133995) |
| SPHINX, IEEE TDSC 2019 | Password-management design with strong compromise-specific guarantees | Exposure and compromise domains are central prior art in password managers; EPSCD must compare security objects precisely. |
| PALPAS, ARES 2015; PwdHash and Password Multiplier, 2005 | Service-separated deterministic credentials and public per-service metadata | Knowing policy/context while lacking the derivation secret is not a new threat model. |

## 4. Credential-space adequacy

Define

```text
B_P = log2(N_P)
```

and keep two independent deployment checks:

1. **Backend-domain eligibility**: whether the concrete permutation accepts the
   domain (currently at least 1,000,000 values and at most 512 bits).
2. **Credential-security adequacy**: whether `B_P` meets the service's configured
   offline/online guessing threat profile.

A domain can satisfy the FF1 minimum and still provide only about 20 bits of
candidate space. Conversely, a policy space can be cryptographically large yet
exceed the current backend's 512-bit implementation ceiling. Neither check
substitutes for the other.

If a service verifier and its salt/hash parameters are compromised, the attacker
can enumerate `L_P` directly against the verifier, bypassing `Kroot`, `Kcred`,
and the PRP. Therefore:

> a 256-bit Root Key does not imply 256-bit service-password guessing strength.

## 5. Key-compromise semantics established by code inspection

Current scheme version 1 computes:

```text
Kcred = HKDF(Kroot, credentialSalt,
             schemeVersion || vaultID || serviceID || accountID || rootGeneration)
```

`policyEpoch` appears only in the permutation tweak. Once `Kcred` leaks, an
attacker can compute the permutation for any public policy epoch/tweak that uses
the same credential key. Thus:

> policyEpoch change does not repair Kcred compromise.

The minimum independent remediation is a new credential-key lineage. A fresh
authenticated credential salt is the smallest compatible mechanism, provided
the new lineage first excludes all authenticated history values required by the
service. A separate `credentialKeyGeneration` counter may make this intent more
auditable; the implementation choice must be fixed before the paper formula is
changed.

Root-Key compromise is broader. Re-randomizing Shamir shares of the same
`Kroot` changes storage encodings but not the compromised secret. Repair requires
a random replacement Root Key, `rootGeneration++`, a new share-set generation,
fresh credential keys, and remote rotation of every affected account.

## 6. Rollback boundary

The current epoch-local theorem assumes `g1 != g2`. Restoring a valid old
authenticated descriptor reuses `g`, so the old password is deterministically
rederived. This violates deployment no-repeat without violating permutation
injectivity. Authenticated storage alone detects tampering, not replay of a
complete old snapshot.

The minimum freshness anchor must monotonically cover at least:

```text
rootGeneration,
credentialGeneration (per credential or through an authenticated aggregate),
policyEpoch,
shareSetGeneration,
checkpoint digest/ancestry.
```

An endpoint credential that can update/read freshness belongs to `Closure(D)`.
It must not also authorize release of `S_N`, or the freshness mechanism itself
would contribute to factor collapse.

## 7. Defensible exposure contribution

The strongest supportable claim is:

> EPSCD gives a generation-indexed without-replacement credential sequence and
> makes prior-password exposure explicit: ideal residual support is exactly the
> unused policy space, while a real instantiation inherits only the selected
> PRP's computational security and concrete small-domain bounds. The lifecycle
> separately specifies credential-key rekeying, Root-Key replacement, verifier
> compromise, and rollback freshness.

This is stronger and more complete than the current manuscript, but it is an
application-specific security treatment, not a new PRP theorem or a proof that
the bounded FF1 prototype withstands every known-pair attack.

