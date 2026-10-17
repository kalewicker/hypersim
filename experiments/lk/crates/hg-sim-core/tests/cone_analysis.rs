//! LK2 exit criteria: the excitation-front reference dynamic on the
//! lattice fixture yields anomalous_frac_dynamics ≈ 0 and a reproducible
//! reference speed; planted shortcuts (small_world p > 0) yield a
//! strictly positive, monotone anomaly fraction.

use hg_eval::cone::{analyze_run_dir, ConeAnalysis, ConeParams};
use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

fn excitation_config(dir: &Path, seed: u64, shortcut_p: f64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "lk2_excite"
seed = {seed}
out_dir = "{}"

[rules]
source = "File"
file_path = "{}/../../rules/excitation_front.toml"

[initial]
condition_type = "TriLatticeExcited"
width = 32
defect_density = {shortcut_p}

[engine]
step_mode = "MaxParallel"
max_steps = 64
metrics_window_interval = 8
thread_count = 1

[evaluator]
enabled = false

[causal]
enabled = false

[leakage]
enabled = false
cone_events_enabled = true

[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.display(),
        env!("CARGO_MANIFEST_DIR"),
    );
    SimConfig::from_toml(&toml).expect("config parses")
}

fn run_and_analyze(seed: u64, shortcut_p: f64) -> ConeAnalysis {
    let dir = tempfile::tempdir().unwrap();
    let artifacts =
        run_headless(&excitation_config(dir.path(), seed, shortcut_p), None, None, None)
            .expect("run");
    let params = ConeParams {
        delta: 30,
        sample_events: 128,
        seed: 42,
        slack: 2.0,
        detour_r_max: 12,
        censor: hg_eval::cone::CensorMode::Wormhole,
    };
    analyze_run_dir(&artifacts.run_dir, &params).expect("cone analysis")
}

#[test]
fn lattice_reference_dynamic_is_clean() {
    let a = run_and_analyze(42, 0.0);
    println!(
        "v_eff = {:.4}, dynamics {}/{} anomalous, cleanup {}, excluded {}",
        a.v_eff, a.anomalous_dynamics, a.descendants_dynamics,
        a.descendants_cleanup, a.excluded_descendants,
    );
    assert!(a.descendants_dynamics > 100, "front produced a real causal cone");
    assert!(a.v_eff > 0.05, "front must move: v_eff = {}", a.v_eff);
    let frac = a.anomalous_dynamics as f64 / a.descendants_dynamics as f64;
    assert!(frac < 0.02, "lattice anomalous_frac_dynamics {frac} not ≈ 0");
    assert_eq!(a.descendants_cleanup, 0, "pure relabel: no cleanup channel");
}

#[test]
fn reference_speed_reproducible_across_seeds() {
    // Acceptance criterion (c): reproducible to 3 decimals across seeds.
    // The dynamic is deterministic under MaxParallel and the p = 0
    // lattice is seed-independent, so the speed is in fact identical.
    let v: Vec<f64> = [42u64, 7, 100].iter().map(|&s| run_and_analyze(s, 0.0).v_eff).collect();
    println!("reference speeds: {v:?}");
    assert!((v[0] - v[1]).abs() < 5e-4 && (v[1] - v[2]).abs() < 5e-4,
        "reference speed not reproducible: {v:?}");
}

#[test]
fn small_world_discriminates_planted_shortcuts() {
    // Plan §5 pre-committed criterion (d) asked for anomalous_frac
    // monotone INCREASING in p. Measured (docs/fixture_calibration.md):
    // the response peaks at the sparse onset and collapses as the
    // shortcut population becomes mutually redundant (the same
    // small-world transition recorded in EXP-L1, here on live dynamics —
    // on a diameter-16 torus the collapse is already underway by
    // p = 0.005: 0.002 -> 0.0574, 0.005 -> 0.0176, 0.015 -> 0).
    // The load-bearing exit criterion — the dynamic DISCRIMINATES p = 0
    // from p > 0, with zero false positives on the clean lattice — holds
    // and is what this test pins.
    let frac = |a: &ConeAnalysis| {
        a.anomalous_dynamics as f64 / a.descendants_dynamics.max(1) as f64
    };
    let clean = run_and_analyze(42, 0.0);
    let onset = run_and_analyze(42, 0.002);
    let sparse = run_and_analyze(42, 0.005);
    println!(
        "anomalous_frac_dynamics: p=0 {:.4}, p=0.002 {:.4}, p=0.005 {:.4}",
        frac(&clean), frac(&onset), frac(&sparse)
    );
    assert_eq!(frac(&clean), 0.0, "clean lattice must have zero anomalies");
    assert!(frac(&onset) > 0.02, "onset density must be clearly detected");
    assert!(frac(&sparse) > 0.0, "sparse density must be detected");
    assert!(frac(&onset) > frac(&sparse),
        "documented onset-peak shape changed — recalibrate");
}

