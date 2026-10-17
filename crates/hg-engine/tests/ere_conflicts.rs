use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
use hg_engine::{conflict_set, find_bindings};
use hg_rules::{compile_rule, Rule, META_BASE};
use std::collections::HashSet;

#[test]
fn disjoint_meta_bindings_preserve_dependency_invariant_with_cleanup_frontier() {
    for mixed in [false, true] {
        let mut s = HypergraphStore::new();
        let ids: Vec<_> = (0..8).map(|i| s.add_node(NodeData::new(
            Label(if mixed && i % 2 == 0 { 0 } else { META_BASE + 5 }), 0))).collect();
        for pair in ids.windows(2) { s.add_edge(HyperEdge::new(Label(META_BASE + 0x50), pair.iter().copied().collect())); }
        let mut raw = Rule::default(); raw.name = "meta_cleanup".into();
        raw.lhs.nodes = vec!["a".into(), "b".into()]; raw.lhs.edges = vec![raw.lhs.nodes.clone()];
        raw.rhs.nodes = vec!["b".into()];
        let rule = compile_rule(&raw).unwrap();
        let bindings = find_bindings(&s.snapshot(), &rule, 0); let mut engaged = 0;
        for a in &bindings {
            for b in &bindings {
                let nodes: HashSet<_> = a.bound_nodes.iter().copied().collect();
                if b.bound_nodes.iter().any(|n| nodes.contains(n)) { continue; }
                engaged += 1;
                let effects = conflict_set(&s, &rule, a);
                let deps: HashSet<_> = b.matched_edges.iter().copied().collect();
                assert!(effects.edges.is_disjoint(&deps));
                for &v in &rule.template.lhs_deleted { assert!(!b.bound_nodes.contains(&a.bound_nodes[v])); }
            }
        }
        assert!(engaged > 0);
    }
}
