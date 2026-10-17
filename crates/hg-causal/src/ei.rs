//! Effective Information / Causal Emergence calculator — the central
//! not-nothing measurement (not_nothing_v2_plan.md §6.5, resolved by
//! hypersim_build_ei_calculator.md).
//!
//! EI = H_int − H_noise over the row-stochastic WL-class transition
//! matrix; CE = EI_macro / EI_micro under a coarse-graining partition.
//! CE > 1 means the macro description has MORE causal power than the
//! micro (causal emergence). On bare substrate CE < 1 is the correct
//! null baseline — nothing here is tuned to exceed it.

use crate::ConstructorRegistry;
use ahash::AHashMap;
use hg_core::WlHash;
use serde::{Deserialize, Serialize};

/// Coarse-graining strategies (config-gated; `Optimal` is a declared
/// variant whose search is deliberately unimplemented today).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoarseStrategy {
    Structural,
    Energetic,
    Functional,
    /// Carrier vs substrate (`motif_class`) — the Experiment-6 partition.
    CarrierSubstrate,
    Optimal,
}

impl CoarseStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoarseStrategy::Structural => "structural",
            CoarseStrategy::Energetic => "energetic",
            CoarseStrategy::Functional => "functional",
            CoarseStrategy::CarrierSubstrate => "carrier_substrate",
            CoarseStrategy::Optimal => "optimal",
        }
    }
}

/// One EI/CE measurement. The cadence is part of the measurement's
/// definition: "EI_micro = 0.83 bits (window cadence, 128 steps)" is a
/// complete statement; the bare number is not.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EiResult {
    pub strategy: CoarseStrategy,
    pub step: u64,
    pub ei_micro: f64,
    pub ei_macro: f64,
    pub ce_coefficient: f64,
    /// Hoel determinism of the MICRO TPM: 1 − H_noise / log2(M).
    pub determinism: f64,
    /// Hoel degeneracy of the MICRO TPM: 1 − H_int / log2(M).
    pub degeneracy: f64,
    pub transition_count: u64,
    /// Steps between the two observation points of each transition.
    pub cadence_steps: u64,
    pub micro_classes: usize,
    pub macro_classes: usize,
    /// M(t): total registry size (live + dead classes) at emission — the
    /// novelty-accumulation curve. EI_micro stability ⟺ this saturating.
    pub class_registry_size: u32,
}

/// Runtime knobs (TOML `[causal_language]` + `[ei_coarse_graining]`).
#[derive(Clone, Copy, Debug)]
pub struct EiSettings {
    pub interval_steps: u64,
    pub min_transitions: u64,
    /// Steps between observations — the window length when transitions
    /// are recorded window-to-window. Constant within a run.
    pub cadence_steps: u64,
    pub try_structural: bool,
    pub try_energetic: bool,
    pub try_functional: bool,
    /// Carrier/substrate partition (Experiment 6). Off by default so existing
    /// (non-ALife) runs emit exactly the same EiResults as before.
    pub try_carrier_substrate: bool,
    pub try_optimal: bool,
}

impl Default for EiSettings {
    fn default() -> Self {
        Self {
            interval_steps: 1000,
            min_transitions: 1000,
            cadence_steps: 1,
            try_structural: true,
            try_energetic: true,
            try_functional: true,
            try_carrier_substrate: false,
            try_optimal: false,
        }
    }
}

/// Per-class exemplar features for the structural/energetic partitions,
/// refreshed each window from a witness node of the class.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClassFeatures {
    pub degree: u32,
    pub dominant_arity: u32,
    pub energy: f32,
    /// The class exemplar's `motif_class` from the active `MotifClassifier`
    /// (0 = substrate; nonzero = carrier under `HypergraphMotifClassifier`).
    /// Drives the carrier/substrate partition (Experiment 6). 0 by default, so
    /// non-ALife runs — where the classifier returns 0 everywhere — see no change.
    pub motif_class: u32,
}

