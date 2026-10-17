//! Step loop, object phase, metrics, CSV, flow audit.
//!
//! We drive the engine as a library (find_bindings / resolve_conflicts /
//! apply_binding) rather than through `Engine::step`, because the economy
//! needs the fired-rule set, effect-gated productivity, per-rule match caps,
//! per-lineage creation budgets, and a token meta phase — none of which
//! belong inside hg-engine (ground rule 1: engine untouched).
//!
//! # Step order (Phase B)
//! influx → rent/starvation → object phase → reward → fission → budget
//! refill → flow audit. Influx leads so the pool is funded before anything
//! draws on it; the audit closes every step.

use crate::decode::{decode_all, DecodedRule};
use crate::economy::*;
use crate::encode::install_rule;
use crate::resolve::{resolve_conflicts_tiebreak, TieBreak};
use crate::schema::*;
use crate::store_ext::*;
use hg_core::{
    compute_wl_class_map, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue,
};
use hg_engine::{apply_binding, find_bindings};
use hg_observe::Binding;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

/// Pre-registered novelty threshold (v4 §7). With `log_every = 100` over
/// 20000 steps the final third spans ~6667 steps, so the smallest slope
/// distinguishable from zero is one new class across the window,
/// 1/6667 ≈ 1.5e-4 per step. Set to 1e-4 BEFORE any v4 run; never retuned.
pub const NOVELTY_SLOPE_THRESHOLD: f64 = 1.0e-4;

/// How many nodes the candidate-concentration metric reports on (v6 P1).
pub const CONCENTRATION_K: usize = 8;

/// How a rule's candidate list is cut down to `match_cap_per_rule` (v6 P1).
///
/// v3–v5 always truncated in canonical order. That is deterministic but it is
/// **not** a representative sample: canonical order sorts on the bound-node
/// key tuple, so the surviving prefix is concentrated on the lowest-keyed
/// nodes. P1's ordering arm exists to measure how much of the realised
/// dynamics that bias is responsible for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TruncationMode {
    Canonical,
    /// Uniform subsample of the same size, drawn from a stream keyed by
    /// `(run seed, world step, rule index)` so it perturbs no other draw.
    Random,
}

impl std::str::FromStr for TruncationMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "canonical" | "canon" => Ok(TruncationMode::Canonical),
            "random" | "rand" => Ok(TruncationMode::Random),
            _ => Err(()),
        }
    }
}

// ---------- configuration ----------

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct RunConfig {
    pub seed: u64,
    pub steps: usize,
    pub log_every: usize,
    pub economy: EconomyMode,
    pub total: i64,
    pub e0: i64,
    pub birth_cost: i64,
    pub rent: i64,
    pub fire_reward: i64,
    pub repro_threshold: i64,
    /// Flux influx per step (ignored in stock mode).
    pub influx: i64,
    /// Flux free-pool cap (ignored in stock mode).
    pub free_cap: i64,
    /// Variation rate: P(offspring is mutated rather than cloned).
    pub mu: f64,
    pub p_dup: f64,
    pub p_del: f64,
    pub abio: f64,
    pub p_meta: f64,
    pub rule_cap: usize,
    /// Terminate-at-0-rules is suppressed for this many steps (v4 F2):
    /// v3 lost a Phase A seed to a startup transient, contaminating survival.
    pub grace_steps: usize,
    pub world_ceiling: usize,
    /// Global edge backstop (v3 excursion fix). With per-lineage budgets in
    /// place these ceilings are backstops, not the primary regulator.
    pub edge_ceiling: usize,
    /// Per-lineage creation allowance (v4 F3).
    pub creation_budget_init: i64,
    pub creation_budget_max: i64,
    pub creation_refill: i64,
    pub world_nodes: usize,
    pub world_edges: usize,
    pub node_explosion_cap: usize,
    /// Per-rule candidate cap. **0 = unlimited** (v6 P1's top sweep arm).
    /// Every result before v6 was produced at 64; that conditioning is stated
    /// wherever v6 compares across this axis (ground rule 7).
    pub match_cap_per_rule: usize,
    /// v6 P1: how the cap cuts the list. `canonical` reproduces v3–v5.
    pub truncation_mode: TruncationMode,
    /// Upper bound on a single rule's estimated enumeration cost; rules
    /// above it are skipped this step (counted, deterministic).
    pub match_enum_budget: f64,
    pub wl_radius: u8,
    pub lhs_max_nodes: usize,
    pub lhs_max_edges: usize,
    pub max_arity: usize,
    pub rhs_max_create: usize,
    pub n_labels: u32,
    /// Seed population: "churn3" | "churnk" | "churn3_grower" | "v2seed".
    pub seed_pop: String,
    // ---- v5 ----
    /// "flat" (v4, binary fitness) | "proportional" (v5 gradient).
    pub income_mode: IncomeMode,
    /// Per-step income stream shared under proportional income.
    pub income_pool: i64,
    /// Cost per match candidate (0 = v4 behaviour).
    pub match_cost_coeff: f64,
    /// "full" | "split" | "partial" | "refractory".
    pub fission_mode: FissionMode,
    pub split_floor: i64,
    pub partial_cost: i64,
    pub refractory_steps: i64,
    /// Balanced redundancy: copies of EVERY seed member (v5 §3.2).
    pub copy_number: usize,
    /// E-stoich: step at which to force duplication (0 = off).
    pub forced_dup_step: usize,
    /// E-stoich: which members to duplicate — "0", "1", "2", "all", or "".
    pub forced_dup_members: String,
    /// G5.3: step at which to seed the invader (0 = off).
    pub invasion_step: usize,
    // ---- v6 ----
    /// Which variant `invasion_step` seeds: "v5" (added pattern edge) or
    /// "guard" (resident R3 + energy guard, the Pareto candidate).
    pub invader_kind: String,
    /// Guard threshold for `invader_kind = "guard"`: `energy(var0) > theta`.
    pub invader_theta: f64,
    /// G6.2 substitution arm: replace this seed member with the candidate
    /// variant instead of adding it alongside. `-1` = off.
    ///
    /// Adding the variant alongside its own template makes the two compete for
    /// the same conflict-resolution slots, so the resident's measured rate
    /// falls as the variant's rises and "variant ≥ resident" becomes easier to
    /// satisfy the more the variant suppresses the resident. Substitution
    /// measures each in the same ecological role with nothing to suppress.
    pub substitute_member: i64,
    /// Override the invader's priority (`i32::MIN` = leave as built). Used to
    /// test queue position as a *cause* of invasion success.
    pub invader_priority: i32,
    /// v7 §1: how equal-priority bindings are ordered entering conflict
    /// resolution. `ruleid` reproduces v3–v6 exactly (the engine's canonical
    /// order, whose rule_id term is the documented bias); `merit` orders by
    /// cumulative productive firings so coexisting equals share work. See
    /// `crate::resolve`.
    pub tie_break: TieBreak,
    /// v8: noise scale for `TieBreak::Jitter` — relative half-width of the
    /// multiplicative noise band around the merit signal. 0 = exactly
    /// `Merit`; ~8+ approaches `Random` in effect. Ignored by other modes.
    pub jitter_eps: f64,
    /// v9: per-step decay factor applied to the merit table before this
    /// step's productive firings are added: `m ← decay·m + productive`.
    /// 1.0 = cumulative merit (v7/v8 exactly); < 1.0 bounds a rule's
    /// steady-state signal at rate/(1−decay) and forces incumbents to
    /// defend position continuously.
    pub merit_decay: f64,
    /// v9: merit credited to a rule at birth (before its first firing).
    /// With decay < 1 this is the entry mechanism: a newborn at `base`
    /// outranks any incumbent whose slumped signal has fallen below it.
    /// 0.0 = v7/v8 behaviour (newborns start at the bottom).
    pub merit_base: f64,
}

