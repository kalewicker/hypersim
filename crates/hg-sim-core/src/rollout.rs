//! The rollout API (NN plan §3) — behind Cargo feature `cognition`.
//!
//! A **rollout** is a training inner-loop evaluation: same engine, same
//! step loop, same matcher, same conflict resolver, same evaluator — but no
//! CSV, no report, no manifest. It returns [`RolloutMetrics`] in memory.
//!
//! Two invariants, both tested rather than asserted:
//!
//! - **No forked physics.** `run_rollout` and `run_controlled` are two thin
//!   wrappers over one function (`runner::run_inner`); the only branches
//!   between them are around writing to disk. `tests/fl0_leak_detector.rs`
//!   checks the consequence directly: a `PassThrough` rollout must produce
//!   the same `canonical_hash` as the same config run with no `[cognition]`
//!   section at all.
//! - **Bit-determinism.** Same `SimConfig` + seed + policy parameters +
//!   policy RNG seed ⟹ identical `final_canonical_hash`. This extends
//!   `docs/reproducibility_policy.md` to the learning layer rather than
//!   exempting the learning layer from it.
//!
//! **Rollouts are never citable results.** They exist only inside
//! campaigns, leave no artifact, and no claim may reference one
//! (`docs/campaign_output.md` §1). A rollout that produced an interesting
//! number is re-run as a validation run before it is discussed.

use crate::config::SimConfig;
use crate::runner::{ConstructorSummary, ProbeFn, SimError, TaskSummary};
use hg_observe::RuleSelectionPolicy;
use hg_schema::MetricsRow;

pub struct RolloutRequest<'a> {
    pub config: &'a SimConfig,
    /// Simulation seed. Overrides `config.seed` — the campaign runner
    /// rotates seeds per generation without rebuilding the config.
    pub seed: u64,
    pub policy: Box<dyn RuleSelectionPolicy>,
    /// Overrides `config.engine.max_steps`.
    pub horizon_steps: u64,
    /// Optional per-window probe: receives the snapshot at each window
    /// boundary and returns a feature row (the reservoir readout's state
    /// vector). Rows land in `RolloutMetrics::probe_rows`, window-aligned
    /// with `windows` by construction.
    pub probe: Option<ProbeFn>,
}

pub struct RolloutMetrics {
    pub final_canonical_hash: u64,
    /// Per-window evaluator output — the same `MetricsRow` values a
    /// scientific run writes to `metrics.csv`, simply not serialized.
    pub windows: Vec<MetricsRow>,
    /// Window counts per `Regime`, indexed per
    /// `docs/fitness_formulas.md` §3.1.
    pub regime_occupancy: [u64; 11],
    pub constructors: Vec<ConstructorSummary>,
    /// The task lattice at run end, and its longest path. Empty / 0 when
    /// `[causal]` is off — Stage 4 must enable it or it is measuring the
    /// absence of a measurement.
    pub tasks: Vec<TaskSummary>,
    pub task_lattice_depth: usize,
    pub probe_rows: Vec<Vec<f32>>,
    /// Store size before step 0 — the `DEGENERATE_OCCUPANCY` denominator
    /// (`docs/fitness_formulas.md` §6.3).
    pub n_nodes_initial: usize,
    pub final_step: u64,
    pub wallclock_ms: u64,
}

/// Execute one rollout. Writes nothing to disk.
pub fn run_rollout(req: RolloutRequest<'_>) -> Result<RolloutMetrics, SimError> {
    let RolloutRequest {
        config,
        seed,
        policy,
        horizon_steps,
        probe,
    } = req;

    let mut cfg = config.clone();
    cfg.seed = seed;
    cfg.engine.max_steps = horizon_steps;
    // A rollout has no run directory, so a run_id would name nothing.
    // Cleared so that a stale explicit run_id in the campaign config cannot
    // make two different rollouts look like the same run in a log line.
    cfg.run_id.clear();

    let started = std::time::Instant::now();
    let (artifacts, probe_rows) = crate::runner::run_rollout_inner(&cfg, policy, probe)?;

    Ok(RolloutMetrics {
        final_canonical_hash: artifacts.canonical_hash,
        windows: artifacts.history,
        regime_occupancy: artifacts.regime_occupancy,
        constructors: artifacts.constructors,
        tasks: artifacts.tasks,
        task_lattice_depth: artifacts.task_lattice_depth,
        probe_rows,
        n_nodes_initial: artifacts.n_nodes_initial,
        final_step: artifacts.final_step,
        wallclock_ms: started.elapsed().as_millis() as u64,
    })
}

/// Build the policy named by a `[cognition]` section.
///
/// Returns `None` for `PolicyKind::None`, which is the same as the section
/// being absent — and in that case the caller must not call
/// `Engine::set_policy` at all, so the step loop's policy block is never
/// entered.
///
/// `n_rules` comes from the loaded rule set (`load_rules`), not from the
/// config: `docs/policy_config.md` §4 fixes the feature dimension at
/// `n_rules + wl_buckets + 7`, and a genome trained against a different
/// rule count has no meaning.
pub fn build_policy(
    cfg: &hg_cognition::CognitionConfig,
    n_rules: usize,
    policy_seed: u64,
) -> Result<Option<Box<dyn RuleSelectionPolicy>>, SimError> {
    use hg_cognition::{
        DegreeGreedy, MlpScorer, PassThrough, PolicyKind, RandomSubset, RuleBias, StaticBias,
    };

    cfg.validate(n_rules)
        .map_err(|e| SimError::Config(e.to_string()))?;

    let seed = policy_seed ^ cfg.rng_seed_salt;
    Ok(match cfg.policy {
        PolicyKind::None => None,
        PolicyKind::PassThrough => Some(Box::new(PassThrough)),
        PolicyKind::RandomSubset => Some(Box::new(RandomSubset::new(cfg.budget, seed))),
        PolicyKind::DegreeGreedy => Some(Box::new(DegreeGreedy::new(
            cfg.budget,
            cfg.degree_direction,
        ))),
        PolicyKind::RuleBias => Some(Box::new(RuleBias::new(
            cfg.rule_bias_weights.clone(),
            cfg.budget,
            seed,
        ))),
        PolicyKind::StaticBias => Some(Box::new(StaticBias::new(n_rules, cfg.budget, seed))),
        PolicyKind::MlpScorer => Some(Box::new(MlpScorer::new(
            n_rules,
            cfg.wl_buckets,
            cfg.mlp.hidden,
            cfg.budget,
            seed,
        ))),
    })
}
