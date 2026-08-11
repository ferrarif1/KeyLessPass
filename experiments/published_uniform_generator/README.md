# Published regular-language uniform-generation baseline

This baseline reproduces Algorithm 1 (“Dichopile”) from:

> Johan Oudinet, Alain Denise, and Marie-Claude Gaudel. “A new dichotomic algorithm for the uniform random generation of words in regular languages.” *Theoretical Computer Science* 502 (2013), 165-176. DOI: 10.1016/j.tcs.2012.07.025.

The complete 23-page journal manuscript was read before implementation. Source copy: HAL `hal-00716558`, SHA-256 `b65b356fd60ffdb733b130377f451de95beb4cacc80049f2c01d87fc5286e4a0`.

## Mapping to the paper

- `L_0` is the vector of accepting-state indicators.
- `advance` is Formula (1), computing `L_j` from `L_{j-1}`.
- the stack, midpoint rule, pop rule, and descending `L_n ... L_0` schedule follow Algorithm 1;
- transition selection uses Formula (2), weighting each outgoing transition by the number of accepting completions;
- multiple accepted lengths are handled outside Algorithm 1 by selecting a length proportional to its exact accepted-word count.

The paper's main large-automata experiments use floating-point counts and describe the resulting generator as quasi-uniform. This reproduction intentionally uses arbitrary-precision `BigUint` counts and exact integer rejection sampling. It is therefore an exact-arithmetic instantiation of the published algorithm, not a reproduction of the paper's floating-point performance numbers.

## Claim boundary

Uniform random generation from regular languages is prior art. The baseline draws independent random words with replacement. EPSCD's additional studied property is a keyed deterministic generation sequence without replacement, plus credential context, policy epoch, and history semantics.

Implementation: `rust_core/src/published_baselines/dichopile.rs`.
