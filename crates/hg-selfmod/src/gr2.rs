//! G-R2 expressivity envelope (EXP-R02).
//!
//! Protocol: `docs/ere_gr2_protocol.md`. Converts every shipped TOML rule to
//! a `RuleSpec` (reporting the exact inexpressibility reason where it cannot),
//! round-trips the expressible ones through encode→decode→compile, and asserts
//! behavioral identity against the directly-compiled rule over long runs on
//! auto-synthesized fixture graphs (engagement-gated: a rule that never fires
//! is Unmeasured, never "identical").

use crate::decode::{decode_all, DecodedRule};
use crate::encode::install_rule;
use crate::runner::{object_step, RunConfig};
use crate::schema::*;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_rules::{compile_rule, load_rules_from_file, Rule};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use smallvec::SmallVec;
use std::collections::HashMap;

// ---------- constants (mirrored in docs/ere_gr2_protocol.md) ----------

/// Behavioral comparison length (plan §6: 1000 steps; FM-R3 bloat-halt may
/// end trajectories early — see `GR2_BLOAT_*`).
pub const GR2_STEPS: usize = 1000;
/// Auto-fixture background soup size.
pub const GR2_SOUP_NODES: usize = 64;
pub const GR2_SOUP_EDGES: usize = 96;
pub const GR2_SOUP_LABELS: u32 = 6;
/// Fixture construction stream (soup + seed layout).
pub const GR2_FIXTURE_SEED: u64 = 0x6232_0000_0000_0001;
/// Per-trajectory object-step stream (same for both arms).
pub const GR2_TRAJ_SEED: u64 = 0x6232_0000_0000_0002;
/// Differential-sweep stream base (xor per-world counter).
pub const GR2_SWEEP_SEED: u64 = 0x6232_0000_0000_0003;
/// Sweep world count (1 step each, all must match exactly).
pub const GR2_SWEEP_WORLDS: usize = 200;
/// FM-R3 bloat halt: stop a trajectory past these sizes (both arms must halt
/// together — a halt-step mismatch is divergence).
pub const GR2_BLOAT_NODES: usize = 2000;
pub const GR2_BLOAT_EDGES: usize = 4000;
/// Minimum compared trajectory steps for a Measured verdict.
pub const GR2_MIN_TRAJ_STEPS: usize = 10;

/// Encoding-reach caps for the envelope (E1's `MAX_ARITY = 8` scale), NOT the
/// evolution experiment caps (4/4/3/3). The envelope measures what the
/// *encoding* can say; experiment throttles are a separate decision.
pub fn gr2_caps() -> Caps {
    Caps {
        lhs_max_nodes: 8,
        lhs_max_edges: 16,
        max_arity: 8,
        rhs_max_create: 8,
        n_labels: GR2_SOUP_LABELS,
    }
}

// ---------- conversion ----------

/// Why a TOML rule cannot be expressed as a `RuleSpec`. Ordered from
/// fundamental (no cap bump fixes it) to parametric. See
/// `docs/ere_encoding_schema.md` §1.4 for the envelope limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inexpressible {
    /// Creation-rule schedule (SB-R2: E1 cannot encode schedules).
    Schedule,
    /// LHS edge-label constraints (E1c).
    LhsEdgeLabels,
    /// Non-default RHS edge labels (spec path always emits Label(0)).
    RhsEdgeLabels,
    /// Property-to-property comparison guards.
    PropertyCompare,
    /// Degree comparison guards.
    DegreeCompare,
    /// Var-to-var quantity comparison guards.
    QuantityCompare,
    /// Guard op outside {Gt, Lt, Eq} (lte/gte/ne have no encoding).
    GuardOp(String),
    /// Non-default rule weight (spec path drops it; matters only for
    /// SequentialWeighted, but the envelope is mode-independent).
    Weight,
    /// LHS var count above the cap.
    TooManyVars(usize),
    /// LHS edge count above the cap.
    TooManyLhsEdges,
    /// Edge arity above the cap.
    Arity(usize),
    /// Create count above the cap.
    TooManyCreates,
    /// Malformed rule (edge/guard references undeclared var) — defensive;
    /// shipped rules are parse-valid so this must never fire.
    BadVar(String),
    /// Effectless rule (no consumable pattern, deletes, relabels, creates,
    /// or edges) — mirrors the V9 vacuity quarantine.
    Vacuous,
    /// Expressible, but the auto-fixture cannot synthesize engagement
    /// (only reachable for exotic guard shapes; loud when hit).
    FixtureUnsupported(String),
}

