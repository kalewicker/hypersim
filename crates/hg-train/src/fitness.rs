//! Fitness functions, held-out metrics, engagement counters, and Goodhart
//! tripwires.
//!
//! Implements `docs/fitness_formulas.md` v2. Every constant here appears in
//! that document; nothing here may introduce one that does not. These are
//! pure functions of a `RolloutMetrics` — fitness is computed *after* the
//! rollout, never during it, so no fitness definition can reach back into
//! the physics.
//!
//! v2 adds F4 (predictive information, §4A) and F5 (constructor reuse,
//! §4B), the engagement counters of `docs/instrument_registry.md` §2, and
//! [`Reported`] — the type distinction between a measured `0.0` and an
//! `unmeasured` whose instrument never engaged.

use hg_schema::{MetricsRow, Regime};
use hg_sim_core::RolloutMetrics;
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// `docs/fitness_formulas.md` version stamped into every campaign artifact.
pub const FITNESS_FORMULAS_VERSION: &str = "v2";

/// F2's weight on the penalized-regime fraction (§3).
pub const LAMBDA_PENALTY: f64 = 0.5;
/// `DEGENERATE_OCCUPANCY`'s collapse threshold (§6.3), shared with
/// `DEGENERATE_REUSE` (§6.5) so neither tripwire needs a per-rule-set
/// constant.
pub const COLLAPSE_FRACTION: f64 = 0.25;
/// `FITNESS_STRUCTURE_DIVERGENCE`'s trailing window, in generations (§6.1).
pub const DIVERGENCE_GENS: usize = 10;
/// `FITNESS_STRUCTURE_DIVERGENCE`'s relative-improvement bar (§6.1).
pub const DIVERGENCE_RATIO: f64 = 0.25;
/// Singular-value cutoff for the kernel-held rank (Appendix A).
pub const EPS_RANK: f64 = 1e-6;

/// F4's symbol count `Q` (§4A.2). F4's ceiling is `log2(Q)`.
pub const PI_ALPHABET: usize = 4;
/// F4's permutation-null replicate count (§4A.3).
pub const PI_SHUFFLES: usize = 64;
/// Below this many consecutive symbol pairs, F4 is `unmeasured` (§4A.5).
///
/// The plug-in mutual-information estimator is biased upward by roughly
/// `(K-1) / (2N ln 2)` bits. At `Q = 4` that bias is comparable to the
/// signal for small `N`, and an estimator whose sample cannot support an
/// estimate must not emit a number.
pub const PI_MIN_PAIRS: usize = 50;
/// `DEGENERATE_REUSE`'s concentration bar (§6.5).
pub const TOP_SHARE_DEGENERATE: f64 = 0.9;

pub const TARGET_REGIMES: [Regime; 2] =
    [Regime::ActivePersistentStructure, Regime::GenerativeCandidate];
pub const PENALIZED_REGIMES: [Regime; 3] =
    [Regime::Exploding, Regime::WhiteNoiseSoup, Regime::Frozen];

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitnessFn {
    F1Interestingness,
    F2RegimeOccupancy,
    F3ConstructorYield,
    /// v2, §4A.
    F4PredictiveInformation,
    /// v2, §4B.
    F5ConstructorReuse,
}

impl FitnessFn {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::F1Interestingness => "f1_interestingness",
            Self::F2RegimeOccupancy => "f2_regime_occupancy",
            Self::F3ConstructorYield => "f3_constructor_yield",
            Self::F4PredictiveInformation => "f4_predictive_information",
            Self::F5ConstructorReuse => "f5_constructor_reuse",
        }
    }
}

/// A measurement that may not have been made.
///
/// `docs/campaign_output.md` §4.3 and `docs/instrument_registry.md` §1: an
/// instrument whose engagement counter read zero reports `Unmeasured`, and
/// that is a *different report state* from a measured `0.0`. Six of the
/// eleven C0–C5 instrument bugs produced a plausible zero from an
/// instrument that never ran; this type is what makes that impossible to
/// write down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reported {
    Value(f64),
    Unmeasured,
}

impl Reported {
    pub fn value(self) -> Option<f64> {
        match self {
            Self::Value(v) => Some(v),
            Self::Unmeasured => None,
        }
    }

    pub fn is_unmeasured(self) -> bool {
        matches!(self, Self::Unmeasured)
    }

    /// CSV / report rendering. Deliberately not `Display for Option<f64>`:
    /// the literal string is part of the schema, not a formatting choice.
    pub fn render(self) -> String {
        match self {
            Self::Value(v) => format!("{v}"),
            Self::Unmeasured => UNMEASURED.to_string(),
        }
    }
}

/// The literal cell contents for an unmeasured value. Part of the schema
/// (`docs/campaign_output.md` §4.3): a reader must never have to infer
/// whether a blank means "unmeasured" or "not applicable".
pub const UNMEASURED: &str = "unmeasured";

impl serde::Serialize for Reported {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Value(v) => s.serialize_f64(*v),
            Self::Unmeasured => s.serialize_str(UNMEASURED),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Reported {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = Reported;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a float or the literal string \"{UNMEASURED}\"")
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Reported, E> {
                Ok(Reported::Value(v))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Reported, E> {
                if v == UNMEASURED {
                    return Ok(Reported::Unmeasured);
                }
                v.parse::<f64>().map(Reported::Value).map_err(|_| {
                    E::custom(format!("expected a float or \"{UNMEASURED}\", got {v:?}"))
                })
            }
        }
        d.deserialize_any(V)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitnessError {
    /// Burn-in consumed every window, or the run produced none.
    ///
    /// Deliberately an error rather than a zero: scoring an empty window set
    /// as 0.0 would make "the policy killed the run instantly" look
    /// identical to "the policy performed badly", and the first is a bug
    /// signal (§1.1).
    NoScoredWindows,
    /// The selected fitness function's instrument did not engage.
    ///
    /// Distinct from `NoScoredWindows`: the rollout is valid and other
    /// fitness functions may be perfectly measurable on it. This one is not.
    Unmeasured(FitnessFn),
}

impl std::fmt::Display for FitnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoScoredWindows => write!(
                f,
                "no windows survived burn-in — the rollout is invalid, not zero-fitness"
            ),
            Self::Unmeasured(which) => write!(
                f,
                "{} is unmeasured on this rollout: its engagement counter read zero \
                 (docs/instrument_registry.md §2) — this is not a score of 0.0",
                which.as_str()
            ),
        }
    }
}

impl std::error::Error for FitnessError {}

/// How many times each instrument actually did work on one rollout.
///
/// `docs/instrument_registry.md` §1, Rule 1: a zero counter emits
/// `INSTRUMENT_NOT_ENGAGED` and the value it gates is written `unmeasured`,
/// never `0.0`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Engagement {
    /// EI intervals actually computed within the horizon. Zero is bug 7:
    /// `ei_interval_steps` defaulting to 1000 against a 384-step horizon.
    pub ei_intervals_emitted: usize,
    /// Task-lattice entries written. Zero means `[causal]` never ran.
    pub task_lattice_entries: usize,
    /// Composable A→B / B→C chains found. Zero makes composition rate
    /// `undefined`, not `0.0` — nothing was ever asked to compose.
    pub task_lattice_chains: usize,
    /// Constructor records examined, *before* the confirmed filter. Zero is
    /// bug 8: `[causal]` off makes F3 identically 0.0 for every arm.
    pub constructor_records_examined: usize,
    /// Same count, named separately because it gates F5 (§4B.2).
    pub f5_records_examined: usize,
    /// Consecutive symbol pairs available to F4 (§4A.5).
    pub f4_pairs: usize,
    /// Windows where the evaluator emitted an interestingness score.
    pub f1_windows_scored: usize,
    /// Probe rows collected over scored windows; gates `kernel_held`.
    pub probe_rows_collected: usize,
}

