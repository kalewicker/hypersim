//! Alphabet accounting: cardinality, composition, effective size, novelty curve.
//!
//! Formulas frozen in `docs/atrack_a1_preregistration.md` §3 and §4.
//!
//! Every container here is ordered (`BTreeMap` / sorted `Vec`) and every sum
//! and ranking runs in sorted key order. That is not decoration: alphabet
//! enumeration is set-and-hash work, which is exactly where index-order
//! contamination lives, and the V-track's C3 finding was an entropy summed
//! over `HashMap::values()` diverging in the last ulps between processes.
//! Building the discipline in at construction is cheaper than retrofitting it.

use std::collections::BTreeMap;

/// Per-symbol occurrence counts plus the discovery record needed to reconstruct
/// the cumulative-novelty curve exactly, without storing a point per event.
#[derive(Clone, Debug, Default)]
pub struct SymbolStats {
    counts: BTreeMap<u64, u64>,
    /// Cumulative event index (1-based) at which each NEW symbol first appeared,
    /// in discovery order. Monotone non-decreasing by construction.
    first_seen_event: Vec<u64>,
    /// Step at which each NEW symbol first appeared, same order.
    first_seen_step: Vec<u64>,
    total_events: u64,
    last_step: u64,
}

impl SymbolStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one occurrence. `event_index` is 1-based and cumulative over the
    /// run; `step` is the step the event fired in.
    pub fn record(&mut self, symbol: u64, event_index: u64, step: u64) {
        let slot = self.counts.entry(symbol).or_insert(0);
        *slot += 1;
        if *slot == 1 {
            self.first_seen_event.push(event_index);
            self.first_seen_step.push(step);
        }
        self.total_events = self.total_events.max(event_index);
        self.last_step = self.last_step.max(step);
    }

    /// Raw alphabet cardinality |A|.
    pub fn cardinality(&self) -> usize {
        self.counts.len()
    }

    /// Instrument-registry Rule 1 engagement counter: how many times the
    /// instrument actually did work. Zero means `unmeasured`, never `0.0`.
    pub fn total_events(&self) -> u64 {
        self.total_events
    }

    pub fn last_step(&self) -> u64 {
        self.last_step
    }

    /// Shannon entropy of the symbol frequency distribution, in bits, summed in
    /// sorted symbol order (`BTreeMap` iterates by key).
    pub fn entropy_bits(&self) -> f64 {
        let n = self.total_events as f64;
        if n <= 0.0 || self.counts.is_empty() {
            return 0.0;
        }
        let mut h = 0.0f64;
        for (_sym, &c) in self.counts.iter() {
            if c == 0 {
                continue;
            }
            let p = c as f64 / n;
            h -= p * p.log2();
        }
        h
    }

    /// Effective alphabet size 2^H (perplexity). Reported BESIDE the raw count,
    /// never instead of it: a 500-symbol alphabet with 99% of its mass on three
    /// symbols is a three-symbol alphabet in practice.
    pub fn effective_size(&self) -> f64 {
        if self.counts.is_empty() {
            return 0.0;
        }
        self.entropy_bits().exp2()
    }

    /// Symbols observed exactly once — the `K_MIN`/no-singletons discipline.
    pub fn singletons(&self) -> usize {
        self.counts.values().filter(|&&c| c == 1).count()
    }

    /// `|A| / n_events` — how close the alphabet is riding its own HARD CEILING.
    ///
    /// One event can contribute at most one symbol, so `|A| ≤ n_events` always.
    /// A run at (or near) that ceiling has not measured how large the
    /// substrate's alphabet is; it has measured how many events were observed.
    /// Its novelty curve cannot exceed linear by construction either, so
    /// "LINEAR growth" from such a run is a statement about the budget as much
    /// as about the substrate. This number is what makes that visible instead
    /// of leaving it for a reader to notice.
    pub fn ceiling_fraction(&self) -> f64 {
        if self.total_events == 0 {
            return 0.0;
        }
        self.cardinality() as f64 / self.total_events as f64
    }

    /// The threshold above which the ceiling caveat is printed.
    pub const CEILING_WARN: f64 = 0.5;

    /// Fraction of events carried by the `k` most frequent symbols.
    pub fn top_mass(&self, k: usize) -> f64 {
        if self.total_events == 0 {
            return 0.0;
        }
        // Rank in sorted-key order first, then by count: ties in count break on
        // the symbol code, which is content, not iteration order.
        let mut ranked: Vec<(u64, u64)> = self.counts.iter().map(|(&s, &c)| (c, s)).collect();
        ranked.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        let taken: u64 = ranked.iter().take(k).map(|(c, _)| *c).sum();
        taken as f64 / self.total_events as f64
    }

    /// Sorted symbol codes — the reproducibility fingerprint of an alphabet.
    /// N1.3's lesson: scalars agreeing across processes is not evidence that the
    /// underlying STRUCTURE agrees, so the structure is exposed for checking.
    pub fn sorted_symbols(&self) -> Vec<u64> {
        self.counts.keys().copied().collect()
    }

    pub fn count_of(&self, symbol: u64) -> u64 {
        self.counts.get(&symbol).copied().unwrap_or(0)
    }

    /// N(e) — distinct symbols after `e` cumulative events. Exact: the
    /// first-seen list is monotone, so this is a binary search.
    pub fn distinct_by_event(&self, e: u64) -> usize {
        self.first_seen_event.partition_point(|&x| x <= e)
    }

    /// N(t) — distinct symbols after step `t`.
    pub fn distinct_by_step(&self, t: u64) -> usize {
        self.first_seen_step.partition_point(|&x| x <= t)
    }

    /// Novelty curve against cumulative events, sampled at `n` log-spaced
    /// points over `[1, total_events]`.
    pub fn curve_by_event(&self, n: usize) -> Vec<(f64, f64)> {
        log_grid(self.total_events, n)
            .into_iter()
            .map(|x| (x as f64, self.distinct_by_event(x) as f64))
            .collect()
    }

    /// Novelty curve against step, sampled at `n` log-spaced points over
    /// `[1, last_step]`.
    pub fn curve_by_step(&self, n: usize) -> Vec<(f64, f64)> {
        log_grid(self.last_step.max(1), n)
            .into_iter()
            .map(|x| (x as f64, self.distinct_by_step(x) as f64))
            .collect()
    }
}

