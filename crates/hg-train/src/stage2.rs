//! Stage 2 — evolved rule-selection bias (NN plan §11).
//!
//! **Hypothesis:** a per-rule preference vector can reliably steer the
//! substrate into target regimes.
//! **Exit:** champion beats budget-matched random by > 2σ on the five
//! frozen hold-out seeds, no Goodhart flags, on ≥ 1 non-fixture rule set.
//!
//! ## Why the baseline is run here rather than read off the campaign
//!
//! A campaign scores its champion on the hold-out seeds and writes that to
//! the report, but it has no budget-matched random arm — nothing in
//! `run_campaign` runs a policy it is not training. The comparison the exit
//! criterion is written against therefore has to be constructed: the same
//! five sim seeds, the same horizon, the same budget, `RandomSubset` in
//! place of the champion.
//!
//! The two arms are **paired on the policy RNG seed**. Both arms take their
//! policy seed from `derive_policy_seed(campaign_seed, g, member, r, salt)`
//! with the same arguments the campaign's own hold-out rollouts used, so
//! the arms differ in the policy and in nothing else. That matters more
//! than usual here: `crates/hg-train/tests/c5_variance_probe.rs` measured
//! that on constructed initial conditions the policy RNG is the *only*
//! source of hold-out variance, so an unpaired comparison would be
//! comparing two different draws of the dominant noise term.

use crate::campaign::{campaign_root, run_campaign, CampaignSpec, TrainError};
use crate::config::{Algo, TrainingConfig};
use crate::diagnostics::Fixture;
use crate::fitness::{self, FitnessFn};
use hg_cognition::{
    derive_policy_seed, CognitionConfig, Parameterized, PolicyKind, RandomSubset, StaticBias,
};
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};
use std::path::{Path, PathBuf};

/// One Stage 2 cell: a rule set, the budget Stage 0a's response curve
/// picked for it, and a horizon the C5 cost probe showed is affordable.
#[derive(Clone, Debug)]
pub struct Stage2Case {
    pub fixture: Fixture,
    /// From the Stage 0a budget response curve. `None` (k = ∞) is a
    /// deliberate error here, not a config: Stage 0a measured that at
    /// unbounded budget every arm is identical on every rule set, so a
    /// campaign there cannot produce a result whatever the optimizer does.
    pub budget: Option<usize>,
    pub generations: u32,
    pub seeds_per_member: usize,
    pub campaign_seed: u64,
    pub burn_in_windows: usize,
    /// CMA-ES initial step size, in **logit** units.
    ///
    /// `TrainingConfig`'s default is 0.02, and that default is wrong for
    /// this policy. `StaticBias`'s parameters are softmax logits: expressing
    /// "spend the budget on rule 1, not rule 0" needs the logits to separate
    /// by O(1)-O(10), and the genome starts at all-zeros. At sigma = 0.02
    /// a 40-generation campaign explores a neighbourhood of +/-0.1 or so,
    /// over which the softmax is nearly uniform and every population member
    /// scores the same — so there is no ranking signal, and CMA-ES stays
    /// where it started.
    ///
    /// The landscape sweep is what exposed this: on `tri_two` an untrained
    /// genome scores 0.9455 at k=16 while a reachable one-hot genome scores
    /// 1.0000, so there was something to find and the optimizer did not
    /// find it. Carried per-case, and into the campaign id, so a re-run at
    /// a different step size is a different campaign rather than a resume
    /// of the old one.
    pub sigma: f32,
    /// Why this rule set is in the sweep at all — carried through to the
    /// findings table so a GO-with-caveat never reads as a clean GO.
    pub stage0a_verdict: &'static str,
}

#[derive(Clone, Debug)]
pub struct ArmSamples {
    pub label: &'static str,
    pub per_seed: Vec<f64>,
}

impl ArmSamples {
    pub fn mean(&self) -> f64 {
        if self.per_seed.is_empty() {
            return f64::NAN;
        }
        self.per_seed.iter().sum::<f64>() / self.per_seed.len() as f64
    }
    pub fn sd(&self) -> f64 {
        if self.per_seed.len() < 2 {
            return 0.0;
        }
        let m = self.mean();
        (self.per_seed.iter().map(|x| (x - m).powi(2)).sum::<f64>()
            / (self.per_seed.len() - 1) as f64)
            .sqrt()
    }
}

