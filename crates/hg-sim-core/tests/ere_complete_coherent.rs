#![cfg(feature = "hg-meta")]
//! Bounded deterministic development checks, not a G-R1 sample or admission gate.
#[allow(dead_code)]
#[path = "support/coherent.rs"]
mod coherent;
#[allow(dead_code)]
#[path = "support/complete.rs"]
mod complete;
#[allow(dead_code)]
#[path = "support/witness.rs"]
mod witness;

use coherent::{apply, slots, targets, Request, Role, Status};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyOp, PropertyValue};
use hg_meta::*;

fn candidate(arity: usize) -> CandidateRule {
    CandidateRule {
        labels: vec![None; arity],
        lhs: vec![(0..arity).collect()],
        rhs: vec![(0..arity).collect()],
        ..CandidateRule::default()
    }
}
fn fixture(c: &CandidateRule, active: bool) -> (HypergraphStore, NodeId) {
    let mut s = HypergraphStore::new();
    let root = encode_complete_family(&mut s, c, c, active, &MetaLimits::default())
        .unwrap()
        .root;
    (s, root)
}
fn bytes(s: &HypergraphStore) -> Vec<u8> {
    bincode::serialize(s).unwrap()
}
fn decode(s: &HypergraphStore, r: NodeId) -> RuleProgram {
    read_complete_program(&s.snapshot(), r, &MetaLimits::default()).unwrap()
}
fn scope(s: &HypergraphStore, r: NodeId, role: Role) -> coherent::Scope {
    targets(s, r, role, &MetaLimits::default()).unwrap()
}
fn ordered_vars(s: &HypergraphStore, sc: &coherent::Scope) -> Vec<NodeId> {
    let mut vs = sc.variables.clone();
    vs.sort_by_key(
        |n| match s.get_node(*n).unwrap().get_property(K_ORDER).unwrap() {
            PropertyValue::Int(i) => i,
            _ => panic!("order type"),
        },
    );
    vs
}
fn owned_count(s: &HypergraphStore, r: NodeId) -> i64 {
    match s.get_node(r).unwrap().get_property(K_OWNED_COUNT).unwrap() {
        PropertyValue::Int(i) => i,
        _ => panic!("count type"),
    }
}
fn applied(s: &mut HypergraphStore, r: NodeId, role: Role, req: Request) -> coherent::Attempt {
    let before = decode(s, r);
    let a = apply(s, r, role, req, &MetaLimits::default());
    assert_eq!(a.status, Status::Applied, "{}", a.receipt);
    let after = decode(s, r);
    after.compile(&MetaLimits::default(), false).unwrap();
    assert_eq!(a.before_program, Some(before.clone()));
    assert_eq!(a.after_program, Some(after.clone()));
    assert_eq!(bytes(&a.staged), bytes(s));
    assert_eq!(a.receipt["committed"], true);
    assert_eq!(a.receipt["codec_reencoded"], false);
    assert_eq!(a.receipt["metadata_recounted"], false);
    assert_eq!(a.receipt["advancing_engine_steps"], 0);
    assert_eq!(a.receipt["training_admitted"], false);
    if role == Role::Core {
        assert_eq!(after.proposal, before.proposal);
    } else {
        assert_eq!(after.core, before.core);
    }
    a
}

#[test]
fn shared_pattern_slot_append_is_a_real_coherent_edit() {
    for role in [Role::Core, Role::Proposal] {
        let mut expected = candidate(3);
        let (mut s, r) = fixture(&expected, true);
        let sc = scope(&s, r, role);
        assert_eq!(sc.patterns.len(), 1); // physical LHS/RHS sharing
        let p = sc.patterns[0];
        let v = ordered_vars(&s, &sc)[0];
        let count = owned_count(&s, r);
        let a = applied(
            &mut s,
            r,
            role,
            Request::AddSlot {
                pattern: p,
                variable: v,
            },
        );
        expected.lhs[0].push(0);
        expected.rhs[0].push(0);
        let after = decode(&s, r);
        assert_eq!(
            if role == Role::Core {
                after.core
            } else {
                after.proposal.unwrap()
            },
            expected
        );
        assert_eq!(owned_count(&s, r), count);
        assert!(!a.receipt["actions"].as_array().unwrap().is_empty());
    }
}

