//! V6 / K3 — separate the PRESENCE-specific signal from the ALWAYS-ON metabolic one.
//!
//! The energy-band (metabolic) partition detects every ABC arm, including arm C,
//! the token dead at step 120. So some of "carrier detectability" is definitional:
//! a carrier holds energy in a different band than substrate whether or not it is
//! sustained. Unless that is separated out, "detectable" conflates "does something
//! interesting" with "is a distinct kind of object".
//!
//! Pre-registration: docs/validation_v6_k3_signal_isolation_preregistration.md
//! (committed before any number here existed). Effect size fixed there:
//!     z = (real_CE - null_mean) / null_sd     over the same 200 permutations.
//! CE MAGNITUDE is registry-dominated (F2.4); z is a DETECTABILITY measure only.
//!
//! K3.1  always-on magnitude          = z_eb(C)   -- C has metabolic distinctness and nothing else
//! K3.2  presence-specific component  = z_wl(arm) - z_wl(C)   for arm in {A, B}
//!       (subtraction is WITHIN a partition; cross-partition differences are
//!        reported as context only and no verdict is read from them)
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
const WL_DEPTH: u8 = 2;
const N_PERM: usize = 200;
const ENERGY: u32 = 1;
const ALPHA: f64 = 0.05;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "k3"
run_id = "k3"
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

/// byte-identical to h2_step3_split.rs
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

fn build_rules(cfg: &SimConfig, sterile: bool, motile: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if sterile { biology.retain(|r| r.name != "replicate"); }
    if motile { biology.extend(parse_rules_toml(motility_toml()).unwrap()); }
    compile_rules(&biology).unwrap()
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
fn energy_band_class(label: i64, e: f64) -> u64 {
    let band = e.floor().clamp(-1.0, 64.0) as i64;
    ((label as u64) << 32) ^ (band as u64 & 0xFFFF_FFFF) ^ 0x9E37_79B9
}

#[derive(Clone, Default)]
struct Score {
    m: usize,
    k_carrier: usize,
    real_ce: f64,
    null_mean: f64,
    null_sd: f64,
    null_ge: usize,
    support: f64,
}
impl Score {
    fn p(&self) -> f64 { self.null_ge as f64 / N_PERM as f64 }
    fn delta(&self) -> f64 { self.real_ce - self.null_mean }
    /// pre-registered effect size: null-standardized distance. NOT a magnitude claim.
    fn z(&self) -> f64 {
        if self.k_carrier < 2 || self.null_sd <= 0.0 { f64::NAN } else { self.delta() / self.null_sd }
    }
    fn detects(&self) -> bool {
        self.k_carrier >= 2 && self.k_carrier < self.m && self.p() < ALPHA && self.support > 100.0
    }
}

fn score(tpm: &TpmAccumulator, is_c: &HashMap<u64, bool>, seed: u64) -> Score {
    let mut compact: HashMap<usize, usize> = HashMap::new();
    let group_of: Vec<usize> = tpm.hashes().iter().map(|h| {
        let raw = *is_c.get(&h.0).unwrap_or(&false) as usize;
        let n = compact.len();
        *compact.entry(raw).or_insert(n)
    }).collect();
    let k = compact.len().max(1);
    let m = tpm.hashes().len();
    let k_carrier = tpm.hashes().iter().filter(|h| *is_c.get(&h.0).unwrap_or(&false)).count();
    let real_ce = if k >= 2 { ce_for(tpm, &group_of, k) } else { 0.0 };
    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7 ^ seed;
    let (mut ge, mut sum, mut sumsq) = (0usize, 0.0f64, 0.0f64);
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
            sum += c; sumsq += c * c;
            if c >= real_ce { ge += 1; }
        }
    }
    let n = N_PERM as f64;
    let mean = sum / n;
    let var = (sumsq / n - mean * mean).max(0.0);
    Score { m, k_carrier, real_ce, null_mean: mean, null_sd: var.sqrt(), null_ge: ge, support: null_support(m, k_carrier) }
}

struct Res { wl: Score, eb: Score, last_alive: u64 }

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
    let mut last_alive = 0u64;
    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
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
        if live > 0 { last_alive = step; }
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
    Res { wl: score(&wl_tpm, &wl_c, seed), eb: score(&eb_tpm, &eb_c, seed), last_alive }
}

fn pstr(s: &Score) -> String {
    if s.k_carrier < 2 { "  n/a".into() }
    else if s.null_ge == 0 { "<.005".into() }
    else { format!("{:.3}", s.p()) }
}
fn zs(x: f64) -> String { if x.is_nan() { "    n/a".into() } else { format!("{x:7.2}") } }

