//! Stateability prechecks P-1 … P-4 (`hypersim_nn_experiments_plan_v2.md`
//! §4).
//!
//! Run before any optimizer starts. All are seconds-to-minutes against
//! campaigns that are hours.
//!
//! v1 ran two campaigns where the hypothesis could not be stated at all:
//! `StaticBias` over a single rule (a softmax over one logit is 1.0 whatever
//! the logit is, so the policy is *provably* `RandomSubset`), and CMA-ES at
//! `sigma = 0.02` in softmax-logit space (forty generations, allocation
//! still uniform to 4%). Neither was false. Both were unstateable, and both
//! produced a clean-looking negative.
//!
//! The gate these checks implement is: *before spending compute, verify the
//! policy class can express a difference on this rule set, that the
//! optimizer can traverse the space, and that the objective can see either.*

use crate::config::TrainingConfig;
use crate::fitness::{self, FitnessFn};
use hg_cognition::{CognitionConfig, Parameterized, PolicyKind, StaticBias};
use hg_core::GraphSnapshot;
use hg_observe::{Binding, RuleSelectionPolicy};
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};

/// P-2's genome-movement bar, in units of the initial step size.
pub const TRAVERSAL_SIGMA_MULTIPLE: f64 = 3.0;
/// P-2's induced-allocation bar, as total variation.
pub const TRAVERSAL_TV_MIN: f64 = 0.05;
/// P-3's discrimination bar, in units of seed sd.
pub const DISCRIMINATION_SIGMA_MULTIPLE: f64 = 2.0;
/// P-4's saturation margin, as a fraction of the objective's achievable
/// range.
pub const SATURATION_MARGIN: f64 = 0.02;

/// Why a campaign must not start. Every variant is one of plan v2 §4's
/// named abort codes; the code is the stable identifier and the payload is
/// what a reader needs to fix it.
#[derive(Clone, Debug, PartialEq)]
pub enum PrecheckAbort {
    /// P-1. The policy class cannot express a difference on this rule set.
    ClassInexpressible(String),
    /// P-2. The optimizer's step size is mis-scaled for this
    /// parameterization; the search never left its initialization ball.
    OptimizerFrozen(String),
    /// P-3. The objective assigns the same value to every policy in the
    /// class, so the campaign is asking a question the objective cannot see.
    ObjectiveBlind(String),
    /// P-4. The objective is already at ceiling or floor for the random
    /// arm, so any margin is measuring noise.
    ObjectiveSaturated(String),
}

impl PrecheckAbort {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ClassInexpressible(_) => "CLASS_INEXPRESSIBLE",
            Self::OptimizerFrozen(_) => "OPTIMIZER_FROZEN",
            Self::ObjectiveBlind(_) => "OBJECTIVE_BLIND",
            Self::ObjectiveSaturated(_) => "OBJECTIVE_SATURATED",
        }
    }

    pub fn detail(&self) -> &str {
        match self {
            Self::ClassInexpressible(s)
            | Self::OptimizerFrozen(s)
            | Self::ObjectiveBlind(s)
            | Self::ObjectiveSaturated(s) => s,
        }
    }
}

impl std::fmt::Display for PrecheckAbort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.detail())
    }
}

impl std::error::Error for PrecheckAbort {}

/// One precheck's measured outcome. Recorded in the campaign report's
/// Precheck Record (`docs/campaign_output.md` §4 item 8) **with its measured
/// value**, not merely as "passed": a marginal pass should be visible before
/// it becomes a marginal result.
#[derive(Clone, Debug, PartialEq)]
pub struct PrecheckResult {
    pub name: &'static str,
    pub passed: bool,
    /// The number the bar was applied to.
    pub measured: f64,
    /// The bar.
    pub threshold: f64,
    pub detail: String,
}

// ---------------------------------------------------------------------
// The rule-trace probe — shared machinery for P-1 and P-2
// ---------------------------------------------------------------------

/// What a short probe rollout observed about rules and selections.
#[derive(Clone, Debug, Default)]
pub struct RuleTrace {
    /// Steps where at least two distinct rules offered a candidate. P-1's
    /// co-liveness measurement.
    pub steps_with_two_live_rules: usize,
    pub steps_observed: usize,
    /// Selections per rule id, summed over the probe. Normalized, this is
    /// the *induced allocation* — what the policy actually did, rather than
    /// what its parameters imply.
    pub selections_per_rule: Vec<u64>,
    /// Candidates offered per rule id, summed over the probe.
    pub candidates_per_rule: Vec<u64>,
}

