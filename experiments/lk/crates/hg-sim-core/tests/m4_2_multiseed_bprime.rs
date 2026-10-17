//! V6 / K2 — multi-seed arm B: is sustained-presence detectability a REGIME
//! property, or a seed-12345 draw?
//!
//! Arm B (non-replicating, sustained on the static substrate by `relocate`) is the
//! only novel positive in the H2 ABC experiment and was single-seed. This is the
//! F8 hardening applied to it, using F8's pipeline verbatim.
//!
//! Pre-registration: docs/validation_v6_k2_multiseed_preregistration.md
//! (commit 38944b1, before any multi-seed arm-B CE existed).
//!   - seeds 1.. ascending until 30 INCLUDED or 120 attempts (disjoint from 12345)
//!   - inclusion = transitions>=1000 AND carrier_classes>=2 AND alive at step 6000
//!   - primary: fraction with p<0.05 on the WL depth-2 partition
//!   - f>=0.90 regime property / 0.50-0.90 majority / <0.50 seed-specific
//! A fidelity anchor reproduces seed 12345's H2.2 arm-B numbers first.
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
const N_MARKERS: usize = 6; // H2.2 arm B; re-selecting is forbidden
const MIN_TRANSITIONS: u64 = 1000;
const TARGET_INCLUDED: usize = 30;
const MAX_ATTEMPTS: u64 = 120;
const ALPHA: f64 = 0.05;
const ENERGY: u32 = 1;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "m42"
run_id = "m42"
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

/// byte-identical to h2_step3_split.rs / k1_turnover_vs_persistence.rs
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

/// arm B' = arm B PLUS the diffuse substrate. byte-identical to l1_presence_vs_motility.rs B'.
fn diffuse_toml() -> &'static str {
    r#"
[[rules]]
name = "diffuse"
priority = 1
[rules.lhs]
nodes = ["hi", "lo"]
edges = [["hi", "lo"]]
[rules.lhs.node_labels]
hi = 0
lo = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["hi", "lo"]
[[rules.lhs.node_property_guards]]
var = "hi"
property = 1
op = "gte"
value = 2.0
[[rules.lhs.node_property_guards]]
var = "lo"
property = 1
op = "lt"
value = 1.0
[rules.rhs]
nodes = ["hi", "lo"]
edges = [["hi", "lo"]]
[[rules.rhs.property_effects]]
var = "hi"
key = 1
op = "subtract"
value = 1.0
[[rules.rhs.property_effects]]
var = "lo"
key = 1
op = "add"
value = 1.0
"#
}

fn build_rules(cfg: &SimConfig) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    biology.retain(|r| r.name != "replicate"); // sterile
    biology.extend(parse_rules_toml(motility_toml()).unwrap()); // motile
    biology.extend(parse_rules_toml(diffuse_toml()).unwrap()); // B': diffuse substrate
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

/// exact two-sided binomial sign test against p = 1/2 (F8's, verbatim)
fn sign_test_p(k: usize, n: usize) -> f64 {
    if n == 0 { return 1.0; }
    let mut pmf = vec![0.0f64; n + 1];
    let mut c = 1.0f64;
    let denom = 2f64.powi(n as i32);
    for i in 0..=n {
        pmf[i] = c / denom;
        c = c * (n - i) as f64 / (i + 1) as f64;
    }
    let obs = pmf[k];
    let p: f64 = pmf.iter().filter(|&&v| v <= obs * (1.0 + 1e-12)).sum();
    p.min(1.0)
}

#[derive(Clone, Default)]
struct Score { m: usize, k_carrier: usize, real_ce: f64, null_mean: f64, null_min: f64, null_max: f64, null_sd: f64, null_ge: usize, null_distinct: usize, support: f64, transitions: u64 }
impl Score {
    fn p(&self) -> f64 { self.null_ge as f64 / N_PERM as f64 }
    fn margin(&self) -> f64 { self.real_ce - self.null_mean }
    /// M2.3 null-spread guard (pre-committed in M4.2 prereg, clause 4)
    fn spread_ok(&self) -> bool {
        self.null_sd > 0.0 && (self.null_max - self.null_min) > 0.0 && self.null_distinct >= 10
    }
}