impl Engagement {
    /// The instruments that did not engage, by counter name. Empty is the
    /// healthy case. Used for `flag_instrument_not_engaged` and for the
    /// campaign report's demotion list.
    pub fn not_engaged(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        for (n, c) in [
            ("ei_intervals_emitted", self.ei_intervals_emitted),
            ("task_lattice_entries", self.task_lattice_entries),
            ("constructor_records_examined", self.constructor_records_examined),
            ("f4_pairs", self.f4_pairs),
            ("f1_windows_scored", self.f1_windows_scored),
        ] {
            if c == 0 {
                out.push(n);
            }
        }
        out
    }
}

/// Every fitness quantity for one rollout. All three functions are computed
/// on every rollout regardless of which is being optimized: they cost
/// nothing beyond the rollout that already happened, and having them means a
/// campaign under F2 can be read against F1 and F3 afterwards without paying
/// for the campaign twice (`docs/campaign_output.md` §3.1).
#[derive(Clone, Debug, PartialEq)]
pub struct FitnessBreakdown {
    pub f1_mean_interestingness: f64,
    pub f2_reward: f64,
    pub f2_penalty: f64,
    pub f2: f64,
    pub f3_constructor_yield: f64,
    pub n_confirmed_constructors: usize,
    pub mean_fidelity: f64,
    pub n_windows_scored: usize,
    /// v2 §4A. `Unmeasured` when `f4_pairs < PI_MIN_PAIRS`.
    pub f4_predictive_information: Reported,
    /// σ of the F4 permutation null. Zero alongside a non-constant symbol
    /// sequence is an instrument fault (`instrument_registry.md` §2.3).
    pub f4_null_sd: f64,
    /// Windows joined across an evaluator gap when building the symbol
    /// sequence (§4A.2) — recorded so the adjacency error is visible.
    pub f4_gap_joins: usize,
    /// Fewer than two distinct symbols: F4 is a *measured* exact zero and
    /// the mechanism predicts it (§4A.5).
    pub f4_degenerate_alphabet: bool,
    /// v2 §4B. `Unmeasured` when the causal layer never ran.
    pub f5_constructor_reuse: Reported,
    /// `max_i cycles_i / T` — F5's concentration, held out (§5).
    pub f5_top_share: f64,
    /// `max_i cycles_i`. F5's hard predicted-range ceiling (§4B.3).
    pub f5_max_cycles: u32,
    pub engagement: Engagement,
}

impl FitnessBreakdown {
    /// The active fitness function's value.
    ///
    /// Returns `Err(Unmeasured)` rather than a number when the selected
    /// function's instrument did not engage. Callers must not paper over
    /// this with `unwrap_or(0.0)` — that is precisely bug 8.
    pub fn select(&self, which: FitnessFn) -> Result<f64, FitnessError> {
        match which {
            FitnessFn::F1Interestingness => Ok(self.f1_mean_interestingness),
            FitnessFn::F2RegimeOccupancy => Ok(self.f2),
            FitnessFn::F3ConstructorYield => {
                if self.engagement.constructor_records_examined == 0 {
                    Err(FitnessError::Unmeasured(which))
                } else {
                    Ok(self.f3_constructor_yield)
                }
            }
            FitnessFn::F4PredictiveInformation => self
                .f4_predictive_information
                .value()
                .ok_or(FitnessError::Unmeasured(which)),
            FitnessFn::F5ConstructorReuse => self
                .f5_constructor_reuse
                .value()
                .ok_or(FitnessError::Unmeasured(which)),
        }
    }
}

// ---------------------------------------------------------------------------
// F4 — predictive information (§4A)
// ---------------------------------------------------------------------------

/// Plug-in Shannon entropy over a count histogram, in bits. `0·log2 0 = 0`.
fn entropy_bits(counts: &[usize]) -> f64 {
    let n: usize = counts.iter().sum();
    if n == 0 {
        return 0.0;
    }
    let n = n as f64;
    counts
        .iter()
        .filter(|c| **c > 0)
        .map(|c| {
            let p = *c as f64 / n;
            -p * p.log2()
        })
        .sum()
}

/// Lag-1 plug-in mutual information of a symbol sequence, in bits.
///
/// `I = H(first coords) + H(second coords) − H(joint)` over the `N = len−1`
/// consecutive pairs. Both marginals are computed from the *pairs*, not from
/// the full sequence, so the identity holds exactly on the sample.
fn lag1_mi(seq: &[usize], alphabet: usize) -> f64 {
    if seq.len() < 2 {
        return 0.0;
    }
    let mut first = vec![0usize; alphabet];
    let mut second = vec![0usize; alphabet];
    let mut joint = vec![0usize; alphabet * alphabet];
    for w in seq.windows(2) {
        first[w[0]] += 1;
        second[w[1]] += 1;
        joint[w[0] * alphabet + w[1]] += 1;
    }
    entropy_bits(&first) + entropy_bits(&second) - entropy_bits(&joint)
}

