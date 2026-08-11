use keylesspass_core::epscd::{
    derive_credential_key, derive_password, permutation_tweak, CredentialContext, SCHEME_VERSION_V1,
};
use keylesspass_core::permutation::Ff1CycleWalking;
use keylesspass_core::policy::{
    compile_dfa, CharacterClassConstraint, CompiledPolicy, FixedCharacterConstraint, PolicyDfa,
    PolicySpec, DEFAULT_MAX_POLICY_STATES,
};
use keylesspass_core::published_baselines::dichopile::ExactDichopile;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, RngCore, SeedableRng};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use uuid::Uuid;

const DISTRIBUTION_SAMPLES: usize = 100_000;
const PERFORMANCE_SAMPLES: usize = 500;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("experiments");
    for directory in [
        "policies",
        "real_policy_corpus",
        "distribution",
        "nonrepetition",
        "performance",
        "recovery",
        "raw",
    ] {
        fs::create_dir_all(root.join(directory))?;
    }

    write_json(root.join("policies/policy_metrics.json"), policy_metrics()?)?;
    write_json(
        root.join("distribution/distribution.json"),
        distribution_experiment()?,
    )?;
    let supplementary = root
        .parent()
        .expect("experiments has a project parent")
        .join("supplementary/mathematical_controls");
    fs::create_dir_all(&supplementary)?;
    write_json(
        supplementary.join("controls.json"),
        mechanistic_controls_experiment()?,
    )?;
    write_json(
        root.join("nonrepetition/nonrepetition.json"),
        collision_experiment()?,
    )?;
    write_json(
        root.join("performance/performance.json"),
        performance_experiment()?,
    )?;
    write_json(
        root.join("raw/run_metadata.json"),
        json!({
            "schemaVersion": 1,
            "generatedAtUtc": chrono::Utc::now().to_rfc3339(),
            "crateVersion": env!("CARGO_PKG_VERSION"),
            "distributionSamples": DISTRIBUTION_SAMPLES,
            "performanceSamples": PERFORMANCE_SAMPLES,
            "claimsBoundary": [
                "Synthetic policies are supplemented by the public historical SOUPS 2022 PCP corpus; neither is a current enterprise deployment sample.",
                "Timing values are single-host prototype measurements, not production latency claims.",
                "Empirical collision counts do not replace the permutation injectivity argument."
            ]
        }),
    )?;
    println!(
        "wrote reproducible evaluation results under {}",
        root.display()
    );
    Ok(())
}

fn write_json(path: PathBuf, value: Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}

fn policy_metrics() -> Result<Value, Box<dyn std::error::Error>> {
    let policies = synthetic_policies();
    let mut rows = Vec::new();
    for (name, spec) in policies {
        let started = Instant::now();
        match CompiledPolicy::compile(spec) {
            Ok(compiled) => rows.push(json!({
                "name": name,
                "status": "compiled",
                "alphabetSize": compiled.dfa().alphabet().len(),
                "lengthRange": [compiled.dfa().min_length(), compiled.dfa().max_length()],
                "stateCount": compiled.dfa().state_count(),
                "countTableCells": compiled.table_cell_count(),
                "countPayloadBytes": compiled.count_payload_bytes(),
                "exactSpace": compiled.total_count().to_string(),
                "entropyBits": compiled.entropy_bits(),
                "compileMicros": started.elapsed().as_secs_f64() * 1_000_000.0,
            })),
            Err(error) => rows.push(json!({
                "name": name,
                "status": "rejected",
                "error": error.to_string(),
                "compileMicros": started.elapsed().as_secs_f64() * 1_000_000.0,
            })),
        }
    }
    Ok(json!({
        "schemaVersion": 1,
        "corpusKind": "synthetic representative policy corpus",
        "memoryMetricBoundary": "countPayloadBytes sums serialized BigUint payloads only; it is not process RSS or allocator usage",
        "policies": rows,
    }))
}

