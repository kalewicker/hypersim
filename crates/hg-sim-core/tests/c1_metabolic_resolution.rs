//! V-track closeout / C1 — the resolution diagnostic: is the metabolic channel's
//! coarseness a property of the CHANNEL or of the INSTRUMENT?
//!
//! Pre-registration: docs/validation_v9_c1_resolution_preregistration.md (f99d423),
//! committed BEFORE this file existed and before any score at any granularity.
//!
//! Phase 2 found that the metabolic channel excludes nothing in the ladder: every live
//! arm detects, and the only line it draws is active-vs-inert. Two independent nulls
//! (selection degenerate-only; metabolic CE finds all live carriers alike) argue that
//! uniformity is real. But F0 is the standing counter-example — an under-resolved
//! partition masquerading as a null reversed a headline once already (p = 0.495 → 0/200
//! on WL depth alone), and F0's specific pathology was a CARRIER GROUP OF SIZE 1.
//!
//! So this file computes, for the metabolic partition, exactly the F0.2 table:
//! class count M and group sizes (substrate, carrier), across depths and across binning
//! granularities.
//!
//! **THIS FILE COMPUTES NO CE, NO CLOSURE DEFECT AND NO REAL SCORE.** Selection under
//! rule C1.3-R uses partition STRUCTURE only; the spread-guard clause is evaluated from
//! NULL values alone (random same-sized partitions), which by construction carry no
//! information about any arm's true score. Selecting granularity by its effect on a
//! score is the same forbidden move as selecting WL depth by its effect on CE.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::{BTreeSet, HashMap, HashSet};

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const STEPS: u64 = 6_000;
const ENERGY: u32 = 1;
const EPS: f64 = 1e-9;
const WL_DEPTH: u8 = 2;
const N_PERM: usize = 200;
const DISTINCT_MIN: usize = 10;
/// Registry cap for the null stage only. A registry this large is the open-registry /
/// class-explosion regime F0 warned about; it is reported, not silently skipped.
const M_CAP: usize = 1_500;
/// Frozen from Phase 1. Re-running C1 at another depth after seeing the result is forbidden.
const FROZEN_DEPTH: usize = 2;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "c1"
run_id = "c1"
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

fn motility_toml() -> &'static str { include_str!("n_shared/relocate.toml") }
fn diffuse_toml() -> &'static str { include_str!("n_shared/diffuse.toml") }
fn churn_toml() -> &'static str { include_str!("n_shared/churn.toml") }

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Arm { A, B, BPrime, C, D, Inert }
fn arm_spec(a: Arm) -> (bool, bool, bool, usize) {
    match a {
        Arm::A => (false, false, false, 1),
        Arm::B => (true, true, false, 6),
        Arm::BPrime => (true, true, true, 6),
        Arm::C => (true, false, false, 6),
        Arm::D => (true, false, true, 6),
        Arm::Inert => (true, false, false, 6),
    }
}

fn build_rules(cfg: &SimConfig, arm: Arm, churn: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if arm == Arm::Inert {
        biology.retain(|r| r.name == "bootstrap_seed");
    } else {
        let (sterile, motile, diffuse, _) = arm_spec(arm);
        if sterile { biology.retain(|r| r.name != "replicate"); }
        if motile { biology.extend(parse_rules_toml(motility_toml()).unwrap()); }
        if diffuse { biology.extend(parse_rules_toml(diffuse_toml()).unwrap()); }
        if churn { biology.extend(parse_rules_toml(churn_toml()).unwrap()); }
    }
    compile_rules(&biology).unwrap()
}

// ---------------------------------------------------------------- the granularity knob
//
// Candidates ordered COARSE -> FINE, all bounded by construction (H0.1 closure), exactly
// as tabled in the pre-registration. `G_CUR` is the frozen N-track instrument.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Gran { Sign, Cur, Fine, Q05, Q01 }
pub const LADDER: [Gran; 5] = [Gran::Sign, Gran::Cur, Gran::Fine, Gran::Q05, Gran::Q01];

