//! Compiled rule representation — what the matcher and apply logic consume.
//!
//! Variables are compiled from names to dense indices so bindings can be
//! `SmallVec<NodeId>` keyed by position (deterministic, no string hashing
//! in the hot loop).

use crate::effects::{CompiledEffectOp, EffectOpDecl, PropertyEffect};
use crate::guards::{NodeQuantity, VarGuardType};
use crate::rule::{Rule, Schedule};
use crate::parser::ParseError;
use hg_core::{Label, PropertyOp, PropertyValue};
use ahash::AHashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PatternEdge {
    /// LHS variable indices forming this hyperedge, in declared order.
    pub var_ixs: Vec<usize>,
    /// Sparse LHS edge-label constraint (None = unconstrained).
    pub label: Option<Label>,
    /// Opt-in codec relation constraint. Raw/world rules remain unordered;
    /// only explicitly compiled finite maintenance macros set this flag.
    pub ordered: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompiledPropertyGuard {
    pub var_ix: usize,
    pub key: u32,
    pub op: PropertyOp,
    #[serde(with = "hg_core::wire::value")]
    pub value: PropertyValue,
}

/// Compiled property-to-property comparison guard (hg-alife S1): both
/// variables resolved to LHS indices. Evaluates `var_a[key_a] <op> var_b[key_b]`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompiledPropertyCompare {
    pub var_a_ix: usize,
    pub key_a: u32,
    pub op: PropertyOp,
    pub var_b_ix: usize,
    pub key_b: u32,
    /// Constant added to side B before comparing (None = plain compare).
    #[serde(with = "hg_core::wire::optional_value")]
    pub offset: Option<hg_core::PropertyValue>,
}

/// Compiled degree comparison guard (plan v4 §6.2): degree of a bound
/// node vs an Int constant (+ optional Int offset, saturating — the
/// same offset machinery as property compares). Degree = count of
/// DISTINCT incident hyperedges (multi-incidence in one edge counts
/// once); the matcher reads it from the incidence index, O(1).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompiledDegreeCompare {
    pub var_ix: usize,
    pub op: PropertyOp,
    pub value: i64,
    pub offset: Option<i64>,
}

/// Compiled var-to-var node-quantity comparison (E-track):
/// `quantity(var_a) <op> (quantity(var_b) + offset)`. A **post-binding
/// filter** — unlike `CompiledDegreeCompare` it cannot be evaluated until both
/// vars are bound, so it never prunes enumeration, only what enumeration
/// produced. See `NodeQuantityCompare` for the full rationale.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompiledQuantityCompare {
    pub var_a_ix: usize,
    pub quantity: NodeQuantity,
    pub op: PropertyOp,
    pub var_b_ix: usize,
    pub offset: Option<i64>,
}

/// The compiled LHS.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MatchPattern {
    /// ERE protection: None leaves legacy matching exactly unchanged.
    pub node_label_ceiling: Option<u32>,
    pub match_work_limit: Option<usize>,
    pub match_result_limit: Option<usize>,
    pub n_vars: usize,
    pub edges: Vec<PatternEdge>,
    /// Node-label constraint per variable index (None = unconstrained).
    pub node_label_constraints: Vec<Option<Label>>,
    pub property_guards: Vec<CompiledPropertyGuard>,
    /// Property-to-property comparison guards (hg-alife S1). Empty for every
    /// rule that does not use `node_property_compares` — exactly the
    /// pre-feature behavior, pinned by the canonical-hash regression.
    pub property_compares: Vec<CompiledPropertyCompare>,
    /// Degree comparison guards (v4 §6.2). Empty for every rule that
    /// does not use `node_degree_compares` — pre-feature behavior.
    pub degree_compares: Vec<CompiledDegreeCompare>,
    /// Var-to-var node-quantity comparison guards (E-track). Empty for every
    /// rule that does not use `node_quantity_compares` — pre-feature behavior,
    /// asserted inert by `tests/e1_2_primitive_gate.rs`.
    pub quantity_compares: Vec<CompiledQuantityCompare>,
    /// Pairwise distinctness constraints auto-generated from
    /// `vars_distinct` guards (each guard's var list expands to all pairs).
    pub distinct_pairs: Vec<(usize, usize)>,
}

