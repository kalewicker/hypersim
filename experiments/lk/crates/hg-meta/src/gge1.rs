//! G-GE1 — mutation-kernel growth-bias audit (GE sub-track plan §4).
//!
//! G-R1's harness plus four additive columns per valid draw:
//!
//! - `static_dn` / `static_de` — compiled net node/edge deltas of the child
//! - `dyn_n_traj`              — node-count trajectory firing the child on
//!   the shared fixture world for W = 100 steps (the G-R1 Catastrophic
//!   window; SB-GE1 flags the horizon risk)
//! - `dyn_class`               — BALANCED / SATURATING / EXPANSIONARY /
//!   REGIME_EXPLODING (pre-committed §4.2 definitions)
//!
//! Pre-committed verdicts (§4.3, committed BEFORE any number existed):
//!
//! - `f_hot <= 0.15` and max-op `expansion_rate <= 0.10` → **PASS**
//! - `f_hot <= 0.35` but some operator hot               → **DESCOPE**
//! - `f_hot > 0.35`                                       → **REDESIGN**
//!
//! where "hot" means `expansion_rate(op) > 0.10`, and the rate counts
//! EXPANSIONARY ∪ REGIME_EXPLODING over the VALID draws of that operator.
//! Independent of the verdict: any operator above 0.10 is
//! REQUIRES-COUPLING (may not ship without a GE-C1(a)/(b) pairing);
//! operators at or below 0.05 are eligible to ship under GE-C1(c) alone.
//!
//! This module holds the PURE logic (classes, counters, verdicts, static
//! deltas). The engine-touching probe driver lives in `gge1_tests.rs` —
//! hg-engine is a dev-dependency only, preserving the crate firewall the
//! same way the G-R1/G-R2/G-R3 harnesses do.

use crate::mutate::Op;

/// Dynamic outcome of one valid mutant fired on the fixture world (§4.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynClass {
    /// Returns to ±5% of baseline node count within the window.
    Balanced,
    /// Leaves the ±5% band but plateaus: last 20 steps within ±1% of the
    /// step-(W−20) value. DIRECTION-NEUTRAL by design — a deflationary
    /// mutant that eats its own substrate and stalls at a lower level
    /// saturates exactly like a growing one stalls at a higher level, and
    /// neither can explode. `late_plateau` marks boundary cases flagged
    /// for a one-off 16×-horizon confirmation (SB-GE1 mitigation).
    Saturating { late_plateau: bool },
    /// Neither balanced nor saturating — still growing at window end.
    Expanding,
    /// Crossed the evaluator's Exploding threshold during the window
    /// (`growth_ratio >= GROWTH_EXPLODE_MIN` over a 10-step sub-window —
    /// the evaluator's own criterion adapted to a raw trajectory; the
    /// NODE_CAP arm cannot trigger at this window length and graph size).
    RegimeExploding,
}

impl DynClass {
    pub fn name(self) -> &'static str {
        match self {
            DynClass::Balanced => "BALANCED",
            DynClass::Saturating { .. } => "SATURATING",
            DynClass::Expanding => "EXPANSIONARY",
            DynClass::RegimeExploding => "REGIME_EXPLODING",
        }
    }
}

/// The evaluator's own explode bar (hg-eval thresholds.rs), applied to
/// 10-step sub-window growth ratios of the probe trajectory.
pub const GROWTH_RATIO_EXPLODE: f64 = 1.5;

/// Bounded-compute bail for the probe driver: a run that reaches this many
/// nodes from the 8-node fixture world within W steps has crossed a
/// 10-step ratio of GROWTH_RATIO_EXPLODE many times over, so its class is
/// already REGIME_EXPLODING — simulating the remaining steps cannot change
/// the verdict, only burn CPU on the very runs the gate exists to flag.
/// Truncation keeps the recorded trajectory internally consistent (the
/// crossing window is fully recorded).
pub const EXPLODE_BAIL_NODES: usize = 5_000;

