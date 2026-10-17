//! Phase E5 — Stage 4′ (task lattice) and Stage 6 (phenotype contrast)
//! (`hypersim_nn_experiments_plan_v3.md` §9.3, §9.5, §10).
//!
//! > **Phase E5.** Stage 4′, then Stage 6 if Stage 8 delivered ≥2
//! > substrates. F6 campaign if Stage 5′ opened it and both cost checks
//! > pass.
//!
//! Stage 8 delivered three (E4), so Stage 6 runs. **Stage 5′ did not open
//! F6** — E3 §5 records a NO-GO because §7.1's first link is falsified — so
//! there is no F6 campaign and this is the last phase of plan v3.
//!
//! # Why both stages share one test
//!
//! Both need champion genomes, and a champion exists only as the output of
//! a campaign. E3 paid for the same four campaigns twice (once for the
//! measurement, once after a statistics correction) because the genomes
//! lived in a tempdir. Here every campaign runs once and both stages read
//! its `CampaignOutcome` in-process.

use hg_cognition::features::FeatureSet;
use hg_cognition::{CognitionConfig, MlpScorer, Parameterized, PolicyKind, RandomSubset, ScoreNorm};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::TaskSummary;
use hg_train::campaign::{n_params_for, CampaignSpec};
use hg_train::config::TrainingConfig;
use hg_train::diagnostics::{Fixture, RuleSource};
use hg_train::fitness::FitnessFn;
use hg_train::prechecks;
use std::collections::HashSet;

const STEPS: u64 = 384;
const WINDOW: u64 = 6;
const BURN_IN: usize = 5;
const BUDGET: usize = 4;
const HIDDEN: usize = 16;
const HOLDOUT: [u64; 5] = [9_600_001, 9_600_002, 9_600_003, 9_600_004, 9_600_005];

/// The four substrates E2/E4 established, so Stage 4′ and Stage 6 are read
/// on the same population every other E-phase result is read on.
fn cells() -> Vec<(&'static str, &'static str, &'static str, usize)> {
    vec![
        ("tri_two", "rules/tri_two.toml", "TriLattice", 2),
        ("relay_tri", "rules/relay_pair.toml", "TriLattice", 2),
        ("fan_tri", "rules/fan_merge.toml", "TriLattice", 2),
        ("fan_pure", "rules/fan_merge.toml", "PureCrystal", 2),
    ]
}

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

fn cognition() -> CognitionConfig {
    let mut c = CognitionConfig {
        policy: PolicyKind::MlpScorer,
        budget: Some(BUDGET),
        ..Default::default()
    };
    c.mlp.hidden = HIDDEN;
    c
}

fn training(which: FitnessFn) -> TrainingConfig {
    let mut t = TrainingConfig::default();
    t.fitness = which;
    t.horizon_steps = STEPS;
    t.burn_in_windows = BURN_IN;
    t.holdout_seeds = HOLDOUT.to_vec();
    t.population_pairs = 16;
    t.seeds_per_member = 2;
    t.generations = 30;
    t.sigma = 0.10;
    t.alpha = 0.032;
    t.campaign_seed = 9_600_000;
    t
}

/// Stage 4's composition rate, unchanged from C5 so the numbers are
/// comparable to the v2 table.
///
/// Over the distinct (input, output) task pairs: for every composable chain
/// A→B, B→C, does A→C also appear? Rate = realized / composable. Self-loops
/// are excluded — A→A is not evidence of composition — and with no chains
/// at all the rate is **undefined**, not zero.
fn composition_rate(tasks: &[TaskSummary]) -> (usize, usize, Option<f64>) {
    let set: HashSet<(u64, u64)> = tasks.iter().map(|t| (t.input_wl, t.output_wl)).collect();
    let (mut realized, mut composable) = (0usize, 0usize);
    for a in &set {
        for b in &set {
            if a.1 == b.0 && a.0 != b.1 {
                composable += 1;
                if set.contains(&(a.0, b.1)) {
                    realized += 1;
                }
            }
        }
    }
    ((composable > 0).then(|| realized as f64 / composable as f64))
        .map_or((realized, composable, None), |r| (realized, composable, Some(r)))
}

