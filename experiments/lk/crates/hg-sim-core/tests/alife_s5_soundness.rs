//! S5 INSTRUMENT SOUNDNESS. The lineage/ecology numbers are only trustworthy if
//! the instrument (a) doesn't perturb the run, (b) reconstructs the live
//! population from events exactly, (c) survives NodeId reuse under turnover, and
//! (d) conserves population. Plus the measurements S5 makes answerable on the
//! existing frozen run.
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::{GraphSnapshot, HypergraphStore, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, EffectOpDecl, StepMode};
use hg_sim_core::SimConfig;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);
const ENERGY: u32 = 1;
const REPL_THR: u32 = 11;
const SATIATION: u32 = 13;

fn cfg(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "s5"
run_id = "s5"
seed = 12345
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
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 5.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn all_rules() -> Vec<CompiledRule> {
    compile_rules(&parse_rules_toml(hg_alife::alife_base_toml()).unwrap()).unwrap()
}

/// Rules for the turnover regime (bootstrap threshold + influx patched).
fn turnover_rules(influx: f64, thr: f64) -> Vec<CompiledRule> {
    let mut raw = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    for r in raw.iter_mut() {
        if r.name == "influx" {
            for eff in &mut r.rhs_property_effects {
                if eff.key == 1 {
                    if let EffectOpDecl::Add { value } = &mut eff.op {
                        *value = PropertyValue::Float(influx);
                    }
                }
            }
        }
        if r.name == "bootstrap_seed" {
            for eff in &mut r.rhs_property_effects {
                if eff.var == "carrier" && eff.key == 11 {
                    if let EffectOpDecl::Set { value } = &mut eff.op {
                        *value = PropertyValue::Float(thr);
                    }
                }
            }
        }
    }
    compile_rules(&raw).unwrap()
}

// ---------------------------------------------------------------------------
// (1) Read-only, provably: the observer must not perturb the run.
// ---------------------------------------------------------------------------

/// A trace observer present in BOTH runs so the comparison is apples-to-apples;
/// it too must be read-only.
#[derive(Default, Debug)]
struct EnergyTrace {
    bits: Vec<u64>,
}
struct TraceHandle {
    inner: Arc<Mutex<EnergyTrace>>,
}
impl Observer for TraceHandle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _s: &StepStats, _e: &[RewriteEvent]) {
        let e: f64 = snap
            .nodes()
            .filter_map(|(_, d)| match d.get_property(ENERGY) {
                Some(PropertyValue::Float(v)) => Some(v),
                _ => None,
            })
            .sum();
        self.inner.lock().unwrap().bits.push(e.to_bits());
    }
}
struct AlifeHandle {
    inner: Arc<Mutex<AlifeObserver>>,
}
impl Observer for AlifeHandle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, e: &[RewriteEvent]) {
        self.inner.lock().unwrap().on_step_end(snap, s, e);
    }
}

#[test]
fn observer_does_not_perturb_the_run() {
    let steps = 3_000u64;
    let run = |with_alife: bool| -> (Vec<u64>, u64) {
        let config = cfg(steps);
        let mut store = hg_sim_core::runner::build_initial_store(&config);
        let mut engine = Engine::new(all_rules(), StepMode::MaxParallel, config.seed, 100000);
        let trace = Arc::new(Mutex::new(EnergyTrace::default()));
        engine.add_observer(Box::new(TraceHandle { inner: trace.clone() }));
        if with_alife {
            let obs = Arc::new(Mutex::new(AlifeObserver::new(500)));
            engine.add_observer(Box::new(AlifeHandle { inner: obs }));
        }
        for _ in 0..steps {
            engine.step(&mut store);
        }
        drop(engine);
        let bits = Arc::try_unwrap(trace).unwrap().into_inner().unwrap().bits;
        (bits, store.canonical_hash())
    };
    let (trace_with, hash_with) = run(true);
    let (trace_without, hash_without) = run(false);
    assert_eq!(trace_with, trace_without, "energy trace changed when the observer was attached");
    assert_eq!(hash_with, hash_without, "canonical_hash changed — the observer perturbed the run");
}

// ---------------------------------------------------------------------------
// (2)(3)(4) Events-alone reconstruction, NodeId-reuse survival, and population
// conservation — one per-step invariant observer, run through BOTH the
// saturation run and the high-turnover run (which forces NodeId reuse).
// ---------------------------------------------------------------------------

