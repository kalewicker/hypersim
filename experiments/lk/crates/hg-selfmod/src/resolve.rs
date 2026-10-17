//! Local conflict resolution with selectable equal-priority tie-break (v7 §1).
//!
//! WHY THIS EXISTS. `hg_engine::resolve_conflicts` accepts greedily over
//! `Binding::canonical_key` = `(Reverse(priority), rule_id, sorted bound
//! nodes)`. Within a priority class **every** binding of a lower `rule_id`
//! rule precedes every binding of a higher one, so the earlier rule takes
//! first claim on the whole world and later rules see only leftovers. v6's
//! routing note (selfmod_v6_findings.md, "Routing for v7") identified this as
//! contaminating every multi-rule result since v3: a neutral clone can never
//! win a contested node, so the invasion gate cannot measure fitness.
//!
//! THE DECISION, RECORDED. Ground rule 1 forbids changing hg-engine in place;
//! v7 resolves the question the routing note left open by implementing
//! conflict resolution LOCALLY in hg-selfmod and leaving hg-engine untouched:
//! the LK track (v7/v8 contention audits, back-action tables) is calibrated
//! against canonical-order trajectories, and hg-sim-core carries uncommitted
//! WIP that depends on them. The local pass restates the engine's greedy
//! algorithm with an injectable order; `TieBreak::RuleId` reproduces the
//! engine's accepted set exactly (`test_ruleid_mode_matches_engine`).
//!
//! MERIT SEMANTICS. Under `TieBreak::Merit`, equal-priority bindings are
//! ordered by their rule's CUMULATIVE productive firings (descending), then
//! by bound-node key. This is round-robin-like work sharing: a rule that has
//! been winning cedes queue position to coexisting equals until they catch
//! up. Two hazards, stated before any run:
//!
//! 1. **Newborn disadvantage.** A newborn clone starts at 0 while its
//!    resident template carries history, so early lockout persists in a
//!    weaker form. The neutral-clone null (P0c, kind="clone") measures
//!    exactly this; if the null still fails, cumulative merit is refuted as
//!    the fix and a windowed/decaying estimator is indicated.
//! 2. **Priority still dominates.** Merit only orders WITHIN a priority
//!    class, exactly as rule_id did. Priority semantics are unchanged.

use hg_observe::Binding;
use serde::{Deserialize, Serialize};

/// How bindings of EQUAL priority are ordered entering the greedy pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TieBreak {
    /// `(priority desc, rule_id asc, node key)` — the engine's historical
    /// order. Reproduces v3–v6 bit-for-bit.
    RuleId,
    /// `(priority desc, cumulative merit desc, node key)` — v7's fair order.
    /// `rule_id` is gone from the sort entirely; remaining ties fall through
    /// to the stable sort's input order (rule-index order — deterministic).
    Merit,
    /// `(priority desc, per-step keyed draw, node key)` — the routing note's
    /// other named option ("merit-bearing **or random**"). Unbiased in
    /// expectation with no bootstrapping problem: a newborn competes
    /// immediately. Deterministic via a stream keyed by `(run seed, world
    /// step)` — the `subsample()` pattern — so the main RNG stream is
    /// untouched and same-seed runs reproduce bit-for-bit.
    Random,
    /// `(priority desc, merit + jitter desc, node key)` — v8's hybrid. The
    /// merit signal is perturbed by a per-binding multiplicative draw:
    /// `key = (m + 1) * (1 + eps·(u - 1/2))` with u uniform in [0,1). At
    /// `jitter_eps = 0` this is exactly `Merit`; as eps grows it approaches
    /// `Random` in effect. Newborns (m = 0) tie at base 1 and are separated
    /// by noise alone, so a seeded variant competes from birth — while an
    /// incumbent's accumulated history still dominates once the gap exceeds
    /// the noise band. Same keyed-stream discipline as `Random`.
    Jitter,
}