impl std::fmt::Display for Inexpressible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Inexpressible::Schedule => write!(f, "schedule(SB-R2)"),
            Inexpressible::LhsEdgeLabels => write!(f, "lhs-edge-labels(E1c)"),
            Inexpressible::RhsEdgeLabels => write!(f, "rhs-edge-labels"),
            Inexpressible::PropertyCompare => write!(f, "property-compare"),
            Inexpressible::DegreeCompare => write!(f, "degree-compare"),
            Inexpressible::QuantityCompare => write!(f, "quantity-compare"),
            Inexpressible::GuardOp(op) => write!(f, "guard-op({op})"),
            Inexpressible::Weight => write!(f, "weight"),
            Inexpressible::TooManyVars(n) => write!(f, "too-many-vars({n})"),
            Inexpressible::TooManyLhsEdges => write!(f, "too-many-lhs-edges"),
            Inexpressible::Arity(a) => write!(f, "arity({a})"),
            Inexpressible::TooManyCreates => write!(f, "too-many-creates"),
            Inexpressible::BadVar(v) => write!(f, "bad-var({v})"),
            Inexpressible::Vacuous => write!(f, "vacuous(V9)"),
            Inexpressible::FixtureUnsupported(w) => write!(f, "fixture-unsupported({w})"),
        }
    }
}

