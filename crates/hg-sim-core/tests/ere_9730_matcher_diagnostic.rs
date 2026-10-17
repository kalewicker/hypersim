//! Engineering only: counted matching over persisted/small read-only snapshots.
//! No Engine construction/step, rewrites, RNG, trajectory or gate admission.
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
// Historical engineering control must remain on the original work model
// even when the opt-in candidate build feature is enabled.
use hg_engine::find_bindings_counted_legacy as find_bindings_counted;
use hg_rules::{CompiledRule, MatchPattern, PatternEdge};
use serde_json::{json, Value};
use smallvec::SmallVec;

const FIXTURE: &str = include_str!("fixtures/ere_9730_matcher_snapshot.json");

fn fixture() -> (Value, HypergraphStore, Vec<CompiledRule>) {
    let f: Value = serde_json::from_str(FIXTURE).unwrap();
    assert_eq!(f["development_only"], true);
    assert_eq!(f["gate_pass"], false);
    assert_eq!(f["training_admitted"], false);
    let s: HypergraphStore = serde_json::from_value(f["store_wire"].clone()).unwrap();
    let rs: Vec<CompiledRule> = serde_json::from_value(f["table"].clone()).unwrap();
    assert!(s.metadata_allocation_isolated());
    assert_eq!(s.step(), 91);
    // Audit every live handle, typed value, endpoint, and canonical index,
    // not just world cardinalities. Vacant allocator history is not resumed.
    for row in f["inventory"]["nodes"].as_array().unwrap() {
        let key = row[0].as_str().unwrap();
        let (id, data) = s.nodes().find(|(id, _)| format!("{id:?}") == key).unwrap();
        // Canonical Python JSON writes scalar f32 energy 0.0 as 0. Keep
        // tagged property variants exact; compare the explicitly-f32 field
        // numerically rather than conflating JSON number token spelling.
        let mut actual = serde_json::to_value(data).unwrap();
        let expected = &row[1]["data"];
        assert_eq!(
            actual["energy"].as_f64().unwrap().to_bits(),
            expected["energy"].as_f64().unwrap().to_bits()
        );
        actual["energy"] = expected["energy"].clone();
        assert_eq!(actual, *expected);
        let incident: Vec<_> = s
            .snapshot()
            .edges_incident_to(id)
            .iter()
            .map(|e| format!("{e:?}"))
            .collect();
        assert_eq!(json!(incident), row[1]["incident"]);
    }
    assert_eq!(
        s.node_count(),
        f["inventory"]["nodes"].as_array().unwrap().len()
    );
    for row in f["inventory"]["edges"].as_array().unwrap() {
        let key = row[0].as_str().unwrap();
        let (_, data) = s.edges().find(|(id, _)| format!("{id:?}") == key).unwrap();
        assert_eq!(serde_json::to_value(data).unwrap(), row[1]);
    }
    assert_eq!(
        s.edge_count(),
        f["inventory"]["edges"].as_array().unwrap().len()
    );
    for row in f["inventory"]["label_indexes"].as_array().unwrap() {
        let ids: Vec<_> = s
            .snapshot()
            .edges_with_label(Label(row[0].as_u64().unwrap() as u32))
            .iter()
            .map(|id| format!("{id:?}"))
            .collect();
        assert_eq!(json!(ids), row[1]);
    }
    (f, s, rs)
}

fn bounded(mut r: CompiledRule, work: usize, results: usize) -> CompiledRule {
    assert!(work <= 400_000 && results <= 50_000);
    r.pattern.match_work_limit = Some(work);
    r.pattern.match_result_limit = Some(results);
    r
}

