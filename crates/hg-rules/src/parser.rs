//! TOML parser for rules (v4 §5)

use crate::{Rule, PatternGraph, NodePropertyCompare, NodePropertyGuard, VarGuard, Schedule};
use crate::guards::{NodeDegreeCompare, NodeQuantity, NodeQuantityCompare};
use crate::effects::{EffectOpDecl, PropertyEffectDecl};
use crate::guards::VarGuardType;
use hg_core::{Label, PropertyOp, PropertyValue};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Duplicate rule name: {0}")]
    DuplicateRuleName(String),
    #[error("Variable {0} in {1} edges not declared in {1} nodes")]
    UndeclaredVariable(String, &'static str),
    #[error("Empty rule name")]
    EmptyRuleName,
    #[error("Invalid guard: {0}")]
    InvalidGuard(String),
    #[error("Rule '{0}': edge_labels index {1} out of range ({2} edges)")]
    EdgeLabelIndexOutOfRange(String, usize, usize),
    #[error("Rule '{0}' is a creation rule (empty LHS) and must carry a [rules.schedule] section")]
    CreationRuleMissingSchedule(String),
    #[error("Rule '{0}' has a non-empty LHS pattern and must not carry a [rules.schedule] section")]
    NonCreationRuleWithSchedule(String),
    #[error("Rule '{0}': invalid schedule: {1}")]
    InvalidSchedule(String, String),
    #[error("Rule '{0}': duplicate variable '{1}' in {2} nodes")]
    DuplicateVariable(String, String, &'static str),
    #[error("Rule '{0}': invalid property effect: {1}")]
    InvalidPropertyEffect(String, String),
}

#[derive(Deserialize)]
struct RawRulesFile {
    rules: Vec<RawRule>,
}

#[derive(Deserialize)]
struct RawRule {
    name: String,
    #[serde(default)]
    priority: i32,
    #[serde(default = "default_weight")]
    weight: f64,
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default)]
    lhs: RawPatternGraph,
    rhs: RawPatternGraph,
    #[serde(default)]
    schedule: Option<RawSchedule>,
    // Legacy top-level guard spellings, still accepted:
    #[serde(default)]
    lhs_node_property_guards: Vec<RawNodePropertyGuard>,
    #[serde(default)]
    lhs_node_property_compares: Vec<RawNodePropertyCompare>,
    #[serde(default)]
    lhs_var_guards: Vec<RawVarGuard>,
    #[serde(default)]
    rhs_node_labels: HashMap<String, u32>,
}

fn default_weight() -> f64 { 1.0 }
fn default_enabled() -> bool { true }

#[derive(Deserialize, Default)]
struct RawPatternGraph {
    #[serde(default)]
    nodes: Vec<String>,
    #[serde(default)]
    edges: Vec<Vec<String>>,
    #[serde(default)]
    node_labels: HashMap<String, u32>,
    /// Sparse map keyed by edge index; TOML table keys are strings.
    #[serde(default)]
    edge_labels: HashMap<String, u32>,
    /// Documented grammar location for guards ([[rules.lhs....]]).
    #[serde(default)]
    node_property_guards: Vec<RawNodePropertyGuard>,
    /// Property-to-property comparison guards ([[rules.lhs.node_property_compares]]).
    #[serde(default)]
    node_property_compares: Vec<RawNodePropertyCompare>,
    #[serde(default)]
    node_degree_compares: Vec<RawNodeDegreeCompare>,
    /// Var-to-var node-quantity comparisons ([[rules.lhs.node_quantity_compares]]).
    #[serde(default)]
    node_quantity_compares: Vec<RawNodeQuantityCompare>,
    #[serde(default)]
    var_guards: Vec<RawVarGuard>,
    /// Property writes ([[rules.rhs.property_effects]]); meaningful on
    /// the RHS only — presence under lhs is a parse error.
    #[serde(default)]
    property_effects: Vec<RawPropertyEffect>,
}

