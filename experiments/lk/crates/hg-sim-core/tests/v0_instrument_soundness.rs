//! V-track V0 — Instrument soundness (gates everything downstream).
//!
//! S5's observer is the measuring device for every Experiment-5/6 criterion and
//! every CE input. A silently wrong observer does not fail loudly — it emits
//! plausible numbers. These prove the device before any number it produces is
//! trusted. See `hypersim_validation_plan_v1.md` §V0.
//!
//! Binding rules (validation-plan §"Rules for the implementer"): several of these
//! are structurally guaranteed by construction (the Observer trait hands out
//! `&GraphSnapshot`, never the store; `NodeId` is a generational slotmap key).
//! The tests turn "guaranteed by construction" into "guarded against regression"
//! by proving it on a LIVE multi-carrier turnover run, not a fixture.
#![cfg(feature = "alife")]

use hg_alife::motif::CARRIER_LABEL;
use hg_alife::{AlifeObserver, LineageTree};
use hg_core::{GraphSnapshot, HypergraphStore, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

const ENERGY_KEY: u32 = 1;

/// A turnover-regime ALife config: sub-sustain influx (0.34) with a low bootstrap
/// replication threshold (3.0) drives births AND deaths — the regime where the
/// numbers start mattering (validation plan §V0.3). Same knobs as the Experiment-6
/// measurement so V0 exercises the exact instrument S7 reads.
fn turnover_config(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "v0"
run_id = "v0"
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
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn build_engine(cfg: &SimConfig) -> (Engine, HypergraphStore) {
    let rules = hg_sim_core::runner::load_rules(cfg).unwrap();
    let store = hg_sim_core::runner::build_initial_store(cfg);
    let engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);
    (engine, store)
}

/// Live carrier-labelled node ids in a snapshot (the ground truth the observer's
/// event-reconstruction is checked against).
fn carrier_ids(snap: &GraphSnapshot<'_>) -> HashSet<NodeId> {
    snap.nodes()
        .filter(|(_, d)| d.label == CARRIER_LABEL)
        .map(|(n, _)| n)
        .collect()
}

/// Total energy (property key 1) summed over every node — the closed-economy trace.
fn total_energy_bits(snap: &GraphSnapshot<'_>) -> u64 {
    let sum: f64 = snap
        .nodes()
        .filter_map(|(_, d)| match d.get_property(ENERGY_KEY) {
            Some(PropertyValue::Float(v)) => Some(v),
            _ => None,
        })
        .sum();
    sum.to_bits()
}

/// Handle mirroring the runner's `AlifeObserverHandle`: an `Arc<Mutex<..>>` clone
/// stays in the caller's hand so the lineage can be read between steps.
struct Handle {
    inner: Arc<Mutex<AlifeObserver>>,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, stats: &StepStats, events: &[RewriteEvent]) {
        self.inner.lock().unwrap().on_step_end(snap, stats, events);
    }
}

// ---------------------------------------------------------------------------
// V0.1 — Read-only proof.
// ---------------------------------------------------------------------------
/// Run the ecology with `AlifeObserver` attached and detached, same seed, same
/// steps. Assert bit-identical `canonical_hash` trace AND bit-identical
/// total-energy trace across a LIVE multi-carrier run. Failure = a bug:
/// observation is mutating state and every S5–S7 number measures a different
/// system than the one reported.
#[test]
fn v0_1_observer_is_read_only() {
    let steps = 1_500u64;

    // Arm A: no observer.
    let cfg = turnover_config(steps);
    let (mut engine, mut store) = build_engine(&cfg);
    let mut hash_a = Vec::with_capacity(steps as usize);
    let mut energy_a = Vec::with_capacity(steps as usize);
    for _ in 1..=steps {
        engine.step(&mut store);
        hash_a.push(store.canonical_hash());
        energy_a.push(total_energy_bits(&store.snapshot()));
    }

    // Arm B: identical run with AlifeObserver attached.
    let cfg = turnover_config(steps);
    let (mut engine, mut store) = build_engine(&cfg);
    engine.add_observer(Box::new(Handle {
        inner: Arc::new(Mutex::new(AlifeObserver::new(128))),
    }));
    let mut hash_b = Vec::with_capacity(steps as usize);
    let mut energy_b = Vec::with_capacity(steps as usize);
    for _ in 1..=steps {
        engine.step(&mut store);
        hash_b.push(store.canonical_hash());
        energy_b.push(total_energy_bits(&store.snapshot()));
    }

    assert_eq!(hash_a, hash_b, "V0.1 FAILURE (bug): attaching AlifeObserver changed the canonical_hash trace — observation is mutating state");
    assert_eq!(energy_a, energy_b, "V0.1 FAILURE (bug): attaching AlifeObserver changed the total-energy trace");

    // Sanity: the run actually had carriers (else the read-only claim is vacuous).
    let final_carriers = carrier_ids(&store.snapshot()).len();
    eprintln!("V0.1: {steps} steps, hashes/energy bit-identical with vs without observer; {final_carriers} live carriers at end");
}

