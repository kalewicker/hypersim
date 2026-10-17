//! V6 / K1 — is arm B TURNOVER or PERSISTENCE?
//!
//! The H2 ABC split treated arm B (non-replicating, sustained by `relocate`) as
//! "arm A minus replication". That holds only if B's population actually turns
//! over — individuals dying and being replaced, just not by heredity. If B is
//! instead the same few individuals grazing indefinitely, then A and B differ in
//! replication AND in turnover-vs-persistence, and the positive property is
//! sustained PRESENCE, not sustained TURNOVER.
//!
//! Criterion and verdict rule pre-committed in
//! docs/validation_v6_k1_turnover_criterion.md (commit 7da6c2e) BEFORE this ran.
//!
//! Everything here is a REALIZED quantity: births/deaths are carrier NodeId set
//! differences step to step, never a configured rate or a rule-firing count.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::{HashMap, HashSet};

const VACUUM: u64 = 0xE6_0000_0000_0000;
const STEPS: u64 = 6_000;
const WL_DEPTH: u8 = 2;
const R_PERSIST: u64 = 3; // H0.1 / J2 definition, verbatim

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "k1"
run_id = "k1"
seed = {seed}
[rules]
source = "Inline"
inline_toml = "rules = []"
[initial]
condition_type = "PureCrystal"
width = 6
height = 6
arity = 3
[engine]
step_mode = "MaxParallel"
max_steps = {STEPS}
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = true
[alife]
enabled = true
seed_carrier_marker = false
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

/// Byte-identical to h2_step3_split.rs — the arms must be the SAME runs whose
/// CE the H2 verdict rests on, or K1 measures a different experiment.
fn motility_toml() -> &'static str {
    r#"
[[rules]]
name = "relocate"
priority = 2
[rules.lhs]
nodes = ["site", "carrier", "target"]
edges = [["site", "carrier"], ["site", "target"]]
[rules.lhs.node_labels]
carrier = 100
site = 0
target = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["site", "carrier", "target"]
[[rules.lhs.node_property_compares]]
var_a = "carrier"
key_a = 1
op = "lt"
var_b = "carrier"
key_b = 13
[[rules.lhs.node_property_guards]]
var = "target"
property = 1
op = "gte"
value = 1.0
[[rules.lhs.node_property_guards]]
var = "target"
property = 22
op = "eq"
value = false
[[rules.lhs.node_property_guards]]
var = "site"
property = 1
op = "lt"
value = 1.0
[rules.rhs]
nodes = ["site", "carrier", "target"]
edges = [["site", "target"], ["target", "carrier"]]
[[rules.rhs.property_effects]]
var = "site"
key = 22
op = "set"
value = false
[[rules.rhs.property_effects]]
var = "target"
key = 22
op = "set"
value = true
"#
}

fn plant_markers(store: &mut HypergraphStore, seed: u64, n: usize) {
    use rand::{Rng, SeedableRng};
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    if ids.is_empty() || n == 0 { return; }
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0x00CA_111E);
    let (mut planted, mut used) = (0usize, HashSet::new());
    let cap = n.min(ids.len());
    while planted < cap {
        let pick = ids[rng.gen_range(0..ids.len())];
        if used.insert(pick) {
            store.get_node_mut(pick).unwrap()
                .set_property(hg_alife::keys::CARRIER_MARKER, PropertyValue::Bool(true));
            planted += 1;
        }
    }
}

fn build_rules(cfg: &SimConfig, sterile: bool, motile: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if sterile { biology.retain(|r| r.name != "replicate"); }
    if motile { biology.extend(parse_rules_toml(motility_toml()).unwrap()); }
    compile_rules(&biology).unwrap()
}

#[derive(Default)]
struct Demo {
    births: u64,
    deaths: u64,
    /// completed lifetimes only (born AND died in-window)
    lifetimes: Vec<u64>,
    censored: u64,
    pop_sum: u64,
    steps: u64,
    peak: usize,
    last_alive: u64,
    /// realized RELOCATIONS: a surviving carrier's incident-site set changed.
    /// Distinguishes "persists in place" from "persists by moving" — a carrier
    /// that never moved would starve, so this must be nonzero for arm B.
    moves: u64,
}
impl Demo {
    fn mean_pop(&self) -> f64 { self.pop_sum as f64 / self.steps.max(1) as f64 }
    fn mean_lifetime(&self) -> f64 {
        if self.lifetimes.is_empty() { return f64::NAN; }
        self.lifetimes.iter().sum::<u64>() as f64 / self.lifetimes.len() as f64
    }
    fn censored_frac(&self) -> f64 {
        let ever = self.births.max(1);
        self.censored as f64 / ever as f64
    }
    /// fraction of the standing population replaced per step
    fn per_capita_death_rate(&self) -> f64 {
        let denom = self.mean_pop() * self.steps as f64;
        if denom > 0.0 { self.deaths as f64 / denom } else { 0.0 }
    }
}

