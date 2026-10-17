//! Phase D6 — Stage 3′: `MlpScorer` under the objectives that cleared
//! Stage 2′ (`hypersim_nn_experiments_plan_v2.md` §9, §10).
//!
//! > Only if Stage 2′ produces a passing pair. Three-arm protocol
//! > mandatory. Comparisons: random, untrained `MlpScorer`, untrained
//! > `StaticBias`, champion — four arms here, since `StaticBias` is the
//! > class-effect reference the lattice and EI stages already established.
//! > **Exit:** champion − untrained `MlpScorer` > 2σ on frozen hold-out
//! > seeds.
//!
//! D5's passing pairs: **F5 × `tri_two`** (primary — no interpretive
//! caveat) and **F4 × `wn_three`** (secondary — carries the drift finding
//! of `docs/cognition_stage2_prime_findings.md` §5).

use hg_cognition::{
    CognitionConfig, MlpScorer, Parameterized, PolicyKind, RandomSubset, StaticBias,
};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};
use hg_train::campaign::{n_params_for, CampaignSpec};
use hg_train::config::TrainingConfig;
use hg_train::diagnostics::{Fixture, RuleSource};
use hg_train::fitness::{self, FitnessFn};
use hg_train::prechecks::{self, class_extreme, static_bias_with};

const STEPS: u64 = 384;
const WINDOW: u64 = 6;
const BURN_IN: usize = 5;
const BUDGET: usize = 4;
/// Frozen hold-out seeds. Disjoint from every training seed the campaign
/// draws, and fixed here so the exit statistic is computed on seeds chosen
/// before any champion existed.
const HOLDOUT: [u64; 5] = [9_600_001, 9_600_002, 9_600_003, 9_600_004, 9_600_005];

fn fixture(name: &'static str, rules: &'static str, ic: &'static str) -> Fixture {
    Fixture {
        name,
        rules: RuleSource::File(rules),
        ic,
        width: 8,
        node_count: 24,
        steps: STEPS,
        window: WINDOW,
        step_mode: "MaxParallel",
    }
}

fn tri_two() -> Fixture {
    fixture("tri_two", "rules/tri_two.toml", "TriLattice")
}
fn wn_three() -> Fixture {
    fixture("wn_three", "rules/wn_three.toml", "WhiteNoise")
}

/// The two D5-passing cells: `(fixture, n_rules, objective, causal)`.
fn cells() -> Vec<(Fixture, usize, FitnessFn, bool)> {
    vec![
        (tri_two(), 2, FitnessFn::F5ConstructorReuse, true),
        (wn_three(), 3, FitnessFn::F4MonotoneDrift, false),
    ]
}

fn mlp_cognition(hidden: usize) -> CognitionConfig {
    CognitionConfig {
        policy: PolicyKind::MlpScorer,
        budget: Some(BUDGET),
        ..Default::default()
    }
    .tap_hidden(hidden)
}

/// `CognitionConfig` has no builder, and `mlp.hidden` is the one field
/// Stage 3′ varies from default.
trait TapHidden {
    fn tap_hidden(self, hidden: usize) -> Self;
}
impl TapHidden for CognitionConfig {
    fn tap_hidden(mut self, hidden: usize) -> Self {
        self.mlp.hidden = hidden;
        self
    }
}

fn training(which: FitnessFn) -> TrainingConfig {
    let mut t = TrainingConfig::default();
    t.fitness = which;
    t.horizon_steps = STEPS;
    t.burn_in_windows = BURN_IN;
    t.holdout_seeds = HOLDOUT.to_vec();
    t
}

// =====================================================================
// Part 1 — what an untrained MlpScorer actually is, and what it costs
// =====================================================================