#[test]
fn slot_compaction_retarget_and_pruning_match_independent_semantics() {
    for role in [Role::Core, Role::Proposal] {
        for retarget in [false, true] {
            let mut c = candidate(3);
            c.labels[1] = Some(2);
            c.guards.push(Guard {
                var: 1,
                key: 42,
                op: PropertyOp::Eq,
                value: PropertyValue::Int(1),
            });
            let (mut s, r) = fixture(&c, true);
            let sc = scope(&s, r, role);
            let vs = ordered_vars(&s, &sc);
            let slot = slots(&s, sc.patterns[0])[1];
            let count = owned_count(&s, r);
            let req = if retarget {
                Request::RetargetSlot {
                    slot,
                    variable: vs[0],
                }
            } else {
                Request::RemoveSlot { slot }
            };
            applied(&mut s, r, role, req);
            let mut expected = candidate(2);
            if retarget {
                expected.lhs = vec![vec![0, 0, 1]];
                expected.rhs = expected.lhs.clone();
            }
            let after = decode(&s, r);
            assert_eq!(
                if role == Role::Core {
                    after.core
                } else {
                    after.proposal.unwrap()
                },
                expected
            );
            assert_eq!(owned_count(&s, r), count - 3); // variable, literal, guard
            assert!(s.get_node(vs[1]).is_none());
            let ss = slots(&s, sc.patterns[0]);
            for (i, id) in ss.iter().enumerate() {
                assert_eq!(s.get_edge(*id).unwrap().label.0, E_SLOT_BASE + i as u32);
            }
        }
    }
}

#[test]
fn add_and_remove_variable_preserve_complete_body_accounting() {
    for role in [Role::Core, Role::Proposal] {
        let (mut s, r) = fixture(&candidate(3), true);
        let sc = scope(&s, r, role);
        let before = owned_count(&s, r);
        applied(
            &mut s,
            r,
            role,
            Request::AddVar {
                pattern: sc.patterns[0],
            },
        );
        assert_eq!(owned_count(&s, r), before + 1);
        let after = decode(&s, r);
        assert_eq!(
            if role == Role::Core {
                after.core
            } else {
                after.proposal.unwrap()
            },
            candidate(4)
        );
        let added = ordered_vars(&s, &scope(&s, r, role))[3];
        applied(&mut s, r, role, Request::RemoveVar { variable: added });
        assert_eq!(owned_count(&s, r), before);
        let after = decode(&s, r);
        assert_eq!(
            if role == Role::Core {
                after.core
            } else {
                after.proposal.unwrap()
            },
            candidate(3)
        );
    }
}

#[test]
fn variable_removal_drops_small_patterns_and_all_repeated_slots() {
    let c = CandidateRule {
        labels: vec![None; 3],
        lhs: vec![vec![0, 0, 1], vec![1, 2]],
        rhs: vec![vec![0, 0, 1], vec![1, 2]],
        ..CandidateRule::default()
    };
    let (mut s, r) = fixture(&c, false);
    let sc = scope(&s, r, Role::Core);
    let v = ordered_vars(&s, &sc)[0];
    applied(&mut s, r, Role::Core, Request::RemoveVar { variable: v });
    assert_eq!(decode(&s, r).core, candidate(2));
    assert_eq!(scope(&s, r, Role::Core).patterns.len(), 1);
}

#[test]
fn pattern_edits_preserve_sharing_multiplicity_and_side_order() {
    for role in [Role::Core, Role::Proposal] {
        for right in [false, true] {
            let (mut s, r) = fixture(&candidate(2), true);
            let sc = scope(&s, r, role);
            let vs = ordered_vars(&s, &sc);
            let count = owned_count(&s, r);
            let side = if right { sc.right } else { sc.left };
            applied(
                &mut s,
                r,
                role,
                Request::AddPatternEdge {
                    side,
                    a: vs[0],
                    b: vs[0],
                },
            );
            let mut expected = candidate(2);
            if right {
                expected.rhs.push(vec![0, 0]);
            } else {
                expected.lhs.push(vec![0, 0]);
            }
            let after = decode(&s, r);
            assert_eq!(
                if role == Role::Core {
                    after.core
                } else {
                    after.proposal.unwrap()
                },
                expected
            );
            assert_eq!(owned_count(&s, r), count + 1);
            let new_p = scope(&s, r, role)
                .patterns
                .into_iter()
                .find(|p| !sc.patterns.contains(p))
                .unwrap();
            applied(
                &mut s,
                r,
                role,
                Request::RemovePatternEdge { pattern: new_p },
            );
            let after = decode(&s, r);
            assert_eq!(
                if role == Role::Core {
                    after.core
                } else {
                    after.proposal.unwrap()
                },
                candidate(2)
            );
            assert_eq!(owned_count(&s, r), count);
            assert_eq!(scope(&s, r, role).patterns, sc.patterns);
        }
        // A shared occurrence must be removed from BOTH sides, not reconstructed.
        let c = CandidateRule {
            labels: vec![None; 2],
            lhs: vec![vec![0, 1], vec![0, 1]],
            rhs: vec![vec![0, 1], vec![0, 1]],
            ..CandidateRule::default()
        };
        let (mut s, r) = fixture(&c, true);
        let sc = scope(&s, r, role);
        applied(
            &mut s,
            r,
            role,
            Request::RemovePatternEdge {
                pattern: sc.patterns[0],
            },
        );
        let after = decode(&s, r);
        assert_eq!(
            if role == Role::Core {
                after.core
            } else {
                after.proposal.unwrap()
            },
            candidate(2)
        );
    }
}

