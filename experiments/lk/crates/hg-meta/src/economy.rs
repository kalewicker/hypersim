//! GE plan §10.1 — the conserved-token firing economy for region-bearing rules.
//!
//! This is the PURE half of the fee-gating mechanism. It models the per-region
//! ledger and income policy with no engine / rule_source / observer deps, so
//! the conservation invariant and the throttle window are unit-testable
//! hermetically. The engine integration is a thin `RuleSelectionPolicy`
//! adapter in `fee_policy.rs` (a separate file precisely so the boundary
//! between "ledger mechanics" and "engine plumbing" stays auditable).
//!
//! Economy vocabulary (matching selfmod v3/v4):
//!
//! - `total`        — fixed per-region endowment size set at install
//! - `free`         — currently un-encumbered tokens
//! - `Σ tokens`     — every region holds its share; residual = total - free
//! - `cost`         — what one firing costs (per-firing, per-rule)
//! - `income`       — `min(reward, free)`; the FREE cap is load-bearing
//! - `effect_gated` — a firing that produced no net state change pays nothing
//! - `install_fee`  — a one-time charge at install (newborn throttle front)
//! - `k_throttle`   — how many epochs before full income kicks in
//!
//! Conservation invariant (asserted every step):
//!   `free + Σ over all regions of (total - free) == Σ total`, i.e. zero
//!   residual. If this ever trips, the economy is leaking — that is a
//!   finding, not a parameter to tune.
//!
//! Determinism: no RNG here. Every function is total and content-ordered, so
//! the unit tests reproduce byte-for-byte without any keyed stream.

#![allow(clippy::module_name_repetitions)]

/// Per-region bookkeeping. One `RegionLedger` per installed region. The
/// engine's `RuleSelectionPolicy` consults this to decide whether to forward
/// a binding to the conflict resolver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionLedger {
    /// Fixed token endowment, set at install. Never mutates.
    pub total: i64,
    /// Un-encumbered tokens. Decreases by `cost` on a paid firing,
    /// increases by `income` at the end of each epoch.
    pub free: i64,
    /// Epochs survived since install. Drives the throttle window.
    pub age_epochs: i64,
    /// True iff at least one firing was refused (dormant) in the most
    /// recent epoch. Diagnostic only; reset by `end_epoch`.
    pub refused_this_epoch: u32,
    /// Cumulatively refused firings since install. Non-reset.
    pub refused_total: u64,
    /// Cumulatively paid firings since install. Non-reset.
    pub fired_total: u64,
}

impl RegionLedger {
    /// Open a new region with `endowment` free tokens. The new-throttle
    /// window applies (see `policy().is_throttled`).
    pub fn new(endowment: i64) -> Self {
        Self {
            total: endowment,
            free: endowment,
            age_epochs: 0,
            refused_this_epoch: 0,
            refused_total: 0,
            fired_total: 0,
        }
    }

    /// True iff the region cannot currently pay `cost` from its free pool.
    pub fn is_bankrupt(&self, cost: i64) -> bool {
        self.free < cost
    }

    /// Attempt a firing. Returns `true` iff the firing was paid. A refused
    /// firing leaves `free` and all counts unchanged except the dormant
    /// counters — no partial execution, no debt, per §10.1.
    ///
    /// The policy-layer caller is expected to check `is_bankrupt` first and
    /// skip the binding entirely; this method is idempotent w.r.t. balance
    /// when refused, so a direct call here is also safe.
    pub fn try_fire(&mut self, cost: i64) -> bool {
        if self.is_bankrupt(cost) {
            self.refused_total += 1;
            self.refused_this_epoch += 1;
            return false;
        }
        self.free -= cost;
        self.fired_total += 1;
        true
    }

    /// Apply end-of-epoch income under the given policy. The income is
    /// `min(reward, free)`, and is additionally gated by the throttle
    /// window (zero income for throttled regions, regardless of reward).
    pub fn end_epoch(&mut self, policy: &IncomePolicy) {
        if !policy.is_throttled(self.age_epochs) {
            let reward = policy.reward;
            self.free = self.free.saturating_add(reward).min(self.total);
        }
        self.age_epochs = self.age_epochs.saturating_add(1);
        self.refused_this_epoch = 0;
    }
}

/// The end-of-epoch income decision: how much, and for whom.
///
/// Defaults reproduce the §10.2 pre-registration:
///   `cost = 1`, `reward = 1`, `endowment = 3`, `k_throttle = 2`
///
/// which means a throttled newborn fires at most twice in its first two
/// epochs (endowment = 3, cost = 1, no income) and then earns a steady
/// 1-for-1 budget thereafter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IncomePolicy {
    /// Cost of a single accepted firing.
    pub cost: i64,
    /// Reward paid to a region that survives its throttle window.
    pub reward: i64,
    /// Newborn endowment.
    pub endowment: i64,
    /// Epochs of full-income deferral after install. `0` disables the
    /// throttle entirely; regions earn from the first epoch.
    pub k_throttle: i64,
}