struct SeedResult { seed: u64, sustained: bool, final_pop: usize, deaths: u64, moves: u64, wl: Score, eb: Score }
impl SeedResult {
    /// pre-registered inclusion: well-defined test AND established-sustained AND non-degenerate null
    fn included(&self) -> bool {
        self.wl.transitions >= MIN_TRANSITIONS && self.wl.k_carrier >= 2 && self.sustained
            && self.wl.spread_ok()
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
    let transitions = tpm.total_transitions();
    let real_ce = if k >= 2 { ce_for(tpm, &group_of, k) } else { 0.0 };
    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7 ^ seed;
    let (mut ge, mut acc, mut sumsq, mut mx, mut mn) = (0usize, 0.0f64, 0.0f64, f64::MIN, f64::MAX);
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
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
            acc += c; sumsq += c * c; mx = mx.max(c); mn = mn.min(c); seen.insert(c.to_bits());
            if c >= real_ce { ge += 1; }
        }
    }
    let mean = acc / N_PERM as f64;
    let var = (sumsq / N_PERM as f64 - mean * mean).max(0.0);
    Score {
        m, k_carrier, real_ce, null_mean: mean,
        null_min: if mn == f64::MAX { 0.0 } else { mn },
        null_max: if mx == f64::MIN { 0.0 } else { mx },
        null_sd: var.sqrt(), null_ge: ge, null_distinct: seen.len(),
        support: null_support(m, k_carrier), transitions,
    }
}

fn run_seed(seed: u64) -> SeedResult {
    let cfg = config(seed);
    let rules = build_rules(&cfg);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    plant_markers(&mut store, seed, N_MARKERS);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut wl_tpm = TpmAccumulator::new();
    let mut eb_tpm = TpmAccumulator::new();
    let mut wl_c: HashMap<u64, bool> = HashMap::new();
    let mut eb_c: HashMap<u64, bool> = HashMap::new();
    wl_c.insert(VACUUM.0, false);
    eb_c.insert(VACUUM.0, false);
    let mut wl_prev: Option<HashMap<NodeId, u64>> = None;
    let mut eb_prev: Option<HashMap<NodeId, u64>> = None;
    let mut prev_carriers: HashSet<NodeId> = HashSet::new();
    let mut site_prev: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let (mut deaths, mut moves, mut final_pop) = (0u64, 0u64, 0usize);

    for _ in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut wl_now: HashMap<NodeId, u64> = HashMap::new();
        let mut eb_now: HashMap<NodeId, u64> = HashMap::new();
        let mut carriers: HashSet<NodeId> = HashSet::new();
        let mut site_now: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            if is_carrier {
                carriers.insert(id);
                let mut sites: Vec<NodeId> = snap.edges_incident_to(id).iter()
                    .filter_map(|&e| snap.get_edge(e))
                    .flat_map(|e| e.nodes.iter().copied())
                    .filter(|&n| n != id)
                    .collect();
                sites.sort_unstable();
                sites.dedup();
                site_now.insert(id, sites);
            }
            let wh = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            wl_now.insert(id, wh);
            wl_c.entry(wh).or_insert(is_carrier);
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let eb = energy_band_class(d.label.0 as i64, e);
            eb_now.insert(id, eb);
            eb_c.entry(eb).or_insert(is_carrier);
        }
        deaths += prev_carriers.difference(&carriers).count() as u64;
        for (id, s) in &site_now {
            if let Some(old) = site_prev.get(id) { if old != s { moves += 1; } }
        }
        site_prev = site_now;
        final_pop = carriers.len();
        prev_carriers = carriers;

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

    SeedResult {
        seed,
        sustained: final_pop > 0,
        final_pop,
        deaths,
        moves,
        wl: score(&wl_tpm, &wl_c, seed),
        eb: score(&eb_tpm, &eb_c, seed),
    }
}

