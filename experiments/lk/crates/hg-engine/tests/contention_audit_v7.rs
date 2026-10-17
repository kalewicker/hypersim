//! v7 §1.2 directional contention audit: the counted resolver must
//! return the SAME accepted set as the legacy one (so wiring it changes
//! no trajectory), and must attribute refusals to the correct winner.

use hg_core::NodeId;
use hg_engine::conflict::{resolve_conflicts, resolve_conflicts_counted};
use hg_observe::Binding;
use slotmap::KeyData;
use smallvec::smallvec;

fn node(i: u64) -> NodeId {
    KeyData::from_ffi((1u64 << 32) | i).into()
}

/// Bindings over a shared node, at different priorities.
fn binding(rule_id: usize, priority: i32, nodes: &[u64]) -> Binding {
    Binding::with_edges(
        rule_id,
        priority,
        nodes.iter().copied().map(node).collect(),
        smallvec![],
    )
}

/// The counted resolver's accepted set is byte-identical to the legacy
/// one across a spread of overlapping cases — the precondition for using
/// it in the live engine.
#[test]
fn counted_resolution_matches_legacy() {
    let cases: Vec<Vec<Binding>> = vec![
        // disjoint: all accepted
        vec![binding(0, 0, &[1, 2]), binding(0, 0, &[3, 4])],
        // shared node, different priority: higher priority wins
        vec![binding(1, 2, &[5]), binding(0, -1, &[4, 5, 6])],
        // chain of overlaps
        vec![
            binding(0, 1, &[1, 2]),
            binding(0, 1, &[2, 3]),
            binding(0, 1, &[3, 4]),
            binding(1, 2, &[2]),
        ],
        // identical bound sets, different rules
        vec![binding(0, 0, &[7, 8]), binding(1, 5, &[7, 8])],
        vec![],
    ];
    for (i, case) in cases.iter().enumerate() {
        let legacy = resolve_conflicts(case);
        let (counted, _) = resolve_conflicts_counted(case);
        assert_eq!(
            legacy.len(),
            counted.len(),
            "case {i}: accepted count differs"
        );
        for (a, b) in legacy.iter().zip(counted.iter()) {
            assert_eq!(a.rule_id, b.rule_id, "case {i}: rule");
            assert_eq!(
                a.bound_nodes.as_slice(),
                b.bound_nodes.as_slice(),
                "case {i}: bound nodes"
            );
        }
    }
}

/// The F-LK22 shape, directional: a single-node binding at HIGH priority
/// (the `lk_tick` analogue) claims a node that a three-node binding at
/// LOW priority (the `lk_fuse` analogue) needs — so fuse loses to tick,
/// and the counter says exactly that.
#[test]
fn tick_shaped_binding_beats_fuse_shaped_and_is_attributed() {
    const TICK: usize = 1;
    const FUSE: usize = 0;
    let bindings = vec![
        binding(FUSE, -1, &[10, 11, 12]), // fuse on w=11 with x=10, y=12
        binding(TICK, 2, &[10]),          // tick claims neighbour x
    ];
    let (accepted, losses) = resolve_conflicts_counted(&bindings);
    assert_eq!(accepted.len(), 1, "only the tick may fire");
    assert_eq!(accepted[0].rule_id, TICK);
    assert_eq!(
        losses,
        vec![(FUSE, TICK, 1)],
        "the refusal must be attributed to the tick as winner"
    );

    // Converse direction, explicitly: when the neighbour is NOT
    // tick-eligible there is no contention and fuse fires. Auditing only
    // one direction is the F-LK22 error class, so both are pinned.
    let (accepted, losses) = resolve_conflicts_counted(&[binding(FUSE, -1, &[10, 11, 12])]);
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].rule_id, FUSE);
    assert!(losses.is_empty(), "no contention, no refusal");
}
