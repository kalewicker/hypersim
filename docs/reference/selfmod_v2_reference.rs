// selfmod — global self-modification experiment (self-contained, hypersim-convention-compatible)
//
// Two disjoint label namespaces share one hypergraph store:
//   DATA space  (labels < RULE_BASE): the "world"
//   RULE space  (labels >= RULE_BASE): rule-subgraphs, first-class and mutable
//
// Per step:
//   1. decode all RuleRoot subgraphs -> Vec<Rule>            (canonical order; snapshotted)
//   2. object phase: match decoded rules against DATA, MaxParallel apply
//   3. meta phase:   fixed procedural mutation operators edit RULE-space subgraphs
//   4. record metrics
//
// v1 is deliberately selection-free (undirected rule drift) and uses a fixed meta-level;
// expressing meta-operators as matched meta-rules (full reflection) is the v2 strange loop.

use std::collections::{HashMap, HashSet};

// ---------- deterministic RNG (SplitMix64) ----------
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self { Rng(seed) }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 { if n == 0 { 0 } else { self.next_u64() % n } }
    fn f01(&mut self) -> f64 { (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64) }
    fn chance(&mut self, p: f64) -> bool { self.f01() < p }
}

// ---------- FNV-1a hashing over a u64 stream ----------
struct H(u64);
impl H {
    fn new() -> Self { H(0xcbf29ce484222325) }
    fn u64(&mut self, x: u64) { let b = x.to_le_bytes(); for &by in &b { self.0 ^= by as u64; self.0 = self.0.wrapping_mul(0x100000001b3); } }
    fn i64(&mut self, x: i64) { self.u64(x as u64) }
    fn finish(&self) -> u64 { self.0 }
}

// ---------- property values (hypersim §3) ----------
#[derive(Clone, Copy, Debug, PartialEq)]
enum PV { Int(i64), Float(f64), Bool(bool) }
impl PV {
    fn hash_into(&self, h: &mut H) {
        match self {
            PV::Int(v) => { h.u64(1); h.i64(*v); }
            PV::Float(v) => { h.u64(2); h.u64(v.to_bits()); }
            PV::Bool(v) => { h.u64(3); h.u64(*v as u64); }
        }
    }
    fn as_i64(&self) -> Option<i64> { if let PV::Int(v) = self { Some(*v) } else { None } }
    fn as_f64(&self) -> Option<f64> { if let PV::Float(v) = self { Some(*v) } else { None } }
    fn as_bool(&self) -> Option<bool> { if let PV::Bool(v) = self { Some(*v) } else { None } }
}

// ---------- label / key / role constants ----------
const RULE_BASE: u32 = 1000;
const RULE_ROOT: u32 = 1000;
const PNODE: u32 = 1003;
const GUARD: u32 = 1005;
const RELABEL: u32 = 1006;
const CREATE: u32 = 1007;
const REDGE: u32 = 1008;

// property keys
const K_PRIORITY: u32 = 1;
const K_NVARS: u32 = 2;
const K_VAR_ID: u32 = 3;
const K_LABEL_CONSTRAINT: u32 = 4;
const K_GVAR: u32 = 5;
const K_GKEY: u32 = 6;
const K_GOP: u32 = 7;      // 0 Gt, 1 Lt, 2 Eq
const K_GVAL: u32 = 8;
const K_NEW_LABEL: u32 = 9;
const K_CREATE_INDEX: u32 = 10;
const K_CREATE_LABEL: u32 = 11;
const K_DELETE_EDGE: u32 = 12;
const K_DELETE_VAR: u32 = 13;   // Int var id, or absent
const K_REF0: u32 = 14;
const K_REF1: u32 = 15;
const K_ENERGY: u32 = 100;      // data-node float property (guard target, GUI color dim by convention)
const K_RESOURCE: u32 = 200;    // RuleRoot token count (v2 conserved-resource substrate)

// edge roles (rule-space edge labels)
const HAS_PNODE: u32 = 2002;
const HAS_GUARD: u32 = 2004;
const HAS_RELABEL: u32 = 2005;
const HAS_CREATE: u32 = 2006;
const HAS_REDGE: u32 = 2007;

fn is_rule_label(l: u32) -> bool { l >= RULE_BASE }

// ---------- store ----------
#[derive(Clone)]
struct Node { label: u32, props: Vec<(u32, PV)> }
#[derive(Clone)]
struct Edge { ends: Vec<u32>, label: Option<u32> }

#[derive(Clone)]
struct Store {
    nodes: HashMap<u32, Node>,
    edges: HashMap<u32, Edge>,
    inc: HashMap<u32, HashSet<u32>>,           // node -> incident edge ids
    edge_index: HashMap<(Vec<u32>, Option<u32>), u32>, // (sorted ends, label) -> edge id (dedupe)
    next: u32,
}
impl Store {
    fn new() -> Self {
        Store { nodes: HashMap::new(), edges: HashMap::new(), inc: HashMap::new(),
                edge_index: HashMap::new(), next: 1 }
    }
    fn nid(&mut self) -> u32 { let id = self.next; self.next += 1; id }
    fn add_node(&mut self, label: u32, props: Vec<(u32, PV)>) -> u32 {
        let id = self.nid();
        self.nodes.insert(id, Node { label, props });
        self.inc.insert(id, HashSet::new());
        id
    }
    fn key(ends: &[u32], label: Option<u32>) -> (Vec<u32>, Option<u32>) {
        let mut e = ends.to_vec(); e.sort_unstable(); (e, label)
    }
    fn add_edge(&mut self, ends: Vec<u32>, label: Option<u32>) -> u32 {
        let k = Store::key(&ends, label);
        if let Some(&id) = self.edge_index.get(&k) { return id; }
        let id = self.nid();
        for &n in &ends { self.inc.entry(n).or_default().insert(id); }
        self.edges.insert(id, Edge { ends, label });
        self.edge_index.insert(k, id);
        id
    }
    fn set_prop(&mut self, node: u32, key: u32, val: PV) {
        if let Some(n) = self.nodes.get_mut(&node) {
            if let Some(slot) = n.props.iter_mut().find(|(k, _)| *k == key) { slot.1 = val; }
            else { n.props.push((key, val)); }
        }
    }
    fn del_prop(&mut self, node: u32, key: u32) {
        if let Some(n) = self.nodes.get_mut(&node) { n.props.retain(|(k, _)| *k != key); }
    }
    fn get_prop(&self, node: u32, key: u32) -> Option<PV> {
        self.nodes.get(&node).and_then(|n| n.props.iter().find(|(k, _)| *k == key).map(|(_, v)| *v))
    }
    fn set_label(&mut self, node: u32, label: u32) {
        if let Some(n) = self.nodes.get_mut(&node) { n.label = label; }
    }
    fn remove_edge(&mut self, id: u32) {
        if let Some(e) = self.edges.remove(&id) {
            let k = Store::key(&e.ends, e.label);
            self.edge_index.remove(&k);
            for n in &e.ends { if let Some(s) = self.inc.get_mut(n) { s.remove(&id); } }
        }
    }
    fn remove_node(&mut self, id: u32) {
        // dangling-edge cleanup: remove incident edges first
        if let Some(edges) = self.inc.remove(&id) {
            for e in edges { self.remove_edge(e); }
        }
        self.nodes.remove(&id);
    }
    fn label_of(&self, id: u32) -> Option<u32> { self.nodes.get(&id).map(|n| n.label) }
    fn is_data(&self, id: u32) -> bool { self.nodes.get(&id).map_or(false, |n| !is_rule_label(n.label)) }