/// Two facts Stage 3′ cannot be designed without.
///
/// **1. A zero-initialized `MlpScorer` is budget-matched random in
/// distribution.** `forward` returns `b2 + Σ_h w2[h] · relu(b1[h] +
/// w1[h]·x)`; at θ=0 every term is zero, so every candidate scores 0.0.
/// Gumbel-top-k over tied scores is a uniform subset, and both policies
/// keep every creation candidate and subsample only `matches` — so the two
/// induce the same distribution over selections.
///
/// They are **not** trajectory-identical, and the canonical hashes below
/// say so: `RandomSubset` samples with `partial_shuffle` while `MlpScorer`
/// consumes Gumbel draws, so the same seed walks two different paths
/// through the same distribution. The claim to check is therefore
/// distributional — do the two arms' hold-out means agree within their
/// spread — not a hash comparison, which would fail for a reason that has
/// nothing to do with policy.
///
/// The consequence for the plan stands either way: the exit criterion
/// `champion − untrained > 2σ` coincides *in expectation* with
/// `champion − random > 2σ`, which plan §3 otherwise forbids as an exit
/// criterion. That is not a defect in the plan; it is a property of this
/// architecture at this initialization, and it needs saying out loud rather
/// than being discovered in the arm table.
///
/// It also means the *architecture* contributes nothing before training:
/// there is no inductive-bias effect for the untrained arm to control for.
/// So a **seeded** untrained arm is measured alongside — an untrained
/// network in the ordinary sense — which is the arm that can show a
/// class effect if one exists.
///
/// **2. Cost.** Stage 3′ is a campaign, not a precheck, and the population
/// must be sized against a measured per-rollout cost.
#[test]
#[ignore = "D6 part 1: characterizes the untrained arm and measures cost"]
fn untrained_mlp_is_random_and_what_a_rollout_costs() {
    println!("\n=== D6 part 1 — the untrained arm, and cost ===");

    for (f, n_rules, which, causal) in cells() {
        let cog = mlp_cognition(16);
        let n = n_params_for(&cog, n_rules);
        println!(
            "\n## {} x {}  (n_rules={n_rules}, feat_dim={}, hidden=16, n_params={n})",
            which.as_str(),
            f.name,
            n_rules + cog.wl_buckets + 7
        );

        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = if causal {
            f.sim_config_causal(dir.path(), 9_600_000)
        } else {
            f.sim_config(dir.path(), 9_600_000)
        };

        let zero = vec![0.0f32; n];
        let seeded = class_extreme(PolicyKind::MlpScorer, n, 4242);

        println!(
            "{:<22} {:>12} {:>12} {:>8}",
            "arm", "fitness", "canon hash", "secs"
        );
        let mut hashes = Vec::new();
        for (label, mk) in [
            (
                "random",
                Box::new(|s: u64| {
                    Box::new(RandomSubset::new(Some(BUDGET), s)) as Box<dyn RuleSelectionPolicy>
                }) as Box<dyn Fn(u64) -> Box<dyn RuleSelectionPolicy>>,
            ),
            (
                "mlp theta=0",
                Box::new({
                    let z = zero.clone();
                    move |s: u64| {
                        let mut p = MlpScorer::new(n_rules, 16, 16, Some(BUDGET), s);
                        p.set_params(&z);
                        Box::new(p) as Box<dyn RuleSelectionPolicy>
                    }
                }),
            ),
            (
                "mlp seeded",
                Box::new({
                    let g = seeded.clone();
                    move |s: u64| {
                        let mut p = MlpScorer::new(n_rules, 16, 16, Some(BUDGET), s);
                        p.set_params(&g);
                        Box::new(p) as Box<dyn RuleSelectionPolicy>
                    }
                }),
            ),
            (
                "staticbias theta=0",
                Box::new(move |s: u64| {
                    Box::new(StaticBias::new(n_rules, Some(BUDGET), s))
                        as Box<dyn RuleSelectionPolicy>
                }),
            ),
        ] {
            let t0 = std::time::Instant::now();
            let m = run_rollout(RolloutRequest {
                config: &cfg,
                seed: HOLDOUT[0],
                policy: mk(HOLDOUT[0]),
                horizon_steps: STEPS,
                probe: None,
            })
            .expect("rollout");
            let secs = t0.elapsed().as_secs_f64();
            let fit = fitness::evaluate(&m, BURN_IN).expect("evaluate");
            println!(
                "{:<22} {:>12} {:>12} {:>8.2}",
                label,
                match fit.select(which) {
                    Ok(v) => format!("{v:.4}"),
                    Err(_) => "unmeasured".into(),
                },
                m.final_canonical_hash % 100_000,
                secs
            );
            hashes.push((label, m.final_canonical_hash));
        }

        // The distributional claim, over the frozen hold-out rather than one
        // seed. If theta=0 really is random, the two arms differ by well
        // under their own seed spread; the seeded arm is the contrast.
        let t = training(which);
        let mk = |g: Option<Vec<f32>>| {
            move |s: u64| -> Box<dyn RuleSelectionPolicy> {
                match &g {
                    None => Box::new(RandomSubset::new(Some(BUDGET), s)),
                    Some(v) => {
                        let mut p = MlpScorer::new(n_rules, 16, 16, Some(BUDGET), s);
                        p.set_params(v);
                        Box::new(p)
                    }
                }
            }
        };
        let a_rnd = prechecks::probe_arm("random", &cfg, &t, &HOLDOUT, mk(None));
        let a_zero = prechecks::probe_arm("theta=0", &cfg, &t, &HOLDOUT, mk(Some(zero.clone())));
        let a_seed = prechecks::probe_arm("seeded", &cfg, &t, &HOLDOUT, mk(Some(seeded.clone())));
        let pooled = ((a_rnd.sd().powi(2) + a_zero.sd().powi(2)) / 2.0).sqrt();
        println!(
            "  over {} holdout seeds: random {:.4}+-{:.4}  theta=0 {:.4}+-{:.4}  seeded {:.4}+-{:.4}",
            HOLDOUT.len(),
            a_rnd.mean(),
            a_rnd.sd(),
            a_zero.mean(),
            a_zero.sd(),
            a_seed.mean(),
            a_seed.sd()
        );
        println!(
            "  |theta=0 - random| = {:.4} = {:.2} pooled sd   (equivalent if << 2)",
            (a_zero.mean() - a_rnd.mean()).abs(),
            if pooled > 0.0 {
                (a_zero.mean() - a_rnd.mean()).abs() / pooled
            } else {
                f64::NAN
            }
        );
        let _ = &hashes;
    }
}

