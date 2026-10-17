//! Isolated atomic host-assay edits. No codec regeneration, RNG or engine step.
use hg_core::{EdgeId, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_meta::*;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const PROFILE: &str = "E2-complete-coherent-kernel-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Role {
    Core,
    Proposal,
}

#[derive(Clone, Debug, Serialize)]
pub enum Request {
    AddSlot { pattern: NodeId, variable: NodeId },
    RemoveSlot { slot: EdgeId },
    RetargetSlot { slot: EdgeId, variable: NodeId },
    AddVar { pattern: NodeId },
    RemoveVar { variable: NodeId },
    AddPatternEdge { side: NodeId, a: NodeId, b: NodeId },
    RemovePatternEdge { pattern: NodeId },
    AddLabel { variable: NodeId, value: u32 },
    RemoveLabel { variable: NodeId },
    ChangeLabel { variable: NodeId, value: u32 },
    ChangePriority,
    Activate,
    Deactivate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Applied,
    Inapplicable,
    RejectedInput,
    RejectedPolicy,
    RejectedPostcondition,
}

pub struct Attempt {
    pub status: Status,
    pub receipt: Value,
    /// Includes rejected, uncommitted material. Never substitute parent as child.
    pub staged: HypergraphStore,
    pub before_program: Option<RuleProgram>,
    pub after_program: Option<RuleProgram>,
}

#[derive(Debug)]
struct Failure(Status, String, Option<Validator>);
type Res<T> = Result<T, Failure>;
fn refuse(reason: &str) -> Failure {
    Failure(Status::RejectedPolicy, reason.into(), None)
}
fn need(ok: bool, reason: &str) -> Res<()> {
    if ok {
        Ok(())
    } else {
        Err(refuse(reason))
    }
}
fn stable<T: Serialize>(id: T) -> (u64, u64) {
    let v = serde_json::to_value(id).unwrap();
    (v["idx"].as_u64().unwrap(), v["version"].as_u64().unwrap())
}
fn int(s: &HypergraphStore, n: NodeId, key: u32) -> Option<i64> {
    match s.get_node(n)?.get_property(key)? {
        PropertyValue::Int(v) => Some(v),
        _ => None,
    }
}
fn edge(s: &HypergraphStore, id: EdgeId) -> Res<HyperEdge> {
    s.get_edge(id).cloned().ok_or_else(|| refuse("stale edge"))
}

#[derive(Clone)]
pub struct Scope {
    pub anchor: NodeId,
    pub left: NodeId,
    pub right: NodeId,
    pub patterns: Vec<NodeId>,
    pub variables: Vec<NodeId>,
    pub body: BTreeSet<NodeId>,
}
impl Scope {
    fn load(s: &HypergraphStore, root: NodeId, role: Role, limits: &MetaLimits) -> Res<Self> {
        let whole = walk_family(&s.snapshot(), root, limits)
            .map_err(|v| Failure(Status::RejectedInput, "region".into(), Some(v)))?;
        for id in &whole.edges {
            let e = edge(s, *id)?;
            if WITNESS_ROLES.contains(&e.label.0) {
                need(e.arity() == 2, "sealed payload is outside this host kernel")?;
            }
        }
        for id in &whole.nodes {
            let n = s.get_node(*id).unwrap();
            if n.label.0 == L_VAR {
                need(
                    int(s, *id, K_ORDER).is_some_and(|v| (0..8).contains(&v)),
                    "missing variable order hint",
                )?;
            }
            if n.label.0 == L_PAT_EDGE {
                need(
                    int(s, *id, K_ORDER).is_some_and(|v| v >= 0),
                    "missing pattern order hint",
                )?;
            }
        }
        let label = if role == Role::Core { E_CORE } else { E_GRAFT };
        let anchor = whole
            .edges
            .iter()
            .filter_map(|id| s.get_edge(*id))
            .find(|e| e.label.0 == label && e.nodes[0] == root)
            .ok_or_else(|| refuse("missing allele"))?
            .nodes[1];
        let sides = whole
            .edges
            .iter()
            .filter_map(|id| s.get_edge(*id))
            .find(|e| e.label.0 == E_BECOMES && e.nodes[0] == anchor)
            .ok_or_else(|| refuse("missing sides"))?;
        let (left, right) = (sides.nodes[1], sides.nodes[2]);
        let pats: BTreeSet<_> = s
            .edges()
            .filter(|(_, e)| {
                e.label.0 == E_MEMBER && e.arity() == 2 && [left, right].contains(&e.nodes[0])
            })
            .map(|(_, e)| e.nodes[1])
            .collect();
        let mut vars = BTreeSet::new();
        for p in &pats {
            for id in slots(s, *p) {
                vars.insert(edge(s, id)?.nodes[1]);
            }
        }
        let mut body = BTreeSet::from([anchor, left, right]);
        body.extend(&pats);
        body.extend(&vars);
        for (_, e) in s.edges().filter(|(_, e)| {
            [E_HASLABEL, E_GUARDS].contains(&e.label.0)
                && e.arity() == 2
                && vars.contains(&e.nodes[0])
        }) {
            body.insert(e.nodes[1]);
        }
        for n in &body {
            need(
                s.edges_incident_to(*n)
                    .iter()
                    .all(|id| whole.edges.contains(id)),
                "foreign attachment to selected body",
            )?;
        }
        let mut patterns = pats.iter().copied().collect::<Vec<_>>();
        patterns.sort_by_key(|n| stable(*n));
        let mut variables = vars.into_iter().collect::<Vec<_>>();
        variables.sort_by_key(|n| stable(*n));
        Ok(Self {
            anchor,
            left,
            right,
            patterns,
            variables,
            body,
        })
    }
}