fn synthetic_policies() -> Vec<(&'static str, PolicySpec)> {
    let mut corpus = vec![
        (
            "lower-digit-8",
            base_policy(8, 8, "abcdefghijklmnopqrstuvwxyz0123456789"),
        ),
        (
            "mixed-classes-12",
            class_policy(
                12,
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$",
                true,
            ),
        ),
        (
            "mixed-classes-18",
            class_policy(
                18,
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*_-+=",
                true,
            ),
        ),
        (
            "variable-length-8-12",
            base_policy(8, 12, "abcdefghijklmnopqrstuvwxyz0123456789"),
        ),
    ];

    let mut edge = class_policy(
        16,
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$",
        true,
    );
    edge.forbidden_first_characters = "!@#$".to_string();
    edge.forbidden_last_characters = "!@#$".to_string();
    corpus.push(("non-symbol-edges-16", edge));

    let mut identical = class_policy(
        14,
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
        false,
    );
    identical.max_identical_run = Some(2);
    corpus.push(("identical-run-at-most-2", identical));

    let mut sequential = class_policy(
        14,
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
        false,
    );
    sequential.max_sequential_run = Some(2);
    corpus.push(("ascii-sequential-run-at-most-2", sequential));

    let mut fixed = base_policy(10, 10, "abcdefghijklmnopqrstuvwxyz0123456789-");
    fixed.fixed_prefix = "A-".to_string();
    fixed.fixed_suffix = "9".to_string();
    fixed.alphabet = "A-abcdefghijklmnopqrstuvwxyz0123456789".to_string();
    corpus.push(("fixed-prefix-and-suffix", fixed));

    let mut substrings = base_policy(12, 12, "ab01");
    substrings.forbidden_substrings = vec!["aa".to_string(), "01".to_string()];
    corpus.push(("forbidden-substrings", substrings));

    let mut substring_stress = base_policy(12, 12, "abcdefghijklmnopqrstuvwxyz0123456789");
    substring_stress.forbidden_substrings = vec![
        "admin".to_string(),
        "password".to_string(),
        "123".to_string(),
    ];
    corpus.push(("forbidden-substrings-state-stress", substring_stress));

    let mut fixed_positions = class_policy(
        12,
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
        false,
    );
    fixed_positions.fixed_characters = vec![
        FixedCharacterConstraint {
            index: 0,
            character: 'A',
        },
        FixedCharacterConstraint {
            index: 11,
            character: '9',
        },
    ];
    corpus.push(("fixed-positions", fixed_positions));

    let mut per_character = base_policy(10, 10, "abcdef012345");
    per_character.max_total_per_character = Some(2);
    corpus.push(("per-character-total-at-most-2", per_character));
    corpus
}

fn base_policy(min_length: usize, max_length: usize, alphabet: &str) -> PolicySpec {
    PolicySpec {
        policy_ir_version: 1,
        min_length,
        max_length,
        alphabet: alphabet.to_string(),
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
    }
}

