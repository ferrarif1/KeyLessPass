# New-paper scope

This directory contains the standalone paper on exact policy-space derivation
and factor-preserving Root-Key recovery. Its scope covers policy compilation,
exact counting, ranking/unranking, finite-domain generation sequences,
credential exposure and rekeying, heterogeneous recovery, multidimensional
freshness, and their evaluation.

The executable published baseline is the exact-arithmetic reproduction of
Oudinet, Denise, and Gaudel's TCS 2013 Dichopile algorithm documented under
`experiments/published_uniform_generator/`. The nearest deterministic-system
comparisons cite peer-reviewed published work. Unpublished local encoders,
prior manuscripts, and preprint artifacts are excluded from the paper and its
experimental comparisons.
Synthetic policies and mathematical controls are used only for stress testing,
mechanistic explanation, and statistical sanity; they are not presented as
competing published systems.

Build from this directory so relative table, figure, and bibliography paths
resolve:

```text
latexmk -pdf -interaction=nonstopmode -halt-on-error \
  -outdir=../output/pdf \
  -jobname=exact_policy_space_credential_derivation manuscript.tex
```

The generated review PDF is
`../output/pdf/exact_policy_space_credential_derivation.pdf`.