#[test]
fn label_priority_and_mode_operators_have_explicit_semantics() {
    for role in [Role::Core, Role::Proposal] {
        let (mut s, r) = fixture(&candidate(2), false);
        let sc = scope(&s, r, role);
        let v = ordered_vars(&s, &sc)[0];
        let count = owned_count(&s, r);
        applied(
            &mut s,
            r,
            role,
            Request::AddLabel {
                variable: v,
                value: 1,
            },
        );
        assert_eq!(owned_count(&s, r), count + 1);
        applied(
            &mut s,
            r,
            role,
            Request::ChangeLabel {
                variable: v,
                value: 2,
            },
        );
        let after = decode(&s, r);
        assert_eq!(
            if role == Role::Core {
                after.core.labels[0]
            } else {
                after.proposal.unwrap().labels[0]
            },
            Some(2)
        );
        applied(&mut s, r, role, Request::ChangePriority);
        let after = decode(&s, r);
        assert_eq!(
            if role == Role::Core {
                after.core.priority
            } else {
                after.proposal.unwrap().priority
            },
            1
        );
        applied(&mut s, r, role, Request::RemoveLabel { variable: v });
        assert_eq!(owned_count(&s, r), count);
        applied(&mut s, r, role, Request::Activate);
        assert!(decode(&s, r).active);
        applied(&mut s, r, role, Request::Deactivate);
        assert!(!decode(&s, r).active);
    }
}

fn refused(
    s: &mut HypergraphStore,
    r: NodeId,
    role: Role,
    req: Request,
    status: Status,
) -> coherent::Attempt {
    let before = bytes(s);
    let a = apply(s, r, role, req, &MetaLimits::default());
    assert_eq!(a.status, status, "{}", a.receipt);
    assert_eq!(bytes(s), before); // allocator and indexes too, not only graph hash
    assert_eq!(a.receipt["committed"], false);
    a
}

#[test]
fn inapplicable_and_id_only_edits_are_not_hidden_semantic_successes() {
    let (mut s, r) = fixture(&candidate(2), false);
    let sc = scope(&s, r, Role::Core);
    let v = ordered_vars(&s, &sc)[0];
    for req in [
        Request::RemoveLabel { variable: v },
        Request::ChangeLabel {
            variable: v,
            value: 1,
        },
    ] {
        let a = refused(&mut s, r, Role::Core, req, Status::Inapplicable);
        assert_eq!(bytes(&a.staged), bytes(&s));
        assert_eq!(a.receipt["actions"], serde_json::json!([]));
    }
    applied(
        &mut s,
        r,
        Role::Core,
        Request::AddLabel {
            variable: v,
            value: 1,
        },
    );
    refused(
        &mut s,
        r,
        Role::Core,
        Request::AddLabel {
            variable: v,
            value: 2,
        },
        Status::Inapplicable,
    );
    let before = bytes(&s);
    let program = decode(&s, r);
    let slot = slots(&s, sc.patterns[0])[0];
    applied(
        &mut s,
        r,
        Role::Core,
        Request::RetargetSlot { slot, variable: v },
    );
    assert_ne!(bytes(&s), before);
    assert_eq!(decode(&s, r), program);
    let a = applied(&mut s, r, Role::Core, Request::Deactivate);
    assert_eq!(a.before_program, a.after_program); // explicit same-value mode attempt
}

#[test]
fn semantic_rejections_retain_invalid_stages_without_committing() {
    for req_type in 0..3 {
        let (mut s, r) = fixture(&candidate(2), false);
        let sc = scope(&s, r, Role::Core);
        let req = match req_type {
            0 => Request::RemoveSlot {
                slot: slots(&s, sc.patterns[0])[0],
            },
            1 => Request::RemoveVar {
                variable: ordered_vars(&s, &sc)[0],
            },
            _ => Request::RemovePatternEdge {
                pattern: sc.patterns[0],
            },
        };
        let a = refused(&mut s, r, Role::Core, req, Status::RejectedPostcondition);
        assert_ne!(bytes(&a.staged), bytes(&s));
        assert!(read_complete_program(&a.staged.snapshot(), r, &MetaLimits::default()).is_err());
        assert!(a.receipt["validator"].is_string());
    }
    // Never strip a surviving RHS-only guard to rescue a deletion.
    let mut c = candidate(2);
    c.rhs = vec![vec![0, 0], vec![1, 1]];
    c.guards.push(Guard {
        var: 0,
        key: 42,
        op: PropertyOp::Eq,
        value: PropertyValue::Int(1),
    });
    let (mut s, r) = fixture(&c, false);
    let sc = scope(&s, r, Role::Core);
    let p = sc
        .patterns
        .iter()
        .copied()
        .find(|p| {
            s.edges()
                .any(|(_, e)| e.label.0 == E_MEMBER && e.nodes.as_slice() == [sc.left, *p])
        })
        .unwrap();
    let a = refused(
        &mut s,
        r,
        Role::Core,
        Request::RemovePatternEdge { pattern: p },
        Status::RejectedPostcondition,
    );
    assert_eq!(a.receipt["validator"], "Bounds"); // empty LHS is rejected before Guards
    assert!(a.staged.edges().any(|(_, e)| e.label.0 == E_GUARDS));
}

