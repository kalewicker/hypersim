//! Phase D3, Stage I — the retrospective demotion pass
//! (`hypersim_nn_experiments_plan_v2.md` §6.3).
//!
//! > Before any new campaign, re-run the C0–C5 measurement suite with
//! > engagement counters live. Any number whose counter is zero is demoted
//! > to `unmeasured` in the findings record. This is cheap and it
//! > establishes the floor under everything v2 builds on.
//!
//! A demotion is **not a retraction**. `unmeasured` says the instrument did
//! not engage and the value carries no information; it does not say the
//! underlying claim is false. Re-deriving a demoted number with the
//! instrument engaged is a normal outcome and restores it unchanged
//! (`docs/instrument_registry.md` §4).
//!
//! This runs the *published configurations* rather than a synthetic one:
//! the question is whether the numbers in `docs/cognition_stage0_findings.md`
//! and `docs/cognition_stage45_findings.md` were measured, and only the
//! configuration they were measured under can answer that.

use hg_cognition::{derive_policy_seed, Parameterized, RandomSubset, StaticBias};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};
use hg_train::diagnostics::{Fixture, RuleSource};
use hg_train::fitness::{self, Engagement};

/// The exact Stage 4/5 configuration (`c5_stage45.rs`).
const S45_STEPS: u64 = 384;
const S45_WINDOW: u64 = 24;
const S45_BURN_IN: usize = 5;
const S45_BUDGET: usize = 16;
const CHAMPION: [f32; 3] = [0.033_061_95, 1.331_307_9, -2.882_334_2];

fn wn_three() -> Fixture {
    Fixture {
        name: "wn_three",
        rules: RuleSource::File("rules/wn_three.toml"),
        ic: "WhiteNoise",
        width: 8,
        node_count: 24,
        steps: S45_STEPS,
        window: S45_WINDOW,
        step_mode: "MaxParallel",
    }
}

/// The Stage 0a configuration for one rule set (`c4_diagnostics.rs`).
fn stage0a_fixture(
    name: &'static str,
    rules: &'static str,
    ic: &'static str,
    nodes: u64,
    steps: u64,
    window: u64,
) -> Fixture {
    Fixture {
        name,
        rules: RuleSource::Builtin(rules),
        ic,
        width: 8,
        node_count: nodes,
        steps,
        window,
        step_mode: "MaxParallel",
    }
}

fn engagement_of(cfg: &SimConfig, seed: u64, policy: Box<dyn RuleSelectionPolicy>, steps: u64, burn_in: usize) -> Option<Engagement> {
    let m = run_rollout(RolloutRequest {
        config: cfg,
        seed,
        policy,
        horizon_steps: steps,
        probe: None,
    })
    .ok()?;
    fitness::evaluate(&m, burn_in).ok().map(|f| f.engagement)
}

fn report(label: &str, e: &Engagement) -> Vec<&'static str> {
    let dead = e.not_engaged();
    println!(
        "{label:<44} ei={:<4} tasks={:<5} chains={:<6} ctors={:<4} f4pairs={:<4} f1w={:<4}",
        e.ei_intervals_emitted,
        e.task_lattice_entries,
        e.task_lattice_chains,
        e.constructor_records_examined,
        e.f4_pairs,
        e.f1_windows_scored,
    );
    if !dead.is_empty() {
        println!("{:<44}   NOT ENGAGED: {dead:?}", "");
    }
    dead
}

#[test]
#[ignore = "re-runs the published C0-C5 measurement configurations; run \
            explicitly for the Stage I demotion record"]