#[derive(Default, Debug)]
struct Soundness {
    live: HashSet<NodeId>, // reconstructed from events
    prev_count: usize,
    step: u64,
    set_mismatches: u64,
    conservation_violations: u64,
    max_live: usize,
    total_deaths: u64,
}
struct SoundnessHandle {
    inner: Arc<Mutex<Soundness>>,
}
impl Observer for SoundnessHandle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _s: &StepStats, events: &[RewriteEvent]) {
        let mut s = self.inner.lock().unwrap();
        s.step += 1;
        let mut adds = 0i64;
        let mut dels = 0i64;
        for e in events {
            match e.rule_name.as_str() {
                "bootstrap_seed" | "replicate" => {
                    if let Some(child) = e
                        .nodes_created
                        .iter()
                        .copied()
                        .find(|&n| snap.get_node(n).map(|d| d.label == CARRIER).unwrap_or(false))
                    {
                        // A freshly created carrier id must NOT already be live
                        // (that would be a NodeId-reuse collision).
                        assert!(s.live.insert(child), "reused a live NodeId for a new carrier @{}", s.step);
                        adds += 1;
                    }
                }
                "die_and_recycle" => {
                    for n in &e.nodes_deleted {
                        if s.live.remove(n) {
                            dels += 1;
                            s.total_deaths += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        // (2) events-alone live set == graph ground truth.
        let graph = carrier_set_snap(snap);
        if s.live != graph {
            s.set_mismatches += 1;
        }
        // (4) conservation: count(t) == count(t-1) + adds - dels.
        let count = graph.len();
        if count as i64 != s.prev_count as i64 + adds - dels {
            s.conservation_violations += 1;
        }
        s.prev_count = count;
        s.max_live = s.max_live.max(count);
    }
}
fn carrier_set_snap(snap: &GraphSnapshot<'_>) -> HashSet<NodeId> {
    snap.nodes().filter(|(_, d)| d.label == CARRIER).map(|(id, _)| id).collect()
}

fn run_soundness(rules: Vec<CompiledRule>, steps: u64) -> Soundness {
    let config = cfg(steps);
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, config.seed, 100000);
    let s = Arc::new(Mutex::new(Soundness::default()));
    engine.add_observer(Box::new(SoundnessHandle { inner: s.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    Arc::try_unwrap(s).unwrap().into_inner().unwrap()
}

#[test]
fn events_reconstruct_live_population_on_saturation_run() {
    let s = run_soundness(all_rules(), 4_000);
    assert_eq!(s.set_mismatches, 0, "live-set-from-events diverged from the graph");
    assert_eq!(s.conservation_violations, 0, "population not conserved (missed/double event)");
}

#[test]
fn lineage_survives_nodeid_reuse_under_heavy_turnover() {
    // The turnover regime has thousands of deaths+births ⟹ SlotMap slots recycle.
    // If a reused id collided, the insert-assert in the observer would fire, and
    // the live set would diverge. Neither may happen.
    let s = run_soundness(turnover_rules(0.34, 3.0), 8_000);
    assert!(s.total_deaths > 500, "turnover regime must actually churn (got {} deaths)", s.total_deaths);
    assert_eq!(s.set_mismatches, 0, "NodeId reuse corrupted the live set under turnover");
    assert_eq!(s.conservation_violations, 0, "population not conserved under turnover");
    eprintln!("turnover soundness: {} deaths, max_live {}, 0 mismatches, 0 violations", s.total_deaths, s.max_live);
}

/// (3) direct: SlotMap keys are versioned, so a removed id never equals a later
/// id in the same slot, and an AHashMap keyed on it cannot confuse them.
#[test]
fn nodeid_is_versioned_so_reuse_cannot_alias() {
    let mut store = HypergraphStore::new();
    let a = store.add_node(hg_core::NodeData::new(CARRIER, 0));
    let mut map = std::collections::HashMap::new();
    map.insert(a, "old");
    store.remove_node(a);
    // Force many allocations so a slot is reused.
    let mut reused_slot_seen = false;
    for _ in 0..8 {
        let b = store.add_node(hg_core::NodeData::new(CARRIER, 0));
        assert_ne!(a, b, "a removed NodeId must never equal a newly minted one");
        assert!(map.get(&b).is_none(), "new NodeId must not alias the dead one's map entry");
        if format!("{a:?}").split('v').next() == format!("{b:?}").split('v').next() {
            reused_slot_seen = true; // same slot index, different version
        }
    }
    let _ = reused_slot_seen; // informational; the assertions above are the gate
}

// ---------------------------------------------------------------------------
// (5) non_trivial_replicator_count — the negative control + the honest scope.
// ---------------------------------------------------------------------------

#[test]
fn non_trivial_count_separates_sustained_from_one_off() {
    use hg_alife::genome::Genome;
    use hg_alife::LineageTree;
    let g = Genome { harvest_rate: Some(1.0), replication_threshold: Some(3.0), maintenance_cost: Some(1.0), satiation: Some(20.0) };
    let ids: Vec<NodeId> = {
        let mut s = HypergraphStore::new();
        (0..4).map(|_| s.add_node(hg_core::NodeData::new(CARRIER, 0))).collect()
    };
    let mut t = LineageTree::new();
    // Founder → one child that never reproduces: NOT a sustained (non-trivial)
    // replicator at depth ≥ 2.
    t.record_birth(ids[0], None, g, 1);
    t.record_birth(ids[1], Some(ids[0]), g, 2);
    assert_eq!(t.non_trivial_replicator_count(2), 0, "a one-off birth is not sustained self-copying");
    // Extend to a depth-3 chain ⟹ founder and its child both qualify.
    t.record_birth(ids[2], Some(ids[1]), g, 3);
    t.record_birth(ids[3], Some(ids[2]), g, 4);
    assert_eq!(t.non_trivial_replicator_count(2), 2, "sustained lineages are counted");
    // A bare substrate (no carriers, no births) is 0 by construction.
    assert_eq!(LineageTree::new().non_trivial_replicator_count(2), 0);
}

// ---------------------------------------------------------------------------
// (6)-(9) Measurements S5 makes answerable on the frozen saturation run.
// ---------------------------------------------------------------------------

#[test]
fn frozen_run_measurements_threshold_satiation_selection_clades() {
    let steps = 8_000u64;
    let config = cfg(steps);
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(all_rules(), StepMode::MaxParallel, config.seed, 100000);
    let obs = Arc::new(Mutex::new(AlifeObserver::new(1_000)));
    engine.add_observer(Box::new(AlifeHandle { inner: obs.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    let o = obs.lock().unwrap();
    let lin = o.lineage();
    let w = o.windows();
    let first = w.first().unwrap();
    let last = w.last().unwrap();

    // Per-carrier threshold/satiation from the final store.
    let read = |key: u32| -> Vec<f64> {
        store
            .nodes()
            .filter(|(_, d)| d.label == CARRIER)
            .filter_map(|(_, d)| match d.get_property(key) {
                Some(PropertyValue::Float(v)) => Some(v),
                _ => None,
            })
            .collect()
    };
    let thr = read(REPL_THR);
    let sat = read(SATIATION);
    let sat_min = sat.iter().cloned().fold(f64::INFINITY, f64::min);
    let sat_max = sat.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let sterile = thr.iter().zip(&sat).filter(|(t, s)| *t > *s).count();

    eprintln!("\n===== S5 frozen-run measurements =====");
    // (6) threshold vs satiation — the structural-impossibility check.
    eprintln!("(6) repl_threshold mean {:.3} vs satiation mean {:.3}; sterile (thr>sat): {}/{}",
        last.trait_mean[1], last.trait_mean[3], sterile, thr.len());
    // (7) satiation spread.
    eprintln!("(7) satiation min {:.3} max {:.3} → max/min {:.3} (vs 1.47 desync budget)",
        sat_min, sat_max, sat_max / sat_min);
    // (8) selection: seed → frozen shift.
    eprintln!("(8) repl_threshold seed 15.0 → frozen {:.3} ({:+.3}); satiation 20.0 → {:.3} ({:+.3})",
        last.trait_mean[1], last.trait_mean[1] - 15.0, last.trait_mean[3], last.trait_mean[3] - 20.0);
    eprintln!("    (first window {:.3} vs last {:.3} — frozen after the colonisation wave)",
        first.trait_mean[1], last.trait_mean[1]);
    // (9) clade structure + newick on real data.
    eprintln!("(9) clade_count {} max_generation {} survival_curve {:?}",
        lin.clade_count(), lin.max_generation(), lin.survival_curve());
    let nwk = lin.export_newick();
    eprintln!("    newick length {} chars, balanced+terminated: {}", nwk.len(),
        nwk.ends_with(';') && nwk.matches('(').count() == nwk.matches(')').count());
    eprintln!("======================================\n");

    // Assertions: the plan-changer (6) is unambiguous; the rest are reported.
    assert_eq!(sterile, 0, "no carrier may have repl_threshold > its own satiation (else genome is misconfigured)");
    assert!(last.trait_mean[1] < 20.0, "threshold sits below satiation — reproduction reachable");
    // (8) directional signal exists (threshold drifted DOWN from the seed 15).
    assert!(last.trait_mean[1] < 15.0, "threshold drifted toward its floor during the wave (degenerate direction)");
    assert_eq!(lin.clade_count(), 1, "single bootstrap founder");
}
