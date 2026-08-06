//! Policy-Space Permutation Password Derivation (PSPPD) research prototype.
//!
//! The production Encoder v2 does not call this module. The prototype starts from a
//! canonical DFA because regex parsing and DFA construction are not research contributions.

use aes::Aes256;
use fpe::ff1::{FlexibleNumeralString, FF1};
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};
use std::collections::HashSet;
use thiserror::Error;

pub const MIN_FF1_DOMAIN_SIZE: u64 = 1_000_000;
pub const DEFAULT_MAX_CYCLE_WALKS: u32 = 1_024;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PsppdError {
    #[error("invalid policy: {0}")]
    InvalidPolicy(String),
    #[error("rank is outside the policy space")]
    RankOutOfRange,
    #[error("password is not in the policy language")]
    PasswordNotAccepted,
    #[error("FF1 requires a policy domain of at least {MIN_FF1_DOMAIN_SIZE}")]
    DomainTooSmallForFf1,
    #[error("FF1 cycle walking exceeded {0} iterations")]
    CycleWalkLimit(u32),
    #[error("FF1 operation failed: {0}")]
    Ff1(String),
}

pub type Result<T> = std::result::Result<T, PsppdError>;

/// A canonical, partial DFA. Alphabet order defines lexicographic rank order.
#[derive(Debug, Clone)]
pub struct PolicyDfa {
    alphabet: Vec<char>,
    transitions: Vec<Vec<Option<usize>>>,
    start_state: usize,
    accepting: Vec<bool>,
    min_length: usize,
    max_length: usize,
}

impl PolicyDfa {
    pub fn new(
        alphabet: Vec<char>,
        transitions: Vec<Vec<Option<usize>>>,
        start_state: usize,
        accepting: Vec<bool>,
        min_length: usize,
        max_length: usize,
    ) -> Result<Self> {
        if alphabet.is_empty() {
            return Err(PsppdError::InvalidPolicy(
                "alphabet must not be empty".to_string(),
            ));
        }
        let mut unique = HashSet::new();
        if !alphabet.iter().all(|character| unique.insert(*character)) {
            return Err(PsppdError::InvalidPolicy(
                "alphabet characters must be unique".to_string(),
            ));
        }
        if transitions.is_empty() || accepting.len() != transitions.len() {
            return Err(PsppdError::InvalidPolicy(
                "transition and accepting-state tables must have equal non-zero size".to_string(),
            ));
        }
        if start_state >= transitions.len() {
            return Err(PsppdError::InvalidPolicy(
                "start state is outside the transition table".to_string(),
            ));
        }
        if min_length > max_length {
            return Err(PsppdError::InvalidPolicy(
                "minimum length exceeds maximum length".to_string(),
            ));
        }
        for row in &transitions {
            if row.len() != alphabet.len() {
                return Err(PsppdError::InvalidPolicy(
                    "every transition row must match alphabet size".to_string(),
                ));
            }
            if row
                .iter()
                .flatten()
                .any(|target| *target >= transitions.len())
            {
                return Err(PsppdError::InvalidPolicy(
                    "transition target is outside the state table".to_string(),
                ));
            }
        }
        Ok(Self {
            alphabet,
            transitions,
            start_state,
            accepting,
            min_length,
            max_length,
        })
    }

    pub fn alphabet(&self) -> &[char] {
        &self.alphabet
    }

    pub fn state_count(&self) -> usize {
        self.transitions.len()
    }

    pub fn min_length(&self) -> usize {
        self.min_length
    }

    pub fn max_length(&self) -> usize {
        self.max_length
    }

    pub fn start_state(&self) -> usize {
        self.start_state
    }

    pub fn is_accepting(&self, state: usize) -> bool {
        self.accepting.get(state).copied().unwrap_or(false)
    }

    pub fn transition(&self, state: usize, alphabet_index: usize) -> Option<usize> {
        self.transitions[state][alphabet_index]
    }
}

#[derive(Debug, Clone)]
struct LengthTable {
    length: usize,
    /// counts[position][state]
    counts: Vec<Vec<BigUint>>,
}