impl Gran {
    fn name(self) -> &'static str {
        match self { Gran::Sign => "G_sign", Gran::Cur => "G_cur*", Gran::Fine => "G_fine",
                     Gran::Q05 => "G_q05", Gran::Q01 => "G_q01" }
    }
    fn codes(self) -> usize {
        match self { Gran::Sign => 3, Gran::Cur => 13, Gran::Fine => 25, Gran::Q05 => 49, Gran::Q01 => 121 }
    }
    fn bucket(self, d: f64) -> u64 {
        match self {
            Gran::Sign => { if d.abs() < EPS { 0 } else if d < 0.0 { 1 } else { 2 } }
            // the FROZEN N-track bucket, byte-for-byte
            Gran::Cur => {
                if d.abs() < EPS { return 0; }
                let mag = d.abs();
                let m = if mag < 0.25 { 1 } else if mag < 0.75 { 2 } else if mag < 1.5 { 3 }
                        else if mag < 4.0 { 4 } else if mag < 9.0 { 5 } else { 6 };
                if d < 0.0 { m } else { m + 8 }
            }
            Gran::Fine => {
                if d.abs() < EPS { return 0; }
                let mag = d.abs();
                let cuts = [0.125, 0.25, 0.5, 0.75, 1.0, 1.5, 2.5, 4.0, 6.0, 9.0, 15.0];
                let mut m = cuts.len() as u64 + 1;
                for (i, c) in cuts.iter().enumerate() { if mag < *c { m = i as u64 + 1; break; } }
                if d < 0.0 { m } else { m + 16 }
            }
            Gran::Q05 => quant(d, 0.05, 24),
            Gran::Q01 => quant(d, 0.01, 60),
        }
    }
}

/// Quantise to steps of `q`, clamped to +/- `lim` steps. Bounded by construction.
fn quant(d: f64, q: f64, lim: i64) -> u64 {
    let s = (d / q).round() as i64;
    (s.clamp(-lim, lim) + lim) as u64
}

fn micro_class(label: i64, hist: &[u64]) -> u64 {
    let mut h = 0xCBF2_9CE4_8422_2325u64 ^ ((label as u64) << 40);
    for b in hist { h = (h ^ b).wrapping_mul(0x1000_0000_01B3); }
    h
}

fn splitmix(s: &mut u64) -> u64 {
    *s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *s;
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

// ---------------------------------------------------------------- C1.1: class structure

#[derive(Clone, Default)]
pub struct Shape {
    pub m: usize,
    pub k_carrier: usize,
    pub k_substrate: usize,
    pub support: f64,
    /// recurrence witness: |first-half classes ∩ second-half| / |union|. A partition whose
    /// registry keeps growing is NOT recurrent, whatever its class count.
    pub half_jaccard: f64,
    pub carrier_class_ids: BTreeSet<u64>,
}

/// One simulation, all (granularity, depth) partitions harvested from it. Returns the
/// class INVENTORY only — no TPM, no statistic.
pub fn shapes(seed: u64, arm: Arm, churn: bool, depths: &[usize]) -> HashMap<(Gran, usize), Shape> {
    let cfg = config(seed);
    let rules = build_rules(&cfg, arm, churn);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let (_, _, _, markers) = arm_spec(arm);
    plant_markers(&mut store, seed, markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let max_depth = *depths.iter().max().unwrap();
    let mut prev_e: HashMap<NodeId, f64> = HashMap::new();
    let mut raw_hist: HashMap<NodeId, Vec<f64>> = HashMap::new();
    // (gran, depth) -> class -> is_carrier ; and per-half class sets for the recurrence witness
    let mut inv: HashMap<(Gran, usize), HashMap<u64, bool>> = HashMap::new();
    let mut half: HashMap<(Gran, usize), (HashSet<u64>, HashSet<u64>)> = HashMap::new();

    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            let hv = raw_hist.entry(id).or_default();
            hv.insert(0, delta);
            hv.truncate(max_depth);
            for &g in LADDER.iter() {
                for &r in depths {
                    if hv.len() < r { continue; }
                    let codes: Vec<u64> = hv[..r].iter().map(|&x| g.bucket(x)).collect();
                    let cls = micro_class(d.label.0 as i64, &codes);
                    inv.entry((g, r)).or_default().entry(cls).or_insert(is_carrier);
                    let hh = half.entry((g, r)).or_default();
                    if step * 2 <= STEPS { hh.0.insert(cls); } else { hh.1.insert(cls); }
                }
            }
            prev_e.insert(id, e);
        }
    }

    let mut out: HashMap<(Gran, usize), Shape> = HashMap::new();
    for (key, classes) in inv {
        // the registry the TPM would build includes the VACUUM state (substrate side)
        let k_carrier = classes.values().filter(|&&c| c).count();
        let k_substrate = classes.len() - k_carrier + 1;
        let m = classes.len() + 1;
        let (a, b) = half.get(&key).cloned().unwrap_or_default();
        let inter = a.intersection(&b).count() as f64;
        let union = a.union(&b).count().max(1) as f64;
        out.insert(key, Shape {
            m, k_carrier, k_substrate,
            support: null_support(m, k_carrier),
            half_jaccard: inter / union,
            carrier_class_ids: classes.iter().filter(|(_, &c)| c).map(|(&h, _)| h).collect(),
        });
    }
    out
}

