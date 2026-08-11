# EPSCD and Recovery Security Argument

## Assets and exposure levels

- `K_root`: uniformly sampled 256-bit Root Key;
- `K_cred`: context-separated credential permutation key;
- service-password plaintext during derivation and use;
- authenticated policy, generation, lineage, share-set, and history state.

The analysis distinguishes observation of one or more generation/password
pairs, compromise of a server verifier, compromise of `K_cred`, compromise of
`K_root`, and compromise of recovery deployment domains. A server verifier can
test the finite accepted language directly; neither `K_root` nor a 256-bit KDF
claim raises the password beyond that language's actual guessing strength.

## EPSCD claims

1. Every successful derivation belongs to the compiled finite policy language.
2. `N_P` exactly counts that language for the supported policy subset.
3. `Rank` and `Unrank` are mutual inverses.
4. In one fixed credential lineage and policy epoch, distinct generation inputs
   below `N_P` cannot repeat a password.
5. For an ideal random permutation conditioned on `q` consistent known pairs,
   an unseen input is uniform over the `N_P-q` unused legal outputs.
6. A real backend supplies only its computational PRP guarantee under known
   input/output queries; the ideal conditional statement is not a key-recovery
   theorem for FF1.
7. Injectively encoded contexts computationally separate credentials under the
   HKDF assumption.
8. Cross-epoch history exclusion terminates within `e+1` distinct inputs when
   `e` excluded legal values exist and is marginally uniform over the remaining
   set in the ideal-permutation model.
9. Suspected `K_cred` compromise opens a new credential lineage by replacing
   the credential salt. Incrementing `policyEpoch` alone is not remediation.
10. `K_root` compromise affects all credential keys derived from that Root Key;
    recovery requires Root-Key replacement and subsequent remote credential
    rotation, not ordinary re-sharing.

## Recovery claims

For a compromise domain `X`, `Closure(X)` contains everything directly held by
the domain plus all protocol results honest infrastructure automatically
releases to its credentials. The intended top-level 2-of-3 structure is
factor-preserving only when no single protected domain's closure contains two
of `{S_D,S_U,S_N}`.

The active profile Shamir-splits the authenticated `S_N` envelope into 3-of-5
node fragments. Network release requires two distinct Ed25519 approvals over a
short-lived, generation-bound ticket and encrypts each response to that
ticket's ephemeral X25519 session. A device request credential is not approval
authority. Any reconstructed `S_N` still requires `S_D` or `S_U`, plus KCV and
envelope-MAC verification, before Root-Key acceptance.

## Freshness claims

The CAS checkpoint binds Root-Key generation, share-set identity/generation,
CDR ancestry, and each credential's policy epoch, generation, and lineage.
Lexicographically older states are rollback; equal counters with different
lineage or digest are forks. This assumes an independently administered,
durable checkpoint. Local-only storage cannot rule out replay of every valid
snapshot.

## Non-claims

- No new FPE, arbitrary-domain cipher, HKDF, Shamir, signature, or automaton
  primitive is claimed.
- Fixed-key permutation outputs are without replacement, not independent.
- Different credential contexts or policy epochs can emit the same string.
- The supported IR is not arbitrary PCRE and cannot encode semantic server
  checks unless translated exactly.
- Bounded FF1 cycle walking may fail closed and is not a total arbitrary-domain
  backend.
- Re-sharing does not revoke already compromised old threshold material.
- Approval independence is a deployment assumption; one administrator or cloud
  account controlling D, N, and A invalidates it.
- The recovery prototype does not provide Byzantine node tolerance, transport
  anonymity, production durability, or human-approval correctness.
- Model checking validates only the finite abstractions and listed invariants.