/// Protect world, other families and the unselected allele, including handles.
fn outside(s: &HypergraphStore, root: NodeId, scope: &Scope) -> Value {
    let nodes = s
        .nodes()
        .filter(|(id, _)| *id != root && !scope.body.contains(id))
        .map(|(id, n)| json!([id, n, s.edges_incident_to(id)]))
        .collect::<Vec<_>>();
    let edges = s
        .edges()
        .filter(|(_, e)| !e.nodes.iter().any(|n| scope.body.contains(n)))
        .map(|(id, e)| json!([id, e]))
        .collect::<Vec<_>>();
    json!([nodes, edges])
}

/// Read-only target discovery; selection is intentionally not implemented here.
pub fn targets(
    s: &HypergraphStore,
    root: NodeId,
    role: Role,
    limits: &MetaLimits,
) -> Result<Scope, String> {
    read_complete_program(&s.snapshot(), root, limits).map_err(|v| format!("{v:?}"))?;
    Scope::load(s, root, role, limits).map_err(|f| f.1)
}
pub fn slots(s: &HypergraphStore, pattern: NodeId) -> Vec<EdgeId> {
    let mut ids = s
        .edges()
        .filter(|(_, e)| {
            e.arity() == 2
                && e.nodes[0] == pattern
                && (E_SLOT_BASE..E_SLOT_BASE + 8).contains(&e.label.0)
        })
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    ids.sort_by_key(|id| (s.get_edge(*id).unwrap().label.0, stable(*id)));
    ids
}