/// The compiled RHS.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct RewriteTemplate {
    pub n_rhs_vars: usize,
    /// For each RHS var index: `Some(lhs_ix)` if it survives from the LHS,
    /// `None` if it is created fresh.
    pub rhs_from_lhs: Vec<Option<usize>>,
    /// Label for each RHS var if created fresh (default 0), by RHS index.
    pub rhs_new_labels: Vec<Label>,
    /// For each RHS var index: `Some(label)` when the var SURVIVES from
    /// the LHS and the RHS explicitly assigns it a label (a relabel —
    /// the glider's mark transfer). `None` = keep the node's label.
    /// Absence of an explicit RHS label never resets a survivor to 0.
    pub rhs_relabels: Vec<Option<Label>>,
    /// RHS edges: variable indices (into RHS vars) + label (sparse map
    /// resolved; absent index = Label(0)).
    pub rhs_edges: Vec<(Vec<usize>, Label)>,
    /// LHS var indices deleted (present in LHS, absent from RHS).
    pub lhs_deleted: Vec<usize>,
    /// Property writes, declaration order, variables resolved to RHS
    /// indices. Empty when the TOML carries no effects — exactly the
    /// pre-feature behavior (pinned by the canonical-hash regression).
    pub property_effects: Vec<PropertyEffect>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompiledRule {
    /// Ordinary ERE rules cannot bind meta nodes. Legacy tables remain inert
    /// until protection is explicitly enabled by the ERE runner.
    pub meta_visible: bool,
    pub name: String,
    pub priority: i32,
    pub weight: f64,
    pub enabled: bool,
    pub schedule: Option<Schedule>,
    pub pattern: MatchPattern,
    pub template: RewriteTemplate,
    /// LHS variable names by index (for diagnostics/events).
    pub lhs_var_names: Vec<String>,
}

impl CompiledRule {
    pub fn is_creation(&self) -> bool {
        self.schedule.is_some()
    }
}

