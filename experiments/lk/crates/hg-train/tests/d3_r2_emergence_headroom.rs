//! Phase D3, R-2 — does causal emergence anti-correlate with micro
//! determinism? (`hypersim_nn_experiments_plan_v2.md` §8)
//!
//! Stage 5's decoupling is the most interesting single observation in
//! C0–C5: `ei_macro` rose 2.0005 → 2.3880 (+2.84σ, the pre-registered
//! direction, confirmed) while `ce_structural` **fell** 0.3925 → 0.3529,
//! because `ei_micro` rose more (5.1027 → 6.7799). The hypothesis is false
//! while its exit criterion is true; the two came apart.
//!
//! ## The mechanism under test [Inferred]
//!
//! In Hoel's decomposition `EI = (determinism − degeneracy) · log2(M)`, and
//! causal emergence requires the coarse-graining to reduce degeneracy
//! faster than it reduces determinism. Budget-constrained allocation
//! already sharpens micro-level causation — fewer, more disjoint,
//! less-interfering rewrites per step — which removes the micro degeneracy
//! the coarse-graining was compensating for.
//!
//! > **Emergence headroom is a function of micro slop.**
//!
//! If that holds it is general — *interventions that increase micro
//! determinism reduce causal emergence* — and it is a contribution to the
//! causal-emergence literature with nothing to do with learning. Note the
//! framing throughout: this is **allocation under scarcity, not policies**.
//! Champion vs untrained was +0.26σ.
//!
//! ## Risk 6, and why this test measures two things
//!
//! Plan v2 risk 6: the correlation could be an artifact of the budget
//! sweep's own confound — budget changes `ei_micro` and `ce_structural`
//! simultaneously, so an anti-correlation could be two effects of one cause
//! rather than a relationship between them. The stated partial control is
//! to compare **across policy arms at fixed budget** as well as across
//! budgets, and report both. This test does exactly that and reports them
//! separately; agreement between the two is the evidence, and disagreement
//! is the artifact.

use hg_cognition::{derive_policy_seed, Parameterized, RandomSubset, StaticBias};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest};
use hg_train::diagnostics::{Fixture, RuleSource};

const STEPS: u64 = 384;
const WINDOW: u64 = 24;
const BURN_IN: usize = 5;
const SEEDS: usize = 5;

/// The budget ladder R-2 sweeps. `k = ∞` is omitted: Stage 0a measured that
/// every arm is bit-identical there on every rule set, so it contributes
/// three copies of one point and no variation to correlate against.
const BUDGETS: [usize; 4] = [1, 4, 16, 64];

/// The σ = 2.0 `wn_three` champion (`gen_0039.policy.toml`, genome hash
/// `0x405a3aa651eeac0f`), inlined as in `c5_stage45.rs`.
const CHAMPION: [f32; 3] = [0.033_061_95, 1.331_307_9, -2.882_334_2];

fn wn_three() -> Fixture {
    Fixture {
        name: "wn_three",
        rules: RuleSource::File("rules/wn_three.toml"),
        ic: "WhiteNoise",
        width: 8,
        node_count: 24,
        steps: STEPS,
        window: WINDOW,
        step_mode: "MaxParallel",
    }
}

fn seeds() -> Vec<u64> {
    (0..SEEDS as u64).map(|i| 7_400_000 + i * 101).collect()
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

/// Pearson correlation. `None` when either series is constant — a
/// correlation against a point mass is `0/0`, not zero, and reporting it as
/// zero would be the null-spread failure the V-track catalogued.
fn pearson(xs: &[f64], ys: &[f64]) -> Option<f64> {
    let n = xs.len();
    if n < 3 || ys.len() != n {
        return None;
    }
    let (mx, my) = (mean(xs), mean(ys));
    let sxx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    let syy: f64 = ys.iter().map(|y| (y - my).powi(2)).sum();
    if sxx <= 0.0 || syy <= 0.0 {
        return None;
    }
    let sxy: f64 = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (x - mx) * (y - my))
        .sum();
    Some(sxy / (sxx * syy).sqrt())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Random,
    Untrained,
    Champion,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::Untrained => "untrained",
            Self::Champion => "champion",
        }
    }
    fn policy(self, n_rules: usize, budget: usize, seed: u64) -> Box<dyn RuleSelectionPolicy> {
        match self {
            Self::Random => Box::new(RandomSubset::new(Some(budget), seed)),
            Self::Untrained => {
                let mut p = StaticBias::new(n_rules, Some(budget), seed);
                p.set_params(&[0.0; 3]);
                Box::new(p)
            }
            Self::Champion => {
                let mut p = StaticBias::new(n_rules, Some(budget), seed);
                p.set_params(&CHAMPION);
                Box::new(p)
            }
        }
    }
}

/// One (arm, budget) cell: seed-averaged EI quantities.
#[derive(Clone, Debug)]
struct Cell {
    arm: Arm,
    budget: usize,
    ei_micro: f64,
    ei_macro: f64,
    ce_structural: f64,
    /// `docs/instrument_registry.md` §2.1. Zero means the EI calculator
    /// never engaged and every number in this row is `unmeasured`.
    intervals: usize,
}

