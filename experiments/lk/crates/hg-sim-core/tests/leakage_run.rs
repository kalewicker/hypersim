//! LK1 exit criteria at the pipeline level: live runs on the canonical
//! rules produce a populated leakage.csv; the CSV schema is golden; rows
//! are deterministic under a fixed seed.

use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

fn config(rule: &str, dir: &Path, seed: u64) -> SimConfig {
    // The λ* baseline (wolfram_analog) ships as a rules/ file, not a
    // builtin; load it by path.
    let rules_section = if rule == "wolfram_analog" {
        format!(
            "source = \"File\"\nfile_path = \"{}/../../rules/wolfram_analog.toml\"",
            env!("CARGO_MANIFEST_DIR")
        )
    } else {
        format!("source = \"Builtin\"\nbuiltin_name = \"{rule}\"")
    };
    let toml = format!(
        r#"
experiment_name = "lk1_{rule}"
seed = {seed}
out_dir = "{}"

[rules]
{rules_section}

[initial]
condition_type = "PureCrystal"
width = 12
height = 12

[engine]
max_steps = 48
metrics_window_interval = 16
thread_count = 1

[leakage]
enabled = true
cadence_steps = 16

[output]
fixed_provenance = true
"#,
        dir.display()
    );
    SimConfig::from_toml(&toml).expect("config parses")
}

fn leakage_lines(run_dir: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(run_dir.join("leakage.csv")).expect("leakage.csv");
    content.lines().map(str::to_string).collect()
}

#[test]
fn leakage_csv_schema_golden() {
    let dir = tempfile::tempdir().unwrap();
    let artifacts = run_headless(&config("triangle_subdivide", dir.path(), 42), None, None, None)
        .expect("run");
    let lines = leakage_lines(&artifacts.run_dir);
    assert!(lines[0].starts_with("#hypersim_csv_v=1,"), "schema comment line");
    // v2 schema (hypersim_leakage_plan_v2.md Appendix A) + the v4
    // Appendix B audit extension (e_fused_total, churn_events) + the v7
    // §1.1 instrument-range precondition (census_in_range).
    // Changing this again is a plan revision, not a code decision.
    // NOTE the v7 addition is inserted BEFORE claim_type (which stays
    // last by convention), so it shifts claim_type from index 19 to 20;
    // every analysis driver in-tree reads indices ≤ 18 and is unaffected.
    assert_eq!(
        lines[1],
        "step,detour_p50,detour_p95,wormhole_frac,cut_exponent,cut_fit_r2,\
         area_law_defect,d_bfs,d_spec,dim_mismatch,wormhole_count_est,\
         wormhole_components,phase,cone_status,e_total_now,n_subdivisions,\
         e_fused_total,churn_events,audit_residual,census_in_range,claim_type"
            .replace(' ', ""),
        "leakage.csv header is a frozen schema surface"
    );
    assert!(lines.len() > 2, "at least one data row");
    for row in &lines[2..] {
        assert!(row.ends_with(",estimated"), "every row carries claim_type: {row}");
    }
}

#[test]
fn canonical_rules_produce_populated_leakage_csv() {
    // The three canonical rules from the LK1 exit criterion (the λ*
    // baseline builtin is wolfram_analog).
    for rule in ["wolfram_analog", "triangle_subdivide", "binary_edge"] {
        let dir = tempfile::tempdir().unwrap();
        let artifacts =
            run_headless(&config(rule, dir.path(), 42), None, None, None).expect("run");
        let lines = leakage_lines(&artifacts.run_dir);
        assert!(
            lines.len() >= 3,
            "{rule}: expected populated leakage.csv, got {} lines",
            lines.len()
        );
        // Every emission carries a real detour census (p50 >= 1 or the
        // graph had no eligible edges, which these rules never produce).
        let first = &lines[2];
        let p50: f32 = first.split(',').nth(1).unwrap().parse().unwrap();
        assert!(p50 >= 1.0, "{rule}: degenerate detour census: {first}");
    }
}

#[test]
fn leakage_rows_deterministic_under_seed() {
    let (a, b) = {
        let d1 = tempfile::tempdir().unwrap();
        let d2 = tempfile::tempdir().unwrap();
        let r1 = run_headless(&config("triangle_subdivide", d1.path(), 7), None, None, None)
            .unwrap();
        let r2 = run_headless(&config("triangle_subdivide", d2.path(), 7), None, None, None)
            .unwrap();
        (leakage_lines(&r1.run_dir), leakage_lines(&r2.run_dir))
    };
    assert_eq!(a, b, "leakage.csv must be byte-identical for a fixed seed");
}

#[test]
fn leakage_disabled_writes_no_csv() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = config("triangle_subdivide", dir.path(), 42);
    cfg.leakage.enabled = false;
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    assert!(!artifacts.run_dir.join("leakage.csv").exists());
}

#[test]
fn tracked_csv_written_for_planted_arms() {
    // LK1R runner wiring: a TriLattice run with [leakage.tracked] on
    // produces tracked.csv whose first row counts exactly the planted
    // population (the excitation dynamic is pure-relabel, so the lineage
    // never changes size).
    let dir = tempfile::tempdir().unwrap();
    let toml = format!(
        r#"
experiment_name = "lk1r_tracked"
seed = 42
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/excitation_front.toml"
[initial]
condition_type = "TriLatticeExcited"
width = 32
defect_density = 0.01
[engine]
step_mode = "MaxParallel"
max_steps = 32
metrics_window_interval = 8
thread_count = 1
[evaluator]
enabled = false
[causal]
enabled = false
[leakage]
enabled = false
[leakage.tracked]
enabled = true
recompute_cadence = 8
[output]
fixed_provenance = true
"#,
        dir.path().display(),
        env!("CARGO_MANIFEST_DIR"),
    );
    let cfg = SimConfig::from_toml(&toml).expect("config parses");
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let content =
        std::fs::read_to_string(artifacts.run_dir.join("tracked.csv")).expect("tracked.csv");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(
        lines[1],
        // v3 §4 / Appendix B schema delta (2026-07-15): + mouth_energy_max,
        // t_far_nucleus, t_ambient_front_at_far (EXP-L10 encounter columns).
        "window,step,tracked_edge_count,tracked_components,tracked_wormhole_count,\
         tracked_detour_p50,tracked_detour_p95,throat_length_max,far_nucleus_flag,\
         mouth_energy_max,t_far_nucleus,t_ambient_front_at_far,claim_type"
            .replace(' ', ""),
        "tracked.csv header is a frozen schema surface"
    );
    assert!(lines.len() >= 4, "one row per cadence window");
    // 32² torus, p = 0.01: round(0.01 · 3072) = 31 planted shortcuts,
    // constant under a pure-relabel dynamic.
    for row in &lines[2..] {
        let count: u32 = row.split(',').nth(2).unwrap().parse().unwrap();
        assert_eq!(count, 31, "planted population constant under relabel: {row}");
    }
}