/// Convert a parsed TOML rule to a `RuleSpec`. Variable order follows
/// `lhs.nodes` (same convention as `compile_rule`); creates follow
/// `rhs.nodes` order. Pure function of the rule + caps (no RNG, no store).
pub fn rule_to_spec(rule: &Rule, caps: &Caps) -> Result<RuleSpec, Inexpressible> {
    if rule.schedule.is_some() {
        return Err(Inexpressible::Schedule);
    }
    if !rule.lhs.edge_labels.is_empty() {
        return Err(Inexpressible::LhsEdgeLabels);
    }
    if rule.rhs.edge_labels.values().any(|&l| l != Label(0)) {
        return Err(Inexpressible::RhsEdgeLabels);
    }
    if !rule.lhs_node_property_compares.is_empty() {
        return Err(Inexpressible::PropertyCompare);
    }
    if !rule.lhs_node_degree_compares.is_empty() {
        return Err(Inexpressible::DegreeCompare);
    }
    if !rule.lhs_node_quantity_compares.is_empty() {
        return Err(Inexpressible::QuantityCompare);
    }
    if rule.weight != 1.0 {
        return Err(Inexpressible::Weight);
    }
    let n_vars = rule.lhs.nodes.len();
    if n_vars > caps.lhs_max_nodes {
        return Err(Inexpressible::TooManyVars(n_vars));
    }
    if rule.lhs.edges.len() > caps.lhs_max_edges {
        return Err(Inexpressible::TooManyLhsEdges);
    }
    let ix = |name: &str| -> Result<u8, Inexpressible> {
        rule.lhs
            .nodes
            .iter()
            .position(|v| v == name)
            .map(|i| i as u8)
            .ok_or_else(|| Inexpressible::BadVar(name.to_string()))
    };
    // LHS pattern edges.
    let mut pedges = Vec::with_capacity(rule.lhs.edges.len());
    for e in &rule.lhs.edges {
        if e.len() > caps.max_arity {
            return Err(Inexpressible::Arity(e.len()));
        }
        pedges.push(e.iter().map(|v| ix(v)).collect::<Result<Vec<u8>, _>>()?);
    }
    // Label constraints (undeclared names ignored, mirroring compile_rule).
    let mut var_labels = vec![None; n_vars];
    for (var, label) in &rule.lhs.node_labels {
        if let Some(i) = rule.lhs.nodes.iter().position(|v| v == var) {
            var_labels[i] = Some(label.0);
        }
    }
    // Property guards: the full PropertyOp set (E1-G completion).
    let mut guards = Vec::with_capacity(rule.lhs_node_property_guards.len());
    for g in &rule.lhs_node_property_guards {
        let op = match g.op {
            hg_core::PropertyOp::Gt => 0,
            hg_core::PropertyOp::Lt => 1,
            hg_core::PropertyOp::Eq => 2,
            hg_core::PropertyOp::Lte => 3,
            hg_core::PropertyOp::Gte => 4,
            hg_core::PropertyOp::Ne => 5,
        };
        guards.push(GuardSpec {
            var: ix(&g.var)?,
            key: g.property,
            op,
            val: g.value,
        });
    }
    // RHS mapping (mirrors compile_rule §§269-301).
    // VarsDistinct (E1d, lifted): each guard's var list expands to all
    // unordered pairs, exactly as compile_rule expands them.
    let mut distinct = Vec::new();
    for g in &rule.lhs_var_guards {
        match g.guard_type {
            hg_rules::VarGuardType::VarsDistinct => {
                let mut ixs = Vec::with_capacity(g.vars.len());
                for v in &g.vars {
                    ixs.push(ix(v)?);
                }
                for i in 0..ixs.len() {
                    for j in (i + 1)..ixs.len() {
                        let (a, b) = (ixs[i].min(ixs[j]), ixs[i].max(ixs[j]));
                        if a != b && !distinct.contains(&(a, b)) {
                            distinct.push((a, b));
                        }
                    }
                }
            }
        }
    }
    distinct.sort_unstable();
    let mut deletes = Vec::new();
    for (i, v) in rule.lhs.nodes.iter().enumerate() {
        if !rule.rhs.nodes.iter().any(|w| w == v) {
            deletes.push(i as u8);
        }
    }
    let mut relabels = Vec::new();
    let mut creates: Vec<u32> = Vec::new();
    let mut create_ix: HashMap<&str, u8> = HashMap::new();
    for v in &rule.rhs.nodes {
        if rule.lhs.nodes.iter().any(|w| w == v) {
            if let Some(l) = rule
                .rhs_node_labels
                .get(v)
                .or_else(|| rule.rhs.node_labels.get(v))
            {
                relabels.push((ix(v)?, l.0));
            }
        } else {
            let k = creates.len() as u8;
            let l = rule
                .rhs_node_labels
                .get(v)
                .or_else(|| rule.rhs.node_labels.get(v))
                .copied()
                .unwrap_or(Label(0));
            creates.push(l.0);
            create_ix.insert(v.as_str(), k);
        }
    }
    if creates.len() > caps.rhs_max_create {
        return Err(Inexpressible::TooManyCreates);
    }
    let mut redges = Vec::with_capacity(rule.rhs.edges.len());
    for e in &rule.rhs.edges {
        if e.len() > caps.max_arity {
            return Err(Inexpressible::Arity(e.len()));
        }
        let mut refs = Vec::with_capacity(e.len());
        for v in e {
            if let Some(i) = rule.lhs.nodes.iter().position(|w| w == v) {
                refs.push(i as u8);
            } else if let Some(&k) = create_ix.get(v.as_str()) {
                refs.push(n_vars as u8 + k);
            } else {
                return Err(Inexpressible::BadVar(v.clone()));
            }
        }
        redges.push(refs);
    }
    deletes.sort_unstable();
    relabels.sort_unstable();
    // Property effects (E2): target/source names to refs (LHS position for
    // survivors, n_vars + create-index for creates — the same convention as
    // redges above, so behavior matches compile_rule exactly).
    let eref = |name: &str| -> Result<u8, Inexpressible> {
        if let Some(i) = rule.lhs.nodes.iter().position(|w| w == name) {
            Ok(i as u8)
        } else if let Some(&k) = create_ix.get(name) {
            Ok(n_vars as u8 + k)
        } else {
            Err(Inexpressible::BadVar(name.to_string()))
        }
    };
    let mut effects = Vec::with_capacity(rule.rhs_property_effects.len());
    for e in &rule.rhs_property_effects {
        use hg_rules::EffectOpDecl as D;
        let op = match &e.op {
            D::Set { value } => EffectOp::Set { value: *value },
            D::CopyFrom {
                source_var,
                source_key,
            } => EffectOp::CopyFrom {
                source: eref(source_var)?,
                source_key: *source_key,
            },
            D::ScaleFrom {
                source_var,
                source_key,
                factor,
            } => EffectOp::ScaleFrom {
                source: eref(source_var)?,
                source_key: *source_key,
                factor: *factor,
            },
            D::Add { value } => EffectOp::Add { value: *value },
            D::Subtract { value } => EffectOp::Subtract { value: *value },
            D::AddFromProperty {
                source_var,
                source_key,
            } => EffectOp::AddFromProperty {
                source: eref(source_var)?,
                source_key: *source_key,
            },
            D::SubtractFromProperty {
                source_var,
                source_key,
            } => EffectOp::SubtractFromProperty {
                source: eref(source_var)?,
                source_key: *source_key,
            },
            D::Jitter { magnitude } => EffectOp::Jitter {
                magnitude: *magnitude,
            },
        };
        effects.push(EffectSpec {
            target: eref(&e.var)?,
            key: e.key,
            op,
        });
    }
    let spec = RuleSpec {
        priority: rule.priority,
        n_vars: n_vars as u8,
        var_labels,
        pedges,
        guards,
        deletes,
        relabels,
        creates,
        redges,
        distinct,
        effects,
    };
    if spec.pedges.is_empty()
        && spec.deletes.is_empty()
        && spec.relabels.is_empty()
        && spec.creates.is_empty()
        && spec.redges.is_empty()
    {
        return Err(Inexpressible::Vacuous);
    }
    Ok(spec)
}

// ---------- auto-fixtures ----------

