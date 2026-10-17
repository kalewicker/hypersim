//! Pattern graph representation (raw, serializable form)

use hg_core::Label;
use serde::{Deserialize, Serialize};
use ahash::AHashMap;
use std::collections::BTreeMap;

/// Pattern graph for LHS/RHS of rules.
///
/// `edge_labels` is a **sparse map keyed by edge index** (v4 §5.1) — an
/// absent key means "unconstrained" (LHS) / "default label 0" (RHS). TOML
/// has no per-element null, so a parallel `Vec<Option<u32>>` cannot express
/// partial constraint; a sparse table can.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PatternGraph {
    pub nodes: Vec<String>,       // variable names
    pub edges: Vec<Vec<String>>,  // each edge = list of variable names
    pub node_labels: AHashMap<String, Label>,
    pub edge_labels: BTreeMap<usize, Label>,
}

impl PatternGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// True iff this is an empty (creation-rule) pattern.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}
