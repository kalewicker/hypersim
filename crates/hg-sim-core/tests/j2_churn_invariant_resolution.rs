//! J2 crux — does a CHURN-INVARIANT carrier resolution recover F8 detectability?
//! (V5 / J-track, rule 4 F8-baseline gate, on static data.)
//!
//! J0 showed the carrier's WL resolution (4 classes) is substrate-geometric and
//! dies under churn. The one carrier-intrinsic, churn-invariant descriptor J0 did
//! NOT test is a node PROPERTY: energy (key 1), WL-excluded by Q2, owned by the
//! carrier not the site, and forming a small BOUNDED band set (recurrent, not
//! ephemeral like WL classes). If a (label, energy-band) partition recovers F8's
//! detectability on the STATIC run, a churn-invariant instrument candidate exists
//! (proceed to J2.3). If it cannot recover F8 even on the best-case static
//! substrate, no churn-invariant resolution can, and J2.1's STOP fires — measured.
//!
//! Scoring is F8's, verbatim. The WL partition is computed alongside as the anchor
//! (must reproduce 0.1633, 0/200, 4 carrier classes).
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::collections::HashMap;

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const STEPS: u64 = 6_000;
const WL_DEPTH: u8 = 2;
const N_PERM: usize = 200;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "j2crux"
run_id = "j2crux"
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

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
fn null_support(m: usize, k: usize) -> f64 {
    if k == 0 || k >= m { return 0.0; }
    let k = k.min(m - k);
    let mut acc = 1.0f64;
    for i in 0..k { acc *= (m - i) as f64 / (i + 1) as f64; }
    acc
}
fn ce_for(tpm: &TpmAccumulator, group_of: &[usize], k: usize) -> f64 {
    let rows = tpm.normalized_rows();
    let micro = ei_from_tpm(&rows.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>());
    let macro_rows = coarse_grain(&rows, group_of, k);
    let mac = ei_from_tpm(&macro_rows);
    if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 }
}

/// Score the carrier/substrate boundary for a TPM whose classes are flagged
/// carrier/substrate by `is_carrier`. Returns (M, k_carrier, real_ce, null_mean,
/// null_ge_real, support). Identical null construction to F8.
fn score(tpm: &TpmAccumulator, is_carrier: &HashMap<u64, bool>, seed: u64) -> (usize, usize, f64, f64, usize, f64) {
    let mut compact: HashMap<usize, usize> = HashMap::new();
    let group_of: Vec<usize> = tpm.hashes().iter().map(|h| {
        let raw = *is_carrier.get(&h.0).unwrap_or(&false) as usize;
        let next = compact.len();
        *compact.entry(raw).or_insert(next)
    }).collect();
    let k = compact.len().max(1);
    let m = tpm.hashes().len();
    let k_carrier = tpm.hashes().iter().filter(|h| *is_carrier.get(&h.0).unwrap_or(&false)).count();
    let real_ce = if k >= 2 { ce_for(tpm, &group_of, k) } else { 0.0 };
    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7_u64 ^ seed;
    let (mut null_ge, mut acc) = (0usize, 0.0f64);
    if k >= 2 {
        for _ in 0..N_PERM {
            let mut labels: Vec<usize> = canon.iter().map(|&i| group_of[i]).collect();
            for i in (1..labels.len()).rev() {
                let j = (splitmix(&mut rng) % (i as u64 + 1)) as usize;
                labels.swap(i, j);
            }
            let mut permuted = vec![0usize; group_of.len()];
            for (pos, &i) in canon.iter().enumerate() { permuted[i] = labels[pos]; }
            let c = ce_for(tpm, &permuted, k);
            acc += c;
            if c >= real_ce { null_ge += 1; }
        }
    }
    (m, k_carrier, real_ce, acc / N_PERM as f64, null_ge, null_support(m, k_carrier))
}

/// (label, energy-band) class hash. Band = floor(energy) — a bounded, recurrent,
/// churn-invariant descriptor (energy is a node property, not site geometry).
fn energy_band_class(label: i64, energy: f64) -> u64 {
    let band = energy.floor().clamp(-1.0, 64.0) as i64;
    ((label as u64) << 32) ^ (band as u64 & 0xFFFF_FFFF) ^ 0x9E37_79B9
}