/// Classify one node-count trajectory (index 0 = pre-step baseline,
/// one entry per probe step thereafter). Deterministic, total.
pub fn classify_trajectory(traj: &[usize]) -> DynClass {
    assert!(traj.len() >= 2, "trajectory needs baseline + >=1 step");
    let n = traj.len();
    let baseline = traj[0];
    let final_n = traj[n - 1];

    // REGIME_EXPLODING first: ANY 10-step sub-window ratio >= the bar.
    // (First-match-wins, like the evaluator's classifier.)
    const SUB: usize = 10;
    if n > SUB {
        for t in (SUB..n).step_by(SUB) {
            let prev = traj[t - SUB];
            if prev > 0 {
                let ratio = traj[t] as f64 / prev as f64;
                if ratio >= GROWTH_RATIO_EXPLODE {
                    return DynClass::RegimeExploding;
                }
            }
        }
    }

    // BALANCED: back to within ±5% of baseline (round half up on the band,
    // integer arithmetic so the class is exactly reproducible).
    let band = ((baseline as f64) * 0.05).round() as usize;
    if final_n.abs_diff(baseline) <= band {
        return DynClass::Balanced;
    }

    // SATURATING: last 20 steps within ±1% of the step-(W−20) value.
    if n >= 21 {
        let anchor = traj[n - 21];
        let tol = (((anchor as f64) * 0.01).ceil() as usize).max(1);
        let flat_20 = traj[n - 20..].iter().all(|&v| v.abs_diff(anchor) <= tol);
        if flat_20 {
            // SB-GE1 boundary check: is the LAST FORTY flat too? If not,
            // the plateau arrived late — flag for 16x-horizon confirmation.
            let late = if n >= 41 {
                let anchor2 = traj[n - 41];
                let tol2 = (((anchor2 as f64) * 0.01).ceil() as usize).max(1);
                !traj[n - 40..].iter().all(|&v| v.abs_diff(anchor2) <= tol2)
            } else {
                false
            };
            return DynClass::Saturating { late_plateau: late };
        }
    }

    DynClass::Expanding
}

/// Static net deltas of a valid child, straight off the candidate.
///
/// - `dn`: the engine creates one world node per var referenced on the RHS
///   but not on the LHS (unbound RHS vars); it never deletes nodes. So
///   `dn = |rhs-only vars|`.
/// - `de`: one pattern edge per side; parallel identical edges are counted
///   as written because the store does not dedupe them (incident B3).
pub fn static_deltas(rule: &crate::schema::CandidateRule) -> (i64, i64) {
    let lhs_vars: Vec<usize> = rule
        .lhs
        .iter()
        .flat_map(|e| e.slots.iter().copied())
        .collect();
    let rhs_vars: Vec<usize> = rule
        .rhs
        .iter()
        .flat_map(|e| e.slots.iter().copied())
        .collect();
    let de = rule.rhs.len() as i64 - rule.lhs.len() as i64;
    // Created world nodes == distinct RHS-only (unbound) vars; deleted
    // world nodes == distinct LHS-only vars (the engine deletes them —
    // see the glider's R2 in gr3.rs). Net of both directions.
    let count_distinct_outside = |vars: &[usize], exclude: &[usize]| -> i64 {
        let mut seen: Vec<usize> = Vec::new();
        for v in vars {
            if !exclude.contains(v) && !seen.contains(v) {
                seen.push(*v);
            }
        }
        seen.len() as i64
    };
    (
        count_distinct_outside(&rhs_vars, &lhs_vars) - count_distinct_outside(&lhs_vars, &rhs_vars),
        de,
    )
}

/// Per-operator counters. Every sampled draw lands in exactly one bucket;
/// `total()` is honest and rates need no denominators beyond it (G-R1
/// exclusivity discipline carried over).
#[derive(Clone, Copy, Debug, Default)]
pub struct OpStats {
    pub nothing: usize,
    pub quarantined: usize,
    /// Valid under V1..V9 but engine-compile-rejected (SB-GE3 diagnostic).
    pub uncompilable: usize,
    pub balanced: usize,
    pub saturating: usize,
    pub expanding: usize,
    pub exploding: usize,
    /// Diagnostic subset of `saturating` (SB-GE1 flag), never summed into
    /// the other buckets.
    pub late_plateau: usize,
}

