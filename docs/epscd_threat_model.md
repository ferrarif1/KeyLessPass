# EPSCD Threat Model

Date: 2026-08-10

## Protected properties

EPSCD aims to provide:

1. exact membership of every successful derived password in the compiled bounded policy language;
2. deterministic reconstruction from authenticated metadata and root material;
3. strict non-repetition for distinct generations in one lineage;
4. no persistent service-password values in the baseline metadata store;
5. no local generation advance without evidence satisfying the selected adapter contract;
6. continued reconstruction of old and new candidates while a remote update is uncertain.

## Adversary capabilities

The adversary may read public policy and generation metadata, steal the metadata database, observe known `(generation,password)` pairs, delay/drop/duplicate/reorder network messages, crash the client at persistence or network boundaries, replay stale local state, compromise an adapter, or compromise the endpoint during a legitimate derivation session.

## Security boundaries

- `generation` is public. A known password reveals its policy rank and therefore a known input/output pair for the selected permutation.
- Policy metadata and `credentialSalt` are not secrets. Confidentiality of unseen outputs relies on the credential key and the selected PRP assumptions.
- Compromise of `K_cred` compromises the corresponding lineage. This is not forward secrecy.
- Compromise of `K_root` compromises every credential context derived from that root. Shamir resharing without replacing the root is not root-compromise recovery.
- Endpoint compromise during display, submission, or verification can capture the plaintext credential and is not prevented.
- A weak accepted language remains weak. Exact derivation measures `log2|L_P|`; it cannot add entropy outside the verifier's accepted set.
- Same-lineage non-repetition prevents deterministic sequence collisions. It is not a general defense against user password reuse or cross-lineage equality.
- A malicious or incorrect adapter can forge observations inside its trust boundary. Adapter identity, implementation review, and deployment isolation are operational requirements.

## Rotation assumptions

An adapter declares its evidence requirement and capabilities before an operation. Old-password verification is optional because repeated authentication can consume a lockout budget. If the declared contract cannot be established safely, the adapter is `UNKNOWN_ONLY` and the protocol must retain an unresolved operation for manual or later authoritative reconciliation.

The abstract model assumes the authenticated local journal is durable and integrity protected. Rollback of the complete journal and freshness checkpoint is separately detectable only when a non-rollbackable or independently administered freshness mechanism is deployed.

## Out of scope

- replacement of password authentication with phishing-resistant modern authentication;
- compromise of the target verifier itself;
- availability against an operator or adapter that refuses all progress;
- constant-time behavior of the complete application and UI stack;
- global non-repetition across independent lineages without retaining comparison material;
- production certification of the FF1 dependency or operating-system secret storage.