/// F4's symbolization and shuffle-corrected statistic (§4A.2–§4A.3).
///
/// `seed` is the rollout's `final_canonical_hash`, so the permutation null
/// is reproducible from the run itself rather than from ambient RNG state —
/// `thread_rng` is banned in this crate for exactly this reason.
fn predictive_information(rows: &[MetricsRow], seed: u64) -> (Reported, f64, usize, usize, bool) {
    // Symbolize. Windows where the evaluator emitted no JSD are dropped
    // before pairing; the join across a gap is counted, not hidden.
    let mut seq = Vec::with_capacity(rows.len());
    let mut gap_joins = 0usize;
    let mut had_gap = false;
    for r in rows {
        match r.type_histogram_jsd {
            Some(j) => {
                if had_gap && !seq.is_empty() {
                    gap_joins += 1;
                }
                had_gap = false;
                let q = (PI_ALPHABET as f64 * j).floor() as isize;
                seq.push(q.clamp(0, PI_ALPHABET as isize - 1) as usize);
            }
            None => had_gap = true,
        }
    }

    let pairs = seq.len().saturating_sub(1);
    let distinct = {
        let mut seen = [false; PI_ALPHABET];
        for s in &seq {
            seen[*s] = true;
        }
        seen.iter().filter(|b| **b).count()
    };

    // A constant symbol sequence is the frozen case: F4 is exactly 0 and
    // the mechanism predicts it, so it is *measured*, not unmeasured — but
    // only once there were enough pairs to have looked.
    if pairs < PI_MIN_PAIRS {
        return (Reported::Unmeasured, 0.0, pairs, gap_joins, distinct < 2);
    }
    if distinct < 2 {
        return (Reported::Value(0.0), 0.0, pairs, gap_joins, true);
    }

    let observed = lag1_mi(&seq, PI_ALPHABET);

    let mut nulls = Vec::with_capacity(PI_SHUFFLES);
    let mut buf = seq.clone();
    for r in 0..PI_SHUFFLES {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(
            seed ^ (r as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
        );
        buf.copy_from_slice(&seq);
        buf.shuffle(&mut rng);
        nulls.push(lag1_mi(&buf, PI_ALPHABET));
    }
    let mu = nulls.iter().sum::<f64>() / nulls.len() as f64;
    let var = nulls.iter().map(|v| (v - mu).powi(2)).sum::<f64>() / (nulls.len() - 1) as f64;

    (
        Reported::Value((observed - mu).max(0.0)),
        var.sqrt(),
        pairs,
        gap_joins,
        false,
    )
}

// ---------------------------------------------------------------------------
// F5 — constructor reuse (§4B)
// ---------------------------------------------------------------------------

/// `(F5, top share, max cycles)`.
///
/// `Unmeasured` when no constructor record was examined at all — the causal
/// layer never ran, which is bug 8 and is *not* the same fact as "the causal
/// layer looked and found no confirmed constructor" (a genuine `0.0`).
fn constructor_reuse(m: &RolloutMetrics) -> (Reported, f64, u32) {
    if m.constructors.is_empty() {
        return (Reported::Unmeasured, 0.0, 0);
    }
    let live: Vec<(f64, f64)> = m
        .constructors
        .iter()
        .filter(|c| c.confirmed_cycles > 0)
        .map(|c| (c.fidelity as f64, c.confirmed_cycles as f64))
        .collect();
    if live.is_empty() {
        return (Reported::Value(0.0), 0.0, 0);
    }
    let total: f64 = live.iter().map(|(_, c)| c).sum();
    let num: f64 = live.iter().map(|(f, c)| f * c * c).sum();
    let max_cycles = live.iter().map(|(_, c)| *c).fold(0.0f64, f64::max);
    (
        Reported::Value(num / total.max(1.0)),
        max_cycles / total,
        max_cycles as u32,
    )
}

fn scored(m: &RolloutMetrics, burn_in: usize) -> &[MetricsRow] {
    m.windows.get(burn_in..).unwrap_or(&[])
}

pub fn evaluate(m: &RolloutMetrics, burn_in: usize) -> Result<FitnessBreakdown, FitnessError> {
    let windows = scored(m, burn_in);
    if windows.is_empty() {
        return Err(FitnessError::NoScoredWindows);
    }
    let w = windows.len() as f64;

    // --- F1: time mean of the evaluator's own interestingness (§2).
    // Windows where the evaluator emitted nothing are excluded from both
    // the sum and the denominator, for F1 only.
    let scores: Vec<f64> = windows.iter().filter_map(|r| r.interestingness_score).collect();
    if scores.is_empty() {
        return Err(FitnessError::NoScoredWindows);
    }
    let f1 = scores.iter().sum::<f64>() / scores.len() as f64;

    // --- F2: target-regime occupancy minus a penalty (§3).
    // Counted over the SCORED windows only, so `regime_occupancy` (which
    // covers the whole run, burn-in included) cannot be used directly.
    let mut target = 0usize;
    let mut penalized = 0usize;
    for r in windows {
        let Some(name) = r.regime.as_deref() else { continue };
        if TARGET_REGIMES.iter().any(|t| t.as_str() == name) {
            target += 1;
        } else if PENALIZED_REGIMES.iter().any(|p| p.as_str() == name) {
            penalized += 1;
        }
    }
    let f2_reward = target as f64 / w;
    let f2_penalty = LAMBDA_PENALTY * (penalized as f64 / w);

    // --- F3: constructor yield (§4). A genuine zero when nothing is
    // confirmed, unlike §1.1's missing-value case.
    let confirmed: Vec<f64> = m
        .constructors
        .iter()
        .filter(|c| c.confirmed_cycles > 0)
        .map(|c| c.fidelity as f64)
        .collect();
    let n = confirmed.len();
    let mean_fidelity = if n == 0 {
        0.0
    } else {
        confirmed.iter().sum::<f64>() / n as f64
    };

    // --- F4: shuffle-corrected lag-1 mutual information (§4A).
    let (f4, f4_null_sd, f4_pairs, f4_gap_joins, f4_degenerate_alphabet) =
        predictive_information(windows, m.final_canonical_hash);

    // --- F5: reuse depth (§4B).
    let (f5, f5_top_share, f5_max_cycles) = constructor_reuse(m);

    let engagement = Engagement {
        ei_intervals_emitted: windows.iter().filter(|r| r.ei_micro.is_some()).count(),
        task_lattice_entries: m.tasks.len(),
        task_lattice_chains: composable_chains(&m.tasks),
        constructor_records_examined: m.constructors.len(),
        f5_records_examined: m.constructors.len(),
        f4_pairs,
        f1_windows_scored: scores.len(),
        probe_rows_collected: m.probe_rows.len().saturating_sub(burn_in),
    };

    Ok(FitnessBreakdown {
        f1_mean_interestingness: f1,
        f2_reward,
        f2_penalty,
        f2: f2_reward - f2_penalty,
        f3_constructor_yield: n as f64 * mean_fidelity,
        n_confirmed_constructors: n,
        mean_fidelity,
        n_windows_scored: windows.len(),
        f4_predictive_information: f4,
        f4_null_sd,
        f4_gap_joins,
        f4_degenerate_alphabet,
        f5_constructor_reuse: f5,
        f5_top_share,
        f5_max_cycles,
        engagement,
    })
}

/// Composable A→B / B→C chains over the *distinct* (input, output) task
/// set, excluding self-loops.
///
/// This is the denominator of Stage 4's composition rate, lifted out as its
/// own counter because a lattice with tasks but no chain has nothing to
/// compose: the rate is then `undefined`, not `0.0`
/// (`docs/instrument_registry.md` §2.1).
pub fn composable_chains(tasks: &[hg_sim_core::TaskSummary]) -> usize {
    let mut pairs: Vec<(u64, u64)> = tasks.iter().map(|t| (t.input_wl, t.output_wl)).collect();
    pairs.sort_unstable();
    pairs.dedup();
    let mut n = 0usize;
    for (a, b) in &pairs {
        for (c, d) in &pairs {
            if b == c && a != d {
                n += 1;
            }
        }
    }
    n
}

/// Recorded every rollout, optimized by nothing (§5).
///
/// Optimizing any of these is a version bump and a rename, not a config
/// change. The pairing `mean_active_persistent` / `mean_static_persistent`
/// is the sharpest: a policy achieving target-regime occupancy by producing
/// *static* repeated motifs rather than active persistent structure has
/// gamed the classifier, and the two columns show it side by side.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeldOut {
    pub growth_exponent_nodes: Option<f64>,
    pub growth_exponent_edges: Option<f64>,
    pub mean_type_entropy: Option<f64>,
    pub mean_type_jsd: Option<f64>,
    pub mean_mean_degree: Option<f64>,
    pub mean_active_persistent: Option<f64>,
    pub mean_static_persistent: Option<f64>,
    pub archive_admission_rate: Option<f64>,
    pub mean_n_nodes: f64,
    pub replicator_rate: f64,
    /// Appendix A. `None` when the rollout carried no probe.
    pub kernel_held: Option<f64>,
    /// v2 §5. F5's concentration made visible: `1.0` means a single
    /// constructor owns every confirmed cycle, F5's degenerate optimum.
    pub f5_top_share: f64,
    /// v2 §5. The F4 permutation null's spread. A small value means the F4
    /// margin has no null to be measured against.
    pub f4_null_sd: f64,
}

