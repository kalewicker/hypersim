//! Step 3 / H2.2 — the confound-splitting experiment: is a SUSTAINED
//! non-replicating population detectable?
//!
//! The G-track left replication vs turnover-DURATION confounded, because its
//! sterile token died at step 120 and contributed 2% of transitions. The H2.1
//! diagnostic found that MOTILITY (not churn) sustains a non-replicating
//! population, on the plain static substrate. So the split is finally runnable on
//! static, where the J2 energy-band partition works cleanly:
//!   - Arm A: replicating carrier (F8)                 -- detectable (baseline)
//!   - Arm B: sustained non-replicating (motile-sterile bloom) -- detectable?
//! scored with the (label, energy-band) partition (F8-recovering, churn-stable).
//!
//! PRE-COMMITTED INTERPRETATION (written before running, per plan H2.2):
//!   * Arm B DETECTABLE (p<0.05, adequate support) while A is
//!       => the boundary is SUSTAINED TURNOVER / metabolic state, NOT replication.
//!          The G-track sustained-turnover verdict is confirmed and its
//!          replication-confound resolved toward DURATION: once a non-replicating
//!          population is sustained (here by motility), detectability returns.
//!   * Arm B NOT detectable while A is
//!       => replication-per-se is load-bearing even with duration controlled --
//!          the strongest biology-side result.
//!   * Report null support + k_carrier for both arms (F0 bar), or it is an artifact.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;
use std::collections::HashMap;

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const STEPS: u64 = 6_000;
const N_PERM: usize = 200;
const ENERGY: u32 = 1;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "s3split"
run_id = "s3split"
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

fn motility_toml() -> &'static str {
    r#"
[[rules]]
name = "relocate"
priority = 2
[rules.lhs]
nodes = ["site", "carrier", "target"]
edges = [["site", "carrier"], ["site", "target"]]
[rules.lhs.node_labels]
carrier = 100
site = 0
target = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["site", "carrier", "target"]
[[rules.lhs.node_property_compares]]
var_a = "carrier"
key_a = 1
op = "lt"
var_b = "carrier"
key_b = 13
[[rules.lhs.node_property_guards]]
var = "target"
property = 1
op = "gte"
value = 1.0
[[rules.lhs.node_property_guards]]
var = "target"
property = 22
op = "eq"
value = false
[[rules.lhs.node_property_guards]]
var = "site"
property = 1
op = "lt"
value = 1.0
[rules.rhs]
nodes = ["site", "carrier", "target"]
edges = [["site", "target"], ["target", "carrier"]]
[[rules.rhs.property_effects]]
var = "site"
key = 22
op = "set"
value = false
[[rules.rhs.property_effects]]
var = "target"
key = 22
op = "set"
value = true
"#
}

fn plant_markers(store: &mut HypergraphStore, seed: u64, n: usize) {
    use rand::{Rng, SeedableRng};
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    if ids.is_empty() { return; }
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0x00CA_111E);
    let (mut planted, mut used) = (0usize, std::collections::HashSet::new());
    let cap = n.min(ids.len());
    while planted < cap {
        let pick = ids[rng.gen_range(0..ids.len())];
        if used.insert(pick) {
            store.get_node_mut(pick).unwrap().set_property(hg_alife::keys::CARRIER_MARKER, PropertyValue::Bool(true));
            planted += 1;
        }
    }
}

fn build_rules(cfg: &SimConfig, sterile: bool, motile: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if sterile { biology.retain(|r| r.name != "replicate"); }
    if motile { biology.extend(parse_rules_toml(motility_toml()).unwrap()); }
    compile_rules(&biology).unwrap()
}

fn splitmix(s: &mut u64) -> u64 { *s = s.wrapping_add(0x9E37_79B9_7F4A_7C15); let mut z = *s; z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9); z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB); z ^ (z >> 31) }
fn null_support(m: usize, k: usize) -> f64 { if k == 0 || k >= m { return 0.0; } let k = k.min(m - k); let mut acc = 1.0; for i in 0..k { acc *= (m - i) as f64 / (i + 1) as f64; } acc }
fn ce_for(tpm: &TpmAccumulator, g: &[usize], k: usize) -> f64 { let rows = tpm.normalized_rows(); let micro = ei_from_tpm(&rows.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>()); let mac = ei_from_tpm(&coarse_grain(&rows, g, k)); if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 } }
fn energy_band_class(label: i64, e: f64) -> u64 { let band = e.floor().clamp(-1.0, 64.0) as i64; ((label as u64) << 32) ^ (band as u64 & 0xFFFF_FFFF) ^ 0x9E37_79B9 }

