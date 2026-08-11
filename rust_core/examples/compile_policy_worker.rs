use keylesspass_core::policy::{CompiledPolicy, PolicySpec, DEFAULT_MAX_POLICY_STATES};
use num_bigint::BigUint;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const PROBE_SAMPLES: usize = 31;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let corpus_path = PathBuf::from(args.next().ok_or("missing corpus path")?);
    let source_row = args.next().ok_or("missing source row")?.parse::<u64>()?;
    if args.next().is_some() {
        return Err("unexpected worker argument".into());
    }

    let corpus: Value = serde_json::from_slice(&fs::read(corpus_path)?)?;
    let record = corpus["records"]
        .as_array()
        .ok_or("missing corpus records")?
        .iter()
        .find(|record| record["sourceRow"].as_u64() == Some(source_row))
        .ok_or("source row not found")?;
    if record["translationStatus"] != "translated" {
        return Err("worker received a non-translated policy".into());
    }
    let spec: PolicySpec = serde_json::from_value(record["policySpec"].clone())?;
    let started = Instant::now();
    match CompiledPolicy::compile_with_limit(spec.clone(), DEFAULT_MAX_POLICY_STATES) {
        Ok(compiled) => {
            let compile_micros = started.elapsed().as_secs_f64() * 1_000_000.0;
            let mut rank_times = Vec::with_capacity(PROBE_SAMPLES);
            let mut unrank_times = Vec::with_capacity(PROBE_SAMPLES);
            for sample in 1..=PROBE_SAMPLES {
                let rank = (compiled.total_count() * BigUint::from(sample))
                    / BigUint::from(PROBE_SAMPLES + 1);
                let unrank_started = Instant::now();
                let password = compiled.unrank(&rank)?;
                unrank_times.push(unrank_started.elapsed());
                let rank_started = Instant::now();
                let recovered = compiled.rank(&password)?;
                rank_times.push(rank_started.elapsed());
                if recovered != rank {
                    return Err("rank/unrank probe mismatch".into());
                }
            }
            println!(
                "{}",
                serde_json::to_string(&json!({
                    "sourceRow": source_row,
                    "website": record["website"],
                    "translationStatus": "translated",
                    "compileStatus": "SUCCESS",
                    "minLength": spec.min_length,
                    "maxLength": spec.max_length,
                    "alphabetSize": spec.alphabet.chars().count(),
                    "reachableStates": compiled.dfa().state_count(),
                    "countTableCells": compiled.table_cell_count(),
                    "countPayloadBytes": compiled.count_payload_bytes(),
                    "compileMicros": compile_micros,
                    "exactSpace": compiled.total_count().to_string(),
                    "entropyBits": compiled.entropy_bits(),
                    "rankMicros": duration_summary(rank_times),
                    "unrankMicros": duration_summary(unrank_times),
                }))?
            );
        }
        Err(error) => {
            let message = error.to_string();
            let status = if message.contains("state limit exceeded") {
                "STATE_LIMIT"
            } else if message.contains("language is empty") {
                "EMPTY_LANGUAGE"
            } else {
                "INTERNAL_ERROR"
            };
            println!(
                "{}",
                serde_json::to_string(&json!({
                    "sourceRow": source_row,
                    "website": record["website"],
                    "translationStatus": "translated",
                    "compileStatus": status,
                    "minLength": spec.min_length,
                    "maxLength": spec.max_length,
                    "alphabetSize": spec.alphabet.chars().count(),
                    "compileMicros": started.elapsed().as_secs_f64() * 1_000_000.0,
                    "error": message,
                }))?
            );
        }
    }
    Ok(())
}

fn duration_summary(mut values: Vec<Duration>) -> Value {
    values.sort_unstable();
    let micros = values
        .iter()
        .map(Duration::as_secs_f64)
        .map(|value| value * 1_000_000.0)
        .collect::<Vec<_>>();
    json!({
        "samples": micros.len(),
        "median": percentile(&micros, 0.50),
        "p95": percentile(&micros, 0.95),
        "p99": percentile(&micros, 0.99),
    })
}

fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    let index = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted[index]
}
