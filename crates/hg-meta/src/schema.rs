use hg_core::{Label, PropertyOp, PropertyValue};
use hg_rules::{compile_rule, CompiledRule, NodePropertyGuard, Rule};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

pub use hg_rules::META_BASE;
pub const L_RULE_ROOT: u32 = META_BASE + 1;
pub const L_SIDE_L: u32 = META_BASE + 2;
pub const L_SIDE_R: u32 = META_BASE + 3;
pub const L_PAT_EDGE: u32 = META_BASE + 4;
pub const L_VAR: u32 = META_BASE + 5;
pub const L_LABEL_LIT: u32 = META_BASE + 6;
pub const L_GUARD: u32 = META_BASE + 7;
pub const L_ALLELE_ROOT: u32 = META_BASE + 8;
pub const E_BECOMES: u32 = META_BASE + 0x40;
pub const E_MEMBER: u32 = META_BASE + 0x41;
pub const E_CORE: u32 = META_BASE + 0x42;
pub const E_GRAFT: u32 = META_BASE + 0x43;
pub const E_SLOT_BASE: u32 = META_BASE + 0x50;
pub const E_HASLABEL: u32 = META_BASE + 0x70;
pub const E_GUARDS: u32 = META_BASE + 0x71;
pub const K_LITERAL: u32 = 0;
pub const K_PRIORITY: u32 = 2;
pub const K_GKEY: u32 = 3;
pub const K_GOP: u32 = 4;
pub const K_GVAL: u32 = 5;
// Optional presentation-order hints, not variable references. They preserve
// legacy matcher tie order during TOML round trips. Identity remains topology.
pub const K_ORDER: u32 = 6;
pub const K_ORDER_R: u32 = 7;
pub const K_ENCODING: u32 = 8;
pub const K_GRAFT_MODE: u32 = 9;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncodingVersion {
    #[default]
    LegacyV1,
    SharedPatternsV2,
    AlternativeFamiliesV3,
}
impl EncodingVersion {
    pub fn is_legacy(&self) -> bool {
        *self == Self::LegacyV1
    }
    pub fn protocol(self) -> &'static str {
        match self {
            Self::LegacyV1 => "E1-native-v2",
            Self::SharedPatternsV2 => "E1.1-shared-v1",
            Self::AlternativeFamiliesV3 => "E2-alternatives-v1",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MetaLimits {
    pub max_arity: usize,
    pub max_vars: usize,
    pub max_pattern_edges: usize,
    pub max_region_nodes: usize,
    pub max_region_edges: usize,
    pub radius: usize,
}
impl Default for MetaLimits {
    fn default() -> Self {
        Self {
            max_arity: 8,
            max_vars: 8,
            max_pattern_edges: 16,
            max_region_nodes: 128,
            max_region_edges: 256,
            radius: 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Guard {
    pub var: usize,
    pub key: u32,
    pub op: PropertyOp,
    pub value: PropertyValue,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CandidateRule {
    pub priority: i32,
    pub labels: Vec<Option<u32>>,
    pub lhs: Vec<Vec<usize>>,
    pub rhs: Vec<Vec<usize>>,
    pub guards: Vec<Guard>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Validator {
    Region = 0,
    Sides = 1,
    Members = 2,
    Slots = 3,
    Variables = 4,
    Labels = 5,
    Guards = 6,
    Bounds = 7,
    Compilability = 8,
}

impl CandidateRule {
    pub fn from_compiled(r: &CompiledRule, limits: &MetaLimits) -> Result<Self, String> {
        if r.pattern.edges.iter().any(|edge| edge.ordered) {
            return Err("ordered-edge maintenance constraint is outside the world envelope".into());
        }
        if r.schedule.is_some() {
            return Err("schedule".into());
        }
        if !r.enabled || r.weight != 1.0 {
            return Err("enabled/weight".into());
        }
        if r.pattern.edges.iter().any(|e| e.label.is_some())
            || r.template.rhs_edges.iter().any(|e| e.1 .0 != 0)
        {
            return Err("edge-label".into());
        }
        if !r.pattern.distinct_pairs.is_empty() {
            return Err("distinctness".into());
        }
        if !r.template.property_effects.is_empty() {
            return Err("property-effects".into());
        }
        if !r.pattern.property_compares.is_empty()
            || !r.pattern.degree_compares.is_empty()
            || !r.pattern.quantity_compares.is_empty()
        {
            return Err("derived/property-compare".into());
        }
        if r.template.rhs_relabels.iter().any(Option::is_some) {
            return Err("survivor-relabel".into());
        }
        let mut c = Self {
            priority: r.priority,
            labels: r
                .pattern
                .node_label_constraints
                .iter()
                .map(|l| l.map(|l| l.0))
                .collect(),
            lhs: r.pattern.edges.iter().map(|e| e.var_ixs.clone()).collect(),
            rhs: Vec::new(),
            guards: r
                .pattern
                .property_guards
                .iter()
                .map(|g| Guard {
                    var: g.var_ix,
                    key: g.key,
                    op: g.op,
                    value: g.value,
                })
                .collect(),
        };
        let mut rhs_map = Vec::new();
        for (i, from) in r.template.rhs_from_lhs.iter().enumerate() {
            rhs_map.push(if let Some(v) = from {
                *v
            } else {
                let v = c.labels.len();
                c.labels.push(Some(r.template.rhs_new_labels[i].0));
                v
            });
        }
        c.rhs = r
            .template
            .rhs_edges
            .iter()
            .map(|(e, _)| e.iter().map(|v| rhs_map[*v]).collect())
            .collect();
        c.validate(limits).map_err(|v| format!("{v:?}"))?;
        if c.left_vars().len() != r.pattern.n_vars || c.right_vars().len() != r.template.n_rhs_vars
        {
            return Err("isolated-variable".into());
        }
        Ok(c)
    }
    pub fn validate(&self, limits: &MetaLimits) -> Result<(), Validator> {
        if self.labels.is_empty() || self.labels.len() > limits.max_vars.min(8) {
            return Err(Validator::Variables);
        }
        if self.lhs.is_empty()
            || self.lhs.len() > limits.max_pattern_edges
            || self.rhs.len() > limits.max_pattern_edges
        {
            return Err(Validator::Bounds);
        }
        for e in self.lhs.iter().chain(&self.rhs) {
            if e.len() < 2 || e.len() > limits.max_arity.min(8) {
                return Err(Validator::Slots);
            }
            if e.iter().any(|v| *v >= self.labels.len()) {
                return Err(Validator::Variables);
            }
        }
        let left = self.left_vars();
        if self.guards.iter().any(|g| !left.contains(&g.var)) {
            return Err(Validator::Guards);
        }
        if (0..self.labels.len()).any(|v| !self.lhs.iter().chain(&self.rhs).any(|e| e.contains(&v)))
        {
            return Err(Validator::Variables);
        }
        Ok(())
    }

    pub fn left_vars(&self) -> BTreeSet<usize> {
        self.lhs.iter().flatten().copied().collect()
    }
    pub fn right_vars(&self) -> BTreeSet<usize> {
        self.rhs.iter().flatten().copied().collect()
    }

    pub fn to_rule(&self, name: String) -> Rule {
        let mut r = Rule {
            name,
            priority: self.priority,
            ..Rule::default()
        };
        let name = |v| format!("v{v}");
        r.lhs.nodes = self.left_vars().into_iter().map(name).collect();
        r.rhs.nodes = self.right_vars().into_iter().map(name).collect();
        r.lhs.edges = self
            .lhs
            .iter()
            .map(|e| e.iter().copied().map(name).collect())
            .collect();
        r.rhs.edges = self
            .rhs
            .iter()
            .map(|e| e.iter().copied().map(name).collect())
            .collect();
        for (v, label) in self.labels.iter().enumerate() {
            if let Some(label) = label {
                if self.left_vars().contains(&v) {
                    r.lhs.node_labels.insert(name(v), Label(*label));
                }
                // Survivor labels mean a constraint, not an implicit reset.
                if !self.left_vars().contains(&v) {
                    r.rhs.node_labels.insert(name(v), Label(*label));
                }
            }
        }
        r.lhs_node_property_guards = self
            .guards
            .iter()
            .map(|g| NodePropertyGuard {
                var: name(g.var),
                property: g.key,
                op: g.op,
                value: g.value,
            })
            .collect();
        r
    }

    pub fn compile(
        &self,
        limits: &MetaLimits,
        meta_visible: bool,
    ) -> Result<CompiledRule, Validator> {
        self.validate(limits)?;
        let mut r = compile_rule(&self.to_rule(format!("e1_{:016x}", self.rcf_hash())))
            .map_err(|_| Validator::Compilability)?;
        r.meta_visible = meta_visible;
        r.pattern.node_label_ceiling = (!meta_visible).then_some(META_BASE);
        Ok(r)
    }

    /// Reject unsupported features instead of silently dropping semantics.
    pub fn from_rule(r: &Rule, limits: &MetaLimits) -> Result<Self, String> {
        if r.schedule.is_some() {
            return Err("schedule".into());
        }
        if !r.enabled || r.weight != 1.0 {
            return Err("enabled/weight".into());
        }
        if !r.lhs.edge_labels.is_empty() || r.rhs.edge_labels.values().any(|l| l.0 != 0) {
            return Err("edge-label".into());
        }
        if !r.lhs_var_guards.is_empty() {
            return Err("distinctness".into());
        }
        if !r.rhs_property_effects.is_empty() {
            return Err("property-effects".into());
        }
        if !r.lhs_node_property_compares.is_empty()
            || !r.lhs_node_degree_compares.is_empty()
            || !r.lhs_node_quantity_compares.is_empty()
        {
            return Err("derived/property-compare".into());
        }
        let mut names = r.lhs.nodes.clone();
        for n in &r.rhs.nodes {
            if !names.contains(n) {
                names.push(n.clone());
            }
        }
        let ix = |n: &String| {
            names
                .iter()
                .position(|x| x == n)
                .ok_or("undeclared variable")
        };
        let edges = |es: &Vec<Vec<String>>| -> Result<Vec<Vec<usize>>, String> {
            es.iter()
                .map(|e| e.iter().map(|n| ix(n).map_err(str::to_string)).collect())
                .collect()
        };
        let mut c = Self {
            priority: r.priority,
            labels: vec![None; names.len()],
            lhs: edges(&r.lhs.edges)?,
            rhs: edges(&r.rhs.edges)?,
            guards: Vec::new(),
        };
        for (n, l) in &r.lhs.node_labels {
            c.labels[ix(n)?] = Some(l.0);
        }
        for (n, l) in r.rhs.node_labels.iter().chain(&r.rhs_node_labels) {
            let v = ix(n)?;
            if r.lhs.nodes.contains(n) {
                return Err("survivor-relabel".into());
            }
            c.labels[v] = Some(l.0);
        }
        for g in &r.lhs_node_property_guards {
            c.guards.push(Guard {
                var: ix(&g.var)?,
                key: g.property,
                op: g.op,
                value: g.value,
            });
        }
        c.validate(limits).map_err(|v| format!("{v:?}"))?;
        // Isolated variables would be silently lost by edge membership.
        if c.left_vars().len() != r.lhs.nodes.len() || c.right_vars().len() != r.rhs.nodes.len() {
            return Err("isolated-variable".into());
        }
        Ok(c)
    }

    /// Exact modulo variable renaming for <=8 vars, an intentional stronger
    /// alternative to v1 WL. Above the cap quarantine, never a greedy fallback.
    pub fn rcf_bytes(&self) -> Vec<u8> {
        if self.labels.len() > 8 {
            return Vec::new();
        }
        let mut perm: Vec<usize> = (0..self.labels.len()).collect();
        let mut best = self.bytes_with_map(&perm, true);
        fn visit(c: &CandidateRule, p: &mut [usize], k: usize, best: &mut Vec<u8>) {
            if k == p.len() {
                let b = c.bytes_with_map(p, true);
                if b < *best {
                    *best = b;
                }
                return;
            }
            for i in k..p.len() {
                p.swap(k, i);
                visit(c, p, k + 1, best);
                p.swap(k, i);
            }
        }
        visit(self, &mut perm, 0, &mut best);
        best
    }
    pub fn rcf_hash(&self) -> u64 {
        hash_bytes(&self.rcf_bytes())
    }
    /// Preserve execution presentation order in the rule-id secondary key.
    /// Unlike legacy class_hash, this includes every supported field.
    pub fn execution_bytes(&self) -> Vec<u8> {
        self.bytes_with_map(&(0..self.labels.len()).collect::<Vec<_>>(), false)
    }
    fn bytes_with_map(&self, map: &[usize], sorted: bool) -> Vec<u8> {
        let mut out = b"E1RCF2".to_vec();
        out.extend(self.priority.to_le_bytes());
        out.push(map.len() as u8);
        let mut labels = vec![None; map.len()];
        for (old, &new) in map.iter().enumerate() {
            labels[new] = self.labels[old];
        }
        for l in labels {
            out.push(u8::from(l.is_some()));
            out.extend(l.unwrap_or(0).to_le_bytes());
        }
        for edges in [&self.lhs, &self.rhs] {
            let mut edges: Vec<Vec<usize>> = edges
                .iter()
                .map(|e| e.iter().map(|v| map[*v]).collect())
                .collect();
            if sorted {
                edges.sort();
            }
            out.extend((edges.len() as u32).to_le_bytes());
            for e in edges {
                out.push(e.len() as u8);
                out.extend(e.iter().map(|v| *v as u8));
            }
        }
        let mut guards: Vec<Vec<u8>> = self
            .guards
            .iter()
            .map(|g| {
                let mut b = vec![map[g.var] as u8, op_code(g.op)];
                b.extend(g.key.to_le_bytes());
                match g.value {
                    PropertyValue::Int(v) => {
                        b.push(0);
                        b.extend(v.to_le_bytes());
                    }
                    PropertyValue::Float(v) => {
                        b.push(1);
                        b.extend(v.to_bits().to_le_bytes());
                    }
                    PropertyValue::Bool(v) => {
                        b.push(2);
                        b.push(v as u8);
                    }
                }
                b
            })
            .collect();
        if sorted {
            guards.sort();
        }
        out.extend((guards.len() as u32).to_le_bytes());
        for g in guards {
            out.extend((g.len() as u32).to_le_bytes());
            out.extend(g);
        }
        out
    }
}
pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h = rustc_hash::FxHasher::default();
    bytes.hash(&mut h);
    h.finish()
}
pub fn op_code(op: PropertyOp) -> u8 {
    match op {
        PropertyOp::Eq => 0,
        PropertyOp::Ne => 1,
        PropertyOp::Lt => 2,
        PropertyOp::Gt => 3,
        PropertyOp::Lte => 4,
        PropertyOp::Gte => 5,
    }
}
pub fn decode_op(code: i64) -> Option<PropertyOp> {
    Some(match code {
        0 => PropertyOp::Eq,
        1 => PropertyOp::Ne,
        2 => PropertyOp::Lt,
        3 => PropertyOp::Gt,
        4 => PropertyOp::Lte,
        5 => PropertyOp::Gte,
        _ => return None,
    })
}