#[derive(Clone, Debug)]
pub struct Stage2Result {
    pub name: String,
    pub budget: Option<usize>,
    pub horizon: u64,
    pub generations: u32,
    pub n_rules: usize,
    pub champion: ArmSamples,
    pub baseline: ArmSamples,
    /// **`StaticBias` with θ = 0 — the control that decides what the margin
    /// means.**
    ///
    /// `StaticBias` allocates budget across rule *groups* by softmax, so it
    /// is group-size independent by design; `RandomSubset` samples uniformly
    /// over *bindings*, so it allocates proportional to group size. On a
    /// composed rule set where one rule supplies most of the pool those are
    /// very different policies **before any training happens**. A champion
    /// beating budget-matched random therefore does not by itself show that
    /// the optimizer found anything — it may only show that the two policy
    /// classes differ.
    ///
    /// θ = 0 is the untrained member of the champion's own class. The
    /// margin worth reporting for "evolution did something" is
    /// champion − untrained; the margin the plan's exit criterion asks for
    /// is champion − random. Both are recorded because they answer
    /// different questions and the first sweep conflated them.
    pub untrained: ArmSamples,
    /// Champion mean − untrained-`StaticBias` mean.
    pub margin_vs_untrained: f64,
    /// Champion mean − baseline mean.
    pub margin: f64,
    /// Margin over the pooled sd of the two arms — the plan's criterion as
    /// literally written.
    pub sigma_pooled: f64,
    /// Margin over the sd of the paired per-seed differences. Stronger and
    /// the honest one to lead with, because the arms share their policy
    /// seeds: it removes the variance the two arms have in common instead
    /// of counting it against the margin.
    pub sigma_paired: f64,
    /// Per-seed sign agreement — 5/5 with a small margin says something
    /// a sigma on five samples does not.
    pub wins: usize,
    pub champion_genome: Vec<f32>,
    pub genome_hash: String,
    pub flag_fitness_structure_divergence: bool,
    pub flag_seed_overfit: bool,
    pub flag_degenerate_occupancy: bool,
    pub campaign_root: PathBuf,
    pub wallclock_s: f64,
}

impl Stage2Result {
    /// The plan's exit as written, plus the Goodhart condition it attaches.
    pub fn meets_exit(&self) -> bool {
        self.sigma_paired > 2.0
            && self.margin > 0.0
            && !self.flag_fitness_structure_divergence
            && !self.flag_seed_overfit
            && !self.flag_degenerate_occupancy
    }
}

