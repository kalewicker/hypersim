//! G-R4 (plan §6): quarantine engagement. An adversarial fixture region per
//! validator V1..V9, all read through GraphRuleSource; every counter ≥ 1.
//! Also pins: counters PRESENT on clean runs (zeros reported, not absent).

use crate::graph_source::GraphRuleSource;
use crate::schema::{CandidatePatternEdge, CandidateRule};
use crate::validate::ValidatorId;
use hg_core::{HypergraphStore, Label, NodeData, PropertyValue};
use hg_rules::source::RuleSource;

/// Build a store with one adversarial region per validator, plus one clean
/// control rule that must install.
fn adversarial_world() -> HypergraphStore {
    let mut s = HypergraphStore::new();

    // Control: valid pair-collapse — installs.
    GraphRuleSource::install_candidates(
        &mut s,
        &[CandidateRule {
            priority: 0,
            lhs: vec![CandidatePatternEdge { slots: vec![0, 1] }],
            rhs: vec![CandidatePatternEdge { slots: vec![0] }],
            var_labels: vec![None, None],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        }],
    );

    // V1 — empty LHS (creation rules out of scope under E1/SB-R2).
    GraphRuleSource::install_candidates(
        &mut s,
        &[CandidateRule {
            priority: 0,
            lhs: vec![],
            rhs: vec![CandidatePatternEdge { slots: vec![0] }],
            var_labels: vec![None],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        }],
    );

    // V2 — arity-0 pattern edge in the LHS.
    GraphRuleSource::install_candidates(
        &mut s,
        &[CandidateRule {
            priority: 0,
            lhs: vec![
                CandidatePatternEdge { slots: vec![0] },
                CandidatePatternEdge { slots: vec![] },
            ],
            rhs: vec![],
            var_labels: vec![None],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        }],
    );

    // V3 — arity overflow: slot at position MAX_ARITY. The reader rejects
    // this at the walk level (ArityOverflow) — quarantine-by-rejection.
    {
        let arity = crate::schema::MAX_ARITY + 1;
        let slots: Vec<usize> = (0..arity).collect();
        GraphRuleSource::install_candidates(
            &mut s,
            &[CandidateRule {
                priority: 0,
                lhs: vec![CandidatePatternEdge { slots }],
                rhs: vec![],
                var_labels: vec![None; arity],
                guards: Vec::new(),
                var_guards: Vec::new(),
                effects: Vec::new(),
            }],
        );
    }

    // V4 — label constraint inside the meta band.
    GraphRuleSource::install_candidates(
        &mut s,
        &[CandidateRule {
            priority: 0,
            lhs: vec![CandidatePatternEdge { slots: vec![0] }],
            rhs: vec![],
            var_labels: vec![Some(crate::schema::L_RULE_ROOT)],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        }],
    );

    // V5 — malformed priority property (Float where Int required).
    {
        let roots = GraphRuleSource::install_candidates(
            &mut s,
            &[CandidateRule {
                priority: 0,
                lhs: vec![CandidatePatternEdge { slots: vec![0] }],
                rhs: vec![],
                var_labels: vec![None],
                guards: Vec::new(),
                var_guards: Vec::new(),
                effects: Vec::new(),
            }],
        );
        if let Some(root) = roots.first() {
            s.get_node_mut(*root)
                .expect("root present")
                .set_property(2, PropertyValue::Float(1.5));
        }
    }

    // V6 — repeated var within one pattern edge.
    GraphRuleSource::install_candidates(
        &mut s,
        &[CandidateRule {
            priority: 0,
            lhs: vec![CandidatePatternEdge { slots: vec![0, 0] }],
            rhs: vec![],
            var_labels: vec![None],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        }],
    );

    // V7 — node budget exceeded: enough pattern edges to blow the 256-node
    // footprint ceiling (nodes = 3 + PEs + vars + lits).
    {
        let n = crate::validate::REGION_NODE_BUDGET + 10;
        let pes: Vec<CandidatePatternEdge> = (0..n)
            .map(|_| CandidatePatternEdge { slots: vec![0] })
            .collect();
        GraphRuleSource::install_candidates(
            &mut s,
            &[CandidateRule {
                priority: 0,
                lhs: pes,
                rhs: vec![],
                var_labels: vec![None],
                guards: Vec::new(),
                var_guards: Vec::new(),
                effects: Vec::new(),
            }],
        );
    }

    // V8 — walk-level rejection: RULE_ROOT with no BECOMES edge at all.
    {
        let orphan = s.add_node(NodeData::new(Label(crate::schema::L_RULE_ROOT), 0));
        s.get_node_mut(orphan)
            .expect("fresh node present")
            .set_property(2, PropertyValue::Int(0));
    }

    // V9 — edge budget exceeded WITHOUT tripping V7: ~100 max-arity edges
    // (nodes ≈ 3+100+8 = 111 < 256; edges ≈ 100×(2 slots… actually
    // 1 MEMBER + 8 SLOT + 1 HASLABEL-free) ≈ 909 > 512).
    {
        let n = 100usize;
        let pes: Vec<CandidatePatternEdge> = (0..n)
            .map(|_| CandidatePatternEdge {
                slots: (0..crate::schema::MAX_ARITY).collect(),
            })
            .collect();
        GraphRuleSource::install_candidates(
            &mut s,
            &[CandidateRule {
                priority: 0,
                lhs: pes,
                rhs: vec![],
                var_labels: vec![None; crate::schema::MAX_ARITY],
                guards: Vec::new(),
                var_guards: Vec::new(),
                effects: Vec::new(),
            }],
        );
    }

    s
}

#[test]
fn gr4_every_validator_engages_through_the_source() {
    let s = adversarial_world();
    let mut src = GraphRuleSource::new();
    let snap = s.snapshot();
    let d = src.rules(&snap, 0);

    // Exactly the control installs; everything else is quarantined or rejected.
    assert_eq!(d.rules.len(), 1, "only the clean control rule installs");
    assert!(d.quarantined >= 7);
    assert!(d.read_rejected >= 1);
    for v in ValidatorId::ALL {
        let c = src.counters.get(v);
        assert!(
            c >= 1,
            "G-R4: validator {} never engaged (counter 0) — gate fails",
            v.counter_key()
        );
    }
}

#[test]
fn gr4_clean_run_reports_all_counters_present_including_zeros() {
    // A run with only the control region: all counters present and zero —
    // a zero here is a measurement, not an absence (plan §6 G-R4).
    let mut s = HypergraphStore::new();
    GraphRuleSource::install_candidates(
        &mut s,
        &[CandidateRule {
            priority: 0,
            lhs: vec![CandidatePatternEdge { slots: vec![0, 1] }],
            rhs: vec![CandidatePatternEdge { slots: vec![0] }],
            var_labels: vec![None, None],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        }],
    );
    let mut src = GraphRuleSource::new();
    let snap = s.snapshot();
    let d = src.rules(&snap, 0);

    assert_eq!(d.rules.len(), 1);
    assert_eq!(d.quarantined, 0);
    assert_eq!(d.read_rejected, 0);
    for v in ValidatorId::ALL {
        assert_eq!(
            src.counters.get(v),
            0,
            "{} present but must be zero on a clean run",
            v.counter_key()
        );
    }
}