impl RuleTrace {
    /// The induced allocation: selections normalized to a distribution.
    /// Uniform when nothing was selected, so a dead probe is not mistaken
    /// for a decisive one.
    pub fn allocation(&self) -> Vec<f64> {
        let total: u64 = self.selections_per_rule.iter().sum();
        let n = self.selections_per_rule.len().max(1);
        if total == 0 {
            return vec![1.0 / n as f64; n];
        }
        self.selections_per_rule
            .iter()
            .map(|c| *c as f64 / total as f64)
            .collect()
    }
}

/// Wraps a policy and records what it saw and what it chose.
///
/// Deliberately a wrapper rather than a reimplementation: P-2's "induced
/// allocation" must be the allocation the *real* policy produces on the
/// *real* candidate stream. Computing `softmax(theta)` instead would be
/// correct for `StaticBias` and meaningless for `MlpScorer`, and would
/// silently stop measuring the thing it names the moment the class changed.
struct TracingPolicy {
    inner: Box<dyn RuleSelectionPolicy>,
    n_rules: usize,
    trace: std::sync::Arc<std::sync::Mutex<RuleTrace>>,
}

impl RuleSelectionPolicy for TracingPolicy {
    fn select(&mut self, snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        let picked = self.inner.select(snapshot, candidates);
        let mut t = self.trace.lock().expect("trace mutex");
        if t.selections_per_rule.len() < self.n_rules {
            t.selections_per_rule = vec![0; self.n_rules];
            t.candidates_per_rule = vec![0; self.n_rules];
        }
        let mut live = vec![false; self.n_rules];
        for b in candidates {
            if b.rule_id < self.n_rules {
                t.candidates_per_rule[b.rule_id] += 1;
                live[b.rule_id] = true;
            }
        }
        for i in &picked {
            if let Some(b) = candidates.get(*i) {
                if b.rule_id < self.n_rules {
                    t.selections_per_rule[b.rule_id] += 1;
                }
            }
        }
        t.steps_observed += 1;
        if live.iter().filter(|b| **b).count() >= 2 {
            t.steps_with_two_live_rules += 1;
        }
        picked
    }
}

/// Run a short rollout with `policy`, returning what it saw.
pub fn trace_rules(
    cfg: &SimConfig,
    seed: u64,
    steps: u64,
    n_rules: usize,
    policy: Box<dyn RuleSelectionPolicy>,
) -> Result<RuleTrace, hg_sim_core::SimError> {
    let trace = std::sync::Arc::new(std::sync::Mutex::new(RuleTrace {
        selections_per_rule: vec![0; n_rules],
        candidates_per_rule: vec![0; n_rules],
        ..Default::default()
    }));
    let wrapper = TracingPolicy {
        inner: policy,
        n_rules,
        trace: std::sync::Arc::clone(&trace),
    };
    run_rollout(RolloutRequest {
        config: cfg,
        seed,
        policy: Box::new(wrapper),
        horizon_steps: steps,
        probe: None,
    })?;
    let t = trace.lock().expect("trace mutex").clone();
    Ok(t)
}

// ---------------------------------------------------------------------
// P-1 — class expressiveness
// ---------------------------------------------------------------------

