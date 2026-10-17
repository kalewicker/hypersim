//! Stage 0a / 0b / 1 diagnostics (NN plan §11).
//!
//! These are cheap gates that run *before* any campaign, and every one of
//! them is designed to be able to kill the programme on a given rule set:
//!
//! - **Stage 0a** — does selection under budget have any causal power at
//!   all? If no hand heuristic can shift a gate-relevant metric against
//!   budget-matched random, Stages 2–3 are pointless on that rule set, and
//!   *why* the dynamics are selection-invariant becomes the interesting
//!   question instead.
//! - **Stage 0b** — does the substrate sit in the reservoir-viable band?
//!   Perturbations that die instantly carry no information forward;
//!   perturbations that diverge to the random baseline carry no *fading*
//!   memory. Only the intermediate plateau is usable.
//! - **Stage 1** — does the substrate actually compute? Memory capacity of
//!   a linear readout over the reservoir state.
//!
//! Everything here writes nothing: results come back as structs, and the
//! caller decides what to record. A diagnostic that silently wrote into
//! `runs/` would be exactly the contamination `docs/campaign_output.md` §5
//! forbids.

use crate::ridge::{memory_capacity, MemoryCapacity, DEFAULT_LAMBDA};
use hg_cognition::readout::ReadoutSet;
use hg_cognition::{
    DegreeDirection, DegreeGreedy, PassThrough, PolicyStack, RandomSubset, StimulusDriver,
};
use hg_core::GraphSnapshot;
use hg_observe::RuleSelectionPolicy;
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig, SimError};

/// A rule set paired with an initial condition it actually fires on.
///
/// The pairing is not incidental. A rule set only matches on an IC
/// containing its LHS; on the wrong one it produces zero candidates for the
/// whole run, and every policy then trivially agrees because there is
/// nothing to select. Reporting such a pairing as "selection-invariant"
/// would be a statement about the initial condition, not the substrate
/// (`docs/policy_config.md` §2.5).
#[derive(Clone, Debug)]
pub struct Fixture {
    pub name: &'static str,
    pub rules: RuleSource,
    pub ic: &'static str,
    pub width: u64,
    pub node_count: u64,
    pub steps: u64,
    pub window: u64,
    /// `MaxParallel` unless a fixture needs rate-limiting; Stage 1's
    /// reservoir uses `SequentialRandom` so injected structure survives
    /// long enough to be processed.
    pub step_mode: &'static str,
}