#[test]
fn guarded_rhs_only_variable_is_rejected_without_guard_stripping() {
    let c = CandidateRule {
        labels: vec![None; 3],
        lhs: vec![vec![0, 1], vec![1, 2]],
        rhs: vec![vec![0, 0], vec![1, 2]],
        guards: vec![Guard {
            var: 0,
            key: 42,
            op: PropertyOp::Eq,
            value: PropertyValue::Int(1),
        }],
        ..CandidateRule::default()
    };
    let (mut s, r) = fixture(&c, false);
    let sc = scope(&s, r, Role::Core);
    let vs = ordered_vars(&s, &sc);
    let p = sc
        .patterns
        .iter()
        .copied()
        .find(|p| {
            slots(&s, *p)
                .iter()
                .any(|id| s.get_edge(*id).unwrap().nodes[1] == vs[0])
                && s.edges()
                    .any(|(_, e)| e.label.0 == E_MEMBER && e.nodes.as_slice() == [sc.left, *p])
        })
        .unwrap();
    let a = refused(
        &mut s,
        r,
        Role::Core,
        Request::RemovePatternEdge { pattern: p },
        Status::RejectedPostcondition,
    );
    assert_eq!(a.receipt["validator"], "Guards");
    assert!(a
        .staged
        .edges()
        .any(|(_, e)| e.label.0 == E_GUARDS && e.nodes[0] == vs[0]));
}

#[test]
fn caps_foreign_and_stale_handles_are_explicit_atomic_refusals() {
    let (mut s, r) = fixture(&candidate(8), false);
    let sc = scope(&s, r, Role::Core);
    let vs = ordered_vars(&s, &sc);
    for req in [
        Request::AddVar {
            pattern: sc.patterns[0],
        },
        Request::AddSlot {
            pattern: sc.patterns[0],
            variable: vs[0],
        },
    ] {
        refused(&mut s, r, Role::Core, req, Status::RejectedPolicy);
    }
    let (mut s, r) = fixture(&candidate(2), false);
    let sc = scope(&s, r, Role::Core);
    let other = scope(&s, r, Role::Proposal);
    let vs = ordered_vars(&s, &sc);
    let foreign = ordered_vars(&s, &other)[0];
    for req in [
        Request::RemoveVar { variable: foreign },
        Request::AddSlot {
            pattern: sc.patterns[0],
            variable: foreign,
        },
        Request::RemovePatternEdge {
            pattern: other.patterns[0],
        },
        Request::AddPatternEdge {
            side: other.left,
            a: vs[0],
            b: vs[1],
        },
        Request::RemoveSlot {
            slot: slots(&s, other.patterns[0])[0],
        },
    ] {
        refused(&mut s, r, Role::Core, req, Status::RejectedPolicy);
    }
    let stale = s.add_edge(HyperEdge::new(
        Label(0),
        [vs[0], vs[1]].into_iter().collect(),
    ));
    s.remove_edge(stale);
    refused(
        &mut s,
        r,
        Role::Core,
        Request::RemoveSlot { slot: stale },
        Status::RejectedPolicy,
    );
    refused(
        &mut s,
        r,
        Role::Core,
        Request::AddLabel {
            variable: vs[0],
            value: 3,
        },
        Status::RejectedPolicy,
    );
    let mut c = candidate(2);
    c.priority = i32::MAX;
    let (mut s, r) = fixture(&c, false);
    refused(
        &mut s,
        r,
        Role::Core,
        Request::ChangePriority,
        Status::RejectedPolicy,
    );
}