/// Append-only WL-hash → dense-index registry plus sparse transition
/// counts. Classes are never capped or evicted.
#[derive(Default, Serialize, Deserialize)]
pub struct TpmAccumulator {
    index: AHashMap<WlHash, u32>,
    hashes: Vec<WlHash>,
    counts: AHashMap<(u32, u32), u64>,
    total: u64,
}

impl TpmAccumulator {
    /// Read-only raw-count export for independent diagnostic audits.
    pub fn diagnostic_counts(&self) -> Vec<(u32, u32, u64)> {
        let mut entries: Vec<_> = self.counts.iter().map(|(&(a,b),&n)|(a,b,n)).collect();
        entries.sort_unstable();
        entries
    }
    pub fn new() -> Self {
        Self::default()
    }

    fn index_of(&mut self, h: WlHash) -> u32 {
        *self.index.entry(h).or_insert_with(|| {
            self.hashes.push(h);
            (self.hashes.len() - 1) as u32
        })
    }

    /// Record one node's class transition (same NodeId at both
    /// observation points; self-transitions ARE recorded — they are the
    /// diagonal mass that makes stable dynamics register as
    /// high-determinism).
    pub fn record(&mut self, from: WlHash, to: WlHash) {
        let i = self.index_of(from);
        let j = self.index_of(to);
        *self.counts.entry((i, j)).or_insert(0) += 1;
        self.total += 1;
    }

    pub fn class_count(&self) -> usize {
        self.hashes.len()
    }

    pub fn total_transitions(&self) -> u64 {
        self.total
    }

    pub fn hashes(&self) -> &[WlHash] {
        &self.hashes
    }

    /// Dense class indices reordered CANONICALLY, sorted by WL hash.
    ///
    /// **Read this before indexing, ranking, permuting, or sampling classes.**
    /// Dense indices are assigned in `record()` call order (see `index_of`), and
    /// callers routinely drive those calls from `HashMap` iteration, which is
    /// `RandomState`-seeded and therefore differs BETWEEN PROCESSES. Consequences:
    ///
    /// * `ei_from_tpm` / `coarse_grain` are permutation-invariant (`coarse_grain`
    ///   indexes `group_of` by dense id), so **CE itself is unaffected** — which is
    ///   exactly why this hazard stays invisible until something else breaks.
    /// * Any statistic computed OVER indices — a permutation null, a "top-N most
    ///   active classes" ranking, a random class draw — silently becomes
    ///   process-dependent while CE continues to reproduce perfectly.
    ///
    /// The V-track hit this three times (validation_v1_findings.md §V3.1 and the F2
    /// harness): a permutation-null p-value and a registry ranking both drifted
    /// across processes while every coefficient stayed bit-identical. Route every
    /// index-order-dependent computation through this function.
    ///
    /// Note `normalized_rows()` EXCLUDES classes with no outgoing transitions, so a
    /// row's POSITION in that Vec is not its dense index — use the tuple's `u32`.
    pub fn canonical_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.hashes.len()).collect();
        order.sort_by_key(|&i| self.hashes[i].0);
        debug_assert!(
            order.windows(2).all(|w| self.hashes[w[0]].0 <= self.hashes[w[1]].0),
            "canonical_order must be sorted by WL hash"
        );
        order
    }

    /// Full dense count matrix over ALL classes (zero rows included —
    /// external verifiers apply the live-row exclusion themselves).
    pub fn counts_dense(&self) -> Vec<Vec<u64>> {
        let c = self.hashes.len();
        let mut m = vec![vec![0u64; c]; c];
        for (&(i, j), &n) in &self.counts {
            m[i as usize][j as usize] = n;
        }
        m
    }

    /// Row-normalized live rows: `(class index, row over ALL classes)`.
    /// Rows with zero observed count are excluded — a class never
    /// observed as a source has no defined row.
    pub fn normalized_rows(&self) -> Vec<(u32, Vec<f64>)> {
        let c = self.hashes.len();
        let mut row_sums = vec![0u64; c];
        for (&(i, _), &n) in &self.counts {
            row_sums[i as usize] += n;
        }
        let mut rows: AHashMap<u32, Vec<f64>> = AHashMap::new();
        for (&(i, j), &n) in &self.counts {
            let row = rows.entry(i).or_insert_with(|| vec![0.0; c]);
            row[j as usize] = n as f64 / row_sums[i as usize] as f64;
        }
        let mut out: Vec<(u32, Vec<f64>)> = rows.into_iter().collect();
        out.sort_by_key(|(i, _)| *i); // deterministic order
        out
    }
}

