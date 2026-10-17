//! V-track V4.1 — no-`alife` build bit-identity (feature-gate leakage check).
//!
//! Deliberately NOT feature-gated: this test compiles and runs BOTH with and
//! without `--features alife`. It fingerprints a pure-substrate run (a real
//! rewriting ruleset, no ALife anywhere) and prints the canonical hash.
//!
//! The claim under test: the ALife layer is genuinely OPTIONAL. Feature-gate
//! leakage accumulates quietly across stages — a `#[cfg]` that changes a struct
//! layout, an observer that is attached unconditionally, an RNG draw that only
//! happens when the feature is compiled in. If any of that had crept in, the two
//! builds would fingerprint differently even though the ALife layer is disabled.
//!
//! Run:  cargo test -p hg-sim-core --test v4_substrate_fingerprint -- --nocapture
//!       cargo test -p hg-sim-core --features alife --test v4_substrate_fingerprint -- --nocapture
//! and compare. The compile-time half of V4.1 (no `hg-engine` edge from
//! `hg-alife`) is checked with `cargo tree`, outside this file.

use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;

fn substrate_config(steps: u64) -> SimConfig {
    // `glider` is a BOUNDED ruleset (pure mark relabel — no nodes created or
    // deleted), so the fingerprint stays cheap over many steps. A growth rule like
    // `binary_edge` explodes the graph and makes per-step canonical hashing
    // intractable. `SequentialRandom` additionally makes the run consume the engine
    // RNG stream each step, so a feature-gated stray draw would show up here.
    let toml = format!(
        r#"
experiment_name = "v4_fp"
run_id = "v4_fp"
seed = 12345
[rules]
source = "Builtin"
builtin_name = "glider"
[initial]
condition_type = "GliderRing"
width = 6
height = 6
arity = 3
[engine]
step_mode = "SequentialRandom"
max_steps = {steps}
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

#[test]
fn v4_1_substrate_fingerprint() {
    let steps = 300u64;
    let cfg = substrate_config(steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::SequentialRandom, cfg.seed, 100_000);

    // Fold every step's canonical hash into one fingerprint, so a divergence at ANY
    // step is caught, not just the final state.
    let mut fingerprint: u64 = 0xcbf2_9ce4_8422_2325;
    for _ in 1..=steps {
        engine.step(&mut store);
        fingerprint ^= store.canonical_hash();
        fingerprint = fingerprint.wrapping_mul(0x100_0000_01b3);
    }
    let snap = store.snapshot();
    let feature = if cfg!(feature = "alife") { "WITH alife" } else { "WITHOUT alife" };

    eprintln!("\n===== V4.1 SUBSTRATE FINGERPRINT ({feature}) =====");
    eprintln!("  steps          : {steps}");
    eprintln!("  nodes          : {}", snap.node_count());
    eprintln!("  edges          : {}", snap.edge_count());
    eprintln!("  final hash     : {:#018x}", store.canonical_hash());
    eprintln!("  trace fingerprint: {fingerprint:#018x}");
    eprintln!("=================================================\n");
}
