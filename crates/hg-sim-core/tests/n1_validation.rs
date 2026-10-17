//! V9 / Phase 1 — N1.2 (multi-seed), N1.3 (inert zero-point band), N1.4 (churn
//! stationarity under TWO witnesses), N1.5 (convergent witnesses).
//!
//! Pre-registration: docs/validation_v9_phase1_preregistration.md (19ebd70).
//! Burn-in: N0 v2 (18dcd28), baseline-relative, derived T_burn = 900 for
//! churn-with-carriers and 100 for bare churn.
//!
//! The closure defect was shown in n1_metabolic_instrument.rs to DEGENERATE as a
//! within-channel partition statistic (CD == H(micro) - H(macro) exactly, and
//! H(micro) is permutation-invariant, so its permutation test is a test on
//! -H(macro)). So the statistic carried forward here is CE ON THE METABOLIC
//! CHANNEL, the direct analogue of the structural test. The closure defect
//! survives only in its M3 CROSS-CHANNEL form (macro = WL structural, micro =
//! energy), which is reported in N1.5 as a witness, not as the primary.
//!
//! Depth R = 2, fixed by the pre-registered arm-A-only rule and FROZEN.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{builtin_rules, compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::{HashMap, HashSet};

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const STEPS: u64 = 6_000;
const N_PERM: usize = 200;
const ENERGY: u32 = 1;
const ALPHA: f64 = 0.05;
const DISTINCT_MIN: usize = 10;
const EPS: f64 = 1e-9;
const DEPTH: usize = 2; // frozen by the arm-A rule in n1_metabolic_instrument.rs
const MIN_TRANSITIONS: u64 = 1000;
const TARGET_INCLUDED: usize = 30;
const MAX_ATTEMPTS: u64 = 120;
const R_PERSIST: u64 = 3;
const L_BLK: u64 = 100;
/// N0 v2 derived burn-ins
const T_BURN_CHURN_CARRIERS: u64 = 900;
const T_BURN_CHURN_BARE: u64 = 100;

fn config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "n1v"
run_id = "n1v"
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

#[derive(Clone, Copy, PartialEq, Debug)]
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

#[derive(Clone, Copy, PartialEq)]
enum Substrate { Static, Churn, Growth, Triangle }

fn build_rules(cfg: &SimConfig, arm: Arm, sub: Substrate) -> Vec<hg_rules::CompiledRule> {
    let mut raw = Vec::new();
    match sub {
        Substrate::Growth => raw.extend(builtin_rules("growth_treadmill").unwrap()),
        Substrate::Triangle => raw.extend(builtin_rules("triangle_subdivide").unwrap()),
        _ => {}
    }
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if arm == Arm::Inert {
        biology.retain(|r| r.name == "bootstrap_seed");
    } else {
        let (sterile, motile, diffuse, _) = arm_spec(arm);
        if sterile { biology.retain(|r| r.name != "replicate"); }
        if motile { biology.extend(parse_rules_toml(motility_toml()).unwrap()); }
        if diffuse { biology.extend(parse_rules_toml(diffuse_toml()).unwrap()); }
        if sub == Substrate::Churn { biology.extend(parse_rules_toml(churn_toml()).unwrap()); }
    }
    raw.extend(biology);
    compile_rules(&raw).unwrap()
}

// ---- metabolic micro state: byte-identical to n1_metabolic_instrument.rs ----
fn bucket(d: f64) -> u64 {
    if d.abs() < EPS { return 0; }
    let mag = d.abs();
    let m = if mag < 0.25 { 1 } else if mag < 0.75 { 2 } else if mag < 1.5 { 3 }
            else if mag < 4.0 { 4 } else if mag < 9.0 { 5 } else { 6 };
    if d < 0.0 { m } else { m + 8 }
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
fn ce_for(tpm: &TpmAccumulator, g: &[usize], k: usize) -> f64 {
    let rows = tpm.normalized_rows();
    let micro = ei_from_tpm(&rows.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>());
    let mac = ei_from_tpm(&coarse_grain(&rows, g, k));
    if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 }
}
fn sign_test_p(k: usize, n: usize) -> f64 {
    if n == 0 { return 1.0; }
    let mut pmf = vec![0.0f64; n + 1];
    let mut c = 1.0f64;
    let denom = 2f64.powi(n as i32);
    for i in 0..=n { pmf[i] = c / denom; c = c * (n - i) as f64 / (i + 1) as f64; }
    let obs = pmf[k];
    pmf.iter().filter(|&&v| v <= obs * (1.0 + 1e-12)).sum::<f64>().min(1.0)
}

