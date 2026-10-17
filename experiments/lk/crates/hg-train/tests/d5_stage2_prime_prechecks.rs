//! Phase D5 — Stage 2′: F4/F5 stateability prechecks, and the programme's
//! go/no-go (`hypersim_nn_experiments_plan_v2.md` §9, §5.4, §10).
//!
//! > Not a re-run of Stage 2. Run P-1 through P-4 for F4 and F5 on
//! > `wn_three`, `tri_two`, and at least one non-fixture rule set. Plus the
//! > §5.4 constructor-yield gate for F5.
//! > **Exit:** at least one (objective, rule set) pair passes all four
//! > prechecks.
//!
//! # Configuration, and why
//!
//! `384` steps at window `6`, burn-in `5` — **58 symbol pairs**, clearing
//! `PI_MIN_PAIRS = 50`. `d5_f4_horizon_probe.rs` measured the alternative
//! (window frozen at the Stage 4/5 value of 24, horizon stretched to 1536)
//! at 28x the cost for an identical drift confound (trend −0.877 vs −0.875,
//! monotone ceiling 1.59 vs 1.61) over a substrate that grows four times as
//! far. It buys nothing the cheap route does not already have.
//!
//! # What P-2 is not doing here
//!
//! P-1, P-3 and P-4 interrogate the **objective** against the policy class
//! and cost one rollout each. P-2 interrogates the **optimizer**, and needs
//! a real θ trajectory — `campaign.rs` gates it at
//! `P2_MIN_GENERATIONS = 8`, which is a campaign, not a precheck. Plan §9
//! budgets Stage 2′ at "minutes". So P-2 runs only for cells that clear the
//! other three, which is the only order in which it is informative anyway:
//! a frozen optimizer on a blind objective is not a finding about the
//! optimizer.

use hg_cognition::{CognitionConfig, PolicyKind, RandomSubset, StaticBias};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};
use hg_train::config::TrainingConfig;
use hg_train::diagnostics::{Fixture, RuleSource};
use hg_train::fitness::{self, FitnessFn};
use hg_train::prechecks::{
    self, allocation_extreme, p1_class_expressiveness, p3_objective_discrimination,
    p4_signal_non_saturation, static_bias_with, PrecheckAbort, ProbeArm,
};

const STEPS: u64 = 384;
const WINDOW: u64 = 6;
const BURN_IN: usize = 5;
/// Budget. `k = 4` is the C0–C5 value and the one Stage 0b′/1′ matched;
/// `policy_config.md` §2.6 records `k = 16` as F-L1's saturation trap.
const BUDGET: usize = 4;
const SEEDS: [u64; 3] = [9_500_101, 9_500_102, 9_500_103];

fn fixture(
    name: &'static str,
    rules: RuleSource,
    ic: &'static str,
    node_count: u64,
) -> Fixture {
    Fixture {
        name,
        rules,
        ic,
        width: 8,
        node_count,
        steps: STEPS,
        window: WINDOW,
        step_mode: "MaxParallel",
    }
}

fn wn_three() -> Fixture {
    fixture("wn_three", RuleSource::File("rules/wn_three.toml"), "WhiteNoise", 24)
}
fn tri_two() -> Fixture {
    fixture("tri_two", RuleSource::File("rules/tri_two.toml"), "TriLattice", 24)
}

/// Training config carrying only what the prechecks read.
fn training(which: FitnessFn) -> TrainingConfig {
    let mut t = TrainingConfig::default();
    t.fitness = which;
    t.horizon_steps = STEPS;
    t.burn_in_windows = BURN_IN;
    t
}

fn arms(cfg: &SimConfig, t: &TrainingConfig, n_rules: usize) -> Vec<ProbeArm> {
    vec![
        prechecks::probe_arm("random", cfg, t, &SEEDS, |s| {
            Box::new(RandomSubset::new(Some(BUDGET), s)) as Box<dyn RuleSelectionPolicy>
        }),
        prechecks::probe_arm("theta=0", cfg, t, &SEEDS, |s| {
            Box::new(StaticBias::new(n_rules, Some(BUDGET), s)) as Box<dyn RuleSelectionPolicy>
        }),
        prechecks::probe_arm("extreme", cfg, t, &SEEDS, |s| {
            static_bias_with(n_rules, Some(BUDGET), &allocation_extreme(n_rules, 0), s)
        }),
    ]
}

struct Cell {
    objective: FitnessFn,
    rule_set: &'static str,
    p1: Result<String, PrecheckAbort>,
    p3: Result<String, PrecheckAbort>,
    p4: Result<String, PrecheckAbort>,
    arm_table: Vec<String>,
}

