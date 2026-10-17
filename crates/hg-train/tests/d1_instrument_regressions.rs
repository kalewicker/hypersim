//! Phase D1 exit criteria — one regression test per real C0–C5 bug.
//!
//! `hypersim_nn_experiments_plan_v2.md` §10, Phase D1:
//!
//! > Exit: the σ=0.02 campaign aborts at generation 10 with
//! > `OPTIMIZER_FROZEN`; a `[causal]`-off campaign aborts with
//! > `OBJECTIVE_BLIND`; one-rule `StaticBias` aborts at P-1; a 384-step
//! > `ei_interval_steps=1000` run reports `unmeasured`, not `NaN`.
//!
//! Each of these ran to completion in C0–C5 and produced a number that
//! would have been written up. The point of this file is that they now
//! cannot: three of them refuse to start, and the fourth refuses to emit a
//! value it did not measure.
//!
//! These are deliberately *end-to-end* — the unit tests in
//! `hg_train::prechecks` and `hg_train::fitness` check the mechanisms in
//! isolation, and would keep passing if the mechanisms were never wired in.
//! A precheck that is correct and unreachable is exactly the failure mode
//! `docs/instrument_registry.md` §1 exists to prevent.

use hg_cognition::{CognitionConfig, PolicyKind};
use hg_sim_core::SimConfig;
use hg_train::campaign::{run_campaign, CampaignSpec, TrainError};
use hg_train::diagnostics::{Fixture, RuleSource};
use hg_train::config::{Algo, TrainingConfig};
use hg_train::fitness::{self, FitnessFn, Reported};
use std::path::Path;

const STEPS: u64 = 192;

/// Fixtures are built through `diagnostics::Fixture` rather than by hand.
///
/// Two settings that a hand-rolled config silently omits are load-bearing
/// here: `step_mode`, and the disabled `Exploding` early-abort gate — which
/// on a `WhiteNoise` initial condition otherwise terminates the run on its
/// startup transient and truncates everything downstream.
///
/// The IC is not cosmetic either. `wn_three` has three rules but none of
/// them match a `GliderRing`, so P-1 correctly refuses that pairing before
/// any later precheck can run (`docs/policy_config.md` §2.5: a rule set on
/// the wrong IC produces zero candidates for the whole run, and selection
/// is then undefined rather than weakly causal).
fn fixture(rules: &'static str, ic: &'static str) -> Fixture {
    Fixture {
        name: "d1_regression",
        rules: RuleSource::File(rules),
        ic,
        width: 8,
        node_count: 24,
        steps: STEPS,
        window: 24,
        step_mode: "MaxParallel",
    }
}

/// The bug-7 configuration, written out because it cannot be reached
/// through `Fixture`: `sim_config_causal` scales `ei_interval_steps` to the
/// horizon precisely so this cannot happen by accident, and the bug *is*
/// the unscaled default of 1000.
fn default_interval_causal_config(dir: &Path) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "d1_bug7"
seed = 4242
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/wn_three.toml"
[initial]
condition_type = "WhiteNoise"
width = 8
height = 8
node_count = 24
[engine]
max_steps = {STEPS}
metrics_window_interval = 24
step_mode = "MaxParallel"
thread_count = 1
[evaluator]
enabled = true
[causal]
enabled = true
[causal_language]
ei_interval_steps = 1000
[campaign.abort]
exploding_windows = 1000000
[output]
fixed_provenance = true
"#,
        dir.display(),
        env!("CARGO_MANIFEST_DIR")
    );
    toml::from_str(&toml).expect("config parses")
}

/// A multi-rule substrate on a live IC — the configuration where P-1 passes
/// and the later prechecks are the ones under test.
fn wn_three(dir: &Path, causal: bool) -> SimConfig {
    let f = fixture("rules/wn_three.toml", "WhiteNoise");
    if causal {
        f.sim_config_causal(dir, 4242)
    } else {
        f.sim_config(dir, 4242)
    }
}

fn training(sigma: f32, generations: u32, fitness: FitnessFn) -> TrainingConfig {
    TrainingConfig {
        algo: Algo::CmaEs,
        generations,
        seeds_per_member: 1,
        population_lambda: Some(4),
        sigma,
        fitness,
        horizon_steps: STEPS,
        burn_in_windows: 1,
        holdout_seeds: vec![900_001, 900_002, 900_003],
        campaign_seed: 77,
        ..Default::default()
    }
}

