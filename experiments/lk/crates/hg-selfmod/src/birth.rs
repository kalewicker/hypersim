//! v12: one naturally funded R1 birth, paired treatment and native descendants.
use crate::decode::{decode_all, decode_spec, spec_to_compiled};
use crate::economy::{
    distribute_income, funded_fission_recorded, refill_creation_budgets, rent_and_starve,
    rule_tokens, set_tokens, total_bound, Ledger,
};
use crate::gates::g2_specs;
use crate::meta::{apply_recorded_operator, recorded_mutation_draw};
use crate::runner::{
    enum_cost, install_population, is_grower, object_step, seed_world, FateCounts, RunConfig,
};
use crate::schema::{RuleSpec, K_CREATION_BUDGET, K_LAST_CANDIDATES, K_MEMBER};
use crate::spectrum::LedgerRow;
use crate::store_ext::{delete_rule, get_int, key_u64, rule_roots, set_int};
use hg_core::{HypergraphStore, NodeId};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub const PROTOCOL: &str = "selfmod-v12-birth-v1";
pub const SEEDS: [u64; 8] = [11, 17, 23, 31, 47, 61, 83, 101];
pub const PRUNE_PARAMETERS: [u64; 5] = [
    14984134727853237840,
    12375471953779355968,
    14176329756113589063,
    15783146528319595889,
    2475482604415574477,
];
pub const CONTRACT: &str = include_str!("../../../docs/selfmod_v12_birth_preregistration.md");

#[path = "birth_turnover.rs"]
pub mod turnover;

