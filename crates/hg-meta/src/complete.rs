//! Opt-in graph-resident ownership and completeness. No live repair or oracle.
use crate::*;
use hg_core::{GraphSnapshot, HyperEdge, HypergraphStore, Label, NodeId, PropertyValue};
use std::collections::{HashMap, HashSet};

pub const COMPLETE_PROTOCOL: &str = "E2-witness-complete-v1";
pub const COMPLETE_VERSION: i64 = 10;
pub const COMPLETE_INSTRUCTION_VERSION: i64 = 11;
pub const E_OWNS: u32 = META_BASE + 0x120;
pub const K_OWNED_COUNT: u32 = 0x200;
pub const K_SCHEMA_LABEL: u32 = 0x201;
pub const K_REQUIRED_BASE: u32 = 0x220;
pub const COMPLETE_MAX_PROPERTIES: usize = 64;
const COUNTS: usize = WITNESS_ROLES.len() * 2;

pub fn encode_complete_family(
    store: &mut HypergraphStore,
    core: &CandidateRule,
    proposal: &CandidateRule,
    active: bool,
    limits: &MetaLimits,
) -> Result<EncodedRegion, Validator> {
    let mut trial = HypergraphStore::new();
    let region = add_complete(&mut trial, core, proposal, active, limits)?;
    read_complete_program(&trial.snapshot(), region.root, limits)?;
    add_complete(store, core, proposal, active, limits)
}

fn add_complete(
    store: &mut HypergraphStore,
    core: &CandidateRule,
    proposal: &CandidateRule,
    active: bool,
    limits: &MetaLimits,
) -> Result<EncodedRegion, Validator> {
    let mut region = encode_witness_family(store, core, proposal, active, limits)?;
    if region.edges.len() + region.nodes.len() - 1 > limits.max_region_edges {
        return Err(Validator::Region);
    }
    let mut counts: HashMap<_, [usize; COUNTS]> =
        region.nodes.iter().map(|&n| (n, [0; COUNTS])).collect();
    for &id in &region.edges {
        let e = store.get_edge(id).unwrap();
        if e.arity() == 2 {
            let ix = WITNESS_ROLES
                .iter()
                .position(|&role| role == e.label.0)
                .ok_or(Validator::Bounds)?;
            counts.get_mut(&e.nodes[0]).unwrap()[2 * ix] += 1;
            counts.get_mut(&e.nodes[1]).unwrap()[2 * ix + 1] += 1;
        }
    }
    for &node in &region.nodes {
        let data = store.get_node_mut(node).unwrap();
        data.set_property(K_SCHEMA_LABEL, PropertyValue::Int(data.label.0 as i64));
        for (index, &count) in counts[&node].iter().enumerate() {
            data.set_property(
                K_REQUIRED_BASE + index as u32,
                PropertyValue::Int(count as i64),
            );
        }
        if node != region.root {
            region.edges.push(store.add_edge(HyperEdge::new(
                Label(E_OWNS),
                [region.root, node].into_iter().collect(),
            )));
        }
    }
    let root = store.get_node_mut(region.root).unwrap();
    root.set_property(
        K_OWNED_COUNT,
        PropertyValue::Int((region.nodes.len() - 1) as i64),
    );
    root.set_property(K_ENCODING, PropertyValue::Int(COMPLETE_VERSION));
    Ok(region)
}

fn declared(
    s: &GraphSnapshot<'_>,
    node: NodeId,
    key: u32,
    maximum: usize,
) -> Result<usize, Validator> {
    let n = s.get_node(node).ok_or(Validator::Region)?;
    if n.properties.iter().filter(|(k, _)| *k == key).count() != 1 {
        return Err(Validator::Members);
    }
    match n.get_property(key) {
        Some(PropertyValue::Int(n)) if n >= 0 && (n as u128) <= maximum as u128 => Ok(n as usize),
        _ => Err(Validator::Members),
    }
}