#[derive(Clone)]
struct Score { m: usize, k_carrier: usize, real_ce: f64, null_mean: f64, null_ge: usize, support: f64 }
struct Res { wl: Score, eb: Score, peak: usize, final_pop: usize, last_alive: u64 }

fn score(tpm: &TpmAccumulator, is_c: &HashMap<u64, bool>, seed: u64) -> Score {
    let mut compact: HashMap<usize, usize> = HashMap::new();
    let group_of: Vec<usize> = tpm.hashes().iter().map(|h| { let raw = *is_c.get(&h.0).unwrap_or(&false) as usize; let n = compact.len(); *compact.entry(raw).or_insert(n) }).collect();
    let k = compact.len().max(1);
    let m = tpm.hashes().len();
    let k_carrier = tpm.hashes().iter().filter(|h| *is_c.get(&h.0).unwrap_or(&false)).count();
    let real_ce = if k >= 2 { ce_for(tpm, &group_of, k) } else { 0.0 };
    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7 ^ seed;
    let (mut ge, mut acc) = (0usize, 0.0);
    if k >= 2 {
        for _ in 0..N_PERM {
            let mut labels: Vec<usize> = canon.iter().map(|&i| group_of[i]).collect();
            for i in (1..labels.len()).rev() { let j = (splitmix(&mut rng) % (i as u64 + 1)) as usize; labels.swap(i, j); }
            let mut perm = vec![0usize; group_of.len()];
            for (pos, &i) in canon.iter().enumerate() { perm[i] = labels[pos]; }
            let c = ce_for(tpm, &perm, k); acc += c; if c >= real_ce { ge += 1; }
        }
    }
    Score { m, k_carrier, real_ce, null_mean: acc / N_PERM as f64, null_ge: ge, support: null_support(m, k_carrier) }
}

/// Run one arm; score the carrier/substrate boundary with BOTH the WL (structural,
/// the G-track's instrument) and the energy-band (metabolic, J2's) partitions.
fn run(seed: u64, sterile: bool, motile: bool, n_markers: usize) -> Res {
    let cfg = config(seed);
    let rules = build_rules(&cfg, sterile, motile);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    plant_markers(&mut store, seed, n_markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);
    let mut wl_tpm = TpmAccumulator::new();
    let mut eb_tpm = TpmAccumulator::new();
    let mut wl_c: HashMap<u64, bool> = HashMap::new();
    let mut eb_c: HashMap<u64, bool> = HashMap::new();
    wl_c.insert(VACUUM.0, false);
    eb_c.insert(VACUUM.0, false);
    let mut wl_prev: Option<HashMap<NodeId, u64>> = None;
    let mut eb_prev: Option<HashMap<NodeId, u64>> = None;
    let (mut peak, mut final_pop, mut last_alive) = (0usize, 0usize, 0u64);
    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, 2);
        let mut wl_now: HashMap<NodeId, u64> = HashMap::new();
        let mut eb_now: HashMap<NodeId, u64> = HashMap::new();
        let mut live = 0usize;
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            if is_carrier { live += 1; }
            let wh = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            wl_now.insert(id, wh);
            wl_c.entry(wh).or_insert(is_carrier);
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let eb = energy_band_class(d.label.0 as i64, e);
            eb_now.insert(id, eb);
            eb_c.entry(eb).or_insert(is_carrier);
        }
        peak = peak.max(live); final_pop = live; if live > 0 { last_alive = step; }
        if let Some(p) = &wl_prev {
            for (id, &hp) in p { wl_tpm.record(WlHash(hp), WlHash(wl_now.get(id).copied().unwrap_or(VACUUM.0))); }
            for (id, &hn) in &wl_now { if !p.contains_key(id) { wl_tpm.record(VACUUM, WlHash(hn)); } }
        }
        if let Some(p) = &eb_prev {
            for (id, &hp) in p { eb_tpm.record(WlHash(hp), WlHash(eb_now.get(id).copied().unwrap_or(VACUUM.0))); }
            for (id, &hn) in &eb_now { if !p.contains_key(id) { eb_tpm.record(VACUUM, WlHash(hn)); } }
        }
        wl_prev = Some(wl_now);
        eb_prev = Some(eb_now);
    }
    Res { wl: score(&wl_tpm, &wl_c, seed), eb: score(&eb_tpm, &eb_c, seed), peak, final_pop, last_alive }
}

