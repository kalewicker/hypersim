//! Epoch-driven recompile: the A2/A3 dynamics loop.
//!
//! Each epoch of E steps:
//!   1. read the current rule regions from the world snapshot,
//!   2. apply ONE single-edit mutation to a uniformly chosen parent
//!      (the G-R1 kernel — deterministic given the run's ChaCha stream),
//!   3. validate; quarantine or install the child region into the store,
//!   4. recompile the whole visible delta under the arm's protection level,
//!   5. hand the engine its new rule table for the next E steps.
//!
//! Determinism: the mutation RNG is a keyed per-epoch stream derived from
//! the run seed and epoch index (never the engine's stream); everything
//! else is content-ordered.

use crate::economy::{IncomePolicy, RegionLedger};
use crate::graph_source::GraphRuleSource;
use crate::mutate::{mutate, mutate_op, Applied, MutationRng, Op};
use crate::protect::compile_delta;
use crate::rcf::rcf_hash;
use crate::schema::{find_rule_roots, read_region_with_footprint, CandidateRule};
use crate::validate::{validate, PriorityObservation};
use hg_core::NodeId;
use hg_observe::{Observer, RewriteEvent};
use hg_rules::source::RuleSource;
use rustc_hash::{FxHashMap, FxHashSet};

/// What happened in one epoch transition.
#[derive(Clone, Copy, Debug, Default)]
pub struct EpochEvent {
    pub mutated: bool,
    pub quarantined: bool,
    pub installed: bool,
    /// Region root chosen as the mutation parent this epoch, when a sampled
    /// edit was applied (G-GE2 row 5 input: the meta-rewrite SITE).
    /// Counts whether the child validates or quarantines — the edit happened
    /// at that site either way. None when no parent was picked (empty
    /// kernel) or the draw was Nothing (no edit happened).
    pub site: Option<NodeId>,
    /// RCF class of the child if it installed; first-time classes are novel.
    pub novel_class: bool,
    pub rules_installed: usize,
    /// `None` when no economy rode along; `Some` (possibly all-zeros with
    /// `balanced: true`) when one did — including the FEE_ZERO control,
    /// where the audit existing at all IS the inertness evidence.
    pub economy: Option<EconomyAudit>,
}

/// Per-epoch economy audit (§10.6, SB-GE5 resolution). Every field is a
/// count or a sum — no rates, no inference — so the driver can assert
/// conservation directly: `balanced` means every live ledger satisfies
/// `0 <= free <= total` AND `Σ live totals == total_issued`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EconomyAudit {
    /// Ledgers alive after this transition.
    pub alive: usize,
    /// New ledgers opened (one per installed child).
    pub opened: usize,
    /// Ledgers dropped (region extinct — root no longer installed).
    pub dropped: usize,
    /// Firings paid this epoch (bulk `try_fire` semantics, §10.1).
    pub paid: u64,
    /// Firings refused whole this epoch (dormant, no debt).
    pub refused: u64,
    /// Firings attributed to roots with no ledger (stale mapping —
    /// a driver bug if ever nonzero; never silently absorbed).
    pub unattributed: u64,
    /// Σ over live ledgers of `(total - free)`: currently encumbered.
    pub encumbered: i64,
    /// Σ over live ledgers of `total`: must equal lifetime opens minus
    /// drops; drift here is a leak, not a parameter.
    pub total_issued: i64,
    pub balanced: bool,
}

/// The epoch-level economy state (SB-GE5 wiring). Ledgers are keyed by
/// region ROOT `NodeId` — the one identity that survives across epochs
/// for a living region (installs only add nodes; a surviving root keeps
/// its id) and vanishes exactly when the region goes extinct, which is
/// when its ledger must drop.
///
/// What this owns (and only this): opening a ledger for each installed
/// child, debiting attributed firings at the end-of-epoch boundary,
/// aging + income for survivors, dropping the extinct, and the audit.
/// What it does NOT own: per-step forwarding (that is `FeePolicy`'s job
/// at the `RuleSelectionPolicy` seam) and per-rule fire attribution
/// (that is the driver's, via [`FireCounter`] below — the engine reports
/// `accepted_rewrites` as a step total only, so attribution MUST come
/// from the observer chain, never from stats).
#[derive(Clone, Debug, Default)]
pub struct EpochEconomy {
    pub income: IncomePolicy,
    ledgers: FxHashMap<NodeId, RegionLedger>,
    total_issued: i64,
    lifetime_paid: u64,
    lifetime_refused: u64,
    lifetime_unattributed: u64,
}