#[test]
fn region_and_store_boundaries_never_commit_or_retry() {
    let (mut s, r) = fixture(&candidate(2), false);
    let sc = scope(&s, r, Role::Core);
    let region = walk_family(&s.snapshot(), r, &MetaLimits::default()).unwrap();
    let limits = MetaLimits {
        max_region_nodes: region.nodes.len(),
        ..MetaLimits::default()
    };
    let before = bytes(&s);
    let a = apply(
        &mut s,
        r,
        Role::Core,
        Request::AddVar {
            pattern: sc.patterns[0],
        },
        &limits,
    );
    assert_eq!(a.status, Status::RejectedPostcondition);
    assert_eq!(bytes(&s), before);
    assert_eq!(a.receipt["validator"], "Region");
    while s.node_count() < 2001 {
        s.add_node(NodeData::new(Label(0), 0));
    }
    refused(
        &mut s,
        r,
        Role::Core,
        Request::ChangePriority,
        Status::RejectedPolicy,
    );
    let (mut s, r) = fixture(&candidate(2), false);
    let world = [
        s.add_node(NodeData::new(Label(0), 0)),
        s.add_node(NodeData::new(Label(0), 0)),
    ];
    while s.edge_count() < 4001 {
        s.add_edge(HyperEdge::new(Label(0), world.into_iter().collect()));
    }
    refused(
        &mut s,
        r,
        Role::Core,
        Request::ChangePriority,
        Status::RejectedPolicy,
    );
}

#[test]
fn world_and_other_family_are_untouched_and_replay_is_deterministic() {
    let (mut original, r) = fixture(&candidate(3), false);
    original.set_step(17);
    let world = [
        original.add_node(NodeData::new(Label(2), 9)),
        original.add_node(NodeData::new(Label(1), 10)),
    ];
    let e = original.add_edge(HyperEdge::new(Label(0), world.into_iter().collect()));
    let other = encode_complete_family(
        &mut original,
        &candidate(2),
        &candidate(2),
        true,
        &MetaLimits::default(),
    )
    .unwrap()
    .root;
    let other_program = decode(&original, other);
    let before_world = serde_json::json!([
        original.get_node(world[0]),
        original.get_node(world[1]),
        original.get_edge(e)
    ]);
    let sc = scope(&original, r, Role::Core);
    let req = Request::AddVar {
        pattern: sc.patterns[0],
    };
    let mut a = original.clone();
    let mut b = original.clone();
    let ra = applied(&mut a, r, Role::Core, req.clone());
    let rb = applied(&mut b, r, Role::Core, req);
    assert_eq!(ra.receipt, rb.receipt);
    assert_eq!(bytes(&a), bytes(&b));
    assert_eq!(a.step(), 17);
    assert_eq!(decode(&a, other), other_program);
    assert_eq!(
        serde_json::json!([a.get_node(world[0]), a.get_node(world[1]), a.get_edge(e)]),
        before_world
    );
    let new_v = scope(&a, r, Role::Core)
        .variables
        .into_iter()
        .find(|v| !sc.variables.contains(v))
        .unwrap();
    assert_eq!(a.get_node(new_v).unwrap().generation, 17);
    // Fresh encoding is an oracle only; the edited root/pattern retain original IDs.
    let p = decode(&a, r);
    let (oracle, rr) = fixture(&p.core, false);
    assert_eq!(decode(&oracle, rr).core, p.core);
    assert_eq!(scope(&a, r, Role::Core).patterns, sc.patterns);
}

#[test]
fn foreign_body_attachment_and_paid_seal_are_not_bypassed() {
    let (mut s, r) = fixture(&candidate(2), false);
    let sc = scope(&s, r, Role::Core);
    let w = s.add_node(NodeData::new(Label(0), 0));
    s.add_edge(HyperEdge::new(
        Label(0),
        [w, sc.patterns[0]].into_iter().collect(),
    ));
    decode(&s, r); // ordinary attachment is outside reader's meta region
    let a = refused(
        &mut s,
        r,
        Role::Core,
        Request::RemovePatternEdge {
            pattern: sc.patterns[0],
        },
        Status::RejectedPolicy,
    );
    assert_eq!(a.receipt["reason"], "foreign attachment to selected body");
    let (mut s, r) = fixture(&candidate(2), false);
    let sc = scope(&s, r, Role::Core);
    let payload = s
        .edges()
        .find(|(_, e)| e.label.0 == E_CORE && e.nodes[0] == r)
        .unwrap()
        .0;
    let instruction = complete::instruction(&mut s, MetaOperation::Seal, 1.0);
    complete::seal(&mut s, payload, instruction, 1);
    decode(&s, r);
    let before_fuel = complete::fuel(&s, instruction);
    let a = refused(
        &mut s,
        r,
        Role::Core,
        Request::AddVar {
            pattern: sc.patterns[0],
        },
        Status::RejectedPolicy,
    );
    assert_eq!(
        a.receipt["reason"],
        "sealed payload is outside this host kernel"
    );
    assert_eq!(complete::fuel(&s, instruction), before_fuel);
}

#[test]
fn all_168_existing_parents_are_eligible_without_measuring_mutation_rates() {
    let bank = hg_sim_core::ere::semantic::parent_bank();
    assert_eq!(bank.len(), 168);
    for (_, p) in bank {
        let mut s = HypergraphStore::new();
        let r = encode_complete_family(
            &mut s,
            &p.core,
            p.proposal.as_ref().unwrap(),
            p.active,
            &MetaLimits::default(),
        )
        .unwrap()
        .root;
        let before = bytes(&s);
        for role in [Role::Core, Role::Proposal] {
            let sc = scope(&s, r, role);
            assert!(!sc.patterns.is_empty());
            assert!(!sc.variables.is_empty());
        }
        assert_eq!(bytes(&s), before); // discovery only: no draws, rollout or gate result
    }
}

