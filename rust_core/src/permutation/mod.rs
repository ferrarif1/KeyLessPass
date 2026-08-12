use crate::error::{KeylessPassError, Result};
use aes::Aes256;
use fpe::ff1::{FlexibleNumeralString, FF1};
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

pub const MIN_FF1_DOMAIN_SIZE: u64 = 1_000_000;
pub const MAX_FF1_DOMAIN_BITS: u64 = 512;
pub const DEFAULT_MAX_CYCLE_WALKS: u32 = 1_024;

pub trait DomainPermutation {
    fn permute(
        &self,
        key: &[u8],
        tweak: &[u8],
        domain_size: &BigUint,
        input: &BigUint,
    ) -> Result<BigUint>;

    fn invert(
        &self,
        key: &[u8],
        tweak: &[u8],
        domain_size: &BigUint,
        input: &BigUint,
    ) -> Result<BigUint>;
}

/// FF1-AES-256 over the smallest binary superset, restricted by cycle walking.
///
/// The configured walk limit is an implementation safeguard. Consequently, this
/// backend can fail closed even though the mathematical cycle-walk construction
/// defines a permutation on the requested domain.
#[derive(Debug, Clone, Copy)]
pub struct Ff1CycleWalking {
    pub max_walks: u32,
}

impl Default for Ff1CycleWalking {
    fn default() -> Self {
        Self {
            max_walks: DEFAULT_MAX_CYCLE_WALKS,
        }
    }
}

impl Ff1CycleWalking {
    /// Executes the forward cycle walk and returns the number of FF1 calls.
    /// This value is evaluation instrumentation and is not credential state.
    pub fn permute_with_walk_count(
        &self,
        key: &[u8],
        tweak: &[u8],
        domain_size: &BigUint,
        input: &BigUint,
    ) -> Result<(BigUint, u32)> {
        cycle_walk_counted(key, tweak, domain_size, input, self.max_walks, false)
    }
}

impl DomainPermutation for Ff1CycleWalking {
    fn permute(
        &self,
        key: &[u8],
        tweak: &[u8],
        domain_size: &BigUint,
        input: &BigUint,
    ) -> Result<BigUint> {
        self.permute_with_walk_count(key, tweak, domain_size, input)
            .map(|(output, _)| output)
    }

    fn invert(
        &self,
        key: &[u8],
        tweak: &[u8],
        domain_size: &BigUint,
        input: &BigUint,
    ) -> Result<BigUint> {
        cycle_walk_counted(key, tweak, domain_size, input, self.max_walks, true)
            .map(|(output, _)| output)
    }
}

fn cycle_walk_counted(
    key: &[u8],
    tweak: &[u8],
    domain_size: &BigUint,
    input: &BigUint,
    max_walks: u32,
    decrypt: bool,
) -> Result<(BigUint, u32)> {
    let key: &[u8; 32] = key
        .try_into()
        .map_err(|_| crypto("FF1 backend requires a 256-bit key"))?;
    if domain_size < &BigUint::from(MIN_FF1_DOMAIN_SIZE) {
        return Err(validation("FF1 domain is smaller than 1,000,000"));
    }
    if input >= domain_size {
        return Err(validation("permutation input is outside the domain"));
    }
    if max_walks == 0 {
        return Err(validation("cycle-walk limit must be positive"));
    }
    let bit_length = (domain_size - BigUint::one()).bits();
    if bit_length > MAX_FF1_DOMAIN_BITS {
        return Err(validation("FF1 domain exceeds 512 bits"));
    }
    let ff1 = FF1::<Aes256>::new(key, 2).map_err(|error| crypto(&error.to_string()))?;
    let mut current = input.clone();
    for walk in 1..=max_walks {
        let numeral = FlexibleNumeralString::from(to_fixed_bits(&current, bit_length as usize));
        let transformed = if decrypt {
            ff1.decrypt(tweak, &numeral)
        } else {
            ff1.encrypt(tweak, &numeral)
        }
        .map_err(|error| crypto(&error.to_string()))?;
        current = from_bits(&Vec::<u16>::from(transformed));
        if &current < domain_size {
            return Ok((current, walk));
        }
    }
    Err(crypto("FF1 cycle-walk limit exceeded"))
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

fn validation(message: &str) -> KeylessPassError {
    KeylessPassError::Validation(message.to_string())
}

fn crypto(message: &str) -> KeylessPassError {
    KeylessPassError::Crypto(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ff1_backend_inverts_and_has_no_sample_collisions() {
        let domain = BigUint::from(1_000_003_u64);
        let backend = Ff1CycleWalking::default();
        let key = [0x42_u8; 32];
        let mut outputs = HashSet::new();
        for value in 0_u32..2_000 {
            let input = BigUint::from(value);
            let output = backend
                .permute(&key, b"test-tweak", &domain, &input)
                .unwrap();
            assert!(outputs.insert(output.clone()));
            assert_eq!(
                backend
                    .invert(&key, b"test-tweak", &domain, &output)
                    .unwrap(),
                input
            );
        }
    }
}