/// One property effect as written in TOML. `deny_unknown_fields` so a
/// typo'd field name is a parse error, not a silent no-op.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPropertyEffect {
    var: String,
    key: u32,
    op: String,
    #[serde(default)]
    value: Option<PropertyValue>,
    #[serde(default)]
    source_var: Option<String>,
    #[serde(default)]
    source_key: Option<u32>,
    #[serde(default)]
    factor: Option<f64>,
    #[serde(default)]
    magnitude: Option<f64>,
}

#[derive(Deserialize)]
struct RawNodePropertyGuard {
    var: String,
    property: u32,
    op: String,
    value: PropertyValue,
}

/// One degree comparison guard as written in TOML (plan v4 §6.2).
/// `deny_unknown_fields` so a typo'd field is a parse error, not a silent no-op.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNodeDegreeCompare {
    var: String,
    op: String,
    value: i64,
    #[serde(default)]
    offset: Option<i64>,
}

/// One var-to-var node-quantity comparison as written in TOML (E-track).
/// `deny_unknown_fields` so a typo'd field is a parse error, not a silent no-op.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNodeQuantityCompare {
    var_a: String,
    quantity: String,
    op: String,
    var_b: String,
    #[serde(default)]
    offset: Option<i64>,
}

/// One property-to-property comparison guard as written in TOML (hg-alife S1).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNodePropertyCompare {
    var_a: String,
    key_a: u32,
    op: String,
    var_b: String,
    key_b: u32,
    /// Optional constant added to side B before comparing (LK-R v2).
    #[serde(default)]
    offset: Option<PropertyValue>,
}

#[derive(Deserialize)]
struct RawVarGuard {
    #[serde(rename = "type")]
    guard_type: String,
    vars: Vec<String>,
}

#[derive(Deserialize)]
struct RawSchedule {
    mode: String,
    #[serde(default)]
    n: Option<u64>,
}

fn parse_op(op: &str) -> Result<PropertyOp, ParseError> {
    Ok(match op {
        "eq" => PropertyOp::Eq,
        "ne" => PropertyOp::Ne,
        "lt" => PropertyOp::Lt,
        "gt" => PropertyOp::Gt,
        "lte" => PropertyOp::Lte,
        "gte" => PropertyOp::Gte,
        other => return Err(ParseError::InvalidGuard(format!("Unknown op: {other}"))),
    })
}

fn parse_edge_labels(
    rule_name: &str,
    raw: &HashMap<String, u32>,
    n_edges: usize,
) -> Result<BTreeMap<usize, Label>, ParseError> {
    let mut out = BTreeMap::new();
    for (k, v) in raw {
        let ix: usize = k.parse().map_err(|_| {
            ParseError::InvalidGuard(format!("edge_labels key '{k}' is not an edge index"))
        })?;
        if ix >= n_edges {
            return Err(ParseError::EdgeLabelIndexOutOfRange(
                rule_name.to_string(),
                ix,
                n_edges,
            ));
        }
        out.insert(ix, Label(*v));
    }
    Ok(out)
}

fn validate_pattern(
    rule_name: &str,
    raw: &RawPatternGraph,
    side: &'static str,
) -> Result<(), ParseError> {
    let mut seen = std::collections::HashSet::new();
    for v in &raw.nodes {
        if !seen.insert(v.clone()) {
            return Err(ParseError::DuplicateVariable(
                rule_name.to_string(),
                v.clone(),
                side,
            ));
        }
    }
    for edge in &raw.edges {
        for var in edge {
            if !seen.contains(var) {
                return Err(ParseError::UndeclaredVariable(var.clone(), side));
            }
        }
    }
    Ok(())
}