struct Twtf { m: usize, twtf: f64, transitions: u64 }

fn twtf_of(live: &HashMap<u64, u64>, src: &HashMap<u64, u64>) -> Twtf {
    let total: u64 = src.values().sum();
    let transient: u64 = src.iter()
        .filter(|(h, _)| *live.get(*h).unwrap_or(&0) < R_PERSIST)
        .map(|(_, c)| *c)
        .sum();
    Twtf {
        m: live.len().saturating_sub(1), // minus VACUUM sentinel
        twtf: if total > 0 { transient as f64 / total as f64 } else { 0.0 },
        transitions: total,
    }
}

struct Out { demo: Demo, all: Twtf, carrier: Twtf }

fn run(seed: u64, sterile: bool, motile: bool, n_markers: usize, window: u64) -> Out {
    let cfg = config(seed);
    let rules = build_rules(&cfg, sterile, motile);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    plant_markers(&mut store, seed, n_markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut demo = Demo::default();
    let mut born_at: HashMap<NodeId, u64> = HashMap::new();
    let mut prev_carriers: HashSet<NodeId> = HashSet::new();
    let mut site_prev: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    // WL-partition liveness / source-transition counts, H0 definitions verbatim.
    let mut live: HashMap<u64, u64> = HashMap::new();
    let mut src_all: HashMap<u64, u64> = HashMap::new();
    let mut src_carrier: HashMap<u64, u64> = HashMap::new();
    let mut is_c: HashMap<u64, bool> = HashMap::new();
    live.insert(VACUUM, u64::MAX);
    is_c.insert(VACUUM, false);
    let mut prev: Option<HashMap<NodeId, u64>> = None;

    for step in 1..=window {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut now: HashMap<NodeId, u64> = HashMap::new();
        let mut present: HashSet<u64> = HashSet::new();
        let mut carriers: HashSet<NodeId> = HashSet::new();
        let mut site_now: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            if is_carrier {
                carriers.insert(id);
                let mut sites: Vec<NodeId> = snap.edges_incident_to(id).iter()
                    .filter_map(|&e| snap.get_edge(e))
                    .flat_map(|e| e.nodes.iter().copied())
                    .filter(|&n| n != id)
                    .collect();
                sites.sort_unstable();
                sites.dedup();
                site_now.insert(id, sites);
            }
            let h = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            now.insert(id, h);
            present.insert(h);
            is_c.entry(h).or_insert(is_carrier);
        }
        for &h in &present { *live.entry(h).or_insert(0) += 1; }

        // demography: realized NodeId set differences
        for &id in carriers.difference(&prev_carriers) { demo.births += 1; born_at.insert(id, step); }
        for &id in prev_carriers.difference(&carriers) {
            demo.deaths += 1;
            if let Some(b) = born_at.remove(&id) { demo.lifetimes.push(step - b); }
        }
        for (id, s) in &site_now {
            if let Some(old) = site_prev.get(id) { if old != s { demo.moves += 1; } }
        }
        site_prev = site_now;
        demo.pop_sum += carriers.len() as u64;
        demo.steps = step;
        demo.peak = demo.peak.max(carriers.len());
        if !carriers.is_empty() { demo.last_alive = step; }
        prev_carriers = carriers;

        if let Some(p) = &prev {
            for (_, &hp) in p {
                *src_all.entry(hp).or_insert(0) += 1;
                if *is_c.get(&hp).unwrap_or(&false) { *src_carrier.entry(hp).or_insert(0) += 1; }
            }
            for (id, _) in &now {
                if !p.contains_key(id) { *src_all.entry(VACUUM).or_insert(0) += 1; }
            }
        }
        prev = Some(now);
    }
    demo.censored = born_at.len() as u64;

    let all = twtf_of(&live, &src_all);
    let carrier_live: HashMap<u64, u64> = live.iter()
        .filter(|(h, _)| *is_c.get(*h).unwrap_or(&false))
        .map(|(&h, &v)| (h, v))
        .collect();
    let mut carrier = twtf_of(&carrier_live, &src_carrier);
    carrier.m = carrier_live.len(); // no VACUUM sentinel in the carrier-only map
    Out { demo, all, carrier }
}

fn f(x: f64) -> String { if x.is_nan() { "   n/a".into() } else { format!("{x:6.1}") } }

