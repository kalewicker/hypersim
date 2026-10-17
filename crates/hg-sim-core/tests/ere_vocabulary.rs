#![cfg(feature = "hg-meta")]
use hg_core::{HyperEdge, HypergraphStore, Label, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_rules::{EpochPolicy, MatchBudgetPolicy, StepMode};
use hg_schema::MetricsRow;
use hg_sim_core::ere::{
    experiments, EreArm, EreConfig, EreRuntime, EreSeedFamily, MetaVocabularyConfig,
};
use rand::SeedableRng;

#[test]
fn all_isolated_bindings_preserve_seed_bank_family_pairs_and_rng() {
    let bank = experiments::seed_bank();
    let limits = MetaLimits::default();
    let mut checked = 0;
    for core in &bank {
        for proposal in &bank {
            for active in [false, true] {
                for operation in [
                    MetaOperation::Activate,
                    MetaOperation::Deactivate,
                    MetaOperation::Transpose,
                ] {
                    let mut store = HypergraphStore::new();
                    let root = encode_family(&mut store, core, proposal, active, &limits)
                        .unwrap()
                        .root;
                    let instruction = encode_instruction(
                        &mut store,
                        &MetaInstructionSeed {
                            operation,
                            fuel: 1.0,
                        },
                    )
                    .unwrap();
                    let rule = compile_operation(operation, &limits).unwrap();
                    let bindings = hg_engine::find_bindings(&store.snapshot(), &rule, 0);
                    for binding in bindings {
                        let mut after = store.clone();
                        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
                        let original_rng = rng.clone();
                        hg_engine::apply_binding(&mut after, &rule, &binding, &mut rng);
                        assert_eq!(rng, original_rng, "deterministic edits consume no RNG");
                        let decoded = read_program(&after.snapshot(), root, &limits).unwrap();
                        assert!(decoded.proposal.is_some());
                        assert_eq!(
                            (store.node_count(), store.edge_count()),
                            (after.node_count(), after.edge_count())
                        );
                        assert_eq!(
                            read_instruction(&after.snapshot(), instruction)
                                .unwrap()
                                .fuel,
                            0.0
                        );
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(checked > 1000);
    eprintln!("verified {checked} isolated primitive bindings across seed-bank pairs");
}

fn core() -> CandidateRule {
    experiments::seed_bank()[0].clone()
}
fn proposal() -> CandidateRule {
    CandidateRule {
        labels: vec![None; 3],
        lhs: vec![vec![0, 1], vec![1, 2]],
        rhs: vec![vec![0, 2]],
        priority: 1,
        ..CandidateRule::default()
    }
}
fn config(arm: EreArm, operation: MetaOperation, fuel: f64, active: bool) -> EreConfig {
    #[derive(serde::Serialize)]
    struct Raw {
        rules: Vec<hg_rules::Rule>,
    }
    let text = |r: CandidateRule| {
        toml::to_string(&Raw {
            rules: vec![r.to_rule("seed".into())],
        })
        .unwrap()
    };
    EreConfig {
        arm,
        encoding_version: EncodingVersion::AlternativeFamiliesV3,
        match_budget_policy: MatchBudgetPolicy::Pooled,
        epoch: EpochPolicy::Fixed(1.try_into().unwrap()),
        meta_seed_energy: 0.0,
        measure_ei: false,
        measure_family_maintenance: true,
        seed_families: Some(vec![EreSeedFamily {
            core_toml: text(core()),
            proposal_toml: text(proposal()),
            active,
        }]),
        vocabulary: Some(MetaVocabularyConfig {
            protocol: VOCABULARY_PROTOCOL.into(),
            instructions: vec![MetaInstructionSeed { operation, fuel }],
        }),
        ..EreConfig::default()
    }
}
fn setup(c: &EreConfig) -> (HypergraphStore, Engine, EreRuntime) {
    let mut s = experiments::fixture(&core());
    let mut e = Engine::new(
        vec![core().compile(&MetaLimits::default(), false).unwrap()],
        StepMode::MaxParallel,
        42,
        16,
    );
    let r = EreRuntime::install(c, &mut s, &mut e).unwrap();
    (s, e, r)
}
fn tick(s: &mut HypergraphStore, e: &mut Engine, r: &mut EreRuntime) {
    let rng = e.rng_state_probe();
    r.before_step(s, e).unwrap();
    assert_eq!(rng, e.rng_state_probe());
    let stats = e.step(s);
    assert!(!e.match_ceiling_binding());
    r.after_step(s, &stats);
}

#[test]
fn every_primitive_preserves_family_and_debits_exactly_once_without_world_credit() {
    for op in [
        MetaOperation::Activate,
        MetaOperation::Deactivate,
        MetaOperation::Transpose,
    ] {
        let (mut s, mut e, mut r) = setup(&config(
            EreArm::A2,
            op,
            1.0,
            op == MetaOperation::Deactivate,
        ));
        let root = s.nodes().find(|(_, n)| n.label.0 == L_RULE_ROOT).unwrap().0;
        let before = read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap();
        let counts = (s.node_count(), s.edge_count());
        tick(&mut s, &mut e, &mut r);
        let after = read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap();
        assert_eq!(counts, (s.node_count(), s.edge_count()));
        match op {
            MetaOperation::Activate => assert!(after.active && !before.active),
            MetaOperation::Deactivate => assert!(!after.active && before.active),
            MetaOperation::Transpose => assert_ne!(after.proposal, before.proposal),
            _ => unreachable!("v1 fixture contains only v1 operations"),
        }
        for _ in 0..4 {
            tick(&mut s, &mut e, &mut r);
        }
        let row = r.close_window(&s, MetricsRow::default(), MetricsRow::default());
        assert_eq!(row.encoded_meta_rewrite_events, Some(1));
        assert_eq!(row.ordinary_meta_rewrite_events, Some(1));
        assert_eq!(row.seeded_editor_rewrite_events, Some(0));
        let traces = row.encoded_meta_operation_traces.unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(
            (traces[0].fuel_before, traces[0].fuel_after),
            (Some(1.0), Some(0.0))
        );
        assert!(traces[0].family_valid_before && traces[0].family_valid_after);
        assert!(row
            .seeded_family_maintenance
            .unwrap()
            .iter()
            .all(|f| f.valid_steps == 5 && f.final_valid));
        let instructions = row.encoded_meta_instruction_states.unwrap();
        assert_eq!(
            (
                instructions[0].checks,
                instructions[0].valid_steps,
                instructions[0].fuel
            ),
            (5, 5, Some(0.0))
        );
        assert!(
            row.structural_proposal_reuse_count_by_class
                .unwrap()
                .is_empty(),
            "meta-only edits cannot earn world credit"
        );
    }
}

#[test]
fn transpose_is_a_reversible_real_body_edit_with_finite_energy() {
    let mut s = HypergraphStore::new();
    let root = encode_family(&mut s, &core(), &proposal(), true, &MetaLimits::default())
        .unwrap()
        .root;
    let op = encode_instruction(
        &mut s,
        &MetaInstructionSeed {
            operation: MetaOperation::Transpose,
            fuel: 2.0,
        },
    )
    .unwrap();
    let mut e = Engine::new(
        vec![compile_operation(MetaOperation::Transpose, &MetaLimits::default()).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let old = read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap();
    assert_eq!(e.step(&mut s).accepted_rewrites, 1);
    let changed = read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap();
    assert_ne!(old.proposal, changed.proposal);
    assert_eq!(e.step(&mut s).accepted_rewrites, 1);
    assert_eq!(
        read_program(&s.snapshot(), root, &MetaLimits::default()).unwrap(),
        old
    );
    assert_eq!(read_instruction(&s.snapshot(), op).unwrap().fuel, 0.0);
    assert_eq!(e.step(&mut s).accepted_rewrites, 0);
}

#[test]
fn cached_native_guards_exclude_damaged_and_exhausted_instructions() {
    for (key, value) in [
        (K_ENCODING, PropertyValue::Int(99)),
        (K_OPCODE, PropertyValue::Int(99)),
        (K_FUEL, PropertyValue::Int(1)),
        (K_FUEL, PropertyValue::Float(f64::NAN)),
        (K_FUEL, PropertyValue::Float(-1.0)),
        (K_FUEL, PropertyValue::Float(33.0)),
        (K_FUEL, PropertyValue::Float(0.0)),
    ] {
        let mut s = HypergraphStore::new();
        encode_family(&mut s, &core(), &proposal(), false, &MetaLimits::default()).unwrap();
        let root = encode_instruction(
            &mut s,
            &MetaInstructionSeed {
                operation: MetaOperation::Activate,
                fuel: 1.0,
            },
        )
        .unwrap();
        let mut e = Engine::new(
            vec![compile_operation(MetaOperation::Activate, &MetaLimits::default()).unwrap()],
            StepMode::MaxParallel,
            42,
            1,
        );
        s.get_node_mut(root).unwrap().set_property(key, value);
        assert_eq!(e.step(&mut s).accepted_rewrites, 0);
    }
    let mut s = HypergraphStore::new();
    encode_family(&mut s, &core(), &proposal(), false, &MetaLimits::default()).unwrap();
    let root = encode_instruction(
        &mut s,
        &MetaInstructionSeed {
            operation: MetaOperation::Activate,
            fuel: 1.0,
        },
    )
    .unwrap();
    s.add_edge(HyperEdge::new(Label(0), [root, root].into_iter().collect()));
    let mut e = Engine::new(
        vec![compile_operation(MetaOperation::Activate, &MetaLimits::default()).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    assert_eq!(e.step(&mut s).accepted_rewrites, 0);
}

#[test]
fn static_and_frozen_tables_have_identical_graph_and_rng_trajectories() {
    let (mut a, mut ea, mut ra) = setup(&config(EreArm::A0, MetaOperation::Activate, 1.0, false));
    let (mut b, mut eb, mut rb) = setup(&config(EreArm::A1, MetaOperation::Activate, 1.0, false));
    for s in [&mut a, &mut b] {
        let nodes: Vec<_> = (0..3)
            .map(|_| s.add_node(hg_core::NodeData::new(Label(0), 0)))
            .collect();
        for pair in [[nodes[0], nodes[1]], [nodes[1], nodes[2]]] {
            s.add_edge(HyperEdge::new(Label(0), pair.into_iter().collect()));
        }
    }
    for _ in 0..100 {
        tick(&mut a, &mut ea, &mut ra);
        tick(&mut b, &mut eb, &mut rb);
        assert_eq!(a.canonical_hash(), b.canonical_hash());
        assert_eq!(ea.rng_state_probe(), eb.rng_state_probe());
    }
    let ar = ra.close_window(&a, MetricsRow::default(), MetricsRow::default());
    let br = rb.close_window(&b, MetricsRow::default(), MetricsRow::default());
    assert_eq!(
        ar.encoded_meta_operation_traces,
        br.encoded_meta_operation_traces
    );
}

#[test]
fn live_activation_changes_the_real_world_table_and_trajectory() {
    let (mut a, mut ea, mut ra) = setup(&config(EreArm::A2, MetaOperation::Activate, 1.0, false));
    let (mut b, mut eb, mut rb) = setup(&config(EreArm::A1, MetaOperation::Activate, 1.0, false));
    for s in [&mut a, &mut b] {
        let nodes: Vec<_> = (0..3)
            .map(|_| s.add_node(hg_core::NodeData::new(Label(0), 0)))
            .collect();
        for pair in [[nodes[0], nodes[1]], [nodes[1], nodes[2]]] {
            s.add_edge(HyperEdge::new(Label(0), pair.into_iter().collect()));
        }
    }
    tick(&mut a, &mut ea, &mut ra);
    tick(&mut b, &mut eb, &mut rb);
    assert_eq!(a.canonical_hash(), b.canonical_hash());
    tick(&mut a, &mut ea, &mut ra);
    tick(&mut b, &mut eb, &mut rb);
    assert_eq!(ea.rules().len(), 3);
    assert_eq!(eb.rules().len(), 2);
    assert_ne!(a.canonical_hash(), b.canonical_hash());
}

#[test]
fn a3_remains_world_meta_visible_and_invalid_configs_do_not_seed() {
    let (mut s, mut e, mut r) = setup(&config(EreArm::A3, MetaOperation::Activate, 0.0, false));
    let a = s.add_node(hg_core::NodeData::new(Label(META_BASE + 99), 0));
    let b = s.add_node(hg_core::NodeData::new(Label(META_BASE + 99), 0));
    s.add_edge(HyperEdge::new(Label(0), [a, b].into_iter().collect()));
    tick(&mut s, &mut e, &mut r);
    let row = r.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert!(row.ordinary_meta_rewrite_events.unwrap() > 0);
    assert_eq!(row.encoded_meta_rewrite_events, Some(0));
    for case in 0..4 {
        let mut cfg = config(EreArm::A2, MetaOperation::Transpose, 1.0, false);
        match case {
            0 => cfg.vocabulary.as_mut().unwrap().protocol = "unknown".into(),
            1 => cfg.meta_seed_energy = 32.0,
            2 => cfg.vocabulary.as_mut().unwrap().instructions[0].fuel = 33.0,
            _ => cfg.limits.max_vars = 7,
        }
        let mut s = experiments::fixture(&core());
        let hash = s.canonical_hash();
        let mut e = Engine::new(
            vec![core().compile(&MetaLimits::default(), false).unwrap()],
            StepMode::MaxParallel,
            42,
            1,
        );
        assert!(EreRuntime::install(&cfg, &mut s, &mut e).is_err());
        assert_eq!(hash, s.canonical_hash());
    }
}
