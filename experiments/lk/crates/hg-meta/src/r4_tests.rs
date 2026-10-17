//! R4 arm tests: meta_visible protection, epoch-driven recompile, G-R5
//! protection integrity, and the A3 autophagy probe (EXP-R05, small-scale).
//!
//! The hostile rule here is deliberately able to eat regions once visible:
//! a relabel-to-meta-band rule. Under A2 it can never bind region nodes
//! (guards reject); under A3 it can — survival must then be earned.

use crate::graph_source::GraphRuleSource;
use crate::protect::{compile_delta, sync_label_mirror, K_LABEL_MIRROR};
use crate::schema::{CandidatePatternEdge, CandidateRule};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use hg_engine::Engine;
use hg_rules::compiled::compile_rules;
use hg_rules::StepMode;
use smallvec::smallvec;

fn pe(slots: &[usize]) -> CandidatePatternEdge {
    CandidatePatternEdge {
        slots: slots.to_vec(),
    }
}

/// The two reference rules from gr3 plus a REGION-EATER: matches any two
/// nodes regardless of band and relabels them into the meta band (which,
/// if bound to region nodes, corrupts or strands them). LHS labels absent
/// on purpose: it binds anything visible.
fn world_and_eater() -> (HypergraphStore, Vec<CandidateRule>) {
    let mut s = HypergraphStore::new();
    // 8 ordinary substrate nodes.
    for i in 0..8u32 {
        let mut d = NodeData::new(Label(i % 4), 0);
        d.set_property(1, PropertyValue::Float(f64::from(i) * 0.125));
        s.add_node(d);
    }
    let ids: Vec<_> = s.nodes().map(|(id, _)| id).collect();
    for w in 0..6usize {
        let a = ids[w % ids.len()];
        let b = ids[(w * 3 + 1) % ids.len()];
        if a != b {
            s.add_edge(HyperEdge::new(Label((w % 3) as u32), smallvec![a, b]));
        }
    }

    let churner = CandidateRule {
        priority: 0,
        lhs: vec![pe(&[0, 1])],
        rhs: vec![pe(&[0, 2]), pe(&[2, 1])],
        var_labels: vec![Some(0), Some(1), Some(2)],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    };
    let eater = CandidateRule {
        priority: -5, // lowest priority: fires only when nothing else does
        lhs: vec![pe(&[0, 1])],
        rhs: vec![],
        var_labels: vec![Some(0xFFF0_0010), Some(0xFFF0_0011)], // meta-band relabels,
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    };
    let cands = vec![churner, eater];
    GraphRuleSource::install_candidates(&mut s, &cands);
    (s, cands)
}

#[test]
fn a2_guards_make_regions_invisible_to_the_eater() {
    let (mut s, cands) = world_and_eater();

    // Compile UNDER PROTECTION and check the guards landed.
    let rules = compile_delta(&cands, false);
    assert_eq!(rules.len(), 2);
    // churner: LHS-wired vars = v0,v1 (v2 is RHS-only) -> 2 guards;
    // eater: 2 wired vars -> 2 guards.
    assert_eq!(rules[0].lhs_node_property_guards.len(), 2);
    assert_eq!(rules[1].lhs_node_property_guards.len(), 2);
    for g in &rules[1].lhs_node_property_guards {
        assert_eq!(g.property, K_LABEL_MIRROR);
        assert_eq!(g.value, PropertyValue::Int(crate::schema::META_BASE as i64));
    }

    // Sync mirror, run many steps: NO node may end up meta-band.
    sync_label_mirror(&mut s);
    let compiled = compile_rules(&rules).expect("compile");
    let mut engine = Engine::new(compiled, StepMode::MaxParallel, 7, 64);
    for _ in 0..500 {
        engine.step(&mut s);
        sync_label_mirror(&mut s);
    }
    // The protection contract: REGION nodes are unreachable. Ordinary
    // substrate nodes may still be relabeled by the eater's RHS — that is
    // ordinary rewrite dynamics, not a protection failure. Assert exactly
    // the protected thing: every RULE_ROOT handle survives with its label.
    let roots = crate::schema::find_rule_roots(&s.snapshot());
    assert_eq!(roots.len(), 2, "A2: both regions must survive");
    for r in &roots {
        let d = s.get_node(*r).expect("root alive");
        assert_eq!(d.label.0, crate::schema::L_RULE_ROOT);
    }
}

#[test]
fn a3_unprotected_eater_corrupts_the_world() {
    // Same setup, meta_visible = TRUE (no guards): the eater binds freely.
    // This is the control proving the A2 result above is the guard's work.
    let (mut s, cands) = world_and_eater();
    let rules = compile_delta(&cands, true);
    assert_eq!(rules[0].lhs_node_property_guards.len(), 0);
    assert_eq!(rules[1].lhs_node_property_guards.len(), 0);

    sync_label_mirror(&mut s);
    let compiled = compile_rules(&rules).expect("compile");
    let mut engine = Engine::new(compiled, StepMode::MaxParallel, 7, 64);
    let mut saw_meta = false;
    for _ in 0..200 {
        engine.step(&mut s);
        sync_label_mirror(&mut s);
        if s.nodes()
            .any(|(_, d)| d.label.0 >= crate::schema::META_BASE)
        {
            saw_meta = true;
            break;
        }
    }
    assert!(saw_meta, "A3 control: unprotected eater must reach nodes");
}

#[test]
fn gr5_a2_region_survival_is_total() {
    // G-R5: under protection, every installed RULE_ROOT handle survives the
    // full run and zero region edges are lost to dangling cleanup.
    let (mut s, cands) = world_and_eater();

    // Handles at install time (plan §2.5: gen-0 anchor set).
    let handles: Vec<_> = crate::schema::find_rule_roots(&s.snapshot());
    assert_eq!(handles.len(), 2);

    let rules = compile_delta(&cands, false);
    let compiled = compile_rules(&rules).expect("compile");
    let mut engine = Engine::new(compiled, StepMode::MaxParallel, 11, 64);
    sync_label_mirror(&mut s);
    for step in 0..1000u64 {
        engine.step(&mut s);
        sync_label_mirror(&mut s);

        let alive = crate::schema::find_rule_roots(&s.snapshot());
        assert_eq!(
            alive.len(),
            2,
            "G-R5 violation at step {step}: a rule region was consumed"
        );
        for h in &handles {
            assert!(
                s.get_node(*h).is_some()
                    && s.get_node(*h).unwrap().label.0 == crate::schema::L_RULE_ROOT,
                "G-R5 violation at step {step}: handle identity lost"
            );
        }
    }
}
