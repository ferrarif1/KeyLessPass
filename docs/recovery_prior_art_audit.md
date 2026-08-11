# Recovery prior-art audit

Audit date: 2026-08-09  
Scope: random-root recovery, threshold/factor recovery, distributed release,
heterogeneous trust, freshness, and endpoint-invokable recovery capability.  
Publication rule: only peer-reviewed works are eligible as manuscript comparison
targets. Preprints are recorded to avoid accidental priority claims but must be
labelled as preprints and are not experimental baselines.

## Research question under audit

The candidate system property is not “2-of-3 Shamir recovery.” It asks whether a
logical access structure remains valid after closing a compromised deployment
domain over files, local secrets, tokens, signing authority, automatic calls, and
responses that honest services will release to those capabilities.

For top-level shares `(S_D,S_U,S_N)`, define factor preservation as:

```text
for every protected single domain X:
    |Closure(X) intersect {S_D,S_U,S_N}| < 2
```

The proposed construction keeps `S_D` on a managed endpoint, `S_U` on a
copyable removable/offline medium, and splits `S_N` over network nodes. The
endpoint may request recovery but independent authorization domain `A` must
approve a session-bound release. `A` holds no Root-Key share.

## Closest published work

| Work | Published status | Recovered object / mechanism | Threshold and trust split | Freshness/revocation | Relevance and remaining distinction |
|---|---|---|---|---|---|
| Shamir, *How to Share a Secret*, CACM 1979 | Peer reviewed | Arbitrary secret from k-of-n polynomial shares | Cryptographic access structure | Not supplied | Establishes the top-level and network-fragment mechanisms. It does not model deployment capabilities or release authorization. |
| Nair and Song, MFKDF, USENIX Security 2023 | Peer reviewed | A key derived/reconstituted from multiple factors | Threshold multi-factor KDF with factor replacement | Stateful factor policy; later cryptanalysis identifies integrity/specification weaknesses | Closest factor-loss recovery work. EPSCD starts from a random Root Key and separates loss re-share from Root-Key replacement. It must not claim first threshold recovery. [USENIX](https://www.usenix.org/conference/usenixsecurity23/presentation/nair-mfkdf) |
| Scarlata, Backendal, Haller, *MFKDF: Multiple Factors Knocked Down Flat*, USENIX Security 2024 | Peer reviewed | Cryptanalysis of MFKDF state and constructions | Shows factor-policy claims depend on precise state integrity and implementation | Central caution | Supports explicit authenticated generations and compromise semantics; not a competing recovery system. |
| Jarecki et al., TOPPSS, ACNS 2017 | Peer reviewed | Password-protected secret sharing using threshold OPRF | Threshold servers and password-only reconstruction | Protocol security, not EPSCD lifecycle freshness | Establishes that threshold OPRF/PPSS is prior art and makes the superseded OPRF prototype unsuitable as a novelty axis. [Springer/Edinburgh](https://www.research.ed.ac.uk/en/publications/toppss-cost-minimal-password-protected-secret-sharing-based-on-th/) |
| Dauterman et al., SafetyPin, OSDI 2020 | Peer reviewed | PIN-protected encrypted backup key/data | Location-hiding selection and threshold HSM cluster; trust distributed across many HSMs | Recovery logging and puncturing/revocation after use | Strongest published warning that distributed recovery needs release logging and post-recovery revocation. It protects a cloud backup, not one heterogeneous share within a local 2-of-3 access structure. [USENIX](https://www.usenix.org/conference/osdi20/presentation/dauterman-safetypin) |
| Connell et al., SVR3, OSDI 2024 | Peer reviewed | End-to-end encryption secret key | Heterogeneous enclave types across different cloud providers | Explicit rollback protection and fault tolerance | Very close in motivation: heterogeneous administrative/technology domains prevent one enclave class/provider from being a central attack point. Therefore “heterogeneous recovery” is not novel. The narrower candidate is closure analysis for an endpoint that already holds one top-level share and can invoke honest recovery services. [USENIX](https://www.usenix.org/conference/osdi24/presentation/connell) |
| Little et al., secure account recovery, USENIX Security 2024 | Peer reviewed | Privacy-preserving website account recovery data | Multiple recovery servers, OPRF-based privacy/rate limiting, proofs against malicious behavior | Online rate limiting and recovery protocol state | Shows that distributed recovery-server authorization/privacy protocols are established. The candidate distinction is preserving a pre-existing heterogeneous Root-Key access structure, not inventing server-assisted recovery. [USENIX paper](https://www.usenix.org/system/files/usenixsecurity24-little.pdf) |
| Shirvanian et al., SPHINX, IEEE TDSC 2019 | Peer reviewed | High-entropy service passwords via device/server protocol | Device compromise resistance through a specialized password-store architecture | Service-specific state; not top-level factor lifecycle | Important nearby password-management architecture: compromise domains, not just stored ciphertext, determine security. SPHINX does not recover a random EPSCD Root Key through D/U/N shares. [IBM Research](https://research.ibm.com/publications/building-and-studying-a-password-store-that-perfectly-hides-passwords-from-itself) |
| Li and Evans, Horcrux, 2017 manuscript | Publication not verified in this audit | Stored credentials secret-shared across servers | Decentralized password storage and split client | Not the proposed lifecycle | Must not be used as a peer-reviewed comparison until venue verification. It also stores/reconstructs credential values, whereas EPSCD recovery must never store service-password values. |

## Preprints and specifications that constrain priority language

- Kintsugi (2025 preprint) combines decentralized key recovery with Shamir,
  libp2p, and threshold OPRF. It rules out claims that peer recovery or threshold
  OPRF integration is new, but it is not an eligible published experimental
  baseline.
- Apollo (2025 preprint) studies metadata-private recovery. It rules out broad
  claims about opaque recovery objects, but is not an eligible published
  baseline.
- Signal's earlier SVR engineering description uses enclave replication and
  consensus for guess-count freshness. The peer-reviewed SVR3 OSDI paper is the
  preferred manuscript citation.

## Capability and authorization analysis

Classical capability-security work establishes that authority includes what a
principal can cause cooperating systems to do, not only secret bytes it stores.
The proposed `Closure(X)` is therefore best positioned as a purpose-built threat
analysis adapted to recovery shares, not as a newly invented general theory of
capabilities. Its useful contribution is the checkable counterexample:

```text
D stores S_D and an automatically honored network-release credential
=> S_D is in Closure(D)
=> honest nodes release S_N, so S_N is in Closure(D)
=> Compromise(D) reconstructs Kroot
```

Network threshold storage does not fix this if compromised D can obtain enough
honest node responses. Release authorization must be in an independently
administered domain and bound to a fresh recovery session.

## Novelty verdict

The audit **does not support** claiming heterogeneous recovery, threshold
recovery, network-assisted recovery, threshold nodes, independent approval,
freshness, or post-recovery revocation individually as new.

It tentatively supports a narrower systems contribution:

> a closure-based method for detecting when the deployment of one top-level
> recovery share grants callable authority over another, together with a
> generation- and session-bound construction that preserves the intended
> top-level access structure.

This must be described as a scoped system-security abstraction and evaluated
against explicit compromise closures. “First,” “novel heterogeneous recovery,”
and “new access structure” are not defensible. If further searching finds a
published construction with the same endpoint-share-plus-callable-network-share
analysis, recovery must be downgraded to integrated lifecycle functionality.

## Requirements imposed by the audit

1. Delete the active design's `K_view/K_data` split, opaque-object scanning,
   threshold OPRF, and erasure-coded ciphertext story. Those features are
   separately well covered and obscure the factor-collapse question.
2. Store only fragments of `S_N`; never store or recover a service-password value.
3. Bind every release to `vaultID`, `rootGeneration`, `shareSetGeneration`,
   random `opID`, purpose, expiry, and an ephemeral recovery public key.
4. Treat endpoint tokens and freshness-service credentials as members of
   `Closure(D)` but do not let them authorize network-share release.
5. Treat the U medium as cloneable.
6. Distinguish ordinary factor-loss re-sharing from Root-Key compromise and
   replacement.
7. Report the independence of D/U/N/A as a deployment assumption; co-location
   behind one administrative account can invalidate factor preservation.