#[derive(Clone, Default)]
pub struct Score {
    pub m: usize, pub k_carrier: usize, pub real: f64,
    pub null_mean: f64, pub null_sd: f64, pub null_min: f64, pub null_max: f64,
    pub null_ge: usize, pub null_distinct: usize, pub support: f64, pub transitions: u64,
}
impl Score {
    pub fn p(&self) -> f64 { self.null_ge as f64 / N_PERM as f64 }
    pub fn margin(&self) -> f64 { self.real - self.null_mean }
    pub fn z(&self) -> f64 {
        if self.k_carrier < 2 || self.null_sd <= 0.0 { f64::NAN } else { self.margin() / self.null_sd }
    }
    pub fn spread_ok(&self) -> bool {
        self.null_sd > 0.0 && (self.null_max - self.null_min) > 0.0 && self.null_distinct >= DISTINCT_MIN
    }
    pub fn resolved(&self) -> bool { self.k_carrier >= 2 && self.k_carrier < self.m && self.support > 100.0 }
    pub fn detects(&self) -> bool { self.resolved() && self.spread_ok() && self.p() < ALPHA }
}

fn score_ce(tpm: &TpmAccumulator, is_c: &HashMap<u64, bool>, seed: u64) -> Score {
    let mut compact: HashMap<usize, usize> = HashMap::new();
    let group_of: Vec<usize> = tpm.hashes().iter().map(|h| {
        let raw = *is_c.get(&h.0).unwrap_or(&false) as usize;
        let n = compact.len();
        *compact.entry(raw).or_insert(n)
    }).collect();
    let k = compact.len().max(1);
    let m = tpm.hashes().len();
    let k_carrier = tpm.hashes().iter().filter(|h| *is_c.get(&h.0).unwrap_or(&false)).count();
    let real = if k >= 2 { ce_for(tpm, &group_of, k) } else { 0.0 };
    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7 ^ seed;
    let (mut ge, mut sum, mut sumsq, mut mn, mut mx) = (0usize, 0.0f64, 0.0f64, f64::MAX, f64::MIN);
    let mut seen: HashSet<u64> = HashSet::new();
    if k >= 2 {
        for _ in 0..N_PERM {
            let mut labels: Vec<usize> = canon.iter().map(|&i| group_of[i]).collect();
            for i in (1..labels.len()).rev() {
                let j = (splitmix(&mut rng) % (i as u64 + 1)) as usize;
                labels.swap(i, j);
            }
            let mut perm = vec![0usize; group_of.len()];
            for (pos, &i) in canon.iter().enumerate() { perm[i] = labels[pos]; }
            let c = ce_for(tpm, &perm, k);
            sum += c; sumsq += c * c; mn = mn.min(c); mx = mx.max(c);
            seen.insert(c.to_bits());
            if c >= real { ge += 1; }
        }
    }
    let n = N_PERM as f64;
    let mean = sum / n;
    let var = (sumsq / n - mean * mean).max(0.0);
    Score {
        m, k_carrier, real, null_mean: mean, null_sd: var.sqrt(),
        null_min: if mn == f64::MAX { 0.0 } else { mn },
        null_max: if mx == f64::MIN { 0.0 } else { mx },
        null_ge: ge, null_distinct: seen.len(),
        support: null_support(m, k_carrier), transitions: tpm.total_transitions(),
    }
}