fn stat(v: &[f64]) -> (f64, f64) {
    if v.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    let m = v.iter().sum::<f64>() / v.len() as f64;
    let sd = if v.len() < 2 {
        0.0
    } else {
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
    };
    (m, sd)
}

/// Champion − reference in pooled sd. Returns `None` when either arm has no
/// spread to divide by, rather than an infinity dressed as significance.
fn contrast(a: &[f64], b: &[f64]) -> Option<f64> {
    let ((ma, sa), (mb, sb)) = (stat(a), stat(b));
    let pooled = ((sa * sa + sb * sb) / 2.0).sqrt();
    (pooled > 0.0 && ma.is_finite() && mb.is_finite()).then(|| (ma - mb) / pooled)
}

#[derive(Default, Clone)]
struct Lattice {
    depth: Vec<f64>,
    tasks: Vec<f64>,
    rate: Vec<f64>,
    composable: Vec<f64>,
    /// Seeds where the rate was **undefined** — no composable chain existed.
    /// Kept out of `rate` so an undefined never averages in as a zero.
    undefined: usize,
}

/// Score one policy across the frozen hold-out, collecting the lattice.
fn measure(
    cfg: &hg_sim_core::SimConfig,
    n_rules: usize,
    wl: usize,
    genome: Option<&Vec<f32>>,
) -> Lattice {
    let mut out = Lattice::default();
    for &s in &HOLDOUT {
        let policy: Box<dyn RuleSelectionPolicy> = match genome {
            None => Box::new(RandomSubset::new(Some(BUDGET), s)),
            Some(g) => {
                let mut p = MlpScorer::with_features_and_norm(
                    n_rules,
                    wl,
                    HIDDEN,
                    Some(BUDGET),
                    s,
                    FeatureSet::V1,
                    ScoreNorm::PoolStandard,
                );
                p.set_params(g);
                Box::new(p)
            }
        };
        let Ok(m) = hg_sim_core::run_rollout(hg_sim_core::RolloutRequest {
            config: cfg,
            seed: s,
            policy,
            horizon_steps: STEPS,
            probe: None,
        }) else {
            continue;
        };
        let (_, composable, rate) = composition_rate(&m.tasks);
        out.depth.push(m.task_lattice_depth as f64);
        out.tasks.push(m.tasks.len() as f64);
        out.composable.push(composable as f64);
        match rate {
            Some(r) => out.rate.push(r),
            None => out.undefined += 1,
        }
    }
    out
}

// =====================================================================
// Stage 4' — task lattice, genuinely three-arm
// =====================================================================