    // data-space neighborhood (for WL): other endpoints of incident data edges
    fn data_neighbors(&self, id: u32) -> Vec<u32> {
        let mut out = Vec::new();
        if let Some(edges) = self.inc.get(&id) {
            for &e in edges {
                if let Some(edge) = self.edges.get(&e) {
                    if edge.label.is_some() { continue; } // data edges are unlabeled
                    for &n in &edge.ends { if n != id && self.is_data(n) { out.push(n); } }
                }
            }
        }
        out
    }
    fn data_node_ids_sorted(&self) -> Vec<u32> {
        let mut v: Vec<u32> = self.nodes.iter().filter(|(_, n)| !is_rule_label(n.label)).map(|(&id, _)| id).collect();
        v.sort_unstable(); v
    }
    fn data_edge_ids_sorted(&self) -> Vec<u32> {
        let mut v: Vec<u32> = self.edges.iter()
            .filter(|(_, e)| e.label.is_none() && e.ends.len() == 2 && e.ends[0] != e.ends[1]
                    && self.is_data(e.ends[0]) && self.is_data(e.ends[1]))
            .map(|(&id, _)| id).collect();
        v.sort_unstable(); v
    }
    fn data_counts(&self) -> (usize, usize) {
        let n = self.nodes.values().filter(|n| !is_rule_label(n.label)).count();
        let e = self.edges.values().filter(|e| e.label.is_none()
                    && e.ends.iter().all(|&x| self.is_data(x))).count();
        (n, e)
    }
    fn rule_roots(&self) -> Vec<u32> {
        let mut v: Vec<u32> = self.nodes.iter().filter(|(_, n)| n.label == RULE_ROOT).map(|(&id, _)| id).collect();
        v.sort_unstable(); v
    }
    fn children(&self, root: u32, role: u32) -> Vec<u32> {
        let mut out = Vec::new();
        if let Some(edges) = self.inc.get(&root) {
            let mut es: Vec<u32> = edges.iter().copied().collect(); es.sort_unstable();
            for e in es {
                if let Some(edge) = self.edges.get(&e) {
                    if edge.label == Some(role) {
                        for &n in &edge.ends { if n != root { out.push(n); } }
                    }
                }
            }
        }
        out
    }
    // full rule subgraph node set reachable from root over rule-space edges
    fn rule_subgraph(&self, root: u32) -> (Vec<u32>, Vec<u32>) {
        let mut seen = HashSet::new();
        let mut stack = vec![root];
        let mut nodes = Vec::new();
        let mut edge_ids = HashSet::new();
        while let Some(cur) = stack.pop() {
            if !seen.insert(cur) { continue; }
            nodes.push(cur);
            if let Some(edges) = self.inc.get(&cur) {
                for &e in edges {
                    if let Some(edge) = self.edges.get(&e) {
                        if edge.label.map_or(false, |l| l >= 2000) { // rule-space role edge
                            edge_ids.insert(e);
                            for &n in &edge.ends { if !seen.contains(&n) && self.nodes.get(&n).map_or(false, |nn| is_rule_label(nn.label)) { stack.push(n); } }
                        }
                    }
                }
            }
        }
        nodes.sort_unstable();
        let mut es: Vec<u32> = edge_ids.into_iter().collect(); es.sort_unstable();
        (nodes, es)
    }
}

// ---------- decoded rule ----------
#[derive(Clone, Debug)]
struct Guard { var: u8, key: u32, op: u8, val: PV }
#[derive(Clone, Debug)]
struct Rule {
    priority: i32,
    n_vars: u8,                    // 1 or 2
    var_label: [Option<u32>; 2],
    guard: Option<Guard>,
    relabel: Vec<(u8, u32)>,
    delete_edge: bool,
    delete_var: Option<u8>,
    create: Vec<u32>,              // labels, local indices n..n+len
    rhs_edges: Vec<[usize; 2]>,    // refs: 0->v0, 1->v1, 2->create0, 3->create1
}
impl Rule {
    fn has_edge(&self) -> bool { self.n_vars == 2 }
    fn class_hash(&self) -> u64 {
        let mut h = H::new();
        h.i64(self.priority as i64); h.u64(self.n_vars as u64);
        for v in &self.var_label { match v { Some(l) => { h.u64(1); h.u64(*l as u64); } None => h.u64(0) } }
        match &self.guard { Some(g) => { h.u64(1); h.u64(g.var as u64); h.u64(g.key as u64); h.u64(g.op as u64); g.val.hash_into(&mut h); } None => h.u64(0) }
        let mut rl = self.relabel.clone(); rl.sort();
        h.u64(rl.len() as u64); for (v, l) in rl { h.u64(v as u64); h.u64(l as u64); }
        h.u64(self.delete_edge as u64);
        match self.delete_var { Some(v) => { h.u64(1); h.u64(v as u64); } None => h.u64(0) }
        h.u64(self.create.len() as u64); for l in &self.create { h.u64(*l as u64); }
        let mut re = self.rhs_edges.clone(); re.sort();
        h.u64(re.len() as u64); for e in re { h.u64(e[0] as u64); h.u64(e[1] as u64); }
        h.finish()
    }
}