/// Shannon entropy in bits; 0·log2(0) = 0 by convention.
pub fn shannon_entropy(p: &[f64]) -> f64 {
    p.iter()
        .filter(|&&v| v > 0.0)
        .map(|&v| -v * v.log2())
        .sum()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EiComponents {
    pub ei: f64,
    pub h_int: f64,
    pub h_noise: f64,
    pub determinism: f64,
    pub degeneracy: f64,
    /// Number of live (source) rows.
    pub m: usize,
}

/// EI of a row-stochastic TPM (live rows only; column width may exceed
/// row count — never-source classes are still valid destinations).
///
/// M = 1 guard: a single-class system has nothing to discriminate —
/// EI = 0, determinism = degeneracy = 0 (log2(1) = 0).
pub fn ei_from_tpm(rows: &[Vec<f64>]) -> EiComponents {
    let m = rows.len();
    if m <= 1 {
        return EiComponents { m, ..EiComponents::default() };
    }
    let cols = rows[0].len();
    let mut interventional = vec![0.0f64; cols];
    for row in rows {
        for (j, &v) in row.iter().enumerate() {
            interventional[j] += v / m as f64;
        }
    }
    let h_int = shannon_entropy(&interventional);
    let h_noise = rows.iter().map(|r| shannon_entropy(r)).sum::<f64>() / m as f64;
    let log2m = (m as f64).log2();
    EiComponents {
        ei: h_int - h_noise,
        h_int,
        h_noise,
        determinism: 1.0 - h_noise / log2m,
        degeneracy: 1.0 - h_int / log2m,
        m,
    }
}

/// Coarse-grain live micro rows into a macro TPM under the partition
/// `group_of[class_index] -> group`, `k` groups.
///
/// T_macro[A][B] = (1/|A|) · Σ_{i∈A} Σ_{j∈B} T_micro[i][j] — the inner
/// sum aggregates destinations; the 1/|A| average over SOURCE-group
/// members (live members only) is what keeps each macro row summing
/// to 1. Naive row-summing breaks row-stochasticity.
pub fn coarse_grain(rows: &[(u32, Vec<f64>)], group_of: &[usize], k: usize) -> Vec<Vec<f64>> {
    let mut by_group: Vec<Vec<&Vec<f64>>> = vec![Vec::new(); k];
    for (ix, row) in rows {
        by_group[group_of[*ix as usize]].push(row);
    }
    let mut out = Vec::new();
    for members in by_group.iter().filter(|m| !m.is_empty()) {
        let mut mrow = vec![0.0f64; k];
        for row in members {
            for (j, &v) in row.iter().enumerate() {
                mrow[group_of[j]] += v;
            }
        }
        for v in &mut mrow {
            *v /= members.len() as f64;
        }
        out.push(mrow);
    }
    out
}

/// Structural: partition by topological type of the class exemplar —
/// (dominant incident-edge arity, degree band). Coarse domain knowledge
/// is the point of this strategy.
pub fn structural_partition(features: &[ClassFeatures]) -> (Vec<usize>, usize) {
    let mut groups: AHashMap<(u32, u32), usize> = AHashMap::new();
    let mut out = Vec::with_capacity(features.len());
    for f in features {
        let band = (f.degree + 1).ilog2(); // degree band: 0, 1-2, 3-6, ...
        let next = groups.len();
        out.push(*groups.entry((f.dominant_arity, band)).or_insert(next));
    }
    (out, groups.len().max(1))
}

/// Energetic: partition by energy quartile of class exemplars. On a
/// substrate with uniform energy this collapses to K = 1 → EI_macro = 0
/// → CE = 0. That is CORRECT behavior, not a failure.
pub fn energetic_partition(features: &[ClassFeatures]) -> (Vec<usize>, usize) {
    let mut energies: Vec<f32> = features.iter().map(|f| f.energy).collect();
    energies.sort_by(|a, b| a.total_cmp(b));
    let (lo, hi) = match (energies.first(), energies.last()) {
        (Some(&lo), Some(&hi)) => (lo, hi),
        _ => return (Vec::new(), 1),
    };
    if lo == hi {
        return (vec![0; features.len()], 1); // degenerate: uniform energy
    }
    let n = energies.len();
    let q = |frac: f64| energies[((n as f64 * frac) as usize).min(n - 1)];
    let (q1, q2, q3) = (q(0.25), q(0.5), q(0.75));
    let out = features
        .iter()
        .map(|f| {
            if f.energy < q1 {
                0
            } else if f.energy < q2 {
                1
            } else if f.energy < q3 {
                2
            } else {
                3
            }
        })
        .collect();
    (out, 4)
}

/// Functional: partition by the constructor registry's classification —
/// the scientifically central strategy, meaningful only because fidelity
/// is now a real measurement. Groups: 0 = confirmed constructor
/// (fidelity ≥ 0.9, cycles ≥ 10), 1 = reactant (fidelity < 0.5 with
/// cycles), 2 = candidate (other classes with cycles), 3 = inert.
pub fn functional_partition(
    hashes: &[WlHash],
    registry: &ConstructorRegistry,
) -> (Vec<usize>, usize) {
    let out = hashes
        .iter()
        .map(|h| match registry.get(h.0) {
            Some(r) => {
                let cycles = r.confirmed_cycles + r.degraded_cycles;
                if r.fidelity >= 0.9 && cycles >= 10 {
                    0
                } else if r.fidelity < 0.5 {
                    1
                } else {
                    2
                }
            }
            None => 3,
        })
        .collect();
    (out, 4)
}

/// Carrier/substrate: partition WL classes by `motif_class` — substrate
/// (`motif_class == 0`) vs carrier (nonzero) — the Experiment-6 "real
/// partition of the same micro state space" (scope §142). Indices are
/// COMPACT: when a group is absent (pre-replication there are no carrier
/// classes) the partition collapses to K = 1 → EI_macro = 0 → CE = 0, the
/// correct degenerate baseline that biology must lift 0 → >0.
pub fn carrier_substrate_partition(features: &[ClassFeatures]) -> (Vec<usize>, usize) {
    let mut map: AHashMap<usize, usize> = AHashMap::new();
    let mut out = Vec::with_capacity(features.len());
    for f in features {
        let raw = (f.motif_class != 0) as usize; // 0 = substrate, 1 = carrier
        let next = map.len();
        out.push(*map.entry(raw).or_insert(next));
    }
    (out, map.len().max(1))
}

/// Compute one `EiResult` per enabled strategy. Empty below the
/// minimum-transition gate: fewer than `min_transitions` observations
/// simply isn't knowledge yet (certainty table: Empirical needs ≥ 1000).
pub fn compute_ei_results(
    tpm: &TpmAccumulator,
    features: &AHashMap<WlHash, ClassFeatures>,
    registry: &ConstructorRegistry,
    settings: &EiSettings,
    step: u64,
) -> Vec<EiResult> {
    if tpm.total_transitions() < settings.min_transitions {
        return Vec::new();
    }
    let rows = tpm.normalized_rows();
    let bare: Vec<Vec<f64>> = rows.iter().map(|(_, r)| r.clone()).collect();
    let micro = ei_from_tpm(&bare);

    let feat_vec: Vec<ClassFeatures> = tpm
        .hashes()
        .iter()
        .map(|h| features.get(h).copied().unwrap_or_default())
        .collect();

    let strategies: [(CoarseStrategy, bool); 5] = [
        (CoarseStrategy::Structural, settings.try_structural),
        (CoarseStrategy::Energetic, settings.try_energetic),
        (CoarseStrategy::Functional, settings.try_functional),
        (CoarseStrategy::CarrierSubstrate, settings.try_carrier_substrate),
        (CoarseStrategy::Optimal, settings.try_optimal),
    ];
    let mut out = Vec::new();
    for (strategy, enabled) in strategies {
        if !enabled {
            continue;
        }
        let (group_of, k) = match strategy {
            CoarseStrategy::Structural => structural_partition(&feat_vec),
            CoarseStrategy::Energetic => energetic_partition(&feat_vec),
            CoarseStrategy::Functional => functional_partition(tpm.hashes(), registry),
            CoarseStrategy::CarrierSubstrate => carrier_substrate_partition(&feat_vec),
            // Exhaustive partition search: declared, deliberately not
            // implemented (try_optimal defaults to false).
            CoarseStrategy::Optimal => continue,
        };
        let macro_rows = coarse_grain(&rows, &group_of, k);
        let mac = ei_from_tpm(&macro_rows);
        // Guarded CE: near-zero EI_micro (white noise) yields CE = 0,
        // not infinity — "CE < 1 on degenerate substrates" stays
        // assertable.
        let ce = if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 };
        out.push(EiResult {
            strategy,
            step,
            ei_micro: micro.ei,
            ei_macro: mac.ei,
            ce_coefficient: ce,
            determinism: micro.determinism,
            degeneracy: micro.degeneracy,
            transition_count: tpm.total_transitions(),
            cadence_steps: settings.cadence_steps,
            micro_classes: micro.m,
            macro_classes: mac.m,
            class_registry_size: tpm.class_count() as u32,
        });
    }
    out
}