fn mean_of<F: Fn(&MetricsRow) -> Option<f64>>(rows: &[MetricsRow], f: F) -> Option<f64> {
    let vals: Vec<f64> = rows.iter().filter_map(&f).collect();
    (!vals.is_empty()).then(|| vals.iter().sum::<f64>() / vals.len() as f64)
}

/// OLS slope of `log y` against `log step`. `None` when fewer than two
/// usable points, or when the series is degenerate in x.
fn log_log_slope(rows: &[MetricsRow], y: impl Fn(&MetricsRow) -> usize) -> Option<f64> {
    let pts: Vec<(f64, f64)> = rows
        .iter()
        .filter_map(|r| {
            let v = y(r);
            (r.step_end > 0 && v > 0).then(|| ((r.step_end as f64).ln(), (v as f64).ln()))
        })
        .collect();
    if pts.len() < 2 {
        return None;
    }
    let n = pts.len() as f64;
    let mx = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx: f64 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
    let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    (sxx > 0.0).then(|| sxy / sxx)
}

/// `fit` supplies the two v2 columns that are byproducts of the fitness
/// pass (§4A, §4B) rather than of the window sweep — recomputing them here
/// would run F4's 64-replicate permutation null a second time for nothing.
pub fn held_out(m: &RolloutMetrics, burn_in: usize, fit: &FitnessBreakdown) -> HeldOut {
    let rows = scored(m, burn_in);
    if rows.is_empty() {
        return HeldOut::default();
    }
    let w = rows.len() as f64;

    let n_repl = m.constructors.iter().filter(|c| c.is_replicator).count();
    HeldOut {
        growth_exponent_nodes: log_log_slope(rows, |r| r.n_nodes),
        growth_exponent_edges: log_log_slope(rows, |r| r.n_edges),
        mean_type_entropy: mean_of(rows, |r| r.type_histogram_entropy),
        mean_type_jsd: mean_of(rows, |r| r.type_histogram_jsd),
        mean_mean_degree: mean_of(rows, |r| r.mean_degree),
        mean_active_persistent: mean_of(rows, |r| r.active_persistent_structures.map(|v| v as f64)),
        mean_static_persistent: mean_of(rows, |r| r.static_persistent_motifs.map(|v| v as f64)),
        archive_admission_rate: mean_of(rows, |r| r.new_signatures.map(|v| v as f64)),
        mean_n_nodes: rows.iter().map(|r| r.n_nodes as f64).sum::<f64>() / w,
        replicator_rate: n_repl as f64 / m.constructors.len().max(1) as f64,
        kernel_held: kernel_held(m, burn_in),
        f5_top_share: fit.f5_top_share,
        f4_null_sd: fit.f4_null_sd,
    }
}

/// The kernel-held metric (Appendix A): normalized numerical rank of the
/// probe-row matrix over scored windows.
///
/// Measured but not addressable. It is never nameable from `[cognition]` or
/// `[training]` config, so no campaign can select for it — which is what
/// keeps it an independent check on the fitness terms rather than becoming
/// one of them.
pub fn kernel_held(m: &RolloutMetrics, burn_in: usize) -> Option<f64> {
    let rows: Vec<&Vec<f32>> = m.probe_rows.get(burn_in..)?.iter().collect();
    if rows.is_empty() {
        return None;
    }
    let dim = rows[0].len();
    if dim == 0 || rows.iter().any(|r| r.len() != dim) {
        return None;
    }
    let n = rows.len();
    let mat = nalgebra::DMatrix::from_fn(n, dim, |i, j| rows[i][j] as f64);
    let sv = mat.singular_values();
    let max = sv.iter().cloned().fold(0.0f64, f64::max);
    if max <= 0.0 {
        return Some(0.0);
    }
    let rank = sv.iter().filter(|s| **s > EPS_RANK * max).count();
    Some(rank as f64 / n.min(dim) as f64)
}

/// One rollout's tripwire booleans that are computable from that rollout
/// alone. `FITNESS_STRUCTURE_DIVERGENCE` and `SEED_OVERFIT` need campaign
/// history and live in [`campaign_flags`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RolloutFlags {
    pub degenerate_occupancy: bool,
    /// §6.5 (v2). One constructor owning nearly every confirmed cycle *on a
    /// collapsed graph*. High concentration on a graph that stayed its own
    /// size is F5 working, not F5 being gamed — the conjunction is what
    /// separates them.
    pub degenerate_reuse: bool,
    /// §4A.5 (v2). F4's symbol sequence had fewer than two distinct
    /// symbols, so `F4 = 0.0` is exact and predicted rather than estimated.
    pub f4_degenerate_alphabet: bool,
    /// `docs/instrument_registry.md` §1 (v2). At least one engagement
    /// counter read zero on this rollout.
    pub instrument_not_engaged: bool,
}

/// §6.3, §6.5. Both floors are relative to the run's own initial condition:
/// the `Frozen` gate has no node floor to borrow (it is purely dynamical),
/// and an absolute floor would have to be chosen per rule set and would
/// misfire on every one it was not chosen for.
pub fn rollout_flags(m: &RolloutMetrics, fit: &FitnessBreakdown, ho: &HeldOut) -> RolloutFlags {
    let collapsed = ho.mean_n_nodes < COLLAPSE_FRACTION * m.n_nodes_initial as f64;
    RolloutFlags {
        degenerate_occupancy: fit.f2_reward >= 0.5 && collapsed,
        degenerate_reuse: fit.f5_top_share >= TOP_SHARE_DEGENERATE && collapsed,
        f4_degenerate_alphabet: fit.f4_degenerate_alphabet,
        instrument_not_engaged: !fit.engagement.not_engaged().is_empty(),
    }
}