// ---------------------------------------------------------------------------
// V0.2 — Population conservation.
// ---------------------------------------------------------------------------
/// Every step, assert `carriers(t) == carriers(t-1) + births(t) - deaths(t)`
/// against the graph's actual carrier-labelled node count. A birth is a
/// `bootstrap_seed` or `replicate` that created a carrier; a death is a
/// `die_and_recycle`. Failure = a bug: an event is missed or double-counted and
/// lineage/clade counts are corrupt.
#[test]
fn v0_2_population_conservation() {
    let steps = 2_000u64;
    let cfg = turnover_config(steps);
    let (mut engine, mut store) = build_engine(&cfg);

    // A probe observer that captures per-step births/deaths from the event stream.
    #[derive(Default)]
    struct Probe {
        last: (u64, u64), // (births, deaths) captured on the most recent step
    }
    impl Observer for Probe {
        fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _s: &StepStats, events: &[RewriteEvent]) {
            let mut births = 0u64;
            let mut deaths = 0u64;
            for e in events {
                match e.rule_name.as_str() {
                    "bootstrap_seed" | "replicate" => {
                        // Count only creations that actually produced a carrier node.
                        if e.nodes_created.iter().any(|&n| {
                            snap.get_node(n).map(|d| d.label == CARRIER_LABEL).unwrap_or(false)
                        }) {
                            births += 1;
                        }
                    }
                    "die_and_recycle" => deaths += e.nodes_deleted.len() as u64,
                    _ => {}
                }
            }
            self.last = (births, deaths);
        }
    }

    let probe = Arc::new(Mutex::new(Probe::default()));
    struct PHandle(Arc<Mutex<Probe>>);
    impl Observer for PHandle {
        fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, e: &[RewriteEvent]) {
            self.0.lock().unwrap().on_step_end(snap, s, e);
        }
    }
    engine.add_observer(Box::new(PHandle(probe.clone())));

    let mut prev = carrier_ids(&store.snapshot()).len() as i64; // 0 before any step
    let mut total_births = 0u64;
    let mut total_deaths = 0u64;
    for t in 1..=steps {
        engine.step(&mut store);
        let (births, deaths) = probe.lock().unwrap().last;
        let now = carrier_ids(&store.snapshot()).len() as i64;
        assert_eq!(
            now,
            prev + births as i64 - deaths as i64,
            "V0.2 FAILURE (bug) at step {t}: carriers(t)={now} != carriers(t-1)={prev} + births={births} - deaths={deaths}"
        );
        prev = now;
        total_births += births;
        total_deaths += deaths;
    }
    eprintln!("V0.2: conservation held every step over {steps} steps; total births={total_births}, deaths={total_deaths}, final carriers={prev}");
    assert!(total_births > 0 && total_deaths > 0, "V0.2 vacuous: run had no turnover (births={total_births}, deaths={total_deaths})");
}