/// Does the policy class admit two parameterizations producing different
/// selections on *this* rule set?
///
/// For `StaticBias` that needs `n_rules >= 2` **and** at least two rules
/// co-live — both offering candidates in the same step. Two rules that
/// never overlap in time give the softmax nothing to allocate between: it
/// renormalizes over whichever single rule is available and reproduces
/// budget-matched random exactly.
pub fn p1_class_expressiveness(
    cog: &CognitionConfig,
    cfg: &SimConfig,
    n_rules: usize,
    probe_steps: u64,
    seed: u64,
) -> Result<PrecheckResult, PrecheckAbort> {
    if !matches!(cog.policy, PolicyKind::StaticBias | PolicyKind::MlpScorer) {
        return Ok(PrecheckResult {
            name: "P-1 class expressiveness",
            passed: true,
            measured: f64::NAN,
            threshold: f64::NAN,
            detail: format!(
                "{:?} is not a learned class; expressiveness is not a precondition",
                cog.policy
            ),
        });
    }

    if cog.policy == PolicyKind::StaticBias && n_rules < 2 {
        return Err(PrecheckAbort::ClassInexpressible(format!(
            "StaticBias over {n_rules} rule(s): a softmax over one logit is 1.0 whatever \
             the logit is, so the policy is provably identical to budget-matched \
             RandomSubset. The hypothesis is unstateable, not false — compose a \
             multi-rule substrate (see rules/wn_three.toml) rather than tuning this one"
        )));
    }

    let probe = hg_sim_core::build_policy(cog, n_rules, seed)
        .map_err(|e| PrecheckAbort::ClassInexpressible(format!("probe policy: {e}")))?
        .ok_or_else(|| PrecheckAbort::ClassInexpressible("policy = none".into()))?;
    let trace = trace_rules(cfg, seed, probe_steps, n_rules, probe)
        .map_err(|e| PrecheckAbort::ClassInexpressible(format!("probe rollout: {e}")))?;

    let frac = if trace.steps_observed == 0 {
        0.0
    } else {
        trace.steps_with_two_live_rules as f64 / trace.steps_observed as f64
    };
    if trace.steps_with_two_live_rules == 0 {
        let live: Vec<usize> = trace
            .candidates_per_rule
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0)
            .map(|(i, _)| i)
            .collect();
        return Err(PrecheckAbort::ClassInexpressible(format!(
            "no step of {} offered candidates from two rules at once (rules ever live: \
             {live:?}). The class has nothing to allocate between — it renormalizes \
             over one rule and reproduces budget-matched random",
            trace.steps_observed
        )));
    }
    Ok(PrecheckResult {
        name: "P-1 class expressiveness",
        passed: true,
        measured: frac,
        threshold: 0.0,
        detail: format!(
            "{}/{} steps had >=2 live rules over {n_rules} rules",
            trace.steps_with_two_live_rules, trace.steps_observed
        ),
    })
}

// ---------------------------------------------------------------------
// P-2 — optimizer traversal
// ---------------------------------------------------------------------

/// Total variation between two distributions.
pub fn total_variation(a: &[f64], b: &[f64]) -> f64 {
    0.5 * a
        .iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .sum::<f64>()
}

/// The generation P-2 is evaluated at: `min(10, generations / 4)`.
pub fn traversal_generation(generations: u32) -> u32 {
    10.min(generations / 4).max(1)
}

