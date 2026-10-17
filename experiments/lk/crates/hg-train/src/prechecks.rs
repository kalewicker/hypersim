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
use hg_cognition::{CognitionConfig, MlpScorer, Parameterized, PolicyKind, StaticBias};
use hg_core::GraphSnapshot;
use hg_observe::{Binding, RuleSelectionPolicy};
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};

/// P-2's genome-movement bar, in units of the initial step size.
pub const TRAVERSAL_SIGMA_MULTIPLE: f64 = 3.0;
/// P-2's induced-allocation bar, as total variation.
pub const TRAVERSAL_TV_MIN: f64 = 0.05;
/// The weight scale of P-3's far-end probe genome for `MlpScorer`.
///
/// Chosen from measurement, not convention: at this scale the probe
/// separates the policy class on both D6 cells (F5 × `tri_two` spread
/// 306.15 over seed sd 38.61; F4 × `wn_three` spread 1.1828 over 0.2597),
/// which is what makes it a usable statement of "the far end of the class".
/// Deliberately **not** the campaign's `sigma` — see `class_extreme`.
pub const MLP_PROBE_SCALE: f32 = 2.0;

/// The L2 norm an `MlpScorer` genome is initialized at.
///
/// # Why a network is not initialized at zero here
///
/// `gumbel_top_k` adds `Gumbel(0, 1)` noise — sd ≈ 1.28 — to every raw
/// score before ranking. A scorer whose outputs differ by much less than
/// that is **behaviourally identical to budget-matched random no matter
/// what its parameters are**. And the output scale is quadratic in the
/// genome, because a score is `Σ_h w2[h] · relu(b1[h] + w1[h]·x)` and both
/// factors grow with `|theta|`. Measured on a 433-parameter scorer over 24
/// candidate feature vectors:
///
/// | `|theta|` | score spread |
/// |---|---|
/// | 0.54 | 0.0055 |
/// | 1 | 0.0188 |
/// | 3 | 0.169 |
/// | 5 | 0.470 |
/// | **7** | **0.921** |
/// | 10 | 1.880 |
/// | 20 | 7.519 |
/// | 41.6 | 32.530 |
///
/// So `theta = 0` sits at the centre of a wide basin in which the policy
/// cannot behave differently — its radius set by the Gumbel temperature,
/// not by float precision. A real Stage 3′ campaign moved `|theta|` to
/// 0.5414 over seven generations and P-2 correctly reported that not one
/// selection had changed in a 64-step probe.
///
/// `7.0` is where the score spread matches the Gumbel scale: the policy is
/// neither random (spread ≪ 1) nor a deterministic argmax (spread ≫ 1),
/// which is where a change in parameters has the most effect on behaviour.
/// Held as an **L2 norm** rather than a per-component sd so it means the
/// same thing across genome sizes, exactly as P-2's drift scale does.
///
/// # That justification is retired; the value is kept (E2)
///
/// Everything above describes `score_norm = "raw"`, which stopped being the
/// default at E1. Under N-1 the standardized scores are *exactly* invariant
/// under `θ → cθ` — the `c·b₂` term is constant across candidates and dies
/// in the mean-subtraction, the surviving `c²` cancels — so above the
/// `SCORE_NORM_EPS` floor **the norm has no behavioural effect at all**.
/// Measured: `‖θ‖ ∈ {1, 7, 42, 420}` produce byte-identical selections over
/// a 64-step probe, and identical F5 to every printed digit
/// (`e2_n1_rerun.rs::where_the_neutral_point_sits_under_n1`).
///
/// The value stays at 7.0 for two reasons, neither of them the original
/// one: it sits two orders of magnitude clear of the ε floor, whose basin
/// the same measurement puts at `‖θ‖ ≲ 1`; and holding it fixed keeps D6
/// and E2 genomes comparable. **It is no longer a tuning parameter** — what
/// `class_init` now samples is a *direction*, and the norm only has to be
/// large enough not to matter. See `docs/policy_config.md` §4C–4D.
pub const MLP_INIT_L2: f32 = 7.0;

