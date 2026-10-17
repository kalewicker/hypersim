#![cfg(feature = "hg-meta")]
#[path = "support/witness.rs"]
mod witness;
use hg_core::{HyperEdge, HypergraphStore, Label, PropertyValue};
use hg_meta::*;
use rand::SeedableRng;
use serde_json::{json, Value};
use witness::*;

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
fn sealed_repair_matrix_reports_exact_recovery_and_composite_damage_loss() {
    let temporary = tempfile::tempdir().unwrap();
    let out = if let Some(path) = std::env::var_os("ERE_SEALED_REPAIR_OUTPUT") {
        let path = std::path::PathBuf::from(path);
        std::fs::create_dir(&path).expect("refusing to overwrite sealed repair receipts");
        path
    } else {
        temporary.path().join("assay")
    };
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../docs/ere_sealed_repair_v3.md"),
    )
    .unwrap();
    std::fs::write(
        out.join("test_source.rs"),
        include_str!("ere_sealed_repair.rs"),
    )
    .unwrap();
    std::fs::write(
        out.join("fixture_source.rs"),
        include_str!("support/witness.rs"),
    )
    .unwrap();
    if std::env::var_os("ERE_SEALED_REPAIR_OUTPUT").is_some() {
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
        &json!({
        "diagnostic": "E2-sealed-repair-controls-v3", "protocol": SEALED_REPAIR_PROTOCOL,
            "instruction_version": SEALED_REPAIR_INSTRUCTION_VERSION, "family_version": WITNESS_VERSION,
            "opcode": MetaOperation::RepairSealed.code(), "seeds": SEEDS, "conditions": CONDITIONS,
            "roles": WITNESS_ROLES, "cases": 728, "horizon": 8, "work_limit": 50000,
            "result_limit": 50000, "max_pairings": 16, "training_admitted": false,
        }),
    );
    let (base, root) = fixture();
    let expected = decoded(&base, root).unwrap();
    let mut summaries = vec![];
    let mut changed_body_cases = 0;
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
                // Native Seal recreates its matched witness too. Never retain
                // a pre-Seal generational ID for a later attack stage.
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
                let post_attack_program = decoded(&s, root);
                let mut repair = source_engine(seed, 50000, true);
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
                        "binary_fuel": fuel(&s, binary_op), "world_classes": report.classes.len(),
                        "quarantined": report.quarantined, "work": usage.work, "results": usage.results,
                        "ceiling": repair.match_ceiling_binding(), "rng": format!("{:016x}", repair.rng_state_probe())}));
                }
                let recovered = condition == "erased_paid";
                let composite = condition == "both_erased";
                let final_valid =
                    condition == "intact_paid" || recovered || (composite && attack_valid);
                let result = decoded(&s, root);
                let after = checkpoint(&dir, "after_repair", &s, root, op);
                let receipt = json!({"case": name, "seed": seed, "role_index": ix, "role": role,
                    "condition": condition, "initial_fuel": initial_fuel, "binary_initial_fuel": binary_fuel,
                    "binary_instruction": binary_op, "seal_instruction": seal_op, "seal_paid": 1,
                    "attack": attack_usage, "attack_valid": attack_valid,
                    "repair_rng": format!("{rng:016x}"), "repair_paid": total_paid,
                    "final_fuel": fuel(&s, op), "final_valid": result.is_ok(),
                    "exact_original": result == Ok(expected.clone()), "observations": observations,
                    "checkpoints": [before, attacked, after]});
                save(&dir.join("receipt.json"), &receipt);
                if !composite {
                    assert_eq!(attack_valid, condition == "intact_paid", "{name}");
                }
                assert_eq!(result.is_ok(), final_valid, "{name}");
                if final_valid && !composite {
                    assert_eq!(result, Ok(expected.clone()), "{name}");
                }
                if composite {
                    assert_ne!(
                        result,
                        Ok(expected.clone()),
                        "not original-body recovery: {name}"
                    );
                    assert_eq!(result, post_attack_program, "no composite repair: {name}");
                    changed_body_cases += usize::from(attack_valid);
                }
                assert_eq!(total_paid, usize::from(recovered), "{name}");
                assert_eq!(fuel(&s, op), initial_fuel - f64::from(recovered));
                assert_eq!(fuel(&s, binary_op), binary_fuel);
                assert_eq!(repair.rng_state_probe(), rng);
                for (i, obs) in observations.iter().enumerate() {
                    let compiled_valid = condition == "intact_paid"
                        || (recovered && i > 0)
                        || (composite && attack_valid);
                    assert_eq!(obs["valid"], final_valid, "{name}");
                    assert_eq!(obs["ceiling"], false, "{name}");
                    assert_eq!(obs["world_classes"], usize::from(compiled_valid), "{name}");
                    assert_eq!(obs["quarantined"], usize::from(!compiled_valid), "{name}");
                    assert_eq!(obs["accepted"], usize::from(recovered && i == 0));
                    assert_eq!(
                        obs["paid"].as_f64().unwrap(),
                        f64::from(recovered && i == 0)
                    );
                    assert_eq!(obs["rng"], format!("{rng:016x}"));
                }
                summaries.push(
                    json!({"case": name, "condition": condition, "valid": final_valid,
                    "exact_original": result == Ok(expected.clone()), "paid": total_paid}),
                );
            }
        }
    }
    assert_eq!(summaries.len(), 728);
    save(
        &out.join("summary.json"),
        &json!({"completed_cases": summaries.len(), "checkpoints_verified": 2184,
        "engine_steps": 6552, "seal_paid": 728, "paid_mechanism_pass": true,
        "all_negative_quarantined": changed_body_cases == 0, "valid_changed_body_cases": changed_body_cases,
        "original_broader_gate_pass": changed_body_cases == 0, "training_admitted": false,
        "cases": summaries}),
    );
}

