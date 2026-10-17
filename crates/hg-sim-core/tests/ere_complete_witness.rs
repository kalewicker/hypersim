#![cfg(feature = "hg-meta")]
#[path = "support/complete.rs"]
mod complete;
#[allow(dead_code)]
#[path = "support/witness.rs"]
mod witness;
use complete::*;
use hg_core::{HyperEdge, HypergraphStore, Label, PropertyValue};
use hg_meta::*;
use serde_json::{json, Value};
use witness::{attack, attack_step, candidate, decoded, engine, inventory, replace, SEEDS};

const CONDITIONS: [&str; 14] = [
    "intact_paid",
    "erased_paid",
    "erased_zero",
    "erased_binary_only",
    "payload_deleted",
    "payload_reversed",
    "payload_erased_reversed",
    "payload_foreign_erased",
    "witness_erased",
    "witness_deleted",
    "both_erased",
    "witness_reversed_payload_erased",
    "duplicate_payload",
    "duplicate_witness",
];
fn save(path: &std::path::Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

#[test]
fn complete_witness_matrix_requires_all_composite_losses_quarantined() {
    let temporary = tempfile::tempdir().unwrap();
    let out = if let Some(path) = std::env::var_os("ERE_COMPLETE_OUTPUT") {
        let path = std::path::PathBuf::from(path);
        std::fs::create_dir(&path).expect("refusing to overwrite complete witness receipts");
        path
    } else {
        temporary.path().join("assay")
    };
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../docs/ere_complete_witness_v1.md"),
    )
    .unwrap();
    std::fs::write(
        out.join("test_source.rs"),
        include_str!("ere_complete_witness.rs"),
    )
    .unwrap();
    std::fs::write(
        out.join("fixture_source.rs"),
        include_str!("support/complete.rs"),
    )
    .unwrap();
    std::fs::write(
        out.join("shared_fixture_source.rs"),
        include_str!("support/witness.rs"),
    )
    .unwrap();
    if std::env::var_os("ERE_COMPLETE_OUTPUT").is_some() {
        std::fs::copy(
            std::env::current_exe().unwrap(),
            out.join("executed_test_binary"),
        )
        .unwrap();
    }
    let sources = out.join("source_snapshots");
    std::fs::create_dir(&sources).unwrap();
    for (name, source) in [
        ("complete.rs", include_str!("../../hg-meta/src/complete.rs")),
        ("lib.rs", include_str!("../../hg-meta/src/lib.rs")),
        ("repair.rs", include_str!("../../hg-meta/src/repair.rs")),
        ("source.rs", include_str!("../../hg-meta/src/source.rs")),
        ("program.rs", include_str!("../../hg-meta/src/program.rs")),
        (
            "vocabulary.rs",
            include_str!("../../hg-meta/src/vocabulary.rs"),
        ),
        ("region.rs", include_str!("../../hg-meta/src/region.rs")),
        ("schema.rs", include_str!("../../hg-meta/src/schema.rs")),
        (
            "compiled.rs",
            include_str!("../../hg-rules/src/compiled.rs"),
        ),
        ("engine.rs", include_str!("../../hg-engine/src/engine.rs")),
        ("matcher.rs", include_str!("../../hg-engine/src/matcher.rs")),
        ("runtime.rs", include_str!("../src/ere.rs")),
        ("graph.rs", include_str!("../../hg-core/src/graph.rs")),
        ("wire.rs", include_str!("../../hg-core/src/wire.rs")),
        ("binlog.rs", include_str!("../../hg-output/src/binlog.rs")),
        ("Cargo.lock", include_str!("../../../Cargo.lock")),
    ] {
        std::fs::write(sources.join(name), source).unwrap();
    }
    save(
        &out.join("manifest.json"),
        &json!({"diagnostic": "E2-complete-witness-controls-v1", "protocol": COMPLETE_PROTOCOL,
        "instruction_version": COMPLETE_INSTRUCTION_VERSION, "family_version": COMPLETE_VERSION, "opcode": 6,
        "seeds": SEEDS, "conditions": CONDITIONS, "roles": WITNESS_ROLES, "cases": 728,
        "horizon": 8, "work_limit": 50000, "result_limit": 50000, "max_pairings": 16, "training_admitted": false}),
    );
    let (base, root) = fixture();
    let expected = decoded(&base, root).unwrap();
    let mut summaries = vec![];
    for seed in SEEDS {
        for (ix, role) in WITNESS_ROLES.into_iter().enumerate() {
            let (payload, edge) = base
                .edges()
                .find(|(_, e)| e.label == Label(role) && e.arity() == 2)
                .unwrap();
            for condition in CONDITIONS {
                let name = format!("{seed}_role{ix:02}_{condition}");
                let dir = out.join(&name);
                std::fs::create_dir(&dir).unwrap();
                let mut s = base.clone();
                let seal_op = instruction(&mut s, MetaOperation::Seal, 1.0);
                let binary_fuel = f64::from(condition == "erased_binary_only");
                let initial_fuel =
                    f64::from(!matches!(condition, "erased_zero" | "erased_binary_only"));
                let binary_op = instruction(&mut s, MetaOperation::Repair, binary_fuel);
                let op = instruction(&mut s, MetaOperation::RepairSealed, initial_fuel);
                let sealed = seal(&mut s, payload, seal_op, seed);
                let witness_id = s
                    .edges()
                    .find(|(_, w)| {
                        w.label == Label(WITNESS_BASE + ix as u32)
                            && w.nodes.as_slice() == [root, edge.nodes[0], edge.nodes[1]]
                    })
                    .unwrap()
                    .0;
                let targets = match condition {
                    "intact_paid" => vec![(sealed, "identity")],
                    "erased_paid" | "erased_zero" | "erased_binary_only" => vec![(sealed, "erase")],
                    "payload_deleted" => vec![(sealed, "delete")],
                    "payload_reversed" => vec![(sealed, "reverse")],
                    "payload_erased_reversed" => vec![(sealed, "reverse_erase")],
                    "payload_foreign_erased" => vec![(sealed, "foreign_erase")],
                    "witness_erased" => vec![(witness_id, "erase")],
                    "witness_deleted" => vec![(witness_id, "delete")],
                    "both_erased" => vec![(witness_id, "erase"), (sealed, "erase")],
                    "witness_reversed_payload_erased" => {
                        vec![(witness_id, "witness_reverse"), (sealed, "erase")]
                    }
                    "duplicate_payload" => vec![(sealed, "duplicate")],
                    "duplicate_witness" => vec![(witness_id, "duplicate")],
                    _ => unreachable!(),
                };
                let rule = attack(&mut s, &targets);
                assert_eq!(decoded(&s, root), Ok(expected.clone()));
                let before = checkpoint(&dir, "before_attack", &s, root, op);
                let attack_usage = attack_step(&mut s, rule, seed);
                let attack_valid = decoded(&s, root).is_ok();
                let attacked = checkpoint(&dir, "after_attack", &s, root, op);
                let mut repair = source_engine(seed, 50000);
                let rng = repair.rng_state_probe();
                let mut observations = vec![];
                let mut total_paid = 0;
                for _ in 0..8 {
                    let old_fuel = fuel(&s, op);
                    let stats = repair.step(&mut s);
                    let report = repair.take_compile_report().unwrap();
                    let usage = repair.match_budget_usage();
                    total_paid += stats.accepted_rewrites;
                    observations.push(json!({"step": s.step(), "valid": decoded(&s, root).is_ok(),
                        "paid": old_fuel - fuel(&s, op), "fuel": fuel(&s, op), "accepted": stats.accepted_rewrites,
                        "binary_fuel": fuel(&s, binary_op), "world_classes": report.classes.len(), "quarantined": report.quarantined,
                        "work": usage.work, "results": usage.results, "ceiling": repair.match_ceiling_binding(),
                        "rng": format!("{:016x}", repair.rng_state_probe())}));
                }
                let recovered = condition == "erased_paid";
                let valid = condition == "intact_paid" || recovered;
                let result = decoded(&s, root);
                let after = checkpoint(&dir, "after_repair", &s, root, op);
                save(
                    &dir.join("receipt.json"),
                    &json!({"case": name, "seed": seed, "role_index": ix, "role": role,
                    "condition": condition, "initial_fuel": initial_fuel, "binary_initial_fuel": binary_fuel,
                    "binary_instruction": binary_op, "seal_instruction": seal_op, "seal_paid": 1,
                    "attack": attack_usage, "attack_valid": attack_valid, "repair_rng": format!("{rng:016x}"),
                    "repair_paid": total_paid, "final_fuel": fuel(&s, op), "final_valid": result.is_ok(),
                    "exact_original": result == Ok(expected.clone()), "observations": observations, "checkpoints": [before, attacked, after]}),
                );
                assert_eq!(attack_valid, condition == "intact_paid", "{name}");
                assert_eq!(result.is_ok(), valid, "{name}");
                if valid {
                    assert_eq!(result, Ok(expected.clone()), "{name}");
                }
                assert_eq!(total_paid, usize::from(recovered), "{name}");
                assert_eq!(fuel(&s, op), initial_fuel - f64::from(recovered));
                assert_eq!(fuel(&s, binary_op), binary_fuel);
                assert_eq!(repair.rng_state_probe(), rng);
                for (i, obs) in observations.iter().enumerate() {
                    let compiled = condition == "intact_paid" || (recovered && i > 0);
                    assert_eq!(obs["valid"], valid, "{name}");
                    assert_eq!(obs["world_classes"], usize::from(compiled), "{name}");
                    assert_eq!(obs["quarantined"], usize::from(!compiled), "{name}");
                    assert_eq!(obs["ceiling"], false, "{name}");
                    assert_eq!(obs["accepted"], usize::from(recovered && i == 0));
                    assert_eq!(
                        obs["paid"].as_f64().unwrap(),
                        f64::from(recovered && i == 0)
                    );
                }
                summaries.push(json!({"case": name, "condition": condition, "valid": valid,
                    "exact_original": result == Ok(expected.clone()), "paid": total_paid}));
            }
        }
    }
    save(
        &out.join("summary.json"),
        &json!({"completed_cases": summaries.len(), "checkpoints_verified": 2184,
        "engine_steps": 6552, "seal_paid": 728, "paid_mechanism_pass": true, "all_negative_quarantined": true,
        "valid_changed_body_cases": 0, "engineering_pass": true, "training_admitted": false, "cases": summaries}),
    );
}