impl EpochEconomy {
    pub fn new(income: IncomePolicy) -> Self {
        Self {
            income,
            ledgers: FxHashMap::default(),
            total_issued: 0,
            lifetime_paid: 0,
            lifetime_refused: 0,
            lifetime_unattributed: 0,
        }
    }

    /// The A0 control: every fee zero. The transition takes this as
    /// "economy consulted, inert" — no ledger opens, no debits, audit
    /// all-zeros with `balanced: true`.
    pub fn is_fee_zero(&self) -> bool {
        self.income == IncomePolicy::FEE_ZERO
    }

    pub fn alive(&self) -> usize {
        self.ledgers.len()
    }

    /// Ledger enrolled for a live root, if any. Missing means outside the
    /// economy (the `FeePolicy` passes such rules through) — the EXP-R09
    /// driver aligns these snapshots by `rule_id` when building each
    /// epoch's policy.
    pub fn ledger(&self, root: &NodeId) -> Option<&RegionLedger> {
        self.ledgers.get(root)
    }

    /// Open a ledger for a newly installed region. A surviving root is
    /// never re-opened (installs mint fresh roots, so presence means a
    /// driver double-install — keep the aged ledger, don't wipe it).
    /// Returns true iff a ledger was opened.
    pub fn open(&mut self, root: NodeId) -> bool {
        if self.ledgers.contains_key(&root) {
            return false;
        }
        let endowment = self.income.endowment;
        self.ledgers.insert(root, RegionLedger::new(endowment));
        self.total_issued += endowment;
        true
    }

    /// End-of-epoch debit (§10.6): `fired` maps region roots to accepted
    /// firing counts attributed by the driver's [`FireCounter`]. Bulk form
    /// of `RegionLedger::try_fire` — each firing is paid whole or refused
    /// whole, no partial execution, no debt. Counts for unknown roots are
    /// reported as `unattributed`, never absorbed into a stranger's ledger.
    pub fn debit(&mut self, fired: &FxHashMap<NodeId, u64>) -> (u64, u64, u64) {
        let (mut paid, mut refused, mut unattributed) = (0u64, 0, 0);
        let cost = self.income.cost;
        for (root, n) in fired {
            let Some(ledger) = self.ledgers.get_mut(root) else {
                unattributed += *n;
                continue;
            };
            // Bulk try_fire: the first `affordable` firings pay, the rest
            // refuse. With `cost > 0`, `paid * cost <= free` by
            // construction, so the subtraction cannot overflow and `free`
            // never goes negative. Zero/negative cost pays every firing
            // with the balance untouched (matching `is_bankrupt`, which is
            // always false there).
            let (p, r) = if cost <= 0 {
                (*n, 0)
            } else {
                let affordable = (ledger.free / cost).max(0) as u64;
                let p = (*n).min(affordable);
                (p, n.saturating_sub(p))
            };
            if cost > 0 {
                ledger.free -= (p as i64).saturating_mul(cost).min(ledger.free);
            }
            ledger.fired_total = ledger.fired_total.saturating_add(p);
            ledger.refused_total = ledger.refused_total.saturating_add(r);
            ledger.refused_this_epoch = ledger
                .refused_this_epoch
                .saturating_add(r.min(u32::MAX as u64) as u32);
            paid += p;
            refused += r;
        }
        self.lifetime_paid += paid;
        self.lifetime_refused += refused;
        self.lifetime_unattributed += unattributed;
        (paid, refused, unattributed)
    }

    /// Income + aging for every ledger whose root is still installed;
    /// drop the extinct (subtracting their totals from issuance so the
    /// conservation check stays exact across turnover).
    pub fn settle(&mut self, live: &FxHashSet<NodeId>) -> (usize, usize) {
        let (mut kept, mut dropped) = (0usize, 0usize);
        let mut dead_totals = 0i64;
        let income = self.income; // copy out: retain's closure can't borrow self
        self.ledgers.retain(|root, ledger| {
            if live.contains(root) {
                ledger.end_epoch(&income);
                kept += 1;
                true
            } else {
                dead_totals += ledger.total;
                dropped += 1;
                false
            }
        });
        self.total_issued -= dead_totals;
        (kept, dropped)
    }