/// Every applicable inexpressibility reason for a rule (independent of the
/// first-wins order in `rule_to_spec`). Used for the "solely blocked by"
/// ranking that prioritizes encoding extensions. Pure TOML-level checks plus
/// cap checks; `BadVar` (parse-validity) is not re-derived here.
pub fn inexpressibility_reasons(rule: &Rule, caps: &Caps) -> Vec<Inexpressible> {
    let mut out = Vec::new();
    if rule.schedule.is_some() {
        out.push(Inexpressible::Schedule);
    }
    if !rule.lhs.edge_labels.is_empty() {
        out.push(Inexpressible::LhsEdgeLabels);
    }
    if rule.rhs.edge_labels.values().any(|&l| l != Label(0)) {
        out.push(Inexpressible::RhsEdgeLabels);
    }
    if !rule.lhs_node_property_compares.is_empty() {
        out.push(Inexpressible::PropertyCompare);
    }
    if !rule.lhs_node_degree_compares.is_empty() {
        out.push(Inexpressible::DegreeCompare);
    }
    if !rule.lhs_node_quantity_compares.is_empty() {
        out.push(Inexpressible::QuantityCompare);
    }
    if rule.weight != 1.0 {
        out.push(Inexpressible::Weight);
    }
    for g in &rule.lhs_node_property_guards {
        match g.op {
            hg_core::PropertyOp::Gt
            | hg_core::PropertyOp::Lt
            | hg_core::PropertyOp::Eq
            | hg_core::PropertyOp::Lte
            | hg_core::PropertyOp::Gte
            | hg_core::PropertyOp::Ne => {}
        }
    }
    if rule.lhs.nodes.len() > caps.lhs_max_nodes {
        out.push(Inexpressible::TooManyVars(rule.lhs.nodes.len()));
    }
    if rule.lhs.edges.len() > caps.lhs_max_edges {
        out.push(Inexpressible::TooManyLhsEdges);
    }
    for e in rule.lhs.edges.iter().chain(rule.rhs.edges.iter()) {
        if e.len() > caps.max_arity {
            out.push(Inexpressible::Arity(e.len()));
            break;
        }
    }
    let creates = rule
        .rhs
        .nodes
        .iter()
        .filter(|v| !rule.lhs.nodes.iter().any(|w| w == *v))
        .count();
    if creates > caps.rhs_max_create {
        out.push(Inexpressible::TooManyCreates);
    }
    // Vacuity from TOML side (mirrors V9): no consumable pattern, no
    // deletes, no survivor relabels, no creates, no rhs edges, no effects.
    let deletes = rule
        .lhs
        .nodes
        .iter()
        .any(|v| !rule.rhs.nodes.iter().any(|w| w == v));
    let relabels = rule.rhs.nodes.iter().any(|v| {
        rule.lhs.nodes.iter().any(|w| w == v)
            && (rule.rhs_node_labels.contains_key(v) || rule.rhs.node_labels.contains_key(v))
    });
    if rule.lhs.edges.is_empty()
        && !deletes
        && !relabels
        && creates == 0
        && rule.rhs.edges.is_empty()
        && rule.rhs_property_effects.is_empty()
    {
        out.push(Inexpressible::Vacuous);
    }
    out
}
/// Property value satisfying `prop OP threshold` (fixture synthesis).
/// `None` = unsatisfiable (Bool `Gt true` / `Lt false` — `false < true` is
/// the engine's total Bool order, so nothing exceeds `true`). An
/// unsatisfiable guard makes its rule dead everywhere: Unmeasured, honestly.
/// Float synthesis is precision-robust (`1e18 - 1.0 == 1e18` in f64 — a
/// naive ±1.0 can equal the threshold and silently unsatisfy a strict op).
fn satisfying_value(op: u8, threshold: PropertyValue) -> Option<PropertyValue> {
    /// Strictly above (finite inputs stay finite except extreme overflow to
    /// +inf, which still satisfies Gt/Gte).
    fn above(f: f64) -> f64 {
        let c = f + 1.0;
        if c > f {
            c
        } else if f > 0.0 {
            f * 2.0
        } else if f < 0.0 {
            f * 0.5
        } else {
            1.0
        }
    }
    /// Strictly below (symmetric).
    fn below(f: f64) -> f64 {
        let c = f - 1.0;
        if c < f {
            c
        } else if f > 0.0 {
            f * 0.5
        } else if f < 0.0 {
            f * 2.0
        } else {
            -1.0
        }
    }
    /// Merely different.
    fn ane(f: f64) -> f64 {
        let c = f + 1.0;
        if c != f {
            c
        } else if f == 0.0 {
            1.0
        } else {
            f * 0.5
        }
    }
    match (op, threshold) {
        // Gt / Gte: above the threshold (Gt strict: Bool true maxes out).
        (0, PropertyValue::Int(i)) => Some(PropertyValue::Int(i.saturating_add(1))),
        (0, PropertyValue::Float(f)) => Some(PropertyValue::Float(above(f))),
        (0, PropertyValue::Bool(false)) => Some(PropertyValue::Bool(true)),
        (4, PropertyValue::Int(i)) => Some(PropertyValue::Int(i.saturating_add(1))),
        (4, PropertyValue::Float(f)) => Some(PropertyValue::Float(above(f))),
        (4, PropertyValue::Bool(false)) => Some(PropertyValue::Bool(true)),
        // Lt / Lte: below (Lt strict: Bool false bottoms out).
        (1, PropertyValue::Int(i)) => Some(PropertyValue::Int(i.saturating_sub(1))),
        (1, PropertyValue::Float(f)) => Some(PropertyValue::Float(below(f))),
        (1, PropertyValue::Bool(true)) => Some(PropertyValue::Bool(false)),
        (3, PropertyValue::Int(i)) => Some(PropertyValue::Int(i.saturating_sub(1))),
        (3, PropertyValue::Float(f)) => Some(PropertyValue::Float(below(f))),
        (3, PropertyValue::Bool(true)) => Some(PropertyValue::Bool(false)),
        // Eq: exact. Ne: step away (Bool flips).
        (2, v) => Some(v),
        (5, PropertyValue::Int(i)) => Some(PropertyValue::Int(i.saturating_add(1))),
        (5, PropertyValue::Float(f)) => Some(PropertyValue::Float(ane(f))),
        (5, PropertyValue::Bool(b)) => Some(PropertyValue::Bool(!b)),
        // Boundary-inclusive ops accept the threshold itself too, but the
        // strict-side value also satisfies them — normalize onto it, except
        // Bool true/false which need exactness.
        (4, PropertyValue::Bool(true)) => Some(PropertyValue::Bool(true)),
        (3, PropertyValue::Bool(false)) => Some(PropertyValue::Bool(false)),
        _ => None,
    }
}