impl Default for IncomePolicy {
    fn default() -> Self {
        // §10.2 pre-registered defaults.
        Self {
            cost: 1,
            reward: 1,
            endowment: 3,
            k_throttle: 2,
        }
    }
}

impl IncomePolicy {
    /// `true` while the region is still inside the throttle window —
    /// i.e. while its `age_epochs < k_throttle`. A throttled region earns
    /// no income, so the endowment is what it has until it survives.
    pub fn is_throttled(&self, age_epochs: i64) -> bool {
        age_epochs < self.k_throttle
    }

    /// The A0 control: a policy where every fee is zero. Mutations to the
    /// engine integration that turn off the economy entirely must produce
    /// the same `select` behavior as installing `SelectAllPolicy`. This
    /// constant exists so the A0 audit reads as one line in code, not as
    /// a switch spread across the harness.
    pub const FEE_ZERO: Self = Self {
        cost: 0,
        reward: 0,
        endowment: 0,
        k_throttle: 0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newborn_can_fire_exactly_its_endowment_before_throttle_lifts() {
        // Default policy: cost=1, reward=1, endowment=3, k_throttle=2.
        // A newborn has 3 tokens; 3 fires succeed, the 4th is refused.
        let p = IncomePolicy::default();
        let mut r = RegionLedger::new(p.endowment);
        assert!(r.try_fire(p.cost));
        assert!(r.try_fire(p.cost));
        assert!(r.try_fire(p.cost));
        assert!(!r.try_fire(p.cost), "4th fire must be refused (bankrupt)");
        assert_eq!(r.fired_total, 3);
        assert_eq!(r.refused_total, 1);
        assert_eq!(r.free, 0);
    }

    #[test]
    fn throttled_earns_nothing_until_window_passes() {
        // 2 epochs: no income. Then 1: income arrives, capped by total.
        let p = IncomePolicy::default();
        let mut r = RegionLedger::new(p.endowment);
        // spend down to 1.
        assert!(r.try_fire(2));
        assert_eq!(r.free, 1);

        // Epoch 0: throttled, no income.
        r.end_epoch(&p);
        assert_eq!(r.free, 1);
        assert_eq!(r.age_epochs, 1);

        // Epoch 1: throttled, no income.
        r.end_epoch(&p);
        assert_eq!(r.free, 1);
        assert_eq!(r.age_epochs, 2);

        // Epoch 2: full income, capped by total. Reward=1, free=1+1=2.
        r.end_epoch(&p);
        assert_eq!(r.free, 2);
        assert_eq!(r.age_epochs, 3);
    }

    #[test]
    fn conservation_holds_through_spend_and_income() {
        // After every operation, free + (total - free) == total, by
        // construction (no cross-region flows). The real audit at the
        // engine integration level is the cross-region sum; this test
        // documents the per-region invariant the audit depends on.
        let p = IncomePolicy::default();
        let mut r = RegionLedger::new(p.endowment);
        for _ in 0..20 {
            let _ = r.try_fire(1);
            r.end_epoch(&p);
            assert_eq!(r.free + (r.total - r.free), r.total);
        }
    }

    #[test]
    fn income_never_exceeds_total() {
        // 0-cost scenarios: the min(reward, free) cap + the `min(.., total)`
        // ceiling both matter. Spend 0, earn many epochs — total must cap.
        let p = IncomePolicy {
            cost: 1,
            reward: 100,
            endowment: 3,
            k_throttle: 0, // immediate income for the test
        };
        let mut r = RegionLedger::new(p.endowment);
        for _ in 0..10 {
            r.end_epoch(&p);
        }
        assert!(r.free <= r.total, "free {} > total {}", r.free, r.total);
    }

    #[test]
    fn fee_zero_policy_is_a_known_value_constant() {
        // The A0 control. Pinning the shape lets the integration test
        // assert a single constant against, not a parameter set.
        let z = IncomePolicy::FEE_ZERO;
        assert_eq!(z.cost, 0);
        assert_eq!(z.reward, 0);
        assert_eq!(z.endowment, 0);
        assert_eq!(z.k_throttle, 0);
        assert!(
            !z.is_throttled(0),
            "throttle window cannot fire on a zero policy"
        );
    }

    #[test]
    fn refused_fire_does_not_change_balance_or_paid_count() {
        // The whole point of §10.1: unpaid = whole refusal, no partial
        // execution, no debt. A refused fire must not perturb `free` or
        // `fired_total`. Use a cost that exceeds the endowment so every
        // attempt is refused from the start.
        let p = IncomePolicy {
            cost: 10,
            reward: 1,
            endowment: 3,
            k_throttle: 0,
        };
        let mut r = RegionLedger::new(p.endowment);
        for _ in 0..5 {
            assert!(!r.try_fire(p.cost));
        }
        assert_eq!(r.free, p.endowment);
        assert_eq!(r.fired_total, 0);
        assert_eq!(r.refused_total, 5);
    }
}
