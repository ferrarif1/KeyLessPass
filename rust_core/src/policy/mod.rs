use crate::error::{KeylessPassError, Result};
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};

pub const DEFAULT_MAX_POLICY_STATES: usize = 250_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CharacterClassConstraint {
    pub name: String,
    pub alphabet: String,
    pub min_count: usize,
    pub max_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FixedCharacterConstraint {
    pub index: usize,
    pub character: char,
}

/// Canonical bounded policy subset used by ASTER exact-domain derivation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PolicySpec {
    pub policy_ir_version: u32,
    pub min_length: usize,
    pub max_length: usize,
    pub alphabet: String,
    #[serde(default)]
    pub forbidden_characters: String,
    #[serde(default)]
    pub classes: Vec<CharacterClassConstraint>,
    #[serde(default)]
    pub fixed_characters: Vec<FixedCharacterConstraint>,
    #[serde(default)]
    pub fixed_prefix: String,
    #[serde(default)]
    pub fixed_suffix: String,
    #[serde(default)]
    pub forbidden_first_characters: String,
    #[serde(default)]
    pub forbidden_last_characters: String,
    #[serde(default)]
    pub max_total_per_character: Option<usize>,
    #[serde(default)]
    pub max_identical_run: Option<usize>,
    #[serde(default)]
    pub max_sequential_run: Option<usize>,
    #[serde(default)]
    pub forbidden_substrings: Vec<String>,
}

impl PolicySpec {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json_canonicalizer::to_vec(self)?)
    }

    pub fn policy_hash(&self) -> Result<[u8; 32]> {
        Ok(Sha256::digest(self.canonical_bytes()?).into())
    }
}

/// Canonical partial DFA. Alphabet order defines shortlex rank order.
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
            return Err(invalid("alphabet must not be empty"));
        }
        let mut unique = HashSet::new();
        if !alphabet.iter().all(|character| unique.insert(*character)) {
            return Err(invalid("alphabet characters must be unique"));
        }
        if transitions.is_empty() || transitions.len() != accepting.len() {
            return Err(invalid(
                "transition and accepting tables must have equal non-zero size",
            ));
        }
        if start_state >= transitions.len() || min_length > max_length {
            return Err(invalid("invalid start state or length range"));
        }
        for row in &transitions {
            if row.len() != alphabet.len()
                || row
                    .iter()
                    .flatten()
                    .any(|target| *target >= transitions.len())
            {
                return Err(invalid("invalid transition table"));
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
    counts: Vec<Vec<BigUint>>,
}

#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    spec: PolicySpec,
    dfa: PolicyDfa,
    lengths: Vec<LengthTable>,
    total: BigUint,
}

impl CompiledPolicy {
    pub fn compile(spec: PolicySpec) -> Result<Self> {
        Self::compile_with_limit(spec, DEFAULT_MAX_POLICY_STATES)
    }

    pub fn compile_with_limit(spec: PolicySpec, max_states: usize) -> Result<Self> {
        let dfa = compile_dfa(&spec, max_states)?;
        Self::from_dfa(spec, dfa)
    }

    pub fn from_dfa(spec: PolicySpec, dfa: PolicyDfa) -> Result<Self> {
        let mut lengths = Vec::new();
        let mut total = BigUint::zero();
        for length in dfa.min_length..=dfa.max_length {
            let mut counts = vec![vec![BigUint::zero(); dfa.state_count()]; length + 1];
            for state in 0..dfa.state_count() {
                if dfa.is_accepting(state) {
                    counts[length][state] = BigUint::one();
                }
            }
            for position in (0..length).rev() {
                for state in 0..dfa.state_count() {
                    let mut count = BigUint::zero();
                    for alphabet_index in 0..dfa.alphabet.len() {
                        if let Some(next) = dfa.transition(state, alphabet_index) {
                            count += &counts[position + 1][next];
                        }
                    }
                    counts[position][state] = count;
                }
            }
            total += &counts[0][dfa.start_state];
            lengths.push(LengthTable { length, counts });
        }
        if total.is_zero() {
            return Err(invalid("policy language is empty"));
        }
        Ok(Self {
            spec,
            dfa,
            lengths,
            total,
        })
    }

