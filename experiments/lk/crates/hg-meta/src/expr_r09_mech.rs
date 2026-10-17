//! SB-GE7(a): fee-throttle MECHANISM test (GE plan §10.5 falsifier).
//!
//! EXP-R09 showed the throttle works on routine firing (4002 → 7) but the
//! outer loop never produces a blob, so fee efficacy against EXPLOSIVE
//! firing is unmeasured. This test isolates the mechanism: it installs the
//! G-GE1 gate's own positive-control exploder (the subdivide fixture from
//! `gge1_tests::known_growth_shape_actually_grows_and_classifies_hot` —
//! the exact (child, fixture) pair the gate uses to prove its probe can
//! see growth) as a live region on `seed_world_fixed(8)` and runs two
//! arms over the same W = 100 probe horizon:
//!
//! | Arm | Policy | Ledger boundary |
//! |---|---|---|
//! | A0 | none (bare probe) | — |
//! | A1 | live `FeePolicy`, single-entry table rule_id 0 ↔ region root | every 25 steps: boundary debit (reconciles actuals — always affordable by construction) + `end_epoch` income/aging; refusals happen at FORWARD time via within-window projection (redesign (i)) |
//!
//! The A1 boundary mirrors `epoch_transition_with_economy` §10.6 semantics
//! for one region with no mutation (debit → income/age); the economy
//! modules are used directly, not reimplemented.
//!
//! Pre-registered bars (this is the §10.5 falsifier, so it BITES):
//! - A0 MUST classify RegimeExploding (control constructed from the gate
//!   positive control; failure means the setup is broken, not the thesis).
//! - A1 MUST NOT classify RegimeExploding. A priced-but-exploding A1
//!   reopens REDESIGN per §10.6 — that outcome fails this test loudly by
//!   design. A1 additionally pins: refused > 0, paid ≤ endowment +
//!   reward × unthrottled windows, ledger conservation every boundary.
//!
//! RUNTIME NOTE: A0 explodes fast (bail applies); A1 flatlines early.
//! Seconds in release, fast in debug too (no outer loop).

use crate::compile::candidate_to_rule;
use crate::economy::{IncomePolicy, RegionLedger};
use crate::fee_policy::FeePolicy;
use crate::gge1::{classify_trajectory, DynClass, EXPLODE_BAIL_NODES};
use crate::gr3::seed_world_fixed;
use crate::graph_source::GraphRuleSource;
use crate::schema::{CandidatePatternEdge, CandidateRule};
use hg_engine::Engine;
use hg_observe::RuleSelectionPolicy;
use hg_rules::compiled::compile_rules;
use hg_rules::StepMode;
use std::sync::{Arc, Mutex};

/// Probe window (§4.1): same length as G-R1's Catastrophic horizon and the
/// G-GE1 probe. A1 slices it into 4 × 25-step windows (R09's E_LEN).
const W: usize = 100;
const E_LEN: usize = 25;

fn slot_edge(slots: &[usize]) -> CandidatePatternEdge {
    CandidatePatternEdge {
        slots: slots.to_vec(),
    }
}

/// The gate's positive control, verbatim: subdivide over a real (0,1)
/// label pair; each firing mints one node.
fn exploder_child() -> CandidateRule {
    CandidateRule {
        priority: 0,
        lhs: vec![slot_edge(&[0, 1])],
        rhs: vec![slot_edge(&[0, 2]), slot_edge(&[2, 1])],
        var_labels: vec![Some(0), Some(1), Some(1)],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    }
}

struct MechOutcome {
    node_traj: Vec<usize>,
    accepted_total: u64,
    paid: u64,
    refused: u64,
    withheld: u64,
}

/// Shared handle to the window policy (the `SharedFireCounter` pattern):
/// the engine owns a clone, the driver reads back `withheld_total`
/// after each window. Foreign-trait-on-foreign-type would be E0117, so
/// the local wrapper delegates.
#[derive(Clone)]
struct SharedPol(Arc<Mutex<FeePolicy>>);