#[test]
fn seal_preamble_replaces_witness_id_and_attacks_must_resolve_the_live_edge() {
    let (mut s, root) = fixture();
    let witness_before = s
        .edges()
        .find(|(_, w)| w.label == Label(WITNESS_BASE))
        .unwrap()
        .0;
    let payload = s.edges().find(|(_, e)| e.label == Label(E_CORE)).unwrap().0;
    let seal_op = instruction(&mut s, MetaOperation::Seal, 1.0);
    seal(&mut s, payload, seal_op, SEEDS[0]);
    assert!(s.get_edge(witness_before).is_none());
    let live = s
        .edges()
        .find(|(_, w)| w.label == Label(WITNESS_BASE))
        .unwrap()
        .0;
    assert_ne!(live, witness_before);
    let rule = attack(&mut s, &[(live, "erase")]);
    attack_step(&mut s, rule, SEEDS[0]);
    let op = instruction(&mut s, MetaOperation::RepairSealed, 1.0);
    let mut repair = source_engine(SEEDS[0], 50000, true);
    assert_eq!(repair.step(&mut s).accepted_rewrites, 0);
    assert!(decoded(&s, root).is_err());
    assert_eq!(fuel(&s, op), 1.0);
}

#[test]
fn every_sealed_payload_binding_is_safe_paid_and_order_sensitive() {
    let limits = MetaLimits::default();
    let (base, root) = fixture();
    let expected = decoded(&base, root).unwrap();
    let payloads: Vec<_> = base
        .edges()
        .filter(|(_, e)| e.arity() == 2)
        .map(|(id, _)| id)
        .collect();
    assert_eq!(payloads.len(), 40);
    let mut total = 0;
    let rules = compile_operations(MetaOperation::RepairSealed, &limits).unwrap();
    assert_eq!(rules.len(), 14);
    assert!(rules
        .iter()
        .all(|r| r.pattern.edges.len() == 2 && r.pattern.edges.iter().all(|e| e.ordered)));
    for payload in payloads {
        let mut prepared = base.clone();
        let seal_op = instruction(&mut prepared, MetaOperation::Seal, 1.0);
        let sealed = seal(&mut prepared, payload, seal_op, SEEDS[0]);
        let mut edge = prepared.get_edge(sealed).unwrap().clone();
        edge.label = Label(0);
        let erased = replace(&mut prepared, sealed, edge);
        let op = instruction(&mut prepared, MetaOperation::RepairSealed, 1.0);
        assert!(decoded(&prepared, root).is_err());
        let mut checked = 0;
        for rule in &rules {
            for binding in hg_engine::find_bindings(&prepared.snapshot(), rule, 0) {
                assert!(binding.matched_edges.contains(&erased));
                let mut s = prepared.clone();
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SEEDS[0]);
                let before = rng.clone();
                hg_engine::apply_binding(&mut s, rule, &binding, &mut rng);
                assert_eq!(rng, before);
                assert_eq!(fuel(&s, op), 0.0);
                assert_eq!(decoded(&s, root), Ok(expected.clone()));
                checked += 1;
            }
        }
        assert!(checked > 0, "unexercised payload");
        total += checked;
    }
    println!("RepairSealed: 40 sealed payloads, {total} native bindings");
    for op in [
        MetaOperation::Seal,
        MetaOperation::Repair,
        MetaOperation::WitnessActivate,
    ] {
        assert!(compile_operations(op, &limits).unwrap().iter().all(|r| r
            .pattern
            .edges
            .iter()
            .all(|e| !e.ordered)));
        let mut bytes = REPAIR_PROTOCOL.as_bytes().to_vec();
        bytes.extend(op.code().to_le_bytes());
        assert_eq!(op.execution_bytes(), bytes);
        assert_eq!(op.instruction_version(), 8);
    }
}