impl Cell {
    fn passes(&self) -> bool {
        self.p1.is_ok() && self.p3.is_ok() && self.p4.is_ok()
    }
}

fn run_cell(f: &Fixture, n_rules: usize, which: FitnessFn, causal: bool) -> Cell {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = if causal {
        f.sim_config_causal(dir.path(), 9_500_000)
    } else {
        f.sim_config(dir.path(), 9_500_000)
    };
    let t = training(which);

    // `budget` MUST be the campaign's. `CognitionConfig::default()` leaves
    // it `None`, which means no truncation — P-1's probe would then run an
    // uncapped substrate while every other number in this file uses k = 4,
    // and the co-liveness fraction it reports would be measured on a
    // different dynamics than the one under test.
    let p1 = p1_class_expressiveness(
        &CognitionConfig {
            policy: PolicyKind::StaticBias,
            budget: Some(BUDGET),
            ..Default::default()
        },
        &cfg,
        n_rules,
        64,
        SEEDS[0],
    )
    .map(|r| r.detail);

    let a = arms(&cfg, &t, n_rules);
    let arm_table = a
        .iter()
        .map(|x| {
            format!(
                "{} = {:.4} +- {:.4} (n={}, unmeasured={})",
                x.label,
                x.mean(),
                x.sd(),
                x.scores.len(),
                x.unmeasured
            )
        })
        .collect();
    let p3 = p3_objective_discrimination(&a).map(|r| r.detail);
    let p4 = p4_signal_non_saturation(which, &a[0]).map(|r| r.detail);

    Cell {
        objective: which,
        rule_set: f.name,
        p1,
        p3,
        p4,
        arm_table,
    }
}

fn show(label: &str, r: &Result<String, PrecheckAbort>) {
    match r {
        Ok(d) => println!("    {label}  PASS  {d}"),
        Err(e) => println!("    {label}  ABORT [{}] {}", e.code(), e.detail()),
    }
}

// =====================================================================
// Part 1 — the third rule set
// =====================================================================

/// Plan §9 asks for "at least one non-fixture rule set" beyond `wn_three`
/// and `tri_two`. That is not free: `policy_config.md` §2.5's liveness
/// matrix has exactly one co-live pairing among the builtins
/// (`binary_edge` + `triangle_subdivide`, plus `growth_treadmill` on
/// `WhiteNoise`), and `wn_three` and `tri_two` already **are** its two
/// compositions. Every other builtin ships a single rule, on which
/// `StaticBias` is provably budget-matched random and P-1 aborts before a
/// rollout.
///
/// So a third substrate has to come from another track. The candidates are
/// multi-rule sets built for their own reasons, with nothing planted to be
/// learned — which is the actual content of "non-fixture".
#[test]
#[ignore = "D5 part 1: selects the third rule set; run explicitly"]
fn third_rule_set_selection() {
    println!("\n=== D5 part 1 — a third, non-fixture rule set ===");
    println!("P-1 over candidate (rules, IC) pairs. Pass = two rules co-live.\n");

    let candidates: Vec<(&'static str, &'static str, usize, &'static str)> = vec![
        ("lk_pair", "rules/lk_pair.toml", 2, "Empty"),
        ("lk_pair", "rules/lk_pair.toml", 2, "WhiteNoise"),
        ("lk_pair", "rules/lk_pair.toml", 2, "PureCrystal"),
        ("lk_pair", "rules/lk_pair.toml", 2, "TriLattice"),
        ("crystal_battery", "rules/crystal_battery.toml", 2, "PureCrystal"),
        ("crystal_battery", "rules/crystal_battery.toml", 2, "CrystalWithDefects"),
        ("crystal_battery", "rules/crystal_battery.toml", 2, "WhiteNoise"),
        ("conservation_soak", "rules/conservation_soak.toml", 3, "Empty"),
        ("conservation_soak", "rules/conservation_soak.toml", 3, "WhiteNoise"),
    ];

    for (name, path, n_rules, ic) in candidates {
        let dir = tempfile::tempdir().expect("tempdir");
        let leaked: &'static str = Box::leak(path.to_string().into_boxed_str());
        let f = fixture(
            Box::leak(name.to_string().into_boxed_str()),
            RuleSource::File(leaked),
            Box::leak(ic.to_string().into_boxed_str()),
            24,
        );
        let cfg = f.sim_config(dir.path(), 9_500_000);
        let r = p1_class_expressiveness(
            &CognitionConfig {
                policy: PolicyKind::StaticBias,
                budget: Some(BUDGET),
                ..Default::default()
            },
            &cfg,
            n_rules,
            64,
            SEEDS[0],
        );
        match r {
            Ok(res) => println!("  {name:<20} {ic:<20} PASS  {}", res.detail),
            Err(e) => println!("  {name:<20} {ic:<20} ABORT [{}] {}", e.code(), {
                let d = e.detail();
                d.chars().take(96).collect::<String>()
            }),
        }
    }
}

