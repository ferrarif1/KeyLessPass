# ADR-001: Authenticated Shamir 2-of-3 Recovery

Status: Accepted  
Date: 2026-08-06

## Context

The v2 system encrypted a complete Root Key once for each authorized factor pair. Reviewers correctly noted that this was not threshold secret sharing. The replacement must fit three passive local factors, be explainable and testable, and avoid new cryptographic mathematics.

## Decision

Use `vsss-rs 5.4.0` Shamir sharing over GF(256) with threshold 2 and count 3. Add KeyLessPass share envelopes, domain-separated HKDF subkeys, envelope HMAC, Root Key Confirmation Value, generation-specific storage, and a manifest-last commit protocol. Protect the managed-computer envelope with the platform provider; treat a standard USB envelope as a copyable possession factor.

## Rejected alternatives

- Pairwise wrappers: not true t-of-n and retain three complete-key ciphertexts.
- Feldman/Pedersen VSS: unnecessary verifier and group state for a trusted local dealer; does not solve freshness.
- Proactive sharing: factors are not online parties and secure erasure assumptions are not verifiable here.
- Threshold MPC: disproportionate and does not prevent password capture on a compromised endpoint.
- Hardware-only recovery: platform fragmentation and device replacement prevent it from being the portable base design.

## Consequences

- Any two current, distinct envelopes reconstruct the Root Key; one share cannot call the library combine API and contains no complete-key ciphertext.
- Recovery temporarily materializes the Root Key in memory.
- New share sets invalidate cross-set mixing, not an already compromised old threshold.
- Full-copy rollback requires an external freshness anchor.
- Migration reads and verifies all three legacy paths before committing v3. The service password does not change during wrapper-to-share migration.
