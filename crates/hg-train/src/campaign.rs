//! The campaign runner (`docs/campaign_output.md`).
//!
//! A campaign is the artifact-producing unit for training: exactly one
//! `campaign.csv`, one `campaign_report.md`, one `campaign_manifest.json`.
//! The rollouts inside it produce nothing and are never citable.
//!
//! Parallelism is over population members with **single-threaded
//! rollouts**. That is deliberate: a rollout's determinism contract is
//! per-rollout, and putting the engine's own thread pool underneath the
//! campaign's would make thread counts interact.
//!
//! The outer parallelism is **scoped OS threads, not rayon**, and that is
//! not a style preference. `run_inner` builds a `rayon::ThreadPool` and
//! `install()`s the whole step loop into it. Driving that from inside a
//! rayon worker nests one pool in another and, measured on the Stage 0a
//! sweep, serializes completely: 1500 rollouts sat at 99.4% of a single
//! core for an hour. Plain threads cannot interact with those inner pools,
//! so each rollout gets a real core.

use crate::config::{Algo, TrainingConfig};
use crate::fitness::{self, CampaignFlags, GenerationSummary, HeldOut};
use crate::optim::{CmaEs, CmaState, OpenAiEs, PairResult};
use crate::row::{Arm, CampaignRow, RolloutStatus, RowIdentity};
use hg_cognition::genome::{genome_hash, genome_hash_hex, ChampionRecord, CHAMPION_SCHEMA_VERSION};
use hg_cognition::{CognitionConfig, MlpScorer, Parameterized, PolicyKind, StaticBias};
use hg_observe::RuleSelectionPolicy;
use hg_output::CsvWriter;
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum TrainError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(String),
    #[error("sim error: {0}")]
    Sim(#[from] hg_sim_core::SimError),
    #[error("serialization error: {0}")]
    Serde(String),
    /// A stateability precheck refused to start the campaign
    /// (`hypersim_nn_experiments_plan_v2.md` §4). Boxed so the abort code
    /// and its remediation text survive to the caller intact rather than
    /// being flattened into a string at the throw site.
    #[error("precheck refused to start the campaign — {0}")]
    Precheck(Box<crate::prechecks::PrecheckAbort>),
}

/// Steps of probe rollout P-1 uses to measure rule co-liveness. Long enough
/// for a slow rule set to get going, short enough to be free against a
/// campaign measured in hours.
pub const P1_PROBE_STEPS: u64 = 64;

/// The one place a campaign's output root is decided.
///
/// There is no output-path key in `[training]` and this function is not
/// config-reachable, so `hg-train` physically cannot write into
/// `runs/substrate/` (`docs/campaign_output.md` §5.1).
pub fn campaign_root(base: &Path, campaign_id: &str) -> PathBuf {
    base.join("runs").join("cognition").join(campaign_id)
}

pub struct CampaignSpec<'a> {
    pub campaign_id: String,
    /// Base directory containing `runs/`. The campaign root is derived.
    pub base_dir: PathBuf,
    pub sim_config: &'a SimConfig,
    pub cognition: CognitionConfig,
    pub training: TrainingConfig,
}

pub struct CampaignOutcome {
    pub campaign_id: String,
    pub root: PathBuf,
    pub csv_path: PathBuf,
    pub report_path: PathBuf,
    pub manifest_path: PathBuf,
    pub generations: Vec<GenerationSummary>,
    pub champion: ChampionRecord,
    /// The final generation's hold-out rows, kept in memory so a caller can
    /// compare arms on exactly the rollouts the campaign scored rather than
    /// re-running them (Stage 2 does this) or re-parsing the CSV.
    pub final_holdout_rows: Vec<CampaignRow>,
    /// The mandatory three-arm comparison (`docs/campaign_output.md` §4.2).
    pub arm_table: ArmTable,
    /// Every precheck that ran, with its measured value. Recorded rather
    /// than merely passed: a marginal pass should be visible before it
    /// becomes a marginal result.
    pub prechecks: Vec<crate::prechecks::PrecheckResult>,
    pub flags: CampaignFlags,
    /// Counts by `status`, present even when everything succeeded.
    pub failure_census: Vec<(String, usize)>,
    pub total_rollouts: usize,
    pub wallclock_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignManifest {
    pub schema_version: u32,
    pub campaign_id: String,
    pub campaign_seed: u64,
    /// `None` before any generation completes.
    pub last_completed_generation: Option<u32>,
    pub theta: Vec<f32>,
    pub cma: Option<CmaState>,
    pub fitness_formulas: String,
    pub feature_set: String,
    pub n_rules: usize,
}

pub const CAMPAIGN_MANIFEST_VERSION: u32 = 1;

// ---------------------------------------------------------------------
// The Arm Table (docs/campaign_output.md §4.2)
// ---------------------------------------------------------------------

/// One arm's hold-out scores, kept per-seed.
///
/// Per-seed rather than mean-and-sd because a mean and an sd hide a bimodal
/// arm, and five numbers cost nothing to carry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArmScores {
    pub per_seed: Vec<f64>,
    /// Seeds whose selected fitness came back `unmeasured`. Never folded
    /// into `per_seed` as a zero (`docs/campaign_output.md` §4.3).
    pub unmeasured: usize,
}

