//! A finite, opt-in graph instruction vocabulary compiled to native rules.
//! No imperative mutation or repair occurs in this decoder/compiler.
use crate::*;
use hg_core::{GraphSnapshot, HypergraphStore, Label, NodeData, NodeId, PropertyOp, PropertyValue};
use hg_rules::{
    compile_rule, CompiledRule, EffectOpDecl, NodeDegreeCompare, NodePropertyGuard,
    PropertyEffectDecl, Rule, VarGuard, VarGuardType,
};
use serde::{Deserialize, Serialize};

pub const VOCABULARY_PROTOCOL: &str = "E2-schema-preserving-vocabulary-v1";
pub const L_META_INSTRUCTION: u32 = META_BASE + 9;
pub const K_OPCODE: u32 = 10;
pub const K_FUEL: u32 = 1;
pub const INSTRUCTION_VERSION: i64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MetaOperation {
    Activate,
    Deactivate,
    Transpose,
    Seal,
    Repair,
    WitnessActivate,
    RepairSealed,
}
impl MetaOperation {
    pub fn code(self) -> i64 {
        match self {
            Self::Activate => 0,
            Self::Deactivate => 1,
            Self::Transpose => 2,
            Self::Seal => 3,
            Self::Repair => 4,
            Self::WitnessActivate => 5,
            Self::RepairSealed => 6,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Activate => "Activate",
            Self::Deactivate => "Deactivate",
            Self::Transpose => "Transpose",
            Self::Seal => "Seal",
            Self::Repair => "Repair",
            Self::WitnessActivate => "WitnessActivate",
            Self::RepairSealed => "RepairSealed",
        }
    }
    pub fn from_code(code: i64) -> Option<Self> {
        match code {
            0 => Some(Self::Activate),
            1 => Some(Self::Deactivate),
            2 => Some(Self::Transpose),
            3 => Some(Self::Seal),
            4 => Some(Self::Repair),
            5 => Some(Self::WitnessActivate),
            6 => Some(Self::RepairSealed),
            _ => None,
        }
    }
    pub fn execution_bytes(self) -> Vec<u8> {
        let mut b = match self {
            Self::RepairSealed => crate::SEALED_REPAIR_PROTOCOL,
            _ if self.code() < 3 => VOCABULARY_PROTOCOL,
            _ => crate::REPAIR_PROTOCOL,
        }.as_bytes().to_vec();
        b.extend(self.code().to_le_bytes());
        b
    }
    pub fn instruction_version(self) -> i64 {
        match self {
            Self::RepairSealed => crate::SEALED_REPAIR_INSTRUCTION_VERSION,
            _ if self.code() < 3 => INSTRUCTION_VERSION,
            _ => crate::REPAIR_INSTRUCTION_VERSION,
        }
    }
    /// Runtime/source admission must not silently upgrade an older profile.
    pub fn admitted(self, witnessed: bool, sealed_repair: bool) -> bool {
        match self {
            Self::RepairSealed => witnessed && sealed_repair,
            _ => (self.code() >= 3) == witnessed,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaInstructionSeed {
    pub operation: MetaOperation,
    pub fuel: f64,
}
impl MetaInstructionSeed {
    pub fn validate(&self) -> Result<(), Validator> {
        if !self.fuel.is_finite() || !(0.0..=32.0).contains(&self.fuel) {
            return Err(Validator::Bounds);
        }
        Ok(())
    }
}
pub fn encode_instruction(
    store: &mut HypergraphStore,
    seed: &MetaInstructionSeed,
) -> Result<NodeId, Validator> {
    seed.validate()?;
    let mut n = NodeData::new(Label(L_META_INSTRUCTION), store.step());
    n.set_property(K_ENCODING, PropertyValue::Int(seed.operation.instruction_version()));
    n.set_property(K_OPCODE, PropertyValue::Int(seed.operation.code()));
    n.set_property(K_FUEL, PropertyValue::Float(seed.fuel));
    Ok(store.add_node(n))
}
pub fn read_instruction(
    s: &GraphSnapshot<'_>,
    root: NodeId,
) -> Result<MetaInstructionSeed, Validator> {
    let n = s.get_node(root).ok_or(Validator::Region)?;
    if n.label.0 != L_META_INSTRUCTION
        || !s.edges_incident_to(root).is_empty()
    {
        return Err(Validator::Region);
    }
    let operation = match n.get_property(K_OPCODE) {
        Some(PropertyValue::Int(code)) => {
            MetaOperation::from_code(code).ok_or(Validator::Compilability)?
        }
        _ => return Err(Validator::Compilability),
    };
    if n.get_property(K_ENCODING)!=Some(PropertyValue::Int(operation.instruction_version())) {
        return Err(Validator::Region);
    }
    let fuel = match n.get_property(K_FUEL) {
        Some(PropertyValue::Float(f)) => f,
        _ => return Err(Validator::Guards),
    };
    let seed = MetaInstructionSeed { operation, fuel };
    seed.validate()?;
    Ok(seed)
}
pub fn operation_rule(operation: MetaOperation) -> Rule {
    let mut r = Rule {
        name: format!("encoded_meta_{}", operation.name()),
        priority: 100,
        ..Rule::default()
    };
    let names: &[&str] = if operation == MetaOperation::Transpose {
        &[
            "op", "root", "proposal", "left", "right", "pattern", "a", "b",
        ]
    } else {
        &["op", "root", "proposal"]
    };
    r.lhs.nodes = names.iter().map(|s| (*s).into()).collect();
    r.rhs.nodes = r.lhs.nodes.clone();
    for (name, label) in [
        ("op", L_META_INSTRUCTION),
        ("root", L_RULE_ROOT),
        ("proposal", L_ALLELE_ROOT),
    ] {
        r.lhs.node_labels.insert(name.into(), Label(label));
    }
    let edges: Vec<(&[&str], u32)> = if operation == MetaOperation::Transpose {
        vec![
            (&["root", "proposal"], E_GRAFT),
            (&["proposal", "left", "right"], E_BECOMES),
            (&["right", "pattern"], E_MEMBER),
            (&["pattern", "a"], E_SLOT_BASE),
            (&["pattern", "b"], E_SLOT_BASE + 1),
        ]
    } else {
        vec![(&["root", "proposal"], E_GRAFT)]
    };
    for (i, (vars, label)) in edges.iter().enumerate() {
        r.lhs.edges.push(vars.iter().map(|s| (*s).into()).collect());
        r.lhs.edge_labels.insert(i, Label(*label));
        let vars = if operation == MetaOperation::Transpose && i == 3 {
            vec!["pattern", "b"]
        } else if operation == MetaOperation::Transpose && i == 4 {
            vec!["pattern", "a"]
        } else {
            vars.to_vec()
        };
        r.rhs.edges.push(vars.iter().map(|s| (*s).into()).collect());
        r.rhs.edge_labels.insert(i, Label(*label));
    }
    let mut guard = |var: &str, key, op, value| {
        r.lhs_node_property_guards.push(NodePropertyGuard {
            var: var.into(),
            property: key,
            op,
            value,
        })
    };
    guard(
        "op",
        K_ENCODING,
        PropertyOp::Eq,
        PropertyValue::Int(INSTRUCTION_VERSION),
    );
    guard(
        "op",
        K_OPCODE,
        PropertyOp::Eq,
        PropertyValue::Int(operation.code()),
    );
    guard("op", K_FUEL, PropertyOp::Gte, PropertyValue::Float(1.0));
    guard("op", K_FUEL, PropertyOp::Lte, PropertyValue::Float(32.0));
    guard("root", K_ENCODING, PropertyOp::Eq, PropertyValue::Int(3));
    if operation != MetaOperation::Transpose {
        guard(
            "root",
            K_GRAFT_MODE,
            PropertyOp::Eq,
            PropertyValue::Int(i64::from(operation == MetaOperation::Deactivate)),
        );
    }
    r.lhs_node_degree_compares.push(NodeDegreeCompare {
        var: "op".into(),
        op: PropertyOp::Eq,
        value: 0,
        offset: None,
    });
    r.rhs_property_effects.push(PropertyEffectDecl {
        var: "op".into(),
        key: K_FUEL,
        op: EffectOpDecl::Subtract {
            value: PropertyValue::Float(1.0),
        },
    });
    if operation == MetaOperation::Transpose {
        for (name, label) in [
            ("left", L_SIDE_L),
            ("right", L_SIDE_R),
            ("pattern", L_PAT_EDGE),
            ("a", L_VAR),
            ("b", L_VAR),
        ] {
            r.lhs.node_labels.insert(name.into(), Label(label));
        }
        r.lhs_var_guards.push(VarGuard {
            guard_type: VarGuardType::VarsDistinct,
            vars: vec!["a".into(), "b".into()],
        });
    } else {
        r.rhs_property_effects.push(PropertyEffectDecl {
            var: "root".into(),
            key: K_GRAFT_MODE,
            op: EffectOpDecl::Set {
                value: PropertyValue::Int(i64::from(operation == MetaOperation::Activate)),
            },
        });
    }
    r
}
pub fn compile_operation(
    operation: MetaOperation,
    limits: &MetaLimits,
) -> Result<CompiledRule, Validator> {
    if operation.code()>=3 {return Err(Validator::Compilability);}
    let mut compiled =
        compile_rule(&operation_rule(operation)).map_err(|_| Validator::Compilability)?;
    if compiled.pattern.n_vars > limits.max_vars.min(8)
        || compiled.pattern.edges.len() > limits.max_pattern_edges
        || compiled.template.n_rhs_vars > limits.max_vars.min(8)
        || compiled.template.rhs_edges.len() > limits.max_pattern_edges
        || compiled
            .pattern
            .edges
            .iter()
            .any(|e| e.var_ixs.len() > limits.max_arity.min(8))
        || compiled
            .template
            .rhs_edges
            .iter()
            .any(|e| e.0.len() > limits.max_arity.min(8))
    {
        return Err(Validator::Bounds);
    }
    compiled.meta_visible = true;
    compiled.pattern.node_label_ceiling = None;
    Ok(compiled)
}

pub fn compile_operations(operation: MetaOperation, limits: &MetaLimits) -> Result<Vec<CompiledRule>,Validator> {
    if operation.code()<3 {compile_operation(operation,limits).map(|r|vec![r])}
    else {crate::compile_repair_operations(operation,limits)}
}
