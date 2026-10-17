//! Campaign guardrail (plan v2 §7): a genuinely exploding run terminates
//! at exactly the k-th consecutive Exploding window, with verdict
//! recorded — the substrate result stands, the host stops paying.

use hg_sim_core::{run_headless, SimConfig};

fn exploding_config(dir: &std::path::Path, k: usize) -> SimConfig {
    // binary_edge on a lattice is the recorded exploding phenotype.
    let toml = format!(
        r#"
experiment_name = "abort_gate"
seed = 42
out_dir = "{}"
[rules]
source = "Builtin"
builtin_name = "binary_edge"
[initial]
condition_type = "TriLattice"
width = 16
[engine]
step_mode = "MaxParallel"
max_steps = 400
metrics_window_interval = 8
thread_count = 1
[leakage]
enabled = false
[campaign.abort]
exploding_windows = {k}
[output]
fixed_provenance = true
"#,
        dir.display()
    );
    SimConfig::from_toml(&toml).expect("config parses")
}

#[test]
fn exploding_run_aborts_at_exactly_window_k() {
    // MEASURED (docs/fixture_calibration.md): binary_edge's Exploding
    // classification is TRANSIENT — a 2-window burst, after which the
    // ratio-based classifier reads GrowthTreadmill even though absolute
    // growth continues. The gate therefore fires only when k <= the
    // burst length; the plan's default k = 3 would sail past this
    // dynamic. Recorded limitation, tested at k = 2.
    let dir = tempfile::tempdir().unwrap();
    let artifacts = run_headless(&exploding_config(dir.path(), 2), None, None, None)
        .expect("aborted run still seals its artifacts");

    // The verdict is recorded and flagged.
    assert!(
        artifacts
            .summary
            .suspicious_flags
            .contains(&"exploding_abort_triggered".to_string()),
        "flags: {:?}",
        artifacts.summary.suspicious_flags
    );
    assert!(
        artifacts.summary.observations.iter().any(|o| o.contains("ExplodingAbort")),
        "observations: {:?}",
        artifacts.summary.observations
    );
    // Manifest sealed Interrupted (the cell did NOT complete).
    assert_eq!(artifacts.summary.provenance.state, "Interrupted");

    // Abort at exactly the k-th consecutive Exploding window: the
    // metrics CSV ends k windows after the first Exploding one.
    let metrics = std::fs::read_to_string(&artifacts.csv_path).unwrap();
    let regime_ix = metrics
        .lines()
        .nth(1)
        .unwrap()
        .split(',')
        .position(|h| h == "regime")
        .expect("regime column");
    let regimes: Vec<String> = metrics
        .lines()
        .skip(2)
        .filter_map(|l| l.split(',').nth(regime_ix).map(str::to_string))
        .collect();
    let first_exploding = regimes.iter().position(|r| r == "Exploding").expect("explodes");
    assert_eq!(
        regimes.len(),
        first_exploding + 2,
        "must stop exactly k = 2 windows after the streak starts: {regimes:?}"
    );
    assert!(
        artifacts.final_step < 400,
        "must abort well before max_steps: {}",
        artifacts.final_step
    );
}

#[test]
fn healthy_run_never_triggers_the_gate() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = exploding_config(dir.path(), 3);
    cfg.rules.builtin_name = Some("glider".into());
    cfg.campaign.abort.exploding_windows = 2;
    cfg.initial.condition_type = hg_sim_core::InitialConditionType::GliderRing;
    cfg.initial.node_count = 64;
    cfg.engine.max_steps = 64;
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    assert!(!artifacts
        .summary
        .suspicious_flags
        .contains(&"exploding_abort_triggered".to_string()));
    assert_eq!(artifacts.summary.provenance.state, "Completed");
}