impl ArmScores {
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
        (self
            .per_seed
            .iter()
            .map(|x| (x - m).powi(2))
            .sum::<f64>()
            / (self.per_seed.len() - 1) as f64)
            .sqrt()
    }
}

/// The mandatory three-arm comparison.
///
/// **`champion − untrained` is the primary reported statistic.**
/// `champion − random` is reported alongside it and is never the exit
/// criterion for a learning claim: on `wn_three` the champion beat
/// budget-matched random by `+0.7636` at `3.11σ` with no Goodhart flags —
/// v1's exit criterion, met — while `champion − untrained` was `+0.0000`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArmTable {
    pub random: ArmScores,
    pub untrained: ArmScores,
    pub champion: ArmScores,
}

impl ArmTable {
    pub fn from_rows(rows: &[CampaignRow]) -> Self {
        let mut t = Self::default();
        for r in rows.iter().filter(|r| r.is_holdout) {
            let slot = match r.arm.as_str() {
                "random" => &mut t.random,
                "untrained" => &mut t.untrained,
                "champion" => &mut t.champion,
                _ => continue,
            };
            match r.fitness {
                Some(v) => slot.per_seed.push(v),
                None if r.status == "unmeasured" => slot.unmeasured += 1,
                None => {}
            }
        }
        t
    }

    /// `(delta, sigma)` for a margin, where sigma is the pooled hold-out sd.
    /// `None` when either arm has no measurable seed — a margin against an
    /// arm that was never measured is not a small margin, it is no margin.
    fn margin(a: &ArmScores, b: &ArmScores) -> Option<(f64, f64)> {
        if a.per_seed.is_empty() || b.per_seed.is_empty() {
            return None;
        }
        let d = a.mean() - b.mean();
        let pooled = ((a.sd().powi(2) + b.sd().powi(2)) / 2.0).sqrt();
        Some((d, if pooled > 0.0 { d / pooled } else { f64::NAN }))
    }

    /// **The primary statistic.** The learning effect.
    pub fn champion_vs_untrained(&self) -> Option<(f64, f64)> {
        Self::margin(&self.champion, &self.untrained)
    }
    /// Reported alongside, never as the exit criterion.
    pub fn champion_vs_random(&self) -> Option<(f64, f64)> {
        Self::margin(&self.champion, &self.random)
    }
    /// The allocation-class effect. A real finding, and a finding about
    /// allocation rather than about learning.
    pub fn untrained_vs_random(&self) -> Option<(f64, f64)> {
        Self::margin(&self.untrained, &self.random)
    }

    /// A table missing an arm cannot support a learning claim. The report
    /// writer refuses to emit one rather than emitting it silently
    /// incomplete.
    pub fn is_complete(&self) -> bool {
        !self.random.per_seed.is_empty()
            && !self.untrained.per_seed.is_empty()
            && !self.champion.per_seed.is_empty()
    }
}

// ---------------------------------------------------------------------
// Policy construction
// ---------------------------------------------------------------------

/// Parameter count for the configured policy. Zero for the unlearned ones,
/// which is what makes "run a control arm through the same campaign
/// machinery" possible without special-casing it.
pub fn n_params_for(cog: &CognitionConfig, n_rules: usize) -> usize {
    match cog.policy {
        PolicyKind::StaticBias => n_rules,
        PolicyKind::MlpScorer => {
            MlpScorer::new(n_rules, cog.wl_buckets, cog.mlp.hidden, cog.budget, 0).n_params()
        }
        _ => 0,
    }
}

/// A constructed policy plus a cap-counter handle that outlives it.
type BuiltPolicy = (Box<dyn RuleSelectionPolicy>, Option<Arc<AtomicU64>>);

