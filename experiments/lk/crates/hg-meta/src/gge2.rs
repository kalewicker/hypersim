//! G-GE2 blob-panel instruments (GE plan §5) — pure logic, no engine dep.
//!
//! The panel's alphabet and concentration rows as total functions over
//! caller-built histograms. Same split as [`crate::gge1`]: the pure
//! classifier lives here, store walks and burn-in slicing belong to the
//! driver. Every instrument returns `None` on empty input (no verdict
//! from no data — zeros are reported, never omitted, is the driver's
//! job via the audit counters).
//!
//! Row map (§5, in order):
//! - A_eff + |A| .............. [`AlphabetHist::a_eff`] / [`AlphabetHist::raw`]
//! - recurrent fraction / singleton share ... [`AlphabetHist::recurrent_fraction`] / [`AlphabetHist::singleton_share`]
//! - novelty both abscissae .... WIRED in the R09 driver (RCF vs epoch + vs
//!   event); no new function here — engagement proof is slice 2.
//! - burn-in ................... [`burn_in_cut`] (first 10% unscored)
//! - site entropy .............. [`site_entropy`] + [`pump_verdict`] (M2)
//! - dual scope ................ driver discipline (`label < META_BASE`
//!   filter per `expr_r10.rs` precedent); no new function — slice 2.
//! - regime + abort ............ [`crate::gge1::classify_trajectory`] +
//!   `hg_eval::campaign::AbortGate` (reuse, both landed); slice 2 cites.
//! - edge trajectory ........... R09 records per-step edge counts; slice 2.

use std::collections::BTreeMap;

/// S1 signature: radius-1 neighborhood key — own label plus the SORTED
/// multiset of neighbor labels. Radius 1 IS the pre-registered radius
/// (§5: "S1 signatures at the rule's pre-registered radius"). Sorting
/// makes the key order-invariant; duplicates count separately (a neighbor
/// class with two members is structurally more constrained than one —
/// same multiset discipline as the bisimulation signatures).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct S1(pub u32, pub Vec<u32>);

/// Build an S1 key from one node's label and its neighbor labels.
pub fn s1_signature(label: u32, mut neighbor_labels: Vec<u32>) -> S1 {
    neighbor_labels.sort_unstable();
    S1(label, neighbor_labels)
}

/// Alphabet histogram over S1 keys. The driver feeds post-burn-in symbols
/// only (row 2 is post-burn-in by pre-registration); burn-in slicing is
/// [`burn_in_cut`].
#[derive(Clone, Debug, Default)]
pub struct AlphabetHist {
    counts: BTreeMap<S1, u64>,
}

impl AlphabetHist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&mut self, sym: S1) {
        *self.counts.entry(sym).or_insert(0) += 1;
    }

    /// Total symbol observations (mass).
    pub fn mass(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Raw alphabet size |A| — the second witness. Verdicts NEVER rest on
    /// this alone (B6: serial numbers maximize it with zero recurrence).
    pub fn raw(&self) -> usize {
        self.counts.len()
    }

    fn shannon(&self) -> Option<f64> {
        let total = self.mass();
        if total == 0 {
            return None;
        }
        let mut h = 0.0;
        for &c in self.counts.values() {
            let p = c as f64 / total as f64;
            h -= p * p.ln();
        }
        Some(h)
    }

    /// Effective alphabet size A_eff = exp(Shannon). Verdicts rest HERE:
    /// a distribution dominated by one symbol with a tail of singletons
    /// reports a small A_eff against a large |A| — the B6 trap, priced.
    pub fn a_eff(&self) -> Option<f64> {
        self.shannon().map(|h| h.exp())
    }

    /// Share of mass on symbols seen ≥2 times (post-burn-in). Treadmill →
    /// 0, bounded churn → ~1: the row-2 separator.
    pub fn recurrent_fraction(&self) -> Option<f64> {
        let total = self.mass();
        if total == 0 {
            return None;
        }
        let rec: u64 = self.counts.values().filter(|&&c| c >= 2).sum();
        Some(rec as f64 / total as f64)
    }

    /// Share of mass on symbols seen exactly once. Treadmill → 1.
    pub fn singleton_share(&self) -> Option<f64> {
        let total = self.mass();
        if total == 0 {
            return None;
        }
        let sing: u64 = self.counts.values().filter(|&&c| c == 1).sum();
        Some(sing as f64 / total as f64)
    }
}