fn class_policy(length: usize, alphabet: &str, symbol: bool) -> PolicySpec {
    let mut spec = base_policy(length, length, alphabet);
    let mut classes = vec![
        class("lower", "abcdefghijklmnopqrstuvwxyz"),
        class("upper", "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        class("digit", "0123456789"),
    ];
    if symbol {
        classes.push(class("symbol", "!@#$%^&*_-+="));
    }
    spec.classes = classes;
    spec
}

fn class(name: &str, alphabet: &str) -> CharacterClassConstraint {
    CharacterClassConstraint {
        name: name.to_string(),
        alphabet: alphabet.to_string(),
        min_count: 1,
        max_count: None,
    }
}

fn distribution_policy() -> keylesspass_core::error::Result<CompiledPolicy> {
    let alphabet = "ab0123456789".chars().collect::<Vec<_>>();
    let mut transitions = vec![vec![None; alphabet.len()]; 4];
    transitions[0][0] = Some(1); // a
    transitions[0][1] = Some(2); // b
    for index in 2..alphabet.len() {
        transitions[2][index] = Some(3); // b0 ... b9
    }
    let dfa = PolicyDfa::new(
        alphabet,
        transitions,
        0,
        vec![false, true, false, true],
        1,
        2,
    )?;
    CompiledPolicy::from_dfa(base_policy(1, 2, "ab0123456789"), dfa)
}

fn distribution_experiment() -> Result<Value, Box<dyn std::error::Error>> {
    let compiled = distribution_policy()?;
    let published_baselines = vec![measure_dichopile_distribution()?];
    let proposed = vec![measure_epscd_marginal_distribution()?];

    Ok(json!({
        "schemaVersion": 1,
        "policy": "enumerable mixed-length regular language {a,b0,...,b9}",
        "exactSpace": compiled.total_count().to_string(),
        "analyticSpace": 11,
        "samplesPerMethod": DISTRIBUTION_SAMPLES,
        "publishedBaselines": published_baselines,
        "proposed": proposed,
        "boundary": "The EPSCD row uses a test-only uniformly shuffled finite-domain permutation on the enumerable domain. It validates the scheme composition and ideal-permutation marginal; it is not a security benchmark of the FF1 backend."
    }))
}

fn measure_epscd_marginal_distribution() -> Result<Value, Box<dyn std::error::Error>> {
    let policy = distribution_policy()?;
    let domain = policy.total_count().to_usize().ok_or("domain too large")?;
    let mut key_rng = StdRng::seed_from_u64(0x4550_5343_445f_5631);
    let mut frequencies = vec![0_usize; domain];
    for _ in 0..DISTRIBUTION_SAMPLES {
        let mut key = [0_u8; 32];
        key_rng.fill_bytes(&mut key);
        let mut permutation = (0..domain).collect::<Vec<_>>();
        permutation.shuffle(&mut StdRng::from_seed(key));
        let password = policy.unrank(&BigUint::from(permutation[0]))?;
        let rank = policy.rank(&password)?.to_usize().ok_or("rank too large")?;
        frequencies[rank] += 1;
    }
    let mut result = summarize_frequencies("EPSCD-scheme-v1-test-permutation", frequencies);
    let object = result.as_object_mut().unwrap();
    object.insert(
        "backendBoundary".to_string(),
        json!("test-only Fisher-Yates permutation; validates the fixed-generation ideal-permutation marginal, not concrete PRP security"),
    );
    Ok(result)
}

fn mechanistic_controls_experiment() -> Result<Value, Box<dyn std::error::Error>> {
    let policy = distribution_policy()?;
    let domain = policy.total_count().to_usize().ok_or("domain too large")?;
    let uniform_rank =
        measure_distribution("uniform-rank-unrank-oracle", domain, |sample, policy| {
            let mut rng = StdRng::seed_from_u64(sample as u64 ^ 0x0055_4e49_464f_524d);
            policy.unrank(&BigUint::from(rng.gen_range(0..domain)))
        })?;
    Ok(json!({
        "schemaVersion": 1,
        "classification": "mechanistic controls; not competing published schemes",
        "language": ["a", "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9"],
        "uniformRankOracle": uniform_rank,
        "equalViableBranchWalk": {
            "probabilityA": 0.5,
            "probabilityEachBDigit": 0.05,
            "uniformTarget": 1.0 / 11.0,
            "analyticTotalVariationDistance": 9.0 / 22.0
        },
        "withReplacementOccupancy": {
            "draws": 11,
            "domain": 11,
            "expectedUnique": 11.0 * (1.0 - (10.0_f64 / 11.0).powi(11)),
            "expectedRepetitions": 11.0 - 11.0 * (1.0 - (10.0_f64 / 11.0).powi(11))
        }
    }))
}

fn measure_dichopile_distribution() -> Result<Value, Box<dyn std::error::Error>> {
    let policy = distribution_policy()?;
    let domain = policy.total_count().to_usize().ok_or("domain too large")?;
    let generator = ExactDichopile::new(policy.dfa())?;
    let mut rng = StdRng::seed_from_u64(0x4f55_4449_4e45_5432);
    let mut frequencies = vec![0_usize; domain];
    let mut recurrence_calls = Vec::with_capacity(DISTRIBUTION_SAMPLES);
    let mut peak_stack_vectors = Vec::with_capacity(DISTRIBUTION_SAMPLES);
    for _ in 0..DISTRIBUTION_SAMPLES {
        let sample = generator.generate(&mut rng)?;
        let rank = policy
            .rank(&sample.word)?
            .to_usize()
            .ok_or("rank too large")?;
        frequencies[rank] += 1;
        recurrence_calls.push(sample.recurrence_calls as u32);
        peak_stack_vectors.push(sample.peak_stack_vectors as u32);
    }
    let mut result = summarize_frequencies("oudinet-2013-dichopile-exact", frequencies);
    let object = result.as_object_mut().unwrap();
    object.insert(
        "publication".to_string(),
        json!("Theoretical Computer Science 502 (2013), DOI 10.1016/j.tcs.2012.07.025"),
    );
    object.insert(
        "arithmetic".to_string(),
        json!("exact BigUint reproduction of Algorithm 1"),
    );
    object.insert(
        "lengthPreprocessingRecurrenceCalls".to_string(),
        json!(generator.length_preprocessing_calls()),
    );
    object.insert(
        "generationRecurrenceCalls".to_string(),
        integer_summary(recurrence_calls),
    );
    object.insert(
        "peakStackVectors".to_string(),
        integer_summary(peak_stack_vectors),
    );
    Ok(result)
}

fn measure_distribution<F>(
    name: &str,
    domain: usize,
    mut generator: F,
) -> Result<Value, Box<dyn std::error::Error>>
where
    F: FnMut(usize, &CompiledPolicy) -> keylesspass_core::error::Result<String>,
{
    let policy = distribution_policy()?;
    let mut frequencies = vec![0_usize; domain];
    for sample in 0..DISTRIBUTION_SAMPLES {
        let password = generator(sample, &policy)?;
        let rank = policy.rank(&password)?.to_usize().ok_or("rank too large")?;
        frequencies[rank] += 1;
    }
    Ok(summarize_frequencies(name, frequencies))
}

fn summarize_frequencies(name: &str, frequencies: Vec<usize>) -> Value {
    let samples = frequencies.iter().sum::<usize>();
    let domain = frequencies.len();
    let expected = samples as f64 / domain as f64;
    let tvd = frequencies
        .iter()
        .map(|count| (*count as f64 / samples as f64 - 1.0 / domain as f64).abs())
        .sum::<f64>()
        / 2.0;
    let chi_square = frequencies
        .iter()
        .map(|count| {
            let delta = *count as f64 - expected;
            delta * delta / expected
        })
        .sum::<f64>();
    let nonzero = frequencies
        .iter()
        .copied()
        .filter(|count| *count > 0)
        .collect::<Vec<_>>();
    let minimum = nonzero.iter().min().copied().unwrap_or(0);
    let maximum = frequencies.iter().max().copied().unwrap_or(0);
    json!({
        "method": name,
        "totalVariationDistanceEmpirical": tvd,
        "chiSquare": chi_square,
        "chiSquareDegreesOfFreedom": domain - 1,
        "expectedCountPerPassword": expected,
        "observedOutputs": nonzero.len(),
        "missingOutputs": domain - nonzero.len(),
        "minimumNonzeroFrequency": minimum,
        "maximumFrequency": maximum,
        "maxMinFrequencyRatio": if minimum == 0 { Value::Null } else { json!(maximum as f64 / minimum as f64) },
    })
}

fn collision_experiment() -> Result<Value, Box<dyn std::error::Error>> {
    let policy = CompiledPolicy::compile(class_policy(
        16,
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$",
        true,
    ))?;
    let context = evaluation_context(21, [0x63; 16]);
    let mut outputs = HashSet::new();
    for generation in 0..2_000_u64 {
        outputs.insert(
            derive_password(
                &[0x63_u8; 32],
                &context,
                generation,
                &policy,
                &Ff1CycleWalking::default(),
            )?
            .password,
        );
    }
    Ok(json!({
        "schemaVersion": 1,
        "prototypeBackendCheck": {
            "method": "EPSCD-scheme-v1-ff1-cycle-walk-unrank",
            "domainSize": policy.total_count().to_string(),
            "generations": 2000,
            "observedCollisions": 2000 - outputs.len(),
            "boundary": "empirical implementation check; the no-repeat claim follows from successful permutation execution and unrank injectivity, not from this sample"
        }
    }))
}

fn performance_experiment() -> Result<Value, Box<dyn std::error::Error>> {
    let spec = class_policy(
        18,
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*_-+=",
        true,
    );
    let compile_started = Instant::now();
    let compiled = CompiledPolicy::compile(spec.clone())?;
    let compile_duration = compile_started.elapsed();
    let policy_hash = spec.policy_hash()?;
    let root_key = [0x42_u8; 32];
    let context = evaluation_context(1, [0; 16]);

    let mut unrank_times = Vec::new();
    let mut rank_times = Vec::new();
    let mut derive_times = Vec::new();
    let mut epscd_cold_times = Vec::new();
    let dichopile_preprocessing_started = Instant::now();
    let dichopile = ExactDichopile::new(compiled.dfa())?;
    let dichopile_preprocessing_duration = dichopile_preprocessing_started.elapsed();
    let mut dichopile_times = Vec::new();
    let mut dichopile_cold_times = Vec::new();
    let mut dichopile_recurrence_calls = Vec::new();
    let mut dichopile_peak_stack_vectors = Vec::new();
    let mut dichopile_peak_stack_payload_bytes = Vec::new();
    let mut dichopile_rng = StdRng::seed_from_u64(0x4449_4348_4f50_494c);
    let mut walk_counts = Vec::new();
    let permutation = Ff1CycleWalking::default();
    let credential_key = derive_credential_key(&root_key, &context)?;
    let tweak = permutation_tweak(&context, &policy_hash)?;
    for generation in 0..PERFORMANCE_SAMPLES as u64 {
        let rank_value = BigUint::from(generation) % compiled.total_count();
        let started = Instant::now();
        let password = compiled.unrank(&rank_value)?;
        unrank_times.push(started.elapsed());

        let started = Instant::now();
        let recovered = compiled.rank(&password)?;
        rank_times.push(started.elapsed());
        assert_eq!(recovered, rank_value);

        let started = Instant::now();
        derive_password(&root_key, &context, generation, &compiled, &permutation)?;
        derive_times.push(started.elapsed());

        let started = Instant::now();
        let cold_policy = CompiledPolicy::compile(spec.clone())?;
        derive_password(&root_key, &context, generation, &cold_policy, &permutation)?;
        epscd_cold_times.push(started.elapsed());

        let (_, walks) = permutation.permute_with_walk_count(
            &credential_key,
            &tweak,
            compiled.total_count(),
            &BigUint::from(generation),
        )?;
        walk_counts.push(walks);

        let started = Instant::now();
        let dichopile_sample = dichopile.generate(&mut dichopile_rng)?;
        dichopile_times.push(started.elapsed());
        dichopile_recurrence_calls.push(dichopile_sample.recurrence_calls as u32);
        dichopile_peak_stack_vectors.push(dichopile_sample.peak_stack_vectors as u32);
        dichopile_peak_stack_payload_bytes.push(dichopile_sample.peak_stack_payload_bytes as u64);

        let started = Instant::now();
        let cold_dfa = compile_dfa(&spec, DEFAULT_MAX_POLICY_STATES)?;
        let cold_dichopile = ExactDichopile::new(&cold_dfa)?;
        cold_dichopile.generate(&mut dichopile_rng)?;
        dichopile_cold_times.push(started.elapsed());
    }
    Ok(json!({
        "schemaVersion": 1,
        "policySpace": compiled.total_count().to_string(),
        "stateCount": compiled.dfa().state_count(),
        "compileMicros": compile_duration.as_secs_f64() * 1_000_000.0,
        "rank": duration_summary(rank_times),
        "unrank": duration_summary(unrank_times),
        "epscd": {
            "coldCompileAndDerive": duration_summary(epscd_cold_times),
            "warmDeriveFromCachedCompiledPolicy": duration_summary(derive_times)
        },
        "publishedDichopileExact": {
            "publication": "Oudinet, Denise, and Gaudel, TCS 2013",
            "arithmetic": "exact BigUint reproduction",
            "lengthPreprocessingMicros": dichopile_preprocessing_duration.as_secs_f64() * 1_000_000.0,
            "lengthPreprocessingRecurrenceCalls": dichopile.length_preprocessing_calls(),
            "coldDfaCompileInitializeAndGenerate": duration_summary(dichopile_cold_times),
            "warmGenerateFromCachedDfaAndGenerator": duration_summary(dichopile_times),
            "generationRecurrenceCalls": integer_summary(dichopile_recurrence_calls),
            "peakStackVectors": integer_summary(dichopile_peak_stack_vectors),
            "peakStackPayloadBytes": integer_summary_u64(dichopile_peak_stack_payload_bytes)
        },
        "cycleWalking": integer_summary(walk_counts),
        "cycleWalkingTheory": {
            "binarySupersetBits": (compiled.total_count() - BigUint::from(1_u8)).bits(),
            "deterministicWorstCaseWalks": "M-N+1 for M=2^ceil(log2 N); the configured backend instead fails closed after 1024",
            "idealPermutationTailBound": "Pr[W>k] = (M-N)_k/(M)_k <= ((M-N)/M)^k < 2^-k for a non-power-of-two N; for N=M, W=1"
        },
        "boundary": "Cold EPSCD begins from PolicySpec and constructs its DFA and full count table. Cold Dichopile begins from the same PolicySpec, constructs only the shared DFA, initializes exact Dichopile length weights, and generates once. Warm rows reuse their respective compiled structures. The algorithms serve different purposes; these measurements are operational costs of this artifact, not a speedup claim against Oudinet et al.'s optimized configuration. Storage, network, approval, and user delay are excluded."
    }))
}

fn evaluation_context(seed: u128, credential_salt: [u8; 16]) -> CredentialContext {
    CredentialContext {
        scheme_version: SCHEME_VERSION_V1,
        vault_id: Uuid::from_u128(seed),
        service_id: Uuid::from_u128(seed + 1),
        account_id: Uuid::from_u128(seed + 2),
        lineage_id: Uuid::nil(),
        credential_salt,
        root_generation: 1,
        policy_id: Uuid::from_u128(seed + 3),
        policy_version: 1,
        policy_epoch: 1,
    }
}

fn integer_summary(mut values: Vec<u32>) -> Value {
    values.sort_unstable();
    let as_f64 = values.iter().map(|value| *value as f64).collect::<Vec<_>>();
    json!({
        "samples": values.len(),
        "mean": as_f64.iter().sum::<f64>() / as_f64.len() as f64,
        "median": percentile(&as_f64, 0.50),
        "p95": percentile(&as_f64, 0.95),
        "p99": percentile(&as_f64, 0.99),
        "maximum": values.last().copied().unwrap_or(0),
        "oneWalkFraction": values.iter().filter(|value| **value == 1).count() as f64 / values.len() as f64,
    })
}

fn integer_summary_u64(mut values: Vec<u64>) -> Value {
    values.sort_unstable();
    let as_f64 = values.iter().map(|value| *value as f64).collect::<Vec<_>>();
    json!({
        "samples": values.len(),
        "mean": as_f64.iter().sum::<f64>() / as_f64.len() as f64,
        "median": percentile(&as_f64, 0.50),
        "p95": percentile(&as_f64, 0.95),
        "p99": percentile(&as_f64, 0.99),
        "maximum": values.last().copied().unwrap_or(0)
    })
}

fn duration_summary(mut values: Vec<Duration>) -> Value {
    values.sort_unstable();
    let micros = values
        .iter()
        .map(Duration::as_secs_f64)
        .map(|v| v * 1_000_000.0)
        .collect::<Vec<_>>();
    let mean = micros.iter().sum::<f64>() / micros.len() as f64;
    let variance = micros
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / micros.len() as f64;
    json!({
        "samples": micros.len(),
        "meanMicros": mean,
        "medianMicros": percentile(&micros, 0.50),
        "p95Micros": percentile(&micros, 0.95),
        "p99Micros": percentile(&micros, 0.99),
        "standardDeviationMicros": variance.sqrt(),
    })
}

fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    let index = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted[index]
}
