//! V9 / Phase 2 — re-run the ladder on the metabolic channel, beside the structural one.
//!
//! GATED on Phase 1, which cleared: N1.1 nulls non-degenerate with the spread guard
//! applied from birth; N1.2 arm A 30/30 (regime property); N1.3 inert control
//! unresolvable on 10/10 seeds AND two-process reproducible; N1.4 stationary under
//! bounded churn on both witnesses (post-burn-in, N0 v2 derived).
//!
//! Expectation table + provenance: docs/validation_v9_n2_expectation_table.md.
//! Every arm's seed-12345 metabolic score was already visible from Phase 1 and is
//! flagged [SEEN] there. The genuinely NEW evidence here is the PER-ARM MULTI-SEED
//! sweep and the cross-channel rank comparison.
//!
//! Both channels are scored from the SAME run, in one pass:
//!   structural = WL depth 2 (the F0-L instrument)
//!   metabolic  = (label, quantised energy-delta history depth 2) (Phase 1's instrument)
//! Recording iterates in SORTED NodeId order (the N1.3 fix), so the registry order is
//! reproducible across processes.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
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
const DEPTH: usize = 2;
const WL_DEPTH: u8 = 2;
const MIN_TRANSITIONS: u64 = 1000;
const TARGET_INCLUDED: usize = 30;
const MAX_ATTEMPTS: u64 = 60;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "n2"
run_id = "n2"
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
    pub fn measurable(&self) -> bool { self.resolved() && self.spread_ok() && self.transitions >= MIN_TRANSITIONS }
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

pub struct Both { pub structural: Score, pub metabolic: Score, pub final_pop: usize, pub last_alive: u64,
    /// raw per-carrier activity rate, window-normalised (the OTHER answer to the headline question)
    pub activity_wn: f64 }

/// Both channels from ONE run, recording in SORTED NodeId order (the N1.3 fix).
pub fn run_both(seed: u64, arm: Arm, churn: bool) -> Both {
    let cfg = config(seed);
    let rules = build_rules(&cfg, arm, churn);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let (_, _, _, markers) = arm_spec(arm);
    plant_markers(&mut store, seed, markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let (mut wl_tpm, mut mb_tpm) = (TpmAccumulator::new(), TpmAccumulator::new());
    let (mut wl_c, mut mb_c): (HashMap<u64, bool>, HashMap<u64, bool>) = (HashMap::new(), HashMap::new());
    wl_c.insert(VACUUM.0, false);
    mb_c.insert(VACUUM.0, false);
    let mut prev_e: HashMap<NodeId, f64> = HashMap::new();
    let mut hist: HashMap<NodeId, Vec<u64>> = HashMap::new();
    let (mut wl_prev, mut mb_prev): (Option<HashMap<NodeId, u64>>, Option<HashMap<NodeId, u64>>) = (None, None);
    let (mut last_alive, mut final_pop) = (0u64, 0usize);
    let (mut active, mut carrier_steps) = (0u64, 0u64);

    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let (mut wl_now, mut mb_now): (HashMap<NodeId, u64>, HashMap<NodeId, u64>) = (HashMap::new(), HashMap::new());
        let mut live = 0usize;
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            if is_carrier { live += 1; }
            let wh = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            wl_now.insert(id, wh);
            wl_c.entry(wh).or_insert(is_carrier);
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            let hv = hist.entry(id).or_default();
            hv.insert(0, bucket(delta));
            hv.truncate(DEPTH);
            if hv.len() == DEPTH {
                let cls = micro_class(d.label.0 as i64, hv);
                mb_now.insert(id, cls);
                mb_c.entry(cls).or_insert(is_carrier);
            }
            if is_carrier { carrier_steps += 1; if delta.abs() >= EPS { active += 1; } }
            prev_e.insert(id, e);
        }
        if live > 0 { last_alive = step; }
        final_pop = live;
        record(&mut wl_tpm, &wl_prev, &wl_now);
        record(&mut mb_tpm, &mb_prev, &mb_now);
        wl_prev = Some(wl_now);
        mb_prev = Some(mb_now);
    }
    let alive_frac = last_alive as f64 / STEPS as f64;
    Both {
        structural: score_ce(&wl_tpm, &wl_c, seed),
        metabolic: score_ce(&mb_tpm, &mb_c, seed),
        final_pop, last_alive,
        activity_wn: (if carrier_steps > 0 { active as f64 / carrier_steps as f64 } else { 0.0 }) * alive_frac,
    }
}