/// Rule 2 of `docs/instrument_registry.md` §1: a value outside its
/// mechanism-derived range is an instrument fault until proven otherwise.
///
/// Only the **hard** bounds are checked here — the ones that are arithmetic
/// rather than empirical, so they need no calibration run and cannot be
/// wrong about a substrate nobody has run yet. Returns the violations; an
/// empty vector is the healthy case.
pub fn predicted_range_violations(fit: &FitnessBreakdown) -> Vec<String> {
    let mut v = Vec::new();
    fn chk(name: &str, x: f64, lo: f64, hi: f64, why: &str) -> Option<String> {
        (x < lo || x > hi).then(|| {
            format!("{name} = {x} outside its predicted range [{lo}, {hi}] — {why}")
        })
    }
    v.extend(chk(
        "f1_mean_interestingness",
        fit.f1_mean_interestingness,
        0.0,
        1.0,
        "the evaluator clamps per window and a mean of clamped values is clamped",
    ));
    v.extend(chk("f2", fit.f2, -0.5, 1.0, "reward in [0,1] minus penalty in [0,0.5]"));
    v.extend(chk("f2_reward", fit.f2_reward, 0.0, 1.0, "a fraction of W"));
    v.extend(chk(
        "f2_penalty",
        fit.f2_penalty,
        0.0,
        LAMBDA_PENALTY,
        "LAMBDA_PENALTY times a fraction of W",
    ));
    v.extend(chk(
        "mean_fidelity",
        fit.mean_fidelity,
        0.0,
        1.0,
        "confirmed_cycles / total is a ratio of counts",
    ));
    if let Some(f4) = fit.f4_predictive_information.value() {
        v.extend(chk(
            "f4_predictive_information",
            f4,
            0.0,
            (PI_ALPHABET as f64).log2(),
            "lag-1 MI of a Q-symbol variable is bounded by log2(Q)",
        ));
        // §4A.5: the permutation null over a non-constant sequence is not a
        // point mass. A degenerate null is what made the V-track's arm-D
        // p = 1.000 arithmetic rather than a measurement.
        if !fit.f4_degenerate_alphabet && fit.f4_null_sd <= 0.0 {
            v.push(
                "f4_null_sd = 0 with a non-constant symbol sequence — the permutation \
                 null collapsed to a point and cannot support a margin"
                    .to_string(),
            );
        }
    }
    if let Some(f5) = fit.f5_constructor_reuse.value() {
        v.extend(chk(
            "f5_constructor_reuse",
            f5,
            0.0,
            fit.f5_max_cycles as f64,
            "weighted-mean bound with fidelity <= 1: F5 cannot exceed max_i cycles_i",
        ));
        v.extend(chk("f5_top_share", fit.f5_top_share, 0.0, 1.0, "it is a share"));
    }
    v
}

/// Per-generation champion record used by the campaign-level tripwires.
#[derive(Clone, Debug)]
pub struct GenerationSummary {
    pub generation: u32,
    pub best_fitness: f64,
    pub champion_holdout_mean: f64,
    pub champion_holdout_sd: f64,
    pub champion_rotation_mean: f64,
    pub champion_held_out: HeldOut,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CampaignFlags {
    pub fitness_structure_divergence: bool,
    pub seed_overfit: bool,
}

/// §6.1 and §6.2, evaluated at the latest generation in `history`.
pub fn campaign_flags(history: &[GenerationSummary]) -> CampaignFlags {
    let Some(last) = history.last() else {
        return CampaignFlags::default();
    };

    // §6.2 SEED_OVERFIT: rotation-seed fitness far above hold-out fitness.
    let seed_overfit = last.champion_holdout_sd > 0.0
        && (last.champion_rotation_mean - last.champion_holdout_mean)
            > 2.0 * last.champion_holdout_sd;

    // §6.1 FITNESS_STRUCTURE_DIVERGENCE.
    let mut divergence = false;
    if history.len() > DIVERGENCE_GENS {
        let prev = &history[history.len() - 1 - DIVERGENCE_GENS];
        // A zero baseline makes the ratio test undefined; the flag does not
        // fire rather than being replaced by an absolute bar this document
        // cannot supply for three different fitness ranges.
        if prev.best_fitness != 0.0 {
            let rel = (last.best_fitness - prev.best_fitness) / prev.best_fitness.abs();
            if rel > DIVERGENCE_RATIO {
                let tail = &history[history.len() - 1 - DIVERGENCE_GENS..];
                let flat = |f: fn(&HeldOut) -> Option<f64>| -> bool {
                    let ys: Vec<f64> = tail.iter().filter_map(|g| f(&g.champion_held_out)).collect();
                    if ys.len() < 2 {
                        return true; // no evidence of structural improvement
                    }
                    trailing_slope(&ys) <= 0.0
                };
                divergence = flat(|h| h.mean_type_entropy)
                    && flat(|h| h.mean_active_persistent)
                    && flat(|h| h.archive_admission_rate)
                    && flat(|h| h.mean_type_jsd);
            }
        }
    }

    CampaignFlags {
        fitness_structure_divergence: divergence,
        seed_overfit,
    }
}

/// OLS slope of `ys` against its own index.
fn trailing_slope(ys: &[f64]) -> f64 {
    let n = ys.len() as f64;
    let mx = (n - 1.0) / 2.0;
    let my = ys.iter().sum::<f64>() / n;
    let sxx: f64 = (0..ys.len()).map(|i| (i as f64 - mx).powi(2)).sum();
    if sxx == 0.0 {
        return 0.0;
    }
    let sxy: f64 = ys
        .iter()
        .enumerate()
        .map(|(i, y)| (i as f64 - mx) * (y - my))
        .sum();
    sxy / sxx
}

/// Compile-time-checked agreement between `docs/fitness_formulas.md` §3.1's
/// index table and `Regime`'s own discriminants.
///
/// `regime_occupancy` is a positional array, so a future `Regime` variant
/// inserted anywhere but the end would silently reindex every published
/// campaign CSV. This makes that a test failure instead.
pub const REGIME_ORDER: [Regime; 11] = [
    Regime::Frozen,
    Regime::StaticOrdered,
    Regime::GrowthTreadmill,
    Regime::WhiteNoiseSoup,
    Regime::SterileChaos,
    Regime::Exploding,
    Regime::FragmentedIsolation,
    Regime::Monoculture,
    Regime::ActivePersistentStructure,
    Regime::GenerativeCandidate,
    Regime::Unknown,
];

#[cfg(test)]
mod tests {
    use super::*;
    use hg_sim_core::{regime_index, ConstructorSummary};

    fn row(window: u64, regime: Regime, interest: f64, nodes: usize) -> MetricsRow {
        MetricsRow {
            window_id: window,
            step_end: (window + 1) * 32,
            n_nodes: nodes,
            n_edges: nodes * 2,
            regime: Some(regime.as_str().to_string()),
            interestingness_score: Some(interest),
            ..Default::default()
        }
    }

    fn metrics(rows: Vec<MetricsRow>) -> RolloutMetrics {
        RolloutMetrics {
            final_canonical_hash: 0,
            windows: rows,
            regime_occupancy: [0; 11],
            constructors: Vec::new(),
            tasks: Vec::new(),
            task_lattice_depth: 0,
            probe_rows: Vec::new(),
            n_nodes_initial: 100,
            final_step: 0,
            wallclock_ms: 0,
        }
    }

    #[test]
    fn regime_index_table_matches_the_document() {
        for (i, r) in REGIME_ORDER.iter().enumerate() {
            assert_eq!(
                regime_index(*r),
                i,
                "docs/fitness_formulas.md §3.1 index table disagrees with Regime for {r:?}"
            );
        }
    }

    #[test]
    fn burn_in_windows_are_excluded() {
        let m = metrics(vec![
            row(0, Regime::Exploding, 0.0, 100),
            row(1, Regime::Exploding, 0.0, 100),
            row(2, Regime::GenerativeCandidate, 1.0, 100),
            row(3, Regime::GenerativeCandidate, 1.0, 100),
        ]);
        let f = evaluate(&m, 2).expect("scored");
        assert_eq!(f.n_windows_scored, 2);
        assert_eq!(f.f1_mean_interestingness, 1.0);
        assert_eq!(f.f2_reward, 1.0);
        assert_eq!(f.f2_penalty, 0.0);
    }

