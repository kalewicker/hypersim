//! Exploding early-abort gate (plan v2 §7): `k` consecutive
//! Exploding-classified windows terminate the cell with verdict
//! `ExplodingAbort` instead of eating the host. The OOM behavior of the
//! shipped rules stays a recorded substrate result; it just stops
//! costing machines.
//!
//! There is no campaign RUNNER yet (v4 §14.3 roadmap) — the gate is
//! wired at the run level (a run is a campaign cell), which is the
//! implementable equivalent; a future campaign driver reuses it as-is.

use hg_schema::Regime;

#[derive(Clone, Copy, Debug)]
pub struct AbortGate {
    k: usize,
    consecutive: usize,
}

impl AbortGate {
    pub fn new(k: usize) -> Self {
        Self {
            k: k.max(1),
            consecutive: 0,
        }
    }

    /// Feed one window's regime; returns true when the run must abort
    /// (exactly at the k-th consecutive Exploding window).
    pub fn observe(&mut self, regime: Regime) -> bool {
        if regime == Regime::Exploding {
            self.consecutive += 1;
        } else {
            self.consecutive = 0;
        }
        self.consecutive >= self.k
    }

    pub fn consecutive(&self) -> usize {
        self.consecutive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aborts_at_exactly_window_k() {
        let mut gate = AbortGate::new(3);
        assert!(!gate.observe(Regime::Exploding));
        assert!(!gate.observe(Regime::Exploding));
        assert!(gate.observe(Regime::Exploding), "third consecutive aborts");
    }

    #[test]
    fn non_exploding_resets_the_streak() {
        let mut gate = AbortGate::new(3);
        assert!(!gate.observe(Regime::Exploding));
        assert!(!gate.observe(Regime::Exploding));
        assert!(!gate.observe(Regime::GrowthTreadmill), "streak reset");
        assert!(!gate.observe(Regime::Exploding));
        assert!(!gate.observe(Regime::Exploding));
        assert!(gate.observe(Regime::Exploding));
    }
}