// =====================================================================
// Part 2 — the §5.4 constructor-yield gate
// =====================================================================

/// Plan §5.4, the gate F5 must clear **before** it is stateable:
///
/// > verify non-zero constructor yield exists under *any* policy on the
/// > target rule set … it must pass on real (non-fixture) rule sets, not
/// > just on a constructed fixture.
///
/// Risk 2 in §11 is that this fails: "F5 may be identically zero on real
/// rule sets even with `[causal]` on, if constructors are rare outside
/// fixtures." The engagement counters make the two outcomes distinguishable
/// — `f5_records_examined = 0` is the causal layer never running (bug 8),
/// which is `unmeasured`; a positive count with no confirmed cycles is a
/// real zero.
#[test]
#[ignore = "D5 part 2: causal rollouts; run explicitly"]
fn f5_constructor_yield_gate() {
    println!("\n=== D5 part 2 — §5.4 constructor-yield gate for F5 ===");
    println!("[causal] ON. Gate passes if ANY policy yields a confirmed constructor.\n");
    println!(
        "{:<14} {:<10} {:>9} {:>9} {:>9} {:>12} {:>8}",
        "rule set", "arm", "records", "confirmed", "maxcycles", "F5", "secs"
    );

    for (f, n_rules) in [(wn_three(), 3usize), (tri_two(), 2)] {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = f.sim_config_causal(dir.path(), 9_500_000);
        for (label, mk) in [
            (
                "random",
                Box::new(|s: u64| Box::new(RandomSubset::new(Some(BUDGET), s)) as Box<dyn RuleSelectionPolicy>)
                    as Box<dyn Fn(u64) -> Box<dyn RuleSelectionPolicy>>,
            ),
            (
                "theta=0",
                Box::new(move |s: u64| {
                    Box::new(StaticBias::new(n_rules, Some(BUDGET), s)) as Box<dyn RuleSelectionPolicy>
                }),
            ),
            (
                "extreme",
                Box::new(move |s: u64| {
                    static_bias_with(n_rules, Some(BUDGET), &allocation_extreme(n_rules, 0), s)
                }),
            ),
        ] {
            let t0 = std::time::Instant::now();
            let m = run_rollout(RolloutRequest {
                config: &cfg,
                seed: SEEDS[0],
                policy: mk(SEEDS[0]),
                horizon_steps: STEPS,
                probe: None,
            })
            .expect("rollout");
            let secs = t0.elapsed().as_secs_f64();
            let fit = fitness::evaluate(&m, BURN_IN).expect("evaluate");
            println!(
                "{:<14} {:<10} {:>9} {:>9} {:>9} {:>12} {:>8.1}",
                f.name,
                label,
                fit.engagement.f5_records_examined,
                fit.n_confirmed_constructors,
                fit.f5_max_cycles,
                match fit.f5_constructor_reuse.value() {
                    Some(v) => format!("{v:.4}"),
                    None => "unmeasured".into(),
                },
                secs
            );
        }
    }
}

// =====================================================================
// Part 3 — the precheck matrix
// =====================================================================

#[test]
#[ignore = "D5 part 3: the Stage 2' precheck matrix; run explicitly"]
fn stage2_prime_precheck_matrix() {
    println!("\n=== D5 part 3 — Stage 2′ precheck matrix ===");
    println!(
        "steps={STEPS} window={WINDOW} burn_in={BURN_IN} k={BUDGET} seeds={SEEDS:?}\n\
         Exit criterion: at least one (objective, rule set) cell passes P-1, P-3 and P-4.\n"
    );

    let mut cells = Vec::new();
    for (f, n_rules) in [(wn_three(), 3usize), (tri_two(), 2)] {
        for which in [FitnessFn::F4MonotoneDrift, FitnessFn::F5ConstructorReuse] {
            // F5 reads the causal layer; running it with `[causal]` off is
            // bug 8 and would report a table of `unmeasured`.
            let causal = matches!(which, FitnessFn::F5ConstructorReuse);
            println!("## {} x {}", which.as_str(), f.name);
            let c = run_cell(&f, n_rules, which, causal);
            for row in &c.arm_table {
                println!("    {row}");
            }
            show("P-1", &c.p1);
            show("P-3", &c.p3);
            show("P-4", &c.p4);
            println!(
                "    => {}\n",
                if c.passes() { "PASSES" } else { "does not pass" }
            );
            cells.push(c);
        }
    }

    println!("\n--- summary ---");
    for c in &cells {
        println!(
            "{:<26} {:<10} P1={} P3={} P4={}  {}",
            c.objective.as_str(),
            c.rule_set,
            if c.p1.is_ok() { "ok " } else { "ABT" },
            if c.p3.is_ok() { "ok " } else { "ABT" },
            if c.p4.is_ok() { "ok " } else { "ABT" },
            if c.passes() { "PASS" } else { "" }
        );
    }
    let passing: Vec<&Cell> = cells.iter().filter(|c| c.passes()).collect();
    println!(
        "\n{} of {} cells pass. Stage 2′ exit criterion: {}",
        passing.len(),
        cells.len(),
        if passing.is_empty() { "NOT MET" } else { "MET" }
    );
}