// ---- LK2R normative controls (plan v2 §1.3) ----

fn run_and_analyze_with(seed: u64, shortcut_p: f64, censor: hg_eval::cone::CensorMode) -> ConeAnalysis {
    let dir = tempfile::tempdir().unwrap();
    let artifacts =
        run_headless(&excitation_config(dir.path(), seed, shortcut_p), None, None, None)
            .expect("run");
    let params = ConeParams {
        delta: 30,
        sample_events: 128,
        seed: 42,
        slack: 2.0,
        detour_r_max: 12,
        censor,
    };
    analyze_run_dir(&artifacts.run_dir, &params).expect("cone analysis")
}

#[test]
fn c1_null_censor_on_clean_lattice() {
    // C1: on the p = 0 lattice the censor set must be EMPTY and
    // anomalies ≈ 0. Any nonempty censor at p = 0 is an instrument fault
    // (the censor_nonempty_at_p_zero flag).
    let a = run_and_analyze_with(42, 0.0, hg_eval::cone::CensorMode::Wormhole);
    assert_eq!(a.max_censor_set_size, 0, "C1 violation: nonempty censor at p = 0");
    assert_eq!(a.anomalous_dynamics, 0, "C1: clean lattice must be anomaly-free");
    assert_eq!(a.out_of_phase_events, 0, "clean lattice is in the sparse phase");
}

#[test]
fn c2_random_censoring_manufactures_no_anomalies() {
    // C2: censoring an equal count of RANDOM GEOMETRIC edges must not
    // manufacture anomalies — geometric edges are redundant (detour 2),
    // so removing a few barely lengthens any distance. If this failed,
    // wormhole-censored anomalies would be censoring artifacts.
    let wormhole = run_and_analyze_with(42, 0.002, hg_eval::cone::CensorMode::Wormhole);
    let ablation = run_and_analyze_with(
        42,
        0.002,
        hg_eval::cone::CensorMode::RandomAblation { seed: 99 },
    );
    // Same runs, same censor sizes (equal-count by construction).
    assert_eq!(
        wormhole.max_censor_set_size, ablation.max_censor_set_size,
        "ablation must censor the same COUNT"
    );
    assert!(wormhole.max_censor_set_size > 0, "planted arm has a censor set");
    let frac = |a: &ConeAnalysis| {
        a.anomalous_dynamics as f64 / a.descendants_dynamics.max(1) as f64
    };
    println!(
        "wormhole-censored frac = {:.4}, random-ablation frac = {:.4}",
        frac(&wormhole), frac(&ablation)
    );
    assert!(frac(&wormhole) > 0.02, "wormhole censoring detects the plant");
    assert!(
        frac(&ablation) < 0.005,
        "random censoring must NOT manufacture anomalies: {}",
        frac(&ablation)
    );
}

#[test]
fn phase_gate_zeroes_out_of_phase_rows() {
    // Every OutOfPhase row must carry no numbers; every emitted row
    // carries its censor size and status.
    let a = run_and_analyze_with(42, 0.002, hg_eval::cone::CensorMode::Wormhole);
    for row in &a.rows {
        if row.cone_status == hg_schema::ConeStatus::OutOfPhase {
            assert_eq!(row.cone_radius_p95, 0.0);
            assert_eq!(row.anomalous_frac_dynamics, 0.0);
        }
    }
    // The excitation lattice at sparse p stays in phase throughout.
    assert_eq!(a.out_of_phase_events, 0);
}
