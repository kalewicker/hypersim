//! G-GE1 driver + tests (GE plan §4). The ONLY engine-touching part of
//! the growth-bias audit: hg-engine is a dev-dependency, so the crate
//! firewall holds (same pattern as gr3.rs / sweep.rs / gr1_gate.rs).
//!
//! Probe contract (§4.4): fire the valid child ALONE on the shared fixture
//! world (`seed_world_fixed(8)`, the G-R3 world) under MaxParallel, seed
//! 42, W = 100 steps; record the node-count trajectory plus accepted-fire
//! count. Classification thresholds are pre-committed in gge1.rs.

use crate::compile::candidate_to_rule;
use crate::gge1::{classify_trajectory, Envelope, Outcome};
use crate::gr3::seed_world_fixed;
use crate::mutate::{mutate_op, MutationRng, Op};
use crate::schema::CandidateRule;
use crate::validate::{validate, PriorityObservation, Verdict};
use hg_engine::Engine;
use hg_rules::compiled::{compile_rule, compile_rules};
use hg_rules::parser::parse_rules_toml;
use hg_rules::StepMode;

/// Probe window (§4.1): same length as G-R1's Catastrophic horizon.
const W: usize = 100;

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

/// Fire ONE child on the shared fixture world; return (node trajectory
/// including the pre-step baseline, total accepted rewrites).
fn probe(child: &CandidateRule) -> (Vec<usize>, usize) {
    let mut store = seed_world_fixed(8);
    let rule = candidate_to_rule(child);
    let compiled = compile_rules(std::slice::from_ref(&rule)).expect("valid child must compile");
    let mut engine = Engine::new(compiled, StepMode::MaxParallel, 42, 10_000);

    let n0 = store.node_count();
    let mut traj = Vec::with_capacity(W + 1);
    traj.push(n0);
    let mut accepted_total = 0usize;
    for _ in 0..W {
        let st = engine.step(&mut store);
        accepted_total += st.accepted_rewrites;
        traj.push(store.node_count());
        // Bounded compute (gge1::EXPLODE_BAIL_NODES): past this size the
        // trajectory is already REGIME_EXPLODING by a wide margin and the
        // remaining steps are pure CPU burn on exactly the runs the gate
        // flags. The crossing window is fully recorded, so
        // classify_trajectory sees identical evidence either way.
        if traj.last().copied().unwrap_or(0) >= crate::gge1::EXPLODE_BAIL_NODES {
            break;
        }
    }
    (traj, accepted_total)
}

/// One classified draw of operator `op_idx` against `parent`.
fn classify_draw(
    parent: &CandidateRule,
    op_idx: usize,
    rng: &mut dyn MutationRng,
) -> (Op, Outcome, bool, bool) {
    // (op, outcome, late_plateau_flag, inert_flag)
    let op = Op::ALL[op_idx % Op::ALL.len()];
    let (child, applied) = mutate_op(parent, op_idx, rng);
    match applied {
        crate::mutate::Applied::Nothing => return (op, Outcome::Nothing, false, false),
        crate::mutate::Applied::Edited => {}
    }
    match validate(
        &child,
        &crate::gr1::offline_footprint(&child),
        &PriorityObservation { malformed: false },
    ) {
        Verdict::Quarantined(_) => (op, Outcome::Quarantined, false, false),
        Verdict::Valid => {
            // Engine-side compile is the second gate (SB-GE3): the validator
            // battery does not model var-guard consistency, so a child can
            // pass V1..V9 and still be uncompilable. Counted separately.
            let rule = candidate_to_rule(&child);
            if compile_rule(&rule).is_err() {
                return (op, Outcome::Uncompilable, false, false);
            }
            let (traj, accepted) = probe(&child);
            let cls = classify_trajectory(&traj);
            let late = matches!(
                cls,
                crate::gge1::DynClass::Saturating { late_plateau: true }
            );
            (op, Outcome::Dynamic(cls), late, accepted == 0)
        }
    }
}