fn measure(arm: Arm, budget: usize) -> Cell {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = wn_three();
    let mut micro = Vec::new();
    let mut macr = Vec::new();
    let mut ce = Vec::new();
    let mut intervals = 0usize;

    for s in seeds() {
        // `[causal]` ON — without it there is no EI at all, which is bug 8's
        // sibling and would produce a table of NaN reading as "budget has no
        // effect on emergence".
        let cfg = f.sim_config_causal(dir.path(), s);
        let Ok(m) = run_rollout(RolloutRequest {
            config: &cfg,
            seed: s,
            policy: arm.policy(3, budget, derive_policy_seed(s, 0, 0, 0, 0)),
            horizon_steps: STEPS,
            probe: None,
        }) else {
            continue;
        };
        let scored = m.windows.get(BURN_IN..).unwrap_or(&[]);
        intervals += scored.iter().filter(|r| r.ei_micro.is_some()).count();
        let avg = |g: fn(&hg_schema::MetricsRow) -> Option<f64>| -> Option<f64> {
            let v: Vec<f64> = scored.iter().filter_map(g).collect();
            (!v.is_empty()).then(|| mean(&v))
        };
        if let (Some(a), Some(b), Some(c)) = (
            avg(|r| r.ei_micro),
            avg(|r| r.ei_macro),
            avg(|r| r.ce_structural),
        ) {
            micro.push(a);
            macr.push(b);
            ce.push(c);
        }
    }

    Cell {
        arm,
        budget,
        ei_micro: mean(&micro),
        ei_macro: mean(&macr),
        ce_structural: mean(&ce),
        intervals,
    }
}

#[test]
#[ignore = "12 (arm x budget) cells x 5 seeds with [causal] on; run \
            explicitly for the R-2 correlation table"]