impl Default for RunConfig {
    fn default() -> Self {
        RunConfig {
            seed: 1,
            steps: 20000,
            log_every: 100,
            economy: EconomyMode::Stock,
            total: 2000,
            e0: 60,
            birth_cost: 50,
            rent: 1,
            fire_reward: 2,
            repro_threshold: 100,
            influx: 20,
            free_cap: 500,
            mu: 1.0,
            p_dup: 0.40,
            p_del: 0.40,
            abio: 0.5,
            p_meta: 1.0,
            rule_cap: 400,
            grace_steps: 100,
            world_ceiling: 500,
            edge_ceiling: 1000,
            creation_budget_init: 200,
            creation_budget_max: 200,
            creation_refill: 1,
            world_nodes: 200,
            world_edges: 300,
            node_explosion_cap: 20000,
            match_cap_per_rule: 64,
            truncation_mode: TruncationMode::Canonical,
            match_enum_budget: 5.0e4,
            wl_radius: 3,
            lhs_max_nodes: 4,
            lhs_max_edges: 4,
            max_arity: 3,
            rhs_max_create: 3,
            n_labels: 4,
            seed_pop: "v2seed".to_string(),
            income_mode: IncomeMode::Flat,
            income_pool: 20,
            match_cost_coeff: 0.0,
            fission_mode: FissionMode::Full,
            split_floor: 25,
            partial_cost: 20,
            refractory_steps: 200,
            copy_number: 1,
            forced_dup_step: 0,
            forced_dup_members: String::new(),
            invasion_step: 0,
            invader_kind: "v5".to_string(),
            invader_theta: 0.5,
            substitute_member: -1,
            invader_priority: i32::MIN,
            tie_break: TieBreak::RuleId,
            jitter_eps: 0.0,
            merit_decay: 1.0,
            merit_base: 0.0,
        }
    }
}

impl RunConfig {
    pub fn caps(&self) -> Caps {
        Caps {
            lhs_max_nodes: self.lhs_max_nodes,
            lhs_max_edges: self.lhs_max_edges,
            max_arity: self.max_arity,
            rhs_max_create: self.rhs_max_create,
            n_labels: self.n_labels,
        }
    }

    pub fn economy_params(&self) -> EconomyParams {
        EconomyParams {
            mode: self.economy,
            total: self.total,
            e0: self.e0,
            birth_cost: self.birth_cost,
            rent: self.rent,
            fire_reward: self.fire_reward,
            repro_threshold: self.repro_threshold,
            influx: self.influx,
            free_cap: self.free_cap,
            p_dup: self.p_dup,
            p_del: self.p_del,
            abio: self.abio,
            rule_cap: self.rule_cap,
            mu: self.mu,
            creation_refill: self.creation_refill,
            creation_budget_max: self.creation_budget_max,
            income_mode: self.income_mode,
            income_pool: self.income_pool,
            match_cost_coeff: self.match_cost_coeff,
            fission_mode: self.fission_mode,
            split_floor: self.split_floor,
            partial_cost: self.partial_cost,
            refractory_steps: self.refractory_steps,
        }
    }

    /// Members named by `forced_dup_members` ("all" = every seed member).
    pub fn forced_dup_member_ids(&self) -> Vec<i64> {
        let n = seed_population(&self.seed_pop).len() as i64;
        match self.forced_dup_members.as_str() {
            "" => vec![],
            "all" => (0..n).collect(),
            s => s.split(',').filter_map(|x| x.trim().parse().ok()).collect(),
        }
    }
}

// ---------- world & seed rules ----------

pub fn seed_world(
    s: &mut HypergraphStore,
    rng: &mut ChaCha8Rng,
    n_nodes: usize,
    n_edges: usize,
    n_labels: u32,
) {
    let mut ids = Vec::with_capacity(n_nodes);
    for _ in 0..n_nodes {
        let lab = (rng.gen::<u64>() % n_labels as u64) as u32;
        let e: f64 = rng.gen();
        let mut d = NodeData::new(Label(lab), 0);
        d.set_property(K_ENERGY, PropertyValue::Float(e));
        ids.push(s.add_node(d));
    }
    let mut placed = 0usize;
    let mut tries = 0usize;
    let mut seen: HashSet<(u64, u64)> = HashSet::new();
    while placed < n_edges && tries < n_edges * 10 {
        tries += 1;
        let a = ids[(rng.gen::<u64>() % ids.len() as u64) as usize];
        let b = ids[(rng.gen::<u64>() % ids.len() as u64) as usize];
        if a == b {
            continue;
        }
        let (ka, kb) = (key_u64(a), key_u64(b));
        let key = (ka.min(kb), ka.max(kb));
        if !seen.insert(key) {
            continue;
        }
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
        placed += 1;
    }
}

/// The v2 seed rules (G, R, P, D), translated into the v3 grammar. Engine
/// semantics: matched LHS edges are always consumed, so "keep the edge"
/// means re-emitting it on the RHS.
pub fn seed_rule_specs() -> Vec<RuleSpec> {
    vec![
        // G: label0 with energy > 0.7 spawns a label1 neighbor.
        RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![Some(0)],
            pedges: vec![],
            guards: vec![GuardSpec {
                var: 0,
                key: K_ENERGY,
                op: 0,
                val: PropertyValue::Float(0.7),
            }],
            deletes: vec![],
            relabels: vec![],
            creates: vec![1],
            redges: vec![vec![0, 1]],
        },
        // R: label1 -> label2.
        RuleSpec {
            priority: 0,
            n_vars: 1,
            var_labels: vec![Some(1)],
            pedges: vec![],
            guards: vec![],
            deletes: vec![],
            relabels: vec![(0, 2)],
            creates: vec![],
            redges: vec![],
        },
        // P: label2--label2 edge pruned; one endpoint recycled to label0.
        RuleSpec {
            priority: 1,
            n_vars: 2,
            var_labels: vec![Some(2), Some(2)],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![],
            relabels: vec![(0, 0)],
            creates: vec![],
            redges: vec![],
        },
        // D: label3--label3 edge deletes one endpoint (population sink).
        RuleSpec {
            priority: 1,
            n_vars: 2,
            var_labels: vec![Some(3), Some(3)],
            pedges: vec![vec![0, 1]],
            guards: vec![],
            deletes: vec![1],
            relabels: vec![],
            creates: vec![],
            redges: vec![],
        },
    ]
}

/// The lone grower from the v2 set (rule G) — E1b's coresidence stressor.
pub fn grower_spec() -> RuleSpec {
    seed_rule_specs().remove(0)
}

/// Seed population for a run, honouring G6.2's substitution arm.
pub fn seed_population_for(cfg: &RunConfig) -> Vec<RuleSpec> {
    let mut v = seed_population(&cfg.seed_pop);
    if cfg.substitute_member >= 0 {
        let i = cfg.substitute_member as usize;
        if i < v.len() {
            v[i] = crate::gates::invader_spec_for(cfg);
        }
    }
    v
}

