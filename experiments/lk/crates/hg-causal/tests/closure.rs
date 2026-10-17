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
        // v9 C2.3: renamed from seed_spread_bits, and joined by the
        // ensemble field that says what it is a spread OVER (§1.2).
        // Archived reports still load via serde(alias) — pinned by
        // `sampler_spread_rename_keeps_archived_closure_json_readable`.
        "sampler_spread_bits",
        "spread_ensemble",
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
    assert!(
        obj["sampler_spread_bits"].is_null(),
        "per-run reports carry no spread"
    );
    assert!(
        obj["spread_ensemble"].is_null(),
        "no spread means no ensemble to name"
    );

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

/// v6 §1.3 clock-property exclusion (by name): micro(w) reads NO node
/// property — WL classes exclude the property map (Q2 invariant) and the
/// other slots are structural. Two snapshots identical in structure but
/// differing in the countdown key (7), and in other properties, must
/// yield byte-identical micro features. This pins that the monotone
/// countdown clock cannot inflate CD.
#[test]
fn micro_features_ignore_countdown_and_all_properties() {
    use hg_causal::closure::features::micro_features;
    use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
    use smallvec::smallvec;

    // A little filament S–w1–w2–N plus two bulk spurs, built once; the
    // two variants differ ONLY in property values.
    let build = |countdowns: &[i64], energy: f64| -> HypergraphStore {
        let mut s = HypergraphStore::new();
        let mut mk = |st: &mut HypergraphStore, gen: f64, cd: Option<i64>| {
            let mut d = NodeData::new(Label(0), 0);
            d.set_property(1, PropertyValue::Float(energy));
            d.set_property(6, PropertyValue::Float(gen));
            if let Some(c) = cd {
                d.set_property(7, PropertyValue::Int(c));
            }
            st.add_node(d)
        };
        let n0 = mk(&mut s, 0.0, None);
        let w1 = mk(&mut s, 1.0, Some(countdowns[0]));
        let w2 = mk(&mut s, 2.0, Some(countdowns[1]));
        let n3 = mk(&mut s, 0.0, None);
        let a = mk(&mut s, 0.0, None);
        let b = mk(&mut s, 0.0, None);
        s.add_edge(HyperEdge::new(Label(0), smallvec![n0, w1]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![w1, w2]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![w2, n3]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![n0, a]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![n3, b]));
        s
    };
    let s1 = build(&[100, 50], 50.0);
    let s2 = build(&[3, 1], 999.0); // different countdowns AND energy
    let hist = [0.0f64; 6];
    let (sn1, sn2) = (s1.snapshot(), s2.snapshot());
    let c1 = hg_core::compute_wl_class_map_snapshot(&sn1, 1);
    let c2 = hg_core::compute_wl_class_map_snapshot(&sn2, 1);
    let m1 = micro_features(&sn1, &c1, &hist);
    let m2 = micro_features(&sn2, &c2, &hist);
    assert_eq!(
        m1, m2,
        "micro(w) must be invariant to countdown/energy — no property may enter CD"
    );
}