impl OpStats {
    pub fn total(&self) -> usize {
        self.nothing
            + self.quarantined
            + self.uncompilable
            + self.balanced
            + self.saturating
            + self.expanding
            + self.exploding
    }
    pub fn valid(&self) -> usize {
        self.balanced + self.saturating + self.expanding + self.exploding
    }
    /// (EXPANSIONARY ∪ REGIME_EXPLODING) / valid draws of this operator.
    pub fn expansion_rate(&self) -> f64 {
        let v = self.valid();
        if v == 0 {
            0.0
        } else {
            (self.expanding + self.exploding) as f64 / v as f64
        }
    }
    /// GE-C1 shipping class from the pre-committed bands.
    pub fn coupling_class(&self) -> Coupling {
        let r = self.expansion_rate();
        if r > 0.10 {
            Coupling::RequiresCoupling
        } else if r <= 0.05 {
            Coupling::CeilingEligible
        } else {
            Coupling::MidBand
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coupling {
    /// expansion_rate > 0.10 — may not ship without GE-C1(a)/(b).
    RequiresCoupling,
    /// 0.05 < rate <= 0.10 — hot only if it is the max and f_hot trips.
    MidBand,
    /// rate <= 0.05 — may ship under GE-C1(c) alone.
    CeilingEligible,
}

impl Coupling {
    pub fn name(self) -> &'static str {
        match self {
            Coupling::RequiresCoupling => "REQUIRES-COUPLING",
            Coupling::MidBand => "MID",
            Coupling::CeilingEligible => "CEILING-ELIGIBLE",
        }
    }
}

/// Aggregate over all operators.
#[derive(Clone, Copy, Debug)]
pub struct Envelope {
    pub ops: [(Op, OpStats); 10],
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            ops: std::array::from_fn(|i| (Op::ALL[i], OpStats::default())),
        }
    }

    pub fn record(&mut self, op: Op, outcome: Outcome) {
        let s = &mut self.ops[self
            .ops
            .iter()
            .position(|(o, _)| *o == op)
            .expect("known op")]
        .1;
        match outcome {
            Outcome::Nothing => s.nothing += 1,
            Outcome::Quarantined => s.quarantined += 1,
            Outcome::Uncompilable => s.uncompilable += 1,
            Outcome::Dynamic(DynClass::Balanced) => s.balanced += 1,
            Outcome::Dynamic(DynClass::Saturating { late_plateau }) => {
                s.saturating += 1;
                if late_plateau {
                    s.late_plateau += 1;
                }
            }
            Outcome::Dynamic(DynClass::Expanding) => s.expanding += 1,
            Outcome::Dynamic(DynClass::RegimeExploding) => s.exploding += 1,
        }
    }

    /// f_hot = (EXPANSIONARY ∪ REGIME_EXPLODING) / valid, whole ensemble.
    pub fn f_hot(&self) -> f64 {
        let valid: usize = self.ops.iter().map(|(_, s)| s.valid()).sum();
        let hot: usize = self
            .ops
            .iter()
            .map(|(_, s)| s.expanding + s.exploding)
            .sum();
        if valid == 0 {
            0.0
        } else {
            hot as f64 / valid as f64
        }
    }

    pub fn max_expansion_rate(&self) -> (Op, f64) {
        self.ops
            .iter()
            .copied()
            .max_by(|a, b| {
                a.1.expansion_rate()
                    .total_cmp(&b.1.expansion_rate())
                    .then(a.0.cmp(&b.0))
            })
            .map(|(op, s)| (op, s.expansion_rate()))
            .expect("ten operators always present")
    }

    /// The pre-committed §4.3 ladder.
    pub fn verdict(&self) -> Verdict {
        let f = self.f_hot();
        let (max_op, max_r) = self.max_expansion_rate();
        if f > 0.35 {
            Verdict::Redesign { f_hot: f }
        } else if f > 0.15 || max_r > 0.10 {
            let hot: Vec<&'static str> = self
                .ops
                .iter()
                .filter(|(_, s)| s.expansion_rate() > 0.10)
                .map(|(op, _)| op.name())
                .collect();
            let descoped = if hot.is_empty() {
                vec![max_op.name()]
            } else {
                hot
            };
            Verdict::Descope { f_hot: f, descoped }
        } else {
            Verdict::Pass { f_hot: f }
        }
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::new()
    }
}