    /// The audit: per-region bounds plus cross-epoch issuance balance.
    pub fn audit(
        &self,
        opened: usize,
        dropped: usize,
        paid: u64,
        refused: u64,
        unattributed: u64,
    ) -> EconomyAudit {
        let mut encumbered = 0i64;
        let mut live_totals = 0i64;
        let mut bounds_ok = true;
        for ledger in self.ledgers.values() {
            if ledger.free < 0 || ledger.free > ledger.total {
                bounds_ok = false;
            }
            encumbered += ledger.total - ledger.free;
            live_totals += ledger.total;
        }
        EconomyAudit {
            alive: self.ledgers.len(),
            opened,
            dropped,
            paid,
            refused,
            unattributed,
            encumbered,
            total_issued: self.total_issued,
            balanced: bounds_ok && live_totals == self.total_issued,
        }
    }
}

/// Per-rule fire attribution for the driver (SB-GE5's missing link at
/// the engine side): the engine's `StepStats.accepted_rewrites` is a step
/// TOTAL, so end-of-epoch debit by region needs these per-`rule_id`
/// counts. Attach with `engine.add_observer`, run the epoch's steps,
/// then `drain()` and map `rule_id → region root` through the
/// consultation order (`GraphRuleSource::last_outcomes` zipped with the
/// driver's own `find_rule_roots` walk — both iterate the same content
/// order, so the k-th `Valid` outcome is `delta.rules[k]`).
///
/// Read-only by construction (the `Observer` contract): counting here
/// cannot perturb the run it observes.
#[derive(Clone, Debug, Default)]
pub struct FireCounter {
    counts: FxHashMap<usize, u64>,
}

impl FireCounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Take the accumulated per-`rule_id` counts, resetting to zero.
    pub fn drain(&mut self) -> FxHashMap<usize, u64> {
        std::mem::take(&mut self.counts)
    }
}

impl Observer for FireCounter {
    fn on_step_end(
        &mut self,
        _snapshot: &hg_core::GraphSnapshot<'_>,
        _stats: &hg_observe::StepStats,
        events: &[RewriteEvent],
    ) {
        for e in events {
            *self.counts.entry(e.rule_id).or_default() += 1;
        }
    }
}

/// Shared-handle wrapper so the driver can keep reading the counter while
/// the engine owns it (`engine.add_observer(Box::new(shared.clone()))`).
/// A bare `Arc<Mutex<FireCounter>>` cannot implement the foreign
/// `Observer` trait (orphan rule), hence the newtype. The mutex is
/// uncontended outside step boundaries (the engine never holds it across
/// calls — `on_step_end` returns before the next step begins), so the
/// `if let Ok` skip-on-poison path is unreachable in practice and
/// deliberately silent rather than panicking inside the engine's loop.
#[derive(Clone, Debug, Default)]
pub struct SharedFireCounter(pub std::sync::Arc<std::sync::Mutex<FireCounter>>);

impl SharedFireCounter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Observer for SharedFireCounter {
    fn on_step_end(
        &mut self,
        snapshot: &hg_core::GraphSnapshot<'_>,
        stats: &hg_observe::StepStats,
        events: &[RewriteEvent],
    ) {
        if let Ok(mut c) = self.0.lock() {
            c.on_step_end(snapshot, stats, events);
        }
    }
}

/// One epoch transition. Returns the new candidate set (parents + child)
/// in canonical order.
pub fn epoch_transition<R: MutationRng>(
    src: &mut GraphRuleSource,
    s: &mut hg_core::HypergraphStore,
    rng: &mut R,
    seen: &mut rustc_hash::FxHashSet<u64>,
    meta_visible: bool,
) -> EpochEvent {
    epoch_transition_with_economy(
        src,
        s,
        rng,
        seen,
        meta_visible,
        &FxHashMap::default(),
        None,
        None,
    )
}

