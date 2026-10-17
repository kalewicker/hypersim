//! GE plan §10.1 — engine-side adapter: the `FeePolicy` selects the
//! solvent bindings and withholds the bankrupt ones.
//!
//! Layered precisely so the "ledger mechanics" and the "engine plumbing"
//! stay independently auditable. This file owns the plumbing; `economy.rs`
//! owns the math. The unit tests here exercise the policy directly with
//! a hand-built `Vec<Binding>` and a hand-built ledger, never the engine.
//!
//! The policy does NOT call `try_fire` itself; it only *forwards* a
//! binding to the conflict resolver (or withholds it). The engine's
//! observer chain records which bindings were forwarded; the actual
//! ledger debit happens at end-of-epoch, off the critical path. That
//! keeps the policy's per-step cost bounded by `O(candidates)` and the
//! debit work confined to the existing window-boundary audit.
//!
//! Crate firewall note: this file pulls in `hg-observe` for the
//! `RuleSelectionPolicy` trait. `hg-observe` is now a runtime
//! dependency of `hg-meta` (this is the only file that uses it; the
//! rest of the crate is still hg-core + hg-rules only). The firewall
//! the crate header comment describes is preserved for everything but
//! the policy seam — which by definition must be at the engine boundary.

#![allow(clippy::module_name_repetitions)]

use crate::economy::{IncomePolicy, RegionLedger};
use hg_observe::RuleSelectionPolicy;

/// Snapshot of the per-region ledger state at a consultation boundary.
/// `ledgers[i]` corresponds to the rule at `delta.rules[i]` (the i-th
/// entry in the engine's compiled rule table for this consultation).
/// `None` means "outside the economy" (e.g. an external static rule from
/// the harness); those pass through, which is the A0 control's natural
/// extension: a 0-cost policy behaves like SelectAllPolicy even when
/// ledgers are populated.
///
/// `RuleSelectionPolicy: Send + 'static` (the engine's contract), so the
/// ledgers and income live INSIDE the policy struct — no borrowed data,
/// no thread-locals on the hot path.
#[derive(Default)]
pub struct RegionLedgers {
    pub ledgers: Vec<Option<RegionLedger>>,
}

/// The fee-gating policy. Forwards a binding iff its rule is outside the
/// economy (no ledger — pass-through, same as A0) or solvent UNDER
/// WITHIN-WINDOW PROJECTION: the policy counts every binding it forwards
/// this window per rule and withholds once one more firing would exceed
/// the rule's window-start free balance (redesign (i), SB-GE7(a)).
///
/// Projection, not real-time debit: the boundary debit in
/// `epoch_transition_with_economy` still charges actual accepted firings,
/// so conflict-resolution losers never pay — but they DO consume
/// projection budget for the rest of the window (conservative:
/// over-withholds under contention, never over-forwards). By
/// construction accepted ≤ forwarded ≤ affordable, so the boundary debit
/// always fully pays and its refused count sits at ~0; refusals happen at
/// forward time and are counted in `withheld_total`.
///
/// Window discipline: ONE policy object per window. Drivers rebuild the
/// policy from post-boundary ledgers each window (R09, mech); a policy
/// object carried across windows keeps projecting against a stale
/// balance and strangles its rules. `forwarded` is public so tests can
/// assert the projection bound directly.
pub struct FeePolicy {
    pub income: IncomePolicy,
    pub ledgers: RegionLedgers,
    /// Per-rule forwarded-binding counts for the current window.
    pub forwarded: Vec<u64>,
    /// Cumulative bindings withheld by projection. Diagnostic only.
    pub withheld_total: u64,
}

impl FeePolicy {
    pub fn new(income: IncomePolicy) -> Self {
        Self {
            income,
            ledgers: RegionLedgers::default(),
            forwarded: Vec::new(),
            withheld_total: 0,
        }
    }

    /// Forward iff one more firing at `cost` still fits `free`. Unit-cost
    /// ledgers admit exactly `free` forwardings per window; a zero cost
    /// (FEE_ZERO) forwards unboundedly, preserving the A0 identity.
    fn fits(free: i64, cost: i64, already_forwarded: u64) -> bool {
        if cost <= 0 {
            return true;
        }
        free > 0 && already_forwarded < free as u64 / cost as u64
    }

    fn can_fire(&mut self, rule_id: usize) -> bool {
        match self
            .ledgers
            .ledgers
            .get(rule_id)
            .and_then(|opt| opt.as_ref())
        {
            // Outside the economy entirely: pass-through, same as A0.
            None => true,
            // In the economy (including FEE_ZERO ledgers, whose zero cost
            // always fits): project against the window-start balance.
            Some(ledger) => {
                let spent = self.forwarded.get(rule_id).copied().unwrap_or(0);
                Self::fits(ledger.free, self.income.cost, spent)
            }
        }
    }

    fn note_forwarded(&mut self, rule_id: usize) {
        if self.forwarded.len() <= rule_id {
            self.forwarded.resize(rule_id + 1, 0);
        }
        self.forwarded[rule_id] = self.forwarded[rule_id].saturating_add(1);
    }
}

impl RuleSelectionPolicy for FeePolicy {
    fn select(
        &mut self,
        _snapshot: &hg_core::GraphSnapshot<'_>,
        candidates: &[hg_observe::Binding],
    ) -> Vec<usize> {
        let mut keep = Vec::with_capacity(candidates.len());
        for (i, b) in candidates.iter().enumerate() {
            if self.can_fire(b.rule_id) {
                // Count projection budget only for economy rules; outside
                // rules pass through unboundedly and carry no budget.
                if self.ledgers.ledgers.get(b.rule_id).is_some() {
                    self.note_forwarded(b.rule_id);
                }
                keep.push(i);
            } else {
                self.withheld_total += 1;
            }
        }
        keep
    }
}

