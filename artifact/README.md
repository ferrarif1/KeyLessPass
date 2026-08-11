# EPSCD Artifact

This directory is the machine-readable reproducibility artifact for exact
policy-space credential derivation and failure-safe legacy password rotation.
The manuscript and rendered publication files are intentionally outside the
versioned artifact boundary.

The primary implementation is the Rust crate in `rust_core/`. `artifact/results/` contains machine-readable outputs regenerated from the public policy corpus, exact policy/compiler evaluation, controlled rejection-density experiment, 100,000-generation sequence run, adapter fault matrix, and TLA+ model checking.

The artifact does not use unpublished manuscripts or an unpublished local
encoder as comparison baselines. Its only implemented algorithmic baseline is
the exact-arithmetic reproduction of Oudinet, Denise, and Gaudel's published
Dichopile algorithm (TCS 2013), and the result tables label the different
purpose of that generator.

The HTTP and LDAP-style rotation adapters are high-fidelity local semantic models with a durable SQLite journal. They are not production deployments or a real OpenLDAP benchmark.