#[test]
fn m4_2_multiseed_bprime() {
    // ---- FIDELITY ANCHOR: seed 12345 through THIS harness must reproduce the L-track arm B'.
    let anchor = run_seed(12345);
    eprintln!("\nFIDELITY ANCHOR seed 12345 (L-track arm B': k_carrier=2, M=40, null>=real 2/200 i.e. p=0.010, support 7.8e2)");
    eprintln!("  got: k_carrier={} M={} CE={:.4} null>=real {}/{N_PERM} p={:.3} support {:.2e} distinct={} final_pop={} deaths={} moves={}",
        anchor.wl.k_carrier, anchor.wl.m, anchor.wl.real_ce, anchor.wl.null_ge, anchor.wl.p(), anchor.wl.support,
        anchor.wl.null_distinct, anchor.final_pop, anchor.deaths, anchor.moves);
    assert!(anchor.wl.k_carrier == 2 && anchor.wl.m == 40 && anchor.wl.null_ge == 2,
        "arm-B' harness diverged from the L-track (k_carrier {}, M {}, null>=real {}) — multi-seed numbers not trustworthy",
        anchor.wl.k_carrier, anchor.wl.m, anchor.wl.null_ge);

    let mut results: Vec<SeedResult> = Vec::new();
    let mut attempts = 0u64;
    for seed in 1..=MAX_ATTEMPTS {
        attempts += 1;
        results.push(run_seed(seed));
        if results.iter().filter(|r| r.included()).count() >= TARGET_INCLUDED { break; }
    }

    let sustained = results.iter().filter(|r| r.sustained).count();
    // degenerate-null seeds: sustained + well-sampled but the realised null does not vary.
    // Reported separately per the M4.2 prereg (they are UNMEASURABLE, not non-detections).
    let degenerate = results.iter()
        .filter(|r| r.sustained && r.wl.transitions >= MIN_TRANSITIONS && r.wl.k_carrier >= 2 && !r.wl.spread_ok())
        .count();
    let included: Vec<&SeedResult> = results.iter().filter(|r| r.included()).collect();
    let n = included.len();

    eprintln!("\n================ V8 / M4.2 — MULTI-SEED ARM B' (WL depth {WL_DEPTH}, N={N_MARKERS}, seeds 1..{attempts}) ================");
    eprintln!("attempts {attempts}, sustained {sustained} ({:.0}%), degenerate-null {degenerate}, included {n} ({:.0}%)",
        100.0 * sustained as f64 / attempts as f64, 100.0 * n as f64 / attempts as f64);
    eprintln!("(inclusion = transitions>={MIN_TRANSITIONS} AND carrier_classes>=2 AND alive at step {STEPS} AND null-spread-ok)\n");
    eprintln!("  seed | sust | pop | deaths | moves | trans  |  M  | k_c | real CE | null mean | null max | dist | p       | support  | spread || eb k_c / p");
    eprintln!("  -----+------+-----+--------+-------+--------+-----+-----+---------+-----------+----------+------+---------+----------+--------++------------");
    for r in &results {
        let p = if r.included() {
            if r.wl.null_ge == 0 { "<.005".to_string() } else { format!("{:.3}", r.wl.p()) }
        } else { "  —  ".to_string() };
        let ebp = if r.eb.k_carrier >= 2 {
            if r.eb.null_ge == 0 { "<.005".to_string() } else { format!("{:.3}", r.eb.p()) }
        } else { " n/a ".to_string() };
        let inc = if r.included() { "" }
            else if r.sustained && r.wl.k_carrier >= 2 && !r.wl.spread_ok() { "  (DEGEN null)" }
            else { "  (excluded)" };
        eprintln!("  {:>4} | {:>4} | {:>3} | {:>6} | {:>5} | {:>6} | {:>3} | {:>3} | {:>7.4} | {:>9.4} | {:>8.4} | {:>4} | {:>7} | {:.2e} | {:>6} || {:>3} / {:>5}{}",
            r.seed, if r.sustained { "yes" } else { "no" }, r.final_pop, r.deaths, r.moves,
            r.wl.transitions, r.wl.m, r.wl.k_carrier, r.wl.real_ce, r.wl.null_mean, r.wl.null_max, r.wl.null_distinct,
            p, r.wl.support, if r.wl.spread_ok() { "ok" } else { "DEGEN" }, r.eb.k_carrier, ebp, inc);
    }

    if n < 10 {
        eprintln!("\n  VACUOUS: only {n} included seeds from {attempts} attempts — no verdict is read from this.");
        return;
    }

    // ---- pre-committed aggregate decision rule
    let detect = included.iter().filter(|r| r.wl.p() < ALPHA).count();
    let f = detect as f64 / n as f64;
    let k_pos = included.iter().filter(|r| r.wl.margin() > 0.0).count();
    let sign_p = sign_test_p(k_pos, n);
    let reversals: Vec<&&SeedResult> = included.iter().filter(|r| r.wl.margin() < 0.0).collect();
    let eb_detect = included.iter().filter(|r| r.eb.k_carrier >= 2 && r.eb.p() < ALPHA).count();

    eprintln!("\n================ M4.2 — PRE-COMMITTED VERDICT (rule fixed in the M4.2 prereg) ================");
    eprintln!("  PRIMARY  fraction detectable on WL (p<{ALPHA}): {detect}/{n} = {f:.3}");
    eprintln!("  SECONDARY sign test (real CE > own null mean): {k_pos}/{n}, exact binomial p = {sign_p:.3e}");
    eprintln!("  (secondary column) energy-band detectable: {eb_detect}/{n}");
    eprintln!("  degenerate-null seeds excluded as UNMEASURABLE (not non-detections): {degenerate}");
    if reversals.is_empty() {
        eprintln!("  REVERSALS none — every included seed's real boundary beats its own null mean");
    } else {
        eprintln!("  REVERSALS {} seed(s):", reversals.len());
        for r in &reversals {
            eprintln!("    seed {}: real {:.4} < null mean {:.4} (delta {:+.4})", r.seed, r.wl.real_ce, r.wl.null_mean, r.wl.margin());
        }
    }
    if f >= 0.90 {
        eprintln!("\n  => REGIME PROPERTY. Arm B''s detectability holds across the established regime, not just at");
        eprintln!("     seed 12345. 'Even a little structural activity suffices' may be stated flat.");
    } else if f >= 0.50 {
        eprintln!("\n  => MAJORITY. Arm B' detects in most established seeds; partial generalisation. The excepted");
        eprintln!("     seeds are listed above — read them individually, do not average.");
    } else {
        eprintln!("\n  => SEED-SPECIFIC. The published p=0.010 was a seed-12345 draw; the clause it supports");
        eprintln!("     ('even a little activity suffices') must be WITHDRAWN and the verdict re-qualified.");
    }
    // the substance of the claim: HOW LITTLE activity, quantified
    let moves: Vec<u64> = included.iter().map(|r| r.moves).collect();
    let (mn_mv, mx_mv) = (moves.iter().min().copied().unwrap_or(0), moves.iter().max().copied().unwrap_or(0));
    let mean_mv = moves.iter().sum::<u64>() as f64 / n.max(1) as f64;
    eprintln!("\n  STRUCTURAL-EVENT COUNT across included seeds (this is the substance of 'a little activity'):");
    eprintln!("    relocations per seed: min {mn_mv}, mean {mean_mv:.1}, max {mx_mv}");
    let min_dist = included.iter().map(|r| r.wl.null_distinct).min().unwrap_or(0);
    eprintln!("    weakest included null spread: {min_dist} distinct values (guard floor is 10)");
    eprintln!("=====================================================================================\n");
}
