//! C5 cost probe — the plan's §12 cost model wants a *measured* steps/sec
//! before any campaign is scheduled, and the answer is per-fixture by two
//! orders of magnitude. It also wants a horizon: the default
//! `horizon_steps = 20000` is a placeholder, and on a rule set whose node
//! count compounds it is a way to run out of memory rather than a way to
//! score more windows.
//!
//! Reports, per fixture and horizon: wallclock, effective steps/sec, final
//! node count, and how many windows actually scored after burn-in. A
//! campaign horizon is only usable if the last column is comfortably above
//! the burn-in.

use hg_cognition::{Parameterized, StaticBias};
use hg_sim_core::{run_rollout, RolloutRequest};
use hg_train::diagnostics::{Fixture, RuleSource};

const BURN_IN: usize = 5;

fn cases() -> Vec<(Fixture, Option<usize>, Vec<u64>)> {
    vec![
        (
            Fixture {
                name: "glider",
                rules: RuleSource::Builtin("glider"),
                ic: "GliderRing",
                width: 8,
                node_count: 31,
                steps: 0,
                window: 64,
                step_mode: "MaxParallel",
            },
            Some(4),
            vec![512, 2048, 8192],
        ),
        (
            Fixture {
                name: "growth_treadmill",
                rules: RuleSource::Builtin("growth_treadmill"),
                ic: "WhiteNoise",
                width: 8,
                node_count: 24,
                steps: 0,
                window: 64,
                step_mode: "MaxParallel",
            },
            Some(1),
            vec![512, 2048, 8192],
        ),
        (
            Fixture {
                name: "triangle_subdivide",
                rules: RuleSource::Builtin("triangle_subdivide"),
                ic: "TriLattice",
                width: 8,
                node_count: 24,
                steps: 0,
                window: 24,
                step_mode: "MaxParallel",
            },
            Some(4),
            vec![96, 384, 1024],
        ),
        (
            Fixture {
                name: "binary_edge",
                rules: RuleSource::Builtin("binary_edge"),
                ic: "PureCrystal",
                width: 8,
                node_count: 24,
                steps: 0,
                window: 24,
                step_mode: "MaxParallel",
            },
            Some(16),
            vec![96, 384, 1024],
        ),
    ]
}

#[test]
fn measured_cost_and_usable_horizons() {
    let dir = tempfile::tempdir().expect("tempdir");
    println!("\n=== C5 cost probe (StaticBias, single-threaded rollout) ===");
    println!(
        "{:<20} {:>4} {:>7} {:>9} {:>11} {:>10} {:>8}",
        "fixture", "k", "steps", "wall (s)", "steps/sec", "final n", "scored"
    );

    for (mut f, budget, horizons) in cases() {
        for h in horizons {
            f.steps = h;
            let cfg = f.sim_config(dir.path(), 7_000_000);
            let n_rules = hg_sim_core::load_rules(&cfg).expect("rules").len();
            let theta: Vec<f32> = (0..n_rules).map(|i| (i as f32 * 0.7).sin()).collect();
            let mut p = StaticBias::new(n_rules, budget, 12345);
            p.set_params(&theta);

            let t0 = std::time::Instant::now();
            let m = run_rollout(RolloutRequest {
                config: &cfg,
                seed: 7_000_000,
                policy: Box::new(p),
                horizon_steps: h,
                probe: None,
            })
            .expect("rollout");
            let wall = t0.elapsed().as_secs_f64();

            let scored = m.windows.len().saturating_sub(BURN_IN);
            println!(
                "{:<20} {:>4} {:>7} {:>9.2} {:>11.0} {:>10} {:>8}",
                f.name,
                budget.map(|b| b.to_string()).unwrap_or("inf".into()),
                h,
                wall,
                h as f64 / wall.max(1e-9),
                m.windows.last().map(|w| w.n_nodes).unwrap_or(0),
                scored,
            );
        }
    }
}
