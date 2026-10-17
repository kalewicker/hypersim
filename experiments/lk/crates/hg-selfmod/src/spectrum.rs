//! v11 diagnostic: single recorded edits, isolated opportunities, and cycle
//! replacement without fission. Uses the existing object/economy functions.

use crate::decode::{decode_all, decode_spec, spec_to_compiled, DecodedRule};
use crate::economy::{
    distribute_income, refill_creation_budgets, rent_and_starve, rule_tokens, total_bound, Ledger,
};
use crate::encode::install_rule;
use crate::gates::{g2_specs, v6_base_cfg};
use crate::meta::{apply_recorded_operator, recorded_mutation_draw, OPERATOR_NAMES};
use crate::resolve::TieBreak;
use crate::runner::{
    enum_cost, install_population, is_grower, object_step, seed_world, FateCounts, RunConfig,
};
use crate::schema::{RuleSpec, K_CREATION_BUDGET, K_LAST_CANDIDATES, K_MEMBER};
use crate::store_ext::{duplicate_rule, get_int, rule_roots, set_int};
use hg_core::{HypergraphStore, NodeId};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

pub const PROTOCOL: &str = "selfmod-v11-spectrum-v1";
pub const WORLD_SEEDS: [u64; 8] = [1, 2, 3, 7, 42, 99, 123, 777];
pub const PARAMETER_SEEDS: [u64; 4] = [0x5E110001, 0x5E110003, 0x5E110005, 0x5E110007];
pub const CONTRACT: &str = include_str!("../../../docs/selfmod_v11_spectrum_preregistration.md");

pub fn config(seed: u64, steps: usize) -> RunConfig {
    let mut cfg = v6_base_cfg(seed, "churn3", steps);
    cfg.tie_break = TieBreak::Random;
    cfg.match_cost_coeff = 0.0;
    cfg
}

fn rng_probe(rng: &ChaCha8Rng) -> u64 {
    rng.clone().gen()
}

pub fn parameters(operator: usize, member: usize, seed: u64) -> [u64; 5] {
    let key = seed
        ^ (operator as u64).wrapping_mul(0x9E3779B97F4A7C15)
        ^ (member as u64).wrapping_mul(0xD1B54A32D192ED03);
    recorded_mutation_draw(&mut ChaCha8Rng::seed_from_u64(key)).1
}

fn roots_by_member(rs: &HypergraphStore) -> [NodeId; 3] {
    let roots = rule_roots(rs);
    std::array::from_fn(|member| {
        *roots
            .iter()
            .find(|r| get_int(rs, **r, K_MEMBER) == Some(member as i64))
            .unwrap()
    })
}