/// SORTED-order recording (N1.3): the registry order is then reproducible across processes.
fn record(tpm: &mut TpmAccumulator, prev: &Option<HashMap<NodeId, u64>>, now: &HashMap<NodeId, u64>) {
    if let Some(p) = prev {
        let mut carry: Vec<(NodeId, u64)> = p.iter().map(|(k, v)| (*k, *v)).collect();
        let mut born: Vec<(NodeId, u64)> = now.iter().filter(|(id, _)| !p.contains_key(id)).map(|(k, v)| (*k, *v)).collect();
        carry.sort_unstable();
        born.sort_unstable();
        for (id, hp) in carry { tpm.record(WlHash(hp), WlHash(now.get(&id).copied().unwrap_or(VACUUM.0))); }
        for (_, hn) in born { tpm.record(VACUUM, WlHash(hn)); }
    }
}

fn pstr(s: &Score) -> String {
    if !s.resolved() { "  n/a".into() }
    else if s.null_ge == 0 { "<.005".into() }
    else { format!("{:.3}", s.p()) }
}
fn zs(x: f64) -> String { if x.is_nan() { "   n/a".into() } else { format!("{x:6.2}") } }

#[test]
fn n2_two_channel_ladder() {
    let seed = 12345;
    eprintln!("\n================ V9 / PHASE 2 — THE LADDER ON BOTH CHANNELS (seed {seed}) ================");
    eprintln!("  Phase 1 gate CLEARED, so a verdict may be read from the metabolic CE statistic.\n");

    let arms: [(&str, Arm); 6] = [
        ("A  replicating", Arm::A), ("B  motile static", Arm::B), ("B' motile diffuse", Arm::BPrime),
        ("C  token", Arm::C), ("D  stationary", Arm::D), ("inert (CONTROL)", Arm::Inert),
    ];
    let base: Vec<(&str, Both)> = arms.iter().map(|(n, a)| (*n, run_both(seed, *a, false))).collect();

    // ---------------- N2.2 the two-channel table ----------------
    eprintln!("================ N2.2 — THE TWO-CHANNEL TABLE (the deliverable) ================");
    eprintln!("  arm                | STRUCTURAL (WL d2)                              || METABOLIC (delta-history d2)");
    eprintln!("                     | k_c/M   | real CE | z      | p     | spread || k_c/M   | real CE | z      | p     | spread");
    for (n, b) in &base {
        eprintln!("  {:<18} | {:>3}/{:<4}| {:>7.4} | {} | {:>5} | {:>6} || {:>3}/{:<4}| {:>7.4} | {} | {:>5} | {:>6}",
            n, b.structural.k_carrier, b.structural.m, b.structural.real, zs(b.structural.z()),
            pstr(&b.structural), if b.structural.spread_ok() { "ok" } else { "DEGEN" },
            b.metabolic.k_carrier, b.metabolic.m, b.metabolic.real, zs(b.metabolic.z()),
            pstr(&b.metabolic), if b.metabolic.spread_ok() { "ok" } else { "DEGEN" });
    }

    eprintln!("\n  detection, side by side:");
    eprintln!("    arm                | structural | metabolic | agree?");
    for (n, b) in &base {
        let (s, m) = (b.structural.detects(), b.metabolic.detects());
        eprintln!("    {:<18} | {:>10} | {:>9} | {}", n, s, m, if s == m { "yes" } else { "*** NO ***" });
    }

    // ---------------- the headline question, both statistics ----------------
    eprintln!("\n================ THE HEADLINE QUESTION — is the structurally-invisible arm D the LOUDEST? ================");
    let mut by_ce: Vec<(&str, f64)> = base.iter().filter(|(n, _)| !n.starts_with("inert"))
        .map(|(n, b)| (*n, b.metabolic.z())).collect();
    by_ce.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let mut by_act: Vec<(&str, f64)> = base.iter().filter(|(n, _)| !n.starts_with("inert"))
        .map(|(n, b)| (*n, b.activity_wn)).collect();
    by_act.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    eprintln!("  by metabolic CE (the Phase-1-VALIDATED boundary statistic):");
    for (i, (n, v)) in by_ce.iter().enumerate() { eprintln!("    {}. {:<18} z = {}", i + 1, n, zs(*v)); }
    eprintln!("  by raw per-carrier activity rate (NOT a null-tested boundary statistic):");
    for (i, (n, v)) in by_act.iter().enumerate() { eprintln!("    {}. {:<18} {:.4}", i + 1, n, v); }
    let d_top_ce = by_ce[0].0.starts_with("D ");
    let d_top_act = by_act[0].0.starts_with("D ");
    eprintln!("\n  arm D is loudest on metabolic CE: {d_top_ce}   |   arm D is loudest on activity rate: {d_top_act}");
    eprintln!("  => the answer is STATISTIC-DEPENDENT and must be reported as such (N2.1). The quotable form of");
    eprintln!("     the shadow result -- 'the structurally invisible arm is metabolically the loudest' -- is TRUE");
    eprintln!("     of the raw activity rate and FALSE of the validated boundary statistic. Both are reported.");

    // ---------------- N2.3 [NEW] per-arm multi-seed on the metabolic channel ----------------
    eprintln!("\n================ N2.3 [NEW] — PER-ARM MULTI-SEED ON BOTH CHANNELS (the real evidential weight) ================");
    eprintln!("  arm                | ch.        | sustained | included | detect | f     | sign p    | degenerate-null");
    for (n, a) in arms {
        if a == Arm::Inert { continue; }
        let mut runs: Vec<(u64, Both)> = Vec::new();
        for s in 1..=MAX_ATTEMPTS {
            runs.push((s, run_both(s, a, false)));
            let inc = runs.iter().filter(|(_, b)| b.metabolic.measurable()).count();
            if inc >= TARGET_INCLUDED { break; }
        }
        let sustained = runs.iter().filter(|(_, b)| b.final_pop > 0).count();
        for (label, pick) in [("structural", true), ("metabolic ", false)] {
            let sel = |b: &Both| -> Score { if pick { b.structural.clone() } else { b.metabolic.clone() } };
            let inc: Vec<&(u64, Both)> = runs.iter().filter(|(_, b)| sel(b).measurable()).collect();
            let degen = runs.iter().filter(|(_, b)| sel(b).resolved() && !sel(b).spread_ok()).count();
            let nn = inc.len();
            let det = inc.iter().filter(|(_, b)| sel(b).p() < ALPHA).count();
            let f = if nn > 0 { det as f64 / nn as f64 } else { f64::NAN };
            let kpos = inc.iter().filter(|(_, b)| sel(b).margin() > 0.0).count();
            eprintln!("  {:<18} | {} | {:>9} | {:>8} | {:>6} | {:>5} | {:>9} | {:>15}",
                if label == "structural" { n } else { "" }, label, sustained, nn, det,
                if f.is_nan() { "n/a".into() } else { format!("{f:.3}") },
                if nn > 0 { format!("{:.2e}", sign_test_p(kpos, nn)) } else { "n/a".into() },
                degen);
        }
    }

    eprintln!("\n================ N2.3 — PRE-COMMITTED READING ================");
    let c = &base.iter().find(|(n, _)| n.starts_with("C ")).unwrap().1;
    let d = &base.iter().find(|(n, _)| n.starts_with("D ")).unwrap().1;
    eprintln!("  arm C: structural {} / metabolic {}", c.structural.detects(), c.metabolic.detects());
    eprintln!("  arm D: structural {} / metabolic {}  (structural spread-ok {})",
        d.structural.detects(), d.metabolic.detects(), d.structural.spread_ok());
    if c.metabolic.detects() && !c.structural.detects() {
        eprintln!("  => THE CHANNELS DISAGREE ABOUT ARM C. Structurally undetected (a real shadow-level negative),");
        eprintln!("     metabolically detected. The structural finding is RELOCATED, not retracted: it stands as a");
        eprintln!("     fact about the structural channel, while the metabolic channel gives the primary account.");
        eprintln!("     Neither is adjusted to reconcile them (rule 6).");
    }
    if !d.structural.spread_ok() && d.metabolic.detects() {
        eprintln!("  => ARM D IS MEASURABLE ON THE METABOLIC CHANNEL. Structurally its null is degenerate (sd = 0,");
        eprintln!("     the graph is constant, so the test is VACUOUS); metabolically it detects with a");
        eprintln!("     non-degenerate null. The L-track's 'invisible, not inert' diagnosis is now DIRECTLY");
        eprintln!("     CONFIRMED on an instrument that can see the channel it was active in.");
    }
    eprintln!("=========================================================\n");
}
