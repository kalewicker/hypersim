#![cfg(feature = "hg-meta")]
//! Bounded engineering controls, not an A3 campaign or training admission.
use hg_core::{EdgeId, HypergraphStore, Label, NodeId, PropertyOp, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_output::{SnapshotReader, SnapshotWriter};
use hg_rules::{compile_rule, EpochPolicy, MatchBudgetPolicy, NodePropertyGuard, Rule, StepMode};
use rand::SeedableRng;
use serde_json::{json, Value};
use std::{num::NonZeroU64, path::Path};

const SEEDS: [u64; 4] = [984927043585, 984927043586, 984927043587, 984927043588];
const MARKER: u32 = 0xE552_0001;
const HORIZON: usize = 8;
const CONDITIONS: [&str; 12] = [
    "no_attack",
    "binary_erase",
    "binary_delete",
    "binary_rewire",
    "witness_erase",
    "witness_delete",
    "witness_rewire",
    "sealed_erase",
    "sealed_delete",
    "sealed_rewire",
    "duplicate_payload",
    "duplicate_witness",
];

fn fixture() -> (HypergraphStore, NodeId) {
    let c = CandidateRule {
        labels: vec![Some(1); 8],
        lhs: vec![(0..8).collect()],
        rhs: vec![(0..8).collect()],
        guards: vec![Guard {
            var: 0,
            key: 42,
            op: PropertyOp::Eq,
            value: PropertyValue::Int(1),
        }],
        ..CandidateRule::default()
    };
    let mut s = HypergraphStore::new();
    let root = encode_witness_family(&mut s, &c, &c, false, &MetaLimits::default())
        .unwrap()
        .root;
    (s, root)
}

fn bounded_engine(rules: Vec<hg_rules::CompiledRule>, seed: u64, budget: usize) -> Engine {
    let mut engine = Engine::new(rules, StepMode::MaxParallel, seed, 16);
    engine.set_match_budget_policy(budget, budget, MatchBudgetPolicy::Pooled);
    engine
}

fn repair_engine(seed: u64, budget: usize) -> Engine {
    let mut e = bounded_engine(vec![], seed, budget);
    let mut source = GraphRuleSource::new(
        EpochPolicy::Fixed(NonZeroU64::new(1).unwrap()),
        MetaLimits::default(),
        true,
    );
    source.vocabulary_enabled = true;
    source.witness_repair_enabled = true;
    e.set_rule_source(Box::new(source));
    e
}

fn fuel(s: &HypergraphStore, instruction: NodeId) -> f64 {
    read_instruction(&s.snapshot(), instruction).unwrap().fuel
}

fn decoded(s: &HypergraphStore, root: NodeId) -> Result<RuleProgram, Validator> {
    // Read-only means byte-identical live storage, including secondary indexes.
    let before = bincode::serialize(s).unwrap();
    let result = read_program(&s.snapshot(), root, &MetaLimits::default());
    assert_eq!(bincode::serialize(s).unwrap(), before);
    result
}

fn replace_label(s: &mut HypergraphStore, id: EdgeId, label: Label) {
    // Never mutate endpoints/labels without updating the store indexes.
    let mut edge = s.get_edge(id).unwrap().clone();
    s.remove_edge(id);
    edge.label = label;
    s.add_edge(edge);
}

/// Marker properties are fixture setup only; the attack itself is a native rule.
fn attack(s: &mut HypergraphStore, id: EdgeId, condition: &str) -> hg_rules::CompiledRule {
    let edge = s.get_edge(id).unwrap().clone();
    let mut unique = vec![];
    for &node in &edge.nodes {
        if !unique.contains(&node) {
            unique.push(node);
        }
    }
    let vars: Vec<_> = (0..unique.len()).map(|i| format!("v{i}")).collect();
    for (i, &node) in unique.iter().enumerate() {
        s.get_node_mut(node)
            .unwrap()
            .set_property(MARKER, PropertyValue::Int(i as i64));
    }
    let endpoints: Vec<_> = edge
        .nodes
        .iter()
        .map(|node| vars[unique.iter().position(|n| n == node).unwrap()].clone())
        .collect();
    let mut r = Rule {
        name: format!("adversarial_{condition}"),
        ..Rule::default()
    };
    r.lhs.nodes = vars.clone();
    r.rhs.nodes = vars.clone(); // deletion removes the edge, not its nodes
    r.lhs.edges = vec![endpoints.clone()];
    r.lhs.edge_labels.insert(0, edge.label);
    for (i, var) in vars.iter().enumerate() {
        r.lhs_node_property_guards.push(NodePropertyGuard {
            var: var.clone(),
            property: MARKER,
            op: PropertyOp::Eq,
            value: PropertyValue::Int(i as i64),
        });
    }
    if !condition.ends_with("delete") {
        let mut rhs = endpoints;
        if condition.ends_with("rewire") {
            let offset = usize::from(condition.starts_with("witness"));
            rhs.swap(offset, offset + 1);
        }
        r.rhs.edges.push(rhs.clone());
        let label = if condition.ends_with("erase") {
            Label(0)
        } else {
            edge.label
        };
        r.rhs.edge_labels.insert(0, label);
        if condition.starts_with("duplicate") {
            r.rhs.edges.push(rhs);
            r.rhs.edge_labels.insert(1, label);
        }
    }
    let mut compiled = compile_rule(&r).unwrap();
    compiled.meta_visible = true;
    assert!(compiled.pattern.node_label_ceiling.is_none());
    compiled
}

fn inventory(s: &HypergraphStore) -> Value {
    let mut nodes: Vec<_> = s
        .nodes()
        .map(|(id, data)| {
            let mut incident: Vec<_> = s
                .edges_incident_to(id)
                .iter()
                .map(|e| format!("{e:?}"))
                .collect();
            incident.sort(); // retain repeated incidences; order of the index is not semantic
            (
                format!("{id:?}"),
                json!({"data": data, "incident": incident}),
            )
        })
        .collect();
    nodes.sort_by(|a, b| a.0.cmp(&b.0));
    let mut edges: Vec<_> = s
        .edges()
        .map(|(id, edge)| (format!("{id:?}"), json!(edge)))
        .collect();
    edges.sort_by(|a, b| a.0.cmp(&b.0));
    let mut labels: Vec<_> = s.edges().map(|(_, e)| e.label.0).collect();
    labels.sort();
    labels.dedup();
    let indexes: Vec<_> = labels
        .into_iter()
        .map(|l| {
            let mut ids: Vec<_> = s
                .edges_with_label(Label(l))
                .iter()
                .map(|e| format!("{e:?}"))
                .collect();
            ids.sort();
            (l, ids)
        })
        .collect();
    json!({"step": s.step(), "nodes": nodes, "edges": edges, "label_indexes": indexes})
}

fn checkpoint(
    dir: &Path,
    phase: &str,
    s: &HypergraphStore,
    root: NodeId,
    instruction: NodeId,
) -> Value {
    let payload = bincode::serialize(s).unwrap();
    let mut writer = SnapshotWriter::new(&dir.join(phase)).unwrap();
    let path = writer.write_snapshot(s.step(), &payload).unwrap();
    let (step, back) = SnapshotReader::read_snapshot(&path).unwrap();
    assert_eq!(back, payload);
    let restored: HypergraphStore = bincode::deserialize(&back).unwrap();
    assert_eq!(step, s.step());
    assert_eq!(inventory(&restored), inventory(s));
    assert_eq!(decoded(&restored, root), decoded(s, root));
    assert_eq!(fuel(&restored, instruction), fuel(s, instruction));
    // Independently recompute the incidence and label indexes, including multiplicities.
    for (id, _) in restored.nodes() {
        let mut expected: Vec<_> = restored
            .edges()
            .flat_map(|(eid, e)| {
                e.nodes
                    .iter()
                    .filter(move |&&n| n == id)
                    .map(move |_| format!("{eid:?}"))
            })
            .collect();
        let mut actual: Vec<_> = restored
            .edges_incident_to(id)
            .iter()
            .map(|eid| format!("{eid:?}"))
            .collect();
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }
    for (_, edge) in restored.edges() {
        let mut expected: Vec<_> = restored
            .edges()
            .filter(|(_, e)| e.label == edge.label)
            .map(|(id, _)| format!("{id:?}"))
            .collect();
        let mut actual: Vec<_> = restored
            .edges_with_label(edge.label)
            .iter()
            .map(|id| format!("{id:?}"))
            .collect();
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }
    json!({"phase": phase, "path": format!("{phase}/{}", path.file_name().unwrap().to_str().unwrap()),
        "step": step, "root": root, "instruction": instruction, "fuel": fuel(s, instruction),
        "decoded": format!("{:?}", decoded(s, root)), "roundtrip_verified": true,
        "bytes": payload.len(), "inventory": inventory(s)})
}

#[test]
fn native_adversarial_matrix_has_exact_recovery_or_persistent_quarantine() {
    let temporary = tempfile::tempdir().unwrap();
    let out = if let Some(path) = std::env::var_os("ERE_ADVERSARIAL_OUTPUT") {
        let path = std::path::PathBuf::from(path);
        std::fs::create_dir(&path).expect("output must be fresh; refusing overwrite");
        path
    } else {
        temporary.path().join("assay")
    };
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../docs/ere_witness_adversarial_tests_v1.md"),
    )
    .unwrap();
    std::fs::write(
        out.join("test_source.rs"),
        include_str!("ere_repair_adversarial.rs"),
    )
    .unwrap();
    if std::env::var_os("ERE_ADVERSARIAL_OUTPUT").is_some() {
        std::fs::copy(
            std::env::current_exe().unwrap(),
            out.join("executed_test_binary"),
        )
        .unwrap();
    }
    let sources = out.join("source_snapshots");
    std::fs::create_dir(&sources).unwrap();
    for (name, source) in [
        ("repair.rs", include_str!("../../hg-meta/src/repair.rs")),
        ("source.rs", include_str!("../../hg-meta/src/source.rs")),
        ("program.rs", include_str!("../../hg-meta/src/program.rs")),
        (
            "vocabulary.rs",
            include_str!("../../hg-meta/src/vocabulary.rs"),
        ),
        ("region.rs", include_str!("../../hg-meta/src/region.rs")),
        ("schema.rs", include_str!("../../hg-meta/src/schema.rs")),
        ("engine.rs", include_str!("../../hg-engine/src/engine.rs")),
        ("matcher.rs", include_str!("../../hg-engine/src/matcher.rs")),
        ("graph.rs", include_str!("../../hg-core/src/graph.rs")),
        ("wire.rs", include_str!("../../hg-core/src/wire.rs")),
        ("binlog.rs", include_str!("../../hg-output/src/binlog.rs")),
        ("Cargo.lock", include_str!("../../../Cargo.lock")),
    ] {
        std::fs::write(sources.join(name), source).unwrap();
    }
    std::fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "protocol": "E2-witness-adversarial-tests-v1", "repair_protocol": REPAIR_PROTOCOL,
            "seeds": SEEDS, "conditions": CONDITIONS, "roles": WITNESS_ROLES,
            "repair_fuels": [0, 1], "horizon": HORIZON, "expected_cases": 1248,
            "match_work_limit": 50000, "match_result_limit": 50000, "max_pairings": 16,
            "training_admitted": false
        }))
        .unwrap(),
    )
    .unwrap();
    let (base, root) = fixture();
    let expected = decoded(&base, root).unwrap();
    let mut total = 0;
    let mut summary = vec![];
    for seed in SEEDS {
        for (role_index, &role) in WITNESS_ROLES.iter().enumerate() {
            let (payload_id, payload) = base
                .edges()
                .find(|(_, e)| e.label == Label(role) && e.arity() == 2)
                .unwrap();
            let witness_id = base
                .edges()
                .find(|(_, e)| {
                    e.label == Label(WITNESS_BASE + role_index as u32)
                        && e.nodes.as_slice() == [root, payload.nodes[0], payload.nodes[1]]
                })
                .unwrap()
                .0;
            for condition in CONDITIONS {
                for initial_fuel in [0, 1] {
                    let name = format!("{seed}_role{role_index:02}_{condition}_fuel{initial_fuel}");
                    let dir = out.join(&name);
                    std::fs::create_dir(&dir).unwrap();
                    let mut s = base.clone();
                    let instruction = encode_instruction(
                        &mut s,
                        &MetaInstructionSeed {
                            operation: MetaOperation::Repair,
                            fuel: initial_fuel as f64,
                        },
                    )
                    .unwrap();
                    let seal = encode_instruction(
                        &mut s,
                        &MetaInstructionSeed {
                            operation: MetaOperation::Seal,
                            fuel: f64::from(condition.starts_with("sealed")),
                        },
                    )
                    .unwrap();
                    let mut seal_paid = 0;
                    if condition.starts_with("sealed") {
                        // Select this payload through the real matcher/apply path, not a host conversion.
                        let mut found = false;
                        for mut rule in
                            compile_operations(MetaOperation::Seal, &MetaLimits::default()).unwrap()
                        {
                            rule.pattern.match_work_limit = Some(50000);
                            rule.pattern.match_result_limit = Some(50000);
                            let (bindings, usage) =
                                hg_engine::find_bindings_counted(&s.snapshot(), &rule, 0);
                            assert!(!usage.enumeration_truncated);
                            if let Some(binding) = bindings
                                .into_iter()
                                .find(|b| b.matched_edges.contains(&payload_id))
                            {
                                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
                                let before = rng.clone();
                                hg_engine::apply_binding(&mut s, &rule, &binding, &mut rng);
                                assert_eq!(rng, before);
                                found = true;
                                seal_paid = 1;
                                break;
                            }
                        }
                        assert!(found);
                        assert_eq!(fuel(&s, seal), 0.0);
                    }
                    let target =
                        if condition.starts_with("witness") || condition == "duplicate_witness" {
                            witness_id
                        } else if condition.starts_with("sealed") {
                            s.edges()
                                .find(|(_, e)| {
                                    e.label == Label(role)
                                        && e.nodes.as_slice()
                                            == [payload.nodes[0], payload.nodes[1], root]
                                })
                                .unwrap()
                                .0
                        } else {
                            payload_id
                        };
                    let rule = attack(&mut s, target, condition);
                    assert_eq!(decoded(&s, root), Ok(expected.clone()));
                    let before = checkpoint(&dir, "before_attack", &s, root, instruction);
                    let mut attacker = bounded_engine(vec![rule], seed, 50000);
                    let rng_attack = attacker.rng_state_probe();
                    let attack_stats = attacker.step(&mut s);
                    assert!(!attacker.match_ceiling_binding());
                    assert_eq!(attack_stats.accepted_rewrites, 1);
                    assert_eq!(attacker.rng_state_probe(), rng_attack);
                    let attack_valid = decoded(&s, root).is_ok();
                    let attacked = checkpoint(&dir, "after_attack", &s, root, instruction);
                    let mut repair = repair_engine(seed, 50000);
                    let rng_repair = repair.rng_state_probe();
                    let mut payments = 0;
                    let mut observations = vec![];
                    for _ in 0..HORIZON {
                        let before_fuel = fuel(&s, instruction);
                        let stats = repair.step(&mut s);
                        let report = repair.take_compile_report().unwrap();
                        let usage = repair.match_budget_usage();
                        let valid = decoded(&s, root).is_ok();
                        let paid = before_fuel - fuel(&s, instruction);
                        payments += stats.accepted_rewrites;
                        observations.push(json!({"step": s.step(), "valid": valid, "fuel": fuel(&s, instruction),
                            "paid": paid, "accepted_rewrites": stats.accepted_rewrites,
                            "world_classes": report.classes.len(), "quarantined": report.quarantined,
                            "work": usage.work, "results": usage.results, "ceiling": repair.match_ceiling_binding(),
                            "rng_probe": format!("{:016x}", repair.rng_state_probe())}));
                    }
                    let final_state = decoded(&s, root);
                    let recovered = condition == "binary_erase" && initial_fuel == 1;
                    let final_valid = condition == "no_attack" || recovered;
                    let after = checkpoint(&dir, "after_repair", &s, root, instruction);
                    let receipt = json!({"case": name, "seed": seed, "role": role, "role_index": role_index,
                        "condition": condition, "initial_fuel": initial_fuel, "seal_paid": seal_paid,
                        "attack_accepted": attack_stats.accepted_rewrites, "attack_valid": attack_valid,
                        "attack_work": attacker.match_budget_usage().work, "attack_results": attacker.match_budget_usage().results,
                        "attack_rng_probe": format!("{rng_attack:016x}"), "repair_rng_probe": format!("{rng_repair:016x}"),
                        "repair_paid": payments, "final_fuel": fuel(&s, instruction),
                        "final_valid": final_state.is_ok(), "expected_final_valid": final_valid,
                        "exact_original": final_state == Ok(expected.clone()), "observations": observations,
                        "checkpoints": [before, attacked, after]});
                    std::fs::write(
                        dir.join("receipt.json"),
                        serde_json::to_vec_pretty(&receipt).unwrap(),
                    )
                    .unwrap();
                    // Outcome assertions follow evidence emission. No retry/adaptive repair.
                    assert_eq!(attack_valid, condition == "no_attack", "{name}");
                    assert_eq!(final_state.is_ok(), final_valid, "{name}");
                    if final_valid {
                        assert_eq!(final_state, Ok(expected.clone()), "{name}");
                    }
                    assert_eq!(payments, usize::from(recovered), "{name}");
                    assert_eq!(initial_fuel as f64 - fuel(&s, instruction), payments as f64);
                    assert_eq!(repair.rng_state_probe(), rng_repair);
                    for (index, observation) in observations.iter().enumerate() {
                        assert_eq!(observation["ceiling"], false, "{name}");
                        assert_eq!(observation["valid"], final_valid, "{name}");
                        assert_eq!(
                            observation["world_classes"],
                            usize::from(condition == "no_attack" || (recovered && index > 0)),
                            "{name}"
                        );
                        assert_eq!(
                            observation["quarantined"],
                            usize::from(!(condition == "no_attack" || (recovered && index > 0))),
                            "{name}"
                        );
                        assert_eq!(
                            observation["paid"].as_f64().unwrap(),
                            observation["accepted_rewrites"].as_u64().unwrap() as f64
                        );
                        assert_eq!(observation["rng_probe"], format!("{rng_repair:016x}"));
                    }
                    summary.push(json!({"case": name, "condition": condition, "final_valid": final_valid, "repair_paid": payments}));
                    total += 1;
                }
            }
        }
    }
    assert_eq!(total, 1248);
    std::fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&json!({
            "completed_cases": total, "checkpoints_verified": total * 3, "cases": summary,
            "engineering_pass": true, "general_repair_claim": false, "training_admitted": false,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn every_all_role_seal_repair_and_activation_binding_is_paid_and_safe() {
    let (base, root) = fixture();
    let expected = decoded(&base, root).unwrap();
    let pairs: Vec<_> = base
        .edges()
        .filter(|(_, e)| e.arity() == 2)
        .map(|(id, e)| (id, e.label.0))
        .collect();
    assert!(WITNESS_ROLES
        .iter()
        .all(|role| pairs.iter().any(|(_, label)| label == role)));
    for operation in [MetaOperation::Seal, MetaOperation::Repair] {
        let mut total_bindings = 0;
        for &(id, role) in &pairs {
            let mut prepared = base.clone();
            let target = if operation == MetaOperation::Repair {
                replace_label(&mut prepared, id, Label(0));
                assert!(decoded(&prepared, root).is_err());
                prepared
                    .edges()
                    .find(|(_, e)| e.label == Label(0))
                    .unwrap()
                    .0
            } else {
                id
            };
            let instruction = encode_instruction(
                &mut prepared,
                &MetaInstructionSeed {
                    operation,
                    fuel: 1.0,
                },
            )
            .unwrap();
            let mut checked = 0;
            for rule in compile_operations(operation, &MetaLimits::default()).unwrap() {
                for binding in hg_engine::find_bindings(&prepared.snapshot(), &rule, 0)
                    .into_iter()
                    .filter(|b| b.matched_edges.contains(&target))
                {
                    let mut s = prepared.clone();
                    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SEEDS[0]);
                    let before = rng.clone();
                    hg_engine::apply_binding(&mut s, &rule, &binding, &mut rng);
                    assert_eq!(rng, before);
                    assert_eq!(fuel(&s, instruction), 0.0);
                    assert_eq!(
                        decoded(&s, root),
                        Ok(expected.clone()),
                        "{operation:?} role {role}"
                    );
                    assert!(s.get_edge(target).is_none());
                    checked += 1;
                }
            }
            assert!(checked > 0, "vacuous {operation:?} role {role}");
            total_bindings += checked;
        }
        println!(
            "all-role {operation:?}: {} payloads, {total_bindings} native bindings",
            pairs.len()
        );
    }
    let mut prepared = base;
    let instruction = encode_instruction(
        &mut prepared,
        &MetaInstructionSeed {
            operation: MetaOperation::WitnessActivate,
            fuel: 1.0,
        },
    )
    .unwrap();
    let mut active = expected;
    active.active = true;
    let mut checked = 0;
    for rule in compile_operations(MetaOperation::WitnessActivate, &MetaLimits::default()).unwrap()
    {
        for binding in hg_engine::find_bindings(&prepared.snapshot(), &rule, 0) {
            let mut s = prepared.clone();
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SEEDS[0]);
            let before = rng.clone();
            hg_engine::apply_binding(&mut s, &rule, &binding, &mut rng);
            assert_eq!(rng, before);
            assert_eq!(fuel(&s, instruction), 0.0);
            assert_eq!(decoded(&s, root), Ok(active.clone()));
            checked += 1;
        }
    }
    assert!(checked > 0);
    println!("all-role WitnessActivate: {checked} native bindings");
}

