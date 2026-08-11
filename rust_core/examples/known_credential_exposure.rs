use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const DOMAIN_SIZE: usize = 11;
const SAMPLES: usize = 100_000;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExposureRow {
    observed_pairs: usize,
    unseen_support_size: usize,
    theoretical_probability_per_unused_rank: f64,
    observed_support_size: usize,
    known_outputs_reappeared: usize,
    total_variation_distance: f64,
    histogram: Vec<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExposureExperiment {
    schema_version: u32,
    model: &'static str,
    domain_size: usize,
    samples_per_q: usize,
    rows: Vec<ExposureRow>,
    claim_boundary: &'static str,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../experiments/known_credential_exposure"));
    let mut rng = StdRng::seed_from_u64(0x4550_5343_445f_4558);
    let mut rows = Vec::new();

    for q in 0..=5 {
        // Condition on the consistent pairs 0->0, ..., q-1->q-1. Every
        // remaining bijection is sampled by uniformly shuffling unused ranks.
        let unused: Vec<_> = (q..DOMAIN_SIZE).collect();
        let target_input = q;
        let mut histogram = vec![0_usize; DOMAIN_SIZE];
        for _ in 0..SAMPLES {
            let mut outputs = unused.clone();
            outputs.shuffle(&mut rng);
            histogram[outputs[target_input - q]] += 1;
        }
        let known_outputs_reappeared = histogram[..q].iter().sum();
        let observed_support_size = histogram.iter().filter(|count| **count > 0).count();
        let expected = 1.0 / (DOMAIN_SIZE - q) as f64;
        let tvd = 0.5
            * histogram[q..]
                .iter()
                .map(|count| (*count as f64 / SAMPLES as f64 - expected).abs())
                .sum::<f64>();
        if known_outputs_reappeared != 0 || observed_support_size != DOMAIN_SIZE - q {
            return Err("conditional permutation support check failed".into());
        }
        rows.push(ExposureRow {
            observed_pairs: q,
            unseen_support_size: DOMAIN_SIZE - q,
            theoretical_probability_per_unused_rank: expected,
            observed_support_size,
            known_outputs_reappeared,
            total_variation_distance: tvd,
            histogram,
        });
    }

    let result = ExposureExperiment {
        schema_version: 1,
        model: "uniform random permutation conditioned on q consistent generation/rank pairs",
        domain_size: DOMAIN_SIZE,
        samples_per_q: SAMPLES,
        rows,
        claim_boundary: "implementation consistency check for ideal residual support; not evidence of concrete FF1 key-recovery resistance",
    };
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("known_credential_exposure.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("wrote q=0..5 conditional-permutation measurements");
    Ok(())
}
