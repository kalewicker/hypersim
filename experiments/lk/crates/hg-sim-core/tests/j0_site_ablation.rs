//! J0.2 — the site-ablation gate (V5 / J-track).
//!
//! Definition committed in `docs/validation_v5_j0_ablation_definition.md`
//! (commit 165c925, BEFORE this measurement). Question: once the substrate's
//! geometry is held constant across all carriers, do carriers still split into
//! more than one WL class from their OWN attributes, or was the 4-class
//! resolution entirely borrowed from the frozen crystal's corner-vs-interior
//! site geometry?
//!
//! Ablation: WL refinement to depth 2, but every non-carrier node's hash is
//! forced to a single constant at round 0 AND after every refinement round, so
//! each carrier aggregates an identical synthetic site neighbourhood. Any
//! residual carrier split is then carrier-intrinsic (energy field / degree /
//! carrier-carrier adjacency). Compared against the un-ablated baseline, which
//! must reproduce F8's 4 carrier classes (fidelity anchor).
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId};
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

const STEPS: u64 = 6_000;
const WL_DEPTH: u8 = 2;
const SEED_ANCHOR: u64 = 12345;
// Local WL seed + a site-constant sentinel. Values are arbitrary: only the
// induced partition (distinct-class COUNT) is reported, which is invariant to
// the hash function choice.
const LWL_SEED: u64 = 0x51D0_5EED_0BAB_A5E5;
const SITE_CONST: u64 = 0xC0FF_EE00_5171_E000;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "j0"
run_id = "j0"
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
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

/// WL class map replicated locally so the SAME algorithm can be run with and
/// without the site ablation. `ablate=false` must reproduce production's carrier
/// class count (checked against `compute_wl_class_map_snapshot`); `ablate=true`
/// forces every non-carrier node's hash to SITE_CONST at round 0 and after every
/// refinement round.
fn wl_classes(store: &HypergraphStore, radius: u8, ablate: bool) -> HashMap<NodeId, u64> {
    let is_carrier = |id: NodeId| store.get_node(id).map(|d| d.label == CARRIER_LABEL).unwrap_or(false);
    let mut cur: HashMap<NodeId, u64> = HashMap::new();
    for (id, d) in store.nodes() {
        let h = if ablate && !is_carrier(id) {
            SITE_CONST
        } else {
            let mut hasher = rustc_hash::FxHasher::default();
            LWL_SEED.hash(&mut hasher);
            d.label.0.hash(&mut hasher);
            d.energy.to_bits().hash(&mut hasher);
            hasher.finish()
        };
        cur.insert(id, h);
    }
    for _ in 0..radius {
        let mut next: HashMap<NodeId, u64> = HashMap::new();
        for (id, _) in store.nodes() {
            if ablate && !is_carrier(id) {
                next.insert(id, SITE_CONST); // hold substrate constant EVERY round
                continue;
            }
            let mut nbr: Vec<u64> = Vec::new();
            for eid in store.edges_incident_to(id) {
                if let Some(e) = store.get_edge(*eid) {
                    for &n in &e.nodes {
                        if n != id {
                            if let Some(&h) = cur.get(&n) {
                                nbr.push(h);
                            }
                        }
                    }
                }
            }
            nbr.sort_unstable();
            let mut hasher = rustc_hash::FxHasher::default();
            LWL_SEED.hash(&mut hasher);
            cur.get(&id).unwrap().hash(&mut hasher);
            for h in nbr {
                h.hash(&mut hasher);
            }
            next.insert(id, hasher.finish());
        }
        cur = next;
    }
    cur
}