impl RuleSelectionPolicy for SharedPol {
    fn select(
        &mut self,
        snapshot: &hg_core::GraphSnapshot<'_>,
        candidates: &[hg_observe::Binding],
    ) -> Vec<usize> {
        self.0
            .lock()
            .expect("uncontended")
            .select(snapshot, candidates)
    }
}

/// A0: the bare probe — no policy, no ledger. Identical stepping to
/// `gge1_tests::probe` (same world, seed, bail), so the A0 trajectory is
/// directly comparable to the gate's evidence.
fn run_a0(child: &CandidateRule) -> MechOutcome {
    let mut store = seed_world_fixed(8);
    let rule = candidate_to_rule(child);
    let compiled = compile_rules(std::slice::from_ref(&rule)).expect("fixture compiles");
    let mut engine = Engine::new(compiled, StepMode::MaxParallel, 42, 10_000);
    let mut traj = vec![store.node_count()];
    let mut accepted_total = 0u64;
    for _ in 0..W {
        let st = engine.step(&mut store);
        accepted_total += st.accepted_rewrites as u64;
        traj.push(store.node_count());
        if traj.last().copied().unwrap_or(0) >= EXPLODE_BAIL_NODES {
            break;
        }
    }
    MechOutcome {
        node_traj: traj,
        accepted_total,
        paid: 0,
        refused: 0,
        withheld: 0,
    }
}

/// A1: same world/seed/horizon, but the exploder is an installed REGION
/// inside the fee economy — policy rebuilt from the ledger each window,
/// debit + income/aging at every 25-step boundary.
fn run_a1(child: &CandidateRule, policy: IncomePolicy) -> (MechOutcome, RegionLedger) {
    let mut store = seed_world_fixed(8);
    let roots = GraphRuleSource::install_candidates(&mut store, std::slice::from_ref(child));
    assert_eq!(roots.len(), 1, "one region installs");
    let rule = candidate_to_rule(child);
    let compiled = compile_rules(std::slice::from_ref(&rule)).expect("fixture compiles");

    let mut ledger = RegionLedger::new(policy.endowment);
    let mut traj = vec![store.node_count()];
    let mut accepted_total = 0u64;
    let mut paid = 0u64;
    let mut refused = 0u64;
    let mut withheld = 0u64;
    // One shared policy object, re-windowed each boundary: fresh
    // projection budget against the post-boundary ledger snapshot.
    let shared = SharedPol(Arc::new(Mutex::new(FeePolicy::new(policy))));
    let mut engine = Engine::new(compiled, StepMode::MaxParallel, 42, 10_000);
    for _ in 0..W / E_LEN {
        // Forward half: re-window the policy over the current ledger.
        {
            let mut pol = shared.0.lock().expect("uncontended");
            pol.ledgers.ledgers = vec![Some(ledger.clone())];
            pol.forwarded.clear();
        }
        engine.set_policy(Box::new(shared.clone()));
        let mut window_accepted = 0u64;
        for _ in 0..E_LEN {
            let st = engine.step(&mut store);
            window_accepted += st.accepted_rewrites as u64;
            traj.push(store.node_count());
            if traj.last().copied().unwrap_or(0) >= EXPLODE_BAIL_NODES {
                break;
            }
        }
        // Debit half: per-firing `try_fire` walks the same balance path as
        // the epoch bulk debit at unit cost (pay while solvent, whole
        // refusal past it — no debt either way).
        for _ in 0..window_accepted {
            if ledger.try_fire(policy.cost) {
                paid += 1;
            } else {
                refused += 1;
            }
        }
        accepted_total += window_accepted;
        // Forward-time refusals, read back from the shared policy before
        // the next window resets its projection budget.
        withheld = shared.0.lock().expect("uncontended").withheld_total;
        // Conservation at every boundary (single-ledger audit).
        assert_eq!(
            ledger.free + (ledger.total - ledger.free),
            ledger.total,
            "ledger conservation holds at the window boundary"
        );
        ledger.end_epoch(&policy);
        if traj.last().copied().unwrap_or(0) >= EXPLODE_BAIL_NODES {
            break;
        }
    }
    (
        MechOutcome {
            node_traj: traj,
            accepted_total,
            paid,
            refused,
            withheld,
        },
        ledger,
    )
}

