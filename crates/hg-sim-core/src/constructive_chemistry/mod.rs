//! CC-1: a bounded scripted profile. No reaction clock, RNG, or evolution.
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::checked::{
    stage_checked, CheckedEdit, CheckedGraphPatch, CheckedStageError, NodeObservation, NodeRef,
};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
pub mod fixture;

pub const SITE: Label = Label(0xE2000000);
pub const BEAD: Label = Label(0xE2000001);
pub const BOND: Label = Label(0xE2000002);
pub const ADJACENT: Label = Label(0xE2000064);
pub const PLACED: Label = Label(0xE2000065);
pub const BOND_A: Label = Label(0xE2000066);
pub const BOND_B: Label = Label(0xE2000067);
pub const ID: u32 = 243001;
pub const SITE_INDEX: u32 = 243002;
pub const SPECIES: u32 = 243003;
pub const ACTIVE: u32 = 243004;
pub const KIND: u32 = 243005;
pub const PORT_A: u32 = 243006;
pub const PORT_B: u32 = 243007;
pub const LAW: &str = "cc1-ab-fuel-ledger-v1";
pub const PROFILE: &str = "constructive-chemistry-cc1-atomic-fixture-v1";
static AUTHORITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ChemicalRejection {
    MalformedInitialState,
    MissingHandle,
    Distinctness,
    Species,
    InactiveFuel,
    PortOccupied,
    Nonlocal,
    Boundary,
    ReservoirExhausted,
    Capacity,
    ArithmeticOverflow,
    WorldIdentity,
    StaleState,
    InjectedFault,
    MaterialBalance,
    EnergyBalance,
    LedgerRange,
    ResourceCap,
    CheckedStage,
}
impl std::fmt::Display for ChemicalRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ChemicalRejection {}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ChemicalHandle {
    pub node: NodeId,
    pub semantic_id: i64,
}
#[derive(Clone, Debug, Serialize)]
pub enum ChemicalAction {
    FormBond {
        a: ChemicalHandle,
        b: ChemicalHandle,
        fuel: ChemicalHandle,
    },
    TransferFuelIn {
        activated: bool,
    },
    TransferFuelOut {
        fuel: ChemicalHandle,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPoint {
    AfterFuelUpdate,
    AfterBondNode,
    AfterFirstIncidence,
    AfterLedgerUpdate,
    BeforePublication,
}
#[derive(Clone, Debug, Serialize)]
pub struct ChemicalEvent {
    pub event_id: String,
    pub revision: u64,
    pub action: ChemicalAction,
    pub allocated_semantic_id: Option<i64>,
    pub heat_delta: i64,
    pub reservoir_delta: [i64; 2],
    pub before_sha256: String,
    /// Exact tagged node/property and relation changes, including reused generations.
    pub graph_delta: Value,
}
#[derive(Clone, Serialize)]
struct State {
    graph: HypergraphStore,
    world_identity: String,
    law: &'static str,
    profile: &'static str,
    revision: u64,
    semantic_id_frontier: i64,
    reservoir: [i64; 2],
    heat: i64,
    baseline_material: [String; 5],
    baseline_energy: String,
    accepted_event_journal: Vec<ChemicalEvent>,
}
/// No mutable graph escape and no public clone of commit authority.
pub struct ChemistryWorld {
    state: State,
    authority: u64,
}
pub struct PreparedChemicalCommit {
    authority: u64,
    before: Vec<u8>,
    before_sha: String,
    next: State,
    event: ChemicalEvent,
}
#[derive(Clone)]
struct View {
    ids: BTreeMap<i64, NodeId>,
    sites: BTreeMap<i64, i64>,
    species: BTreeMap<i64, i64>,
    activated: BTreeMap<i64, bool>,
    bonds: Vec<(i64, i64, i64)>,
    occupied_ports: BTreeSet<i64>,
    material: [i128; 5],
    world_energy: i128,
}
fn integer(n: &NodeData, k: u32) -> Result<i64, ChemicalRejection> {
    match n.get_property(k) {
        Some(PropertyValue::Int(i)) => Ok(i),
        _ => Err(ChemicalRejection::MalformedInitialState),
    }
}
fn boolean(n: &NodeData, k: u32) -> Result<bool, ChemicalRejection> {
    match n.get_property(k) {
        Some(PropertyValue::Bool(i)) => Ok(i),
        _ => Err(ChemicalRejection::MalformedInitialState),
    }
}
fn schema(g: &HypergraphStore, revision: u64, frontier: i64) -> Result<View, ChemicalRejection> {
    use ChemicalRejection::MalformedInitialState as Bad;
    if g.node_count() > 32 || g.edge_count() > 64 {
        return Err(ChemicalRejection::ResourceCap);
    }
    if g.step() != revision || frontier <= 0 {
        return Err(Bad);
    }
    let mut v = View {
        ids: BTreeMap::new(),
        sites: BTreeMap::new(),
        species: BTreeMap::new(),
        activated: BTreeMap::new(),
        bonds: vec![],
        occupied_ports: BTreeSet::new(),
        material: [0; 5],
        world_energy: 0,
    };
    let mut site_nodes = BTreeMap::new();
    let mut bond_nodes = vec![];
    for (h, n) in g.nodes() {
        let keys: BTreeSet<_> = n.properties.iter().map(|p| p.0).collect();
        let expected: BTreeSet<_> = match n.label {
            SITE => [ID, SITE_INDEX].into_iter().collect(),
            BEAD => [ID, SPECIES, ACTIVE].into_iter().collect(),
            BOND => [ID, KIND, PORT_A, PORT_B].into_iter().collect(),
            _ => return Err(Bad),
        };
        if keys != expected
            || keys.len() != n.properties.len()
            || n.energy.to_bits() != 0
            || n.generation > revision
        {
            return Err(Bad);
        }
        let id = integer(n, ID)?;
        if id <= 0 || id >= frontier || v.ids.insert(id, h).is_some() {
            return Err(Bad);
        }
        match n.label {
            SITE => {
                let site = integer(n, SITE_INDEX)?;
                if !(0..3).contains(&site)
                    || id != site + 1
                    || n.generation != 0
                    || site_nodes.insert(site, h).is_some()
                {
                    return Err(Bad);
                }
            }
            BEAD => {
                let s = integer(n, SPECIES)?;
                let active = boolean(n, ACTIVE)?;
                if !(0..5).contains(&s) || (s != 4 && active) {
                    return Err(Bad);
                }
                v.species.insert(id, s);
                v.activated.insert(id, active);
                v.material[s as usize] += 1;
                if active {
                    v.world_energy += 4;
                }
            }
            BOND => {
                if [KIND, PORT_A, PORT_B]
                    .iter()
                    .any(|k| integer(n, *k) != Ok(0))
                {
                    return Err(Bad);
                }
                bond_nodes.push((id, h));
            }
            _ => unreachable!(),
        }
    }
    if site_nodes.len() != 3 || v.species.len() > 16 {
        return Err(Bad);
    }
    let mut adjacent = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut roles: BTreeMap<(i64, u32), i64> = BTreeMap::new();
    for (_, e) in g.edges() {
        if e.nodes.len() != 2 || e.nodes[0] == e.nodes[1] {
            return Err(Bad);
        }
        let x = g.get_node(e.nodes[0]).ok_or(Bad)?;
        let y = g.get_node(e.nodes[1]).ok_or(Bad)?;
        let (ix, iy) = (integer(x, ID)?, integer(y, ID)?);
        let pair = (ix.min(iy), ix.max(iy));
        if !seen.insert((e.label.0, pair)) {
            return Err(Bad);
        }
        match e.label {
            ADJACENT if x.label == SITE && y.label == SITE => {
                adjacent.insert((
                    integer(x, SITE_INDEX)?.min(integer(y, SITE_INDEX)?),
                    integer(x, SITE_INDEX)?.max(integer(y, SITE_INDEX)?),
                ));
            }
            PLACED => {
                let (bead, site) = if x.label == BEAD && y.label == SITE {
                    (x, y)
                } else if y.label == BEAD && x.label == SITE {
                    (y, x)
                } else {
                    return Err(Bad);
                };
                if v.sites
                    .insert(integer(bead, ID)?, integer(site, SITE_INDEX)?)
                    .is_some()
                {
                    return Err(Bad);
                }
            }
            BOND_A | BOND_B => {
                let (bond, bead) = if x.label == BOND && y.label == BEAD {
                    (x, y)
                } else if y.label == BOND && x.label == BEAD {
                    (y, x)
                } else {
                    return Err(Bad);
                };
                let required = if e.label == BOND_A { 0 } else { 1 };
                if integer(bead, SPECIES)? != required
                    || roles
                        .insert((integer(bond, ID)?, e.label.0), integer(bead, ID)?)
                        .is_some()
                {
                    return Err(Bad);
                }
            }
            _ => return Err(Bad),
        }
    }
    // Admission does not trust mutable secondary indexes. Re-derive both from edges.
    for (h, _) in g.nodes() {
        let expected: BTreeSet<_> = g
            .edges()
            .filter(|(_, e)| e.nodes.contains(&h))
            .map(|(id, _)| id)
            .collect();
        let actual: BTreeSet<_> = g.edges_incident_to(h).iter().copied().collect();
        if actual != expected || actual.len() != g.edges_incident_to(h).len() {
            return Err(Bad);
        }
    }
    let expected_labels: BTreeSet<_> = g.edges().map(|(_, e)| e.label.0.to_string()).collect();
    let wire = serde_json::to_value(g).map_err(|_| Bad)?;
    let actual_labels: BTreeSet<_> = wire["label_index"]
        .as_object()
        .ok_or(Bad)?
        .keys()
        .cloned()
        .collect();
    if expected_labels != actual_labels {
        return Err(Bad);
    }
    for label in [ADJACENT, PLACED, BOND_A, BOND_B] {
        let expected: BTreeSet<_> = g
            .edges()
            .filter(|(_, e)| e.label == label)
            .map(|(id, _)| id)
            .collect();
        let actual: BTreeSet<_> = g.edges_with_label(label).iter().copied().collect();
        if actual != expected || actual.len() != g.edges_with_label(label).len() {
            return Err(Bad);
        }
    }
    if adjacent != [(0, 1), (1, 2)].into_iter().collect() || v.sites.len() != v.species.len() {
        return Err(Bad);
    }
    for site in 0..3 {
        if v.sites.values().filter(|s| **s == site).count() > 8 {
            return Err(Bad);
        }
    }
    for (id, _) in bond_nodes {
        let a = *roles.get(&(id, BOND_A.0)).ok_or(Bad)?;
        let b = *roles.get(&(id, BOND_B.0)).ok_or(Bad)?;
        if a == b
            || v.sites.get(&a) != v.sites.get(&b)
            || !v.occupied_ports.insert(a)
            || !v.occupied_ports.insert(b)
        {
            return Err(Bad);
        }
        v.bonds.push((id, a, b));
        v.world_energy += 2;
    }
    Ok(v)
}
fn totals(
    v: &View,
    reservoir: [i64; 2],
    heat: i64,
) -> Result<([i128; 5], i128), ChemicalRejection> {
    if reservoir.iter().any(|i| *i < 0) || heat < 0 {
        return Err(ChemicalRejection::LedgerRange);
    }
    let mut m = v.material;
    m[4] = m[4]
        .checked_add(i128::from(reservoir[0]))
        .and_then(|x| x.checked_add(i128::from(reservoir[1])))
        .ok_or(ChemicalRejection::ArithmeticOverflow)?;
    let energy = i128::from(reservoir[0])
        .checked_mul(4)
        .and_then(|x| x.checked_add(v.world_energy))
        .and_then(|x| x.checked_add(i128::from(heat)))
        .ok_or(ChemicalRejection::ArithmeticOverflow)?;
    Ok((m, energy))
}
fn validate(s: &State) -> Result<View, ChemicalRejection> {
    let v = schema(&s.graph, s.revision, s.semantic_id_frontier)?;
    let (m, e) = totals(&v, s.reservoir, s.heat)?;
    if m.map(|x| x.to_string()) != s.baseline_material {
        return Err(ChemicalRejection::MaterialBalance);
    }
    if e.to_string() != s.baseline_energy {
        return Err(ChemicalRejection::EnergyBalance);
    }
    Ok(v)
}
fn exact(s: &State) -> Result<Vec<u8>, ChemicalRejection> {
    // JSON objects use BTreeMap ordering; arrays and tagged property variants are retained.
    let value = json!({"state":s,"allocator":s.graph.allocation_state()});
    let bytes = serde_json::to_vec(&value).map_err(|_| ChemicalRejection::ResourceCap)?;
    if bytes.len() > 1_048_576 {
        return Err(ChemicalRejection::ResourceCap);
    }
    Ok(bytes)
}
pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
impl ChemistryWorld {
    pub fn admit(
        graph: HypergraphStore,
        identity: String,
        revision: u64,
        frontier: i64,
        reservoir: [i64; 2],
        heat: i64,
    ) -> Result<Self, ChemicalRejection> {
        let v = schema(&graph, revision, frontier)
            .map_err(|_| ChemicalRejection::MalformedInitialState)?;
        let (m, e) =
            totals(&v, reservoir, heat).map_err(|_| ChemicalRejection::MalformedInitialState)?;
        let state = State {
            graph,
            world_identity: identity,
            law: LAW,
            profile: PROFILE,
            revision,
            semantic_id_frontier: frontier,
            reservoir,
            heat,
            baseline_material: m.map(|i| i.to_string()),
            baseline_energy: e.to_string(),
            accepted_event_journal: vec![],
        };
        exact(&state)?;
        let authority = AUTHORITY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| x.checked_add(1))
            .map_err(|_| ChemicalRejection::ResourceCap)?;
        Ok(Self { state, authority })
    }
    pub fn graph(&self) -> &HypergraphStore {
        &self.state.graph
    }
    pub fn handle(&self, id: i64) -> Option<ChemicalHandle> {
        self.state
            .graph
            .nodes()
            .find(|(_, n)| integer(n, ID) == Ok(id))
            .map(|(node, _)| ChemicalHandle {
                node,
                semantic_id: id,
            })
    }
    pub fn exact_bytes(&self) -> Vec<u8> {
        exact(&self.state).expect("admitted bounded world")
    }
    pub fn observation(&self) -> Value {
        observation(&self.state)
    }
}
fn observation(s: &State) -> Value {
    let v = schema(&s.graph, s.revision, s.semantic_id_frontier)
        .expect("schema-valid diagnostic state");
    let nodes:Vec<_>=v.ids.iter().map(|(id,h)| {
        let n=s.graph.get_node(*h).unwrap();
        json!({"id":id,"label":n.label.0,"generation":n.generation,"species":v.species.get(id),"activated":v.activated.get(id),
            "site":if n.label==SITE {Some(integer(n,SITE_INDEX).unwrap())} else {v.sites.get(id).copied()}})
    }).collect();
    let edges: Vec<_> = s
        .graph
        .edges()
        .map(|(_, e)| {
            let mut ids: Vec<_> = e
                .nodes
                .iter()
                .map(|h| integer(s.graph.get_node(*h).unwrap(), ID).unwrap())
                .collect();
            ids.sort();
            json!({"label":e.label.0,"nodes":ids})
        })
        .collect();
    let mut edges = edges;
    edges.sort_by_key(|e| e.to_string());
    let mut material = v.material;
    material[4] += i128::from(s.reservoir[0]) + i128::from(s.reservoir[1]);
    let energy = v.world_energy + 4 * i128::from(s.reservoir[0]) + i128::from(s.heat);
    let bytes = exact(s).unwrap();
    json!({"semantic":{"nodes":nodes,"edges":edges,"revision":s.revision,"frontier":s.semantic_id_frontier,
        "reservoir":s.reservoir,"heat":s.heat,"world_material":v.material.map(|x|x.to_string()),
        "total_material":material.map(|x|x.to_string()),"world_energy":v.world_energy.to_string(),"total_energy":energy.to_string()},
        "exact":serde_json::from_slice::<Value>(&bytes).unwrap(),"exact_sha256":sha(&bytes)})
}
fn checked_handle(v: &View, h: ChemicalHandle) -> Result<i64, ChemicalRejection> {
    if v.ids.get(&h.semantic_id) != Some(&h.node) {
        Err(ChemicalRejection::MissingHandle)
    } else {
        Ok(h.semantic_id)
    }
}
fn data(label: Label, id: i64, generation: u64, props: &[(u32, PropertyValue)]) -> NodeData {
    let mut n = NodeData::new(label, generation);
    n.set_property(ID, PropertyValue::Int(id));
    for (k, v) in props {
        n.set_property(*k, *v);
    }
    n
}
fn stage_error(error: CheckedStageError) -> ChemicalRejection {
    match error {
        CheckedStageError::Capacity | CheckedStageError::GenerationExhausted => {
            ChemicalRejection::ResourceCap
        }
        _ => ChemicalRejection::CheckedStage,
    }
}
fn patch(edits: Vec<CheckedEdit>) -> CheckedGraphPatch {
    CheckedGraphPatch {
        observations: vec![],
        edits,
        max_nodes: 32,
        max_edges: 64,
    }
}
fn fail_at(actual: FaultPoint, requested: Option<FaultPoint>) -> Result<(), ChemicalRejection> {
    if requested == Some(actual) {
        Err(ChemicalRejection::InjectedFault)
    } else {
        Ok(())
    }
}
pub fn prepare(
    world: &ChemistryWorld,
    action: &ChemicalAction,
) -> Result<PreparedChemicalCommit, ChemicalRejection> {
    prepare_with_fault(world, action, None)
}
/// Diagnostic fault injection mutates only owned staging, never the live world.
pub fn prepare_with_fault(
    world: &ChemistryWorld,
    action: &ChemicalAction,
    fault: Option<FaultPoint>,
) -> Result<PreparedChemicalCommit, ChemicalRejection> {
    use CheckedEdit::*;
    use ChemicalRejection::*;
    use PropertyValue::{Bool, Int};
    let before = exact(&world.state)?;
    let before_sha = sha(&before);
    let v = validate(&world.state)?;
    let mut next = world.state.clone();
    let revision = next.revision.checked_add(1).ok_or(ArithmeticOverflow)?;
    let mut allocated = None;
    let mut edits = vec![];
    match *action {
        ChemicalAction::FormBond { a, b, fuel } => {
            let (ai, bi, fi) = (
                checked_handle(&v, a)?,
                checked_handle(&v, b)?,
                checked_handle(&v, fuel)?,
            );
            if ai == bi || ai == fi || bi == fi {
                return Err(Distinctness);
            }
            if v.species.get(&ai) != Some(&0)
                || v.species.get(&bi) != Some(&1)
                || v.species.get(&fi) != Some(&4)
            {
                return Err(Species);
            }
            if !v.activated[&fi] {
                return Err(InactiveFuel);
            }
            if v.occupied_ports.contains(&ai) || v.occupied_ports.contains(&bi) {
                return Err(PortOccupied);
            }
            if v.sites[&ai] != v.sites[&bi] || v.sites[&ai] != v.sites[&fi] {
                return Err(Nonlocal);
            }
            let id = next.semantic_id_frontier;
            next.semantic_id_frontier = id.checked_add(1).ok_or(ArithmeticOverflow)?;
            allocated = Some(id);
            edits.push(ReplaceProperty {
                node: fuel.node,
                key: ACTIVE,
                expected: Bool(true),
                value: Bool(false),
            });
            if fault == Some(FaultPoint::AfterFuelUpdate) {
                stage_checked(&next.graph, &patch(edits.clone())).map_err(stage_error)?;
                return Err(InjectedFault);
            }
            edits.push(AddNode {
                local: 0,
                data: data(
                    BOND,
                    id,
                    revision,
                    &[(KIND, Int(0)), (PORT_A, Int(0)), (PORT_B, Int(0))],
                ),
            });
            if fault == Some(FaultPoint::AfterBondNode) {
                stage_checked(&next.graph, &patch(edits.clone())).map_err(stage_error)?;
                return Err(InjectedFault);
            }
            edits.push(AddEdge {
                label: BOND_A,
                nodes: vec![NodeRef::Local(0), NodeRef::Existing(a.node)],
            });
            if fault == Some(FaultPoint::AfterFirstIncidence) {
                stage_checked(&next.graph, &patch(edits.clone())).map_err(stage_error)?;
                return Err(InjectedFault);
            }
            edits.push(AddEdge {
                label: BOND_B,
                nodes: vec![NodeRef::Local(0), NodeRef::Existing(b.node)],
            });
            next.heat = next.heat.checked_add(2).ok_or(ArithmeticOverflow)?;
        }
        ChemicalAction::TransferFuelIn { activated } => {
            let lane = usize::from(!activated);
            if next.reservoir[lane] == 0 {
                return Err(ReservoirExhausted);
            }
            if v.sites.values().filter(|s| **s == 0).count() >= 8 {
                return Err(Capacity);
            }
            if v.species.len() >= 16 {
                return Err(ResourceCap);
            }
            next.reservoir[lane] = next.reservoir[lane]
                .checked_sub(1)
                .ok_or(ArithmeticOverflow)?;
            let id = next.semantic_id_frontier;
            next.semantic_id_frontier = id.checked_add(1).ok_or(ArithmeticOverflow)?;
            allocated = Some(id);
            edits.push(AddNode {
                local: 0,
                data: data(
                    BEAD,
                    id,
                    revision,
                    &[(SPECIES, Int(4)), (ACTIVE, Bool(activated))],
                ),
            });
            edits.push(AddEdge {
                label: PLACED,
                nodes: vec![NodeRef::Local(0), NodeRef::Existing(v.ids[&1])],
            });
        }
        ChemicalAction::TransferFuelOut { fuel } => {
            let fi = checked_handle(&v, fuel)?;
            if v.species.get(&fi) != Some(&4) {
                return Err(Species);
            }
            if v.sites[&fi] != 0 {
                return Err(Boundary);
            }
            let lane = usize::from(!v.activated[&fi]);
            next.reservoir[lane] = next.reservoir[lane]
                .checked_add(1)
                .ok_or(ArithmeticOverflow)?;
            for e in next.graph.edges_incident_to(fuel.node) {
                edits.push(RemoveEdge(*e));
            }
            edits.push(RemoveNode(fuel.node));
        }
    }
    let observations = world
        .state
        .graph
        .nodes()
        .map(|(node, n)| NodeObservation {
            node,
            label: n.label,
            properties: n.properties.iter().copied().collect(),
            incident: world.state.graph.edges_incident_to(node).to_vec(),
        })
        .collect();
    next.graph = stage_checked(
        &next.graph,
        &CheckedGraphPatch {
            observations,
            ..patch(edits)
        },
    )
    .map_err(stage_error)?
    .into_graph();
    next.revision = revision;
    next.graph.set_step(revision);
    fail_at(FaultPoint::AfterLedgerUpdate, fault)?;
    validate(&next)?;
    let event = ChemicalEvent {
        event_id: format!("{}:{revision}", next.world_identity),
        revision,
        action: action.clone(),
        allocated_semantic_id: allocated,
        heat_delta: next.heat - world.state.heat,
        reservoir_delta: [
            next.reservoir[0] - world.state.reservoir[0],
            next.reservoir[1] - world.state.reservoir[1],
        ],
        before_sha256: before_sha.clone(),
        graph_delta: graph_delta(&world.state.graph, &next.graph),
    };
    next.accepted_event_journal.push(event.clone());
    exact(&next)?;
    fail_at(FaultPoint::BeforePublication, fault)?;
    Ok(PreparedChemicalCommit {
        authority: world.authority,
        before,
        before_sha,
        next,
        event,
    })
}
pub fn commit(
    world: &mut ChemistryWorld,
    prepared: PreparedChemicalCommit,
) -> Result<ChemicalEvent, ChemicalRejection> {
    if world.authority != prepared.authority
        || world.state.world_identity != prepared.next.world_identity
    {
        return Err(ChemicalRejection::WorldIdentity);
    }
    let actual = exact(&world.state)?;
    if sha(&actual) != prepared.before_sha || actual != prepared.before {
        return Err(ChemicalRejection::StaleState);
    }
    // Event buffers, graph, ledgers, and journal already exist. Publication cannot fail.
    world.state = prepared.next;
    Ok(prepared.event)
}

fn graph_delta(before: &HypergraphStore, after: &HypergraphStore) -> Value {
    let old = serde_json::to_value(before).expect("bounded serializable graph");
    let new = serde_json::to_value(after).expect("bounded serializable graph");
    let mut result = serde_json::Map::new();
    for arena in ["nodes", "edges"] {
        let a = old[arena].as_array().unwrap();
        let b = new[arena].as_array().unwrap();
        let changes: Vec<_> = (1..a.len().max(b.len()))
            .filter_map(|idx| {
                let before = a.get(idx).cloned().unwrap_or(Value::Null);
                let after = b.get(idx).cloned().unwrap_or(Value::Null);
                if before == after {
                    None
                } else {
                    Some(json!({"slot_index":idx,"before":before,"after":after}))
                }
            })
            .collect();
        result.insert(arena.to_owned(), json!(changes));
    }
    result.insert(
        "step".to_owned(),
        json!({"before":before.step(),"after":after.step()}),
    );
    Value::Object(result)
}