#[test]
fn complete_damage_on_edited_families_stays_rejected_and_is_never_healed() {
    let mut c = candidate(8);
    c.labels = vec![Some(1); 8];
    c.guards.push(Guard {
        var: 0,
        key: 42,
        op: PropertyOp::Eq,
        value: PropertyValue::Int(1),
    });
    let (mut edited, r) = fixture(&c, true);
    let initial_scope = scope(&edited, r, Role::Core);
    let vs = ordered_vars(&edited, &initial_scope);
    applied(
        &mut edited,
        r,
        Role::Core,
        Request::AddPatternEdge {
            side: initial_scope.left,
            a: vs[0],
            b: vs[1],
        },
    );
    let check = |mut damaged: HypergraphStore| {
        assert!(read_complete_program(&damaged.snapshot(), r, &MetaLimits::default()).is_err());
        let before = bytes(&damaged);
        let a = refused(
            &mut damaged,
            r,
            Role::Core,
            Request::ChangePriority,
            Status::RejectedInput,
        );
        assert_eq!(bytes(&a.staged), before);
        assert_eq!(a.receipt["actions"], serde_json::json!([]));
        assert!(a.receipt["validator"].is_string());
    };
    // All 13 relation roles: loss/duplication of payload or its witness.
    for (ix, role) in WITNESS_ROLES.iter().enumerate() {
        let p = edited.edges().find(|(_, e)| e.label.0 == *role).unwrap().0;
        let e = edited.get_edge(p).unwrap();
        let w = edited
            .edges()
            .find(|(_, w)| {
                w.label.0 == WITNESS_BASE + ix as u32
                    && w.nodes.as_slice() == [r, e.nodes[0], e.nodes[1]]
            })
            .unwrap()
            .0;
        for id in [p, w] {
            let mut d = edited.clone();
            d.remove_edge(id);
            check(d);
            let mut d = edited.clone();
            d.add_edge(d.get_edge(id).unwrap().clone());
            check(d);
        }
    }
    let sc = scope(&edited, r, Role::Core);
    let variable = ordered_vars(&edited, &sc)[0];
    // All 26 outgoing/incoming declarations, including declared zeroes.
    let keys = (0..26)
        .map(|i| K_REQUIRED_BASE + i)
        .chain([K_SCHEMA_LABEL])
        .collect::<Vec<_>>();
    for key in keys {
        for kind in 0..4 {
            let mut d = edited.clone();
            let n = d.get_node_mut(variable).unwrap();
            let old = n.get_property(key).unwrap();
            match kind {
                0 => n.properties.retain(|(k, _)| *k != key),
                1 => n.properties.push((key, old)),
                2 => n.set_property(key, PropertyValue::Bool(false)),
                _ => {
                    let PropertyValue::Int(v) = old else {
                        panic!("type")
                    };
                    n.set_property(key, PropertyValue::Int(v + 1));
                }
            }
            check(d);
        }
    }
    for kind in 0..4 {
        let mut d = edited.clone();
        let n = d.get_node_mut(r).unwrap();
        let old = n.get_property(K_OWNED_COUNT).unwrap();
        match kind {
            0 => n.properties.retain(|(k, _)| *k != K_OWNED_COUNT),
            1 => n.properties.push((K_OWNED_COUNT, old)),
            2 => n.set_property(K_OWNED_COUNT, PropertyValue::Bool(false)),
            _ => n.set_property(K_OWNED_COUNT, PropertyValue::Int(0)),
        }
        check(d);
    }
    let own = edited
        .edges()
        .find(|(_, e)| e.label.0 == E_OWNS && e.nodes[1] == variable)
        .unwrap()
        .0;
    let mut d = edited.clone();
    d.remove_edge(own);
    check(d);
    let mut d = edited.clone();
    d.add_edge(d.get_edge(own).unwrap().clone());
    check(d);
    let mut d = edited.clone();
    d.remove_node(variable);
    check(d);
    let mut d = edited.clone();
    d.remove_node(r);
    check(d);
}

