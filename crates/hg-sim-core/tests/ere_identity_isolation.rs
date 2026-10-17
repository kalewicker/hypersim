#![cfg(feature = "hg-meta")]
//! Prospective engineering diagnostic; not a scientific admission assay.
#[allow(dead_code)]
#[path = "support/coherent.rs"]
mod coherent;
#[allow(dead_code)]
#[path = "support/witness.rs"]
mod witness;
use coherent::{apply, targets, Request, Role, Status};
use flate2::{write::GzEncoder, Compression};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_rules::{EpochPolicy, MatchBudgetPolicy, StepMode};
use hg_sim_core::ere::{meta_excluded_store, semantic};
use serde_json::{json, Value};
use std::{
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};

fn key(id: NodeId) -> u64 {
    serde_json::to_value(id).unwrap()["idx"].as_u64().unwrap()
}
fn node(label: u32) -> NodeData {
    NodeData::new(Label(label), 0)
}
fn edge(label: u32, ns: &[NodeId]) -> HyperEdge {
    HyperEdge::new(Label(label), ns.iter().copied().collect())
}
fn roundtrip(s: &HypergraphStore) -> HypergraphStore {
    bincode::deserialize(&bincode::serialize(s).unwrap()).unwrap()
}
fn world(s: &HypergraphStore) -> Value {
    witness::inventory(&meta_excluded_store(s))
}

#[test]
fn graph_indexes_mixed_edges_relabel_stale_keys_clone_and_restore() {
    assert_eq!(META_BASE, 0xFFF00000);
    let mut a = HypergraphStore::new_metadata_isolated();
    let w = a.add_node(node(0));
    let x = a.add_node(node(1));
    let m = a.add_node(node(META_BASE));
    let ordinary = a.add_edge(edge(0, &[w, x]));
    let mixed = a.add_edge(edge(META_BASE, &[m, w, w]));
    assert_eq!(key(w) % 2, 1);
    assert_eq!(key(m) % 2, 0);
    assert_eq!(a.edges_incident_to(w), &[ordinary, mixed, mixed]);
    let mut b = a.clone();
    b.remove_node(m);
    assert!(b.get_edge(mixed).is_none());
    assert_eq!(b.edges_incident_to(w), &[ordinary]);
    assert!(b.edges_with_label(Label(META_BASE)).is_empty());
    let recycled = b.add_node(node(META_BASE));
    assert_eq!(key(m), key(recycled));
    assert_ne!(m, recycled);
    assert!(b.get_node(m).is_none());
    b.get_node_mut(recycled).unwrap().label = Label(2);
    b.remove_node(recycled); // removal uses original domain, not new label
    b.add_node(node(META_BASE));
    b.add_edge(edge(META_BASE, &[w]));
    assert_eq!(a.add_node(node(2)), b.add_node(node(2)));
    assert_eq!(a.add_edge(edge(2, &[w])), b.add_edge(edge(2, &[w])));
    for s in [&a, &b] {
        let mut r = roundtrip(s);
        assert!(r.metadata_allocation_isolated());
        assert_eq!(witness::inventory(s), witness::inventory(&r));
        let mut c = s.clone();
        assert_eq!(r.add_node(node(3)), c.add_node(node(3)));
        assert_eq!(r.add_edge(edge(3, &[w])), c.add_edge(edge(3, &[w])));
    }
    // World deletion/recycling is unaffected by metadata recycling.
    a.remove_node(x);
    b.remove_node(x);
    let n = a.add_node(node(4));
    assert_eq!(n, b.add_node(node(4)));
    assert_ne!(n, x);
    assert!(a.get_node(x).is_none());
}

#[test]
fn legacy_graph_payload_and_new_mode_marker_are_explicit() {
    // This is exactly the pre-change HypergraphStore payload shape.
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Legacy {
        nodes: slotmap::SlotMap<NodeId, NodeData>,
        edges: slotmap::SlotMap<hg_core::EdgeId, HyperEdge>,
        node_to_edges: slotmap::SecondaryMap<NodeId, smallvec::SmallVec<[hg_core::EdgeId; 8]>>,
        label_index: ahash::AHashMap<Label, Vec<hg_core::EdgeId>>,
        step: u64,
    }
    let mut s = HypergraphStore::new();
    let n = s.add_node(node(0));
    s.add_edge(edge(0, &[n, n]));
    s.set_step(17);
    let bytes = bincode::serialize(&s).unwrap();
    let old: Legacy = bincode::deserialize(&bytes).unwrap();
    assert_eq!(bytes, bincode::serialize(&old).unwrap());
    let restored: HypergraphStore =
        bincode::deserialize(&bincode::serialize(&old).unwrap()).unwrap();
    assert!(!restored.metadata_allocation_isolated());
    assert_eq!(witness::inventory(&s), witness::inventory(&restored));
    let isolated = HypergraphStore::new_metadata_isolated();
    assert!(bincode::deserialize::<Legacy>(&bincode::serialize(&isolated).unwrap()).is_err());
    let mut malformed = serde_json::to_value(&isolated).unwrap();
    malformed["edges"][0]["version"] = json!(0);
    assert!(serde_json::from_value::<HypergraphStore>(malformed).is_err());
}