#[test]
fn repeated_native_damage_exhausts_fuel_without_renewal_or_cached_body() {
    let (mut s, root) = fixture();
    let expected = decoded(&s, root).unwrap();
    let instruction = encode_instruction(
        &mut s,
        &MetaInstructionSeed {
            operation: MetaOperation::Repair,
            fuel: 2.0,
        },
    )
    .unwrap();
    let mut repair = repair_engine(SEEDS[0], 50000);
    for round in 0..3 {
        // A repaired edge is sealed. Consume finite fuel on distinct still-binary payloads.
        let payload = s
            .edges()
            .find(|(_, e)| e.arity() == 2 && WITNESS_ROLES.contains(&e.label.0))
            .unwrap()
            .0;
        let rule = attack(&mut s, payload, "binary_erase");
        assert_eq!(
            bounded_engine(vec![rule], SEEDS[0], 50000)
                .step(&mut s)
                .accepted_rewrites,
            1
        );
        assert!(decoded(&s, root).is_err());
        let paid = repair.step(&mut s).accepted_rewrites;
        let _ = repair.take_compile_report();
        assert!(!repair.match_ceiling_binding());
        assert_eq!(paid, usize::from(round < 2));
        assert_eq!(fuel(&s, instruction), (1 - round as i32).max(0) as f64);
        if round < 2 {
            assert_eq!(decoded(&s, root), Ok(expected.clone()));
        } else {
            assert!(decoded(&s, root).is_err());
        }
    }
    for _ in 0..HORIZON {
        assert_eq!(repair.step(&mut s).accepted_rewrites, 0);
        assert!(!repair.match_ceiling_binding());
        assert!(decoded(&s, root).is_err());
    }
}

