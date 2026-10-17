//! V7 / L2.2 — does any partition satisfy the L2.1 criteria, and does the test
//! have POWER on arm D at all?
//!
//! Criteria pre-committed in docs/validation_v7_l2_instrument_criteria.md (847b36c):
//!   (a) BIOLOGY-DISCRIMINATING: detects A (p<0.05, k_c>=2, k_c<M, support>100)
//!       AND gives the dead token |z_C| < 2.0 with p_C >= 0.05.
//!   (b) STATIONARY: H0.1 TWTF <= 0.5 at R=3. Tier 1 = on L1's own substrate
//!       (gates L1); Tier 2 = under node-turnover churn (the arc-level dilemma).
//!
//! ADDED HERE, and reported as a criteria GAP rather than a silent fix: the F0
//! support bar C(M,k)>100 is COMBINATORIAL support. It does not guarantee the
//! REALIZED permutation null has any spread. Arm D's first scoring returned
//! null_sd = 0 exactly — a single-point null, on which no partition whatsoever
//! could be detected. A non-detection from a zero-power test is an ARTIFACT, not
//! a biological negative, so realized null spread is measured for every arm and
//! any arm whose null is degenerate is reported as UNMEASURABLE, not as negative.
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
const R_PERSIST: u64 = 3;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "l2iv"
run_id = "l2iv"
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

/// H1 bounded-churn rule (Tier 2 substrate), byte-identical to h1_churn_substrate.rs
fn churn_toml() -> &'static str {
    r#"
[[rules]]
name = "churn"
priority = 2
[rules.lhs]
nodes = ["a", "old", "b"]
edges = [["a", "old"], ["old", "b"]]
[rules.lhs.node_labels]
a = 0
old = 0
b = 0
[[rules.lhs.node_property_guards]]
var = "old"
property = 1
op = "gte"
value = 20.0
[[rules.lhs.node_property_guards]]
var = "old"
property = 22
op = "eq"
value = false
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "old", "b"]
[rules.rhs]
nodes = ["a", "fresh", "b"]
edges = [["a", "fresh"], ["fresh", "b"]]
[rules.rhs.node_labels]
fresh = 0
[[rules.rhs.property_effects]]
var = "fresh"
key = 1
op = "set"
value = 0.0
[[rules.rhs.property_effects]]
var = "fresh"
key = 22
op = "set"
value = false
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

fn build_rules(cfg: &SimConfig, sterile: bool, motile: bool, diffuse: bool, churn: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if sterile { biology.retain(|r| r.name != "replicate"); }
    if motile { biology.extend(parse_rules_toml(motility_toml()).unwrap()); }
    if diffuse { biology.extend(parse_rules_toml(diffuse_toml()).unwrap()); }
    if churn { biology.extend(parse_rules_toml(churn_toml()).unwrap()); }
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
fn eis(tpm: &TpmAccumulator, g: &[usize], k: usize) -> (f64, f64) {
    let rows = tpm.normalized_rows();
    let micro = ei_from_tpm(&rows.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>());
    let mac = ei_from_tpm(&coarse_grain(&rows, g, k));
    (micro.ei, mac.ei)
}
fn energy_band_class(label: i64, e: f64) -> u64 {
    let band = e.floor().clamp(-1.0, 64.0) as i64;
    ((label as u64) << 32) ^ (band as u64 & 0xFFFF_FFFF) ^ 0x9E37_79B9
}

#[derive(Clone, Default)]
struct Score {
    m: usize, k_carrier: usize, real_ce: f64,
    null_mean: f64, null_sd: f64, null_min: f64, null_max: f64, null_ge: usize,
    support: f64, transitions: u64, ei_micro: f64, ei_macro: f64,
    /// distinct CE values seen across the 200 permutations
    null_distinct: usize,
}
impl Score {
    fn p(&self) -> f64 { self.null_ge as f64 / N_PERM as f64 }
    fn z(&self) -> f64 {
        if self.k_carrier < 2 || self.null_sd <= 0.0 { f64::NAN } else { (self.real_ce - self.null_mean) / self.null_sd }
    }
    fn detects(&self) -> bool {
        self.k_carrier >= 2 && self.k_carrier < self.m && self.p() < ALPHA && self.support > 100.0
    }
    /// the added check: does the realized null have ANY spread? a single-point
    /// null means the test cannot detect anything, so a negative is meaningless.
    /// Keyed on the RANGE and sd, not on `null_distinct` — the distinct-value count
    /// is sensitive to floating-point summation order (HashMap row iteration) and
    /// wobbles run to run, whereas max-min == 0 and sd == 0 are stable.
    fn null_degenerate(&self) -> bool { self.null_sd <= 0.0 || (self.null_max - self.null_min) <= 0.0 }
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
    let (ei_micro, ei_macro) = if k >= 2 { eis(tpm, &group_of, k) } else { (0.0, 0.0) };
    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7 ^ seed;
    let (mut ge, mut sum, mut sumsq) = (0usize, 0.0f64, 0.0f64);
    let (mut mn, mut mx) = (f64::MAX, f64::MIN);
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
            sum += c; sumsq += c * c;
            mn = mn.min(c); mx = mx.max(c);
            seen.insert(c.to_bits());
            if c >= real_ce { ge += 1; }
        }
    }
    let n = N_PERM as f64;
    let mean = sum / n;
    let var = (sumsq / n - mean * mean).max(0.0);
    Score {
        m, k_carrier, real_ce, null_mean: mean, null_sd: var.sqrt(),
        null_min: if mn == f64::MAX { 0.0 } else { mn },
        null_max: if mx == f64::MIN { 0.0 } else { mx },
        null_ge: ge, support: null_support(m, k_carrier),
        transitions: tpm.total_transitions(), ei_micro, ei_macro,
        null_distinct: seen.len(),
    }
}