// =====================================================================
// Part 2 — prechecks for the MLP class
// =====================================================================

/// P-1/P-3/P-4 for `MlpScorer`, before any campaign.
///
/// D5 ran these for `StaticBias`. They are not transferable: the class is
/// different, and P-3's far-end probe genome had to be made class-aware
/// before it measured anything at all (`prechecks::class_extreme`).
#[test]
#[ignore = "D6 part 2: MlpScorer prechecks; run explicitly"]
fn mlp_prechecks() {
    println!("\n=== D6 part 2 — P-1/P-3/P-4 for MlpScorer ===");

    for (f, n_rules, which, causal) in cells() {
        let cog = mlp_cognition(16);
        let n = n_params_for(&cog, n_rules);
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = if causal {
            f.sim_config_causal(dir.path(), 9_600_000)
        } else {
            f.sim_config(dir.path(), 9_600_000)
        };
        let t = training(which);

        println!("\n## {} x {}", which.as_str(), f.name);

        let p1 = prechecks::p1_class_expressiveness(&cog, &cfg, n_rules, 64, HOLDOUT[0]);
        match &p1 {
            Ok(r) => println!("    P-1  PASS  {}", r.detail),
            Err(e) => println!("    P-1  ABORT [{}] {}", e.code(), e.detail()),
        }

        let extreme = class_extreme(PolicyKind::MlpScorer, n, 4242);
        println!(
            "    probe genome live: {:?}",
            prechecks::probe_genome_is_live(PolicyKind::MlpScorer, n_rules, 16, 16, &extreme)
        );

        let mk_mlp = |g: Vec<f32>| {
            move |s: u64| {
                let mut p = MlpScorer::new(n_rules, 16, 16, Some(BUDGET), s);
                p.set_params(&g);
                Box::new(p) as Box<dyn RuleSelectionPolicy>
            }
        };
        let arms = vec![
            prechecks::probe_arm("random", &cfg, &t, &HOLDOUT, |s| {
                Box::new(RandomSubset::new(Some(BUDGET), s)) as Box<dyn RuleSelectionPolicy>
            }),
            prechecks::probe_arm("mlp theta=0", &cfg, &t, &HOLDOUT, mk_mlp(vec![0.0; n])),
            prechecks::probe_arm("mlp extreme", &cfg, &t, &HOLDOUT, mk_mlp(extreme)),
        ];
        for a in &arms {
            println!(
                "    {:<14} {:.4} +- {:.4} (n={}, unmeasured={})",
                a.label,
                a.mean(),
                a.sd(),
                a.scores.len(),
                a.unmeasured
            );
        }
        match prechecks::p3_objective_discrimination(&arms) {
            Ok(r) => println!("    P-3  PASS  {}", r.detail),
            Err(e) => println!("    P-3  ABORT [{}] {}", e.code(), e.detail()),
        }
        match prechecks::p4_signal_non_saturation(which, &arms[0]) {
            Ok(r) => println!("    P-4  PASS  {}", r.detail),
            Err(e) => println!("    P-4  ABORT [{}] {}", e.code(), e.detail()),
        }
    }
}