/// Plan §9.3.
///
/// > Unchanged in design from v2. Now genuinely three-arm with §5.1's
/// > neutral-point measurement, so the class effect and the learning effect
/// > are separated by construction rather than in re-analysis.
///
/// # What v2's Stage 4 found, and why the re-run is not a formality
///
/// On `wn_three` with `StaticBias`, composition rate more than doubled
/// under shaped dynamics (0.10 → 0.25) — and **champion vs untrained was
/// +0.28σ**. The entire effect was the policy *class*, not learning. That
/// finding is what made the three-arm protocol mandatory.
///
/// Stage 4′ asks the same question of a class whose neutral point is
/// different: `MlpScorer` at its `class_init` draw is **not** the uniform
/// allocation `StaticBias` at θ=0 is, and E2 §3 measured its neutral point
/// as a *draw* rather than a point. So `champion − untrained` here is a
/// genuinely different contrast from v2's, and its sign is not predictable
/// from the v2 result.
///
/// Stage 6 (below) reuses these same campaigns.
#[test]
#[ignore = "E5: Stage 4' + Stage 6; many hours, run explicitly"]
fn stage4_prime_and_stage6() {
    println!("\n=== E5 — Stage 4′ (task lattice) and Stage 6 (phenotype) ===");
    println!("v2 Stage 4 reference (wn_three, StaticBias): champion vs untrained +0.28sigma");
    println!("i.e. the lattice effect was ENTIRELY the policy class.\n");

    for (name, rules, ic, n_rules) in cells() {
        let f = fixture(name, rules, ic);
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = f.sim_config_causal(dir.path(), 9_600_000);
        let cog = cognition();
        let n = n_params_for(&cog, n_rules);
        let untrained =
            prechecks::class_init(PolicyKind::MlpScorer, n, 9_600_000u64 ^ 0x1_c001_1u64);

        println!("\n############ {name} ({ic}) ############");

        // --- the F5 campaign, shared by both stages
        let f5_champ = match hg_train::campaign::run_campaign(&CampaignSpec {
            campaign_id: format!("e5_f5_{name}"),
            base_dir: dir.path().to_path_buf(),
            sim_config: &cfg,
            cognition: cog.clone(),
            training: training(FitnessFn::F5ConstructorReuse),
        }) {
            Ok(o) => {
                println!(
                    "   F5 campaign: champion {:.4}  untrained {:.4}  random {:.4}",
                    o.arm_table.champion.mean(),
                    o.arm_table.untrained.mean(),
                    o.arm_table.random.mean()
                );
                Some(o.champion.params.clone())
            }
            Err(e) => {
                println!("   F5 campaign refused: {e}");
                None
            }
        };

        // ---------------- Stage 4′ ----------------
        println!("\n-- Stage 4′: task lattice, three arms --");
        let arms: Vec<(&str, Lattice)> = vec![
            ("random", measure(&cfg, n_rules, cog.wl_buckets, None)),
            (
                "untrained",
                measure(&cfg, n_rules, cog.wl_buckets, Some(&untrained)),
            ),
            (
                "champion",
                match &f5_champ {
                    Some(g) => measure(&cfg, n_rules, cog.wl_buckets, Some(g)),
                    None => Lattice::default(),
                },
            ),
        ];
        println!(
            "   {:<11} {:>16} {:>16} {:>18} {:>12}",
            "arm", "lattice depth", "tasks", "composition rate", "composable"
        );
        for (label, a) in &arms {
            if a.depth.is_empty() {
                println!("   {label:<11}  not measured");
                continue;
            }
            let (dm, ds) = stat(&a.depth);
            let (tm, ts) = stat(&a.tasks);
            let (rm, rs) = stat(&a.rate);
            let (cm, _) = stat(&a.composable);
            println!(
                "   {:<11} {:>8.2}+-{:<6.2} {:>8.1}+-{:<6.1} {:>9.4}+-{:<7.4} {:>12.1}{}",
                label,
                dm,
                ds,
                tm,
                ts,
                rm,
                rs,
                cm,
                if a.undefined > 0 {
                    format!("  [{} undefined]", a.undefined)
                } else {
                    String::new()
                }
            );
        }
        let get = |w: &str| arms.iter().find(|(l, _)| *l == w).map(|(_, a)| a);
        if let (Some(r), Some(u), Some(c)) =
            (get("random"), get("untrained"), get("champion"))
        {
            let line = |what: &str, f: fn(&Lattice) -> &Vec<f64>| {
                println!(
                    "   {:<18} champ-random {:>8}   untrained-random {:>8}   \
                     CHAMP-UNTRAINED {:>8}",
                    what,
                    contrast(f(c), f(r)).map_or("--".into(), |v| format!("{v:+.2}sd")),
                    contrast(f(u), f(r)).map_or("--".into(), |v| format!("{v:+.2}sd")),
                    contrast(f(c), f(u)).map_or("--".into(), |v| format!("{v:+.2}sd")),
                );
            };
            line("depth", |a| &a.depth);
            line("task count", |a| &a.tasks);
            line("composition rate", |a| &a.rate);
        }

        // ---------------- Stage 6 ----------------
        //
        // The phenotype contrast: the SAME configuration trained under a
        // different objective. F3 (constructor yield) is the contrast plan
        // v2 §Stage 6 names, and it is the one still available — F4 is a
        // drift detector and not a candidate objective (plan §7.3), and F6
        // is a NO-GO (E3 §5).
        println!("\n-- Stage 6: phenotype contrast, F5 champion vs F3 champion --");
        let f3_champ = match hg_train::campaign::run_campaign(&CampaignSpec {
            campaign_id: format!("e5_f3_{name}"),
            base_dir: dir.path().to_path_buf(),
            sim_config: &cfg,
            cognition: cog.clone(),
            training: training(FitnessFn::F3ConstructorYield),
        }) {
            Ok(o) => {
                println!(
                    "   F3 campaign: champion {:.4}  untrained {:.4}  random {:.4}",
                    o.arm_table.champion.mean(),
                    o.arm_table.untrained.mean(),
                    o.arm_table.random.mean()
                );
                Some(o.champion.params.clone())
            }
            Err(e) => {
                println!("   F3 campaign refused: {e}");
                None
            }
        };

        // Phenotype = the held-out structural description, never the
        // objective either champion was trained on. Scoring an F3 champion
        // by F3 and calling the difference a phenotype would be circular.
        if let (Some(a), Some(b)) = (&f5_champ, &f3_champ) {
            println!(
                "   {:<14} {:>12} {:>14} {:>12} {:>12} {:>10}",
                "champion of", "n_nodes", "active_persist", "mean_degree", "type_entropy", "lattice"
            );
            for (label, g) in [("F5", a), ("F3", b)] {
                let (mut nodes, mut ap, mut deg, mut ent, mut lat) =
                    (vec![], vec![], vec![], vec![], vec![]);
                for &s in &HOLDOUT {
                    let mut p = MlpScorer::with_features_and_norm(
                        n_rules,
                        cog.wl_buckets,
                        HIDDEN,
                        Some(BUDGET),
                        s,
                        FeatureSet::V1,
                        ScoreNorm::PoolStandard,
                    );
                    p.set_params(g);
                    let Ok(m) = hg_sim_core::run_rollout(hg_sim_core::RolloutRequest {
                        config: &cfg,
                        seed: s,
                        policy: Box::new(p),
                        horizon_steps: STEPS,
                        probe: None,
                    }) else {
                        continue;
                    };
                    let w = &m.windows[BURN_IN.min(m.windows.len())..];
                    let avg = |f: &dyn Fn(&hg_schema::metrics::MetricsRow) -> Option<f64>| {
                        let v: Vec<f64> = w.iter().filter_map(|r| f(r)).collect();
                        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
                    };
                    if let Some(v) = avg(&|r| Some(r.n_nodes as f64)) {
                        nodes.push(v);
                    }
                    if let Some(v) = avg(&|r| r.active_persistent_structures.map(|x| x as f64)) {
                        ap.push(v);
                    }
                    if let Some(v) = avg(&|r| r.mean_degree) {
                        deg.push(v);
                    }
                    if let Some(v) = avg(&|r| r.type_histogram_entropy) {
                        ent.push(v);
                    }
                    lat.push(m.task_lattice_depth as f64);
                }
                let p = |v: &Vec<f64>| {
                    let (m, s) = stat(v);
                    format!("{m:.2}+-{s:.2}")
                };
                println!(
                    "   {:<14} {:>12} {:>14} {:>12} {:>12} {:>10}",
                    label,
                    p(&nodes),
                    p(&ap),
                    p(&deg),
                    p(&ent),
                    p(&lat)
                );
            }
        } else {
            println!("   phenotype contrast unavailable — a campaign did not produce a champion");
        }
    }
}
