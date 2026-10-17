//! LK4 required tests (plan §7) + the v3 §5 tuple-protocol tests: sign
//! discrimination on synthetic cases, the window floor, blocked
//! (never-shuffled) CV, the negative-CD pathology flag, and the clock
//! baseline (CD_raw vs CD_clock discriminator).

use hg_causal::closure::{
    clock_features, estimate, estimate_stratum, fold_ranges, ClosureSettings, WindowSample,
};
use hg_schema::LeakagePhase;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn settings() -> ClosureSettings {
    ClosureSettings::default()
}

/// Macro-closed synthetic: macro is a deterministic AR(1) plus small
/// noise; micro is independent noise. CD must be ≈ 0.
fn macro_closed(n: usize, seed: u64) -> Vec<WindowSample> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut m = 1.0f64;
    (0..n)
        .map(|i| {
            m = 0.9 * m + 0.1 + 0.01 * rng.gen_range(-1.0..1.0);
            WindowSample {
                macro_v: vec![m, m * 0.5],
                micro_v: (0..4).map(|_| rng.gen_range(-1.0..1.0)).collect(),
                clock_v: clock_features(i, LeakagePhase::Sparse),
            }
        })
        .collect()
}

/// Planted leak: macro(w+1)'s first component is driven by micro(w)[0],
/// which the macro alone cannot see.
fn planted_leak(n: usize, seed: u64) -> Vec<WindowSample> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut hidden: f64 = rng.gen_range(-1.0..1.0);
    let mut m = 0.0f64;
    (0..n)
        .map(|i| {
            let sample = WindowSample {
                macro_v: vec![m, 0.3 * m],
                micro_v: vec![
                    hidden,
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                ],
                clock_v: clock_features(i, LeakagePhase::Sparse),
            };
            m = 0.5 * m + hidden + 0.01 * rng.gen_range(-1.0..1.0);
            hidden = rng.gen_range(-1.0..1.0);
            sample
        })
        .collect()
}

#[test]
fn cd_zero_on_macro_closed_synthetic() {
    let report = estimate(&macro_closed(400, 42), &settings());
    println!("macro-closed CD_raw = {}", report.cd_raw_bits);
    assert_eq!(report.certainty, "Inferred");
    assert!(
        report.cd_raw_bits.abs() < 0.05,
        "CD {} not ≈ 0 on a macro-closed system",
        report.cd_raw_bits
    );
}

#[test]
fn cd_positive_on_planted_leak() {
    // 3σ over seeds: every seed must be decisively positive.
    let cds: Vec<f64> = (0..10u64)
        .map(|s| estimate(&planted_leak(400, 1000 + s), &settings()).cd_raw_bits)
        .collect();
    println!("planted-leak CD_raws = {cds:?}");
    let mean = cds.iter().sum::<f64>() / cds.len() as f64;
    let sd = (cds.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / cds.len() as f64).sqrt();
    assert!(
        mean > 3.0 * sd && mean > 0.5,
        "planted leak not detected at 3σ: mean {mean}, sd {sd}"
    );
}

#[test]
fn cd_unknown_below_min_windows() {
    let report = estimate(&macro_closed(150, 7), &settings());
    assert_eq!(report.certainty, "Unknown", "below 200 windows must be Unknown");
    let report = estimate(&macro_closed(200, 7), &settings());
    assert_eq!(report.certainty, "Inferred");
}

#[test]
fn blocked_cv_never_shuffles() {
    // The fold structure is contiguous, ordered, and covers 0..n exactly
    // once — the testable form of "never shuffle windows".
    for (n, k) in [(10, 5), (203, 5), (7, 5), (400, 5)] {
        let folds = fold_ranges(n, k);
        let mut next = 0;
        for f in &folds {
            assert_eq!(f.start, next, "folds must be contiguous and ordered");
            assert!(f.end >= f.start);
            next = f.end;
        }
        assert_eq!(next, n, "folds must cover every transition exactly once");
    }
}

#[test]
fn cd_nonnegative_or_flagged() {
    // Pure-noise macro: the augmented model can only overfit, so CD lands
    // near 0 and often below — every negative CD must carry the pathology
    // flag (and never the reverse).
    let mut rng = ChaCha8Rng::seed_from_u64(99);
    for trial in 0..10 {
        let samples: Vec<WindowSample> = (0..250)
            .map(|_| WindowSample {
                macro_v: (0..2).map(|_| rng.gen_range(-1.0..1.0)).collect(),
                micro_v: (0..6).map(|_| rng.gen_range(-1.0..1.0)).collect(),
                clock_v: Vec::new(),
            })
            .collect();
        let report = estimate(&samples, &settings());
        assert_eq!(
            report.pathology_negative_cd,
            report.cd_raw_bits < 0.0,
            "trial {trial}: flag must mirror the sign exactly"
        );
    }
}