/// Exact completion counts and canonical rank/unrank for one finite policy language.
#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    dfa: PolicyDfa,
    lengths: Vec<LengthTable>,
    total: BigUint,
}

impl CompiledPolicy {
    pub fn compile(dfa: PolicyDfa) -> Result<Self> {
        let mut lengths = Vec::new();
        let mut total = BigUint::zero();
        for length in dfa.min_length..=dfa.max_length {
            let mut counts = vec![vec![BigUint::zero(); dfa.state_count()]; length + 1];
            for (state, accepting) in dfa.accepting.iter().copied().enumerate() {
                if accepting {
                    counts[length][state] = BigUint::one();
                }
            }
            for position in (0..length).rev() {
                for state in 0..dfa.state_count() {
                    let mut count = BigUint::zero();
                    for alphabet_index in 0..dfa.alphabet.len() {
                        if let Some(next_state) = dfa.transition(state, alphabet_index) {
                            count += &counts[position + 1][next_state];
                        }
                    }
                    counts[position][state] = count;
                }
            }
            total += &counts[0][dfa.start_state];
            lengths.push(LengthTable { length, counts });
        }
        if total.is_zero() {
            return Err(PsppdError::InvalidPolicy(
                "policy language is empty in the configured length range".to_string(),
            ));
        }
        Ok(Self {
            dfa,
            lengths,
            total,
        })
    }

    pub fn dfa(&self) -> &PolicyDfa {
        &self.dfa
    }

    pub fn total_count(&self) -> &BigUint {
        &self.total
    }

    pub fn count_for_length(&self, length: usize) -> Option<&BigUint> {
        self.lengths
            .iter()
            .find(|table| table.length == length)
            .map(|table| &table.counts[0][self.dfa.start_state])
    }

    pub fn table_cell_count(&self) -> usize {
        self.lengths
            .iter()
            .map(|table| table.counts.len() * self.dfa.state_count())
            .sum()
    }

    /// Returns log2 of the exact integer space size, rounded only for display.
    pub fn entropy_bits(&self) -> f64 {
        let bits = self.total.bits();
        if bits <= 53 {
            return self.total.to_f64().unwrap().log2();
        }
        let shift = bits - 53;
        let top = (&self.total >> shift).to_f64().unwrap();
        top.log2() + shift as f64
    }

    pub fn unrank(&self, rank: &BigUint) -> Result<String> {
        if rank >= &self.total {
            return Err(PsppdError::RankOutOfRange);
        }
        let mut residual = rank.clone();
        let mut selected = None;
        for table in &self.lengths {
            let count = &table.counts[0][self.dfa.start_state];
            if &residual < count {
                selected = Some(table);
                break;
            }
            residual -= count;
        }
        let table = selected.expect("rank was checked against total count");
        let mut state = self.dfa.start_state;
        let mut password = String::with_capacity(table.length);
        for position in 0..table.length {
            let mut chosen = None;
            for (alphabet_index, character) in self.dfa.alphabet.iter().copied().enumerate() {
                let Some(next_state) = self.dfa.transition(state, alphabet_index) else {
                    continue;
                };
                let completions = &table.counts[position + 1][next_state];
                if &residual < completions {
                    chosen = Some((character, next_state));
                    break;
                }
                residual -= completions;
            }
            let (character, next_state) = chosen.expect("positive completion interval exists");
            password.push(character);
            state = next_state;
        }
        debug_assert!(self.dfa.accepting[state]);
        Ok(password)
    }

    pub fn rank(&self, password: &str) -> Result<BigUint> {
        let characters: Vec<char> = password.chars().collect();
        let table = self
            .lengths
            .iter()
            .find(|table| table.length == characters.len())
            .ok_or(PsppdError::PasswordNotAccepted)?;
        let mut rank = BigUint::zero();
        for earlier in self
            .lengths
            .iter()
            .take_while(|earlier| earlier.length < table.length)
        {
            rank += &earlier.counts[0][self.dfa.start_state];
        }
        let mut state = self.dfa.start_state;
        for (position, character) in characters.iter().copied().enumerate() {
            let actual_index = self
                .dfa
                .alphabet
                .iter()
                .position(|candidate| *candidate == character)
                .ok_or(PsppdError::PasswordNotAccepted)?;
            for alphabet_index in 0..actual_index {
                if let Some(next_state) = self.dfa.transition(state, alphabet_index) {
                    rank += &table.counts[position + 1][next_state];
                }
            }
            state = self
                .dfa
                .transition(state, actual_index)
                .ok_or(PsppdError::PasswordNotAccepted)?;
        }
        if !self.dfa.accepting[state] {
            return Err(PsppdError::PasswordNotAccepted);
        }
        Ok(rank)
    }