// ---------- encode a Rule into RULE-space subgraph ----------
fn install_rule(s: &mut Store, r: &Rule) -> u32 {
    let mut props = vec![
        (K_PRIORITY, PV::Int(r.priority as i64)),
        (K_NVARS, PV::Int(r.n_vars as i64)),
        (K_DELETE_EDGE, PV::Bool(r.delete_edge)),
    ];
    if let Some(v) = r.delete_var { props.push((K_DELETE_VAR, PV::Int(v as i64))); }
    let root = s.add_node(RULE_ROOT, props);
    for i in 0..r.n_vars as usize {
        if let Some(lab) = r.var_label[i] {
            let p = s.add_node(PNODE, vec![(K_VAR_ID, PV::Int(i as i64)), (K_LABEL_CONSTRAINT, PV::Int(lab as i64))]);
            s.add_edge(vec![root, p], Some(HAS_PNODE));
        }
    }
    if let Some(g) = &r.guard {
        let gn = s.add_node(GUARD, vec![(K_GVAR, PV::Int(g.var as i64)), (K_GKEY, PV::Int(g.key as i64)),
                                        (K_GOP, PV::Int(g.op as i64)), (K_GVAL, g.val)]);
        s.add_edge(vec![root, gn], Some(HAS_GUARD));
    }
    for (v, l) in &r.relabel {
        let rn = s.add_node(RELABEL, vec![(K_VAR_ID, PV::Int(*v as i64)), (K_NEW_LABEL, PV::Int(*l as i64))]);
        s.add_edge(vec![root, rn], Some(HAS_RELABEL));
    }
    for (k, l) in r.create.iter().enumerate() {
        let cn = s.add_node(CREATE, vec![(K_CREATE_INDEX, PV::Int(k as i64)), (K_CREATE_LABEL, PV::Int(*l as i64))]);
        s.add_edge(vec![root, cn], Some(HAS_CREATE));
    }
    for e in &r.rhs_edges {
        let en = s.add_node(REDGE, vec![(K_REF0, PV::Int(e[0] as i64)), (K_REF1, PV::Int(e[1] as i64))]);
        s.add_edge(vec![root, en], Some(HAS_REDGE));
    }
    root
}

// ---------- decode a RULE-space subgraph back into a Rule ----------
fn decode_rule(s: &Store, root: u32) -> Option<Rule> {
    let n_vars = s.get_prop(root, K_NVARS)?.as_i64()? as u8;
    if n_vars < 1 || n_vars > 2 { return None; }
    let priority = s.get_prop(root, K_PRIORITY)?.as_i64()? as i32;
    let delete_edge = s.get_prop(root, K_DELETE_EDGE).and_then(|p| p.as_bool()).unwrap_or(false);
    let delete_var = s.get_prop(root, K_DELETE_VAR).and_then(|p| p.as_i64()).map(|v| v as u8)
        .filter(|v| (*v as u8) < n_vars);
    let mut var_label = [None, None];
    for pn in s.children(root, HAS_PNODE) {
        let vid = s.get_prop(pn, K_VAR_ID).and_then(|p| p.as_i64()).unwrap_or(-1);
        let lab = s.get_prop(pn, K_LABEL_CONSTRAINT).and_then(|p| p.as_i64());
        if vid >= 0 && (vid as u8) < n_vars { if let Some(l) = lab { var_label[vid as usize] = Some(l as u32); } }
    }
    let guard = s.children(root, HAS_GUARD).first().and_then(|&gn| {
        Some(Guard {
            var: s.get_prop(gn, K_GVAR)?.as_i64()? as u8,
            key: s.get_prop(gn, K_GKEY)?.as_i64()? as u32,
            op: s.get_prop(gn, K_GOP)?.as_i64()? as u8,
            val: s.get_prop(gn, K_GVAL)?,
        })
    }).filter(|g| g.var < n_vars);
    let mut relabel = Vec::new();
    for rn in s.children(root, HAS_RELABEL) {
        let v = s.get_prop(rn, K_VAR_ID).and_then(|p| p.as_i64());
        let l = s.get_prop(rn, K_NEW_LABEL).and_then(|p| p.as_i64());
        if let (Some(v), Some(l)) = (v, l) { if (v as u8) < n_vars { relabel.push((v as u8, l as u32)); } }
    }
    // creates, ordered by create_index
    let mut cvec: Vec<(i64, u32)> = Vec::new();
    for cn in s.children(root, HAS_CREATE) {
        let k = s.get_prop(cn, K_CREATE_INDEX).and_then(|p| p.as_i64()).unwrap_or(0);
        let l = s.get_prop(cn, K_CREATE_LABEL).and_then(|p| p.as_i64()).unwrap_or(0);
        cvec.push((k, l as u32));
    }
    cvec.sort_by_key(|x| x.0);
    let create: Vec<u32> = cvec.into_iter().map(|x| x.1).collect();
    if create.len() > 2 { return None; }
    let n_refs = n_vars as usize + create.len();
    let mut rhs_edges = Vec::new();
    for en in s.children(root, HAS_REDGE) {
        let a = s.get_prop(en, K_REF0).and_then(|p| p.as_i64());
        let b = s.get_prop(en, K_REF1).and_then(|p| p.as_i64());
        if let (Some(a), Some(b)) = (a, b) {
            let (a, b) = (a as usize, b as usize);
            // ref must be a surviving var (0..n_vars, not deleted) or a created node
            let ok = |r: usize| -> bool {
                if r < n_vars as usize { delete_var.map_or(true, |dv| dv as usize != r) }
                else { r < n_refs }
            };
            if a != b && ok(a) && ok(b) { rhs_edges.push([a, b]); }
        }
    }
    Some(Rule { priority, n_vars, var_label, guard, relabel, delete_edge, delete_var, create, rhs_edges })
}