// =====================================================================
// Part 5 — P-2 for the cells that cleared P-1/P-3/P-4
// =====================================================================

/// P-2 asks whether the **optimizer** moved somewhere the policy can act
/// on, which needs a real θ trajectory rather than a hand-set genome.
/// `campaign.rs` evaluates it at `traversal_generation(generations)` and
/// only when `generations >= P2_MIN_GENERATIONS`, so the minimum honest
/// test is an 8-generation campaign.
///
/// `sigma = 2.0`, not the config default of `0.02`.
/// `docs/policy_config.md` §6 records why: `StaticBias`'s parameters are
/// softmax logits, and a 0.02 step moves the induced allocation by
/// ~0.5% — below what P-2 can distinguish from a frozen optimizer, and the
/// original cause of the Stage 2 re-run.
#[test]
#[ignore = "D5 part 5: short campaigns to fire P-2; run explicitly"]
fn p2_for_the_passing_cells() {
    println!("\n=== D5 part 5 — P-2 on the cells that cleared P-1/P-3/P-4 ===");

    let cells: Vec<(Fixture, usize, FitnessFn, bool)> = vec![
        (wn_three(), 3, FitnessFn::F4MonotoneDrift, false),
        (tri_two(), 2, FitnessFn::F4MonotoneDrift, false),
        (tri_two(), 2, FitnessFn::F5ConstructorReuse, true),
    ];

    for (f, _n_rules, which, causal) in cells {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = if causal {
            f.sim_config_causal(dir.path(), 9_500_000)
        } else {
            f.sim_config(dir.path(), 9_500_000)
        };
        let mut t = training(which);
        t.generations = hg_train::campaign::P2_MIN_GENERATIONS;
        t.population_pairs = 3;
        t.seeds_per_member = 1;
        t.sigma = 2.0;
        t.holdout_seeds = SEEDS.to_vec();
        t.campaign_seed = 9_500_000;

        println!("\n## {} x {}", which.as_str(), f.name);
        let spec = hg_train::campaign::CampaignSpec {
            campaign_id: format!("d5_p2_{}_{}", which.as_str(), f.name),
            base_dir: dir.path().to_path_buf(),
            sim_config: &cfg,
            cognition: CognitionConfig {
                policy: PolicyKind::StaticBias,
                budget: Some(BUDGET),
                ..Default::default()
            },
            training: t,
        };
        match hg_train::campaign::run_campaign(&spec) {
            Ok(o) => {
                for p in &o.prechecks {
                    println!(
                        "    {:<28} {:<5} {}",
                        p.name,
                        if p.passed { "PASS" } else { "fail" },
                        p.detail
                    );
                }
            }
            Err(e) => println!("    campaign refused: {e}"),
        }
    }
}