#[path = "birth_bounded.rs"]
pub mod bounded;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arm {
    Faithful,
    Prune,
    Unscreened,
    Refund,
}
impl Arm {
    pub fn name(self) -> &'static str {
        match self {
            Self::Faithful => "faithful",
            Self::Prune => "prune",
            Self::Unscreened => "unscreened",
            Self::Refund => "refund",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        ARMS.into_iter().find(|a| a.name() == s)
    }
}
pub const ARMS: [Arm; 4] = [Arm::Faithful, Arm::Prune, Arm::Unscreened, Arm::Refund];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gene {
    pub id: u64,
    pub parent: Option<u64>,
    pub member: usize,
    pub lineage: bool,
    pub born: usize,
    pub spec: Option<RuleSpec>,
    pub executable: bool,
    pub class_hash: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    pub id: u64,
    pub capital: i64,
    pub budget: i64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gate {
    pub creation_blocked: bool,
    pub node_ceiling_blocked: bool,
    pub edge_ceiling_blocked: bool,
    pub enumeration_estimate: Option<f64>,
    pub enumeration_skipped: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RootRow {
    pub id: u64,
    pub member: usize,
    pub lineage: bool,
    pub executable: bool,
    pub capital_before: i64,
    pub budget_before: i64,
    pub active: bool,
    pub cost: i64,
    pub income: i64,
    pub spend: i64,
    pub fate: FateCounts,
    pub gate: Option<Gate>,
    pub capital_after_income: Option<i64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BirthEvent {
    pub parent: u64,
    pub child: u64,
    pub parent_before: i64,
    pub parent_after_native: i64,
    pub child_tokens: i64,
    pub inherited_budget: i64,
    pub coin: f64,
    pub operator: usize,
    pub parameters: [u64; 5],
    pub cancelled: bool,
    pub gene: Gene,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub absolute_step: usize,
    pub continuation_step: Option<usize>,
    pub roots: Vec<RootRow>,
    pub births: Vec<BirthEvent>,
    pub stocks: Vec<Stock>,
    pub label_counts_before: Vec<usize>,
    pub world_nodes_before: usize,
    pub world_edges_before: usize,
    pub world_nodes: usize,
    pub world_edges: usize,
    pub world_hash: u64,
    pub rng_word_pos: u64,
    pub rng_probe: u64,
    pub ledger: LedgerRow,
    pub rule_cap_binding: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    pub arm: Arm,
    pub parent: u64,
    pub child: u64,
    pub executable: bool,
    pub changed_class: bool,
    pub cancelled: bool,
}

#[derive(Clone)]
struct State {
    cfg: RunConfig,
    world: HypergraphStore,
    rs: HypergraphStore,
    rng: ChaCha8Rng,
    led: Ledger,
    seq: i64,
    genes: HashMap<u64, Gene>,
}

fn probe(r: &ChaCha8Rng) -> u64 {
    r.clone().gen()
}
fn register(s: &mut State, root: NodeId, parent: Option<u64>, lineage: bool) -> Gene {
    let spec = decode_spec(&s.rs, root, &s.cfg.caps());
    let gene = Gene {
        id: key_u64(root),
        parent,
        member: get_int(&s.rs, root, K_MEMBER).unwrap() as usize,
        lineage,
        born: s.world.step() as usize,
        executable: spec
            .as_ref()
            .is_some_and(|x| spec_to_compiled(x, "birth").is_some()),
        class_hash: spec.as_ref().map(RuleSpec::class_hash),
        spec,
    };
    s.genes.insert(gene.id, gene.clone());
    gene
}
fn stocks(s: &State) -> Vec<Stock> {
    rule_roots(&s.rs)
        .into_iter()
        .map(|r| Stock {
            id: key_u64(r),
            capital: rule_tokens(&s.rs, r),
            budget: get_int(&s.rs, r, K_CREATION_BUDGET).unwrap(),
        })
        .collect()
}
fn ledger(s: &State) -> LedgerRow {
    let bound = total_bound(&s.rs);
    LedgerRow {
        free: s.led.free,
        bound,
        cumulative_influx: s.led.cumulative_influx,
        cumulative_dissipated: s.led.cumulative_dissipated,
        audit_residual: s.led.audit_residual(bound),
    }
}
fn initialize(seed: u64, horizon: usize) -> State {
    let cfg = crate::spectrum::config(seed, horizon);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut world = HypergraphStore::new();
    seed_world(
        &mut world,
        &mut rng,
        cfg.world_nodes,
        cfg.world_edges,
        cfg.n_labels,
    );
    let mut rs = HypergraphStore::new();
    let mut seq = 0;
    install_population(&mut rs, &g2_specs(), &cfg, &mut seq);
    let led = Ledger::new(cfg.free_cap.min(cfg.total).max(0), total_bound(&rs));
    let mut s = State {
        cfg,
        world,
        rs,
        rng,
        led,
        seq,
        genes: HashMap::new(),
    };
    for root in rule_roots(&s.rs) {
        register(&mut s, root, None, false);
    }
    s
}
fn snapshot(s: &State) -> Value {
    let mut genes: Vec<_> = s.genes.values().cloned().collect();
    genes.sort_by_key(|g| g.id);
    json!({"cfg":s.cfg,"world":s.world,"rules":s.rs,"ledger":ledger(s),"seq":s.seq,
        "genes":genes,"rng_seed":s.rng.get_seed(),"rng_stream":s.rng.get_stream(),
        "rng_word_pos":u64::try_from(s.rng.get_word_pos()).unwrap(),"rng_probe":probe(&s.rng)})
}
fn restore(value: Value) -> io::Result<State> {
    let cfg = serde_json::from_value(value["cfg"].clone())?;
    let world = serde_json::from_value(value["world"].clone())?;
    let rs = serde_json::from_value(value["rules"].clone())?;
    let seed: [u8; 32] = serde_json::from_value(value["rng_seed"].clone())?;
    let mut rng = ChaCha8Rng::from_seed(seed);
    rng.set_stream(value["rng_stream"].as_u64().unwrap());
    rng.set_word_pos(value["rng_word_pos"].as_u64().unwrap() as u128);
    assert_eq!(probe(&rng), value["rng_probe"].as_u64().unwrap());
    let l: LedgerRow = serde_json::from_value(value["ledger"].clone())?;
    let genes: Vec<Gene> = serde_json::from_value(value["genes"].clone())?;
    Ok(State {
        cfg,
        world,
        rs,
        rng,
        led: Ledger {
            free: l.free,
            cumulative_influx: l.cumulative_influx,
            cumulative_dissipated: l.cumulative_dissipated,
        },
        seq: value["seq"].as_i64().unwrap(),
        genes: genes.into_iter().map(|g| (g.id, g)).collect(),
    })
}

fn object(s: &mut State) -> Row {
    object_observed(s, None)
}

fn object_observed(s: &mut State, trace: Option<&mut Value>) -> Row {
    let pr = s.cfg.economy_params();
    let before = rule_roots(&s.rs);
    let balances: HashMap<_, _> = before
        .iter()
        .map(|r| {
            (
                *r,
                (
                    rule_tokens(&s.rs, *r),
                    get_int(&s.rs, *r, K_CREATION_BUDGET).unwrap(),
                ),
            )
        })
        .collect();
    let n = s.world.node_count();
    let e = s.world.edge_count();
    let mut labels = vec![0; s.cfg.n_labels as usize];
    for (_, d) in s.world.nodes() {
        labels[d.label.0 as usize] += 1;
    }
    s.led.influx(&pr);
    let mut costs = HashMap::new();
    rent_and_starve(&mut s.rs, &mut s.led, &pr, &mut costs);
    let (rules, _) = decode_all(&s.rs, &s.cfg.caps());
    let budgets: HashMap<_, _> = rule_roots(&s.rs)
        .into_iter()
        .map(|r| (r, get_int(&s.rs, r, K_CREATION_BUDGET).unwrap()))
        .collect();
    let gates: HashMap<_, _> = rules
        .iter()
        .map(|r| {
            let grower = is_grower(&r.spec);
            let creation = grower && budgets[&r.root] <= 0;
            let node = grower && !r.spec.creates.is_empty() && n >= s.cfg.world_ceiling;
            let edge =
                grower && r.spec.redges.len() > r.spec.pedges.len() && e >= s.cfg.edge_ceiling;
            let estimate = if creation || node || edge {
                None
            } else {
                Some(enum_cost(&r.spec, &labels, n, e))
            };
            (
                r.root,
                Gate {
                    creation_blocked: creation,
                    node_ceiling_blocked: node,
                    edge_ceiling_blocked: edge,
                    enumeration_estimate: estimate,
                    enumeration_skipped: estimate.is_some_and(|v| v > s.cfg.match_enum_budget),
                },
            )
        })
        .collect();
    if let Some(trace) = trace {
        *trace = bounded::conflict_attribution::probe(&s.world, &rules, &s.cfg, &budgets);
    }
    let out = object_step(&mut s.world, &rules, &mut s.rng, &s.cfg, &budgets, &[]);
    assert_eq!(
        out.budget_skips,
        gates.values().filter(|g| g.enumeration_skipped).count()
    );
    for root in rule_roots(&s.rs) {
        set_int(
            &mut s.rs,
            root,
            K_LAST_CANDIDATES,
            out.candidates_per_rule.get(&root).copied().unwrap_or(0),
        );
    }
    for (root, spend) in &out.creation_spend {
        let b = get_int(&s.rs, *root, K_CREATION_BUDGET).unwrap();
        set_int(&mut s.rs, *root, K_CREATION_BUDGET, b - spend);
    }
    let mut income = HashMap::new();
    distribute_income(
        &mut s.rs,
        &out.productive_per_rule,
        &mut s.led,
        &pr,
        &mut income,
    );
    let roots = before
        .iter()
        .map(|r| {
            let gene = &s.genes[&key_u64(*r)];
            let fate = out.fate_per_rule.get(r).copied().unwrap_or_default();
            assert!(fate.balances());
            RootRow {
                id: gene.id,
                member: gene.member,
                lineage: gene.lineage,
                executable: gene.executable,
                capital_before: balances[r].0,
                budget_before: balances[r].1,
                active: s.rs.get_node(*r).is_some(),
                cost: costs.get(r).copied().unwrap_or(0),
                income: income.get(r).copied().unwrap_or(0),
                spend: out.creation_spend.get(r).copied().unwrap_or(0),
                fate,
                gate: gates.get(r).cloned(),
                capital_after_income: s.rs.get_node(*r).map(|_| rule_tokens(&s.rs, *r)),
            }
        })
        .collect();
    Row {
        absolute_step: s.world.step() as usize,
        continuation_step: None,
        roots,
        births: vec![],
        stocks: vec![],
        label_counts_before: labels,
        world_nodes_before: n,
        world_edges_before: e,
        world_nodes: s.world.node_count(),
        world_edges: s.world.edge_count(),
        world_hash: s.world.canonical_hash(),
        rng_word_pos: 0,
        rng_probe: 0,
        ledger: ledger(s),
        rule_cap_binding: false,
    }
}

fn finish(s: &mut State, row: Row, arm: Option<Arm>) -> (Row, Option<Selection>) {
    finish_named(s, row, arm, None)
}

fn finish_named(s: &mut State, mut row: Row, arm: Option<Arm>, named: Option<(usize, &[u64; 5])>) -> (Row, Option<Selection>) {
    let pr = s.cfg.economy_params();
    let pre: HashMap<_, _> = rule_roots(&s.rs)
        .into_iter()
        .map(|r| {
            (
                r,
                (
                    rule_tokens(&s.rs, r),
                    get_int(&s.rs, r, K_CREATION_BUDGET).unwrap(),
                ),
            )
        })
        .collect();
    let mut shadow = s.rng.clone();
    let (fo, events) = funded_fission_recorded(
        &mut s.rs,
        &mut s.rng,
        &pr,
        &s.cfg.caps(),
        &mut s.seq,
        row.absolute_step as i64 - 1,
    );
    assert_eq!(fo.clonal as usize, events.len());
    assert_eq!(fo.mutant, 0);
    let selected = if arm.is_some() {
        events
            .iter()
            .find(|(p, _)| s.genes[&key_u64(*p)].member == 0)
            .copied()
    } else {
        None
    };
    let mut selection = None;
    for (parent, child) in events {
        let coin = shadow.gen::<f64>();
        let (operator, parameters) = recorded_mutation_draw(&mut shadow);
        let parent_id = key_u64(parent);
        let child_id = key_u64(child);
        let chosen = selected == Some((parent, child));
        let inherited_budget = get_int(&s.rs, child, K_CREATION_BUDGET).unwrap();
        assert_eq!(inherited_budget, pre[&parent].1);
        let child_tokens = rule_tokens(&s.rs, child);
        let parent_after_native = rule_tokens(&s.rs, parent);
        assert_eq!(child_tokens, pr.birth_cost);
        assert_eq!(parent_after_native, pre[&parent].0 - pr.birth_cost);
        if chosen {
            match arm.unwrap() {
                Arm::Prune => {
                    let (named_operator, named_parameters) = named.unwrap_or((11, &PRUNE_PARAMETERS));
                    assert!(apply_recorded_operator(
                        &mut s.rs,
                        child,
                        named_operator,
                        named_parameters,
                        &s.cfg.caps()
                    )
                    .unwrap());
                    if named.is_none() {
                        let mut expected = g2_specs()[0].clone();
                        expected.redges = vec![vec![2, 1]];
                        assert_eq!(decode_spec(&s.rs, child, &s.cfg.caps()), Some(expected));
                    }
                }
                Arm::Unscreened => {
                    apply_recorded_operator(&mut s.rs, child, operator, &parameters, &s.cfg.caps())
                        .unwrap();
                }
                _ => {}
            }
        }
        let lineage = chosen || s.genes[&parent_id].lineage;
        let gene = register(s, child, Some(parent_id), lineage);
        let cancelled = chosen && arm == Some(Arm::Refund);
        if cancelled {
            set_tokens(&mut s.rs, parent, parent_after_native + child_tokens);
            delete_rule(&mut s.rs, child);
        }
        if chosen {
            selection = Some(Selection {
                arm: arm.unwrap(),
                parent: parent_id,
                child: child_id,
                executable: gene.executable,
                changed_class: gene.class_hash != s.genes[&parent_id].class_hash,
                cancelled,
            });
        }
        row.births.push(BirthEvent {
            parent: parent_id,
            child: child_id,
            parent_before: pre[&parent].0,
            parent_after_native,
            child_tokens,
            inherited_budget,
            coin,
            operator,
            parameters,
            cancelled,
            gene,
        });
    }
    assert_eq!(shadow.get_word_pos(), s.rng.get_word_pos());
    assert_eq!(probe(&shadow), probe(&s.rng));
    refill_creation_budgets(&mut s.rs, &pr);
    row.stocks = stocks(s);
    row.ledger = ledger(s);
    assert_eq!(row.ledger.audit_residual, 0);
    row.rng_word_pos = u64::try_from(s.rng.get_word_pos()).unwrap();
    row.rng_probe = probe(&s.rng);
    row.rule_cap_binding = row.stocks.len() >= s.cfg.rule_cap;
    (row, selection)
}

#[derive(Default, Serialize, Deserialize)]
struct Totals {
    productive: [i64; 3],
    member_copy_steps: [usize; 3],
    lineage_productive: i64,
    lineage_copy_steps: usize,
    lineage_income: i64,
    lineage_rent: i64,
    late_lineage_income: i64,
    late_lineage_rent: i64,
    parent_productive: i64,
    parent_income: i64,
    parent_rent: i64,
    parent_death_step: Option<usize>,
    child_death_step: Option<usize>,
    first_lineage_productive: Option<usize>,
    late_member_windows: Vec<[i64; 3]>,
    late_lineage_windows: Vec<i64>,
    enumeration_skips: usize,
    resource_gates: usize,
    rule_cap_steps: usize,
}
impl Totals {
    fn new(_horizon: usize) -> Self {
        Self {
            late_member_windows: vec![[0; 3]; 4],
            late_lineage_windows: vec![0; 4],
            ..Self::default()
        }
    }
    fn add(&mut self, row: &Row, selection: &Selection, horizon: usize) {
        let step = row.continuation_step.unwrap();
        let late = step > horizon / 2;
        let window = if late {
            Some((step - horizon / 2 - 1) / (horizon / 8))
        } else {
            None
        };
        for r in &row.roots {
            self.productive[r.member] += r.fate.productive;
            self.member_copy_steps[r.member] += usize::from(r.active);
            if let Some(g) = &r.gate {
                self.enumeration_skips += usize::from(g.enumeration_skipped);
                self.resource_gates += usize::from(
                    g.creation_blocked || g.node_ceiling_blocked || g.edge_ceiling_blocked,
                );
            }
            if r.lineage {
                self.lineage_productive += r.fate.productive;
                self.lineage_copy_steps += usize::from(r.active);
                self.lineage_income += r.income;
                self.lineage_rent += r.cost;
                if late {
                    self.late_lineage_income += r.income;
                    self.late_lineage_rent += r.cost;
                }
                if r.fate.productive > 0 && self.first_lineage_productive.is_none() {
                    self.first_lineage_productive = Some(step);
                }
                if let Some(w) = window {
                    self.late_lineage_windows[w] += r.fate.productive;
                }
            }
            if r.id == selection.parent {
                self.parent_productive += r.fate.productive;
                self.parent_income += r.income;
                self.parent_rent += r.cost;
            }
            if let Some(w) = window {
                self.late_member_windows[w][r.member] += r.fate.productive;
            }
        }
        if !row.stocks.iter().any(|x| x.id == selection.parent) && self.parent_death_step.is_none()
        {
            self.parent_death_step = Some(step);
        }
        if !selection.cancelled
            && !row.stocks.iter().any(|x| x.id == selection.child)
            && self.child_death_step.is_none()
        {
            self.child_death_step = Some(step);
        }
        self.rule_cap_steps += usize::from(row.rule_cap_binding);
    }
}
fn verdict(
    s: &State,
    selection: &Selection,
    total: &Totals,
    completed: usize,
    error: &Option<String>,
) -> Value {
    let horizon = s.cfg.steps;
    let end = stocks(s);
    let mut final_members = [0; 3];
    let mut lineage_copies = 0;
    let mut lineage_capital = 0;
    for stock in &end {
        let g = &s.genes[&stock.id];
        final_members[g.member] += 1;
        if g.lineage {
            lineage_copies += 1;
            lineage_capital += stock.capital;
        }
    }
    let complete = completed == horizon && error.is_none();
    let cycle = complete
        && final_members.iter().all(|n| *n > 0)
        && total
            .late_member_windows
            .iter()
            .all(|w| w.iter().all(|p| *p > 0));
    let measured = complete && total.enumeration_skips == 0;
    let established = selection.executable
        && measured
        && cycle
        && lineage_copies > 0
        && total.late_lineage_windows.iter().all(|p| *p > 0)
        && total.late_lineage_income >= total.late_lineage_rent;
    json!({"complete":complete,"completed_steps":completed,"requested_steps":horizon,"error":error,
        "whole_cycle_persistent":cycle,"whole_cycle_uncensored":cycle&&measured,
        "bounded_establishment":if selection.cancelled {None} else {Some(established)},
        "selected_child_executable":if selection.cancelled {None} else {Some(selection.executable)},
        "final_member_copies":final_members,"final_lineage_copies":lineage_copies,"final_lineage_capital":lineage_capital,
        "parent_final_capital":end.iter().find(|x|x.id==selection.parent).map(|x|x.capital),
        "lineage_productive_rate":if measured&&selection.executable&&!selection.cancelled {Some(total.lineage_productive as f64/horizon as f64)} else {None},
        "lineage_per_copy_rate":if measured&&selection.executable&&!selection.cancelled&&total.lineage_copy_steps>0 {Some(total.lineage_productive as f64/total.lineage_copy_steps as f64)} else {None},
        "parent_productive_rate":if measured {Some(total.parent_productive as f64/horizon as f64)} else {None},
        "totals":total})
}
fn save(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer(&mut f, value)?;
    f.write_all(b"\n")
}
fn write_row(file: &mut BufWriter<File>, row: &Row) -> io::Result<()> {
    serde_json::to_writer(&mut *file, row)?;
    file.write_all(b"\n")?;
    file.flush()
}
fn status(out: &Path, value: Value) -> io::Result<()> {
    save(&out.join("status.next.json"), &value)?;
    fs::rename(out.join("status.next.json"), out.join("status.json"))
}
fn continuation(out: &Path, mut s: State, selection: Selection) -> io::Result<Value> {
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join("rows.jsonl"))?,
    );
    let mut totals = Totals::new(s.cfg.steps);
    let mut completed = 0;
    let mut error = None;
    for step in 1..=s.cfg.steps {
        let mut row = object(&mut s);
        row.continuation_step = Some(step);
        let (row, _) = finish(&mut s, row, None);
        totals.add(&row, &selection, s.cfg.steps);
        write_row(&mut file, &row)?;
        completed += 1;
        if s.world.node_count() > s.cfg.node_explosion_cap {
            error = Some("hard node explosion stop".into());
            break;
        }
    }
    let score = verdict(&s, &selection, &totals, completed, &error);
    save(&out.join("final.json"), &snapshot(&s))?;
    save(&out.join("score.json"), &score)?;
    if error.is_some() {
        return Err(io::Error::other(
            "incomplete continuation; raw prefix retained",
        ));
    }
    Ok(score)
}

pub fn run(out: &Path, preflight: bool) -> io::Result<Value> {
    fs::create_dir_all(out)?;
    let _lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("assay.lock"))?;
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_locked(out, preflight)));
    let result =
        result.unwrap_or_else(|_| Err(io::Error::other("native/harness panic; output incomplete")));
    if let Err(error) = &result {
        let prior = fs::read(out.join("status.json"))
            .ok()
            .and_then(|v| serde_json::from_slice::<Value>(&v).ok());
        status(
            out,
            json!({"state":"failed","error":error.to_string(),"last_progress":prior}),
        )?;
    }
    result
}
fn run_locked(out: &Path, preflight: bool) -> io::Result<Value> {
    let seeds: &[u64] = if preflight { &[7, 42] } else { &SEEDS };
    let horizon = if preflight { 64 } else { 2000 };
    save(
        &out.join("manifest.json"),
        &json!({"protocol":PROTOCOL,"engineering_preflight":preflight,"seeds":seeds,
        "arms":ARMS,"horizon":horizon,"prefix_limit":256,"prune_parameters":PRUNE_PARAMETERS,
        "configs":seeds.iter().map(|s|crate::spectrum::config(*s,horizon)).collect::<Vec<_>>(),
        "continuations_requested":seeds.len()*4,"future_mutation":false,"fission_enabled":true}),
    )?;
    fs::write(out.join("preregistration.md"), CONTRACT)?;
    status(out, json!({"state":"running","continuations_completed":0}))?;
    let mut results = vec![];
    let mut prefix_steps = 0;
    let mut triggered = 0;
    for &seed in seeds {
        let dir = out.join(format!("seed_{seed}"));
        fs::create_dir(&dir)?;
        let mut s = initialize(seed, horizon);
        save(&dir.join("initial.json"), &snapshot(&s))?;
        let mut prefix = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(dir.join("prefix.jsonl"))?,
        );
        let mut fork = None;
        for _ in 0..256 {
            let row = object(&mut s);
            prefix_steps += 1;
            let checkpoint = snapshot(&s);
            let pending = row.clone();
            let (finished, selected) = finish(&mut s, row, Some(Arm::Faithful));
            if let Some(selection) = selected {
                fork = Some((checkpoint, pending, finished, selection));
                break;
            }
            write_row(&mut prefix, &finished)?;
            if s.world.node_count() > s.cfg.node_explosion_cap {
                return Err(io::Error::other("hard prefix explosion stop"));
            }
        }
        let Some((checkpoint, pending, faithful_birth, faithful_selection)) = fork else {
            save(
                &dir.join("untriggered.json"),
                &json!({"seed":seed,"prefix_steps":256,"reason":"no natural R1 birth"}),
            )?;
            continue;
        };
        triggered += 1;
        save(
            &dir.join("fork.json"),
            &json!({"seed":seed,"checkpoint":checkpoint,"pending":pending,
            "faithful_selection":faithful_selection,"faithful_birth":faithful_birth}),
        )?;
        let faithful_state = snapshot(&s);
        for arm in ARMS {
            let arm_dir = dir.join(arm.name());
            fs::create_dir(&arm_dir)?;
            let mut branch = restore(checkpoint.clone())?;
            let (birth, selection) = finish(&mut branch, pending.clone(), Some(arm));
            let selection = selection.unwrap();
            assert_eq!(birth.rng_word_pos, faithful_birth.rng_word_pos);
            assert_eq!(birth.rng_probe, faithful_birth.rng_probe);
            assert_eq!(birth.world_hash, faithful_birth.world_hash);
            assert_eq!(selection.parent, faithful_selection.parent);
            assert_eq!(selection.child, faithful_selection.child);
            if arm == Arm::Faithful {
                assert_eq!(birth, faithful_birth);
                assert_eq!(snapshot(&branch), faithful_state);
            }
            save(
                &arm_dir.join("birth.json"),
                &json!({"selection":selection,"row":birth,"initial":snapshot(&branch)}),
            )?;
            let score = continuation(&arm_dir, branch, selection)?;
            results.push(json!({"seed":seed,"arm":arm,"score":score}));
            status(
                out,
                json!({"state":"running","continuations_completed":results.len(),"shared_prefix_steps":prefix_steps,
                "scientific_advancing_steps":prefix_steps+results.len()*horizon,"active_seed":seed}),
            )?;
            println!(
                "seed {seed} {}: {}/{} continuations recorded",
                arm.name(),
                results.len(),
                seeds.len() * 4
            );
            io::stdout().flush()?;
        }
    }
    let summary = json!({"protocol":PROTOCOL,"engineering_preflight":preflight,"triggered_seeds":triggered,
        "shared_prefix_steps":prefix_steps,"continuations_completed":results.len(),"continuation_steps":results.len()*horizon,
        "scientific_advancing_steps":prefix_steps+results.len()*horizon,"results":results,
        "probation_launched":false,"mutation_sweep_launched":false});
    save(&out.join("summary.json"), &summary)?;
    status(
        out,
        json!({"state":"complete","continuations_completed":results.len(),
        "scientific_advancing_steps":prefix_steps+results.len()*horizon}),
    )?;
    Ok(summary)
}

