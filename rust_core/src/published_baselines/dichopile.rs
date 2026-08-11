//! Exact-arithmetic reproduction of Oudinet, Denise, and Gaudel's
//! Dichopile algorithm (Theoretical Computer Science 502, 2013).
//!
//! Their Algorithm 1 maintains a logarithmic stack of path-count vectors and
//! recomputes intermediate vectors while drawing a fixed-length path. This
//! reproduction uses `BigUint` rather than the floating-point optimization
//! used by the paper's main experiments, so transition selection is exact.

use crate::error::{KeylessPassError, Result};
use crate::policy::PolicyDfa;
use num_bigint::BigUint;
use num_traits::{One, Zero};
use rand::RngCore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DichopileSample {
    pub word: String,
    pub selected_length: usize,
    pub recurrence_calls: usize,
    pub peak_stack_vectors: usize,
    pub peak_stack_payload_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct ExactDichopile<'a> {
    dfa: &'a PolicyDfa,
    length_counts: Vec<(usize, BigUint)>,
    total_count: BigUint,
    length_preprocessing_calls: usize,
}

impl<'a> ExactDichopile<'a> {
    pub fn new(dfa: &'a PolicyDfa) -> Result<Self> {
        let mut length_counts = Vec::new();
        let mut total_count = BigUint::zero();
        let mut current = accepting_vector(dfa);
        let mut length_preprocessing_calls = 0;
        for length in 0..=dfa.max_length() {
            if length >= dfa.min_length() {
                let count = current[dfa.start_state()].clone();
                if !count.is_zero() {
                    total_count += &count;
                    length_counts.push((length, count));
                }
            }
            if length < dfa.max_length() {
                current = advance(dfa, &current);
                length_preprocessing_calls += 1;
            }
        }
        if total_count.is_zero() {
            return Err(validation("Dichopile input language is empty"));
        }
        Ok(Self {
            dfa,
            length_counts,
            total_count,
            length_preprocessing_calls,
        })
    }

    pub fn total_count(&self) -> &BigUint {
        &self.total_count
    }

    pub fn length_preprocessing_calls(&self) -> usize {
        self.length_preprocessing_calls
    }

    /// Selects a length in proportion to its accepted-word count, then runs
    /// published Algorithm 1 for that exact length.
    pub fn generate<R: RngCore>(&self, rng: &mut R) -> Result<DichopileSample> {
        let mut draw = uniform_below(&self.total_count, rng);
        let mut length = None;
        for (candidate, count) in &self.length_counts {
            if &draw < count {
                length = Some(*candidate);
                break;
            }
            draw -= count;
        }
        generate_fixed_length(self.dfa, length.expect("draw is below total"), rng)
    }
}

/// Exact-arithmetic transcription of Algorithm 1 for one fixed path length.
pub fn generate_fixed_length<R: RngCore>(
    dfa: &PolicyDfa,
    length: usize,
    rng: &mut R,
) -> Result<DichopileSample> {
    if length < dfa.min_length() || length > dfa.max_length() {
        return Err(validation("requested length is outside the DFA range"));
    }

    let l_zero = accepting_vector(dfa);
    let mut stack = vec![(0_usize, l_zero)];
    let mut peak_stack_vectors = stack.len();
    let mut peak_stack_payload_bytes = stack_payload_bytes(&stack);
    let mut previous = vec![BigUint::zero(); dfa.state_count()];
    let mut recurrence_calls = 0;
    let mut state = dfa.start_state();
    let mut word = String::with_capacity(length);

    for emitted in 0..=length {
        let target = length - emitted;
        while stack.last().is_some_and(|(index, _)| *index > target) {
            stack.pop();
        }
        let (mut index, mut current) = stack
            .last()
            .cloned()
            .ok_or_else(|| validation("Dichopile stack invariant failed"))?;

        while index < target.saturating_sub(1) {
            let midpoint = (index + target) / 2;
            while index < midpoint {
                current = advance(dfa, &current);
                index += 1;
                recurrence_calls += 1;
            }
            stack.push((index, current.clone()));
            peak_stack_vectors = peak_stack_vectors.max(stack.len());
            peak_stack_payload_bytes = peak_stack_payload_bytes.max(stack_payload_bytes(&stack));
        }
        if index == target.saturating_sub(1) && target > 0 {
            current = advance(dfa, &current);
            recurrence_calls += 1;
        }

        if emitted > 0 {
            let expected_total = &previous[state];
            if expected_total.is_zero() {
                return Err(validation("requested length has no accepting path"));
            }
            let mut choice = uniform_below(expected_total, rng);
            let mut selected = None;
            for (alphabet_index, character) in dfa.alphabet().iter().copied().enumerate() {
                let Some(next) = dfa.transition(state, alphabet_index) else {
                    continue;
                };
                let weight = &current[next];
                if &choice < weight {
                    selected = Some((character, next));
                    break;
                }
                choice -= weight;
            }
            let (character, next) = selected
                .ok_or_else(|| validation("Dichopile transition weights are inconsistent"))?;
            word.push(character);
            state = next;
        }
        previous = current;
    }

    if word.chars().count() != length || !dfa.is_accepting(state) {
        return Err(validation("Dichopile produced a non-accepting path"));
    }
    Ok(DichopileSample {
        word,
        selected_length: length,
        recurrence_calls,
        peak_stack_vectors,
        peak_stack_payload_bytes,
    })
}

