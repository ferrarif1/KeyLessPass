# Review revision checklist

## Closed in the artifact and manuscript

- Added MFDPG, MFKDF, and the USENIX 2024 MFKDF cryptanalysis to related work
  and replaced generic comparison categories with cited nearest neighbors.
- Implemented CDR derivation version 2 with an RFC 8785 input, 128-bit salt
  validation, explicit included/excluded fields, policy hash, vault subkey, and
  fixed end-to-end vector.
- Replaced the uniform-output claim with the narrower claim of
  modulo-bias-free bounded sampling.
- Specified manifest-held full-length KCV and post-reconstruction envelope MAC
  verification order.
- Added all-pair recovery and single-damaged-factor diagnosis when three shares
  are available.
- Reframed the 108-word object as an offline paper recovery share, not a phrase
  to memorize.
- Added the both-passwords-valid rotation outcome and an explicit target adapter
  contract.
- Added a persistent SQLite compare-and-set freshness prototype and restart
  test.
- Added a TLA+ rotation model; TLC exhausted 16 reachable states with no checked
  invariant violation.
- Added mean, median, P95, and standard deviation to per-iteration experiments
  and regenerated the full macOS result from the actual derivation path.
- Reorganized the manuscript into ten sections and removed revision-history and
  self-doubting contribution language.

## Still open and not claimed as complete

- Two production target adapters with real remote password changes.
- Systematic crash/network fault injection at every persistence boundary.
- Windows and Linux quantitative runs.
- Human-subject recovery usability and a QR workflow.
- Complete desktop enrollment/recovery UI for the paper-share lifecycle.
- Coordinated Root-Key rotation for a non-empty vault.
- Independent security audit and production freshness service.