/// Verifier-facing dump of the raw TPM state (verify_ei.py's documented
/// JSON shape): class names, dense counts, cadence, and the active
/// partitions as group-index arrays — everything an independent
/// implementation needs to recompute EI/CE from scratch.
#[derive(Serialize)]
pub struct EiDump {
    pub classes: Vec<String>,
    pub counts: Vec<Vec<u64>>,
    pub cadence_steps: u64,
    pub partitions: std::collections::BTreeMap<String, Vec<usize>>,
}

/// The enabled strategies' partitions over the CURRENT class registry —
/// the same group arrays compute_ei_results uses.
pub fn partitions_for(
    tpm: &TpmAccumulator,
    features: &AHashMap<WlHash, ClassFeatures>,
    registry: &ConstructorRegistry,
    settings: &EiSettings,
) -> std::collections::BTreeMap<String, Vec<usize>> {
    let feat_vec: Vec<ClassFeatures> = tpm
        .hashes()
        .iter()
        .map(|h| features.get(h).copied().unwrap_or_default())
        .collect();
    let mut out = std::collections::BTreeMap::new();
    if settings.try_structural {
        out.insert("structural".into(), structural_partition(&feat_vec).0);
    }
    if settings.try_energetic {
        out.insert("energetic".into(), energetic_partition(&feat_vec).0);
    }
    if settings.try_functional {
        out.insert(
            "functional".into(),
            functional_partition(tpm.hashes(), registry).0,
        );
    }
    if settings.try_carrier_substrate {
        out.insert(
            "carrier_substrate".into(),
            carrier_substrate_partition(&feat_vec).0,
        );
    }
    out
}