/// Resolve a seed-population name to its rule specs.
pub fn seed_population(name: &str) -> Vec<RuleSpec> {
    match name {
        "churn3" => crate::gates::g2_specs(),
        "churnk" | "churn1" => crate::gates::churn_k_specs(),
        "churn3_grower" => {
            let mut v = crate::gates::g2_specs();
            v.push(grower_spec());
            v
        }
        _ => seed_rule_specs(),
    }
}

/// Install the seed population with **balanced** redundancy: `copy_number`
/// copies of EVERY member, so seed-time member ratios are preserved (v5
/// §3.2 — unbalanced redundancy is predicted harmful, balanced protective).
/// Each rule is tagged with the member index it descends from, which is what
/// makes copy number, stoichiometric deviation and lineage share measurable.
pub fn install_population(
    rs: &mut HypergraphStore,
    specs: &[RuleSpec],
    cfg: &RunConfig,
    seq: &mut i64,
) {
    for _ in 0..cfg.copy_number.max(1) {
        for (m, spec) in specs.iter().enumerate() {
            let root = install_rule(rs, spec, seq);
            set_tokens(rs, root, cfg.e0);
            set_int(rs, root, K_CREATION_BUDGET, cfg.creation_budget_init);
            set_int(rs, root, K_DEPTH, 0);
            set_int(rs, root, K_MEMBER, m as i64);
            set_int(rs, root, K_LAST_FISSION, i64::MIN / 2);
            set_int(rs, root, K_LAST_CANDIDATES, 0);
        }
    }
}

/// Live copies per seed member, and the maximum deviation of the realised
/// member proportions from their seed proportions.
pub fn member_census(rs: &HypergraphStore, n_members: usize) -> (Vec<usize>, f64) {
    let mut counts = vec![0usize; n_members.max(1)];
    let mut other = 0usize;
    for root in rule_roots(rs) {
        let m = get_int(rs, root, K_MEMBER).unwrap_or(-1);
        if m >= 0 && (m as usize) < counts.len() {
            counts[m as usize] += 1;
        } else {
            other += 1;
        }
    }
    let total: usize = counts.iter().sum::<usize>() + other;
    let seed_ratio = 1.0 / n_members.max(1) as f64;
    let dev = if total == 0 {
        0.0
    } else {
        counts
            .iter()
            .map(|&c| (c as f64 / total as f64 - seed_ratio).abs())
            .fold(0.0, f64::max)
    };
    (counts, dev)
}

/// Share of the live population held by each member lineage, plus the
/// invader's share (members at or above `INVADER_MEMBER_BASE`).
pub fn lineage_shares(rs: &HypergraphStore) -> (f64, f64) {
    let roots = rule_roots(rs);
    if roots.is_empty() {
        return (0.0, 0.0);
    }
    let mut by: HashMap<i64, usize> = HashMap::new();
    for r in &roots {
        *by.entry(get_int(rs, *r, K_MEMBER).unwrap_or(-1))
            .or_insert(0) += 1;
    }
    let n = roots.len() as f64;
    let top = by.values().copied().max().unwrap_or(0) as f64 / n;
    let invader = by
        .iter()
        .filter(|(m, _)| **m >= INVADER_MEMBER_BASE)
        .map(|(_, c)| *c)
        .sum::<usize>() as f64
        / n;
    (top, invader)
}

// ---------- object phase ----------

/// The five-way fate of one rule's match candidates in one step (v6 C2).
///
/// `generated == truncated + rejected + sterile + productive`, exactly, every
/// rule every step — that identity is G6.1, and it is asserted rather than
/// assumed because every efficiency number in v6 is a ratio of these.
///
/// v5 reported only `productive / generated`, which conflates four distinct
/// ways of not being productive. Which one dominates decides whether an
/// efficiency-improving variant is even constructible (P0b).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FateCounts {
    /// Bindings `find_bindings` returned.
    pub generated: i64,
    /// Discarded by the per-rule cap before conflict resolution ever saw them.
    pub truncated: i64,
    /// Submitted to conflict resolution and lost the node-disjointness draw.
    pub rejected: i64,
    /// Applied, but left the store content-identical (v4's effect gate).
    pub sterile: i64,
    /// Applied and changed the store.
    pub productive: i64,
}

impl FateCounts {
    /// G6.1's identity, checked per rule per step.
    pub fn balances(&self) -> bool {
        self.generated == self.truncated + self.rejected + self.sterile + self.productive
    }

    pub fn add(&mut self, o: &FateCounts) {
        self.generated += o.generated;
        self.truncated += o.truncated;
        self.rejected += o.rejected;
        self.sterile += o.sterile;
        self.productive += o.productive;
    }

    /// The five buckets as fractions of `generated` (0 if nothing generated).
    pub fn fractions(&self) -> [f64; 4] {
        let g = self.generated as f64;
        if g <= 0.0 {
            return [0.0; 4];
        }
        [
            self.truncated as f64 / g,
            self.rejected as f64 / g,
            self.sterile as f64 / g,
            self.productive as f64 / g,
        ]
    }
}

pub struct ObjectOutcome {
    pub applied: usize,
    pub productive_applied: usize,
    pub sterile_firings: usize,
    pub fired_roots: HashSet<NodeId>,
    pub productive_roots: HashSet<NodeId>,
    pub match_candidates: usize,
    pub budget_skips: usize,
    /// Net creation charged to each rule's per-lineage allowance.
    pub creation_spend: HashMap<NodeId, i64>,
    /// Productive applications per rule this step — the numerator of the
    /// v5 proportional-income share.
    pub productive_per_rule: HashMap<NodeId, u32>,
    /// Match candidates generated per rule this step — the basis of the
    /// v5 match-proportional cost (charged next step).
    pub candidates_per_rule: HashMap<NodeId, i64>,
    // ---- v6 ----
    /// Full candidate-fate breakdown per rule (C2 / G6.1).
    pub fate_per_rule: HashMap<NodeId, FateCounts>,
    /// Run-wide totals of the same five buckets.
    pub fate_total: FateCounts,
    /// Node-disjoint bindings conflict resolution accepted this step.
    pub accepted_bindings: usize,
    /// Distinct world nodes the accepted set touches, ÷ world node count.
    /// v5 measured ~5 accepted bindings on a ≤500-node world without ever
    /// asking how much of the world that actually covers.
    pub node_support_frac: f64,
    /// Share of all candidate node-incidences falling on the
    /// `CONCENTRATION_K` most-contested nodes — the size of the hot region.
    pub candidate_concentration_topk: f64,
}

/// Estimated enumeration cost of a rule against the current world — a crude
/// upper bound: (edge candidates × endpoint permutations × label selectivity)
/// per pattern edge, × live-node count per isolated var.
pub(crate) fn enum_cost(
    spec: &RuleSpec,
    label_counts: &[usize],
    n_nodes: usize,
    n_edges: usize,
) -> f64 {
    let mut cost = 1.0f64;
    let mut in_edge = vec![false; spec.n_vars as usize];
    for pe in &spec.pedges {
        let perms: f64 = (1..=pe.len()).product::<usize>() as f64;
        let mut sel = 1.0f64;
        for &r in pe {
            if let Some(Some(l)) = spec.var_labels.get(r as usize) {
                let n = label_counts.get(*l as usize).copied().unwrap_or(0);
                sel *= (n as f64 / n_nodes.max(1) as f64).min(1.0);
            }
        }
        cost *= ((n_edges.max(1)) as f64 * perms * sel).max(1.0);
        for &r in pe {
            if (r as usize) < in_edge.len() {
                in_edge[r as usize] = true;
            }
        }
    }
    for (i, covered) in in_edge.iter().enumerate() {
        if !covered {
            let n = match spec.var_labels[i] {
                Some(l) => label_counts.get(l as usize).copied().unwrap_or(0),
                None => n_nodes,
            };
            cost *= n.max(1) as f64;
        }
    }
    cost
}