#[test]
fn metadata_obligations_are_exact_local_deltas_not_bulk_refresh() {
    let (mut s, r) = fixture(&candidate(2), false);
    let sc = scope(&s, r, Role::Core);
    let v = ordered_vars(&s, &sc)[0];
    let p = sc.patterns[0];
    let ix = WITNESS_ROLES
        .iter()
        .position(|x| *x == E_SLOT_BASE + 2)
        .unwrap() as u32;
    let outgoing = K_REQUIRED_BASE + ix * 2;
    let incoming = outgoing + 1;
    let declarations = |s: &HypergraphStore| {
        s.nodes()
            .map(|(id, n)| {
                (
                    id,
                    (0..26)
                        .map(|i| n.get_property(K_REQUIRED_BASE + i).unwrap())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    let before = declarations(&s);
    applied(
        &mut s,
        r,
        Role::Core,
        Request::AddSlot {
            pattern: p,
            variable: v,
        },
    );
    for (id, ds) in before {
        for (i, old) in ds.into_iter().enumerate() {
            let key = K_REQUIRED_BASE + i as u32;
            let expected = if (id == p && key == outgoing) || (id == v && key == incoming) {
                let PropertyValue::Int(value) = old else {
                    panic!("type")
                };
                PropertyValue::Int(value + 1)
            } else {
                old
            };
            assert_eq!(s.get_node(id).unwrap().get_property(key), Some(expected));
        }
    }
    let a = applied(&mut s, r, Role::Core, Request::AddVar { pattern: p });
    let property_writes = a.receipt["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["op"] == "set_property")
        .count();
    assert_eq!(property_writes, 3); // root ownership, slot outgoing, slot incoming
    let added = scope(&s, r, Role::Core)
        .variables
        .into_iter()
        .find(|n| !sc.variables.contains(n))
        .unwrap();
    let n = s.get_node(added).unwrap();
    assert_eq!(
        n.get_property(K_SCHEMA_LABEL),
        Some(PropertyValue::Int(L_VAR as i64))
    );
    assert_eq!(
        n.properties
            .iter()
            .filter(|(k, _)| (K_REQUIRED_BASE..K_REQUIRED_BASE + 26).contains(k))
            .count(),
        26
    );
}

#[test]
fn literal_deletion_waits_for_its_last_consumer() {
    let c = CandidateRule {
        labels: vec![Some(1); 2],
        ..candidate(2)
    };
    let (mut s, r) = fixture(&c, false);
    let sc = scope(&s, r, Role::Core);
    let vs = ordered_vars(&s, &sc);
    let labels = vs
        .iter()
        .map(|v| {
            s.edges()
                .find(|(_, e)| e.label.0 == E_HASLABEL && e.nodes[0] == *v)
                .unwrap()
                .0
        })
        .collect::<Vec<_>>();
    let first = s.get_edge(labels[0]).unwrap().nodes[1];
    let second = s.get_edge(labels[1]).unwrap().nodes[1];
    // Build an explicitly valid shared-literal input independently of the kernel.
    let ix = WITNESS_ROLES.iter().position(|r| *r == E_HASLABEL).unwrap() as u32;
    let witness = s
        .edges()
        .find(|(_, e)| e.label.0 == WITNESS_BASE + ix && e.nodes.as_slice() == [r, vs[1], second])
        .unwrap()
        .0;
    s.remove_edge(labels[1]);
    s.remove_edge(witness);
    s.add_edge(HyperEdge::new(
        Label(E_HASLABEL),
        [vs[1], first].into_iter().collect(),
    ));
    s.add_edge(HyperEdge::new(
        Label(WITNESS_BASE + ix),
        [r, vs[1], first].into_iter().collect(),
    ));
    s.get_node_mut(first)
        .unwrap()
        .set_property(K_REQUIRED_BASE + 2 * ix + 1, PropertyValue::Int(2));
    s.remove_node(second);
    let count = owned_count(&s, r);
    s.get_node_mut(r)
        .unwrap()
        .set_property(K_OWNED_COUNT, PropertyValue::Int(count - 1));
    decode(&s, r);
    let count = owned_count(&s, r);
    applied(
        &mut s,
        r,
        Role::Core,
        Request::ChangeLabel {
            variable: vs[0],
            value: 2,
        },
    );
    assert_eq!(decode(&s, r).core.labels, vec![Some(2), Some(2)]); // shared literal semantics
    applied(
        &mut s,
        r,
        Role::Core,
        Request::RemoveLabel { variable: vs[0] },
    );
    assert!(s.get_node(first).is_some());
    assert_eq!(owned_count(&s, r), count);
    assert_eq!(decode(&s, r).core.labels, vec![None, Some(2)]);
    applied(
        &mut s,
        r,
        Role::Core,
        Request::RemoveLabel { variable: vs[1] },
    );
    assert!(s.get_node(first).is_none());
    assert_eq!(owned_count(&s, r), count - 1);
}

#[test]
fn decodable_but_unsupported_order_presentations_are_not_reencoded() {
    for variable_hints in [false, true] {
        let (mut s, r) = fixture(&candidate(2), false);
        let sc = scope(&s, r, Role::Core);
        let ids = if variable_hints {
            sc.variables
        } else {
            sc.patterns
        };
        for n in ids {
            s.get_node_mut(n)
                .unwrap()
                .properties
                .retain(|(k, _)| *k != K_ORDER);
        }
        decode(&s, r); // optional codec hints are a stricter host-kernel policy
        let a = refused(
            &mut s,
            r,
            Role::Core,
            Request::ChangePriority,
            Status::RejectedPolicy,
        );
        assert!(a.receipt["actions"].as_array().unwrap().is_empty());
        assert_eq!(bytes(&a.staged), bytes(&s));
    }
}

fn source_engine(s: &HypergraphStore) -> hg_engine::Engine {
    use hg_rules::{EpochPolicy, MatchBudgetPolicy, StepMode};
    let mut e = hg_engine::Engine::new(vec![], StepMode::MaxParallel, 42, 1);
    e.set_match_budget_policy(400000, 50000, MatchBudgetPolicy::Pooled);
    let mut src = GraphRuleSource::new(
        EpochPolicy::Fixed(1.try_into().unwrap()),
        MetaLimits::default(),
        false,
    );
    src.complete_witness_enabled = true;
    e.set_rule_source(Box::new(src));
    e.compile_at_boundary(s);
    e
}

#[test]
fn short_complete_source_execution_matches_direct_oracle_and_dormancy() {
    use hg_rules::{MatchBudgetPolicy, StepMode};
    use hg_sim_core::ere::{experiments, meta_excluded_store};
    let mut c = candidate(2);
    c.rhs.push(vec![0, 0]);
    let mut initial = experiments::fixture(&c);
    let root = encode_complete_family(&mut initial, &c, &c, false, &MetaLimits::default())
        .unwrap()
        .root;
    let initial_program = decode(&initial, root);
    let mut traces = vec![];
    for case in 0..4 {
        let mut graph = initial.clone();
        if case > 0 {
            let role = if case == 1 {
                Role::Core
            } else {
                Role::Proposal
            };
            let sc = scope(&graph, root, role);
            let v = ordered_vars(&graph, &sc)[0];
            let p = sc
                .patterns
                .iter()
                .copied()
                .find(|p| {
                    graph
                        .edges()
                        .any(|(_, e)| e.label.0 == E_MEMBER && e.nodes.as_slice() == [sc.left, *p])
                })
                .unwrap();
            let probe = source_engine(&graph);
            let before_rng = probe.rng_state_probe();
            applied(
                &mut graph,
                root,
                role,
                Request::AddSlot {
                    pattern: p,
                    variable: v,
                },
            );
            assert_eq!(probe.rng_state_probe(), before_rng); // kernel has no RNG handle
            if case == 3 {
                applied(&mut graph, root, Role::Proposal, Request::Activate);
            }
        }
        let p = decode(&graph, root);
        if case == 2 {
            assert_eq!(p.execution_bytes(), initial_program.execution_bytes());
            assert_ne!(p.proposal, initial_program.proposal);
        }
        if case == 3 {
            assert_ne!(p.rcf_bytes(), initial_program.rcf_bytes());
        }
        // Keep identical inert meta allocation in the static leg. Deleting it
        // would change the allocator free list and invalidate an exact typed
        // edge-handle comparison despite equal executable world semantics.
        let mut direct = graph.clone();
        let mut static_engine = hg_engine::Engine::new(
            p.compile(&MetaLimits::default(), false).unwrap(),
            StepMode::MaxParallel,
            42,
            1,
        );
        static_engine.set_match_budget_policy(400000, 50000, MatchBudgetPolicy::Pooled);
        let mut e = source_engine(&graph);
        assert_eq!(e.compile_report().unwrap().quarantined, 0);
        assert_eq!(e.compile_report().unwrap().validated_regions, 1);
        let mut trace = vec![];
        for _ in 0..4 {
            e.step(&mut graph);
            static_engine.step(&mut direct);
            assert!(!e.match_ceiling_binding());
            assert!(!static_engine.match_ceiling_binding());
            let projected = meta_excluded_store(&graph);
            assert_eq!(
                projected.canonical_hash(),
                meta_excluded_store(&direct).canonical_hash()
            );
            assert_eq!(
                witness::inventory(&projected),
                witness::inventory(&meta_excluded_store(&direct))
            );
            assert_eq!(e.rng_state_probe(), static_engine.rng_state_probe());
            trace.push((
                projected.canonical_hash(),
                projected.edge_count(),
                e.rng_state_probe(),
            ));
            assert_eq!(e.compile_report().unwrap().quarantined, 0);
        }
        traces.push(trace);
    }
    assert_ne!(traces[0], traces[1]); // actual coherent LHS edit stops this binary motif
    assert_eq!(traces[0], traces[2]); // dormant genotype edits do not invent novelty
    assert!(traces[0][3].1 > meta_excluded_store(&initial).edge_count());
}