#[test]
fn complete_ownership_rejects_double_erasure_for_every_role_without_host_history() {
    for role in WITNESS_ROLES {
        let (mut s, root) = fixture();
        let payload = s
            .edges()
            .find(|(_, e)| e.label == Label(role) && e.arity() == 2)
            .unwrap()
            .0;
        let op = instruction(&mut s, MetaOperation::Seal, 1.0);
        let payload = seal(&mut s, payload, op, SEEDS[0]);
        let endpoints = s.get_edge(payload).unwrap().nodes.clone();
        let ix = WITNESS_ROLES.iter().position(|&r| r == role).unwrap();
        let w = s
            .edges()
            .find(|(_, w)| {
                w.label == Label(WITNESS_BASE + ix as u32)
                    && w.nodes.as_slice() == [root, endpoints[0], endpoints[1]]
            })
            .unwrap()
            .0;
        let rule = attack(&mut s, &[(w, "erase"), (payload, "erase")]);
        attack_step(&mut s, rule, SEEDS[0]);
        assert!(decoded(&s, root).is_err());
        let bytes = bincode::serialize(&s).unwrap();
        let restored: HypergraphStore = bincode::deserialize(&bytes).unwrap();
        assert!(decoded(&restored, root).is_err());
        assert_eq!(bincode::serialize(&s).unwrap(), bytes);
    }
}