/// Static net growth of a rule: does one application add nodes/edges?
pub(crate) fn is_grower(spec: &RuleSpec) -> bool {
    spec.creates.len() > spec.deletes.len() || spec.redges.len() > spec.pedges.len()
}

/// Run one object phase over the world with the decoded ruleset.
/// `rule_id` = index into `rules` (already canonically ordered by decode_all).
/// `budgets` maps RuleRoot → remaining per-lineage creation allowance.
/// `merit_by_rid` is cumulative productive firings per decode index, used
/// only when `cfg.tie_break` is `Merit` (v7 §1).
pub fn object_step(
    world: &mut HypergraphStore,
    rules: &[DecodedRule],
    rng: &mut ChaCha8Rng,
    cfg: &RunConfig,
    budgets: &HashMap<NodeId, i64>,
    merit_by_rid: &[f64],
) -> ObjectOutcome {
    let mut out = ObjectOutcome {
        applied: 0,
        productive_applied: 0,
        sterile_firings: 0,
        fired_roots: HashSet::new(),
        productive_roots: HashSet::new(),
        match_candidates: 0,
        budget_skips: 0,
        creation_spend: HashMap::new(),
        productive_per_rule: HashMap::new(),
        candidates_per_rule: HashMap::new(),
        fate_per_rule: HashMap::new(),
        fate_total: FateCounts::default(),
        accepted_bindings: 0,
        node_support_frac: 0.0,
        candidate_concentration_topk: 0.0,
    };

    let n_nodes = world.node_count();
    let n_edges = world.edge_count();
    // Global ceilings: backstops only (v4 F3). Per-lineage budgets are the
    // primary regulator, so one grower can no longer externalize its cost
    // onto coresident churners (v3 E1b's failure mode).
    let at_node_ceiling = n_nodes >= cfg.world_ceiling;
    let at_edge_ceiling = n_edges >= cfg.edge_ceiling;
    let mut label_counts = vec![0usize; cfg.n_labels as usize];
    for (_, d) in world.nodes() {
        if (d.label.0 as usize) < label_counts.len() {
            label_counts[d.label.0 as usize] += 1;
        }
    }

    let mut candidates: Vec<Binding> = Vec::new();
    // rid -> bindings that survived truncation and were submitted to conflict
    // resolution. `rejected` is derived from this against the accepted set, so
    // the two sides of G6.1's identity come from independent code paths.
    let mut submitted_per_rid: Vec<i64> = vec![0; rules.len()];
    // Node incidence over the FULL generated pool (pre-truncation), so the
    // concentration metric describes the true match space rather than the
    // aliased prefix the cap admits.
    let mut node_incidence: HashMap<NodeId, usize> = HashMap::new();
    let cap = if cfg.match_cap_per_rule == 0 {
        usize::MAX
    } else {
        cfg.match_cap_per_rule
    };
    let step_key = world.step() as u64;
    {
        let snapshot = world.snapshot();
        for (rid, dr) in rules.iter().enumerate() {
            let grower = is_grower(&dr.spec);
            if grower {
                // Per-lineage allowance: a rule with an exhausted budget
                // cannot grow, but every other rule is untouched.
                if budgets.get(&dr.root).copied().unwrap_or(0) <= 0 {
                    continue;
                }
                // Global backstops.
                if at_node_ceiling && !dr.spec.creates.is_empty() {
                    continue;
                }
                if at_edge_ceiling && dr.spec.redges.len() > dr.spec.pedges.len() {
                    continue;
                }
            }
            if enum_cost(&dr.spec, &label_counts, n_nodes, n_edges) > cfg.match_enum_budget {
                out.budget_skips += 1;
                continue;
            }
            let mut bs = find_bindings(&snapshot, &dr.compiled, rid);
            let generated = bs.len();
            out.match_candidates += generated;
            *out.candidates_per_rule.entry(dr.root).or_insert(0) += generated as i64;
            for b in &bs {
                for &n in b.bound_nodes.iter() {
                    *node_incidence.entry(n).or_insert(0) += 1;
                }
            }
            // Canonical sort first in BOTH modes: it is the deterministic
            // reference order, and the random arm re-sorts back onto it after
            // sampling so only the *membership* of the surviving set differs.
            bs.sort_by(|a, b| a.canonical_key().cmp(&b.canonical_key()));
            if bs.len() > cap {
                match cfg.truncation_mode {
                    TruncationMode::Canonical => bs.truncate(cap),
                    TruncationMode::Random => subsample(&mut bs, cap, cfg.seed, step_key, rid),
                }
            }
            submitted_per_rid[rid] = bs.len() as i64;
            let f = out.fate_per_rule.entry(dr.root).or_default();
            f.generated += generated as i64;
            f.truncated += generated as i64 - bs.len() as i64;
            candidates.extend(bs);
        }
    }

    // Concentration of the true match space: how much of all candidate node
    // incidence lands on the few most-contested nodes.
    {
        let total: usize = node_incidence.values().sum();
        if total > 0 {
            let mut inc: Vec<usize> = node_incidence.values().copied().collect();
            inc.sort_unstable_by(|a, b| b.cmp(a));
            let top: usize = inc.iter().take(CONCENTRATION_K).sum();
            out.candidate_concentration_topk = top as f64 / total as f64;
        }
    }

    // v7 §1: the local resolver replaces the engine call. `RuleId` mode is
    // byte-equivalent to `resolve_conflicts` (asserted in resolve.rs tests);
    // `Merit` re-orders equal-priority bindings by cumulative productive
    // firings; `Random` orders them by a per-step keyed draw. The key mixes
    // run seed and world step (`subsample()`'s pattern) so tie-break mode
    // never shifts a draw of the main RNG stream.
    let chosen = resolve_conflicts_tiebreak(
        &candidates,
        cfg.tie_break,
        merit_by_rid,
        cfg.seed ^ step_key.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        cfg.jitter_eps,
    );
    out.accepted_bindings = chosen.len();
    {
        // Rejected-by-conflict is `submitted − accepted`, per rule.
        let mut accepted_per_rid: Vec<i64> = vec![0; rules.len()];
        let mut support: HashSet<NodeId> = HashSet::new();
        for b in &chosen {
            accepted_per_rid[b.rule_id] += 1;
            support.extend(b.bound_nodes.iter().copied());
        }
        out.node_support_frac = support.len() as f64 / n_nodes.max(1) as f64;
        for (rid, dr) in rules.iter().enumerate() {
            if submitted_per_rid[rid] == 0 && accepted_per_rid[rid] == 0 {
                continue;
            }
            let f = out.fate_per_rule.entry(dr.root).or_default();
            f.rejected += submitted_per_rid[rid] - accepted_per_rid[rid];
        }
    }
    for binding in &chosen {
        let dr = &rules[binding.rule_id];
        // Effect-gating (v4 G4.2) needs pre-apply labels of bound nodes —
        // relabels are absent from RewriteEvent (v3 recon gap) — and the
        // pre-apply content of matched edges, so that a delete-and-recreate
        // of the identical edge scores sterile (the store ends
        // content-identical, which is exactly a weed rule).
        let pre_labels: Vec<Option<u32>> = binding
            .bound_nodes
            .iter()
            .map(|&n| world.get_node(n).map(|d| d.label.0))
            .collect();
        let pre_edges: Vec<(Vec<u64>, u32)> = binding
            .matched_edges
            .iter()
            .filter_map(|&e| world.get_edge(e))
            .map(|e| {
                let mut ends: Vec<u64> = e.nodes.iter().map(|&n| key_u64(n)).collect();
                ends.sort_unstable();
                (ends, e.label.0)
            })
            .collect();

        let (event, _fx_applied, _fx_skipped) = apply_binding(world, &dr.compiled, binding, rng);

        // 1. node created or deleted
        let mut productive = !event.nodes_created.is_empty() || !event.nodes_deleted.is_empty();
        // 2. any bound node's label changed
        if !productive {
            for (i, &n) in binding.bound_nodes.iter().enumerate() {
                let now = world.get_node(n).map(|d| d.label.0);
                if now.is_some() && now != pre_labels[i] {
                    productive = true;
                    break;
                }
            }
        }
        // 3. edge set changed in content (not merely churned identically)
        if !productive && (!event.edges_created.is_empty() || !event.edges_deleted.is_empty()) {
            let mut deleted = pre_edges;
            let mut created: Vec<(Vec<u64>, u32)> = event
                .edges_created
                .iter()
                .filter_map(|&e| world.get_edge(e))
                .map(|e| {
                    let mut ends: Vec<u64> = e.nodes.iter().map(|&n| key_u64(n)).collect();
                    ends.sort_unstable();
                    (ends, e.label.0)
                })
                .collect();
            deleted.sort();
            created.sort();
            // Cleanup deletions beyond the matched set also count as change.
            if event.edges_deleted.len() != deleted.len() || deleted != created {
                productive = true;
            }
        }

        // Charge NET growth to the rule's per-lineage allowance. An
        // edge-neutral, node-neutral churner spends nothing and is therefore
        // never throttled; a grower pays for every net addition it makes.
        let net = (event.nodes_created.len() as i64 - event.nodes_deleted.len() as i64)
            + (event.edges_created.len() as i64 - event.edges_deleted.len() as i64);
        if net > 0 {
            *out.creation_spend.entry(dr.root).or_insert(0) += net;
        }

        out.applied += 1;
        out.fired_roots.insert(dr.root);
        let f = out.fate_per_rule.entry(dr.root).or_default();
        if productive {
            out.productive_applied += 1;
            out.productive_roots.insert(dr.root);
            *out.productive_per_rule.entry(dr.root).or_insert(0) += 1;
            f.productive += 1;
        } else {
            out.sterile_firings += 1;
            f.sterile += 1;
        }
    }

    // G6.1 — the fate breakdown must close, per rule, every step. The two
    // sides are counted on independent paths (submitted/accepted from conflict
    // resolution, sterile/productive from the apply loop), so a mismatch is a
    // real bookkeeping fault and not a restatement.
    for (root, f) in &out.fate_per_rule {
        assert!(
            f.balances(),
            "G6.1 violated for rule {root:?}: generated={} != truncated={} + rejected={} \
             + sterile={} + productive={}",
            f.generated,
            f.truncated,
            f.rejected,
            f.sterile,
            f.productive
        );
        out.fate_total.generated += f.generated;
        out.fate_total.truncated += f.truncated;
        out.fate_total.rejected += f.rejected;
        out.fate_total.sterile += f.sterile;
        out.fate_total.productive += f.productive;
    }

    // Engine::step discipline: the step counter advances once per object
    // phase, after applies — created nodes carry the pre-increment step as
    // their generation, exactly as under `Engine::step`.
    world.set_step(world.step() + 1);
    out
}