/// Has the search done anything the policy can act on?
///
/// # Deviation from plan v2 §4's literal wording, and why
///
/// The plan states the test as a **conjunction**: abort unless the L2
/// distance from `theta_init` exceeds `3 * sigma_init` **and** the induced
/// allocation's total variation from its initial value exceeds 0.05.
/// Implemented literally, that rejects the very genome it exists to bless.
///
/// Measured, on the real `sigma = 2.0` `wn_three` champion: the allocation
/// moved from uniform to `[0.212, 0.777, 0.011]`, TV = 0.4437 — decisive —
/// while `|dtheta| = 5.39` sat under `3 * 2.0 = 6.0`. The conjunction aborts
/// a search that plainly worked.
///
/// The L2 clause is also dimension-inconsistent as stated. `|dtheta|` grows
/// like `sigma * sqrt(n_params)` under pure drift, so comparing it to a
/// scalar `3 * sigma` makes the bar trivially passable for a
/// several-hundred-parameter `MlpScorer` genome and unreachably strict for
/// a three-parameter `StaticBias` one. It measures dimension, not movement.
///
/// So **the allocation clause is the decisive one** and the L2 value is
/// retained as a diagnostic that names *which* failure occurred:
///
/// | `|dtheta|` | TV | reading |
/// |---|---|---|
/// | small | small | the step size is too small — bug 10 |
/// | large | small | movement along a direction the policy cannot see |
/// | any | large | the search is traversing; proceed |
///
/// Both failures are `OPTIMIZER_FROZEN`, because both mean the campaign is
/// about to spend hours on a search that has not started. The second is not
/// hypothetical: adding a constant to every logit moves `theta` arbitrarily
/// far and changes the softmax not at all.
pub fn p2_optimizer_traversal(
    theta_init: &[f32],
    theta_now: &[f32],
    sigma_init: f64,
    alloc_init: &[f64],
    alloc_now: &[f64],
    generation: u32,
) -> Result<PrecheckResult, PrecheckAbort> {
    let l2: f64 = theta_init
        .iter()
        .zip(theta_now)
        .map(|(a, b)| ((*b - *a) as f64).powi(2))
        .sum::<f64>()
        .sqrt();
    // The drift-scale reference: the expected norm of one sampling step in
    // this genome's own dimension. Dimensionless, so it means the same
    // thing for a 3-parameter and a 300-parameter policy.
    let drift = sigma_init * (theta_init.len().max(1) as f64).sqrt();
    let moved_in_units_of_drift = if drift > 0.0 { l2 / drift } else { 0.0 };
    let tv = total_variation(alloc_init, alloc_now);

    if tv <= TRAVERSAL_TV_MIN {
        let which = if moved_in_units_of_drift <= 1.0 {
            "the step size is too small for this parameterization: sigma is a step in \
             the policy's OWN parameter space, and for StaticBias that is softmax-logit \
             space, where the default 0.02 moves the allocation by under a percentage \
             point"
        } else {
            "the genome moved but along a direction the policy cannot act on — adding a \
             constant to every logit moves theta arbitrarily far and changes the softmax \
             not at all"
        };
        return Err(PrecheckAbort::OptimizerFrozen(format!(
            "at generation {generation}: induced-allocation TV = {tv:.4} against a bar of \
             {TRAVERSAL_TV_MIN}, with |dtheta| = {l2:.4} = {moved_in_units_of_drift:.2}x \
             the drift scale (sigma_init * sqrt(n) = {drift:.4}). {which}"
        )));
    }
    Ok(PrecheckResult {
        name: "P-2 optimizer traversal",
        passed: true,
        measured: tv,
        threshold: TRAVERSAL_TV_MIN,
        detail: format!(
            "allocation TV = {tv:.4}; |dtheta| = {l2:.4} = {moved_in_units_of_drift:.2}x \
             the drift scale"
        ),
    })
}

// ---------------------------------------------------------------------
// P-3 / P-4 — objective discrimination and saturation
// ---------------------------------------------------------------------

/// One probe arm's scores across seeds.
#[derive(Clone, Debug)]
pub struct ProbeArm {
    pub label: String,
    pub scores: Vec<f64>,
    /// Seeds whose selected fitness came back `unmeasured`. Kept separate
    /// from `scores` so a suppressed value never averages in as a zero.
    pub unmeasured: usize,
}

impl ProbeArm {
    pub fn mean(&self) -> f64 {
        if self.scores.is_empty() {
            return f64::NAN;
        }
        self.scores.iter().sum::<f64>() / self.scores.len() as f64
    }
    pub fn sd(&self) -> f64 {
        if self.scores.len() < 2 {
            return 0.0;
        }
        let m = self.mean();
        (self
            .scores
            .iter()
            .map(|x| (x - m).powi(2))
            .sum::<f64>()
            / (self.scores.len() - 1) as f64)
            .sqrt()
    }
}

/// Score one policy across `seeds`, honouring the unmeasured/zero
/// distinction.
pub fn probe_arm(
    label: impl Into<String>,
    cfg: &SimConfig,
    t: &TrainingConfig,
    seeds: &[u64],
    mut make: impl FnMut(u64) -> Box<dyn RuleSelectionPolicy>,
) -> ProbeArm {
    let mut scores = Vec::new();
    let mut unmeasured = 0usize;
    for &s in seeds {
        let Ok(m) = run_rollout(RolloutRequest {
            config: cfg,
            seed: s,
            policy: make(s),
            horizon_steps: t.horizon_steps,
            probe: None,
        }) else {
            continue;
        };
        let Ok(fit) = fitness::evaluate(&m, t.burn_in_windows) else {
            continue;
        };
        match fit.select(t.fitness) {
            Ok(v) => scores.push(v),
            Err(_) => unmeasured += 1,
        }
    }
    ProbeArm {
        label: label.into(),
        scores,
        unmeasured,
    }
}