#[test]
fn ownership_and_declaration_damage_is_invalid_without_mutating_the_reader() {
    let (base, root) = fixture();
    let owner = base
        .edges()
        .find(|(_, e)| e.label == Label(E_OWNS))
        .unwrap()
        .0;
    let node = base.get_edge(owner).unwrap().nodes[1];
    for variant in 0..13 {
        let mut s = base.clone();
        match variant {
            0 => {
                erased(&mut s, owner);
            }
            1 => {
                s.remove_edge(owner);
            }
            2 => {
                let mut e = s.get_edge(owner).unwrap().clone();
                e.nodes.swap(0, 1);
                replace(&mut s, owner, e);
            }
            3 => {
                s.add_edge(s.get_edge(owner).unwrap().clone());
            }
            4 => s
                .get_node_mut(root)
                .unwrap()
                .set_property(K_OWNED_COUNT, PropertyValue::Int(0)),
            5 => s
                .get_node_mut(node)
                .unwrap()
                .set_property(K_SCHEMA_LABEL, PropertyValue::Int(0)),
            6 => s
                .get_node_mut(node)
                .unwrap()
                .set_property(K_REQUIRED_BASE, PropertyValue::Float(0.0)),
            7 => {
                s.get_node_mut(node)
                    .unwrap()
                    .properties
                    .retain(|(k, _)| *k != K_REQUIRED_BASE);
            }
            8 => s
                .get_node_mut(node)
                .unwrap()
                .set_property(K_REQUIRED_BASE, PropertyValue::Int(9999)),
            9 => {
                s.remove_node(node);
            }
            10 => {
                s.get_node_mut(node)
                    .unwrap()
                    .properties
                    .push((K_REQUIRED_BASE, PropertyValue::Int(0)));
            }
            11 => s
                .get_node_mut(root)
                .unwrap()
                .set_property(K_OWNED_COUNT, PropertyValue::Float(43.0)),
            12 => {
                for k in 0..65 {
                    s.get_node_mut(node)
                        .unwrap()
                        .set_property(0xA000 + k, PropertyValue::Int(0));
                }
            }
            _ => unreachable!(),
        }
        let before = bincode::serialize(&s).unwrap();
        assert!(decoded(&s, root).is_err(), "variant {variant}");
        assert_eq!(bincode::serialize(&s).unwrap(), before);
        let mut e = source_engine(SEEDS[0], 50000);
        assert_eq!(e.step(&mut s).accepted_rewrites, 0);
        let report = e.take_compile_report().unwrap();
        assert_eq!(report.classes.len(), 0);
        assert_eq!(report.quarantined, 1);
    }
    let mut cross = base.clone();
    let other = encode_complete_family(
        &mut cross,
        &candidate(),
        &candidate(),
        false,
        &MetaLimits::default(),
    )
    .unwrap()
    .root;
    cross.add_edge(HyperEdge::new(
        Label(E_OWNS),
        [other, node].into_iter().collect(),
    ));
    assert!(decoded(&cross, root).is_err());
    assert!(decoded(&cross, other).is_err());
}