/// Keep a uniform `cap`-sized subsample of `bs`, then restore canonical order.
///
/// The stream is keyed by `(run seed, world step, rule index)` and created
/// fresh here, so switching truncation mode changes **which** candidates
/// compete without shifting a single draw in the run's main RNG stream. That
/// keeps the ordering-bias arm a clean one-variable contrast against the
/// canonical arm.
fn subsample(bs: &mut Vec<Binding>, cap: usize, seed: u64, step: u64, rid: usize) {
    let mut r = ChaCha8Rng::seed_from_u64(
        seed ^ step.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((rid as u64).wrapping_shl(32)),
    );
    let n = bs.len();
    for i in 0..cap.min(n) {
        let j = i + (r.gen::<u64>() % (n - i) as u64) as usize;
        bs.swap(i, j);
    }
    bs.truncate(cap);
    bs.sort_by(|a, b| a.canonical_key().cmp(&b.canonical_key()));
}

// ---------- run loop ----------

pub struct RunResult {
    pub term: String,
    pub final_rules: usize,
    pub m_rules: usize,
    pub m_data: usize,
    pub csv: String,
    pub time_to_first_fission: Option<usize>,
    pub time_to_freeze: Option<usize>,
    pub slope_m_rules: f64,
    pub slope_m_data: f64,
    pub audit_residual: i64,
    pub clonal_births: u32,
    pub mutant_births: u32,
    pub noop_mutations: u32,
    pub survived: bool,
    // ---- v5 ----
    /// Mean over logged steps of the per-step variance of `net_income`
    /// across productive rules. G5.1 requires this to be exactly 0 under
    /// v4's flat economy — that is the measurement that converts "fitness
    /// is binary" from an argument into a fact.
    pub net_income_var_mean: f64,
    pub net_income_var_max: f64,
    pub income_gini_mean: f64,
    pub invasion_share_final: f64,
    pub invasion_share_seed: f64,
    pub stoich_deviation_final: f64,
    pub copy_number_final: Vec<usize>,
    pub time_to_collapse: Option<usize>,
    // ---- v6 ----
    /// Whole-run candidate fate totals (C2 / G6.1).
    pub fate: FateCounts,
    pub accepted_per_step: f64,
    pub node_support_frac_mean: f64,
    pub candidate_concentration_mean: f64,
    /// Candidate fate summed per seed-member lineage, alongside the number of
    /// (copy × step) observations backing it. Dividing one by the other gives
    /// the **per copy per step** rates G6.2 compares — measured in the live
    /// run loop, so the construction gate and the invasion run read the same
    /// instrument.
    pub fate_per_member: HashMap<i64, FateCounts>,
    pub copy_steps_per_member: HashMap<i64, i64>,
}

/// Per copy per step productive applications and candidates generated, by
/// member lineage — the exact pair of axes G6.2 gates on.
pub fn per_copy_rates(res: &RunResult) -> HashMap<i64, (f64, f64)> {
    let mut out = HashMap::new();
    for (m, f) in &res.fate_per_member {
        let cs = res.copy_steps_per_member.get(m).copied().unwrap_or(0);
        if cs > 0 {
            out.insert(
                *m,
                (
                    f.productive as f64 / cs as f64,
                    f.generated as f64 / cs as f64,
                ),
            );
        }
    }
    out
}