/// Background soup: `nodes` nodes of `labels` labels, ~`edges` binary edges.
fn soup_into(
    w: &mut HypergraphStore,
    rng: &mut ChaCha8Rng,
    nodes: usize,
    edges: usize,
    labels: u32,
) {
    let mut soup = Vec::with_capacity(nodes);
    for _ in 0..nodes {
        let lab = (rng.gen::<u64>() % labels as u64) as u32;
        soup.push(w.add_node(NodeData::new(Label(lab), 0)));
    }
    for _ in 0..edges {
        let a = soup[(rng.gen::<u64>() % soup.len() as u64) as usize];
        let b = soup[(rng.gen::<u64>() % soup.len() as u64) as usize];
        if a != b {
            w.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
        }
    }
}

/// Inject one guaranteed match for `rule` into a world: one node per LHS var
/// (label-constrained, guard-satisfying), then the LHS edges. Soup built
/// beforehand never interferes (seed nodes are fresh). Returns `None` when
/// guard shapes are unsynthesizable.
fn inject_seed_match(w: &mut HypergraphStore, rule: &Rule) -> Option<()> {
    let mut bound: HashMap<&str, NodeId> = HashMap::new();
    for v in &rule.lhs.nodes {
        let lab = rule.lhs.node_labels.get(v).copied().unwrap_or(Label(0));
        bound.insert(v.as_str(), w.add_node(NodeData::new(lab, 0)));
    }
    // Guard-satisfying properties (applied after, so they win).
    for g in &rule.lhs_node_property_guards {
        let op = match g.op {
            hg_core::PropertyOp::Gt => 0,
            hg_core::PropertyOp::Lt => 1,
            hg_core::PropertyOp::Eq => 2,
            hg_core::PropertyOp::Lte => 3,
            hg_core::PropertyOp::Gte => 4,
            hg_core::PropertyOp::Ne => 5,
        };
        let node = *bound.get(g.var.as_str())?;
        let val = satisfying_value(op, g.value)?;
        if let Some(d) = w.get_node_mut(node) {
            d.set_property(g.property, val);
        }
    }
    // LHS edges over the seed nodes.
    for e in &rule.lhs.edges {
        let ends: SmallVec<[NodeId; 8]> = e.iter().map(|v| bound[v.as_str()]).collect();
        w.add_edge(HyperEdge::new(Label(0), ends));
    }
    Some(())
}

/// Trajectory fixture: background soup + guaranteed step-0 match.
fn synthesize_fixture(rule: &Rule) -> Option<HypergraphStore> {
    let mut rng = ChaCha8Rng::seed_from_u64(GR2_FIXTURE_SEED);
    let mut w = HypergraphStore::new();
    soup_into(
        &mut w,
        &mut rng,
        GR2_SOUP_NODES,
        GR2_SOUP_EDGES,
        GR2_SOUP_LABELS,
    );
    inject_seed_match(&mut w, rule)?;
    Some(w)
}

