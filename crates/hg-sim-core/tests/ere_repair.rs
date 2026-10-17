#![cfg(feature = "hg-meta")]
use hg_core::{HypergraphStore, Label, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_rules::StepMode;
use rand::SeedableRng;

fn material() -> (HypergraphStore, hg_core::NodeId) {
    let mut s = HypergraphStore::new();
    let core = hg_sim_core::ere::experiments::seed_bank()[0].clone();
    let proposal = CandidateRule {
        labels: vec![None; 3],
        lhs: vec![vec![0, 1], vec![1, 2]],
        rhs: vec![vec![0, 1], vec![0, 2]],
        priority: 1,
        ..CandidateRule::default()
    };
    let root = encode_witness_family(&mut s, &core, &proposal, false, &MetaLimits::default())
        .unwrap()
        .root;
    (s, root)
}
fn engine(op: MetaOperation) -> Engine {
    Engine::new(
        compile_operations(op, &MetaLimits::default()).unwrap(),
        StepMode::MaxParallel,
        42,
        16,
    )
}
#[test]
fn seal_and_repair_are_paid_real_native_rewrites_with_exact_semantic_roundtrip() {
    for op in [MetaOperation::Seal, MetaOperation::Repair] {
        let (base, root) = material();
        let expected = read_program(&base.snapshot(), root, &MetaLimits::default()).unwrap();
        let ids: Vec<_> = base
            .edges()
            .filter(|(_, e)| e.nodes.len() == 2)
            .map(|(id, _)| id)
            .collect();
        assert!(!ids.is_empty());
        for id in ids {
            let mut s = base.clone();
            if op == MetaOperation::Repair {
                s.get_edge_mut(id).unwrap().label = Label(0);
                assert!(read_program(&s.snapshot(), root, &MetaLimits::default()).is_err());
            }
            let instruction = encode_instruction(
                &mut s,
                &MetaInstructionSeed {
                    operation: op,
                    fuel: 1.0,
                },
            )
            .unwrap();
            let mut e = engine(op);
            let before = e.rng_state_probe();
            assert_eq!(e.step(&mut s).accepted_rewrites, 1);
            assert_eq!(before, e.rng_state_probe());
            assert_eq!(
                read_instruction(&s.snapshot(), instruction).unwrap().fuel,
                0.0
            );
            assert_eq!(
                read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap(),
                expected
            );
            assert_eq!(e.step(&mut s).accepted_rewrites, 0);
        }
    }
}
#[test]
fn sealed_arity_prevents_binary_world_damage_without_meta_shield() {
    let (mut s, root) = material();
    let expected = read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap();
    let pairs = s.edges().filter(|(_, e)| e.nodes.len() == 2).count();
    let instruction = encode_instruction(
        &mut s,
        &MetaInstructionSeed {
            operation: MetaOperation::Seal,
            fuel: 32.0,
        },
    )
    .unwrap();
    let mut e = engine(MetaOperation::Seal);
    for _ in 0..pairs {
        assert_eq!(e.step(&mut s).accepted_rewrites, 1);
    }
    assert_eq!(s.edges().filter(|(_, e)| e.nodes.len() == 2).count(), 0);
    assert_eq!(
        read_instruction(&s.snapshot(), instruction).unwrap().fuel,
        32.0 - pairs as f64
    );
    let world = expected.core.compile(&MetaLimits::default(), true).unwrap();
    assert!(world.meta_visible && world.pattern.node_label_ceiling.is_none());
    let mut exposed = Engine::new(vec![world], StepMode::MaxParallel, 42, 16);
    for _ in 0..12 {
        assert_eq!(exposed.step(&mut s).accepted_rewrites, 0);
    }
    assert_eq!(
        read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap(),
        expected
    );
}
#[test]
fn decoder_never_completes_missing_payload_or_damaged_witness() {
    let (base, root) = material();
    let id = base.edges().find(|(_, e)| e.nodes.len() == 2).unwrap().0;
    let mut missing = base.clone();
    missing.remove_edge(id);
    assert!(read_program(&missing.snapshot(), root, &MetaLimits::default()).is_err());
    encode_instruction(
        &mut missing,
        &MetaInstructionSeed {
            operation: MetaOperation::Repair,
            fuel: 32.0,
        },
    )
    .unwrap();
    assert_eq!(
        engine(MetaOperation::Repair)
            .step(&mut missing)
            .accepted_rewrites,
        0
    );
    let mut broken = base.clone();
    let witness = broken
        .edges()
        .find(|(_, e)| e.label.0 == WITNESS_BASE)
        .unwrap()
        .0;
    broken.get_edge_mut(witness).unwrap().label = Label(0);
    assert!(read_program(&broken.snapshot(), root, &MetaLimits::default()).is_err());
    let counts = (broken.node_count(), broken.edge_count());
    let _ = read_program(&broken.snapshot(), root, &MetaLimits::default());
    assert_eq!(counts, (broken.node_count(), broken.edge_count()));
}
#[test]
fn instruction_version_opcode_fuel_isolation_and_protocol_guards_are_enforced() {
    for (key, value) in [
        (K_ENCODING, PropertyValue::Int(4)),
        (K_OPCODE, PropertyValue::Int(99)),
        (K_FUEL, PropertyValue::Int(1)),
        (K_FUEL, PropertyValue::Float(0.0)),
        (K_FUEL, PropertyValue::Float(f64::NAN)),
        (K_FUEL, PropertyValue::Float(33.0)),
    ] {
        let (mut s, _) = material();
        let op = encode_instruction(
            &mut s,
            &MetaInstructionSeed {
                operation: MetaOperation::Seal,
                fuel: 1.0,
            },
        )
        .unwrap();
        s.get_node_mut(op).unwrap().set_property(key, value);
        assert_eq!(
            engine(MetaOperation::Seal).step(&mut s).accepted_rewrites,
            0
        );
    }
    assert!(compile_operation(MetaOperation::Repair, &MetaLimits::default()).is_err());
    assert!(compile_operations(
        MetaOperation::Repair,
        &MetaLimits {
            max_arity: 2,
            ..MetaLimits::default()
        }
    )
    .is_err());
}

#[test]
fn every_seal_binding_preserves_the_decoded_family() {
    let (mut base, root) = material();
    let expected = read_program(&base.snapshot(), root, &MetaLimits::default()).unwrap();
    encode_instruction(
        &mut base,
        &MetaInstructionSeed {
            operation: MetaOperation::Seal,
            fuel: 1.0,
        },
    )
    .unwrap();
    for rule in compile_operations(MetaOperation::Seal, &MetaLimits::default()).unwrap() {
        for binding in hg_engine::find_bindings(&base.snapshot(), &rule, 0) {
            let mut s = base.clone();
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
            hg_engine::apply_binding(&mut s, &rule, &binding, &mut rng);
            assert_eq!(
                read_program(&s.snapshot(), root, &MetaLimits::default()),
                Ok(expected.clone())
            );
        }
    }
}

#[test]
fn all_thirteen_schema_roles_and_all_repair_bindings_are_safe() {
    let limits = MetaLimits::default();
    let c = CandidateRule {
        labels: vec![Some(1); 8],
        lhs: vec![(0..8).collect()],
        rhs: vec![(0..8).collect()],
        guards: vec![Guard {
            var: 0,
            key: 42,
            op: hg_core::PropertyOp::Eq,
            value: PropertyValue::Int(1),
        }],
        ..CandidateRule::default()
    };
    let mut base = HypergraphStore::new();
    let root = encode_witness_family(&mut base, &c, &c, false, &limits)
        .unwrap()
        .root;
    let expected = read_program(&base.snapshot(), root, &limits).unwrap();
    let pairs: Vec<_> = base
        .edges()
        .filter(|(_, e)| e.nodes.len() == 2)
        .map(|(id, e)| (id, e.label.0))
        .collect();
    let roles: std::collections::HashSet<_> = pairs.iter().map(|(_, role)| *role).collect();
    assert!(WITNESS_ROLES.iter().all(|role| roles.contains(role)));
    let mut checked = 0;
    for (id, _) in pairs {
        let mut damaged = base.clone();
        damaged.get_edge_mut(id).unwrap().label = Label(0);
        let instruction = encode_instruction(
            &mut damaged,
            &MetaInstructionSeed {
                operation: MetaOperation::Repair,
                fuel: 1.0,
            },
        )
        .unwrap();
        assert!(read_program(&damaged.snapshot(), root, &limits).is_err());
        for rule in compile_operations(MetaOperation::Repair, &limits).unwrap() {
            for binding in hg_engine::find_bindings(&damaged.snapshot(), &rule, 0) {
                let mut s = damaged.clone();
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
                let before_rng = rng.clone();
                hg_engine::apply_binding(&mut s, &rule, &binding, &mut rng);
                assert_eq!(rng, before_rng);
                assert_eq!(
                    read_instruction(&s.snapshot(), instruction).unwrap().fuel,
                    0.0
                );
                assert_eq!(
                    read_program(&s.snapshot(), root, &limits),
                    Ok(expected.clone())
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 13);
}

/// Supplement to the frozen order-reversal matrix: actually change the endpoint
/// multiset, or delete a schema node (with its incident-edge cascade).
#[test]
fn foreign_endpoints_and_node_deletion_cannot_be_completed_from_cached_body() {
    let mut checked = 0;
    for target_kind in ["binary", "witness", "sealed"] {
        for damage in ["foreign_typed", "foreign_erased", "node_delete"] {
            for repair_fuel in [0.0, 32.0] {
                let (mut s, root) = material();
                let (payload_id, payload) = s
                    .edges()
                    .find(|(_, e)| e.label == Label(E_SLOT_BASE))
                    .unwrap();
                let (a, b) = (payload.nodes[0], payload.nodes[1]);
                let witness_label = WITNESS_BASE
                    + WITNESS_ROLES
                        .iter()
                        .position(|role| *role == E_SLOT_BASE)
                        .unwrap() as u32;
                if target_kind == "sealed" {
                    let instruction = encode_instruction(
                        &mut s,
                        &MetaInstructionSeed {
                            operation: MetaOperation::Seal,
                            fuel: 1.0,
                        },
                    )
                    .unwrap();
                    let mut sealed = false;
                    for rule in
                        compile_operations(MetaOperation::Seal, &MetaLimits::default()).unwrap()
                    {
                        if let Some(binding) = hg_engine::find_bindings(&s.snapshot(), &rule, 0)
                            .into_iter()
                            .find(|binding| binding.matched_edges.contains(&payload_id))
                        {
                            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
                            hg_engine::apply_binding(&mut s, &rule, &binding, &mut rng);
                            sealed = true;
                            break;
                        }
                    }
                    assert!(sealed);
                    assert_eq!(
                        read_instruction(&s.snapshot(), instruction).unwrap().fuel,
                        0.0
                    );
                }
                assert!(read_program(&s.snapshot(), root, &MetaLimits::default()).is_ok());
                // Artificial markers are fixture setup only, not a runtime repair hook.
                for (node, marker) in [(root, 0), (a, 1), (b, 2)] {
                    s.get_node_mut(node)
                        .unwrap()
                        .set_property(0xE5520002, PropertyValue::Int(marker));
                }
                let (nodes, endpoints, edge_label) = match target_kind {
                    "binary" => (vec!["a", "b"], vec!["a", "b"], E_SLOT_BASE),
                    "witness" => (
                        vec!["root", "a", "b"],
                        vec!["root", "a", "b"],
                        witness_label,
                    ),
                    _ => (vec!["a", "b", "root"], vec!["a", "b", "root"], E_SLOT_BASE),
                };
                let mut attack = hg_rules::Rule {
                    name: format!("{target_kind}_{damage}"),
                    ..Default::default()
                };
                attack.lhs.nodes = nodes.iter().map(|var| var.to_string()).collect();
                attack
                    .lhs
                    .edges
                    .push(endpoints.iter().map(|var| var.to_string()).collect());
                attack.lhs.edge_labels.insert(0, Label(edge_label));
                for var in &nodes {
                    attack
                        .lhs_node_property_guards
                        .push(hg_rules::NodePropertyGuard {
                            var: var.to_string(),
                            property: 0xE5520002,
                            op: hg_core::PropertyOp::Eq,
                            value: PropertyValue::Int(match *var {
                                "root" => 0,
                                "a" => 1,
                                _ => 2,
                            }),
                        });
                }
                attack.rhs.nodes = attack.lhs.nodes.clone();
                if damage == "node_delete" {
                    attack.rhs.nodes.retain(|var| var != "b");
                } else {
                    attack.rhs.nodes.push("foreign".into());
                    attack
                        .rhs_node_labels
                        .insert("foreign".into(), Label(L_VAR));
                    attack.rhs.edges.push(
                        endpoints
                            .iter()
                            .map(|var| if *var == "b" { "foreign" } else { *var }.to_string())
                            .collect(),
                    );
                    attack.rhs.edge_labels.insert(
                        0,
                        Label(if damage == "foreign_erased" {
                            0
                        } else {
                            edge_label
                        }),
                    );
                }
                let mut compiled = hg_rules::compile_rule(&attack).unwrap();
                compiled.meta_visible = true;
                let mut attacker = Engine::new(vec![compiled], StepMode::MaxParallel, 42, 16);
                attacker.set_match_budget_policy(50000, 50000, hg_rules::MatchBudgetPolicy::Pooled);
                let before_nodes = s.node_count();
                let before_rng = attacker.rng_state_probe();
                let stats = attacker.step(&mut s);
                assert!(!attacker.match_ceiling_binding());
                assert_eq!(stats.accepted_rewrites, 1);
                assert_eq!(attacker.rng_state_probe(), before_rng);
                if damage == "node_delete" {
                    assert!(s.get_node(b).is_none());
                    assert_eq!(s.node_count(), before_nodes - 1);
                } else {
                    assert!(s.get_node(b).is_some());
                    assert_eq!(s.node_count(), before_nodes + 1);
                    let replacement = s
                        .edges()
                        .find(|(_, e)| {
                            e.label
                                == Label(if damage == "foreign_erased" {
                                    0
                                } else {
                                    edge_label
                                })
                                && e.nodes.contains(&a)
                                && !e.nodes.contains(&b)
                        })
                        .unwrap()
                        .1;
                    assert!(replacement
                        .nodes
                        .iter()
                        .any(|&n| n != root && n != a && n != b));
                }
                assert!(read_program(&s.snapshot(), root, &MetaLimits::default()).is_err());
                let instruction = encode_instruction(
                    &mut s,
                    &MetaInstructionSeed {
                        operation: MetaOperation::Repair,
                        fuel: repair_fuel,
                    },
                )
                .unwrap();
                let mut repair = engine(MetaOperation::Repair);
                repair.set_match_budget_policy(50000, 50000, hg_rules::MatchBudgetPolicy::Pooled);
                let before_rng = repair.rng_state_probe();
                for _ in 0..8 {
                    assert_eq!(
                        repair.step(&mut s).accepted_rewrites,
                        0,
                        "{target_kind}/{damage}"
                    );
                    assert!(!repair.match_ceiling_binding());
                    assert_eq!(
                        read_instruction(&s.snapshot(), instruction).unwrap().fuel,
                        repair_fuel
                    );
                    assert!(read_program(&s.snapshot(), root, &MetaLimits::default()).is_err());
                }
                assert_eq!(repair.rng_state_probe(), before_rng);
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 18);
    println!("foreign endpoint / schema-node deletion: {checked} native fixture controls");
}
