//! H1.1 + H1.2 — the bounded-churn substrate, characterised as PHYSICS first
//! (firewall: zero carriers, no biology; the rule is NOT tuned to a biological
//! target). V4 H-track plan H1.
//!
//! Balanced-churn rule: match two adjacent label-0 nodes [keep, old], delete
//! `old` and create a fresh label-0 `fresh` attached to `keep`. Net node count 0
//! per firing (delete 1 + create 1), so n(t) is flat BY CONSTRUCTION while node
//! identity turns over. Rate is controlled by age-gating: nodes carry an age
//! counter (key 21), and only nodes past `churn_age` are eligible to be `old`, so
//! identity half-life ~ churn_age — a substrate parameter, recorded, not tuned.
//!
//! This harness sweeps churn_age and reports the physics: n(t) flatness, identity
//! turnover rate, mean degree (connectivity survival), WL-class churn (why WL
//! fails), and the energy-band class stationarity (why the J-track partition
//! should survive). Gate (H1.2): n(t) flat with genuine identity turnover.
#![cfg(feature = "alife")]

use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, Label, NodeData, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use std::collections::{HashMap, HashSet};

const ENERGY: u32 = 1;
const AGE: u32 = 21;
const OCCUPIED: u32 = 22;
const WL_DEPTH: u8 = 2;
const R_PERSIST: u64 = 3;

/// Energy-gated balanced churn + influx. A node fills via influx to the cap, then
/// churn recycles it to a fresh empty node — so the churn rate is set by how fast
/// nodes fill (the influx rate), with NO separate aging rule (whose per-step
/// increment would conflict with influx on the same node). Guards are disjoint
/// (influx: energy < cap; churn: energy >= cap), so nothing competes. `influx` is
/// the recorded substrate parameter; the churn rule itself is untuned.
fn churn_rules(influx: f64) -> Vec<hg_rules::CompiledRule> {
    let toml = format!(
        r#"
# substrate energy source (influx): label-0, energy < cap(20), +rate.
[[rules]]
name = "influx"
priority = 0
[rules.lhs]
nodes = ["s"]
edges = []
[rules.lhs.node_labels]
s = 0
[[rules.lhs.node_property_guards]]
var = "s"
property = {ENERGY}
op = "lt"
value = 20.0
[rules.rhs]
nodes = ["s"]
edges = []
[[rules.rhs.property_effects]]
var = "s"
key = {ENERGY}
op = "add"
value = {influx}

# balanced churn (path-preserving): a FULL (energy>=cap), UNOCCUPIED `old` bridges
# `a` and `b`; delete `old`, mint a fresh empty label-0 node re-bridging `a`-`b`.
# net 0 nodes; keeps TWO edges so connectivity does not collapse to dust; energy
# cap gate means it never fires on the same node as influx (disjoint guards) and
# only recycles free substrate (occupied==false), never a carrier's site.
[[rules]]
name = "churn"
priority = 2
[rules.lhs]
nodes = ["a", "old", "b"]
edges = [["a", "old"], ["old", "b"]]
[rules.lhs.node_labels]
a = 0
old = 0
b = 0
[[rules.lhs.node_property_guards]]
var = "old"
property = {ENERGY}
op = "gte"
value = 20.0
[[rules.lhs.node_property_guards]]
var = "old"
property = {OCCUPIED}
op = "eq"
value = false
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "old", "b"]
[rules.rhs]
nodes = ["a", "fresh", "b"]
edges = [["a", "fresh"], ["fresh", "b"]]
[rules.rhs.node_labels]
fresh = 0
[[rules.rhs.property_effects]]
var = "fresh"
key = {ENERGY}
op = "set"
value = 0.0
[[rules.rhs.property_effects]]
var = "fresh"
key = {OCCUPIED}
op = "set"
value = false
"#
    );
    compile_rules(&parse_rules_toml(&toml).unwrap()).unwrap()
}

fn bare_crystal() -> HypergraphStore {
    let mut store = HypergraphStore::new();
    hg_core::generate_crystal(&mut store, 6, 6, 3, false, 1);
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    for id in ids {
        let d = store.get_node_mut(id).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(5.0));
        d.set_property(AGE, PropertyValue::Float(0.0));
        d.set_property(OCCUPIED, PropertyValue::Bool(false));
    }
    store
}

fn mean_degree(store: &HypergraphStore) -> f64 {
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    if ids.is_empty() { return 0.0; }
    let tot: usize = ids.iter().map(|&id| store.edges_incident_to(id).len()).sum();
    tot as f64 / ids.len() as f64
}

fn energy_band_class(energy: f64) -> u64 {
    (energy.floor().clamp(-1.0, 64.0) as i64 as u64) ^ 0x9E37_79B9
}

struct Physics {
    influx: f64,
    n_min: usize,
    n_max: usize,
    survivors_frac: f64,     // fraction of original 36 nodes still alive at end
    churn_per_step: f64,     // fresh nodes minted per step (identity turnover rate)
    mean_deg_start: f64,
    mean_deg_end: f64,
    wl_m: usize,             // distinct WL classes over the run (churns)
    wl_twtf: f64,
    eb_m: usize,             // distinct energy-band classes (should stay small)
    eb_twtf: f64,
    energy_spread: f64,      // max-min energy at end (heterogeneity, H1.3)
}