/// Build the policy for one rollout, loaded with `theta`.
///
/// Returns the policy and a cap-counter handle that outlives it: the policy
/// itself is moved into the engine and dropped when the rollout ends, so
/// `cap_bound_steps` has to be read through a shared handle or not at all.
fn make_policy(
    cog: &CognitionConfig,
    n_rules: usize,
    theta: &[f32],
    policy_seed: u64,
) -> Result<BuiltPolicy, TrainError> {
    match cog.policy {
        PolicyKind::StaticBias => {
            let mut p = StaticBias::new(n_rules, cog.budget, policy_seed);
            p.set_params(theta);
            Ok((Box::new(p), None))
        }
        PolicyKind::MlpScorer => {
            let mut p = MlpScorer::new(
                n_rules,
                cog.wl_buckets,
                cog.mlp.hidden,
                cog.budget,
                policy_seed,
            );
            p.set_params(theta);
            let counter = p.cap_counter();
            Ok((Box::new(p), Some(counter)))
        }
        _ => {
            let p = hg_sim_core::build_policy(cog, n_rules, policy_seed)
                .map_err(TrainError::Sim)?
                .ok_or_else(|| {
                    TrainError::Config(
                        "policy = \"none\" cannot be trained; pick a policy or omit the \
                         campaign"
                            .into(),
                    )
                })?;
            Ok((p, None))
        }
    }
}

// ---------------------------------------------------------------------
// One rollout
// ---------------------------------------------------------------------

struct RolloutJob {
    generation: u32,
    member_idx: u32,
    antithetic: i8,
    rollout_idx: u32,
    sim_seed: u64,
    is_holdout: bool,
    theta: Vec<f32>,
    /// `docs/campaign_output.md` §4.2. Training rollouts are `Champion`
    /// (they are candidates for it); the `Random` and `Untrained` arms are
    /// produced by the hold-out pass, which is the only place the three-arm
    /// comparison is made.
    arm: Arm,
}

/// The `[cognition]` section this arm runs under.
///
/// `Champion` and `Untrained` are the *same policy class* — they differ
/// only in `theta`, which is the point: the untrained arm isolates what the
/// allocation class does before any training. `Random` swaps the class for
/// budget-matched `RandomSubset`, at the identical budget, which is the
/// only thing about it that may differ (`docs/campaign_output.md` §4.2).
fn cognition_for_arm(base: &CognitionConfig, arm: Arm) -> CognitionConfig {
    match arm {
        Arm::Champion | Arm::Untrained => base.clone(),
        Arm::Random => CognitionConfig {
            policy: PolicyKind::RandomSubset,
            ..base.clone()
        },
    }
}

/// The allocation `theta` actually induces, measured over a short probe
/// rollout rather than derived from the parameters.
///
/// Falls back to uniform when the probe cannot run — a probe failure must
/// not be reported as "the allocation did not move", which would abort a
/// healthy campaign on an unrelated fault.
fn induced_allocation(
    spec: &CampaignSpec<'_>,
    n_rules: usize,
    theta: &[f32],
    seed: u64,
) -> Vec<f64> {
    let Ok((policy, _)) = make_policy(&spec.cognition, n_rules, theta, seed) else {
        return vec![1.0 / n_rules.max(1) as f64; n_rules.max(1)];
    };
    match crate::prechecks::trace_rules(spec.sim_config, seed, P1_PROBE_STEPS, n_rules, policy) {
        Ok(t) => t.allocation(),
        Err(_) => vec![1.0 / n_rules.max(1) as f64; n_rules.max(1)],
    }
}

fn evaluate_one(spec: &CampaignSpec<'_>, n_rules: usize, job: &RolloutJob) -> CampaignRow {
    let t = &spec.training;
    let cognition = cognition_for_arm(&spec.cognition, job.arm);
    let policy_seed = hg_cognition::derive_policy_seed(
        t.campaign_seed,
        job.generation,
        job.member_idx,
        job.rollout_idx,
        spec.cognition.rng_seed_salt,
    );
    let id = RowIdentity {
        generation: job.generation,
        member_idx: job.member_idx,
        antithetic: job.antithetic,
        sim_seed: job.sim_seed,
        rollout_idx: job.rollout_idx,
        policy_rng_seed: policy_seed,
        genome_hash: genome_hash(&job.theta),
        is_holdout: job.is_holdout,
        fitness_fn: t.fitness.as_str().to_string(),
        budget: spec.cognition.budget,
        arm: job.arm,
    };

    let started = std::time::Instant::now();
    let (policy, cap_counter) =
        match make_policy(&cognition, n_rules, &job.theta, policy_seed) {
            Ok(v) => v,
            Err(e) => {
                return CampaignRow::failed(
                    id,
                    RolloutStatus::SimError,
                    e.to_string(),
                    started.elapsed().as_millis() as u64,
                )
            }
        };

    let metrics = match run_rollout(RolloutRequest {
        config: spec.sim_config,
        seed: job.sim_seed,
        policy,
        horizon_steps: t.horizon_steps,
        probe: None,
    }) {
        Ok(m) => m,
        Err(e) => {
            return CampaignRow::failed(
                id,
                RolloutStatus::SimError,
                e.to_string(),
                started.elapsed().as_millis() as u64,
            )
        }
    };

    let fit = match fitness::evaluate(&metrics, t.burn_in_windows) {
        Ok(f) => f,
        Err(e) => {
            return CampaignRow::failed(
                id,
                RolloutStatus::NoScoredWindows,
                e.to_string(),
                metrics.wallclock_ms,
            )
        }
    };
    let ho = fitness::held_out(&metrics, t.burn_in_windows, &fit);
    let flags = fitness::rollout_flags(&metrics, &fit, &ho);

    // Rule 2 of docs/instrument_registry.md §1: a value outside its
    // mechanism-derived range is an instrument fault, not a finding. These
    // are the hard (arithmetic) bounds only, so a violation is never a
    // property of the substrate.
    for v in fitness::predicted_range_violations(&fit) {
        tracing::error!(target: "hg_train::instrument", rollout = %id.sim_seed, "PREDICTED_RANGE_VIOLATION: {v}");
    }

    // Windows where the engine's own `max_pairings_per_step` bound. The
    // engine does not expose a per-step counter, so this is measured at
    // window granularity from `accepted_per_step` — a window whose mean
    // accepted-per-step reaches the cap is a window where the cap bound.
    let cap = spec.sim_config.engine.max_pairings_per_step as f64;
    let pairings_bound = metrics
        .windows
        .iter()
        .filter(|w| w.accepted_per_step >= cap)
        .count() as u64;

    CampaignRow::ok(
        id,
        fit.select(t.fitness),
        &fit,
        &ho,
        flags,
        metrics.regime_occupancy,
        metrics.final_canonical_hash,
        metrics.n_nodes_initial,
        metrics.wallclock_ms,
        cap_counter.map(|c| c.load(Ordering::Relaxed)).unwrap_or(0),
        pairings_bound,
    )
}

