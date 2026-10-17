//! Shared fixtures for the separately versioned sealed-repair controls.
use hg_core::{EdgeId, HyperEdge, HypergraphStore, Label, NodeId, PropertyOp, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_output::{SnapshotReader, SnapshotWriter};
use hg_rules::{compile_rule, EpochPolicy, MatchBudgetPolicy, NodePropertyGuard, Rule, StepMode};
use rand::SeedableRng;
use serde_json::{json, Value};
use std::path::Path;

pub const SEEDS: [u64; 4] = [984927043585, 984927043586, 984927043587, 984927043588];
const MARKER: u32 = 0xE5520003;

pub fn candidate() -> CandidateRule {
    CandidateRule {
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
    }
}
pub fn fixture() -> (HypergraphStore, NodeId) {
    let mut s = HypergraphStore::new();
    let c = candidate();
    let root = encode_witness_family(&mut s, &c, &c, false, &MetaLimits::default())
        .unwrap()
        .root;
    (s, root)
}
pub fn engine(rules: Vec<hg_rules::CompiledRule>, seed: u64, budget: usize) -> Engine {
    let mut e = Engine::new(rules, StepMode::MaxParallel, seed, 16);
    e.set_match_budget_policy(budget, budget, MatchBudgetPolicy::Pooled);
    e
}
pub fn source_engine(seed: u64, budget: usize, extended: bool) -> Engine {
    let mut e = engine(vec![], seed, budget);
    let mut source = GraphRuleSource::new(
        EpochPolicy::Fixed(1.try_into().unwrap()),
        MetaLimits::default(),
        true,
    );
    source.vocabulary_enabled = true;
    source.witness_repair_enabled = true;
    source.sealed_repair_enabled = extended;
    e.set_rule_source(Box::new(source));
    e
}
pub fn instruction(s: &mut HypergraphStore, operation: MetaOperation, fuel: f64) -> NodeId {
    encode_instruction(s, &MetaInstructionSeed { operation, fuel }).unwrap()
}
pub fn fuel(s: &HypergraphStore, id: NodeId) -> f64 {
    read_instruction(&s.snapshot(), id).unwrap().fuel
}
pub fn decoded(s: &HypergraphStore, root: NodeId) -> Result<RuleProgram, Validator> {
    let before = bincode::serialize(s).unwrap();
    let result = read_program(&s.snapshot(), root, &MetaLimits::default());
    assert_eq!(bincode::serialize(s).unwrap(), before);
    result
}
pub fn replace(s: &mut HypergraphStore, id: EdgeId, edge: HyperEdge) -> EdgeId {
    s.remove_edge(id);
    s.add_edge(edge)
}
pub fn seal(s: &mut HypergraphStore, payload: EdgeId, instruction: NodeId, seed: u64) -> EdgeId {
    let original = s.get_edge(payload).unwrap().clone();
    for mut rule in compile_operations(MetaOperation::Seal, &MetaLimits::default()).unwrap() {
        rule.pattern.match_work_limit = Some(50000);
        rule.pattern.match_result_limit = Some(50000);
        let (bindings, usage) = hg_engine::find_bindings_counted(&s.snapshot(), &rule, 0);
        assert!(!usage.enumeration_truncated);
        if let Some(binding) = bindings
            .into_iter()
            .find(|b| b.matched_edges.contains(&payload))
        {
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
            let before = rng.clone();
            hg_engine::apply_binding(s, &rule, &binding, &mut rng);
            assert_eq!(rng, before);
            assert_eq!(fuel(s, instruction), 0.0);
            return s
                .edges()
                .find(|(_, e)| {
                    e.label == original.label
                        && e.arity() == 3
                        && e.nodes[..2] == original.nodes[..]
                })
                .unwrap()
                .0;
        }
    }
    panic!("target has no native Seal binding");
}

/// All runtime attacks are ordinary native rewrites; only setup uses markers.
/// Multiple lesions are emitted in ONE matched native attack, with no stepping hook.
pub fn attack(s: &mut HypergraphStore, targets: &[(EdgeId, &str)]) -> hg_rules::CompiledRule {
    let edges: Vec<_> = targets
        .iter()
        .map(|(id, _)| s.get_edge(*id).unwrap().clone())
        .collect();
    let mut unique = vec![];
    for e in &edges {
        for &n in &e.nodes {
            if !unique.contains(&n) {
                unique.push(n);
            }
        }
    }
    let vars: Vec<_> = (0..unique.len()).map(|i| format!("n{i}")).collect();
    let mut r = Rule {
        name: "sealed_repair_targeted_attack".into(),
        ..Rule::default()
    };
    r.lhs.nodes = vars.clone();
    r.rhs.nodes = vars.clone();
    for (i, &node) in unique.iter().enumerate() {
        s.get_node_mut(node)
            .unwrap()
            .set_property(MARKER, PropertyValue::Int(i as i64));
        r.lhs_node_property_guards.push(NodePropertyGuard {
            var: vars[i].clone(),
            property: MARKER,
            op: PropertyOp::Eq,
            value: PropertyValue::Int(i as i64),
        });
    }
    for (index, (edge, (_, kind))) in edges.iter().zip(targets).enumerate() {
        let endpoints: Vec<_> = edge
            .nodes
            .iter()
            .map(|n| vars[unique.iter().position(|v| v == n).unwrap()].clone())
            .collect();
        r.lhs.edges.push(endpoints.clone());
        r.lhs.edge_labels.insert(index, edge.label);
        if *kind == "delete" {
            continue;
        }
        let mut rhs = endpoints;
        if kind.contains("reverse") {
            let offset = usize::from(kind.starts_with("witness"));
            rhs.swap(offset, offset + 1);
        }
        if *kind == "foreign_erase" {
            let foreign = format!("foreign{index}");
            r.rhs.nodes.push(foreign.clone());
            r.rhs_node_labels
                .insert(foreign.clone(), s.get_node(edge.nodes[1]).unwrap().label);
            rhs[1] = foreign;
        }
        let rhs_index = r.rhs.edges.len();
        r.rhs.edges.push(rhs.clone());
        let label = if kind.contains("erase") {
            Label(0)
        } else {
            edge.label
        };
        r.rhs.edge_labels.insert(rhs_index, label);
        if *kind == "duplicate" {
            r.rhs.edge_labels.insert(r.rhs.edges.len(), label);
            r.rhs.edges.push(rhs);
        }
    }
    let mut compiled = compile_rule(&r).unwrap();
    compiled.meta_visible = true;
    assert!(compiled.pattern.node_label_ceiling.is_none());
    compiled
}
pub fn attack_step(s: &mut HypergraphStore, rule: hg_rules::CompiledRule, seed: u64) -> Value {
    let mut e = engine(vec![rule], seed, 50000);
    let rng = e.rng_state_probe();
    let stats = e.step(s);
    assert!(!e.match_ceiling_binding());
    assert_eq!(stats.accepted_rewrites, 1);
    assert_eq!(e.rng_state_probe(), rng);
    json!({"accepted": stats.accepted_rewrites, "work": e.match_budget_usage().work,
        "results": e.match_budget_usage().results, "rng": format!("{rng:016x}"), "rng_unchanged": true})
}

pub fn inventory(s: &HypergraphStore) -> Value {
    let mut nodes: Vec<_> = s
        .nodes()
        .map(|(id, data)| {
            let mut incident: Vec<_> = s
                .edges_incident_to(id)
                .iter()
                .map(|e| format!("{e:?}"))
                .collect();
            incident.sort();
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
pub fn checkpoint(
    dir: &Path,
    phase: &str,
    s: &HypergraphStore,
    root: NodeId,
    instruction: NodeId,
) -> Value {
    let payload = bincode::serialize(s).unwrap();
    let path = SnapshotWriter::new(&dir.join(phase))
        .unwrap()
        .write_snapshot(s.step(), &payload)
        .unwrap();
    let (step, back) = SnapshotReader::read_snapshot(&path).unwrap();
    assert_eq!(back, payload);
    let restored: HypergraphStore = bincode::deserialize(&back).unwrap();
    assert_eq!(step, s.step());
    assert_eq!(inventory(&restored), inventory(s));
    assert_eq!(decoded(&restored, root), decoded(s, root));
    assert_eq!(fuel(&restored, instruction), fuel(s, instruction));
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
            .map(|e| format!("{e:?}"))
            .collect();
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }
    let inv = inventory(&restored);
    for row in inv["label_indexes"].as_array().unwrap() {
        let label = row[0].as_u64().unwrap() as u32;
        let mut expected: Vec<_> = restored
            .edges()
            .filter(|(_, e)| e.label == Label(label))
            .map(|(id, _)| format!("{id:?}"))
            .collect();
        expected.sort();
        assert_eq!(row[1], json!(expected));
    }
    json!({"phase": phase, "path": format!("{phase}/{}", path.file_name().unwrap().to_str().unwrap()),
        "step": step, "root": root, "instruction": instruction, "fuel": fuel(s, instruction),
        "decoded": format!("{:?}", decoded(s, root)), "roundtrip_verified": true,
        "bytes": payload.len(), "inventory": inv})
}
