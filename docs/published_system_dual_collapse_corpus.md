# Published-work corpus for dual-collapse credential analysis

Audit date: 2026-08-09  
Status: prior-art and model-routing corpus; not an experimental comparison

## Corpus rule

Only peer-reviewed, published work is used as a comparison object.  The frozen
project's earlier encoders, recovery drafts, and unpublished protocol variants
are excluded.  A row records only properties stated by the cited publication;
an omitted application-level authorization rule is marked `not specified`, not
inferred to be insecure.

## Published systems and foundations

| Published work | Venue | Published contribution relevant here | Master/factor coordinate | Per-context exposure coordinate | Modeling disposition |
|---|---|---|---|---|---|
| MFKDF | USENIX Security 2023 | Policy and threshold combinations of authentication factors derive a key. | Directly relevant to factor policies. | Credential-by-credential callable output scope is not its stated object. | Use for factor-policy prior art, not as a weak threshold baseline. [USENIX](https://www.usenix.org/conference/usenixsecurity23/presentation/nair-mfkdf) |
| TOPPSS | ACNS 2017 | Password-protected secret sharing using a threshold OPRF. | Directly relevant to threshold recovery and server corruption. | Its protected-secret reconstruction interface is not a deterministic multi-credential oracle. | Use for threshold OPRF and recovery prior art. [Springer DOI](https://doi.org/10.1007/978-3-319-61204-1_3) |
| Pythia | USENIX Security 2015 | A password PRF service with tweaks, rate limiting, and key rotation. | Central PRF-service key, not the proposed deployment-domain threshold. | Relevant: evaluations are indexed by password-service inputs and tweaks. | Encode only after taking authorization and rate-limit semantics from the paper. [Paper](https://www.usenix.org/system/files/conference/usenixsecurity15/sec15-paper-everspaugh.pdf) |
| LaKey | USENIX Security 2024 | Distributed key derivation without revealing the master in the clear. | Directly relevant to master non-materialization. | Identity/input authorization is application-defined rather than a claimed complete-vault exposure analysis. | Decisive prior art against a new-DPRF claim; suitable scope-audit subject. [USENIX](https://www.usenix.org/conference/usenixsecurity24/presentation/geihs) |
| SafetyPin | OSDI 2020 | Distributed-HSM encrypted-backup recovery protected by a human-memorable secret. | Directly relevant to compromise threshold and recovery availability. | Recovers a selected backup key, not a service-password context family. | Use for recovery architecture and compromise assumptions. [Paper](https://www.usenix.org/system/files/osdi20-dauterman_safetypin.pdf) |
| SVR3 | OSDI 2024 | Deployed secret recovery across heterogeneous enclaves and cloud providers with rollback protection. | Directly relevant to deployment independence and recovery. | Recovers a user's protected value; a derived-credential context space is not its object. | Use for operational trust-domain and freshness prior art. [USENIX](https://www.usenix.org/conference/osdi24/presentation/connell) |
| Flock | OSDI 2024 | On-demand deployment of distributed trust across cloud providers for multiple applications. | Directly relevant to whether distinct logical nodes are distinct trust domains. | Scope depends on each hosted application. | Use for deployment feasibility; do not assign application scope absent an app specification. [USENIX](https://www.usenix.org/conference/osdi24/presentation/kaviani) |
| SPHINX | ICDCS 2017 / IEEE TDSC | Device/server password storage intended to hide passwords from the store. | Relevant compromise-case analysis. | Per-site output compartmentalization is relevant. | Encode only from the complete published protocol. [IBM](https://research.ibm.com/publications/sphinx-a-password-store-that-perfectly-hides-passwords-from-itself) |
| Constrained PRFs | ASIACRYPT 2013 | A constrained key evaluates a PRF only on an allowed input subset. | Master access is handled cryptographically, not as deployment closure. | Direct foundation for cryptographic subset restriction. | Decisive prior art against claiming context restriction as a new primitive. [Author page](https://crypto.stanford.edu/~dabo/pubs/abstracts/dumbledore.html) |
| Macaroons | NDSS 2014 | Authorization credentials attenuated through contextual caveats. | Not a threshold-root analysis. | Direct foundation for context-restricted delegated authority. | Decisive prior art against claiming context-bound tickets. [NDSS](https://www.ndss-symposium.org/ndss2014/ndss-2014-programme/macaroons-cookies-contextual-caveats-decentralized-authorization-cloud/) |
| Stateful Least Privilege Authorization | USENIX Security 2024 | Stateful, attenuated tokens enforce minimal access. | Not a threshold-root analysis. | Direct foundation for fine-grained, stateful token scope. | Use as closest authorization-system prior art. [USENIX](https://www.usenix.org/conference/usenixsecurity24/presentation/cao-leo) |
| PolyScope | USENIX Security 2021 | Multi-policy permission expansion is reduced to concrete authorized attack operations. | Not a cryptographic master-access analysis. | Closest general precedent for computing expanded consequences. | The candidate must exceed a generic permission-expansion restatement. [USENIX](https://www.usenix.org/conference/usenixsecurity21/presentation/lee-yu-tsung) |
| MulVAL | USENIX Security 2005 | Datalog rules compute multistage attacker consequences and witnesses. | Can encode master reachability generically. | Can encode context facts if a modeler supplies them. | Decisive prior art against claiming least-fixed-point reachability. [USENIX](https://www.usenix.org/conference/14th-usenix-security-symposium/mulval-logic-based-network-security-analyzer) |
| Paralysis Proofs / Dynamic Access Structure Systems | AFT 2019 | Effective dynamic access structures and safe policy migration for custody. | Directly relevant to effective access and lifecycle changes. | Not a credential-derivation exposure profile. | Decisive prior art against claiming effective/dynamic access structures. [DOI](https://doi.org/10.1145/3318041.3355459) |

## What the corpus supports

The corpus does not support novelty for any primitive or for generic policy
analysis.  It supports a narrower gap statement:

1. recovery and threshold work primarily state when a protected key/value can
   be recovered;
2. DPRF and constrained-authorization work state how a requested evaluation or
   allowed subset is protected;
3. generic policy analyzers compute permissions or attack operations; but
4. the reviewed works do not present the specific two-coordinate diagnostic
   used here: master-capability reachability together with an exact,
   approval-budget-indexed set of deterministically derivable credential
   contexts.

This is a provisional gap, not a priority proof.  The next evidence gate is to
encode message-level profiles from publications whose specifications are
complete enough, record every ambiguity, and seek independent review of those
encodings.

