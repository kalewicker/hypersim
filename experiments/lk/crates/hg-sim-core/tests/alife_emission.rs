//! S0.2 / S5.3 emission gate: an ALife `run_headless` writes a SEPARATE
//! `ecology.csv` (lineage + ecology per window), while `metrics.csv` stays a
//! normal run artifact. The separate-file design keeps the no-feature / flag-off
//! `metrics.csv` byte-identical (exit criterion 7); this test confirms the ALife
//! side actually emits.
#![cfg(feature = "alife")]

use hg_sim_core::SimConfig;

fn config(out: &std::path::Path, alife_enabled: bool) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "emit"
run_id = "emit"
seed = 12345
out_dir = "{}"
[rules]
source = "Inline"
inline_toml = "rules = []"
[initial]
condition_type = "PureCrystal"
width = 6
height = 6
arity = 3
[engine]
step_mode = "MaxParallel"
max_steps = 3000
metrics_window_interval = 500
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
[alife]
enabled = {alife_enabled}
seed_carrier_marker = {alife_enabled}
initial_substrate_energy = 5.0
"#,
        out.display()
    );
    SimConfig::from_toml(&toml).unwrap()
}

#[test]
fn alife_run_emits_ecology_csv_and_leaves_metrics_csv() {
    let tmp = std::env::temp_dir().join("hypersim_ecology_emit");
    let _ = std::fs::remove_dir_all(&tmp);
    let cfg = config(&tmp, true);
    let out = hg_sim_core::run_headless(&cfg, None, None, None).unwrap();

    // metrics.csv still exists (normal artifact).
    assert!(out.csv_path.exists(), "metrics.csv must still be written");

    // ecology.csv exists next to it.
    let eco = out.run_dir.join("ecology.csv");
    assert!(eco.exists(), "ecology.csv must be written for an ALife run");

    let text = std::fs::read_to_string(&eco).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    // provenance header + column header + one row per window (6 windows @ 500).
    assert!(lines[0].starts_with("#hypersim_csv_v="), "provenance header present");
    let header = lines[1];
    for col in [
        "window_id",
        "carriers",
        "carrier_ratio",
        "births",
        "deaths",
        "non_trivial_replicators",
        "repl_threshold_mean",
        "satiation_mean",
    ] {
        assert!(header.contains(col), "ecology.csv header missing column `{col}`: {header}");
    }
    let data_rows = lines.len() - 2;
    assert_eq!(data_rows, 6, "one ecology row per 500-step window over 3000 steps");

    // The last row should reflect a live carrier population that reproduced.
    let last = lines.last().unwrap();
    let fields: Vec<&str> = last.split(',').collect();
    let carriers: usize = fields[2].parse().unwrap();
    assert!(carriers > 0, "a live carrier population at the final window");
}

#[test]
fn no_ecology_csv_when_alife_flag_off() {
    // Feature is compiled in, but the flag is off ⟹ no carriers, no ecology.csv,
    // and the run is a plain substrate run.
    let tmp = std::env::temp_dir().join("hypersim_ecology_emit_off");
    let _ = std::fs::remove_dir_all(&tmp);
    let cfg = config(&tmp, false);
    let out = hg_sim_core::run_headless(&cfg, None, None, None).unwrap();
    assert!(out.csv_path.exists(), "metrics.csv written");
    assert!(
        !out.run_dir.join("ecology.csv").exists(),
        "no ecology.csv when the alife flag is off (no carriers emitted)"
    );
}
