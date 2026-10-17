//! S4 — the first full composed ecology run.
//!
//! Per the plan, S4 EXPECTS explosion or extinction and does NOT tune. Its job
//! is to OBSERVE the composed run and DIAGNOSE which energy term dominates and
//! which regime emerges, then check the pre-registered prediction
//! (`docs/experiment5_preregistration.md` §1): directional collapse of
//! `replication_threshold` toward its floor via the node-funded degenerate route.
//!
//! This file adds no new mechanism and changes no rule. It attaches a read-only
//! observer that records, over the run:
//!   * carrier population trajectory,
//!   * the live distribution (mean/min/max) of every genome trait, and
//!   * the cumulative + windowed magnitude of each energy term (source vs sink).
//!
//! The scope §8.1 forbidden-response column governs any follow-up: no hand-
//! injected selective force to hide a degeneracy. This test only measures.
#![cfg(feature = "alife")]

use hg_core::{GraphSnapshot, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const ENERGY: u32 = 1;
const CARRIER: hg_core::Label = hg_core::Label(100);

/// Active genome traits, in the order printed. (harvest, repl_thr, maint, satiation)
const TRAIT_KEYS: [u32; 4] = [10, 11, 12, 13];
const TRAIT_NAMES: [&str; 4] = ["harvest_rate", "repl_threshold", "maint_cost", "satiation"];

/// A live-population summary sampled every `sample_every` steps.
#[derive(Clone, Debug)]
struct Sample {
    step: u64,
    carriers: usize,
    /// Distinct substrate nodes occupied by at least one carrier. If this stays
    /// at 1 while carriers > 1, the population is confined to the founding site.
    occupied_sites: usize,
    substrate_energy: f64,
    carrier_energy: f64,
    /// (mean, min, max) per active trait, over the live carrier population.
    trait_stats: [(f64, f64, f64); 4],
}

#[derive(Default)]
struct S4Recorder {
    sample_every: u64,
    step: u64,
    samples: Vec<Sample>,
    // Cumulative ledger terms (source positive, sink positive-magnitude).
    influx: f64,
    maintenance: f64,
    birth_entropy: f64,
    death_recycle: f64,
    births: u64,
    deaths: u64,
    // Windowed (per-sample-interval) flows, to read the *steady-state* balance
    // rather than the transient. Each entry sums the interval since the last.
    window: LedgerWindow,
    windows: Vec<LedgerWindow>,
    // Ledger self-check: cumulative modeled energy vs measured (must stay ~0).
    modeled_energy: f64,
    max_divergence: f64,
    // Death residual: energy removed when a carrier is deleted at r <= 0.
    prev_carrier_energy: std::collections::HashMap<hg_core::NodeId, f64>,
    death_residual: f64,
}

#[derive(Default, Clone, Copy, Debug)]
struct LedgerWindow {
    influx: f64,
    maintenance: f64,
    birth_entropy: f64,
    death_recycle: f64,
    death_residual: f64,
}

fn trait_stats(snap: &GraphSnapshot<'_>, key: u32) -> (f64, f64, f64) {
    let mut sum = 0.0;
    let mut n = 0u64;
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for (_, d) in snap.nodes().filter(|(_, d)| d.label == CARRIER) {
        if let Some(PropertyValue::Float(v)) = d.get_property(key) {
            sum += v;
            n += 1;
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    if n == 0 {
        (f64::NAN, f64::NAN, f64::NAN)
    } else {
        (sum / n as f64, lo, hi)
    }
}

impl Observer for S4Recorder {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _stats: &StepStats, events: &[RewriteEvent]) {
        self.step += 1;
        for e in events {
            match e.rule_name.as_str() {
                "influx" => {
                    self.influx += 2.0;
                    self.window.influx += 2.0;
                    self.modeled_energy += 2.0;
                }
                "bootstrap_seed" => {
                    self.modeled_energy += 10.0;
                }
                "metabolise" => {
                    self.maintenance += 0.5;
                    self.window.maintenance += 0.5;
                    self.modeled_energy -= 0.5;
                }
                "replicate" => {
                    self.birth_entropy += 2.0;
                    self.window.birth_entropy += 2.0;
                    self.births += 1;
                    self.modeled_energy -= 2.0;
                }
                "die_and_recycle" => {
                    let mut r = 0.0;
                    for n in &e.nodes_deleted {
                        if let Some(&pe) = self.prev_carrier_energy.get(n) {
                            r += pe;
                        }
                    }
                    self.death_recycle += 8.0;
                    self.death_residual += -r;
                    self.window.death_recycle += 8.0;
                    self.window.death_residual += -r;
                    self.deaths += 1;
                    self.modeled_energy += 8.0 - r;
                }
                _ => {} // harvest: internal transfer
            }
        }

        // Ledger self-check every step.
        let mut substrate_e = 0.0;
        let mut carrier_e = 0.0;
        for (_, d) in snap.nodes() {
            if let Some(PropertyValue::Float(v)) = d.get_property(ENERGY) {
                if d.label == CARRIER {
                    carrier_e += v;
                } else {
                    substrate_e += v;
                }
            }
        }
        let measured = substrate_e + carrier_e;
        let div = (self.modeled_energy - measured).abs();
        if div > self.max_divergence {
            self.max_divergence = div;
        }

        // Refresh the death-residual lookup for next step.
        self.prev_carrier_energy = snap
            .nodes()
            .filter(|(_, d)| d.label == CARRIER)
            .filter_map(|(id, d)| match d.get_property(ENERGY) {
                Some(PropertyValue::Float(v)) => Some((id, v)),
                _ => None,
            })
            .collect();

        if self.step % self.sample_every == 0 {
            let carrier_ids: std::collections::HashSet<hg_core::NodeId> = snap
                .nodes()
                .filter(|(_, d)| d.label == CARRIER)
                .map(|(id, _)| id)
                .collect();
            let carriers = carrier_ids.len();
            // Distinct non-carrier nodes sharing an edge with some carrier.
            let mut occupied: std::collections::HashSet<hg_core::NodeId> = Default::default();
            for (_, e) in snap.edges() {
                if e.nodes.iter().any(|n| carrier_ids.contains(n)) {
                    for n in e.nodes.iter() {
                        if !carrier_ids.contains(n) {
                            occupied.insert(*n);
                        }
                    }
                }
            }
            let mut ts = [(0.0, 0.0, 0.0); 4];
            for (i, &k) in TRAIT_KEYS.iter().enumerate() {
                ts[i] = trait_stats(snap, k);
            }
            self.samples.push(Sample {
                step: self.step,
                carriers,
                occupied_sites: occupied.len(),
                substrate_energy: substrate_e,
                carrier_energy: carrier_e,
                trait_stats: ts,
            });
            self.windows.push(std::mem::take(&mut self.window));
        }
    }
}

fn ecology_config(steps: u64, influx_energy: f64, width: u32, height: u32) -> SimConfig {
    ecology_config_seed(steps, influx_energy, width, height, 12345)
}

fn ecology_config_seed(steps: u64, influx_energy: f64, width: u32, height: u32, seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "s4_ecology"
run_id = "s4_ecology"
seed = {seed}
[rules]
source = "Inline"
inline_toml = "rules = []"
[initial]
condition_type = "PureCrystal"
width = {width}
height = {height}
arity = 3
[engine]
step_mode = "MaxParallel"
max_steps = {steps}
max_pairings_per_step = 1000000
[evaluator]
enabled = false
[causal]
enabled = false
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = {influx_energy}
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn all_rules() -> Vec<hg_rules::CompiledRule> {
    compile_rules(&parse_rules_toml(hg_alife::alife_base_toml()).unwrap()).unwrap()
}

fn run_ecology(config: &SimConfig, sample_every: u64) -> Arc<Mutex<S4Recorder>> {
    let store = hg_sim_core::runner::build_initial_store(config);
    // Seed the ledger self-check with E(0): the initial substrate energy the
    // generator wrote. Without this the divergence is a constant offset = E(0),
    // not a real accounting error.
    let e0: f64 = store
        .snapshot()
        .nodes()
        .filter_map(|(_, d)| match d.get_property(ENERGY) {
            Some(PropertyValue::Float(v)) => Some(v),
            _ => None,
        })
        .sum();
    let mut store = store;
    let mut engine = Engine::new(all_rules(), StepMode::MaxParallel, config.seed, 1_000_000);
    let rec = Arc::new(Mutex::new(S4Recorder {
        sample_every,
        modeled_energy: e0,
        ..Default::default()
    }));
    engine.add_observer(Box::new(S4RecorderHandle { inner: rec.clone() }));
    let steps = config.engine.max_steps;
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    rec
}

/// The engine takes ownership of the observer box; this handle forwards to the
/// shared recorder so the test can read it after the run.
struct S4RecorderHandle {
    inner: Arc<Mutex<S4Recorder>>,
}
impl Observer for S4RecorderHandle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, stats: &StepStats, events: &[RewriteEvent]) {
        self.inner.lock().unwrap().on_step_end(snap, stats, events);
    }
}

/// The headline S4 diagnostic. Runs the full composed ecology and prints the
/// trajectory, the dominant energy term, and the pre-registered trait test.
#[test]
fn s4_first_composed_ecology_run() {
    let steps = 30_000u64;
    let sample_every = 1_000u64;
    let config = ecology_config(steps, 5.0, 6, 6);
    let rec = run_ecology(&config, sample_every);
    let r = rec.lock().unwrap();

    eprintln!("\n================ S4: first composed ecology run ================");
    eprintln!("substrate: 6x6 crystal (36 sites), {steps} steps, seed {}", config.seed);
    eprintln!(
        "\n  step | carriers | sites | subs_E  | carr_E  | repl_thr(mean/min/max)"
    );
    eprintln!("  -----+----------+-------+---------+---------+-----------------------");
    for s in r.samples.iter() {
        let (m, lo, hi) = s.trait_stats[1]; // replication_threshold
        eprintln!(
            "  {:>5}| {:>8} | {:>5} | {:>7.1} | {:>7.1} | {:>6.2} / {:>6.2} / {:>6.2}",
            s.step, s.carriers, s.occupied_sites, s.substrate_energy, s.carrier_energy, m, lo, hi
        );
    }

    // --- Energy-term dominance (cumulative) ---
    eprintln!("\n  --- cumulative energy terms over {steps} steps ---");
    eprintln!("    influx        (SOURCE): {:>12.1}", r.influx);
    eprintln!("    death recycle (source): {:>12.1}  ({} deaths)", r.death_recycle, r.deaths);
    eprintln!("    death residual(source): {:>12.4}", r.death_residual);
    eprintln!("    maintenance   (sink):   {:>12.1}", r.maintenance);
    eprintln!("    birth entropy (sink):   {:>12.1}  ({} births)", r.birth_entropy, r.births);
    let sources = r.influx + r.death_recycle + r.death_residual;
    let sinks = r.maintenance + r.birth_entropy;
    eprintln!("    Σ sources: {:.1}   Σ sinks: {:.1}   net: {:+.1}", sources, sinks, sources - sinks);
    eprintln!("    ledger max divergence: {:.3e}", r.max_divergence);

    // Which single term is largest by magnitude?
    let terms = [
        ("influx", r.influx),
        ("death_recycle", r.death_recycle),
        ("maintenance", r.maintenance),
        ("birth_entropy", r.birth_entropy),
    ];
    let dominant = terms.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
    eprintln!("    DOMINANT term: {} ({:.1})", dominant.0, dominant.1);

    // --- Steady-state windowed flows (last window) ---
    if let Some(w) = r.windows.last() {
        eprintln!("\n  --- last-window (final {sample_every} steps) flows ---");
        eprintln!(
            "    influx {:.1} | maint {:.1} | birth {:.1} | recycle {:.1} | residual {:.3}",
            w.influx, w.maintenance, w.birth_entropy, w.death_recycle, w.death_residual
        );
        let realized_influx_per_step = w.influx / 2.0 / sample_every as f64;
        eprintln!("    realized influx firings/step (steady state): {:.2}", realized_influx_per_step);
    }

    // --- Final trait distribution (all four) ---
    if let Some(last) = r.samples.last() {
        eprintln!("\n  --- final trait distribution (live carriers) ---");
        for (i, name) in TRAIT_NAMES.iter().enumerate() {
            let (m, lo, hi) = last.trait_stats[i];
            eprintln!("    {:>16}: mean {:>7.3}  min {:>7.3}  max {:>7.3}", name, m, lo, hi);
        }
    }

    // --- Regime diagnosis ---
    let final_carriers = r.samples.last().map(|s| s.carriers).unwrap_or(0);
    let peak_carriers = r.samples.iter().map(|s| s.carriers).max().unwrap_or(0);
    let regime = if final_carriers == 0 {
        "EXTINCTION"
    } else if final_carriers >= peak_carriers && peak_carriers > 5 {
        "GROWTH/EXPLOSION (still rising at end)"
    } else {
        "BOUNDED (peaked then settled)"
    };
    eprintln!("\n  REGIME: {regime}  (peak {peak_carriers}, final {final_carriers})");

    // --- Pre-registered prediction test (§1): repl_threshold trend ---
    // Compare the first non-empty sample's mean repl_threshold to the last.
    let first_thr = r.samples.iter().find(|s| s.carriers > 0).map(|s| s.trait_stats[1].0);
    let last_thr = r.samples.iter().rev().find(|s| s.carriers > 0).map(|s| s.trait_stats[1].0);
    if let (Some(f), Some(l)) = (first_thr, last_thr) {
        let drift = l - f;
        eprintln!(
            "\n  PRE-REGISTERED (§1): repl_threshold {:.2} -> {:.2}  (drift {:+.2})",
            f, l, drift
        );
        eprintln!(
            "    prediction was: directional collapse toward the floor (drift < 0)."
        );
        eprintln!(
            "    OBSERVED: {}",
            if drift < -0.5 {
                "CONFIRMED — threshold collapsing"
            } else if drift > 0.5 {
                "REFUTED — threshold rising (report this)"
            } else {
                "INCONCLUSIVE at this horizon (near-static)"
            }
        );
    }
    eprintln!("================================================================\n");

    // --- Assertions: only accounting invariants, never the outcome. ---
    // S4 must not fail on explosion/extinction — those are legitimate findings.
    assert!(
        r.max_divergence < 1e-6,
        "ledger diverged by {:.3e} during the composed run — accounting is broken",
        r.max_divergence
    );
    // The run must at least have been a live economy at some point (bootstrap
    // fired, biology engaged); a dead run would mean the harness is misconfigured.
    assert!(peak_carriers > 0, "no carrier ever existed — bootstrap/seed misconfigured");
}

/// GATING DIAGNOSTIC (pre-registration §6), retained as a regression under the
/// birth-dispersal rule: a fertile lineage must chain reproduction repeatedly
/// when a fundable, unoccupied neighbour is available — confirming that the fix
/// for confinement did not introduce a per-lineage sterility.
///
/// Setup: founder on site A with an adjacent well-fed site B. Each step we delete
/// the child and RESET B (refuel, mark unoccupied), so the founder always has one
/// fresh neighbour to colonise. We trace founder + target energy and count births.
///
/// Interpretation (committed before running, §6):
///   * MANY births, recurring across the run  ⟹ the per-lineage economy is healthy.
///   * ONE (or zero) then a monotone approach to a fixed point below threshold ⟹
///     the economy is the bug; no dispersal rule rescues it.
#[test]
fn s4_gating_fertile_lineage_chains_reproduction() {
    use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId};
    use smallvec::SmallVec;

    let mut store = HypergraphStore::new();
    // Founder's own site A, and an adjacent target site B, joined by a lattice edge.
    let a = store.add_node(NodeData::new(Label(0), 0));
    let b = store.add_node(NodeData::new(Label(0), 0));
    for s in [a, b] {
        let d = store.get_node_mut(s).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(20.0));
        d.set_property(22, PropertyValue::Bool(false)); // occupied
    }
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b]))); // adjacency
    let founder = store.add_node(NodeData::new(CARRIER, 0));
    {
        let d = store.get_node_mut(founder).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(10.0));
        d.set_property(10, PropertyValue::Float(1.0)); // harvest_rate
        d.set_property(11, PropertyValue::Float(15.0)); // replication_threshold
        d.set_property(12, PropertyValue::Float(1.0)); // maintenance_cost
        d.set_property(13, PropertyValue::Float(20.0)); // satiation
    }
    store.get_node_mut(a).unwrap().set_property(22, PropertyValue::Bool(true)); // A occupied
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, founder])));

    let rules = {
        let raw: Vec<_> = parse_rules_toml(hg_alife::alife_base_toml())
            .unwrap()
            .into_iter()
            .filter(|r| matches!(r.name.as_str(), "harvest" | "metabolise" | "influx" | "replicate"))
            .collect();
        compile_rules(&raw).unwrap()
    };
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 7, 100000);

    let steps = 200u64;
    let mut births = 0u64;
    let mut carrier_trace: Vec<f64> = Vec::new();
    let read = |s: &HypergraphStore, id: NodeId| -> f64 {
        match s.get_node(id).and_then(|d| d.get_property(ENERGY)) {
            Some(PropertyValue::Float(v)) => v,
            _ => f64::NAN,
        }
    };

    for _ in 0..steps {
        engine.step(&mut store);
        // Delete every child (carrier != founder), then reset B as a fresh target.
        let children: Vec<NodeId> = store
            .nodes()
            .filter(|(id, d)| d.label == CARRIER && *id != founder)
            .map(|(id, _)| id)
            .collect();
        births += children.len() as u64;
        for c in children {
            store.remove_node(c);
        }
        let d = store.get_node_mut(b).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(20.0));
        d.set_property(22, PropertyValue::Bool(false));
        carrier_trace.push(read(&store, founder));
    }

    eprintln!("\n--- S4 gating: fertile lineage chains reproduction (dispersal) ---");
    eprintln!("births over {steps} steps: {births}");
    eprintln!("founder energy first10: {:?}", &carrier_trace[..10]);
    eprintln!("founder energy last10:  {:?}", &carrier_trace[carrier_trace.len() - 10..]);
    let thr = 15.0;
    let above = carrier_trace[steps as usize / 2..].iter().filter(|&&e| e >= thr).count();
    eprintln!("founder steps ≥ threshold ({thr}) in last half: {above}/{}", steps as usize / 2);

    assert!(
        births >= 5,
        "fertile lineage did NOT chain reproduction ({births} in {steps} steps) — the per-lineage \
         economy is the bug, not confinement"
    );
    assert!(above > 0, "founder never reproduction-ready in the run's second half");
}