// ------------------------------------------------- added structural diagnostics (C1.1b/C1.1c)
//
// NOT pre-registered, and they CANNOT change C1.2's branch — that branch is decided by
// k_carrier alone and is already determined. They are added because "many carrier classes"
// is only meaningful if (b) those classes carry realised mass rather than being singletons,
// and (c) the granularity knob genuinely fails to split them rather than coincidentally
// producing the same COUNT. Both are partition structure; neither computes a score.

pub struct Refine {
    /// carrier-class occupancy in CARRIER NODE-STEPS, descending
    pub occupancy: HashMap<Gran, Vec<u64>>,
    /// (coarse, fine, number of coarse carrier classes that split, total coarse classes)
    pub splits: Vec<(Gran, Gran, usize, usize)>,
    /// C1.1d root cause: the DISTINCT raw energy-delta values a carrier ever takes, with counts.
    /// If this set is tiny, no binning refinement can split anything, and the saturation above
    /// is mechanical rather than a coincidence of the chosen cut points.
    pub raw_deltas: Vec<(f64, u64)>,
}

/// One run; every granularity's carrier class recorded per carrier node-step, so both the
/// occupancy distribution and the refinement relation come from the SAME observations.
pub fn refine_and_occupancy(seed: u64, arm: Arm, depth: usize) -> Refine {
    let cfg = config(seed);
    let rules = build_rules(&cfg, arm, false);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let (_, _, _, markers) = arm_spec(arm);
    plant_markers(&mut store, seed, markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut prev_e: HashMap<NodeId, f64> = HashMap::new();
    let mut raw_hist: HashMap<NodeId, Vec<f64>> = HashMap::new();
    let mut occ: HashMap<Gran, HashMap<u64, u64>> = HashMap::new();
    // (coarse gran, fine gran) -> coarse class -> set of fine classes it contains
    let mut pair: HashMap<(usize, usize), HashMap<u64, HashSet<u64>>> = HashMap::new();
    // C1.1d: distinct raw carrier deltas, keyed at 1e-12 so float noise cannot inflate the count
    let mut raw: HashMap<i64, u64> = HashMap::new();

    for _ in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        for (id, d) in snap.nodes() {
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            let hv = raw_hist.entry(id).or_default();
            hv.insert(0, delta);
            hv.truncate(depth);
            if d.label == CARRIER_LABEL {
                *raw.entry((delta * 1e12).round() as i64).or_insert(0) += 1;
            }
            if d.label == CARRIER_LABEL && hv.len() == depth {
                let cls: Vec<u64> = LADDER.iter().map(|g| {
                    let codes: Vec<u64> = hv[..depth].iter().map(|&x| g.bucket(x)).collect();
                    micro_class(d.label.0 as i64, &codes)
                }).collect();
                for (i, &c) in cls.iter().enumerate() {
                    *occ.entry(LADDER[i]).or_default().entry(c).or_insert(0) += 1;
                    if i + 1 < cls.len() {
                        pair.entry((i, i + 1)).or_default().entry(c).or_default().insert(cls[i + 1]);
                    }
                }
            }
            prev_e.insert(id, e);
        }
    }

    let occupancy = occ.into_iter().map(|(g, m)| {
        let mut v: Vec<u64> = m.into_values().collect();
        v.sort_unstable_by(|a, b| b.cmp(a));
        (g, v)
    }).collect();
    let mut splits: Vec<(Gran, Gran, usize, usize)> = pair.into_iter().map(|((i, j), m)| {
        let n_split = m.values().filter(|s| s.len() > 1).count();
        (LADDER[i], LADDER[j], n_split, m.len())
    }).collect();
    splits.sort_by_key(|(a, _, _, _)| LADDER.iter().position(|g| g == a).unwrap());
    let mut raw_deltas: Vec<(f64, u64)> = raw.into_iter().map(|(k, v)| (k as f64 / 1e12, v)).collect();
    raw_deltas.sort_unstable_by(|a, b| b.1.cmp(&a.1));
    Refine { occupancy, splits, raw_deltas }
}