#[test]
fn feasibility_revision_completes_only_the_recorded_snapshot_pool() {
    use hg_engine::{find_bindings_counted_feasible, find_bindings_counted_legacy};
    let (_, s, rs) = fixture();
    let before = serde_json::to_value(&s).unwrap();
    let table_before = serde_json::to_value(&rs).unwrap();
    let (core, core_stats) =
        find_bindings_counted_feasible(&s.snapshot(), &bounded(rs[0].clone(), 400_000, 50_000), 0);
    let (control, control_stats) =
        find_bindings_counted_legacy(&s.snapshot(), &bounded(rs[0].clone(), 400_000, 50_000), 0);
    let keys = |bindings: &[hg_observe::Binding]| {
        bindings
            .iter()
            .map(|b| {
                (
                    b.rule_id,
                    b.priority,
                    b.bound_nodes.clone(),
                    b.matched_edges.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&core), keys(&control));
    assert_eq!(core_stats, control_stats);
    assert_eq!((core_stats.enumeration_work, core.len()), (8462, 372));
    let (proposal, stats) = find_bindings_counted_feasible(
        &s.snapshot(),
        &bounded(
            rs[1].clone(),
            400_000 - core_stats.enumeration_work,
            50_000 - core.len(),
        ),
        1,
    );
    assert!(proposal.is_empty() && !stats.enumeration_truncated);
    assert_eq!(stats.property_guards, vec![(0, 0)]);
    assert!(stats.enumeration_work > 0 && stats.enumeration_work < 10_000);
    assert_eq!(serde_json::to_value(&s).unwrap(), before);
    assert_eq!(serde_json::to_value(&rs).unwrap(), table_before);
    println!(
        "ERE_FEASIBILITY_V1 {}",
        json!({"development_only":true,"gate_pass":false,"training_admitted":false,"scope":"matcher_snapshot_only","engine_steps":0,"feature_enabled":cfg!(feature="repeated-edge-feasibility"),"candidate_work_model":"repeated-self-pair-feasibility-v1","build_dispatch_work_model":hg_engine::MATCHER_WORK_MODEL,"core_work":core_stats.enumeration_work,"core_results":core.len(),"proposal_work":stats.enumeration_work,"proposal_results":0,"truncated":false,"total_work":core_stats.enumeration_work+stats.enumeration_work,"work_cap":400000,"result_cap":50000})
    );
}

#[test]
fn recorded_cap_is_reproduced_without_an_engine_step() {
    let (f, s, rs) = fixture();
    assert_eq!(rs.len(), 2);
    assert_eq!(rs[0].pattern.edges[0].var_ixs, [0, 1, 2]);
    assert_eq!(
        rs[1]
            .pattern
            .edges
            .iter()
            .map(|e| e.var_ixs.clone())
            .collect::<Vec<_>>(),
        vec![vec![0, 1, 2], vec![3, 3]]
    );
    // No variables in edge 2 were assigned by edge 1: every edge takes the
    // global numeric arena path, so canonical incidence-order reconstruction
    // does not affect this exact finite-work reproduction.
    assert!(rs
        .iter()
        .all(|r| r.pattern.edges.iter().all(|e| e.label.is_none())));
    assert_eq!(
        s.nodes().filter(|(_, n)| n.label.0 < 0xFFF00000).count(),
        159
    );
    assert_eq!(
        s.edges().filter(|(_, e)| e.label.0 < 0xFFF00000).count(),
        279
    );
    let before = serde_json::to_value(&s).unwrap();
    let mut work = 0;
    let mut results = 0;
    for (id, r) in rs.into_iter().enumerate() {
        let rule = bounded(r, 400_000 - work, 50_000 - results);
        let (bindings, stats) = find_bindings_counted(&s.snapshot(), &rule, id);
        let expected = &f["recorded_binding_step"]["usage"]["rules"][id];
        assert_eq!(
            stats.enumeration_work as u64,
            expected["work"].as_u64().unwrap()
        );
        assert_eq!(bindings.len() as u64, expected["results"].as_u64().unwrap());
        assert_eq!(
            stats.enumeration_truncated,
            expected["truncated"].as_bool().unwrap()
        );
        work += stats.enumeration_work;
        results += bindings.len();
    }
    assert_eq!((work, results), (400_000, 372));
    assert_eq!(serde_json::to_value(&s).unwrap(), before);
    println!(
        "ERE_9730_DIAGNOSTIC {}",
        json!({"development_only":true,"gate_pass":false,"training_admitted":false,"work":work,"results":results,"exact_recorded_cap_reproduced":true,"engine_steps":0})
    );
}

#[test]
fn selective_first_edge_is_only_a_diagnostic_not_a_kernel_change() {
    let (_, s, rs) = fixture();
    let mut proposal = rs[1].clone();
    let loops = s
        .edges()
        .filter(|(_, e)| {
            e.nodes.len() == 2
                && e.nodes[0] == e.nodes[1]
                && s.get_node(e.nodes[0]).unwrap().label.0 < 0xFFF00000
        })
        .count();
    assert_eq!(loops, 0);
    proposal.pattern.edges.swap(0, 1);
    let before = serde_json::to_value(&s).unwrap();
    let (bindings, stats) =
        find_bindings_counted(&s.snapshot(), &bounded(proposal, 391_538, 49_628), 1);
    assert!(bindings.is_empty());
    assert!(!stats.enumeration_truncated);
    assert!(stats.enumeration_work < 10_000);
    assert_eq!(serde_json::to_value(&s).unwrap(), before);
    println!(
        "ERE_9730_DIAGNOSTIC {}",
        json!({"development_only":true,"gate_pass":false,"training_admitted":false,"ordinary_self_pairs":loops,"diagnostic_reordered_work":stats.enumeration_work,"results":0,"truncated":false,"installed":false})
    );
}

fn pattern(n: usize, edges: &[&[usize]]) -> MatchPattern {
    MatchPattern {
        n_vars: n,
        node_label_constraints: vec![None; n],
        node_label_ceiling: Some(0xFFF00000),
        edges: edges
            .iter()
            .map(|e| PatternEdge {
                var_ixs: e.to_vec(),
                label: None,
                ordered: false,
            })
            .collect(),
        match_work_limit: Some(400_000),
        match_result_limit: Some(50_000),
        ..MatchPattern::default()
    }
}

#[test]
fn repeated_vars_require_repeated_endpoints_and_distinct_pattern_edges() {
    let (_, _, rs) = fixture();
    let mut s = HypergraphStore::new();
    let a = s.add_node(NodeData::new(Label(0), 0));
    let b = s.add_node(NodeData::new(Label(0), 0));
    for (self_pairs, expected) in [(0, 0), (1, 0), (2, 2)] {
        if self_pairs == 0 {
            s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
        } else {
            s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, a])));
        }
        let mut r = rs[1].clone();
        r.pattern = pattern(1, &[&[0, 0], &[0, 0]]);
        let (bindings, stats) = find_bindings_counted(&s.snapshot(), &r, 1);
        assert!(!stats.enumeration_truncated);
        assert_eq!(bindings.len(), expected);
        for binding in bindings {
            assert_ne!(binding.matched_edges[0], binding.matched_edges[1]);
        }
    }
}