/// v3 §5 required test — the discriminator: micro = pure monotone trend
/// over a drifting target. The trend predicts macro(w+1) better than the
/// (noise) macro can, so CD_raw > 0; the clock baseline sees the window
/// index, so the same trend buys nothing — CD_clock ≈ 0. A clock is not
/// a leak.
#[test]
fn synthetic_clock_leak_collapses_under_clock_baseline() {
    let mut raws = Vec::new();
    let mut clocks = Vec::new();
    for seed in 0..10u64 {
        let mut rng = ChaCha8Rng::seed_from_u64(5000 + seed);
        let n = 400;
        let samples: Vec<WindowSample> = (0..n)
            .map(|i| WindowSample {
                // Nonstationary target: macro(w) rides a deterministic
                // era-progress drift the macro STATE cannot invert
                // (noise dominates the per-step increment).
                macro_v: vec![0.01 * i as f64 + rng.gen_range(-1.0..1.0)],
                // Micro = the pure monotone trend (plus dead noise dims).
                micro_v: vec![
                    i as f64,
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                ],
                clock_v: clock_features(i, LeakagePhase::Sparse),
            })
            .collect();
        let report = estimate(&samples, &settings());
        raws.push(report.cd_raw_bits);
        clocks.push(report.cd_clock_bits.expect("clock features supplied"));
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let (mean_raw, mean_clock) = (mean(&raws), mean(&clocks));
    println!("clock-leak synthetic: CD_raw mean {mean_raw:.4}, CD_clock mean {mean_clock:.4}");
    assert!(
        mean_raw > 0.1,
        "the monotone micro must beat the macro-only baseline (CD_raw {mean_raw})"
    );
    assert!(
        mean_clock.abs() < 0.2 * mean_raw,
        "the clock baseline must absorb the trend: CD_clock {mean_clock} vs CD_raw {mean_raw}"
    );
}

/// v3 §5 required test: a GENUINE leak (hidden white-noise driver) is
/// not a clock — the sign must survive the clock baseline.
#[test]
fn planted_leak_sign_test_survives_clock_baseline() {
    let cds: Vec<f64> = (0..10u64)
        .map(|s| {
            estimate(&planted_leak(400, 2000 + s), &settings())
                .cd_clock_bits
                .expect("clock features supplied")
        })
        .collect();
    println!("planted-leak CD_clocks = {cds:?}");
    let mean = cds.iter().sum::<f64>() / cds.len() as f64;
    let sd = (cds.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / cds.len() as f64).sqrt();
    assert!(
        mean > 3.0 * sd && mean > 0.5,
        "a genuine leak must persist under the clock baseline: mean {mean}, sd {sd}"
    );
}

/// v3 §5 required test: the closure.json tuple schema is frozen
/// (Appendix B). Field renames/removals are spec revisions and must
/// break here first.
#[test]
fn cd_tuple_schema_golden() {
    let report = estimate(&macro_closed(250, 3), &settings());
    let json = serde_json::to_value(&report).unwrap();
    let obj = json.as_object().unwrap();
    let expected = [
        "cd_raw_bits",
        "cd_clock_bits",
        "n_windows",
        "seed_spread_bits",
        "stratum",
        "cv_folds",
        "macro_fields",
        "micro_fields",
        "clock_fields",
        "certainty",
        "pathology_negative_cd",
    ];
    for key in expected {
        assert!(obj.contains_key(key), "closure.json missing frozen key '{key}'");
    }
    assert_eq!(obj.len(), expected.len(), "unexpected extra keys: {obj:?}");
    assert_eq!(obj["stratum"], "all");
    assert!(obj["seed_spread_bits"].is_null(), "per-run reports carry no spread");

    // The stratified variant tags itself.
    let samples = macro_closed(250, 3);
    let mask: Vec<bool> = (0..samples.len()).map(|i| i % 2 == 0).collect();
    let strat = estimate_stratum(&samples, &mask, "sparse_only", &settings());
    assert_eq!(strat.stratum, "sparse_only");
    // Alternating mask retains no adjacent pairs: nothing to fit, and
    // the report must say Unknown rather than fabricate.
    assert_eq!(strat.n_windows, 0);
    assert_eq!(strat.certainty, "Unknown");
}
