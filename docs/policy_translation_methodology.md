# SOUPS 2022 PCP Translation Methodology

## Source integrity

The importer consumes `pcp dataset/clean_data.xlsx` from the source authors' public archive. The generated corpus records SHA-256 `2d424c30fa1e4d2e5f0b82a5c67b4214fc2a96574d8b70052994d6f92712a77a` for the workbook used in this evaluation.

## Direct translations

| PCP field | EPSCD Policy IR field | Rule |
|---|---|---|
| `min_length` | `minLength` | copied as an integer; absent becomes 1 per source semantics |
| `max_length` | `maxLength` | must be finite and at most 128 |
| `require` | `classes[]` | each named standard class becomes `minCount=1` |
| `max_consecutive` | `maxIdenticalRun` | copied exactly |
| `prohibited_substrings` | `forbiddenSubstrings` | copied exactly |

The effective alphabet is the explicit union of lowercase, uppercase, decimal digits, printable ASCII symbols, and space used by the importer. Every generated Policy IR is stored alongside the source JSON.

## Fail-closed exclusions

The importer rejects, with a machine-readable reason:

- context-dependent `policy_exclusions`;
- disjunctive `rules`;
- “require a subset of classes” constraints;
- custom character sets;
- per-character-set location or run constraints;
- unbounded maximum length;
- source maximum length above the protocol limit;
- unknown named classes.

No unsupported rule is dropped, weakened, or approximated. A record with any such field is excluded from the “supported” count.

## Resource classification

Exact semantic translation and experiment eligibility are separate. Translated policies with `maxLength > 32` are recorded as `resource-skipped` for the present evaluation. Eligible policies are compiled with the same 250,000-state limit as the public implementation. Compilation failures are reported separately from semantic rejections.

For each compiled policy, the evaluator records reachable states, count-table cells, BigUint payload bytes, exact `N_P`, `log2(N_P)`, compile time, and rank/unrank time for the midpoint rank. Timing is a single-host implementation measurement and does not affect support classification.