/// Pre-registered, A0 control identity (plan §10.3): a fee-zero policy
/// with no per-region map MUST be byte-equivalent to `SelectAllPolicy`
/// at the engine level. The `select` body above already achieves that
/// (every rule is either out-of-economy or has a 0-cost ledger, which
/// always fits), but this helper exists so the integration test asserts
/// the identity in one place.
pub fn select_all_when_fee_is_zero() -> FeePolicy {
    FeePolicy::new(IncomePolicy::FEE_ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hg_observe::Binding;
    use smallvec::smallvec;

    /// A hand-built vector of bindings for policy tests, never the engine.
    fn bindings(rule_ids: &[usize]) -> Vec<Binding> {
        rule_ids
            .iter()
            .map(|&rid| Binding::new(rid, 0, smallvec![]))
            .collect()
    }

    /// Thread-local empty store; the policy never reads the snapshot but
    /// the trait signature demands one. A `OnceLock<Box<...>>` gives a
    /// `'static` borrow without leaking: every test reuses the same one.
    fn fake_snapshot() -> hg_core::GraphSnapshot<'static> {
        use std::sync::OnceLock;
        static STORE: OnceLock<Box<hg_core::HypergraphStore>> = OnceLock::new();
        STORE
            .get_or_init(|| Box::new(hg_core::HypergraphStore::new()))
            .snapshot()
    }

    #[test]
    fn empty_region_map_forwards_every_candidate() {
        // Same shape as `SelectAllPolicy`: every binding through.
        let cands = bindings(&[0, 1, 2, 3]);
        let mut pol = select_all_when_fee_is_zero();
        let sel = pol.select(&fake_snapshot(), &cands);
        assert_eq!(sel, vec![0, 1, 2, 3]);
    }

    #[test]
    fn bankrupt_region_withholds_all_its_bindings() {
        // One rule, one region, 0 endowment under cost=1 ⇒ bankrupt.
        let p = IncomePolicy {
            cost: 1,
            reward: 1,
            endowment: 0,
            k_throttle: 0,
        };
        let mut pol = FeePolicy::new(p);
        pol.ledgers.ledgers.push(Some(RegionLedger::new(0)));
        let cands = bindings(&[0]);
        let sel = pol.select(&fake_snapshot(), &cands);
        assert!(sel.is_empty(), "bankrupt rule must withhold every binding");
    }

    #[test]
    fn solvent_region_forwards_solvent_rules_and_withholds_bankrupt() {
        // 3 rules, 3 ledgers: rich, bankrupt, rich. The bankrupt rule
        // vanishes; the rich ones survive.
        let p = IncomePolicy {
            cost: 1,
            reward: 1,
            endowment: 5,
            k_throttle: 0,
        };
        let mut pol = FeePolicy::new(p);
        pol.ledgers.ledgers = vec![
            Some(RegionLedger::new(5)),
            Some(RegionLedger::new(0)),
            Some(RegionLedger::new(5)),
        ];
        let cands = bindings(&[0, 1, 2]);
        let sel = pol.select(&fake_snapshot(), &cands);
        assert_eq!(sel, vec![0, 2], "indices of solvent rules preserved");
    }

    #[test]
    fn mixed_economy_and_external_rules_forwards_both() {
        // rule_id 0 is outside the economy (None), 1 is in and solvent,
        // 2 is in and bankrupt. Both 0 and 1 forward.
        let p = IncomePolicy {
            cost: 1,
            reward: 1,
            endowment: 5,
            k_throttle: 0,
        };
        let mut pol = FeePolicy::new(p);
        pol.ledgers.ledgers = vec![None, Some(RegionLedger::new(5)), Some(RegionLedger::new(0))];
        let cands = bindings(&[0, 1, 2]);
        let sel = pol.select(&fake_snapshot(), &cands);
        assert_eq!(
            sel,
            vec![0, 1],
            "external rules pass through; bankrupt rules don't"
        );
    }

    #[test]
    fn fee_zero_with_mixed_map_still_passes_everyone() {
        // The A0 control: even if the map mentions regions, a 0-cost
        // policy never blocks anything.
        let mut pol = select_all_when_fee_is_zero();
        pol.ledgers.ledgers = vec![
            Some(RegionLedger::new(0)),
            Some(RegionLedger::new(0)),
            Some(RegionLedger::new(0)),
        ];
        let cands = bindings(&[0, 1, 2]);
        let sel = pol.select(&fake_snapshot(), &cands);
        assert_eq!(sel, vec![0, 1, 2]);
    }

    #[test]
    fn projection_caps_forwarding_at_the_window_start_balance() {
        // Redesign (i): endowment 3 under cost 1 admits exactly 3
        // forwardings, then withholds — within ONE select call (one step
        // with 5 bindings) and across calls (the window's later steps).
        let mut pol = FeePolicy::new(IncomePolicy::default());
        pol.ledgers.ledgers.push(Some(RegionLedger::new(3)));
        let cands = bindings(&[0, 0, 0, 0, 0]);
        let sel = pol.select(&fake_snapshot(), &cands);
        assert_eq!(sel, vec![0, 1, 2], "first 3 forward, rest withheld");
        assert_eq!(pol.forwarded, vec![3]);
        assert_eq!(pol.withheld_total, 2);
        let sel2 = pol.select(&fake_snapshot(), &cands);
        assert!(sel2.is_empty(), "budget exhausted for the window");
        assert_eq!(pol.withheld_total, 7);
    }
}
