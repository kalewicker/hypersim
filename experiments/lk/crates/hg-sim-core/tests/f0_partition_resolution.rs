//! F-track F0 — Partition resolution diagnostic (branch point).
//!
//! The V1 track reported CE across the carrier/substrate partition as
//! indistinguishable from an arbitrary partition (p = 0.495) and took the fork
//! branch labelled "boundary spurious." That label is contradicted by V2.2 in the
//! same document: the `NonTrivial` 0→>0 crossing IS a real depth-2 sustained
//! self-copying event. What is degenerate is the MACRO DESCRIPTION — the partition
//! has group sizes (9, 1), one carrier WL class against nine substrate-side
//! classes, so the permutation null had support of only ~10 partitions.
//!
//! Leading candidate cause, and the hypothesis under test here: **WL iteration
//! depth**. `base_hash` is `WL_SEED + label + energy field` — and ALife energy is
//! property key 1, DELIBERATELY excluded from WL (the Q2 invariant). So every
//! substrate site has an identical base hash. A carrier has exactly one neighbour
//! (its occupancy site), so at radius 1 every carrier hashes to
//! `f(carrier_base, {site_base})` — necessarily identical for all carriers,
//! regardless of which site they occupy. Sites themselves split at radius 1 only
//! because their neighbour-multiset SIZES differ (degree / occupancy). The site's
//! own neighbourhood — the thing that actually differs — only enters the carrier's
//! hash at radius 2.
//!
//! **This file computes NO CE.** Per F1, the depth-selection rule is committed
//! from this table alone, before any coefficient is seen (plan rule 6: selecting
//! depth by its effect on CE is the same forbidden move as tuning influx).
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_core::{compute_wl_class_map_snapshot, GraphSnapshot, NodeId, WlHash};
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::collections::{BTreeMap, HashMap, HashSet};

/// The exact configuration the reported CE was computed on.
const STEPS: u64 = 6_000;
/// Class INVENTORY is what the partition is built from; sampling every 20 steps
/// enumerates it faithfully while keeping depth-5 WL over 300 snapshots cheap.
const SAMPLE_EVERY: u64 = 20;
const MAX_DEPTH: u8 = 5;

fn config() -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "f0"
run_id = "f0"
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
max_steps = {STEPS}
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

/// The substrate site a carrier occupies: the non-carrier endpoint of any edge
/// incident to the carrier.
fn occupied_site(snap: &GraphSnapshot<'_>, carrier: NodeId) -> Option<NodeId> {
    for &e in snap.edges_incident_to(carrier) {
        let edge = snap.get_edge(e)?;
        for &n in &edge.nodes {
            if n != carrier && snap.get_node(n).map(|d| d.label != CARRIER_LABEL).unwrap_or(false) {
                return Some(n);
            }
        }
    }
    None
}