struct Res { wl: Score, eb: Score, twtf: f64, classes: usize, frozen_frac: f64, last_alive: u64, moves: u64,
    /// realized node-identity turnover: node ids present now that were absent last step.
    /// Tier 2 is meaningless unless churn actually FIRED, so this is measured, not assumed.
    fresh_nodes: u64, n_end: usize,
    /// steps on which some node's ALife energy (property key 1) changed. WL excludes the
    /// properties map (the Q2 invariant), so this is the activity WL CANNOT see. Decisive
    /// for telling "arm D is inert" apart from "arm D is invisible to this instrument".
    energy_active_steps: u64 }

fn run(seed: u64, sterile: bool, motile: bool, diffuse: bool, churn: bool, n_markers: usize) -> Res {
    let cfg = config(seed);
    let rules = build_rules(&cfg, sterile, motile, diffuse, churn);
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
    let mut live: HashMap<u64, u64> = HashMap::new();
    let mut src: HashMap<u64, u64> = HashMap::new();
    live.insert(VACUUM.0, u64::MAX);
    let mut site_prev: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let (mut last_alive, mut moves, mut frozen_steps) = (0u64, 0u64, 0u64);
    let mut fresh_nodes = 0u64;
    let mut n_end = 0usize;
    let mut prev_ids: HashSet<NodeId> = store.nodes().map(|(id, _)| id).collect();
    let mut energy_active_steps = 0u64;
    let mut prev_energies: Vec<(NodeId, u64)> = Vec::new();

    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut wl_now: HashMap<NodeId, u64> = HashMap::new();
        let mut eb_now: HashMap<NodeId, u64> = HashMap::new();
        let mut present: HashSet<u64> = HashSet::new();
        let mut site_now: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut live_c = 0usize;
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            if is_carrier {
                live_c += 1;
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
            present.insert(wh);
            wl_c.entry(wh).or_insert(is_carrier);
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let eb = energy_band_class(d.label.0 as i64, e);
            eb_now.insert(id, eb);
            eb_c.entry(eb).or_insert(is_carrier);
        }
        let mut e_now: Vec<(NodeId, u64)> = snap.nodes().map(|(id, d)| {
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            (id, e.to_bits())
        }).collect();
        e_now.sort_by_key(|(id, _)| *id);
        if !prev_energies.is_empty() && e_now != prev_energies { energy_active_steps += 1; }
        prev_energies = e_now;
        let now_ids: HashSet<NodeId> = snap.nodes().map(|(id, _)| id).collect();
        fresh_nodes += now_ids.difference(&prev_ids).count() as u64;
        prev_ids = now_ids;
        n_end = snap.nodes().count();
        for h in &present { *live.entry(*h).or_insert(0) += 1; }
        for (id, s) in &site_now { if let Some(o) = site_prev.get(id) { if o != s { moves += 1; } } }
        site_prev = site_now;
        if live_c > 0 { last_alive = step; }

        if let Some(p) = &wl_prev {
            // "frozen" = the whole WL class assignment is unchanged from the previous step
            if *p == wl_now { frozen_steps += 1; }
            for (id, &hp) in p {
                wl_tpm.record(WlHash(hp), WlHash(wl_now.get(id).copied().unwrap_or(VACUUM.0)));
                *src.entry(hp).or_insert(0) += 1;
            }
            for (id, &hn) in &wl_now {
                if !p.contains_key(id) { wl_tpm.record(VACUUM, WlHash(hn)); *src.entry(VACUUM.0).or_insert(0) += 1; }
            }
        }
        if let Some(p) = &eb_prev {
            for (id, &hp) in p { eb_tpm.record(WlHash(hp), WlHash(eb_now.get(id).copied().unwrap_or(VACUUM.0))); }
            for (id, &hn) in &eb_now { if !p.contains_key(id) { eb_tpm.record(VACUUM, WlHash(hn)); } }
        }
        wl_prev = Some(wl_now);
        eb_prev = Some(eb_now);
    }
    let total: u64 = src.values().sum();
    let transient: u64 = src.iter()
        .filter(|(h, _)| *live.get(*h).unwrap_or(&0) < R_PERSIST)
        .map(|(_, c)| *c).sum();
    Res {
        wl: score(&wl_tpm, &wl_c, seed),
        eb: score(&eb_tpm, &eb_c, seed),
        twtf: if total > 0 { transient as f64 / total as f64 } else { 0.0 },
        classes: live.len().saturating_sub(1),
        frozen_frac: frozen_steps as f64 / (STEPS - 1) as f64,
        last_alive, moves, fresh_nodes, n_end, energy_active_steps,
    }
}

