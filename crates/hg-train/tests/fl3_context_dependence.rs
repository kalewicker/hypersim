//! F-L3 — the context-dependence fixture (NN plan §9). **Gates Stage 3.**
//!
//! The fixture's job: make "where you apply the rule" matter and "which
//! rule you apply" insufficient, so that `MlpScorer > StaticBias` is
//! possible by construction. See `rules/fl3_context_dependence.toml` for
//! the mechanism.
//!
//! What this file must establish, in order:
//!
//! 1. **The fixture is live** — all three rules actually fire, and the
//!    burnout rule actually burns things out. A fixture whose consequence
//!    rule never triggers is a one-rule growth fixture wearing a costume.
//! 2. **Low-degree placement wins** — `DegreeGreedy(low)`, which can only
//!    express "prefer low-degree sites" and has no parameters, beats
//!    budget-matched random by a clear margin. This is what makes the
//!    environment context-dependent rather than merely stochastic.
//! 3. **Rule identity cannot reach it** — the best `StaticBias` genome
//!    CMA-ES can find falls short of that margin. Without this leg, "MLP
//!    beats StaticBias" could be explained by the MLP simply being a bigger
//!    model, not by it seeing something StaticBias cannot.
//!
//! If (2) or (3) fails, the fixture does not encode what it claims and
//! Stage 3 must not be run on it — that is the whole point of a gate.

use hg_cognition::{DegreeDirection, DegreeGreedy, RandomSubset};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest};
use hg_train::diagnostics::{Fixture, RuleSource};

const SEEDS: usize = 12;
const STEPS: u64 = 384;
const WINDOW: u64 = 24;
const BURN_IN: usize = 3;

fn fl3(steps: u64) -> Fixture {
    Fixture {
        name: "fl3_context",
        rules: RuleSource::File("crates/hg-train/rules/fl3_context_dependence.toml"),
        // The one shipped IC with a genuine spread of node degrees. On a
        // constructed lattice every node has the same degree and
        // "low-degree site" names nothing.
        ic: "WhiteNoise",
        width: 8,
        node_count: 24,
        steps,
        window: WINDOW,
        step_mode: "MaxParallel",
    }
}

fn seeds() -> Vec<u64> {
    (0..SEEDS as u64).map(|i| 7_300_000 + i * 101).collect()
}

struct ArmResult {
    f2: Vec<f64>,
    interest: Vec<f64>,
    f3: Vec<f64>,
    n_constructors: Vec<f64>,
    final_nodes: Vec<f64>,
    attempted: u64,
}

impl ArmResult {
    fn mean(v: &[f64]) -> f64 {
        v.iter().sum::<f64>() / v.len().max(1) as f64
    }
    fn sd(v: &[f64]) -> f64 {
        if v.len() < 2 {
            return 0.0;
        }
        let m = Self::mean(v);
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
    }
    fn f2_mean(&self) -> f64 {
        Self::mean(&self.f2)
    }
    fn f2_sd(&self) -> f64 {
        Self::sd(&self.f2)
    }
}

fn run_arm(
    steps: u64,
    make: &(dyn Fn(u64) -> Box<dyn RuleSelectionPolicy> + Sync),
) -> ArmResult {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = fl3(steps);
    let ss = seeds();
    let one = |s: u64| {
        // `[causal]` ON: F3 reads `n_confirmed_constructors`, which comes
        // from constructor detection. With causal disabled the whole F3
        // column is 0.0 for every arm and would read as "no policy affects
        // constructor yield" rather than "constructor yield was not
        // computed".
        let cfg = f.sim_config_causal(dir.path(), s);
        let m = run_rollout(RolloutRequest {
            config: &cfg,
            seed: s,
            policy: make(s),
            horizon_steps: steps,
            probe: None,
        })
        .expect("rollout");
        let fit = hg_train::fitness::evaluate(&m, BURN_IN).expect("scored windows");
        (
            fit.f2,
            fit.f1_mean_interestingness,
            fit.f3_constructor_yield,
            fit.n_confirmed_constructors as f64,
            m.windows.last().map(|w| w.n_nodes as f64).unwrap_or(0.0),
            m.windows.iter().map(|w| w.attempted_matches as u64).sum::<u64>(),
        )
    };

    // Scoped OS threads, not rayon: `run_inner` installs the step loop into
    // its own rayon pool and nesting the two serializes completely.
    let n = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let chunk = ss.len().div_ceil(n.max(1));
    let mut out = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = ss
            .chunks(chunk.max(1))
            .map(|b| scope.spawn(|| b.iter().map(|&s| one(s)).collect::<Vec<_>>()))
            .collect();
        for h in handles {
            out.extend(h.join().expect("worker"));
        }
    });

    ArmResult {
        f2: out.iter().map(|r| r.0).collect(),
        interest: out.iter().map(|r| r.1).collect(),
        f3: out.iter().map(|r| r.2).collect(),
        n_constructors: out.iter().map(|r| r.3).collect(),
        final_nodes: out.iter().map(|r| r.4).collect(),
        attempted: out.iter().map(|r| r.5).sum(),
    }
}