#[test]
fn erased_payload_cannot_be_repaired_without_its_witness() {
    for condition in ["witness_erase", "witness_delete"] {
        let (mut s, root) = fixture();
        let witness = s
            .edges()
            .find(|(_, e)| e.label == Label(WITNESS_BASE))
            .unwrap()
            .0;
        let rule = attack(&mut s, witness, condition);
        assert_eq!(
            bounded_engine(vec![rule], SEEDS[0], 50000)
                .step(&mut s)
                .accepted_rewrites,
            1
        );
        let payload = s.edges().find(|(_, e)| e.label == Label(E_CORE)).unwrap().0;
        let rule = attack(&mut s, payload, "binary_erase");
        assert_eq!(
            bounded_engine(vec![rule], SEEDS[0], 50000)
                .step(&mut s)
                .accepted_rewrites,
            1
        );
        let instruction = encode_instruction(
            &mut s,
            &MetaInstructionSeed {
                operation: MetaOperation::Repair,
                fuel: 1.0,
            },
        )
        .unwrap();
        let mut repair = repair_engine(SEEDS[0], 50000);
        for _ in 0..HORIZON {
            assert_eq!(repair.step(&mut s).accepted_rewrites, 0);
            assert!(!repair.match_ceiling_binding());
            assert!(decoded(&s, root).is_err());
            assert_eq!(fuel(&s, instruction), 1.0);
            assert_eq!(repair.take_compile_report().unwrap().classes.len(), 0);
        }
    }
}