/// Compile a parsed rule. Assumes `Rule` already passed parse validation;
/// re-checks the invariants it depends on and errors rather than panics.
pub fn compile_rule(rule: &Rule) -> Result<CompiledRule, ParseError> {
    let mut lhs_ix: AHashMap<&str, usize> = AHashMap::default();
    for (i, v) in rule.lhs.nodes.iter().enumerate() {
        lhs_ix.insert(v.as_str(), i);
    }

    // LHS edges.
    let mut edges = Vec::with_capacity(rule.lhs.edges.len());
    for (ei, edge_vars) in rule.lhs.edges.iter().enumerate() {
        let mut var_ixs = Vec::with_capacity(edge_vars.len());
        for v in edge_vars {
            let ix = lhs_ix.get(v.as_str()).ok_or_else(|| {
                ParseError::UndeclaredVariable(v.clone(), "LHS")
            })?;
            var_ixs.push(*ix);
        }
        edges.push(PatternEdge {
            var_ixs,
            label: rule.lhs.edge_labels.get(&ei).copied(),
            ordered: false,
        });
    }

    // Node-label constraints by index.
    let mut node_label_constraints = vec![None; rule.lhs.nodes.len()];
    for (var, label) in &rule.lhs.node_labels {
        if let Some(&ix) = lhs_ix.get(var.as_str()) {
            node_label_constraints[ix] = Some(*label);
        }
    }

    // Property guards.
    let mut property_guards = Vec::new();
    for g in &rule.lhs_node_property_guards {
        let ix = lhs_ix.get(g.var.as_str()).ok_or_else(|| {
            ParseError::InvalidGuard(format!("guard var '{}' not in LHS", g.var))
        })?;
        property_guards.push(CompiledPropertyGuard {
            var_ix: *ix,
            key: g.property,
            op: g.op,
            value: g.value,
        });
    }

    // Property-to-property comparison guards (hg-alife S1): resolve both
    // variables to LHS indices. Parse-time validation already checked both
    // vars are declared in the LHS; re-check and error rather than panic.
    let mut property_compares = Vec::new();
    for c in &rule.lhs_node_property_compares {
        let var_a_ix = *lhs_ix.get(c.var_a.as_str()).ok_or_else(|| {
            ParseError::InvalidGuard(format!("compare var_a '{}' not in LHS", c.var_a))
        })?;
        let var_b_ix = *lhs_ix.get(c.var_b.as_str()).ok_or_else(|| {
            ParseError::InvalidGuard(format!("compare var_b '{}' not in LHS", c.var_b))
        })?;
        property_compares.push(CompiledPropertyCompare {
            var_a_ix,
            key_a: c.key_a,
            op: c.op,
            var_b_ix,
            key_b: c.key_b,
            offset: c.offset,
        });
    }

    // Auto-generate pairwise distinctness from each vars_distinct guard.
    // Degree comparison guards (v4 §6.2).
    let mut degree_compares = Vec::new();
    for d in &rule.lhs_node_degree_compares {
        let var_ix = *lhs_ix.get(d.var.as_str()).ok_or_else(|| {
            ParseError::InvalidGuard(format!("degree-compare var '{}' not in LHS", d.var))
        })?;
        degree_compares.push(CompiledDegreeCompare {
            var_ix,
            op: d.op,
            value: d.value,
            offset: d.offset,
        });
    }

    // E-track: var-to-var quantity compares. Both vars must be declared in the
    // LHS — a comparison against an undeclared var can never be evaluated, so it
    // is a parse error rather than a guard that silently returns false forever.
    let mut quantity_compares = Vec::new();
    for q in &rule.lhs_node_quantity_compares {
        let var_a_ix = *lhs_ix.get(q.var_a.as_str()).ok_or_else(|| {
            ParseError::InvalidGuard(format!("quantity-compare var_a '{}' not in LHS", q.var_a))
        })?;
        let var_b_ix = *lhs_ix.get(q.var_b.as_str()).ok_or_else(|| {
            ParseError::InvalidGuard(format!("quantity-compare var_b '{}' not in LHS", q.var_b))
        })?;
        quantity_compares.push(CompiledQuantityCompare {
            var_a_ix,
            quantity: q.quantity,
            op: q.op,
            var_b_ix,
            offset: q.offset,
        });
    }

    let mut distinct_pairs = Vec::new();
    for g in &rule.lhs_var_guards {
        match g.guard_type {
            VarGuardType::VarsDistinct => {
                let ixs: Result<Vec<usize>, ParseError> = g
                    .vars
                    .iter()
                    .map(|v| {
                        lhs_ix
                            .get(v.as_str())
                            .copied()
                            .ok_or_else(|| ParseError::InvalidGuard(format!(
                                "vars_distinct var '{v}' not in LHS"
                            )))
                    })
                    .collect();
                let ixs = ixs?;
                for i in 0..ixs.len() {
                    for j in (i + 1)..ixs.len() {
                        let (a, b) = (ixs[i].min(ixs[j]), ixs[i].max(ixs[j]));
                        if a != b && !distinct_pairs.contains(&(a, b)) {
                            distinct_pairs.push((a, b));
                        }
                    }
                }
            }
        }
    }

    // RHS mapping.
    let mut rhs_from_lhs = Vec::with_capacity(rule.rhs.nodes.len());
    let mut rhs_new_labels = Vec::with_capacity(rule.rhs.nodes.len());
    let mut rhs_relabels = Vec::with_capacity(rule.rhs.nodes.len());
    let mut rhs_ix: AHashMap<&str, usize> = AHashMap::default();
    for (i, v) in rule.rhs.nodes.iter().enumerate() {
        rhs_ix.insert(v.as_str(), i);
        let survives = lhs_ix.get(v.as_str()).copied();
        rhs_from_lhs.push(survives);
        let explicit = rule
            .rhs_node_labels
            .get(v)
            .or_else(|| rule.rhs.node_labels.get(v))
            .copied();
        rhs_new_labels.push(explicit.unwrap_or(Label(0)));
        // Survivor + explicit RHS label = relabel on apply (the mark
        // transfer). Created nodes are labelled at creation instead.
        rhs_relabels.push(if survives.is_some() { explicit } else { None });
    }

    // RHS edges with sparse labels.
    let mut rhs_edges = Vec::with_capacity(rule.rhs.edges.len());
    for (ei, edge_vars) in rule.rhs.edges.iter().enumerate() {
        let mut var_ixs = Vec::with_capacity(edge_vars.len());
        for v in edge_vars {
            let ix = rhs_ix.get(v.as_str()).ok_or_else(|| {
                ParseError::UndeclaredVariable(v.clone(), "RHS")
            })?;
            var_ixs.push(*ix);
        }
        let label = rule.rhs.edge_labels.get(&ei).copied().unwrap_or(Label(0));
        rhs_edges.push((var_ixs, label));
    }

    // Property effects: resolve target/source names to RHS indices.
    // Parse-time validation already rejected non-RHS names; re-check and
    // error rather than panic (same policy as the rest of compile).
    let mut property_effects = Vec::with_capacity(rule.rhs_property_effects.len());
    for e in &rule.rhs_property_effects {
        let target_ix = *rhs_ix.get(e.var.as_str()).ok_or_else(|| {
            ParseError::InvalidPropertyEffect(
                rule.name.clone(),
                format!("target var '{}' not in RHS", e.var),
            )
        })?;
        let resolve_source = |name: &str| -> Result<usize, ParseError> {
            rhs_ix.get(name).copied().ok_or_else(|| {
                ParseError::InvalidPropertyEffect(
                    rule.name.clone(),
                    format!("source_var '{name}' not in RHS"),
                )
            })
        };
        let op = match &e.op {
            EffectOpDecl::Set { value } => CompiledEffectOp::Set { value: *value },
            EffectOpDecl::CopyFrom { source_var, source_key } => CompiledEffectOp::CopyFrom {
                source_ix: resolve_source(source_var)?,
                source_key: *source_key,
            },
            EffectOpDecl::ScaleFrom { source_var, source_key, factor } => {
                CompiledEffectOp::ScaleFrom {
                    source_ix: resolve_source(source_var)?,
                    source_key: *source_key,
                    factor: *factor,
                }
            }
            EffectOpDecl::Add { value } => CompiledEffectOp::Add { value: *value },
            EffectOpDecl::Subtract { value } => CompiledEffectOp::Subtract { value: *value },
            EffectOpDecl::AddFromProperty { source_var, source_key } => {
                CompiledEffectOp::AddFromProperty {
                    source_ix: resolve_source(source_var)?,
                    source_key: *source_key,
                }
            }
            EffectOpDecl::SubtractFromProperty { source_var, source_key } => {
                CompiledEffectOp::SubtractFromProperty {
                    source_ix: resolve_source(source_var)?,
                    source_key: *source_key,
                }
            }
            EffectOpDecl::Jitter { magnitude } => CompiledEffectOp::Jitter { magnitude: *magnitude },
        };
        property_effects.push(PropertyEffect {
            target_ix,
            key: e.key,
            op,
        });
    }

    // Deleted LHS vars.
    let rhs_names: std::collections::HashSet<&str> =
        rule.rhs.nodes.iter().map(|s| s.as_str()).collect();
    let lhs_deleted: Vec<usize> = rule
        .lhs
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, v)| !rhs_names.contains(v.as_str()))
        .map(|(i, _)| i)
        .collect();

    Ok(CompiledRule {
        meta_visible: false,
        name: rule.name.clone(),
        priority: rule.priority,
        weight: rule.weight,
        enabled: rule.enabled,
        schedule: rule.schedule,
        pattern: MatchPattern {
            node_label_ceiling: None,
            match_work_limit: None,
            match_result_limit: None,
            n_vars: rule.lhs.nodes.len(),
            edges,
            node_label_constraints,
            property_guards,
            property_compares,
            degree_compares,
            quantity_compares,
            distinct_pairs,
        },
        template: RewriteTemplate {
            n_rhs_vars: rule.rhs.nodes.len(),
            rhs_from_lhs,
            rhs_new_labels,
            rhs_relabels,
            rhs_edges,
            lhs_deleted,
            property_effects,
        },
        lhs_var_names: rule.lhs.nodes.clone(),
    })
}

/// Compile a whole rule set, preserving file order (rule_id = index).
pub fn compile_rules(rules: &[Rule]) -> Result<Vec<CompiledRule>, ParseError> {
    rules.iter().map(compile_rule).collect()
}