    #[test]
    fn an_empty_scored_set_is_an_error_not_a_zero() {
        let m = metrics(vec![row(0, Regime::Frozen, 0.0, 100)]);
        assert_eq!(evaluate(&m, 5), Err(FitnessError::NoScoredWindows));
        assert_eq!(evaluate(&metrics(vec![]), 0), Err(FitnessError::NoScoredWindows));
    }

    #[test]
    fn f2_range_is_minus_half_to_one() {
        let all_target = metrics(vec![row(0, Regime::ActivePersistentStructure, 0.0, 100)]);
        assert_eq!(evaluate(&all_target, 0).unwrap().f2, 1.0);

        let all_penalized = metrics(vec![row(0, Regime::Exploding, 0.0, 100)]);
        assert_eq!(evaluate(&all_penalized, 0).unwrap().f2, -0.5);

        // StaticOrdered is in neither set: neither rewarded nor punished.
        let neutral = metrics(vec![row(0, Regime::StaticOrdered, 0.0, 100)]);
        assert_eq!(evaluate(&neutral, 0).unwrap().f2, 0.0);
    }

    #[test]
    fn unknown_windows_dilute_rather_than_being_farmable() {
        let m = metrics(vec![
            row(0, Regime::GenerativeCandidate, 0.0, 100),
            row(1, Regime::Unknown, 0.0, 100),
        ]);
        let f = evaluate(&m, 0).unwrap();
        assert_eq!(f.f2_reward, 0.5, "Unknown counts toward W but not toward target");
        assert_eq!(f.f2_penalty, 0.0);
    }

    #[test]
    fn f2_reward_and_penalty_are_separable() {
        let m = metrics(vec![
            row(0, Regime::GenerativeCandidate, 0.0, 100),
            row(1, Regime::Exploding, 0.0, 100),
        ]);
        let f = evaluate(&m, 0).unwrap();
        assert_eq!(f.f2_reward, 0.5);
        assert_eq!(f.f2_penalty, 0.25);
        assert_eq!(f.f2, 0.25);
    }

    #[test]
    fn f3_counts_only_confirmed_constructors() {
        let mut m = metrics(vec![row(0, Regime::StaticOrdered, 0.0, 100)]);
        m.constructors = vec![
            ConstructorSummary { wl_hash: 1, fidelity: 0.8, is_replicator: false, confirmed_cycles: 3 },
            ConstructorSummary { wl_hash: 2, fidelity: 0.4, is_replicator: true, confirmed_cycles: 1 },
            // Never confirmed: excluded from the count and the mean.
            ConstructorSummary { wl_hash: 3, fidelity: 0.0, is_replicator: false, confirmed_cycles: 0 },
        ];
        let f = evaluate(&m, 0).unwrap();
        assert_eq!(f.n_confirmed_constructors, 2);
        assert!((f.mean_fidelity - 0.6).abs() < 1e-6);
        assert!((f.f3_constructor_yield - 1.2).abs() < 1e-6);
    }

    #[test]
    fn f3_is_a_genuine_zero_when_nothing_is_confirmed() {
        let m = metrics(vec![row(0, Regime::StaticOrdered, 0.0, 100)]);
        let f = evaluate(&m, 0).unwrap();
        assert_eq!(f.f3_constructor_yield, 0.0);
        assert_eq!(f.mean_fidelity, 0.0);
    }

    #[test]
    fn degenerate_occupancy_fires_on_collapse_only() {
        let mut m = metrics(vec![
            row(0, Regime::GenerativeCandidate, 0.0, 10),
            row(1, Regime::GenerativeCandidate, 0.0, 10),
        ]);
        m.n_nodes_initial = 100;
        let f = evaluate(&m, 0).unwrap();
        let ho = held_out(&m, 0, &f);
        assert!(rollout_flags(&m, &f, &ho).degenerate_occupancy, "10 << 0.25 * 100");

        // Same occupancy, healthy graph: no flag.
        let mut big = metrics(vec![
            row(0, Regime::GenerativeCandidate, 0.0, 90),
            row(1, Regime::GenerativeCandidate, 0.0, 90),
        ]);
        big.n_nodes_initial = 100;
        let f2 = evaluate(&big, 0).unwrap();
        let ho2 = held_out(&big, 0, &f2);
        assert!(!rollout_flags(&big, &f2, &ho2).degenerate_occupancy);
    }

    #[test]
    fn kernel_held_reads_the_probe_rows_rank() {
        let mut m = metrics(vec![row(0, Regime::StaticOrdered, 0.0, 100)]);
        // Rank 1: every row a multiple of the same vector.
        m.probe_rows = vec![vec![1.0, 2.0], vec![2.0, 4.0], vec![3.0, 6.0]];
        assert_eq!(kernel_held(&m, 0), Some(0.5), "rank 1 of min(3,2) = 2");

        // Full rank.
        m.probe_rows = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]];
        assert_eq!(kernel_held(&m, 0), Some(1.0));