#[test]
fn repeated_same_ternary_erasure_recovers_until_finite_fuel_is_exhausted() {
    let (mut s, root) = fixture();
    let expected = decoded(&s, root).unwrap();
    let payload = s.edges().find(|(_, e)| e.label == Label(E_CORE)).unwrap().0;
    let seal_op = instruction(&mut s, MetaOperation::Seal, 1.0);
    seal(&mut s, payload, seal_op, SEEDS[0]);
    let op = instruction(&mut s, MetaOperation::RepairSealed, 2.0);
    let mut repair = source_engine(SEEDS[0], 50000, true);
    for round in 0..3 {
        let payload = s
            .edges()
            .find(|(_, e)| e.label == Label(E_CORE) && e.arity() == 3)
            .unwrap()
            .0;
        let rule = attack(&mut s, &[(payload, "erase")]);
        attack_step(&mut s, rule, SEEDS[0]);
        assert!(decoded(&s, root).is_err());
        let stats = repair.step(&mut s);
        assert!(!repair.match_ceiling_binding());
        assert_eq!(stats.accepted_rewrites, usize::from(round < 2));
        assert_eq!(fuel(&s, op), (1 - round as i32).max(0) as f64);
        if round < 2 {
            assert_eq!(decoded(&s, root), Ok(expected.clone()));
        } else {
            assert!(decoded(&s, root).is_err());
        }
    }
    for _ in 0..8 {
        assert_eq!(repair.step(&mut s).accepted_rewrites, 0);
        assert!(decoded(&s, root).is_err());
    }
}

#[test]
fn native_guards_and_ceiling_reject_corrupted_instruction_without_payment() {
    let (mut base, root) = fixture();
    let payload = base
        .edges()
        .find(|(_, e)| e.label == Label(E_SLOT_BASE))
        .unwrap()
        .0;
    let seal_op = instruction(&mut base, MetaOperation::Seal, 1.0);
    let sealed = seal(&mut base, payload, seal_op, SEEDS[0]);
    let rule = attack(&mut base, &[(sealed, "erase")]);
    attack_step(&mut base, rule, SEEDS[0]);
    let op = instruction(&mut base, MetaOperation::RepairSealed, 1.0);
    for (key, value) in [
        (K_ENCODING, PropertyValue::Int(8)),
        (K_OPCODE, PropertyValue::Int(4)),
        (K_FUEL, PropertyValue::Int(1)),
        (K_FUEL, PropertyValue::Float(0.0)),
        (K_FUEL, PropertyValue::Float(f64::NAN)),
        (K_FUEL, PropertyValue::Float(f64::INFINITY)),
        (K_FUEL, PropertyValue::Float(-1.0)),
        (K_FUEL, PropertyValue::Float(33.0)),
    ] {
        let mut s = base.clone();
        s.get_node_mut(op).unwrap().set_property(key, value);
        let mut e = engine(
            compile_operations(MetaOperation::RepairSealed, &MetaLimits::default()).unwrap(),
            SEEDS[0],
            50000,
        );
        let before = bincode::serialize(&s).unwrap();
        let rng = e.rng_state_probe();
        assert_eq!(e.step(&mut s).accepted_rewrites, 0);
        assert!(!e.match_ceiling_binding());
        s.set_step(s.step() - 1);
        assert_eq!(bincode::serialize(&s).unwrap(), before);
        assert_eq!(e.rng_state_probe(), rng);
    }
    let mut attached = base.clone();
    attached.add_edge(HyperEdge::new(Label(0), [op, op].into_iter().collect()));
    assert_eq!(
        engine(
            compile_operations(MetaOperation::RepairSealed, &MetaLimits::default()).unwrap(),
            SEEDS[0],
            50000
        )
        .step(&mut attached)
        .accepted_rewrites,
        0
    );
    let mut e = source_engine(SEEDS[0], 1, true);
    let mut s = base.clone();
    let before = inventory(&s);
    let rng = e.rng_state_probe();
    assert_eq!(e.step(&mut s).accepted_rewrites, 0);
    assert!(e.match_ceiling_binding());
    assert_eq!(inventory(&s), before);
    assert_eq!(fuel(&s, op), 1.0);
    assert_eq!(e.rng_state_probe(), rng);
    let mut sufficient = source_engine(SEEDS[0], 50000, true);
    assert_eq!(sufficient.step(&mut s).accepted_rewrites, 1);
    assert!(decoded(&s, root).is_ok());
    assert!(compile_operations(
        MetaOperation::RepairSealed,
        &MetaLimits {
            max_arity: 2,
            ..Default::default()
        }
    )
    .is_err());
    assert!(compile_operation(MetaOperation::RepairSealed, &MetaLimits::default()).is_err());
}