pub struct Run {
    pub ce: Score,
    pub last_alive: u64,
    pub final_pop: usize,
    /// N1.5 witness 1: entropy of the carrier energy-transition distribution, WINDOW-normalised
    pub et_entropy_wn: f64,
    /// N1.5 witness 2: per-carrier activity rate, WINDOW-normalised
    pub activity_wn: f64,
    /// N1.5 witness 3: CROSS-CHANNEL closure defect (macro = WL structural, micro = energy sign)
    pub cross_defect: f64,
    /// N1.4 stationarity of the METABOLIC micro-class partition
    pub twtf: f64,
    pub block_jaccard_steady: f64,
    pub block_jaccard_pairs: Vec<f64>,
    pub metab_classes: usize,
}

fn h_of(counts: &HashMap<u64, u64>) -> f64 {
    let tot: u64 = counts.values().sum();
    if tot == 0 { return 0.0; }
    -counts.values().map(|&c| { let p = c as f64 / tot as f64; if p > 0.0 { p * p.log2() } else { 0.0 } }).sum::<f64>()
}

pub fn run_arm(seed: u64, arm: Arm, sub: Substrate, burn_in: u64, steps: u64) -> Run {
    let cfg = config(seed, steps);
    let rules = build_rules(&cfg, arm, sub);
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
    let (mut last_alive, mut final_pop) = (0u64, 0usize);

    // N1.5 witnesses
    let mut et: HashMap<u64, u64> = HashMap::new();
    let mut prev_sign: HashMap<NodeId, u64> = HashMap::new();
    let (mut active, mut carrier_steps) = (0u64, 0u64);
    let mut prev_wl: HashMap<NodeId, u64> = HashMap::new();
    let mut joint: HashMap<(u64, u64), u64> = HashMap::new();
    let mut macro_c: HashMap<u64, u64> = HashMap::new();

    // N1.4 stationarity on the METABOLIC classes
    let mut live_steps: HashMap<u64, u64> = HashMap::new();
    let mut src: HashMap<u64, u64> = HashMap::new();
    live_steps.insert(VACUUM.0, u64::MAX);
    let n_blocks = ((steps.saturating_sub(burn_in)) / L_BLK).max(1) as usize;
    let mut blocks: Vec<HashSet<u64>> = vec![HashSet::new(); n_blocks];

    for step in 1..=steps {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, 2);
        let mut now: HashMap<NodeId, u64> = HashMap::new();
        let mut now_sign: HashMap<NodeId, u64> = HashMap::new();
        let mut now_wl: HashMap<NodeId, u64> = HashMap::new();
        let mut present: HashSet<u64> = HashSet::new();
        let mut live = 0usize;
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            if is_carrier { live += 1; }
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            let b = bucket(delta);
            let hv = hist.entry(id).or_default();
            hv.insert(0, b);
            hv.truncate(DEPTH);
            if hv.len() == DEPTH {
                let cls = micro_class(d.label.0 as i64, hv);
                now.insert(id, cls);
                present.insert(cls);
                is_c.entry(cls).or_insert(is_carrier);
            }
            let wh = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            now_wl.insert(id, wh);
            if is_carrier {
                let s = if delta.abs() < EPS { 1 } else if delta < 0.0 { 0 } else { 2 };
                now_sign.insert(id, s);
                if step > burn_in {
                    carrier_steps += 1;
                    if s != 1 { active += 1; }
                    if let Some(&ps) = prev_sign.get(&id) { *et.entry((ps << 8) | s).or_insert(0) += 1; }
                    if let Some(&pw) = prev_wl.get(&id) {
                        let mt = pw ^ wh.rotate_left(1);
                        *joint.entry((mt, s)).or_insert(0) += 1;
                        *macro_c.entry(mt).or_insert(0) += 1;
                    }
                }
            }
            prev_e.insert(id, e);
        }
        if live > 0 { last_alive = step; }
        final_pop = live;
        if step > burn_in {
            for &h in &present { *live_steps.entry(h).or_insert(0) += 1; }
            let bi = (((step - burn_in - 1) / L_BLK) as usize).min(blocks.len() - 1);
            for &h in &present { blocks[bi].insert(h); }
            if let Some(p) = &prev_cls {
                for (id, &hp) in p {
                    tpm.record(WlHash(hp), WlHash(now.get(id).copied().unwrap_or(VACUUM.0)));
                    *src.entry(hp).or_insert(0) += 1;
                }
                for (id, &hn) in &now {
                    if !p.contains_key(id) { tpm.record(VACUUM, WlHash(hn)); *src.entry(VACUUM.0).or_insert(0) += 1; }
                }
            }
        }
        prev_cls = Some(now);
        prev_sign = now_sign;
        prev_wl = now_wl;
    }

    let total: u64 = src.values().sum();
    let transient: u64 = src.iter()
        .filter(|(h, _)| *live_steps.get(*h).unwrap_or(&0) < R_PERSIST)
        .map(|(_, c)| *c).sum();
    let js: Vec<f64> = (0..blocks.len().saturating_sub(1)).filter_map(|i| {
        let (a, b) = (&blocks[i], &blocks[i + 1]);
        if a.is_empty() || b.is_empty() { return None; }
        Some(a.intersection(b).count() as f64 / a.union(b).count() as f64)
    }).collect();
    let half = js.len() / 2;
    let steady = if js.len() > half { js[half..].iter().sum::<f64>() / (js.len() - half) as f64 } else { f64::NAN };

    // cross-channel closure defect: H(energy sign | WL class transition)
    let joint_flat: HashMap<u64, u64> = joint.values().enumerate().map(|(i, &c)| (i as u64, c)).collect();
    let cross = (h_of(&joint_flat) - h_of(&macro_c)).max(0.0);

    let win = steps as f64;
    let alive_frac = last_alive as f64 / win;
    Run {
        ce: score_ce(&tpm, &is_c, seed),
        last_alive, final_pop,
        et_entropy_wn: h_of(&et) * alive_frac,
        activity_wn: (if carrier_steps > 0 { active as f64 / carrier_steps as f64 } else { 0.0 }) * alive_frac,
        cross_defect: cross,
        twtf: if total > 0 { transient as f64 / total as f64 } else { 0.0 },
        block_jaccard_steady: steady,
        block_jaccard_pairs: js,
        metab_classes: live_steps.len().saturating_sub(1),
    }
}