/// Shannon entropy over per-site meta-rewrite event counts (M2). The
/// caller passes one count per site — site = a region root (driver key);
/// any map or counter vec works since only the counts are read. The ERE
/// spec phrases this over edit-site WL class — the root-identity
/// substitution is recorded in `docs/gge2_audit.md` with its rationale
/// (roots ARE the meta-level sites; WL-classing them adds a second
/// instrument's failure modes to the pump detector).
pub fn site_entropy(counts: impl IntoIterator<Item = u64>) -> Option<f64> {
    let cs: Vec<u64> = counts.into_iter().collect();
    let total: u64 = cs.iter().sum();
    if total == 0 {
        return None;
    }
    let mut h = 0.0;
    for &c in &cs {
        if c == 0 {
            continue;
        }
        let p = c as f64 / total as f64;
        h -= p * p.ln();
    }
    Some(h)
}

/// Pre-committed pump floor (G-GE2 pre-registration — chosen before any
/// run exists, recorded in `docs/gge2_audit.md`): below this many events
/// the entropy is noise, not a verdict.
pub const PUMP_EVENT_FLOOR: u64 = 20;
/// Pre-committed entropy ceiling for the pump verdict: at ≤0.5 nats the
/// distribution is concentrated on essentially one site (a 90/10 split
/// already reads 0.33).
pub const PUMP_ENTROPY_CEIL: f64 = 0.5;

/// M2 verdict, pre-committed interpretation verbatim: entropy collapsing
/// toward 0 while events stay high = the serial point-source pump
/// recurring at the meta level — a finding about energy metering, not
/// about rule encoding.
pub fn pump_verdict(entropy: f64, events: u64) -> bool {
    events >= PUMP_EVENT_FLOOR && entropy <= PUMP_ENTROPY_CEIL
}

/// Burn-in cut: first 10% of steps unscored (§5, M1-mandated). Returns the
/// index of the first SCORED step.
pub fn burn_in_cut(total_steps: usize) -> usize {
    total_steps / 10
}

/// Novelty per kernel EVENT over a cumulative-pair series (row 3 verdict
/// axis): marginal never-before-seen classes per kernel mutation event
/// across the series. `None` when the series is too short or no events
/// elapsed. The driver passes the scored range (post-[`burn_in_cut`]) so
/// transient inflation never enters the rate; whole-series input answers
/// the un-scored question instead — both are legitimate, never mixed.
pub fn novel_per_event(novel_cumulative: &[usize], events_cumulative: &[usize]) -> Option<f64> {
    if novel_cumulative.len() < 2 || events_cumulative.len() < 2 {
        return None;
    }
    let dn = novel_cumulative.last()? - novel_cumulative.first()?;
    let de = events_cumulative.last()? - events_cumulative.first()?;
    if de == 0 {
        return None;
    }
    Some(dn as f64 / de as f64)
}

/// Scored novelty gain over a cumulative curve (row 4): whole-run total
/// minus the burn-in cut value. Matches the R09 driver's post-burn-in
/// math (curve[last] − curve[EPOCHS/10]) exactly.
pub fn scored_gain(cumulative: &[usize]) -> usize {
    if cumulative.is_empty() {
        return 0;
    }
    let cut = burn_in_cut(cumulative.len());
    cumulative[cumulative.len() - 1] - cumulative[cut.min(cumulative.len() - 1)]
}

/// One epoch of substrate observation: dual-scope counts plus both S1
/// histograms (rows 1, 2, 6, 8 inputs). Built by [`observe_substrate`].
#[derive(Clone, Debug, Default)]
pub struct SubstratePanel {
    /// All live nodes (meta band included).
    pub nodes_incl: usize,
    /// Substrate nodes only (`label < META_BASE`).
    pub nodes_excl: usize,
    /// All live hyperedges.
    pub edges_total: usize,
    /// Hyperedges whose members are ALL substrate nodes.
    pub edges_excl: usize,
    pub hist_incl: AlphabetHist,
    pub hist_excl: AlphabetHist,
}