/// The genome a campaign starts from, for the policy class in use.
///
/// Zero for logit classes — `StaticBias` at zero is the uniform allocation,
/// which is the right neutral start and is *not* behaviourally degenerate.
/// A seeded draw at `MLP_INIT_L2` for `MlpScorer`, for the reason above.
pub fn class_init(policy: PolicyKind, n_params: usize, seed: u64) -> Vec<f32> {
    match policy {
        PolicyKind::MlpScorer if n_params > 0 => {
            let mut rng = <rand_chacha::ChaCha8Rng as rand::SeedableRng>::seed_from_u64(seed);
            let raw: Vec<f32> = (0..n_params)
                .map(|_| crate::optim::standard_normal(&mut rng))
                .collect();
            let norm = raw.iter().map(|v| v * v).sum::<f32>().sqrt();
            if norm <= 0.0 {
                return raw;
            }
            raw.iter().map(|v| v * MLP_INIT_L2 / norm).collect()
        }
        _ => vec![0.0f32; n_params],
    }
}

/// P-3's discrimination bar, in units of seed sd.
pub const DISCRIMINATION_SIGMA_MULTIPLE: f64 = 2.0;
/// P-4's saturation margin, as a fraction of the objective's achievable
/// range.
pub const SATURATION_MARGIN: f64 = 0.02;

/// Minimum independent draws a go/no-go gate must take (plan v3 §5.2).
///
/// Stage 0b's separation ratio exceeded its own hard bound on a majority of
/// seed triples under both r1 and r2, and every published class turned out
/// to be one draw from a distribution spanning two classes. The instructive
/// part is not that a statistic was wrong — it is that the single-triple
/// table was internally consistent, told a satisfying story, and was wrong
/// in every particular. **Nothing about it looked wrong.** Eight draws
/// caught it, and eight draws cost 47 seconds.
///
/// Cheapness is the argument *for* replication, not against it.
pub const MIN_GATE_DRAWS: usize = 5;

/// What a gate reports once it is required to draw more than once.
#[derive(Clone, Debug, PartialEq)]
pub struct GateVerdict {
    pub name: &'static str,
    /// Per-draw pass/fail.
    pub draws: Vec<bool>,
    /// Per-draw value of whatever statistic the gate thresholds.
    pub values: Vec<f64>,
}

impl GateVerdict {
    pub fn n(&self) -> usize {
        self.draws.len()
    }
    /// Fraction of draws agreeing with the modal verdict.
    pub fn modal_fraction(&self) -> f64 {
        if self.draws.is_empty() {
            return f64::NAN;
        }
        let pass = self.draws.iter().filter(|d| **d).count();
        pass.max(self.draws.len() - pass) as f64 / self.draws.len() as f64
    }
    pub fn modal_pass(&self) -> bool {
        self.draws.iter().filter(|d| **d).count() * 2 > self.draws.len()
    }
    pub fn spread(&self) -> f64 {
        if self.values.len() < 2 {
            return 0.0;
        }
        self.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            - self.values.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// The gate's answer, or a refusal to give one.
    ///
    /// Two distinct refusals, and conflating them is the failure this
    /// exists to prevent:
    ///
    /// * **too few draws** — the gate was not asked properly. A verdict
    ///   from `< MIN_GATE_DRAWS` draws is not a weak verdict, it is not a
    ///   verdict.
    /// * **draws disagree** — the gate was asked properly and cannot say.
    ///   `GATE_UNSTABLE`, and the decision escalates to explicit judgment
    ///   rather than being resolved by whichever draw ran first.
    ///
    /// Neither is an abort. An abort says *this campaign must not run*;
    /// these say *this gate cannot say*.
    pub fn resolve(&self) -> Result<bool, GateAbort> {
        if self.n() < MIN_GATE_DRAWS {
            return Err(GateAbort::TooFewDraws {
                got: self.n(),
                need: MIN_GATE_DRAWS,
                name: self.name,
            });
        }
        if self.modal_fraction() < 1.0 {
            return Err(GateAbort::Unstable {
                name: self.name,
                draws: self.draws.clone(),
                values: self.values.clone(),
                modal_fraction: self.modal_fraction(),
                spread: self.spread(),
            });
        }
        Ok(self.modal_pass())
    }
}

/// A gate declining to return a verdict. **Not** a `PrecheckAbort`.
#[derive(Clone, Debug, PartialEq)]
pub enum GateAbort {
    TooFewDraws {
        name: &'static str,
        got: usize,
        need: usize,
    },
    Unstable {
        name: &'static str,
        draws: Vec<bool>,
        values: Vec<f64>,
        modal_fraction: f64,
        spread: f64,
    },
}

impl GateAbort {
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooFewDraws { .. } => "GATE_UNDERDRAWN",
            Self::Unstable { .. } => "GATE_UNSTABLE",
        }
    }
}