/// Differential-sweep world `idx`: small random soup (labels 0..3, dense
/// enough to exercise overlaps) + injected seed match (engagement assured).
fn sweep_world(idx: usize, rule: &Rule) -> Option<HypergraphStore> {
    let mut rng =
        ChaCha8Rng::seed_from_u64(GR2_SWEEP_SEED ^ (idx as u64).wrapping_mul(0x9E3779B97F4A7C15));
    let mut w = HypergraphStore::new();
    let nodes = 12 + (rng.gen::<u64>() % 17) as usize;
    let edges = nodes + (rng.gen::<u64>() % nodes as u64) as usize;
    soup_into(&mut w, &mut rng, nodes, edges, 4);
    inject_seed_match(&mut w, rule)?;
    Some(w)
}

/// Harness economics: the match-enumeration budget is a *production*
/// throttle, and G-R2 measures semantics — censoring expensive matches would
/// turn real behavior into Unmeasured artifacts (the triangle_subdivide
/// lesson). Unlimited here; FM-R3 bloat-halt bounds the cost instead.
fn gr2_cfg() -> RunConfig {
    let mut cfg = RunConfig::default();
    cfg.match_enum_budget = f64::INFINITY;
    cfg
}

// ---------- behavioral harness ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BehaviorVerdict {
    Identical,
    DivergentAt(usize),
    /// Sweep-world `idx` diverged on a single step (`GR2_STEPS + idx`
    /// encoding keeps one enum; trajectory steps are always < GR2_STEPS).
    SweepDivergentAt(usize),
    Unmeasured,
}

pub struct Gr2Behavior {
    pub verdict: BehaviorVerdict,
    pub traj_steps: usize,
    pub sweep_engaged: usize,
    pub engaged: bool,
}

pub struct Gr2RuleReport {
    pub file: String,
    pub name: String,
    pub conversion: Result<(), Inexpressible>,
    pub behavioral: Option<BehaviorVerdict>,
    pub traj_steps: usize,
    pub sweep_engaged: usize,
    pub engaged: bool,
}

pub struct Gr2Report {
    pub steps: usize,
    pub rules: Vec<Gr2RuleReport>,
}

impl Gr2Report {
    pub fn total(&self) -> usize {
        self.rules.len()
    }
    pub fn expressible(&self) -> usize {
        self.rules.iter().filter(|r| r.conversion.is_ok()).count()
    }
    pub fn identical(&self) -> usize {
        self.rules
            .iter()
            .filter(|r| r.behavioral == Some(BehaviorVerdict::Identical))
            .count()
    }
    pub fn bad(&self) -> Vec<&Gr2RuleReport> {
        self.rules
            .iter()
            .filter(|r| {
                matches!(
                    r.behavioral,
                    Some(BehaviorVerdict::DivergentAt(_))
                        | Some(BehaviorVerdict::SweepDivergentAt(_))
                        | Some(BehaviorVerdict::Unmeasured)
                )
            })
            .collect()
    }
}

fn unlimited(root: NodeId) -> HashMap<NodeId, i64> {
    let mut m = HashMap::new();
    m.insert(root, i64::MAX / 4);
    m
}

