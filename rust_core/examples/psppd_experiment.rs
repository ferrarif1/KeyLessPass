use keylesspass_core::crypto::encoder;
use keylesspass_core::domain::{EncodingDescriptor, RequiredClass};
use keylesspass_core::research::psppd::{
    derive_password_ff1, required_class_policy, CompiledPolicy, PolicyDfa,
};
use num_bigint::BigUint;
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::time::Instant;

const SAMPLES: u64 = 2_000;

#[derive(Debug)]
struct Measurement {
    name: &'static str,
    nanoseconds: Vec<u128>,
    collisions: usize,
    auxiliary_average: f64,
}

fn main() {
    let alphabet: Vec<char> = "abcdABCD0123!@".chars().collect();
    let classes = [
        "abcd".chars().collect(),
        "ABCD".chars().collect(),
        "0123".chars().collect(),
        "!@".chars().collect(),
    ];
    let dfa = required_class_policy(alphabet.clone(), 8, &classes).unwrap();
    let policy = CompiledPolicy::compile(dfa.clone()).unwrap();
    let descriptor = encoder_descriptor();

    println!("PSPPD reproducible prototype experiment");
    println!("samples={SAMPLES}");
    println!("alphabet_size={}", alphabet.len());
    println!("length=8");
    println!("dfa_states={}", dfa.state_count());
    println!("count_table_cells={}", policy.table_cell_count());
    println!("exact_policy_space={}", policy.total_count());
    println!("entropy_bits={:.9}", policy.entropy_bits());
    println!("toy_unweighted_walk_tvd={:.9}", 9.0 / 22.0);
    println!("toy_completion_weighted_tvd=0.000000000");
    println!("toy_unweighted_prob_a=0.500000000");
    println!("toy_unweighted_prob_each_b_digit=0.050000000");
    println!("toy_uniform_prob_each={:.9}", 1.0 / 11.0);

    let measurements = [
        measure_encoder_v2(&descriptor),
        measure_local_walk(&dfa),
        measure_whole_rejection(&dfa),
        measure_psppd(&policy),
    ];
    println!("method,median_us,p95_us,collisions,aux_average");
    for mut measurement in measurements {
        measurement.nanoseconds.sort_unstable();
        let median = percentile(&measurement.nanoseconds, 0.50) as f64 / 1_000.0;
        let p95 = percentile(&measurement.nanoseconds, 0.95) as f64 / 1_000.0;
        println!(
            "{},{median:.3},{p95:.3},{},{:.6}",
            measurement.name, measurement.collisions, measurement.auxiliary_average
        );
    }
    println!(
        "toy_11_generation_hash_collisions={}",
        toy_hash_collisions()
    );
    println!("toy_11_generation_permutation_collisions=0");
    print_state_growth();
}

fn encoder_descriptor() -> EncodingDescriptor {
    EncodingDescriptor {
        length: 8,
        alphabet_profile: "psppd-experiment".to_string(),
        allowed_alphabet: "abcdABCD0123!@".to_string(),
        required_classes: vec![
            required("lower", "abcd"),
            required("upper", "ABCD"),
            required("digit", "0123"),
            required("symbol", "!@"),
        ],
        fixed_positions: Vec::new(),
        normalization: "none".to_string(),
        forbidden_chars: String::new(),
        forbidden_first_chars: String::new(),
        forbidden_last_chars: String::new(),
        forbid_repeated_characters: false,
        forbid_sequential_characters: false,
        max_attempts: 1_024,
        rule_version: 2,
    }
}

fn required(name: &str, alphabet: &str) -> RequiredClass {
    RequiredClass {
        name: name.to_string(),
        alphabet: alphabet.to_string(),
        position: None,
        min_count: 1,
        max_count: None,
    }
}

fn measure_encoder_v2(descriptor: &EncodingDescriptor) -> Measurement {
    let mut times = Vec::with_capacity(SAMPLES as usize);
    let mut outputs = HashSet::new();
    for generation in 0..SAMPLES {
        let secret = Sha256::digest(generation.to_be_bytes());
        let start = Instant::now();
        let password = encoder::encode_password(&secret, descriptor).unwrap();
        times.push(start.elapsed().as_nanos());
        outputs.insert(password);
    }
    Measurement {
        name: "encoder_v2",
        nanoseconds: times,
        collisions: SAMPLES as usize - outputs.len(),
        auxiliary_average: f64::NAN,
    }
}

