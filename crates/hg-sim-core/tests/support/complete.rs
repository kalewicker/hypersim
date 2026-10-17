//! Version-10/11 helpers; prior witness fixtures and artifacts stay unchanged.
use crate::witness::{candidate, decoded, engine, inventory};
use hg_core::{EdgeId, HyperEdge, HypergraphStore, Label, NodeId};
use hg_engine::Engine;
use hg_meta::*;
use hg_output::{SnapshotReader, SnapshotWriter};
use hg_rules::EpochPolicy;
use rand::SeedableRng;
use serde_json::{json, Value};
use std::path::Path;

pub fn fixture() -> (HypergraphStore, NodeId) {
    let mut s = HypergraphStore::new();
    let c = candidate();
    let root = encode_complete_family(&mut s, &c, &c, false, &MetaLimits::default())
        .unwrap()
        .root;
    (s, root)
}
pub fn source_engine(seed: u64, budget: usize) -> Engine {
    let mut e = engine(vec![], seed, budget);
    let mut source = GraphRuleSource::new(
        EpochPolicy::Fixed(1.try_into().unwrap()),
        MetaLimits::default(),
        true,
    );
    source.vocabulary_enabled = true;
    source.witness_repair_enabled = true;
    source.sealed_repair_enabled = true;
    source.complete_witness_enabled = true;
    e.set_rule_source(Box::new(source));
    e
}
pub fn instruction(s: &mut HypergraphStore, operation: MetaOperation, fuel: f64) -> NodeId {
    encode_complete_instruction(s, &MetaInstructionSeed { operation, fuel }).unwrap()
}
pub fn fuel(s: &HypergraphStore, id: NodeId) -> f64 {
    read_complete_instruction(&s.snapshot(), id).unwrap().fuel
}
pub fn seal(s: &mut HypergraphStore, payload: EdgeId, instruction: NodeId, seed: u64) -> EdgeId {
    let original = s.get_edge(payload).unwrap().clone();
    for mut rule in
        compile_complete_operations(MetaOperation::Seal, &MetaLimits::default()).unwrap()
    {
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
    panic!("target has no complete-profile Seal binding");
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

pub fn erased(s: &mut HypergraphStore, id: EdgeId) -> EdgeId {
    let edge = s.get_edge(id).unwrap();
    let replacement = HyperEdge::new(Label(0), edge.nodes.clone());
    crate::witness::replace(s, id, replacement)
}