    pub fn spec(&self) -> &PolicySpec {
        &self.spec
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

    pub fn count_payload_bytes(&self) -> usize {
        self.lengths
            .iter()
            .flat_map(|table| table.counts.iter())
            .flat_map(|row| row.iter())
            .map(|count| count.to_bytes_be().len())
            .sum()
    }

    pub fn completion_count(
        &self,
        length: usize,
        position: usize,
        state: usize,
    ) -> Option<&BigUint> {
        self.lengths
            .iter()
            .find(|table| table.length == length)
            .and_then(|table| table.counts.get(position))
            .and_then(|row| row.get(state))
    }

    pub fn entropy_bits(&self) -> f64 {
        let bits = self.total.bits();
        if bits <= 53 {
            return self.total.to_f64().expect("small BigUint fits f64").log2();
        }
        let shift = bits - 53;
        let top = (&self.total >> shift)
            .to_f64()
            .expect("top 53 bits fit f64");
        top.log2() + shift as f64
    }

    pub fn unrank(&self, rank: &BigUint) -> Result<String> {
        if rank >= &self.total {
            return Err(invalid("rank is outside the policy space"));
        }
        let mut residual = rank.clone();
        let mut table = None;
        for candidate in &self.lengths {
            let count = &candidate.counts[0][self.dfa.start_state];
            if &residual < count {
                table = Some(candidate);
                break;
            }
            residual -= count;
        }
        let table = table.expect("rank was bounded by total count");
        let mut state = self.dfa.start_state;
        let mut password = String::new();
        for position in 0..table.length {
            let mut selected = None;
            for (alphabet_index, character) in self.dfa.alphabet.iter().copied().enumerate() {
                let Some(next) = self.dfa.transition(state, alphabet_index) else {
                    continue;
                };
                let completions = &table.counts[position + 1][next];
                if &residual < completions {
                    selected = Some((character, next));
                    break;
                }
                residual -= completions;
            }
            let (character, next) = selected.expect("positive completion interval exists");
            password.push(character);
            state = next;
        }
        Ok(password)
    }

    pub fn rank(&self, password: &str) -> Result<BigUint> {
        let characters: Vec<char> = password.chars().collect();
        let table = self
            .lengths
            .iter()
            .find(|table| table.length == characters.len())
            .ok_or_else(|| invalid("password length is outside the policy"))?;
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
            let actual = self
                .dfa
                .alphabet
                .iter()
                .position(|candidate| *candidate == character)
                .ok_or_else(|| invalid("password contains a character outside the alphabet"))?;
            for alphabet_index in 0..actual {
                if let Some(next) = self.dfa.transition(state, alphabet_index) {
                    rank += &table.counts[position + 1][next];
                }
            }
            state = self
                .dfa
                .transition(state, actual)
                .ok_or_else(|| invalid("password is not accepted by the policy"))?;
        }
        if !self.dfa.is_accepting(state) {
            return Err(invalid("password is not accepted by the policy"));
        }
        Ok(rank)
    }