/// Walk the live store once, keying every node by its S1 signature
/// (own label + sorted co-member labels across incident hyperedges),
/// into meta-included and meta-excluded histograms. The excluded scope
/// ignores meta-band nodes ENTIRELY (a substrate node's excluded key
/// does not name meta neighbors) — the metric is over the excluded set,
/// per M1. Insertion order cannot leak into any verdict (histograms are
/// `BTreeMap`s; neighbor lists are sorted by [`s1_signature`]).
pub fn observe_substrate(store: &hg_core::HypergraphStore) -> SubstratePanel {
    let mut p = SubstratePanel::default();
    for (id, data) in store.nodes() {
        let label = data.label.0;
        let mut neighbors_incl = Vec::new();
        let mut neighbors_excl = Vec::new();
        for eid in store.edges_incident_to(id) {
            let Some(e) = store.get_edge(*eid) else {
                continue;
            };
            for m in &e.nodes {
                if *m == id {
                    continue;
                }
                if let Some(nd) = store.get_node(*m) {
                    neighbors_incl.push(nd.label.0);
                    if nd.label.0 < crate::schema::META_BASE {
                        neighbors_excl.push(nd.label.0);
                    }
                }
            }
        }
        p.hist_incl.observe(s1_signature(label, neighbors_incl));
        p.nodes_incl += 1;
        if label < crate::schema::META_BASE {
            p.hist_excl.observe(s1_signature(label, neighbors_excl));
            p.nodes_excl += 1;
        }
    }
    for (_, e) in store.edges() {
        p.edges_total += 1;
        if e.nodes.iter().all(|m| {
            store
                .get_node(*m)
                .is_some_and(|d| d.label.0 < crate::schema::META_BASE)
        }) {
            p.edges_excl += 1;
        }
    }
    p
}

/// One epoch of the panel as the driver reports it (G-GE2 rows 1–6, 8;
/// row 7 regime is trajectory-level, row 4 burn-in is the `scored` flag).
/// All-`None`/zero rows are measurements, never omissions.
#[derive(Clone, Debug)]
pub struct PanelEpoch {
    pub epoch: usize,
    /// False inside the burn-in window — row excluded from verdicts.
    pub scored: bool,
    pub nodes_incl: usize,
    pub nodes_excl: usize,
    pub edges_total: usize,
    pub edges_excl: usize,
    pub a_eff_incl: Option<f64>,
    pub a_eff_excl: Option<f64>,
    pub raw_incl: usize,
    pub raw_excl: usize,
    pub recurrent_incl: Option<f64>,
    pub recurrent_excl: Option<f64>,
    pub singleton_incl: Option<f64>,
    pub singleton_excl: Option<f64>,
    pub rcf_cumulative: usize,
    pub kernel_events_cumulative: usize,
    pub site_entropy: Option<f64>,
    pub site_events: u64,
    pub pump: bool,
}