struct Tx {
    s: HypergraphStore,
    root: NodeId,
    actions: Vec<Value>,
}
impl Tx {
    fn set(&mut self, n: NodeId, key: u32, value: PropertyValue) -> Res<()> {
        let node = self.s.get_node_mut(n).ok_or_else(|| refuse("stale node"))?;
        let old = node.get_property(key);
        node.set_property(key, value);
        self.actions
            .push(json!({"op":"set_property","id":n,"key":key,"before":old,"value":value}));
        Ok(())
    }
    fn adjust(&mut self, n: NodeId, key: u32, delta: i64) -> Res<()> {
        let value =
            int(&self.s, n, key).ok_or_else(|| refuse("missing declaration; recount forbidden"))?;
        let next = value
            .checked_add(delta)
            .filter(|v| *v >= 0)
            .ok_or_else(|| refuse("declaration underflow/overflow"))?;
        self.set(n, key, PropertyValue::Int(next))
    }
    fn add_edge(&mut self, label: u32, ns: &[NodeId]) -> EdgeId {
        let data = HyperEdge::new(Label(label), ns.iter().copied().collect());
        let id = self.s.add_edge(data.clone());
        self.actions
            .push(json!({"op":"add_edge","id":id,"data":data}));
        id
    }
    fn del_edge(&mut self, id: EdgeId) -> Res<()> {
        let before = edge(&self.s, id)?;
        self.s.remove_edge(id);
        self.actions
            .push(json!({"op":"delete_edge","id":id,"before":before}));
        Ok(())
    }
    fn owned(&mut self, label: u32, props: &[(u32, PropertyValue)]) -> Res<NodeId> {
        let mut data = NodeData::new(Label(label), self.s.step());
        for &(k, v) in props {
            data.set_property(k, v);
        }
        data.set_property(K_SCHEMA_LABEL, PropertyValue::Int(label as i64));
        for i in 0..26 {
            data.set_property(K_REQUIRED_BASE + i, PropertyValue::Int(0));
        }
        let id = self.s.add_node(data.clone());
        self.actions
            .push(json!({"op":"add_node","id":id,"data":data}));
        self.add_edge(E_OWNS, &[self.root, id]);
        self.adjust(self.root, K_OWNED_COUNT, 1)?;
        Ok(id)
    }
    fn relation(&mut self, role: u32, a: NodeId, b: NodeId) -> Res<EdgeId> {
        let ix = WITNESS_ROLES
            .iter()
            .position(|&r| r == role)
            .ok_or_else(|| refuse("unsupported payload role"))?;
        let id = self.add_edge(role, &[a, b]);
        self.add_edge(WITNESS_BASE + ix as u32, &[self.root, a, b]);
        self.adjust(a, K_REQUIRED_BASE + 2 * ix as u32, 1)?;
        self.adjust(b, K_REQUIRED_BASE + 2 * ix as u32 + 1, 1)?;
        Ok(id)
    }
    fn unlink(&mut self, id: EdgeId) -> Res<HyperEdge> {
        let e = edge(&self.s, id)?;
        let ix = WITNESS_ROLES
            .iter()
            .position(|&r| r == e.label.0)
            .ok_or_else(|| refuse("unsupported payload role"))?;
        need(e.arity() == 2, "protected payload")?;
        let ws = self
            .s
            .edges()
            .filter(|(_, w)| {
                w.label.0 == WITNESS_BASE + ix as u32
                    && w.nodes.as_slice() == [self.root, e.nodes[0], e.nodes[1]]
            })
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        need(ws.len() == 1, "missing/duplicate witness; repair forbidden")?;
        self.del_edge(ws[0])?;
        self.del_edge(id)?;
        self.adjust(e.nodes[0], K_REQUIRED_BASE + 2 * ix as u32, -1)?;
        self.adjust(e.nodes[1], K_REQUIRED_BASE + 2 * ix as u32 + 1, -1)?;
        Ok(e)
    }
    fn delete_owned(&mut self, n: NodeId) -> Res<()> {
        let incident = self.s.edges_incident_to(n).to_vec();
        need(
            incident.len() == 1,
            "foreign/unexpected incidence; cascade forbidden",
        )?;
        let own = edge(&self.s, incident[0])?;
        need(
            own.label.0 == E_OWNS && own.nodes.as_slice() == [self.root, n],
            "foreign ownership",
        )?;
        need(
            (0..26).all(|i| int(&self.s, n, K_REQUIRED_BASE + i) == Some(0)),
            "body still declares consumption",
        )?;
        self.del_edge(incident[0])?;
        self.adjust(self.root, K_OWNED_COUNT, -1)?;
        let before = self.s.get_node(n).unwrap().clone();
        self.s.remove_node(n);
        self.actions
            .push(json!({"op":"delete_node","id":n,"before":before}));
        Ok(())
    }
    fn labels(&self, v: NodeId) -> Vec<EdgeId> {
        self.s
            .edges()
            .filter(|(_, e)| e.arity() == 2 && e.label.0 == E_HASLABEL && e.nodes[0] == v)
            .map(|(id, _)| id)
            .collect()
    }
    fn drop_label(&mut self, v: NodeId) -> Res<bool> {
        let ls = self.labels(v);
        if ls.is_empty() {
            return Ok(false);
        }
        need(ls.len() == 1, "ambiguous label")?;
        let lit = self.unlink(ls[0])?.nodes[1];
        if !self
            .s
            .edges()
            .any(|(_, e)| e.label.0 == E_HASLABEL && e.arity() == 2 && e.nodes[1] == lit)
        {
            self.delete_owned(lit)?;
        }
        Ok(true)
    }
    fn compact(&mut self, p: NodeId) -> Res<()> {
        for (ix, id) in slots(&self.s, p).into_iter().enumerate() {
            let e = edge(&self.s, id)?;
            let role = E_SLOT_BASE + ix as u32;
            if e.label.0 != role {
                self.unlink(id)?;
                self.relation(role, p, e.nodes[1])?;
            }
        }
        Ok(())
    }
    fn drop_pattern(&mut self, p: NodeId) -> Res<()> {
        let mut ids = self
            .s
            .edges()
            .filter(|(_, e)| {
                e.arity() == 2
                    && ((e.label.0 == E_MEMBER && e.nodes[1] == p)
                        || ((E_SLOT_BASE..E_SLOT_BASE + 8).contains(&e.label.0) && e.nodes[0] == p))
            })
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| stable(*id));
        for id in ids {
            self.unlink(id)?;
        }
        self.delete_owned(p)
    }
    fn prune(&mut self, scope: &Scope) -> Res<()> {
        for &v in &scope.variables {
            if self.s.get_node(v).is_none()
                || self.s.edges().any(|(_, e)| {
                    e.arity() == 2
                        && (E_SLOT_BASE..E_SLOT_BASE + 8).contains(&e.label.0)
                        && e.nodes[1] == v
                })
            {
                continue;
            }
            self.drop_label(v)?;
            let guards = self
                .s
                .edges()
                .filter(|(_, e)| e.arity() == 2 && e.label.0 == E_GUARDS && e.nodes[0] == v)
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            for id in guards {
                let g = self.unlink(id)?.nodes[1];
                self.delete_owned(g)?;
            }
            self.delete_owned(v)?;
        }
        Ok(())
    }
    fn append_slot(&mut self, p: NodeId, v: NodeId) -> Res<()> {
        let ss = slots(&self.s, p);
        need(ss.len() < 8, "arity cap")?;
        need(
            ss.iter()
                .enumerate()
                .all(|(i, id)| self.s.get_edge(*id).unwrap().label.0 == E_SLOT_BASE + i as u32),
            "noncontiguous slot presentation",
        )?;
        self.relation(E_SLOT_BASE + ss.len() as u32, p, v)?;
        Ok(())
    }
    fn execute(&mut self, scope: &Scope, req: &Request) -> Res<bool> {
        let variable = |v: NodeId| {
            need(
                scope.variables.contains(&v),
                "variable outside selected allele",
            )
        };
        let pattern = |p: NodeId| {
            need(
                scope.patterns.contains(&p),
                "pattern outside selected allele",
            )
        };
        let selected_slot = |s: &HypergraphStore, id: EdgeId| -> Res<HyperEdge> {
            let e = edge(s, id)?;
            need(
                e.arity() == 2 && (E_SLOT_BASE..E_SLOT_BASE + 8).contains(&e.label.0),
                "not a supported slot",
            )?;
            pattern(e.nodes[0])?;
            variable(e.nodes[1])?;
            Ok(e)
        };
        match *req {
            Request::AddSlot {
                pattern: p,
                variable: v,
            } => {
                pattern(p)?;
                variable(v)?;
                self.append_slot(p, v)?;
            }
            Request::RemoveSlot { slot } => {
                let e = selected_slot(&self.s, slot)?;
                self.unlink(slot)?;
                self.compact(e.nodes[0])?;
                self.prune(scope)?;
            }
            Request::RetargetSlot { slot, variable: v } => {
                let e = selected_slot(&self.s, slot)?;
                variable(v)?;
                self.unlink(slot)?;
                self.relation(e.label.0, e.nodes[0], v)?;
                self.prune(scope)?;
            }
            Request::AddVar { pattern: p } => {
                pattern(p)?;
                need(scope.variables.len() < 8, "variable cap")?;
                need(slots(&self.s, p).len() < 8, "arity cap")?;
                let order = (0..8)
                    .find(|i| {
                        !scope
                            .variables
                            .iter()
                            .any(|v| int(&self.s, *v, K_ORDER) == Some(*i))
                    })
                    .ok_or_else(|| refuse("variable order cap"))?;
                let v = self.owned(L_VAR, &[(K_ORDER, PropertyValue::Int(order))])?;
                self.append_slot(p, v)?;
            }
            Request::RemoveVar { variable: v } => {
                variable(v)?;
                for &p in &scope.patterns {
                    for id in slots(&self.s, p) {
                        if edge(&self.s, id)?.nodes[1] == v {
                            self.unlink(id)?;
                        }
                    }
                    self.compact(p)?;
                    if slots(&self.s, p).len() < 2 {
                        self.drop_pattern(p)?;
                    }
                }
                self.prune(scope)?;
            }
            Request::AddPatternEdge { side, a, b } => {
                need(
                    [scope.left, scope.right].contains(&side),
                    "side outside selected allele",
                )?;
                variable(a)?;
                variable(b)?;
                let key = if side == scope.right {
                    K_ORDER_R
                } else {
                    K_ORDER
                };
                let members = self
                    .s
                    .edges()
                    .filter(|(_, e)| e.arity() == 2 && e.label.0 == E_MEMBER && e.nodes[0] == side)
                    .map(|(_, e)| e.nodes[1])
                    .collect::<Vec<_>>();
                let order = (0..=members.len() as i64)
                    .find(|i| {
                        !members.iter().any(|p| {
                            int(&self.s, *p, key).or_else(|| int(&self.s, *p, K_ORDER)) == Some(*i)
                        })
                    })
                    .unwrap();
                let p = self.owned(
                    L_PAT_EDGE,
                    &[
                        (K_ORDER, PropertyValue::Int(order)),
                        (key, PropertyValue::Int(order)),
                    ],
                )?;
                self.relation(E_MEMBER, side, p)?;
                self.relation(E_SLOT_BASE, p, a)?;
                self.relation(E_SLOT_BASE + 1, p, b)?;
            }
            Request::RemovePatternEdge { pattern: p } => {
                pattern(p)?;
                self.drop_pattern(p)?;
                self.prune(scope)?;
            }
            Request::AddLabel { variable: v, value } => {
                variable(v)?;
                need(value <= 2, "label value envelope")?;
                if !self.labels(v).is_empty() {
                    return Ok(false);
                }
                let lit = self.owned(
                    L_LABEL_LIT,
                    &[(K_LITERAL, PropertyValue::Int(value as i64))],
                )?;
                self.relation(E_HASLABEL, v, lit)?;
            }
            Request::RemoveLabel { variable: v } => {
                variable(v)?;
                if !self.drop_label(v)? {
                    return Ok(false);
                }
            }
            Request::ChangeLabel { variable: v, value } => {
                variable(v)?;
                need(value <= 2, "label value envelope")?;
                let ls = self.labels(v);
                if ls.is_empty() {
                    return Ok(false);
                }
                need(ls.len() == 1, "ambiguous label")?;
                let lit = edge(&self.s, ls[0])?.nodes[1];
                self.set(lit, K_LITERAL, PropertyValue::Int(value as i64))?;
            }
            Request::ChangePriority => {
                let p = int(&self.s, scope.anchor, K_PRIORITY)
                    .and_then(|v| i32::try_from(v).ok())
                    .and_then(|v| v.checked_add(1))
                    .ok_or_else(|| refuse("priority cap"))?;
                self.set(scope.anchor, K_PRIORITY, PropertyValue::Int(p as i64))?;
            }
            Request::Activate | Request::Deactivate => {
                self.set(
                    self.root,
                    K_GRAFT_MODE,
                    PropertyValue::Int(i64::from(matches!(req, Request::Activate))),
                )?;
            }
        }
        Ok(true)
    }
}