impl std::str::FromStr for TieBreak {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "ruleid" | "rule_id" | "canonical" => Ok(TieBreak::RuleId),
            "merit" => Ok(TieBreak::Merit),
            "random" | "rand" => Ok(TieBreak::Random),
            "jitter" => Ok(TieBreak::Jitter),
            _ => Err(()),
        }
    }
}

/// Total order over bindings under the selected tie-break.
///
/// Equivalent to `canonical_key`'s ordering in `RuleId` mode (priority desc,
/// rule_id asc, sorted-node key asc); in `Merit` mode the rule_id slot is
/// replaced by cumulative productive firings, descending; in `Random`/`Jitter`
/// modes by a per-binding keyed draw descending. `merit_by_rid` is indexed by
/// decode index; out-of-range ids read as 0 (defensive only — the caller sizes
/// it to the decoded ruleset). `draws[i]` is the random key for `bindings[i]`,
/// drawn once in input order (empty unless `tie` draws). `jitter_eps` scales
/// the noise band in `Jitter` mode (ignored elsewhere).
fn cmp_bindings(
    a: &Binding,
    b: &Binding,
    tie: TieBreak,
    merit_by_rid: &[f64],
    draws: &[f64],
    ia: usize,
    ib: usize,
) -> std::cmp::Ordering {
    // Priority descending (engine: Reverse(priority)).
    match b.priority.cmp(&a.priority) {
        std::cmp::Ordering::Equal => {}
        o => return o,
    }
    // Second slot: rule_id ascending / merit / random / jitter, descending.
    // Merit keys are f64 (decayed merit produces fractional values); the
    // u64→f64 cast is exact for the counts the cumulative table holds
    // (< 2^53 by any conceivable run length).
    let second = match tie {
        TieBreak::RuleId => a.rule_id.cmp(&b.rule_id),
        TieBreak::Merit => {
            let ma = merit_by_rid.get(a.rule_id).copied().unwrap_or(0.0);
            let mb = merit_by_rid.get(b.rule_id).copied().unwrap_or(0.0);
            mb.total_cmp(&ma)
        }
        // u64 draws mapped to f64 for a uniform comparison path; exact
        // ordering is preserved (u64 → f64 is monotone below 2^53, and these
        // are raw RNG words compared only against each other).
        TieBreak::Random => draws[ib].total_cmp(&draws[ia]),
        TieBreak::Jitter => draws[ib].total_cmp(&draws[ia]),
    };
    second.then_with(|| a.node_ids_sorted().cmp(&b.node_ids_sorted()))
}

/// Greedy node-disjointness acceptance over the selected order — the local
/// restatement of `hg_engine::resolve_conflicts`. Accepted bindings claim
/// their bound nodes; later bindings touching a claimed node are rejected.
/// Creation bindings (empty bound set) can never conflict and are always
/// accepted. Index-stable sort keeps the pass deterministic for any input
/// order.
///
/// RNG discipline (the `subsample()` rule): in `Random`/`Jitter` modes the
/// per-binding draws come from a stream keyed by `key` — the caller passes
/// `seed ^ step.wrapping_mul(GOLDEN)` — created fresh here, so switching
/// tie-break mode never shifts a single draw of the run's main stream.
/// `jitter_eps` is the noise scale in `Jitter` mode (0 = exactly `Merit`).
pub fn resolve_conflicts_tiebreak(
    bindings: &[Binding],
    tie: TieBreak,
    merit_by_rid: &[f64],
    key: u64,
    jitter_eps: f64,
) -> Vec<Binding> {
    let mut draws: Vec<f64> = Vec::with_capacity(bindings.len());
    match tie {
        TieBreak::Random => {
            use rand::{Rng, SeedableRng};
            let mut r = rand_chacha::ChaCha8Rng::seed_from_u64(key ^ 0x7138_4C2D_9B05_A1F7);
            draws.extend((0..bindings.len()).map(|_| r.gen::<u64>() as f64));
        }
        TieBreak::Jitter => {
            use rand::{Rng, SeedableRng};
            let mut r = rand_chacha::ChaCha8Rng::seed_from_u64(key ^ 0x7138_4C2D_9B05_A1F7);
            for i in 0..bindings.len() {
                let m = merit_by_rid
                    .get(bindings[i].rule_id)
                    .copied()
                    .unwrap_or(0.0);
                let u: f64 = r.gen::<u64>() as f64 / u64::MAX as f64;
                // key = ln(m + 1) + eps·(u − ½): an absolute noise band of
                // half-width eps/2 in log-merit space — equivalently,
                // multiplicative noise e^±eps/2 around the merit signal.
                // (A linear multiplier 1 + eps·(u−½) was tried first and
                // rejected: at eps ≥ 2 it goes NEGATIVE, letting a
                // high-merit incumbent lose outright. Log space cannot.)
                // Newborns (m = 0) sit at 0 and separate by noise alone.
                draws.push((m + 1.0).ln() + jitter_eps * (u - 0.5));
            }
        }
        _ => {}
    };
    let mut order: Vec<usize> = (0..bindings.len()).collect();
    order.sort_by(|&i, &j| {
        cmp_bindings(&bindings[i], &bindings[j], tie, merit_by_rid, &draws, i, j)
    });

    let mut accepted: Vec<Binding> = Vec::new();
    let mut claimed: rustc_hash::FxHashSet<hg_core::NodeId> = rustc_hash::FxHashSet::default();
    for i in order {
        let b = &bindings[i];
        if b.bound_nodes.iter().any(|n| claimed.contains(n)) {
            continue;
        }
        claimed.extend(b.bound_nodes.iter().copied());
        accepted.push(b.clone());
    }
    accepted
}

