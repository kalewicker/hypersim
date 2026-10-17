//! F-L2 — the deceptive-horizon fixture (NN plan §9).
//!
//! A rule that inflates early interestingness and then tips the run into
//! `Exploding`. Correct long-horizon behaviour is to under-use it. The
//! plan's required result is a **contrast**, not a single verdict:
//!
//! > F2-trained policies avoid the trap at `horizon_steps = 20000`;
//! > F1-trained policies at short horizons take the bait — and that
//! > contrast is itself the documented, expected outcome, validating both
//! > the horizon handling and the Goodhart tripwires' ability to fire.
//!
//! This file establishes the contrast **as a property of the fixture**,
//! by measurement on the pure arms rather than by running two campaigns.
//! That is the honest cheap version: if the bait is not baited and the trap
//! does not spring, no amount of training will produce the contrast, and
//! training would only tell you so after two full campaigns. Whether an
//! F1-trained *policy* actually takes it is Stage 0a/2 work at real scale.
//!
//! ## Measured (budget 4, 31-node GliderRing, burn-in 2)
//!
//! | horizon | arm | F1 | F2 | regimes after burn-in |
//! |---|---|---|---|---|
//! | 256 | glider | 0.432 | 1.000 | ActivePersistentStructure ×6 |
//! | 256 | bloat | **0.498** | 1.000 | GenerativeCandidate ×6 |
//! | 4096 | glider | 0.432 | **1.000** | ActivePersistentStructure ×6 |
//! | 4096 | bloat | 0.011 | **−0.500** | Exploding |
//!
//! Both halves of the contrast are present: at 256 steps the trap arm wins
//! on F1 (0.498 vs 0.432), and at 4096 steps it loses catastrophically on
//! F2 (−0.500 vs 1.000). The trap is a genuine reversal, not a difference
//! of degree.
//!
//! Note the budget: this fixture uses `k = 4`, where F-L1 uses `k = 16`.
//! At `k = 16` the growth rule explodes immediately and there is no early
//! bait to take — the deception exists only in the band where growth is
//! slow enough to read as generative. Same rule set, different budget,
//! opposite lesson.

use hg_cognition::RuleBias;
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};
use hg_train::fitness::FitnessBreakdown;

const BUDGET: usize = 4;
const BURN_IN: usize = 2;
const SHORT_STEPS: u64 = 256;
const SHORT_WINDOW: u64 = 32;
const LONG_STEPS: u64 = 4096;
const LONG_WINDOW: u64 = 512;

fn sim_config(dir: &std::path::Path, steps: u64, window: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "fl2"
seed = 4242
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/fl1_learnable_signal.toml"
[initial]
condition_type = "GliderRing"
node_count = 31
[engine]
max_steps = {steps}
metrics_window_interval = {window}
thread_count = 1
[evaluator]
enabled = true
[causal]
enabled = false
[output]
fixed_provenance = true
"#,
        dir.display(),
        env!("CARGO_MANIFEST_DIR")
    );
    toml::from_str(&toml).expect("config parses")
}

fn arm(cfg: &SimConfig, steps: u64, weights: Vec<f32>) -> FitnessBreakdown {
    let m = run_rollout(RolloutRequest {
        config: cfg,
        seed: 4242,
        policy: Box::new(RuleBias::new(weights, Some(BUDGET), 1)),
        horizon_steps: steps,
        probe: None,
    })
    .expect("rollout");
    hg_train::fitness::evaluate(&m, BURN_IN).expect("fitness")
}

const GLIDER: [f32; 2] = [1.0, 0.0];
const TRAP: [f32; 2] = [0.0, 1.0];

/// Half one: at a short horizon the trap arm looks *better* on F1. This is
/// what an F1-trained short-horizon policy would chase.
#[test]
fn at_a_short_horizon_f1_prefers_the_trap() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path(), SHORT_STEPS, SHORT_WINDOW);

    let safe = arm(&cfg, SHORT_STEPS, GLIDER.to_vec());
    let trap = arm(&cfg, SHORT_STEPS, TRAP.to_vec());

    assert!(
        trap.f1_mean_interestingness > safe.f1_mean_interestingness,
        "the bait is not baited: trap F1 {:.3} <= safe F1 {:.3} at {SHORT_STEPS} steps",
        trap.f1_mean_interestingness,
        safe.f1_mean_interestingness
    );
    // And F2 does not yet see anything wrong — that is what makes it a trap
    // rather than an obviously bad rule.
    assert!(
        trap.f2 >= safe.f2 - 1e-9,
        "trap F2 {:.3} already below safe {:.3} at the short horizon — the deception \
         has collapsed into an immediately visible cost",
        trap.f2,
        safe.f2
    );
}

/// Half two: at a long horizon the trap springs, and F2 sees it.
#[test]
fn at_a_long_horizon_f2_rejects_the_trap() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = sim_config(dir.path(), LONG_STEPS, LONG_WINDOW);

    let safe = arm(&cfg, LONG_STEPS, GLIDER.to_vec());
    let trap = arm(&cfg, LONG_STEPS, TRAP.to_vec());

    assert!(
        trap.f2 < safe.f2 - 0.5,
        "the trap did not spring: trap F2 {:.3} vs safe {:.3} at {LONG_STEPS} steps",
        trap.f2,
        safe.f2
    );
    assert!(
        trap.f2_penalty > 0.0,
        "trap paid no penalized-regime cost at the long horizon"
    );
    // The reversal is the point: F1 must also have turned against the trap
    // by now, or the "deceptive" framing would be about F2's horizon alone.
    assert!(
        trap.f1_mean_interestingness < safe.f1_mean_interestingness,
        "trap F1 {:.3} still beats safe {:.3} at the long horizon",
        trap.f1_mean_interestingness,
        safe.f1_mean_interestingness
    );
}

/// The contrast stated as one assertion, which is the form the plan's exit
/// criterion actually takes: the SAME arm that wins early loses late.
#[test]
fn the_horizon_contrast_is_a_reversal_not_a_gradient() {
    let dir = tempfile::tempdir().expect("tempdir");

    let short = sim_config(dir.path(), SHORT_STEPS, SHORT_WINDOW);
    let short_safe = arm(&short, SHORT_STEPS, GLIDER.to_vec());
    let short_trap = arm(&short, SHORT_STEPS, TRAP.to_vec());

    let long = sim_config(dir.path(), LONG_STEPS, LONG_WINDOW);
    let long_safe = arm(&long, LONG_STEPS, GLIDER.to_vec());
    let long_trap = arm(&long, LONG_STEPS, TRAP.to_vec());

    let early_advantage = short_trap.f1_mean_interestingness - short_safe.f1_mean_interestingness;
    let late_advantage = long_trap.f2 - long_safe.f2;

    assert!(
        early_advantage > 0.0 && late_advantage < 0.0,
        "no reversal: early F1 advantage {early_advantage:+.3}, late F2 advantage \
         {late_advantage:+.3}"
    );
    assert!(
        late_advantage.abs() > early_advantage,
        "the late cost ({late_advantage:+.3}) does not outweigh the early gain \
         ({early_advantage:+.3}) — a policy trading one for the other would be right \
         to take the bait, and the fixture would not be a trap"
    );
}