#[test]
fn j2_churn_invariant_resolution() {
    let seed = 12345;
    let cfg = config(seed);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    // Two TPMs over the SAME run: WL classes (F8 anchor) and (label,energy-band).
    let mut wl_tpm = TpmAccumulator::new();
    let mut eb_tpm = TpmAccumulator::new();
    let mut wl_is_c: HashMap<u64, bool> = HashMap::new();
    let mut eb_is_c: HashMap<u64, bool> = HashMap::new();
    wl_is_c.insert(VACUUM.0, false);
    eb_is_c.insert(VACUUM.0, false);
    let mut wl_prev: Option<HashMap<NodeId, WlHash>> = None;
    let mut eb_prev: Option<HashMap<NodeId, u64>> = None;

    for _ in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut wl_now: HashMap<NodeId, WlHash> = HashMap::new();
        let mut eb_now: HashMap<NodeId, u64> = HashMap::new();
        for (id, data) in snap.nodes() {
            let is_c = data.label == CARRIER_LABEL;
            let h = wl.get(&id).copied().unwrap_or(WlHash(0));
            wl_now.insert(id, h);
            wl_is_c.entry(h.0).or_insert(is_c);
            let energy = match data.get_property(hg_alife::keys::ENERGY) {
                Some(PropertyValue::Float(v)) => v,
                _ => 0.0,
            };
            let eb = energy_band_class(data.label.0 as i64, energy);
            eb_now.insert(id, eb);
            eb_is_c.entry(eb).or_insert(is_c);
        }
        if let Some(p) = &wl_prev {
            for (id, &hp) in p { wl_tpm.record(hp, wl_now.get(id).copied().unwrap_or(VACUUM)); }
            for (id, &hn) in &wl_now { if !p.contains_key(id) { wl_tpm.record(VACUUM, hn); } }
        }
        if let Some(p) = &eb_prev {
            for (id, &hp) in p { eb_tpm.record(WlHash(hp), WlHash(eb_now.get(id).copied().unwrap_or(VACUUM.0))); }
            for (id, &hn) in &eb_now { if !p.contains_key(id) { eb_tpm.record(VACUUM, WlHash(hn)); } }
        }
        wl_prev = Some(wl_now);
        eb_prev = Some(eb_now);
    }

    let (wm, wk, wce, wnull, wge, wsup) = score(&wl_tpm, &wl_is_c, seed);
    let (em, ek, ece, enull, ege, esup) = score(&eb_tpm, &eb_is_c, seed);

    eprintln!("\n================ J2 CRUX — churn-invariant (energy-band) resolution vs WL (seed {seed}) ================");
    eprintln!("  resolution              | M   | k_carrier | real CE | null mean | null>=real | p       | support");
    eprintln!("  ------------------------+-----+-----------+---------+-----------+------------+---------+---------");
    eprintln!("  WL (site geom, F8 anchor)| {:>3} | {:>9} | {:>7.4} | {:>9.4} | {:>4}/{:<4}  | {:>7} | {:.2e}",
        wm, wk, wce, wnull, wge, N_PERM, if wge == 0 { "<.005".into() } else { format!("{:.3}", wge as f64 / N_PERM as f64) }, wsup);
    eprintln!("  (label, energy-band)     | {:>3} | {:>9} | {:>7.4} | {:>9.4} | {:>4}/{:<4}  | {:>7} | {:.2e}",
        em, ek, ece, enull, ege, N_PERM, if ege == 0 { "<.005".into() } else { format!("{:.3}", ege as f64 / N_PERM as f64) }, esup);
    eprintln!("=========================================================================================================\n");

    // WL anchor must reproduce F8.
    assert!((wce - 0.1633).abs() < 0.01 && wge == 0 && wk == 4, "WL anchor did not reproduce F8 (CE {wce:.4}, k {wk}, null>=real {wge})");

    let eb_detects = ek >= 2 && ek < em && (ege as f64 / N_PERM as f64) < 0.05 && esup > 100.0;
    eprintln!("================ J2 VERDICT (rule 4: churn-invariant resolution must recover F8) ================");
    if eb_detects {
        eprintln!("  (label, energy-band) RECOVERS F8 detectability on static data (p<0.05, support {esup:.1e}).");
        eprintln!("  A churn-invariant, above-F0 carrier resolution EXISTS => J2.1 STOP does NOT fire; PROCEED to");
        eprintln!("  J2.3 (build the measure over this resolution, then re-check its TWTF gate under churn).");
    } else {
        eprintln!("  (label, energy-band) does NOT recover F8 on static data: k_carrier={ek}, real CE {ece:.4} vs");
        eprintln!("  null mean {enull:.4}, p={:.3}, support {esup:.1e}. The one churn-invariant carrier-intrinsic", ege as f64 / N_PERM as f64);
        eprintln!("  descriptor available (energy) cannot resolve the boundary even on the BEST-CASE static substrate.");
        eprintln!("  With WL resolution dying under churn (H0/J0) and this the only other candidate, no variable set");
        eprintln!("  is both churn-invariant AND above-F0 resolved => J2.1 STOP FIRES (measured, not argued).");
    }
    eprintln!("=================================================================================================\n");
}