fn spec<'a>(dir: &Path, cfg: &'a SimConfig, t: TrainingConfig) -> CampaignSpec<'a> {
    CampaignSpec {
        campaign_id: "d1_regression_77".into(),
        base_dir: dir.to_path_buf(),
        sim_config: cfg,
        cognition: CognitionConfig {
            policy: PolicyKind::StaticBias,
            budget: Some(8),
            ..Default::default()
        },
        training: t,
    }
}

/// The abort code, or a panic naming what happened instead.
///
/// `CampaignOutcome` is deliberately not `Debug` (it carries every hold-out
/// row), so `expect_err` is not available and the success case is unwrapped
/// by hand.
fn abort_code(r: Result<hg_train::campaign::CampaignOutcome, TrainError>, why: &str) -> String {
    match r {
        Err(TrainError::Precheck(p)) => p.code().to_string(),
        Err(other) => panic!("expected a precheck abort ({why}), got error: {other}"),
        Ok(_) => panic!("expected a precheck abort ({why}), but the campaign ran"),
    }
}

// =====================================================================
// Bug: StaticBias over a single rule (plan v2 §1.2)
// =====================================================================

#[test]
fn one_rule_static_bias_aborts_at_p1() {
    // Every builtin rule set ships exactly one rule. A softmax over one
    // logit is 1.0 whatever the logit is, so `StaticBias` there is
    // *provably* budget-matched `RandomSubset` — the hypothesis cannot be
    // stated, and a campaign would spend hours fitting the policy RNG and
    // report the result as a margin.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = fixture("rules/glider.toml", "GliderRing").sim_config(dir.path(), 4242);
    let code = abort_code(
        run_campaign(&spec(
            dir.path(),
            &cfg,
            training(2.0, 4, FitnessFn::F2RegimeOccupancy),
        )),
        "a one-rule StaticBias campaign must not start",
    );
    assert_eq!(code, "CLASS_INEXPRESSIBLE", "P-1 must be the gate that fires");
}

// =====================================================================
// Bug 10: sigma = 0.02 in softmax-logit space (plan v2 §1.2)
// =====================================================================

#[test]
#[ignore = "runs a short campaign; explicit, per the D1 exit criteria"]
fn sigma_002_aborts_with_optimizer_frozen() {
    // Forty generations at this sigma left the allocation uniform to 4% and
    // reported a clean negative from a search that never left the origin.
    // P-2 is evaluated at min(10, generations/4), so a 40-generation
    // campaign now aborts at generation 10 rather than at generation 40.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = wn_three(dir.path(), false);
    let code = abort_code(
        run_campaign(&spec(
            dir.path(),
            &cfg,
            training(0.02, 8, FitnessFn::F2RegimeOccupancy),
        )),
        "a frozen search must not run to completion",
    );
    assert_eq!(code, "OPTIMIZER_FROZEN");
}

// =====================================================================
// Bug 8: [causal] off makes F3 identically 0.0 (plan v2 §6.1)
// =====================================================================

#[test]
#[ignore = "runs the P-3 probe arms; explicit, per the D1 exit criteria"]
fn a_causal_off_f3_campaign_aborts_with_objective_blind() {
    // With `[causal]` disabled, `RolloutMetrics::constructors` is empty,
    // `n_confirmed_constructors` is 0, and F3 is identically 0.0 for every
    // arm. The C0–C5 run of this produced a table of zeros that reads as
    // "no policy affects constructor yield" — a statement about the config,
    // presented as a statement about policies.
    //
    // Under v2 the fitness comes back `unmeasured` rather than 0.0, so P-3
    // sees zero measurable arms and refuses to start.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = wn_three(dir.path(), false);
    let code = abort_code(
        run_campaign(&spec(
            dir.path(),
            &cfg,
            training(2.0, 4, FitnessFn::F3ConstructorYield),
        )),
        "an objective that cannot be measured must not be optimized",
    );
    assert_eq!(code, "OBJECTIVE_BLIND");
}

#[test]
#[ignore = "runs the P-3 probe arms; explicit, per the D1 exit criteria"]
fn f5_inherits_the_same_gate() {
    // Plan v2 §5.4: "Instrument bug 8 made F3 identically zero and would
    // have made F5 identically zero for the same reason." Pinned, so that
    // the new objective cannot repeat the old objective's failure.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = wn_three(dir.path(), false);
    let code = abort_code(
        run_campaign(&spec(
            dir.path(),
            &cfg,
            training(2.0, 4, FitnessFn::F5ConstructorReuse),
        )),
        "F5 with no causal layer is unmeasured, not zero",
    );
    assert_eq!(code, "OBJECTIVE_BLIND");
}