fn pstr(s: &Score) -> String {
    if s.k_carrier < 2 { "  n/a".into() }
    else if s.null_ge == 0 { "<.005".into() }
    else { format!("{:.3}", s.p()) }
}
fn zs(x: f64) -> String { if x.is_nan() { "    n/a".into() } else { format!("{x:7.2}") } }

#[test]
fn l2_instrument_validation() {
    let seed = 12345;
    let a = run(seed, false, false, false, false, 1); // A replicating, static
    let c = run(seed, true, false, false, false, 6);  // C token, static
    let bp = run(seed, true, true, true, false, 6);   // B' motile, diffuse
    let d = run(seed, true, false, true, false, 6);   // D stationary, diffuse
    let churn_bare = run(seed, true, false, false, true, 0); // Tier 2: bare H1 churn substrate
    let churn_carriers = run(seed, true, true, false, true, 6); // Tier 2 with carriers present

    eprintln!("\n================ V7 / L2.2 — INSTRUMENT VALIDATION against the L2.1 criteria (seed {seed}) ================");

    // ---------- criterion (a) on candidate 0: WL depth 2 ----------
    eprintln!("\nCRITERION (a) BIOLOGY-DISCRIMINATING — candidate 0 = WL depth {WL_DEPTH} (measured on static A and C)");
    eprintln!("  arm | k_c/M   | real CE  | null mean +- sd    | z       | p     | support  | detects");
    eprintln!("  ----+---------+----------+--------------------+---------+-------+----------+--------");
    for (n, r) in [("A", &a), ("C", &c)] {
        eprintln!("   {}  | {:>3}/{:<4}| {:>8.4} | {:>8.4} +- {:.4} | {} | {:>5} | {:.2e} | {}",
            n, r.wl.k_carrier, r.wl.m, r.wl.real_ce, r.wl.null_mean, r.wl.null_sd, zs(r.wl.z()), pstr(&r.wl), r.wl.support, r.wl.detects());
    }
    let a_ok = a.wl.detects();
    let c_ok = !c.wl.detects() && c.wl.z().abs() < 2.0 && c.wl.p() >= ALPHA;
    eprintln!("  => WL: detects A = {a_ok}; dead token |z_C| = {:.2} < 2.0 and p_C = {:.3} >= {ALPHA} = {c_ok}",
        c.wl.z().abs(), c.wl.p());
    eprintln!("  => criterion (a): {}", if a_ok && c_ok { "PASS" } else { "FAIL" });
    eprintln!("  (energy-band is excluded from criterion (a) by construction — K3: dead token z_eb = {:.2}, ~98% definitional)", c.eb.z());

    // ---------- criterion (b), both tiers ----------
    eprintln!("\nCRITERION (b) STATIONARY — H0.1 TWTF <= 0.5 at R = {R_PERSIST}");
    eprintln!("  TIER 1 (L1's own substrate; gates L1):");
    for (n, r) in [("A  static ", &a), ("C  static ", &c), ("B' diffuse", &bp), ("D  diffuse", &d)] {
        eprintln!("    {n}: WL classes {:>4}, TWTF {:.3} => {}", r.classes, r.twtf,
            if r.twtf <= 0.5 { "STATIONARY" } else { "EPHEMERALISED" });
    }
    eprintln!("  TIER 2 (node-turnover churn; the arc-level dilemma):");
    eprintln!("    bare H1 churn substrate: WL classes {:>4}, TWTF {:.3} => {}", churn_bare.classes, churn_bare.twtf,
        if churn_bare.twtf <= 0.5 { "STATIONARY" } else { "EPHEMERALISED" });
    // A Tier-2 reading is worthless unless churn ACTUALLY FIRED — measured, not assumed.
    let churn_active = churn_bare.fresh_nodes > 0;
    eprintln!("    churn ACTIVITY CHECK: {} fresh node identities minted over {STEPS} steps ({:.3}/step), n_end {} => churn {}",
        churn_bare.fresh_nodes, churn_bare.fresh_nodes as f64 / STEPS as f64, churn_bare.n_end,
        if churn_active { "ACTIVE" } else { "*** NEVER FIRED — Tier 2 reading is VACUOUS ***" });
    eprintln!("    (for comparison, static arms mint {} / {} / {} fresh ids)", a.fresh_nodes, bp.fresh_nodes, d.fresh_nodes);
    eprintln!("    WITH CARRIERS on the churn substrate (motile sterile, N=6): WL classes {:>4}, TWTF {:.3} => {}, {} fresh ids, n_end {}",
        churn_carriers.classes, churn_carriers.twtf,
        if churn_carriers.twtf <= 0.5 { "STATIONARY" } else { "EPHEMERALISED" },
        churn_carriers.fresh_nodes, churn_carriers.n_end);

    // ---------- the added power check ----------
    eprintln!("\nPOWER CHECK (added; see header) — does the REALIZED permutation null have any spread?");
    eprintln!("  arm | k_c/M   | transitions | EI_micro | EI_macro | null min / max     | null sd  | distinct (fp-order~) | DEGENERATE?");
    eprintln!("  ----+---------+-------------+----------+----------+--------------------+----------+----------------------+------------");
    for (n, r) in [("A ", &a), ("C ", &c), ("B'", &bp), ("D ", &d)] {
        eprintln!("  {}  | {:>3}/{:<4}| {:>11} | {:>8.4} | {:>8.4} | {:>8.4} / {:<8.4} | {:>8.2e} | {:>20} | {}",
            n, r.wl.k_carrier, r.wl.m, r.wl.transitions, r.wl.ei_micro, r.wl.ei_macro,
            r.wl.null_min, r.wl.null_max, r.wl.null_sd, r.wl.null_distinct,
            if r.wl.null_degenerate() { "*** YES ***" } else { "no" });
    }
    eprintln!("\n  DYNAMICS DIAGNOSTIC — is arm D INERT, or merely INVISIBLE to WL?");
    eprintln!("  WL hashes label + the node_data.energy FIELD + structure; the ALife energy economy lives in");
    eprintln!("  property key 1, which the Q2 invariant EXCLUDES from WL. So metabolic activity is, by design,");
    eprintln!("  a channel WL cannot see. Both channels measured:");
    eprintln!("    arm               | WL-frozen fraction | steps with ALife-energy change | relocations | last-alive");
    eprintln!("    ------------------+--------------------+--------------------------------+-------------+-----------");
    for (n, r) in [("A  replicating   ", &a), ("C  token         ", &c), ("B' motile diffuse", &bp), ("D  stationary    ", &d)] {
        eprintln!("    {n} | {:>18.4} | {:>30} | {:>11} | {:>10}",
            r.frozen_frac, r.energy_active_steps, r.moves, r.last_alive);
    }
    if d.frozen_frac >= 1.0 && d.energy_active_steps > 0 {
        eprintln!("\n    => ARM D IS NOT INERT — it is INVISIBLE. D's graph is structurally CONSTANT (WL-frozen 1.0000,");
        eprintln!("       so the WL TPM is the IDENTITY and EI_macro = {:.4} exactly), yet its ALife energy changes on", d.wl.ei_macro);
        eprintln!("       {} of {} steps. Everything arm D does is metabolic, and metabolism is exactly what WL", d.energy_active_steps, STEPS - 1);
        eprintln!("       excludes by construction. D's non-detection is therefore a statement about the INSTRUMENT'S");
        eprintln!("       BLIND SPOT, not about presence or motility.");
    }

    // ---------- verdict on the instrument ----------
    let tier1_ok = a.twtf <= 0.5 && c.twtf <= 0.5 && bp.twtf <= 0.5 && d.twtf <= 0.5;
    let tier2_ok = churn_bare.twtf <= 0.5 && churn_active;
    let d_measurable = !d.wl.null_degenerate();
    eprintln!("\n================ L2 VERDICT (criteria fixed in 847b36c) ================");
    eprintln!("  criterion (a) biology-discriminating : {}", if a_ok && c_ok { "PASS (WL)" } else { "FAIL" });
    eprintln!("  criterion (b) TIER 1 stationary      : {}", if tier1_ok { "PASS (WL)" } else { "FAIL" });
    eprintln!("  criterion (b) TIER 2 stationary      : {}", if tier2_ok { "PASS (WL)" } else { "FAIL" });
    eprintln!("  arm-D realized null non-degenerate   : {}", if d_measurable { "PASS" } else { "*** FAIL ***" });
    if a_ok && c_ok && tier1_ok && !d_measurable {
        eprintln!("\n  => WL PASSES the two committed L2.1 criteria at Tier 1, BUT arm D's realized null is a SINGLE");
        eprintln!("     POINT (sd = 0, {} distinct value across {N_PERM} permutations). On such a null NO partition", d.wl.null_distinct);
        eprintln!("     could be detected, so arm D's non-detection carries NO information about presence or motility.");
        eprintln!("     Per the L2.1 STOP condition this is reported as UNMEASURABLE / BLOCKED, not as a negative.");
        eprintln!("     The F0 support bar C(M,k) = {:.2e} > 100 PASSED while the realized null was degenerate —", d.wl.support);
        eprintln!("     combinatorial support does NOT imply realized null spread. That is a gap in the standing");
        eprintln!("     criterion and is reported as such.");
    } else if a_ok && c_ok && tier1_ok && d_measurable {
        eprintln!("\n  => WL qualifies at Tier 1 AND arm D is measurable; L1's verdict may be read.");
    } else {
        eprintln!("\n  => see the rows above; L1's verdict may NOT be read on a failing partition.");
    }
    if tier2_ok {
        eprintln!("\n  TIER 2 UPDATE — WL is STATIONARY on the BOUNDED H1 churn substrate with churn demonstrably");
        eprintln!("  active ({} fresh node identities). H0's ephemeralisation verdict was measured on GROWING", churn_bare.fresh_nodes);
        eprintln!("  builtins (growth_treadmill / triangle_subdivide), so it was about substrate GROWTH, not node");
        eprintln!("  turnover as such. Bounded turnover keeps the WL class registry recurrent. Reported as an");
        eprintln!("  update to H0's scope, NOT as a claim that the general instrument dilemma is dissolved.");
    }
    if tier1_ok && !tier2_ok {
        eprintln!("\n  TIER 2 UNRESOLVED (and NOT to be reported as resolved): WL is stationary on the fixed-node-set");
        eprintln!("  diffusion substrate but ephemeralises under node-turnover churn (TWTF {:.3}). The arc still has", churn_bare.twtf);
        eprintln!("  NO partition that is both biology-discriminating and churn-stable.");
    }
    eprintln!("=======================================================================\n");
}
