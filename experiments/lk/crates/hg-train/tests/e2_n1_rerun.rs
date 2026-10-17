//! Phase E2 — the D6 re-run under N-1, and the α/σ re-derivation it forces
//! (`hypersim_nn_experiments_plan_v3.md` §4, §10).
//!
//! > **Phase E2.** D6 re-run under N-1. Re-derive α/σ.
//! > **Exit:** margin survives; the calibrated-vs-mis-scaled gap narrows as
//! > predicted, or the prediction is recorded as wrong.
//!
//! Plan §4 is explicit that the D6 calibration's *justification* does not
//! carry: `|Δθ| ≈ 1.56·α/σ` was a model for landing inside the Gumbel-live
//! window, and N-1 removes the window. Re-deriving means measuring again,
//! not re-reading D6's arithmetic with the conclusion already fixed.

use hg_cognition::features::FeatureSet;
use hg_cognition::{
    CognitionConfig, MlpScorer, Parameterized, PolicyKind, RandomSubset, ScoreNorm,
};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest};
use hg_train::campaign::{n_params_for, CampaignSpec};
use hg_train::config::TrainingConfig;
use hg_train::diagnostics::{Fixture, RuleSource};
use hg_train::fitness::{self, FitnessFn};
use hg_train::prechecks::{self, MLP_INIT_L2};

const STEPS: u64 = 384;
const WINDOW: u64 = 6;
const BURN_IN: usize = 5;
const BUDGET: usize = 4;
const HIDDEN: usize = 16;
const WL: usize = 16;
const N_RULES: usize = 2;
/// The same frozen hold-out D6 used, so the two phases' arm tables are
/// scored on identical seeds and nothing in a comparison is a seed effect.
const HOLDOUT: [u64; 5] = [9_600_001, 9_600_002, 9_600_003, 9_600_004, 9_600_005];

fn tri_two() -> Fixture {
    Fixture {
        name: "tri_two",
        rules: RuleSource::File("rules/tri_two.toml"),
        ic: "TriLattice",
        width: 8,
        node_count: 24,
        steps: STEPS,
        window: WINDOW,
        step_mode: "MaxParallel",
    }
}

fn mlp_cognition(set: &str) -> CognitionConfig {
    let mut c = CognitionConfig {
        policy: PolicyKind::MlpScorer,
        budget: Some(BUDGET),
        ..Default::default()
    };
    c.mlp.hidden = HIDDEN;
    c.feature_set = set.to_string();
    c
}

fn training() -> TrainingConfig {
    let mut t = TrainingConfig::default();
    t.fitness = FitnessFn::F5ConstructorReuse;
    t.horizon_steps = STEPS;
    t.burn_in_windows = BURN_IN;
    t.holdout_seeds = HOLDOUT.to_vec();
    t
}

fn mk_mlp(
    theta: Vec<f32>,
    set: FeatureSet,
    norm: ScoreNorm,
) -> impl Fn(u64) -> Box<dyn RuleSelectionPolicy> {
    move |s: u64| {
        let mut p = MlpScorer::with_features_and_norm(
            N_RULES,
            WL,
            HIDDEN,
            Some(BUDGET),
            s,
            set,
            norm,
        );
        p.set_params(&theta);
        Box::new(p) as Box<dyn RuleSelectionPolicy>
    }
}