/// Population variance and Gini of a set of values.
fn var_and_gini(v: &[f64]) -> (f64, f64) {
    let n = v.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let mean = v.iter().sum::<f64>() / n as f64;
    let var = v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n as f64;
    let mut s: Vec<f64> = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let sum: f64 = s.iter().sum();
    let gini = if sum.abs() < 1e-12 {
        0.0
    } else {
        let num: f64 = s
            .iter()
            .enumerate()
            .map(|(i, x)| (2.0 * (i as f64 + 1.0) - n as f64 - 1.0) * x)
            .sum();
        num / (n as f64 * sum)
    };
    (var, gini)
}

/// Ordinary least-squares slope of `ys` against `xs` (per step).
fn ols_slope(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        num += (x - mx) * (y - my);
        den += (x - mx) * (x - mx);
    }
    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

pub fn run_phase(phase: char, cfg: &RunConfig) -> RunResult {
    let caps = cfg.caps();
    let pr = cfg.economy_params();
    let mut rng = ChaCha8Rng::seed_from_u64(cfg.seed);
    let mut world = HypergraphStore::new();
    let mut rulestore = HypergraphStore::new();
    let mut seq: i64 = 0;
    seed_world(
        &mut world,
        &mut rng,
        cfg.world_nodes,
        cfg.world_edges,
        cfg.n_labels,
    );
    let specs = seed_population_for(cfg);
    let n_members = specs.len();
    install_population(&mut rulestore, &specs, cfg, &mut seq);

    let initial_bound = total_bound(&rulestore);
    let initial_free = match cfg.economy {
        EconomyMode::Stock => cfg.total - initial_bound,
        EconomyMode::Flux => cfg.free_cap.min(cfg.total).max(0),
    };
    let mut led = Ledger::new(initial_free, initial_bound);

    let mut rule_reg: HashSet<u64> = HashSet::new();
    let mut data_reg: HashSet<u64> = HashSet::new();
    let mut csv = String::from(
        "step,data_nodes,data_edges,rule_count,alive_rule_classes,M_rules,M_data,obj_applied,\
productive_applied,sterile_firings,frac_fired,free,bound,cumulative_influx,cumulative_dissipated,\
births,clonal_births,mutant_births,deaths,mean_tokens,lineage_depth_mean,lineage_depth_max,\
mean_lhs_nodes,mean_lhs_edges,decode_failures,match_candidates,net_income_mean,net_income_var,\
income_gini,match_cost_total,productive_per_candidate,copy_number_min,copy_number_mean,\
copy_number_max,stoich_deviation,lineage_share_top,invasion_share,\
cand_truncated,cand_rejected_conflict,accepted_bindings,node_support_frac,\
cand_concentration_top8,data_hash\n",
    );
    let (mut obj_acc, mut prod_acc, mut sterile_acc) = (0usize, 0usize, 0usize);
    let (mut b_acc, mut clonal_acc, mut mutant_acc, mut d_acc) = (0u32, 0u32, 0u32, 0u32);
    let mut cand_acc = 0usize;
    // v6 fate accounting, accumulated per logging window.
    let mut fate_win = FateCounts::default();
    let mut accepted_win = 0usize;
    let mut support_win = 0.0f64;
    let mut conc_win = 0.0f64;
    let mut win_steps = 0usize;
    // Whole-run totals, for the summary and the P0b/P1 tables.
    let mut fate_run = FateCounts::default();
    let mut accepted_run = 0usize;
    let mut support_run = 0.0f64;
    let mut conc_run = 0.0f64;
    let mut run_steps = 0usize;
    let mut fate_member: HashMap<i64, FateCounts> = HashMap::new();
    let mut copy_steps: HashMap<i64, i64> = HashMap::new();
    // v7 §1 / v9: merit per RuleRoot — the signal behind `TieBreak::Merit`
    // and `TieBreak::Jitter`. RuleRoot-keyed so entries survive re-decoding.
    // v9 semantics: each step, every live rule's entry is first multiplied
    // by `merit_decay`, then this step's productive firings are added;
    // newborns enter at `merit_base`. decay=1.0/base=0.0 is exactly the v8
    // cumulative table (byte-compat asserted by regression run).
    let mut merit_by_root: HashMap<NodeId, f64> = HashMap::new();
    let mut frac_fired_last = 0.0f64;
    let mut decode_failures_last: usize;
    let mut term = String::from("budget");

    let mut tot_clonal = 0u32;
    let mut tot_mutant = 0u32;
    let mut tot_noop = 0u32;
    let mut time_to_first_fission: Option<usize> = None;
    let mut time_to_freeze: Option<usize> = None;
    let mut time_to_collapse: Option<usize> = None;
    let mut frozen_run = 0usize;
    let mut log_pts: Vec<(f64, f64, f64)> = Vec::new();

    // v5 per-step economics, carried to the next log point.
    let mut net_income_mean = 0.0f64;
    let mut net_income_var = 0.0f64;
    let mut income_gini = 0.0f64;
    let mut match_cost_total = 0i64;
    let mut prod_per_cand = 0.0f64;
    let mut var_samples: Vec<f64> = Vec::new();
    let mut gini_samples: Vec<f64> = Vec::new();
    let mut invasion_share_seed = 0.0f64;
    let mut invasion_share_last = 0.0f64;
    let mut stoich_last = 0.0f64;
    let mut copies_last: Vec<usize> = vec![0; n_members.max(1)];

    let forced_members = cfg.forced_dup_member_ids();

    for step in 0..=cfg.steps {
        // E-stoich: forced duplication of chosen members, no mutation, no
        // reproduction economics involved (v5 §5.1).
        if phase == 'B'
            && cfg.forced_dup_step > 0
            && step == cfg.forced_dup_step
            && !forced_members.is_empty()
        {
            forced_duplicate(&mut rulestore, &forced_members, &mut seq, step as i64);
        }
        // G5.3: seed a single instance of the hand-built superior variant.
        if phase == 'B' && cfg.invasion_step > 0 && step == cfg.invasion_step {
            let inv = crate::gates::invader_spec_for(cfg);
            let root = install_rule(&mut rulestore, &inv, &mut seq);
            set_tokens(&mut rulestore, root, cfg.e0);
            set_int(
                &mut rulestore,
                root,
                K_CREATION_BUDGET,
                cfg.creation_budget_init,
            );
            set_int(&mut rulestore, root, K_DEPTH, 0);
            set_int(&mut rulestore, root, K_MEMBER, INVADER_MEMBER_BASE);
            set_int(&mut rulestore, root, K_LAST_FISSION, i64::MIN / 2);
            set_int(&mut rulestore, root, K_LAST_CANDIDATES, 0);
            // The invader arrives funded like a resident, so the test is
            // about its efficiency, not about a starting-capital advantage.
            led.cumulative_influx += cfg.e0;
            let (_, inv_share) = lineage_shares(&rulestore);
            invasion_share_seed = inv_share;
        }

        if phase == 'B' && step < cfg.steps {
            led.influx(&pr);
        }
        let mut costs: HashMap<NodeId, i64> = HashMap::new();
        if phase == 'B' && step < cfg.steps {
            d_acc += rent_and_starve(&mut rulestore, &mut led, &pr, &mut costs);
        }

        let (rules, decode_failures) = decode_all(&rulestore, &caps);
        decode_failures_last = decode_failures;
        // v9: decay every live rule's merit, then credit births at base.
        // Done after decode so "live" means present in this step's ruleset.
        if cfg.merit_decay < 1.0 {
            for dr in &rules {
                merit_by_root
                    .entry(dr.root)
                    .and_modify(|m| *m *= cfg.merit_decay);
            }
        }
        let merit_by_rid: Vec<f64> = rules
            .iter()
            .map(|dr| {
                merit_by_root
                    .get(&dr.root)
                    .copied()
                    .unwrap_or(cfg.merit_base)
            })
            .collect();
        for dr in &rules {
            rule_reg.insert(dr.class_hash);
        }

        if step % cfg.log_every == 0 {
            for h in compute_wl_class_map(&world, cfg.wl_radius).values() {
                data_reg.insert(h.0);
            }
            let roots = rule_roots(&rulestore);
            let rc = roots.len();
            let bound = total_bound(&rulestore);
            let mean_tok = if rc > 0 {
                bound as f64 / rc as f64
            } else {
                0.0
            };
            let alive_classes: HashSet<u64> = rules.iter().map(|d| d.class_hash).collect();
            let depths: Vec<i64> = roots
                .iter()
                .map(|&r| get_int(&rulestore, r, K_DEPTH).unwrap_or(0))
                .collect();
            let dmean = if depths.is_empty() {
                0.0
            } else {
                depths.iter().sum::<i64>() as f64 / depths.len() as f64
            };
            let dmax = depths.iter().copied().max().unwrap_or(0);
            let (mln, mle) = if rules.is_empty() {
                (0.0, 0.0)
            } else {
                (
                    rules.iter().map(|d| d.spec.n_vars as f64).sum::<f64>() / rules.len() as f64,
                    rules
                        .iter()
                        .map(|d| d.spec.pedges.len() as f64)
                        .sum::<f64>()
                        / rules.len() as f64,
                )
            };
            let (counts, dev) = member_census(&rulestore, n_members);
            let (top_share, inv_share) = lineage_shares(&rulestore);
            copies_last = counts.clone();
            stoich_last = dev;
            invasion_share_last = inv_share;
            let cmin = counts.iter().copied().min().unwrap_or(0);
            let cmax = counts.iter().copied().max().unwrap_or(0);
            let cmean = if counts.is_empty() {
                0.0
            } else {
                counts.iter().sum::<usize>() as f64 / counts.len() as f64
            };
            let _ = writeln!(
                csv,
                "{},{},{},{},{},{},{},{},{},{},{:.3},{},{},{},{},{},{},{},{},{:.2},{:.2},{},{:.2},{:.2},{},{},{:.3},{:.5},{:.4},{},{:.4},{},{:.2},{},{:.4},{:.4},{:.4},{},{},{},{:.5},{:.5},{}",
                step,
                world.node_count(),
                world.edge_count(),
                rc,
                alive_classes.len(),
                rule_reg.len(),
                data_reg.len(),
                obj_acc,
                prod_acc,
                sterile_acc,
                frac_fired_last,
                led.free,
                bound,
                led.cumulative_influx,
                led.cumulative_dissipated,
                b_acc,
                clonal_acc,
                mutant_acc,
                d_acc,
                mean_tok,
                dmean,
                dmax,
                mln,
                mle,
                decode_failures_last,
                cand_acc,
                net_income_mean,
                net_income_var,
                income_gini,
                match_cost_total,
                prod_per_cand,
                cmin,
                cmean,
                cmax,
                dev,
                top_share,
                inv_share,
                fate_win.truncated,
                fate_win.rejected,
                accepted_win,
                if win_steps > 0 { support_win / win_steps as f64 } else { 0.0 },
                if win_steps > 0 { conc_win / win_steps as f64 } else { 0.0 },
                world.canonical_hash()
            );
            log_pts.push((step as f64, rule_reg.len() as f64, data_reg.len() as f64));
            obj_acc = 0;
            prod_acc = 0;
            sterile_acc = 0;
            b_acc = 0;
            clonal_acc = 0;
            mutant_acc = 0;
            d_acc = 0;
            cand_acc = 0;
            fate_win = FateCounts::default();
            accepted_win = 0;
            support_win = 0.0;
            conc_win = 0.0;
            win_steps = 0;
        }
        if step == cfg.steps {
            break;
        }

        // Object phase.
        let budgets: HashMap<NodeId, i64> = rule_roots(&rulestore)
            .into_iter()
            .map(|r| (r, get_int(&rulestore, r, K_CREATION_BUDGET).unwrap_or(0)))
            .collect();
        let oo = object_step(&mut world, &rules, &mut rng, cfg, &budgets, &merit_by_rid);
        obj_acc += oo.applied;
        prod_acc += oo.productive_applied;
        sterile_acc += oo.sterile_firings;
        cand_acc += oo.match_candidates;
        // v7 §1 / v9: accumulate merit — decayed, then incremented by this
        // step's productive firings (f64; fractional under decay < 1).
        for (root, n) in &oo.productive_per_rule {
            *merit_by_root.entry(*root).or_insert(cfg.merit_base) += *n as f64;
        }
        fate_win.add(&oo.fate_total);
        fate_run.add(&oo.fate_total);
        accepted_win += oo.accepted_bindings;
        accepted_run += oo.accepted_bindings;
        support_win += oo.node_support_frac;
        support_run += oo.node_support_frac;
        conc_win += oo.candidate_concentration_topk;
        conc_run += oo.candidate_concentration_topk;
        win_steps += 1;
        run_steps += 1;
        // Attribute this step's fate to member lineages. Every LIVE rule
        // contributes a copy-step, whether or not it generated candidates —
        // a copy that sat idle still had its chance and must dilute the rate,
        // otherwise a rule that only ever fires when conditions are perfect
        // scores as if it fired constantly.
        //
        // On an invasion run, accumulation starts at `invasion_step`: G6.2
        // compares the invader's rate against the resident's, and the resident
        // must not be credited with the steps it ran before the invader
        // existed. With no invader (`invasion_step = 0`) this is the whole run.
        if step >= cfg.invasion_step {
            for root in rule_roots(&rulestore) {
                let m = get_int(&rulestore, root, K_MEMBER).unwrap_or(-1);
                *copy_steps.entry(m).or_insert(0) += 1;
                if let Some(f) = oo.fate_per_rule.get(&root) {
                    fate_member.entry(m).or_default().add(f);
                }
            }
        }
        let rc_now = rule_roots(&rulestore).len();
        frac_fired_last = if rc_now > 0 {
            oo.fired_roots.len() as f64 / rc_now as f64
        } else {
            0.0
        };
        prod_per_cand = if oo.match_candidates > 0 {
            oo.productive_applied as f64 / oo.match_candidates as f64
        } else {
            0.0
        };

        // Record this step's candidate counts; next step's cost is charged
        // against them (keeps rent in its v4 position in the step order).
        for root in rule_roots(&rulestore) {
            let c = oo.candidates_per_rule.get(&root).copied().unwrap_or(0);
            set_int(&mut rulestore, root, K_LAST_CANDIDATES, c);
        }

        for (root, spend) in &oo.creation_spend {
            if rulestore.get_node(*root).is_some() {
                let b = get_int(&rulestore, *root, K_CREATION_BUDGET).unwrap_or(0);
                set_int(&mut rulestore, *root, K_CREATION_BUDGET, b - spend);
            }
        }

        if oo.productive_applied == 0 {
            frozen_run += 1;
            if frozen_run >= 50 && time_to_freeze.is_none() {
                time_to_freeze = Some(step + 1 - frozen_run);
            }
        } else {
            frozen_run = 0;
        }

        // Meta phase.
        match phase {
            'A' => {
                if cfg.p_meta >= 1.0 || rng.gen::<f64>() < cfg.p_meta {
                    let (b, d) =
                        meta_step_a(&mut rulestore, &mut rng, &mut led, &pr, &caps, &mut seq);
                    b_acc += b;
                    d_acc += d;
                }
            }
            _ => {
                let mut income: HashMap<NodeId, i64> = HashMap::new();
                distribute_income(
                    &mut rulestore,
                    &oo.productive_per_rule,
                    &mut led,
                    &pr,
                    &mut income,
                );

                // net_income = income − cost, over rules that earned this
                // step. Its VARIANCE is the existence of a fitness gradient:
                // exactly 0 under flat income (G5.1), nonzero under
                // proportional income + match cost.
                let nets: Vec<f64> = income
                    .iter()
                    .map(|(r, inc)| (*inc - costs.get(r).copied().unwrap_or(0)) as f64)
                    .collect();
                net_income_mean = if nets.is_empty() {
                    0.0
                } else {
                    nets.iter().sum::<f64>() / nets.len() as f64
                };
                let (v, g) = var_and_gini(&nets);
                net_income_var = v;
                income_gini = g;
                if !nets.is_empty() {
                    var_samples.push(v);
                    gini_samples.push(g);
                }
                match_cost_total = costs.values().sum::<i64>() - pr.rent * costs.len() as i64;

                let fo =
                    funded_fission(&mut rulestore, &mut rng, &pr, &caps, &mut seq, step as i64);
                let births = fo.clonal + fo.mutant;
                if births > 0 && time_to_first_fission.is_none() {
                    time_to_first_fission = Some(step);
                }
                b_acc += births;
                clonal_acc += fo.clonal;
                mutant_acc += fo.mutant;
                tot_clonal += fo.clonal;
                tot_mutant += fo.mutant;
                tot_noop += fo.noop;
                refill_creation_budgets(&mut rulestore, &pr);
            }
        }

        // Flow audit — every step, every economy.
        let bound = total_bound(&rulestore);
        let residual = led.audit_residual(bound);
        assert_eq!(
            residual, 0,
            "flow audit violated at step {step}: influx={} bound={} free={} dissipated={}",
            led.cumulative_influx, bound, led.free, led.cumulative_dissipated
        );

        if world.node_count() > cfg.node_explosion_cap {
            term = "explosion_nodes".into();
            break;
        }
        if step >= cfg.grace_steps && rule_roots(&rulestore).is_empty() {
            term = "collapse_no_rules".into();
            time_to_collapse = Some(step);
            break;
        }
    }

    let (mut sr, mut sd) = (0.0, 0.0);
    if log_pts.len() >= 6 {
        let tail = &log_pts[log_pts.len() * 2 / 3..];
        let xs: Vec<f64> = tail.iter().map(|p| p.0).collect();
        sr = ols_slope(&xs, &tail.iter().map(|p| p.1).collect::<Vec<_>>());
        sd = ols_slope(&xs, &tail.iter().map(|p| p.2).collect::<Vec<_>>());
    }

    let final_rules = rule_roots(&rulestore).len();
    let mean_of = |v: &Vec<f64>| {
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };
    RunResult {
        survived: term == "budget" && final_rules > 0,
        term,
        final_rules,
        m_rules: rule_reg.len(),
        m_data: data_reg.len(),
        csv,
        time_to_first_fission,
        time_to_freeze,
        slope_m_rules: sr,
        slope_m_data: sd,
        audit_residual: led.audit_residual(total_bound(&rulestore)),
        clonal_births: tot_clonal,
        mutant_births: tot_mutant,
        noop_mutations: tot_noop,
        net_income_var_mean: mean_of(&var_samples),
        net_income_var_max: var_samples.iter().copied().fold(0.0, f64::max),
        income_gini_mean: mean_of(&gini_samples),
        invasion_share_final: invasion_share_last,
        invasion_share_seed,
        stoich_deviation_final: stoich_last,
        copy_number_final: copies_last,
        time_to_collapse,
        fate: fate_run,
        accepted_per_step: if run_steps > 0 {
            accepted_run as f64 / run_steps as f64
        } else {
            0.0
        },
        node_support_frac_mean: if run_steps > 0 {
            support_run / run_steps as f64
        } else {
            0.0
        },
        candidate_concentration_mean: if run_steps > 0 {
            conc_run / run_steps as f64
        } else {
            0.0
        },
        fate_per_member: fate_member,
        copy_steps_per_member: copy_steps,
    }
}

