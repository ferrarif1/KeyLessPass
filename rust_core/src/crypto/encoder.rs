use crate::domain::cdr::{EncodingDescriptor, RequiredClass};
use crate::error::{KeylessPassError, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashSet;

type HmacSha256 = Hmac<Sha256>;

pub fn encode_password(secret: &[u8], descriptor: &EncodingDescriptor) -> Result<String> {
    validate_descriptor(descriptor)?;
    let alphabet = effective_alphabet(descriptor)?;
    let mut stream = DeterministicStream::new(secret, b"KeyLessPass/password-encoder/v2");
    for _ in 0..descriptor.max_attempts {
        if let Some(password) = construct_candidate(&alphabet, descriptor, &mut stream)? {
            return Ok(password);
        }
    }
    Err(KeylessPassError::Validation(format!(
        "password policy was not satisfied within {} deterministic attempts",
        descriptor.max_attempts
    )))
}

pub fn ensure_rotation_required(
    old_descriptor: &EncodingDescriptor,
    new_descriptor: &EncodingDescriptor,
    old_version: u32,
    new_version: u32,
) -> Result<()> {
    if old_descriptor != new_descriptor && new_version <= old_version {
        return Err(KeylessPassError::Validation(
            "encodingDescriptor changes must create a new CDR version".to_string(),
        ));
    }
    Ok(())
}

pub fn password_space_upper_bound_log2(descriptor: &EncodingDescriptor) -> Result<f64> {
    validate_descriptor(descriptor)?;
    let alphabet_len = effective_alphabet(descriptor)?.len();
    let unfixed = descriptor
        .length
        .saturating_sub(descriptor.fixed_positions.len());
    Ok(unfixed as f64 * (alphabet_len as f64).log2())
}

pub fn validate_descriptor(descriptor: &EncodingDescriptor) -> Result<()> {
    if descriptor.length < 8 || descriptor.length > 128 {
        return Err(KeylessPassError::Validation(
            "password length must be between 8 and 128".to_string(),
        ));
    }
    if descriptor.max_attempts == 0 || descriptor.max_attempts > 1_000_000 {
        return Err(KeylessPassError::Validation(
            "maxAttempts must be between 1 and 1000000".to_string(),
        ));
    }
    let alphabet = effective_alphabet(descriptor)?;
    if alphabet.len() < 2 {
        return Err(KeylessPassError::Validation(
            "allowed alphabet is too small after forbidden character filtering".to_string(),
        ));
    }
    let allowed: HashSet<char> = alphabet.iter().copied().collect();
    let mut fixed_indexes = HashSet::new();
    for fixed in &descriptor.fixed_positions {
        if fixed.index >= descriptor.length || !fixed_indexes.insert(fixed.index) {
            return Err(KeylessPassError::Validation(
                "fixed positions must be unique and inside password length".to_string(),
            ));
        }
        let ch = single_character(&fixed.character, "fixed position")?;
        if !allowed.contains(&ch) {
            return Err(KeylessPassError::Validation(
                "fixed position character is not in the effective alphabet".to_string(),
            ));
        }
    }
    for class in &descriptor.required_classes {
        let class_chars = class_alphabet(class, descriptor);
        if class_chars.is_empty() {
            return Err(KeylessPassError::Validation(format!(
                "required class {} is empty after policy filtering",
                class.name
            )));
        }
        if class.min_count > descriptor.length
            || class
                .max_count
                .is_some_and(|maximum| maximum < class.min_count || maximum > descriptor.length)
        {
            return Err(KeylessPassError::Validation(format!(
                "required class {} has inconsistent min/max counts",
                class.name
            )));
        }
        if let Some(position) = class.position {
            if position >= descriptor.length {
                return Err(KeylessPassError::Validation(
                    "required class position is outside password length".to_string(),
                ));
            }
        }
    }
    if descriptor.forbid_repeated_characters && alphabet.len() < descriptor.length {
        return Err(KeylessPassError::Validation(
            "no-repeat policy needs at least as many allowed characters as password positions"
                .to_string(),
        ));
    }
    Ok(())
}

fn construct_candidate(
    alphabet: &[char],
    descriptor: &EncodingDescriptor,
    stream: &mut DeterministicStream,
) -> Result<Option<String>> {
    let mut slots = vec![None; descriptor.length];
    for fixed in &descriptor.fixed_positions {
        slots[fixed.index] = Some(single_character(&fixed.character, "fixed position")?);
    }

    for class in descriptor
        .required_classes
        .iter()
        .filter(|class| class.position.is_some())
    {
        let position = class.position.unwrap();
        let class_chars = class_alphabet(class, descriptor);
        match slots[position] {
            Some(ch) if !class_chars.contains(&ch) => return Ok(None),
            Some(_) => {}
            None => {
                let ch = class_chars[stream.sample_below(class_chars.len())?];
                if !can_place(ch, &slots, descriptor) {
                    return Ok(None);
                }
                slots[position] = Some(ch);
            }
        }
    }

    let mut free_positions: Vec<usize> = slots
        .iter()
        .enumerate()
        .filter_map(|(index, value)| value.is_none().then_some(index))
        .collect();
    fisher_yates(&mut free_positions, stream)?;

    for class in &descriptor.required_classes {
        let class_chars = class_alphabet(class, descriptor);
        while class_count(&slots, &class_chars) < class.min_count {
            let Some(position) = free_positions.pop() else {
                return Ok(None);
            };
            let candidates: Vec<char> = class_chars
                .iter()
                .copied()
                .filter(|ch| can_place(*ch, &slots, descriptor))
                .collect();
            if candidates.is_empty() {
                return Ok(None);
            }
            slots[position] = Some(candidates[stream.sample_below(candidates.len())?]);
        }
    }

    for position in free_positions {
        let candidates: Vec<char> = alphabet
            .iter()
            .copied()
            .filter(|ch| can_place(*ch, &slots, descriptor))
            .collect();
        if candidates.is_empty() {
            return Ok(None);
        }
        slots[position] = Some(candidates[stream.sample_below(candidates.len())?]);
    }
    let password: String = slots.into_iter().flatten().collect();
    Ok(policy_satisfied(&password, descriptor).then_some(password))
}

fn can_place(ch: char, slots: &[Option<char>], descriptor: &EncodingDescriptor) -> bool {
    if descriptor.forbid_repeated_characters && slots.contains(&Some(ch)) {
        return false;
    }
    descriptor.required_classes.iter().all(|class| {
        let class_chars = class_alphabet(class, descriptor);
        !class_chars.contains(&ch)
            || class
                .max_count
                .map_or(true, |maximum| class_count(slots, &class_chars) < maximum)
    })
}

fn policy_satisfied(password: &str, descriptor: &EncodingDescriptor) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() != descriptor.length {
        return false;
    }
    if chars
        .first()
        .is_some_and(|ch| descriptor.forbidden_first_chars.contains(*ch))
        || chars
            .last()
            .is_some_and(|ch| descriptor.forbidden_last_chars.contains(*ch))
    {
        return false;
    }
    if descriptor.forbid_repeated_characters {
        let unique: HashSet<char> = chars.iter().copied().collect();
        if unique.len() != chars.len() {
            return false;
        }
    }
    if descriptor.forbid_sequential_characters
        && chars
            .windows(2)
            .any(|pair| ascii_sequential(pair[0], pair[1]))
    {
        return false;
    }
    descriptor.required_classes.iter().all(|class| {
        let class_chars = class_alphabet(class, descriptor);
        let count = chars.iter().filter(|ch| class_chars.contains(ch)).count();
        count >= class.min_count && class.max_count.map_or(true, |maximum| count <= maximum)
    })
}