#[derive(Clone, Debug)]
pub enum RuleSource {
    Builtin(&'static str),
    File(&'static str),
}

fn steps_of(f: &Fixture) -> u64 {
    f.steps
}

impl Fixture {
    /// Config with `[causal]` **off** — the default for Stage 0a/0b/1,
    /// which score F1/F2 and never touch constructor detection.
    pub fn sim_config(&self, out_dir: &std::path::Path, seed: u64) -> SimConfig {
        self.sim_config_inner(out_dir, seed, false)
    }

    /// Config with `[causal]` **on**.
    ///
    /// Required for anything reading F3: `n_confirmed_constructors` comes
    /// from `hg-causal`'s constructor detection, and with `[causal]`
    /// disabled `RolloutMetrics::constructors` is empty, so F3 is
    /// identically 0.0 for every arm. Scoring an F3 comparison against a
    /// causal-disabled config would produce a table of zeros and read as
    /// "no policy affects constructor yield".
    pub fn sim_config_causal(&self, out_dir: &std::path::Path, seed: u64) -> SimConfig {
        self.sim_config_inner(out_dir, seed, true)
    }

    fn sim_config_inner(
        &self,
        out_dir: &std::path::Path,
        seed: u64,
        causal: bool,
    ) -> SimConfig {
        let rules = match self.rules {
            RuleSource::Builtin(n) => {
                format!("source = \"Builtin\"\nbuiltin_name = \"{n}\"")
            }
            RuleSource::File(p) => format!(
                "source = \"File\"\nfile_path = \"{}/../../{p}\"",
                env!("CARGO_MANIFEST_DIR")
            ),
        };
        let ei_interval = (steps_of(self) / 3).max(1);
        let (name, ic, width, node_count, steps, window, step_mode) = (
            self.name,
            self.ic,
            self.width,
            self.node_count,
            self.steps,
            self.window,
            self.step_mode,
        );
        let toml = format!(
            r#"
experiment_name = "diag_{name}"
seed = {seed}
out_dir = "{}"
[rules]
{rules}
[initial]
condition_type = "{ic}"
width = {width}
height = {width}
node_count = {node_count}
[engine]
max_steps = {steps}
metrics_window_interval = {window}
step_mode = "{step_mode}"
thread_count = 1
[evaluator]
enabled = true
[causal]
enabled = {causal}
[causal_language]
# EI is emitted every `ei_interval_steps`, and the DEFAULT IS 1000 — longer
# than most fixture horizons here, so with the default nothing is ever
# emitted and `ei_micro`/`ei_macro`/`ce_structural` are None on every row.
# Read through `mean()` that surfaces as NaN, which looks like "shaping has
# no effect on EI" rather than "EI was never computed". Scaled to the
# horizon so roughly three emissions land inside any run.
#
# `ei_minimum_transitions` is deliberately LEFT AT ITS DEFAULT of 1000: it
# is a certainty threshold, not a tuning knob (the certainty table requires
# >= 1000 observed transitions before an EI claim counts as Empirical), and
# lowering it to force emissions would be manufacturing the measurement.
ei_interval_steps = {ei_interval}
[campaign.abort]
# Diagnostics are not campaign cells and nothing is paying for their host
# time by the window, so the Exploding early-abort gate is off here. It
# defaults to 2 consecutive Exploding windows, which on an `Empty` initial
# condition fires on the STARTUP TRANSIENT — a graph going 0 -> 3 -> 6
# nodes has an enormous growth ratio for reasons that have nothing to do
# with its dynamics. Left at the default, a Stage 1 run terminated at
# window 3 of 128 and reported MC = 0 for want of samples.
exploding_windows = 1000000
[output]
fixed_provenance = true
"#,
            out_dir.display()
        );
        toml::from_str(&toml).expect("diagnostic config parses")
    }
}

// ---------------------------------------------------------------------
// Stage 0a — budget response curve + heuristic causal-power probe
// ---------------------------------------------------------------------

/// The budget ladder. `None` is the plan's `k = ∞` arm — but see
/// [`ArmStats::pairings_cap_bound_windows`]: it is only unconstrained if the
/// engine's own `max_pairings_per_step` never binds.
pub const BUDGET_LADDER: [Option<usize>; 5] =
    [Some(1), Some(4), Some(16), Some(64), None];

#[derive(Clone, Debug, Default)]
pub struct ArmStats {
    pub n_seeds: usize,
    /// Gate-relevant metrics, mean ± sd over seeds.
    pub f2_mean: f64,
    pub f2_sd: f64,
    pub target_fraction_mean: f64,
    pub interestingness_mean: f64,
    pub interestingness_sd: f64,
    pub final_nodes_mean: f64,
    pub active_persistent_mean: f64,
    pub active_persistent_sd: f64,
    /// Windows in which the engine's `max_pairings_per_step` bound.
    pub pairings_cap_bound_windows: u64,
    pub attempted_total: u64,
    pub failures: usize,
}

fn stats(v: &[f64]) -> (f64, f64) {
    if v.is_empty() {
        return (0.0, 0.0);
    }
    let m = v.iter().sum::<f64>() / v.len() as f64;
    let sd = if v.len() < 2 {
        0.0
    } else {
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
    };
    (m, sd)
}

/// Factory for one Stage 0a arm: takes the simulation seed and returns a
/// policy for that rollout.
pub type ArmFactory = Box<dyn Fn(u64) -> Box<dyn RuleSelectionPolicy> + Send + Sync>;

/// The policies Stage 0a compares at each budget. All zero-parameter: this
/// stage runs before any training, on purpose.
///
/// ## Why the factory takes the seed
///
/// The first version built `RandomSubset::new(b, 1)` — a policy RNG seed
/// pinned to the literal 1, identical on every rollout. Combined with the
/// fact that three of the four remaining fixtures have *constructed*
/// (not sampled) initial conditions and `MaxParallel` never consults the
/// engine RNG, that made the baseline arm bit-identical across all 20
/// seeds, which is what produced the wall of `DETERMINISTIC` cells in the
/// first sweep. The determinism was partly the harness's, not the
/// substrate's: `crates/hg-train/tests/c5_variance_probe.rs` measures a
/// hold-out F2 sd of 0.111 on `glider` from the policy seed alone.
///
/// A budget-matched random baseline has to actually be random, so the
/// policy seed is derived per rollout — the same thing a campaign does
/// (`derive_policy_seed`). The `degree_*` arms take the seed and ignore
/// it: they are deterministic given a snapshot, and that is the point of
/// them.
pub fn stage0a_arms(budget: Option<usize>) -> Vec<(String, ArmFactory)> {
    let b = budget;
    vec![
        (
            "random".into(),
            Box::new(move |seed: u64| {
                Box::new(RandomSubset::new(b, hg_cognition::derive_policy_seed(seed, 0, 0, 0, 0)))
                    as Box<dyn RuleSelectionPolicy>
            }),
        ),
        (
            "degree_high".into(),
            Box::new(move |_seed: u64| Box::new(DegreeGreedy::new(b, DegreeDirection::High)) as Box<dyn RuleSelectionPolicy>),
        ),
        (
            "degree_low".into(),
            Box::new(move |_seed: u64| Box::new(DegreeGreedy::new(b, DegreeDirection::Low)) as Box<dyn RuleSelectionPolicy>),
        ),
    ]
}

/// Run one arm over `seeds` and summarize.
pub fn run_arm(
    fixture: &Fixture,
    out_dir: &std::path::Path,
    seeds: &[u64],
    burn_in: usize,
    make: &(dyn Fn(u64) -> Box<dyn RuleSelectionPolicy> + Send + Sync),
) -> ArmStats {
    // Parallel over seeds using SCOPED OS THREADS, deliberately not rayon.
    //
    // `run_inner` builds its own `rayon::ThreadPool` and `install()`s the
    // whole step loop into it (`crates/hg-sim-core/src/runner.rs`). Driving
    // that from inside a rayon worker nests one pool inside another, and
    // measured, it serializes completely: a 1500-rollout sweep sat at 99.4%
    // of a single core for an hour. Plain threads cannot interact with
    // those inner pools, so each rollout gets a real core.
    struct Sample {
        f2: f64,
        target: f64,
        interest: f64,
        nodes: f64,
        aps: f64,
        cap_bound: u64,
        attempted: u64,
    }

    let one = |s: u64| -> Option<Sample> {
        let cfg = fixture.sim_config(out_dir, s);
        let m = run_rollout(RolloutRequest {
            config: &cfg,
            seed: s,
            policy: make(s),
            horizon_steps: fixture.steps,
            probe: None,
        })
        .ok()?;
        let fit = crate::fitness::evaluate(&m, burn_in).ok()?;
        let ho = crate::fitness::held_out(&m, burn_in, &fit);
        let cap = cfg.engine.max_pairings_per_step as f64;
        Some(Sample {
            f2: fit.f2,
            target: fit.f2_reward,
            interest: fit.f1_mean_interestingness,
            nodes: m.windows.last().map(|w| w.n_nodes as f64).unwrap_or(0.0),
            aps: ho.mean_active_persistent.unwrap_or(0.0),
            cap_bound: m.windows.iter().filter(|w| w.accepted_per_step >= cap).count() as u64,
            attempted: m.windows.iter().map(|w| w.attempted_matches as u64).sum(),
        })
    };

    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(seeds.len().max(1));
    let chunk = seeds.len().div_ceil(n_threads.max(1));
    let mut results: Vec<Option<Sample>> = Vec::with_capacity(seeds.len());
    std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .chunks(chunk.max(1))
            .map(|batch| scope.spawn(|| batch.iter().map(|&s| one(s)).collect::<Vec<_>>()))
            .collect();
        for h in handles {
            results.extend(h.join().expect("diagnostic worker panicked"));
        }
    });

    let failures = results.iter().filter(|r| r.is_none()).count();
    let ok: Vec<&Sample> = results.iter().flatten().collect();
    let f2: Vec<f64> = ok.iter().map(|s| s.f2).collect();
    let target: Vec<f64> = ok.iter().map(|s| s.target).collect();
    let interest: Vec<f64> = ok.iter().map(|s| s.interest).collect();
    let nodes: Vec<f64> = ok.iter().map(|s| s.nodes).collect();
    let aps: Vec<f64> = ok.iter().map(|s| s.aps).collect();
    let cap_bound: u64 = ok.iter().map(|s| s.cap_bound).sum();
    let attempted: u64 = ok.iter().map(|s| s.attempted).sum();

    let (f2_mean, f2_sd) = stats(&f2);
    let (i_mean, i_sd) = stats(&interest);
    let (a_mean, a_sd) = stats(&aps);
    ArmStats {
        n_seeds: f2.len(),
        f2_mean,
        f2_sd,
        target_fraction_mean: stats(&target).0,
        interestingness_mean: i_mean,
        interestingness_sd: i_sd,
        final_nodes_mean: stats(&nodes).0,
        active_persistent_mean: a_mean,
        active_persistent_sd: a_sd,
        pairings_cap_bound_windows: cap_bound,
        attempted_total: attempted,
        failures,
    }
}

#[derive(Clone, Debug)]
pub struct Stage0aCell {
    pub budget: Option<usize>,
    pub arms: Vec<(String, ArmStats)>,
}

impl Stage0aCell {
    pub fn random(&self) -> Option<&ArmStats> {
        self.arms.iter().find(|(n, _)| n == "random").map(|(_, s)| s)
    }