#[test]
fn k3_signal_isolation() {
    let seed = 12345;
    let a = run(seed, false, false, 1);
    let b = run(seed, true, true, 6);
    let c = run(seed, true, false, 6);

    eprintln!("\n================ V6 / K3 — PRESENCE-SPECIFIC vs ALWAYS-ON METABOLIC SIGNAL (seed {seed}) ================");
    eprintln!("  effect size z = (real_CE - null_mean)/null_sd over the same 200 permutations. DETECTABILITY only;");
    eprintln!("  CE magnitude is registry-dominated (F2.4) and no magnitude claim is made from any number here.\n");
    eprintln!("  arm                     | last-alive || WL:  k_c /  M  |   real CE / null mean +- sd |    delta |    z    | p     | support");
    eprintln!("  ------------------------+------------++----------------+-----------------------------+----------+---------+-------+---------");
    for (name, r) in [("A replicating", &a), ("B non-repl SUSTAINED", &b), ("C non-repl token", &c)] {
        let s = &r.wl;
        eprintln!("  {:<23} | {:>10} || {:>4} / {:>3}    | {:>9.4} / {:.4} +- {:.4} | {:>+8.4} | {} | {:>5} | {:.2e}",
            name, r.last_alive, s.k_carrier, s.m, s.real_ce, s.null_mean, s.null_sd, s.delta(), zs(s.z()), pstr(s), s.support);
    }
    eprintln!();
    eprintln!("  arm                     | last-alive || eb:  k_c /  M  |   real CE / null mean +- sd |    delta |    z    | p     | support");
    eprintln!("  ------------------------+------------++----------------+-----------------------------+----------+---------+-------+---------");
    for (name, r) in [("A replicating", &a), ("B non-repl SUSTAINED", &b), ("C non-repl token", &c)] {
        let s = &r.eb;
        eprintln!("  {:<23} | {:>10} || {:>4} / {:>3}    | {:>9.4} / {:.4} +- {:.4} | {:>+8.4} | {} | {:>5} | {:.2e}",
            name, r.last_alive, s.k_carrier, s.m, s.real_ce, s.null_mean, s.null_sd, s.delta(), zs(s.z()), pstr(s), s.support);
    }

    // ---------- K3.1 — magnitude of the always-on definitional signal ----------
    eprintln!("\n================ K3.1 — the ALWAYS-ON metabolic signal, isolated on arm C ================");
    eprintln!("  Arm C is dead from step 120: no sustained presence, no replication. Whatever it shows on the");
    eprintln!("  metabolic partition is purely definitional (a carrier holds energy in a different band).");
    eprintln!("    z_eb(C) = {}   delta {:+.4}   p {}   k_carrier {}   support {:.2e}",
        zs(c.eb.z()), c.eb.delta(), pstr(&c.eb), c.eb.k_carrier, c.eb.support);
    eprintln!("    For scale, on the SAME partition: z_eb(A) = {}, z_eb(B) = {}", zs(a.eb.z()), zs(b.eb.z()));

    // ---------- K3.2 — presence-specific component (within-partition subtraction) ----------
    let s_a = a.wl.z() - c.wl.z();
    let s_b = b.wl.z() - c.wl.z();
    eprintln!("\n================ K3.2 — the PRESENCE-SPECIFIC component (within the WL partition) ================");
    eprintln!("  C calibrates the WL-partition signal attributable to bare metabolic distinctness with NO presence.");
    eprintln!("    z_wl(C) = {}  (p {})   <-- the calibrator", zs(c.wl.z()), pstr(&c.wl));
    eprintln!("    S(A) = z_wl(A) - z_wl(C) = {} - {} = {}", zs(a.wl.z()), zs(c.wl.z()), zs(s_a));
    eprintln!("    S(B) = z_wl(B) - z_wl(C) = {} - {} = {}", zs(b.wl.z()), zs(c.wl.z()), zs(s_b));

    // ---------- K3.3 — pre-committed reading ----------
    let survives = s_a > 0.0 && s_b > 0.0 && a.wl.detects() && b.wl.detects() && !c.wl.detects();
    let vanishes = s_a <= 0.0 || s_b <= 0.0 || c.wl.detects();
    eprintln!("\n================ K3.3 — PRE-COMMITTED READING (rule fixed before any number above) ================");
    eprintln!("  WL detects:  A={} B={} C={}   |   e-band detects: A={} B={} C={}",
        a.wl.detects(), b.wl.detects(), c.wl.detects(), a.eb.detects(), b.eb.detects(), c.eb.detects());
    if survives && !vanishes {
        eprintln!("  => SURVIVES. A presence-specific signal exists that is NOT mere metabolic distinctness.");
        eprintln!("     The direct argument: arm C carries a LARGE always-on metabolic signal and NO WL signal, so");
        eprintln!("     the WL boundary cannot be the metabolic boundary in disguise. Sustained presence adds WL");
        eprintln!("     structure that metabolic distinctness alone does not produce. This is the object worth");
        eprintln!("     carrying to any future causal-emergence claim.");
    } else if vanishes && !survives {
        eprintln!("  => VANISHES. A/B's WL detectability is not distinguishable from the always-on metabolic level:");
        eprintln!("     the 'presence boundary' was substantially the metabolic boundary in disguise. A strong");
        eprintln!("     deflation and a partial vindication of the construction hypothesis. Reported as such.");
    } else {
        eprintln!("  => PARTIAL. S(A)={}, S(B)={} — reported at arm level, not resolved by picking an arm.", zs(s_a), zs(s_b));
    }
    eprintln!("  (No verdict is read from any cross-partition difference; those are context only.)");
    eprintln!("=========================================================================================\n");
}
