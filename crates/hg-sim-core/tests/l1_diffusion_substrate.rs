//! V7 / L1.2 — characterise the `diffuse` substrate modification as PHYSICS, bare,
//! before any biology uses it (firewall; V7 rule 4).
//!
//! Mechanism committed in docs/validation_v7_l1_sustain_mechanism.md (commit
//! 01bf010) BEFORE this ran. Two characterisations, both pre-declared there:
//!
//!   (1) AT THE OPERATING POINT, BARE (5.0 start, influx 0.34, ZERO carriers):
//!       every node starts above the depletion threshold, so `diffuse` should
//!       never fire and the substrate should be IDENTICAL to plain static.
//!       This is the no-back-reaction / firewall check.
//!   (2) ON A GRADIENT FIXTURE (one node loaded, rest empty, influx REMOVED):
//!       the rule is exercised. Expect EXACT conservation (it is neither source
//!       nor sink), flat n(t), unchanged degree, one-directional transport.
//!
//! `diffuse` is gradient-driven: no gradient, no transport. Reporting it inert at
//! the operating point is the honest characterisation, not a gap.
#![cfg(feature = "alife")]

use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::{HashMap, HashSet};

const ENERGY: u32 = 1;
const R_PERSIST: u64 = 3; // H0.1 / J2 definition, verbatim
const WL_DEPTH: u8 = 2;

fn config(seed: u64, substrate_energy: f64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "l1phys"
run_id = "l1phys"
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
max_steps = {steps}
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = true
[alife]
enabled = true
seed_carrier_marker = false
initial_substrate_energy = {substrate_energy}
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

/// The committed `diffuse` rule: conservative, gradient-driven, substrate-label-only.
/// It names NO carrier property and NO carrier label.
pub fn diffuse_toml() -> &'static str {
    r#"
[[rules]]
name = "diffuse"
priority = 1
[rules.lhs]
nodes = ["hi", "lo"]
edges = [["hi", "lo"]]
[rules.lhs.node_labels]
hi = 0
lo = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["hi", "lo"]
[[rules.lhs.node_property_guards]]
var = "hi"
property = 1
op = "gte"
value = 2.0
[[rules.lhs.node_property_guards]]
var = "lo"
property = 1
op = "lt"
value = 1.0
[rules.rhs]
nodes = ["hi", "lo"]
edges = [["hi", "lo"]]
[[rules.rhs.property_effects]]
var = "hi"
key = 1
op = "subtract"
value = 1.0
[[rules.rhs.property_effects]]
var = "lo"
key = 1
op = "add"
value = 1.0
"#
}

fn build(cfg: &SimConfig, diffuse: bool, influx: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if !influx { biology.retain(|r| r.name != "influx"); }
    if diffuse { biology.extend(parse_rules_toml(diffuse_toml()).unwrap()); }
    compile_rules(&biology).unwrap()
}

fn energies_snap(snap: &hg_core::GraphSnapshot<'_>) -> Vec<(NodeId, f64)> {
    let mut v: Vec<(NodeId, f64)> = snap.nodes().map(|(id, d)| {
        (id, match d.get_property(ENERGY) { Some(PropertyValue::Float(x)) => x, _ => 0.0 })
    }).collect();
    v.sort_by_key(|(id, _)| *id);
    v
}

fn energies(store: &HypergraphStore) -> Vec<(NodeId, f64)> {
    let mut v: Vec<(NodeId, f64)> = store.nodes().map(|(id, d)| {
        (id, match d.get_property(ENERGY) { Some(PropertyValue::Float(x)) => x, _ => 0.0 })
    }).collect();
    v.sort_by_key(|(id, _)| *id);
    v
}

struct Trace {
    n: Vec<usize>,
    edges: Vec<usize>,
    total_energy: Vec<f64>,
    /// per-step full energy vector, for the bit-level identity check
    vecs: Vec<Vec<(NodeId, f64)>>,
    /// steps on which any node's energy changed
    active_steps: u64,
    wl_twtf: f64,
    wl_m: usize,
    eb_twtf: f64,
    eb_m: usize,
}

fn energy_band_class(label: i64, e: f64) -> u64 {
    let band = e.floor().clamp(-1.0, 64.0) as i64;
    ((label as u64) << 32) ^ (band as u64 & 0xFFFF_FFFF) ^ 0x9E37_79B9
}

fn twtf(live: &HashMap<u64, u64>, src: &HashMap<u64, u64>) -> (usize, f64) {
    let total: u64 = src.values().sum();
    let transient: u64 = src.iter()
        .filter(|(h, _)| *live.get(*h).unwrap_or(&0) < R_PERSIST)
        .map(|(_, c)| *c).sum();
    (live.len(), if total > 0 { transient as f64 / total as f64 } else { 0.0 })
}