#[cfg(test)]
mod tests {
    use super::*;
    use hg_core::NodeId;

    fn node(k: u64) -> NodeId {
        // Tests only compare keys; any id with the right ffi payload works.
        use slotmap::KeyData;
        NodeId::from(KeyData::from_ffi(k))
    }

    fn b(rule_id: usize, priority: i32, nodes: &[u64]) -> Binding {
        Binding::new(rule_id, priority, nodes.iter().copied().map(node).collect())
    }

    /// RuleId mode must reproduce the engine's accepted set exactly — this is
    /// the license for running v7 arms locally without touching hg-engine.
    #[test]
    fn test_ruleid_mode_matches_engine() {
        let pool = vec![
            b(0, 0, &[10, 11]),
            b(1, 0, &[11, 12]),
            b(2, 5, &[12, 13]), // high priority, considered first
            b(0, 0, &[20, 21]),
            b(1, 0, &[21, 22]),
            b(2, -3, &[99]), // low priority, last
            b(3, 0, &[]),    // creation binding: always accepted
        ];
        let mut engine = hg_engine::resolve_conflicts(&pool);
        let mut mine = resolve_conflicts_tiebreak(&pool, TieBreak::RuleId, &[], 0, 0.0);
        // Same set, same order (both greedy over the same total order).
        let key = |x: &Binding| {
            (
                std::cmp::Reverse(x.priority),
                x.rule_id,
                x.node_ids_sorted(),
            )
        };
        engine.sort_by_key(key);
        mine.sort_by_key(key);
        assert_eq!(engine.len(), mine.len());
        for (e, m) in engine.iter().zip(mine.iter()) {
            assert_eq!(e.rule_id, m.rule_id);
            assert_eq!(e.node_ids_sorted(), m.node_ids_sorted());
        }
    }

