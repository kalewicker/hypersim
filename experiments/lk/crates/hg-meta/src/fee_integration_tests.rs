//! GE plan §10.1 — integration test: drive a real `Engine` with `FeePolicy`
//! installed, prove A0 (fee=0) matches the no-policy baseline, prove A1
//! (fee>0, bankrupt region) withholds every binding.
//!
//! Crate discipline: `hg-engine` is already a dev-dep, so this file sits
//! behind `#[cfg(test)]` like the other engine-touching harnesses. The
//! intent is to lock the engine ↔ policy contract end-to-end: that the
//! `select` body in `fee_policy.rs` actually withholds the bankrupt
//! region at the engine level (not just in unit-test land), and that
//! the A0 control is observably inert.
//!
//! What the test does NOT do: assert anything about the per-region
//! ledger debits (that is `economy.rs`'s job, hermetic); wire the
//! runner config (a separate, larger change); implement the epoch
//! loop (lives in `epoch.rs` and gets composed at the driver level).

#![cfg(test)]

use crate::economy::{IncomePolicy, RegionLedger};
use crate::fee_policy::{select_all_when_fee_is_zero, FeePolicy};
use crate::mutate::mutate;
use crate::mutate::MutationRng;
use hg_engine::Engine;
use hg_rules::compiled::compile_rules;
use hg_rules::StepMode;

/// 8-node 6-edge fixture, identical to the G-GE1 + G-R3 harnesses. Has
/// label-0 / label-1 / label-2 nodes; the growth-shaped rule below
/// matches a (label-0, label-1) pair — the engine fires it on every
/// step, which is the stress case for the fee policy.
fn growth_rule() -> hg_rules::Rule {
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
    hg_rules::parser::parse_rules_toml(toml)
        .expect("growth rule parses")
        .into_iter()
        .next()
        .expect("at least one rule")
}

fn seed_world() -> hg_core::HypergraphStore {
    let mut s = hg_core::HypergraphStore::new();
    for i in 0..8 {
        let mut d = hg_core::NodeData::new(hg_core::Label((i % 4) as u32), 0);
        d.set_property(1, hg_core::PropertyValue::Float((i as f64) * 0.125));
        s.add_node(d);
    }
    let ids: Vec<_> = s.nodes().map(|(id, _)| id).collect();
    for w in 0..6usize {
        let a = ids[w % ids.len()];
        let b = ids[(w * 3 + 1) % ids.len()];
        if a != b {
            s.add_edge(hg_core::HyperEdge::new(
                hg_core::Label((w % 3) as u32),
                smallvec::smallvec![a, b],
            ));
        }
    }
    s
}

/// One deterministically-seeded mutator: the G-R1 XorShift.
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

/// Run W steps of an engine and return the total accepted-rewrites count.
fn run_with_policy(policy: Option<Box<dyn hg_observe::RuleSelectionPolicy>>, w: usize) -> usize {
    let r = growth_rule();
    let compiled = compile_rules(std::slice::from_ref(&r)).expect("growth rule compiles");
    let mut engine = Engine::new(compiled, StepMode::MaxParallel, 42, 10_000);
    if let Some(p) = policy {
        engine.set_policy(p);
    }
    let mut store = seed_world();
    let mut accepted_total = 0usize;
    for _ in 0..w {
        let st = engine.step(&mut store);
        accepted_total += st.accepted_rewrites;
    }
    accepted_total
}

#[test]
fn a0_fee_zero_policy_is_inert_compared_to_no_policy() {
    // The §10.3 A0 control contract: fee = 0, throttle off, all rules
    // live — this MUST reproduce the no-policy baseline. The engine
    // counts what actually fired; the policy (even installed) must not
    // change the count.
    let n0 = run_with_policy(None, 50);
    let n1 = run_with_policy(Some(Box::new(select_all_when_fee_is_zero())), 50);
    assert_eq!(
        n0, n1,
        "A0 (fee=0) changed accepted_rewrites: {n0} -> {n1}. Policy is not inert."
    );
    // Sanity: the rule should fire plenty on this fixture (label-0/1 pairs
    // are abundant). If it ever stops firing entirely, the fixture
    // changed and this test is no longer proving what it claims.
    assert!(n0 > 0, "baseline accepted no rewrites — fixture is dead");
}

#[test]
fn a1_with_broken_ledger_withholds_every_firing() {
    // A fee-1 policy against a region whose endowment is 0: every
    // binding is refused at the policy seam, so accepted_rewrites must
    // be zero. This proves the engine↔policy plumbing, not just the
    // unit test in fee_policy.rs.
    let mut pol = FeePolicy::new(IncomePolicy {
        cost: 1,
        reward: 1,
        endowment: 0,
        k_throttle: 0,
    });
    // Map rule 0 (the only rule in the compiled table) to a bankrupt
    // ledger. The select body should never forward any binding.
    pol.ledgers.ledgers.push(Some(RegionLedger::new(0)));
    let n = run_with_policy(Some(Box::new(pol)), 50);
    assert_eq!(
        n, 0,
        "A1 with bankrupt ledger let {n} firings through; engine↔policy contract broken"
    );
}

#[test]
fn a1_with_solvent_ledger_is_capped_by_projection_not_by_window() {
    // AMENDED by redesign (i), SB-GE7(a): the old pin asserted a solvent
    // ledger leaves the 50-step fire count unchanged (snapshot-per-window
    // forwarding). That contract was FALSIFIED — a solvent window
    // forwards everything, letting a fast exploder through on endowment
    // alone. New contract: within-window projection caps FORWARDED
    // bindings at the window-start balance (endowment 3, cost 1 ⇒ at most
    // 3), so accepted sits at or under the cap while the no-policy
    // baseline fires freely. If this fails, the projection has shifted.
    let mut pol = FeePolicy::new(IncomePolicy::default());
    pol.ledgers.ledgers.push(Some(RegionLedger::new(3)));
    let n0 = run_with_policy(None, 50);
    let n1 = run_with_policy(Some(Box::new(pol)), 50);
    assert!(
        n0 > 3,
        "baseline too quiet to prove the cap binds (n0={n0}); harness changed"
    );
    assert!(
        n1 <= 3,
        "projection leaked: solvent-ledger A1 accepted {n1} > endowment 3 over 50 steps"
    );
}

#[test]
fn mutate_is_total_and_distributable_across_calls() {
    // Spot-check: the same mutate calls used by epoch.rs (10-op single
    // edit) compose with the XorShift used by gr1_gate, producing
    // either a real edit or `Nothing`. This guards the integration's
    // RNG-stream expectations.
    let r = growth_rule();
    let cand = crate::compile::rule_to_candidate(&r).expect("growth rule -> candidate");
    let mut rng = XorShift(0x6E_11D);
    for _ in 0..200 {
        let (_, applied) = mutate(&cand, &mut rng);
        assert!(
            matches!(
                applied,
                crate::mutate::Applied::Edited | crate::mutate::Applied::Nothing
            ),
            "mutate must be total"
        );
    }
}