fn characterize(influx: f64, steps: u64) -> Physics {
    let rules = churn_rules(influx);
    let mut store = bare_crystal();
    let original: HashSet<NodeId> = store.nodes().map(|(id, _)| id).collect();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1_000_000);

    let mean_deg_start = mean_degree(&store);
    let (mut n_min, mut n_max) = (original.len(), original.len());
    let mut fresh_total = 0u64;
    let mut prev_ids: HashSet<NodeId> = original.clone();

    // stationarity accumulators for WL vs energy-band classes.
    let mut wl_live: HashMap<u64, u64> = HashMap::new();
    let mut eb_live: HashMap<u64, u64> = HashMap::new();
    let mut wl_src: HashMap<u64, u64> = HashMap::new();
    let mut eb_src: HashMap<u64, u64> = HashMap::new();
    let mut wl_prev: Option<HashMap<NodeId, u64>> = None;
    let mut eb_prev: Option<HashMap<NodeId, u64>> = None;
    const VAC: u64 = 0xE6_0000_0000_0000;
    wl_live.insert(VAC, u64::MAX);
    eb_live.insert(VAC, u64::MAX);

    for _ in 1..=steps {
        engine.step(&mut store);
        let now_ids: HashSet<NodeId> = store.nodes().map(|(id, _)| id).collect();
        let n = now_ids.len();
        n_min = n_min.min(n);
        n_max = n_max.max(n);
        fresh_total += now_ids.difference(&prev_ids).count() as u64;
        prev_ids = now_ids;

        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut wl_now = HashMap::new();
        let mut eb_now = HashMap::new();
        let mut wl_present = HashSet::new();
        let mut eb_present = HashSet::new();
        for (id, d) in snap.nodes() {
            let wh = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            wl_now.insert(id, wh);
            wl_present.insert(wh);
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let eb = energy_band_class(e);
            eb_now.insert(id, eb);
            eb_present.insert(eb);
        }
        for h in wl_present { *wl_live.entry(h).or_insert(0) += 1; }
        for h in eb_present { *eb_live.entry(h).or_insert(0) += 1; }
        if let Some(p) = &wl_prev {
            for (_, &hp) in p { *wl_src.entry(hp).or_insert(0) += 1; }
        }
        if let Some(p) = &eb_prev {
            for (_, &hp) in p { *eb_src.entry(hp).or_insert(0) += 1; }
        }
        wl_prev = Some(wl_now);
        eb_prev = Some(eb_now);
    }

    let survivors = store.nodes().filter(|(id, _)| original.contains(id)).count();
    let (mut emin, mut emax) = (f64::INFINITY, f64::NEG_INFINITY);
    for (_, d) in store.nodes() {
        if let Some(PropertyValue::Float(e)) = d.get_property(ENERGY) { emin = emin.min(e); emax = emax.max(e); }
    }
    let twtf = |live: &HashMap<u64,u64>, src: &HashMap<u64,u64>| -> f64 {
        let tot: u64 = src.values().sum();
        let tr: u64 = src.iter().filter(|(h,_)| *live.get(*h).unwrap_or(&0) < R_PERSIST).map(|(_,c)| *c).sum();
        if tot > 0 { tr as f64 / tot as f64 } else { 0.0 }
    };
    Physics {
        influx,
        n_min, n_max,
        survivors_frac: survivors as f64 / original.len() as f64,
        churn_per_step: fresh_total as f64 / steps as f64,
        mean_deg_start, mean_deg_end: mean_degree(&store),
        wl_m: wl_live.len() - 1, wl_twtf: twtf(&wl_live, &wl_src),
        eb_m: eb_live.len() - 1, eb_twtf: twtf(&eb_live, &eb_src),
        energy_spread: if emax >= emin { emax - emin } else { 0.0 },
    }
}

#[test]
fn h1_churn_bare_characterization() {
    let steps = 300u64;
    eprintln!("\n================ H1.2 — BARE CHURN SUBSTRATE CHARACTERIZATION (firewall: 0 carriers, {steps} steps) ================");
    eprintln!("  influx | n(min..max) | survivors | fresh/step | mean deg (start->end) | WL: M/TWTF     | e-band: M/TWTF | E-spread");
    eprintln!("  -------+-------------+-----------+------------+-----------------------+----------------+----------------+---------");
    for inf in [0.34, 1.0, 2.0] {
        let p = characterize(inf, steps);
        eprintln!(
            "  {:>6.2} | {:>4}..{:<5} | {:>8.2}% | {:>10.2} | {:>8.2} -> {:<8.2} | {:>4}/{:>7.3}  | {:>4}/{:>7.3}   | {:>7.2}",
            p.influx, p.n_min, p.n_max, p.survivors_frac * 100.0, p.churn_per_step,
            p.mean_deg_start, p.mean_deg_end, p.wl_m, p.wl_twtf, p.eb_m, p.eb_twtf, p.energy_spread
        );
    }
    eprintln!("=========================================================================================================================\n");
    eprintln!("H1.2 gate: n(t) flat (min==max, guaranteed net-zero) AND genuine identity turnover (survivors < 100%, fresh/step > 0).");
    eprintln!("Characterized: influx 0.34 (the biology regime) gives working churn; HIGH influx (>=1.0) synchronizes");
    eprintln!("all fills to cap on the same step and DEADLOCKS into a frozen all-full state (a characterized property).");
    eprintln!("Limitation: mean degree drops (crystal -> sparse chain ~1.3), a property of the path-preserving churn.");
    // n(t) must be exactly flat by construction (net-zero churn); the working
    // regime (influx 0.34, the biology's regime) must show genuine turnover.
    let p = characterize(0.34, steps);
    assert_eq!(p.n_min, p.n_max, "churn is not net-zero: n(t) ranged {}..{}", p.n_min, p.n_max);
    assert!(p.survivors_frac < 1.0, "no identity turnover: all original nodes survived");
    assert!(p.churn_per_step > 0.0, "no fresh nodes minted");
    assert!(p.eb_twtf < 0.5 && p.eb_m > 1, "energy-band partition not stable+resolved under churn");
    // Suppress unused imports used only in other build configs.
    let _ = (NodeData::new(Label(0), 0), AGE);
}
