//! Validator tests: one per V-class (G-R4 engagement at unit level), plus
//! ordering determinism and budget-boundary behavior.

use crate::schema::{CandidatePatternEdge, CandidateRule, RegionFootprint};
use crate::validate::{
    validate, EngagementCounters, PriorityObservation, ValidatorId, Verdict, REGION_EDGE_BUDGET,
    REGION_NODE_BUDGET,
};

fn rule(lhs: Vec<Vec<usize>>, rhs: Vec<Vec<usize>>, labels: Vec<Option<u32>>) -> CandidateRule {
    CandidateRule {
        priority: 0,
        lhs: lhs
            .into_iter()
            .map(|slots| CandidatePatternEdge { slots })
            .collect(),
        rhs: rhs
            .into_iter()
            .map(|slots| CandidatePatternEdge { slots })
            .collect(),
        var_labels: labels,
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    }
}

const OK_PRIORITY: PriorityObservation = PriorityObservation { malformed: false };
const BAD_PRIORITY: PriorityObservation = PriorityObservation { malformed: true };

fn footprint(nodes: usize, edges: usize) -> RegionFootprint {
    RegionFootprint { nodes, edges }
}

#[test]
fn v1_fires_on_empty_lhs() {
    let r = rule(vec![], vec![vec![0]], vec![Some(0)]);
    assert_eq!(
        validate(&r, &footprint(8, 6), &OK_PRIORITY),
        Verdict::Quarantined(ValidatorId::V1)
    );
}

#[test]
fn v2_fires_on_arity_zero_edge() {
    let r = rule(vec![vec![0], vec![]], vec![vec![0]], vec![Some(0)]);
    assert_eq!(
        validate(&r, &footprint(10, 7), &OK_PRIORITY),
        Verdict::Quarantined(ValidatorId::V2)
    );
}

#[test]
fn v3_fires_over_max_arity() {
    // MAX_ARITY is 8; a 9-slot edge trips V3.
    let big: Vec<usize> = (0..9).collect();
    let r = rule(vec![big], vec![vec![0]], (0..9).map(Some).collect());
    assert_eq!(
        validate(&r, &footprint(20, 14), &OK_PRIORITY),
        Verdict::Quarantined(ValidatorId::V3)
    );
}

#[test]
fn v4_fires_on_meta_band_label() {
    let r = rule(vec![vec![0]], vec![vec![0]], vec![Some(0xFFF0_0000)]);
    assert_eq!(
        validate(&r, &footprint(8, 5), &OK_PRIORITY),
        Verdict::Quarantined(ValidatorId::V4)
    );
}

#[test]
fn v5_fires_on_malformed_priority() {
    let r = rule(vec![vec![0]], vec![vec![0]], vec![None]);
    assert_eq!(
        validate(&r, &footprint(8, 5), &BAD_PRIORITY),
        Verdict::Quarantined(ValidatorId::V5)
    );
}

#[test]
fn v6_fires_on_repeated_var_within_one_edge() {
    let r = rule(vec![vec![0, 0, 1]], vec![vec![1]], vec![None, None]);
    assert_eq!(
        validate(&r, &footprint(10, 7), &OK_PRIORITY),
        Verdict::Quarantined(ValidatorId::V6)
    );
}

#[test]
fn v7_fires_over_node_budget() {
    let r = rule(vec![vec![0, 1]], vec![vec![2]], vec![None, None, None]);
    assert_eq!(
        validate(&r, &footprint(REGION_NODE_BUDGET + 1, 10), &OK_PRIORITY),
        Verdict::Quarantined(ValidatorId::V7)
    );
}

#[test]
fn v9_fires_over_edge_budget() {
    let r = rule(vec![vec![0, 1]], vec![vec![2]], vec![None, None, None]);
    assert_eq!(
        validate(&r, &footprint(12, REGION_EDGE_BUDGET + 1), &OK_PRIORITY),
        Verdict::Quarantined(ValidatorId::V9)
    );
}

#[test]
fn budgets_exactly_at_ceiling_are_valid() {
    let r = rule(vec![vec![0, 1]], vec![vec![2]], vec![None, None, None]);
    assert_eq!(
        validate(
            &r,
            &footprint(REGION_NODE_BUDGET, REGION_EDGE_BUDGET),
            &OK_PRIORITY
        ),
        Verdict::Valid,
        "ceiling is inclusive; only strictly-greater footprints quarantine"
    );
}

#[test]
fn first_failure_wins_in_v_order() {
    // Region violates V6 (repeated var) AND V7 (node budget): V6 must be
    // reported because it precedes V7 in the deterministic order.
    let r = rule(vec![vec![0, 0]], vec![vec![1]], vec![None, None]);
    assert_eq!(
        validate(&r, &footprint(REGION_NODE_BUDGET + 5, 10), &OK_PRIORITY),
        Verdict::Quarantined(ValidatorId::V6)
    );
    // And an LHS-empty + meta-label region reports V1, not V4.
    let r2 = rule(vec![], vec![vec![0]], vec![Some(0xFFFF_FFFF)]);
    assert_eq!(
        validate(&r2, &footprint(8, 5), &OK_PRIORITY),
        Verdict::Quarantined(ValidatorId::V1)
    );
}

#[test]
fn healthy_triangle_rule_is_valid() {
    // The §3.4 worked example shape: LHS [a,b,c], RHS [a,b,d],[b,c,d],[a,c,d].
    let r = rule(
        vec![vec![0, 1, 2]],
        vec![vec![0, 1, 3], vec![1, 2, 3], vec![0, 2, 3]],
        vec![None; 4],
    );
    // Footprint from the reader for that region: root+sides+4 PEs+4 vars =
    // 11 nodes; BECOMES+4 MEMBERs+11 SLOTs = 16 edges.
    assert_eq!(
        validate(&r, &footprint(11, 16), &OK_PRIORITY),
        Verdict::Valid
    );
}

#[test]
fn counters_record_and_read_back() {
    let mut c = EngagementCounters::default();
    assert_eq!(
        c.get(ValidatorId::V4),
        0,
        "zeros are measurements, not absences"
    );
    c.record(ValidatorId::V4);
    c.record(ValidatorId::V4);
    c.record(ValidatorId::V1);
    assert_eq!(c.get(ValidatorId::V4), 2);
    assert_eq!(c.get(ValidatorId::V1), 1);
    assert_eq!(c.get(ValidatorId::V9), 0);
    // Counter keys are stable strings for metrics rows.
    assert_eq!(ValidatorId::V4.counter_key(), "v4_meta_label");
}
