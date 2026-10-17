//! C5 cost probe, second pass — horizon scaling on the composed substrates.
//!
//! The first Stage 2 campaign was sized from the budget probe's 192-step
//! numbers by assuming cost scales with the horizon. It does not: node
//! count compounds and candidate matching is superlinear in graph size, so
//! a 2× horizon is far more than 2× the work. The campaign at 384 steps
//! burned 178 CPU-minutes without finishing generation 0 of 40.
//!
//! This measures the curve instead of extrapolating it, and reports the
//! quantity a campaign is actually sized by: seconds per rollout, and how
//! many windows survive the burn-in.

use hg_cognition::{Parameterized, StaticBias};
use hg_sim_core::{run_rollout, RolloutRequest};
use hg_train::diagnostics::{Fixture, RuleSource};

const BURN_IN: usize = 5;
const K: usize = 16;

fn fixture(name: &'static str, path: &'static str, ic: &'static str, steps: u64, window: u64) -> Fixture {
    Fixture {
        name,
        rules: RuleSource::File(path),
        ic,
        width: 8,
        node_count: 24,
        steps,
        window,
        step_mode: "MaxParallel",
    }
}

#[test]
#[ignore = "long-running; run explicitly when re-sizing Stage 2"]
fn composed_substrate_horizon_scaling() {
    let dir = tempfile::tempdir().expect("tempdir");
    println!("\n=== C5 cost probe 2 — composed substrates at k = {K} ===");
    println!(
        "{:<10} {:>7} {:>7} {:>9} {:>10} {:>8} {:>12}",
        "fixture", "steps", "window", "wall (s)", "final n", "scored", "gen (s, 8x)"
    );

    for (name, path, ic) in [
        ("wn_three", "rules/wn_three.toml", "WhiteNoise"),
        ("tri_two", "rules/tri_two.toml", "TriLattice"),
    ] {
        for (steps, window) in [(128u64, 16u64), (192, 16), (256, 16), (384, 24)] {
            let f = fixture(name, path, ic, steps, window);
            let cfg = f.sim_config(dir.path(), 7_100_000);
            let n_rules = hg_sim_core::load_rules(&cfg).expect("rules").len();
            let theta: Vec<f32> = (0..n_rules).map(|i| (i as f32 * 0.7).sin()).collect();
            let mut p = StaticBias::new(n_rules, Some(K), 12345);
            p.set_params(&theta);

            let t0 = std::time::Instant::now();
            let m = run_rollout(RolloutRequest {
                config: &cfg,
                seed: 7_100_000,
                policy: Box::new(p),
                horizon_steps: steps,
                probe: None,
            })
            .expect("rollout");
            let wall = t0.elapsed().as_secs_f64();

            // A CMA-ES generation on a 2-3 parameter genome is lambda = 6-7
            // members x 3 seeds + 5 hold-out rollouts, over 8 cores.
            let per_gen = 26.0 * wall / 8.0;
            println!(
                "{:<10} {:>7} {:>7} {:>9.2} {:>10} {:>8} {:>12.1}",
                name,
                steps,
                window,
                wall,
                m.windows.last().map(|w| w.n_nodes).unwrap_or(0),
                m.windows.len().saturating_sub(BURN_IN),
                per_gen,
            );
        }
    }
}