fn l2(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn scaled(dir: &[f32], to: f32) -> Vec<f32> {
    let n = l2(dir);
    if n == 0.0 {
        return dir.to_vec();
    }
    dir.iter().map(|v| v * to / n).collect()
}

// =====================================================================
// Part 1 — where the neutral point actually is under N-1
// =====================================================================

/// Plan §5.1 makes the neutral point a **measurement**, and plan §4 predicts
/// where it moves to: under N-1, `θ = 0` gives `sd = 0` gives uniform —
/// still random-equivalent, but "by a *different* mechanism (degenerate
/// normalization, not sub-threshold spread) and no longer surrounded by a
/// basin. Any small perturbation now produces real selection change."
///
/// The second clause is the one worth checking rather than inheriting,
/// because `pool_standardize` divides by `sd + ε` with a **fixed** `ε`. That
/// ε is a floor, and a floor is exactly the shape that produces a basin. So
/// the question is not whether N-1 removed the basin but how far it shrank
/// it, and the answer is a radius in ‖θ‖ that this measures directly.
///
/// Two things are reported per magnitude:
///
/// - **score spread** after normalization, against the Gumbel sd of 1.28 —
///   the same currency D6 §5's table used, so the two are readable side by
///   side;
/// - **selection divergence** against a rollout at `MLP_INIT_L2`, which is
///   the behavioural statistic and the one that decides. A policy whose
///   selections never differ from the reference over 64 steps *is* the
///   reference policy, whatever its parameters read.
#[test]
#[ignore = "E2 part 1: the neutral point and the residual ε-basin; run explicitly"]
fn where_the_neutral_point_sits_under_n1() {
    println!("\n=== E2 part 1 — the neutral point under N-1 ===");

    let cog = mlp_cognition("v1");
    let n = n_params_for(&cog, N_RULES);
    // The campaign's own init draw, not a fresh one, so this arm and the
    // campaign's `untrained` arm are the same genome and the numbers below
    // are directly comparable to part 3's arm table.
    let dir = prechecks::class_init(PolicyKind::MlpScorer, n, 9_600_000u64 ^ 0x1_c001_1u64);
    let dir_alt = prechecks::class_init(PolicyKind::MlpScorer, n, 9999);

    let dir_t = tempfile::tempdir().expect("tempdir");
    let cfg = tri_two().sim_config_causal(dir_t.path(), 9_600_000);

    let trace_at = |theta: Vec<f32>, norm: ScoreNorm| {
        prechecks::trace_rules(
            &cfg,
            HOLDOUT[0],
            64,
            N_RULES,
            mk_mlp(theta, FeatureSet::V1, norm)(HOLDOUT[0]),
        )
        .expect("trace")
    };

    println!("\n## selection divergence over 64 steps, each norm against ITS OWN");
    println!("## reference at |theta| = {MLP_INIT_L2}");
    println!(
        "{:>12}  {:>14}  {:>10}  {}",
        "|theta|", "norm", "diverge@", "reading"
    );

    // Decades either side of the init norm, plus the exact init (identity
    // control) and zero (the degenerate case).
    let mags: [f32; 10] = [
        0.0, 1e-4, 1e-3, 1e-2, 0.1, 0.5, 1.0, MLP_INIT_L2, 42.0, 420.0,
    ];
    for norm in [ScoreNorm::PoolStandard, ScoreNorm::Raw] {
        // Each norm is compared to itself at the init norm. Comparing the
        // raw rows against an N-1 reference would only restate that the two
        // norms differ, which is not the question: the question is whether
        // *within* a norm the policy depends on |theta|.
        let reference = trace_at(scaled(&dir, MLP_INIT_L2), norm);
        for m in mags {
            let theta = if m == 0.0 {
                vec![0.0f32; n]
            } else {
                scaled(&dir, m)
            };
            let tr = trace_at(theta, norm);
            let (at, cmp) = prechecks::selection_divergence(&reference, &tr);
            let reading = match at {
                None => "SAME POLICY as the reference",
                Some(0) => "differs at once",
                Some(_) => "differs",
            };
            println!(
                "{:>12}  {:>14}  {:>10}  {reading} (of {cmp})",
                format!("{m:.4}"),
                match norm {
                    ScoreNorm::PoolStandard => "N-1",
                    ScoreNorm::Raw => "raw",
                },
                match at {
                    None => "NONE".to_string(),
                    Some(i) => i.to_string(),
                }
            );
        }
    }

    // The direction control. Two *different* directions at the same norm
    // must be different policies — otherwise "same policy" above would be
    // reporting a dead instrument rather than scale invariance.
    let n1_reference = trace_at(scaled(&dir, MLP_INIT_L2), ScoreNorm::PoolStandard);
    let alt = trace_at(
        scaled(&dir_alt, MLP_INIT_L2),
        ScoreNorm::PoolStandard,
    );
    let (at, cmp) = prechecks::selection_divergence(&n1_reference, &alt);
    println!(
        "\n  control — a DIFFERENT direction at the same norm: diverges at {:?} of {cmp}",
        at
    );

    // The divergence statistic above saturates: "differs at once" is what
    // both a random-equivalent policy and a barely-perturbed one report, so
    // it locates the plateau's *upper* edge and says nothing about how far
    // below it the policy is still random. The fitness sweep does say —
    // random is a known number on this cell, so distance from it in pooled
    // sd is the neutral-point statistic plan §5.1 asks for.
    let t = training();
    let a_rnd = prechecks::probe_arm("random", &cfg, &t, &HOLDOUT, |s| {
        Box::new(RandomSubset::new(Some(BUDGET), s)) as Box<dyn RuleSelectionPolicy>
    });
    println!(
        "\n## F5 across |theta| under N-1, against budget-matched random\n   random = {:.4} +- {:.4}",
        a_rnd.mean(),
        a_rnd.sd()
    );
    println!(
        "{:>12}  {:>12}  {:>10}  {:>12}  {}",
        "|theta|", "F5", "sd", "vs random", "reading"
    );
    for m in [0.0f32, 1e-2, 0.1, 0.5, 1.0, MLP_INIT_L2, 42.0] {
        let theta = if m == 0.0 {
            vec![0.0f32; n]
        } else {
            scaled(&dir, m)
        };
        let a = prechecks::probe_arm(
            format!("|theta|={m}"),
            &cfg,
            &t,
            &HOLDOUT,
            mk_mlp(theta, FeatureSet::V1, ScoreNorm::PoolStandard),
        );
        let pooled = ((a_rnd.sd().powi(2) + a.sd().powi(2)) / 2.0).sqrt();
        let z = (a.mean() - a_rnd.mean()) / pooled.max(1e-9);
        println!(
            "{:>12}  {:>12.4}  {:>10.4}  {:>11.2}sd  {}",
            format!("{m:.4}"),
            a.mean(),
            a.sd(),
            z,
            if z.abs() < 2.0 {
                "random-equivalent"
            } else {
                "separated from random"
            }
        );
    }

    // Under N-1 the policy is a function of theta's DIRECTION alone (the
    // table above: 7 and 42 agree to every printed digit). The campaign's
    // untrained arm is therefore one draw from the sphere, and
    // `champion − untrained` is measured against whatever that draw scored.
    // Plan §5.2 requires >= 5 draws of any quantity a decision rests on, and
    // the primary statistic's own baseline is exactly such a quantity.
    println!("\n## the untrained arm is a DRAW: 8 directions at |theta| = {MLP_INIT_L2}");
    let mut untrained: Vec<f64> = Vec::new();
    for k in 0..8u64 {
        let g = prechecks::class_init(PolicyKind::MlpScorer, n, 0xE2_D00D + k);
        let a = prechecks::probe_arm(
            "draw",
            &cfg,
            &t,
            &HOLDOUT,
            mk_mlp(g, FeatureSet::V1, ScoreNorm::PoolStandard),
        );
        println!(
            "   draw {k}:  {:>10.4} +- {:>8.4}   ({:+.2} sd from random)",
            a.mean(),
            a.sd(),
            (a.mean() - a_rnd.mean()) / a_rnd.sd().max(1e-9)
        );
        untrained.push(a.mean());
    }
    let m_u = untrained.iter().sum::<f64>() / untrained.len() as f64;
    let sd_u = (untrained.iter().map(|x| (x - m_u).powi(2)).sum::<f64>()
        / (untrained.len() - 1) as f64)
        .sqrt();
    println!(
        "\n   across DIRECTIONS: {m_u:.4} +- {sd_u:.4}   (range {:.4} .. {:.4})",
        untrained.iter().cloned().fold(f64::INFINITY, f64::min),
        untrained.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    );
    println!(
        "   across SEEDS at one direction: +- {:.4}\n   direction sd / seed sd = {:.2}",
        a_rnd.sd(),
        sd_u / a_rnd.sd().max(1e-9)
    );
}

// =====================================================================
// Part 2 — re-deriving sigma, from measurement
// =====================================================================

/// D6 §5.3 fixed σ by a constraint that no longer exists: keep `σ√n` at
/// ~30% of the init norm so the *population* stays inside the band where
/// the scorer can act. Under N-1 there is no band — part 1 measures the
/// policy as invariant to ‖θ‖ above the ε floor — so that constraint is
/// void and σ needs a different one.
///
/// What still binds is the thing D6 §1.1 measured for a different purpose:
/// **one perturbation has to move the fitness by more than seed noise**, or
/// the ES is ranking noise. That is a property of σ and the objective
/// jointly, it is measurable without running a campaign, and it does not
/// depend on the Gumbel geometry at all — which is why it survives N-1
/// when the band argument does not.
///
/// Reported per σ:
///
/// - `|r⁺ − r⁻|` averaged over antithetic pairs — the signal the optimizer
///   ranks on, in the units the objective is measured in;
/// - the same against the **seed** sd, which is the noise it must clear;
/// - the angle `atan(σ√n / ‖θ‖)` — how far a perturbation moves the
///   *direction*, which under scale invariance is the only thing that moved.
///   At 90° the population is a set of uniformly random directions and the
///   "search" is a random restart per generation.
#[test]
#[ignore = "E2 part 2: the sigma sweep; long-running, run explicitly"]
fn sigma_re_derived_from_the_antithetic_signal() {
    println!("\n=== E2 part 2 — re-deriving sigma under N-1 ===");

    let cog = mlp_cognition("v1");
    let n = n_params_for(&cog, N_RULES);
    let dir_t = tempfile::tempdir().expect("tempdir");
    let cfg = tri_two().sim_config_causal(dir_t.path(), 9_600_000);
    let t = training();

    // Measured at TWO starting directions, not one. Under N-1 the genome is
    // a direction (part 1), the campaign starts from a draw, and part 1
    // measures those draws as spanning a wide range of F5 — so a calibration
    // read off one of them is a calibration for one point in the landscape.
    // If the verdict agrees at both it is a property of sigma; if it does
    // not, that is the finding.
    for (where_, init_seed) in [
        ("the campaign's own init draw", 9_600_000u64 ^ 0x1_c001_1u64),
        ("a second draw at the same norm", 4242),
    ] {
    let theta0 = prechecks::class_init(PolicyKind::MlpScorer, n, init_seed);

    // The noise floor: the same policy across the hold-out seeds. This is
    // what an antithetic difference has to beat to be signal.
    let base = prechecks::probe_arm(
        "init",
        &cfg,
        &t,
        &HOLDOUT,
        mk_mlp(theta0.clone(), FeatureSet::V1, ScoreNorm::PoolStandard),
    );
    let seed_sd = base.sd();
    println!("\n### at {where_}");
    println!(
        "  seed noise at the init genome: {:.4} +- {:.4} over {} seeds",
        base.mean(),
        seed_sd,
        base.scores.len()
    );
    println!("  n_params = {n}, |theta_init| = {:.4}", l2(&theta0));

    let pairs = 4usize;
    let seeds: [u64; 2] = [7_100_001, 7_100_002];
    println!(
        "\n{:>8}  {:>10}  {:>8}  {:>14}  {:>10}  {}",
        "sigma", "sigma*sqn", "angle", "mean |r+ - r-|", "vs seedsd", "verdict"
    );

    for sigma in [0.02f32, 0.05, 0.10, 0.30, 1.0, 2.0] {
        let mut rng =
            <rand_chacha::ChaCha8Rng as rand::SeedableRng>::seed_from_u64(0xE2_0000 + 1);
        let mut diffs = Vec::new();
        for _ in 0..pairs {
            let eps: Vec<f32> = (0..n)
                .map(|_| hg_train::optim::standard_normal(&mut rng))
                .collect();
            let plus: Vec<f32> = theta0
                .iter()
                .zip(&eps)
                .map(|(a, e)| a + sigma * e)
                .collect();
            let minus: Vec<f32> = theta0
                .iter()
                .zip(&eps)
                .map(|(a, e)| a - sigma * e)
                .collect();
            let score = |g: &Vec<f32>| -> f64 {
                let mut acc = Vec::new();
                for &s in &seeds {
                    let Ok(m) = run_rollout(RolloutRequest {
                        config: &cfg,
                        seed: s,
                        policy: mk_mlp(g.clone(), FeatureSet::V1, ScoreNorm::PoolStandard)(s),
                        horizon_steps: STEPS,
                        probe: None,
                    }) else {
                        continue;
                    };
                    let Ok(f) = fitness::evaluate(&m, BURN_IN) else {
                        continue;
                    };
                    if let Ok(v) = f.select(FitnessFn::F5ConstructorReuse) {
                        acc.push(v);
                    }
                }
                if acc.is_empty() {
                    f64::NAN
                } else {
                    acc.iter().sum::<f64>() / acc.len() as f64
                }
            };
            let d = (score(&plus) - score(&minus)).abs();
            if d.is_finite() {
                diffs.push(d);
            }
        }
        let mean_d = if diffs.is_empty() {
            f64::NAN
        } else {
            diffs.iter().sum::<f64>() / diffs.len() as f64
        };
        let drift = sigma * (n as f32).sqrt();
        let angle = (drift / l2(&theta0)).atan().to_degrees();
        println!(
            "{:>8.3}  {:>10.2}  {:>7.1}°  {:>14.2}  {:>10.2}  {}",
            sigma,
            drift,
            angle,
            mean_d,
            mean_d / seed_sd.max(1e-9),
            if mean_d > seed_sd {
                "signal > noise"
            } else {
                "under the noise floor"
            }
        );
    }
    }
}

// =====================================================================
// Part 2b — is the perturbation LOCAL, not merely visible
// =====================================================================

/// Part 2's statistic answers "can the objective see this perturbation".
/// Under N-1 the answer turns out to be yes at every σ tested, which makes
/// it useless as a selection criterion and — worse — **actively
/// misleading**: `|r⁺ − r⁻|` is large at σ = 2.0 too, but there the two arms
/// are near-orthogonal directions, so the difference is the spread between
/// two unrelated policies rather than a signal about a local slope. A
/// criterion built on it would happily license a random restart per
/// generation.
///
/// The property `OpenAiEs` actually assumes is **local linearity**: it
/// estimates a gradient from `(r⁺ − r⁻)` and that is only a gradient if
/// `r(θ ± σε)` is approximately affine over the perturbation. Measured
/// directly, per pair:
///
/// ```text
/// kappa = |r⁺ + r⁻ − 2·r₀| / |r⁺ − r⁻|
/// ```
///
/// The numerator is the second difference — exactly the curvature the
/// linear model discards — and the denominator is the signal that model
/// extracts. `kappa ≪ 1` is the regime ES was derived for. `kappa ≳ 1` means
/// the estimator is being handed mostly curvature, and the update direction
/// is not the local ascent direction whatever its magnitude.
///
/// This is the criterion that survives N-1, because it is a statement about
/// the objective's shape rather than about the Gumbel geometry that N-1
/// deleted.
#[test]
#[ignore = "E2 part 2b: local linearity across sigma; run explicitly"]
fn sigma_bounded_above_by_local_linearity() {
    println!("\n=== E2 part 2b — is the perturbation local? ===");

    let cog = mlp_cognition("v1");
    let n = n_params_for(&cog, N_RULES);
    let dir_t = tempfile::tempdir().expect("tempdir");
    let cfg = tri_two().sim_config_causal(dir_t.path(), 9_600_000);
    let theta0 = prechecks::class_init(PolicyKind::MlpScorer, n, 9_600_000u64 ^ 0x1_c001_1u64);
    let seeds: [u64; 2] = [7_100_001, 7_100_002];

    let score = |g: &Vec<f32>| -> f64 {
        let mut acc = Vec::new();
        for &s in &seeds {
            let Ok(m) = run_rollout(RolloutRequest {
                config: &cfg,
                seed: s,
                policy: mk_mlp(g.clone(), FeatureSet::V1, ScoreNorm::PoolStandard)(s),
                horizon_steps: STEPS,
                probe: None,
            }) else {
                continue;
            };
            let Ok(f) = fitness::evaluate(&m, BURN_IN) else {
                continue;
            };
            if let Ok(v) = f.select(FitnessFn::F5ConstructorReuse) {
                acc.push(v);
            }
        }
        if acc.is_empty() {
            f64::NAN
        } else {
            acc.iter().sum::<f64>() / acc.len() as f64
        }
    };

    // r0 is evaluated once, on the same seeds, and reused across sigma — it
    // is a property of theta0, not of the perturbation.
    let r0 = score(&theta0);
    println!("\n  r0 at the campaign init = {r0:.4}  (seeds {seeds:?})");
    println!(
        "\n{:>8}  {:>8}  {:>12}  {:>12}  {:>8}  {:>10}  {}",
        "sigma", "angle", "|r+ - r-|", "|r+ +r- -2r0|", "kappa", "(naive)", "reading"
    );

    for sigma in [0.02f32, 0.05, 0.10, 0.30, 1.0, 2.0] {
        // The SAME epsilon draws at every sigma, so the rows differ in the
        // step size alone and not in which directions were sampled.
        let mut rng = <rand_chacha::ChaCha8Rng as rand::SeedableRng>::seed_from_u64(0xE2_2B);
        let mut kappas = Vec::new();
        let mut sigs = Vec::new();
        let mut curvs = Vec::new();
        for _ in 0..4 {
            let eps: Vec<f32> = (0..n)
                .map(|_| hg_train::optim::standard_normal(&mut rng))
                .collect();
            let plus: Vec<f32> = theta0.iter().zip(&eps).map(|(a, e)| a + sigma * e).collect();
            let minus: Vec<f32> = theta0.iter().zip(&eps).map(|(a, e)| a - sigma * e).collect();
            let (rp, rm) = (score(&plus), score(&minus));
            if !rp.is_finite() || !rm.is_finite() {
                continue;
            }
            let sig = (rp - rm).abs();
            let curv = (rp + rm - 2.0 * r0).abs();
            sigs.push(sig);
            curvs.push(curv);
            if sig > 0.0 {
                kappas.push(curv / sig);
            }
        }
        let mean = |v: &Vec<f64>| {
            if v.is_empty() {
                f64::NAN
            } else {
                v.iter().sum::<f64>() / v.len() as f64
            }
        };
        // RATIO OF MEANS, not mean of ratios. A per-pair ratio whose
        // denominator is itself noisy is the classic unstable ratio
        // estimator: over four pairs, one pair that happens to land near
        // |r+ - r-| = 0 dominates the average and reports curvature
        // domination where the aggregate shows none. Both are printed so the
        // divergence is visible rather than hidden by the choice.
        let k = mean(&curvs) / mean(&sigs).max(1e-9);
        let k_naive = mean(&kappas);
        let angle = (sigma * (n as f32).sqrt() / l2(&theta0)).atan().to_degrees();
        println!(
            "{:>8.3}  {:>7.1}°  {:>12.2}  {:>12.2}  {:>8.2}  {:>10.2}  {}",
            sigma,
            angle,
            mean(&sigs),
            mean(&curvs),
            k,
            k_naive,
            if k < 0.5 {
                "locally linear — ES's estimator is a gradient"
            } else if k < 1.5 {
                "marginal"
            } else {
                "CURVATURE-DOMINATED — not a local gradient"
            }
        );
    }
}

// =====================================================================
// Part 4 — powering the class control
// =====================================================================

/// Part 3's class-level control lands at **1.85 direction-sd**, under the
/// 2σ bar, estimated from eight draws. That is the one statistic in E2 that
/// could undercut the headline, and eight samples is not enough power for
/// it: the sd of an sd estimated from n = 8 is about 25%, so "1.85" could
/// as easily be 1.4 or 2.5.
///
/// Plan §5.2's gate discipline exists for exactly this shape — a quantity a
/// conclusion rests on, resolved by however many draws happened to run
/// first. Thirty draws, and two statistics rather than one:
///
/// - the **parametric** z, `(champion − mean) / sd`, which assumes a normal
///   upper tail that nothing has established;
/// - the **nonparametric** count, how many draws reach the champion, which
///   assumes nothing at all and is the number a skeptical reader wants.
///
/// The champion value is passed in rather than recomputed so this reads the
/// arm-A result exactly as part 3 published it.
#[test]
#[ignore = "E2 part 4: 30-draw class control; ~30 min, run explicitly"]
fn the_class_control_at_thirty_draws() {
    println!("\n=== E2 part 4 — how good is a random direction? ===");

    // Arm A, part 3. Hard-coded deliberately: this test adjudicates a
    // published number and must not silently drift off it.
    const CHAMPION_A: f64 = 1148.8103;

    let cog = mlp_cognition("v1");
    let n = n_params_for(&cog, N_RULES);
    let dir_t = tempfile::tempdir().expect("tempdir");
    let cfg = tri_two().sim_config_causal(dir_t.path(), 9_600_000);
    let t = training();

    let mut means: Vec<f64> = Vec::new();
    for k in 0..30u64 {
        let g = prechecks::class_init(PolicyKind::MlpScorer, n, 0xE2_D00D + k);
        let a = prechecks::probe_arm(
            "draw",
            &cfg,
            &t,
            &HOLDOUT,
            mk_mlp(g, FeatureSet::V1, ScoreNorm::PoolStandard),
        );
        means.push(a.mean());
    }
    means.retain(|v| v.is_finite());
    let m = means.iter().sum::<f64>() / means.len() as f64;
    let sd =
        (means.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (means.len() - 1) as f64).sqrt();
    let mut sorted = means.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let reach = means.iter().filter(|v| **v >= CHAMPION_A).count();

    println!(
        "\n  {} draws: {m:.4} +- {sd:.4}   (min {:.2}, max {:.2})",
        means.len(),
        sorted[0],
        sorted[sorted.len() - 1]
    );
    println!(
        "  deciles: {}",
        (0..=10)
            .map(|i| format!("{:.0}", sorted[(i * (sorted.len() - 1)) / 10]))
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("\n  champion (arm A) = {CHAMPION_A:.4}");
    println!(
        "  parametric      : {:.2} direction-sd above the class mean",
        (CHAMPION_A - m) / sd.max(1e-9)
    );
    println!(
        "  nonparametric   : {reach}/{} draws reach it; best draw is {:.2} ({:+.2} sd from champion)",
        means.len(),
        sorted[sorted.len() - 1],
        (sorted[sorted.len() - 1] - CHAMPION_A) / sd.max(1e-9)
    );
    println!(
        "\n  and against budget-matched random (825.5782): {}/{} draws beat it",
        means.iter().filter(|v| **v > 825.5782).count(),
        means.len()
    );
}

// =====================================================================
// Part 3 — the campaigns
// =====================================================================

/// Three campaigns, all under N-1, all on F5 × `tri_two`:
///
/// | arm | features | σ, α | what it isolates |
/// |---|---|---|---|
/// | A | v1 | calibrated | N-1 alone, against D6 §7A.1 |
/// | B | v1 | 2.0, 0.2 | the mis-scaled search, against D6 §6.1 |
/// | C | v2 | calibrated | E0's rescale, against A |
///
/// A and B are E2's exit: they are the calibrated-vs-mis-scaled gap, and
/// they differ from each other in nothing but σ and α. C is separate
/// because E0's feature rescale is otherwise unexercised in any campaign,
/// and folding it into A would confound the one comparison E2 exists to
/// make. **Absolute fitness is comparable to D6 only for A and B**; C
/// changes the feature encoding and so changes what the network sees.
///
/// # The primary statistic needs a second denominator
///
/// `ArmTable::champion_vs_untrained` divides by pooled **seed** sd. Part 1
/// measures the untrained arm's spread across **directions** at 224.50
/// against a seed spread of 38.61 — 5.8× larger — because under N-1
/// `class_init` samples a direction and the policy is a function of nothing
/// else. So `champion − untrained` in seed units answers *"did training beat
/// this particular draw"*, and the class-level question *"did training beat
/// an untrained network of this class"* needs the direction spread as its
/// denominator. Both are reported; neither is discarded.
///
/// The A-vs-B comparison is untouched by this, since both campaigns start
/// from the same `campaign_seed` and therefore the same draw.
#[test]
#[ignore = "E2 part 3: three campaigns; hours, run explicitly"]
fn e2_campaigns() {
    println!("\n=== E2 part 3 — the D6 re-run under N-1 ===");

    // Filled in from part 2 before this runs; see the findings doc.
    let calibrated = (0.10f32, 0.032f32);

    // The class-level baseline, per feature set: what an untrained network
    // of this class scores, as a distribution rather than as a point.
    let mut direction_sd: std::collections::HashMap<&str, (f64, f64)> =
        std::collections::HashMap::new();
    {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = tri_two().sim_config_causal(dir.path(), 9_600_000);
        let t = training();
        for set in ["v1", "v2"] {
            let cog = mlp_cognition(set);
            let n = n_params_for(&cog, N_RULES);
            let fs = FeatureSet::parse(set).expect("feature set");
            let means: Vec<f64> = (0..8u64)
                .map(|k| {
                    let g = prechecks::class_init(PolicyKind::MlpScorer, n, 0xE2_D00D + k);
                    prechecks::probe_arm(
                        "draw",
                        &cfg,
                        &t,
                        &HOLDOUT,
                        mk_mlp(g, fs, ScoreNorm::PoolStandard),
                    )
                    .mean()
                })
                .collect();
            let m = means.iter().sum::<f64>() / means.len() as f64;
            let sd = (means.iter().map(|x| (x - m).powi(2)).sum::<f64>()
                / (means.len() - 1) as f64)
                .sqrt();
            println!(
                "  untrained across 8 DIRECTIONS, features={set}: {m:.4} +- {sd:.4}  \
                 (range {:.1} .. {:.1})",
                means.iter().cloned().fold(f64::INFINITY, f64::min),
                means.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            );
            direction_sd.insert(set, (m, sd));
        }
    }

    for (label, set, (sigma, alpha)) in [
        ("A  v1 calibrated", "v1", calibrated),
        ("B  v1 mis-scaled", "v1", (2.0, 0.2)),
        ("C  v2 calibrated", "v2", calibrated),
    ] {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = tri_two().sim_config_causal(dir.path(), 9_600_000);
        let mut t = training();
        t.population_pairs = 16;
        t.seeds_per_member = 2;
        t.generations = 30;
        t.sigma = sigma;
        t.alpha = alpha;
        t.campaign_seed = 9_600_000;

        let cog = mlp_cognition(set);
        println!(
            "\n## {label}   sigma={sigma}, alpha={alpha}, features={set}  ({} params)",
            n_params_for(&cog, N_RULES)
        );

        let spec = CampaignSpec {
            campaign_id: format!("e2_{}", label.split_whitespace().next().unwrap()),
            base_dir: dir.path().to_path_buf(),
            sim_config: &cfg,
            cognition: cog,
            training: t,
        };
        let t0 = std::time::Instant::now();
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
                let a = &o.arm_table;
                println!(
                    "    arms: random {:.4}+-{:.4}  untrained {:.4}+-{:.4}  champion {:.4}+-{:.4}",
                    a.random.mean(),
                    a.random.sd(),
                    a.untrained.mean(),
                    a.untrained.sd(),
                    a.champion.mean(),
                    a.champion.sd()
                );
                match a.champion_vs_untrained() {
                    Some((d, s)) => println!(
                        "    PRIMARY champion - untrained = {d:+.4} at {s:.2}sigma  => exit {}",
                        if s > 2.0 { "MET" } else { "NOT met" }
                    ),
                    None => println!("    PRIMARY  no margin (an arm was never measured)"),
                }
                match a.champion_vs_random() {
                    Some((d, s)) => {
                        println!("    (reported) champion - random = {d:+.4} at {s:.2}sigma")
                    }
                    None => println!("    (reported) champion - random = no margin"),
                }
                // The class-level control. Dividing by seed sd answers "did
                // training beat THIS draw"; dividing by the direction spread
                // answers "did training beat an untrained network of this
                // class", which is the question the arm name implies.
                if let Some((dm, dsd)) = direction_sd.get(set) {
                    let c = a.champion.mean();
                    println!(
                        "    (class control) champion - untrained_MEAN = {:+.4} at {:.2} \
                         DIRECTION-sd   [untrained draw was {:.4}, class mean {:.4} +- {:.4}]",
                        c - dm,
                        (c - dm) / dsd.max(1e-9),
                        a.untrained.mean(),
                        dm,
                        dsd
                    );
                }
                println!(
                    "    {:<12} {:>10} {:>14} {:>10} {:>8}",
                    "arm", "fitness", "active_persist", "n_nodes", "flags"
                );
                for want in ["random", "untrained", "champion"] {
                    let rows: Vec<_> = o
                        .final_holdout_rows
                        .iter()
                        .filter(|r| r.arm == want)
                        .collect();
                    if rows.is_empty() {
                        continue;
                    }
                    let avg = |g: fn(&hg_train::row::CampaignRow) -> Option<f64>| {
                        let v: Vec<f64> = rows.iter().filter_map(|r| g(r)).collect();
                        if v.is_empty() {
                            f64::NAN
                        } else {
                            v.iter().sum::<f64>() / v.len() as f64
                        }
                    };
                    let flags = rows
                        .iter()
                        .filter(|r| {
                            r.flag_fitness_structure_divergence
                                || r.flag_degenerate_occupancy
                                || r.flag_degenerate_reuse
                        })
                        .count();
                    println!(
                        "    {:<12} {:>10.4} {:>14.2} {:>10.1} {:>8}",
                        want,
                        avg(|r| r.fitness),
                        avg(|r| r.mean_active_persistent),
                        avg(|r| r.mean_n_nodes),
                        flags
                    );
                }
                println!("    {:.1} min", t0.elapsed().as_secs_f64() / 60.0);
            }
            Err(e) => println!("    campaign refused: {e}"),
        }
    }
}