fn decode_all(s: &Store) -> Vec<(u32, Rule)> {
    let mut out = Vec::new();
    for root in s.rule_roots() {
        if let Some(r) = decode_rule(s, root) { out.push((root, r)); }
    }
    out
}

// ---------- matching (DATA space; injective; connected patterns only) ----------
struct Match { vmap: [u32; 2], nv: u8, edge: Option<u32> }

fn guard_ok(s: &Store, node: u32, g: &Guard) -> bool {
    let v = match s.get_prop(node, g.key) { Some(v) => v, None => return false };
    match (v, g.val) {
        (PV::Float(a), PV::Float(b)) => match g.op { 0 => a > b, 1 => a < b, _ => a == b },
        (PV::Int(a), PV::Int(b)) => match g.op { 0 => a > b, 1 => a < b, _ => a == b },
        _ => false,
    }
}

fn find_matches(s: &Store, r: &Rule, dnodes: &[u32], dedges: &[u32]) -> Vec<Match> {
    let mut out = Vec::new();
    if r.n_vars == 1 {
        for &d in dnodes {
            if let Some(lc) = r.var_label[0] { if s.label_of(d) != Some(lc) { continue; } }
            if let Some(g) = &r.guard { if g.var == 0 && !guard_ok(s, d, g) { continue; } }
            out.push(Match { vmap: [d, 0], nv: 1, edge: None });
        }
    } else {
        for &e in dedges {
            let ends = &s.edges[&e].ends;
            let (a, b) = (ends[0], ends[1]);
            for &(v0, v1) in &[(a, b), (b, a)] {
                if let Some(lc) = r.var_label[0] { if s.label_of(v0) != Some(lc) { continue; } }
                if let Some(lc) = r.var_label[1] { if s.label_of(v1) != Some(lc) { continue; } }
                if let Some(g) = &r.guard {
                    let tv = if g.var == 0 { v0 } else { v1 };
                    if !guard_ok(s, tv, g) { continue; }
                }
                out.push(Match { vmap: [v0, v1], nv: 2, edge: Some(e) });
            }
        }
    }
    out
}

fn bound_nodes(m: &Match) -> Vec<u32> {
    if m.nv == 1 { vec![m.vmap[0]] } else { vec![m.vmap[0], m.vmap[1]] }
}

fn apply_match(s: &mut Store, r: &Rule, m: &Match, rng: &mut Rng) {
    if r.delete_edge { if let Some(e) = m.edge { s.remove_edge(e); } }
    // relabels on survivors (before deletion so refs stay valid); skip deleted var
    for (v, l) in &r.relabel {
        if r.delete_var == Some(*v) { continue; }
        if (*v as usize) < r.n_vars as usize { s.set_label(m.vmap[*v as usize], *l); }
    }
    if let Some(dv) = r.delete_var {
        if (dv as usize) < r.n_vars as usize { s.remove_node(m.vmap[dv as usize]); }
    }
    // create RHS nodes
    let mut created = Vec::new();
    for lab in &r.create {
        let e = rng.f01();
        let id = s.add_node(*lab, vec![(K_ENERGY, PV::Float(e))]);
        created.push(id);
    }
    // resolve a ref (local index) to a concrete data node id, if valid
    let resolve = |r: &Rule, m: &Match, created: &Vec<u32>, refi: usize| -> Option<u32> {
        if refi < r.n_vars as usize {
            if r.delete_var == Some(refi as u8) { return None; }
            Some(m.vmap[refi])
        } else {
            created.get(refi - r.n_vars as usize).copied()
        }
    };
    for e in &r.rhs_edges {
        if let (Some(x), Some(y)) = (resolve(r, m, &created, e[0]), resolve(r, m, &created, e[1])) {
            if x != y { s.add_edge(vec![x, y], None); }
        }
    }
}

// object phase: gather candidates across all rules, MaxParallel conflict-resolve by priority, apply.
// Carrying-capacity clamp: once data nodes reach `ceiling`, node-creating rules are gated off
// (density-dependent regulation) so the substrate saturates instead of exploding.
fn object_step(s: &mut Store, rules: &[(u32, Rule)], rng: &mut Rng, ceiling: usize) -> (usize, HashSet<u32>) {
    let (data_n, _) = s.data_counts();
    let allow_create = data_n < ceiling;
    let dnodes = s.data_node_ids_sorted();   // computed ONCE per step, not per rule
    let dedges = s.data_edge_ids_sorted();
    // candidates: (priority, rule_idx, tiebreak sorted bound nodes, match)
    let mut cands: Vec<(i32, usize, Vec<u32>, Match)> = Vec::new();
    for (ri, (_, r)) in rules.iter().enumerate() {
        if !allow_create && !r.create.is_empty() { continue; } // gated at carrying capacity
        for m in find_matches(s, r, &dnodes, &dedges) {
            let mut bn = bound_nodes(&m); bn.sort_unstable();
            cands.push((r.priority, ri, bn, m));
        }
    }
    // sort: higher priority first, then rule idx, then bound-node tiebreak (deterministic)
    cands.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut used: HashSet<u32> = HashSet::new();
    let mut applied = 0usize;
    let mut fired: HashSet<u32> = HashSet::new();
    for (_, ri, bn, m) in &cands {
        if bn.iter().any(|n| used.contains(n)) { continue; }
        for &n in bn { used.insert(n); }
        apply_match(s, &rules[*ri].1, m, rng);
        fired.insert(rules[*ri].0); // record the RuleRoot id that fired
        applied += 1;
    }
    (applied, fired)
}

