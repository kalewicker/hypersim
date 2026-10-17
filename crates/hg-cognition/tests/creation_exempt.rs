//! The creation-rule exemption, asserted across the whole policy zoo
//! (`docs/policy_config.md` §3).
//!
//! This is a single-invariant test file on purpose. The engine runs a
//! policy *before* its own creations/ordinary split and does not restore
//! dropped creation bindings — the comment at
//! `crates/hg-engine/src/engine.rs:163` says so explicitly. So the
//! exemption is not enforced anywhere except in the policies themselves,
//! and a policy that forgets it can starve a scheduled input-injection or
//! seeding rule for an entire run. That failure would not crash; it would
//! quietly produce a Frozen result that reads like a finding.
//!
//! Every policy added to the zoo must be added here. `StimulusDriver` is
//! the one sanctioned exception and is tested for its *specific* gating
//! behaviour instead (`policies::stack::tests`), not for blanket
//! pass-through.

use hg_cognition::config::DegreeDirection;
use hg_cognition::testkit::{candidate_pool, creation, matched, node_ids, ring_store};
use hg_cognition::{
    DegreeGreedy, MlpScorer, Parameterized, PassThrough, PolicyStack, RandomSubset, RuleBias,
    StaticBias, StimulusDriver,
};
use hg_observe::RuleSelectionPolicy;

const N_RULES: usize = 4;
const BUDGET: usize = 6;

fn zoo() -> Vec<(&'static str, Box<dyn RuleSelectionPolicy>)> {
    let mut mlp = MlpScorer::new(N_RULES, 16, 8, Some(BUDGET), 4242);
    let n = mlp.n_params();
    mlp.set_params(&(0..n).map(|i| ((i as f32) * 0.11).cos()).collect::<Vec<_>>());

    let mut bias = StaticBias::new(N_RULES, Some(BUDGET), 99);
    bias.set_params(&[1.0, -2.0, 0.5, 3.0]);

    vec![
        ("PassThrough", Box::new(PassThrough)),
        ("RandomSubset", Box::new(RandomSubset::new(Some(BUDGET), 7))),
        (
            "DegreeGreedy",
            Box::new(DegreeGreedy::new(Some(BUDGET), DegreeDirection::High)),
        ),
        (
            "RuleBias",
            Box::new(RuleBias::new(vec![1.0, 0.0, 5.0, 2.0], Some(BUDGET), 7)),
        ),
        ("StaticBias", Box::new(bias)),
        ("MlpScorer", Box::new(mlp)),
        (
            "PolicyStack",
            Box::new(PolicyStack::new(
                // A driver whose gated pair (rules 90/91) is absent from the
                // pool, so the stack's creation half is pure pass-through
                // and the learned half is what is under test.
                Box::new(StimulusDriver::new(90, 91, vec![true], 8)),
                Box::new(RandomSubset::new(Some(BUDGET), 7)),
            )),
        ),
    ]
}

#[test]
fn every_policy_passes_creation_bindings_through() {
    let store = ring_store(64, 8);
    let snap = store.snapshot();
    // 5 creation bindings, then 80 pattern matches — a budget of 6 forces
    // every budgeted policy to discard most of the pool.
    let cands = candidate_pool(&store, 5, 80, N_RULES, 2);
    let expected: Vec<usize> = (0..cands.len()).filter(|&i| cands[i].is_creation()).collect();

    for (name, mut policy) in zoo() {
        let picked = policy.select(&snap, &cands);
        let got: Vec<usize> = picked
            .iter()
            .copied()
            .filter(|&i| cands[i].is_creation())
            .collect();
        assert_eq!(got, expected, "{name} dropped a creation binding");
    }
}

#[test]
fn every_budgeted_policy_holds_its_budget_on_pattern_matches() {
    let store = ring_store(64, 8);
    let snap = store.snapshot();
    let cands = candidate_pool(&store, 5, 80, N_RULES, 2);

    for (name, mut policy) in zoo() {
        if name == "PassThrough" {
            continue; // 0 params, no budget, by definition
        }
        let picked = policy.select(&snap, &cands);
        let n_match = picked.iter().filter(|&&i| !cands[i].is_creation()).count();
        // RuleBias intentionally leaves the budget unspent when a rule is
        // zero-weighted; every other policy fills it exactly.
        assert!(
            n_match <= BUDGET,
            "{name} selected {n_match} pattern matches over a budget of {BUDGET}"
        );
        if name != "RuleBias" {
            assert_eq!(n_match, BUDGET, "{name} under-spent its budget");
        }
    }
}

#[test]
fn every_policy_returns_valid_deduplicated_indices() {
    let store = ring_store(64, 8);
    let snap = store.snapshot();
    let cands = candidate_pool(&store, 3, 50, N_RULES, 3);

    for (name, mut policy) in zoo() {
        let picked = policy.select(&snap, &cands);
        assert!(
            picked.iter().all(|&i| i < cands.len()),
            "{name} returned an out-of-range index"
        );
        let mut sorted = picked.clone();
        sorted.sort_unstable();
        let mut dedup = sorted.clone();
        dedup.dedup();
        assert_eq!(sorted, dedup, "{name} returned a duplicate index");
    }
}

/// A pool with nothing to select from must not panic, divide by zero, or
/// invent an index. Empty steps happen: every rule's guards can fail on the
/// same step, and a policy that panics there takes the whole rollout down
/// mid-campaign.
#[test]
fn every_policy_survives_an_empty_and_a_creation_only_pool() {
    let store = ring_store(16, 2);
    let snap = store.snapshot();

    for (name, mut policy) in zoo() {
        assert!(
            policy.select(&snap, &[]).is_empty(),
            "{name} invented a selection from an empty pool"
        );
    }

    let only_creations = vec![creation(0, 0), creation(1, 0), creation(2, 0)];
    for (name, mut policy) in zoo() {
        let picked = policy.select(&snap, &only_creations);
        assert_eq!(picked, vec![0, 1, 2], "{name} mishandled a creation-only pool");
    }
}

/// A single pattern match under a budget of 6: policies must return it, not
/// pad, not loop looking for more.
#[test]
fn every_policy_handles_a_pool_smaller_than_the_budget() {
    let store = ring_store(16, 2);
    let snap = store.snapshot();
    let ids = node_ids(&store);
    let cands = vec![creation(0, 0), matched(2, 0, &ids[0..2])];

    for (name, mut policy) in zoo() {
        let picked = policy.select(&snap, &cands);
        assert_eq!(picked, vec![0, 1], "{name} mishandled an under-budget pool");
    }
}