    /// Classify the largest heuristic-vs-random effect in this cell.
    ///
    /// Three outcomes, not two, because "no measurable sigma" and "no
    /// effect" are different findings and conflating them loses the more
    /// interesting one.
    ///
    /// ## Why a relative spread floor
    ///
    /// The first version guarded with `pooled <= 0.0` and it did not work.
    /// When all 20 seeds return an identical value, the sample standard
    /// deviation is not exactly zero — the mean is not bit-identical to the
    /// samples, so `sqrt(sum((x-m)^2)/(n-1))` lands around 1e-16. Dividing a
    /// real difference by that produced sigma values up to 2.9e16 and a
    /// spurious "exit criterion MET" on cells with no seed variance at all.
    /// Spread is therefore tested against a floor RELATIVE to the metric's
    /// own magnitude.
    /// Every heuristic × metric comparison in this cell, **signed**.
    ///
    /// `classify_effect` reports the largest effect by magnitude, which is
    /// the right statistic for "does selection have causal power" and the
    /// wrong one for everything downstream. R-1 (plan v2 §8) needs the
    /// sign: if the diversity-reward diagnosis is right, signed differences
    /// should be overwhelmingly **negative** — every non-uniform heuristic
    /// worse than budget-matched random, not merely different — and any
    /// positive cell is exploitable signal the diagnosis does not account
    /// for.
    ///
    /// Emitted for every arm and metric rather than only the best, because
    /// "the largest effect happened to be negative" and "every effect is
    /// negative" are different claims and only the second tests §5.1.
    pub fn signed_effects(&self) -> Vec<SignedEffect> {
        let Some(base) = self.random() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (name, s) in &self.arms {
            if name == "random" {
                continue;
            }
            for (metric, a, b, sa, sb) in metric_rows(s, base) {
                let delta = a - b;
                let scale = a.abs() + b.abs() + 1.0;
                if delta.abs() <= DELTA_EPS * scale {
                    out.push(SignedEffect {
                        arm: name.clone(),
                        metric,
                        delta: 0.0,
                        sigma: Some(0.0),
                        heuristic_mean: a,
                        random_mean: b,
                    });
                    continue;
                }
                let denom = match (has_spread(sa, a), has_spread(sb, b)) {
                    (true, true) => Some(((sa * sa + sb * sb) / 2.0).sqrt()),
                    (true, false) => Some(sa),
                    (false, true) => Some(sb),
                    (false, false) => None,
                };
                out.push(SignedEffect {
                    arm: name.clone(),
                    metric,
                    delta,
                    sigma: denom.map(|d| delta / d),
                    heuristic_mean: a,
                    random_mean: b,
                });
            }
        }
        out
    }