// ---------- meta phase: procedural mutation operators over RULE space ----------
fn meta_step(s: &mut Store, rng: &mut Rng, rule_cap: usize, p_dup: f64, p_del: f64) -> usize {
    let roots = s.rule_roots();
    if roots.is_empty() { return 0; }
    let root = roots[rng.below(roots.len() as u64) as usize];
    // choose operator
    let roll = rng.f01();
    let mut applied = 0usize;
    if roll < p_dup {
        if roots.len() < rule_cap { duplicate_rule(s, root); applied = 1; }
    } else if roll < p_dup + p_del {
        delete_rule(s, root); applied = 1;
    } else {
        // content mutation on this rule
        applied = mutate_content(s, root, rng);
    }
    applied
}

fn duplicate_rule(s: &mut Store, root: u32) {
    let (nodes, edges) = s.rule_subgraph(root);
    let mut remap: HashMap<u32, u32> = HashMap::new();
    for n in &nodes {
        let node = s.nodes[n].clone();
        let id = s.add_node(node.label, node.props);
        remap.insert(*n, id);
    }
    for e in &edges {
        let edge = s.edges[e].clone();
        let ends: Vec<u32> = edge.ends.iter().map(|x| *remap.get(x).unwrap_or(x)).collect();
        s.add_edge(ends, edge.label);
    }
}

fn delete_rule(s: &mut Store, root: u32) {
    let (nodes, _) = s.rule_subgraph(root);
    for n in nodes { s.remove_node(n); }
}

fn mutate_content(s: &mut Store, root: u32, rng: &mut Rng) -> usize {
    // collect candidate targets
    let guards = s.children(root, HAS_GUARD);
    let creates = s.children(root, HAS_CREATE);
    let relabels = s.children(root, HAS_RELABEL);
    let pnodes = s.children(root, HAS_PNODE);
    let mut ops: Vec<u8> = Vec::new();
    if !guards.is_empty() { ops.push(0); }
    if !creates.is_empty() { ops.push(1); }
    if !relabels.is_empty() { ops.push(2); }
    ops.push(3); // toggle/alter a var constraint (works even if none: may add one)
    if ops.is_empty() { return 0; }
    let op = ops[rng.below(ops.len() as u64) as usize];
    match op {
        0 => { // jitter guard threshold
            let gn = guards[rng.below(guards.len() as u64) as usize];
            if let Some(v) = s.get_prop(gn, K_GVAL) {
                let nv = match v {
                    PV::Float(x) => PV::Float(x + (rng.f01() - 0.5) * 0.2),
                    PV::Int(x) => PV::Int(x + (rng.below(3) as i64 - 1)),
                    b => b,
                };
                s.set_prop(gn, K_GVAL, nv);
                return 1;
            }
        }
        1 => { // mutate a created node's label
            let cn = creates[rng.below(creates.len() as u64) as usize];
            s.set_prop(cn, K_CREATE_LABEL, PV::Int(rng.below(4) as i64));
            return 1;
        }
        2 => { // mutate a relabel target label
            let rn = relabels[rng.below(relabels.len() as u64) as usize];
            s.set_prop(rn, K_NEW_LABEL, PV::Int(rng.below(4) as i64));
            return 1;
        }
        3 => { // alter/remove/add a var label constraint
            let nvars = s.get_prop(root, K_NVARS).and_then(|p| p.as_i64()).unwrap_or(1);
            let vid = rng.below(nvars.max(1) as u64) as i64;
            // find existing pnode for vid
            let existing = pnodes.iter().find(|&&pn| s.get_prop(pn, K_VAR_ID).and_then(|p| p.as_i64()) == Some(vid)).copied();
            let action = rng.below(3);
            match (existing, action) {
                (Some(pn), 0) => { s.remove_node(pn); } // drop constraint
                (Some(pn), _) => { s.set_prop(pn, K_LABEL_CONSTRAINT, PV::Int(rng.below(4) as i64)); } // change
                (None, _) => { // add
                    let pn = s.add_node(PNODE, vec![(K_VAR_ID, PV::Int(vid)), (K_LABEL_CONSTRAINT, PV::Int(rng.below(4) as i64))]);
                    s.add_edge(vec![root, pn], Some(HAS_PNODE));
                }
            }
            return 1;
        }
        _ => {}
    }
    0
}

// ---------- metrics ----------
fn wl_classes(s: &Store, rounds: usize) -> HashSet<u64> {
    let ids = s.data_node_ids_sorted();
    let mut color: HashMap<u32, u64> = HashMap::new();
    for &id in &ids {
        let mut h = H::new();
        h.u64(s.label_of(id).unwrap_or(0) as u64);
        // include sorted properties (energy bucketed to keep WL from exploding on floats)
        if let Some(n) = s.nodes.get(&id) {
            let mut ps: Vec<(u32, u64)> = n.props.iter().map(|(k, v)| {
                let vb = match v { PV::Float(f) => (f * 10.0).floor() as i64 as u64, PV::Int(i) => *i as u64, PV::Bool(b) => *b as u64 };
                (*k, vb)
            }).collect();
            ps.sort();
            for (k, vb) in ps { h.u64(k as u64); h.u64(vb); }
        }
        color.insert(id, h.finish());
    }
    for _ in 0..rounds {
        let mut nc: HashMap<u32, u64> = HashMap::new();
        for &id in &ids {
            let mut ncol: Vec<u64> = s.data_neighbors(id).iter().map(|n| color[n]).collect();
            ncol.sort_unstable();
            let mut h = H::new();
            h.u64(color[&id]);
            for c in ncol { h.u64(c); }
            nc.insert(id, h.finish());
        }
        color = nc;
    }
    color.values().copied().collect()
}