/// Compare baseline (direct-compiled) vs roundtrip (spec→encode→decode→compile).
/// Two legs: (1) trajectory up to `steps` on the cloned auto-fixture, with an
/// FM-R3 bloat halt (both arms must halt together); (2) differential sweep —
/// one step on each of `GR2_SWEEP_WORLDS` random worlds, all must match
/// exactly. Compares canonical hash AND applied/productive counts everywhere
/// (a vacuous-shape difference that leaves the graph unchanged still shows in
/// firing counts).
pub fn compare_rule(rule: &Rule, spec: &RuleSpec, steps: usize) -> Gr2Behavior {
    let caps = gr2_caps();
    let cfg = gr2_cfg();
    let baseline = compile_rule(rule).expect("shipped rule must compile");
    // Roundtrip arm.
    let mut rs = HypergraphStore::new();
    let mut seq = 0i64;
    let root = install_rule(&mut rs, spec, &mut seq);
    let (rt_rules, rt_report) = decode_all(&rs, &caps);
    if rt_report.total != 0 || rt_rules.len() != 1 {
        return Gr2Behavior {
            verdict: BehaviorVerdict::Unmeasured,
            traj_steps: 0,
            sweep_engaged: 0,
            engaged: false,
        };
    }
    // Baseline arm (same spec metadata, directly-compiled rule).
    let base_rule = DecodedRule {
        root,
        spec: spec.clone(),
        class_hash: spec.class_hash(),
        compiled: baseline,
    };
    let bad = |verdict| Gr2Behavior {
        verdict,
        traj_steps: 0,
        sweep_engaged: 0,
        engaged: false,
    };
    let Some(world0) = synthesize_fixture(rule) else {
        return bad(BehaviorVerdict::Unmeasured);
    };
    let mut wa = world0.clone();
    let mut wb = world0;
    let mut ra = ChaCha8Rng::seed_from_u64(GR2_TRAJ_SEED);
    let mut rb = ChaCha8Rng::seed_from_u64(GR2_TRAJ_SEED);
    let ba = unlimited(root);
    let bb = unlimited(rt_rules[0].root);
    let mut engaged = false;
    let mut traj_steps = 0usize;
    for step in 0..steps {
        let oa = object_step(
            &mut wa,
            std::slice::from_ref(&base_rule),
            &mut ra,
            &cfg,
            &ba,
        );
        let ob = object_step(&mut wb, &rt_rules, &mut rb, &cfg, &bb);
        if oa.applied > 0 || ob.applied > 0 {
            engaged = true;
        }
        if wa.canonical_hash() != wb.canonical_hash()
            || oa.applied != ob.applied
            || oa.productive_applied != ob.productive_applied
        {
            return Gr2Behavior {
                verdict: BehaviorVerdict::DivergentAt(step),
                traj_steps: step,
                sweep_engaged: 0,
                engaged,
            };
        }
        traj_steps = step + 1;
        if wa.node_count() > GR2_BLOAT_NODES || wa.edge_count() > GR2_BLOAT_EDGES {
            break; // FM-R3 halt: agreed explosion, prefix compared exactly
        }
    }
    if !engaged || traj_steps < GR2_MIN_TRAJ_STEPS {
        return Gr2Behavior {
            verdict: BehaviorVerdict::Unmeasured,
            traj_steps,
            sweep_engaged: 0,
            engaged,
        };
    }
    // Leg 2: differential sweep (single steps, broad geometries).
    let mut sweep_engaged = 0usize;
    for idx in 0..GR2_SWEEP_WORLDS {
        let Some(sw0) = sweep_world(idx, rule) else {
            return Gr2Behavior {
                verdict: BehaviorVerdict::Unmeasured,
                traj_steps,
                sweep_engaged,
                engaged,
            };
        };
        let mut sa = sw0.clone();
        let mut sb = sw0;
        let mut rxa = ChaCha8Rng::seed_from_u64(GR2_TRAJ_SEED);
        let mut rxb = ChaCha8Rng::seed_from_u64(GR2_TRAJ_SEED);
        let oa = object_step(
            &mut sa,
            std::slice::from_ref(&base_rule),
            &mut rxa,
            &cfg,
            &ba,
        );
        let ob = object_step(&mut sb, &rt_rules, &mut rxb, &cfg, &bb);
        if oa.applied > 0 || ob.applied > 0 {
            sweep_engaged += 1;
        }
        if sa.canonical_hash() != sb.canonical_hash()
            || oa.applied != ob.applied
            || oa.productive_applied != ob.productive_applied
        {
            return Gr2Behavior {
                verdict: BehaviorVerdict::SweepDivergentAt(idx),
                traj_steps,
                sweep_engaged,
                engaged,
            };
        }
    }
    if sweep_engaged != GR2_SWEEP_WORLDS {
        return Gr2Behavior {
            verdict: BehaviorVerdict::Unmeasured,
            traj_steps,
            sweep_engaged,
            engaged,
        };
    }
    // Bonus: RNG streams must agree (no jitter in expressible rules, but pin it).
    if ra.gen::<u64>() != rb.gen::<u64>() {
        return Gr2Behavior {
            verdict: BehaviorVerdict::DivergentAt(traj_steps),
            traj_steps,
            sweep_engaged,
            engaged,
        };
    }
    Gr2Behavior {
        verdict: BehaviorVerdict::Identical,
        traj_steps,
        sweep_engaged,
        engaged: true,
    }
}

/// Shipped-rules directory, robust to the caller's working directory
/// (`cargo test` runs in the crate dir, the CLI in the workspace root).
fn rules_dir() -> std::path::PathBuf {
    let cwd = std::path::PathBuf::from("rules");
    if cwd.is_dir() {
        return cwd;
    }
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules")
}

/// Load one shipped rule by file + name (reporting helper).
fn find_rule(file: &str, name: &str) -> Option<Rule> {
    load_rules_from_file(&rules_dir().join(file))
        .ok()?
        .into_iter()
        .find(|r| r.name == name)
}

/// Load one shipped rule by file + name, panicking with context (tests).
pub fn load_shipped_rule(file: &str, name: &str) -> Rule {
    find_rule(file, name).unwrap_or_else(|| panic!("shipped rule {file}::{name} must exist"))
}

