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
use crate::schema::*;
use crate::store_ext::*;
use hg_core::{
    compute_wl_class_map, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue,
};
use hg_engine::{apply_binding, find_bindings, resolve_conflicts};
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
    pub match_cap_per_rule: usize,
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
            match_enum_budget: 5.0e4,
            wl_radius: 3,
            lhs_max_nodes: 4,
            lhs_max_edges: 4,
            max_arity: 3,
            rhs_max_create: 3,
            n_labels: 4,
            seed_pop: "v2seed".to_string(),
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

            distinct: vec![],

            effects: vec![],
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

            distinct: vec![],

            effects: vec![],
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

            distinct: vec![],

            effects: vec![],
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

            distinct: vec![],

            effects: vec![],
        },
    ]
}

/// The lone grower from the v2 set (rule G) — E1b's coresidence stressor.
pub fn grower_spec() -> RuleSpec {
    seed_rule_specs().remove(0)
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

pub fn install_population(
    rs: &mut HypergraphStore,
    specs: &[RuleSpec],
    cfg: &RunConfig,
    seq: &mut i64,
) {
    for spec in specs {
        let root = install_rule(rs, spec, seq);
        set_tokens(rs, root, cfg.e0);
        set_int(rs, root, K_CREATION_BUDGET, cfg.creation_budget_init);
        set_int(rs, root, K_DEPTH, 0);
    }
}

// ---------- object phase ----------

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
}