// =====================================================================
// Part 3 — the campaigns
// =====================================================================

/// Stage 3′ proper.
///
/// **Population is sized from part 1's measured cost, and the shortfall is
/// a limitation on any negative this produces.** The config default is 32
/// pairs × 3 seeds × 60 generations = 11 520 rollouts; at ~6.5 s per
/// `tri_two` rollout with `[causal]` on that is over a day even across
/// eight cores. What runs here is 16 pairs × 2 seeds × 30 generations =
/// 1 920 training rollouts — half the population, two-thirds the seeds,
/// half the generations. A null from an under-powered search is weaker
/// evidence than a null from a full one, which is exactly what C0–C5's
/// σ = 0.02 negative turned out to be, so the shortfall is stated with the
/// result rather than left for a reader to infer.
///
/// `sigma`/`alpha` are **not** the `StaticBias` values, and not guesses.
/// D5 found α = 0.01 freezes `StaticBias` in logit space; `MlpScorer`'s
/// parameters are network weights, not logits. `sigma = 2.0` is the scale
/// at which part 1 measured a draw to produce a strongly differentiated
/// policy (F5 524 vs 830 at θ=0; F4 1.57 vs 0.39). The first attempt used
/// `sigma = 0.5`, where one perturbation moves F5 by 41.43 against a seed
/// sd of 53.00 — below the noise the ES would have to climb through — and
/// both cells refused to start. α is left for P-2 to arbitrate.
#[test]
#[ignore = "D6 part 3: Stage 3' campaigns; long-running, run explicitly"]
fn stage3_prime_campaigns() {
    println!("\n=== D6 part 3 — Stage 3′ campaigns ===");

    for (f, n_rules, which, causal) in cells() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = if causal {
            f.sim_config_causal(dir.path(), 9_600_000)
        } else {
            f.sim_config(dir.path(), 9_600_000)
        };
        let mut t = training(which);
        t.population_pairs = 16;
        t.seeds_per_member = 2;
        t.generations = 30;
        // Calibrated, per the two constraints derived in
        // `docs/cognition_stage3_prime_findings.md` §5.3:
        //   sigma keeps the POPULATION in the behaviourally-live band —
        //     sigma*sqrt(n) ~ 0.3 * MLP_INIT_L2 = 2.1, so sigma ~ 2.1/sqrt(433) = 0.10
        //   alpha/sigma sets the STEP — |dtheta|/gen ~ 1.56*alpha/sigma, and
        //     targeting ~7% of the init norm per generation gives alpha/sigma ~ 0.32
        t.sigma = 0.10;
        t.alpha = 0.032;
        t.campaign_seed = 9_600_000;

        let cog = mlp_cognition(16);
        println!(
            "\n## {} x {}  ({} params, {} pairs x {} gens)",
            which.as_str(),
            f.name,
            n_params_for(&cog, n_rules),
            t.population_pairs,
            t.generations
        );

        let spec = CampaignSpec {
            campaign_id: format!("d6_{}_{}", which.as_str(), f.name),
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
                    None => println!("    PRIMARY champion - untrained = no margin (an arm was never measured)"),
                }
                match a.champion_vs_random() {
                    Some((d, s)) => println!("    (reported) champion - random = {d:+.4} at {s:.2}sigma"),
                    None => println!("    (reported) champion - random = no margin"),
                }

                // A margin is not a finding on its own. §6.4: flags are
                // findings, and a champion that wins by shutting the
                // substrate down is a Goodhart result, not a learning one —
                // which is the specific hazard D5 §5 predicted for F4.
                println!(
                    "    {:<12} {:>10} {:>12} {:>12} {:>10} {:>8}",
                    "arm", "fitness", "active_persist", "n_nodes", "f4 share", "flags"
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
                        "    {:<12} {:>10.4} {:>12.2} {:>12.1} {:>10.3} {:>8}",
                        want,
                        avg(|r| r.fitness),
                        avg(|r| r.mean_active_persistent),
                        avg(|r| r.mean_n_nodes),
                        avg(|r| r.f4_share_of_ceiling),
                        flags
                    );
                }
                println!("    {:.1} min", t0.elapsed().as_secs_f64() / 60.0);
            }
            Err(e) => println!("    campaign refused: {e}"),
        }
    }
}

