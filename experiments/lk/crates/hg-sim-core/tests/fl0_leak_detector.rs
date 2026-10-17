//! F-L0 — the leak detector (NN plan §9), plus the Phase C2 exit criteria.
//!
//! The whole learning programme rests on one claim: installing the policy
//! machinery does not, by itself, change the physics. If it does, every
//! later comparison between a learned policy and a budget-matched control
//! is measuring the harness rather than the policy, and no amount of
//! careful statistics downstream recovers from that.
//!
//! So this file checks the claim from both sides:
//!
//! - an inert policy (`PassThrough`) must reproduce the no-`[cognition]`
//!   canonical hash **exactly**;
//! - a budgeted policy must NOT — otherwise the first check is vacuous and
//!   would pass just as happily if the seam were disconnected.
//!
//! ## Why the fixture table is not just `BUILTIN_NAMES`
//!
//! A builtin rule set only matches on an initial condition that contains
//! its LHS. `triangle_subdivide` needs 3-cliques and finds none on a
//! `PureCrystal`; `glider` needs its ring. On the wrong IC a rule set
//! produces **zero candidates for the whole run** — and then every policy
//! trivially agrees, because there is nothing to select. A hash-equality
//! test over such a pairing passes while proving nothing, which is the
//! worst failure mode a leak detector can have.
//!
//! Measured liveness (64 steps, `attempted_matches` summed over windows):
//!
//! | rule set | PureCrystal | TriLattice | GliderRing | WhiteNoise |
//! |---|---|---|---|---|
//! | `triangle_subdivide` | 0 | 774 912 | 0 | 73 728 |
//! | `binary_edge` | 1 328 256 | 1 354 128 | 0 | 1 311 920 |
//! | `tetrahedral` | 0 | 0 | 0 | 0 |
//! | `glider` | 0 | 0 | 384 | 0 |
//! | `growth_treadmill` | 0 | 0 | 0 | 576 |
//!
//! So [`FIXTURES`] pairs each rule set with an IC on which it actually
//! fires, and [`assert_live`] re-checks that at run time — if a future
//! change to a rule set or a generator freezes one of these pairings, this
//! file fails loudly instead of going quietly vacuous.
//!
//! `tetrahedral` has no live pairing among the shipped initial conditions
//! and is therefore **absent from the table on purpose**, not by oversight.

#![cfg(feature = "cognition")]

use hg_cognition::{MlpScorer, Parameterized, PassThrough, RandomSubset};
use hg_sim_core::{run_headless, run_rollout, RolloutMetrics, RolloutRequest, SimConfig};
use std::path::Path;

const SEED: u64 = 4242;

struct Fixture {
    rule: &'static str,
    ic: &'static str,
    width: u64,
    node_count: u64,
    steps: u64,
    window: u64,
}

/// Rule set × initial condition pairings verified to produce candidates.
///
/// Horizons are per-fixture because candidate volume differs by three
/// orders of magnitude across them: `binary_edge` offers ~56 000 candidates
/// per step on a crystal while `glider` offers ~6. Running them all at one
/// horizon would either starve the cheap fixtures of windows or make the
/// expensive ones dominate the suite's runtime for no extra coverage.
const FIXTURES: &[Fixture] = &[
    Fixture { rule: "glider", ic: "GliderRing", width: 8, node_count: 24, steps: 128, window: 32 },
    Fixture { rule: "growth_treadmill", ic: "WhiteNoise", width: 8, node_count: 24, steps: 128, window: 32 },
    Fixture { rule: "triangle_subdivide", ic: "TriLattice", width: 8, node_count: 24, steps: 32, window: 8 },
    Fixture { rule: "binary_edge", ic: "PureCrystal", width: 8, node_count: 24, steps: 32, window: 8 },
];

fn config(f: &Fixture, dir: &Path, seed: u64) -> SimConfig {
    let (rule, ic, width, node_count, steps, window) =
        (f.rule, f.ic, f.width, f.node_count, f.steps, f.window);
    let toml = format!(
        r#"
experiment_name = "fl0_{rule}"
seed = {seed}
out_dir = "{}"

[rules]
source = "Builtin"
builtin_name = "{rule}"

[initial]
condition_type = "{ic}"
width = {width}
height = {width}
node_count = {node_count}

[engine]
max_steps = {steps}
metrics_window_interval = {window}
thread_count = 1

[evaluator]
enabled = true

[causal]
enabled = true

[output]
fixed_provenance = true
"#,
        dir.display()
    );
    toml::from_str(&toml).expect("config parses")
}

/// Guard against a silently vacuous test: a fixture that offers no
/// candidates cannot distinguish any two policies.
fn assert_live(m: &RolloutMetrics, rule: &str) {
    let attempted: usize = m.windows.iter().map(|w| w.attempted_matches).sum();
    assert!(
        attempted > 0,
        "{rule}: zero candidates over the whole run — this fixture is inert, so every \
         policy comparison over it is vacuous. Fix the rule/IC pairing, do not delete \
         this assertion."
    );
}