impl std::fmt::Display for GateAbort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooFewDraws { name, got, need } => write!(
                f,
                "{}: gate {name:?} took {got} draw(s) against a minimum of {need}. A \
                 verdict from too few draws is not a weak verdict — it is not a verdict \
                 (plan v3 §5.2)",
                self.code()
            ),
            Self::Unstable {
                name,
                draws,
                values,
                modal_fraction,
                spread,
            } => write!(
                f,
                "{}: gate {name:?} disagreed across draws — {}/{} agree with the modal \
                 verdict (fraction {modal_fraction:.2}), statistic spread {spread:.4}, \
                 per-draw {draws:?} values {values:?}. The gate returns no verdict; the \
                 decision escalates to explicit judgment rather than being resolved by \
                 whichever draw ran first",
                self.code(),
                (modal_fraction * draws.len() as f64).round() as usize,
                draws.len()
            ),
        }
    }
}

impl std::error::Error for GateAbort {}

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
    /// Per-step hash of *which candidates were selected*.
    ///
    /// The per-rule counts above are a `StaticBias` notion: that class
    /// chooses a distribution over rules, so a change in its behaviour
    /// necessarily shows up there. `MlpScorer` chooses among individual
    /// bindings, and can change completely while leaving the per-rule
    /// proportions untouched — so a traversal statistic built only on
    /// `selections_per_rule` cannot see it (`selection_divergence`).
    pub selection_signature: Vec<u64>,
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
        // FNV-1a over the sorted selection, salted with the candidate count
        // so "picked the same indices from a different-sized pool" cannot
        // collide with "behaved identically".
        let mut sorted = picked.clone();
        sorted.sort_unstable();
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for v in std::iter::once(candidates.len()).chain(sorted) {
            for b in (v as u64).to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        t.selection_signature.push(h);
        picked
    }
}