    pub fn classify_effect(&self) -> EffectClass {
        let Some(base) = self.random() else {
            return EffectClass::NoDifference;
        };
        let mut best_measurable: Option<(String, &'static str, f64, f64)> = None;
        let mut best_deterministic: Option<(String, &'static str, f64)> = None;

        for (name, s) in &self.arms {
            if name == "random" {
                continue;
            }
            for (metric, a, b, sa, sb) in metric_rows(s, base) {
                // SIGNED as of plan v2 R-1. Ranking below is by magnitude —
                // "which is the biggest effect" is unchanged — but the
                // value carried out says which way it went, because
                // "selection has causal power" and "this heuristic beats
                // random" are different claims and v1 only ever reported
                // the first.
                let signed = a - b;
                let delta = signed.abs();
                let scale = a.abs() + b.abs() + 1.0;
                if delta <= DELTA_EPS * scale {
                    continue; // the arms agree
                }
                // Denominator selection. Requiring BOTH arms to vary was
                // too strict and it cost real measurements: a `degree_*`
                // arm has no RNG and, on a constructed initial condition,
                // is a point mass by construction — but the budget-matched
                // random baseline is not, and "how many baseline sigmas
                // from the baseline mean does the heuristic sit" is a
                // perfectly well-defined one-sample z. It is also the
                // statistic the plan's criterion means in substance: a
                // heuristic shifts a metric > 2 sigma *versus budget-
                // matched random*, and the spread that matters is the
                // spread of the thing being beaten.
                let denom = match (has_spread(sa, a), has_spread(sb, b)) {
                    (true, true) => Some(((sa * sa + sb * sb) / 2.0).sqrt()),
                    (true, false) => Some(sa),
                    (false, true) => Some(sb),
                    (false, false) => None,
                };
                if let Some(pooled) = denom {
                    let sigma = signed / pooled;
                    if best_measurable
                        .as_ref()
                        .is_none_or(|(_, _, v, _): &(String, &str, f64, f64)| {
                            sigma.abs() > v.abs()
                        })
                    {
                        best_measurable = Some((name.clone(), metric, sigma, signed));
                    }
                } else if best_deterministic
                    .as_ref()
                    .is_none_or(|(_, _, v): &(String, &str, f64)| delta > v.abs())
                {
                    best_deterministic = Some((name.clone(), metric, signed));
                }
            }
        }

        // A measurable effect outranks a deterministic one only because it
        // is the form the plan's exit criterion is written in — NOT because
        // it is the larger finding. A heuristic that moves F2 from 1.000 to
        // 0.000 on every one of 20 seeds is a bigger causal effect than a
        // 4-sigma shift in a noisy metric; it just cannot be written as a
        // multiple of a standard deviation that is zero.
        if let Some((arm, metric, sigma, delta)) = best_measurable {
            return EffectClass::Measurable { arm, metric, sigma, delta };
        }
        if let Some((arm, metric, delta)) = best_deterministic {
            return EffectClass::Deterministic { arm, metric, delta };
        }
        EffectClass::NoDifference
    }
}

/// The three metrics every Stage 0a comparison is made on, as
/// `(name, heuristic mean, random mean, heuristic sd, random sd)`.
///
/// Factored out so `classify_effect` and `signed_effects` cannot drift
/// apart: two lists of the same three metrics is exactly the kind of
/// duplication that produces a signed table disagreeing with the
/// unsigned one it is supposed to refine.
fn metric_rows<'a>(
    s: &'a ArmStats,
    base: &'a ArmStats,
) -> [(&'static str, f64, f64, f64, f64); 3] {
    [
        ("f2", s.f2_mean, base.f2_mean, s.f2_sd, base.f2_sd),
        (
            "interestingness",
            s.interestingness_mean,
            base.interestingness_mean,
            s.interestingness_sd,
            base.interestingness_sd,
        ),
        (
            "active_persistent",
            s.active_persistent_mean,
            base.active_persistent_mean,
            s.active_persistent_sd,
            base.active_persistent_sd,
        ),
    ]
}

/// One signed heuristic-vs-random comparison (plan v2 R-1).
#[derive(Clone, Debug, PartialEq)]
pub struct SignedEffect {
    pub arm: String,
    pub metric: &'static str,
    /// `heuristic − random`. **Positive means the heuristic beat random.**
    pub delta: f64,
    /// `delta / pooled sd`. `None` when neither arm has spread, which is a
    /// real deterministic effect with no denominator — not a zero.
    pub sigma: Option<f64>,
    pub heuristic_mean: f64,
    pub random_mean: f64,
}

impl SignedEffect {
    /// Did the heuristic beat budget-matched random by more than `bar`
    /// sigmas? This is the R-1 question, and it is one-sided on purpose.
    pub fn beats_random(&self, bar: f64) -> bool {
        self.sigma.is_some_and(|s| s > bar)
    }
    /// Did it lose by more than `bar` sigmas?
    pub fn loses_to_random(&self, bar: f64) -> bool {
        self.sigma.is_some_and(|s| s < -bar)
    }
}

/// Relative floor below which a standard deviation is treated as zero.
pub const SPREAD_EPS: f64 = 1e-9;
/// Relative floor below which two means are treated as equal.
pub const DELTA_EPS: f64 = 1e-9;

fn has_spread(sd: f64, mean: f64) -> bool {
    sd > SPREAD_EPS * (mean.abs() + 1.0)
}

/// The outcome of one Stage 0a cell.
#[derive(Clone, Debug, PartialEq)]
pub enum EffectClass {
    /// Both arms vary across seeds, so a sigma is meaningful. This is the
    /// form Stage 0a's exit criterion is stated in.
    Measurable {
        arm: String,
        metric: &'static str,
        sigma: f64,
        delta: f64,
    },
    /// The arms differ, reproducibly, on every seed — and **neither** arm
    /// has any spread, so there is no denominator anywhere to divide by.
    /// A REAL effect that the exit criterion cannot express, not an absence
    /// of one.
    ///
    /// One-sided spread does *not* land here: if the budget-matched random
    /// baseline varies, a deterministic heuristic still gets a one-sample
    /// z against it, and that is reported as `Measurable`.
    Deterministic {
        arm: String,
        metric: &'static str,
        delta: f64,
    },
    /// No heuristic differs from budget-matched random on any metric.
    NoDifference,
}

impl EffectClass {
    pub fn label(&self) -> String {
        match self {
            Self::Measurable { arm, metric, sigma, delta } => format!(
                "MEASURABLE: {arm} on {metric}, delta {delta:+.3} = {sigma:+.2} sigma \
                 ({}){}",
                if *delta > 0.0 { "BEATS random" } else { "WORSE than random" },
                if sigma.abs() > 2.0 { "  <== EXIT MET" } else { "" }
            ),
            Self::Deterministic { arm, metric, delta } => format!(
                "DETERMINISTIC: {arm} on {metric}, delta {delta:+.3} ({}), seed sd = 0 \
                 (real and reproducible; sigma undefined)",
                if *delta > 0.0 { "BEATS random" } else { "WORSE than random" }
            ),
            Self::NoDifference => "NO DIFFERENCE: every arm matches random exactly".into(),
        }
    }

    /// Does this cell satisfy the plan's ">2 sigma" exit as written?
    ///
    /// Magnitude, not sign: Stage 0a's question is "does selection have
    /// causal power", for which a heuristic that is reliably *worse* than
    /// random is just as decisive. The sign matters downstream and is
    /// carried by [`Stage0aCell::signed_effects`] — v1 conflated the two
    /// and the exit cell turned out to be a heuristic losing
    /// (`growth_treadmill` k=1: random 0.147, `degree_high` 0.091).
    pub fn meets_exit(&self) -> bool {
        matches!(self, Self::Measurable { sigma, .. } if sigma.abs() > 2.0)
    }

    /// The signed effect size, where one is defined.
    pub fn signed_sigma(&self) -> Option<f64> {
        match self {
            Self::Measurable { sigma, .. } => Some(*sigma),
            _ => None,
        }
    }
}

pub fn stage0a(
    fixture: &Fixture,
    out_dir: &std::path::Path,
    seeds: &[u64],
    burn_in: usize,
) -> Vec<Stage0aCell> {
    stage0a_with_ladder(fixture, out_dir, seeds, burn_in, &BUDGET_LADDER)
}

/// [`stage0a`] over a caller-chosen budget ladder.
///
/// Exists so a sweep can omit the `k = ∞` rung. On a dense composed rule
/// set that rung is both the most expensive by orders of magnitude (it is
/// the arm that reached 84 600 nodes in 96 steps on `binary_edge`) and the
/// least informative: Stage 0a measured "no difference between any arms" at
/// `k = ∞` on all five single-rule sets, and `stage2::run_case` refuses an
/// unbounded budget outright. Sweeping it again on a rule set no campaign
/// can use is paying the worst cost for the one answer already known.
pub fn stage0a_with_ladder(
    fixture: &Fixture,
    out_dir: &std::path::Path,
    seeds: &[u64],
    burn_in: usize,
    ladder: &[Option<usize>],
) -> Vec<Stage0aCell> {
    ladder
        .iter()
        .map(|&budget| Stage0aCell {
            budget,
            arms: stage0a_arms(budget)
                .into_iter()
                .map(|(name, make)| {
                    let s = run_arm(fixture, out_dir, seeds, burn_in, make.as_ref());
                    (name, s)
                })
                .collect(),
        })
        .collect()
}

// ---------------------------------------------------------------------
// Stage 0b — separation / fading memory
// ---------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Viability {
    /// Perturbation dies: no separation, nothing to read out.
    Ordered,
    /// Perturbation grows to the between-seed baseline: no fading memory.
    Chaotic,
    /// Intermediate plateau: usable.
    Viable,
    /// The diagnostic could not be run (too few windows, no divergence
    /// baseline). Recorded as such rather than folded into `Ordered`.
    Undetermined,
}

/// Bucket count for Stage 0b's state vector. Matches Stage 1's readout
/// width so the two diagnostics coarse-grain the substrate identically —
/// "the state Stage 0b says is separating" and "the state Stage 1 tries to
/// decode" have to be the same object or neither result constrains the
/// other.
pub const SEPARATION_BUCKETS: usize = 16;

/// Floor on the *reference*, below which the classification is refused.
///
/// The classifier is a ratio against the independent-seed baseline, and a
/// ratio of two negligible numbers is arithmetic rather than measurement —
/// `binary_edge` under a budgeted policy produced plateau and baseline both
/// below 1e-5 and a ratio of 0.995, which would have been published as
/// "Chaotic" on the strength of the third significant figure of noise.
/// JSD is bounded in [0, 1], so this is 0.01% of the available range: below
/// it, two independent runs' coarse-grained states are indistinguishable
/// and there is no diverged reference to measure a perturbation against.
pub const REFERENCE_FLOOR: f64 = 1e-4;

/// Jensen-Shannon divergence, base 2, between two non-negative vectors.
/// Renormalized rather than assumed normalized: a window whose graph is
/// empty yields an all-zero probe row, and that must come back 0 rather
/// than NaN.
pub fn jsd_pub(p: &[f32], q: &[f32]) -> f64 {
    jsd(p, q)
}

fn jsd(p: &[f32], q: &[f32]) -> f64 {
    let (sp, sq) = (
        p.iter().map(|x| *x as f64).sum::<f64>(),
        q.iter().map(|x| *x as f64).sum::<f64>(),
    );
    if sp <= 0.0 || sq <= 0.0 {
        return 0.0;
    }
    let mut acc = 0.0;
    for (x, y) in p.iter().zip(q.iter()) {
        let (pi, qi) = (*x as f64 / sp, *y as f64 / sq);
        let m = 0.5 * (pi + qi);
        if pi > 0.0 {
            acc += 0.5 * pi * (pi / m).log2();
        }
        if qi > 0.0 {
            acc += 0.5 * qi * (qi / m).log2();
        }
    }
    acc.clamp(0.0, 1.0)
}

#[derive(Clone, Debug)]
pub struct SeparationResult {
    /// Per-window WL type-histogram JSD between the perturbed pair.
    pub trajectory: Vec<f64>,
    /// Mean JSD over the final third of the run.
    pub plateau: f64,
    /// Mean JSD between two INDEPENDENT seeds — the "fully diverged"
    /// reference. Without it, "the perturbation grew" has no scale.
    pub independent_baseline: f64,
    pub class: Viability,
}

/// JSD trajectory of a perturbation pair.
///
/// The perturbation is a **different simulation seed applied to the same
/// configuration** rather than a hand-edited initial edge. On these rule
/// sets the initial condition is generated deterministically from the
/// config, so there is no supported way to add one edge without forking the
/// generator — and a forked generator would be a different substrate, which
/// is precisely what a separation diagnostic must not vary.
///
/// ## The perturbation only exists if `policy` is stochastic
///
/// The seed reaches the run through two channels: the initial-condition
/// generator and the engine's RNG stream. `GliderRing`, `TriLattice` and
/// `PureCrystal` are *constructed*, so the first channel is constant; and
/// under `MaxParallel` with no property jitter the engine never draws from
/// the second. On those fixtures a `PassThrough` run is a bit-identical
/// function of the config and the diagnostic has nothing to measure — it
/// returns `Undetermined`, which is a statement about the harness, not the
/// substrate.
///
/// Passing a **budgeted stochastic policy** (`RandomSubset` at the rule
/// set's Stage 0a budget) restores a real perturbation: the seed then
/// decides *which* rewrites fire each step. That is also the regime Stages
/// 2–3 operate in, so it is the separation question those stages actually
/// depend on. Callers should run both arms; the contrast between them is
/// the finding.
pub fn stage0b(
    fixture: &Fixture,
    out_dir: &std::path::Path,
    base_seed: u64,
    policy: impl Fn(u64) -> Box<dyn RuleSelectionPolicy>,
) -> Result<SeparationResult, SimError> {
    stage0b_with_readout(fixture, out_dir, base_seed, policy, ReadoutSet::R1)
}

/// Number of edge labels r2's block 4 reserves.
///
/// A constant rather than a per-rule-set count because the readout's width
/// must not change when a new label first appears mid-run — a ragged probe
/// matrix is rejected by `kernel_held`'s SVD and by the memory-capacity
/// ridge, after the rollout has been paid for. Every rule set in the
/// battery uses single-digit labels, so 8 covers them with room.
pub const R2_LABEL_SLOTS: usize = 8;
/// r2's raw-WL-multiset depth (`docs/policy_config.md` §4B.3).
pub const R2_TOP_N_WL: usize = 16;

/// [`stage0b`] under a chosen readout (plan v2 Stage 0b′).
///
/// The separation verdict is a joint statement about the substrate and the
/// projection, so the readout is an experimental variable rather than an
/// implementation detail. r1/r2 disagreements are the finding, not noise
/// to be resolved by picking one (`docs/policy_config.md` §4B.5).
pub fn stage0b_with_readout(
    fixture: &Fixture,
    out_dir: &std::path::Path,
    base_seed: u64,
    policy: impl Fn(u64) -> Box<dyn RuleSelectionPolicy>,
    readout: ReadoutSet,
) -> Result<SeparationResult, SimError> {
    // Per-window STATE VECTORS, not per-window scalars.
    //
    // This used the evaluator's own `type_histogram_jsd` — a window-to-
    // window churn scalar — and took `|jsd_a[i] - jsd_b[i]|` as the
    // separation between two runs. That is not a distance between
    // trajectories: two runs in completely different parts of state space
    // can churn at the same rate and score 0, and two nearby runs can
    // differ in rate and score high. It showed: on three of five fixtures
    // the perturbed pair "separated" MORE than two independent seeds
    // (ratios 1.7, 3.3, 5.6), which is impossible for a real distance
    // against a fully-diverged reference and so condemned the normalizer.
    //
    // The state is the same bucketed WL type histogram Stage 1 reads out
    // (`wl_bucket_probe`), and the distance between two runs at a window is
    // the Jensen-Shannon divergence between their histograms. That is
    // bounded, symmetric, zero exactly when the coarse-grained states
    // agree, and it makes the independent-seed reference an upper bound the
    // way the classifier assumes.
    let run = |seed: u64| -> Result<Vec<Vec<f32>>, SimError> {
        let cfg = fixture.sim_config(out_dir, seed);
        let m = run_rollout(RolloutRequest {
            config: &cfg,
            seed,
            // The factory takes the seed because a stochastic policy is the
            // only perturbation channel on a constructed initial condition.
            // Built from a fixed seed instead, all three runs would share
            // one policy RNG stream and the diagnostic would report
            // `Undetermined` for the arm that exists to avoid exactly that.
            policy: policy(seed),
            horizon_steps: fixture.steps,
            probe: Some(Box::new(hg_cognition::readout::probe(
                readout,
                SEPARATION_BUCKETS,
                R2_TOP_N_WL,
                R2_LABEL_SLOTS,
            ))),
        })?;
        Ok(m.probe_rows)
    };

    let a = run(base_seed)?;
    let b = run(base_seed.wrapping_add(1))?;
    let far = run(base_seed.wrapping_add(7919))?;

    let n = a.len().min(b.len()).min(far.len());
    if n < 4 {
        return Ok(SeparationResult {
            trajectory: Vec::new(),
            plateau: 0.0,
            independent_baseline: 0.0,
            class: Viability::Undetermined,
        });
    }
    let trajectory: Vec<f64> = (0..n).map(|i| jsd(&a[i], &b[i])).collect();
    let tail = n - n / 3;
    let plateau = trajectory[tail..].iter().sum::<f64>() / (n - tail) as f64;
    let baseline_traj: Vec<f64> = (0..n).map(|i| jsd(&a[i], &far[i])).collect();
    let independent_baseline =
        baseline_traj[tail..].iter().sum::<f64>() / (n - tail) as f64;

    // Classification. The thresholds are relative to the independent
    // baseline, not absolute: JSD magnitudes differ by orders of magnitude
    // across rule sets and an absolute cut would classify the rule set
    // rather than its dynamics.
    let class = if independent_baseline <= REFERENCE_FLOOR {
        // Nothing separates even two unrelated seeds — so there is no
        // scale to measure the perturbation against, and no perturbation
        // either.
        //
        // This returned `Ordered`, and that was a conflation: "the
        // perturbation decayed" and "there was never a perturbation" are
        // different findings and only the first is a substrate property.
        // Under `MaxParallel` with no jitter the engine RNG is never
        // consulted, and three of the five Stage 0a initial conditions are
        // *constructed* rather than sampled, so a different simulation seed
        // produces a bit-identical run and this branch is the only one that
        // can fire. `Undetermined` is the honest label: the diagnostic did
        // not run, and the caller must supply a perturbation the substrate
        // actually exposes (a budgeted stochastic policy does).
        Viability::Undetermined
    } else if plateau < 0.05 * independent_baseline {
        Viability::Ordered
    } else if plateau > 0.8 * independent_baseline {
        Viability::Chaotic
    } else {
        Viability::Viable
    };

    Ok(SeparationResult {
        trajectory,
        plateau,
        independent_baseline,
        class,
    })
}

// ---------------------------------------------------------------------
// Stage 1 — reservoir characterization
// ---------------------------------------------------------------------

/// Bucketed WL type-histogram probe: the reservoir state vector.
///
/// Buckets rather than raw WL classes because the class set grows without
/// bound on a generative substrate, and a readout needs a **fixed-width**
/// state. `hash % buckets` is the same bucketing `feature_set = "v1"` uses
/// for its per-candidate WL block, so the two are reading the same
/// coarse-graining.
pub fn wl_bucket_probe(buckets: usize) -> impl FnMut(&GraphSnapshot<'_>, u64) -> Vec<f32> + Send {
    // Delegates to `hg_cognition::readout::r1` rather than reimplementing
    // it. The two must agree byte-for-byte or every published Stage 0a and
    // Stage 0b number moves, so there is exactly one implementation and
    // this is a name for it.
    move |snap, _window| hg_cognition::readout::r1(snap, buckets)
}

#[derive(Clone, Debug)]
pub struct ReservoirResult {
    pub mc: MemoryCapacity,
    pub esn_mc: MemoryCapacity,
    pub n_windows: usize,
    pub tape_len: usize,
    /// The readout's width, which is also the ESN's unit count. Recorded
    /// because `MC` is a joint measurement of substrate memory and readout
    /// sensitivity, and a ratio quoted without it is not interpretable.
    pub state_dim: usize,
}

/// Stage 1: inject a random bit tape through paired creation rules and ask
/// whether the state carries linearly decodable information about the input
/// history.
///
/// `rule_zero` / `rule_one` are the indices of the paired creation rules in
/// the loaded rule set.
#[allow(clippy::too_many_arguments)]
pub fn stage1(
    fixture: &Fixture,
    out_dir: &std::path::Path,
    seed: u64,
    rule_zero: usize,
    rule_one: usize,
    buckets: usize,
    max_delay: usize,
    // `budget`: ordinary-rewrite cap. Sets the token population's
    // equilibrium size and therefore the reservoir's memory horizon;
    // unthrottled, decay clears the population faster than one window and
    // there is nothing left to remember.
    budget: Option<usize>,
) -> Result<ReservoirResult, SimError> {
    stage1_with_readout(
        fixture, out_dir, seed, rule_zero, rule_one, buckets, max_delay, budget,
        ReadoutSet::R1,
    )
}

/// [`stage1`] under a chosen readout (plan v2 Stage 1′).
///
/// # The ESN baseline is matched to the PROBE, not to `buckets`
///
/// `MC` is a joint measurement of substrate memory **×** readout
/// sensitivity, so the comparison it is read against has to hold the second
/// factor fixed. The reservoir's state dimension is whatever the readout
/// emits — 16 under r1, `buckets + top_n + labels + 8` under r2 — and the
/// ESN is sized from the probe rows themselves rather than from `buckets`.
///
/// Under r1 those two numbers coincide, so this changes nothing about the
/// C0–C5 measurement; under r2 they differ by a factor of ~2.7 and using
/// `buckets` would compare a 43-dimensional substrate readout against a
/// 16-unit ESN.
#[allow(clippy::too_many_arguments)]
pub fn stage1_with_readout(
    fixture: &Fixture,
    out_dir: &std::path::Path,
    seed: u64,
    rule_zero: usize,
    rule_one: usize,
    buckets: usize,
    max_delay: usize,
    budget: Option<usize>,
    readout: ReadoutSet,
) -> Result<ReservoirResult, SimError> {
    let cfg = fixture.sim_config(out_dir, seed);

    // One symbol per metrics window, so probe rows and symbols are aligned
    // by construction rather than by arithmetic the caller has to redo.
    let n_symbols = (fixture.steps / fixture.window).max(1) as usize;
    let tape = random_tape(n_symbols, seed);

    // The driver gates the creation pair; the learned half of the stack is
    // a plain budget so ordinary rewrites are rate-limited. Neither half
    // sees the other's partition (`docs/policy_config.md` §3.3).
    let driver = StimulusDriver::new(rule_zero, rule_one, tape.clone(), fixture.window);
    let policy = PolicyStack::new(Box::new(driver), Box::new(RandomSubset::new(budget, seed)));
    let m = run_rollout(RolloutRequest {
        config: &cfg,
        seed,
        policy: Box::new(policy),
        horizon_steps: fixture.steps,
        probe: Some(Box::new(hg_cognition::readout::probe(
            readout,
            buckets,
            R2_TOP_N_WL,
            R2_LABEL_SLOTS,
        ))),
    })?;

    let states: Vec<Vec<f64>> = m
        .probe_rows
        .iter()
        .map(|r| r.iter().map(|x| *x as f64).collect())
        .collect();
    let inputs: Vec<f64> = (0..states.len())
        .map(|i| f64::from(tape[i % tape.len()]))
        .collect();

    let mc = memory_capacity(&states, &inputs, max_delay, DEFAULT_LAMBDA);
    // Baseline of MATCHED state dimension on the SAME tape — otherwise
    // "MC = 1.4" has nothing to be read against.
    //
    // Sized from the PROBE ROWS, not from `buckets`. Under r1 the two
    // agree (the readout is exactly `buckets` wide) so no C0-C5 number
    // moves; under r2 they differ by ~2.7x and using `buckets` would pit a
    // 43-dimensional substrate readout against a 16-unit ESN and call the
    // ratio a substrate property.
    let esn_dim = states.first().map(|r| r.len()).unwrap_or(buckets);
    let esn = crate::ridge::esn_states(&inputs, esn_dim, 0.9, seed);
    let esn_mc = memory_capacity(&esn, &inputs, max_delay, DEFAULT_LAMBDA);

    Ok(ReservoirResult {
        mc,
        esn_mc,
        n_windows: states.len(),
        tape_len: tape.len(),
        state_dim: esn_dim,
    })
}

pub fn random_tape(n: usize, seed: u64) -> Vec<bool> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0x7a9e_5c31_0d2b_4f68);
    (0..n).map(|_| rng.gen_bool(0.5)).collect()
}