fn rollout(cfg: &SimConfig, f: &Fixture, policy: Box<dyn hg_observe::RuleSelectionPolicy>) -> RolloutMetrics {
    run_rollout(RolloutRequest {
        config: cfg,
        seed: SEED,
        policy,
        horizon_steps: f.steps,
        probe: None,
    })
    .expect("rollout")
}

fn mlp_for(n_rules: usize, budget: Option<usize>, seed: u64) -> MlpScorer {
    let mut p = MlpScorer::new(n_rules, 16, 16, budget, seed);
    let n = p.n_params();
    p.set_params(&(0..n).map(|i| ((i as f32) * 0.19).sin()).collect::<Vec<_>>());
    p
}

/// F-L0 proper. `PassThrough` returns every index, so the conflict resolver
/// sees exactly the candidate set it would have seen with no policy at all,
/// and the resulting graph must be bit-identical.
#[test]
fn passthrough_rollout_reproduces_the_no_cognition_hash() {
    for f in FIXTURES {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = config(f, dir.path(), SEED);

        // Baseline: an ordinary scientific run, no policy anywhere.
        let baseline = run_headless(&cfg, None, None, None).expect("baseline run");
        // Rollout with the machinery installed but inert.
        let metrics = rollout(&cfg, f, Box::new(PassThrough));
        assert_live(&metrics, f.rule);

        assert_eq!(
            metrics.final_canonical_hash, baseline.canonical_hash,
            "{}: PassThrough rollout diverged from the no-policy run",
            f.rule
        );
        assert_eq!(metrics.final_step, baseline.final_step, "{}: step count", f.rule);
        assert_eq!(
            metrics.n_nodes_initial, baseline.n_nodes_initial,
            "{}: initial node count",
            f.rule
        );
    }
}

/// The evaluator must produce the same science in a rollout as in a run —
/// a rollout skips the CSV, not the computation. A hash match alone would
/// not catch an evaluator silently running with different settings.
#[test]
fn rollout_computes_the_same_windows_as_a_run() {
    for f in FIXTURES {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = config(f, dir.path(), SEED);

        let baseline = run_headless(&cfg, None, None, None).expect("baseline run");
        let metrics = rollout(&cfg, f, Box::new(PassThrough));
        assert_live(&metrics, f.rule);

        assert_eq!(metrics.windows.len(), baseline.history.len(), "{}: windows", f.rule);
        assert_eq!(
            metrics.regime_occupancy, baseline.regime_occupancy,
            "{}: regime occupancy",
            f.rule
        );
        for (a, b) in metrics.windows.iter().zip(&baseline.history) {
            let w = a.window_id;
            assert_eq!(a.regime, b.regime, "{} window {w}: regime", f.rule);
            assert_eq!(a.n_nodes, b.n_nodes, "{} window {w}: n_nodes", f.rule);
            assert_eq!(
                a.interestingness_score, b.interestingness_score,
                "{} window {w}: interestingness",
                f.rule
            );
            assert_eq!(a.pi_normalized, b.pi_normalized, "{} window {w}: PI", f.rule);
            assert_eq!(a.ei_macro, b.ei_macro, "{} window {w}: EI macro", f.rule);
        }
        assert_eq!(
            metrics.constructors, baseline.constructors,
            "{}: constructor registry",
            f.rule
        );
    }
}

/// The check that keeps the two above honest.
///
/// If the seam were disconnected — a policy constructed but never
/// installed, or installed and then ignored — `PassThrough` would still
/// match the baseline and F-L0 would pass while proving nothing. A budget
/// of 1 must visibly change the outcome on every live fixture.
#[test]
fn a_budgeted_policy_does_change_the_dynamics() {
    for f in FIXTURES {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = config(f, dir.path(), SEED);
        let baseline = run_headless(&cfg, None, None, None).expect("baseline run");

        let metrics = rollout(&cfg, f, Box::new(RandomSubset::new(Some(1), 7)));
        assert_live(&metrics, f.rule);
        assert_ne!(
            metrics.final_canonical_hash, baseline.canonical_hash,
            "{}: a budget of 1 changed nothing — the policy seam is not connected, and \
             F-L0's hash equality proves nothing",
            f.rule
        );
    }
}

/// Bit-determinism (NN plan §3): same config, sim seed, policy params, and
/// policy RNG seed ⟹ identical hash and identical fitness inputs.
#[test]
fn rollout_is_bit_deterministic() {
    let f = &FIXTURES[2]; // triangle_subdivide: ~20 000 candidates/step
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = config(f, dir.path(), SEED);
    let n_rules = hg_sim_core::load_rules(&cfg).expect("rules").len();

    let a = rollout(&cfg, f, Box::new(mlp_for(n_rules, Some(4), 11)));
    let b = rollout(&cfg, f, Box::new(mlp_for(n_rules, Some(4), 11)));
    assert_live(&a, f.rule);

    assert_eq!(a.final_canonical_hash, b.final_canonical_hash);
    assert_eq!(a.regime_occupancy, b.regime_occupancy);
    assert_eq!(a.windows.len(), b.windows.len());
    for (x, y) in a.windows.iter().zip(&b.windows) {
        assert_eq!(x.interestingness_score, y.interestingness_score);
        assert_eq!(x.n_nodes, y.n_nodes);
    }
}

