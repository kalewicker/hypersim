//! Explicit alternative-rule semantics, compiled to the unchanged engine.
use crate::*;
use hg_core::{GraphSnapshot, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct RuleProgram {
    pub core: CandidateRule,
    pub proposal: Option<CandidateRule>,
    pub active: bool,
}
impl RuleProgram {
    pub fn strict(core: CandidateRule) -> Self {
        Self {
            core,
            proposal: None,
            active: false,
        }
    }
    /// Canonical execution order; exact duplicates do not double sampling weight.
    pub fn candidates(&self) -> Vec<CandidateRule> {
        let mut cs = vec![self.core.clone()];
        if self.active {
            if let Some(p) = &self.proposal {
                cs.push(p.clone());
            }
        }
        cs.sort_by_key(|c| (c.rcf_bytes(), c.execution_bytes()));
        cs.dedup_by(|a, b| a.execution_bytes() == b.execution_bytes());
        cs
    }
    /// Only executable semantics, never dormant genotype, constitute novelty.
    pub fn rcf_bytes(&self) -> Vec<u8> {
        let mut parts: Vec<_> = self
            .candidates()
            .iter()
            .map(CandidateRule::rcf_bytes)
            .collect();
        parts.sort();
        parts.dedup();
        if parts.len() == 1 {
            return parts.pop().unwrap();
        }
        pack(parts, b"E2-PROGRAM\0")
    }
    pub fn execution_bytes(&self) -> Vec<u8> {
        let parts = self
            .candidates()
            .iter()
            .map(CandidateRule::execution_bytes)
            .collect();
        pack(parts, b"E2-EXEC\0")
    }
    pub fn compile(
        &self,
        limits: &MetaLimits,
        visible: bool,
    ) -> Result<Vec<hg_rules::CompiledRule>, Validator> {
        self.candidates()
            .iter()
            .map(|c| c.compile(limits, visible))
            .collect()
    }
}
fn pack(parts: Vec<Vec<u8>>, tag: &[u8]) -> Vec<u8> {
    let mut out = tag.to_vec();
    out.extend_from_slice(&(parts.len() as u32).to_le_bytes());
    for p in parts {
        out.extend_from_slice(&(p.len() as u32).to_le_bytes());
        out.extend(p);
    }
    out
}

pub struct FamilyParts {
    pub whole: EncodedRegion,
    pub core: EncodedRegion,
    pub proposal: EncodedRegion,
    pub active: bool,
}

pub fn encode_family(
    s: &mut HypergraphStore,
    core: &CandidateRule,
    proposal: &CandidateRule,
    active: bool,
    limits: &MetaLimits,
) -> Result<EncodedRegion, Validator> {
    // Validate and count with isolated temporary encodings BEFORE touching caller.
    let mut count = HypergraphStore::new();
    let a = encode_versioned(&mut count, core, limits, EncodingVersion::SharedPatternsV2)?;
    let b = encode_versioned(
        &mut count,
        proposal,
        limits,
        EncodingVersion::SharedPatternsV2,
    )?;
    if a.nodes.len() + b.nodes.len() + 1 > limits.max_region_nodes
        || a.edges.len() + b.edges.len() + 2 > limits.max_region_edges
    {
        return Err(Validator::Region);
    }
    let test = assemble_family(&mut count, a, b, active);
    read_program(&count.snapshot(), test.root, limits)?;
    let a = encode_versioned(s, core, limits, EncodingVersion::SharedPatternsV2)?;
    let b = encode_versioned(s, proposal, limits, EncodingVersion::SharedPatternsV2)?;
    Ok(assemble_family(s, a, b, active))
}

fn assemble_family(
    s: &mut HypergraphStore,
    a: EncodedRegion,
    b: EncodedRegion,
    active: bool,
) -> EncodedRegion {
    let mut n = NodeData::new(Label(L_RULE_ROOT), s.step());
    n.set_property(K_ENCODING, PropertyValue::Int(3));
    n.set_property(K_GRAFT_MODE, PropertyValue::Int(i64::from(active)));
    let root = s.add_node(n);
    s.get_node_mut(a.root).unwrap().label = Label(L_ALLELE_ROOT);
    s.get_node_mut(b.root).unwrap().label = Label(L_ALLELE_ROOT);
    let mut edges = vec![
        s.add_edge(HyperEdge::new(
            Label(E_CORE),
            [root, a.root].into_iter().collect(),
        )),
        s.add_edge(HyperEdge::new(
            Label(E_GRAFT),
            [root, b.root].into_iter().collect(),
        )),
    ];
    let mut nodes = vec![root];
    nodes.extend(a.nodes);
    nodes.extend(b.nodes);
    edges.extend(a.edges);
    edges.extend(b.edges);
    EncodedRegion { root, nodes, edges }
}

pub fn family_parts(
    s: &GraphSnapshot<'_>,
    root: NodeId,
    limits: &MetaLimits,
) -> Result<FamilyParts, Validator> {
    let whole = walk_family(s, root, limits)?;
    if s.get_node(root).is_none_or(|n| {
        n.label.0 != L_RULE_ROOT || n.get_property(K_ENCODING) != Some(PropertyValue::Int(3))
    }) {
        return Err(Validator::Region);
    }
    let active = match s.get_node(root).unwrap().get_property(K_GRAFT_MODE) {
        Some(PropertyValue::Int(0)) => false,
        Some(PropertyValue::Int(1)) => true,
        _ => return Err(Validator::Compilability),
    };
    let mut a = None;
    let mut b = None;
    for &eid in &whole.edges {
        let e = s.get_edge(eid).unwrap();
        if matches!(e.label.0, E_CORE | E_GRAFT) {
            if e.nodes.len() != 2
                || e.nodes[0] != root
                || s.get_node(e.nodes[1])
                    .is_none_or(|n| n.label.0 != L_ALLELE_ROOT)
            {
                return Err(Validator::Members);
            }
            let slot = if e.label.0 == E_CORE { &mut a } else { &mut b };
            if slot.replace(e.nodes[1]).is_some() {
                return Err(Validator::Members);
            }
        }
    }
    let (a, b) = match (a, b) {
        (Some(a), Some(b)) if a != b => (a, b),
        (Some(_), Some(_)) => return Err(Validator::Members),
        _ => return Err(Validator::Sides),
    };
    let core = walk(s, a, limits)?;
    let proposal = walk(s, b, limits)?;
    let core_nodes: HashSet<_> = core.nodes.iter().copied().collect();
    if proposal.nodes.iter().any(|n| core_nodes.contains(n)) {
        return Err(Validator::Members);
    }
    if whole.nodes.len() != core.nodes.len() + proposal.nodes.len() + 1
        || whole.edges.len() != core.edges.len() + proposal.edges.len() + 2
    {
        return Err(Validator::Members);
    }
    Ok(FamilyParts {
        whole,
        core,
        proposal,
        active,
    })
}

pub fn read_program(
    s: &GraphSnapshot<'_>,
    root: NodeId,
    limits: &MetaLimits,
) -> Result<RuleProgram, Validator> {
    if s.get_node(root).and_then(|n| n.get_property(K_ENCODING)) == Some(PropertyValue::Int(crate::COMPLETE_VERSION)) {
        return crate::read_complete_program(s, root, limits);
    }
    if s.get_node(root).and_then(|n| n.get_property(K_ENCODING)) == Some(PropertyValue::Int(crate::WITNESS_VERSION)) {
        return crate::read_witness_program(s,root,limits);
    }
    if s.get_node(root).and_then(|n| n.get_property(K_ENCODING)) != Some(PropertyValue::Int(3)) {
        return read(s, root, limits).map(RuleProgram::strict);
    }
    let parts = family_parts(s, root, limits)?;
    // All bodies are validated, including dormant ones. No partial install.
    let core = read(s, parts.core.root, limits)?;
    let proposal = read(s, parts.proposal.root, limits)?;
    Ok(RuleProgram {
        core,
        proposal: Some(proposal),
        active: parts.active,
    })
}