fn parent(index: usize, isolated: bool) -> (HypergraphStore, NodeId) {
    let p = &semantic::parent_bank()[index].1;
    assert!(!p.active);
    let mut s = if isolated {
        HypergraphStore::new_metadata_isolated()
    } else {
        HypergraphStore::new()
    };
    // Same fixture and construction order as coherent-assay v1.
    let initial = p
        .core
        .labels
        .iter()
        .map(|l| {
            let mut n = node(l.unwrap_or(0));
            n.set_property(1, PropertyValue::Int(100));
            s.add_node(n)
        })
        .collect::<Vec<_>>();
    for e in &p.core.lhs {
        s.add_edge(edge(0, &e.iter().map(|v| initial[*v]).collect::<Vec<_>>()));
    }
    let ring = (0..64)
        .map(|_| {
            let mut n = node(0);
            n.set_property(1, PropertyValue::Int(100));
            s.add_node(n)
        })
        .collect::<Vec<_>>();
    for i in 0..96 {
        s.add_edge(edge(0, &[ring[i % 64], ring[(i + 1) % 64]]));
    }
    let root = encode_complete_family(
        &mut s,
        &p.core,
        p.proposal.as_ref().unwrap(),
        false,
        &MetaLimits::default(),
    )
    .unwrap()
    .root;
    assert_eq!(
        read_complete_program(&s.snapshot(), root, &MetaLimits::default()).unwrap(),
        *p
    );
    (s, root)
}

fn pair(
    index: usize,
    pattern_index: usize,
    isolated: bool,
    checkpoint: bool,
    recycled: bool,
) -> (HypergraphStore, HypergraphStore) {
    let (a, root) = parent(index, isolated);
    let mut b = a.clone();
    let scope = targets(&b, root, Role::Proposal, &MetaLimits::default()).unwrap();
    let pattern = scope.patterns[pattern_index];
    if !isolated {
        assert_eq!(key(pattern), if index == 138 { 91 } else { 85 });
    }
    let result = apply(
        &mut b,
        root,
        Role::Proposal,
        Request::AddVar { pattern },
        &MetaLimits::default(),
    );
    assert_eq!(result.status, Status::Applied, "{}", result.receipt);
    assert_eq!(
        result.before_program.as_ref().unwrap().rcf_bytes(),
        result.after_program.as_ref().unwrap().rcf_bytes()
    );
    assert_eq!(
        result.before_program.as_ref().unwrap().execution_bytes(),
        result.after_program.as_ref().unwrap().execution_bytes()
    );
    if recycled {
        let now = targets(&b, root, Role::Proposal, &MetaLimits::default()).unwrap();
        let variable = *now
            .variables
            .iter()
            .find(|v| !scope.variables.contains(v))
            .unwrap();
        assert_eq!(
            apply(
                &mut b,
                root,
                Role::Proposal,
                Request::RemoveVar { variable },
                &MetaLimits::default()
            )
            .status,
            Status::Applied
        );
        assert_eq!(
            apply(
                &mut b,
                root,
                Role::Proposal,
                Request::AddVar { pattern },
                &MetaLimits::default()
            )
            .status,
            Status::Applied
        );
        let restored = targets(&b, root, Role::Proposal, &MetaLimits::default()).unwrap();
        let new_var = *restored
            .variables
            .iter()
            .find(|v| !scope.variables.contains(v))
            .unwrap();
        assert_ne!(variable, new_var);
        assert_eq!(key(variable), key(new_var));
        let decoded = read_complete_program(&b.snapshot(), root, &MetaLimits::default()).unwrap();
        assert_eq!(
            decoded.execution_bytes(),
            result.before_program.as_ref().unwrap().execution_bytes()
        );
    }
    assert_eq!(world(&a), world(&b));
    if checkpoint {
        (roundtrip(&a), roundtrip(&b))
    } else {
        (a, b)
    }
}

