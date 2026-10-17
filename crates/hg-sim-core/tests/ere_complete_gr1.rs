#![cfg(feature = "hg-meta")]
//! Isolated, opt-in complete-profile mutation assay. No production mutator changes.
#[allow(dead_code)]
#[path = "support/witness.rs"]
mod witness;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use hg_core::{EdgeId, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_rules::{EpochPolicy, MatchBudgetPolicy, RuleSetDelta, RuleSource, StepMode};
use hg_sim_core::ere::{experiments, meta_excluded_store, semantic};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};
type Res<T> = Result<T, String>;
const PROFILE: &str = "E2-complete-mutation-classification-v1";
const SEED: u64 = 0xE352310001;
const OPS: [&str; 13] = [
    "AddSlot",
    "RemoveSlot",
    "RetargetSlot",
    "AddVar",
    "RemoveVar",
    "AddPatternEdge",
    "RemovePatternEdge",
    "AddLabel",
    "RemoveLabel",
    "ChangeLabel",
    "ChangePriority",
    "Activate",
    "Deactivate",
];
const ATTACKS: [&str; 32] = [
    "owned_count_delete",
    "owned_count_mistype",
    "owned_count_increment",
    "owned_count_duplicate",
    "schema_delete",
    "schema_mistype",
    "schema_wrong_label",
    "schema_duplicate",
    "obligation_delete",
    "obligation_mistype",
    "obligation_increment",
    "obligation_duplicate",
    "ownership_delete",
    "ownership_reverse",
    "ownership_duplicate",
    "ownership_retarget_foreign",
    "witness_delete",
    "witness_relabel",
    "witness_reverse",
    "witness_duplicate",
    "payload_delete",
    "payload_relabel",
    "payload_reverse",
    "payload_duplicate",
    "body_delete",
    "body_relabel",
    "owned_orphan",
    "double_erasure",
    "encoding_wrong_version",
    "activation_mistype",
    "root_delete",
    "root_relabel",
];
fn need(v: bool, why: &str) -> Res<()> {
    if v {
        Ok(())
    } else {
        Err(why.into())
    }
}
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn hex(b: Vec<u8>) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
fn typed(v: PropertyValue) -> Value {
    match v {
        PropertyValue::Int(v) => json!({"Int":v}),
        PropertyValue::Float(v) => json!({"Float":v}),
        PropertyValue::Bool(v) => json!({"Bool":v}),
    }
}
fn candidate(c: &CandidateRule) -> Value {
    json!({"priority":c.priority,"labels":c.labels,"lhs":c.lhs,"rhs":c.rhs,"guards":c.guards.iter().map(|g|json!({"var":g.var,"key":g.key,"op":format!("{:?}",g.op),"value":typed(g.value)})).collect::<Vec<_>>(),"rcf":hex(c.rcf_bytes()),"execution":hex(c.execution_bytes())})
}
fn program(p: &RuleProgram) -> Value {
    json!({"core":candidate(&p.core),"proposal":p.proposal.as_ref().map(candidate),"active":p.active,"rcf":hex(p.rcf_bytes()),"execution":hex(p.execution_bytes())})
}
fn stable<T: serde::Serialize>(id: T) -> (u64, u64) {
    let v = serde_json::to_value(id).unwrap();
    (v["idx"].as_u64().unwrap(), v["version"].as_u64().unwrap())
}
fn delta(a: &Value, b: &Value) -> Value {
    let mut v = json!({});
    for kind in ["nodes", "edges"] {
        let map = |x: &Value| {
            x[kind]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        r[0].as_str().unwrap().to_owned(),
                        if kind == "nodes" {
                            r[1]["data"].clone()
                        } else {
                            r[1].clone()
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        let (a, b) = (map(a), map(b));
        v[kind] = json!({"deleted":a.keys().filter(|k|!b.contains_key(*k)).collect::<Vec<_>>(),"upsert":b.iter().filter(|(k,v)|a.get(*k)!=Some(*v)).map(|(k,v)|json!([k,v])).collect::<Vec<_>>()});
    }
    v
}
fn topology(v: &Value) -> Value {
    let nodes = v["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| json!([r[0], r[1]["data"]["label"]]))
        .collect::<Vec<_>>();
    let mut edges = v["edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| serde_json::to_string(&r[1]).unwrap())
        .collect::<Vec<_>>();
    edges.sort();
    json!([nodes, edges])
}
fn metadata(v: &Value) -> Value {
    json!(v["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| json!([r[0], r[1]["data"]["properties"]]))
        .collect::<Vec<_>>())
}
#[derive(Clone)]
struct Parent {
    index: usize,
    shape: usize,
    p: RuleProgram,
    s: HypergraphStore,
    root: NodeId,
}
fn world(c: &CandidateRule) -> HypergraphStore {
    let mut s = experiments::fixture(c);
    let ids = (0..64)
        .map(|_| {
            let mut n = NodeData::new(Label(0), 0);
            n.set_property(1, PropertyValue::Int(100));
            s.add_node(n)
        })
        .collect::<Vec<_>>();
    for i in 0..96 {
        s.add_edge(HyperEdge::new(
            Label(0),
            [ids[i % 64], ids[(i + 1) % 64]].into_iter().collect(),
        ));
    }
    s
}
fn parents() -> Res<Vec<Parent>> {
    semantic::parent_bank()
        .into_iter()
        .enumerate()
        .map(|(index, (base, p))| {
            let mut s = world(&p.core);
            let r = encode_complete_family(
                &mut s,
                &p.core,
                p.proposal.as_ref().unwrap(),
                p.active,
                &MetaLimits::default(),
            )
            .map_err(|e| format!("parent {index} complete encoding: {e:?}"))?;
            need(
                read_complete_program(&s.snapshot(), r.root, &MetaLimits::default())
                    .map_err(|e| format!("parent {index}: {e:?}"))?
                    == p,
                "parent round trip",
            )?;
            Ok(Parent {
                index,
                shape: base / 4,
                p,
                s,
                root: r.root,
            })
        })
        .collect()
}
fn source(epoch: EpochPolicy) -> GraphRuleSource {
    let mut s = GraphRuleSource::new(epoch, MetaLimits::default(), false);
    s.complete_witness_enabled = true;
    s
}
fn compile(s: &HypergraphStore, src: &mut GraphRuleSource, step: u64) -> Res<Value> {
    let before = bincode::serialize(s).unwrap();
    let r = match src.rules(&s.snapshot(), step) {
        RuleSetDelta::Replace { rules, report } => json!({"table":rules,"report":report}),
        RuleSetDelta::Unchanged => return Err("compile unexpectedly unchanged".into()),
    };
    need(
        bincode::serialize(s).unwrap() == before,
        "read/compile mutated graph",
    )?;
    Ok(r)
}
fn decoded(s: &HypergraphStore, root: NodeId) -> Value {
    let before = bincode::serialize(s).unwrap();
    let r = match read_complete_program(&s.snapshot(), root, &MetaLimits::default()) {
        Ok(p) => json!({"program":program(&p),"validator":null}),
        Err(v) => json!({"program":null,"validator":format!("{v:?}")}),
    };
    assert_eq!(before, bincode::serialize(s).unwrap());
    r
}
struct Edit {
    actions: Vec<Value>,
}
impl Edit {
    fn new() -> Self {
        Self { actions: vec![] }
    }
    fn node(&mut self, s: &mut HypergraphStore, n: NodeData) -> NodeId {
        let id = s.add_node(n.clone());
        self.actions.push(json!({"op":"add_node","id":id,"data":n}));
        id
    }
    fn edge(&mut self, s: &mut HypergraphStore, label: u32, ns: &[NodeId]) -> EdgeId {
        let edge = HyperEdge::new(Label(label), ns.iter().copied().collect());
        let id = s.add_edge(edge.clone());
        self.actions
            .push(json!({"op":"add_edge","id":id,"data":edge}));
        id
    }
    fn del_edge(&mut self, s: &mut HypergraphStore, id: EdgeId) {
        s.remove_edge(id);
        self.actions.push(json!({"op":"delete_edge","id":id}));
    }
    fn del_node(&mut self, s: &mut HypergraphStore, id: NodeId) {
        s.remove_node(id);
        self.actions.push(json!({"op":"delete_node","id":id}));
    }
    fn prop(&mut self, s: &mut HypergraphStore, id: NodeId, key: u32, value: PropertyValue) {
        s.get_node_mut(id).unwrap().set_property(key, value);
        self.actions
            .push(json!({"op":"set_property","id":id,"key":key,"value":typed(value)}));
    }
    fn label(&mut self, s: &mut HypergraphStore, id: NodeId, label: u32) {
        s.get_node_mut(id).unwrap().label = Label(label);
        self.actions
            .push(json!({"op":"node_label","id":id,"label":label}));
    }
    fn property_attack(&mut self, s: &mut HypergraphStore, id: NodeId, key: u32, kind: &str) {
        let old = s.get_node(id).unwrap().get_property(key).unwrap();
        match kind {
            "delete" => {
                s.get_node_mut(id)
                    .unwrap()
                    .properties
                    .retain(|(k, _)| *k != key);
                self.actions
                    .push(json!({"op":"delete_property","id":id,"key":key}));
            }
            "duplicate" => {
                s.get_node_mut(id).unwrap().properties.push((key, old));
                self.actions
                    .push(json!({"op":"append_property","id":id,"key":key,"value":typed(old)}));
            }
            "mistype" => self.prop(s, id, key, PropertyValue::Float(0.)),
            "wrong_label" => self.prop(s, id, key, PropertyValue::Int(0)),
            "increment" => {
                if let PropertyValue::Int(v) = old {
                    self.prop(s, id, key, PropertyValue::Int(v + 1))
                } else {
                    panic!("integer record")
                }
            }
            _ => panic!("unknown property attack"),
        }
    }
}
fn allele(s: &HypergraphStore, root: NodeId, role: &str) -> (NodeId, Vec<NodeId>) {
    let link = if role == "Core" { E_CORE } else { E_GRAFT };
    let anchor = s
        .edges()
        .find(|(_, e)| e.label.0 == link && e.nodes[0] == root && e.arity() == 2)
        .unwrap()
        .1
        .nodes[1];
    let sides = s
        .edges()
        .find(|(_, e)| e.label.0 == E_BECOMES && e.nodes[0] == anchor)
        .unwrap()
        .1
        .nodes[1..]
        .to_vec();
    let mut nodes = BTreeSet::new();
    nodes.insert(anchor);
    nodes.extend(&sides);
    let pats = s
        .edges()
        .filter(|(_, e)| e.label.0 == E_MEMBER && sides.contains(&e.nodes[0]) && e.arity() == 2)
        .map(|(_, e)| e.nodes[1])
        .collect::<Vec<_>>();
    nodes.extend(&pats);
    let vars = s
        .edges()
        .filter(|(_, e)| {
            (E_SLOT_BASE..E_HASLABEL).contains(&e.label.0)
                && pats.contains(&e.nodes[0])
                && e.arity() == 2
        })
        .map(|(_, e)| e.nodes[1])
        .collect::<Vec<_>>();
    nodes.extend(&vars);
    for (_, e) in s.edges().filter(|(_, e)| {
        [E_HASLABEL, E_GUARDS].contains(&e.label.0) && vars.contains(&e.nodes[0]) && e.arity() == 2
    }) {
        nodes.insert(e.nodes[1]);
    }
    let mut nodes = nodes.into_iter().collect::<Vec<_>>();
    nodes.sort_by_key(|n| stable(*n));
    (anchor, nodes)
}
fn choose<T: Copy + serde::Serialize>(
    name: &str,
    xs: &[T],
    rng: &mut ChaCha8Rng,
    choices: &mut Vec<Value>,
) -> Option<T> {
    let i = rng.gen_range(0..xs.len().max(1));
    let value = xs.get(i).copied();
    choices.push(json!({"name":name,"eligible":xs,"index":i,"selected":value}));
    value
}
fn scalar(name: &str, bound: usize, rng: &mut ChaCha8Rng, choices: &mut Vec<Value>) -> usize {
    let i = rng.gen_range(0..bound);
    choices.push(json!({"name":name,"bound":bound,"index":i}));
    i
}
fn raw_edit(
    s: &mut HypergraphStore,
    root: NodeId,
    op: usize,
    role: &str,
    rng: &mut ChaCha8Rng,
) -> Value {
    let before = witness::inventory(s);
    let mut e = Edit::new();
    let mut choices = vec![];
    let mut applicable = false;
    if op >= 11 {
        applicable = true;
        e.prop(
            s,
            root,
            K_GRAFT_MODE,
            PropertyValue::Int(i64::from(op == 11)),
        );
    } else {
        let (anchor, body) = allele(s, root, role);
        let vars = body
            .iter()
            .copied()
            .filter(|&n| s.get_node(n).unwrap().label.0 == L_VAR)
            .collect::<Vec<_>>();
        let pats = body
            .iter()
            .copied()
            .filter(|&n| s.get_node(n).unwrap().label.0 == L_PAT_EDGE)
            .collect::<Vec<_>>();
        if let Some(var) = choose("variable", &vars, rng, &mut choices) {
            if let Some(pat) = choose("pattern", &pats, rng, &mut choices) {
                let mut slots = s
                    .edges()
                    .filter(|(_, e)| {
                        e.arity() == 2
                            && e.nodes[0] == pat
                            && (E_SLOT_BASE..E_HASLABEL).contains(&e.label.0)
                    })
                    .map(|(id, _)| id)
                    .collect::<Vec<_>>();
                slots.sort_by_key(|id| (s.get_edge(*id).unwrap().label.0, stable(*id)));
                let slot = choose("slot", &slots, rng, &mut choices);
                let free = (0..9).find(|k| {
                    !slots
                        .iter()
                        .any(|id| s.get_edge(*id).unwrap().label.0 == E_SLOT_BASE + k)
                });
                match op {
                    0 => {
                        if let Some(k) = free {
                            applicable = true;
                            e.edge(s, E_SLOT_BASE + k, &[pat, var]);
                        }
                    }
                    1 => {
                        if let Some(id) = slot {
                            applicable = true;
                            e.del_edge(s, id);
                        }
                    }
                    2 => {
                        if let Some(id) = slot {
                            applicable = true;
                            let label = s.get_edge(id).unwrap().label.0;
                            e.del_edge(s, id);
                            e.edge(s, label, &[pat, var]);
                        }
                    }
                    3 => {
                        if vars.len() < 8 {
                            applicable = true;
                            let order = (0..8)
                                .find(|k| {
                                    !vars.iter().any(|v| {
                                        s.get_node(*v).unwrap().get_property(K_ORDER)
                                            == Some(PropertyValue::Int(*k))
                                    })
                                })
                                .unwrap();
                            let mut n = NodeData::new(Label(L_VAR), 0);
                            n.set_property(K_ORDER, PropertyValue::Int(order));
                            let n = e.node(s, n);
                            if let Some(k) = free {
                                e.edge(s, E_SLOT_BASE + k, &[pat, n]);
                            }
                        }
                    }
                    4 => {
                        applicable = true;
                        e.del_node(s, var);
                    }
                    5 => {
                        let sides = body
                            .iter()
                            .copied()
                            .filter(|&n| {
                                [L_SIDE_L, L_SIDE_R].contains(&s.get_node(n).unwrap().label.0)
                            })
                            .collect::<Vec<_>>();
                        if let Some(side) = choose("side", &sides, rng, &mut choices) {
                            applicable = true;
                            let p = e.node(s, NodeData::new(Label(L_PAT_EDGE), 0));
                            e.edge(s, E_MEMBER, &[side, p]);
                            e.edge(s, E_SLOT_BASE, &[p, var]);
                            if let Some(v2) = choose("second_variable", &vars, rng, &mut choices) {
                                e.edge(s, E_SLOT_BASE + 1, &[p, v2]);
                            }
                        }
                    }
                    6 => {
                        applicable = true;
                        e.del_node(s, pat);
                    }
                    7 => {
                        if !s.edges().any(|(_, e)| {
                            e.label.0 == E_HASLABEL && e.nodes[0] == var && e.arity() == 2
                        }) {
                            applicable = true;
                            let literal = scalar("label_value", 3, rng, &mut choices);
                            let mut n = NodeData::new(Label(L_LABEL_LIT), 0);
                            n.set_property(K_LITERAL, PropertyValue::Int(literal as i64));
                            let lit = e.node(s, n);
                            e.edge(s, E_HASLABEL, &[var, lit]);
                        }
                    }
                    8 | 9 => {
                        let mut literals = s
                            .edges()
                            .filter(|(_, e)| {
                                e.label.0 == E_HASLABEL && e.nodes[0] == var && e.arity() == 2
                            })
                            .map(|(_, e)| e.nodes[1])
                            .collect::<Vec<_>>();
                        literals.sort_by_key(|n| stable(*n));
                        if let Some(lit) = choose("literal", &literals, rng, &mut choices) {
                            applicable = true;
                            if op == 8 {
                                e.del_node(s, lit)
                            } else {
                                let literal = scalar("label_value", 3, rng, &mut choices);
                                e.prop(s, lit, K_LITERAL, PropertyValue::Int(literal as i64));
                            }
                        }
                    }
                    10 => {
                        if let Some(PropertyValue::Int(p)) =
                            s.get_node(anchor).unwrap().get_property(K_PRIORITY)
                        {
                            applicable = true;
                            e.prop(s, anchor, K_PRIORITY, PropertyValue::Int(p + 1));
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    let after = witness::inventory(s);
    json!({"choices":choices,"actions":e.actions,"applicable":applicable,"delta":delta(&before,&after),"raw_changed":before!=after,"logical_topology_changed":topology(&before)!=topology(&after),"metadata_changed":metadata(&before)!=metadata(&after)})
}
fn draw(bank: &[Parent], i: usize, rng: &mut ChaCha8Rng) -> (Value, HypergraphStore) {
    let parent = scalar("parent", bank.len(), rng, &mut vec![]);
    let op = scalar("operator", 13, rng, &mut vec![]);
    let role = if op < 11 {
        if rng.gen_range(0..2) == 0 {
            "Core"
        } else {
            "Proposal"
        }
    } else {
        "Family"
    };
    let mut s = bank[parent].s.clone();
    let edit = raw_edit(&mut s, bank[parent].root, op, role, rng);
    (
        json!({"index":i,"parent":parent,"operator":OPS[op],"operator_index":op,"role":role,"edit":edit,"rng_word_pos_after":rng.get_word_pos().to_string()}),
        s,
    )
}
fn attack(s: &mut HypergraphStore, root: NodeId, parent: usize, name: &str) -> Value {
    let mut body = s
        .edges()
        .filter(|(_, e)| e.label.0 == E_OWNS && e.nodes[0] == root)
        .map(|(_, e)| e.nodes[1])
        .collect::<Vec<_>>();
    body.sort_by_key(|n| stable(*n));
    let selected = body[parent % body.len()];
    let key = K_REQUIRED_BASE + (parent % 26) as u32;
    let mut relation = None;
    for offset in 0..13 {
        let ix = (parent % 13 + offset) % 13;
        let role = WITNESS_ROLES[ix];
        let mut ps = s
            .edges()
            .filter(|(_, e)| e.label.0 == role && e.arity() == 2 && e.nodes[0] != e.nodes[1])
            .map(|(id, e)| (id, e.nodes.clone()))
            .collect::<Vec<_>>();
        ps.sort_by_key(|(id, _)| stable(*id));
        if let Some((id, ns)) = ps.first() {
            let w = s
                .edges()
                .find(|(_, e)| {
                    e.label.0 == WITNESS_BASE + ix as u32
                        && e.nodes.as_slice() == [root, ns[0], ns[1]]
                })
                .unwrap()
                .0;
            relation = Some((ix, *id, w));
            break;
        }
    }
    let (ix, payload, witness) = relation.unwrap();
    let ownership = s
        .edges()
        .find(|(_, e)| e.label.0 == E_OWNS && e.nodes.as_slice() == [root, selected])
        .unwrap()
        .0;
    let mut e = Edit::new();
    let obligation_zero =
        s.get_node(selected).unwrap().get_property(key) == Some(PropertyValue::Int(0));
    if let Some(kind) = name.strip_prefix("owned_count_") {
        e.property_attack(s, root, K_OWNED_COUNT, kind)
    } else if let Some(kind) = name.strip_prefix("schema_") {
        e.property_attack(s, selected, K_SCHEMA_LABEL, kind)
    } else if let Some(kind) = name.strip_prefix("obligation_") {
        e.property_attack(s, selected, key, kind)
    } else if name == "body_delete" {
        e.del_node(s, selected)
    } else if name == "body_relabel" {
        e.label(s, selected, 0)
    } else if name == "root_delete" {
        e.del_node(s, root)
    } else if name == "root_relabel" {
        e.label(s, root, 0)
    } else if name == "encoding_wrong_version" {
        e.prop(s, root, K_ENCODING, PropertyValue::Int(3))
    } else if name == "activation_mistype" {
        e.prop(s, root, K_GRAFT_MODE, PropertyValue::Float(0.))
    } else if name == "owned_orphan" {
        let mut n = s.get_node(selected).unwrap().clone();
        for key in K_REQUIRED_BASE..K_REQUIRED_BASE + 26 {
            n.set_property(key, PropertyValue::Int(0));
        }
        let n = e.node(s, n);
        e.edge(s, E_OWNS, &[root, n]);
        e.property_attack(s, root, K_OWNED_COUNT, "increment");
    } else if name == "double_erasure" {
        for id in [payload, witness] {
            let ns = s.get_edge(id).unwrap().nodes.clone();
            e.del_edge(s, id);
            e.edge(s, 0, &ns);
        }
    } else {
        let (id, kind) = if let Some(k) = name.strip_prefix("ownership_") {
            (ownership, k)
        } else if let Some(k) = name.strip_prefix("witness_") {
            (witness, k)
        } else {
            (payload, name.strip_prefix("payload_").unwrap())
        };
        let old = s.get_edge(id).unwrap().clone();
        match kind {
            "delete" => e.del_edge(s, id),
            "duplicate" => {
                e.edge(s, old.label.0, &old.nodes);
            }
            "relabel" | "reverse" | "retarget_foreign" => {
                let mut ns = old.nodes.clone();
                let mut label = old.label.0;
                if kind == "relabel" {
                    label = 0
                } else if kind == "reverse" {
                    let offset = usize::from(name.starts_with("witness_"));
                    ns.swap(offset, offset + 1);
                } else {
                    let n = s.get_node(selected).unwrap().clone();
                    ns[1] = e.node(s, n);
                }
                e.del_edge(s, id);
                e.edge(s, label, &ns);
            }
            _ => unreachable!(),
        }
    }
    json!({"parent":parent,"attack":name,"body":selected,"obligation_key":key,"obligation_zero":obligation_zero,"relation_role_index":ix,"payload":payload,"witness":witness,"ownership":ownership,"actions":e.actions})
}
fn save(path: &Path, v: &Value) -> Res<()> {
    std::fs::write(
        path,
        serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
fn files(dir: &Path) -> Res<Vec<PathBuf>> {
    let mut v = vec![];
    for f in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let p = f.map_err(|e| e.to_string())?.path();
        need(!p.is_symlink(), "symlink refused")?;
        if p.is_dir() {
            v.extend(files(&p)?)
        } else if p.is_file() {
            v.push(p)
        }
    }
    v.sort();
    Ok(v)
}
struct Limits {
    out: PathBuf,
    start: Instant,
    bytes: u64,
    wall: f64,
    output: u64,
    attempts: u64,
    advancing: u64,
    steps: u64,
    audit_seconds: f64,
    audit_cap: f64,
    work: usize,
    results: usize,
}
impl Limits {
    fn audit(&mut self, stage: &str, dev: bool) -> Res<()> {
        let bytes_before = files(&self.out)?
            .iter()
            .map(|p| p.metadata().unwrap().len())
            .sum::<u64>();
        let seconds = (self.audit_cap - self.audit_seconds)
            .min(self.wall - self.start.elapsed().as_secs_f64());
        self.audit_seconds += auditor(&self.out, stage, dev, seconds)?;
        let bytes_after = files(&self.out)?
            .iter()
            .map(|p| p.metadata().unwrap().len())
            .sum::<u64>();
        self.account(bytes_after.saturating_sub(bytes_before))
    }
    fn check(&self) -> Res<()> {
        need(
            self.start.elapsed().as_secs_f64() < self.wall,
            "assay wall cap",
        )?;
        need(self.bytes <= self.output, "assay output cap")?;
        need(self.attempts <= self.steps, "advancing/attempted step cap")
    }
    fn account(&mut self, n: u64) -> Res<()> {
        self.bytes = self.bytes.checked_add(n).ok_or("byte overflow")?;
        self.check()
    }
    fn record(&mut self, name: &str, v: &Value) -> Res<()> {
        let p = self.out.join(name);
        let old = p.metadata().map(|m| m.len()).unwrap_or(0);
        save(&p, v)?;
        self.account(p.metadata().unwrap().len().saturating_sub(old))
    }
}
struct Stream {
    writer: GzEncoder<BufWriter<File>>,
    bytes: u64,
}
impl Stream {
    fn new(path: &Path) -> Res<Self> {
        Ok(Self {
            writer: GzEncoder::new(
                BufWriter::new(File::create_new(path).map_err(|e| e.to_string())?),
                Compression::default(),
            ),
            bytes: 0,
        })
    }
    fn line(&mut self, v: &Value, lim: &mut Limits) -> Res<()> {
        let mut b = serde_json::to_vec(v).map_err(|e| e.to_string())?;
        b.push(b'\n');
        // Conservative deflate bound plus a small failure/footer reserve:
        // refuse BEFORE emitting a row that could cross the archive ceiling.
        need(
            lim.bytes
                .saturating_add(b.len() as u64)
                .saturating_add(8192)
                <= lim.output,
            "assay output cap before emission",
        )?;
        self.writer.write_all(&b).map_err(|e| e.to_string())?;
        self.writer.flush().map_err(|e| e.to_string())?;
        let now = self.writer.get_ref().get_ref().metadata().unwrap().len();
        lim.account(now - self.bytes)?;
        self.bytes = now;
        Ok(())
    }
    fn finish(self, lim: &mut Limits) -> Res<()> {
        let old = self.bytes;
        let mut w = self.writer.finish().map_err(|e| e.to_string())?;
        w.flush().map_err(|e| e.to_string())?;
        lim.account(w.get_ref().metadata().unwrap().len() - old)
    }
}
fn read_stream(path: &Path) -> Res<impl Iterator<Item = Res<Value>>> {
    Ok(
        BufReader::new(GzDecoder::new(File::open(path).map_err(|e| e.to_string())?))
            .lines()
            .map(|l| {
                serde_json::from_str(&l.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
            }),
    )
}
fn auditor(out: &Path, stage: &str, dev: bool, seconds: f64) -> Res<f64> {
    need(seconds > 0., "auditor exhausted")?;
    let now = Instant::now();
    let mut cmd = std::process::Command::new("python3");
    cmd.arg("-B")
        .arg(out.join("source_snapshots/scripts/verify_ere_complete_gr1.py"))
        .arg(out)
        .arg("--stage")
        .arg(stage)
        .arg("--seconds")
        .arg(seconds.to_string());
    if dev {
        cmd.arg("--development");
    }
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            need(status.success(), "frozen auditor failed")?;
            return Ok(now.elapsed().as_secs_f64());
        }
        if now.elapsed().as_secs_f64() >= seconds {
            let _ = child.kill();
            let _ = child.wait();
            return Err("shared auditor wall cap".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}
fn basis_copies(out: &Path, design: &Value) -> Res<()> {
    let b = &design["basis"];
    let mut paths = b["design_time_source_sha256"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    for name in b["full_files_sha256"].as_object().unwrap().keys() {
        paths.push(format!("{}/{name}", b["full_archive"].as_str().unwrap()));
    }
    for name in ["summary.json", "independent_audit.json"] {
        paths.push(format!("{}/{name}", b["bounded_archive"].as_str().unwrap()));
    }
    paths.push(b["historical_classification"].as_str().unwrap().into());
    for rel in paths {
        let dst = out.join("basis").join(&rel);
        std::fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::copy(repo().join(&rel), dst).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn freeze(out: &Path, bank: &[Parent], dev: bool, draws: usize, horizon: usize) -> Res<()> {
    let root = repo();
    let design: Value =
        serde_json::from_slice(&std::fs::read(root.join("docs/ere_complete_gr1_v1.json")).unwrap())
            .unwrap();
    let mut inputs = vec![
        root.join("Cargo.lock"),
        root.join("Cargo.toml"),
        root.join("docs/ere_complete_gr1_v1.json"),
        root.join("docs/ere_complete_gr1_v1.md"),
    ];
    for dir in [
        "crates/hg-core",
        "crates/hg-engine",
        "crates/hg-rules",
        "crates/hg-meta",
        "crates/hg-observe",
        "crates/hg-schema",
        "crates/hg-eval",
        "crates/hg-causal",
        "crates/hg-output",
        "crates/hg-report",
        "crates/hg-app-api",
        "crates/hg-sim-core/src",
        "crates/hg-sim-core/tests/support",
    ] {
        inputs.extend(files(&root.join(dir))?);
    }
    for rel in [
        "crates/hg-sim-core/Cargo.toml",
        "crates/hg-sim-core/tests/ere_complete_gr1.rs",
        "scripts/verify_ere_complete_gr1.py",
        "scripts/test_verify_ere_complete_gr1.py",
        "scripts/verify_ere_complete_gates.py",
        "scripts/verify_ere_complete_witness.py",
        "scripts/verify_ere_complete_long_gates.py",
    ] {
        inputs.push(root.join(rel));
    }
    inputs.sort();
    inputs.dedup();
    let mut sources = vec![];
    for path in inputs {
        let rel = path.strip_prefix(&root).unwrap();
        let dst = out.join("source_snapshots").join(rel);
        std::fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::copy(&path, &dst).map_err(|e| e.to_string())?;
        sources.push(rel.to_str().unwrap().to_owned());
    }
    std::fs::copy(
        std::env::current_exe().unwrap(),
        out.join("executed_test_binary"),
    )
    .map_err(|e| e.to_string())?;
    basis_copies(out, &design)?;
    let mut lim = Limits {
        out: out.into(),
        start: Instant::now(),
        bytes: files(out)?
            .iter()
            .map(|p| p.metadata().unwrap().len())
            .sum(),
        wall: 300.,
        output: if dev { 536870912 } else { 4294967296 },
        attempts: 0,
        advancing: 0,
        steps: 0,
        audit_seconds: 0.,
        audit_cap: 900.,
        work: 400000,
        results: 50000,
    };
    let mut ps = Stream::new(&out.join("parents.jsonl.gz"))?;
    for p in bank {
        ps.line(&json!({"index":p.index,"shape":p.shape,"root":p.root,"program":program(&p.p),"inventory":witness::inventory(&p.s),"world":witness::inventory(&meta_excluded_store(&p.s))}),&mut lim)?;
    }
    ps.finish(&mut lim)?;
    let mut ds = Stream::new(&out.join("draw_manifest.jsonl.gz"))?;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    for i in 0..draws {
        ds.line(&draw(bank, i, &mut rng).0, &mut lim)?;
    }
    ds.finish(&mut lim)?;
    let mut cs = Stream::new(&out.join("control_manifest.jsonl.gz"))?;
    for p in bank {
        for name in ATTACKS {
            cs.line(&attack(&mut p.s.clone(), p.root, p.index, name), &mut lim)?;
        }
    }
    cs.finish(&mut lim)?;
    save(
        &out.join("manifest.json"),
        &json!({"profile":PROFILE,"protocol":COMPLETE_PROTOCOL,"family_version":10,"development_only":dev,"training_admitted":false,"scientific_run_launched":!dev,"parents":168,"draws":draws,"horizon":horizon,"controls":5376,"sources":sources,"resources":design["proposed_resources"],"development_resources":if dev{json!({"wall":300,"output":536870912,"audit":120,"steps":(168+draws)*horizon})}else{Value::Null},"format":"canonical compressed JSONL","frozen_before_measurement":true}),
    )?;
    auditor(out, "freeze", dev, 120.)?;
    Ok(())
}
fn controls(bank: &[Parent], lim: &mut Limits) -> Res<()> {
    let mut stream = Stream::new(&lim.out.join("controls.jsonl.gz"))?;
    let mut manifest = read_stream(&lim.out.join("control_manifest.jsonl.gz"))?;
    for p in bank {
        for name in ATTACKS {
            lim.check()?;
            let mut s = p.s.clone();
            let before = witness::inventory(&s);
            let mut src = source(EpochPolicy::Fixed(1.try_into().unwrap()));
            let clean = compile(&s, &mut src, 0)?;
            let a = attack(&mut s, p.root, p.index, name);
            need(
                Some(a.clone()) == manifest.next().transpose()?,
                "control manifest regeneration mismatch",
            )?;
            let result = decoded(&s, p.root);
            let compiled = compile(&s, &mut src, 1)?;
            let absence = ["root_delete", "root_relabel"].contains(&name);
            let valid = result["program"].is_object();
            let report = &compiled["report"];
            let ok = !valid
                && compiled["table"].as_array().unwrap().is_empty()
                && report["retired"] == 1
                && report["quarantined"] == usize::from(!absence)
                && report["validated_regions"] == 0;
            stream.line(&json!({"parent":p.index,"attack":name,"manifest":a,"clean":clean,"clean_decoded":program(&p.p),"result":result,"compiled":compiled,"delta":delta(&before,&witness::inventory(&s)),"read_only":true,"root_alive":s.get_node(p.root).is_some(),"root_discovered":s.get_node(p.root).is_some_and(|n|n.label.0==L_RULE_ROOT),"native_pass":ok}),lim)?;
            need(ok, &format!("control {} {name} failed", p.index))?;
        }
    }
    need(manifest.next().is_none(), "extra control manifest")?;
    stream.finish(lim)
}
#[derive(Clone, Default)]
struct Trace {
    frames: Vec<(String, u64)>,
    rng: Vec<String>,
    complete: bool,
    exploding: bool,
    frozen: bool,
}
struct Capture(Arc<Mutex<Option<Value>>>);
impl hg_observe::Observer for Capture {
    fn on_step_end(
        &mut self,
        _: &hg_core::GraphSnapshot<'_>,
        stats: &hg_observe::StepStats,
        events: &[hg_observe::RewriteEvent],
    ) {
        *self.0.lock().unwrap() = Some(
            json!({"stats":stats,"events":events.iter().map(|e|json!({"rule_id":e.rule_id,"rule_name":e.rule_name,"bound_nodes":e.binding.bound_nodes,"matched_edges":e.binding.matched_edges,"nodes_created":e.nodes_created,"nodes_deleted":e.nodes_deleted,"edges_created":e.edges_created,"edges_deleted":e.edges_deleted,"cleanup_neighbors":e.cleanup_neighbors,"vanished_energy":e.vanished_energy})).collect::<Vec<_>>()}),
        );
    }
}
fn engine(s: &HypergraphStore) -> Engine {
    let mut e = Engine::new(vec![], StepMode::MaxParallel, 42, 1);
    e.set_match_budget_policy(400000, 50000, MatchBudgetPolicy::Pooled);
    e.set_rule_source(Box::new(source(EpochPolicy::Never)));
    e.compile_at_boundary(s);
    e
}
fn trace(
    id: &str,
    mut s: HypergraphStore,
    root: NodeId,
    horizon: usize,
    stream: &mut Stream,
    lim: &mut Limits,
) -> Res<Trace> {
    let mut e = engine(&s);
    e.set_match_budget_policy(lim.work, lim.results, MatchBudgetPolicy::Pooled);
    let native = Arc::new(Mutex::new(None));
    e.add_observer(Box::new(Capture(native.clone())));
    let mut previous = witness::inventory(&s);
    let initial_world = meta_excluded_store(&s);
    let initial = (initial_world.node_count(), initial_world.edge_count());
    stream.line(&json!({"kind":"start","id":id,"initial":previous,"root":root,"decoded":decoded(&s,root),"table":e.rules(),"compile_report":e.compile_report(),"initial_rng":format!("{:016x}",e.rng_state_probe()),"initial_full_hash":format!("{:016x}",s.canonical_hash()),"initial_world_hash":format!("{:016x}",meta_excluded_store(&s).canonical_hash()),"horizon":horizon}),lim)?;
    let mut result = Trace::default();
    let mut reason = None;
    for step in 1..=horizon {
        lim.check()?;
        let rng = e.rng_state_probe();
        let bytes = bincode::serialize(&s).unwrap();
        e.compile_at_boundary(&s);
        need(
            rng == e.rng_state_probe() && bytes == bincode::serialize(&s).unwrap(),
            "compile noninterference",
        )?;
        let full_before = format!("{:016x}", s.canonical_hash());
        let world_before = format!("{:016x}", meta_excluded_store(&s).canonical_hash());
        let before_step = s.step();
        let stats = e.step(&mut s);
        lim.attempts += 1;
        lim.advancing += s.step() - before_step;
        let after = witness::inventory(&s);
        let world = meta_excluded_store(&s);
        let capture = native
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| json!({"stats":stats,"events":[]}));
        let cap = e.match_ceiling_binding() || s.node_count() > 2000 || s.edge_count() > 4000;
        stream.line(&json!({"kind":"step","id":id,"step":step,"store_step":s.step(),"full_hash_before":full_before,"full_hash":format!("{:016x}",s.canonical_hash()),"world_hash_before":world_before,"world_hash":format!("{:016x}",world.canonical_hash()),"world_nodes":world.node_count(),"world_edges":world.edge_count(),"rng_before":format!("{rng:016x}"),"rng_after":format!("{:016x}",e.rng_state_probe()),"usage":e.match_budget_usage(),"guards":e.drain_guard_stats(),"ceiling":e.match_ceiling_binding(),"graph_cap":s.node_count()>2000||s.edge_count()>4000,"native":capture,"delta":delta(&previous,&after)}),lim)?;
        need(
            e.match_budget_usage().work <= lim.work
                && e.match_budget_usage().results <= lim.results,
            "pooled budget violation",
        )?;
        if cap {
            reason = Some(if e.match_ceiling_binding() {
                "matching_cap"
            } else {
                "store_cap"
            });
            break;
        }
        need(s.step() == step as u64, "nonadvancing uncapped Engine")?;
        result.frames.push((
            format!("{:016x}", world.canonical_hash()),
            stats.accepted_rewrites as u64,
        ));
        result.rng.push(format!("{:016x}", e.rng_state_probe()));
        result.exploding |=
            world.node_count() > 4 * initial.0 || world.edge_count() > 4 * initial.1;
        previous = after;
    }
    result.complete = result.frames.len() == horizon;
    result.frozen = result
        .frames
        .windows(50)
        .any(|w| w.iter().all(|(_, n)| *n == 0));
    stream.line(&json!({"kind":"end","id":id,"complete":result.complete,"reason":reason,"measured_steps":result.frames.len(),"exploding":result.exploding,"frozen":result.frozen,"terminal":witness::inventory(&s)}),lim)?;
    Ok(result)
}
fn outcome(valid: bool, neutral: bool, parent: &Trace, child: Option<&Trace>) -> &'static str {
    if !valid {
        return "quarantined";
    }
    if !parent.complete || child.is_some_and(|c| !c.complete) {
        return "unmeasured";
    }
    if neutral {
        return "neutral";
    }
    let c = child.unwrap();
    if c.frames == parent.frames {
        "near_neutral"
    } else if c.exploding || c.frozen {
        "catastrophic"
    } else {
        "divergent"
    }
}
fn measurement_complete(classification: &Value) -> bool {
    classification["parent_baselines_complete"] == true
        && classification["strata"]["overall"]["unmeasured"]
            .as_u64()
            .unwrap_or(0)
            == 0
}
fn at_cap(p: &RuleProgram) -> bool {
    std::iter::once(&p.core)
        .chain(p.proposal.as_ref())
        .any(|c| c.lhs.iter().chain(&c.rhs).any(|e| e.len() == 8))
}
#[derive(Default)]
struct Tally {
    strata: BTreeMap<String, BTreeMap<String, u64>>,
    samples: usize,
    mode_changes: u64,
    changed_modes: u64,
    checks: u64,
    failures: u64,
}
impl Tally {
    fn record(&mut self, row: &Value) {
        self.samples += 1;
        let op = row["operator_index"].as_u64().unwrap();
        let o = row["outcome"].as_str().unwrap();
        let mut keys = vec![
            "overall".to_owned(),
            format!("operator:{}", row["operator"].as_str().unwrap()),
            format!("shape:{}", row["shape"]),
            format!("active:{}", row["parent_active"]),
            format!("at_cap:{}", row["at_cap"]),
            format!("logical_change:{}", row["logical_topology_changed"]),
            format!("raw_change:{}", row["raw_changed"]),
            format!("metadata_change:{}", row["metadata_changed"]),
            format!("applicable:{}", row["applicable"]),
            format!("parent_identity:{}", row["parent_rcf"].as_str().unwrap()),
            format!("genotype_parent:{}", row["parent"]),
            format!("validator:{}", row["validator"]),
        ];
        if op < 11 {
            keys.push("structural".into());
            keys.push(format!("role:{}", row["role"].as_str().unwrap()));
            if row["parent_active"] == true {
                keys.push("active_parent_structural".into())
            }
        } else {
            keys.push("activation".into());
            self.mode_changes += u64::from(row["mode_changed"] == true);
            self.changed_modes +=
                u64::from(row["mode_changed"] == true && row["trace_changed"] == true);
        }
        for key in keys {
            let r = self.strata.entry(key).or_default();
            *r.entry("samples".into()).or_default() += 1;
            *r.entry(o.into()).or_default() += 1;
            if o != "quarantined" {
                *r.entry("valid".into()).or_default() += 1;
            }
        }
        self.checks += u64::from(row["soundness_checked"] == true);
        self.failures += u64::from(row["soundness_failure"] == true);
    }
    fn finish(self) -> Value {
        let Self {
            strata,
            samples,
            mode_changes,
            changed_modes,
            checks,
            failures,
        } = self;
        let thresholds = |key: &str, near: bool, min: u64| {
            let c = strata.get(key);
            let n = c.and_then(|r| r.get("samples")).copied().unwrap_or(0);
            let count = |k: &str| c.and_then(|r| r.get(k)).copied().unwrap_or(0);
            n >= min
                && n > 0
                && 5 * count("valid") >= 2 * n
                && (!near || 10 * count("near_neutral") >= n)
                && 4 * count("catastrophic") <= n
                && count("unmeasured") == 0
        };
        let numerical = samples == 10000
            && thresholds("overall", true, 10000)
            && thresholds("structural", true, 1)
            && thresholds("active_parent_structural", true, 2000)
            && thresholds("activation", false, 1000)
            && mode_changes >= 100
            && changed_modes >= 20
            && checks >= 50
            && failures == 0
            && OPS
                .iter()
                .all(|op| strata.contains_key(&format!("operator:{op}")))
            && [
                "role:Core",
                "role:Proposal",
                "at_cap:true",
                "at_cap:false",
                "active:true",
                "active:false",
            ]
            .iter()
            .all(|k| strata.contains_key(*k));
        json!({"strata":strata,"mode_changes":mode_changes,"trajectory_changing_mode_edits":changed_modes,"neutral_soundness_checked":checks,"neutral_soundness_failures":failures,"numerical_pass":numerical})
    }
}
fn tally(rows: &[Value]) -> Value {
    let mut t = Tally::default();
    for row in rows {
        t.record(row);
    }
    t.finish()
}
fn primary(bank: &[Parent], horizon: usize, draws: usize, lim: &mut Limits) -> Res<Value> {
    let mut ps = Stream::new(&lim.out.join("parent_traces.jsonl.gz"))?;
    let mut cache = vec![];
    for p in bank {
        cache.push(trace(
            &format!("parent_{}", p.index),
            p.s.clone(),
            p.root,
            horizon,
            &mut ps,
            lim,
        )?);
    }
    ps.finish(lim)?;
    let mut ds = Stream::new(&lim.out.join("child_traces.jsonl.gz"))?;
    let mut rs = Stream::new(&lim.out.join("classifications.jsonl.gz"))?;
    let mut manifest = read_stream(&lim.out.join("draw_manifest.jsonl.gz"))?;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut totals = Tally::default();
    let mut checked = 0;
    for i in 0..draws {
        let (d, s) = draw(bank, i, &mut rng);
        need(
            manifest.next().transpose()? == Some(d.clone()),
            "draw manifest regeneration mismatch",
        )?;
        let idx = d["parent"].as_u64().unwrap() as usize;
        let p = &bank[idx];
        let result = decoded(&s, p.root);
        let mut src = source(EpochPolicy::Never);
        let compiled = compile(&s, &mut src, 0)?;
        let valid = result["program"].is_object();
        let neutral = valid && result["program"]["rcf"] == program(&p.p)["rcf"];
        let soundness = valid && neutral && cache[idx].complete && checked < 50;
        if soundness {
            checked += 1;
        }
        let child = if valid && (!neutral || soundness) {
            Some(trace(
                &format!("child_{i}"),
                s,
                p.root,
                horizon,
                &mut ds,
                lim,
            )?)
        } else {
            None
        };
        let o = outcome(valid, neutral, &cache[idx], child.as_ref());
        let failure = soundness
            && child.as_ref().is_none_or(|c| {
                !c.complete || c.frames != cache[idx].frames || c.rng != cache[idx].rng
            });
        let changed = child
            .as_ref()
            .is_some_and(|c| c.complete && cache[idx].complete && c.frames != cache[idx].frames);
        let row = json!({"index":i,"parent":idx,"operator":d["operator"],"operator_index":d["operator_index"],"role":d["role"],"manifest":d,"decoded":result,"compiled":compiled,"valid":valid,"neutral_identity":neutral,"outcome":o,"trace_id":child.as_ref().map(|_|format!("child_{i}")),"soundness_checked":soundness,"soundness_failure":failure,"trace_changed":changed,"parent_active":p.p.active,"at_cap":at_cap(&p.p),"shape":p.shape,"parent_rcf":hex(p.p.rcf_bytes()),"validator":result["validator"],"mode_changed":valid&&result["program"]["active"]!=p.p.active,"raw_changed":d["edit"]["raw_changed"],"logical_topology_changed":d["edit"]["logical_topology_changed"],"metadata_changed":d["edit"]["metadata_changed"],"applicable":d["edit"]["applicable"],"executable_changed":valid&&!neutral});
        rs.line(&row, lim)?;
        totals.record(&row);
        if i % 16 == 0 {
            lim.record("progress.json",&json!({"classified":i+1,"attempted_steps":lim.attempts,"advancing_steps":lim.advancing,"elapsed_seconds":lim.start.elapsed().as_secs_f64(),"training_admitted":false}))?;
        }
    }
    need(manifest.next().is_none(), "extra frozen draws")?;
    ds.finish(lim)?;
    rs.finish(lim)?;
    let mut classification = totals.finish();
    // Every baseline is required even if its parent's only sampled edits are
    // quarantined, or that parent happens not to be selected by the draw stream.
    classification["parent_baselines_complete"] = json!(cache.iter().all(|t| t.complete));
    Ok(classification)
}
fn campaign(out: &Path, dev: bool) -> Res<()> {
    std::fs::create_dir(out).map_err(|e| format!("fresh output required: {e}"))?;
    let result = (|| {
        let bank = parents()?;
        need(bank.len() == 168, "wrong parent bank")?;
        let (draws, horizon) = if dev { (64, 8) } else { (10000, 100) };
        freeze(out, &bank, dev, draws, horizon)?;
        let mut lim = Limits {
            out: out.into(),
            start: Instant::now(),
            bytes: files(out)?
                .iter()
                .map(|p| p.metadata().unwrap().len())
                .sum(),
            wall: if dev { 300. } else { 3600. },
            output: if dev { 536870912 } else { 4294967296 },
            attempts: 0,
            advancing: 0,
            steps: ((168 + draws) * horizon) as u64,
            audit_seconds: 0.,
            audit_cap: if dev { 120. } else { 900. },
            work: 400000,
            results: 50000,
        };
        controls(&bank, &mut lim)?;
        lim.audit("controls", dev)?;
        lim.record("stage_barrier.json",&json!({"controls_native_and_frozen_audit_pass":true,"primary_mutations":"not_started","advancing_steps":0,"development_only":dev,"training_admitted":false}))?;
        let classification = primary(&bank, horizon, draws, &mut lim)?;
        lim.record("classification_summary.json", &classification)?;
        lim.audit("full", dev)?;
        let audit: Value =
            serde_json::from_slice(&std::fs::read(out.join("audit_full.json")).unwrap()).unwrap();
        need(
            audit["numerical_pass"] == classification["numerical_pass"],
            "native/auditor numerical disagreement",
        )?;
        let measured = measurement_complete(&classification);
        need(
            audit["measurement_complete"] == measured,
            "native/auditor measurement disagreement",
        )?;
        lim.record("run_status.json",&json!({"status":"complete","development_only":dev,"scientific_run_launched":!dev,"gate_pass":!dev&&classification["numerical_pass"]==true&&measured,"numerical_pass":classification["numerical_pass"],"measurement_complete":measured,"training_admitted":false,"manifest_regenerated":true,"advancing_engine_steps":lim.advancing,"attempted_engine_steps":lim.attempts,"draws":draws,"controls":5376,"elapsed_seconds":lim.start.elapsed().as_secs_f64(),"auditor_seconds":lim.audit_seconds,"accounted_output_bytes":lim.bytes}))?;
        lim.audit("seal", dev)?;
        lim.check()?;
        need(
            files(out)?
                .iter()
                .map(|p| p.metadata().unwrap().len())
                .sum::<u64>()
                <= lim.output,
            "sealed output cap",
        )?;
        Ok(())
    })();
    if let Err(e) = &result {
        let _ = save(
            &out.join("run_status.json"),
            &json!({"status":"failed_or_unmeasured","error":e,"development_only":dev,"scientific_run_launched":!dev,"gate_pass":false,"training_admitted":false,"primary_mutations":if out.join("parent_traces.jsonl.gz").exists(){"partial"}else{"not_started"}}),
        );
    }
    result
}
#[test]
fn full_parent_bank_preflight_and_deterministic_unrepaired_draws() {
    let bank = parents().unwrap();
    assert_eq!(bank.len(), 168);
    let mut a = ChaCha8Rng::seed_from_u64(SEED);
    let mut b = a.clone();
    for i in 0..32 {
        let (x, s) = draw(&bank, i, &mut a);
        let (y, t) = draw(&bank, i, &mut b);
        assert_eq!(x, y);
        assert_eq!(
            bincode::serialize(&s).unwrap(),
            bincode::serialize(&t).unwrap()
        );
    }
    assert_eq!(a, b);
}
#[test]
fn damage_matrix_native_controls() {
    let bank = parents().unwrap();
    for p in bank.iter().filter(|p| [0, 15, 151, 167].contains(&p.index)) {
        for name in ATTACKS {
            let mut s = p.s.clone();
            let mut src = source(EpochPolicy::Fixed(1.try_into().unwrap()));
            compile(&s, &mut src, 0).unwrap();
            attack(&mut s, p.root, p.index, name);
            assert!(
                decoded(&s, p.root)["program"].is_null(),
                "{} {name}",
                p.index
            );
            let r = compile(&s, &mut src, 1).unwrap();
            assert!(r["table"].as_array().unwrap().is_empty());
            assert_eq!(r["report"]["retired"], 1);
            assert_eq!(
                r["report"]["quarantined"],
                usize::from(!["root_delete", "root_relabel"].contains(&name))
            );
        }
    }
}
#[test]
fn observer_compile_and_read_noninterference() {
    let bank = parents().unwrap();
    for index in [0, 37, 49, 151] {
        let p = &bank[index];
        let (mut a, mut b) = (p.s.clone(), p.s.clone());
        let (mut ea, mut eb) = (engine(&a), engine(&b));
        let capture = Arc::new(Mutex::new(None));
        eb.add_observer(Box::new(Capture(capture.clone())));
        for _ in 0..8 {
            let prior = bincode::serialize(&b).unwrap();
            let rng = eb.rng_state_probe();
            let _ = decoded(&b, p.root);
            eb.compile_at_boundary(&b);
            let _ = eb.compile_report();
            assert_eq!(prior, bincode::serialize(&b).unwrap());
            assert_eq!(rng, eb.rng_state_probe());
            let sa = ea.step(&mut a);
            let sb = eb.step(&mut b);
            assert_eq!(
                serde_json::to_value(sa).unwrap(),
                serde_json::to_value(sb).unwrap()
            );
            assert_eq!(ea.rng_state_probe(), eb.rng_state_probe());
            assert_eq!(
                bincode::serialize(&a).unwrap(),
                bincode::serialize(&b).unwrap()
            );
            assert!(!ea.match_ceiling_binding() && !eb.match_ceiling_binding());
            assert!(capture.lock().unwrap().take().is_some());
        }
    }
}
#[test]
fn raw_noops_dormant_damage_and_metadata_preservation() {
    let bank = parents().unwrap();
    let p = &bank[0];
    let mut s = p.s.clone();
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let no_op = raw_edit(&mut s, p.root, 8, "Proposal", &mut rng);
    assert_eq!(no_op["applicable"], false);
    assert_eq!(no_op["raw_changed"], false);
    assert_eq!(no_op["choices"].as_array().unwrap().len(), 4);
    for op in [0, 1, 2] {
        let mut s = p.s.clone();
        let m = metadata(&witness::inventory(&s));
        raw_edit(&mut s, p.root, op, "Proposal", &mut rng);
        assert_eq!(metadata(&witness::inventory(&s)), m);
        if op != 2 {
            assert!(decoded(&s, p.root)["program"].is_null());
        }
    }
    // RetargetSlot is applicable even when it replaces the identical relation.
    let mut found = false;
    for seed in 0..16 {
        let mut s = p.s.clone();
        let edit = raw_edit(
            &mut s,
            p.root,
            2,
            "Core",
            &mut ChaCha8Rng::seed_from_u64(seed),
        );
        if edit["logical_topology_changed"] == false {
            assert_eq!(edit["raw_changed"], true);
            assert_eq!(edit["applicable"], true);
            assert!(decoded(&s, p.root)["program"].is_object());
            found = true;
            break;
        }
    }
    assert!(found);
}
#[test]
fn budget_resource_and_overwrite_refusals() {
    let tmp = tempfile::tempdir().unwrap();
    let mut lim = Limits {
        out: tmp.path().into(),
        start: Instant::now(),
        bytes: 0,
        wall: 60.,
        output: 1024,
        attempts: 0,
        advancing: 0,
        steps: 8,
        audit_seconds: 0.,
        audit_cap: 60.,
        work: 400000,
        results: 50000,
    };
    assert!(lim.account(1025).unwrap_err().contains("output cap"));
    lim.bytes = 0;
    lim.wall = 0.;
    assert!(lim.check().unwrap_err().contains("wall cap"));
    lim.wall = 60.;
    lim.attempts = 9;
    assert!(lim.check().unwrap_err().contains("step cap"));
    let path = tmp.path().join("raw.gz");
    let _ = Stream::new(&path).unwrap();
    assert!(Stream::new(&path).is_err());
    assert!(campaign(tmp.path(), true)
        .unwrap_err()
        .contains("fresh output"));
    let bank = parents().unwrap();
    let mut s = bank[0].s.clone();
    let mut e = engine(&s);
    e.set_match_budget_policy(0, 0, MatchBudgetPolicy::Pooled);
    let before = bincode::serialize(&s).unwrap();
    let rng = e.rng_state_probe();
    let stats = e.step(&mut s);
    assert!(e.match_ceiling_binding());
    assert_eq!(s.step(), 0);
    assert_eq!(stats.accepted_rewrites, 0);
    assert_eq!(rng, e.rng_state_probe());
    assert_eq!(before, bincode::serialize(&s).unwrap());
}
#[test]
fn classification_precedence_and_exact_boundaries() {
    let mut classified = json!({"parent_baselines_complete": true,"strata":{"overall":{"quarantined":10000,"unmeasured":0}}});
    assert!(measurement_complete(&classified));
    classified["parent_baselines_complete"] = json!(false);
    assert!(!measurement_complete(&classified));
    classified["parent_baselines_complete"] = json!(true);
    classified["strata"]["overall"]["unmeasured"] = json!(1);
    assert!(!measurement_complete(&classified));
    let p = Trace {
        complete: true,
        ..Default::default()
    };
    let c = Trace {
        complete: false,
        ..Default::default()
    };
    assert_eq!(outcome(false, false, &p, None), "quarantined");
    assert_eq!(outcome(true, true, &c, None), "unmeasured");
    assert_eq!(outcome(true, true, &p, Some(&c)), "unmeasured");
    assert_eq!(outcome(true, true, &p, None), "neutral");
    assert_eq!(outcome(true, false, &p, Some(&p)), "near_neutral");
    assert!(!vec![("x", 0); 49]
        .windows(50)
        .any(|w| w.iter().all(|(_, n)| *n == 0)));
    assert!(vec![("x", 0); 50]
        .windows(50)
        .any(|w| w.iter().all(|(_, n)| *n == 0)));
    assert!(!(400 > 4 * 100));
    assert!(401 > 4 * 100);
    assert_eq!(tally(&[])["numerical_pass"], false);
}
#[test]
#[ignore = "bounded development archive; no scientific acceptance"]
fn complete_gr1_development_smoke() {
    let out = std::env::var("ERE_COMPLETE_GR1_OUTPUT").expect("explicit fresh development output");
    campaign(Path::new(&out), true).unwrap();
}
#[test]
#[ignore = "requires separate scientific resource approval"]
fn complete_gr1_frozen_assay() {
    assert_eq!(
        std::env::var("ERE_COMPLETE_GR1_APPROVED_STEPS").as_deref(),
        Ok("1016800")
    );
    assert_eq!(
        std::env::var("ERE_COMPLETE_GR1_APPROVED_CAPS").as_deref(),
        Ok("3600s,4GiB,900s,1GiB,400k/50k")
    );
    let out = std::env::var("ERE_COMPLETE_GR1_OUTPUT").expect("explicit fresh scientific output");
    campaign(Path::new(&out), false).unwrap();
}