fn ascii_sequential(left: char, right: char) -> bool {
    left.is_ascii_alphanumeric()
        && right.is_ascii_alphanumeric()
        && left.is_ascii_digit() == right.is_ascii_digit()
        && (left as i32 - right as i32).unsigned_abs() == 1
}

fn effective_alphabet(descriptor: &EncodingDescriptor) -> Result<Vec<char>> {
    let forbidden: HashSet<char> = descriptor.forbidden_chars.chars().collect();
    let mut seen = HashSet::new();
    let alphabet: Vec<char> = descriptor
        .allowed_alphabet
        .chars()
        .filter(|ch| !forbidden.contains(ch))
        .filter(|ch| seen.insert(*ch))
        .collect();
    if alphabet.is_empty() {
        Err(KeylessPassError::Validation(
            "allowed alphabet is empty".to_string(),
        ))
    } else {
        Ok(alphabet)
    }
}

fn class_alphabet(class: &RequiredClass, descriptor: &EncodingDescriptor) -> Vec<char> {
    let allowed: HashSet<char> = descriptor.allowed_alphabet.chars().collect();
    let forbidden: HashSet<char> = descriptor.forbidden_chars.chars().collect();
    let mut seen = HashSet::new();
    class
        .alphabet
        .chars()
        .filter(|ch| allowed.contains(ch) && !forbidden.contains(ch))
        .filter(|ch| seen.insert(*ch))
        .collect()
}

fn class_count(slots: &[Option<char>], alphabet: &[char]) -> usize {
    slots
        .iter()
        .filter(|value| value.is_some_and(|ch| alphabet.contains(&ch)))
        .count()
}

fn single_character(value: &str, label: &str) -> Result<char> {
    let mut chars = value.chars();
    let first = chars
        .next()
        .ok_or_else(|| KeylessPassError::Validation(format!("{label} character is empty")))?;
    if chars.next().is_some() {
        return Err(KeylessPassError::Validation(format!(
            "{label} must contain one Unicode scalar value"
        )));
    }
    Ok(first)
}

