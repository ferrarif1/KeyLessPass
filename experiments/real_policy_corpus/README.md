# Public real-policy corpus

Source: Anuj Gautam, Shan Lalani, and Scott Ruoti, “Improving Password Generation Through the Design of a Password Composition Policy Description Language,” SOUPS 2022.

The source authors' public workbook contains 270 historical website policies. It is a reproducibility corpus, not a claim about current live enterprise policies.

## Reproduction

```sh
curl -L https://userlab.utk.edu/files/data/ruoti/2022/gautam2022improving.zip \
  -o /tmp/gautam2022improving.zip
unzip -q /tmp/gautam2022improving.zip -d /tmp/gautam2022improving
python3 experiments/scripts/import_soups2022_pcp.py \
  "/tmp/gautam2022improving/pcp dataset/clean_data.xlsx" \
  experiments/real_policy_corpus/translated_corpus.json
cargo build --release --manifest-path rust_core/Cargo.toml \
  --example compile_policy_worker
python3 experiments/real_policy_corpus/run_full_corpus.py
```

The importer records the workbook SHA-256, original JSON policy, exact translated policy, source row, website, and every rejection reason. Unsupported semantics are rejected rather than approximated.

## Status classes

- `translated` means every recognized source rule has a direct Policy IR representation.
- `rejected` means at least one semantic rule cannot be represented exactly.
- Every `translated` row is passed to the compiler. There is no length prefilter.
- `SUCCESS`, `STATE_LIMIT`, `MEMORY_LIMIT`, `TIME_LIMIT`, `EMPTY_LANGUAGE`,
  and `INTERNAL_ERROR` describe the worker outcome under the fixed per-policy
  budget recorded in `policy_metrics.json`.

The 32-character ceiling is an evaluation resource bound. EPSCD scheme version 1 separately caps policy length at 128.