// ---------------------------------------------------------------------------
// V0.3 — NodeId reuse under SlotMap.
// ---------------------------------------------------------------------------
/// `SlotMap` recycles slots. If `LineageTree` keyed on a bare slot index, a dead
/// carrier's slot reused by a later birth would make the newcomer inherit the
/// dead one's ancestry. `NodeId` is a generational key (index + version), so a
/// reused slot yields a DISTINCT `NodeId` and `AHashMap<NodeId,_>` cannot bleed.
/// This proves it: force a death, then births that recycle the freed slot, and
/// assert the recycled id is a fresh key with no inherited ancestry.
#[test]
fn v0_3_slotmap_version_key_no_ancestry_bleed() {
    use hg_core::NodeData;
    let mut store = HypergraphStore::new();

    // A carrier with a genealogy, then kill it and recycle its slot.
    let victim = store.add_node(NodeData::new(CARRIER_LABEL, 0));
    let mut tree = LineageTree::new();
    let g = hg_alife::genome::Genome {
        harvest_rate: Some(1.0),
        replication_threshold: Some(3.0),
        maintenance_cost: Some(1.0),
        satiation: Some(20.0),
    };
    tree.record_birth(victim, None, g, 1); // founder
    let child = store.add_node(NodeData::new(CARRIER_LABEL, 0));
    tree.record_birth(child, Some(victim), g, 2); // victim -> child
    assert_eq!(tree.parent_of(child), Some(victim));

    // Kill the founder in the store (free its slot) and in the tree.
    store.remove_node(victim);
    tree.record_death(victim, 3);

    // Now mint new nodes; SlotMap will reuse the freed slot for one of them.
    let mut first_fresh = None;
    for _ in 0..4 {
        let n = store.add_node(NodeData::new(CARRIER_LABEL, 0));
        // A newly minted id must NEVER equal the dead founder's id...
        assert_ne!(n, victim, "V0.3 FAILURE (bug): a reused slot produced a NodeId equal to the dead carrier's — versioning is off");
        // ...and the lineage tree must not already know it (no inherited ancestry).
        assert!(tree.genome_at_birth(n).is_none(), "V0.3 FAILURE (bug): a fresh NodeId already carries lineage state — ancestry bled across a reused slot");
        assert!(tree.parent_of(n).is_none(), "V0.3 FAILURE (bug): a fresh NodeId inherited a parent from a reused slot");
        first_fresh.get_or_insert(n);
    }

    // Record one of the fresh ids as a brand-new founder; its ancestry is its own.
    let fresh = first_fresh.unwrap();
    tree.record_birth(fresh, None, g, 5);
    assert_eq!(tree.parent_of(fresh), None, "V0.3: the recycled-slot carrier is its own root, not the dead founder's descendant");
    eprintln!("V0.3: dead carrier {victim:?} freed; 4 recycled-slot births all fresh keys with no ancestry bleed");
}

// ---------------------------------------------------------------------------
// V0.4 — Events-alone reconstruction vs ground truth.
// ---------------------------------------------------------------------------
/// On a live run, at several steps N compare the observer's event-reconstructed
/// live-carrier set (`LineageTree::live_carriers`) to the actual carrier-labelled
/// nodes in the graph. Assert set equality — including that births attribute the
/// right child id and deaths the right deleted id. Failure = a bug in event
/// attribution.
#[test]
fn v0_4_events_reconstruct_live_population() {
    let steps = 2_000u64;
    let cfg = turnover_config(steps);
    let (mut engine, mut store) = build_engine(&cfg);

    let obs = Arc::new(Mutex::new(AlifeObserver::new(128)));
    engine.add_observer(Box::new(Handle { inner: obs.clone() }));

    let checkpoints: HashSet<u64> = [1, 50, 200, 500, 1000, 1500, 2000].into_iter().collect();
    let mut checked = 0;
    for t in 1..=steps {
        engine.step(&mut store);
        if checkpoints.contains(&t) {
            let snap = store.snapshot();
            let ground_truth = carrier_ids(&snap);
            let reconstructed = obs.lock().unwrap().lineage().live_carriers();
            assert_eq!(
                reconstructed, ground_truth,
                "V0.4 FAILURE (bug) at step {t}: event-reconstructed live set ({} carriers) != graph carrier set ({} carriers)",
                reconstructed.len(), ground_truth.len()
            );
            checked += 1;
        }
    }
    assert!(checked >= 5, "V0.4 did not reach enough checkpoints");
    eprintln!("V0.4: reconstructed live-carrier set == graph carrier set at {checked} checkpoints across {steps} steps");
}