fn normalized_clones(specs: &[RuleSpec], cfg: &RunConfig) -> Vec<RuleSpec> {
    specs
        .iter()
        .map(|spec| {
            let mut rs = HypergraphStore::new();
            let root = install_rule(&mut rs, spec, &mut 0);
            let child = duplicate_rule(&mut rs, root);
            let decoded = decode_spec(&rs, child, &cfg.caps()).unwrap();
            assert_eq!(&decoded, spec, "faithful duplication changed programme");
            decoded
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GateObservation {
    pub member: usize,
    pub creation_blocked: bool,
    pub node_ceiling_blocked: bool,
    pub edge_ceiling_blocked: bool,
    pub enumeration_estimate: Option<f64>,
    pub enumeration_skipped: bool,
}

fn gates(
    world: &HypergraphStore,
    rules: &[DecodedRule],
    budgets: &HashMap<NodeId, i64>,
    cfg: &RunConfig,
    members: &[NodeId],
) -> Vec<GateObservation> {
    let mut labels = vec![0; cfg.n_labels as usize];
    for (_, node) in world.nodes() {
        if let Some(count) = labels.get_mut(node.label.0 as usize) {
            *count += 1;
        }
    }
    rules
        .iter()
        .map(|rule| {
            let grower = is_grower(&rule.spec);
            let creation_blocked = grower && budgets.get(&rule.root).copied().unwrap_or(0) <= 0;
            let node_ceiling_blocked =
                grower && !rule.spec.creates.is_empty() && world.node_count() >= cfg.world_ceiling;
            let edge_ceiling_blocked = grower
                && rule.spec.redges.len() > rule.spec.pedges.len()
                && world.edge_count() >= cfg.edge_ceiling;
            let estimate = if creation_blocked || node_ceiling_blocked || edge_ceiling_blocked {
                None
            } else {
                Some(enum_cost(
                    &rule.spec,
                    &labels,
                    world.node_count(),
                    world.edge_count(),
                ))
            };
            GateObservation {
                member: members.iter().position(|r| *r == rule.root).unwrap(),
                creation_blocked,
                node_ceiling_blocked,
                edge_ceiling_blocked,
                enumeration_estimate: estimate,
                enumeration_skipped: estimate.is_some_and(|v| v > cfg.match_enum_budget),
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LedgerRow {
    pub free: i64,
    pub bound: i64,
    pub cumulative_influx: i64,
    pub cumulative_dissipated: i64,
    pub audit_residual: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepRow {
    pub step: usize,
    pub fates: [FateCounts; 3],
    pub live: [bool; 3],
    pub capital: [Option<i64>; 3],
    pub income: [i64; 3],
    pub costs: [i64; 3],
    pub creation_remaining: [Option<i64>; 3],
    pub creation_spend: [i64; 3],
    pub world_label_counts_before: Vec<usize>,
    pub gates: Vec<GateObservation>,
    pub decode_failures: usize,
    pub world_nodes: usize,
    pub world_edges: usize,
    pub world_hash: u64,
    pub rng_probe: u64,
    pub ledger: LedgerRow,
}

#[derive(Clone)]
struct Reference {
    cut: usize,
    world: HypergraphStore,
    rng: ChaCha8Rng,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CycleReceipt {
    pub requested_steps: usize,
    pub completed_steps: usize,
    pub initial_world: Value,
    pub initial_rules: Value,
    pub initial_capital: [i64; 3],
    pub initial_free: i64,
    pub initial_influx: i64,
    pub rows: Vec<StepRow>,
    pub error: Option<String>,
}

fn population(cfg: &RunConfig, specs: &[RuleSpec]) -> (HypergraphStore, [NodeId; 3]) {
    let mut rs = HypergraphStore::new();
    install_population(&mut rs, specs, cfg, &mut 0);
    let roots = roots_by_member(&rs);
    (rs, roots)
}

fn cycle(
    cfg: &RunConfig,
    mut rs: HypergraphStore,
    members: [NodeId; 3],
    reference_cuts: &[usize],
) -> (CycleReceipt, Vec<Reference>) {
    let mut rng = ChaCha8Rng::seed_from_u64(cfg.seed);
    let mut world = HypergraphStore::new();
    seed_world(
        &mut world,
        &mut rng,
        cfg.world_nodes,
        cfg.world_edges,
        cfg.n_labels,
    );
    let pr = cfg.economy_params();
    let mut led = Ledger::new(cfg.free_cap.min(cfg.total).max(0), total_bound(&rs));
    let mut receipt = CycleReceipt {
        requested_steps: cfg.steps,
        completed_steps: 0,
        initial_world: serde_json::to_value(&world).unwrap(),
        initial_rules: serde_json::to_value(&rs).unwrap(),
        initial_capital: members.map(|r| rule_tokens(&rs, r)),
        initial_free: led.free,
        initial_influx: led.cumulative_influx,
        rows: vec![],
        error: None,
    };
    let mut references = vec![];
    for step in 0..cfg.steps {
        if reference_cuts.contains(&step) {
            references.push(Reference {
                cut: step,
                world: world.clone(),
                rng: rng.clone(),
            });
        }
        led.influx(&pr);
        let mut costs = HashMap::new();
        rent_and_starve(&mut rs, &mut led, &pr, &mut costs);
        let (rules, decode_failures) = decode_all(&rs, &cfg.caps());
        let budgets: HashMap<_, _> = rule_roots(&rs)
            .into_iter()
            .map(|r| (r, get_int(&rs, r, K_CREATION_BUDGET).unwrap_or(0)))
            .collect();
        let mut world_label_counts_before = vec![0; cfg.n_labels as usize];
        for (_, node) in world.nodes() {
            if let Some(count) = world_label_counts_before.get_mut(node.label.0 as usize) {
                *count += 1;
            }
        }
        let observed = gates(&world, &rules, &budgets, cfg, &members);
        // Retain the completed prefix if an existing native path panics.
        let applied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            object_step(&mut world, &rules, &mut rng, cfg, &budgets, &[])
        }));
        let oo = match applied {
            Ok(value) => value,
            Err(_) => {
                receipt.error = Some(format!(
                    "object phase panic before completed step {}",
                    step + 1
                ));
                break;
            }
        };
        assert_eq!(
            oo.budget_skips,
            observed.iter().filter(|g| g.enumeration_skipped).count()
        );
        for root in rule_roots(&rs) {
            set_int(
                &mut rs,
                root,
                K_LAST_CANDIDATES,
                oo.candidates_per_rule.get(&root).copied().unwrap_or(0),
            );
        }
        for (root, spend) in &oo.creation_spend {
            let balance = get_int(&rs, *root, K_CREATION_BUDGET).unwrap_or(0);
            set_int(&mut rs, *root, K_CREATION_BUDGET, balance - spend);
        }
        let mut income = HashMap::new();
        distribute_income(&mut rs, &oo.productive_per_rule, &mut led, &pr, &mut income);
        // Deliberately omit funded_fission: no rule births, capital splits or
        // mutation draw occur in this diagnostic. Other phase order is retained.
        refill_creation_budgets(&mut rs, &pr);
        let bound = total_bound(&rs);
        let residual = led.audit_residual(bound);
        assert_eq!(residual, 0, "v11 flow audit");
        let live = members.map(|r| rs.get_node(r).is_some());
        let row = StepRow {
            step: step + 1,
            fates: members.map(|r| oo.fate_per_rule.get(&r).copied().unwrap_or_default()),
            live,
            capital: std::array::from_fn(|i| live[i].then(|| rule_tokens(&rs, members[i]))),
            income: members.map(|r| income.get(&r).copied().unwrap_or(0)),
            costs: members.map(|r| costs.get(&r).copied().unwrap_or(0)),
            creation_remaining: members.map(|r| get_int(&rs, r, K_CREATION_BUDGET)),
            creation_spend: members.map(|r| oo.creation_spend.get(&r).copied().unwrap_or(0)),
            world_label_counts_before,
            gates: observed,
            decode_failures,
            world_nodes: world.node_count(),
            world_edges: world.edge_count(),
            world_hash: world.canonical_hash(),
            rng_probe: rng_probe(&rng),
            ledger: LedgerRow {
                free: led.free,
                bound,
                cumulative_influx: led.cumulative_influx,
                cumulative_dissipated: led.cumulative_dissipated,
                audit_residual: residual,
            },
        };
        assert!(row.fates.iter().all(FateCounts::balances));
        receipt.rows.push(row);
        receipt.completed_steps += 1;
        if world.node_count() > cfg.node_explosion_cap {
            receipt.error = Some("hard world explosion stop".into());
            break;
        }
    }
    (receipt, references)
}

fn isolation(
    rs: &HypergraphStore,
    root: NodeId,
    cfg: &RunConfig,
    references: &[Reference],
) -> Vec<Value> {
    references.iter().map(|reference| {
        let mut world = reference.world.clone();
        let mut rng = reference.rng.clone();
        let spec = decode_spec(rs,root,&cfg.caps());
        let before = rng_probe(&rng);
            let mut labels=vec![0;cfg.n_labels as usize];
            for (_,node) in world.nodes() {
                if let Some(count)=labels.get_mut(node.label.0 as usize) {*count+=1;}
            }
            let base = json!({"reference_step":reference.cut,"source_world_hash":world.canonical_hash(),
            "world_nodes_before":world.node_count(),"world_edges_before":world.edge_count(),
            "world_label_counts_before":labels,"before_rng_probe":before});
        let mut result = base.as_object().unwrap().clone();
        let Some(spec) = spec else {
            result.extend(json!({"status":"decode_invalid","completed_steps":0,"fate":null}).as_object().unwrap().clone());
            return Value::Object(result);
        };
        let Some(compiled) = spec_to_compiled(&spec,"isolated") else {
            result.extend(json!({"status":"compile_invalid","completed_steps":0,"fate":null}).as_object().unwrap().clone());
            return Value::Object(result);
        };
        let rule = DecodedRule { root, class_hash:spec.class_hash(),spec,compiled };
        let budgets = HashMap::from([(root,cfg.creation_budget_init)]);
        let observed = gates(&world,std::slice::from_ref(&rule),&budgets,cfg,&[root]);
        let oo = object_step(&mut world,&[rule],&mut rng,cfg,&budgets,&[]);
        assert_eq!(oo.budget_skips,observed.iter().filter(|g|g.enumeration_skipped).count());
        let skipped = oo.budget_skips > 0;
        result.extend(json!({"status":if skipped {"enumeration_unmeasured"} else {"measured"},
            "completed_steps":1,"fate":if skipped {None} else {Some(oo.fate_total)},
            "gates":observed,"after_world_hash":world.canonical_hash(),
            "after_rng_probe":rng_probe(&rng)}).as_object().unwrap().clone());
        Value::Object(result)
    }).collect()
}

pub fn score(
    cycle: &CycleReceipt,
    member: usize,
    executable: bool,
    changed_executable: bool,
) -> Value {
    let rows = &cycle.rows;
    let half = cycle.requested_steps / 2;
    let mut total = FateCounts::default();
    let mut late = FateCounts::default();
    for row in rows {
        total.add(&row.fates[member]);
        if row.step > half {
            late.add(&row.fates[member]);
        }
    }
    let enum_skips = rows
        .iter()
        .flat_map(|r| &r.gates)
        .filter(|g| g.enumeration_skipped)
        .count();
    let resource_gates = rows
        .iter()
        .flat_map(|r| &r.gates)
        .filter(|g| g.creation_blocked || g.node_ceiling_blocked || g.edge_ceiling_blocked)
        .count();
    let window = cycle.requested_steps / 8;
    let tail_active = rows.len() == cycle.requested_steps
        && rows[half..].chunks(window).all(|chunk| {
            chunk
                .iter()
                .map(|r| r.fates.iter().map(|f| f.productive).sum::<i64>())
                .sum::<i64>()
                > 0
        });
    let complete = cycle.error.is_none() && rows.len() == cycle.requested_steps;
    let final_live = rows.last().is_some_and(|r| r.live.iter().all(|x| *x));
    let candidate = changed_executable
        && complete
        && enum_skips == 0
        && final_live
        && late.productive > 0
        && tail_active;
    json!({"executable":executable,"changed_executable":changed_executable,"complete":complete,
        "total_fate":total,"late_fate":late,"full_horizon_denominator":cycle.requested_steps,
        "late_horizon_denominator":cycle.requested_steps-half,
        "productive_rate":if executable&&complete&&enum_skips==0 {Some(total.productive as f64/cycle.requested_steps as f64)} else {None},
        "late_productive_rate":if executable&&complete&&enum_skips==0 {Some(late.productive as f64/(cycle.requested_steps-half) as f64)} else {None},
        "enumeration_skips":enum_skips,"resource_gate_observations":resource_gates,
        "all_members_final_live":final_live,"late_cycle_windows_active":tail_active,
        "final_capital":rows.last().and_then(|r|r.capital[member]),
        "focus_death_step":rows.iter().find(|r|!r.live[member]).map(|r|r.step),
        "bounded_functional_candidate":candidate})
}

fn save(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer(&mut file, value)?;
    file.write_all(b"\n")
}

fn status(out: &Path, value: Value) -> io::Result<()> {
    let tmp = out.join("status.next.json");
    save(&tmp, &value)?;
    fs::rename(tmp, out.join("status.json"))
}

/// The output directory must already contain only the runner's frozen source
/// archive/provenance. An exclusive assay lock refuses repeated invocations.
pub fn run(out: &Path, preflight: bool) -> io::Result<Value> {
    fs::create_dir_all(out)?;
    let _lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("assay.lock"))?;
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_locked(out, preflight)));
    let result = result.unwrap_or_else(|_| {
        Err(io::Error::other(
            "harness panic; retained output is incomplete",
        ))
    });
    if let Err(error) = &result {
        let previous = fs::read(out.join("status.json"))
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
        status(
            out,
            json!({"state":"failed","error":error.to_string(),"last_progress":previous}),
        )?;
    }
    result
}

fn run_locked(out: &Path, preflight: bool) -> io::Result<Value> {
    fs::create_dir(out.join("controls"))?;
    fs::create_dir(out.join("mutations"))?;
    fs::create_dir(out.join("cells"))?;
    let seeds: &[u64] = if preflight { &[7, 42] } else { &WORLD_SEEDS };
    let draws: &[u64] = if preflight {
        &PARAMETER_SEEDS[..1]
    } else {
        &PARAMETER_SEEDS
    };
    let horizon = if preflight { 32 } else { 256 };
    let cuts = [0, horizon / 4, horizon / 2, 3 * horizon / 4];
    save(
        &out.join("manifest.json"),
        &json!({"protocol":PROTOCOL,"engineering_preflight":preflight,
        "world_seeds":seeds,"parameter_seeds":draws,"operators":OPERATOR_NAMES,"members":3,
        "horizon":horizon,"reference_cuts":cuts,"fission_enabled":false,
        "configs":seeds.iter().map(|s|config(*s,horizon)).collect::<Vec<_>>(),
        "mutation_cells_requested":15*3*draws.len()*seeds.len(),"training_launched":false}),
    )?;
    fs::write(out.join("preregistration.md"), CONTRACT)?;
    status(
        out,
        json!({"state":"running","controls_recorded":0,"mutation_cells_recorded":0}),
    )?;
    let specs = g2_specs();
    let mut references = HashMap::new();
    let mut control_rows = vec![];
    let mut reference_engaged = [false; 3];
    let mut advancing_steps = 0;
    for &seed in seeds {
        let cfg = config(seed, horizon);
        let (rs, roots) = population(&cfg, &specs);
        let (parent, refs) = cycle(&cfg, rs, roots, &cuts);
        let clone_specs = normalized_clones(&specs, &cfg);
        let (clone_rs, clone_roots) = population(&cfg, &clone_specs);
        let (clone, _) = cycle(&cfg, clone_rs, clone_roots, &[]);
        assert_eq!(
            parent.rows, clone.rows,
            "parent/clone science-bearing rows diverged"
        );
        assert_eq!(parent.initial_world, clone.initial_world);
        assert_eq!(parent.initial_rules, clone.initial_rules);
        let (rs, roots) = population(&cfg, &specs);
        let (clone_rs, clone_roots) = population(&cfg, &clone_specs);
        let p_iso: Vec<_> = roots
            .iter()
            .map(|r| isolation(&rs, *r, &cfg, &refs))
            .collect();
        let c_iso: Vec<_> = clone_roots
            .iter()
            .map(|r| isolation(&clone_rs, *r, &cfg, &refs))
            .collect();
        assert_eq!(p_iso, c_iso, "parent/clone isolated opportunities diverged");
        for member in 0..3 {
            reference_engaged[member] |= p_iso[member]
                .iter()
                .any(|v| v["fate"]["productive"].as_i64().unwrap_or(0) > 0);
        }
        let isolated_steps: usize = p_iso
            .iter()
            .chain(&c_iso)
            .flatten()
            .map(|v| v["completed_steps"].as_u64().unwrap() as usize)
            .sum();
        advancing_steps += parent.completed_steps + clone.completed_steps + isolated_steps;
        let result = json!({"seed":seed,"parent":parent,"clone":clone,
            "parent_isolated":p_iso,"clone_isolated":c_iso,
            "reference_worlds":refs.iter().map(|r|json!({"cut":r.cut,"world":r.world,
                "rng_probe":rng_probe(&r.rng)})).collect::<Vec<_>>(),
            "parent_scores":(0..3).map(|m|score(&parent,m,true,false)).collect::<Vec<_>>(),
            "clone_equivalent":true});
        save(
            &out.join("controls").join(format!("seed_{seed}.json")),
            &result,
        )?;
        control_rows.push(
            json!({"seed":seed,"complete":parent.error.is_none()&&clone.error.is_none(),
            "clone_equivalent":true}),
        );
        references.insert(seed, refs);
    }
    assert!(
        reference_engaged.iter().all(|x| *x),
        "unengaged parent role"
    );
    assert!(
        control_rows.iter().all(|r| r["complete"] == true),
        "incomplete controls"
    );
    let mut results = vec![];
    for (operator, name) in OPERATOR_NAMES.iter().enumerate() {
        for member in 0..3 {
            for (draw_index, &parameter_seed) in draws.iter().enumerate() {
                let p = parameters(operator, member, parameter_seed);
                let cfg = config(seeds[0], horizon);
                let (mut material, roots) = population(&cfg, &specs);
                let before = serde_json::to_value(&material).unwrap();
                let reported = apply_recorded_operator(
                    &mut material,
                    roots[member],
                    operator,
                    &p,
                    &cfg.caps(),
                )
                .unwrap();
                let after = serde_json::to_value(&material).unwrap();
                let decoded = decode_spec(&material, roots[member], &cfg.caps());
                let compiled = decoded
                    .as_ref()
                    .is_some_and(|s| spec_to_compiled(s, "candidate").is_some());
                let changed = compiled
                    && decoded
                        .as_ref()
                        .is_some_and(|s| s.class_hash() != specs[member].class_hash());
                let id = format!("op{operator:02}_member{member}_draw{draw_index}");
                let mutation = json!({"id":id,"operator":operator,"operator_name":name,"member":member,
                    "parameter_seed":parameter_seed,"parameters":p,"operator_reported_change":reported,
                    "encoded_changed":before!=after,"parent_spec":specs[member],"decoded_spec":decoded,
                    "parent_class_hash":specs[member].class_hash(),
                    "mutant_class_hash":decoded.as_ref().map(RuleSpec::class_hash),
                    "compiled":compiled,"changed_executable":changed,"before_material":before,"after_material":after});
                save(&out.join("mutations").join(format!("{id}.json")), &mutation)?;
                for &seed in seeds {
                    let cfg = config(seed, horizon);
                    let iso = isolation(&material, roots[member], &cfg, &references[&seed]);
                    let iso_steps: usize = iso
                        .iter()
                        .map(|v| v["completed_steps"].as_u64().unwrap() as usize)
                        .sum();
                    let (receipt, _) = cycle(&cfg, material.clone(), roots, &[]);
                    let scored = score(&receipt, member, compiled, changed);
                    advancing_steps += receipt.completed_steps + iso_steps;
                    let row = json!({"id":format!("{id}_seed_{seed}"),"mutation_id":id,"seed":seed,
                        "member":member,"isolated":iso,"cycle":receipt,"score":scored});
                    save(
                        &out.join("cells").join(format!("{id}_seed_{seed}.json")),
                        &row,
                    )?;
                    results.push(json!({"id":row["id"],"mutation_id":id,"seed":seed,"member":member,
                        "score":scored,"completed_steps":receipt.completed_steps,"isolated_steps":iso_steps}));
                    status(
                        out,
                        json!({"state":"running","controls_recorded":seeds.len()*2,
                        "mutation_cells_recorded":results.len(),"actual_advancing_steps":advancing_steps}),
                    )?;
                    if receipt.error.is_some() {
                        return Err(io::Error::other(
                            "incomplete native cycle: raw receipt retained",
                        ));
                    }
                }
            }
        }
        println!(
            "operator {}/15 {}: {} mutation cells recorded",
            operator + 1,
            name,
            results.len()
        );
        io::stdout().flush()?;
    }
    let summary = json!({"protocol":PROTOCOL,"engineering_preflight":preflight,
        "controls":control_rows,"parent_roles_engaged":reference_engaged,
        "mutation_candidates":15*3*draws.len(),"mutation_cells":results.len(),
        "actual_advancing_steps":advancing_steps,"cells":results,
        "training_launched":false,"evolution_probe_launched":false});
    save(&out.join("summary.json"), &summary)?;
    status(
        out,
        json!({"state":"complete","mutation_cells_recorded":results.len(),
        "actual_advancing_steps":advancing_steps,"training_launched":false}),
    )?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::mutate_once;

    #[test]
    fn recorded_draw_matches_production_mutation_and_stream() {
        let cfg = config(7, 32);
        for seed in 0..128 {
            let (mut actual, roots) = population(&cfg, &g2_specs());
            let mut forced = actual.clone();
            let mut a = ChaCha8Rng::seed_from_u64(seed);
            let mut b = a.clone();
            let reported = mutate_once(&mut actual, roots[0], &mut a, &cfg.caps());
            let (operator, params) = recorded_mutation_draw(&mut b);
            assert_eq!(
                reported,
                apply_recorded_operator(&mut forced, roots[0], operator, &params, &cfg.caps())
                    .unwrap()
            );
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(forced).unwrap()
            );
            assert_eq!(a.gen::<u64>(), b.gen::<u64>());
        }
    }

    #[test]
    fn clone_controls_keep_fates_capital_and_rng_exact() {
        let cfg = config(7, 32);
        let specs = g2_specs();
        let (rs, roots) = population(&cfg, &specs);
        let (parent, _) = cycle(&cfg, rs, roots, &[]);
        let (rs, roots) = population(&cfg, &normalized_clones(&specs, &cfg));
        let (clone, _) = cycle(&cfg, rs, roots, &[]);
        assert_eq!(parent.rows, clone.rows);
        assert_eq!(parent.completed_steps, 32);
        assert!(parent
            .rows
            .iter()
            .any(|r| r.fates.iter().any(|f| f.productive > 0)));
    }

    #[test]
    fn enumeration_failure_is_unmeasured_not_sterile() {
        let mut cfg = config(7, 32);
        cfg.match_enum_budget = 0.0;
        let (rs, roots) = population(&cfg, &g2_specs());
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let mut world = HypergraphStore::new();
        seed_world(&mut world, &mut rng, 200, 300, 4);
        let iso = isolation(&rs, roots[0], &cfg, &[Reference { cut: 0, world, rng }]);
        assert_eq!(iso[0]["status"], "enumeration_unmeasured");
        assert!(iso[0]["fate"].is_null());
        let (receipt, _) = cycle(&cfg, rs, roots, &[]);
        let scored = score(&receipt, 0, true, true);
        assert!(scored["productive_rate"].is_null());
        assert_eq!(scored["bounded_functional_candidate"], false);
    }

    #[test]
    fn malformed_rule_pays_rent_without_becoming_a_zero_measurement() {
        let cfg = config(7, 64);
        let (mut rs, roots) = population(&cfg, &g2_specs());
        // Removing the created node leaves dangling references in R1's RHS.
        apply_recorded_operator(&mut rs, roots[0], 9, &[0; 5], &cfg.caps()).unwrap();
        assert!(decode_spec(&rs, roots[0], &cfg.caps()).is_none());
        let (receipt, _) = cycle(&cfg, rs, roots, &[]);
        assert_eq!(receipt.rows[0].decode_failures, 1);
        assert_eq!(receipt.rows[0].capital[0], Some(59));
        assert_eq!(receipt.rows[60].capital[0], None);
        assert_eq!(receipt.rows[60].ledger.audit_residual, 0);
        assert_eq!(score(&receipt, 0, false, false)["focus_death_step"], 61);
        assert!(score(&receipt, 0, false, false)["productive_rate"].is_null());
    }
}
