//! S0.2 — substrate-only metric emission + the PAIRED bare-substrate control.
//!
//! Q3 (scope §9): carriers are hypergraph nodes, so on the full graph they
//! inflate `n_nodes`/`d_eff` and a carrier bloom reads as the substrate
//! `Exploding`. `substrate.csv` recomputes the metrics over the substrate
//! subgraph (carriers removed). The load-bearing check: the substrate subgraph of
//! an ALife run must be IDENTICAL to a same-seed bare-substrate control — i.e.
//! carriers do not perturb substrate physics, so substrate metrics stay
//! attributable. (The current biology runs a static substrate, so the control is
//! exact; with an active substrate this test would detect back-reaction.)
#![cfg(feature = "alife")]

use hg_sim_core::SimConfig;
use std::path::Path;

fn config(out: &Path, alife: bool) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "sub"
run_id = "sub"
seed = 777
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
enabled = {alife}
seed_carrier_marker = {alife}
initial_substrate_energy = 5.0
"#,
        out.display()
    );
    SimConfig::from_toml(&toml).unwrap()
}

/// Read a hypersim CSV (provenance line, header, rows) → the column named `col`.
fn column(path: &Path, col: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    let _prov = lines.next();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let idx = header.iter().position(|h| *h == col).unwrap_or_else(|| panic!("no col {col} in {header:?}"));
    lines.map(|l| l.split(',').nth(idx).unwrap().to_string()).collect()
}

#[test]
fn substrate_subgraph_matches_paired_bare_control() {
    let base = std::env::temp_dir().join("hypersim_substrate_ctrl");
    let _ = std::fs::remove_dir_all(&base);

    // Run A: ALife (carriers on). Run B: same seed, carriers off (control).
    let a = hg_sim_core::run_headless(&config(&base.join("a"), true), None, None, None).unwrap();
    let b = hg_sim_core::run_headless(&config(&base.join("b"), false), None, None, None).unwrap();

    // A emits substrate.csv; B (no carriers) does not.
    let sub_csv = a.run_dir.join("substrate.csv");
    assert!(sub_csv.exists(), "ALife run must emit substrate.csv");
    assert!(
        !b.run_dir.join("substrate.csv").exists(),
        "bare control emits no substrate.csv (no carriers)"
    );

    // Column presence.
    let text = std::fs::read_to_string(&sub_csv).unwrap();
    let header = text.lines().nth(1).unwrap();
    for c in ["substrate_n_nodes", "substrate_n_edges", "substrate_regime", "full_n_nodes", "full_regime"] {
        assert!(header.contains(c), "substrate.csv missing `{c}`: {header}");
    }

    // THE paired-control invariant: A's substrate node/edge counts equal the
    // bare control's full-graph counts, window by window.
    let a_sub_nodes = column(&sub_csv, "substrate_n_nodes");
    let a_sub_edges = column(&sub_csv, "substrate_n_edges");
    let a_full_nodes = column(&sub_csv, "full_n_nodes");
    let b_nodes = column(&b.csv_path, "n_nodes");
    let b_edges = column(&b.csv_path, "n_edges");
    assert_eq!(a_sub_nodes.len(), b_nodes.len(), "same window count");
    for w in 0..a_sub_nodes.len() {
        assert_eq!(
            a_sub_nodes[w], b_nodes[w],
            "window {w}: ALife substrate n_nodes must equal the bare control's n_nodes"
        );
        assert_eq!(
            a_sub_edges[w], b_edges[w],
            "window {w}: ALife substrate n_edges must equal the bare control's n_edges"
        );
    }

    // The pollution is real and the filter removes it: the full graph carries
    // MORE nodes than the substrate (carriers), while the substrate == control.
    let sub: usize = a_sub_nodes[0].parse().unwrap();
    let full: usize = a_full_nodes[0].parse().unwrap();
    assert!(full > sub, "full graph ({full}) carries carriers the substrate ({sub}) does not");

    // The substrate never reads Exploding purely because carriers appeared.
    for r in column(&sub_csv, "substrate_regime") {
        assert_ne!(r, "Exploding", "substrate regime must not read Exploding from a carrier bloom");
    }
}