/// Does the objective assign different values to at least two policies in
/// the class?
///
/// Evaluated at theta=0, at a hand-set extreme (all mass on one rule), and
/// at budget-matched random. If the spread across those three is under
/// `DISCRIMINATION_SIGMA_MULTIPLE` times seed sd, the objective cannot see
/// the class and the campaign is unstateable.
///
/// The seed sd used is the **largest** of the three arms', which is the
/// conservative choice: it is harder to pass, and passing on the noisiest
/// arm's noise is the claim being made.
pub fn p3_objective_discrimination(arms: &[ProbeArm]) -> Result<PrecheckResult, PrecheckAbort> {
    let usable: Vec<&ProbeArm> = arms.iter().filter(|a| !a.scores.is_empty()).collect();
    if usable.len() < 2 {
        return Err(PrecheckAbort::ObjectiveBlind(format!(
            "only {} of {} probe arms produced a measurable score ({} arms came back \
             entirely unmeasured). The objective's instrument did not engage — see \
             docs/instrument_registry.md §2 before reading this as a null",
            usable.len(),
            arms.len(),
            arms.iter().filter(|a| a.scores.is_empty()).count()
        )));
    }
    let means: Vec<f64> = usable.iter().map(|a| a.mean()).collect();
    let spread = means.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - means.iter().cloned().fold(f64::INFINITY, f64::min);
    let sd = usable.iter().map(|a| a.sd()).fold(0.0f64, f64::max);
    let bar = DISCRIMINATION_SIGMA_MULTIPLE * sd;

    if spread <= bar {
        let table: Vec<String> = usable
            .iter()
            .map(|a| format!("{} = {:.4} +- {:.4}", a.label, a.mean(), a.sd()))
            .collect();
        return Err(PrecheckAbort::ObjectiveBlind(format!(
            "spread across the class is {spread:.4}, under \
             {DISCRIMINATION_SIGMA_MULTIPLE}x the largest seed sd ({sd:.4} -> bar \
             {bar:.4}). The objective cannot distinguish the policies it would be \
             asked to rank. Arms: [{}]",
            table.join("; ")
        )));
    }
    Ok(PrecheckResult {
        name: "P-3 objective discrimination",
        passed: true,
        measured: spread,
        threshold: bar,
        detail: format!("spread {spread:.4} over max seed sd {sd:.4}"),
    })
}

/// Is the objective at ceiling or floor for the random arm?
///
/// `tri_two` at 384 steps is the standing example: every arm sat within 0.02
/// of the F2 ceiling because the budget had been chosen for headroom at a
/// 192-step horizon. Any margin measured there is noise, and the campaign
/// was uninformative rather than negative.
pub fn p4_signal_non_saturation(
    which: FitnessFn,
    random: &ProbeArm,
) -> Result<PrecheckResult, PrecheckAbort> {
    let Some((lo, hi)) = achievable_range(which) else {
        return Ok(PrecheckResult {
            name: "P-4 signal non-saturation",
            passed: true,
            measured: random.mean(),
            threshold: f64::NAN,
            detail: format!(
                "{} is unbounded above, so it has no achievable range to saturate; \
                 comparisons must instead be at matched horizon_steps",
                which.as_str()
            ),
        });
    };
    if random.scores.is_empty() {
        return Err(PrecheckAbort::ObjectiveSaturated(format!(
            "the random arm produced no measurable score for {} ({} seeds unmeasured); \
             saturation cannot be assessed and the campaign must not proceed on the \
             assumption that it is fine",
            which.as_str(),
            random.unmeasured
        )));
    }
    let m = random.mean();
    let margin = SATURATION_MARGIN * (hi - lo);
    let headroom = (hi - m).min(m - lo);
    if headroom < margin {
        return Err(PrecheckAbort::ObjectiveSaturated(format!(
            "the random arm scores {m:.4} on {}, within {headroom:.4} of the achievable \
             range [{lo}, {hi}] whose {}% margin is {margin:.4}. There is no headroom, \
             so any champion margin here is measuring noise. Re-choose the horizon or \
             the budget and report the achievable range alongside",
            which.as_str(),
            SATURATION_MARGIN * 100.0
        )));
    }
    Ok(PrecheckResult {
        name: "P-4 signal non-saturation",
        passed: true,
        measured: headroom,
        threshold: margin,
        detail: format!("random at {m:.4}, headroom {headroom:.4} in [{lo}, {hi}]"),
    })
}