/// Is the P-2 abort above about F4, or about the optimizer's step size?
///
/// P-2's own diagnostic says the latter — `|dtheta| = 0.0142 = 0.00x the
/// drift scale` selects the "step size too small" branch, not the "a
/// direction the policy cannot act on" branch — and the magnitude is the
/// same (~0.014) on the cell that *passed*. At `alpha = 0.01` over the 8
/// generations `P2_MIN_GENERATIONS` allows, θ has barely left its
/// initialization whatever the objective.
///
/// That is an inference from one number, so it gets measured: the same cell
/// at learning rates that actually traverse. If P-2 passes once the
/// optimizer moves, the abort was never about F4.
#[test]
#[ignore = "D5 part 5b: separates a frozen optimizer from a blind objective"]
fn is_the_p2_abort_about_f4_or_about_alpha() {
    println!("\n=== D5 part 5b — F4 x wn_three at increasing learning rates ===");
    for alpha in [0.01f32, 0.2, 1.0] {
        let dir = tempfile::tempdir().expect("tempdir");
        let f = wn_three();
        let cfg = f.sim_config(dir.path(), 9_500_000);
        let mut t = training(FitnessFn::F4MonotoneDrift);
        t.generations = hg_train::campaign::P2_MIN_GENERATIONS;
        t.population_pairs = 3;
        t.seeds_per_member = 1;
        t.sigma = 2.0;
        t.alpha = alpha;
        t.holdout_seeds = SEEDS.to_vec();
        t.campaign_seed = 9_500_000;

        let spec = hg_train::campaign::CampaignSpec {
            campaign_id: format!("d5_p2_alpha_{alpha}"),
            base_dir: dir.path().to_path_buf(),
            sim_config: &cfg,
            cognition: CognitionConfig {
                policy: PolicyKind::StaticBias,
                budget: Some(BUDGET),
                ..Default::default()
            },
            training: t,
        };
        match hg_train::campaign::run_campaign(&spec) {
            Ok(o) => {
                let p2 = o
                    .prechecks
                    .iter()
                    .find(|p| p.name.starts_with("P-2"))
                    .map(|p| p.detail.clone())
                    .unwrap_or_else(|| "no P-2 record".into());
                println!("  alpha={alpha:<5} PASS   {p2}");
            }
            Err(e) => {
                let s = e.to_string();
                let tail = s.split("the step size").next().unwrap_or(&s).to_string();
                println!("  alpha={alpha:<5} ABORT  {}", tail.trim());
            }
        }
    }
}

// =====================================================================
// Part 4 — what F4's P-3 spread is actually made of
// =====================================================================

/// P-3 asks whether the objective separates the policy class. It does not
/// ask **on what**, and on F4 that turns out to be the whole question.
///
/// The `extreme` arm scores 1.60 on `tri_two` against random's 0.30 — and
/// `d5_f4_horizon_probe.rs` measured the monotone ceiling of these
/// substrates at ~1.60. So the arm that P-3 rewards may be the arm that
/// makes the type histogram decay smoothly, which is drift, not predictive
/// information about the dynamics (§4A.6).
///
/// `share_of_ceiling` settles it: near 1 means sorting the same values
/// would have produced the same score, so F4 saw nothing the ordering did
/// not already supply.
#[test]
#[ignore = "D5 part 4: F4 drift decomposition per arm; run explicitly"]
fn f4_drift_decomposition_per_arm() {
    println!("\n=== D5 part 4 — is F4's arm separation drift or structure? ===");
    println!(
        "{:<12} {:<10} {:>7} {:>9} {:>9} {:>10}",
        "rule set", "arm", "F4", "trend", "ceiling", "share"
    );

    for (f, n_rules) in [(wn_three(), 3usize), (tri_two(), 2)] {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = f.sim_config(dir.path(), 9_500_000);
        for (label, mk) in [
            (
                "random",
                Box::new(|s: u64| {
                    Box::new(RandomSubset::new(Some(BUDGET), s)) as Box<dyn RuleSelectionPolicy>
                }) as Box<dyn Fn(u64) -> Box<dyn RuleSelectionPolicy>>,
            ),
            (
                "theta=0",
                Box::new(move |s: u64| {
                    Box::new(StaticBias::new(n_rules, Some(BUDGET), s))
                        as Box<dyn RuleSelectionPolicy>
                }),
            ),
            (
                "extreme",
                Box::new(move |s: u64| {
                    static_bias_with(n_rules, Some(BUDGET), &allocation_extreme(n_rules, 0), s)
                }),
            ),
        ] {
            let mut agg = Vec::new();
            for &s in &SEEDS {
                let m = run_rollout(RolloutRequest {
                    config: &cfg,
                    seed: s,
                    policy: mk(s),
                    horizon_steps: STEPS,
                    probe: None,
                })
                .expect("rollout");
                agg.push(fitness::f4_drift(&m, BURN_IN));
            }
            let n = agg.len() as f64;
            let mean = |g: fn(&fitness::F4Drift) -> f64| agg.iter().map(g).sum::<f64>() / n;
            println!(
                "{:<12} {:<10} {:>7.4} {:>9.3} {:>9.4} {:>10.3}",
                f.name,
                label,
                mean(|d| d.f4),
                mean(|d| d.trend),
                mean(|d| d.monotone_ceiling),
                mean(|d| d.share_of_ceiling),
            );
        }
    }
    println!(
        "\nshare ~ 1.0 => the score is the ordering, not the dynamics.\n\
         share > 1.0 => the sequence carries structure sorting cannot supply."
    );
}