        // No probe at all.
        m.probe_rows = Vec::new();
        assert_eq!(kernel_held(&m, 0), None);
    }

    #[test]
    fn growth_exponent_recovers_a_known_power_law() {
        // n_nodes = step^2 exactly => log-log slope 2.
        let rows: Vec<MetricsRow> = (1..=8)
            .map(|i| {
                let mut r = row(i as u64, Regime::StaticOrdered, 0.0, (i * i) as usize);
                r.step_end = i as u64;
                r
            })
            .collect();
        let mm = metrics(rows);
        let ff = evaluate(&mm, 0).unwrap();
        let ho = held_out(&mm, 0, &ff);
        let slope = ho.growth_exponent_nodes.expect("slope");
        assert!((slope - 2.0).abs() < 1e-9, "slope was {slope}");
    }

    fn gen_summary(g: u32, best: f64, entropy: f64) -> GenerationSummary {
        GenerationSummary {
            generation: g,
            best_fitness: best,
            champion_holdout_mean: best,
            champion_holdout_sd: 0.1,
            champion_rotation_mean: best,
            champion_held_out: HeldOut {
                mean_type_entropy: Some(entropy),
                mean_active_persistent: Some(entropy),
                archive_admission_rate: Some(entropy),
                mean_type_jsd: Some(entropy),
                ..Default::default()
            },
        }
    }

    #[test]
    fn divergence_fires_when_fitness_climbs_and_structure_does_not() {
        // Fitness 0.10 -> 0.50 over 10 gens; every structure metric flat.
        let hist: Vec<_> = (0..=10)
            .map(|g| gen_summary(g, 0.10 + 0.04 * g as f64, 1.0))
            .collect();
        assert!(campaign_flags(&hist).fitness_structure_divergence);
    }

    #[test]
    fn divergence_does_not_fire_when_structure_climbs_too() {
        let hist: Vec<_> = (0..=10)
            .map(|g| gen_summary(g, 0.10 + 0.04 * g as f64, 1.0 + 0.1 * g as f64))
            .collect();
        assert!(!campaign_flags(&hist).fitness_structure_divergence);
    }

    #[test]
    fn divergence_does_not_fire_on_a_zero_baseline() {
        let hist: Vec<_> = (0..=10).map(|g| gen_summary(g, 0.0, 1.0)).collect();
        assert!(!campaign_flags(&hist).fitness_structure_divergence);
    }

    // -----------------------------------------------------------------
    // F4 — predictive information (§4A)
    // -----------------------------------------------------------------

    /// `n` windows whose JSD follows `f(i)`, enough to clear PI_MIN_PAIRS.
    fn jsd_run(n: usize, f: impl Fn(usize) -> f64) -> RolloutMetrics {
        let rows = (0..n)
            .map(|i| {
                let mut r = row(i as u64, Regime::StaticOrdered, 0.5, 100);
                r.type_histogram_jsd = Some(f(i));
                r
            })
            .collect();
        metrics(rows)
    }

    #[test]
    fn f4_is_exactly_zero_on_a_frozen_symbol_sequence() {
        // A frozen substrate emits jsd = 0 every window. §4A.4's first row:
        // H(S) = 0, so I = 0 and every shuffle replicate is 0 too.
        let f = evaluate(&jsd_run(80, |_| 0.0), 0).unwrap();
        assert_eq!(f.f4_predictive_information, Reported::Value(0.0));
        assert!(f.f4_degenerate_alphabet, "one symbol is the degenerate case");
        assert_eq!(f.f4_null_sd, 0.0, "a constant sequence has a point-mass null");
    }

    #[test]
    fn f4_is_unmeasured_below_the_minimum_pair_count() {
        // The plug-in MI bias is comparable to the signal here. An estimator
        // whose sample cannot support an estimate must not emit a number —
        // and `unmeasured` is not `0.0` (§4A.5).
        let m = jsd_run(PI_MIN_PAIRS, |i| if i % 2 == 0 { 0.05 } else { 0.95 });
        let f = evaluate(&m, 0).unwrap();
        assert_eq!(f.f4_predictive_information, Reported::Unmeasured);
        assert_eq!(f.engagement.f4_pairs, PI_MIN_PAIRS - 1);
        assert_eq!(
            f.select(FitnessFn::F4PredictiveInformation),
            Err(FitnessError::Unmeasured(FitnessFn::F4PredictiveInformation)),
            "an unmeasured fitness is an error, never a score"
        );
    }

    #[test]
    fn f4_detects_a_perfectly_predictable_alternation() {
        // Period-2 alternation between two symbols: I(S; S') = 1 bit, and
        // the shuffle null of a balanced two-symbol sequence is near zero.
        let f = evaluate(&jsd_run(200, |i| if i % 2 == 0 { 0.05 } else { 0.95 }), 0).unwrap();
        let v = f.f4_predictive_information.value().expect("measured");
        assert!(v > 0.8, "a period-2 sequence carries ~1 bit, got {v}");
        assert!(v <= 2.0, "log2(PI_ALPHABET) is the hard ceiling");
        assert!(f.f4_null_sd > 0.0, "a non-constant sequence has a real null");
        assert!(!f.f4_degenerate_alphabet);
        assert!(predicted_range_violations(&f).is_empty());
    }

    #[test]
    fn f4_is_near_zero_on_an_unpredictable_sequence() {
        // The whole point of the shuffle correction: a varied but
        // structureless sequence must NOT score, or F4 is a diversity
        // objective wearing a different hat (§4A.4's second row).
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(20260727);
        let vals: Vec<f64> = (0..400)
            .map(|_| rand::Rng::gen_range(&mut rng, 0.0..1.0))
            .collect();
        let f = evaluate(&jsd_run(400, |i| vals[i]), 0).unwrap();
        let v = f.f4_predictive_information.value().expect("measured");
        assert!(
            v < 3.0 * f.f4_null_sd.max(1e-6),
            "iid sequence scored {v} against a null sd of {}",
            f.f4_null_sd
        );
    }

    #[test]
    fn f4_symbolization_covers_the_alphabet_ends() {
        // jsd = 1.0 must land in the top bin, not out of bounds.
        let f = evaluate(&jsd_run(120, |i| if i % 2 == 0 { 0.0 } else { 1.0 }), 0).unwrap();
        assert!(!f.f4_degenerate_alphabet, "0.0 and 1.0 are two distinct symbols");
        assert!(f.f4_predictive_information.value().unwrap() > 0.8);
    }

    #[test]
    fn f4_records_gap_joins_rather_than_hiding_them() {
        let rows = (0..120)
            .map(|i| {
                let mut r = row(i as u64, Regime::StaticOrdered, 0.5, 100);
                // Every tenth window emitted nothing.
                r.type_histogram_jsd = (i % 10 != 3).then_some(if i % 2 == 0 { 0.1 } else { 0.9 });
                r
            })
            .collect();
        let f = evaluate(&metrics(rows), 0).unwrap();
        assert!(f.f4_gap_joins > 0, "the adjacency error must be visible");
        assert_eq!(f.engagement.f4_pairs, 120 - 12 - 1);
    }

    // -----------------------------------------------------------------
    // F5 — constructor reuse (§4B)
    // -----------------------------------------------------------------

    fn with_constructors(cs: &[(f32, u32)]) -> RolloutMetrics {
        let mut m = metrics(vec![row(0, Regime::StaticOrdered, 0.0, 100)]);
        m.constructors = cs
            .iter()
            .enumerate()
            .map(|(i, (fid, cyc))| ConstructorSummary {
                wl_hash: i as u64,
                fidelity: *fid,
                is_replicator: false,
                confirmed_cycles: *cyc,
            })
            .collect();
        m
    }

    #[test]
    fn f5_is_maximized_by_concentration_and_f3_by_spreading() {
        // §4B.1, the whole argument for F5, checked numerically. Total
        // yield T = 12 both ways, fidelity 1.0 both ways.
        let concentrated = evaluate(&with_constructors(&[(1.0, 12)]), 0).unwrap();
        let spread = evaluate(&with_constructors(&[(1.0, 1); 12]), 0).unwrap();

        assert_eq!(concentrated.f5_constructor_reuse, Reported::Value(12.0));
        assert_eq!(spread.f5_constructor_reuse, Reported::Value(1.0));
        // F3 orders them the other way round. That opposition is what makes
        // F5 a real test of the diversity diagnosis rather than a re-skin.
        assert_eq!(concentrated.f3_constructor_yield, 1.0);
        assert_eq!(spread.f3_constructor_yield, 12.0);
    }

    #[test]
    fn f5_is_unmeasured_when_the_causal_layer_never_ran() {
        // Bug 8: `[causal]` off leaves `constructors` empty, and reporting
        // 0.0 would read as "no policy affects constructor reuse".
        let f = evaluate(&metrics(vec![row(0, Regime::StaticOrdered, 0.0, 100)]), 0).unwrap();
        assert_eq!(f.f5_constructor_reuse, Reported::Unmeasured);
        assert_eq!(f.engagement.f5_records_examined, 0);
        assert!(f.select(FitnessFn::F5ConstructorReuse).is_err());
        assert!(
            f.select(FitnessFn::F3ConstructorYield).is_err(),
            "F3 inherits the same gate — it was the original victim"
        );
    }

    #[test]
    fn f5_is_a_genuine_zero_when_the_layer_looked_and_found_nothing() {
        // Records examined, none confirmed. Different fact, different state.
        let f = evaluate(&with_constructors(&[(0.0, 0), (0.0, 0)]), 0).unwrap();
        assert_eq!(f.f5_constructor_reuse, Reported::Value(0.0));
        assert_eq!(f.engagement.f5_records_examined, 2);
        assert_eq!(f.select(FitnessFn::F5ConstructorReuse), Ok(0.0));
    }

    #[test]
    fn f5_never_exceeds_its_hard_ceiling() {
        // §4B.3's arithmetic bound, the check that needs no calibration run.
        for cs in [
            &[(1.0, 7), (0.5, 3), (0.9, 1)][..],
            &[(1.0, 100), (1.0, 1)][..],
            &[(0.1, 4), (0.2, 4), (0.3, 4)][..],
        ] {
            let f = evaluate(&with_constructors(cs), 0).unwrap();
            let v = f.f5_constructor_reuse.value().unwrap();
            assert!(
                v <= f.f5_max_cycles as f64 + 1e-9,
                "F5 {v} exceeded max cycles {}",
                f.f5_max_cycles
            );
            assert!(predicted_range_violations(&f).is_empty());
        }
    }

    #[test]
    fn degenerate_reuse_needs_collapse_as_well_as_concentration() {
        // High concentration on a graph that kept its size is F5 working.
        let mut healthy = with_constructors(&[(1.0, 50), (1.0, 1)]);
        healthy.windows = vec![row(0, Regime::StaticOrdered, 0.0, 90)];
        healthy.n_nodes_initial = 100;
        let f = evaluate(&healthy, 0).unwrap();
        let ho = held_out(&healthy, 0, &f);
        assert!(f.f5_top_share > TOP_SHARE_DEGENERATE);
        assert!(!rollout_flags(&healthy, &f, &ho).degenerate_reuse);

        // Same concentration, graph collapsed to a tenth: the flag fires.
        let mut collapsed = with_constructors(&[(1.0, 50), (1.0, 1)]);
        collapsed.windows = vec![row(0, Regime::StaticOrdered, 0.0, 10)];
        collapsed.n_nodes_initial = 100;
        let f2 = evaluate(&collapsed, 0).unwrap();
        let ho2 = held_out(&collapsed, 0, &f2);
        assert!(rollout_flags(&collapsed, &f2, &ho2).degenerate_reuse);
    }

    // -----------------------------------------------------------------
    // Instrument discipline
    // -----------------------------------------------------------------

    #[test]
    fn v2_is_additive_over_v1() {
        // `docs/fitness_formulas.md` §7: a v1 campaign re-scored under v2
        // must produce identical F1/F2/F3. Asserted rather than argued,
        // because "additive" is a claim about every C0–C5 number staying
        // comparable.
        let m = metrics(vec![
            row(0, Regime::GenerativeCandidate, 0.7, 100),
            row(1, Regime::Exploding, 0.2, 100),
            row(2, Regime::StaticOrdered, 0.5, 100),
            row(3, Regime::ActivePersistentStructure, 0.9, 100),
        ]);
        let f = evaluate(&m, 0).unwrap();
        // Hand-computed under v1's formulas, with no reference to v2.
        assert!((f.f1_mean_interestingness - (0.7 + 0.2 + 0.5 + 0.9) / 4.0).abs() < 1e-12);
        assert!((f.f2_reward - 0.5).abs() < 1e-12);
        assert!((f.f2_penalty - LAMBDA_PENALTY * 0.25).abs() < 1e-12);
        assert!((f.f2 - (0.5 - 0.125)).abs() < 1e-12);
        assert_eq!(f.f3_constructor_yield, 0.0);
    }

    #[test]
    fn instrument_not_engaged_fires_and_names_the_counters() {
        // A rollout with no causal layer, no EI, and too few F4 pairs.
        let m = metrics(vec![row(0, Regime::StaticOrdered, 0.5, 100)]);
        let f = evaluate(&m, 0).unwrap();
        let names = f.engagement.not_engaged();
        for expect in ["ei_intervals_emitted", "task_lattice_entries", "f4_pairs"] {
            assert!(names.contains(&expect), "{expect} should be flagged, got {names:?}");
        }
        assert!(
            !names.contains(&"f1_windows_scored"),
            "F1 did engage — the flag must not be all-or-nothing"
        );
        let ho = held_out(&m, 0, &f);
        assert!(rollout_flags(&m, &f, &ho).instrument_not_engaged);
    }

    #[test]
    fn the_predicted_range_check_actually_fires() {
        // A guard that never rejects cannot support a verdict
        // (`docs/instrument_registry.md` §1). This is that guard's firing
        // test, and it is the reason the C0-C5 sigma guard sat inert.
        let mut f = evaluate(&metrics(vec![row(0, Regime::StaticOrdered, 0.5, 100)]), 0).unwrap();
        assert!(predicted_range_violations(&f).is_empty(), "healthy baseline");

        f.f1_mean_interestingness = 1.5;
        assert_eq!(predicted_range_violations(&f).len(), 1);

        // F4 above its log2(Q) ceiling: arithmetically impossible.
        f.f1_mean_interestingness = 0.5;
        f.f4_predictive_information = Reported::Value(2.5);
        f.f4_null_sd = 0.1;
        assert!(predicted_range_violations(&f)[0].contains("f4_predictive_information"));

        // A collapsed permutation null on a non-constant sequence.
        f.f4_predictive_information = Reported::Value(0.5);
        f.f4_null_sd = 0.0;
        f.f4_degenerate_alphabet = false;
        assert!(predicted_range_violations(&f)[0].contains("f4_null_sd"));
    }

    #[test]
    fn reported_round_trips_through_its_two_states() {
        // The literal string is part of the schema, not a formatting choice.
        assert_eq!(Reported::Unmeasured.render(), "unmeasured");
        assert_eq!(Reported::Value(0.0).render(), "0");
        let j = serde_json::to_string(&Reported::Unmeasured).unwrap();
        assert_eq!(j, "\"unmeasured\"");
        assert_eq!(
            serde_json::from_str::<Reported>(&j).unwrap(),
            Reported::Unmeasured
        );
        assert_eq!(
            serde_json::from_str::<Reported>("0.25").unwrap(),
            Reported::Value(0.25)
        );
        // CSV hands everything back as a string.
        assert_eq!(
            serde_json::from_str::<Reported>("\"0.25\"").unwrap(),
            Reported::Value(0.25)
        );
    }

    #[test]
    fn seed_overfit_fires_on_a_rotation_holdout_gap() {
        let mut g = gen_summary(5, 0.5, 1.0);
        g.champion_rotation_mean = 0.9;
        g.champion_holdout_mean = 0.5;
        g.champion_holdout_sd = 0.1; // gap 0.4 > 2 * 0.1
        assert!(campaign_flags(&[g]).seed_overfit);

        let mut ok = gen_summary(5, 0.5, 1.0);
        ok.champion_rotation_mean = 0.55;
        ok.champion_holdout_mean = 0.5;
        ok.champion_holdout_sd = 0.1; // gap 0.05 < 0.2
        assert!(!campaign_flags(&[ok]).seed_overfit);
    }
}
