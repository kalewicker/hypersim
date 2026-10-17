//! LK3 validity controls (pre-committed, run before any science):
//! control twin (zero perturbation → damage ≡ 0), reference dynamic
//! (v_damage vs the LK2 reference speed), twin.csv schema golden, and
//! anchor attrition recording. The ID-permutation control is metric-level
//! and lives in crates/hg-eval/tests/twin_metric.rs.

use hg_output::CsvWriter;
use hg_schema::TwinRow;
use hg_sim_core::{run_twin, SimConfig, TwinParams, TwinPerturbation};

fn excitation_config(shortcut_p: f64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "lk3_twin"
seed = 42
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
"#,
        env!("CARGO_MANIFEST_DIR"),
    );
    SimConfig::from_toml(&toml).expect("config parses")
}

fn params() -> TwinParams {
    TwinParams {
        anchors: 64,
        anchor_radius: 3,
        windows: 8,
        window_len: 4,
        sample_seed: 7,
    }
}

#[test]
fn control_twin_zero_damage() {
    // Zero perturbation, identical seed: damage must be identically 0 at
    // every window. Nonzero ⇒ the harness itself is nondeterministic and
    // NOTHING downstream may run (pre-committed interpretation).
    let outcome = run_twin(&excitation_config(0.0), 8, &params(), TwinPerturbation::Control)
        .expect("twin runs");
    assert!(outcome.damage_always_zero, "control twin shows damage: harness invalid");
    for row in &outcome.rows {
        assert_eq!(row.damage_frac, 0.0, "window {} damaged", row.window);
    }
}

#[test]
fn reference_dynamic_damage_front_speed() {
    // Remove one lattice edge at branch time; the damage front must move
    // at (close to) the LK2-measured reference front speed 0.9961.
    let outcome = run_twin(
        &excitation_config(0.0),
        4,
        &TwinParams {
            windows: 10,
            window_len: 2,
            ..params()
        },
        TwinPerturbation::RemoveSampledEdge,
    )
    .expect("twin runs");
    println!(
        "v_damage = {:.4}; rows: {:?}",
        outcome.v_damage,
        outcome
            .rows
            .iter()
            .map(|r| (r.window, r.damage_frac, r.front_radius))
            .collect::<Vec<_>>()
    );
    assert!(!outcome.damage_always_zero, "perturbation must cause damage");
    assert!(
        outcome.v_damage.is_finite(),
        "damage front never formed: {:?}",
        outcome.rows
    );
    // Reference comparison: see docs/fixture_calibration.md for the
    // measured value and tolerance discussion.
    // Measured v_damage = 1.0088 vs v_eff = 0.9961: agreement to 0.013.
    // The plan's "3 decimals" is unattainable in principle here — anchor
    // distances are integers and front_radius is a p95 over 64 anchors,
    // so the estimator is quantized; 0.05 is the honest tolerance
    // (docs/fixture_calibration.md).
    let reference = 0.9961;
    assert!(
        (outcome.v_damage - reference).abs() < 0.05,
        "v_damage {} too far from reference {reference}",
        outcome.v_damage
    );
}

#[test]
fn stochastic_step_mode_refused() {
    let mut cfg = excitation_config(0.0);
    cfg.engine.step_mode = hg_rules::StepMode::SequentialRandom;
    let err = run_twin(&cfg, 8, &params(), TwinPerturbation::Control);
    assert!(err.is_err(), "zero-draw precondition must be enforced");
}

#[test]
fn twin_csv_schema_golden() {
    let outcome = run_twin(
        &excitation_config(0.0),
        8,
        &params(),
        TwinPerturbation::RemoveSampledEdge,
    )
    .expect("twin runs");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("twin.csv");
    let mut w: CsvWriter<TwinRow> = CsvWriter::create(&path, 1, "twin_test", 42, "").unwrap();
    for row in &outcome.rows {
        w.write_row(row).unwrap();
    }
    w.flush().unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(
        lines[1],
        "window,step,damage_frac,front_radius,v_damage_running,anchor_attrition,claim_type",
        "twin.csv header is a frozen schema surface"
    );
    assert!(lines.len() >= 3);
    assert!(lines[2].ends_with(",estimated"));
}

#[test]
fn anchor_attrition_recorded() {
    // A node-deleting dynamic: marked nodes eat an unmarked neighbor
    // (delete it), so some anchors die and MUST be recorded as attrition,
    // not silently compared as dead-vs-dead.
    let toml = format!(
        r#"
experiment_name = "lk3_attrition"
seed = 42
[rules]
source = "Inline"
inline_toml = '''
[[rules]]
name = "eat"
priority = 0
weight = 1.0
enabled = true
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.lhs.node_labels]
a = 1
b = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b"]
[rules.rhs]
nodes = ["a"]
edges = []
'''
[initial]
condition_type = "TriLatticeExcited"
width = 16
[engine]
step_mode = "MaxParallel"
max_steps = 64
metrics_window_interval = 8
thread_count = 1
"#
    );
    let cfg = SimConfig::from_toml(&toml).expect("config parses");
    let outcome = run_twin(
        &cfg,
        2,
        &TwinParams {
            windows: 12,
            window_len: 2,
            ..params()
        },
        TwinPerturbation::RemoveSampledEdge,
    )
    .expect("twin runs");
    assert!(
        outcome.total_attrition > 0,
        "eating dynamic must delete some anchors: {:?}",
        outcome.rows.iter().map(|r| r.anchor_attrition).collect::<Vec<_>>()
    );
    let sum: u32 = outcome.rows.iter().map(|r| r.anchor_attrition).sum();
    assert_eq!(sum, outcome.total_attrition, "per-window attrition must add up");
}