/// The G-R1 seed set, extended with E1-extractable members of the shipped
/// library (plan §4.4). Extraction failures are EXPECTED rejects — counted,
/// reported, never silently dropped.
fn extended_seed_set() -> (Vec<CandidateRule>, Vec<&'static str>, Vec<String>) {
    use hg_rules::ParseError;

    let mut set: Vec<CandidateRule> = crate::gr1_gate::seed_set();
    let mut skipped: Vec<&'static str> = Vec::new();
    let envelope_reject = || ParseError::InvalidGuard("outside E1 envelope".into());

    // Builtins come from inside hg-rules (include_str — no fs needed).
    for name in hg_rules::BUILTIN_NAMES {
        match hg_rules::builtin_rules(name)
            .ok()
            .and_then(|rules| rules.first().cloned())
            .map(|r| crate::compile::rule_to_candidate(&r).ok_or_else(envelope_reject))
        {
            Some(Ok(cand)) => set.push(cand),
            _ => skipped.push(name),
        }
    }

    // Live-zoo extension from the repo's rules/ directory (test cwd is the
    // crate dir; the repo root is two levels up).
    let mut missing: Vec<String> = Vec::new();
    for fname in [
        "wn_three",
        "excitation_front",
        "fan_merge",
        "relay_pair",
        "tri_two",
        "stage1_reservoir",
    ] {
        let path = format!("../../rules/{fname}.toml");
        match std::fs::read_to_string(&path) {
            Ok(src) => match parse_rules_toml(&src)
                .ok()
                .and_then(|rules| {
                    rules
                        .iter()
                        .find(|r| r.enabled && !r.is_creation())
                        .cloned()
                })
                .map(|r| crate::compile::rule_to_candidate(&r).ok_or_else(envelope_reject))
            {
                Some(Ok(cand)) => set.push(cand),
                _ => skipped.push(fname),
            },
            Err(_) => missing.push(fname.to_string()),
        }
    }
    (set, skipped, missing)
}

// ---------------------------------------------------------------------------
// unit-level pins
// ---------------------------------------------------------------------------

#[test]
fn dispatch_indices_map_to_the_historical_operators() {
    use crate::mutate::Applied;
    let mut rng = XorShift(0x0F05);
    let _ = rng.next_u64();

    // Spot-check the distinctive arms against their documented behavior.
    let mk = || crate::gr1_gate::seed_set()[0].clone();
    let base = mk();

    // Arm 9 = ChangePriority: priority moves within [-3,+3].
    let (r, a) = mutate_op(&base, 9, &mut XorShift(7));
    assert_eq!(a, Applied::Edited);
    assert!(i32::abs_diff(r.priority, base.priority) <= 3);

    // Arm 7 = SetLabel: some var gains a label constraint in 0..8.
    let (r, a) = mutate_op(&base, 7, &mut XorShift(7));
    assert_eq!(a, Applied::Edited);
    assert!(r
        .var_labels
        .iter()
        .zip(base.var_labels.iter())
        .any(|(now, was)| now.is_some() && now != was));

    // Arm 6 = RemovePatternEdge: some side loses an edge (or Nothing).
    let (r, a) = mutate_op(&base, 6, &mut XorShift(11));
    assert_eq!(
        r.lhs.len() + r.rhs.len(),
        base.lhs.len() + base.rhs.len() - if matches!(a, Applied::Edited) { 1 } else { 0 }
    );

    // Every arm is total: no panic, and Edited implies a structural change
    // somewhere in the candidate (priority included). Arms that honestly
    // return Nothing on this seed are fine — but they must be Nothing, not
    // Edited-and-invisible.
    for i in 0..10 {
        let (r, applied) = mutate_op(&mk(), i, &mut XorShift(0x1234 + i as u64));
        let changed = r.lhs != base.lhs
            || r.rhs != base.rhs
            || r.var_labels != base.var_labels
            || r.priority != base.priority;
        assert!(
            changed || applied == Applied::Nothing,
            "arm {i} claimed Edited but changed nothing"
        );
    }

    // Names unique and complete.
    let names: std::collections::BTreeSet<&'static str> =
        Op::ALL.iter().map(|o| o.name()).collect();
    assert_eq!(names.len(), 10);
}

#[test]
fn probe_is_deterministic() {
    // A growth-shaped child: LHS 2-edge over a (label-0, label-1) pair —
    // which the seeded world actually contains — subdividing through a
    // newly created label-1 node.
    let child = CandidateRule {
        priority: 0,
        lhs: vec![slot_edge(&[0, 1])],
        rhs: vec![slot_edge(&[0, 2]), slot_edge(&[2, 1])],
        var_labels: vec![Some(0), Some(1), Some(1)],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    };
    let (t1, a1) = probe(&child);
    let (t2, a2) = probe(&child);
    assert_eq!(t1, t2, "probe trajectories must be bit-identical");
    assert_eq!(a1, a2);
}