// ---------- run directory hygiene ----------

pub fn write_run(
    dir: &std::path::Path,
    cfg: &RunConfig,
    phase: char,
    res: &RunResult,
) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join("run.csv"), &res.csv)?;
    let cfg_toml = toml::to_string_pretty(cfg).unwrap_or_default();
    std::fs::write(
        dir.join("run_config.toml"),
        format!("phase = \"{phase}\"\n{cfg_toml}"),
    )?;
    std::fs::write(
        dir.join("summary.txt"),
        format!(
            "phase={phase} seed={} economy={:?} mu={} seed_pop={}\n\
             term={} survived={} final_rules={} M_rules={} M_data={}\n\
             time_to_first_fission={}\ntime_to_freeze={}\n\
             slope_M_rules={:.6e}\nslope_M_data={:.6e}\nnovelty_threshold={:.1e}\n\
             births_clonal={} births_mutant={} mutant_noop={}\n\
             flow_audit_residual={}\n\
             time_to_collapse={}\n\
             net_income_var_mean={:.6e} net_income_var_max={:.6e} income_gini_mean={:.4}\n\
             invasion_share_seed={:.4} invasion_share_final={:.4}\n\
             stoich_deviation_final={:.4} copy_number_final={:?}\n\
             match_cap_per_rule={} truncation_mode={:?}\n\
             fate_generated={} fate_truncated={} fate_rejected_conflict={} \
             fate_applied_sterile={} fate_applied_productive={}\n\
             fate_frac_truncated={:.4} fate_frac_rejected={:.4} fate_frac_sterile={:.4} \
             fate_frac_productive={:.6}\n\
             accepted_bindings_per_step={:.3} node_support_frac_mean={:.5} \
             candidate_concentration_top{}_mean={:.5}\n",
            cfg.seed,
            cfg.economy,
            cfg.mu,
            cfg.seed_pop,
            res.term,
            res.survived,
            res.final_rules,
            res.m_rules,
            res.m_data,
            res.time_to_first_fission
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".into()),
            res.time_to_freeze
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".into()),
            res.slope_m_rules,
            res.slope_m_data,
            NOVELTY_SLOPE_THRESHOLD,
            res.clonal_births,
            res.mutant_births,
            res.noop_mutations,
            res.audit_residual,
            res.time_to_collapse
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".into()),
            res.net_income_var_mean,
            res.net_income_var_max,
            res.income_gini_mean,
            res.invasion_share_seed,
            res.invasion_share_final,
            res.stoich_deviation_final,
            res.copy_number_final,
            cfg.match_cap_per_rule,
            cfg.truncation_mode,
            res.fate.generated,
            res.fate.truncated,
            res.fate.rejected,
            res.fate.sterile,
            res.fate.productive,
            res.fate.fractions()[0],
            res.fate.fractions()[1],
            res.fate.fractions()[2],
            res.fate.fractions()[3],
            res.accepted_per_step,
            res.node_support_frac_mean,
            CONCENTRATION_K,
            res.candidate_concentration_mean,
        ),
    )?;
    Ok(())
}