pub fn read_complete_program(
    s: &GraphSnapshot<'_>,
    root: NodeId,
    limits: &MetaLimits,
) -> Result<RuleProgram, Validator> {
    if s.get_node(root).is_none_or(|n| {
        n.properties.len() > COMPLETE_MAX_PROPERTIES
            || n.label.0 != L_RULE_ROOT
            || n.get_property(K_ENCODING) != Some(PropertyValue::Int(COMPLETE_VERSION))
    }) {
        return Err(Validator::Region);
    }
    let r = walk_family(s, root, limits)?;
    let nodes: HashSet<_> = r.nodes.iter().copied().collect();
    let mut owned = HashSet::new();
    for &id in &r.edges {
        let e = s.get_edge(id).ok_or(Validator::Region)?;
        if e.label.0 == E_OWNS {
            if e.nodes.len() != 2
                || e.nodes[0] != root
                || e.nodes[1] == root
                || !owned.insert(e.nodes[1])
            {
                return Err(Validator::Members);
            }
        }
    }
    if owned.len() != declared(s, root, K_OWNED_COUNT, limits.max_region_nodes)?
        || owned.len() + 1 != nodes.len()
        || !owned.iter().all(|n| nodes.contains(n))
    {
        return Err(Validator::Members);
    }
    for &node in &r.nodes {
        if s.get_node(node).unwrap().properties.len() > COMPLETE_MAX_PROPERTIES {
            return Err(Validator::Region);
        }
        if declared(s, node, K_SCHEMA_LABEL, u32::MAX as usize)?
            != s.get_node(node).unwrap().label.0 as usize
        {
            return Err(Validator::Members);
        }
    }
    let mut actual: HashMap<_, [usize; COUNTS]> =
        r.nodes.iter().map(|&n| (n, [0; COUNTS])).collect();
    let mut witnesses = HashSet::new();
    let mut consumed = HashSet::new();
    let mut payloads = vec![];
    let mut work = 0usize;
    for &id in &r.edges {
        let witness = s.get_edge(id).unwrap();
        let Some(ix) = witness
            .label
            .0
            .checked_sub(WITNESS_BASE)
            .filter(|&i| (i as usize) < WITNESS_ROLES.len())
            .map(|i| i as usize)
        else {
            continue;
        };
        if witness.nodes.len() != 3 || witness.nodes[0] != root {
            return Err(Validator::Members);
        }
        let role = WITNESS_ROLES[ix];
        let (a, b) = (witness.nodes[1], witness.nodes[2]);
        if !witnesses.insert((role, a, b)) || !nodes.contains(&a) || !nodes.contains(&b) {
            return Err(Validator::Members);
        }
        let mut matched = None;
        for &eid in s.edges_incident_to(a) {
            work += 1;
            if work > limits.max_region_edges.saturating_mul(16) {
                return Err(Validator::Region);
            }
            let e = s.get_edge(eid).ok_or(Validator::Region)?;
            if e.label.0 == role
                && (e.nodes.as_slice() == [a, b] || e.nodes.as_slice() == [a, b, root])
                && matched.replace(eid).is_some_and(|old| old != eid)
            {
                return Err(Validator::Members);
            }
        }
        let eid = matched.ok_or(Validator::Members)?;
        if !consumed.insert(eid) {
            return Err(Validator::Members);
        }
        actual.get_mut(&a).unwrap()[2 * ix] += 1;
        actual.get_mut(&b).unwrap()[2 * ix + 1] += 1;
        payloads.push((role, a, b));
    }
    for &node in &r.nodes {
        for (index, &count) in actual[&node].iter().enumerate() {
            if declared(
                s,
                node,
                K_REQUIRED_BASE + index as u32,
                limits.max_region_edges,
            )? != count
            {
                return Err(Validator::Members);
            }
        }
    }
    let mut projection = HypergraphStore::new();
    let mut map = HashMap::new();
    for &id in &r.nodes {
        let mut data = s.get_node(id).unwrap().clone();
        if id == root {
            data.set_property(K_ENCODING, PropertyValue::Int(3));
        }
        map.insert(id, projection.add_node(data));
    }
    for (role, a, b) in payloads {
        projection.add_edge(HyperEdge::new(
            Label(role),
            [map[&a], map[&b]].into_iter().collect(),
        ));
    }
    for &id in &r.edges {
        let e = s.get_edge(id).unwrap();
        if consumed.contains(&id) || e.label.0 == E_OWNS || witnesses_label(e.label.0) {
            continue;
        }
        if e.label.0 != E_BECOMES || e.nodes.len() != 3 {
            return Err(Validator::Bounds);
        }
        projection.add_edge(HyperEdge::new(
            e.label,
            e.nodes.iter().map(|n| map[n]).collect(),
        ));
    }
    // An owned orphan must not silently disappear from the projected program.
    let projected = family_parts(&projection.snapshot(), map[&root], limits)?;
    if projected.whole.nodes.len() != r.nodes.len() {
        return Err(Validator::Members);
    }
    read_program(&projection.snapshot(), map[&root], limits)
}

fn witnesses_label(label: u32) -> bool {
    (WITNESS_BASE..WITNESS_BASE + WITNESS_ROLES.len() as u32).contains(&label)
}

pub fn encode_complete_instruction(
    store: &mut HypergraphStore,
    seed: &MetaInstructionSeed,
) -> Result<NodeId, Validator> {
    if seed.operation.code() < 3 {
        return Err(Validator::Compilability);
    }
    let id = encode_instruction(store, seed)?;
    store
        .get_node_mut(id)
        .unwrap()
        .set_property(K_ENCODING, PropertyValue::Int(COMPLETE_INSTRUCTION_VERSION));
    Ok(id)
}

pub fn read_complete_instruction(
    s: &GraphSnapshot<'_>,
    root: NodeId,
) -> Result<MetaInstructionSeed, Validator> {
    let n = s.get_node(root).ok_or(Validator::Region)?;
    if n.properties.len() > COMPLETE_MAX_PROPERTIES
        || n.label.0 != L_META_INSTRUCTION
        || !s.edges_incident_to(root).is_empty()
        || n.get_property(K_ENCODING) != Some(PropertyValue::Int(COMPLETE_INSTRUCTION_VERSION))
    {
        return Err(Validator::Region);
    }
    let operation = match n.get_property(K_OPCODE) {
        Some(PropertyValue::Int(code)) if code >= 3 => {
            MetaOperation::from_code(code).ok_or(Validator::Compilability)?
        }
        _ => return Err(Validator::Compilability),
    };
    let fuel = match n.get_property(K_FUEL) {
        Some(PropertyValue::Float(f)) => f,
        _ => return Err(Validator::Guards),
    };
    let seed = MetaInstructionSeed { operation, fuel };
    seed.validate()?;
    Ok(seed)
}

pub fn complete_operation_bytes(operation: MetaOperation) -> Vec<u8> {
    let mut bytes = COMPLETE_PROTOCOL.as_bytes().to_vec();
    bytes.extend(operation.code().to_le_bytes());
    bytes
}

pub fn compile_complete_operations(
    operation: MetaOperation,
    limits: &MetaLimits,
) -> Result<Vec<hg_rules::CompiledRule>, Validator> {
    let mut rules = crate::repair::compile_witness_operations(
        operation,
        limits,
        COMPLETE_VERSION,
        COMPLETE_INSTRUCTION_VERSION,
        true,
    )?;
    for rule in &mut rules {
        rule.name = format!("complete_{}", rule.name);
    }
    Ok(rules)
}