/// K_MIN, DERIVED: the number of carrier WL classes arm A exhibits at the frozen
/// structural depth 2 — the resolution of the structural instrument the metabolic one is
/// tabled against in Phase 2. Baseline-relative, per the N0 v1 failure.
fn derive_k_min(seed: u64) -> (usize, usize) {
    let cfg = config(seed);
    let rules = build_rules(&cfg, Arm::A, false);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    plant_markers(&mut store, seed, 1);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);
    let mut wl_c: HashMap<u64, bool> = HashMap::new();
    wl_c.insert(VACUUM.0, false);
    for _ in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        for (id, d) in snap.nodes() {
            let wh = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            wl_c.entry(wh).or_insert(d.label == CARRIER_LABEL);
        }
    }
    (wl_c.values().filter(|&&c| c).count(), wl_c.len())
}

// ------------------------------------------------- C1.3-R clause 2: the spread guard
//
// NULL VALUES ONLY. The real partition's score is never computed here. Null CE values
// come from random same-sized partitions and are independent of which classes are
// actually carriers, so this reveals nothing about any arm's result.

fn ce_for(tpm: &TpmAccumulator, g: &[usize], k: usize) -> f64 {
    let rows = tpm.normalized_rows();
    let micro = ei_from_tpm(&rows.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>());
    let mac = ei_from_tpm(&coarse_grain(&rows, g, k));
    if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 }
}

pub struct NullSpread { pub sd: f64, pub min: f64, pub max: f64, pub distinct: usize, pub skipped: bool }
impl NullSpread {
    pub fn ok(&self) -> bool {
        !self.skipped && self.sd > 0.0 && (self.max - self.min) > 0.0 && self.distinct >= DISTINCT_MIN
    }
}

/// Build the metabolic TPM at one (granularity, depth) and return ONLY the null spread.
pub fn null_spread(seed: u64, arm: Arm, g: Gran, depth: usize) -> NullSpread {
    let cfg = config(seed);
    let rules = build_rules(&cfg, arm, false);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let (_, _, _, markers) = arm_spec(arm);
    plant_markers(&mut store, seed, markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut tpm = TpmAccumulator::new();
    let mut is_c: HashMap<u64, bool> = HashMap::new();
    is_c.insert(VACUUM.0, false);
    let mut prev_e: HashMap<NodeId, f64> = HashMap::new();
    let mut hist: HashMap<NodeId, Vec<u64>> = HashMap::new();
    let mut prev_cls: Option<HashMap<NodeId, u64>> = None;

    for _ in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let mut now: HashMap<NodeId, u64> = HashMap::new();
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            let hv = hist.entry(id).or_default();
            hv.insert(0, g.bucket(delta));
            hv.truncate(depth);
            if hv.len() == depth {
                let cls = micro_class(d.label.0 as i64, hv);
                now.insert(id, cls);
                is_c.entry(cls).or_insert(is_carrier);
            }
            prev_e.insert(id, e);
        }
        // SORTED-order recording (the N1.3 fix)
        if let Some(p) = &prev_cls {
            let mut carry: Vec<(NodeId, u64)> = p.iter().map(|(k, v)| (*k, *v)).collect();
            let mut born: Vec<(NodeId, u64)> = now.iter().filter(|(id, _)| !p.contains_key(id)).map(|(k, v)| (*k, *v)).collect();
            carry.sort_unstable();
            born.sort_unstable();
            for (id, hp) in carry { tpm.record(WlHash(hp), WlHash(now.get(&id).copied().unwrap_or(VACUUM.0))); }
            for (_, hn) in born { tpm.record(VACUUM, WlHash(hn)); }
        }
        prev_cls = Some(now);
    }

    let m = tpm.hashes().len();
    if m > M_CAP || m < 2 {
        return NullSpread { sd: 0.0, min: 0.0, max: 0.0, distinct: 0, skipped: true };
    }
    let mut compact: HashMap<usize, usize> = HashMap::new();
    let group_of: Vec<usize> = tpm.hashes().iter().map(|h| {
        let raw = *is_c.get(&h.0).unwrap_or(&false) as usize;
        let n = compact.len();
        *compact.entry(raw).or_insert(n)
    }).collect();
    let k = compact.len().max(1);
    if k < 2 { return NullSpread { sd: 0.0, min: 0.0, max: 0.0, distinct: 0, skipped: false }; }

    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7 ^ seed;
    let (mut sum, mut sumsq, mut mn, mut mx) = (0.0f64, 0.0f64, f64::MAX, f64::MIN);
    let mut seen: HashSet<u64> = HashSet::new();
    for _ in 0..N_PERM {
        let mut labels: Vec<usize> = canon.iter().map(|&i| group_of[i]).collect();
        for i in (1..labels.len()).rev() {
            let j = (splitmix(&mut rng) % (i as u64 + 1)) as usize;
            labels.swap(i, j);
        }
        let mut perm = vec![0usize; group_of.len()];
        for (pos, &i) in canon.iter().enumerate() { perm[i] = labels[pos]; }
        let c = ce_for(&tpm, &perm, k);
        sum += c; sumsq += c * c; mn = mn.min(c); mx = mx.max(c);
        seen.insert(c.to_bits());
    }
    let n = N_PERM as f64;
    let mean = sum / n;
    NullSpread { sd: (sumsq / n - mean * mean).max(0.0).sqrt(), min: mn, max: mx, distinct: seen.len(), skipped: false }
}