#[test]
fn f0_partition_resolution_diagnostic() {
    let cfg = config();
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    // Per depth: the set of distinct WL classes seen over the run, tagged carrier
    // or substrate. This is exactly the inventory the CE micro partition is built
    // from (the TPM accumulates classes across the whole run, not one snapshot).
    let mut carrier_classes: BTreeMap<u8, HashSet<u64>> = BTreeMap::new();
    let mut substrate_classes: BTreeMap<u8, HashSet<u64>> = BTreeMap::new();
    // F0.3: which substrate class (at depth 1) does each carrier-occupied site belong to?
    let mut occupancy_by_site_class: HashMap<u64, u64> = HashMap::new();
    let mut all_site_classes_d1: HashSet<u64> = HashSet::new();
    let mut samples = 0u64;

    for step in 1..=STEPS {
        engine.step(&mut store);
        if step % SAMPLE_EVERY != 0 {
            continue;
        }
        samples += 1;
        let snap = store.snapshot();

        for d in 1..=MAX_DEPTH {
            let classes = compute_wl_class_map_snapshot(&snap, d);
            for (id, data) in snap.nodes() {
                let h = classes.get(&id).copied().unwrap_or(WlHash(0)).0;
                if data.label == CARRIER_LABEL {
                    carrier_classes.entry(d).or_default().insert(h);
                } else {
                    substrate_classes.entry(d).or_default().insert(h);
                }
            }
        }

        // F0.3 uses the depth-1 substrate classes (the 8 the V1 partition saw).
        let d1 = compute_wl_class_map_snapshot(&snap, 1);
        for (id, data) in snap.nodes() {
            if data.label != CARRIER_LABEL {
                all_site_classes_d1.insert(d1.get(&id).copied().unwrap_or(WlHash(0)).0);
            }
        }
        for (id, data) in snap.nodes() {
            if data.label == CARRIER_LABEL {
                if let Some(site) = occupied_site(&snap, id) {
                    let sc = d1.get(&site).copied().unwrap_or(WlHash(0)).0;
                    *occupancy_by_site_class.entry(sc).or_insert(0) += 1;
                }
            }
        }
    }

    eprintln!("\n================ F0 — PARTITION RESOLUTION DIAGNOSTIC ================");
    eprintln!("config: seed 12345, influx 0.34, bootstrap thr 3.0, {STEPS} steps");
    eprintln!("class inventory sampled every {SAMPLE_EVERY} steps ({samples} snapshots)");
    eprintln!("NOTE: this file computes NO CE. Depth is selected from this table alone (F1).\n");

    // ---------------- F0.1 ----------------
    eprintln!("--- F0.1  configured WL iteration depth in the CE pipeline ---");
    eprintln!("  experiment6_ce_across_onset.rs:102   compute_wl_class_map_snapshot(&snapshot, 1)");
    eprintln!("  experiment6b_birthdeath_ce.rs:95     compute_wl_class_map_snapshot(&snapshot, 1)");
    eprintln!("  v3_ce_validation.rs:101              compute_wl_class_map_snapshot(&snapshot, 1)");
    eprintln!("  runner.rs:1045                       compute_wl_class_map_snapshot(&snapshot, 1)");
    eprintln!("  hg-causal/observer.rs                WL_RADIUS = 1");
    eprintln!("  hg-eval/thresholds.rs                WL_RADIUS = 1  (pub)");
    eprintln!("  hg-selfmod/runner.rs                 cfg.wl_radius, DEFAULT 3 (configurable)");
    eprintln!("  ⟹ CE stack is internally consistent at depth 1, but the depth is a MAGIC");
    eprintln!("    NUMBER duplicated at 4 sites, never a parameter; a sibling subsystem chose 3.\n");

    // ---------------- F0.2 ----------------
    eprintln!("--- F0.2  carrier class count as a function of WL depth ---");
    eprintln!("   d | total M | carrier classes | substrate classes | group sizes (sub, carr)");
    eprintln!("  ---+---------+-----------------+-------------------+------------------------");
    let mut first_split_depth: Option<u8> = None;
    let mut m_by_depth: BTreeMap<u8, usize> = BTreeMap::new();
    for d in 1..=MAX_DEPTH {
        let c = carrier_classes.get(&d).map(|s| s.len()).unwrap_or(0);
        let s = substrate_classes.get(&d).map(|s| s.len()).unwrap_or(0);
        let m = c + s;
        m_by_depth.insert(d, m);
        eprintln!("   {d} | {m:>7} | {c:>15} | {s:>17} | ({s}, {c})");
        if c > 1 && first_split_depth.is_none() {
            first_split_depth = Some(d);
        }
    }

    // WL convergence depth: where total M stabilises (secondary reference, F1 rule 2).
    // `None` = never stabilises within the tested range, which is itself informative:
    // an ever-growing M is the open-registry / class-explosion regime the WL Q2
    // comment warns about, NOT a convergence point to run F2 at.
    let convergence_depth: Option<u8> = (1..MAX_DEPTH)
        .find(|d| m_by_depth.get(d) == m_by_depth.get(&(d + 1)));

    eprintln!("\n  GATE: carrier-group classes > 1 at some d <= {MAX_DEPTH}?");
    match first_split_depth {
        Some(d) => {
            eprintln!("  ⟹ YES — carriers first resolve into >1 class at depth {d}.");
            eprintln!("    The (9, 1) partition was a DEPTH ARTIFACT. The V1 headline is");
            eprintln!("    provisional and may reverse. Proceed to F1 (commit depth rule), then F2.");
        }
        None => {
            eprintln!("  ⟹ NO — carriers share one class at every depth 1..{MAX_DEPTH}.");
            eprintln!("    Not a depth problem. F0.3 below distinguishes the alternatives.");
        }
    }
    match convergence_depth {
        Some(d) => eprintln!("  WL convergence depth (total M stabilises): {d}"),
        None => {
            eprintln!("  WL convergence depth: NONE within 1..{MAX_DEPTH} — M is still growing");
            eprintln!("    ({}). This is the open-registry / class-explosion regime (each carrier",
                m_by_depth.values().map(|m| m.to_string()).collect::<Vec<_>>().join(" → "));
            eprintln!("    approaching its own class), NOT a convergence point to run F2 at.");
        }
    }

    // ---------------- F0.3 ----------------
    eprintln!("\n--- F0.3  carrier site-occupancy across substrate classes (depth-1 classes) ---");
    eprintln!("  distinct substrate classes at depth 1 over the run: {}", all_site_classes_d1.len());
    let total_occ: u64 = occupancy_by_site_class.values().sum();
    let mut rows: Vec<(u64, u64)> = occupancy_by_site_class.iter().map(|(k, v)| (*k, *v)).collect();
    rows.sort_by_key(|(k, _)| *k);
    eprintln!("  carrier-occupied site observations by site class ({} total):", total_occ);
    for (cls, n) in &rows {
        let pct = 100.0 * *n as f64 / total_occ.max(1) as f64;
        eprintln!("    site class {:#018x} : {:>7} ({:5.1}%)", cls, n, pct);
    }
    eprintln!("  ⟹ carriers occupy {} of {} distinct substrate classes",
        rows.len(), all_site_classes_d1.len());
    if rows.len() <= 1 {
        eprintln!("    CONCENTRATED on one class ⟹ a finding about the DISPERSAL mechanic");
        eprintln!("    (birth-dispersal to adjacent nodes may never reach boundary sites),");
        eprintln!("    not about WL.");
    } else {
        eprintln!("    SPREAD across classes ⟹ there IS site heterogeneity for carriers to");
        eprintln!("    inherit; whether they inherit it is the depth question in F0.2.");
    }

    eprintln!("\n================ F0 DECISION ================");
    match (first_split_depth, rows.len() > 1) {
        (Some(d), _) => eprintln!("  BRANCH: split at depth {d} ⟹ F1 → F2. V1 headline provisional."),
        (None, true) => eprintln!("  BRANCH: no split, occupancy SPREAD ⟹ classifier/hashing investigation; do NOT proceed to F2."),
        (None, false) => eprintln!("  BRANCH: no split, occupancy CONCENTRATED ⟹ dispersal finding; V1 negative stands, scope narrows."),
    }
    eprintln!("=============================================================\n");

    // The diagnostic must be reported whichever way it goes (plan rule 7); the only
    // hard assertion is that the run actually produced carriers to classify.
    assert!(
        carrier_classes.get(&1).map(|s| !s.is_empty()).unwrap_or(false),
        "F0 vacuous: no carriers observed in the run"
    );
}