#[test]
fn small_positive_reordering_preserves_variables_and_translated_edge_identity() {
    let (_, _, rs) = fixture();
    let mut s = HypergraphStore::new();
    let nodes: Vec<_> = (0..4)
        .map(|_| {
            let mut n = NodeData::new(Label(0), 0);
            n.set_property(1, PropertyValue::Int(1));
            s.add_node(n)
        })
        .collect();
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&nodes[..3])));
    s.add_edge(HyperEdge::new(
        Label(0),
        SmallVec::from_slice(&[nodes[3], nodes[3]]),
    ));
    let original = bounded(rs[1].clone(), 400_000, 50_000);
    let mut reordered = original.clone();
    reordered.pattern.edges.swap(0, 1);
    let (a, sa) = find_bindings_counted(&s.snapshot(), &original, 1);
    let (b, sb) = find_bindings_counted(&s.snapshot(), &reordered, 1);
    assert!(!sa.enumeration_truncated && !sb.enumeration_truncated);
    let keys = |bs: Vec<hg_observe::Binding>, swapped: bool| {
        let mut out: Vec<_> = bs
            .into_iter()
            .map(|mut binding| {
                if swapped {
                    binding.matched_edges.swap(0, 1);
                }
                format!(
                    "{}:{}:{:?}:{:?}",
                    binding.rule_id, binding.priority, binding.bound_nodes, binding.matched_edges
                )
            })
            .collect();
        out.sort();
        out
    };
    let ka = keys(a, false);
    assert_eq!(ka.len(), 6);
    assert_eq!(ka, keys(b, true));
    // Equal complete binding sets do NOT prove finite-work prefixes,
    // counters or engine RNG/trajectory equivalence.
}