// =====================================================================
// Part 4 — the fourth arm
// =====================================================================

/// The campaign's `ArmTable` carries three arms. Plan §9 asks for a fourth
/// on Stage 3′ — untrained `StaticBias`, "the class-effect reference the
/// lattice and EI stages already established" — which is a different policy
/// class and so cannot come out of an `MlpScorer` campaign's own rows. It is
/// scored here on the same frozen hold-out seeds.
#[test]
#[ignore = "D6 part 4: the untrained-StaticBias reference arm; run explicitly"]
fn untrained_static_bias_reference_arm() {
    println!("\n=== D6 part 4 — untrained StaticBias on the frozen hold-out ===");
    println!("{:<26} {:<22} {:>10} {:>9}", "cell", "arm", "mean", "sd");

    for (f, n_rules, which, causal) in cells() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg: SimConfig = if causal {
            f.sim_config_causal(dir.path(), 9_600_000)
        } else {
            f.sim_config(dir.path(), 9_600_000)
        };
        let t = training(which);
        let cell = format!("{} x {}", which.as_str(), f.name);
        for (label, arm) in [
            (
                "staticbias theta=0",
                prechecks::probe_arm("sb0", &cfg, &t, &HOLDOUT, |s| {
                    Box::new(StaticBias::new(n_rules, Some(BUDGET), s))
                        as Box<dyn RuleSelectionPolicy>
                }),
            ),
            (
                "staticbias extreme",
                prechecks::probe_arm("sbx", &cfg, &t, &HOLDOUT, |s| {
                    static_bias_with(
                        n_rules,
                        Some(BUDGET),
                        &prechecks::allocation_extreme(n_rules, 0),
                        s,
                    )
                }),
            ),
        ] {
            println!(
                "{:<26} {:<22} {:>10.4} {:>9.4}",
                cell,
                label,
                arm.mean(),
                arm.sd()
            );
        }
    }
}