// ---------------------------------------------------------------------------- the test

const LIVE: [(&str, Arm); 5] = [
    ("A  replicating", Arm::A), ("B  motile static", Arm::B), ("B' motile diffuse", Arm::BPrime),
    ("C  token", Arm::C), ("D  stationary", Arm::D),
];

#[test]
fn c1_metabolic_partition_resolution() {
    let seed = 12345;
    eprintln!("\n================ V-TRACK CLOSEOUT / C1 — METABOLIC PARTITION RESOLUTION ================");
    eprintln!("  Pre-registration f99d423, committed before this file existed.");
    eprintln!("  This file computes NO CE, NO closure defect and NO real score (see module header).\n");

    // ---------------- K_MIN, derived ----------------
    let (k_min, wl_m) = derive_k_min(seed);
    eprintln!("--- K_MIN (DERIVED, not chosen: arm A's carrier WL classes at frozen structural depth 2) ---");
    eprintln!("  arm A structural registry: M = {wl_m}, carrier classes = {k_min}");
    eprintln!("  => K_MIN = {k_min}. The metabolic partition must resolve carriers at least this finely,");
    eprintln!("     or the Phase-2 two-channel table is not comparing like with like.\n");

    // ---------------- C1.1: the F0.2 table, one level up ----------------
    let depths = [1usize, 2, 3];
    let mut all: HashMap<(Arm, Gran, usize), Shape> = HashMap::new();
    let arms: Vec<(&str, Arm)> = LIVE.iter().cloned().chain([("inert (CONTROL)", Arm::Inert)]).collect();
    for (_, a) in &arms {
        for (key, s) in shapes(seed, *a, false, &depths) { all.insert((*a, key.0, key.1), s); }
    }

    eprintln!("--- C1.1  metabolic class count and GROUP SIZES, by granularity and depth ---");
    eprintln!("  (F0's pathology was group sizes (9, 1): ONE carrier class against nine substrate classes)\n");
    for &g in LADDER.iter() {
        eprintln!("  granularity {} ({} codes)", g.name(), g.codes());
        eprintln!("    arm                      | R | total M | (substrate, CARRIER) | support   | half-Jaccard");
        eprintln!("    -------------------------+---+---------+----------------------+-----------+-------------");
        for (n, a) in &arms {
            for &r in &depths {
                let s = match all.get(&(*a, g, r)) { Some(s) => s, None => continue };
                eprintln!("    {:<24} | {} | {:>7} | ({:>4}, {:>5})        | {:>9.2e} | {:>11.3}",
                    n, r, s.m, s.k_substrate, s.k_carrier, s.support, s.half_jaccard);
            }
        }
        eprintln!();
    }

    // ---------------- C1.2: the pre-committed reading, at the FROZEN setting ----------------
    eprintln!("================ C1.2 — THE PRE-COMMITTED READING (frozen instrument: G_cur, R = {FROZEN_DEPTH}) ================");
    eprintln!("  arm                      | carrier classes at the frozen setting");
    let mut frozen_kc: Vec<(&str, usize)> = Vec::new();
    for (n, a) in &arms {
        let kc = all.get(&(*a, Gran::Cur, FROZEN_DEPTH)).map(|s| s.k_carrier).unwrap_or(0);
        eprintln!("  {:<24} | {}", n, kc);
        if LIVE.iter().any(|(ln, _)| ln == n) { frozen_kc.push((n, kc)); }
    }
    let min_live_kc = frozen_kc.iter().map(|(_, k)| *k).min().unwrap_or(0);
    let max_live_kc = frozen_kc.iter().map(|(_, k)| *k).max().unwrap_or(0);
    eprintln!("\n  live-arm carrier class count: min {min_live_kc}, max {max_live_kc}   (K_MIN = {k_min})");
    if max_live_kc <= 2 {
        eprintln!("  ⟹ BRANCH: ~2 CLASSES. The (9,1)-shaped degeneracy is recurring one level up; the");
        eprintln!("     metabolic channel's failure to separate the ladder is an INSTRUMENT artifact. REOPEN per C2.");
    } else if min_live_kc >= k_min {
        eprintln!("  ⟹ BRANCH: MANY CLASSES that all score alike. The frozen instrument already resolves carriers");
        eprintln!("     at least as finely as the structural one it is tabled against, so the coarseness is");
        eprintln!("     SUBSTANTIVE — the channel genuinely does not differentiate live carriers. TRACK CLOSES.");
    } else {
        eprintln!("  ⟹ BRANCH: INTERMEDIATE. Report class count against realised spread; bound the claim by both.");
    }

    // ---------------- C1.3-R: the selection rule, applied ----------------
    eprintln!("\n================ C1.3-R — GRANULARITY SELECTION (structure only; rule fixed at f99d423) ================");
    eprintln!("  minimum G, coarse->fine, with every live arm at k_carrier >= K_MIN = {k_min}, k_carrier < M,");
    eprintln!("  support > 100, and the realised-spread guard (NULL values only — no real score is computed).\n");
    eprintln!("  G        | live-arm k_carrier (A, B, B', C, D) | min k_c | k<M & support>100 | spread guard | passes");
    eprintln!("  ---------+-------------------------------------+---------+-------------------+--------------+-------");
    let mut selected: Option<Gran> = None;
    for &g in LADDER.iter() {
        let kcs: Vec<usize> = LIVE.iter()
            .map(|(_, a)| all.get(&(*a, g, FROZEN_DEPTH)).map(|s| s.k_carrier).unwrap_or(0)).collect();
        let mink = kcs.iter().cloned().min().unwrap_or(0);
        let struct_ok = LIVE.iter().all(|(_, a)| {
            all.get(&(*a, g, FROZEN_DEPTH)).map(|s| s.k_carrier < s.m && s.support > 100.0).unwrap_or(false)
        });
        let clause1 = mink >= k_min;
        // clause 2's spread guard is only evaluated when the cheap structural clauses pass
        let (spread_txt, spread_ok) = if clause1 && struct_ok {
            let mut ok = true;
            let mut worst = String::new();
            for (n, a) in LIVE.iter() {
                let ns = null_spread(seed, *a, g, FROZEN_DEPTH);
                if !ns.ok() { ok = false; worst = format!("{} fails", n.split_whitespace().next().unwrap()); }
            }
            (if ok { "ok".to_string() } else { worst }, ok)
        } else { ("(not reached)".to_string(), false) };
        let passes = clause1 && struct_ok && spread_ok;
        eprintln!("  {:<8} | {:>35} | {:>7} | {:>17} | {:<12} | {}",
            g.name(), format!("{kcs:?}"), mink, struct_ok, spread_txt, passes);
        if passes && selected.is_none() { selected = Some(g); }
    }

    eprintln!();
    let rank = |g: Gran| LADDER.iter().position(|x| *x == g).unwrap();
    let cur_passes = {
        let kcs_ok = LIVE.iter().all(|(_, a)| all.get(&(*a, Gran::Cur, FROZEN_DEPTH))
            .map(|s| s.k_carrier >= k_min && s.k_carrier < s.m && s.support > 100.0).unwrap_or(false));
        kcs_ok
    };
    match selected {
        Some(g) if g == Gran::Cur => {
            eprintln!("  ⟹ C1.3-R SELECTS {} — the FROZEN instrument. It was not under-resolved.", g.name());
            eprintln!("     C1's confirming branch holds and C2 DOES NOT RUN.");
        }
        Some(g) if rank(g) < rank(Gran::Cur) => {
            eprintln!("  ⟹ C1.3-R SELECTS {} — COARSER than the frozen G_cur, and G_cur itself passes ({cur_passes}).", g.name());
            eprintln!("     The rule takes the MINIMUM passing granularity, and the frozen instrument sits ABOVE it.");
            eprintln!("     Under-resolution is therefore excluded a fortiori: the bar is cleared by a partition");
            eprintln!("     strictly coarser than the one actually used. C2 DOES NOT RUN.");
        }
        Some(g) => {
            eprintln!("  ⟹ C1.3-R SELECTS {} — FINER than the frozen G_cur. The frozen instrument was", g.name());
            eprintln!("     under-resolved against its own baseline bar. C2 RUNS on this candidate.");
        }
        None => {
            eprintln!("  ⟹ NO CANDIDATE REACHES K_MIN at any granularity on the ladder. The metabolic partition");
            eprintln!("     cannot be resolved to the structural instrument's own resolution, so C2.3's branch");
            eprintln!("     applies: the channel is uniform, by the longer route.");
        }
    }

    // ---------------- C1.1b / C1.1c: added structural diagnostics ----------------
    eprintln!("\n================ ADDED STRUCTURAL DIAGNOSTICS (not pre-registered; cannot move C1.2's branch) ================");
    eprintln!("  C1.2's branch is decided by k_carrier alone and is already fixed above. These two ask whether");
    eprintln!("  that verdict deserves to be believed: (b) do the carrier classes carry realised MASS, or are");
    eprintln!("  they singletons? (c) does the granularity knob genuinely FAIL to split them, or does it merely");
    eprintln!("  coincide on the COUNT?\n");
    for (n, a) in LIVE.iter() {
        let r = refine_and_occupancy(seed, *a, FROZEN_DEPTH);
        let occ = r.occupancy.get(&Gran::Cur).cloned().unwrap_or_default();
        let tot: u64 = occ.iter().sum();
        let top = occ.first().copied().unwrap_or(0);
        let singletons = occ.iter().filter(|&&c| c == 1).count();
        eprintln!("  {n}");
        eprintln!("    C1.1b occupancy at G_cur (carrier node-steps per class, desc): {:?}", occ);
        eprintln!("          total {tot}, largest class {:.1}% of mass, singleton classes {singletons}/{}",
            100.0 * top as f64 / tot.max(1) as f64, occ.len());
        eprint!("    C1.1c refinement:");
        for (cg, fg, n_split, n_coarse) in &r.splits {
            eprint!("  {}->{}: {}/{} split;", cg.name(), fg.name(), n_split, n_coarse);
        }
        eprintln!();
        let shown: Vec<String> = r.raw_deltas.iter().take(8)
            .map(|(v, c)| format!("{v:+.6}×{c}")).collect();
        eprintln!("    C1.1d distinct RAW carrier deltas: {} value(s) — {}{}",
            r.raw_deltas.len(), shown.join(", "),
            if r.raw_deltas.len() > 8 { ", …" } else { "" });
    }
    eprintln!("\n  NOTE: no property verdict is drawn here (pre-registration, Forbidden #4). C1 diagnoses the");
    eprintln!("  instrument only.\n");

    // the diagnostic must be reported whichever way it goes; the only hard assertion is
    // that the run actually produced carriers to classify
    assert!(all.get(&(Arm::A, Gran::Cur, FROZEN_DEPTH)).map(|s| s.k_carrier > 0).unwrap_or(false),
        "C1 vacuous: arm A produced no carrier metabolic classes");
}