fn run(cfg: &SimConfig, rules: Vec<hg_rules::CompiledRule>, steps: u64, load_one: Option<f64>) -> Trace {
    let mut store = hg_sim_core::runner::build_initial_store(cfg);
    if let Some(e) = load_one {
        let first = store.nodes().next().map(|(id, _)| id);
        if let Some(id) = first {
            store.get_node_mut(id).unwrap().set_property(ENERGY, PropertyValue::Float(e));
        }
    }
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);
    let mut t = Trace { n: vec![], edges: vec![], total_energy: vec![], vecs: vec![],
        active_steps: 0, wl_twtf: 0.0, wl_m: 0, eb_twtf: 0.0, eb_m: 0 };
    let mut wl_live: HashMap<u64, u64> = HashMap::new();
    let mut eb_live: HashMap<u64, u64> = HashMap::new();
    let mut wl_src: HashMap<u64, u64> = HashMap::new();
    let mut eb_src: HashMap<u64, u64> = HashMap::new();
    let mut wl_prev: Option<HashMap<NodeId, u64>> = None;
    let mut eb_prev: Option<HashMap<NodeId, u64>> = None;
    let mut prev_e = energies(&store);

    for _ in 1..=steps {
        engine.step(&mut store);
        let snap = store.snapshot();
        let e = energies_snap(&snap);
        if e != prev_e { t.active_steps += 1; }
        prev_e = e.clone();
        t.n.push(snap.nodes().count());
        t.edges.push(snap.edge_count());
        t.total_energy.push(e.iter().map(|(_, x)| *x).sum());
        t.vecs.push(e);

        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut wl_now: HashMap<NodeId, u64> = HashMap::new();
        let mut eb_now: HashMap<NodeId, u64> = HashMap::new();
        let mut wl_seen: HashSet<u64> = HashSet::new();
        let mut eb_seen: HashSet<u64> = HashSet::new();
        for (id, d) in snap.nodes() {
            let h = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            wl_now.insert(id, h);
            wl_seen.insert(h);
            let en = match d.get_property(ENERGY) { Some(PropertyValue::Float(x)) => x, _ => 0.0 };
            let b = energy_band_class(d.label.0 as i64, en);
            eb_now.insert(id, b);
            eb_seen.insert(b);
        }
        for h in &wl_seen { *wl_live.entry(*h).or_insert(0) += 1; }
        for h in &eb_seen { *eb_live.entry(*h).or_insert(0) += 1; }
        if let Some(p) = &wl_prev { for (_, &hp) in p { *wl_src.entry(hp).or_insert(0) += 1; } }
        if let Some(p) = &eb_prev { for (_, &hp) in p { *eb_src.entry(hp).or_insert(0) += 1; } }
        wl_prev = Some(wl_now);
        eb_prev = Some(eb_now);
    }
    let (wm, wt) = twtf(&wl_live, &wl_src);
    let (em, et) = twtf(&eb_live, &eb_src);
    t.wl_m = wm; t.wl_twtf = wt; t.eb_m = em; t.eb_twtf = et;
    t
}

fn spread(v: &[(NodeId, f64)]) -> (usize, f64, f64) {
    let n_pos = v.iter().filter(|(_, e)| *e > 0.0).count();
    let mx = v.iter().map(|(_, e)| *e).fold(f64::MIN, f64::max);
    let mn = v.iter().map(|(_, e)| *e).fold(f64::MAX, f64::min);
    (n_pos, mn, mx)
}