/// Deterministic transaction. All non-Applied attempts leave caller bytes intact.
pub fn apply(
    s: &mut HypergraphStore,
    root: NodeId,
    role: Role,
    req: Request,
    limits: &MetaLimits,
) -> Attempt {
    let before = inventory(s);
    let mut tx = Tx {
        s: s.clone(),
        root,
        actions: vec![],
    };
    let mut pre = None;
    let mut post = None;
    let result = (|| -> Res<Status> {
        let p = read_complete_program(&s.snapshot(), root, limits).map_err(|v| {
            Failure(
                Status::RejectedInput,
                "complete precondition".into(),
                Some(v),
            )
        })?;
        p.compile(limits, false).map_err(|v| {
            Failure(
                Status::RejectedInput,
                "compile precondition".into(),
                Some(v),
            )
        })?;
        pre = Some(p.clone());
        need(
            s.node_count() <= 2000 && s.edge_count() <= 4000,
            "input store cap",
        )?;
        let scope = Scope::load(s, root, role, limits)?;
        if !tx.execute(&scope, &req)? {
            post = Some(p.clone());
            return Ok(Status::Inapplicable);
        }
        let after = read_complete_program(&tx.s.snapshot(), root, limits).map_err(|v| {
            Failure(
                Status::RejectedPostcondition,
                "complete postcondition".into(),
                Some(v),
            )
        })?;
        post = Some(after.clone());
        after.compile(limits, false).map_err(|v| {
            Failure(
                Status::RejectedPostcondition,
                "compile postcondition".into(),
                Some(v),
            )
        })?;
        need(
            tx.s.node_count() <= 2000 && tx.s.edge_count() <= 4000,
            "output store cap",
        )?;
        need(
            if role == Role::Core {
                after.proposal == p.proposal
            } else {
                after.core == p.core
            },
            "unselected allele changed",
        )?;
        if !matches!(req, Request::Activate | Request::Deactivate) {
            need(after.active == p.active, "unrequested activation change")?;
        }
        need(tx.s.step() == s.step(), "store clock changed")?;
        // Rediscover the edited scope read-only; every fresh body must belong
        // to the selected allele. Do not exempt arbitrary newly created world
        // nodes from the noninterference check.
        let expanded = Scope::load(&tx.s, root, role, limits)?;
        need(
            (expanded.anchor, expanded.left, expanded.right)
                == (scope.anchor, scope.left, scope.right),
            "anchor or side identity changed",
        )?;
        for (id, _) in tx.s.nodes().filter(|(id, _)| s.get_node(*id).is_none()) {
            need(
                expanded.body.contains(&id),
                "new body outside selected allele",
            )?;
        }
        need(
            outside(s, root, &scope) == outside(&tx.s, root, &expanded),
            "outside-footprint material changed",
        )?;
        let unchanged_root = |store: &HypergraphStore| {
            let mut n = store.get_node(root).unwrap().clone();
            n.properties
                .retain(|(k, _)| *k != K_OWNED_COUNT && *k != K_GRAFT_MODE);
            serde_json::to_value(n).unwrap()
        };
        need(
            unchanged_root(s) == unchanged_root(&tx.s),
            "unrequested root data changed",
        )?;
        Ok(Status::Applied)
    })();
    let (status, reason, validator) = match result {
        Ok(status) => (status, None, None),
        Err(Failure(status, reason, v)) => (status, Some(reason), v),
    };
    let staged = inventory(&tx.s);
    if status == Status::Applied {
        *s = tx.s.clone();
    }
    let receipt = json!({"profile":PROFILE,"role":role,"request":req,"status":format!("{status:?}"),"reason":reason,"validator":validator.map(|v|format!("{v:?}")),"committed":status==Status::Applied,"before":before,"staged":staged,"actions":tx.actions,"advancing_engine_steps":0,"codec_reencoded":false,"metadata_recounted":false,"training_admitted":false});
    Attempt {
        status,
        receipt,
        staged: tx.s,
        before_program: pre,
        after_program: post,
    }
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
