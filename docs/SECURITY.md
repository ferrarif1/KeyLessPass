# Technical Security Notes

The normative security boundary is [../SECURITY.md](../SECURITY.md). The v3 design uses authenticated and generation-bound Shamir 2-of-3 shares; legacy `W_MC`, `W_MU`, and `W_CU` packages are migration-only v2 data and are not secret shares.

See:

- [THREAT_MODEL.md](THREAT_MODEL.md) for assets, assumptions, attacker capabilities, and residual risk;
- [KEY_HIERARCHY.md](KEY_HIERARCHY.md) for Root-Key subkeys and domain separation;
- [RECOVERY_DESIGN_REVIEW.md](RECOVERY_DESIGN_REVIEW.md) and [adr/ADR-001-AUTHENTICATED-SHAMIR-RECOVERY.md](adr/ADR-001-AUTHENTICATED-SHAMIR-RECOVERY.md) for the recovery decision;
- [ROTATION_PROTOCOL.md](ROTATION_PROTOCOL.md) for uncertain-result handling;
- [SYNC_AND_ROLLBACK.md](SYNC_AND_ROLLBACK.md) for the local-only versus enterprise-anchored claim boundary;
- [LIMITATIONS.md](LIMITATIONS.md) for features that are not implemented or not claimed.