fn data_canon_hash(s: &Store) -> u64 {
    // canonical over DATA nodes/edges only (hypersim §10 style):
    // key each node by (label, sorted props), sort keys -> canonical index, remap+sort edges, hash.
    let mut keyed: Vec<(u64, u32)> = s.nodes.iter()
        .filter(|(_, n)| !is_rule_label(n.label))
        .map(|(&id, n)| {
            let mut h = H::new(); h.u64(n.label as u64);
            let mut p = n.props.clone(); p.sort_by_key(|x| x.0);
            for (k, v) in &p { h.u64(*k as u64); v.hash_into(&mut h); }
            (h.finish(), id)
        }).collect();
    keyed.sort();
    let mut index: HashMap<u32, usize> = HashMap::new();
    for (i, (_, id)) in keyed.iter().enumerate() { index.insert(*id, i); }
    let mut edge_keys: Vec<Vec<usize>> = Vec::new();
    for (_, e) in s.edges.iter() {
        if e.label.is_some() { continue; }
        if !e.ends.iter().all(|x| s.is_data(*x)) { continue; }
        let mut ix: Vec<usize> = e.ends.iter().filter_map(|x| index.get(x).copied()).collect();
        if ix.len() != e.ends.len() { continue; }
        ix.sort_unstable();
        edge_keys.push(ix);
    }
    edge_keys.sort();
    let mut h = H::new();
    for (k, _) in &keyed { h.u64(*k); }
    h.u64(0xffff_ffff);
    for ek in &edge_keys { for i in ek { h.u64(*i as u64); } h.u64(0xfffe); }
    h.finish()
}

// ---------- seed world & rules ----------
fn seed_world(s: &mut Store, rng: &mut Rng, n_nodes: usize, n_edges: usize) {
    let mut ids = Vec::new();
    for _ in 0..n_nodes {
        let lab = rng.below(4) as u32;
        let e = rng.f01();
        ids.push(s.add_node(lab, vec![(K_ENERGY, PV::Float(e))]));
    }
    let mut placed = 0;
    let mut tries = 0;
    while placed < n_edges && tries < n_edges * 10 {
        tries += 1;
        let a = ids[rng.below(ids.len() as u64) as usize];
        let b = ids[rng.below(ids.len() as u64) as usize];
        if a == b { continue; }
        let before = s.edges.len();
        s.add_edge(vec![a, b], None);
        if s.edges.len() > before { placed += 1; }
    }
}

fn seed_rules(s: &mut Store) {
    // G: label0 with energy>0.7 spawns a label1 neighbor (throttled growth)
    install_rule(s, &Rule {
        priority: 0, n_vars: 1, var_label: [Some(0), None],
        guard: Some(Guard { var: 0, key: K_ENERGY, op: 0, val: PV::Float(0.7) }),
        relabel: vec![], delete_edge: false, delete_var: None,
        create: vec![1], rhs_edges: vec![[0, 1]],
    });
    // R: label1 -> label2
    install_rule(s, &Rule {
        priority: 0, n_vars: 1, var_label: [Some(1), None],
        guard: None, relabel: vec![(0, 2)], delete_edge: false, delete_var: None,
        create: vec![], rhs_edges: vec![],
    });
    // P: label2--label2 edge pruned; one endpoint recycled to label0
    install_rule(s, &Rule {
        priority: 1, n_vars: 2, var_label: [Some(2), Some(2)],
        guard: None, relabel: vec![(0, 0)], delete_edge: true, delete_var: None,
        create: vec![], rhs_edges: vec![],
    });
    // D: label3--label3 edge deletes one endpoint (population sink)
    install_rule(s, &Rule {
        priority: 1, n_vars: 2, var_label: [Some(3), Some(3)],
        guard: None, relabel: vec![], delete_edge: false, delete_var: Some(1),
        create: vec![], rhs_edges: vec![],
    });
}

// ---------- runs ----------
fn run_experiment(seed: u64, steps: usize, log_every: usize, p_meta: f64,
                  p_dup: f64, p_del: f64, node_cap: usize, rule_cap: usize, node_ceiling: usize, path: &str) {
    let mut rng = Rng::new(seed);
    let mut s = Store::new();
    seed_world(&mut s, &mut rng, 200, 300);
    seed_rules(&mut s);

    let mut rule_reg: HashSet<u64> = HashSet::new();
    let mut data_reg: HashSet<u64> = HashSet::new();
    let mut csv = String::from("step,data_nodes,data_edges,rule_count,rules_ok,M_rules,M_data,obj_applied,meta_applied,data_hash\n");
    let mut obj_acc = 0usize;
    let mut meta_acc = 0usize;
    let mut term = "budget";

    for step in 0..=steps {
        // decode + snapshot ruleset
        let rules = decode_all(&s);
        for (_, r) in &rules { rule_reg.insert(r.class_hash()); }
        for c in wl_classes(&s, 3) { data_reg.insert(c); }

        if step % log_every == 0 {
            let (dn, de) = s.data_counts();
            let rc = s.rule_roots().len();
            let hash = data_canon_hash(&s);
            csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{}\n",
                step, dn, de, rc, rules.len(), rule_reg.len(), data_reg.len(),
                obj_acc, meta_acc, hash));
            obj_acc = 0; meta_acc = 0;
        }

        if step == steps { break; }

        // object phase
        obj_acc += object_step(&mut s, &rules, &mut rng, node_ceiling).0;
        // meta phase
        if rng.chance(p_meta) {
            meta_acc += meta_step(&mut s, &mut rng, rule_cap, p_dup, p_del);
        }

        // safety caps -> regime terminations
        let (dn, _) = s.data_counts();
        if dn > node_cap { term = "explosion_nodes"; log_final(&mut csv, step, &s, &rule_reg, &data_reg); break; }
        if s.rule_roots().len() > rule_cap { term = "explosion_rules"; log_final(&mut csv, step, &s, &rule_reg, &data_reg); break; }
        if s.rule_roots().is_empty() { term = "collapse_no_rules"; log_final(&mut csv, step, &s, &rule_reg, &data_reg); break; }
    }

    std::fs::write(path, csv).unwrap();
    // summary to stdout
    let (dn, de) = s.data_counts();
    println!("SEED {} | term={} | final data=({},{}) rules={} rules_ok={} M_rules={} M_data={}",
        seed, term, dn, de, s.rule_roots().len(), decode_all(&s).len(), rule_reg.len(), data_reg.len());
}

fn log_final(csv: &mut String, step: usize, s: &Store, rule_reg: &HashSet<u64>, data_reg: &HashSet<u64>) {
    let (dn, de) = s.data_counts();
    csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{}\n",
        step, dn, de, s.rule_roots().len(), decode_all(s).len(), rule_reg.len(), data_reg.len(), 0, 0, data_canon_hash(s)));
}