/// v8 §3.3 clock exclusion for the PASSIVE clock, by name. The v6 gate got
/// its exclusion for free: `countdown` was a property, and micro(w) reads
/// no property. The age guard reads `NodeData::generation` — a FIELD, not
/// a property — so that inheritance does not apply and the guarantee has
/// to be pinned on its own channel.
///
/// It holds because the WL base hash covers label + energy field +
/// structure only, deliberately excluding generation (hg-core `hash.rs`).
/// This test asserts the consequence directly: two structurally identical
/// graphs whose nodes were born at different steps, observed at different
/// step counters, yield byte-identical micro features. If someone later
/// adds generation to the WL base hash, this fails rather than quietly
/// letting a monotone clock into CD.
#[test]
fn micro_features_ignore_birth_step_and_age() {
    use hg_causal::closure::features::micro_features;
    use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
    use smallvec::smallvec;

    // Same filament as above; the variants differ ONLY in birth steps and
    // in the observation step — i.e. in every node's AGE.
    let build = |births: [u64; 6], now: u64| -> HypergraphStore {
        let mut s = HypergraphStore::new();
        let mut i = 0usize;
        let mut mk = |st: &mut HypergraphStore, gen: f64| {
            let mut d = NodeData::new(Label(0), births[i]);
            d.set_property(1, PropertyValue::Float(50.0));
            d.set_property(6, PropertyValue::Float(gen));
            i += 1;
            st.add_node(d)
        };
        let n0 = mk(&mut s, 0.0);
        let w1 = mk(&mut s, 1.0);
        let w2 = mk(&mut s, 2.0);
        let n3 = mk(&mut s, 0.0);
        let a = mk(&mut s, 0.0);
        let b = mk(&mut s, 0.0);
        s.add_edge(HyperEdge::new(Label(0), smallvec![n0, w1]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![w1, w2]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![w2, n3]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![n0, a]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![n3, b]));
        s.set_step(now);
        s
    };
    // Arm 1: all newborn at an early clock. Arm 2: a dispersed cohort
    // observed 5000 steps later — ages 0 vs ages up to 5000.
    let s1 = build([0, 0, 0, 0, 0, 0], 0);
    let s2 = build([10, 900, 1750, 30, 4000, 4999], 5000);
    let hist = [0.0f64; 6];
    let (sn1, sn2) = (s1.snapshot(), s2.snapshot());
    let c1 = hg_core::compute_wl_class_map_snapshot(&sn1, 1);
    let c2 = hg_core::compute_wl_class_map_snapshot(&sn2, 1);
    assert_eq!(
        c1.values().collect::<std::collections::BTreeSet<_>>().len(),
        c2.values().collect::<std::collections::BTreeSet<_>>().len(),
        "WL must not mint classes from birth steps"
    );
    let m1 = micro_features(&sn1, &c1, &hist);
    let m2 = micro_features(&sn2, &c2, &hist);
    assert_eq!(
        m1, m2,
        "micro(w) must be invariant to birth_step/age — the passive clock \
         may not enter CD any more than the tick clock could"
    );
}

/// v9 C2.3: `seed_spread_bits` → `sampler_spread_bits`, with the archived
/// name still readable. Both directions are pinned — an archived
/// `closure.json` must load under the old key, and a spread must not be
/// emitted without naming its ensemble.
#[test]
fn sampler_spread_rename_keeps_archived_closure_json_readable() {
    use hg_causal::closure::{ClosureReport, SpreadEnsemble};
    // Exactly the shape docs/leakage_closure_spec.md documents for v3.
    let archived = r#"{
        "cd_raw_bits": 0.041, "cd_clock_bits": 0.017, "n_windows": 412,
        "seed_spread_bits": 0.25, "stratum": "all", "cv_folds": 5,
        "macro_fields": [], "micro_fields": [], "clock_fields": [],
        "certainty": "Inferred", "pathology_negative_cd": false }"#;
    let r: ClosureReport = serde_json::from_str(archived).expect("archived closure.json must load");
    assert_eq!(r.sampler_spread_bits, Some(0.25), "the alias must carry the value");
    assert_eq!(
        r.spread_ensemble, None,
        "archived reports predate the ensemble field and must read as unstated"
    );

    // A freshly written report names its ensemble, and round-trips.
    let fresh = ClosureReport {
        spread_ensemble: Some(SpreadEnsemble::Sampler),
        ..r
    };
    let s = serde_json::to_string(&fresh).unwrap();
    assert!(
        s.contains("\"spread_ensemble\":\"sampler\""),
        "the ensemble must serialize by name: {s}"
    );
    let back: ClosureReport = serde_json::from_str(&s).unwrap();
    assert_eq!(back.spread_ensemble, Some(SpreadEnsemble::Sampler));
    assert_eq!(back.sampler_spread_bits, Some(0.25));
}