/// One epoch transition with the §10 firing economy ridden along
/// (SB-GE5 resolution — the per-region debit the plan sited HERE).
///
/// `fired` maps region roots to attributed accepted-firing counts for the
/// epoch just ending (driver-built via [`FireCounter`]); empty when the
/// driver has no attribution, in which case ledgers age on income alone.
/// `econ: None` is the pre-economy behavior exactly (no RNG drawn, no
/// store touched beyond the original path — the A0 byte-equivalence the
/// fee-zero test pins).
///
/// Redesign (i) (SB-GE7(a)) moved the policy half of the boundary INTO
/// the window: `FeePolicy` projects within-window spend and withholds
/// past the window-start balance at forward time, so by the time this
/// debit runs every accepted firing is affordable and the debit fully
/// pays — the walk below reconciles actuals against the projection
/// (conflict-resolution losers never pay, though they consumed
/// projection budget).
/// `allowed_ops: None` samples the full 10-op kernel; `Some(ops)` samples
/// uniformly from `ops` (the A2 exclusion-shadow arm). Empty = sampling
/// closed, nothing mutates — total, never panicking.
///
/// Boundary order per §10.6: debit the ending epoch's firings FIRST
/// (against pre-existing ledgers), then mutate/install, then enroll every
/// live root lacking a ledger (newborn + seeds) at age 0, then income +
/// aging for survivors, drop the extinct, audit.
#[allow(clippy::too_many_arguments)]
pub fn epoch_transition_with_economy<R: MutationRng>(
    src: &mut GraphRuleSource,
    s: &mut hg_core::HypergraphStore,
    rng: &mut R,
    seen: &mut rustc_hash::FxHashSet<u64>,
    meta_visible: bool,
    fired: &FxHashMap<NodeId, u64>,
    mut econ: Option<&mut EpochEconomy>,
    allowed_ops: Option<&[Op]>,
) -> EpochEvent {
    let mut ev = EpochEvent::default();

    // FEE_ZERO fast path: economy consulted, inert. The original path
    // below runs untouched; the audit exists so A0's inertness is
    // measured, not assumed.
    if econ.as_ref().is_some_and(|e| e.is_fee_zero()) {
        let inner = epoch_transition(src, s, rng, seen, meta_visible);
        ev.mutated = inner.mutated;
        ev.quarantined = inner.quarantined;
        ev.installed = inner.installed;
        ev.site = inner.site;
        ev.novel_class = inner.novel_class;
        ev.rules_installed = inner.rules_installed;
        ev.economy = Some(EconomyAudit {
            balanced: true,
            ..EconomyAudit::default()
        });
        return ev;
    }

    // 0. Debit the ending epoch's attributed firings (pre-existing
    // ledgers only — the newborn below opens after this).
    let (mut paid, mut refused, mut unattributed) = (0u64, 0, 0);
    if let Some(e) = econ.as_deref_mut() {
        (paid, refused, unattributed) = e.debit(fired);
    }

    // 1. Read parents from the CURRENT store state.
    let snap = s.snapshot();
    let mut parents: Vec<CandidateRule> = Vec::new();
    let mut parent_roots: Vec<NodeId> = Vec::new();
    for root in find_rule_roots(&snap) {
        if let Ok((cand, _)) = read_region_with_footprint(&snap, root) {
            parent_roots.push(root);
            parents.push(cand);
        }
    }
    ev.rules_installed = parents.len();

    // 2+3. Mutate one parent; validate; install if valid.
    let mut child_installed = false;
    if !parents.is_empty() {
        let pick = rng.below(parents.len());
        // A2 exclusion-shadow samples uniformly from the surviving ops;
        // every other arm takes the full kernel through `mutate`.
        let (child, applied) = match allowed_ops {
            Some(ops) if !ops.is_empty() => {
                let chosen = ops[rng.below(ops.len())];
                let di = Op::ALL
                    .iter()
                    .position(|o| *o == chosen)
                    .expect("allowed op is a kernel op");
                mutate_op(&parents[pick], di, rng)
            }
            Some(_) => (parents[pick].clone(), Applied::Nothing),
            None => mutate(&parents[pick], rng),
        };
        if applied == Applied::Edited {
            ev.mutated = true;
            ev.site = Some(parent_roots[pick]);
            let fp = crate::gr1::offline_footprint(&child);
            match validate(&child, &fp, &PriorityObservation { malformed: false }) {
                crate::validate::Verdict::Valid => {
                    let h = rcf_hash(&child);
                    ev.novel_class = seen.insert(h);
                    GraphRuleSource::install_candidates(s, std::slice::from_ref(&child));
                    parents.push(child);
                    child_installed = true;
                }
                _ => ev.quarantined = true,
            }
        }
    }
    ev.installed = child_installed;

    // 4. Economy settle: enroll every live root lacking a ledger (the
    // newborn AND the pre-economy seeds — plan §10 interpretation: every
    // region-bearing rule pays; seeds enter throttled as newborns),
    // income + age the living, drop the extinct, audit. Enrollment walks
    // content order so ledger-opening is deterministic.
    if let Some(e) = econ {
        let live_ordered = find_rule_roots(&s.snapshot());
        let mut opened = 0usize;
        for root in &live_ordered {
            if e.open(*root) {
                opened += 1;
            }
        }
        let live: FxHashSet<NodeId> = live_ordered.into_iter().collect();
        let (_kept, dropped) = e.settle(&live);
        ev.economy = Some(e.audit(opened, dropped, paid, refused, unattributed));
    }

    // Recompile under the arm's protection (the caller steps the engine).
    let _ = compile_delta(&parents, meta_visible);
    // The compiled table is returned via the source on next consultation;
    // here we just refresh src counters by consulting once.
    let snap2 = s.snapshot();
    let _delta = src.rules(&snap2, 0);

    ev
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{CandidatePatternEdge, CandidateRule};

    struct XorShift(u64);
    impl MutationRng for XorShift {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    fn fresh_root(s: &mut hg_core::HypergraphStore) -> NodeId {
        s.add_node(hg_core::NodeData::new(hg_core::Label(7), 0))
    }

    fn pe(slots: Vec<usize>) -> CandidatePatternEdge {
        CandidatePatternEdge { slots }
    }

    /// Two parent regions, same shape as `expr_tests::seed_world`
    /// (duplicated — that helper is private to its module).
    fn seed_store() -> hg_core::HypergraphStore {
        let mut s = hg_core::HypergraphStore::new();
        for i in 0..10u32 {
            let mut d = hg_core::NodeData::new(hg_core::Label(i % 4), 0);
            d.set_property(1, hg_core::PropertyValue::Float(f64::from(i) * 0.1));
            s.add_node(d);
        }
        let ids: Vec<_> = s.nodes().map(|(id, _)| id).collect();
        for w in 0..8usize {
            let a = ids[w % ids.len()];
            let b = ids[(w * 3 + 1) % ids.len()];
            if a != b {
                s.add_edge(hg_core::HyperEdge::new(
                    hg_core::Label((w % 3) as u32),
                    smallvec::smallvec![a, b],
                ));
            }
        }
        let cands = vec![
            CandidateRule {
                priority: 0,
                lhs: vec![pe(vec![0, 1])],
                rhs: vec![pe(vec![0, 2]), pe(vec![2, 1])],
                var_labels: vec![Some(0), Some(1), Some(2)],
                guards: Vec::new(),
                var_guards: Vec::new(),
                effects: Vec::new(),
            },
            CandidateRule {
                priority: 1,
                lhs: vec![pe(vec![0])],
                rhs: vec![pe(vec![0])],
                var_labels: vec![None],
                guards: Vec::new(),
                var_guards: Vec::new(),
                effects: Vec::new(),
            },
        ];
        GraphRuleSource::install_candidates(&mut s, &cands);
        s
    }

    #[test]
    fn open_debit_settle_audit_balances() {
        // Default policy: cost=1, reward=1, endowment=3, k_throttle=2.
        let mut s = hg_core::HypergraphStore::new();
        let a = fresh_root(&mut s);
        let b = fresh_root(&mut s);
        let mut econ = EpochEconomy::new(IncomePolicy::default());
        assert!(econ.open(a));
        assert!(econ.open(b));
        assert!(!econ.open(a), "re-open keeps the aged ledger");
        // a fires 2 (both paid); b fires 5 (3 paid, 2 refused whole).
        let fired = FxHashMap::from_iter([(a, 2u64), (b, 5u64)]);
        let (paid, refused, unattributed) = econ.debit(&fired);
        assert_eq!((paid, refused, unattributed), (5, 2, 0));
        // A count for an unknown root is unattributed, never absorbed.
        let ghost = fresh_root(&mut s);
        let (_, _, u) = econ.debit(&FxHashMap::from_iter([(ghost, 4u64)]));
        assert_eq!(u, 4);
        // Settle with only `a` live: b drops, issuance tightens exactly.
        // (Both ledgers are throttled at age 0, so no income moves.)
        let live = FxHashSet::from_iter([a]);
        let (kept, dropped) = econ.settle(&live);
        assert_eq!((kept, dropped), (1, 1));
        let audit = econ.audit(2, 1, paid, refused, u);
        assert!(audit.balanced, "bounds + issuance must balance: {audit:?}");
        assert_eq!(audit.alive, 1);
        assert_eq!(audit.total_issued, 3, "one endowment of 3 survives");
        assert_eq!(audit.encumbered, 2, "a paid 2 of its 3");
    }

    #[test]
    fn bankrupt_newborn_refuses_whole() {
        let mut s = hg_core::HypergraphStore::new();
        let r = fresh_root(&mut s);
        let mut econ = EpochEconomy::new(IncomePolicy {
            cost: 1,
            reward: 1,
            endowment: 0,
            k_throttle: 0,
        });
        assert!(econ.open(r));
        let (paid, refused, _) = econ.debit(&FxHashMap::from_iter([(r, 3u64)]));
        assert_eq!((paid, refused), (0, 3));
        let audit = econ.audit(1, 0, paid, refused, 0);
        assert!(audit.balanced);
        assert_eq!(audit.encumbered, 0, "refused firings encumber nothing");
    }

    #[test]
    fn fee_zero_transition_matches_plain_transition() {
        // A0 at the epoch level: a FEE_ZERO economy rides along but changes
        // nothing — same RNG stream, same installs, audit all-zeros.
        let mut s0 = seed_store();
        let mut s1 = seed_store();
        let (mut src0, mut src1) = (GraphRuleSource::new(), GraphRuleSource::new());
        let (mut seen0, mut seen1) = (FxHashSet::default(), FxHashSet::default());
        let mut econ = EpochEconomy::new(IncomePolicy::FEE_ZERO);
        for _ in 0..10 {
            let e0 = epoch_transition(&mut src0, &mut s0, &mut XorShift(0xE0), &mut seen0, false);
            let e1 = epoch_transition_with_economy(
                &mut src1,
                &mut s1,
                &mut XorShift(0xE0),
                &mut seen1,
                false,
                &FxHashMap::default(),
                Some(&mut econ),
                None,
            );
            assert_eq!(
                (
                    e0.mutated,
                    e0.quarantined,
                    e0.installed,
                    e0.site,
                    e0.novel_class,
                    e0.rules_installed
                ),
                (
                    e1.mutated,
                    e1.quarantined,
                    e1.installed,
                    e1.site,
                    e1.novel_class,
                    e1.rules_installed
                ),
                "FEE_ZERO must not perturb the transition"
            );
            assert!(e0.economy.is_none());
            assert!(e1.economy.is_some_and(|a| a.balanced && a.alive == 0));
        }
        assert_eq!(seen0.len(), seen1.len());
        assert_eq!(
            find_rule_roots(&s0.snapshot()).len(),
            find_rule_roots(&s1.snapshot()).len()
        );
        assert_eq!(econ.alive(), 0, "FEE_ZERO opens no ledgers");
    }

    #[test]
    fn transition_reports_the_mutation_site() {
        // G-GE2 row 5 input: every applied edit names its parent root.
        // Parents survive installs (installs only add nodes), so a reported
        // site must still be a live root after its epoch.
        let mut s = seed_store();
        let mut src = GraphRuleSource::new();
        let mut seen = FxHashSet::default();
        let mut rng = XorShift(0x51E);
        let mut mutated_epochs = 0usize;
        for _ in 0..10 {
            let ev = epoch_transition(&mut src, &mut s, &mut rng, &mut seen, false);
            if ev.mutated {
                mutated_epochs += 1;
                let site = ev.site.expect("applied edit names its site");
                assert!(
                    find_rule_roots(&s.snapshot()).contains(&site),
                    "reported site must be a live root post-install"
                );
            } else {
                assert!(ev.site.is_none(), "no edit means no site");
            }
        }
        assert!(
            mutated_epochs > 0,
            "seed must mutate at least once in 10 epochs"
        );
    }

    #[test]
    fn economy_transition_opens_newborn_and_stays_balanced() {
        // Live economy, no attribution: every installed child opens a
        // ledger, regions never go extinct here, audit balanced each epoch.
        let mut s = seed_store();
        let mut src = GraphRuleSource::new();
        let mut seen = FxHashSet::default();
        let mut rng = XorShift(0xEC);
        let mut econ = EpochEconomy::new(IncomePolicy::default());
        let mut opened_total = 0usize;
        for _ in 0..10 {
            let ev = epoch_transition_with_economy(
                &mut src,
                &mut s,
                &mut rng,
                &mut seen,
                false,
                &FxHashMap::default(),
                Some(&mut econ),
                None,
            );
            let audit = ev.economy.expect("economy rides along");
            assert!(audit.balanced, "audit must balance every epoch: {audit:?}");
            assert_eq!(audit.dropped, 0, "nothing goes extinct in this harness");
            opened_total += audit.opened;
        }
        assert_eq!(econ.alive(), opened_total);
        assert_eq!(
            econ.total_issued,
            opened_total as i64 * IncomePolicy::default().endowment
        );
    }

    #[test]
    fn fire_counter_counts_accepted_events_per_rule() {
        // Hermetic: fabricated events, no engine. Two for rule 3, one for
        // rule 0; drain takes exactly that and resets.
        use hg_observe::Binding;
        let mut s = hg_core::HypergraphStore::new();
        s.add_node(hg_core::NodeData::new(hg_core::Label(0), 0));
        let snap = s.snapshot();
        let mk = |rule_id: usize| RewriteEvent {
            rule_id,
            rule_name: String::new(),
            binding: Binding::new(rule_id, 0, smallvec::smallvec![]),
            nodes_created: Vec::new(),
            nodes_deleted: Vec::new(),
            edges_created: Vec::new(),
            edges_deleted: Vec::new(),
            cleanup_neighbors: Vec::new(),
            vanished_energy: 0.0,
        };
        let mut c = FireCounter::new();
        c.on_step_end(
            &snap,
            &hg_observe::StepStats::default(),
            &[mk(3), mk(3), mk(0)],
        );
        let counts = c.drain();
        assert_eq!(counts.get(&3), Some(&2));
        assert_eq!(counts.get(&0), Some(&1));
        assert!(c.drain().is_empty(), "drain resets");
    }

    #[test]
    fn fire_counter_wired_to_engine_matches_accepted_total() {
        // The add_observer plumbing: one real step, counter sum equals the
        // engine's accepted_rewrites total. Shared via Arc<Mutex<..>> since
        // the engine owns the boxed observer.
        use hg_engine::Engine;
        use hg_rules::StepMode;
        let toml = r#"
[[rules]]
name = "g"
priority = 0
weight = 1.0
enabled = true

[rules.lhs]
nodes = ["a", "b"]
edges = [["a","b"]]

[rules.lhs.node_labels]
a = 0
b = 1

[rules.rhs]
nodes = ["a", "b", "c"]
edges = [["a","c"], ["c","b"]]

[rules.rhs.node_labels]
c = 1
"#;
        let rule = hg_rules::parser::parse_rules_toml(toml)
            .expect("growth rule parses")
            .into_iter()
            .next()
            .expect("at least one rule");
        let compiled =
            hg_rules::compiled::compile_rules(std::slice::from_ref(&rule)).expect("compiles");
        let mut engine = Engine::new(compiled, StepMode::MaxParallel, 42, 10_000);
        let shared = SharedFireCounter::new();
        engine.add_observer(Box::new(shared.clone()));
        let mut store = seed_store();
        let mut accepted_total = 0usize;
        for _ in 0..5 {
            accepted_total += engine.step(&mut store).accepted_rewrites;
        }
        let counts = shared.0.lock().expect("uncontended").drain();
        let counted: u64 = counts.values().sum();
        assert_eq!(
            counted as usize, accepted_total,
            "observer attribution must equal the engine total"
        );
        assert!(accepted_total > 0, "fixture must actually fire");
    }
}