#[test]
fn repair_and_quarantine_do_not_borrow_another_familys_body() {
    let (mut s, root) = fixture();
    let expected = decoded(&s, root).unwrap();
    let other = encode_witness_family(
        &mut s,
        &expected.core,
        expected.proposal.as_ref().unwrap(),
        false,
        &MetaLimits::default(),
    )
    .unwrap()
    .root;
    let payload = s
        .edges()
        .find(|(_, e)| e.label == Label(E_CORE) && e.nodes[0] == root)
        .unwrap()
        .0;
    let rule = attack(&mut s, payload, "binary_delete");
    assert_eq!(
        bounded_engine(vec![rule], SEEDS[0], 50000)
            .step(&mut s)
            .accepted_rewrites,
        1
    );
    let instruction = encode_instruction(
        &mut s,
        &MetaInstructionSeed {
            operation: MetaOperation::Repair,
            fuel: 1.0,
        },
    )
    .unwrap();
    let mut repair = repair_engine(SEEDS[0], 50000);
    for _ in 0..HORIZON {
        assert_eq!(repair.step(&mut s).accepted_rewrites, 0);
        assert!(!repair.match_ceiling_binding());
        assert!(decoded(&s, root).is_err());
        assert_eq!(decoded(&s, other), Ok(expected.clone()));
        assert_eq!(fuel(&s, instruction), 1.0);
        let report = repair.take_compile_report().unwrap();
        assert_eq!(report.classes.len(), 1);
        assert_eq!(report.quarantined, 1);
    }
}