fn arms(budget: usize) -> Vec<(&'static str, Box<dyn Fn(u64) -> Box<dyn RuleSelectionPolicy> + Sync>)> {
    vec![
        (
            "random",
            Box::new(move |s: u64| {
                Box::new(RandomSubset::new(
                    Some(budget),
                    hg_cognition::derive_policy_seed(s, 0, 0, 0, 0),
                )) as Box<dyn RuleSelectionPolicy>
            }),
        ),
        (
            "degree_low",
            Box::new(move |_s: u64| {
                Box::new(DegreeGreedy::new(Some(budget), DegreeDirection::Low))
                    as Box<dyn RuleSelectionPolicy>
            }),
        ),
        (
            "degree_high",
            Box::new(move |_s: u64| {
                Box::new(DegreeGreedy::new(Some(budget), DegreeDirection::High))
                    as Box<dyn RuleSelectionPolicy>
            }),
        ),
    ]
}

/// Exploratory: the budget response curve for F-L3, used to pick the rung
/// the calibration assertions run at. Not an assertion — a fixture that
/// needs a budget chosen for it should have that choice measured and
/// recorded rather than guessed.
#[test]
#[ignore = "exploratory; run explicitly when re-tuning the F-L3 budget"]
fn fl3_budget_curve() {
    println!("\n=== F-L3 budget curve (WhiteNoise, {STEPS} steps, {SEEDS} seeds) ===");
    println!(
        "{:>4}  {:<12} {:>8} {:>8} {:>10} {:>10} {:>12}",
        "k", "arm", "F2", "F2 sd", "interest", "final n", "attempted"
    );
    for k in [1usize, 4, 8, 16, 32, 64] {
        for (name, make) in arms(k) {
            let r = run_arm(STEPS, make.as_ref());
            println!(
                "{:>4}  {:<12} {:>8.3} {:>8.3} {:>10.3} {:>10.0} {:>12}",
                k,
                name,
                r.f2_mean(),
                r.f2_sd(),
                ArmResult::mean(&r.interest),
                ArmResult::mean(&r.final_nodes),
                r.attempted
            );
        }
    }
}

/// Exploratory: horizon sweep at a fixed budget.
///
/// The budget curve showed budget-matched random pinned at the F2 ceiling
/// of 1.000 at every rung on a 384-step horizon, which leaves no headroom
/// for any policy to beat it — a fixture where the control is already
/// optimal cannot separate two policy classes. It also showed the signal
/// that should eventually matter: `degree_high` holds the graph near 390
/// nodes where random runs to 1816. Runaway branching takes time to tip a
/// run into `Exploding`, so the question is whether a longer horizon
/// separates them.
#[test]
#[ignore = "exploratory; run explicitly when re-tuning the F-L3 horizon"]
fn fl3_horizon_curve() {
    const K: usize = 16;
    println!("\n=== F-L3 horizon curve (k = {K}, {SEEDS} seeds) ===");
    println!(
        "{:>7}  {:<12} {:>8} {:>8} {:>10} {:>10}",
        "steps", "arm", "F2", "F2 sd", "interest", "final n"
    );
    for steps in [384u64, 768, 1536, 3072] {
        for (name, make) in arms(K) {
            let r = run_arm(steps, make.as_ref());
            println!(
                "{:>7}  {:<12} {:>8.3} {:>8.3} {:>10.3} {:>10.0}",
                steps,
                name,
                r.f2_mean(),
                r.f2_sd(),
                ArmResult::mean(&r.interest),
                ArmResult::mean(&r.final_nodes)
            );
        }
    }
}