fn measure_local_walk(dfa: &PolicyDfa) -> Measurement {
    let mut times = Vec::with_capacity(SAMPLES as usize);
    let mut outputs = HashSet::new();
    let mut total_attempts = 0_u64;
    for generation in 0..SAMPLES {
        let mut rng = StdRng::seed_from_u64(generation);
        let start = Instant::now();
        let (password, attempts) = loop {
            let mut state = dfa.start_state();
            let mut candidate = String::new();
            for _ in 0..dfa.max_length() {
                let alphabet_index = rng.gen_range(0..dfa.alphabet().len());
                candidate.push(dfa.alphabet()[alphabet_index]);
                state = dfa.transition(state, alphabet_index).unwrap();
            }
            total_attempts += 1;
            if dfa.is_accepting(state) {
                break (candidate, total_attempts);
            }
        };
        let _ = attempts;
        times.push(start.elapsed().as_nanos());
        outputs.insert(password);
    }
    Measurement {
        name: "local_unweighted_walk_restart",
        nanoseconds: times,
        collisions: SAMPLES as usize - outputs.len(),
        auxiliary_average: total_attempts as f64 / SAMPLES as f64,
    }
}

fn measure_whole_rejection(dfa: &PolicyDfa) -> Measurement {
    let mut times = Vec::with_capacity(SAMPLES as usize);
    let mut outputs = HashSet::new();
    let mut total_attempts = 0_u64;
    for generation in 0..SAMPLES {
        let mut rng = StdRng::seed_from_u64(generation ^ 0x7265_6a65_6374);
        let start = Instant::now();
        let password = loop {
            total_attempts += 1;
            let candidate: String = (0..dfa.max_length())
                .map(|_| dfa.alphabet()[rng.gen_range(0..dfa.alphabet().len())])
                .collect();
            let mut state = dfa.start_state();
            for character in candidate.chars() {
                let index = dfa
                    .alphabet()
                    .iter()
                    .position(|candidate| *candidate == character)
                    .unwrap();
                state = dfa.transition(state, index).unwrap();
            }
            if dfa.is_accepting(state) {
                break candidate;
            }
        };
        times.push(start.elapsed().as_nanos());
        outputs.insert(password);
    }
    Measurement {
        name: "whole_rejection",
        nanoseconds: times,
        collisions: SAMPLES as usize - outputs.len(),
        auxiliary_average: total_attempts as f64 / SAMPLES as f64,
    }
}

fn measure_psppd(policy: &CompiledPolicy) -> Measurement {
    let key = [0x5a_u8; 32];
    let tweak = b"service/account/policy-epoch-1";
    let mut times = Vec::with_capacity(SAMPLES as usize);
    let mut outputs = HashSet::new();
    let mut total_walks = 0_u64;
    for generation in 0..SAMPLES {
        let start = Instant::now();
        let derived = derive_password_ff1(policy, &key, tweak, &BigUint::from(generation)).unwrap();
        times.push(start.elapsed().as_nanos());
        total_walks += u64::from(derived.cycle_walks);
        outputs.insert(derived.password);
    }
    Measurement {
        name: "count_permute_unrank",
        nanoseconds: times,
        collisions: SAMPLES as usize - outputs.len(),
        auxiliary_average: total_walks as f64 / SAMPLES as f64,
    }
}

fn toy_hash_collisions() -> usize {
    let mut outputs = HashSet::new();
    for generation in 0_u64..11 {
        let digest = Sha256::digest(generation.to_be_bytes());
        outputs.insert(u64::from_be_bytes(digest[..8].try_into().unwrap()) % 11);
    }
    11 - outputs.len()
}

fn percentile(sorted: &[u128], percentile: f64) -> u128 {
    let index = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted[index]
}

fn print_state_growth() {
    println!("required_classes,dfa_states,count_table_cells,compile_us,exact_space");
    for class_count in [4_usize, 6, 8, 10, 12] {
        let alphabet: Vec<char> = "abcdefghijkl".chars().take(class_count).collect();
        let classes: Vec<Vec<char>> = alphabet.iter().copied().map(|ch| vec![ch]).collect();
        let dfa = required_class_policy(alphabet, class_count, &classes).unwrap();
        let start = Instant::now();
        let policy = CompiledPolicy::compile(dfa).unwrap();
        let compile_us = start.elapsed().as_nanos() as f64 / 1_000.0;
        println!(
            "{class_count},{},{},{compile_us:.3},{}",
            policy.dfa().state_count(),
            policy.table_cell_count(),
            policy.total_count()
        );
    }
}