/// The first step at which two probe traces selected differently, and how
/// many steps were comparable.
///
/// Both traces must come from the **same sim seed**: the engine and the
/// policy RNG are then identical, so the two rollouts stay in lockstep for
/// exactly as long as the two parameterizations make the same choices. The
/// first divergence is therefore a clean "these are different policies"
/// signal, and no divergence over the whole probe is a clean "these are the
/// same policy".
///
/// Nothing after the first divergence is comparable — the graphs differ
/// from the next step on — which is why this returns the first index rather
/// than a rate.
pub fn selection_divergence(a: &RuleTrace, b: &RuleTrace) -> (Option<usize>, usize) {
    let n = a.selection_signature.len().min(b.selection_signature.len());
    for i in 0..n {
        if a.selection_signature[i] != b.selection_signature[i] {
            return (Some(i), n);
        }
    }
    (None, n)
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
    policy: PolicyKind,
    theta_init: &[f32],
    theta_now: &[f32],
    sigma_init: f64,
    alloc_init: &[f64],
    alloc_now: &[f64],
    // First step at which a theta_init probe and a theta_now probe chose
    // differently, over a shared sim seed, and the steps compared.
    divergence: (Option<usize>, usize),
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
    let (diverged_at, steps_compared) = divergence;

    // Which statistic is decisive depends on what the class actually
    // chooses. `StaticBias` picks a distribution over rules, so any change
    // in its behaviour must show in the per-rule allocation. `MlpScorer`
    // picks among individual bindings and can change completely while
    // leaving the per-rule proportions untouched — measured, at generation 7
    // of a real Stage 3' campaign, TV was 0.0000 for both D6 cells while
    // |dtheta| had moved 0.54 and 0.70. Reading that TV as "frozen" is the
    // per-rule statistic being applied to a policy that is not per-rule.
    let per_rule_class = !matches!(policy, PolicyKind::MlpScorer);
    let frozen = if per_rule_class {
        tv <= TRAVERSAL_TV_MIN
    } else {
        diverged_at.is_none()
    };

    let both = format!(
        "induced-allocation TV = {tv:.4} (bar {TRAVERSAL_TV_MIN}); selection divergence \
         {} over {steps_compared} probe steps; |dtheta| = {l2:.4} = \
         {moved_in_units_of_drift:.2}x the drift scale (sigma_init * sqrt(n) = {drift:.4})",
        match diverged_at {
            Some(i) => format!("at step {i}"),
            None => "NONE".to_string(),
        }
    );

    if frozen {
        // Both the failure mode AND the class matter here. A logit-space
        // explanation handed to a scorer campaign is the same category
        // error this check exists to avoid, one level up.
        let barely_moved = moved_in_units_of_drift <= 1.0;
        let which = match (per_rule_class, barely_moved) {
            (true, true) => {
                "the step size is too small for this parameterization: sigma is a step \
                 in the policy's OWN parameter space, and for a logit policy that is \
                 softmax-logit space, where the default 0.02 moves the allocation by \
                 under a percentage point"
            }
            (true, false) => {
                "the genome moved but along a direction the policy cannot act on — \
                 adding a constant to every logit moves theta arbitrarily far and \
                 changes the softmax not at all"
            }
            (false, true) => {
                "the step size is too small for this parameterization. Note that raising \
                 sigma will not fix it: the ES update is alpha/(pairs*sigma) times the \
                 rank-weighted perturbation sum, so sigma DIVIDES the step even as it \
                 widens the exploration. Raise alpha, or lower sigma, and check that the \
                 population still lands where the policy is behaviourally live \
                 (prechecks::MLP_INIT_L2)"
            }
            (false, false) => {
                "the genome moved but the scorer's ranking did not: every step of the \
                 probe selected the same candidates as at initialization, so the \
                 movement is in directions the ranking is invariant to"
            }
        };
        return Err(PrecheckAbort::OptimizerFrozen(format!(
            "at generation {generation}: {both}. {which}"
        )));
    }
    Ok(PrecheckResult {
        name: "P-2 optimizer traversal",
        passed: true,
        measured: if per_rule_class {
            tv
        } else {
            diverged_at.map(|i| i as f64).unwrap_or(f64::NAN)
        },
        threshold: if per_rule_class {
            TRAVERSAL_TV_MIN
        } else {
            0.0
        },
        detail: both,
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

/// Is there room above the random arm for a champion to win in?
///
/// `tri_two` at 384 steps is the standing example: every arm sat within 0.02
/// of the F2 ceiling because the budget had been chosen for headroom at a
/// 192-step horizon. Any margin measured there is noise, and the campaign
/// was uninformative rather than negative.
///
/// # Deviation from plan v2 §4's literal wording, and why
///
/// The plan asks "is the objective at ceiling **or floor** for the random
/// arm?" and treats the two symmetrically. They are not symmetric, and the
/// symmetric reading aborts campaigns that are in the best possible shape.
///
/// Headroom is directional because the margin under test is directional:
/// every stage asks whether a champion can **beat** budget-matched random.
/// A random arm at the *ceiling* leaves nowhere to go and any margin there
/// is noise — that is `tri_two`, and it is the case the check exists for. A
/// random arm at the *floor* leaves the entire range to win in, which is
/// the ideal baseline, not a defect. Measured: the F-L1 artifacts fixture
/// pins random at F2's floor of `-0.5` while `theta=0` and the allocation
/// extreme separate cleanly — P-3 passes, and a symmetric P-4 would abort
/// it anyway.
///
/// So saturation is checked against the ceiling only. The floor case is
/// still *reported*, because a zero-variance random arm makes every
/// margin's pooled sd zero and every effect size infinite — which is bug
/// 4's shape — but that is the spread guard's job
/// (`docs/instrument_registry.md` §2.6), not this one's.
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
    // Directional: room ABOVE random, which is the direction every stage's
    // margin is measured in.
    let headroom = hi - m;
    if headroom < margin {
        return Err(PrecheckAbort::ObjectiveSaturated(format!(
            "the random arm scores {m:.4} on {}, only {headroom:.4} below the ceiling of \
             the achievable range [{lo}, {hi}], whose {}% margin is {margin:.4}. There is \
             no room above it, so any champion margin here is measuring noise. Re-choose \
             the horizon or the budget and report the achievable range alongside",
            which.as_str(),
            SATURATION_MARGIN * 100.0
        )));
    }
    let at_floor = (m - lo) < margin;
    Ok(PrecheckResult {
        name: "P-4 signal non-saturation",
        passed: true,
        measured: headroom,
        threshold: margin,
        detail: format!(
            "random at {m:.4}, {headroom:.4} of room below the ceiling of [{lo}, {hi}]{}",
            if at_floor {
                " — random sits AT THE FLOOR, which is maximum headroom, but watch its \
                 seed spread: a zero-variance baseline makes every effect size infinite"
            } else {
                ""
            }
        ),
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
        FitnessFn::F4MonotoneDrift => {
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

/// P-3's far-end probe genome **for the policy class actually in use**.
///
/// # Why this is not just `allocation_extreme`
///
/// `allocation_extreme` is a statement about `StaticBias`'s parameter
/// space: one softmax logit at 8.0 puts >99% of the allocation on one rule,
/// which is genuinely the far end of that class. Applied to an `MlpScorer`
/// genome it means `w1[0] = 8.0` and everything else zero — and `w2` is
/// part of "everything else", so the one live hidden row is multiplied by
/// zero on the way out. **Every candidate scores exactly 0.0, which is the
/// same policy as θ=0, which is the same policy as budget-matched random.**
///
/// Measured, on a `tri_two`-shaped `MlpScorer` (433 parameters, feature
/// dim 25): `score` is `0.000000` for two very different feature vectors
/// under θ=0 *and* under `allocation_extreme`. So P-3 would have compared
/// three arms that are provably one arm, found no spread, and aborted
/// `OBJECTIVE_BLIND` — reporting "the objective cannot see the class" when
/// what actually happened is that the probe never left the origin. That is
/// the `MlpScorer` analogue of P-1's one-rule `StaticBias` case, and it has
/// the same remedy: refuse to draw a conclusion from a degenerate probe.
///
/// For `MlpScorer` the far end is a **seeded draw at `MLP_PROBE_SCALE`**.
///
/// # Why the scale is fixed rather than the campaign's `sigma`
///
/// This first shipped scaled by the campaign's `sigma`, on the reasoning
/// that the probe should live where the optimizer will actually look. That
/// is wrong, and it aborted both D6 cells before either campaign started:
/// at `sigma = 0.5` the probe sits close enough to θ=0 that its behaviour
/// is close to random, so P-3 saw a spread of 41.43 against a seed sd of
/// 53.00 on F5 × `tri_two` and reported `OBJECTIVE_BLIND` — while the same
/// objective and the same substrate separate the class by **306.15 over
/// sd 38.61** at `MLP_PROBE_SCALE`.
///
/// The error is a category one. P-3 asks *"does the objective assign
/// different values to at least two policies **in the class**"* — a
/// question about the class. Scaling its probe by the optimizer's step size
/// makes P-3's verdict a function of `sigma`, so a small `sigma` produces
/// an `OBJECTIVE_BLIND` abort: a fact about the optimizer, reported under
/// the objective's abort code, in exactly the way plan v2 §4 separates
/// P-2's question from P-3's. Whether the *search* can see the objective
/// from one perturbation is P-2's business, and P-2 measures it on real
/// steps rather than on a synthetic genome.
pub fn class_extreme(policy: PolicyKind, n_params: usize, seed: u64) -> Vec<f32> {
    match policy {
        PolicyKind::MlpScorer => {
            let mut rng = <rand_chacha::ChaCha8Rng as rand::SeedableRng>::seed_from_u64(seed);
            (0..n_params)
                .map(|_| MLP_PROBE_SCALE * crate::optim::standard_normal(&mut rng))
                .collect()
        }
        // `StaticBias` and anything else parameterized per rule.
        _ => allocation_extreme(n_params, 0),
    }
}

/// Does this probe genome actually make the policy behave differently from
/// `theta = 0`?
///
/// P-3 compares arms; if two of them are the same policy, its spread is
/// measuring nothing. This answers the question directly for score-based
/// classes, by scoring two dissimilar feature vectors under both genomes.
/// `None` for classes whose behaviour is not a per-candidate score.
pub fn probe_genome_is_live(
    policy: PolicyKind,
    n_rules: usize,
    wl_buckets: usize,
    hidden: usize,
    theta: &[f32],
) -> Option<bool> {
    if policy != PolicyKind::MlpScorer {
        return None;
    }
    let mut p = MlpScorer::new(n_rules, wl_buckets, hidden, None, 0);
    if theta.len() != p.n_params() {
        return Some(false);
    }
    p.set_params(theta);
    let d = p.feat_dim();
    let a: Vec<f32> = (0..d).map(|i| 0.1 * (i as f32 + 1.0)).collect();
    let b: Vec<f32> = (0..d).map(|i| 0.9 - 0.05 * i as f32).collect();
    Some((p.score(&a) - p.score(&b)).abs() > 1e-9)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The D6 defect, pinned. `allocation_extreme` is a `StaticBias` idea,
    /// and applied to an `MlpScorer` genome it produces a policy that is
    /// bit-identical in behaviour to `theta = 0` — so P-3's three arms
    /// collapse to one and its abort would be about the probe, not the
    /// objective.
    #[test]
    fn the_static_bias_extreme_is_a_dead_probe_for_an_mlp() {
        let (n_rules, wl, hidden) = (2usize, 16usize, 16usize);
        let n = MlpScorer::new(n_rules, wl, hidden, None, 0).n_params();

        let dead = allocation_extreme(n, 0);
        assert_eq!(
            probe_genome_is_live(PolicyKind::MlpScorer, n_rules, wl, hidden, &dead),
            Some(false),
            "w1[0] = 8.0 is gated by w2 = 0, so every candidate still scores 0"
        );

        let live = class_extreme(PolicyKind::MlpScorer, n, 12345);
        assert_eq!(
            probe_genome_is_live(PolicyKind::MlpScorer, n_rules, wl, hidden, &live),
            Some(true),
            "a seeded draw at the search's own scale must actually score differently"
        );

        // And the check stays out of the way of the class it was written for.
        assert_eq!(
            probe_genome_is_live(PolicyKind::StaticBias, n_rules, wl, hidden, &dead),
            None
        );
        assert_eq!(
            class_extreme(PolicyKind::StaticBias, n_rules, 1),
            allocation_extreme(n_rules, 0),
            "StaticBias keeps the logit probe that is correct for it"
        );
    }

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
        let e = p2_optimizer_traversal(PolicyKind::StaticBias, &init, &frozen, 0.02, &uniform, &barely, (Some(3), 64), 10).unwrap_err();
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
        let r = p2_optimizer_traversal(PolicyKind::StaticBias, &init, &moved, 2.0, &uniform, &now, (Some(1), 64), 10).unwrap();
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
        let e = p2_optimizer_traversal(PolicyKind::StaticBias, &init, &shifted, 0.5, &uniform, &uniform, (Some(2), 64), 10).unwrap_err();
        assert_eq!(e.code(), "OPTIMIZER_FROZEN");
        assert!(
            e.detail().contains("cannot act on"),
            "the two frozen modes must be distinguishable in the message: {}",
            e.detail()
        );
    }

    /// The D6 defect, pinned. P-2's decisive statistic was the **per-rule**
    /// allocation, which is what `StaticBias` chooses. `MlpScorer` chooses
    /// among individual bindings and can change completely while leaving
    /// the per-rule proportions untouched — measured, at generation 7 of a
    /// real Stage 3′ campaign, TV was `0.0000` on both cells while `|dθ|`
    /// had moved 0.54 and 0.70. Read as "frozen", that is a per-rule
    /// statistic applied to a policy that is not per-rule.
    #[test]
    fn p2_reads_a_scorer_by_its_selections_and_a_logit_policy_by_its_allocation() {
        let init = vec![0.0f32; 400];
        // |dtheta| = 3 * sqrt(400) = 60, against a drift scale of
        // sigma * sqrt(n) = 2 * 20 = 40 — so the genome moved 1.5x drift and
        // "the step size is too small" is not available as an explanation.
        let moved: Vec<f32> = (0..400).map(|i| if i % 2 == 0 { 3.0 } else { -3.0 }).collect();
        let uniform = vec![0.5, 0.5];

        // Identical per-rule allocation, but the scorer picked different
        // candidates from step 4 on. That is a traversing search.
        let ok = p2_optimizer_traversal(
            PolicyKind::MlpScorer,
            &init,
            &moved,
            2.0,
            &uniform,
            &uniform,
            (Some(4), 64),
            7,
        )
        .expect("a scorer that selects differently is not frozen");
        assert!(ok.passed);
        assert!(
            ok.detail.contains("TV = 0.0000"),
            "the per-rule number is still reported, just not decisive: {}",
            ok.detail
        );

        // Same allocation AND the same selections every step: genuinely frozen.
        let e = p2_optimizer_traversal(
            PolicyKind::MlpScorer,
            &init,
            &moved,
            2.0,
            &uniform,
            &uniform,
            (None, 64),
            7,
        )
        .unwrap_err();
        assert_eq!(e.code(), "OPTIMIZER_FROZEN");
        assert!(
            e.detail().contains("ranking did not"),
            "a scorer's frozen mode must name the ranking, not the softmax: {}",
            e.detail()
        );

        // And the per-rule class is unaffected: allocation still decides,
        // even though these probes diverged.
        let e2 = p2_optimizer_traversal(
            PolicyKind::StaticBias,
            &init,
            &moved,
            2.0,
            &uniform,
            &uniform,
            (Some(4), 64),
            7,
        )
        .unwrap_err();
        assert_eq!(e2.code(), "OPTIMIZER_FROZEN");
    }

    /// `MLP_INIT_L2`'s justification, as an assertion.
    ///
    /// A scorer is only behaviourally live where its output spread is
    /// comparable to `gumbel_top_k`'s `Gumbel(0, 1)` noise. Too small and
    /// the policy is budget-matched random whatever its weights; too large
    /// and it is a deterministic argmax that the Gumbel draw cannot
    /// perturb. Both extremes make a search uninformative, in opposite
    /// ways.
    #[test]
    fn the_mlp_init_scale_lands_where_the_scorer_can_actually_act() {
        let mut p = MlpScorer::new(2, 16, 16, Some(4), 1);
        let d = p.feat_dim();
        let xs: Vec<Vec<f32>> = (0..24)
            .map(|c| (0..d).map(|i| (((i * 7 + c * 13) % 11) as f32) / 10.0).collect())
            .collect();
        let spread_at = |p: &mut MlpScorer, g: &[f32]| {
            p.set_params(g);
            let s: Vec<f32> = xs.iter().map(|x| p.score(x)).collect();
            s.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
                - s.iter().cloned().fold(f32::INFINITY, f32::min)
        };
        let n = p.n_params();

        let init = class_init(PolicyKind::MlpScorer, n, 4242);
        let l2 = init.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!(
            (l2 - MLP_INIT_L2).abs() < 1e-3,
            "class_init must land on the stated norm, got {l2}"
        );

        // Gumbel(0,1) has sd ~1.28. "Comparable" is generous on purpose —
        // the claim is an order of magnitude, not a tuned constant.
        let s = spread_at(&mut p, &init);
        assert!(
            (0.3..=4.0).contains(&s),
            "score spread {s} is not comparable to the Gumbel scale"
        );

        // The basin the campaign was starting in: the genome P-2 measured
        // after seven generations from zero.
        let tiny: Vec<f32> = init.iter().map(|v| v * 0.5414 / MLP_INIT_L2).collect();
        assert!(
            spread_at(&mut p, &tiny) < 0.05,
            "|theta| = 0.54 must be far inside the behaviourally-flat basin"
        );

        // And zero is exactly flat.
        assert_eq!(spread_at(&mut p, &vec![0.0; n]), 0.0);

        // A logit class keeps its zero start — that one is not degenerate.
        assert_eq!(class_init(PolicyKind::StaticBias, 3, 1), vec![0.0; 3]);
    }

    // ---- gate discipline (plan v3 §5.2) ----

    fn gate(name: &'static str, draws: &[bool], values: &[f64]) -> GateVerdict {
        GateVerdict {
            name,
            draws: draws.to_vec(),
            values: values.to_vec(),
        }
    }

    /// E1's exit criterion: **a single-draw gate refuses to return a
    /// verdict.**
    #[test]
    fn a_single_draw_gate_refuses_to_return_a_verdict() {
        let g = gate("P-3", &[true], &[1.0]);
        let e = g.resolve().unwrap_err();
        assert_eq!(e.code(), "GATE_UNDERDRAWN");
        assert!(
            e.to_string().contains("not a verdict"),
            "the message must say what a too-few-draws result is: {e}"
        );

        // And the boundary is where it is stated to be.
        let four = gate("P-3", &[true; 4], &[1.0; 4]);
        assert!(four.resolve().is_err());
        let five = gate("P-3", &[true; 5], &[1.0; 5]);
        assert_eq!(five.resolve(), Ok(true));
    }

    /// Disagreement is a distinct refusal from underdrawing, and neither is
    /// an abort.
    #[test]
    fn disagreeing_draws_return_gate_unstable_with_the_distribution() {
        // Stage 0b's actual shape: 5 of 8 triples over the hard bound.
        let g = gate(
            "stage0b separation",
            &[true, true, true, false, false, false, false, false],
            &[0.75, 0.84, 0.93, 1.24, 1.31, 1.54, 1.70, 2.09],
        );
        let e = g.resolve().unwrap_err();
        assert_eq!(e.code(), "GATE_UNSTABLE");
        let msg = e.to_string();
        for want in ["5/8", "spread", "escalates"] {
            assert!(msg.contains(want), "message must carry {want:?}: {msg}");
        }
        assert!((g.modal_fraction() - 0.625).abs() < 1e-9);
        assert!((g.spread() - 1.34).abs() < 1e-9);

        // A unanimous gate resolves either way.
        assert_eq!(gate("u", &[false; 6], &[0.1; 6]).resolve(), Ok(false));
        assert_eq!(gate("u", &[true; 6], &[0.9; 6]).resolve(), Ok(true));
    }

    /// The two refusals must not be conflated with each other or with an
    /// abort — an abort says the campaign must not run, these say the gate
    /// cannot say.
    #[test]
    fn the_two_gate_refusals_are_distinguishable() {
        let under = gate("g", &[true, true], &[1.0, 1.0]).resolve().unwrap_err();
        let unstable = gate("g", &[true, true, true, false, false], &[1.0; 5])
            .resolve()
            .unwrap_err();
        assert_ne!(under.code(), unstable.code());
        assert_eq!(under.code(), "GATE_UNDERDRAWN");
        assert_eq!(unstable.code(), "GATE_UNSTABLE");
    }

    /// `selection_divergence` itself.
    #[test]
    fn selection_divergence_finds_the_first_differing_step() {
        let mk = |sig: &[u64]| RuleTrace {
            selection_signature: sig.to_vec(),
            ..Default::default()
        };
        assert_eq!(
            selection_divergence(&mk(&[1, 2, 3, 4]), &mk(&[1, 2, 9, 4])),
            (Some(2), 4)
        );
        assert_eq!(
            selection_divergence(&mk(&[1, 2, 3, 4]), &mk(&[1, 2, 3, 4])),
            (None, 4)
        );
        // Only the common prefix is comparable — after a divergence the two
        // rollouts are on different graphs, and a shorter trace must not be
        // read as agreement past its end.
        assert_eq!(
            selection_divergence(&mk(&[1, 2]), &mk(&[1, 2, 3, 4])),
            (None, 2)
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
        assert!(e.detail().contains("no room above it"), "{}", e.detail());
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
    fn p4_treats_the_floor_as_headroom_not_as_saturation() {
        // A random arm pinned at F2's floor of -0.5 has the ENTIRE range to
        // be beaten in. Reading that as saturation aborts the campaigns in
        // the best possible shape — measured on the F-L1 artifacts fixture,
        // where random sits at -0.5 while theta=0 and the allocation
        // extreme separate cleanly.
        let r = p4_signal_non_saturation(
            FitnessFn::F2RegimeOccupancy,
            &arm("random", &[-0.5, -0.5, -0.5]),
        )
        .expect("the floor is not saturation");
        assert!(r.passed);
        assert!((r.measured - 1.5).abs() < 1e-9, "headroom is hi - mean");
        assert!(
            r.detail.contains("AT THE FLOOR"),
            "the zero-variance hazard must still be surfaced: {}",
            r.detail
        );
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
            achievable_range(FitnessFn::F4MonotoneDrift),
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