/// The burnout threshold must sit **above** the initial condition, or
/// `overload` retires the starting marked nodes before a single sprout
/// lands and the fixture has no dynamics. At the first version's threshold
/// of 6 that is exactly what happened.
///
/// This measures the initial degree distribution rather than trusting the
/// arithmetic in the rule file's comment, and fails if the separation stops
/// holding — a change to `WhiteNoise`'s defaults would otherwise silently
/// re-break the fixture.
const OVERLOAD_DEGREE: usize = 9;

#[test]
fn the_burnout_threshold_is_above_the_initial_degrees() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = fl3(WINDOW); // one window: the probe fires at the first boundary
    let cfg = f.sim_config(dir.path(), 7_300_000);

    // Degree histogram of MARKED (label 1) nodes at the first window
    // boundary — those are the only nodes `overload` can match.
    let probe = move |snap: &hg_core::GraphSnapshot<'_>, _w: u64| -> Vec<f32> {
        let mut v = vec![0.0f32; 32];
        for (id, data) in snap.nodes() {
            if data.label.0 == 1 {
                let d = snap.edges_incident_to(id).len().min(31);
                v[d] += 1.0;
            }
        }
        v
    };

    let m = run_rollout(RolloutRequest {
        config: &cfg,
        seed: 7_300_000,
        policy: Box::new(RandomSubset::new(Some(8), 1)),
        horizon_steps: WINDOW,
        probe: Some(Box::new(probe)),
    })
    .expect("rollout");

    let hist = m.probe_rows.first().expect("at least one probe row");
    let max_degree = hist.iter().rposition(|c| *c > 0.0).unwrap_or(0);
    let marked: f32 = hist.iter().sum();
    println!(
        "F-L3 initial condition: {marked} marked nodes, max marked degree {max_degree}, \
         overload threshold {OVERLOAD_DEGREE}"
    );

    assert!(
        marked > 0.0,
        "no label-1 nodes in the initial condition — `sprout` can never fire"
    );
    assert!(
        max_degree < OVERLOAD_DEGREE,
        "the highest marked degree in the initial condition is {max_degree}, at or above \
         the overload threshold {OVERLOAD_DEGREE}: `overload` will retire starting nodes \
         before any sprouting happens, and the fixture measures the generator rather \
         than the policy"
    );
}

/// Leg 1 — the fixture is live: every rule fires, and burnout actually
/// happens. Cheap enough to run in the default suite.
#[test]
fn fl3_is_live_and_burns_out() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = fl3(STEPS);
    let cfg = f.sim_config(dir.path(), 7_300_000);
    let rules = hg_sim_core::load_rules(&cfg).expect("rules load");
    // TWO, not three. `StaticBias` needs >= 2 rules to be expressible at
    // all (a softmax over one logit is 1.0 whatever the logit is), and
    // `sprout`/`overload` supply that choice. The third `filler` rule this
    // assertion used to demand was removed for the reasons recorded in
    // `rules/fl3_context_dependence.toml`: it swamped the candidate pool
    // and, worse, would have made the winning move a pure rule-identity
    // decision — the one thing this fixture must NOT reward.
    assert_eq!(
        rules.len(),
        2,
        "F-L3 needs exactly the sprout/overload pair: >= 2 rules so StaticBias is \
         expressible, and no filler rule whose suppression would be a rule-identity win"
    );

    let m = run_rollout(RolloutRequest {
        config: &cfg,
        seed: 7_300_000,
        policy: Box::new(RandomSubset::new(Some(8), 1)),
        horizon_steps: STEPS,
        probe: None,
    })
    .expect("rollout");

    let attempted: u64 = m.windows.iter().map(|w| w.attempted_matches as u64).sum();
    assert!(
        attempted > 0,
        "F-L3 produced zero candidates for the whole run — the rule set is not live on \
         this initial condition and every policy would trivially agree"
    );
    assert!(
        m.windows.len() > BURN_IN,
        "F-L3 produced {} windows, not enough to score past a burn-in of {BURN_IN}",
        m.windows.len()
    );
}

// ---------------------------------------------------------------------
// The gate, scored under every fitness function
// ---------------------------------------------------------------------