fn fisher_yates(values: &mut [usize], stream: &mut DeterministicStream) -> Result<()> {
    for index in (1..values.len()).rev() {
        let selected = stream.sample_below(index + 1)?;
        values.swap(index, selected);
    }
    Ok(())
}

struct DeterministicStream {
    key: Vec<u8>,
    label: Vec<u8>,
    counter: u64,
    buffer: Vec<u8>,
    offset: usize,
}

impl DeterministicStream {
    fn new(key: &[u8], label: &[u8]) -> Self {
        Self {
            key: key.to_vec(),
            label: label.to_vec(),
            counter: 0,
            buffer: Vec::new(),
            offset: 0,
        }
    }

    fn sample_below(&mut self, upper: usize) -> Result<usize> {
        if upper == 0 {
            return Err(KeylessPassError::Validation(
                "cannot sample from an empty set".to_string(),
            ));
        }
        let upper = upper as u64;
        let limit = u64::MAX - (u64::MAX % upper);
        loop {
            let value = self.next_u64()?;
            if value < limit {
                return Ok((value % upper) as usize);
            }
        }
    }

    fn next_u64(&mut self) -> Result<u64> {
        let mut bytes = [0_u8; 8];
        for byte in &mut bytes {
            *byte = self.next_byte()?;
        }
        Ok(u64::from_be_bytes(bytes))
    }

    fn next_byte(&mut self) -> Result<u8> {
        if self.offset >= self.buffer.len() {
            self.refill()?;
        }
        let byte = self.buffer[self.offset];
        self.offset += 1;
        Ok(byte)
    }

    fn refill(&mut self) -> Result<()> {
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .map_err(|_| KeylessPassError::Crypto("invalid encoder stream key".to_string()))?;
        mac.update(&self.label);
        mac.update(&self.counter.to_be_bytes());
        self.buffer = mac.finalize().into_bytes().to_vec();
        self.offset = 0;
        self.counter = self
            .counter
            .checked_add(1)
            .ok_or_else(|| KeylessPassError::Crypto("encoder stream exhausted".to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_secret_and_descriptor_are_stable_and_policy_compliant() {
        let descriptor = EncodingDescriptor::default();
        let left = encode_password(&[7_u8; 32], &descriptor).unwrap();
        let right = encode_password(&[7_u8; 32], &descriptor).unwrap();
        assert_eq!(left, right);
        assert!(policy_satisfied(&left, &descriptor));
    }

    #[test]
    fn supports_counts_edges_repetition_and_sequence_constraints() {
        let mut descriptor = EncodingDescriptor {
            length: 20,
            forbidden_first_chars: "!@#$%*-_=+".to_string(),
            forbidden_last_chars: "!@#$%*-_=+".to_string(),
            forbid_repeated_characters: true,
            forbid_sequential_characters: true,
            ..EncodingDescriptor::default()
        };
        descriptor.required_classes[0].min_count = 2;
        descriptor.required_classes[0].max_count = Some(4);
        let password = encode_password(&[9_u8; 32], &descriptor).unwrap();
        assert!(policy_satisfied(&password, &descriptor));
    }

    #[test]
    fn reports_contradictory_policy_instead_of_relaxing_it() {
        let descriptor = EncodingDescriptor {
            allowed_alphabet: "A".to_string(),
            forbidden_chars: "A".to_string(),
            ..EncodingDescriptor::default()
        };
        assert!(encode_password(&[1_u8; 32], &descriptor).is_err());
    }

    #[test]
    fn required_characters_are_not_fixed_at_predictable_positions() {
        let mut descriptor = EncodingDescriptor::default();
        descriptor.required_classes[0].max_count = Some(1);
        let upper_positions: HashSet<usize> = (0_u8..64)
            .map(|seed| encode_password(&[seed; 32], &descriptor).unwrap())
            .map(|password| password.chars().position(char::is_uppercase).unwrap())
            .collect();
        assert!(upper_positions.len() > 12, "{upper_positions:?}");
    }

    #[test]
    fn descriptor_change_requires_new_version() {
        let old_descriptor = EncodingDescriptor::default();
        let mut new_descriptor = old_descriptor.clone();
        new_descriptor.length += 1;
        assert!(ensure_rotation_required(&old_descriptor, &new_descriptor, 1, 1).is_err());
        assert!(ensure_rotation_required(&old_descriptor, &new_descriptor, 1, 2).is_ok());
    }

    #[test]
    fn property_generated_passwords_satisfy_legal_policy_family() {
        for seed in 0_u8..128 {
            let mut descriptor = EncodingDescriptor {
                length: 12 + usize::from(seed % 13),
                forbid_repeated_characters: seed % 3 == 0,
                forbid_sequential_characters: seed % 5 == 0,
                ..EncodingDescriptor::default()
            };
            descriptor.required_classes[0].min_count = 1 + usize::from(seed % 2);
            let password = encode_password(&[seed; 32], &descriptor).unwrap();
            assert!(policy_satisfied(&password, &descriptor), "seed={seed}");
        }
    }
}
