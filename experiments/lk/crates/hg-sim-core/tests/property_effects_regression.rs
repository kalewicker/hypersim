//! The zero-change proof (B6 test 11): rulesets WITHOUT property_effects
//! behave byte-identically to pre-feature behavior. The expected hashes
//! below were recorded by running these exact configs on the pre-feature
//! build (2026-07-09, before property_effects landed). Any drift means an
//! accidental RNG draw or an apply-order perturbation — a bug, not
//! something to rationalize.
//!
//! The glider config runs SequentialRandom, so its hash additionally pins
//! the RNG-stream guarantee (selection draws must be unshifted). Note the
//! glider's canonical hash is position-invariant (a marked symmetric ring
//! is isomorphic at every position), so it pins structure + stream-driven
//! structural divergence; the direct stream pin is the zero-draw test in
//! hg-engine.

use hg_sim_core::{run_headless, SimConfig};

/// Recorded pre-feature: triangle_subdivide, MaxParallel, seed 42,
/// 48 steps, 6x6 crystal, pairings 4, 2 threads.
const PRE_FEATURE_TRI_HASH: u64 = 0x526d68a26e84d7a9;
/// Recorded pre-feature: glider, SequentialRandom, seed 42, 256 steps,
/// ring 24, 1 thread.
const PRE_FEATURE_GLIDER_HASH: u64 = 0xcaa02b792c679980;

fn run_hash(toml: &str, name: &str) -> u64 {
    let mut config = SimConfig::from_toml(toml).expect("config parses");
    let dir = std::env::temp_dir().join(format!("hypersim_pe_regress_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    config.out_dir = dir;
    let artifacts = run_headless(&config, None, None, None).expect("run completes");
    artifacts.canonical_hash
}

#[test]
fn triangle_subdivide_hash_unchanged_by_feature() {
    let toml = r#"
experiment_name = "tri_small"
run_id = "tri_small"
seed = 42

[rules]
source = "Builtin"
builtin_name = "triangle_subdivide"

[initial]
condition_type = "PureCrystal"
width = 6
height = 6
arity = 3

[engine]
step_mode = "MaxParallel"
max_steps = 48
metrics_window_interval = 12
max_pairings_per_step = 4
thread_count = 2

[evaluator]
enabled = false

[causal]
enabled = false
"#;
    assert_eq!(
        run_hash(toml, "tri"),
        PRE_FEATURE_TRI_HASH,
        "triangle_subdivide final canonical hash drifted from the \
pre-property_effects recording — apply-order or RNG perturbation"
    );
}

#[test]
fn glider_hash_and_rng_stream_unchanged_by_feature() {
    let toml = r#"
experiment_name = "glider_small"
run_id = "glider_small"
seed = 42

[rules]
source = "Builtin"
builtin_name = "glider"

[initial]
condition_type = "GliderRing"
node_count = 24

[engine]
step_mode = "SequentialRandom"
max_steps = 256
metrics_window_interval = 64
max_pairings_per_step = 1000
thread_count = 1

[evaluator]
enabled = false

[causal]
enabled = false
"#;
    assert_eq!(
        run_hash(toml, "glider"),
        PRE_FEATURE_GLIDER_HASH,
        "glider (SequentialRandom) canonical hash drifted — the seeded \
RNG stream shifted, violating the zero-draw guarantee"
    );
}