#[test]
fn ordered_constraint_is_opt_in_and_outside_the_world_candidate_envelope() {
    let mut s = HypergraphStore::new();
    let a = s.add_node(hg_core::NodeData::new(Label(1), 0));
    let b = s.add_node(hg_core::NodeData::new(Label(2), 0));
    s.add_edge(HyperEdge::new(Label(0), [b, a].into_iter().collect()));
    let c = CandidateRule {
        labels: vec![Some(1), Some(2)],
        lhs: vec![vec![0, 1]],
        rhs: vec![vec![0, 1]],
        ..Default::default()
    };
    let mut rule = c.compile(&MetaLimits::default(), true).unwrap();
    assert!(!rule.pattern.edges[0].ordered);
    assert_eq!(hg_engine::find_bindings(&s.snapshot(), &rule, 0).len(), 1);
    rule.pattern.edges[0].ordered = true;
    assert!(hg_engine::find_bindings(&s.snapshot(), &rule, 0).is_empty());
    assert!(CandidateRule::from_compiled(&rule, &MetaLimits::default())
        .unwrap_err()
        .contains("ordered-edge"));
    let mut missing = HypergraphStore::new();
    let n = missing.add_node(hg_core::NodeData::new(Label(1), 0));
    missing.add_edge(HyperEdge::new(Label(0), [n, n].into_iter().collect()));
    let c = CandidateRule {
        labels: vec![Some(1)],
        lhs: vec![vec![0, 0]],
        rhs: vec![vec![0, 0]],
        ..Default::default()
    };
    let mut repeated = c.compile(&MetaLimits::default(), true).unwrap();
    repeated.pattern.edges[0].ordered = true;
    assert_eq!(
        hg_engine::find_bindings(&missing.snapshot(), &repeated, 0).len(),
        1
    );
}

fn runtime_config(protocol: &str) -> hg_sim_core::ere::EreConfig {
    #[derive(serde::Serialize)]
    struct Raw {
        rules: Vec<hg_rules::Rule>,
    }
    let mut raw = toml::Value::try_from(Raw {
        rules: vec![candidate().to_rule("seed".into())],
    })
    .unwrap();
    // Rule's derived enum serializer writes "Eq"; the native TOML grammar
    // spells this guard "eq". Keep this fixture conversion explicit.
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
            protocol: protocol.into(),
            instructions: vec![MetaInstructionSeed {
                operation: MetaOperation::RepairSealed,
                fuel: 1.0,
            }],
        }),
        ..Default::default()
    }
}