    pub fn accepts(&self, password: &str) -> bool {
        self.rank(password).is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedPassword {
    pub password: String,
    pub rank: BigUint,
    pub cycle_walks: u32,
}

pub fn derive_password_ff1(
    policy: &CompiledPolicy,
    key: &[u8; 32],
    tweak: &[u8],
    generation: &BigUint,
) -> Result<DerivedPassword> {
    let (rank, cycle_walks) = permute_rank_ff1(
        key,
        tweak,
        policy.total_count(),
        generation,
        DEFAULT_MAX_CYCLE_WALKS,
    )?;
    Ok(DerivedPassword {
        password: policy.unrank(&rank)?,
        rank,
        cycle_walks,
    })
}

/// Cycle-walks FF1-AES-256 from a power-of-two superset back into `[0, domain_size)`.
pub fn permute_rank_ff1(
    key: &[u8; 32],
    tweak: &[u8],
    domain_size: &BigUint,
    input: &BigUint,
    max_walks: u32,
) -> Result<(BigUint, u32)> {
    cycle_walk_ff1(key, tweak, domain_size, input, max_walks, false)
}

pub fn invert_rank_ff1(
    key: &[u8; 32],
    tweak: &[u8],
    domain_size: &BigUint,
    input: &BigUint,
    max_walks: u32,
) -> Result<(BigUint, u32)> {
    cycle_walk_ff1(key, tweak, domain_size, input, max_walks, true)
}

fn cycle_walk_ff1(
    key: &[u8; 32],
    tweak: &[u8],
    domain_size: &BigUint,
    input: &BigUint,
    max_walks: u32,
    decrypt: bool,
) -> Result<(BigUint, u32)> {
    if domain_size < &BigUint::from(MIN_FF1_DOMAIN_SIZE) {
        return Err(PsppdError::DomainTooSmallForFf1);
    }
    if input >= domain_size {
        return Err(PsppdError::RankOutOfRange);
    }
    if max_walks == 0 {
        return Err(PsppdError::CycleWalkLimit(max_walks));
    }
    let bit_length = (domain_size - BigUint::one()).bits() as usize;
    let ff1 = FF1::<Aes256>::new(key, 2).map_err(|error| PsppdError::Ff1(error.to_string()))?;
    let mut current = input.clone();
    for walk in 1..=max_walks {
        let numeral = FlexibleNumeralString::from(to_fixed_bits(&current, bit_length));
        let transformed = if decrypt {
            ff1.decrypt(tweak, &numeral)
        } else {
            ff1.encrypt(tweak, &numeral)
        }
        .map_err(|error| PsppdError::Ff1(error.to_string()))?;
        current = from_bits(Vec::<u16>::from(transformed).as_slice());
        if &current < domain_size {
            return Ok((current, walk));
        }
    }
    Err(PsppdError::CycleWalkLimit(max_walks))
}

fn to_fixed_bits(value: &BigUint, bit_length: usize) -> Vec<u16> {
    (0..bit_length)
        .rev()
        .map(|bit| ((value >> bit) & BigUint::one()).to_u16().unwrap())
        .collect()
}

fn from_bits(bits: &[u16]) -> BigUint {
    bits.iter().fold(BigUint::zero(), |value, bit| {
        (value << 1_usize) + BigUint::from(*bit)
    })
}

/// Builds a fixed-length DFA whose state is the bitset of required classes already seen.
pub fn required_class_policy(
    alphabet: Vec<char>,
    length: usize,
    required_classes: &[Vec<char>],
) -> Result<PolicyDfa> {
    if required_classes.len() > 20 {
        return Err(PsppdError::InvalidPolicy(
            "prototype limits required-class products to 20 classes".to_string(),
        ));
    }
    let alphabet_set: HashSet<char> = alphabet.iter().copied().collect();
    if required_classes
        .iter()
        .any(|class| class.is_empty() || class.iter().all(|ch| !alphabet_set.contains(ch)))
    {
        return Err(PsppdError::InvalidPolicy(
            "each required class must contain an alphabet character".to_string(),
        ));
    }
    let state_count = 1_usize << required_classes.len();
    let mut transitions = vec![vec![None; alphabet.len()]; state_count];
    for (state, row) in transitions.iter_mut().enumerate() {
        for (alphabet_index, character) in alphabet.iter().copied().enumerate() {
            let mut next_state = state;
            for (class_index, class) in required_classes.iter().enumerate() {
                if class.contains(&character) {
                    next_state |= 1_usize << class_index;
                }
            }
            row[alphabet_index] = Some(next_state);
        }
    }
    let mut accepting = vec![false; state_count];
    accepting[state_count - 1] = true;
    PolicyDfa::new(alphabet, transitions, 0, accepting, length, length)
}

/// The illustrative language `{a, b0, ..., b9}` from the design note.
pub fn biased_walk_example_policy() -> Result<PolicyDfa> {
    let alphabet: Vec<char> = "ab0123456789".chars().collect();
    let mut transitions = vec![vec![None; alphabet.len()]; 4];
    transitions[0][0] = Some(1); // a
    transitions[0][1] = Some(2); // b
    for alphabet_index in 2..alphabet.len() {
        transitions[2][alphabet_index] = Some(3);
    }
    PolicyDfa::new(
        alphabet,
        transitions,
        0,
        vec![false, true, false, true],
        1,
        2,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_count_and_bijection_hold_for_biased_walk_example() {
        let policy = CompiledPolicy::compile(biased_walk_example_policy().unwrap()).unwrap();
        assert_eq!(policy.total_count(), &BigUint::from(11_u8));
        let expected = [
            "a", "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9",
        ];
        for (rank, password) in expected.iter().enumerate() {
            let rank = BigUint::from(rank);
            assert_eq!(policy.unrank(&rank).unwrap(), *password);
            assert_eq!(policy.rank(password).unwrap(), rank);
        }
    }

    #[test]
    fn class_policy_count_matches_inclusion_exclusion() {
        let dfa = required_class_policy(
            vec!['a', 'b', '0', '1'],
            3,
            &[vec!['a', 'b'], vec!['0', '1']],
        )
        .unwrap();
        let policy = CompiledPolicy::compile(dfa).unwrap();
        // 4^3 - 2^3 - 2^3 = 48 strings contain at least one letter and one digit.
        assert_eq!(policy.total_count(), &BigUint::from(48_u8));
        for rank in 0_u8..48 {
            let rank = BigUint::from(rank);
            let password = policy.unrank(&rank).unwrap();
            assert!(policy.accepts(&password));
            assert_eq!(policy.rank(&password).unwrap(), rank);
        }
    }

    #[test]
    fn ff1_cycle_walk_is_a_permutation_and_decrypts() {
        let domain = BigUint::from(1_000_003_u64);
        let key = [0x42_u8; 32];
        let tweak = b"policy/example/epoch/1";
        let mut outputs = HashSet::new();
        for value in 0_u32..2_000 {
            let input = BigUint::from(value);
            let (encrypted, _) =
                permute_rank_ff1(&key, tweak, &domain, &input, DEFAULT_MAX_CYCLE_WALKS).unwrap();
            assert!(outputs.insert(encrypted.clone()));
            let (decrypted, _) =
                invert_rank_ff1(&key, tweak, &domain, &encrypted, DEFAULT_MAX_CYCLE_WALKS).unwrap();
            assert_eq!(decrypted, input);
        }
    }

    #[test]
    fn ff1_rejects_small_policy_domains() {
        let error = permute_rank_ff1(
            &[0_u8; 32],
            b"tweak",
            &BigUint::from(999_999_u64),
            &BigUint::zero(),
            DEFAULT_MAX_CYCLE_WALKS,
        )
        .unwrap_err();
        assert_eq!(error, PsppdError::DomainTooSmallForFf1);
    }
}
