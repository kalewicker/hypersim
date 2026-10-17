//! Rule definition (raw, serializable form) and creation-rule schedules

use hg_core::Label;
use crate::effects::PropertyEffectDecl;
use crate::pattern::PatternGraph;
use crate::guards::{
    NodeDegreeCompare, NodePropertyCompare, NodePropertyGuard, NodeQuantityCompare, VarGuard,
};
use serde::{Serialize, Deserialize};
use ahash::AHashMap;

/// When a creation rule fires (v4 §5.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Schedule {
    /// Fires once, at step 0.
    Once,
    /// Fires every `n` steps (steps 0, n, 2n, …).
    EveryNSteps(u64),
}

impl Schedule {
    pub fn is_due(&self, step: u64) -> bool {
        match self {
            Schedule::Once => step == 0,
            Schedule::EveryNSteps(n) => *n > 0 && step.is_multiple_of(*n),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub priority: i32,
    pub weight: f64,
    pub enabled: bool,
    pub lhs: PatternGraph,
    pub rhs: PatternGraph,
    pub lhs_node_property_guards: Vec<NodePropertyGuard>,
    /// Property-to-property comparison guards (hg-alife S1, scope §5.3).
    pub lhs_node_property_compares: Vec<NodePropertyCompare>,
    pub lhs_node_degree_compares: Vec<NodeDegreeCompare>,
    /// Var-to-var comparison of a derived node quantity (E-track). Empty for
    /// every rule that does not use `node_quantity_compares` — exactly the
    /// pre-feature behavior.
    pub lhs_node_quantity_compares: Vec<NodeQuantityCompare>,
    pub lhs_var_guards: Vec<VarGuard>,
    pub rhs_node_labels: AHashMap<String, Label>,
    /// Property writes applied at rewrite step 5.5, declaration order.
    pub rhs_property_effects: Vec<PropertyEffectDecl>,
    /// Present iff this is a creation rule (validated at parse time).
    pub schedule: Option<Schedule>,
}

impl Rule {
    /// A creation rule has an empty LHS pattern (and a schedule).
    pub fn is_creation(&self) -> bool {
        self.lhs.is_empty()
    }
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            name: String::new(),
            priority: 0,
            weight: 1.0,
            enabled: true,
            lhs: PatternGraph::new(),
            rhs: PatternGraph::new(),
            lhs_node_property_guards: Vec::new(),
            lhs_node_property_compares: Vec::new(),
            lhs_node_degree_compares: Vec::new(),
            lhs_node_quantity_compares: Vec::new(),
            lhs_var_guards: Vec::new(),
            rhs_node_labels: AHashMap::new(),
            rhs_property_effects: Vec::new(),
            schedule: None,
        }
    }
}