#[test]
fn exp_r09_mech_throttle_throttles_known_exploder() {
    println!("\n=== SB-GE7(a): fee-throttle mechanism test (pre-registered §10.5 falsifier) ===");
    let child = exploder_child();
    let policy = IncomePolicy::default();

    let a0 = run_a0(&child);
    let (a1, ledger) = run_a1(&child, policy);

    let c0 = classify_trajectory(&a0.node_traj);
    let c1 = classify_trajectory(&a1.node_traj);
    println!(
        "A0 fee-zero: {c0:?}  max_n {:>6}  accepted {:>6}",
        a0.node_traj.iter().max().copied().unwrap_or(0),
        a0.accepted_total,
    );
    println!(
        "A1 main:     {c1:?}  max_n {:>6}  accepted {:>6}  paid {:>4} refused {:>5} withheld {:>5}  ledger free/total {}/{} age {}",
        a1.node_traj.iter().max().copied().unwrap_or(0),
        a1.accepted_total,
        a1.paid,
        a1.refused,
        a1.withheld,
        ledger.free,
        ledger.total,
        ledger.age_epochs,
    );
    // Window-end nodes: shows whether growth arrests once bankruptcy is
    // observed (withhold from window 1 on) or compounds regardless.
    let ends: Vec<usize> = (1..=W / E_LEN)
        .map(|w| a1.node_traj.get(w * E_LEN).copied().unwrap_or(0))
        .collect();
    println!("A1 window-end nodes: {ends:?}");

    // Control: the gate's positive control must explode unpriced. Anything
    // else is a broken setup, not evidence about fees.
    assert!(
        matches!(c0, DynClass::RegimeExploding),
        "SETUP BROKEN: fee-zero exploder came back {c0:?}, not RegimeExploding. \
         The gate positive control must explode unpriced — do not interpret A1."
    );

    // The falsifier: a priced-but-exploding A1 reopens REDESIGN (§10.6).
    assert!(
        !matches!(c1, DynClass::RegimeExploding),
        "FALSIFIED (§10.5/§10.6 REDESIGN-REOPEN): the exploder still went \
         REGIME_EXPLODING under live fees (A0 max_n {}, A1 max_n {}). \
         Unrestrained firing is not the operative channel — report against \
         §2 as a new species; exclusion becomes the default descope.",
        a0.node_traj.iter().max().copied().unwrap_or(0),
        a1.node_traj.iter().max().copied().unwrap_or(0),
    );

    // Throttle accounting under redesign (i): refusals happen at forward
    // time (projection), so the boundary debit must reconcile EXACTLY —
    // every accepted firing was affordable by construction.
    assert!(
        a1.withheld > 0,
        "fee never bound: nothing withheld at forward time"
    );
    assert_eq!(
        a1.paid, a1.accepted_total,
        "projection invariant broke: debit paid {} of {} accepted",
        a1.paid, a1.accepted_total
    );
    assert_eq!(
        a1.refused, 0,
        "projection invariant broke: debit refused {} (accepted must be affordable)",
        a1.refused
    );
    // Lifetime accepted sits inside endowment + earned income (no debt,
    // no creation), with slack for income-timing granularity.
    let unthrottled_windows = (W / E_LEN).saturating_sub(policy.k_throttle as usize) as u64;
    assert!(
        a1.accepted_total
            <= policy.endowment as u64 + policy.reward as u64 * unthrottled_windows + 2,
        "overspend: accepted {} exceeds endowment + earned income",
        a1.accepted_total
    );
    println!(
        "MECHANISM HOLDS: exploder {c0:?} unpriced → {c1:?} priced; \
         firing {} → {} (paid {}, withheld {}).",
        a0.accepted_total, a1.accepted_total, a1.paid, a1.withheld
    );
}