    /// The bias itself: under RuleId, rule 0 takes the contested node even
    /// when rule 1 has done all the work so far.
    #[test]
    fn test_ruleid_lockout_reproduced() {
        let pool = vec![b(0, 0, &[7]), b(1, 0, &[7])];
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::RuleId, &[], 0, 0.0);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].rule_id, 0);
    }

    /// The fix: with merit 4 vs 1, rule 1 wins the contested node.
    #[test]
    fn test_merit_lets_equal_rules_share_work() {
        let pool = vec![b(0, 0, &[7]), b(1, 0, &[7])];
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Merit, &[1.0, 4.0], 0, 0.0);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].rule_id, 1);
    }

    /// Zero merit everywhere falls back to stable input order (no rule_id
    /// resurrection through the tie).
    #[test]
    fn test_zero_merit_keeps_input_order() {
        let pool = vec![b(5, 0, &[7]), b(2, 0, &[7])];
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Merit, &[0.0; 6], 0, 0.0);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].rule_id, 5);
    }

    /// Merit never crosses the priority boundary.
    #[test]
    fn test_priority_dominates_merit() {
        let pool = vec![b(0, 0, &[7]), b(1, 1, &[7])];
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Merit, &[9999.0, 0.0], 0, 0.0);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].rule_id, 1); // priority 1 beats merit 9999 at prio 0
    }

    /// Equal merit on the same node falls to stable input order — rule_id
    /// must not resurrect through the tie.
    #[test]
    fn test_equal_merit_ties_fall_to_input_order() {
        let pool = vec![b(9, 0, &[7]), b(4, 0, &[7])];
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Merit, &[3.0; 10], 0, 0.0);
        assert_eq!(got.len(), 1);
        // rule 9 precedes rule 4 in INPUT order despite the higher id.
        assert_eq!(got[0].rule_id, 9);
    }

    /// Creation bindings never conflict and are always accepted.
    #[test]
    fn test_creation_bindings_always_accepted() {
        let pool = vec![b(0, 0, &[]), b(1, 0, &[]), b(0, 0, &[5]), b(1, 0, &[5])];
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Merit, &[0.0; 2], 0, 0.0);
        assert_eq!(got.len(), 3);
        assert_eq!(got.iter().filter(|b| b.is_creation()).count(), 2);
    }

    /// Deterministic under repetition regardless of mode.
    #[test]
    fn test_determinism_both_modes() {
        let pool = vec![
            b(0, 0, &[1, 2]),
            b(1, 0, &[2, 3]),
            b(2, 0, &[3, 1]),
            b(1, 2, &[9]),
            b(0, -1, &[8]),
        ];
        for tie in [TieBreak::RuleId, TieBreak::Merit] {
            let a = resolve_conflicts_tiebreak(&pool, tie, &[5.0, 2.0, 9.0], 0, 0.0);
            let c = resolve_conflicts_tiebreak(&pool, tie, &[5.0, 2.0, 9.0], 0, 0.0);
            assert_eq!(a.len(), c.len());
            for (x, y) in a.iter().zip(c.iter()) {
                assert_eq!(x.rule_id, y.rule_id);
                assert_eq!(x.node_ids_sorted(), y.node_ids_sorted());
            }
        }
    }

    /// Random mode: same key → same outcome (determinism), different keys →
    /// the winner of a contested pair varies (the bias is actually gone).
    #[test]
    fn test_random_mode_keyed_and_unbiased() {
        let pool = vec![b(0, 0, &[7]), b(1, 0, &[7])];
        let a = resolve_conflicts_tiebreak(&pool, TieBreak::Random, &[], 1234, 0.0);
        let c = resolve_conflicts_tiebreak(&pool, TieBreak::Random, &[], 1234, 0.0);
        assert_eq!(a[0].rule_id, c[0].rule_id);
        // Across many keys both rules must win sometimes. A fixed-bias
        // resolver (RuleId) would give rule 0 all 64.
        let mut r0_wins = 0;
        for k in 0..64u64 {
            let got = resolve_conflicts_tiebreak(&pool, TieBreak::Random, &[], k, 0.0);
            if got[0].rule_id == 0 {
                r0_wins += 1;
            }
        }
        assert!(
            r0_wins > 8 && r0_wins < 56,
            "winner should vary across keys, got r0_wins={r0_wins}/64"
        );
    }

    /// Random mode never touches the merit table and still respects priority.
    #[test]
    fn test_random_mode_respects_priority() {
        let pool = vec![b(0, 0, &[7]), b(1, 1, &[7])];
        for k in 0..16u64 {
            let got = resolve_conflicts_tiebreak(&pool, TieBreak::Random, &[], k, 0.0);
            assert_eq!(got.len(), 1);
            assert_eq!(got[0].rule_id, 1, "priority must dominate at key {k}");
        }
    }

    /// Jitter at eps=0 is exactly Merit (the noise band collapses).
    #[test]
    fn test_jitter_eps_zero_is_merit() {
        let pool = vec![b(0, 0, &[7]), b(1, 0, &[7])];
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Jitter, &[2.0, 9.0], 0, 0.0);
        assert_eq!(got[0].rule_id, 1, "higher merit must win at eps=0");
    }

    /// Jitter: a newborn (merit 0) beats a low-history incumbent when the
    /// noise band exceeds the merit gap — the bootstrapping fix.
    #[test]
    fn test_jitter_newborn_can_beat_low_history() {
        let pool = vec![b(0, 0, &[7]), b(1, 0, &[7])];
        // merit gap 1; eps=8 gives the newborn a ±50% band around 1 while the
        // incumbent sits at 2·[0.75, 1.25] — overlap in most draws.
        let mut newborn_wins = 0;
        for k in 0..64u64 {
            let got = resolve_conflicts_tiebreak(&pool, TieBreak::Jitter, &[5.0, 0.0], k, 8.0);
            if got[0].rule_id == 1 {
                newborn_wins += 1;
            }
        }
        assert!(
            newborn_wins > 8,
            "newborn must sometimes beat merit-5 incumbent at eps=8, got {newborn_wins}/64"
        );
    }

    /// Jitter: a large merit gap is noise-proof — history still dominates.
    #[test]
    fn test_jitter_history_dominates_when_gap_large() {
        let pool = vec![b(0, 0, &[7]), b(1, 0, &[7])];
        for k in 0..64u64 {
            let got = resolve_conflicts_tiebreak(&pool, TieBreak::Jitter, &[10000.0, 0.0], k, 8.0);
            assert_eq!(
                got[0].rule_id, 0,
                "merit 10000 must be noise-proof at eps=8, key {k}"
            );
        }
    }

    /// Jitter determinism under a fixed key.
    #[test]
    fn test_jitter_deterministic() {
        let pool = vec![b(0, 0, &[7]), b(1, 0, &[7])];
        let a = resolve_conflicts_tiebreak(&pool, TieBreak::Jitter, &[1.0, 1.0], 777, 2.0);
        let c = resolve_conflicts_tiebreak(&pool, TieBreak::Jitter, &[1.0, 1.0], 777, 2.0);
        assert_eq!(a[0].rule_id, c[0].rule_id);
    }

    /// v9 decayed merit: a slumped incumbent (merit 0.5) loses to a fresh
    /// rule with base 1.0 — the entry mechanism cumulative merit lacks.
    /// (The resolver is agnostic to HOW fractional merit arises; the base
    /// is applied by the runner's accumulation.)
    #[test]
    fn test_fractional_merit_ordering() {
        let pool = vec![b(0, 0, &[7]), b(1, 0, &[7])];
        // Incumbent slumped below the newborn's base.
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Merit, &[0.5, 1.0], 0, 0.0);
        assert_eq!(
            got[0].rule_id, 1,
            "base-1.0 newborn must beat 0.5-slumped incumbent"
        );
        // Incumbent still above: history wins.
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Merit, &[2.5, 1.0], 0, 0.0);
        assert_eq!(
            got[0].rule_id, 0,
            "2.5 incumbent must hold against base-1.0 newborn"
        );
    }

    /// Exact integer merit (the cumulative case) orders identically in f64.
    #[test]
    fn test_f64_merit_matches_u64_semantics() {
        let pool = vec![b(0, 0, &[7]), b(1, 0, &[7]), b(2, 0, &[7])];
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Merit, &[3.0, 17.0, 9.0], 0, 0.0);
        assert_eq!(got[0].rule_id, 1);
        // Tie falls to input order, not id.
        let got = resolve_conflicts_tiebreak(&pool, TieBreak::Merit, &[5.0, 5.0, 5.0], 0, 0.0);
        assert_eq!(got[0].rule_id, 0);
    }
}