/// The objective's own bound, where it has one.
///
/// `None` for the unbounded functions, and that is reported as such rather
/// than substituted with an observed range: a range fitted to observed
/// values would ratify whatever the instrument currently does, including its
/// bugs (`docs/instrument_registry.md` §1.1).
pub fn achievable_range(which: FitnessFn) -> Option<(f64, f64)> {
    match which {
        FitnessFn::F1Interestingness => Some((0.0, 1.0)),
        FitnessFn::F2RegimeOccupancy => Some((-fitness::LAMBDA_PENALTY, 1.0)),
        FitnessFn::F4PredictiveInformation => {
            Some((0.0, (fitness::PI_ALPHABET as f64).log2()))
        }
        // F3 and F5 both grow with the run: `n` constructors and `T` cycles
        // are unbounded above. Comparisons are only valid at matched
        // horizons, which is a different discipline from a range check.
        FitnessFn::F3ConstructorYield | FitnessFn::F5ConstructorReuse => None,
    }
}

/// The extreme genome for P-3: all softmax mass on rule `k`.
///
/// 8.0 rather than something larger because `exp(8) / (exp(8) + (n-1))`
/// already exceeds 0.99 for any rule count these substrates carry, and
/// larger logits only buy floating-point noise.
pub fn allocation_extreme(n_rules: usize, k: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; n_rules];
    if let Some(slot) = v.get_mut(k) {
        *slot = 8.0;
    }
    v
}