// ---------- reification-neutrality gate ----------
fn run_gate(seed: u64, steps: usize) {
    let mut rng = Rng::new(seed);
    let mut base = Store::new();
    seed_world(&mut base, &mut rng, 120, 180);

    // the rule under test (label0 -> label1, plus spawn) — deterministic, no RNG in apply-create energy?
    // apply_match draws rng.f01() for created-node energy, so both paths must share an identical RNG stream.
    let test = Rule {
        priority: 0, n_vars: 1, var_label: [Some(0), None], guard: None,
        relabel: vec![(0, 1)], delete_edge: false, delete_var: None,
        create: vec![2], rhs_edges: vec![[0, 1]],
    };

    // DIRECT path
    let mut sd = base.clone();
    let mut rng_d = Rng::new(seed ^ 0xABCD);
    let mut hd = Vec::new();
    for _ in 0..steps {
        let rules = vec![(0u32, test.clone())];
        let _ = object_step(&mut sd, &rules, &mut rng_d, 100000);
        hd.push(data_canon_hash(&sd));
    }

    // ENCODED path — identical RNG stream, rule lives in the graph, decoded each step
    let mut se = base.clone();
    install_rule(&mut se, &test);
    let mut rng_e = Rng::new(seed ^ 0xABCD);
    let mut he = Vec::new();
    for _ in 0..steps {
        let rules = decode_all(&se);      // decode from graph
        let _ = object_step(&mut se, &rules, &mut rng_e, 100000);
        he.push(data_canon_hash(&se));    // DATA-only hash excludes the rule subgraph
    }

    let mut first_div = None;
    for i in 0..steps { if hd[i] != he[i] { first_div = Some(i); break; } }
    match first_div {
        None => println!("GATE PASS: data_canon_hash identical for all {} steps (decode+apply == direct apply)", steps),
        Some(i) => println!("GATE FAIL: first divergence at step {} (direct={:#x} encoded={:#x})", i, hd[i], he[i]),
    }
}

// ============================================================================
// v2: conserved-resource substrate (Phase A regulation, Phase B selection)
// Invariant asserted every step:  free + Σ_rules tokens == total
// ============================================================================

fn rule_tokens(s: &Store, root: u32) -> i64 { s.get_prop(root, K_RESOURCE).and_then(|p| p.as_i64()).unwrap_or(0) }
fn set_tokens(s: &mut Store, root: u32, t: i64) { s.set_prop(root, K_RESOURCE, PV::Int(t)); }
fn total_bound(s: &Store) -> i64 { s.rule_roots().iter().map(|&r| rule_tokens(s, r)).sum() }

fn seed_rules_endowed(s: &mut Store, e0: i64) {
    seed_rules(s);
    for r in s.rule_roots() { set_tokens(s, r, e0); }
}

// duplicate a rule then apply exactly one content mutation to the CHILD (variation-at-birth).
// returns the new child's RuleRoot id.
fn fission_mutated(s: &mut Store, parent: u32, rng: &mut Rng) -> u32 {
    let before: HashSet<u32> = s.rule_roots().into_iter().collect();
    duplicate_rule(s, parent);
    let child = *s.rule_roots().iter().find(|r| !before.contains(r)).unwrap();
    mutate_content(s, child, rng); // one mutation on the offspring
    child
}

struct ResParams {
    total: i64, e0: i64, birth_cost: i64,
    rent: i64, fire_reward: i64, repro_threshold: i64,
    p_dup: f64, p_del: f64, abio: f64,
}

// ---------- Phase A: resource-regulated, fitness-blind birth/death ----------
fn meta_step_A(s: &mut Store, rng: &mut Rng, free: &mut i64, rule_cap: usize, pr: &ResParams) -> (u32, u32) {
    let roots = s.rule_roots();
    let mut births = 0u32; let mut deaths = 0u32;
    // optional abiogenesis: chance ∝ free/total
    if pr.abio > 0.0 && *free >= pr.birth_cost && rng.chance(pr.abio * (*free as f64 / pr.total as f64)) {
        // assemble a random minimal rule (1-var relabel) from free
        let root = install_rule(s, &Rule {
            priority: 0, n_vars: 1, var_label: [Some(rng.below(4) as u32), None], guard: None,
            relabel: vec![(0, rng.below(4) as u32)], delete_edge: false, delete_var: None,
            create: vec![], rhs_edges: vec![],
        });
        set_tokens(s, root, pr.birth_cost); *free -= pr.birth_cost; births += 1;
    }
    if roots.is_empty() { return (births, deaths); }
    let root = roots[rng.below(roots.len() as u64) as usize];
    let roll = rng.f01();
    if roll < pr.p_dup {
        if s.rule_roots().len() < rule_cap && *free >= pr.birth_cost {
            let before: HashSet<u32> = s.rule_roots().into_iter().collect(); // FRESH (post-abiogenesis)
            duplicate_rule(s, root);
            let child = *s.rule_roots().iter().find(|r| !before.contains(r)).unwrap();
            set_tokens(s, child, pr.birth_cost);  // explicit — do NOT inherit parent's tokens
            *free -= pr.birth_cost; births += 1;
        }
    } else if roll < pr.p_dup + pr.p_del {
        *free += rule_tokens(s, root);           // return tokens to pool
        delete_rule(s, root); deaths += 1;
    } else {
        mutate_content(s, root, rng);            // free
    }
    (births, deaths)
}