#[test]
fn foreign_family_root_is_not_a_surviving_root_link_witness() {
    for role in [E_CORE, E_GRAFT] {
        let (mut s, root) = fixture();
        let c = candidate();
        let other = encode_witness_family(&mut s, &c, &c, false, &MetaLimits::default())
            .unwrap()
            .root;
        let payload = s
            .edges()
            .find(|(_, e)| e.label == Label(role) && e.nodes[0] == root)
            .unwrap()
            .0;
        let op_seal = instruction(&mut s, MetaOperation::Seal, 1.0);
        let sealed = seal(&mut s, payload, op_seal, SEEDS[0]);
        let ix = WITNESS_ROLES.iter().position(|r| *r == role).unwrap();
        let id = s
            .edges()
            .find(|(_, e)| e.label == Label(WITNESS_BASE + ix as u32) && e.nodes[0] == root)
            .unwrap()
            .0;
        // Malformed loaded graph control; do not bypass incidence-index updates.
        let mut edge = s.get_edge(id).unwrap().clone();
        edge.nodes[1] = other;
        replace(&mut s, id, edge);
        let mut edge = s.get_edge(sealed).unwrap().clone();
        edge.label = Label(0);
        replace(&mut s, sealed, edge);
        let op = instruction(&mut s, MetaOperation::RepairSealed, 1.0);
        assert!(decoded(&s, root).is_err());
        let mut e = engine(
            compile_operations(MetaOperation::RepairSealed, &MetaLimits::default()).unwrap(),
            SEEDS[0],
            50000,
        );
        for _ in 0..8 {
            assert_eq!(e.step(&mut s).accepted_rewrites, 0);
            assert!(!e.match_ceiling_binding());
        }
        assert_eq!(fuel(&s, op), 1.0);
        assert!(decoded(&s, root).is_err());
    }
}

#[test]
fn protocol_isolation_and_runtime_trace_cover_paid_invalid_to_valid_transition() {
    for protocol in [REPAIR_PROTOCOL, VOCABULARY_PROTOCOL] {
        let mut s = HypergraphStore::new();
        let mut e = engine(
            vec![candidate().compile(&MetaLimits::default(), true).unwrap()],
            SEEDS[0],
            50000,
        );
        assert!(
            hg_sim_core::ere::EreRuntime::install(&runtime_config(protocol), &mut s, &mut e)
                .err()
                .unwrap()
                .contains("different semantic protocol")
        );
        assert_eq!(s.node_count(), 0);
        assert_eq!(s.edge_count(), 0);
    }
    let (mut s, _) = fixture();
    instruction(&mut s, MetaOperation::RepairSealed, 1.0);
    let mut old_source = source_engine(SEEDS[0], 50000, false);
    old_source.step(&mut s);
    let report = old_source.take_compile_report().unwrap();
    assert_eq!(report.quarantined, 1);
    assert!(report.encoded_meta_operations.is_empty());
    let mut s = HypergraphStore::new();
    let mut e = engine(
        vec![candidate().compile(&MetaLimits::default(), true).unwrap()],
        SEEDS[0],
        50000,
    );
    let mut runtime = hg_sim_core::ere::EreRuntime::install(
        &runtime_config(SEALED_REPAIR_PROTOCOL),
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
    let sealed = seal(&mut s, payload, seal_op, SEEDS[0]);
    let rule = attack(&mut s, &[(sealed, "erase")]);
    attack_step(&mut s, rule, SEEDS[0]);
    // Attack stage is explicitly outside runtime observations. Observe the real paid repair step.
    let rng = e.rng_state_probe();
    runtime.before_step(&s, &mut e).unwrap();
    assert_eq!(e.rng_state_probe(), rng);
    let stats = e.step(&mut s);
    runtime.after_step(&s, &stats);
    assert_eq!(stats.accepted_rewrites, 1);
    assert_eq!(e.rng_state_probe(), rng);
    assert_eq!(decoded(&s, root), Ok(expected));
    let row = runtime.close_window(
        &s,
        hg_schema::MetricsRow::default(),
        hg_schema::MetricsRow::default(),
    );
    assert_eq!(
        row.encoded_meta_vocabulary_protocol.as_deref(),
        Some(SEALED_REPAIR_PROTOCOL)
    );
    assert_eq!(row.encoded_meta_rewrite_events, Some(1));
    let trace = &row.encoded_meta_operation_traces.as_ref().unwrap()[0];
    assert_eq!(trace.operation, "RepairSealed");
    assert_eq!(
        (trace.fuel_before, trace.fuel_after),
        (Some(1.0), Some(0.0))
    );
    assert!(!trace.family_valid_before && trace.family_valid_after);
}