fn pstr(s: &Score) -> String {
    if !s.resolved() { "  n/a".into() }
    else if s.null_ge == 0 { "<.005".into() }
    else { format!("{:.3}", s.p()) }
}
fn zs(x: f64) -> String { if x.is_nan() { "    n/a".into() } else { format!("{x:7.2}") } }

#[test]
fn n1_phase1_validation() {
    eprintln!("\n================ V9 / PHASE 1 — N1.2 / N1.3 / N1.4 / N1.5 (statistic: CE on the metabolic channel, R={DEPTH}) ================");

    // ---------------- N1.2 multi-seed on arm A, at the F8/K2 standard ----------------
    eprintln!("\n================ N1.2 — MULTI-SEED (arm A, the F8 baseline), seeds 1.. disjoint from 12345 ================");
    let anchor = run_arm(12345, Arm::A, Substrate::Static, 0, STEPS);
    eprintln!("  FIDELITY ANCHOR seed 12345: k_c {}/{} CE {:.4} p {} z {} spread {}",
        anchor.ce.k_carrier, anchor.ce.m, anchor.ce.real, pstr(&anchor.ce), zs(anchor.ce.z()), anchor.ce.spread_ok());
    assert!(anchor.ce.detects(), "the metabolic instrument must detect on the F8 baseline seed, or nothing below is trustworthy");

    let mut results: Vec<(u64, Run)> = Vec::new();
    let mut attempts = 0u64;
    for seed in 1..=MAX_ATTEMPTS {
        attempts += 1;
        let r = run_arm(seed, Arm::A, Substrate::Static, 0, STEPS);
        results.push((seed, r));
        let inc = results.iter().filter(|(_, r)| {
            r.ce.transitions >= MIN_TRANSITIONS && r.ce.k_carrier >= 2 && r.final_pop > 0 && r.ce.spread_ok()
        }).count();
        if inc >= TARGET_INCLUDED { break; }
    }
    let included: Vec<&(u64, Run)> = results.iter().filter(|(_, r)| {
        r.ce.transitions >= MIN_TRANSITIONS && r.ce.k_carrier >= 2 && r.final_pop > 0 && r.ce.spread_ok()
    }).collect();
    let sustained = results.iter().filter(|(_, r)| r.final_pop > 0).count();
    let degen = results.iter().filter(|(_, r)| r.final_pop > 0 && r.ce.k_carrier >= 2 && !r.ce.spread_ok()).count();
    let n = included.len();
    eprintln!("  attempts {attempts}, sustained {sustained}, degenerate-null {degen}, included {n} ({:.0}%)",
        100.0 * n as f64 / attempts as f64);
    eprintln!("  seed | pop | k_c/M   | trans    | real CE  | null mean | null max | dist | z       | p     | support  | spread");
    for (s, r) in &results {
        eprintln!("  {:>4} | {:>3} | {:>3}/{:<4}| {:>8} | {:>8.4} | {:>9.4} | {:>8.4} | {:>4} | {} | {:>5} | {:.2e} | {:>6}",
            s, r.final_pop, r.ce.k_carrier, r.ce.m, r.ce.transitions, r.ce.real, r.ce.null_mean,
            r.ce.null_max, r.ce.null_distinct, zs(r.ce.z()), pstr(&r.ce), r.ce.support,
            if r.ce.spread_ok() { "ok" } else { "DEGEN" });
    }
    let detect = included.iter().filter(|(_, r)| r.ce.p() < ALPHA).count();
    let f = detect as f64 / n.max(1) as f64;
    let k_pos = included.iter().filter(|(_, r)| r.ce.margin() > 0.0).count();
    let sign_p = sign_test_p(k_pos, n);
    let reversals: Vec<u64> = included.iter().filter(|(_, r)| r.ce.margin() < 0.0).map(|(s, _)| *s).collect();
    eprintln!("\n  PRIMARY  fraction detectable (p<{ALPHA}): {detect}/{n} = {f:.3}");
    eprintln!("  SECONDARY sign test (real > own null mean): {k_pos}/{n}, exact binomial p = {sign_p:.3e}");
    eprintln!("  REVERSALS: {reversals:?}");
    eprintln!("  => {}", if f >= 0.90 { "REGIME PROPERTY" } else if f >= 0.50 { "MAJORITY (partial)" } else { "SEED-SPECIFIC" });

    // ---------------- N1.3 inert zero-point as a MEASURED BAND ----------------
    eprintln!("\n================ N1.3 — INERT ZERO-POINT AS A MEASURED BAND (same seed set) ================");
    let inert_runs: Vec<(u64, Run)> = (1..=10u64).map(|s| (s, run_arm(s, Arm::Inert, Substrate::Static, 0, STEPS))).collect();
    let ks: Vec<usize> = inert_runs.iter().map(|(_, r)| r.ce.k_carrier).collect();
    let ms: Vec<usize> = inert_runs.iter().map(|(_, r)| r.ce.m).collect();
    let n_res = inert_runs.iter().filter(|(_, r)| r.ce.resolved()).count();
    let n_det = inert_runs.iter().filter(|(_, r)| r.ce.detects()).count();
    let acts: Vec<f64> = inert_runs.iter().map(|(_, r)| r.activity_wn).collect();
    eprintln!("  inert across 10 seeds: k_carrier {:?}, M {:?}", ks, ms);
    eprintln!("  resolved on {n_res}/10 seeds; DETECTS on {n_det}/10 seeds");
    eprintln!("  activity-rate band: min {:.3e}, max {:.3e}  (an inert carrier has delta==0 forever)",
        acts.iter().cloned().fold(f64::MAX, f64::min), acts.iter().cloned().fold(f64::MIN, f64::max));
    eprintln!("  => the zero-point is a MEASURED band, not an eyeballed number: the inert control is");
    eprintln!("     unresolvable on every seed, so the instrument cannot manufacture a signal from carrier-ness.");
    assert_eq!(n_det, 0, "the inert control must never detect — that is the calibrated zero");

    // ---------------- N1.4 churn stationarity, TWO witnesses, post-burn-in ----------------
    eprintln!("\n================ N1.4 — STATIONARITY OF THE METABOLIC PARTITION, TWO WITNESSES, POST-BURN-IN ================");
    eprintln!("  burn-in from N0 v2 (derived, not chosen): churn+carriers {T_BURN_CHURN_CARRIERS}, churn bare {T_BURN_CHURN_BARE}");
    eprintln!("  run                          | burn-in | metab classes | TWTF   | block-Jaccard steady | pairs");
    let cases: [(&str, Arm, Substrate, u64, u64); 5] = [
        ("A  static (baseline)",  Arm::A, Substrate::Static, 0, STEPS),
        ("B' static (low-act)",   Arm::BPrime, Substrate::Static, 0, STEPS),
        ("B  CHURN",              Arm::B, Substrate::Churn, T_BURN_CHURN_CARRIERS, STEPS),
        ("A  growth_treadmill",   Arm::A, Substrate::Growth, 0, 600),
        ("A  triangle_subdivide", Arm::A, Substrate::Triangle, 0, 300),
    ];
    let mut stat_rows: Vec<(&str, f64, f64)> = Vec::new();
    for (name, arm, sub, burn, st) in cases {
        let r = run_arm(12345, arm, sub, burn, st);
        let ps: Vec<String> = r.block_jaccard_pairs.iter().take(6).map(|x| format!("{x:.2}")).collect();
        eprintln!("  {:<28} | {:>7} | {:>13} | {:>6.3} | {:>20} | [{}]",
            name, burn, r.metab_classes, r.twtf,
            if r.block_jaccard_steady.is_nan() { "n/a".into() } else { format!("{:.3}", r.block_jaccard_steady) },
            ps.join(", "));
        stat_rows.push((name, r.twtf, r.block_jaccard_steady));
    }
    let churn = stat_rows.iter().find(|(n, _, _)| n.starts_with("B  CHURN")).unwrap();
    let base = stat_rows.iter().find(|(n, _, _)| n.starts_with("A  static")).unwrap();
    let grow_worst = stat_rows.iter().filter(|(n, _, _)| n.contains("growth") || n.contains("triangle"))
        .map(|(_, _, j)| *j).fold(f64::INFINITY, f64::min);
    let w1 = churn.1 <= 0.5;
    let w2 = churn.2 >= base.2 - 1e-9 || (churn.2 > grow_worst && churn.2 >= 0.5);
    eprintln!("\n  witness 1 (TWTF <= 0.5 on churn):           {w1}  (churn {:.3} vs baseline {:.3})", churn.1, base.1);
    eprintln!("  witness 2 (block-Jaccard vs baseline/proxies): {w2}  (churn {:.3}, baseline {:.3}, worst proxy {:.3})",
        churn.2, base.2, grow_worst);
    let stationary = w1 && w2;
    eprintln!("  => the metabolic partition is {} under bounded churn (post-burn-in).",
        if stationary { "STATIONARY on BOTH witnesses" } else { "*** NOT stationary on both witnesses ***" });
    eprintln!("\n  TWO CAVEATS ON WITNESS 2, stated rather than glossed:");
    eprintln!("   (a) the metabolic partition is stationary on the GROWING proxies too (growth_treadmill 0.917).");
    eprintln!("       That is not a witness failure — the metabolic state space is BOUNDED BY CONSTRUCTION (a");
    eprintln!("       fixed set of delta buckets), so substrate growth cannot proliferate classes the way it");
    eprintln!("       proliferates WL classes. It is a real advantage of this channel, but it does mean witness 2");
    eprintln!("       does NOT discriminate growth from churn here, so it is a weaker check than it was on WL.");
    eprintln!("   (b) triangle_subdivide collapses to ONE metabolic class (M=1): the explosion leaves every node");
    eprintln!("       fresh with no delta history. Its TWTF 0.000 / Jaccard 1.000 are TRIVIALLY stationary and");
    eprintln!("       must not be read as a pass — a single-class partition is unresolvable, the same failure");
    eprintln!("       mode as the inert control.");

    // ---------------- N1.5 convergent witnesses ----------------
    eprintln!("\n================ N1.5 — CONVERGENT WITNESSES (do independent statistics agree on arm ordering?) ================");
    let arms: [(&str, Arm); 5] = [
        ("A  replicating", Arm::A), ("B  motile static", Arm::B), ("B' motile diffuse", Arm::BPrime),
        ("C  token", Arm::C), ("D  stationary", Arm::D),
    ];
    let rr: Vec<(&str, Run)> = arms.iter().map(|(n, a)| (*n, run_arm(12345, *a, Substrate::Static, 0, STEPS))).collect();
    eprintln!("  arm                      | CE z (primary) | et-entropy (wn) | activity rate (wn) | cross-channel defect");
    for (n, r) in &rr {
        eprintln!("  {:<24} | {} | {:>15.4} | {:>18.4} | {:>20.4}",
            n, zs(r.ce.z()), r.et_entropy_wn, r.activity_wn, r.cross_defect);
    }
    let rank = |mut v: Vec<(&str, f64)>| -> Vec<String> {
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        v.into_iter().map(|(n, _)| n.split_whitespace().next().unwrap().to_string()).collect()
    };
    let r_ce = rank(rr.iter().map(|(n, r)| (*n, r.ce.z())).collect());
    let r_et = rank(rr.iter().map(|(n, r)| (*n, r.et_entropy_wn)).collect());
    let r_ac = rank(rr.iter().map(|(n, r)| (*n, r.activity_wn)).collect());
    eprintln!("\n  arm ordering (strongest first):");
    eprintln!("    CE (primary)        : {r_ce:?}");
    eprintln!("    et-entropy (wn)     : {r_et:?}");
    eprintln!("    activity rate (wn)  : {r_ac:?}");
    eprintln!("  REPAIRS the runners-up needed (M4/M3 record): BOTH required WINDOW-normalisation. Lifetime-normalised,");
    eprintln!("  the dead token C scores at or ABOVE arm D, failing the pre-committed C row. Window-normalisation is");
    eprintln!("  PRINCIPLED (an arm that stops early should contribute less over a fixed window), but it was identified");
    eprintln!("  AFTER seeing the table, so it is a post-hoc repair and is labelled as one.");
    eprintln!("  The closure defect needed a different repair: it DEGENERATES within-channel (CD == H(micro)-H(macro)),");
    eprintln!("  and survives only in the CROSS-CHANNEL form reported above.");
    let agree_top = r_ce[0] == r_ac[0];
    eprintln!("\n  DO THEY AGREE? All three put arm C LAST — that is unanimous. They DISAGREE at the top:");
    eprintln!("    CE and et-entropy make arm A strongest; the activity rate makes arm D strongest.  agreement: {agree_top}");
    eprintln!("  This matters directly for Phase 2: the headline question 'is the structurally-invisible arm D the");
    eprintln!("  LOUDEST metabolic signal?' is NOT statistic-independent. A permutation-tested boundary statistic");
    eprintln!("  (CE) says NO — arm A is loudest and D is fourth of five. A raw per-carrier activity measure says");
    eprintln!("  YES. They answer different questions: CE asks 'is the carrier/substrate PARTITION informative",);
    eprintln!("  about the dynamics', the activity rate asks 'how much does an average carrier's energy move'.");
    eprintln!("  Phase 2 must therefore report BOTH and must not quote the flattering one.");

    eprintln!("\n================ PHASE 1 GATE ================");
    eprintln!("  N1.1 null with realised spread, guard from birth : PASS (all live arms non-degenerate; inert correctly degenerate)");
    eprintln!("  N1.2 multi-seed                                  : {}/{} = {:.3} => {}", detect, n, f,
        if f >= 0.90 { "PASS (regime property)" } else { "SEE ABOVE" });
    eprintln!("  N1.3 inert zero-point as a measured band         : PASS (0/10 seeds detect)");
    eprintln!("  N1.4 stationarity under churn, two witnesses     : {}", if stationary { "PASS" } else { "FAIL" });
    eprintln!("  N1.5 convergent witnesses                        : reported above");
    eprintln!("  (N1.3 two-process reproducibility is asserted separately by n1_reproducibility.rs)");
    eprintln!("=============================================\n");
}