#[test]
fn known_growth_shape_actually_grows_and_classifies_hot() {
    // Subdivide shape over a real (0,1) label pair of the seeded world:
    // each firing mints one node, so the run grows monotonically.
    // (+1/step from 8 stays under the 1.5x ratio bar -> EXPANSIONARY,
    // not REGIME_EXPLODING — exactly the distinction the gate draws.)
    let child = CandidateRule {
        priority: 0,
        lhs: vec![slot_edge(&[0, 1])],
        rhs: vec![slot_edge(&[0, 2]), slot_edge(&[2, 1])],
        var_labels: vec![Some(0), Some(1), Some(1)],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    };
    let (traj, accepted) = probe(&child);
    assert!(accepted > 0, "fixture child must fire on the probe world");
    assert!(
        *traj.last().expect("non-empty") > traj[0],
        "subdivide child must grow: {} -> {}",
        traj[0],
        traj.last().expect("non-empty")
    );
    let cls = classify_trajectory(&traj);
    assert!(
        matches!(
            cls,
            crate::gge1::DynClass::Expanding | crate::gge1::DynClass::RegimeExploding
        ),
        "growth shape classified {cls:?}"
    );
}

#[test]
fn pure_delete_shape_never_grows() {
    // LHS a (1,0)+(0,1) edge path — present in the seeded world — RHS one
    // edge: net -1 edge, no creation. Nodes are never deleted, so the node
    // count returns to baseline even as edges thin.
    let child = CandidateRule {
        priority: 0,
        lhs: vec![slot_edge(&[0, 1]), slot_edge(&[1, 2])],
        rhs: vec![slot_edge(&[0, 1])],
        var_labels: vec![Some(1), Some(0), Some(1)],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    };
    let (traj, accepted) = probe(&child);
    assert!(accepted > 0, "delete fixture must fire on the probe world");
    assert!(
        *traj.last().expect("non-empty") <= traj[0],
        "pure delete cannot gain nodes"
    );
    // Direction-neutral saturation (see DynClass docs): the deflating run
    // plateaus at a lower node count — SATURATING, never EXPANSIONARY.
    assert!(matches!(
        classify_trajectory(&traj),
        crate::gge1::DynClass::Balanced | crate::gge1::DynClass::Saturating { .. }
    ));
}

#[test]
fn empty_lhs_child_quarantines_without_probing() {
    let parent = crate::gr1_gate::seed_set()[0].clone();
    // Force an empty-LHS child directly rather than hoping the kernel finds
    // one: RemovePatternEdge is biased against emptying the LHS.
    let child = CandidateRule {
        lhs: Vec::new(),
        ..parent.clone()
    };
    match validate(
        &child,
        &crate::gr1::offline_footprint(&child),
        &PriorityObservation { malformed: false },
    ) {
        Verdict::Quarantined(_) => {}
        Verdict::Valid => panic!("V1 must quarantine an empty-LHS emission"),
    }
    let _ = parent;
}

#[test]
fn envelope_buckets_are_exclusive_and_complete() {
    let (seeds, skipped, missing) = extended_seed_set();
    // Missing zoo files and envelope rejects are reported, not fatal —
    // the seed set just gets smaller (the gate asserts its own floor).
    let _ = (&skipped, &missing);

    let mut env = Envelope::new();
    let draws_per_cell = 3;
    let mut rng = XorShift(0xBEEF);
    for parent in seeds.iter().take(3) {
        for i in 0..10 {
            for _ in 0..draws_per_cell {
                let (op, outcome, late, _inert) = classify_draw(parent, i, &mut rng);
                env.record(op, outcome);
                if late {
                    // late_plateau is a diagnostic subset; recorded inside
                    // Envelope::record via Saturating{late_plateau:true}.
                }
            }
        }
    }
    let total: usize = env.ops.iter().map(|(_, s)| s.total()).sum();
    assert_eq!(total, 3 * 10 * draws_per_cell);
    // Rates stay in [0,1] and the verdict ladder returns something sane.
    assert!((0.0..=1.0).contains(&env.f_hot()));
    let _ = env.verdict();
}