fn r2_ce_structural_anticorrelates_with_ei_micro() {
    let mut cells = Vec::new();
    for &k in &BUDGETS {
        for arm in [Arm::Random, Arm::Untrained, Arm::Champion] {
            cells.push(measure(arm, k));
        }
    }

    println!("\n=== R-2 — emergence headroom vs micro determinism ===");
    println!("wn_three / WhiteNoise, {STEPS} steps, {SEEDS} seeds, [causal] ON\n");
    println!(
        "{:>5} {:<11} {:>10} {:>10} {:>14} {:>10}",
        "k", "arm", "ei_micro", "ei_macro", "ce_structural", "intervals"
    );
    for c in &cells {
        println!(
            "{:>5} {:<11} {:>10.4} {:>10.4} {:>14.4} {:>10}",
            c.budget,
            c.arm.label(),
            c.ei_micro,
            c.ei_macro,
            c.ce_structural,
            c.intervals
        );
    }

    // Rule 1: a row whose EI counter read zero carries no information and
    // must not enter a correlation as a number.
    let live: Vec<&Cell> = cells.iter().filter(|c| c.intervals > 0).collect();
    let dead = cells.len() - live.len();
    if dead > 0 {
        println!(
            "\n{dead} of {} cells are `unmeasured` (ei_intervals_emitted == 0) and are \
             excluded — not folded in as zeros (docs/instrument_registry.md §1).",
            cells.len()
        );
    }

    // ---- the pooled correlation --------------------------------------
    let xs: Vec<f64> = live.iter().map(|c| c.ei_micro).collect();
    let ys: Vec<f64> = live.iter().map(|c| c.ce_structural).collect();
    let pooled = pearson(&xs, &ys);
    println!(
        "\npooled over all {} live cells:  r(ei_micro, ce_structural) = {}",
        live.len(),
        pooled.map(|r| format!("{r:+.4}")).unwrap_or_else(|| "undefined".into())
    );

    // ---- risk 6's control: across ARMS at FIXED budget ----------------
    //
    // Budget moves both quantities at once, so a pooled anti-correlation is
    // consistent with two effects of one cause. Within a budget rung the
    // budget is held constant and only the allocation differs, so a
    // surviving anti-correlation there is evidence of a relationship rather
    // than of the confound.
    println!("\n--- risk 6 control: within budget rung (arms only) ---");
    let mut within: Vec<f64> = Vec::new();
    for &k in &BUDGETS {
        let rung: Vec<&&Cell> = live.iter().filter(|c| c.budget == k).collect();
        let x: Vec<f64> = rung.iter().map(|c| c.ei_micro).collect();
        let y: Vec<f64> = rung.iter().map(|c| c.ce_structural).collect();
        match pearson(&x, &y) {
            Some(r) => {
                println!("  k = {k:>2}: r = {r:+.4}  ({} arms)", rung.len());
                within.push(r);
            }
            None => println!(
                "  k = {k:>2}: undefined ({} arms, one series constant)",
                rung.len()
            ),
        }
    }

    // ---- and across BUDGETS at fixed arm ------------------------------
    println!("\n--- across budget rungs, one arm at a time ---");
    let mut across: Vec<f64> = Vec::new();
    for arm in [Arm::Random, Arm::Untrained, Arm::Champion] {
        let series: Vec<&&Cell> = live.iter().filter(|c| c.arm == arm).collect();
        let x: Vec<f64> = series.iter().map(|c| c.ei_micro).collect();
        let y: Vec<f64> = series.iter().map(|c| c.ce_structural).collect();
        // Spread, reported alongside r. A Pearson correlation over a series
        // that barely varies is arithmetically defined and carries almost
        // no information — the same null-spread hazard `f4_null_sd` and the
        // V-track's arm D are about, one level up. Without this column a
        // near-zero r on a flat series reads as "no relationship" when the
        // honest reading is "not enough variation to have one".
        let span = |v: &[f64]| {
            v.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                - v.iter().cloned().fold(f64::INFINITY, f64::min)
        };
        match pearson(&x, &y) {
            Some(r) => {
                println!(
                    "  {:<11}: r = {r:+.4}  ({} rungs, ce span {:.4}, ei span {:.4}){}",
                    arm.label(),
                    series.len(),
                    span(&y),
                    span(&x),
                    if span(&y) < 0.02 { "  <-- LOW POWER" } else { "" }
                );
                across.push(r);
            }
            None => println!(
                "  {:<11}: undefined ({} rungs, one series constant)",
                arm.label(),
                series.len()
            ),
        }
    }

    // ---- verdict ------------------------------------------------------
    println!("\n=== R-2 verdict ===");
    let neg_within = within.iter().filter(|r| **r < 0.0).count();
    let neg_across = across.iter().filter(|r| **r < 0.0).count();
    println!(
        "within-rung correlations negative: {neg_within}/{}",
        within.len()
    );
    println!(
        "across-rung correlations negative: {neg_across}/{}",
        across.len()
    );
    // **The within-rung column is the control**, because risk 6 designates
    // it as such: it holds budget fixed and varies only allocation, so an
    // anti-correlation there cannot be the budget moving both quantities at
    // once. The across-rung column is the *confounded* view and is reported
    // as corroboration, not as a second test that has to agree.
    //
    // Both are printed above either way. The verdict rests on the control
    // because that is what a control is for.
    if within.is_empty() {
        println!("No within-rung correlation is defined; risk 6 cannot be controlled here.");
    } else if neg_within == within.len() {
        println!(
            "CONTROL PASSES: all {}/{} within-rung correlations are negative, with budget \
             held fixed. The anti-correlation is NOT the budget moving both quantities at \
             once, so 'emergence headroom is a function of micro slop' is supported as a \
             RELATIONSHIP between allocation-driven micro determinism and causal \
             emergence.",
            neg_within,
            within.len()
        );
        if neg_across < across.len() {
            println!(
                "  The confounded across-rung view is {neg_across}/{} negative. Check the \
                 LOW POWER markers above before reading any exception as a counterexample: \
                 a Pearson r over four points whose y barely moves is defined but carries \
                 almost nothing.",
                across.len()
            );
        }
    } else {
        println!(
            "CONTROL FAILS: only {neg_within}/{} within-rung correlations are negative. \
             Risk 6 is live — budget may be moving ei_micro and ce_structural together \
             with no relationship between them. The Stage 5 decoupling stands as an \
             observation; the mechanism does not.",
            within.len()
        );
    }

    // Machinery only. Every sign pattern above is a legitimate scientific
    // outcome and the test must not prefer one.
    assert!(!cells.is_empty());
    assert!(
        !live.is_empty(),
        "every cell came back with ei_intervals_emitted == 0 — the EI calculator never \
         engaged, so this pass measured nothing. Check [causal] and ei_interval_steps \
         before reading any of the table above."
    );
}

#[test]
fn pearson_is_undefined_rather_than_zero_on_a_constant_series() {
    // The null-spread discipline, at the smallest scale it applies. A
    // correlation against a point mass is 0/0; returning 0.0 would report
    // "no relationship" where the honest answer is "no measurement".
    assert_eq!(pearson(&[1.0, 1.0, 1.0], &[1.0, 2.0, 3.0]), None);
    assert_eq!(pearson(&[1.0, 2.0, 3.0], &[5.0, 5.0, 5.0]), None);
    assert_eq!(pearson(&[1.0, 2.0], &[1.0, 2.0]), None, "n < 3");
}

#[test]
fn pearson_recovers_known_correlations() {
    let r = pearson(&[1.0, 2.0, 3.0, 4.0], &[2.0, 4.0, 6.0, 8.0]).expect("defined");
    assert!((r - 1.0).abs() < 1e-12);
    let r = pearson(&[1.0, 2.0, 3.0, 4.0], &[8.0, 6.0, 4.0, 2.0]).expect("defined");
    assert!((r + 1.0).abs() < 1e-12, "perfect anti-correlation is -1");
}
