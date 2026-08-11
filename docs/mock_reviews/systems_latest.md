# Hostile systems review — latest manuscript

## Recommendation

**Weak reject, potentially major revision.** The artifact is unusually candid
and well structured, but several measurements validate local code rather than
the deployed failure modes that motivate the paper.

## Strengths

- 120 published-corpus policies complete under one declared resource budget;
  exact counts and failure reasons are retained.
- The experimental comparison uses a published uniform-language algorithm and
  avoids unpublished local baselines.
- Eighty-five Rust tests and three bounded TLA+ models cover meaningful negative
  paths across rotation, recovery, rollback, and compromise.
- Recovery generations and credential freshness are represented in executable
  state rather than prose alone.

## Major concerns

- The recovery path has no network transport. The reported 6.036 ms omits RTT,
  retries, partitions, durable replay state, and human approval.
- No target-service adapter demonstrates that the credential sequence and
  evidence-bounded rotation work against a real password-only service.
- No crash/fault injection spans actual database and network boundaries.
- The freshness service is a local SQLite CAS prototype, not an independently
  administered rollback anchor.
- No Windows/Linux measurements or recovery usability study are provided.

## Required revision

Deploy five recovery nodes and two independent approvers across at least two
administrative failure domains; report median/P95 end-to-end recovery, expired
and replayed requests, node loss, partitions, durable restart, and wrong-node
responses. Add at least one real target adapter with crash injection. Without
that evidence, retain the research-prototype framing and avoid production or
enterprise-readiness claims.