// ---------------------------------------------------------------------------
// THE GATE (§4.4): 24-seed × 10-op × 40-draw ≈ 9.6k classified draws,
// every valid draw probed for W = 100 engine steps. Deterministic end to
// end (XorShift keyed once; probes draw nothing).
//
// Pre-registration status: thresholds committed in the GE plan §4.3 BEFORE
// any number existed. The assertions below ENFORCE the ladder: REDESIGN is
// meant to be loud until the kernel is redesigned (G-R1 precedent — the
// gate asserts its bar).
// ---------------------------------------------------------------------------

#[test]
fn gge1_gate_run_10k_samples() {
    let (seeds, skipped, missing) = extended_seed_set();
    println!("\n=== G-GE1 setup ===");
    println!("seed set          : {} rules", seeds.len());
    println!("envelope-skipped  : {skipped:?}");
    println!("file-missing      : {missing:?}");
    assert!(seeds.len() >= 20, "plan §4.4 requires the G-R1 ≥20 bar");

    // Arity-ceiling precondition carried over from G-R1 (SB-R1 breakout).
    let max_arity_seen = seeds
        .iter()
        .flat_map(|r| r.lhs.iter().chain(r.rhs.iter()))
        .map(|e| e.slots.len())
        .max()
        .unwrap_or(0);
    assert!(
        max_arity_seen + 1 < crate::schema::MAX_ARITY,
        "seed set touches the arity ceiling; SB-R1 breakout required"
    );

    const DRAWS_PER_CELL: usize = 40; // seeds × ops × draws ≈ 9.6k–12k
    let mut env = Envelope::new();
    let mut rng = XorShift(0x6E_11D);

    for parent in &seeds {
        for i in 0..10 {
            for _ in 0..DRAWS_PER_CELL {
                let (op, outcome, late, _inert) = classify_draw(parent, i, &mut rng);
                env.record(op, outcome);
                let _ = late;
            }
        }
    }

    // Exclusivity: every draw landed in exactly one bucket.
    let total: usize = env.ops.iter().map(|(_, s)| s.total()).sum();
    let expect = seeds.len() * 10 * DRAWS_PER_CELL;
    assert_eq!(total, expect, "bucket exclusivity broken");

    println!("\n=== G-GE1 growth-bias report ===");
    println!(
        "{:<18}{:>7}{:>8}{:>10}{:>9}{:>9}{:>9}{:>9}{:>6}{:>20}",
        "operator",
        "valid",
        "uncomp",
        "balanced",
        "satur",
        "expand",
        "explod",
        "hot%",
        "late",
        "coupling"
    );
    for (op, s) in env.ops.iter() {
        println!(
            "{:<18}{:>7}{:>8}{:>10}{:>9}{:>9}{:>9}{:>9.3}{:>6}{:>20}",
            op.name(),
            s.valid(),
            s.uncompilable,
            s.balanced,
            s.saturating,
            s.expanding,
            s.exploding,
            s.expansion_rate(),
            s.late_plateau,
            s.coupling_class().name(),
        );
    }
    let (max_op, max_r) = env.max_expansion_rate();
    println!("f_hot             : {:.4}", env.f_hot());
    println!("hottest operator  : {} ({:.4})", max_op.name(), max_r);

    match env.verdict() {
        crate::gge1::Verdict::Pass { f_hot } => {
            println!("VERDICT           : PASS (f_hot = {f_hot:.4})")
        }
        crate::gge1::Verdict::Descope { f_hot, descoped } => {
            println!("VERDICT           : DESCOPE (f_hot = {f_hot:.4})");
            println!("descoped operators: {descoped:?}");
        }
        crate::gge1::Verdict::Redesign { f_hot } => {
            panic!(
                "G-GE1 REDESIGN (f_hot = {f_hot:.4} > 0.35): the mutation kernel \
                 drives substrate explosion; per GE plan §4.3 the kernel must be \
                 redesigned before EXP-R09 opens. This failure is the gate \
                 working, not a bug."
            )
        }
    }
}

fn slot_edge(slots: &[usize]) -> crate::schema::CandidatePatternEdge {
    crate::schema::CandidatePatternEdge {
        slots: slots.to_vec(),
    }
}