/// `n` distinct log-spaced integers in `[1, hi]`, ascending. Deduplicated, so a
/// small `hi` simply yields fewer points rather than repeats (which would
/// silently reweight a fit).
fn log_grid(hi: u64, n: usize) -> Vec<u64> {
    if hi == 0 {
        return Vec::new();
    }
    if hi as usize <= n {
        return (1..=hi).collect();
    }
    let lo_l = 0.0f64;
    let hi_l = (hi as f64).ln();
    let mut out: Vec<u64> = (0..n)
        .map(|i| {
            let t = i as f64 / (n as f64 - 1.0);
            (lo_l + t * (hi_l - lo_l)).exp().round().max(1.0) as u64
        })
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// The complete accounting for one run: every signature variant, every radius.
///
/// Index convention: `context[r]` / `delta[r]` are the S1 / S2 alphabets at
/// radius `r`; `degree` is S3 (radius-free); `rule_only` is the vacuous
/// definition, kept as the contrast.
#[derive(Clone, Debug)]
pub struct RunAlphabets {
    pub context: Vec<SymbolStats>,
    pub delta: Vec<SymbolStats>,
    pub degree: SymbolStats,
    pub rule_only: SymbolStats,
    /// **C0** — the S1 context alphabet accumulated SEPARATELY per rule,
    /// indexed `[rule_id][radius]`.
    ///
    /// `A_eff` over a multi-rule file is entropy-based, so it rises partly
    /// because the *rule-share* distribution got less degenerate, independently
    /// of whether any rule contributed context diversity. The tell is in B1's
    /// own data: at k = 2 the two horizons give `A_eff` 3.74 vs 7.53 with
    /// **identical** `|A|` = 16 and **identical** recurrent = 16, because
    /// `influx`'s event share moved 4.3% → 29.9%.
    ///
    /// These per-rule streams are what make the share factor separable. They
    /// are recorded alongside the existing accumulators and change none of
    /// them — `tests/b0_instrument_pin.rs` still holds.
    pub per_rule_context: Vec<Vec<SymbolStats>>,
    /// Distinct node-WL-classes at radius 2 per step — the A0.1 byproduct
    /// (node-state class count M(t), which is a DIFFERENT object from the event
    /// alphabet and is reported as such).
    pub node_class_count: Vec<(u64, usize)>,
    pub steps_run: u64,
    pub stop_reason: StopReason,
    pub final_nodes: usize,
    pub final_edges: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    StepCap,
    NodeCap,
    Quiescent,
    /// The B-track's equal-event budget was reached. `|A| ≤ n_events` is a hard
    /// bound, so holding events constant is what holds the binding resource
    /// constant across rows that differ in event rate.
    EventCap,
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            StopReason::StepCap => "step cap",
            StopReason::NodeCap => "node cap",
            StopReason::Quiescent => "quiescent",
            StopReason::EventCap => "event cap",
        };
        f.write_str(s)
    }
}

