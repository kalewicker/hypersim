//! Phase 7/10 exit criterion (v4 §13): a 50k-step run produces a non-empty
//! task lattice and at least one constructor candidate.
//!
//! World: a 4-node chain x—y—w—z where x is deleted/recreated with its
//! label flipping 3↔4 each step. At an odd window length the surviving
//! node y's WL class alternates across window boundaries (a task), while
//! w's class never changes (a persisting catalyst 1 hop from the
//! transformation) — the constructor signature, sustained for ~98 windows.

use hg_sim_core::{run_headless, SimConfig};

#[test]
fn fifty_k_step_run_produces_task_lattice_and_constructor() {
    let dir = tempfile::tempdir().unwrap();
    let config = SimConfig::from_toml(&format!(
        r#"
experiment_name = "fifty_k"
run_id = "fifty_k"
seed = 5
out_dir = "{}"

[rules]
source = "Inline"
inline_toml = '''
[[rules]]
name = "flip_up"
[rules.lhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[rules.lhs.node_labels]
x = 3
y = 0
[rules.rhs]
nodes = ["x2", "y"]
edges = [["x2", "y"]]
[rules.rhs.node_labels]
x2 = 4

[[rules]]
name = "flip_down"
[rules.lhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[rules.lhs.node_labels]
x = 4
y = 0
[rules.rhs]
nodes = ["x2", "y"]
edges = [["x2", "y"]]
[rules.rhs.node_labels]
x2 = 3
'''

[initial]
condition_type = "Empty"

[engine]
max_steps = 50000
metrics_window_interval = 511
max_pairings_per_step = 4
thread_count = 2

[output]
fixed_provenance = true
"#,
        dir.path().display()
    ))
    .unwrap();

    // Seed the chain x(3)—y(0)—w(0)—z(9) via a creation-rule-free path:
    // build it as part of the rule set would over-complicate the grammar,
    // so construct through an inline creation rule instead.
    let mut config = config;
    let inline = config.rules.inline_toml.take().unwrap();
    config.rules.inline_toml = Some(format!(
        r#"{inline}

[[rules]]
name = "seed_world"
[rules.lhs]
nodes = []
edges = []
[rules.rhs]
nodes = ["x", "y", "w", "z"]
edges = [["x", "y"], ["y", "w"], ["w", "z"]]
[rules.rhs.node_labels]
x = 3
y = 0
w = 0
z = 9
[rules.schedule]
mode = "once"
"#
    ));

    let artifacts = run_headless(&config, None, None, None).unwrap();
    assert_eq!(artifacts.final_step, 50_000);

    let m = artifacts.summary.final_metrics.as_ref().unwrap();
    assert!(
        m.task_lattice_depth.unwrap_or(0) >= 1,
        "non-empty task lattice, got {:?}",
        m.task_lattice_depth
    );
    assert!(
        m.constructor_count.unwrap_or(0) >= 1,
        "at least one constructor candidate, got {:?}",
        m.constructor_count
    );
    // The catalyst persisted for ~98 windows: it must be well past
    // Empirical by run end.
    let ecology = &artifacts.summary.constructor_ecology;
    assert!(!ecology.is_empty());
    assert!(
        ecology.iter().any(|(_, v)| v.contains("Law") || v.contains("Empirical")),
        "constructor certainty should reach Empirical+: {ecology:?}"
    );
}
