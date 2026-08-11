# Factor-Preserving Network Recovery Profile

## Scope

The optional `peer-recovery` feature implements a research profile for
recovering the network member of a top-level Shamir 2-of-3 Root-Key share set:

```text
(S_D, S_U, S_N) <- ShamirSplit(2, 3, K_root)
```

`S_D` is held by the managed endpoint, `S_U` by an offline/removable factor,
and `S_N` by independently administered recovery infrastructure. The profile
does not use a View Key, Data Key, OPRF, opaque-object scan, or erasure-coded
ciphertext. Those mechanisms belong to a superseded experiment and are not in
the active implementation or paper.

## Deployment property

For a protected compromise domain `X`, let `Closure(X)` include files, secure
storage, credentials, cookies, request-signing authority, automatic protocol
calls, and all responses honest services release to those capabilities. The
logical threshold is factor-preserving only when:

```text
|Closure(X) intersect {S_D, S_U, S_N}| < 2
```

for every single protected domain. Giving endpoint `D` both `S_D` and an
unattended API that releases `S_N` violates this condition even though the
stored shares are algebraically 2-of-3.

## Implemented protocol

- The canonical authenticated `S_N` `ShareEnvelope` is itself split into five
  Shamir fragments with threshold three.
- A recovery ticket binds `vaultID`, `rootGeneration`, `shareSetID`,
  `shareSetGeneration`, a random `opID`, an ephemeral X25519 public key,
  issue/expiry times, purpose, and authorized node identifiers.
- At least two distinct independently administered approvers sign the complete
  ticket with Ed25519. A request credential possessed by `D` is not approval
  authority.
- Each authorized node verifies the ticket, generations, expiry, approvals,
  freshness, node scope, and replay ledger before releasing one fragment.
- The fragment is encrypted to the ticket's ephemeral X25519 session key with
  HKDF-SHA256 and AES-GCM; the ticket hash and node identity are associated
  data.
- `(nodeID, opID, ticketHash)` is idempotent. Reusing an operation identifier
  with different ticket contents is rejected.
- Any three valid responses reconstruct `S_N`; the caller still needs `S_D` or
  `S_U` and validates the top-level KCV and envelope MAC before accepting the
  Root Key.

## Freshness and replacement

Nodes reject stale Root-Key, share-set identity, or share-set-generation
claims. Ordinary re-sharing retains `K_root` and `rootGeneration` but advances
`shareSetGeneration`. Recovery followed by suspected threshold compromise
requires a new Root Key and advances both generations. Re-sharing alone cannot
revoke two already compromised old top-level shares.

## Evidence and limits

The artifact tests every 3-of-5 fragment combination; stale tickets, mixed
generations, insufficient or duplicate approvals, wrong-node use, replay,
ciphertext tampering, and incomplete closures. A bounded TLA+ model checks
authorization, freshness, and non-collapse invariants.

The benchmark is a local cryptographic baseline. It excludes network RTT,
human approval delay, hardware-backed signing, transport failures, and
multi-region consistency. The profile is not enabled in the desktop product
path and does not claim a new Shamir scheme, production recovery service,
anonymity mechanism, or Byzantine node tolerance.