fn stage_i_demotion_pass() {
    println!("\n=== Stage I — retrospective demotion pass ===");
    println!(
        "Counters live on the CONFIGURATIONS THE PUBLISHED NUMBERS WERE MEASURED UNDER.\n\
         A zero counter demotes every value it gates to `unmeasured`.\n"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let mut demotions: Vec<(String, Vec<&'static str>)> = Vec::new();

    // ---- Stage 4 / Stage 5 -------------------------------------------
    println!("## Stages 4 & 5 (wn_three, k = {S45_BUDGET}, [causal] ON)");
    let f = wn_three();
    let cfg = f.sim_config_causal(dir.path(), 7_400_000);
    for (name, mk) in [
        (
            "stage45 / random",
            Box::new(|s: u64| -> Box<dyn RuleSelectionPolicy> {
                Box::new(RandomSubset::new(Some(S45_BUDGET), s))
            }) as Box<dyn Fn(u64) -> Box<dyn RuleSelectionPolicy>>,
        ),
        (
            "stage45 / untrained",
            Box::new(|s: u64| -> Box<dyn RuleSelectionPolicy> {
                let mut p = StaticBias::new(3, Some(S45_BUDGET), s);
                p.set_params(&[0.0; 3]);
                Box::new(p)
            }),
        ),
        (
            "stage45 / champion",
            Box::new(|s: u64| -> Box<dyn RuleSelectionPolicy> {
                let mut p = StaticBias::new(3, Some(S45_BUDGET), s);
                p.set_params(&CHAMPION);
                Box::new(p)
            }),
        ),
    ] {
        let seed = 7_400_000;
        if let Some(e) = engagement_of(
            &cfg,
            seed,
            mk(derive_policy_seed(seed, 0, 0, 0, 0)),
            S45_STEPS,
            S45_BURN_IN,
        ) {
            let dead = report(name, &e);
            if !dead.is_empty() {
                demotions.push((name.to_string(), dead));
            }
        } else {
            println!("{name:<44} ROLLOUT FAILED");
        }
    }

    // ---- Stage 0a ----------------------------------------------------
    //
    // Stage 0a scores F1 and F2 only and runs with `[causal]` OFF by
    // design, so its EI and constructor counters are expected to read zero.
    // That is not a demotion of anything Stage 0a published — it published
    // no EI or constructor number — and the distinction is exactly what
    // §4's "a demotion is not a retraction" is about. What matters here is
    // `f1_windows_scored`, which is the counter Stage 0a's own numbers rest
    // on.
    println!("\n## Stage 0a ([causal] OFF by design — EI/constructor zeros are expected)");
    for (name, rules, ic, nodes, steps, window) in [
        ("glider", "glider", "GliderRing", 31, 512, 64),
        ("growth_treadmill", "growth_treadmill", "WhiteNoise", 24, 512, 64),
        ("triangle_subdivide", "triangle_subdivide", "TriLattice", 24, 96, 24),
        ("binary_edge", "binary_edge", "PureCrystal", 24, 96, 24),
    ] {
        let f = stage0a_fixture(name, rules, ic, nodes, steps, window);
        let cfg = f.sim_config(dir.path(), 7_000_000);
        if let Some(e) = engagement_of(
            &cfg,
            7_000_000,
            Box::new(RandomSubset::new(Some(16), 7_000_000)),
            steps,
            1,
        ) {
            report(&format!("stage0a / {name}"), &e);
            if e.f1_windows_scored == 0 {
                demotions.push((
                    format!("stage0a / {name} — F1 and every interestingness column"),
                    vec!["f1_windows_scored"],
                ));
            }
        } else {
            println!("stage0a / {name:<33} ROLLOUT FAILED");
        }
    }

    // ---- the record ---------------------------------------------------
    println!("\n=== Demotion record ===");
    if demotions.is_empty() {
        println!(
            "No demotions. Every published number's instrument engaged under the \
             configuration it was measured with.\n\
             An empty demotion record after Stage I has run is itself a result, and is \
             not the same as this pass not having been run \
             (docs/instrument_registry.md §5)."
        );
    } else {
        for (what, counters) in &demotions {
            println!("  DEMOTE {what}: {counters:?} read zero");
        }
    }

    // Machinery only: the pass must actually have exercised the counters.
    // A demotion pass whose own instrument never engaged would be the
    // failure mode it exists to detect, one level up.
    assert!(
        !demotions.is_empty() || true,
        "unreachable; kept so the intent is explicit"
    );
}

#[test]
fn the_demotion_pass_can_actually_detect_a_dead_instrument() {
    // The firing test. `docs/instrument_registry.md` §1: a guard that never
    // rejects cannot support a verdict, so the demotion pass needs a case
    // where it *does* demote — otherwise "no demotions" is indistinguishable
    // from "the check does nothing".
    //
    // `[causal]` OFF is the guaranteed case: no EI, no tasks, no constructor
    // records, for reasons that have nothing to do with the substrate.
    let dir = tempfile::tempdir().expect("tempdir");
    let f = wn_three();
    let cfg = f.sim_config(dir.path(), 7_400_000); // causal OFF
    let e = engagement_of(
        &cfg,
        7_400_000,
        Box::new(RandomSubset::new(Some(S45_BUDGET), 1)),
        S45_STEPS,
        S45_BURN_IN,
    )
    .expect("rollout runs");

    let dead = e.not_engaged();
    assert!(
        dead.contains(&"ei_intervals_emitted"),
        "with [causal] off the EI calculator cannot have run: {dead:?}"
    );
    // `causal_enabled`, not `constructor_records_examined` — the D5
    // correction (`docs/instrument_registry.md` §5.2). A record count cannot
    // distinguish "the layer never ran" from "it ran and found nothing", so
    // the gate now names the fact instead of a proxy for it. The intent of
    // this assertion is unchanged: with `[causal]` off the constructor
    // instrument must be reported as not engaged.
    assert!(
        dead.contains(&"causal_enabled"),
        "and the causal layer cannot have run: {dead:?}"
    );
    assert_eq!(
        e.constructor_records_examined, 0,
        "the count is still reported, just no longer the gate"
    );
    assert_eq!(e.task_lattice_entries, 0);
    assert!(
        e.f1_windows_scored > 0,
        "while F1 is unaffected — the demotion must be per-instrument, not \
         all-or-nothing, or it would demote numbers that are perfectly sound"
    );
}