/// Post-dispersal robustness: birth-dispersal FIXES the single-site confinement
/// S4 originally exposed. Across several seeds and a larger substrate, every
/// composed run now spreads the population across MANY distinct sites (one carrier
/// per site, occupied_sites == carriers), colonising a large fraction of the
/// substrate rather than freezing at the founding site. This is the regression
/// that confinement stays fixed.
#[test]
fn s4_dispersal_colonizes_the_substrate() {
    for seed in [1u64, 7, 12345, 99999] {
        let config = ecology_config_seed(8_000, 5.0, 8, 8, seed); // 64 sites
        let rec = run_ecology(&config, 8_000);
        let r = rec.lock().unwrap();
        let last = r.samples.last().expect("one sample at the final step");
        eprintln!(
            "seed {seed}: carriers={} occupied_sites={} births={} deaths={}",
            last.carriers, last.occupied_sites, r.births, r.deaths
        );
        // One carrier per site (dispersal never stacks two on one node).
        assert_eq!(
            last.occupied_sites, last.carriers,
            "seed {seed}: occupied_sites must equal carriers (one carrier per site)"
        );
        // The population escaped the founding site and colonised broadly.
        assert!(
            last.carriers >= 20,
            "seed {seed}: dispersal failed to colonise the substrate ({} carriers on 64 sites)",
            last.carriers
        );
        assert!(r.max_divergence < 1e-6, "seed {seed}: ledger drift {:.3e}", r.max_divergence);
    }
}