/// The inert control: no policy at all. Used to check that a Stage 0a cell's
/// `k = ∞` arm really is the engine default.
///
/// Takes and ignores a seed so it can be handed to [`stage0b`] alongside
/// [`budgeted_random`]; that it ignores the seed is the whole point of the
/// contrast between the two arms.
pub fn passthrough(_seed: u64) -> Box<dyn RuleSelectionPolicy> {
    Box::new(PassThrough)
}

/// Stage 0b's perturbing arm: budget-matched random, seeded per run.
///
/// This is the only channel through which a simulation seed reaches a run
/// on a constructed initial condition under `MaxParallel` — see [`stage0b`].
pub fn budgeted_random(budget: usize) -> impl Fn(u64) -> Box<dyn RuleSelectionPolicy> {
    move |seed| {
        Box::new(RandomSubset::new(
            Some(budget),
            hg_cognition::derive_policy_seed(seed, 0, 0, 0, 0),
        )) as Box<dyn RuleSelectionPolicy>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arm(name: &str, f2: f64, f2_sd: f64, aps: f64, aps_sd: f64) -> (String, ArmStats) {
        (
            name.into(),
            ArmStats {
                n_seeds: 20,
                f2_mean: f2,
                f2_sd,
                active_persistent_mean: aps,
                active_persistent_sd: aps_sd,
                ..Default::default()
            },
        )
    }

    /// The regression this classifier exists for.
    ///
    /// Twenty identical samples do NOT produce sd = 0.0 exactly — the mean
    /// is not bit-identical to the samples, so the sample sd lands near
    /// 1e-16. The original `pooled <= 0.0` guard let that through and
    /// produced sigma values up to 2.9e16, reporting a spurious "exit
    /// criterion MET" on cells with no seed variance at all.
    #[test]
    fn float_noise_sd_is_not_mistaken_for_spread() {
        let cell = Stage0aCell {
            budget: Some(1),
            arms: vec![
                arm("random", 1.0, 4.4e-16, 3.43, 8.9e-16),
                arm("degree_high", 0.0, 0.0, 4.00, 0.0),
            ],
        };
        match cell.classify_effect() {
            EffectClass::Deterministic { arm, metric, delta } => {
                assert_eq!(arm, "degree_high");
                assert_eq!(metric, "f2");
                // NEGATIVE as of plan v2 R-1: the heuristic scores 0.0
                // against random's 1.0, so `heuristic - random = -1.0`.
                // Under the unsigned classifier this read `+1.0` and the
                // cell looked like a heuristic win.
                assert!((delta + 1.0).abs() < 1e-9, "delta was {delta}");
            }
            other => panic!("float-noise sd was read as real spread: {other:?}"),
        }
        assert!(
            !cell.classify_effect().meets_exit(),
            "a cell with no seed variance must never satisfy a sigma criterion"
        );
    }

    /// Genuine spread on both arms is still measured, with the sigma the
    /// plan's criterion is written in.
    #[test]
    fn real_spread_still_yields_a_sigma() {
        let cell = Stage0aCell {
            budget: Some(1),
            arms: vec![
                arm("random", 0.104, 0.004, 1.0, 0.0),
                arm("degree_high", 0.091, 0.001, 1.0, 0.0),
            ],
        };
        match cell.classify_effect() {
            EffectClass::Measurable { sigma, metric, .. } => {
                assert_eq!(metric, "f2");
                // Also negative, and this is the real `growth_treadmill`
                // k=1 shape: random 0.104, degree_high 0.091. v1 reported
                // it as +4.46 sigma and it is the cell Stage 0a's exit
                // criterion was met on.
                assert!((sigma + 4.46).abs() < 0.2, "sigma was {sigma}");
                assert!(sigma < 0.0, "the heuristic LOST; the sign now says so");
            }
            other => panic!("expected a measurable effect, got {other:?}"),
        }
    }

    /// A deterministic heuristic against a varying baseline is measurable.
    ///
    /// This is the common case once the baseline is seeded per rollout:
    /// `DegreeGreedy` has no RNG and a constructed initial condition never
    /// consults the engine's, so the heuristic arm is a point mass while
    /// `RandomSubset` is not. Requiring spread on both arms discarded
    /// thirteen cells' worth of measurable effects in the first sweep.
    #[test]
    fn a_point_mass_against_a_varying_baseline_is_measurable() {
        let cell = Stage0aCell {
            budget: Some(1),
            arms: vec![
                arm("random", 0.743, 0.204, 3.31, 0.544),
                arm("degree_high", 1.000, 0.0, 4.00, 0.0),
            ],
        };
        match cell.classify_effect() {
            EffectClass::Measurable { sigma, .. } => {
                // The larger of |1.000-0.743|/0.204 and |4.00-3.31|/0.544.
                assert!((sigma - 1.268).abs() < 0.01, "sigma was {sigma}");
            }
            other => panic!("one-sided spread must still yield a sigma: {other:?}"),
        }
    }

    #[test]
    fn identical_arms_are_no_difference() {
        let cell = Stage0aCell {
            budget: None,
            arms: vec![
                arm("random", -0.071, 0.018, 1.0, 0.0),
                arm("degree_high", -0.071, 0.018, 1.0, 0.0),
            ],
        };
        assert_eq!(cell.classify_effect(), EffectClass::NoDifference);
    }

    /// Zero variance is not zero effect: a deterministic difference must be
    /// reported as a real finding, not folded into "no difference".
    #[test]
    fn a_deterministic_difference_is_not_a_null() {
        let cell = Stage0aCell {
            budget: Some(1),
            arms: vec![
                arm("random", 1.0, 0.0, 1.0, 0.0),
                arm("degree_low", 0.0, 0.0, 4.0, 0.0),
            ],
        };
        assert!(matches!(
            cell.classify_effect(),
            EffectClass::Deterministic { .. }
        ));
        assert_ne!(cell.classify_effect(), EffectClass::NoDifference);
    }
}

// ---------------------------------------------------------------------
// Stage 0b" — the pooled statistic (plan v3 §9.1, D4 §2.5)
// ---------------------------------------------------------------------

/// Independent reference PAIRS averaged per draw, per D4 §2.5 clause 1.
///
/// The pairs are disjoint — no run appears in two of them — so the average
/// is over independent samples rather than over correlated re-uses of one
/// trajectory.
pub const REFERENCE_PAIRS: usize = 4;

/// Minimum base seeds ("triples") per rule set, per plan v3 §5.2.
pub const MIN_SEPARATION_DRAWS: usize = 5;

/// One base seed's contribution to the pooled statistic.
#[derive(Clone, Debug)]
pub struct SeparationDraw {
    pub base_seed: u64,
    pub plateau: f64,
    /// Mean over [`REFERENCE_PAIRS`] disjoint independent pairs.
    pub reference: f64,
    /// Each independent pair's own plateau, kept so the *reference's* own
    /// spread is inspectable. D4 §2.3 diagnosed the defect as the reference
    /// being a single unstable sample; hiding its spread behind a mean
    /// would remove the evidence for that diagnosis.
    pub reference_samples: Vec<f64>,
    pub ratio: f64,
    pub class: Viability,
}

/// The pooled verdict for one (rule set, readout).
#[derive(Clone, Debug)]
pub struct PooledSeparation {
    pub draws: Vec<SeparationDraw>,
    /// `Some(class)` only when every draw agrees. Plan v3 §9.1: classes
    /// that span two categories are reported as `Undetermined` **with the
    /// distribution**, never resolved by fiat.
    pub class: Viability,
    pub unanimous: bool,
    /// Draws whose ratio exceeded 1 — the hard bound. A perturbed pair
    /// cannot diverge further than two independent seeds, so this counts
    /// impossibilities, not extreme values (D4 §2.5 clause 3).
    pub hard_bound_violations: usize,
    pub ratio_median: f64,
    pub ratio_min: f64,
    pub ratio_max: f64,
}

impl PooledSeparation {
    /// The line a report prints. Deliberately carries the spread and the
    /// violation count in every case, including the unanimous one: a class
    /// that is unanimous over five draws and a class that is unanimous over
    /// five draws *with two hard-bound violations* are different findings.
    pub fn detail(&self) -> String {
        let mut counts: Vec<(Viability, usize)> = Vec::new();
        for d in &self.draws {
            match counts.iter_mut().find(|(c, _)| *c == d.class) {
                Some((_, n)) => *n += 1,
                None => counts.push((d.class, 1)),
            }
        }
        let dist = counts
            .iter()
            .map(|(c, n)| format!("{c:?}x{n}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "{:?} over {} draws [{}]; ratio median {:.3} (min {:.3}, max {:.3}); \
             hard-bound violations {}/{}",
            self.class,
            self.draws.len(),
            dist,
            self.ratio_median,
            self.ratio_min,
            self.ratio_max,
            self.hard_bound_violations,
            self.draws.len()
        )
    }
}

/// Stage 0b", the redesign D4 §2.5 specified and left unbuilt.
///
/// Three changes from [`stage0b_with_readout`], and none of them is a
/// readout change — D4 §2.3's diagnosis was that the **denominator** was a
/// single unstable sample, not that the projection was wrong:
///
/// 1. The independent-seed reference is averaged over [`REFERENCE_PAIRS`]
///    **disjoint** pairs, so it is an estimate rather than one draw.
/// 2. The whole triple is repeated over `base_seeds` (≥ [`MIN_SEPARATION_DRAWS`]),
///    and the ratio is reported with its spread across them.
/// 3. The hard bound stays live: a ratio above 1 is counted as a violation
///    rather than silently labelled `Chaotic`.
///
/// A class is returned only when every draw agrees. Otherwise the result is
/// `Undetermined` **with the distribution attached** — plan v3 §9.1 is
/// explicit that a distribution-spanning class is not to be resolved by
/// fiat, and D4 §2.2 found `fl1_two_rule` at Chaotic×5 / Viable×5 under the
/// old statistic, which is a coin flip dressed as a verdict.
pub fn stage0b_pooled(
    fixture: &Fixture,
    out_dir: &std::path::Path,
    base_seeds: &[u64],
    policy: &(impl Fn(u64) -> Box<dyn RuleSelectionPolicy> + ?Sized),
    readout: ReadoutSet,
) -> Result<PooledSeparation, SimError> {
    let run = |seed: u64| -> Result<Vec<Vec<f32>>, SimError> {
        let cfg = fixture.sim_config(out_dir, seed);
        let m = run_rollout(RolloutRequest {
            config: &cfg,
            seed,
            policy: policy(seed),
            horizon_steps: fixture.steps,
            probe: Some(Box::new(hg_cognition::readout::probe(
                readout,
                SEPARATION_BUCKETS,
                R2_TOP_N_WL,
                R2_LABEL_SLOTS,
            ))),
        })?;
        Ok(m.probe_rows)
    };
    // Tail-mean JSD between two probe streams, over their common length.
    let tail_jsd = |x: &[Vec<f32>], y: &[Vec<f32>]| -> Option<f64> {
        let n = x.len().min(y.len());
        if n < 4 {
            return None;
        }
        let tail = n - n / 3;
        let s: f64 = (tail..n).map(|i| jsd(&x[i], &y[i])).sum();
        Some(s / (n - tail) as f64)
    };

    let mut draws = Vec::new();
    for &base in base_seeds {
        let a = run(base)?;
        let b = run(base.wrapping_add(1))?;
        let Some(plateau) = tail_jsd(&a, &b) else {
            continue;
        };

        // Disjoint independent pairs. The stride is chosen so no seed here
        // collides with `base` or `base+1`, and so the pairs share no run
        // with each other — averaging over pairs that share a trajectory
        // would re-introduce the correlation the fix exists to remove.
        let mut reference_samples = Vec::new();
        for p in 0..REFERENCE_PAIRS {
            let s1 = base.wrapping_add(7919).wrapping_add(2 * p as u64 * 7919);
            let s2 = s1.wrapping_add(7919);
            let (u, v) = (run(s1)?, run(s2)?);
            if let Some(r) = tail_jsd(&u, &v) {
                reference_samples.push(r);
            }
        }
        if reference_samples.is_empty() {
            continue;
        }
        let reference =
            reference_samples.iter().sum::<f64>() / reference_samples.len() as f64;

        let (ratio, class) = if reference <= REFERENCE_FLOOR {
            (f64::NAN, Viability::Undetermined)
        } else {
            let r = plateau / reference;
            let c = if r < 0.05 {
                Viability::Ordered
            } else if r > 0.8 {
                Viability::Chaotic
            } else {
                Viability::Viable
            };
            (r, c)
        };
        draws.push(SeparationDraw {
            base_seed: base,
            plateau,
            reference,
            reference_samples,
            ratio,
            class,
        });
    }

    let finite: Vec<f64> = draws.iter().map(|d| d.ratio).filter(|r| r.is_finite()).collect();
    let mut sorted = finite.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if sorted.is_empty() {
        f64::NAN
    } else {
        sorted[sorted.len() / 2]
    };
    let unanimous = !draws.is_empty() && draws.iter().all(|d| d.class == draws[0].class);
    Ok(PooledSeparation {
        class: if unanimous { draws[0].class } else { Viability::Undetermined },
        unanimous,
        hard_bound_violations: finite.iter().filter(|r| **r > 1.0).count(),
        ratio_median: median,
        ratio_min: sorted.first().copied().unwrap_or(f64::NAN),
        ratio_max: sorted.last().copied().unwrap_or(f64::NAN),
        draws,
    })
}