    pub fn accepts(&self, password: &str) -> bool {
        self.rank(password).is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StateKey {
    length: usize,
    class_counts: Vec<usize>,
    character_counts: Vec<usize>,
    last: Option<usize>,
    identical_run: usize,
    sequential_direction: i8,
    sequential_run: usize,
    tail: Vec<usize>,
}

struct CompilerContext {
    alphabet: Vec<char>,
    classes: Vec<HashSet<usize>>,
    fixed: HashMap<usize, usize>,
    prefix: Vec<usize>,
    suffix: Vec<usize>,
    forbidden_first: HashSet<usize>,
    forbidden_last: HashSet<usize>,
    forbidden_substrings: Vec<Vec<usize>>,
    tail_limit: usize,
    track_last: bool,
}

/// Compiles the canonical policy IR to its reachable DFA without constructing
/// ASTER's completion-count tables. This is exposed for matched algorithmic
/// baselines that consume the same automaton.
pub fn compile_dfa(spec: &PolicySpec, max_states: usize) -> Result<PolicyDfa> {
    if spec.policy_ir_version != 1 {
        return Err(invalid("unsupported policy IR version"));
    }
    if spec.min_length == 0 || spec.min_length > spec.max_length || spec.max_length > 128 {
        return Err(invalid("password length range must be within 1..=128"));
    }
    if max_states == 0 {
        return Err(invalid("policy state limit must be positive"));
    }
    for bound in [
        spec.max_total_per_character,
        spec.max_identical_run,
        spec.max_sequential_run,
    ]
    .into_iter()
    .flatten()
    {
        if bound == 0 {
            return Err(invalid("enabled policy maxima must be positive"));
        }
    }

    let forbidden: HashSet<char> = spec.forbidden_characters.chars().collect();
    let mut seen = HashSet::new();
    let alphabet: Vec<char> = spec
        .alphabet
        .chars()
        .filter(|character| !forbidden.contains(character))
        .filter(|character| seen.insert(*character))
        .collect();
    if alphabet.len() < 2 {
        return Err(invalid(
            "effective alphabet must contain at least two characters",
        ));
    }
    let alphabet_index: HashMap<char, usize> = alphabet
        .iter()
        .copied()
        .enumerate()
        .map(|(index, character)| (character, index))
        .collect();
    let mut class_names = HashSet::new();
    let mut classes = Vec::new();
    for class in &spec.classes {
        if class.name.is_empty() || !class_names.insert(class.name.clone()) {
            return Err(invalid(
                "character class names must be unique and non-empty",
            ));
        }
        if class.min_count > spec.max_length
            || class
                .max_count
                .is_some_and(|maximum| maximum < class.min_count || maximum > spec.max_length)
        {
            return Err(invalid("character class count bounds are inconsistent"));
        }
        let members: HashSet<usize> = class
            .alphabet
            .chars()
            .filter_map(|character| alphabet_index.get(&character).copied())
            .collect();
        if members.is_empty() && class.min_count > 0 {
            return Err(invalid("required character class is empty"));
        }
        classes.push(members);
    }

    let mut fixed = HashMap::new();
    for constraint in &spec.fixed_characters {
        if constraint.index >= spec.max_length || fixed.contains_key(&constraint.index) {
            return Err(invalid(
                "fixed character positions are invalid or duplicated",
            ));
        }
        let index = *alphabet_index
            .get(&constraint.character)
            .ok_or_else(|| invalid("fixed character is outside the effective alphabet"))?;
        fixed.insert(constraint.index, index);
    }
    let prefix = map_string(&spec.fixed_prefix, &alphabet_index, "fixed prefix")?;
    let suffix = map_string(&spec.fixed_suffix, &alphabet_index, "fixed suffix")?;
    if prefix.len() > spec.max_length || suffix.len() > spec.max_length {
        return Err(invalid("fixed prefix or suffix exceeds maximum length"));
    }
    for (position, character) in prefix.iter().copied().enumerate() {
        if fixed
            .get(&position)
            .is_some_and(|fixed| *fixed != character)
        {
            return Err(invalid("fixed prefix conflicts with a fixed character"));
        }
    }
    let forbidden_first = map_set(&spec.forbidden_first_characters, &alphabet_index);
    let forbidden_last = map_set(&spec.forbidden_last_characters, &alphabet_index);
    let mut forbidden_substrings = spec
        .forbidden_substrings
        .iter()
        .map(|substring| {
            if substring.is_empty() {
                return Err(invalid("forbidden substrings must not be empty"));
            }
            map_string(substring, &alphabet_index, "forbidden substring")
        })
        .collect::<Result<Vec<_>>>()?;
    forbidden_substrings.sort();
    forbidden_substrings.dedup();
    let tail_limit = suffix
        .len()
        .max(forbidden_substrings.iter().map(Vec::len).max().unwrap_or(0));
    let context = CompilerContext {
        alphabet: alphabet.clone(),
        classes,
        fixed,
        prefix,
        suffix,
        forbidden_first,
        forbidden_last,
        forbidden_substrings,
        tail_limit,
        track_last: spec.max_identical_run.is_some()
            || spec.max_sequential_run.is_some()
            || !spec.forbidden_last_characters.is_empty(),
    };

    let start = StateKey {
        length: 0,
        class_counts: vec![0; spec.classes.len()],
        character_counts: spec
            .max_total_per_character
            .map_or_else(Vec::new, |_| vec![0; alphabet.len()]),
        last: None,
        identical_run: 0,
        sequential_direction: 0,
        sequential_run: 1,
        tail: Vec::new(),
    };
    let mut states = vec![start.clone()];
    let mut ids = HashMap::from([(start, 0_usize)]);
    let mut rows: Vec<Vec<Option<usize>>> = Vec::new();
    let mut accepting = Vec::new();
    let mut queue = VecDeque::from([0_usize]);

    while let Some(state_id) = queue.pop_front() {
        let state = states[state_id].clone();
        let mut row = vec![None; alphabet.len()];
        if state.length < spec.max_length {
            for (character, target) in row.iter_mut().enumerate() {
                let Some(next) = transition_state(spec, &context, &state, character) else {
                    continue;
                };
                let next_id = if let Some(existing) = ids.get(&next).copied() {
                    existing
                } else {
                    if states.len() >= max_states {
                        return Err(invalid("policy compiler state limit exceeded"));
                    }
                    let new_id = states.len();
                    ids.insert(next.clone(), new_id);
                    states.push(next);
                    queue.push_back(new_id);
                    new_id
                };
                *target = Some(next_id);
            }
        }
        accepting.push(state_accepts(spec, &context, &state));
        rows.push(row);
    }
    PolicyDfa::new(
        alphabet,
        rows,
        0,
        accepting,
        spec.min_length,
        spec.max_length,
    )
}

fn transition_state(
    spec: &PolicySpec,
    context: &CompilerContext,
    state: &StateKey,
    character: usize,
) -> Option<StateKey> {
    if state.length == 0 && context.forbidden_first.contains(&character) {
        return None;
    }
    if context
        .fixed
        .get(&state.length)
        .is_some_and(|required| *required != character)
        || context
            .prefix
            .get(state.length)
            .is_some_and(|required| *required != character)
    {
        return None;
    }

    let mut class_counts = state.class_counts.clone();
    for (class_index, members) in context.classes.iter().enumerate() {
        if members.contains(&character) {
            if let Some(maximum) = spec.classes[class_index].max_count {
                class_counts[class_index] += 1;
                if class_counts[class_index] > maximum {
                    return None;
                }
            } else {
                class_counts[class_index] =
                    (class_counts[class_index] + 1).min(spec.classes[class_index].min_count);
            }
        }
    }
    let mut character_counts = state.character_counts.clone();
    if let Some(maximum) = spec.max_total_per_character {
        character_counts[character] += 1;
        if character_counts[character] > maximum {
            return None;
        }
    }

    let identical_run = spec.max_identical_run.map_or(0, |_| {
        if state.last == Some(character) {
            state.identical_run + 1
        } else {
            1
        }
    });
    if spec
        .max_identical_run
        .is_some_and(|maximum| identical_run > maximum)
    {
        return None;
    }

    let (sequential_direction, sequential_run) = if spec.max_sequential_run.is_some() {
        state.last.map_or((0, 1), |last| {
            let direction =
                sequential_direction(context.alphabet[last], context.alphabet[character]);
            if direction == 0 {
                (0, 1)
            } else if direction == state.sequential_direction {
                (direction, state.sequential_run + 1)
            } else {
                (direction, 2)
            }
        })
    } else {
        (0, 0)
    };
    if spec
        .max_sequential_run
        .is_some_and(|maximum| sequential_run > maximum)
    {
        return None;
    }

    let mut tail = state.tail.clone();
    if context.tail_limit > 0 {
        tail.push(character);
    }
    if context
        .forbidden_substrings
        .iter()
        .any(|forbidden| tail.ends_with(forbidden))
    {
        return None;
    }
    if tail.len() > context.tail_limit {
        tail.remove(0);
    }
    Some(StateKey {
        length: state.length + 1,
        class_counts,
        character_counts,
        last: context.track_last.then_some(character),
        identical_run,
        sequential_direction,
        sequential_run,
        tail,
    })
}

fn state_accepts(spec: &PolicySpec, context: &CompilerContext, state: &StateKey) -> bool {
    (spec.min_length..=spec.max_length).contains(&state.length)
        && spec
            .classes
            .iter()
            .zip(&state.class_counts)
            .all(|(class, count)| *count >= class.min_count)
        && (context.forbidden_last.is_empty()
            || state
                .last
                .is_some_and(|last| !context.forbidden_last.contains(&last)))
        && state.tail.ends_with(&context.suffix)
}

fn sequential_direction(left: char, right: char) -> i8 {
    if left.is_ascii_alphanumeric()
        && right.is_ascii_alphanumeric()
        && left.is_ascii_digit() == right.is_ascii_digit()
    {
        match (right as i32) - (left as i32) {
            1 => 1,
            -1 => -1,
            _ => 0,
        }
    } else {
        0
    }
}

fn map_string(value: &str, alphabet: &HashMap<char, usize>, label: &str) -> Result<Vec<usize>> {
    value
        .chars()
        .map(|character| {
            alphabet.get(&character).copied().ok_or_else(|| {
                invalid(&format!(
                    "{label} contains a character outside the alphabet"
                ))
            })
        })
        .collect()
}

fn map_set(value: &str, alphabet: &HashMap<char, usize>) -> HashSet<usize> {
    value
        .chars()
        .filter_map(|character| alphabet.get(&character).copied())
        .collect()
}

fn invalid(message: &str) -> KeylessPassError {
    KeylessPassError::Validation(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn tiny_spec(length: usize, require_digit: bool) -> PolicySpec {
        PolicySpec {
            policy_ir_version: 1,
            min_length: length,
            max_length: length,
            alphabet: "ab01".to_string(),
            forbidden_characters: String::new(),
            classes: require_digit
                .then(|| CharacterClassConstraint {
                    name: "digit".to_string(),
                    alphabet: "01".to_string(),
                    min_count: 1,
                    max_count: None,
                })
                .into_iter()
                .collect(),
            fixed_characters: Vec::new(),
            fixed_prefix: String::new(),
            fixed_suffix: String::new(),
            forbidden_first_characters: String::new(),
            forbidden_last_characters: String::new(),
            max_total_per_character: None,
            max_identical_run: None,
            max_sequential_run: None,
            forbidden_substrings: Vec::new(),
        }
    }

    #[test]
    fn exact_count_matches_inclusion_exclusion() {
        let policy = CompiledPolicy::compile(tiny_spec(3, true)).unwrap();
        assert_eq!(policy.total_count(), &BigUint::from(56_u8));
    }

    #[test]
    fn supports_edges_runs_suffix_and_forbidden_substrings() {
        let mut spec = tiny_spec(4, false);
        spec.fixed_prefix = "a".to_string();
        spec.fixed_suffix = "1".to_string();
        spec.forbidden_substrings = vec!["b0".to_string()];
        spec.max_identical_run = Some(2);
        spec.max_sequential_run = Some(2);
        let policy = CompiledPolicy::compile(spec).unwrap();
        for rank in 0_u64..policy.total_count().to_u64().unwrap() {
            let password = policy.unrank(&BigUint::from(rank)).unwrap();
            assert!(password.starts_with('a'));
            assert!(password.ends_with('1'));
            assert!(!password.contains("b0"));
            assert_eq!(policy.rank(&password).unwrap(), BigUint::from(rank));
        }
    }

    #[test]
    fn count_exceeds_u128_without_overflow() {
        let spec = PolicySpec {
            min_length: 128,
            max_length: 128,
            alphabet: "ab".to_string(),
            ..tiny_spec(1, false)
        };
        let policy = CompiledPolicy::compile(spec).unwrap();
        assert_eq!(policy.total_count(), &(BigUint::one() << 128_usize));
    }

    proptest! {
        #[test]
        fn rank_unrank_are_inverses(length in 1_usize..6, require_digit in any::<bool>()) {
            let policy = CompiledPolicy::compile(tiny_spec(length, require_digit)).unwrap();
            let total = policy.total_count().to_u64().unwrap();
            for value in 0..total {
                let rank = BigUint::from(value);
                let password = policy.unrank(&rank).unwrap();
                prop_assert!(policy.accepts(&password));
                prop_assert_eq!(policy.rank(&password).unwrap(), rank);
            }
        }
    }
}
