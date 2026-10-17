//! Canonical-hash regression for the adjacency-guided matcher: these
//! hashes were captured on the PRE-optimization matcher (2026-07-16,
//! all-edges candidate scan). The optimization must be trajectory-
//! invariant — candidate ORDER may change, the selected event set may
//! not (the engine canonically sorts bindings before selection).

use hg_sim_core::{run_headless, SimConfig};

fn run_hash(rules_file: &str, fuse: bool) -> u64 {
    let dir = tempfile::tempdir().unwrap();
    let toml = format!(
        r#"
experiment_name = "hashpin"
seed = 42
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../{rules_file}"
[initial]
condition_type = "TriLattice"
width = 16
[engine]
step_mode = "MaxParallel"
max_steps = 400
metrics_window_interval = 100
thread_count = 1
[lk_rules]
enabled = true
e_total = 20000.0
g_max = 1.0e18
[lk_rules.fuse]
enabled = {fuse}
[output]
fixed_provenance = true
"#,
        dir.path().display(),
        env!("CARGO_MANIFEST_DIR"),
    );
    let cfg = SimConfig::from_toml(&toml).unwrap();
    run_headless(&cfg, None, None, None).unwrap().canonical_hash
}

#[test]
fn canonical_hashes_unchanged_by_matcher_optimization() {
    assert_eq!(run_hash("rules/lk_pair.toml", false), 0xc7c1e92817914349, "lk_pair trajectory changed");
    assert_eq!(run_hash("rules/lk_pair_capped_inf.toml", true), 0xc83da304abc246bc, "fuse trajectory changed");
    assert_eq!(run_hash("rules/glider.toml", false), 0xd16d714add3e5e52, "glider trajectory changed");
}