/// `StaticBias` loaded with `theta`, for probe arms.
pub fn static_bias_with(n_rules: usize, budget: Option<usize>, theta: &[f32], seed: u64) -> Box<dyn RuleSelectionPolicy> {
    let mut p = StaticBias::new(n_rules, budget, seed);
    p.set_params(theta);
    Box::new(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal live config, only needed by the P-1 tests that abort
    /// before any rollout starts.
    fn tiny_cfg() -> SimConfig {
        let dir = std::env::temp_dir();
        crate::diagnostics::Fixture {
            name: "precheck_probe",
            rules: crate::diagnostics::RuleSource::Builtin("binary_edge"),
            ic: "PureCrystal",
            width: 4,
            node_count: 16,
            steps: 8,
            window: 4,
            step_mode: "MaxParallel",
        }
        .sim_config(&dir, 7)
    }

    fn arm(label: &str, scores: &[f64]) -> ProbeArm {
        ProbeArm {
            label: label.into(),
            scores: scores.to_vec(),
            unmeasured: 0,
        }
    }

    #[test]
    fn p1_rejects_a_single_rule_static_bias_before_any_rollout() {
        // The C0-C5 blocker: every builtin rule set ships one rule.
        let cog = CognitionConfig {
            policy: PolicyKind::StaticBias,
            ..Default::default()
        };
        let cfg = tiny_cfg();
        let e = p1_class_expressiveness(&cog, &cfg, 1, 8, 7).unwrap_err();
        assert_eq!(e.code(), "CLASS_INEXPRESSIBLE");
        assert!(e.detail().contains("softmax over one logit"));
    }

    #[test]
    fn p1_does_not_gate_unlearned_policies() {
        let cog = CognitionConfig {
            policy: PolicyKind::RandomSubset,
            ..Default::default()
        };
        let r = p1_class_expressiveness(&cog, &tiny_cfg(), 1, 8, 7).unwrap();
        assert!(r.passed);
    }

    #[test]
    fn p2_catches_the_sigma_002_freeze() {
        // Bug 10, reproduced: forty generations at sigma = 0.02 in logit
        // space left the allocation uniform to 4%.
        let init = vec![0.0f32; 3];
        let frozen = vec![0.01f32, -0.008, 0.004];
        let uniform = vec![1.0 / 3.0; 3];
        let barely = vec![0.34, 0.33, 0.33];
        let e = p2_optimizer_traversal(&init, &frozen, 0.02, &uniform, &barely, 10).unwrap_err();
        assert_eq!(e.code(), "OPTIMIZER_FROZEN");
        assert!(e.detail().contains("softmax-logit space"), "{}", e.detail());
    }

    #[test]
    fn p2_passes_a_search_that_actually_moved() {
        let init = vec![0.0f32; 3];
        // The sigma = 2.0 wn_three champion's allocation, measured.
        let moved = vec![2.0f32, 4.0, -3.0];
        let uniform = vec![1.0 / 3.0; 3];
        let now = vec![0.212, 0.777, 0.011];
        let r = p2_optimizer_traversal(&init, &moved, 2.0, &uniform, &now, 10).unwrap();
        assert!(r.passed);
        assert!(r.measured > TRAVERSAL_TV_MIN);
    }

    #[test]
    fn p2_rejects_movement_the_softmax_cannot_see() {
        // Adding a constant to every logit moves theta arbitrarily far and
        // changes the allocation not at all. The L2 clause alone passes it.
        let init = vec![0.0f32; 3];
        let shifted = vec![50.0f32; 3];
        let uniform = vec![1.0 / 3.0; 3];
        let e = p2_optimizer_traversal(&init, &shifted, 0.5, &uniform, &uniform, 10).unwrap_err();
        assert_eq!(e.code(), "OPTIMIZER_FROZEN");
        assert!(
            e.detail().contains("cannot act on"),
            "the two frozen modes must be distinguishable in the message: {}",
            e.detail()
        );
    }

    #[test]
    fn p3_fires_when_the_objective_cannot_see_the_class() {
        // The wn_three shape: champion and untrained identical.
        let e = p3_objective_discrimination(&[
            arm("theta=0", &[1.0, 1.0, 1.0, 1.0, 1.0]),
            arm("extreme", &[1.0, 1.0, 1.0, 1.0, 1.0]),
            arm("random", &[0.999, 1.0, 1.0, 1.001, 1.0]),
        ])
        .unwrap_err();
        assert_eq!(e.code(), "OBJECTIVE_BLIND");
    }

    #[test]
    fn p3_passes_a_class_the_objective_separates() {
        let r = p3_objective_discrimination(&[
            arm("theta=0", &[1.00, 0.98, 1.02, 0.99, 1.01]),
            arm("extreme", &[0.40, 0.42, 0.38, 0.41, 0.39]),
            arm("random", &[0.24, 0.23, 0.25, 0.24, 0.23]),
        ])
        .unwrap();
        assert!(r.passed);
        assert!(r.measured > r.threshold);
    }

    #[test]
    fn p3_refuses_to_read_an_unengaged_instrument_as_a_null() {
        let mut blind = arm("extreme", &[]);
        blind.unmeasured = 5;
        let e = p3_objective_discrimination(&[arm("theta=0", &[0.5]), blind]).unwrap_err();
        assert_eq!(e.code(), "OBJECTIVE_BLIND");
        assert!(e.detail().contains("instrument_registry"));
    }

    #[test]
    fn p4_catches_the_tri_two_ceiling() {
        // Every arm within 0.02 of the F2 ceiling; the 2% margin of
        // [-0.5, 1.0] is 0.03, so this aborts.
        let e = p4_signal_non_saturation(
            FitnessFn::F2RegimeOccupancy,
            &arm("random", &[1.0, 0.99, 1.0, 1.0, 0.995]),
        )
        .unwrap_err();
        assert_eq!(e.code(), "OBJECTIVE_SATURATED");
        assert!(e.detail().contains("no headroom"));
    }

    #[test]
    fn p4_passes_an_objective_with_room_to_move() {
        let r = p4_signal_non_saturation(
            FitnessFn::F2RegimeOccupancy,
            &arm("random", &[0.24, 0.23, 0.25, 0.24, 0.23]),
        )
        .unwrap();
        assert!(r.passed);
    }

    #[test]
    fn p4_reports_unbounded_objectives_as_such_rather_than_guessing() {
        for f in [FitnessFn::F3ConstructorYield, FitnessFn::F5ConstructorReuse] {
            assert_eq!(achievable_range(f), None);
            let r = p4_signal_non_saturation(f, &arm("random", &[3.0, 4.0])).unwrap();
            assert!(r.detail.contains("unbounded above"));
        }
    }

    #[test]
    fn f4s_achievable_range_is_its_alphabet_ceiling() {
        assert_eq!(
            achievable_range(FitnessFn::F4PredictiveInformation),
            Some((0.0, 2.0))
        );
    }

    #[test]
    fn the_extreme_genome_really_is_extreme() {
        let v = allocation_extreme(3, 1);
        let denom: f32 = v.iter().map(|x| x.exp()).sum();
        assert!(v[1].exp() / denom > 0.99);
    }

    #[test]
    fn traversal_generation_is_early_enough_to_save_the_campaign() {
        assert_eq!(traversal_generation(40), 10);
        assert_eq!(traversal_generation(20), 5);
        assert_eq!(traversal_generation(2), 1, "never zero");
    }
}