/// Estimated enumeration cost of a rule against the current world — a crude
/// upper bound: (edge candidates × endpoint permutations × label selectivity)
/// per pattern edge, × live-node count per isolated var.
fn enum_cost(spec: &RuleSpec, label_counts: &[usize], n_nodes: usize, n_edges: usize) -> f64 {
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
fn is_grower(spec: &RuleSpec) -> bool {
    spec.creates.len() > spec.deletes.len() || spec.redges.len() > spec.pedges.len()
}

/// Run one object phase over the world with the decoded ruleset.
/// `rule_id` = index into `rules` (already canonically ordered by decode_all).
/// `budgets` maps RuleRoot → remaining per-lineage creation allowance.
pub fn object_step(
    world: &mut HypergraphStore,
    rules: &[DecodedRule],
    rng: &mut ChaCha8Rng,
    cfg: &RunConfig,
    budgets: &HashMap<NodeId, i64>,
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
            out.match_candidates += bs.len();
            // Per-rule cap, truncated in canonical order (deterministic).
            bs.sort_by(|a, b| a.canonical_key().cmp(&b.canonical_key()));
            bs.truncate(cfg.match_cap_per_rule);
            candidates.extend(bs);
        }
    }

    let chosen = resolve_conflicts(&candidates);
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
        if productive {
            out.productive_applied += 1;
            out.productive_roots.insert(dr.root);
        } else {
            out.sterile_firings += 1;
        }
    }
    // Engine::step discipline: the step counter advances once per object
    // phase, after applies — created nodes carry the pre-increment step as
    // their generation, exactly as under `Engine::step`.
    world.set_step(world.step() + 1);
    out
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
    /// Quarantine snapshot at the final decode, per validator V1..V9
    /// (G-R4: present on every run including zeros).
    pub quarantine_by_validator: [usize; crate::decode::N_VALIDATORS],
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
    let specs = seed_population(&cfg.seed_pop);
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
mean_lhs_nodes,mean_lhs_edges,decode_failures,qv1,qv2,qv3,qv4,qv5,qv6,qv7,qv8,qv9,match_candidates,data_hash\n",
    );
    let (mut obj_acc, mut prod_acc, mut sterile_acc) = (0usize, 0usize, 0usize);
    let (mut b_acc, mut clonal_acc, mut mutant_acc, mut d_acc) = (0u32, 0u32, 0u32, 0u32);
    let mut cand_acc = 0usize;
    let mut frac_fired_last = 0.0f64;
    let mut decode_failures_last: usize;
    // Per-validator quarantine snapshot at the last decode (G-R4: present
    // on every run including zeros).
    let mut quarantine_last: [usize; crate::decode::N_VALIDATORS] =
        [0; crate::decode::N_VALIDATORS];
    let mut term = String::from("budget");

    let mut tot_clonal = 0u32;
    let mut tot_mutant = 0u32;
    let mut tot_noop = 0u32;
    let mut time_to_first_fission: Option<usize> = None;
    let mut time_to_freeze: Option<usize> = None;
    let mut frozen_run = 0usize; // consecutive steps with productive_applied == 0
    let mut log_pts: Vec<(f64, f64, f64)> = Vec::new(); // (step, M_rules, M_data)

    for step in 0..=cfg.steps {
        // Influx first: fund the pool before anything draws on it.
        if phase == 'B' && step < cfg.steps {
            led.influx(&pr);
        }
        // Rent + starvation (Phase B step order).
        if phase == 'B' && step < cfg.steps {
            d_acc += rent_and_starve(&mut rulestore, &mut led, &pr);
        }

        let (rules, report) = decode_all(&rulestore, &caps);
        decode_failures_last = report.total;
        quarantine_last = report.by_validator;
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
            let _ = writeln!(
                csv,
                "{},{},{},{},{},{},{},{},{},{},{:.3},{},{},{},{},{},{},{},{},{:.2},{:.2},{},{:.2},{:.2},{},{},{},{},{},{},{},{},{},{},{},{}",
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
                quarantine_last[0],
                quarantine_last[1],
                quarantine_last[2],
                quarantine_last[3],
                quarantine_last[4],
                quarantine_last[5],
                quarantine_last[6],
                quarantine_last[7],
                quarantine_last[8],
                cand_acc,
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
        }
        if step == cfg.steps {
            break;
        }

        // Object phase.
        let budgets: HashMap<NodeId, i64> = rule_roots(&rulestore)
            .into_iter()
            .map(|r| (r, get_int(&rulestore, r, K_CREATION_BUDGET).unwrap_or(0)))
            .collect();
        let oo = object_step(&mut world, &rules, &mut rng, cfg, &budgets);
        obj_acc += oo.applied;
        prod_acc += oo.productive_applied;
        sterile_acc += oo.sterile_firings;
        cand_acc += oo.match_candidates;
        let rc_now = rule_roots(&rulestore).len();
        frac_fired_last = if rc_now > 0 {
            oo.fired_roots.len() as f64 / rc_now as f64
        } else {
            0.0
        };

        // Charge per-lineage creation allowances.
        for (root, spend) in &oo.creation_spend {
            if rulestore.get_node(*root).is_some() {
                let b = get_int(&rulestore, *root, K_CREATION_BUDGET).unwrap_or(0);
                set_int(&mut rulestore, *root, K_CREATION_BUDGET, b - spend);
            }
        }

        // Freeze detection: productive_applied == 0 sustained >= 50 steps.
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
                reward_productive(&mut rulestore, &oo.productive_roots, &mut led, &pr);
                let fo = funded_fission(&mut rulestore, &mut rng, &pr, &caps, &mut seq);
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

        // Flow audit — every step, both economies.
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
        // F2: the terminate-at-0 check is suppressed during the grace window,
        // so a startup transient no longer contaminates the survival count.
        if step >= cfg.grace_steps && rule_roots(&rulestore).is_empty() {
            term = "collapse_no_rules".into();
            break;
        }
    }

    // Late-window (final third) OLS slopes over logged points.
    let (mut sr, mut sd) = (0.0, 0.0);
    if log_pts.len() >= 6 {
        let tail = &log_pts[log_pts.len() * 2 / 3..];
        let xs: Vec<f64> = tail.iter().map(|p| p.0).collect();
        sr = ols_slope(&xs, &tail.iter().map(|p| p.1).collect::<Vec<_>>());
        sd = ols_slope(&xs, &tail.iter().map(|p| p.2).collect::<Vec<_>>());
    }

    let final_rules = rule_roots(&rulestore).len();
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
        quarantine_by_validator: quarantine_last,
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
             quarantine_by_validator=[{}]\n",
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
            res.quarantine_by_validator
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ),
    )?;
    Ok(())
}
