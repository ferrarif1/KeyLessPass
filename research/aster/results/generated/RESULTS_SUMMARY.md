# ASTER results summary

- RQ1: 121 exact translations compiled; 97 completed 9,700,000 derivations with 0 policy violations, 0 same-lineage duplicates, and 0 replay mismatches. 24 policies exceeded the configured 512-bit FF1 backend ceiling and failed closed.
- RQ2: exact capabilities had zero spill for q=1,2,4,8,16,32 over 32 contexts; all eight single-check negative controls admitted a concrete witness.
- RQ3: the controlled endpoint inventory contains no Root-Epoch or reusable lineage key; the intended exact-scope configuration exposed 0/1/0 outputs before/during/after the output window without new approval in the 32-context harness.
- RQ4: share refresh preserved all sampled outputs. Independent Root-Epoch replacement reduced old-root exposure from 100 to 0 in direct proportion to 0/10/25/50/75/100 conclusive migrations.
- RQ5: 96 traces across 16 scenarios, two adapter semantics, and 3 repetitions had 0 commit and 0 uncertainty-preservation violations. A pinned real OpenLDAP modify/bind smoke test also passed.
- TLA+: 777 distinct positive states at depth 11 with no invariant violation; all eight negative controls produced counterexamples.
- RQ7: SQLite public metadata and replay state reached 100,000 paired records; see the raw file for storage and percentile lookup measurements.
- RQ6 MPC: three- and five-party fixed vectors agree; loopback feasibility samples are complete.