impl RunAlphabets {
    pub fn new(n_radii: usize) -> Self {
        Self {
            context: vec![SymbolStats::new(); n_radii],
            delta: vec![SymbolStats::new(); n_radii],
            degree: SymbolStats::new(),
            rule_only: SymbolStats::new(),
            per_rule_context: Vec::new(),
            node_class_count: Vec::new(),
            steps_run: 0,
            stop_reason: StopReason::StepCap,
            final_nodes: 0,
            final_edges: 0,
        }
    }

    /// Instrument-registry Rule 1: the engagement counter. Zero here means the
    /// row is written `unmeasured`, not `0`.
    pub fn n_events(&self) -> u64 {
        self.degree.total_events()
    }

    pub fn engaged(&self) -> bool {
        self.n_events() > 0
    }

    /// S1 cardinalities across the swept radii.
    pub fn context_sizes(&self) -> Vec<usize> {
        self.context.iter().map(|s| s.cardinality()).collect()
    }

    pub fn delta_sizes(&self) -> Vec<usize> {
        self.delta.iter().map(|s| s.cardinality()).collect()
    }

    /// **C0 — the exact decomposition of `A_eff`.**
    ///
    /// Write `w_r` for rule `r`'s share of events and `p_{r,i}` for its
    /// within-rule context distribution. The global distribution is
    /// `w_r · p_{r,i}`, so global entropy splits exactly:
    ///
    /// ```text
    /// H_global = H(w) + Σ_r w_r H_r
    /// A_eff    = 2^H(w)  ×  2^(Σ_r w_r H_r)
    ///          = share perplexity × weighted geometric mean of within-rule A_eff
    /// ```
    ///
    /// **`2^H(w)` is the contamination, and it is now a reported number rather
    /// than an unmeasured worry.** A single-rule file has `H(w) = 0` and
    /// `2^H(w) = 1`, which is why the A-track's rows were never affected.
    ///
    /// Returns `(share_perplexity, corrected, within_sum, per_rule)` at the
    /// given radius, where `corrected = A_eff / share_perplexity` and
    /// `within_sum = Σ_r 2^(H_r)` is fully share-independent — the rule shares
    /// do not appear in it at all.
    pub fn share_decomposition(&self, radius: usize) -> ShareDecomposition {
        let total = self.rule_only.total_events();
        let mut h_share = 0.0f64;
        let mut per_rule: Vec<(usize, u64, f64)> = Vec::new();
        let mut within_sum = 0.0f64;
        if total > 0 {
            for (rule_id, radii) in self.per_rule_context.iter().enumerate() {
                let Some(s) = radii.get(radius) else { continue };
                let n = s.total_events();
                if n == 0 {
                    continue;
                }
                let w = n as f64 / total as f64;
                h_share -= w * w.log2();
                let eff = s.effective_size();
                within_sum += eff;
                per_rule.push((rule_id, n, eff));
            }
        }
        let share_perplexity = h_share.exp2();
        let a_eff = self.context.get(radius).map(|s| s.effective_size()).unwrap_or(0.0);
        ShareDecomposition {
            a_eff,
            share_perplexity,
            corrected: if share_perplexity > 0.0 { a_eff / share_perplexity } else { f64::NAN },
            within_sum,
            per_rule,
        }
    }
}

/// The C0 witnesses for one radius. `a_eff` is the B-track's headline metric;
/// everything else exists to say how much of it is context diversity and how
/// much is rule-share structure.
#[derive(Clone, Debug)]
pub struct ShareDecomposition {
    pub a_eff: f64,
    /// `2^H(w)` — the contaminating factor. 1.0 on a single-rule file.
    pub share_perplexity: f64,
    /// `A_eff / 2^H(w)` = the weighted geometric mean of within-rule `A_eff`.
    pub corrected: f64,
    /// `Σ_r 2^(H_r)` — fully share-independent; the shares do not appear.
    pub within_sum: f64,
    /// `(rule_id, events, within-rule A_eff)`.
    pub per_rule: Vec<(usize, u64, f64)>,
}