struct Capture(Arc<Mutex<Vec<Value>>>);
impl hg_observe::Observer for Capture {
    fn on_step_end(
        &mut self,
        _: &hg_core::GraphSnapshot<'_>,
        _: &hg_observe::StepStats,
        events: &[hg_observe::RewriteEvent],
    ) {
        *self.0.lock().unwrap() = events
            .iter()
            .map(|e| {
                json!({
                    "rule_id":e.rule_id,"rule_name":e.rule_name,"bound_nodes":e.binding.bound_nodes,
                    "matched_edges":e.binding.matched_edges,"nodes_created":e.nodes_created,
                    "nodes_deleted":e.nodes_deleted,"edges_created":e.edges_created,
                    "edges_deleted":e.edges_deleted,"cleanup_neighbors":e.cleanup_neighbors,
                    "vanished_energy":e.vanished_energy
                })
            })
            .collect();
    }
}
fn trajectory(mut s: HypergraphStore, steps: usize, start: Instant) -> Vec<Value> {
    let mut e = Engine::new(vec![], StepMode::MaxParallel, 42, 1);
    e.set_match_budget_policy(400000, 50000, MatchBudgetPolicy::Pooled);
    let mut source = GraphRuleSource::new(EpochPolicy::Never, MetaLimits::default(), false);
    source.complete_witness_enabled = true;
    e.set_rule_source(Box::new(source));
    e.compile_at_boundary(&s);
    assert_eq!(e.rules().len(), 1);
    let captured = Arc::new(Mutex::new(Vec::new()));
    e.add_observer(Box::new(Capture(captured.clone())));
    let mut frames =
        vec![json!({"step":0,"world":world(&s),"rng":format!("{:016x}",e.rng_state_probe())})];
    for step in 1..=steps {
        assert!(start.elapsed().as_secs() < 300, "diagnostic wall cap");
        let bytes = bincode::serialize(&s).unwrap();
        let rng = e.rng_state_probe();
        e.compile_at_boundary(&s);
        assert_eq!(rng, e.rng_state_probe());
        assert_eq!(bytes, bincode::serialize(&s).unwrap());
        let stats = e.step(&mut s);
        assert!(!e.match_ceiling_binding());
        assert!(s.node_count() <= 2000 && s.edge_count() <= 4000);
        assert_eq!(s.step(), step as u64);
        frames.push(
            json!({"step":step,"world":world(&s),"rng":format!("{:016x}",e.rng_state_probe()),
            "accepted":stats.accepted_rewrites,"events":captured.lock().unwrap().clone(),
            "world_hash":format!("{:016x}",meta_excluded_store(&s).canonical_hash()),
            "usage":e.match_budget_usage()}),
        );
    }
    frames
}

fn strict_trace(frames: &[Value]) -> Vec<Value> {
    // Match work is separately retained and bounded, not ordinary-world state.
    // Do not erase handles, endpoint order, transitions, typed data or RNG.
    frames
        .iter()
        .map(|f| {
            let mut f = f.clone();
            f.as_object_mut().unwrap().remove("usage");
            f
        })
        .collect()
}

fn measure(steps: usize) -> Value {
    let start = Instant::now();
    let mut cases: Vec<Value> = Vec::new();
    let mut engine_steps = 0;
    for (index, pattern_index, draw) in [(138, 1, 19), (30, 0, 27)] {
        for (isolated, checkpoint, recycled) in [
            (false, false, false),
            (true, false, false),
            (true, true, false),
            (true, false, true),
        ] {
            let (a, b) = pair(index, pattern_index, isolated, checkpoint, recycled);
            let pa = trajectory(a, steps, start);
            let pb = trajectory(b, steps, start);
            engine_steps += steps * 2;
            assert!(engine_steps <= 2000);
            let identical = strict_trace(&pa) == strict_trace(&pb);
            assert_eq!(
                identical, isolated,
                "parent {index}, checkpoint {checkpoint}"
            );
            let births = pa
                .iter()
                .skip(1)
                .flat_map(|f| f["events"].as_array().unwrap())
                .map(|e| e["nodes_created"].as_array().unwrap().len())
                .sum::<usize>();
            assert!(births > 0);
            if checkpoint {
                let previous = cases.last().unwrap();
                assert_eq!(
                    json!(pa),
                    previous["parent_frames"],
                    "checkpoint continuation"
                );
                assert_eq!(
                    json!(pb),
                    previous["child_frames"],
                    "checkpoint continuation"
                );
            }
            cases.push(json!({"parent_index":index,"original_draw":draw,"pattern_index":pattern_index,
                "isolated":isolated,"checkpoint":checkpoint,"recycled":recycled,"identical":identical,"world_births":births,
                "parent_frames":pa,"child_frames":pb}));
        }
    }
    let mut a = HypergraphStore::new_metadata_isolated();
    let x = a.add_node(node(1));
    let y = a.add_node(node(2));
    let eid = a.add_edge(edge(0, &[x, y]));
    let mut b = a.clone();
    b.get_edge_mut(eid).unwrap().nodes.reverse();
    assert_eq!(a.canonical_hash(), b.canonical_hash());
    assert_ne!(world(&a), world(&b));
    json!({"profile":"E2-allocator-identity-diagnostic-v1","horizon":steps,"engine_steps":engine_steps,
        "elapsed_seconds":start.elapsed().as_secs_f64(),"cases":cases,
        "negative_control":{"parent_world":world(&a),"child_world":world(&b),"equal_hash":true},
        "gate_pass":false,"training_admitted":false})
}