// =====================================================================
// Bug 7: ei_interval_steps = 1000 against a 384-step horizon
// =====================================================================

#[test]
#[ignore = "the positive control needs a WhiteNoise run long enough to clear \
            ei_minimum_transitions (~140 s in release); explicit, per the D1 \
            exit criteria"]
fn a_384_step_run_at_ei_interval_1000_reports_unmeasured_not_nan() {
    // The interval never elapses, no EI is computed, every field comes
    // through `None`, and the C0–C5 averages came out `NaN` — three arms of
    // `NaN` that would read as "shaping has no effect on EI".
    //
    // The engagement counter is what separates "the calculator ran and
    // found nothing" from "the calculator never ran".
    let dir = tempfile::tempdir().expect("tempdir");
    // `Fixture::sim_config_causal` scales the interval to the horizon, so
    // the bug-7 configuration has to be written out: the DEFAULT of 1000 is
    // exactly the value that never elapses.
    let cfg = default_interval_causal_config(dir.path());
    let m = hg_sim_core::run_rollout(hg_sim_core::RolloutRequest {
        config: &cfg,
        seed: 900_001,
        policy: Box::new(hg_observe::policy::SelectAllPolicy),
        horizon_steps: STEPS,
        probe: None,
    })
    .expect("rollout runs");

    let fit = fitness::evaluate(&m, 1).expect("scored");
    assert_eq!(
        fit.engagement.ei_intervals_emitted, 0,
        "the interval (1000) is longer than the horizon ({STEPS}), so no EI can exist"
    );
    assert!(
        fit.engagement.not_engaged().contains(&"ei_intervals_emitted"),
        "the counter must name itself: {:?}",
        fit.engagement.not_engaged()
    );

    // And a config where the calculator CAN run does engage — otherwise
    // this test would pass on a broken counter that always read zero,
    // which is the firing test every guard in `docs/instrument_registry.md`
    // §1 is required to carry.
    // Built through `Fixture::sim_config_causal` rather than by hand: it
    // is the configuration Stage 5 actually measured EI under, and it
    // carries two settings this file's hand-rolled config does not —
    // `step_mode` and the disabled `Exploding` early-abort gate, which on a
    // WhiteNoise IC otherwise terminates the run on its startup transient.
    let cfg_ok = wn_three(dir.path(), true);
    let m2 = hg_sim_core::run_rollout(hg_sim_core::RolloutRequest {
        config: &cfg_ok,
        seed: 900_001,
        policy: Box::new(hg_observe::policy::SelectAllPolicy),
        horizon_steps: STEPS,
        probe: None,
    })
    .expect("rollout runs");
    let fit2 = fitness::evaluate(&m2, 1).expect("scored");
    assert!(
        fit2.engagement.ei_intervals_emitted > 0,
        "the counter must be able to read non-zero, or it is not a counter"
    );
}

// =====================================================================
// The schema distinction the four above depend on
// =====================================================================

#[test]
fn unmeasured_reaches_the_csv_as_a_word_not_a_blank() {
    // `docs/campaign_output.md` §4.3: a reader must never have to infer
    // whether a blank means "unmeasured" or "not applicable". This is the
    // single change plan v2 §6.1 identifies as catching six of eleven
    // C0–C5 instrument bugs, so it is pinned at the serialization boundary
    // rather than only at the type level.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = wn_three(dir.path(), false);
    let m = hg_sim_core::run_rollout(hg_sim_core::RolloutRequest {
        config: &cfg,
        seed: 900_001,
        policy: Box::new(hg_observe::policy::SelectAllPolicy),
        horizon_steps: STEPS,
        probe: None,
    })
    .expect("rollout runs");
    let fit = fitness::evaluate(&m, 1).expect("scored");

    assert_eq!(
        fit.f5_constructor_reuse,
        Reported::Unmeasured,
        "[causal] is off, so no constructor record was ever examined"
    );
    assert_eq!(fit.f5_constructor_reuse.render(), "unmeasured");

    let mut w = csv::Writer::from_writer(vec![]);
    w.serialize(fit.f5_constructor_reuse).expect("serializes");
    let out = String::from_utf8(w.into_inner().expect("flush")).expect("utf8");
    assert!(
        out.contains("unmeasured"),
        "the literal string is part of the schema, got {out:?}"
    );
    assert!(
        !out.trim().is_empty(),
        "an unmeasured value must never round-trip to an empty cell"
    );
}