#[test]
fn l1_2_diffusion_substrate_bare_characterisation() {
    let seed = 12345;
    eprintln!("\n================ V7 / L1.2 — `diffuse` CHARACTERISED BARE AS PHYSICS (seed {seed}) ================");

    // ---------- (1) operating point, bare: the no-back-reaction / firewall check ----------
    let steps = 2_000u64;
    let cfg = config(seed, 5.0, steps);
    let without = run(&cfg, build(&cfg, false, true), steps, None);
    let with = run(&cfg, build(&cfg, true, true), steps, None);
    let identical = without.vecs == with.vecs;

    eprintln!("\n(1) OPERATING POINT, BARE (5.0 start, influx 0.34, ZERO carriers, {steps} steps)");
    eprintln!("  variant        |  n  | edges | total energy (first -> last) | steps with any energy change | WL M/TWTF | eb M/TWTF");
    eprintln!("  ---------------+-----+-------+------------------------------+------------------------------+-----------+----------");
    for (name, t) in [("static (no diffuse)", &without), ("static + diffuse", &with)] {
        eprintln!("  {:<14} | {:>3} | {:>5} | {:>12.2} -> {:>10.2} | {:>28} | {:>3} / {:.3} | {:>3} / {:.3}",
            name, t.n[t.n.len()-1], t.edges[t.edges.len()-1],
            t.total_energy[0], t.total_energy[t.total_energy.len()-1], t.active_steps,
            t.wl_m, t.wl_twtf, t.eb_m, t.eb_twtf);
    }
    eprintln!("  => per-step energy vectors identical with vs without `diffuse`: {identical}");
    if identical {
        eprintln!("     FIREWALL HOLDS: at the operating point the rule NEVER fires (no node is ever below the");
        eprintln!("     depletion threshold, because every node starts at 5.0 and influx drives it up). Adding");
        eprintln!("     `diffuse` perturbs bare substrate physics NOT AT ALL. Exactly as pre-declared in L1.1.");
    } else {
        eprintln!("     UNEXPECTED: `diffuse` perturbs the bare substrate at the operating point — the L1.1");
        eprintln!("     prediction is REFUTED and the rule must be re-examined before biology uses it.");
    }

    // ---------- (2) gradient fixture: the rule actually exercised ----------
    let gsteps = 200u64;
    let gcfg = config(seed, 0.0, gsteps);
    let grad = run(&gcfg, build(&gcfg, true, false), gsteps, Some(20.0));
    let gnull = run(&gcfg, build(&gcfg, false, false), gsteps, Some(20.0));
    let e0 = grad.total_energy[0];
    let e_end = grad.total_energy[grad.total_energy.len() - 1];
    let (n_pos0, _, _) = spread(&grad.vecs[0]);
    let (n_pos, mn, mx) = spread(&grad.vecs[grad.vecs.len() - 1]);
    let drift = (e_end - 20.0).abs();

    eprintln!("\n(2) GRADIENT FIXTURE (one node loaded 20.0, rest 0.0, influx REMOVED, {gsteps} steps)");
    eprintln!("  variant        |  n  | edges | total energy (step1 -> end) | conservation drift vs 20.0 | nodes>0 (step1 -> end) | min/max energy | active steps");
    eprintln!("  ---------------+-----+-------+-----------------------------+----------------------------+------------------------+----------------+-------------");
    eprintln!("  {:<14} | {:>3} | {:>5} | {:>11.4} -> {:>10.4} | {:>26.2e} | {:>10} -> {:>10} | {:>5.1} / {:>5.1} | {:>12}",
        "diffuse", grad.n[grad.n.len()-1], grad.edges[grad.edges.len()-1], e0, e_end, drift,
        n_pos0, n_pos, mn, mx, grad.active_steps);
    eprintln!("  {:<14} | {:>3} | {:>5} | {:>11.4} -> {:>10.4} | {:>26} | {:>10} -> {:>10} | {:>5.1} / {:>5.1} | {:>12}",
        "no diffuse", gnull.n[gnull.n.len()-1], gnull.edges[gnull.edges.len()-1],
        gnull.total_energy[0], gnull.total_energy[gnull.total_energy.len()-1], "-",
        spread(&gnull.vecs[0]).0, spread(&gnull.vecs[gnull.vecs.len()-1]).0,
        spread(&gnull.vecs[gnull.vecs.len()-1]).1, spread(&gnull.vecs[gnull.vecs.len()-1]).2,
        gnull.active_steps);
    eprintln!("  partition stationarity under transport: WL M {} TWTF {:.3} | energy-band M {} TWTF {:.3}",
        grad.wl_m, grad.wl_twtf, grad.eb_m, grad.eb_twtf);

    let n_flat = grad.n.iter().all(|&x| x == grad.n[0]);
    let e_flat = grad.edges.iter().all(|&x| x == grad.edges[0]);
    eprintln!("\n  n(t) flat: {n_flat} (n = {});  edge count flat: {e_flat} (edges = {})", grad.n[0], grad.edges[0]);
    eprintln!("  CONSERVATION: total energy drift from the loaded 20.0 = {drift:.3e} (the rule is neither source nor sink)");
    eprintln!("  ONE-DIRECTIONAL / SELF-LIMITING: transport moves quanta only INTO nodes below the depletion");
    eprintln!("  threshold and stops as soon as they are above it, so it tops a depleted node up rather than");
    eprintln!("  equalising the substrate. That is the behaviour arm D needs and it cannot run away.");
    eprintln!("=========================================================================================\n");

    // Firewall assertions — these gate L1.3.
    assert!(identical, "diffuse perturbs the bare substrate at the operating point (firewall violated)");
    assert!(drift < 1e-9, "diffuse is not energy-conserving (drift {drift:.3e})");
    assert!(n_flat && e_flat, "diffuse changed the node/edge set — it must be transport only");
    assert!(grad.active_steps > 0, "gradient fixture never exercised the rule");
}