fn pack_worlds(receipt: &mut Value) {
    // Lossless interning of repeated complete snapshots: no handle normalization
    // and no hash-based equality. This keeps 100-step receipts small to audit.
    let mut lookup = std::collections::BTreeMap::new();
    let mut worlds = Vec::new();
    for case in receipt["cases"].as_array_mut().unwrap() {
        for name in ["parent_frames", "child_frames"] {
            for frame in case[name].as_array_mut().unwrap() {
                let w = frame.as_object_mut().unwrap().remove("world").unwrap();
                let key = serde_json::to_string(&w).unwrap();
                let index = *lookup.entry(key).or_insert_with(|| {
                    let i = worlds.len();
                    worlds.push(w);
                    i
                });
                frame["world_id"] = json!(index);
            }
        }
    }
    receipt["world_table"] = json!(worlds);
}

#[test]
fn failing_draws_reproduce_and_isolated_draws_keep_strict_identity() {
    measure(8);
}

#[test]
#[ignore = "bounded persisted 100-step diagnostic; requires a fresh explicit output directory"]
fn identity_isolation_diagnostic() {
    let output = std::env::var("ERE_IDENTITY_OUTPUT").expect("absolute fresh ERE_IDENTITY_OUTPUT");
    let output = Path::new(&output);
    assert!(output.is_absolute());
    std::fs::create_dir(output).expect("never overwrite diagnostic receipts");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let mut pins = [
        "docs/ere_identity_isolation_v1.md",
        "Cargo.lock",
        "Cargo.toml",
        "crates/hg-core/src/arena.rs",
        "crates/hg-core/src/graph.rs",
        "crates/hg-core/src/lib.rs",
        "crates/hg-core/Cargo.toml",
        "crates/hg-sim-core/tests/ere_identity_isolation.rs",
        "crates/hg-sim-core/tests/support/coherent.rs",
        "crates/hg-sim-core/tests/support/witness.rs",
        "crates/hg-sim-core/src/ere/semantic.rs",
        "crates/hg-sim-core/Cargo.toml",
        "scripts/test_verify_ere_identity_isolation.py",
        "scripts/verify_ere_identity_isolation.py",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<std::collections::BTreeSet<_>>();
    fn collect(root: &Path, dir: &Path, pins: &mut std::collections::BTreeSet<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(root, &path, pins);
            } else if path.extension().is_some_and(|e| e == "rs") {
                pins.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                );
            }
        }
    }
    for name in [
        "hg-core",
        "hg-engine",
        "hg-meta",
        "hg-rules",
        "hg-observe",
        "hg-output",
        "hg-sim-core",
    ] {
        pins.insert(format!("crates/{name}/Cargo.toml"));
        collect(root, &root.join(format!("crates/{name}/src")), &mut pins);
    }
    std::fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
        "profile":"E2-allocator-identity-diagnostic-v1","run_status":"measuring",
        "gate_pass":false,"training_admitted":false}))
        .unwrap(),
    )
    .unwrap();
    let mut frozen = Vec::new();
    for pin in pins {
        let dst = output.join("source_snapshots").join(&pin);
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(root.join(&pin), &dst).unwrap();
        frozen.push(pin);
    }
    std::fs::write(
        output.join("sources.json"),
        serde_json::to_vec_pretty(&frozen).unwrap(),
    )
    .unwrap();
    std::fs::copy(
        std::env::current_exe().unwrap(),
        output.join("native_harness"),
    )
    .unwrap();
    let mut result = measure(100);
    pack_worlds(&mut result);
    let file = std::fs::File::create_new(output.join("receipt.json.gz")).unwrap();
    let mut gz = GzEncoder::new(file, Compression::default());
    gz.write_all(&serde_json::to_vec(&result).unwrap()).unwrap();
    gz.finish().unwrap();
    let auditor = output.join("source_snapshots/scripts/verify_ere_identity_isolation.py");
    let status = std::process::Command::new("python3")
        .arg("-B")
        .arg(auditor)
        .arg(output)
        .arg("--seal")
        .status()
        .unwrap();
    assert!(status.success());
    println!(
        "{}",
        serde_json::to_string(
            &json!({"output":output,"engine_steps":result["engine_steps"],
        "elapsed_seconds":result["elapsed_seconds"],"gate_pass":false,"training_admitted":false})
        )
        .unwrap()
    );
}