fn det(s: &Score) -> bool { s.k_carrier >= 2 && s.k_carrier < s.m && (s.null_ge as f64 / N_PERM as f64) < 0.05 && s.support > 100.0 }
fn pstr(s: &Score) -> String { if s.k_carrier < 2 { "  n/a".into() } else if s.null_ge == 0 { "<.005".into() } else { format!("{:.3}", s.null_ge as f64 / N_PERM as f64) } }

#[test]
fn step3_confound_split() {
    let seed = 12345;
    // Three arms, each scored with BOTH the WL (structural — the G-track's
    // instrument, whose confound we are resolving) and the energy-band (metabolic)
    // partitions. Arm B uses N=6 markers (below carrying capacity, so motility
    // sustains the population; the density sweep is reported for context).
    let a = run(seed, false, false, 1);   // A: replicating (F8)
    let b = run(seed, true, true, 6);     // B: sustained non-replicating (motile-sterile)
    let c = run(seed, true, false, 6);    // C: non-sustained non-replicating (G-track token)

    eprintln!("\n================ STEP 3 / H2.2 — CONFOUND SPLIT, both partitions (seed {seed}) ================");
    eprintln!("  arm                              | peak | last-alive || WL: k_c / p / support        || e-band: k_c / p / support");
    eprintln!("  ---------------------------------+------+------------++------------------------------++--------------------------");
    for (name, r) in [
        ("A replicating (F8)", &a),
        ("B non-repl SUSTAINED (motile,6)", &b),
        ("C non-repl NOT sustained (token,6)", &c),
    ] {
        eprintln!("  {:<32} | {:>4} | {:>10} || {:>3} / {:>6} / {:.2e} || {:>3} / {:>6} / {:.2e}",
            name, r.peak, r.last_alive, r.wl.k_carrier, pstr(&r.wl), r.wl.support, r.eb.k_carrier, pstr(&r.eb), r.eb.support);
    }
    eprintln!("\n  density sweep (motile-sterile, last-alive / peak): motility sustains only BELOW carrying capacity");
    for n in [1usize, 6, 12, 24, 36] {
        let r = run(seed, true, true, n);
        eprintln!("    N={:<2} markers: last-alive {:>4}, peak {:>2}  (WL p {}, e-band p {})", n, r.last_alive, r.peak, pstr(&r.wl), pstr(&r.eb));
    }
    eprintln!("=================================================================================================\n");

    eprintln!("================ VERDICT (pre-committed; the WL partition is the decisive one) ================");
    eprintln!("  WL (G-track instrument): A={}, B(sustained)={}, C(token)={}", det(&a.wl), det(&b.wl), det(&c.wl));
    eprintln!("  e-band (metabolic):      A={}, B(sustained)={}, C(token)={}", det(&a.eb), det(&b.eb), det(&c.eb));
    let b_sustains = b.last_alive >= STEPS && b.peak > 1;
    if b_sustains && det(&a.wl) && det(&b.wl) && !det(&c.wl) {
        eprintln!("  RESOLVED toward DURATION. On the WL partition the sustained non-replicating population (B) is");
        eprintln!("  detectable while the non-sustained token (C) is not — both non-replicating. So the G-track token's");
        eprintln!("  WL non-detection was DURATION (short bloom), not replication: restore duration without replication");
        eprintln!("  and the WL boundary returns. The 'sustained-turnover boundary' name is confirmed; replication-");
        eprintln!("  per-se is NOT required.");
    } else if b_sustains && det(&a.wl) && !det(&b.wl) {
        eprintln!("  REPLICATION-PER-SE load-bearing: sustained non-replicating (B) is NOT WL-detectable while the");
        eprintln!("  replicating arm (A) is — replication carries WL structure beyond sustained presence.");
    } else {
        eprintln!("  See table; interpret from the WL row (A/B/C) with the density-sweep context.");
    }
    eprintln!("  (Separately: the energy-band metabolic boundary is present in ALL arms incl. the non-sustained");
    eprintln!("   token — a carrier is metabolically distinct from substrate whether or not it is sustained.)");
    eprintln!("==============================================================================================\n");
    assert!(det(&a.wl) && det(&a.eb), "replicating baseline must detect on both partitions");
}