#[test]
fn all_complete_native_bindings_are_paid_and_preserve_the_declared_program() {
    use rand::SeedableRng;
    let (base, root) = fixture();
    let expected = decoded(&base, root).unwrap();
    let payloads: Vec<_> = base
        .edges()
        .filter(|(_, e)| e.arity() == 2 && WITNESS_ROLES.contains(&e.label.0))
        .map(|(id, _)| id)
        .collect();
    assert_eq!(payloads.len(), 40);
    for operation in [
        MetaOperation::Seal,
        MetaOperation::Repair,
        MetaOperation::RepairSealed,
    ] {
        let rules = compile_complete_operations(operation, &MetaLimits::default()).unwrap();
        assert_eq!(rules.len(), 14);
        assert!(rules
            .iter()
            .all(|r| r.pattern.edges.iter().all(|e| e.ordered)));
        let mut count = 0;
        for &payload in &payloads {
            let mut s = base.clone();
            let target = if operation == MetaOperation::RepairSealed {
                let op = instruction(&mut s, MetaOperation::Seal, 1.0);
                seal(&mut s, payload, op, SEEDS[0])
            } else {
                payload
            };
            let target = if operation != MetaOperation::Seal {
                erased(&mut s, target)
            } else {
                target
            };
            let op = instruction(&mut s, operation, 1.0);
            let mut local = 0;
            for rule in &rules {
                for binding in hg_engine::find_bindings(&s.snapshot(), rule, 0) {
                    if !binding.matched_edges.contains(&target) {
                        // Seal is applicable to every still-binary payload;
                        // each such target is exercised by its own iteration.
                        assert_eq!(operation, MetaOperation::Seal);
                        continue;
                    }
                    let mut repaired = s.clone();
                    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SEEDS[0]);
                    let before = rng.clone();
                    hg_engine::apply_binding(&mut repaired, rule, &binding, &mut rng);
                    assert_eq!(rng, before);
                    assert_eq!(fuel(&repaired, op), 0.0);
                    assert_eq!(decoded(&repaired, root), Ok(expected.clone()));
                    local += 1;
                }
            }
            assert!(local > 0);
            count += local;
        }
        println!("complete {operation:?}: 40 payloads, {count} bindings");
        assert_eq!(count, 40);
    }
    let mut s = base.clone();
    let op = instruction(&mut s, MetaOperation::WitnessActivate, 1.0);
    let rules = compile_complete_operations(MetaOperation::WitnessActivate, &MetaLimits::default())
        .unwrap();
    assert_eq!(rules.len(), 1);
    let bindings = hg_engine::find_bindings(&s.snapshot(), &rules[0], 0);
    assert_eq!(bindings.len(), 1);
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SEEDS[0]);
    let before = rng.clone();
    hg_engine::apply_binding(&mut s, &rules[0], &bindings[0], &mut rng);
    assert_eq!(rng, before);
    assert_eq!(fuel(&s, op), 0.0);
    let mut active = expected;
    active.active = true;
    assert_eq!(decoded(&s, root), Ok(active));
}