fn sd_of(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

/// Relative spread floor, shared with the Stage 0a classifier: with N
/// identical samples the sample sd lands near 1e-16, not at zero.
fn spread_or_none(sd: f64, mean: f64) -> Option<f64> {
    (sd > 1e-9 * (mean.abs() + 1.0)).then_some(sd)
}

/// Score a fixed policy on the frozen hold-out seeds, paired to the
/// campaign's own hold-out rollouts on the policy RNG seed.
fn fixed_arm(
    label: &'static str,
    sim: &SimConfig,
    cog: &CognitionConfig,
    t: &TrainingConfig,
    n_members: u32,
    last_generation: u32,
    make: &dyn Fn(u64) -> Box<dyn RuleSelectionPolicy>,
) -> ArmSamples {
    let mut per_seed = Vec::with_capacity(t.holdout_seeds.len());
    for (r, &sim_seed) in t.holdout_seeds.iter().enumerate() {
        // Identical derivation to the campaign's own hold-out rollouts
        // (`campaign::run_campaign` uses member_idx = candidates.len()).
        let policy_seed = derive_policy_seed(
            t.campaign_seed,
            last_generation,
            n_members,
            r as u32,
            cog.rng_seed_salt,
        );
        let m = run_rollout(RolloutRequest {
            config: sim,
            seed: sim_seed,
            policy: make(policy_seed),
            horizon_steps: t.horizon_steps,
            probe: None,
        });
        if let Ok(m) = m {
            if let Ok(fit) = fitness::evaluate(&m, t.burn_in_windows) {
                // Unmeasured is not a score: a seed whose instrument did
                // not engage contributes no sample rather than a zero
                // (docs/instrument_registry.md §1).
                if let Ok(v) = fit.select(t.fitness) {
                    per_seed.push(v);
                }
            }
        }
    }
    ArmSamples { label, per_seed }
}

/// Build the `[cognition]` section for a Stage 2 cell.
pub fn cognition_for(case: &Stage2Case) -> CognitionConfig {
    CognitionConfig {
        policy: PolicyKind::StaticBias,
        budget: case.budget,
        ..Default::default()
    }
}

pub fn training_for(case: &Stage2Case) -> TrainingConfig {
    TrainingConfig {
        algo: Algo::CmaEs,
        generations: case.generations,
        seeds_per_member: case.seeds_per_member,
        fitness: FitnessFn::F2RegimeOccupancy,
        sigma: case.sigma,
        horizon_steps: case.fixture.steps,
        burn_in_windows: case.burn_in_windows,
        campaign_seed: case.campaign_seed,
        ..Default::default()
    }
}

pub fn run_case(case: &Stage2Case, base_dir: &Path) -> Result<Stage2Result, TrainError> {
    if case.budget.is_none() {
        return Err(TrainError::Config(format!(
            "Stage 2 cell {} has budget = None (k = inf). Stage 0a measured that at \
             unbounded budget every arm is identical on every rule set tested, so this \
             campaign cannot produce a result whatever the optimizer does \
             (docs/cognition_stage0_findings.md §5.1)",
            case.fixture.name
        )));
    }

    let started = std::time::Instant::now();
    let sim = case.fixture.sim_config(base_dir, case.campaign_seed);
    let cog = cognition_for(case);
    let training = training_for(case);
    // Sigma is in the id: a re-run at a different step size must be a new
    // campaign, not a resume of the old one's manifest.
    let campaign_id = format!(
        "stage2_{}_k{}_s{}",
        case.fixture.name,
        case.budget.unwrap(),
        format!("{:.2}", case.sigma).replace('.', "p")
    );

    let spec = CampaignSpec {
        campaign_id: campaign_id.clone(),
        base_dir: base_dir.to_path_buf(),
        sim_config: &sim,
        cognition: cog.clone(),
        training: training.clone(),
    };
    let outcome = run_campaign(&spec)?;

    let n_rules = hg_sim_core::load_rules(&sim)?.len();
    let n_members = training
        .population_lambda
        .unwrap_or_else(|| crate::optim::CmaEs::default_lambda(n_rules))
        .max(4) as u32;
    let last_gen = training.generations.saturating_sub(1);

    // All three arms go through ONE code path, differing only in the policy
    // handed to it.
    //
    // The champion arm first read the campaign's own final-generation
    // hold-out rows. That was wrong twice. It made the champion the only
    // arm not scored by this function, so "champion vs baseline" quietly
    // compared two different pieces of code; and a fully-resumed campaign
    // runs zero generations, leaving `final_holdout_rows` empty and the
    // champion arm silently NaN. Re-scoring the champion genome here is
    // both cheaper to trust and correct under resume.
    let champion_theta = outcome.champion.params.clone();
    let champion = fixed_arm(
        "champion",
        &sim,
        &cog,
        &training,
        n_members,
        last_gen,
        &|seed| {
            let mut p = StaticBias::new(n_rules, cog.budget, seed);
            p.set_params(&champion_theta);
            Box::new(p)
        },
    );
    let baseline = fixed_arm(
        "budget-matched random",
        &sim,
        &cog,
        &training,
        n_members,
        last_gen,
        &|seed| Box::new(RandomSubset::new(cog.budget, seed)),
    );
    let untrained = fixed_arm(
        "untrained StaticBias (theta = 0)",
        &sim,
        &cog,
        &training,
        n_members,
        last_gen,
        &|seed| {
            let mut p = StaticBias::new(n_rules, cog.budget, seed);
            p.set_params(&vec![0.0f32; n_rules]);
            Box::new(p)
        },
    );

    let margin = champion.mean() - baseline.mean();
    let pooled = {
        let (ca, ba) = (champion.sd(), baseline.sd());
        match (
            spread_or_none(ca, champion.mean()),
            spread_or_none(ba, baseline.mean()),
        ) {
            (Some(a), Some(b)) => Some(((a * a + b * b) / 2.0).sqrt()),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    };
    let diffs: Vec<f64> = champion
        .per_seed
        .iter()
        .zip(baseline.per_seed.iter())
        .map(|(c, b)| c - b)
        .collect();
    let diff_sd = sd_of(&diffs);
    let wins = diffs.iter().filter(|d| **d > 0.0).count();

    // Read off the rows rather than recomputed: `rollout_flags` already
    // evaluated §6.3 against each rollout's own `n_nodes_initial`, and
    // recomputing it here against the fixture's configured node count would
    // be a second, subtly different definition of the same tripwire.
    let occupancy_degenerate = outcome
        .final_holdout_rows
        .iter()
        .any(|r| r.flag_degenerate_occupancy);

    Ok(Stage2Result {
        name: case.fixture.name.to_string(),
        budget: case.budget,
        horizon: case.fixture.steps,
        generations: case.generations,
        n_rules,
        margin,
        sigma_pooled: pooled.map(|p| margin / p).unwrap_or(f64::NAN),
        sigma_paired: spread_or_none(diff_sd, margin)
            .map(|s| margin / s)
            .unwrap_or(f64::NAN),
        wins,
        margin_vs_untrained: champion.mean() - untrained.mean(),
        champion,
        baseline,
        untrained,
        champion_genome: outcome.champion.params.clone(),
        genome_hash: outcome.champion.genome_hash.clone(),
        flag_fitness_structure_divergence: outcome.flags.fitness_structure_divergence,
        flag_seed_overfit: outcome.flags.seed_overfit,
        flag_degenerate_occupancy: occupancy_degenerate,
        campaign_root: campaign_root(base_dir, &campaign_id),
        wallclock_s: started.elapsed().as_secs_f64(),
    })
}