/// Full envelope over `rules/*.toml` (sorted). Builtins are embedded copies
/// of shipped files, so the directory is the de-duplicated denominator.
pub fn gr2_envelope(steps: usize) -> Gr2Report {
    let caps = gr2_caps();
    let mut paths: Vec<_> = std::fs::read_dir(rules_dir())
        .expect("rules/ directory must exist (run from workspace root)")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "toml").unwrap_or(false))
        .collect();
    paths.sort();
    let mut rules_out = Vec::new();
    for path in paths {
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let parsed = load_rules_from_file(&path)
            .unwrap_or_else(|e| panic!("shipped rule file {file} must parse: {e:?}"));
        for rule in &parsed {
            if !rule.enabled {
                continue; // no disabled rules ship today; excluded, not counted
            }
            match rule_to_spec(rule, &caps) {
                Err(reason) => rules_out.push(Gr2RuleReport {
                    file: file.clone(),
                    name: rule.name.clone(),
                    conversion: Err(reason),
                    behavioral: None,
                    traj_steps: 0,
                    sweep_engaged: 0,
                    engaged: false,
                }),
                Ok(spec) => {
                    let beh = compare_rule(rule, &spec, steps);
                    rules_out.push(Gr2RuleReport {
                        file: file.clone(),
                        name: rule.name.clone(),
                        conversion: Ok(()),
                        behavioral: Some(beh.verdict),
                        traj_steps: beh.traj_steps,
                        sweep_engaged: beh.sweep_engaged,
                        engaged: beh.engaged,
                    });
                }
            }
        }
    }
    Gr2Report {
        steps,
        rules: rules_out,
    }
}

pub fn print_gr2_report(rep: &Gr2Report) {
    println!("G-R2: {} rules over {} steps", rep.total(), rep.steps);
    let mut reasons: HashMap<String, usize> = HashMap::new();
    for r in &rep.rules {
        match &r.conversion {
            Ok(()) => println!(
                "  [ok] {:>32} :: {:<28} {:?} (traj {} steps, sweep {}/{})",
                r.file,
                r.name,
                r.behavioral.as_ref().unwrap(),
                r.traj_steps,
                r.sweep_engaged,
                GR2_SWEEP_WORLDS
            ),
            Err(reason) => {
                println!(
                    "  [..] {:>32} :: {:<28} inexpressible: {reason}",
                    r.file, r.name
                );
                *reasons.entry(reason.to_string()).or_insert(0) += 1;
            }
        }
    }
    let mut rank: Vec<_> = reasons.into_iter().collect();
    rank.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    println!("  blocking reasons (ranked): {rank:?}");
    // "Solely blocked by" ranking (extension prioritization): rules whose
    // full reason set is exactly one reason — lifting that reason alone
    // would express them.
    {
        let caps = gr2_caps();
        let mut sole: HashMap<String, usize> = HashMap::new();
        let mut multi = 0usize;
        for r in &rep.rules {
            if r.conversion.is_err() {
                // Re-derive from the shipped rule (names file+rule).
                if let Some(rule) = find_rule(&r.file, &r.name) {
                    let all = inexpressibility_reasons(&rule, &caps);
                    if all.len() == 1 {
                        *sole.entry(all[0].to_string()).or_insert(0) += 1;
                    } else {
                        multi += 1;
                    }
                }
            }
        }
        let mut srank: Vec<_> = sole.into_iter().collect();
        srank.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        println!("  solely-blocked-by (ranked): {srank:?}; multi-blocked: {multi}");
    }
    let non_creation: Vec<_> = rep
        .rules
        .iter()
        .filter(|r| !matches!(r.conversion, Err(Inexpressible::Schedule)))
        .collect();
    let non_creation_expr = non_creation.iter().filter(|r| r.conversion.is_ok()).count();
    println!(
        "  expressible {}/{} ({:.3}); non-creation {}/{} ({:.3}) [SB-R2 companion]",
        rep.expressible(),
        rep.total(),
        rep.expressible() as f64 / rep.total().max(1) as f64,
        non_creation_expr,
        non_creation.len(),
        non_creation_expr as f64 / non_creation.len().max(1) as f64
    );
}

/// Gate: every expressible rule must be behaviorally identical with
/// engagement. Low coverage is a stated limit (PASS with a reported number),
/// divergence is a converter bug (FAIL).
pub fn gate_gr2() -> bool {
    let rep = gr2_envelope(GR2_STEPS);
    print_gr2_report(&rep);
    let bad = rep.bad();
    if bad.is_empty() {
        println!(
            "G-R2 PASS: all {} expressible rules behaviorally identical ({} inexpressible, reported above)",
            rep.identical(),
            rep.total() - rep.expressible()
        );
        true
    } else {
        for r in bad {
            println!("G-R2 FAIL: {} :: {} → {:?}", r.file, r.name, r.behavioral);
        }
        false
    }
}