/// One sampled draw's destination.
#[derive(Clone, Copy, Debug)]
pub enum Outcome {
    Nothing,
    Quarantined,
    /// Valid under V1..V9 but rejected by the engine's compile step
    /// (e.g. a vars_distinct guard naming a var no longer wired into the
    /// LHS — SB-GE3: the validator battery does not model var-guard
    /// consistency; engine compile is the only thing that catches it).
    /// Counted, never folded into Quarantined: it measures a different
    /// instrument boundary.
    Uncompilable,
    Dynamic(DynClass),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    Pass {
        f_hot: f64,
    },
    Descope {
        f_hot: f64,
        descoped: Vec<&'static str>,
    },
    Redesign {
        f_hot: f64,
    },
}

impl Verdict {
    pub fn name(&self) -> &'static str {
        match self {
            Verdict::Pass { .. } => "PASS",
            Verdict::Descope { .. } => "DESCOPE",
            Verdict::Redesign { .. } => "REDESIGN",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{CandidatePatternEdge, CandidateRule};

    fn pe(slots: &[usize]) -> CandidatePatternEdge {
        CandidatePatternEdge {
            slots: slots.to_vec(),
        }
    }

    fn rule(lhs: Vec<CandidatePatternEdge>, rhs: Vec<CandidatePatternEdge>) -> CandidateRule {
        let nv = lhs
            .iter()
            .chain(rhs.iter())
            .flat_map(|e| e.slots.iter().copied())
            .max()
            .map_or(0, |m| m + 1);
        CandidateRule {
            priority: 0,
            lhs,
            rhs,
            var_labels: vec![None; nv],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        }
    }

    #[test]
    fn flat_trajectory_is_balanced() {
        let t = vec![36usize; 101];
        assert_eq!(classify_trajectory(&t), DynClass::Balanced);
    }

    #[test]
    fn return_to_baseline_is_balanced_even_after_excursion() {
        let mut t: Vec<usize> = (0..50).map(|i| 36 + i).collect();
        t.extend(std::iter::repeat_n(37usize, 51));
        assert_eq!(classify_trajectory(&t), DynClass::Balanced);
    }

    #[test]
    fn steady_ramp_is_expansionary() {
        // +1/step from 36: every 10-step ratio <= 46/36 ≈ 1.28, below the
        // explode bar, but never returning to baseline — EXPANSIONARY.
        // (A +2/step ramp DOES cross 1.5x on the early small-base windows,
        // which is the evaluator-faithful outcome; covered by the
        // fast_doubling test.)
        let t: Vec<usize> = (0..101).map(|i| 36 + i).collect();
        assert_eq!(classify_trajectory(&t), DynClass::Expanding);
    }

    #[test]
    fn fast_doubling_window_is_regime_exploding() {
        // 10-step sub-window ratio hits 1.5 quickly.
        let mut t: Vec<usize> = Vec::new();
        let mut n = 36.0f64;
        for _ in 0..101 {
            t.push(n as usize);
            n *= 1.06; // 1.06^10 ≈ 1.79 >= 1.5
        }
        assert_eq!(classify_trajectory(&t), DynClass::RegimeExploding);
    }

    #[test]
    fn growth_then_long_flat_is_saturating_not_late() {
        // Slow enough (+2/step from 100) that no 10-step window ratio
        // reaches 1.5 — the explode class must NOT swallow saturation.
        let mut t: Vec<usize> = (0..61).map(|i| 100 + i * 2).collect();
        t.extend(std::iter::repeat_n(t[60], 41));
        match classify_trajectory(&t[..101]) {
            DynClass::Saturating { late_plateau } => assert!(!late_plateau),
            other => panic!("expected SATURATING, got {other:?}"),
        }
    }

    #[test]
    fn plateau_arriving_in_the_last_quarter_flags_late_sb_ge1() {
        // Grow (+3/step from 100, still under the ratio bar) until index
        // 80, flat only for the last ~20 steps: the SB-GE1 forty-step
        // back-anchor lands on still-rising data.
        let mut t: Vec<usize> = (0..81).map(|i| 100 + i * 3).collect();
        t.extend(std::iter::repeat_n(t[80], 21));
        assert_eq!(t.len(), 102);
        match classify_trajectory(&t[..101]) {
            DynClass::Saturating { late_plateau } => assert!(late_plateau),
            other => panic!("expected SATURATING-late, got {other:?}"),
        }
    }

    #[test]
    fn static_deltas_known_values() {
        // Pure relabel: same single edge both sides -> (0, 0).
        let (dn, de) = static_deltas(&rule(vec![pe(&[0, 1])], vec![pe(&[0, 1])]));
        assert_eq!((dn, de), (0, 0));
        // Subdivide shape: LHS 2-edge, RHS two 2-edges sharing a NEW var.
        let (dn, de) = static_deltas(&rule(vec![pe(&[0, 1])], vec![pe(&[0, 2]), pe(&[2, 1])]));
        assert_eq!((dn, de), (1, 1));
        // Delete: LHS two edges, RHS one -> (0, -1). The LHS-only var v2 is
        // DELETED by the engine (LHS-only vars do not survive), so dn
        // counts created-minus-deleted: 0 created, 1 deleted.
        let (dn, de) = static_deltas(&rule(vec![pe(&[0, 1]), pe(&[1, 2])], vec![pe(&[0, 1])]));
        assert_eq!((dn, de), (-1, -1));
        // Fan-out: one LHS edge, three RHS edges sharing ONE new var ->
        // (+1 node, +2 edges) — the incident-B3 shape.
        let (dn, de) = static_deltas(&rule(
            vec![pe(&[0, 1])],
            vec![pe(&[2, 0]), pe(&[2, 1]), pe(&[2, 0])],
        ));
        assert_eq!((dn, de), (1, 2));
    }

    #[test]
    fn verdict_ladder_matches_precommitted_bands() {
        let mut env = Envelope::new();
        for (_, s) in env.ops.iter_mut() {
            s.balanced = 90;
            s.saturating = 6;
            s.expanding = 4; // rate 0.04 per op, f_hot 0.04
        }
        assert_eq!(env.verdict(), Verdict::Pass { f_hot: env.f_hot() });

        // One hot operator (rate 0.20), ensemble still low -> DESCOPE names it.
        let mut env = Envelope::new();
        for (_, s) in env.ops.iter_mut() {
            s.balanced = 95;
            s.expanding = 5; // 0.05
        }
        {
            let (_, s) = &mut env.ops[5];
            s.balanced = 60;
            s.expanding = 15; // 0.20 hot
        }
        match env.verdict() {
            Verdict::Descope { descoped, .. } => assert_eq!(descoped, vec!["add_pattern_edge"]),
            other => panic!("expected DESCOPE, got {other:?}"),
        }

        // Ensemble hot -> REDESIGN.
        let mut env = Envelope::new();
        for (_, s) in env.ops.iter_mut() {
            s.balanced = 50;
            s.expanding = 50; // 0.50 everywhere
        }
        assert!(matches!(env.verdict(), Verdict::Redesign { .. }));
    }

    #[test]
    fn coupling_classes_band_exactly() {
        let mk = |expanding: usize, balanced: usize| OpStats {
            expanding,
            balanced,
            ..OpStats::default()
        };
        assert_eq!(mk(4, 96).coupling_class(), Coupling::CeilingEligible);
        assert_eq!(mk(6, 94).coupling_class(), Coupling::MidBand);
        assert_eq!(mk(11, 89).coupling_class(), Coupling::RequiresCoupling);
        // Zero-valid operator: rate 0.0, ceiling-eligible (cannot claim heat).
        assert_eq!(
            OpStats::default().coupling_class(),
            Coupling::CeilingEligible
        );
    }
}
