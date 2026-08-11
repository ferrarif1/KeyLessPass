# Published verified-password-generator baseline status

Published source:

> Miguel Grilo, Joao Campos, Joao F. Ferreira, Jose Bacelar Almeida, and Alexandra Mendes. “Verified Password Generation from Password Composition Policies.” iFM 2022, pp. 271-288.

Official artifact: `https://github.com/passcert-project/random-password-generator`
Pinned commit: `ceeb8988f87b0ac4b6826fc20af4f8acafb3c841` (2022-01-23).

The paper and artifact establish an important prior-art boundary: policy-compliant random password generation and a uniform-distribution security specification are not EPSCD contributions.

## Reproduction status

The repository contains EasyCrypt specifications, Jasmin source, and a C harness, but no generated assembly. Its Makefile requires a sibling checkout of the Jasmin compiler and does not pin a Jasmin or EasyCrypt revision. The current environment therefore cannot reproduce a byte-identical executable without choosing unrecorded toolchain versions. No invented port or substitute implementation is used as a benchmark.

Decision:

- include the work in the published feature/property comparison;
- cite its verified compliance and uniform-sampling objectives;
- do not report latency or distribution numbers as an official-artifact reproduction;
- use the published Oudinet Algorithm 1 exact-arithmetic reproduction as the executable published uniform-language baseline.

This fail-closed decision avoids presenting a reimplementation as if it were the authors' measured artifact.