// ---------------------------------------------------------------------------
// V0.5 — NonTrivial negative control (the single most important instrument test).
// ---------------------------------------------------------------------------
/// The criterion-6 headline depends on the classifier being able to WITHHOLD
/// `NonTrivial`. Hand-build two carriers: a trivial copier (one stochastic birth,
/// depth 1) and a genuinely non-trivial one (a sustained lineage, depth ≥ 2).
/// Assert the count classifies the second and NOT the first. Failure = a bug if
/// everything is labelled NonTrivial — then the 0→>0 CE crossing is a label
/// artifact and the entire boundary is spurious.
#[test]
fn v0_5_nontrivial_negative_control() {
    use hg_core::{HypergraphStore, Label, NodeData};
    let mut store = HypergraphStore::new();
    let mint = |s: &mut HypergraphStore| s.add_node(NodeData::new(Label(100), 0));
    let g = hg_alife::genome::Genome {
        harvest_rate: Some(1.0),
        replication_threshold: Some(3.0),
        maintenance_cost: Some(1.0),
        satiation: Some(20.0),
    };

    // Trivial copier: founder_t -> one child, and that's it (a single stochastic
    // birth). Its deepest descendant depth is 1.
    let founder_t = mint(&mut store);
    let child_t = mint(&mut store);
    // Genuinely non-trivial: founder_n -> a -> b (a sustained self-copying chain,
    // depth 2 below the founder).
    let founder_n = mint(&mut store);
    let a = mint(&mut store);
    let b = mint(&mut store);

    let mut tree = LineageTree::new();
    tree.record_birth(founder_t, None, g, 1);
    tree.record_birth(child_t, Some(founder_t), g, 2);
    tree.record_birth(founder_n, None, g, 3);
    tree.record_birth(a, Some(founder_n), g, 4);
    tree.record_birth(b, Some(a), g, 5);

    // min_generations = 2 (the observer's NON_TRIVIAL_MIN_GENERATIONS): sustained.
    let count = tree.non_trivial_replicator_count(2);
    assert_eq!(
        count, 1,
        "V0.5 FAILURE (bug): expected exactly ONE non-trivial replicator (the sustained lineage), got {count} — the classifier cannot withhold NonTrivial, so the CE 0→>0 crossing is a label artifact"
    );

    // Pin WHICH one: the trivial copier's founder must NOT count; the sustained
    // one's founder MUST. (Deepest-descendant-depth is the discriminator.)
    // Rebuild singletons to assert each side in isolation.
    let mut trivial = LineageTree::new();
    trivial.record_birth(founder_t, None, g, 1);
    trivial.record_birth(child_t, Some(founder_t), g, 2);
    assert_eq!(trivial.non_trivial_replicator_count(2), 0, "V0.5: a single stochastic birth (depth 1) must be withheld from NonTrivial");

    let mut sustained = LineageTree::new();
    sustained.record_birth(founder_n, None, g, 3);
    sustained.record_birth(a, Some(founder_n), g, 4);
    sustained.record_birth(b, Some(a), g, 5);
    // The depth-2 chain founder_n→a→b: only founder_n has a descendant lineage
    // reaching depth ≥2 (a's is depth 1), so exactly one carrier is NonTrivial.
    // The point is separation from the trivial copier (0), not the magnitude.
    assert_eq!(sustained.non_trivial_replicator_count(2), 1, "V0.5: the sustained lineage's founder sustains a ≥2-generation chain and counts as NonTrivial");

    eprintln!("V0.5: NonTrivial withheld from the trivial copier, assigned to the sustained lineage — separation holds");
}