#[test]
fn distinct_proposal_activation_survives_completeness_and_next_compile_boundary() {
    let mut s = HypergraphStore::new();
    let core = candidate();
    let mut proposal = core.clone();
    proposal.guards[0].value = PropertyValue::Int(2);
    let root = encode_complete_family(&mut s, &core, &proposal, false, &MetaLimits::default())
        .unwrap()
        .root;
    let op = instruction(&mut s, MetaOperation::WitnessActivate, 1.0);
    let declared_before = inventory(&s);
    let mut e = source_engine(SEEDS[0], 50000);
    let rng = e.rng_state_probe();
    assert_eq!(e.step(&mut s).accepted_rewrites, 1);
    assert_eq!(e.take_compile_report().unwrap().classes.len(), 1);
    let p = decoded(&s, root).unwrap();
    assert!(p.active);
    assert_eq!(p.core, core);
    assert_eq!(p.proposal, Some(proposal));
    assert_eq!(fuel(&s, op), 0.0);
    assert_eq!(e.step(&mut s).accepted_rewrites, 0);
    let report = e.take_compile_report().unwrap();
    assert_eq!(report.classes.len(), 2);
    assert_eq!(report.quarantined, 0);
    assert_eq!(e.rng_state_probe(), rng);
    // Logical declarations and all ownership links are unchanged by paid activation.
    let owners = |v: &Value| {
        v["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row[1]["label"] == E_OWNS)
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(owners(&declared_before), owners(&inventory(&s)));
}

#[test]
fn repeated_complete_repair_is_finite_and_match_ceiling_never_partially_applies() {
    let (mut s, root) = fixture();
    let expected = decoded(&s, root).unwrap();
    let target = s.edges().find(|(_, e)| e.label == Label(E_CORE)).unwrap().0;
    let op = instruction(&mut s, MetaOperation::Seal, 1.0);
    seal(&mut s, target, op, SEEDS[0]);
    let op = instruction(&mut s, MetaOperation::RepairSealed, 2.0);
    for round in 0..3 {
        let target = s
            .edges()
            .find(|(_, e)| e.label == Label(E_CORE) && e.arity() == 3)
            .unwrap()
            .0;
        let rule = attack(&mut s, &[(target, "erase")]);
        attack_step(&mut s, rule, SEEDS[0]);
        let before = inventory(&s);
        let mut capped = source_engine(SEEDS[0], 1);
        let rng = capped.rng_state_probe();
        assert_eq!(capped.step(&mut s).accepted_rewrites, 0);
        assert!(capped.match_ceiling_binding());
        assert_eq!(inventory(&s), before);
        assert_eq!(capped.rng_state_probe(), rng);
        let mut e = source_engine(SEEDS[0], 50000);
        let rng = e.rng_state_probe();
        assert_eq!(e.step(&mut s).accepted_rewrites, usize::from(round < 2));
        assert!(!e.match_ceiling_binding());
        assert_eq!(e.rng_state_probe(), rng);
        assert_eq!(fuel(&s, op), (1 - round as i32).max(0) as f64);
        if round < 2 {
            assert_eq!(decoded(&s, root), Ok(expected.clone()));
        } else {
            assert!(decoded(&s, root).is_err());
        }
    }
}

#[test]
fn complete_reader_is_bounded_and_seed_rejection_is_atomic() {
    let (mut s, root) = fixture();
    for _ in 0..4097 {
        s.add_edge(HyperEdge::new(Label(0), [root, root].into_iter().collect()));
    }
    let before = bincode::serialize(&s).unwrap();
    assert_eq!(decoded(&s, root), Err(Validator::Region));
    assert_eq!(bincode::serialize(&s).unwrap(), before);
    let mut out = HypergraphStore::new();
    let before = bincode::serialize(&out).unwrap();
    for limits in [
        MetaLimits {
            max_region_edges: 100,
            ..Default::default()
        },
        MetaLimits {
            radius: 0,
            ..Default::default()
        },
    ] {
        assert!(
            encode_complete_family(&mut out, &candidate(), &candidate(), false, &limits).is_err()
        );
        assert_eq!(bincode::serialize(&out).unwrap(), before);
    }
}

#[test]
fn old_composite_counterexample_and_profile_isolation_are_preserved() {
    let (mut old, root) = witness::fixture();
    let payload = old
        .edges()
        .find(|(_, e)| e.label == Label(E_SLOT_BASE) && e.arity() == 2)
        .unwrap()
        .0;
    let op = witness::instruction(&mut old, MetaOperation::Seal, 1.0);
    let target = witness::seal(&mut old, payload, op, SEEDS[0]);
    let endpoints = old.get_edge(target).unwrap().nodes.clone();
    let w = old
        .edges()
        .find(|(_, e)| {
            e.label == Label(WITNESS_BASE + 3)
                && e.nodes.as_slice() == [root, endpoints[0], endpoints[1]]
        })
        .unwrap()
        .0;
    let original = decoded(&old, root).unwrap();
    let rule = attack(&mut old, &[(target, "erase"), (w, "erase")]);
    attack_step(&mut old, rule, SEEDS[0]);
    assert!(decoded(&old, root).is_ok());
    assert_ne!(decoded(&old, root), Ok(original));
    instruction(&mut old, MetaOperation::RepairSealed, 1.0);
    let mut old_source = witness::source_engine(SEEDS[0], 50000, true);
    assert_eq!(old_source.step(&mut old).accepted_rewrites, 0);
    let report = old_source.take_compile_report().unwrap();
    assert_eq!(report.classes.len(), 1);
    assert_eq!(report.quarantined, 1);
    let (mut new, _) = fixture();
    let id = instruction(&mut new, MetaOperation::RepairSealed, 1.0);
    assert!(read_instruction(&new.snapshot(), id).is_err());
    let old_id = witness::instruction(&mut new, MetaOperation::RepairSealed, 1.0);
    assert!(read_complete_instruction(&new.snapshot(), old_id).is_err());
    let mut old_source = witness::source_engine(SEEDS[0], 50000, true);
    old_source.step(&mut new);
    let report = old_source.take_compile_report().unwrap();
    assert_eq!(report.classes.len(), 0);
    assert_eq!(report.quarantined, 2);
    let mut new_source = source_engine(SEEDS[0], 50000);
    new_source.step(&mut new);
    let report = new_source.take_compile_report().unwrap();
    assert_eq!(report.classes.len(), 1);
    assert_eq!(report.quarantined, 1);
}

#[test]
fn malformed_complete_instructions_never_authorize_native_payment() {
    let (mut base, root) = fixture();
    let target = base
        .edges()
        .find(|(_, e)| e.label == Label(E_SLOT_BASE))
        .unwrap()
        .0;
    let seal_op = instruction(&mut base, MetaOperation::Seal, 1.0);
    let target = seal(&mut base, target, seal_op, SEEDS[0]);
    erased(&mut base, target);
    let op = instruction(&mut base, MetaOperation::RepairSealed, 1.0);
    for (key, value) in [
        (K_ENCODING, PropertyValue::Int(9)),
        (K_OPCODE, PropertyValue::Int(4)),
        (K_OPCODE, PropertyValue::Int(99)),
        (K_FUEL, PropertyValue::Int(1)),
        (K_FUEL, PropertyValue::Float(0.0)),
        (K_FUEL, PropertyValue::Float(f64::NAN)),
        (K_FUEL, PropertyValue::Float(f64::INFINITY)),
        (K_FUEL, PropertyValue::Float(-1.0)),
        (K_FUEL, PropertyValue::Float(33.0)),
    ] {
        let mut s = base.clone();
        s.get_node_mut(op).unwrap().set_property(key, value);
        let before = bincode::serialize(&s).unwrap();
        let mut e = engine(
            compile_complete_operations(MetaOperation::RepairSealed, &MetaLimits::default())
                .unwrap(),
            SEEDS[0],
            50000,
        );
        let rng = e.rng_state_probe();
        assert_eq!(e.step(&mut s).accepted_rewrites, 0);
        assert_eq!(e.rng_state_probe(), rng);
        s.set_step(s.step() - 1);
        assert_eq!(bincode::serialize(&s).unwrap(), before);
        assert!(decoded(&s, root).is_err());
    }
    let mut attached = base.clone();
    attached.add_edge(HyperEdge::new(Label(0), [op, op].into_iter().collect()));
    let mut e = engine(
        compile_complete_operations(MetaOperation::RepairSealed, &MetaLimits::default()).unwrap(),
        SEEDS[0],
        50000,
    );
    assert_eq!(e.step(&mut attached).accepted_rewrites, 0);
    assert!(read_complete_instruction(&attached.snapshot(), op).is_err());
    let mut empty = HypergraphStore::new();
    for operation in [
        MetaOperation::Activate,
        MetaOperation::Deactivate,
        MetaOperation::Transpose,
    ] {
        assert!(encode_complete_instruction(
            &mut empty,
            &MetaInstructionSeed {
                operation,
                fuel: 1.0
            }
        )
        .is_err());
        assert!(compile_complete_operations(operation, &MetaLimits::default()).is_err());
    }
    assert_eq!(empty.node_count(), 0);
    assert!(compile_complete_operations(
        MetaOperation::RepairSealed,
        &MetaLimits {
            max_arity: 2,
            ..Default::default()
        }
    )
    .is_err());
}

fn runtime_config(operation: MetaOperation) -> hg_sim_core::ere::EreConfig {
    #[derive(serde::Serialize)]
    struct Raw {
        rules: Vec<hg_rules::Rule>,
    }
    let mut raw = toml::Value::try_from(Raw {
        rules: vec![candidate().to_rule("seed".into())],
    })
    .unwrap();
    raw["rules"][0]["lhs_node_property_guards"][0]["op"] = toml::Value::String("eq".into());
    let text = toml::to_string(&raw).unwrap();
    hg_sim_core::ere::EreConfig {
        arm: hg_sim_core::ere::EreArm::A3,
        encoding_version: EncodingVersion::AlternativeFamiliesV3,
        match_budget_policy: hg_rules::MatchBudgetPolicy::Pooled,
        epoch: hg_rules::EpochPolicy::Fixed(1.try_into().unwrap()),
        meta_seed_energy: 0.0,
        measure_family_maintenance: true,
        measure_ei: false,
        seed_families: Some(vec![hg_sim_core::ere::EreSeedFamily {
            core_toml: text.clone(),
            proposal_toml: text,
            active: false,
        }]),
        vocabulary: Some(hg_sim_core::ere::MetaVocabularyConfig {
            protocol: COMPLETE_PROTOCOL.into(),
            instructions: vec![MetaInstructionSeed {
                operation,
                fuel: 1.0,
            }],
        }),
        ..Default::default()
    }
}

#[test]
fn real_complete_runtime_records_paid_invalid_to_valid_transition_and_atomic_admission() {
    let mut s = HypergraphStore::new();
    let mut e = engine(
        vec![candidate().compile(&MetaLimits::default(), true).unwrap()],
        SEEDS[0],
        50000,
    );
    assert!(hg_sim_core::ere::EreRuntime::install(
        &runtime_config(MetaOperation::Activate),
        &mut s,
        &mut e
    )
    .is_err());
    assert_eq!(s.node_count(), 0);
    assert_eq!(s.edge_count(), 0);
    let mut runtime = hg_sim_core::ere::EreRuntime::install(
        &runtime_config(MetaOperation::RepairSealed),
        &mut s,
        &mut e,
    )
    .unwrap();
    let root = s
        .nodes()
        .find(|(_, n)| n.label == Label(L_RULE_ROOT))
        .unwrap()
        .0;
    let expected = decoded(&s, root).unwrap();
    let payload = s.edges().find(|(_, e)| e.label == Label(E_CORE)).unwrap().0;
    let seal_op = instruction(&mut s, MetaOperation::Seal, 1.0);
    let target = seal(&mut s, payload, seal_op, SEEDS[0]);
    let rule = attack(&mut s, &[(target, "erase")]);
    attack_step(&mut s, rule, SEEDS[0]);
    let rng = e.rng_state_probe();
    runtime.before_step(&s, &mut e).unwrap();
    let stats = e.step(&mut s);
    runtime.after_step(&s, &stats);
    assert_eq!(stats.accepted_rewrites, 1);
    assert_eq!(decoded(&s, root), Ok(expected));
    assert_eq!(e.rng_state_probe(), rng);
    let row = runtime.close_window(
        &s,
        hg_schema::MetricsRow::default(),
        hg_schema::MetricsRow::default(),
    );
    assert_eq!(
        row.encoded_meta_vocabulary_protocol.as_deref(),
        Some(COMPLETE_PROTOCOL)
    );
    let traces = row.encoded_meta_operation_traces.as_ref().unwrap();
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].operation, "RepairSealed");
    assert!(!traces[0].family_valid_before && traces[0].family_valid_after);
    assert_eq!(
        (traces[0].fuel_before, traces[0].fuel_after),
        (Some(1.0), Some(0.0))
    );
    assert!(row
        .encoded_meta_instruction_states
        .as_ref()
        .unwrap()
        .iter()
        .all(|state| state.valid));
}