/// Separate-process deterministic replay of an existing pre-fission checkpoint.
pub fn replay(fork_path: &Path, arm: Arm, out: &Path) -> io::Result<Value> {
    fs::create_dir(out)?;
    let fork: Value = serde_json::from_slice(&fs::read(fork_path)?)?;
    let mut state = restore(fork["checkpoint"].clone())?;
    let row: Row = serde_json::from_value(fork["pending"].clone())?;
    let (birth, selected) = finish(&mut state, row, Some(arm));
    let selected = selected.unwrap();
    save(
        &out.join("birth.json"),
        &json!({"selection":selected,"row":birth,"initial":snapshot(&state)}),
    )?;
    continuation(out, state, selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::funded_fission;
    #[test]
    fn recorded_fission_is_native_material_money_and_rng() {
        for seed in 0..64 {
            let mut s = initialize(seed, 64);
            for r in rule_roots(&s.rs) {
                set_tokens(&mut s.rs, r, 120);
            }
            let mut other = s.clone();
            let pr = s.cfg.economy_params();
            let native = funded_fission(&mut s.rs, &mut s.rng, &pr, &s.cfg.caps(), &mut s.seq, 0);
            let (recorded, events) = funded_fission_recorded(
                &mut other.rs,
                &mut other.rng,
                &pr,
                &other.cfg.caps(),
                &mut other.seq,
                0,
            );
            assert_eq!(
                (native.clonal, native.mutant, native.noop),
                (recorded.clonal, recorded.mutant, recorded.noop)
            );
            assert_eq!(events.len(), 3);
            assert_eq!(
                serde_json::to_value(&s.rs).unwrap(),
                serde_json::to_value(&other.rs).unwrap()
            );
            assert_eq!(probe(&s.rng), probe(&other.rng));
            assert_eq!(s.rng.get_word_pos(), other.rng.get_word_pos());
        }
    }
    #[test]
    fn fork_restore_and_named_birth_pair_finances_and_allowance() {
        let mut s = initialize(7, 64);
        let mut fork = None;
        for _ in 0..256 {
            let pending = object(&mut s);
            let checkpoint = snapshot(&s);
            let (row, selection) = finish(&mut s, pending.clone(), Some(Arm::Faithful));
            if selection.is_some() {
                fork = Some((checkpoint, pending, row));
                break;
            }
        }
        let (checkpoint, pending, faithful) = fork.unwrap();
        let mut outputs = vec![];
        for arm in ARMS {
            let mut branch = restore(checkpoint.clone()).unwrap();
            let (row, selected) = finish(&mut branch, pending.clone(), Some(arm));
            let selected = selected.unwrap();
            assert_eq!(row.rng_probe, faithful.rng_probe);
            assert_eq!(row.rng_word_pos, faithful.rng_word_pos);
            let parent = row.stocks.iter().find(|r| r.id == selected.parent).unwrap();
            let base = faithful
                .stocks
                .iter()
                .find(|r| r.id == selected.parent)
                .unwrap();
            assert_eq!(
                parent.capital,
                base.capital + if arm == Arm::Refund { 50 } else { 0 }
            );
            let b = row
                .births
                .iter()
                .find(|b| b.child == selected.child)
                .unwrap();
            assert_eq!(
                b.inherited_budget,
                faithful
                    .births
                    .iter()
                    .find(|b| b.child == selected.child)
                    .unwrap()
                    .inherited_budget
            );
            assert_eq!(row.ledger.audit_residual, 0);
            if arm == Arm::Prune {
                assert!(selected.changed_class);
                assert!(selected.executable);
            }
            outputs.push(branch);
        }
        let mut restored = restore(snapshot(&outputs[0])).unwrap();
        let a = object(&mut outputs[0]);
        let b = object(&mut restored);
        assert_eq!(
            finish(&mut outputs[0], a, None).0,
            finish(&mut restored, b, None).0
        );
    }
    #[test]
    fn invalid_and_censored_child_rates_are_null() {
        let s = initialize(7, 64);
        let root = key_u64(rule_roots(&s.rs)[0]);
        let selection = Selection {
            arm: Arm::Unscreened,
            parent: root,
            child: root,
            executable: false,
            changed_class: true,
            cancelled: false,
        };
        let total = Totals::new(64);
        assert!(verdict(&s, &selection, &total, 64, &None)["lineage_productive_rate"].is_null());
        let selection = Selection {
            executable: true,
            ..selection
        };
        let total = Totals {
            enumeration_skips: 1,
            ..Totals::new(64)
        };
        assert!(verdict(&s, &selection, &total, 64, &None)["lineage_productive_rate"].is_null());
        assert_eq!(
            verdict(&s, &selection, &total, 64, &None)["bounded_establishment"],
            false
        );
    }

    #[test]
    fn faithful_classes_survive_refund_slot_reuse() {
        let mut s = initialize(7, 64);
        for _ in 0..256 {
            let pending = object(&mut s);
            let (_, selected) = finish(&mut s, pending, Some(Arm::Refund));
            if selected.is_some() {
                break;
            }
        }
        for _ in 0..64 {
            let pending = object(&mut s);
            let (row, _) = finish(&mut s, pending, None);
            for birth in row.births {
                assert_eq!(birth.gene.class_hash, s.genes[&birth.parent].class_hash);
                assert_eq!(birth.gene.executable, s.genes[&birth.parent].executable);
            }
        }
    }

    #[test]
    fn disk_checkpoint_preserves_all_float_bits_and_next_step() {
        let mut s = initialize(11, 64);
        for _ in 0..27 {
            let pending = object(&mut s);
            finish(&mut s, pending, None);
        }
        let original = snapshot(&s);
        let disk = serde_json::to_vec(&original).unwrap();
        let parsed: Value = serde_json::from_slice(&disk).unwrap();
        assert_eq!(original, parsed, "JSON checkpoint changed numerical values");
        let mut restored = restore(parsed).unwrap();
        assert_eq!(snapshot(&s), snapshot(&restored));
        let a = object(&mut s);
        let b = object(&mut restored);
        assert_eq!(finish(&mut s, a, None).0, finish(&mut restored, b, None).0);
    }
}