pub fn parse_rules_toml(content: &str) -> Result<Vec<Rule>, ParseError> {
    let raw: RawRulesFile = toml::from_str(content)?;

    let mut rules = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for raw_rule in raw.rules {
        if raw_rule.name.is_empty() {
            return Err(ParseError::EmptyRuleName);
        }
        if !seen_names.insert(raw_rule.name.clone()) {
            return Err(ParseError::DuplicateRuleName(raw_rule.name));
        }

        validate_pattern(&raw_rule.name, &raw_rule.lhs, "LHS")?;
        validate_pattern(&raw_rule.name, &raw_rule.rhs, "RHS")?;

        // Creation-rule schedule validation, both directions (v4 §5.3).
        let is_creation = raw_rule.lhs.nodes.is_empty() && raw_rule.lhs.edges.is_empty();
        let schedule = match (&raw_rule.schedule, is_creation) {
            (None, true) => {
                return Err(ParseError::CreationRuleMissingSchedule(raw_rule.name))
            }
            (Some(_), false) => {
                return Err(ParseError::NonCreationRuleWithSchedule(raw_rule.name))
            }
            (Some(s), true) => Some(match s.mode.as_str() {
                "once" => Schedule::Once,
                "every_n_steps" => {
                    let n = s.n.ok_or_else(|| ParseError::InvalidSchedule(
                        raw_rule.name.clone(),
                        "mode 'every_n_steps' requires n".into(),
                    ))?;
                    if n == 0 {
                        return Err(ParseError::InvalidSchedule(
                            raw_rule.name.clone(),
                            "n must be > 0".into(),
                        ));
                    }
                    Schedule::EveryNSteps(n)
                }
                other => {
                    return Err(ParseError::InvalidSchedule(
                        raw_rule.name.clone(),
                        format!("unknown mode '{other}'"),
                    ))
                }
            }),
            (None, false) => None,
        };

        let lhs_edge_labels =
            parse_edge_labels(&raw_rule.name, &raw_rule.lhs.edge_labels, raw_rule.lhs.edges.len())?;
        let rhs_edge_labels =
            parse_edge_labels(&raw_rule.name, &raw_rule.rhs.edge_labels, raw_rule.rhs.edges.len())?;

        let lhs_vars: std::collections::HashSet<_> =
            raw_rule.lhs.nodes.iter().cloned().collect();

        // Guards: documented nested spelling plus legacy top-level spelling.
        let mut lhs_node_property_guards = Vec::new();
        for g in raw_rule.lhs.node_property_guards.iter().chain(raw_rule.lhs_node_property_guards.iter()) {
            if !lhs_vars.contains(&g.var) {
                return Err(ParseError::InvalidGuard(format!(
                    "Property guard variable '{}' not declared in LHS nodes",
                    g.var
                )));
            }
            lhs_node_property_guards.push(NodePropertyGuard {
                var: g.var.clone(),
                property: g.property,
                op: parse_op(&g.op)?,
                value: g.value,
            });
        }

        // Property-to-property comparison guards (hg-alife S1): documented
        // nested spelling plus legacy top-level spelling. Both variables must
        // be declared in the LHS nodes.
        let mut lhs_node_property_compares = Vec::new();
        for c in raw_rule
            .lhs
            .node_property_compares
            .iter()
            .chain(raw_rule.lhs_node_property_compares.iter())
        {
            for (role, var) in [("var_a", &c.var_a), ("var_b", &c.var_b)] {
                if !lhs_vars.contains(var) {
                    return Err(ParseError::InvalidGuard(format!(
                        "Property-compare {role} '{var}' not declared in LHS nodes"
                    )));
                }
            }
            lhs_node_property_compares.push(NodePropertyCompare {
                var_a: c.var_a.clone(),
                key_a: c.key_a,
                op: parse_op(&c.op)?,
                var_b: c.var_b.clone(),
                key_b: c.key_b,
                offset: c.offset,
            });
        }

        // Degree comparison guards (plan v4 §6.2).
        let mut lhs_node_degree_compares = Vec::new();
        for d in &raw_rule.lhs.node_degree_compares {
            if !lhs_vars.contains(&d.var) {
                return Err(ParseError::InvalidGuard(format!(
                    "Degree-compare var '{}' not declared in LHS nodes",
                    d.var
                )));
            }
            lhs_node_degree_compares.push(NodeDegreeCompare {
                var: d.var.clone(),
                op: parse_op(&d.op)?,
                value: d.value,
                offset: d.offset,
            });
        }

        // E-track: var-to-var node-quantity comparisons. The quantity name is
        // validated here, so an unknown quantity is a PARSE error rather than a
        // guard that silently never fires.
        let mut lhs_node_quantity_compares = Vec::new();
        for q in &raw_rule.lhs.node_quantity_compares {
            for v in [&q.var_a, &q.var_b] {
                if !lhs_vars.contains(v) {
                    return Err(ParseError::InvalidGuard(format!(
                        "Quantity-compare var '{v}' not declared in LHS nodes"
                    )));
                }
            }
            let quantity = match q.quantity.as_str() {
                "degree" => NodeQuantity::Degree,
                other => {
                    return Err(ParseError::InvalidGuard(format!(
                        "Unknown node quantity '{other}' (supported: degree)"
                    )))
                }
            };
            lhs_node_quantity_compares.push(NodeQuantityCompare {
                var_a: q.var_a.clone(),
                quantity,
                op: parse_op(&q.op)?,
                var_b: q.var_b.clone(),
                offset: q.offset,
            });
        }

        let mut lhs_var_guards = Vec::new();
        for g in raw_rule.lhs.var_guards.iter().chain(raw_rule.lhs_var_guards.iter()) {
            let guard_type = match g.guard_type.as_str() {
                "vars_distinct" => VarGuardType::VarsDistinct,
                other => {
                    return Err(ParseError::InvalidGuard(format!(
                        "Unknown guard type: {other}"
                    )))
                }
            };
            for var in &g.vars {
                if !lhs_vars.contains(var) {
                    return Err(ParseError::InvalidGuard(format!(
                        "Var guard variable '{var}' not declared in LHS nodes"
                    )));
                }
            }
            lhs_var_guards.push(VarGuard {
                guard_type,
                vars: g.vars.clone(),
            });
        }

        // Property effects: RHS only; every var/source_var must be an
        // RHS variable. LHS-only (deleted) vars are rejected HERE with an
        // explanatory error — their nodes are gone before effects run.
        if !raw_rule.lhs.property_effects.is_empty() {
            return Err(ParseError::InvalidPropertyEffect(
                raw_rule.name.clone(),
                "property_effects belong under [rules.rhs], not [rules.lhs] — \
effects run after the rewrite and can only touch RHS-bound nodes"
                    .into(),
            ));
        }
        let rhs_vars: std::collections::HashSet<_> =
            raw_rule.rhs.nodes.iter().cloned().collect();
        let eff_err = |msg: String| ParseError::InvalidPropertyEffect(raw_rule.name.clone(), msg);
        let check_rhs_var = |name: &str, role: &str| -> Result<(), ParseError> {
            if rhs_vars.contains(name) {
                Ok(())
            } else if lhs_vars.contains(name) {
                Err(eff_err(format!(
                    "{role} '{name}' is an LHS-only variable — this rule DELETES \
its node (present in LHS, absent from RHS), and deletion happens before \
property effects run, so there is nothing to read or write. Keep the \
variable in the RHS if its properties are needed."
                )))
            } else {
                Err(eff_err(format!("{role} '{name}' is not declared in RHS nodes")))
            }
        };
        let mut rhs_property_effects = Vec::with_capacity(raw_rule.rhs.property_effects.len());
        for e in &raw_rule.rhs.property_effects {
            check_rhs_var(&e.var, "target var")?;
            let need = |field: &str, present: bool| -> Result<(), ParseError> {
                if present {
                    Ok(())
                } else {
                    Err(eff_err(format!("op '{}' requires field '{field}'", e.op)))
                }
            };
            let op = match e.op.as_str() {
                "set" => {
                    need("value", e.value.is_some())?;
                    EffectOpDecl::Set { value: e.value.unwrap() }
                }
                "copy_from" => {
                    need("source_var", e.source_var.is_some())?;
                    need("source_key", e.source_key.is_some())?;
                    let sv = e.source_var.clone().unwrap();
                    check_rhs_var(&sv, "source_var")?;
                    EffectOpDecl::CopyFrom {
                        source_var: sv,
                        source_key: e.source_key.unwrap(),
                    }
                }
                "scale_from" => {
                    need("source_var", e.source_var.is_some())?;
                    need("source_key", e.source_key.is_some())?;
                    need("factor", e.factor.is_some())?;
                    let sv = e.source_var.clone().unwrap();
                    check_rhs_var(&sv, "source_var")?;
                    EffectOpDecl::ScaleFrom {
                        source_var: sv,
                        source_key: e.source_key.unwrap(),
                        factor: e.factor.unwrap(),
                    }
                }
                "add" => {
                    need("value", e.value.is_some())?;
                    EffectOpDecl::Add { value: e.value.unwrap() }
                }
                "subtract" => {
                    need("value", e.value.is_some())?;
                    EffectOpDecl::Subtract { value: e.value.unwrap() }
                }
                "add_from_property" => {
                    need("source_var", e.source_var.is_some())?;
                    need("source_key", e.source_key.is_some())?;
                    let sv = e.source_var.clone().unwrap();
                    check_rhs_var(&sv, "source_var")?;
                    EffectOpDecl::AddFromProperty {
                        source_var: sv,
                        source_key: e.source_key.unwrap(),
                    }
                }
                "subtract_from_property" => {
                    need("source_var", e.source_var.is_some())?;
                    need("source_key", e.source_key.is_some())?;
                    let sv = e.source_var.clone().unwrap();
                    check_rhs_var(&sv, "source_var")?;
                    EffectOpDecl::SubtractFromProperty {
                        source_var: sv,
                        source_key: e.source_key.unwrap(),
                    }
                }
                "jitter" => {
                    need("magnitude", e.magnitude.is_some())?;
                    let m = e.magnitude.unwrap();
                    if m <= 0.0 {
                        return Err(eff_err(format!("jitter magnitude must be > 0, got {m}")));
                    }
                    EffectOpDecl::Jitter { magnitude: m }
                }
                other => return Err(eff_err(format!("unknown op '{other}'"))),
            };
            rhs_property_effects.push(PropertyEffectDecl {
                var: e.var.clone(),
                key: e.key,
                op,
            });
        }

        let lhs = PatternGraph {
            nodes: raw_rule.lhs.nodes,
            edges: raw_rule.lhs.edges,
            node_labels: raw_rule.lhs.node_labels.into_iter()
                .map(|(k, v)| (k, Label(v)))
                .collect(),
            edge_labels: lhs_edge_labels,
        };
        let rhs = PatternGraph {
            nodes: raw_rule.rhs.nodes,
            edges: raw_rule.rhs.edges,
            node_labels: raw_rule.rhs.node_labels.into_iter()
                .map(|(k, v)| (k, Label(v)))
                .collect(),
            edge_labels: rhs_edge_labels,
        };

        rules.push(Rule {
            name: raw_rule.name,
            priority: raw_rule.priority,
            weight: raw_rule.weight,
            enabled: raw_rule.enabled,
            lhs,
            rhs,
            lhs_node_property_guards,
            lhs_node_property_compares,
            lhs_node_degree_compares,
            lhs_node_quantity_compares,
            lhs_var_guards,
            rhs_node_labels: raw_rule.rhs_node_labels.into_iter()
                .map(|(k, v)| (k, Label(v)))
                .collect(),
            rhs_property_effects,
            schedule,
        });
    }

    // File order is preserved: rule_id = index. Priority is applied in
    // conflict resolution (Reverse<i32> in the sort key), not by reordering.
    Ok(rules)
}

pub fn load_rules_from_file(path: &std::path::Path) -> Result<Vec<Rule>, ParseError> {
    let content = std::fs::read_to_string(path)?;
    parse_rules_toml(&content)
}