#[test]
fn k1_turnover_vs_persistence() {
    let seed = 12345;
    eprintln!("\n================ V6 / K1 — IS ARM B TURNOVER OR PERSISTENCE? (seed {seed}, W={STEPS}) ================");

    let a = run(seed, false, false, 1, STEPS);   // A: replicating (F8)
    let b = run(seed, true, true, 6, STEPS);     // B: non-replicating, sustained by motility
    let c = run(seed, true, false, 6, STEPS);    // C: non-replicating, not sustained (context)

    eprintln!("\nK1.1 — demographic profile (realized NodeId births/deaths over the scoring window)");
    eprintln!("  arm                       | peak | last-alive | mean pop | births | deaths | d/b   | mean completed lifetime | censored | per-capita death rate | relocations");
    eprintln!("  --------------------------+------+------------+----------+--------+--------+-------+-------------------------+----------+-----------------------+------------");
    for (name, o) in [("A replicating", &a), ("B non-repl SUSTAINED", &b), ("C non-repl token", &c)] {
        let d = &o.demo;
        let db = if d.births > 0 { d.deaths as f64 / d.births as f64 } else { f64::NAN };
        eprintln!("  {:<25} | {:>4} | {:>10} | {:>8.2} | {:>6} | {:>6} | {:>5.3} | {:>23} | {:>7.1}% | {:>21.6} | {:>11}",
            name, d.peak, d.last_alive, d.mean_pop(), d.births, d.deaths, db,
            f(d.mean_lifetime()), 100.0 * d.censored_frac(), d.per_capita_death_rate(), d.moves);
    }

    eprintln!("\nK1.2 — TWTF on the WL depth-2 partition (R={R_PERSIST}); static floor = same biology, ZERO carriers");
    let floor_long = run(seed, false, false, 0, STEPS);
    let floor_short = run(seed, false, false, 0, 120);
    let a_short = run(seed, false, false, 1, 120);
    let b_short = run(seed, true, true, 6, 120);
    eprintln!("  run                       | W    | all: M / TWTF / transitions        | carrier-only: M / TWTF / transitions");
    eprintln!("  --------------------------+------+------------------------------------+-------------------------------------");
    for (name, w, o) in [
        ("static floor (0 carriers)", 120u64, &floor_short),
        ("A replicating", 120, &a_short),
        ("B non-repl SUSTAINED", 120, &b_short),
        ("static floor (0 carriers)", STEPS, &floor_long),
        ("A replicating", STEPS, &a),
        ("B non-repl SUSTAINED", STEPS, &b),
    ] {
        eprintln!("  {:<25} | {:>4} | {:>3} / {:>6.3} / {:>10}          | {:>3} / {:>6.3} / {:>10}",
            name, w, o.all.m, o.all.twtf, o.all.transitions,
            o.carrier.m, o.carrier.twtf, o.carrier.transitions);
    }

    // ---- pre-committed verdict rule, applied mechanically (docs/validation_v6_k1_turnover_criterion.md)
    let (ra, rb) = (a.demo.per_capita_death_rate(), b.demo.per_capita_death_rate());
    let (la, lb) = (a.demo.mean_lifetime(), b.demo.mean_lifetime());
    let cens_b = b.demo.censored_frac();
    let turnover = rb >= 0.5 * ra && (lb.is_nan() || lb <= 2.0 * la) && cens_b <= 0.5;
    let persistence = rb <= 0.1 * ra || (!lb.is_nan() && lb >= 10.0 * la) || cens_b >= 0.5;

    eprintln!("\n================ K1.3 — PRE-COMMITTED VERDICT (rule fixed in commit 7da6c2e) ================");
    eprintln!("  r_A = {ra:.6}, r_B = {rb:.6}  (r_B / r_A = {:.3}; TURNOVER needs >= 0.5)", rb / ra);
    eprintln!("  L_A = {}, L_B = {}  (L_B / L_A = {:.3}; PERSISTENCE needs >= 10)", f(la), f(lb), lb / la);
    eprintln!("  censored_frac_B = {:.3}  (PERSISTENCE needs >= 0.5, TURNOVER needs <= 0.5)", cens_b);
    if turnover && !persistence {
        eprintln!("  => TURNOVER. Arm B has A-comparable turnover; A and B differ only in replication as the ABC");
        eprintln!("     design assumed. 'Sustained TURNOVER' confirmed. Proceed to K2/K3 on the confirmed verdict.");
    } else if persistence && !turnover {
        eprintln!("  => PERSISTENCE. Arm B sustains by the SAME individuals staying alive, not by replacement.");
        eprintln!("     The verdict SHARPENS to 'sustained PRESENCE is load-bearing'. Replication-per-se stays");
        eprintln!("     excluded (C is non-replicating and undetectable), but the positive property is presence,");
        eprintln!("     not turnover. This is the finding: arm B is NOT redefined and relocate is NOT adjusted.");
    } else {
        eprintln!("  => INTERMEDIATE. Neither branch fires cleanly; the verdict is BOUNDED by the numbers above,");
        eprintln!("     not asserted. Report the demography and state the bound.");
    }
    eprintln!("=========================================================================================\n");
}