/// Run `f` over `jobs` on scoped OS threads, preserving input order.
///
/// See the module header for why this is not `rayon`. Order is preserved
/// because the campaign CSV is a record: rows arriving in a
/// thread-scheduling-dependent order would make two runs of the same
/// campaign produce different files with identical content.
fn parallel_rollouts(
    jobs: &[RolloutJob],
    f: &(dyn Fn(&RolloutJob) -> CampaignRow + Sync),
) -> Vec<CampaignRow> {
    if jobs.is_empty() {
        return Vec::new();
    }
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(jobs.len());
    let chunk = jobs.len().div_ceil(n_threads);
    let mut out = Vec::with_capacity(jobs.len());
    std::thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .chunks(chunk)
            .map(|batch| scope.spawn(move || batch.iter().map(f).collect::<Vec<_>>()))
            .collect();
        for h in handles {
            out.extend(h.join().expect("rollout worker panicked"));
        }
    });
    out
}

/// Mean fitness over a member's seeds. `None` when every rollout failed —
/// which the optimizers treat as "no information from this member", not as a
/// bad score.
fn member_fitness(rows: &[&CampaignRow]) -> Option<f32> {
    let vals: Vec<f64> = rows.iter().filter_map(|r| r.fitness).collect();
    (!vals.is_empty()).then(|| (vals.iter().sum::<f64>() / vals.len() as f64) as f32)
}

// ---------------------------------------------------------------------
// The campaign
// ---------------------------------------------------------------------