#[test]
fn j0_site_ablation() {
    let cfg = config(SEED_ANCHOR);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    // Accumulate the distinct carrier classes over the whole run, three ways:
    //  - production WL (fidelity anchor; expect 4)
    //  - local WL, un-ablated (must match production count)
    //  - local WL, ablated (the intrinsic count — the gate)
    let mut prod_carrier: HashSet<u64> = HashSet::new();
    let mut local_carrier: HashSet<u64> = HashSet::new();
    let mut ablated_carrier: HashSet<u64> = HashSet::new();
    // For interpretation: per ablated carrier class, the (energy_field, degree)
    // signature of its members, so we can name what drives any residual split.
    let mut ablated_signature: HashMap<u64, HashSet<(u32, usize)>> = HashMap::new();
    let mut max_carrier_degree = 0usize;
    let mut distinct_energy_fields: HashSet<u32> = HashSet::new();

    for _ in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let prod = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let local = wl_classes(&store, WL_DEPTH, false);
        let ablated = wl_classes(&store, WL_DEPTH, true);
        for (id, d) in snap.nodes() {
            if d.label == CARRIER_LABEL {
                if let Some(h) = prod.get(&id) {
                    prod_carrier.insert(h.0);
                }
                local_carrier.insert(local[&id]);
                let ah = ablated[&id];
                ablated_carrier.insert(ah);
                let deg = store.edges_incident_to(id).len();
                max_carrier_degree = max_carrier_degree.max(deg);
                distinct_energy_fields.insert(d.energy.to_bits());
                ablated_signature.entry(ah).or_default().insert((d.energy.to_bits(), deg));
            }
        }
    }

    let baseline = prod_carrier.len();
    let local_count = local_carrier.len();
    let ablated_count = ablated_carrier.len();

    eprintln!("\n================ J0.2 — SITE ABLATION (seed {SEED_ANCHOR}, WL depth {WL_DEPTH}) ================");
    eprintln!("  carrier classes WITH site geometry (production WL, fidelity anchor): {baseline}  (F8: 4)");
    eprintln!("  carrier classes WITH site geometry (local WL, must match anchor):    {local_count}");
    eprintln!("  carrier classes WITHOUT site geometry (ABLATED — the gate):          {ablated_count}");
    eprintln!("\n  intrinsic attributes actually observed across all carriers over the run:");
    eprintln!("    distinct energy-FIELD values : {} ({:?})  [expected 1 = uniformly 0.0]",
        distinct_energy_fields.len(),
        distinct_energy_fields.iter().map(|b| f32::from_bits(*b)).collect::<Vec<_>>());
    eprintln!("    max carrier degree           : {max_carrier_degree}  [expected 1 = one occupancy edge]");
    if ablated_count > 1 {
        eprintln!("\n  RESIDUAL SPLIT — (energy_field_bits, degree) signatures per ablated class:");
        for (h, sigs) in &ablated_signature {
            eprintln!("    class {:016x}: {:?}", h, sigs);
        }
    }

    eprintln!("\n================ GATE VERDICT ================");
    // Fidelity anchor first.
    assert_eq!(baseline, 4, "production WL did not reproduce F8's 4 carrier classes — run drifted");
    assert_eq!(local_count, baseline, "local WL replication diverged from production carrier count");

    if ablated_count > 1 {
        eprintln!("  INTRINSIC STRUCTURE SURVIVES ({ablated_count} classes without site geometry) => J1 (hybrid partition).");
        eprintln!("  Driving attribute(s): energy-field distinct={}, max degree={} (see signatures above).",
            distinct_energy_fields.len(), max_carrier_degree);
    } else {
        eprintln!("  STRUCTURE IS BORROWED (carriers collapse to 1 class without site geometry) => J2 (non-partition measure).");
        eprintln!("  The 4-class carrier resolution was entirely substrate-geometric (corner-vs-interior");
        eprintln!("  site neighbourhoods). Carriers have NO churn-survivable WL structure of their own:");
        eprintln!("  energy field uniformly 0.0 (ALife energy is property key 1, WL-excluded) and every");
        eprintln!("  carrier has exactly one occupancy edge. A label anchor would reinstate F0's (9,1)");
        eprintln!("  under-resolution. The sustained-turnover boundary is GEOMETRY-DEPENDENT (J3).");
    }
    eprintln!("=================================================================\n");
}