fn accepting_vector(dfa: &PolicyDfa) -> Vec<BigUint> {
    (0..dfa.state_count())
        .map(|state| BigUint::from(dfa.is_accepting(state)))
        .collect()
}

fn advance(dfa: &PolicyDfa, previous: &[BigUint]) -> Vec<BigUint> {
    (0..dfa.state_count())
        .map(|state| {
            (0..dfa.alphabet().len())
                .filter_map(|alphabet_index| dfa.transition(state, alphabet_index))
                .fold(BigUint::zero(), |sum, next| sum + &previous[next])
        })
        .collect()
}

fn uniform_below<R: RngCore>(upper: &BigUint, rng: &mut R) -> BigUint {
    debug_assert!(!upper.is_zero());
    if upper == &BigUint::one() {
        return BigUint::zero();
    }
    let bit_length = (upper - BigUint::one()).bits() as usize;
    let byte_length = (bit_length + 7) / 8;
    let excess_bits = byte_length * 8 - bit_length;
    loop {
        let mut bytes = vec![0_u8; byte_length];
        rng.fill_bytes(&mut bytes);
        if excess_bits > 0 {
            bytes[0] &= 0xff_u8 >> excess_bits;
        }
        let candidate = BigUint::from_bytes_be(&bytes);
        if &candidate < upper {
            return candidate;
        }
    }
}

fn stack_payload_bytes(stack: &[(usize, Vec<BigUint>)]) -> usize {
    stack
        .iter()
        .flat_map(|(_, vector)| vector)
        .map(|value| value.to_bytes_be().len())
        .sum()
}

fn validation(message: &str) -> KeylessPassError {
    KeylessPassError::Validation(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{CompiledPolicy, PolicySpec};
    use rand::{rngs::StdRng, SeedableRng};
    use std::collections::HashMap;

    fn language() -> CompiledPolicy {
        let alphabet = "ab0123456789".chars().collect::<Vec<_>>();
        let mut transitions = vec![vec![None; alphabet.len()]; 4];
        transitions[0][0] = Some(1);
        transitions[0][1] = Some(2);
        for index in 2..alphabet.len() {
            transitions[2][index] = Some(3);
        }
        let dfa = PolicyDfa::new(
            alphabet,
            transitions,
            0,
            vec![false, true, false, true],
            1,
            2,
        )
        .unwrap();
        CompiledPolicy::from_dfa(
            PolicySpec {
                policy_ir_version: 1,
                min_length: 1,
                max_length: 2,
                alphabet: "ab0123456789".to_string(),
                forbidden_characters: String::new(),
                classes: Vec::new(),
                fixed_characters: Vec::new(),
                fixed_prefix: String::new(),
                fixed_suffix: String::new(),
                forbidden_first_characters: String::new(),
                forbidden_last_characters: String::new(),
                max_total_per_character: None,
                max_identical_run: None,
                max_sequential_run: None,
                forbidden_substrings: Vec::new(),
            },
            dfa,
        )
        .unwrap()
    }

    #[test]
    fn exact_dichopile_covers_the_enumerable_language_nearly_uniformly() {
        let policy = language();
        let generator = ExactDichopile::new(policy.dfa()).unwrap();
        assert_eq!(generator.total_count(), &BigUint::from(11_u8));
        let mut rng = StdRng::seed_from_u64(0x4449_4348_4f50_494c);
        let mut counts = HashMap::<String, usize>::new();
        for _ in 0..22_000 {
            let sample = generator.generate(&mut rng).unwrap();
            assert!(policy.accepts(&sample.word));
            *counts.entry(sample.word).or_default() += 1;
        }
        assert_eq!(counts.len(), 11);
        assert!(counts.values().all(|count| (1_800..2_200).contains(count)));
    }
}