pub fn run_campaign(spec: &CampaignSpec<'_>) -> Result<CampaignOutcome, TrainError> {
    let t = &spec.training;
    t.check_seed_disjointness().map_err(TrainError::Config)?;

    let rules = hg_sim_core::load_rules(spec.sim_config)?;
    let n_rules = rules.len();
    spec.cognition
        .validate(n_rules)
        .map_err(|e| TrainError::Config(e.to_string()))?;

    let n_params = n_params_for(&spec.cognition, n_rules);
    if n_params == 0 {
        return Err(TrainError::Config(format!(
            "policy {:?} has no parameters — there is nothing to train",
            spec.cognition.policy
        )));
    }
    // --- P-1: class expressiveness (plan v2 §4).
    //
    // `StaticBias` allocates budget by a masked softmax over the rule
    // logits, then draws uniformly inside the chosen rule's group. A
    // softmax over ONE logit is 1.0 for every value of that logit, so on a
    // single-rule set the genome cannot change the distribution at all and
    // the policy is *identical to budget-matched random*. Two rules that
    // never co-live fail for the same reason one step down: the softmax
    // renormalizes over whichever single rule is available.
    //
    // A campaign in either case spends hours fitting the policy RNG and
    // reports the result as a margin. Every builtin rule set ships exactly
    // one rule, so this is the common case rather than an edge case, and
    // `rules/wn_three.toml` / `rules/tri_two.toml` exist because of it.
    let mut prechecks = Vec::new();
    prechecks.push(
        crate::prechecks::p1_class_expressiveness(
            &spec.cognition,
            spec.sim_config,
            n_rules,
            P1_PROBE_STEPS,
            t.campaign_seed,
        )
        .map_err(|e| TrainError::Precheck(Box::new(e)))?,
    );

    // --- P-3 and P-4: can the objective see the class, and does it have
    // room to move? (plan v2 §4)
    //
    // Together these are the gate v1 lacked entirely. P-3 catches an
    // objective that is constant across the policy class — the shape bug 8
    // produced when `[causal]` was off and F3 came back identically 0.0 for
    // every arm, a table of zeros that reads as "no policy affects
    // constructor yield". P-4 catches the `tri_two` ceiling, where every
    // arm sat within 0.02 of F2's maximum and the margin was noise.
    //
    // Cost is three arms over the frozen hold-out seeds, against a campaign
    // of thousands of rollouts.
    {
        let n = n_params.max(1);
        let zero = vec![0.0f32; n];
        let extreme = crate::prechecks::allocation_extreme(n, 0);
        let mk = |theta: Vec<f32>, cog: CognitionConfig| {
            move |s: u64| {
                make_policy(&cog, n_rules, &theta, s)
                    .map(|(p, _)| p)
                    .unwrap_or_else(|_| Box::new(hg_observe::policy::SelectAllPolicy))
            }
        };
        let random_cfg = cognition_for_arm(&spec.cognition, Arm::Random);
        let arms = vec![
            crate::prechecks::probe_arm(
                "theta=0",
                spec.sim_config,
                t,
                &t.holdout_seeds,
                mk(zero, spec.cognition.clone()),
            ),
            crate::prechecks::probe_arm(
                "allocation extreme",
                spec.sim_config,
                t,
                &t.holdout_seeds,
                mk(extreme, spec.cognition.clone()),
            ),
            crate::prechecks::probe_arm(
                "budget-matched random",
                spec.sim_config,
                t,
                &t.holdout_seeds,
                mk(Vec::new(), random_cfg),
            ),
        ];
        prechecks.push(
            crate::prechecks::p3_objective_discrimination(&arms)
                .map_err(|e| TrainError::Precheck(Box::new(e)))?,
        );
        prechecks.push(
            crate::prechecks::p4_signal_non_saturation(t.fitness, &arms[2])
                .map_err(|e| TrainError::Precheck(Box::new(e)))?,
        );
    }

    let root = campaign_root(&spec.base_dir, &spec.campaign_id);
    std::fs::create_dir_all(root.join("champions"))?;
    let csv_path = root.join("campaign.csv");
    let report_path = root.join("campaign_report.md");
    let manifest_path = root.join("campaign_manifest.json");

    // Resume (§7). A prior manifest means a prior campaign.csv, and the
    // rows for completed generations are already in it — so the writer
    // appends rather than truncating, and "exactly one CSV" survives.
    let prior: Option<CampaignManifest> = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    if let Some(p) = prior.as_ref() {
        if p.n_rules != n_rules {
            return Err(TrainError::Config(format!(
                "resuming a campaign trained against {} rules with {n_rules} loaded — the \
                 feature layout differs, so the saved genome has no meaning here",
                p.n_rules
            )));
        }
    }
    let start_gen = prior
        .as_ref()
        .and_then(|p| p.last_completed_generation)
        .map(|g| g + 1)
        .unwrap_or(0);

    let created = chrono::Utc::now().to_rfc3339();
    let mut csv: CsvWriter<CampaignRow> = if start_gen > 0 && csv_path.exists() {
        CsvWriter::append(&csv_path)?
    } else {
        CsvWriter::create(
            &csv_path,
            crate::row::CAMPAIGN_CSV_SCHEMA_VERSION,
            &spec.campaign_id,
            t.campaign_seed,
            &created,
        )?
    };

    let mut theta = prior
        .as_ref()
        .filter(|p| p.theta.len() == n_params)
        .map(|p| p.theta.clone())
        .unwrap_or_else(|| vec![0.0f32; n_params]);

    let mut es = OpenAiEs::new(
        t.sigma,
        t.alpha,
        t.population_pairs,
        rand_chacha::ChaCha8Rng::seed_from_u64(t.campaign_seed ^ 0x1111),
    );
    let mut cma = CmaEs::new(
        theta.clone(),
        t.sigma,
        t.population_lambda,
        rand_chacha::ChaCha8Rng::seed_from_u64(t.campaign_seed ^ 0x2222),
    );
    if let Some(s) = prior.as_ref().and_then(|p| p.cma.as_ref()) {
        if s.mean.len() == n_params {
            cma.restore(s);
        }
    }

    let started = std::time::Instant::now();
    let mut history: Vec<GenerationSummary> = Vec::new();
    let mut census: std::collections::BTreeMap<String, usize> = Default::default();
    let mut total_rollouts = 0usize;
    let mut champion = champion_record(spec, n_rules, n_params, 0, &theta, f64::NEG_INFINITY);
    let mut final_holdout_rows: Vec<CampaignRow> = Vec::new();

    // P-2 needs the *induced* allocation at the initialization to compare
    // against, and "induced" means what the policy actually does on the
    // real candidate stream — not `softmax(theta)`, which would be right
    // for `StaticBias` and meaningless for `MlpScorer`.
    let theta_init = theta.clone();
    let alloc_init = induced_allocation(spec, n_rules, &theta_init, t.holdout_seeds[0]);
    let p2_gen = start_gen + crate::prechecks::traversal_generation(t.generations - start_gen);

    for g in start_gen..t.generations {
        let seeds = t.seed_set(g);

        // --- build this generation's candidates
        let (candidates, eps): (Vec<Vec<f32>>, Option<Vec<Vec<f32>>>) = match t.algo {
            Algo::OpenaiEs => {
                let eps = es.perturbations(n_params);
                let mut cands = Vec::with_capacity(eps.len() * 2);
                for e in &eps {
                    cands.push(es.candidate(&theta, e, 1.0));
                    cands.push(es.candidate(&theta, e, -1.0));
                }
                (cands, Some(eps))
            }
            Algo::CmaEs => (cma.ask(), None),
        };

        // --- one job per (member, seed)
        let jobs: Vec<RolloutJob> = candidates
            .iter()
            .enumerate()
            .flat_map(|(m, th)| {
                seeds.iter().enumerate().map(move |(r, &s)| RolloutJob {
                    generation: g,
                    member_idx: m as u32,
                    antithetic: match t.algo {
                        Algo::OpenaiEs => {
                            if m % 2 == 0 {
                                1
                            } else {
                                -1
                            }
                        }
                        Algo::CmaEs => 0,
                    },
                    rollout_idx: r as u32,
                    sim_seed: s,
                    is_holdout: false,
                    arm: Arm::Champion,
                    theta: th.clone(),
                })
            })
            .collect();

        let rows: Vec<CampaignRow> =
            parallel_rollouts(&jobs, &|j| evaluate_one(spec, n_rules, j));
        total_rollouts += rows.len();

        // --- per-member mean fitness
        let fits: Vec<Option<f32>> = (0..candidates.len())
            .map(|m| {
                let mine: Vec<&CampaignRow> =
                    rows.iter().filter(|r| r.member_idx == m as u32).collect();
                member_fitness(&mine)
            })
            .collect();

        // --- optimizer update
        match t.algo {
            Algo::OpenaiEs => {
                let eps = eps.expect("ES generation carries perturbations");
                let results: Vec<PairResult> = (0..eps.len())
                    .map(|p| PairResult {
                        plus: fits[2 * p],
                        minus: fits[2 * p + 1],
                    })
                    .collect();
                es.update(&mut theta, &eps, &results);
            }
            Algo::CmaEs => {
                cma.tell(&candidates, &fits);
                theta = cma.mean.clone();
            }
        }

        // --- champion evaluation on the FROZEN hold-out seeds
        let holdout_jobs: Vec<RolloutJob> = t
            .holdout_seeds
            .iter()
            .enumerate()
            .map(|(r, &s)| RolloutJob {
                generation: g,
                // A member index past the population so champion rows can
                // never be confused with population rows on `member_idx`
                // alone; `is_holdout` is the real discriminator.
                member_idx: candidates.len() as u32,
                antithetic: 0,
                rollout_idx: r as u32,
                sim_seed: s,
                is_holdout: true,
                theta: theta.clone(),
                arm: Arm::Champion,
            })
            .collect();
        let holdout_rows: Vec<CampaignRow> =
            parallel_rollouts(&holdout_jobs, &|j| evaluate_one(spec, n_rules, j));
        total_rollouts += holdout_rows.len();

        let hv: Vec<f64> = holdout_rows.iter().filter_map(|r| r.fitness).collect();
        let holdout_mean = mean(&hv);
        let holdout_sd = sd(&hv, holdout_mean);
        let rotation_best = fits.iter().filter_map(|f| *f).fold(f64::NEG_INFINITY, |a, b| a.max(b as f64));

        history.push(GenerationSummary {
            generation: g,
            best_fitness: if rotation_best.is_finite() { rotation_best } else { 0.0 },
            champion_holdout_mean: holdout_mean,
            champion_holdout_sd: holdout_sd,
            champion_rotation_mean: mean(&fits.iter().filter_map(|f| f.map(|v| v as f64)).collect::<Vec<_>>()),
            champion_held_out: held_out_of(&holdout_rows),
        });
        let flags = fitness::campaign_flags(&history);

        // --- write every row, population and hold-out, flags attached
        final_holdout_rows.clear();
        for mut r in rows.into_iter().chain(holdout_rows) {
            r.flag_fitness_structure_divergence = flags.fitness_structure_divergence;
            r.flag_seed_overfit = flags.seed_overfit;
            *census.entry(r.status.clone()).or_default() += 1;
            csv.write_row(&r)?;
            if r.is_holdout {
                final_holdout_rows.push(r);
            }
        }
        // Flushed per generation, so an unresumed crash still leaves a
        // readable, complete-through-generation-N CSV.
        csv.flush()?;

        champion = champion_record(spec, n_rules, n_params, g, &theta, holdout_mean);
        write_champion(&root, &champion)?;
        write_manifest(
            &manifest_path,
            &CampaignManifest {
                schema_version: CAMPAIGN_MANIFEST_VERSION,
                campaign_id: spec.campaign_id.clone(),
                campaign_seed: t.campaign_seed,
                last_completed_generation: Some(g),
                theta: theta.clone(),
                cma: matches!(t.algo, Algo::CmaEs).then(|| cma.snapshot()),
                fitness_formulas: fitness::FITNESS_FORMULAS_VERSION.to_string(),
                feature_set: spec.cognition.feature_set.clone(),
                n_rules,
            },
        )?;

        tracing::info!(
            generation = g,
            best = history.last().unwrap().best_fitness,
            holdout = holdout_mean,
            "generation complete"
        );

        // --- P-2: has the search actually started? (plan v2 §4)
        //
        // Checked once, at min(10, generations/4). The C0-C5 sigma=0.02
        // campaign ran forty generations with its allocation still uniform
        // to 4% and reported the result as a clean negative; this turns
        // that into an abort at generation ten.
        if g == p2_gen {
            let alloc_now = induced_allocation(spec, n_rules, &theta, t.holdout_seeds[0]);
            let r = crate::prechecks::p2_optimizer_traversal(
                &theta_init,
                &theta,
                t.sigma as f64,
                &alloc_init,
                &alloc_now,
                g,
            )
            .map_err(|e| TrainError::Precheck(Box::new(e)))?;
            prechecks.push(r);
        }
    }

    // --- the three-arm control pass (docs/campaign_output.md §4.2)
    //
    // Run once, at the end, on the same frozen hold-out seeds the champion
    // was scored on. Cost is two extra rollout sets for the whole campaign
    // and nothing at all during training — which is why the control that
    // would have caught the C0-C5 confound is now mandatory rather than
    // optional.
    let last_gen = t.generations.saturating_sub(1);
    let zero_theta = vec![0.0f32; n_params];
    let mut control_jobs = Vec::new();
    for (arm, theta_for_arm) in [
        (Arm::Untrained, &zero_theta),
        (Arm::Random, &zero_theta),
    ] {
        for (r, &s) in t.holdout_seeds.iter().enumerate() {
            control_jobs.push(RolloutJob {
                generation: last_gen,
                member_idx: u32::MAX,
                antithetic: 0,
                rollout_idx: r as u32,
                sim_seed: s,
                is_holdout: true,
                theta: theta_for_arm.clone(),
                arm,
            });
        }
    }
    let control_rows: Vec<CampaignRow> =
        parallel_rollouts(&control_jobs, &|j| evaluate_one(spec, n_rules, j));
    total_rollouts += control_rows.len();
    for r in &control_rows {
        *census.entry(r.status.clone()).or_default() += 1;
        csv.write_row(r)?;
    }
    csv.flush()?;
    final_holdout_rows.extend(control_rows);

    let arm_table = ArmTable::from_rows(&final_holdout_rows);

    let flags = fitness::campaign_flags(&history);
    let outcome = CampaignOutcome {
        campaign_id: spec.campaign_id.clone(),
        root: root.clone(),
        csv_path,
        report_path: report_path.clone(),
        manifest_path,
        generations: history,
        champion,
        final_holdout_rows,
        arm_table,
        prechecks,
        flags,
        failure_census: census.into_iter().collect(),
        total_rollouts,
        wallclock_ms: started.elapsed().as_millis() as u64,
    };
    crate::report::write_campaign_report(spec, &outcome, &report_path)?;
    Ok(outcome)
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

fn sd(v: &[f64], m: f64) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

fn held_out_of(rows: &[CampaignRow]) -> HeldOut {
    let pick = |f: fn(&CampaignRow) -> Option<f64>| -> Option<f64> {
        let v: Vec<f64> = rows.iter().filter_map(f).collect();
        (!v.is_empty()).then(|| mean(&v))
    };
    HeldOut {
        growth_exponent_nodes: pick(|r| r.growth_exponent_nodes),
        growth_exponent_edges: pick(|r| r.growth_exponent_edges),
        mean_type_entropy: pick(|r| r.mean_type_entropy),
        mean_type_jsd: pick(|r| r.mean_type_jsd),
        mean_mean_degree: pick(|r| r.mean_mean_degree),
        mean_active_persistent: pick(|r| r.mean_active_persistent),
        mean_static_persistent: pick(|r| r.mean_static_persistent),
        archive_admission_rate: pick(|r| r.archive_admission_rate),
        mean_n_nodes: pick(|r| r.mean_n_nodes).unwrap_or(0.0),
        replicator_rate: pick(|r| r.replicator_rate).unwrap_or(0.0),
        kernel_held: pick(|r| r.kernel_held),
        f5_top_share: pick(|r| r.f5_top_share).unwrap_or(0.0),
        f4_null_sd: pick(|r| r.f4_null_sd).unwrap_or(0.0),
    }
}

fn champion_record(
    spec: &CampaignSpec<'_>,
    n_rules: usize,
    _n_params: usize,
    generation: u32,
    theta: &[f32],
    holdout_fitness: f64,
) -> ChampionRecord {
    ChampionRecord {
        schema_version: CHAMPION_SCHEMA_VERSION,
        campaign_id: spec.campaign_id.clone(),
        generation,
        policy: spec.cognition.policy,
        feature_set: spec.cognition.feature_set.clone(),
        n_rules,
        wl_buckets: spec.cognition.wl_buckets,
        budget: spec.cognition.budget,
        hidden: spec.cognition.mlp.hidden,
        genome_hash: genome_hash_hex(theta),
        fitness_fn: spec.training.fitness.as_str().to_string(),
        holdout_fitness: if holdout_fitness.is_finite() { holdout_fitness } else { 0.0 },
        params: theta.to_vec(),
    }
}

fn write_champion(root: &Path, c: &ChampionRecord) -> Result<(), TrainError> {
    let path = root
        .join("champions")
        .join(format!("gen_{:04}.policy.toml", c.generation));
    let text = toml::to_string(c).map_err(|e| TrainError::Serde(e.to_string()))?;
    std::fs::write(path, text)?;
    Ok(())
}

fn write_manifest(path: &Path, m: &CampaignManifest) -> Result<(), TrainError> {
    let text = serde_json::to_string_pretty(m).map_err(|e| TrainError::Serde(e.to_string()))?;
    std::fs::write(path, text)?;
    Ok(())
}

/// Load a champion and check it means what it says against this rule set.
pub fn load_champion(
    path: &Path,
    n_rules: usize,
    wl_buckets: usize,
    n_params: usize,
) -> Result<ChampionRecord, TrainError> {
    let text = std::fs::read_to_string(path)?;
    let rec: ChampionRecord =
        toml::from_str(&text).map_err(|e| TrainError::Serde(e.to_string()))?;
    rec.check(n_rules, wl_buckets, n_params)
        .map_err(|e| TrainError::Config(e.to_string()))?;
    Ok(rec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_root_is_never_the_substrate_directory() {
        let r = campaign_root(Path::new("/tmp/x"), "abc_1");
        assert!(r.ends_with("runs/cognition/abc_1"));
        assert!(
            !r.to_string_lossy().contains("runs/substrate"),
            "hg-train must be structurally unable to write substrate output"
        );
    }

    #[test]
    fn n_params_matches_the_policy_layout() {
        let mut cog = CognitionConfig {
            policy: PolicyKind::StaticBias,
            ..Default::default()
        };
        assert_eq!(n_params_for(&cog, 6), 6);

        cog.policy = PolicyKind::MlpScorer;
        cog.wl_buckets = 16;
        cog.mlp.hidden = 8;
        // feat_dim = 6 + 16 + 7 = 29; w1 8*29 + b1 8 + w2 8 + b2 1
        assert_eq!(n_params_for(&cog, 6), 8 * 29 + 8 + 8 + 1);

        cog.policy = PolicyKind::RandomSubset;
        assert_eq!(n_params_for(&cog, 6), 0);
    }

    #[test]
    fn member_fitness_ignores_failed_rollouts() {
        let id = |m: u32| RowIdentity {
            generation: 0,
            member_idx: m,
            antithetic: 0,
            sim_seed: 1,
            rollout_idx: 0,
            policy_rng_seed: 0,
            genome_hash: 0,
            is_holdout: false,
            fitness_fn: "f2_regime_occupancy".into(),
            budget: None,
            arm: Arm::Champion,
        };
        let mut ok = CampaignRow::failed(id(0), RolloutStatus::Ok, String::new(), 0);
        ok.fitness = Some(0.4);
        let failed = CampaignRow::failed(id(0), RolloutStatus::SimError, "boom".into(), 0);
        assert_eq!(member_fitness(&[&ok, &failed]), Some(0.4));
        assert_eq!(member_fitness(&[&failed]), None);
    }
}
