use crate::domain::cdr::EncodingDescriptor;
use crate::error::{KeylessPassError, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashSet;

type HmacSha256 = Hmac<Sha256>;

pub fn encode_password(secret: &[u8], descriptor: &EncodingDescriptor) -> Result<String> {
    validate_descriptor(descriptor)?;
    let alphabet = effective_alphabet(descriptor)?;
    let mut stream = DeterministicStream::new(secret, b"KeylessPass password encoder")?;
    let mut chars: Vec<char> = (0..descriptor.length)
        .map(|_| choose_char(&alphabet, &mut stream))
        .collect::<Result<Vec<_>>>()?;

    apply_fixed_positions(&mut chars, descriptor)?;
    apply_required_classes(&mut chars, descriptor, &mut stream)?;

    let password: String = chars.into_iter().collect();
    if password
        .chars()
        .any(|ch| descriptor.forbidden_chars.chars().any(|bad| bad == ch))
    {
        return Err(KeylessPassError::Validation(
            "encoded password contains forbidden character".to_string(),
        ));
    }
    Ok(password)
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

fn validate_descriptor(descriptor: &EncodingDescriptor) -> Result<()> {
    if descriptor.length < 8 || descriptor.length > 128 {
        return Err(KeylessPassError::Validation(
            "password length must be between 8 and 128".to_string(),
        ));
    }
    let alphabet = effective_alphabet(descriptor)?;
    if alphabet.len() < 2 {
        return Err(KeylessPassError::Validation(
            "allowed alphabet is too small after forbidden character filtering".to_string(),
        ));
    }
    for fixed in &descriptor.fixed_positions {
        if fixed.index >= descriptor.length {
            return Err(KeylessPassError::Validation(
                "fixed position is outside password length".to_string(),
            ));
        }
        let mut fixed_chars = fixed.character.chars();
        let Some(ch) = fixed_chars.next() else {
            return Err(KeylessPassError::Validation(
                "fixed position character is empty".to_string(),
            ));
        };
        if fixed_chars.next().is_some() {
            return Err(KeylessPassError::Validation(
                "fixed position character must be a single scalar value".to_string(),
            ));
        }
        if descriptor.forbidden_chars.chars().any(|bad| bad == ch) {
            return Err(KeylessPassError::Validation(
                "fixed position uses a forbidden character".to_string(),
            ));
        }
    }
    for class in &descriptor.required_classes {
        let filtered: Vec<char> = class
            .alphabet
            .chars()
            .filter(|ch| !descriptor.forbidden_chars.chars().any(|bad| bad == *ch))
            .collect();
        if filtered.is_empty() {
            return Err(KeylessPassError::Validation(format!(
                "required class {} is empty after forbidden filtering",
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
    Ok(())
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

fn choose_char(alphabet: &[char], stream: &mut DeterministicStream) -> Result<char> {
    let index = stream.next_usize()? % alphabet.len();
    Ok(alphabet[index])
}

fn apply_fixed_positions(chars: &mut [char], descriptor: &EncodingDescriptor) -> Result<()> {
    for fixed in &descriptor.fixed_positions {
        let ch = fixed.character.chars().next().ok_or_else(|| {
            KeylessPassError::Validation("fixed position character is empty".to_string())
        })?;
        chars[fixed.index] = ch;
    }
    Ok(())
}

fn apply_required_classes(
    chars: &mut [char],
    descriptor: &EncodingDescriptor,
    stream: &mut DeterministicStream,
) -> Result<()> {
    let fixed_indexes: HashSet<usize> = descriptor
        .fixed_positions
        .iter()
        .map(|fixed| fixed.index)
        .collect();
    let mut used_positions = HashSet::new();

    for class in &descriptor.required_classes {
        let class_chars: Vec<char> = class
            .alphabet
            .chars()
            .filter(|ch| !descriptor.forbidden_chars.chars().any(|bad| bad == *ch))
            .collect();

        let position = if let Some(position) = class.position {
            if fixed_indexes.contains(&position) {
                let fixed_char = chars[position];
                if class_chars.iter().any(|ch| *ch == fixed_char) {
                    used_positions.insert(position);
                    continue;
                }
                return Err(KeylessPassError::Validation(format!(
                    "fixed position conflicts with required class {}",
                    class.name
                )));
            }
            position
        } else {
            let candidates: Vec<usize> = (0..chars.len())
                .filter(|idx| !fixed_indexes.contains(idx) && !used_positions.contains(idx))
                .collect();
            if candidates.is_empty() {
                return Err(KeylessPassError::Validation(
                    "no free position for required class".to_string(),
                ));
            }
            candidates[stream.next_usize()? % candidates.len()]
        };

        chars[position] = class_chars[stream.next_usize()? % class_chars.len()];
        used_positions.insert(position);
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
    fn new(key: &[u8], label: &[u8]) -> Result<Self> {
        Ok(Self {
            key: key.to_vec(),
            label: label.to_vec(),
            counter: 0,
            buffer: Vec::new(),
            offset: 0,
        })
    }

    fn next_usize(&mut self) -> Result<usize> {
        let mut bytes = [0_u8; 8];
        for item in &mut bytes {
            *item = self.next_byte()?;
        }
        Ok(u64::from_be_bytes(bytes) as usize)
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
        self.counter += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EncodingDescriptor;

    #[test]
    fn same_secret_and_descriptor_are_stable() {
        let descriptor = EncodingDescriptor::default();
        let left = encode_password(&[7_u8; 32], &descriptor).unwrap();
        let right = encode_password(&[7_u8; 32], &descriptor).unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn forbidden_chars_are_filtered() {
        let mut descriptor = EncodingDescriptor::default();
        descriptor.forbidden_chars.push('@');
        descriptor.forbidden_chars.push('#');
        let password = encode_password(&[9_u8; 32], &descriptor).unwrap();
        assert!(!password.contains('@'));
        assert!(!password.contains('#'));
    }

    #[test]
    fn descriptor_change_requires_new_version() {
        let old_descriptor = EncodingDescriptor::default();
        let mut new_descriptor = old_descriptor.clone();
        new_descriptor.length += 1;
        assert!(ensure_rotation_required(&old_descriptor, &new_descriptor, 1, 1).is_err());
        assert!(ensure_rotation_required(&old_descriptor, &new_descriptor, 1, 2).is_ok());
    }
}