// ---------- Phase B: metabolism (rent / reward / starvation / funded fission) ----------
fn metabolic_step(s: &mut Store, fired: &HashSet<u32>, free: &mut i64, rng: &mut Rng,
                  rule_cap: usize, pr: &ResParams) -> (u32, u32) {
    let mut births = 0u32; let mut deaths = 0u32;
    // 1. rent + starvation
    for root in s.rule_roots() {
        let t = rule_tokens(s, root);
        if t < pr.rent { *free += t; delete_rule(s, root); deaths += 1; }
        else { set_tokens(s, root, t - pr.rent); *free += pr.rent; }
    }
    // 2. reward rules that fired (flat per rule per step, capped by free)
    for &root in fired {
        if s.nodes.contains_key(&root) {                // may have starved above
            let pay = pr.fire_reward.min(*free);
            if pay > 0 { set_tokens(s, root, rule_tokens(s, root) + pay); *free -= pay; }
        }
    }
    // 3. funded fission of the rich (deterministic, self-limiting; mutate-at-birth)
    for root in s.rule_roots() {
        if s.rule_roots().len() >= rule_cap { break; }
        if rule_tokens(s, root) >= pr.repro_threshold {
            let child = fission_mutated(s, root, rng);
            set_tokens(s, root, rule_tokens(s, root) - pr.birth_cost); // parent funds
            set_tokens(s, child, pr.birth_cost);
            births += 1;
        }
    }
    (births, deaths)
}

fn run_phase(phase: char, seed: u64, steps: usize, pr: ResParams,
             node_ceiling: usize, rule_cap: usize, log_every: usize, path: &str) {
    let mut rng = Rng::new(seed);
    let mut s = Store::new();
    seed_world(&mut s, &mut rng, 200, 300);
    seed_rules_endowed(&mut s, pr.e0);
    let mut free: i64 = pr.total - total_bound(&s);

    let mut rule_reg: HashSet<u64> = HashSet::new();
    let mut data_reg: HashSet<u64> = HashSet::new();
    let mut csv = String::from(
        "step,data_nodes,data_edges,rule_count,M_rules,M_data,obj_applied,free,mean_tokens,births,deaths,frac_fired,data_hash\n");
    let (mut obj_acc, mut b_acc, mut d_acc) = (0usize, 0u32, 0u32);
    let mut frac_fired_last = 0.0f64;
    let mut term = "budget";

    for step in 0..=steps {
        let rules = decode_all(&s);
        for (_, r) in &rules { rule_reg.insert(r.class_hash()); }

        if step % log_every == 0 {
            for c in wl_classes(&s, 3) { data_reg.insert(c); }  // sampled at log cadence (perf)
            let (dn, de) = s.data_counts();
            let rc = s.rule_roots().len();
            let mean_tok = if rc > 0 { total_bound(&s) as f64 / rc as f64 } else { 0.0 };
            csv.push_str(&format!("{},{},{},{},{},{},{},{},{:.2},{},{},{:.3},{}\n",
                step, dn, de, rc, rule_reg.len(), data_reg.len(), obj_acc, free, mean_tok,
                b_acc, d_acc, frac_fired_last, data_canon_hash(&s)));
            obj_acc = 0; b_acc = 0; d_acc = 0;
        }
        if step == steps { break; }

        // object phase
        let (applied, fired) = object_step(&mut s, &rules, &mut rng, node_ceiling);
        obj_acc += applied;
        let rc_now = s.rule_roots().len();
        frac_fired_last = if rc_now > 0 { fired.len() as f64 / rc_now as f64 } else { 0.0 };

        // meta phase (phase-specific)
        let (b, d) = match phase {
            'A' => meta_step_A(&mut s, &mut rng, &mut free, rule_cap, &pr),
            _   => metabolic_step(&mut s, &fired, &mut free, &mut rng, rule_cap, &pr),
        };
        b_acc += b; d_acc += d;

        // conservation invariant
        debug_assert_eq!(free + total_bound(&s), pr.total, "resource conservation violated at step {}", step);
        if free + total_bound(&s) != pr.total { term = "CONSERVATION_BUG"; break; }

        let (dn, _) = s.data_counts();
        if dn > 20000 { term = "explosion_nodes"; break; }
        if s.rule_roots().is_empty() { term = "collapse_no_rules"; break; }
    }

    std::fs::write(path, csv).unwrap();
    let (dn, de) = s.data_counts();
    let rc = s.rule_roots().len();
    println!("PHASE {} SEED {} | term={} | data=({},{}) rules={} M_rules={} M_data={} free={}",
        phase, seed, term, dn, de, rc, rule_reg.len(), data_reg.len(), free);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("run");
    let f = |i: usize, d: f64| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let n = |i: usize, d: i64| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    match mode {
        "gate" => {
            let seed = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(42);
            let steps = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(60);
            run_gate(seed, steps);
        }
        // phaseA <seed> <steps> <birth_cost> <abio> <path>
        "phaseA" => {
            let seed = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(42);
            let steps = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(20000);
            let pr = ResParams {
                total: 2000, e0: 60, birth_cost: n(4, 40),
                rent: 0, fire_reward: 0, repro_threshold: 0,
                p_dup: 0.40, p_del: 0.40, abio: f(5, 0.0),
            };
            let path = args.get(6).cloned().unwrap_or_else(|| format!("A_seed{}.csv", seed));
            run_phase('A', seed, steps, pr, 500, 400, 100, &path);
        }
        // phaseB <seed> <steps> <rent> <fire_reward> <e0> <repro_threshold> <birth_cost> <path>
        "phaseB" => {
            let seed = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(42);
            let steps = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(20000);
            let pr = ResParams {
                total: 2000, e0: n(6, 60), birth_cost: n(8, 50),
                rent: n(4, 1), fire_reward: n(5, 2), repro_threshold: n(7, 100),
                p_dup: 0.0, p_del: 0.0, abio: 0.0,
            };
            let path = args.get(9).cloned().unwrap_or_else(|| format!("B_seed{}.csv", seed));
            let le: usize = args.get(10).and_then(|s| s.parse().ok()).unwrap_or(100);
            run_phase('B', seed, steps, pr, 500, 400, le, &path);
        }
        _ => {
            let seed = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(42);
            let steps = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(20000);
            let p_dup: f64 = f(4, 0.30);
            let p_del: f64 = f(5, 0.30);
            let path = args.get(6).cloned().unwrap_or_else(|| format!("out_seed{}.csv", seed));
            let rule_cap: usize = args.get(7).and_then(|s| s.parse().ok()).unwrap_or(200);
            run_experiment(seed, steps, 100, 0.5, p_dup, p_del, 20000, rule_cap, 1500, &path);
        }
    }
}