#[test]
fn repair_match_ceiling_never_spends_fuel_or_applies_partial_enumeration() {
    let (mut s, root) = fixture();
    let payload = s.edges().find(|(_, e)| e.label == Label(E_CORE)).unwrap().0;
    let rule = attack(&mut s, payload, "binary_erase");
    assert_eq!(
        bounded_engine(vec![rule], SEEDS[0], 50000)
            .step(&mut s)
            .accepted_rewrites,
        1
    );
    let instruction = encode_instruction(
        &mut s,
        &MetaInstructionSeed {
            operation: MetaOperation::Repair,
            fuel: 1.0,
        },
    )
    .unwrap();
    let before = inventory(&s);
    let mut repair = repair_engine(SEEDS[0], 1);
    let rng = repair.rng_state_probe();
    assert_eq!(repair.step(&mut s).accepted_rewrites, 0);
    assert!(repair.match_ceiling_binding());
    assert_eq!(inventory(&s), before);
    assert_eq!(fuel(&s, instruction), 1.0);
    assert_eq!(repair.rng_state_probe(), rng);
    assert!(decoded(&s, root).is_err());
    // Same damaged graph with a non-binding allowance must engage the positive control.
    let mut adequate = repair_engine(SEEDS[0], 50000);
    assert_eq!(adequate.step(&mut s).accepted_rewrites, 1);
    assert!(!adequate.match_ceiling_binding());
    assert_eq!(fuel(&s, instruction), 0.0);
    assert!(decoded(&s, root).is_ok());
}