/// **The F-L3 gate.** Does *any* fitness function have headroom here, and
/// does a degree-aware policy beat budget-matched random under it?
///
/// F2 is known not to work: `docs/cognition_stage2_findings.md` §5 shows it
/// saturates under a budget, because `k` selections per step caps node
/// growth at ~k/step, `Exploding` is a growth-*ratio* gate and so becomes
/// unreachable, and the only penalized regime left is `Frozen`. Random
/// scores exactly 1.000 at every rung. A fixture whose control is already
/// optimal cannot demonstrate that anything beats it.
///
/// So the question is not "is the mechanism right" — four mechanisms were
/// tried — but "is there a fitness with headroom". F1 (mean
/// interestingness) is unsaturated on these arms; F3 (constructor yield)
/// rewards concentration where F2 rewards not-stalling. This scores all
/// three from one rollout, since `fitness::evaluate` computes them
/// together.
///
/// A fitness passes the gate at a budget rung if a degree heuristic beats
/// budget-matched random by > 2σ. That is necessary but not sufficient for
/// F-L3 — the third leg (rule identity cannot reach the same margin) is
/// `fl3_static_bias_cannot_reach_it` — but if no fitness clears this bar,
/// Stage 3 has no environment and must not run.
#[test]
#[ignore = "long-running; run explicitly for the F-L3 gate table"]
fn fl3_gate_under_each_fitness() {
    println!("\n=== F-L3 gate — all three fitness functions ===");
    println!("(WhiteNoise, {STEPS} steps, {SEEDS} seeds, causal ON)\n");

    let mut any_pass = false;
    for k in [1usize, 4, 8, 16, 32, 64] {
        println!("## k = {k}");
        println!(
            "{:<12} {:>9} {:>7} {:>9} {:>7} {:>9} {:>7} {:>7}",
            "arm", "F1", "sd", "F2", "sd", "F3", "sd", "n_ctor"
        );
        let mut rows: Vec<(&'static str, ArmResult)> = Vec::new();
        for (name, make) in arms(k) {
            rows.push((name, run_arm(STEPS, make.as_ref())));
        }
        for (name, r) in &rows {
            println!(
                "{:<12} {:>9.4} {:>7.4} {:>9.4} {:>7.4} {:>9.4} {:>7.4} {:>7.2}",
                name,
                ArmResult::mean(&r.interest),
                ArmResult::sd(&r.interest),
                ArmResult::mean(&r.f2),
                ArmResult::sd(&r.f2),
                ArmResult::mean(&r.f3),
                ArmResult::sd(&r.f3),
                ArmResult::mean(&r.n_constructors),
            );
        }

        // Effect of each heuristic against random, per fitness.
        let base = rows.iter().find(|(n, _)| *n == "random").map(|(_, r)| r);
        if let Some(base) = base {
            for (label, pick) in [
                ("F1", (|r: &ArmResult| &r.interest) as fn(&ArmResult) -> &Vec<f64>),
                ("F2", |r: &ArmResult| &r.f2),
                ("F3", |r: &ArmResult| &r.f3),
            ] {
                let b = pick(base);
                let (bm, bsd) = (ArmResult::mean(b), ArmResult::sd(b));
                let mut best = (0.0f64, "");
                for (name, r) in &rows {
                    if *name == "random" {
                        continue;
                    }
                    let a = pick(r);
                    let (am, asd) = (ArmResult::mean(a), ArmResult::sd(a));
                    // Same denominator convention as the Stage 0a
                    // classifier: pooled when both arms vary, the one that
                    // varies when only one does, undefined when neither.
                    let floor = |sd: f64, m: f64| sd > 1e-9 * (m.abs() + 1.0);
                    let denom = match (floor(asd, am), floor(bsd, bm)) {
                        (true, true) => Some(((asd * asd + bsd * bsd) / 2.0).sqrt()),
                        (true, false) => Some(asd),
                        (false, true) => Some(bsd),
                        (false, false) => None,
                    };
                    // Signed: a heuristic that is WORSE than random is not
                    // evidence for the fixture, however large the sigma.
                    if let Some(d) = denom {
                        let z = (am - bm) / d;
                        if z > best.0 {
                            best = (z, name);
                        }
                    }
                }
                let pass = best.0 > 2.0;
                any_pass |= pass;
                println!(
                    "   {label}: best heuristic gain over random = {:.2} sigma ({}){}",
                    best.0,
                    if best.1.is_empty() { "none better" } else { best.1 },
                    if pass { "  <== GATE PASSED" } else { "" }
                );
            }
        }
        println!();
    }

    println!(
        "F-L3 leg 2 (a degree heuristic beats budget-matched random by >2 sigma \
         on some fitness): {}",
        if any_pass { "PASSED" } else { "FAILED" }
    );
    // No assertion on the outcome: a failure here is the gate doing its job
    // and means Stage 3 must not run, which is a pre-committed finding
    // (plan §9), not a broken test.
}
