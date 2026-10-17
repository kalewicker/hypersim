use hg_core::{HyperEdge, HypergraphStore, Label, PropertyValue};
use hg_meta::*;
use hg_rules::{EpochPolicy, RuleSetDelta, RuleSource};

#[test]
fn codec_is_explicit_bounded_and_non_mutating_on_rejection() {
    for operation in [
        MetaOperation::Activate,
        MetaOperation::Deactivate,
        MetaOperation::Transpose,
    ] {
        let mut s = HypergraphStore::new();
        let root = encode_instruction(
            &mut s,
            &MetaInstructionSeed {
                operation,
                fuel: 32.0,
            },
        )
        .unwrap();
        assert_eq!(
            read_instruction(&s.snapshot(), root).unwrap().operation,
            operation
        );
        for fuel in [f64::NAN, f64::INFINITY, -1.0, 33.0] {
            let hash = s.canonical_hash();
            assert!(encode_instruction(&mut s, &MetaInstructionSeed { operation, fuel }).is_err());
            assert_eq!(hash, s.canonical_hash());
        }
        for (key, value) in [
            (K_ENCODING, PropertyValue::Int(99)),
            (K_OPCODE, PropertyValue::Int(99)),
            (K_FUEL, PropertyValue::Int(1)),
        ] {
            let mut bad = s.clone();
            bad.get_node_mut(root).unwrap().set_property(key, value);
            assert!(read_instruction(&bad.snapshot(), root).is_err());
        }
        s.add_edge(HyperEdge::new(Label(0), [root, root].into_iter().collect()));
        assert!(read_instruction(&s.snapshot(), root).is_err());
    }
    assert_ne!(
        MetaOperation::Activate.execution_bytes(),
        MetaOperation::Deactivate.execution_bytes()
    );
    assert!(compile_operation(
        MetaOperation::Transpose,
        &MetaLimits {
            max_vars: 7,
            ..MetaLimits::default()
        }
    )
    .is_err());
    assert!(compile_operation(
        MetaOperation::Transpose,
        &MetaLimits {
            max_arity: 2,
            ..MetaLimits::default()
        }
    )
    .is_err());
}

#[test]
fn source_opt_in_dedup_fuel_identity_and_quarantine_retirement() {
    let mut s = HypergraphStore::new();
    let mut roots = Vec::new();
    for operation in [
        MetaOperation::Activate,
        MetaOperation::Deactivate,
        MetaOperation::Transpose,
        MetaOperation::Transpose,
    ] {
        roots.push(
            encode_instruction(
                &mut s,
                &MetaInstructionSeed {
                    operation,
                    fuel: 32.0,
                },
            )
            .unwrap(),
        );
    }
    let mut source = GraphRuleSource::new(
        EpochPolicy::Fixed(1.try_into().unwrap()),
        MetaLimits::default(),
        true,
    );
    let hash = s.canonical_hash();
    match source.rules(&s.snapshot(), 0) {
        RuleSetDelta::Replace { rules, .. } => assert!(rules.is_empty()),
        _ => panic!(),
    }
    source.vocabulary_enabled = true;
    match source.rules(&s.snapshot(), 1) {
        RuleSetDelta::Replace { rules, report } => {
            assert_eq!(rules.len(), 3);
            assert_eq!(
                report.encoded_meta_operations,
                ["Activate", "Deactivate", "Transpose"]
            );
            assert_eq!(report.validated_regions, 4);
            assert!(report.classes.is_empty());
        }
        _ => panic!(),
    }
    assert_eq!(hash, s.canonical_hash(), "compilation does not edit graph");
    s.get_node_mut(roots[0])
        .unwrap()
        .set_property(K_FUEL, PropertyValue::Float(0.0));
    match source.rules(&s.snapshot(), 2) {
        RuleSetDelta::Replace { report, .. } => assert!(!report.changed),
        _ => panic!(),
    }
    s.get_node_mut(roots[0])
        .unwrap()
        .set_property(K_OPCODE, PropertyValue::Int(99));
    match source.rules(&s.snapshot(), 3) {
        RuleSetDelta::Replace { rules, report } => {
            assert_eq!(rules.len(), 2);
            assert_eq!(report.quarantined, 1);
            assert_eq!(report.retired, 1);
            assert!(report.changed);
        }
        _ => panic!(),
    }
}