/// The policy RNG seed is part of the determinism contract, which means it
/// has to be load-bearing — a seed that changed nothing would make the
/// contract trivially true and the Gumbel perturbation decorative.
///
/// Run on a fixture where the budget genuinely bites (~20 000 candidates
/// per step against a budget of 4). On a fixture offering fewer candidates
/// than the budget, every policy selects everything and the seed provably
/// cannot matter — which is a property of that fixture, not evidence about
/// the RNG.
#[test]
fn the_policy_rng_seed_is_load_bearing() {
    let f = &FIXTURES[2]; // triangle_subdivide + TriLattice
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = config(f, dir.path(), SEED);
    let n_rules = hg_sim_core::load_rules(&cfg).expect("rules").len();

    let a = rollout(&cfg, f, Box::new(mlp_for(n_rules, Some(4), 11)));
    let b = rollout(&cfg, f, Box::new(mlp_for(n_rules, Some(4), 12)));
    assert_live(&a, f.rule);
    assert!(
        a.windows.iter().any(|w| w.attempted_matches > 4),
        "budget of 4 never bound — this fixture cannot test the RNG seed"
    );
    assert_ne!(
        a.final_canonical_hash, b.final_canonical_hash,
        "policy RNG seed had no effect"
    );
}

/// A rollout leaves no artifact. Not "artifacts somewhere harmless" — none,
/// because `runs/` is the scientific record and a training inner loop has
/// no business writing to it (`docs/campaign_output.md` §5).
#[test]
fn rollout_writes_nothing_to_disk() {
    let f = &FIXTURES[0];
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = config(f, dir.path(), SEED);

    rollout(&cfg, f, Box::new(PassThrough));

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read out_dir")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        entries.is_empty(),
        "rollout wrote {entries:?} into the output directory"
    );
}

/// The probe fires once per window, on the same snapshot the evaluator saw,
/// so probe rows and metric rows are index-aligned. Stage 1's reservoir
/// readout depends on that alignment being structural rather than something
/// the caller has to reconstruct.
#[test]
fn probe_rows_are_window_aligned() {
    let f = &FIXTURES[0];
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = config(f, dir.path(), SEED);

    let metrics = run_rollout(RolloutRequest {
        config: &cfg,
        seed: SEED,
        policy: Box::new(PassThrough),
        horizon_steps: f.steps,
        probe: Some(Box::new(|snap, window| {
            vec![snap.node_count() as f32, window as f32]
        })),
    })
    .expect("rollout");

    assert_eq!(metrics.probe_rows.len(), metrics.windows.len());
    for (i, (row, window)) in metrics.probe_rows.iter().zip(&metrics.windows).enumerate() {
        assert_eq!(row[1] as u64, window.window_id, "probe row {i} misaligned");
        assert_eq!(
            row[0] as usize, window.n_nodes,
            "probe row {i} saw a different snapshot than the evaluator"
        );
    }
}

/// The budget applies to pattern matches only. A scheduled creation rule
/// must fire on its schedule no matter how tight the budget gets — a
/// learned policy able to starve input injection could silence its own
/// environment (`docs/policy_config.md` §3).
#[test]
fn a_tight_budget_never_starves_a_scheduled_creation_rule() {
    let dir = tempfile::tempdir().expect("tempdir");
    let toml = format!(
        r#"
experiment_name = "fl0_creation"
seed = {SEED}
out_dir = "{}"

[rules]
source = "File"
file_path = "{}/../../rules/conservation_soak.toml"

[initial]
condition_type = "PureCrystal"
width = 8
height = 8

[engine]
max_steps = 64
metrics_window_interval = 16
thread_count = 1

[evaluator]
enabled = true

[output]
fixed_provenance = true
"#,
        dir.path().display(),
        env!("CARGO_MANIFEST_DIR")
    );
    let cfg: SimConfig = toml::from_str(&toml).expect("config parses");

    let starved = run_rollout(RolloutRequest {
        config: &cfg,
        seed: SEED,
        // A budget of 1 is as tight as it gets — 0 is rejected by config
        // validation rather than silently read as "unbounded".
        policy: Box::new(RandomSubset::new(Some(1), 3)),
        horizon_steps: 64,
        probe: None,
    })
    .expect("budgeted rollout");

    let final_nodes = starved
        .windows
        .last()
        .map(|w| w.n_nodes)
        .expect("at least one window");
    assert!(
        final_nodes >= starved.n_nodes_initial,
        "budget of 1 shrank the graph from {} to {final_nodes} — a creation rule was starved",
        starved.n_nodes_initial
    );
}