impl PanelEpoch {
    /// Assemble a row from one epoch's observations. `sites` carries one
    /// meta-rewrite count per region root for the window just ending
    /// (driver-accumulated from [`crate::epoch::EpochEvent::site`]).
    pub fn from_observed(
        epoch: usize,
        total_epochs: usize,
        panel: &SubstratePanel,
        rcf_cumulative: usize,
        kernel_events_cumulative: usize,
        sites: impl IntoIterator<Item = u64>,
    ) -> Self {
        let counts: Vec<u64> = sites.into_iter().collect();
        let site_events: u64 = counts.iter().sum();
        let entropy = site_entropy(counts.iter().copied());
        Self {
            epoch,
            scored: epoch >= burn_in_cut(total_epochs),
            nodes_incl: panel.nodes_incl,
            nodes_excl: panel.nodes_excl,
            edges_total: panel.edges_total,
            edges_excl: panel.edges_excl,
            a_eff_incl: panel.hist_incl.a_eff(),
            a_eff_excl: panel.hist_excl.a_eff(),
            raw_incl: panel.hist_incl.raw(),
            raw_excl: panel.hist_excl.raw(),
            recurrent_incl: panel.hist_incl.recurrent_fraction(),
            recurrent_excl: panel.hist_excl.recurrent_fraction(),
            singleton_incl: panel.hist_incl.singleton_share(),
            singleton_excl: panel.hist_excl.singleton_share(),
            rcf_cumulative,
            kernel_events_cumulative,
            site_entropy: entropy,
            site_events,
            pump: entropy.is_some_and(|h| pump_verdict(h, site_events)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// B6 treadmill archetype: 100 serial-number symbols, zero recurrence.
    fn treadmill() -> AlphabetHist {
        let mut h = AlphabetHist::new();
        for i in 0..100u32 {
            h.observe(s1_signature(i, vec![i + 1]));
        }
        h
    }

    /// Bounded-churn archetype: 3 symbols reused 50× each.
    fn churn() -> AlphabetHist {
        let mut h = AlphabetHist::new();
        for _ in 0..50 {
            for s in 0..3u32 {
                h.observe(s1_signature(s, vec![0]));
            }
        }
        h
    }

    #[test]
    fn treadmill_raw_equals_effective_with_no_recurrence() {
        let h = treadmill();
        assert_eq!(h.raw(), 100);
        let eff = h.a_eff().expect("nonempty");
        assert!(
            (eff - 100.0).abs() < 1e-6,
            "uniform 100 → A_eff 100, got {eff}"
        );
        assert_eq!(h.recurrent_fraction(), Some(0.0));
        assert_eq!(h.singleton_share(), Some(1.0));
    }

    #[test]
    fn verdicts_on_a_eff_not_raw_the_b6_trap() {
        // One dominant + a tail of singletons: raw counts 11 "types",
        // effective counts fewer than 2. A verdict on |A| would call this
        // diverse; the panel verdict on A_eff calls it concentrated.
        let mut h = AlphabetHist::new();
        for _ in 0..90 {
            h.observe(s1_signature(0, vec![0]));
        }
        for i in 1..=10u32 {
            h.observe(s1_signature(i, vec![i]));
        }
        assert_eq!(h.raw(), 11);
        let eff = h.a_eff().expect("nonempty");
        assert!(
            eff < 2.0,
            "dominated + tail must read concentrated, got {eff}"
        );
        assert!(h.singleton_share().expect("nonempty") > 0.0);
        assert!(h.recurrent_fraction().expect("nonempty") > 0.85);
    }

    #[test]
    fn churn_recurs_and_treadmill_does_not() {
        let c = churn();
        assert_eq!(c.raw(), 3);
        let eff = c.a_eff().expect("nonempty");
        assert!((eff - 3.0).abs() < 1e-6, "uniform 3 → A_eff 3, got {eff}");
        assert_eq!(c.recurrent_fraction(), Some(1.0));
        assert_eq!(c.singleton_share(), Some(0.0));
    }

    #[test]
    fn empty_histogram_has_no_verdict() {
        let h = AlphabetHist::new();
        assert_eq!(h.mass(), 0);
        assert_eq!(h.raw(), 0);
        assert!(h.a_eff().is_none());
        assert!(h.recurrent_fraction().is_none());
        assert!(h.singleton_share().is_none());
        assert!(site_entropy(Vec::<u64>::new()).is_none());
    }

    #[test]
    fn site_pump_fires_on_concentration_not_volume() {
        // Single site, 100 events: entropy 0 → PUMP.
        let pump: BTreeMap<u64, u64> = [(7u64, 100u64)].into_iter().collect();
        let h = site_entropy(pump.values().copied()).expect("nonempty");
        assert_eq!(h, 0.0);
        assert!(pump_verdict(h, 100));
        // Ten even sites, same volume: entropy ln10 → no pump.
        let diffuse: BTreeMap<u64, u64> = (0..10u64).map(|s| (s, 10u64)).collect();
        let hd = site_entropy(diffuse.values().copied()).expect("nonempty");
        assert!((hd - 10f64.ln()).abs() < 1e-9);
        assert!(!pump_verdict(hd, 100));
        // Concentrated but thin: below the floor → no verdict of pump.
        assert!(!pump_verdict(0.0, PUMP_EVENT_FLOOR - 1));
    }

    #[test]
    fn burn_in_cuts_first_tenth() {
        assert_eq!(burn_in_cut(40), 4);
        assert_eq!(burn_in_cut(100), 10);
        assert_eq!(burn_in_cut(9), 0);
    }

    #[test]
    fn s1_key_is_neighbor_order_invariant_with_multiplicities() {
        let a = s1_signature(3, vec![1, 2, 2]);
        let b = s1_signature(3, vec![2, 1, 2]);
        assert_eq!(a, b);
        // ...but multiplicities matter: two members ≠ one.
        assert_ne!(a, s1_signature(3, vec![1, 2]));
        assert_ne!(a, s1_signature(4, vec![1, 2, 2]));
    }

    /// Row 6+8 engagement on a REAL store (not synthetic histograms): one
    /// meta-band node shares an edge with the substrate. Included scope
    /// counts 4 nodes / 2 edges; excluded scope counts 3 / 1; the
    /// border node's excluded key drops its meta neighbor.
    #[test]
    fn walk_dual_scope_separates_meta_band() {
        use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
        use smallvec::smallvec;
        let mut s = HypergraphStore::new();
        let a = s.add_node(NodeData::new(Label(0), 0));
        let b = s.add_node(NodeData::new(Label(1), 0));
        let c = s.add_node(NodeData::new(Label(2), 0));
        let m = s.add_node(NodeData::new(Label(crate::schema::META_BASE + 1), 0));
        s.add_edge(HyperEdge::new(Label(0), smallvec![a, b, c]));
        s.add_edge(HyperEdge::new(Label(0), smallvec![c, m]));
        let p = observe_substrate(&s);
        assert_eq!((p.nodes_incl, p.nodes_excl), (4, 3));
        assert_eq!((p.edges_total, p.edges_excl), (2, 1));
        assert_eq!(p.hist_incl.mass(), 4);
        assert_eq!(p.hist_excl.mass(), 3);
        // Border node c: included key names the meta neighbor, excluded
        // key does not — the M1 discipline at the key level.
        let m_label = crate::schema::META_BASE + 1;
        assert!(p
            .hist_incl
            .counts
            .contains_key(&s1_signature(2, vec![0, 1, m_label])));
        assert!(p
            .hist_excl
            .counts
            .contains_key(&s1_signature(2, vec![0, 1])));
        assert!(!p
            .hist_excl
            .counts
            .contains_key(&s1_signature(2, vec![0, 1, m_label])));
    }

    /// Row 8 engagement (B3 shape): flat nodes + growing edges. The SAME
    /// reused classifier reads Balanced on the node series and non-quiet
    /// on the edge series — a node-only panel would report nothing while
    /// the edge row fires. Reuse, not reimplementation (§5).
    #[test]
    fn edge_panel_decouples_from_node_panel() {
        use crate::gge1::{classify_trajectory, DynClass};
        let flat = vec![100usize; 101];
        let growing: Vec<usize> = (0..101).map(|i| 100 + 5 * i).collect();
        assert!(matches!(classify_trajectory(&flat), DynClass::Balanced));
        assert!(
            !matches!(
                classify_trajectory(&growing),
                DynClass::Balanced | DynClass::Saturating { .. }
            ),
            "edge-only growth must not read quiet"
        );
    }

    /// Row 3 engagement (binary_edge illusion): +10 classes/step reads
    /// steep vs STEP and 0.01/class-per-event flat vs EVENT. The panel
    /// reports both; the verdict axis is per-event.
    #[test]
    fn step_event_divergence_verdict_on_per_event() {
        let novel: Vec<usize> = (0..=10).map(|i| 10 * i).collect();
        let events: Vec<usize> = (0..=10).map(|i| 1000 * i).collect();
        let per_step = (novel[10] - novel[0]) as f64 / 10.0;
        let per_event = novel_per_event(&novel, &events).expect("events elapsed");
        assert_eq!(per_step, 10.0);
        assert!((per_event - 0.01).abs() < 1e-12);
        assert!(per_event < 0.1, "verdict axis flat while step axis steep");
        assert!(novel_per_event(&[5usize], &[5usize]).is_none());
        assert!(novel_per_event(&[0usize, 5], &[7usize, 7]).is_none());
    }

    /// Row 4 engagement (churn 80%-transient): 8 of 10 classes arrive in
    /// the first 4 of 40 epochs. Whole-run counts 10; the scored gain
    /// counts 2 — burn-in changes the verdict, which is what makes it an
    /// instrument rather than a ritual.
    #[test]
    fn burn_in_transient_excluded_from_scored_gain() {
        let mut cum = vec![0usize; 41];
        for (i, c) in cum.iter_mut().enumerate() {
            *c = if i <= 4 { 2 * i } else { 8 + (i - 4) / 18 };
        }
        assert_eq!(cum[40], 10);
        assert_eq!(scored_gain(&cum), 2);
        assert!(scored_gain(&cum) < cum[40] / 2);
        assert_eq!(scored_gain(&[]), 0);
    }

    /// Row assembly: burn-in flag, dual counts, site pump state — one
    /// constructor, no silent defaults.
    #[test]
    fn panel_row_marks_burn_in_and_reports_pump_state() {
        use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
        use smallvec::smallvec;
        let mut s = HypergraphStore::new();
        let a = s.add_node(NodeData::new(Label(0), 0));
        let b = s.add_node(NodeData::new(Label(1), 0));
        s.add_edge(HyperEdge::new(Label(0), smallvec![a, b]));
        let panel = observe_substrate(&s);
        let early = PanelEpoch::from_observed(0, 40, &panel, 3, 5, [3u64, 1u64]);
        assert!(!early.scored);
        assert_eq!((early.nodes_incl, early.nodes_excl), (2, 2));
        assert_eq!((early.edges_total, early.edges_excl), (1, 1));
        assert_eq!(early.site_events, 4);
        assert!(!early.pump, "4 events under the floor is not a pump");
        let late = PanelEpoch::from_observed(40, 40, &panel, 3, 5, [25u64]);
        assert!(late.scored);
        assert!(late.pump, "25 single-site events is the M2 pump");
        assert_eq!(late.site_entropy, Some(0.0));
    }
}
