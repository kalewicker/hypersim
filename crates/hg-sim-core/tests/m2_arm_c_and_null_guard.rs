//! V8 / M2 — does the verdict's one empirical leg survive its degeneracy check?
//!
//! Pre-registration: docs/validation_v8_m2_preregistration.md (commit 0b5fd17),
//! written and committed BEFORE any truncated-window CE for arm C existed.
//!
//! M2.1 arm C is re-scored on its LIVE PHASE (W = 100/110/120), where its carrier
//!      group is populated throughout, instead of the 6000-step window over 98% of
//!      which the group is empty (K=1, CE=0 by construction).
//! M2.2 pre-committed reading: C undetected there => non-detection is substantive
//!      and the empirical leg holds; C DETECTED => the full-window result was the
//!      empty-group artifact, "duration is necessary" becomes "a populated carrier
//!      group is necessary", and the verdict WEAKENS. Mandatory positive control:
//!      if arm A does not detect at that window, nothing is concluded either way.
//! M2.3 the null-spread guard replaces the support bar, is applied RETROACTIVELY to
//!      every G/K/L-track arm, and ships with a FIRING test (arm D must be rejected).
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
const WL_DEPTH: u8 = 2;
const N_PERM: usize = 200;
const ENERGY: u32 = 1;
const ALPHA: f64 = 0.05;
/// M2.3 guard: minimum distinct realised null values (pre-committed)
const DISTINCT_MIN: usize = 10;

fn config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "m2"
run_id = "m2"
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

fn build_rules(cfg: &SimConfig, sterile: bool, motile: bool, diffuse: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if sterile { biology.retain(|r| r.name != "replicate"); }
    if motile { biology.extend(parse_rules_toml(motility_toml()).unwrap()); }
    if diffuse { biology.extend(parse_rules_toml(diffuse_toml()).unwrap()); }
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
pub struct Score {
    pub m: usize, pub k_carrier: usize, pub real_ce: f64,
    pub null_mean: f64, pub null_sd: f64, pub null_min: f64, pub null_max: f64,
    pub null_ge: usize, pub null_distinct: usize,
    pub support: f64, pub transitions: u64,
}
impl Score {
    pub fn p(&self) -> f64 { self.null_ge as f64 / N_PERM as f64 }
    pub fn z(&self) -> f64 {
        if self.k_carrier < 2 || self.null_sd <= 0.0 { f64::NAN } else { (self.real_ce - self.null_mean) / self.null_sd }
    }
    /// the OLD bar: combinatorial support only
    pub fn old_support_bar(&self) -> bool { self.support > 100.0 }
    /// M2.3 NULL-SPREAD GUARD (pre-committed in 0b5fd17): admissible iff the realised
    /// null actually varies. Clauses 1-2 are exact-zero tests and carry the decision;
    /// clause 3 is the near-degeneracy bar.
    pub fn null_spread_ok(&self) -> bool {
        self.null_sd > 0.0 && (self.null_max - self.null_min) > 0.0 && self.null_distinct >= DISTINCT_MIN
    }
    pub fn degenerate(&self) -> bool { self.null_sd <= 0.0 || (self.null_max - self.null_min) <= 0.0 }
    pub fn detects(&self) -> bool {
        self.k_carrier >= 2 && self.k_carrier < self.m && self.p() < ALPHA
            && self.old_support_bar() && self.null_spread_ok()
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
        null_ge: ge, null_distinct: seen.len(),
        support: null_support(m, k_carrier), transitions: tpm.total_transitions(),
    }
}

struct Res { wl: Score, eb: Score, last_alive: u64, final_pop: usize, empty_frac: f64 }

fn run(seed: u64, sterile: bool, motile: bool, diffuse: bool, markers: usize, steps: u64) -> Res {
    let cfg = config(seed, steps);
    let rules = build_rules(&cfg, sterile, motile, diffuse);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    plant_markers(&mut store, seed, markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);
    let mut wl_tpm = TpmAccumulator::new();
    let mut eb_tpm = TpmAccumulator::new();
    let mut wl_c: HashMap<u64, bool> = HashMap::new();
    let mut eb_c: HashMap<u64, bool> = HashMap::new();
    wl_c.insert(VACUUM.0, false);
    eb_c.insert(VACUUM.0, false);
    let mut wl_prev: Option<HashMap<NodeId, u64>> = None;
    let mut eb_prev: Option<HashMap<NodeId, u64>> = None;
    let (mut last_alive, mut final_pop, mut empty_steps) = (0u64, 0usize, 0u64);

    for step in 1..=steps {
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
        if live > 0 { last_alive = step; } else { empty_steps += 1; }
        final_pop = live;
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
    Res {
        wl: score(&wl_tpm, &wl_c, seed), eb: score(&eb_tpm, &eb_c, seed),
        last_alive, final_pop, empty_frac: empty_steps as f64 / steps as f64,
    }
}

fn pstr(s: &Score) -> String {
    if s.k_carrier < 2 { "  n/a".into() }
    else if s.null_ge == 0 { "<.005".into() }
    else { format!("{:.3}", s.p()) }
}
fn zs(x: f64) -> String { if x.is_nan() { "    n/a".into() } else { format!("{x:7.2}") } }

fn row(name: &str, s: &Score) {
    eprintln!("  {:<28} | {:>3}/{:<4}| {:>8} | {:>8.4} | {:>8.4} +- {:<8.2e} | {:>8.4}/{:<8.4} | {:>5} | {} | {:>5} | {:.2e} | {:>6} | {}",
        name, s.k_carrier, s.m, s.transitions, s.real_ce, s.null_mean, s.null_sd,
        s.null_min, s.null_max, s.null_distinct, zs(s.z()), pstr(s), s.support,
        if s.null_spread_ok() { "ok" } else { "DEGEN" },
        if s.detects() { "DETECTS" } else { "no" });
}

#[test]
fn m2_arm_c_truncated_and_null_guard() {
    let seed = 12345;
    eprintln!("\n================ V8 / M2 — DOES THE VERDICT'S ONE EMPIRICAL LEG SURVIVE? (seed {seed}) ================");
    eprintln!("  header: k_c/M | transitions | real CE | null mean +- sd | null min/max | distinct | z | p | support | spread | verdict");

    // ---------- M2.1 truncated-window scoring, with the mandatory positive control ----------
    for w in [100u64, 110, 120] {
        eprintln!("\n  --- TRUNCATED WINDOW W = {w} (arm C's carriers are alive through step 120) ---");
        eprintln!("  arm                          | k_c/M   | trans    | real CE  | null mean +- sd       | null min/max        | dist. | z       | p     | support  | spread | verdict");
        let a = run(seed, false, false, false, 1, w);
        let b = run(seed, true, true, false, 6, w);
        let bp = run(seed, true, true, true, 6, w);
        let d = run(seed, true, false, true, 6, w);
        let c = run(seed, true, false, false, 6, w);
        row("A  replicating (CONTROL)", &a.wl);
        row("B  motile (static)", &b.wl);
        row("B' motile (diffuse)", &bp.wl);
        row("D  stationary (diffuse)", &d.wl);
        row("C  token", &c.wl);
        eprintln!("    arm-A positive control detects at W={w}: {}   |   arm C detects: {}",
            a.wl.detects(), c.wl.detects());
    }

    // ---------- full-window reference ----------
    eprintln!("\n  --- FULL WINDOW W = 6000 (reference; arm C's group is EMPTY for 98% of it) ---");
    eprintln!("  arm                          | k_c/M   | trans    | real CE  | null mean +- sd       | null min/max        | dist. | z       | p     | support  | spread | verdict");
    let a6 = run(seed, false, false, false, 1, 6000);
    let b6 = run(seed, true, true, false, 6, 6000);
    let bp6 = run(seed, true, true, true, 6, 6000);
    let d6 = run(seed, true, false, true, 6, 6000);
    let c6 = run(seed, true, false, false, 6, 6000);
    row("A  replicating", &a6.wl);
    row("B  motile (static)", &b6.wl);
    row("B' motile (diffuse)", &bp6.wl);
    row("D  stationary (diffuse)", &d6.wl);
    row("C  token", &c6.wl);
    eprintln!("    arm C: last-alive {}, carrier group EMPTY for {:.1}% of the window",
        c6.last_alive, 100.0 * c6.empty_frac);

    // ---------- M2.3 retroactive null-spread audit ----------
    eprintln!("\n================ M2.3 — RETROACTIVE NULL-SPREAD AUDIT of every published arm ================");
    eprintln!("  guard (0b5fd17): null_sd > 0 AND null_max-null_min > 0 AND >= {DISTINCT_MIN} distinct null values");
    eprintln!("  arm / partition                    | support  | old bar | null sd    | max-min    | distinct | NEW GUARD | published p");
    eprintln!("  -----------------------------------+----------+---------+------------+------------+----------+-----------+------------");
    let audit: Vec<(&str, &Score)> = vec![
        ("A  replicating   WL", &a6.wl), ("A  replicating   e-band", &a6.eb),
        ("B  motile static WL", &b6.wl), ("B  motile static e-band", &b6.eb),
        ("B' motile diff   WL", &bp6.wl), ("B' motile diff   e-band", &bp6.eb),
        ("D  stationary    WL", &d6.wl), ("D  stationary    e-band", &d6.eb),
        ("C  token         WL", &c6.wl), ("C  token         e-band", &c6.eb),
    ];
    let mut flagged: Vec<&str> = Vec::new();
    for (name, s) in &audit {
        let guard = s.null_spread_ok();
        if !guard { flagged.push(name); }
        eprintln!("  {:<34} | {:.2e} | {:>7} | {:>10.3e} | {:>10.3e} | {:>8} | {:>9} | {}",
            name, s.support, s.old_support_bar(), s.null_sd, s.null_max - s.null_min,
            s.null_distinct, if guard { "PASS" } else { "*REJECT*" }, pstr(s));
    }
    eprintln!("\n  ARMS WHOSE PUBLISHED p RESTS ON A DEGENERATE/NEAR-DEGENERATE NULL: {:?}", flagged);

    // ---------- FIRING TEST (V8 rule 5): the guard must REJECT something ----------
    eprintln!("\n================ FIRING TEST (rule 5) — the guard must actually REJECT ================");
    eprintln!("  arm D WL: support {:.2e} (old bar PASSES: {}), null_sd {:.3e}, max-min {:.3e}, distinct {}",
        d6.wl.support, d6.wl.old_support_bar(), d6.wl.null_sd, d6.wl.null_max - d6.wl.null_min, d6.wl.null_distinct);
    eprintln!("  old support bar accepts it: {}   |   NEW null-spread guard rejects it: {}",
        d6.wl.old_support_bar(), !d6.wl.null_spread_ok());
    assert!(d6.wl.old_support_bar(), "arm D should pass the OLD bar — that is the defect being fixed");
    assert!(!d6.wl.null_spread_ok(), "FIRING TEST FAILED: the new guard must REJECT arm D");
    eprintln!("  => the guard FIRES on a real case the old bar accepted. It is tested, not merely asserted.");

    // ---------- M2.2 pre-committed reading ----------
    let c120 = run(seed, true, false, false, 6, 120);
    let a120 = run(seed, false, false, false, 1, 120);
    eprintln!("\n================ M2.2 — PRE-COMMITTED READING (fixed in 0b5fd17) ================");
    if !a120.wl.detects() {
        eprintln!("  INCONCLUSIVE: the arm-A positive control does NOT detect at W=120, so the truncated test has");
        eprintln!("  no power and NOTHING is concluded from arm C either way (pre-committed).");
        eprintln!("    A at W=120: k_c {} / M {}, p {}, support {:.2e}, spread {}",
            a120.wl.k_carrier, a120.wl.m, pstr(&a120.wl), a120.wl.support, a120.wl.null_spread_ok());
    } else if c120.wl.degenerate() {
        // The pre-registration's FOURTH clause, which the other branches must not pre-empt:
        // a degenerate truncated null means C is UNMEASURABLE here, exactly as D is, so its
        // non-detection cannot be read as substantive.
        eprintln!("  *** C's TRUNCATED NULL IS ITSELF DEGENERATE *** (sd = {:.3e}, max-min = {:.3e}, {} distinct value).",
            c120.wl.null_sd, c120.wl.null_max - c120.wl.null_min, c120.wl.null_distinct);
        eprintln!("  So TRUNCATION CANNOT RESOLVE THE AMBIGUITY: over its live phase arm C's graph is structurally");
        eprintln!("  near-constant (its only structural events are 6 births at step 1 and 6 deaths at step 120),");
        eprintln!("  which puts it in the SAME BLIND SPOT as arm D. C is partly unmeasurable in exactly D's way.");
        eprintln!("  Per the pre-committed fourth clause: THE EMPIRICAL LEG IS THINNER STILL. C's FULL-window");
        eprintln!("  non-detection remains a valid measurement (spread ok, {} distinct, z {:.2}, p {:.3}), but the",
            c6.wl.null_distinct, c6.wl.z(), c6.wl.p());
        eprintln!("  specific challenge -- is that non-detection an empty-group artifact? -- is NOT answered here,");
        eprintln!("  and must not be reported as answered in either direction.");
    } else if c120.wl.detects() {
        eprintln!("  *** C DETECTS ON THE TRUNCATED WINDOW *** => the full-window non-detection was the EMPTY-GROUP");
        eprintln!("  ARTIFACT. 'Duration is necessary' must be restated as 'A POPULATED CARRIER GROUP IS NECESSARY',");
        eprintln!("  which is a claim about partition well-formedness rather than about biology. The");
        eprintln!("  sustained-structural-activity verdict WEAKENS accordingly and the stronger framing is dropped.");
    } else {
        eprintln!("  C does NOT detect on the truncated window while arm A DOES => C's non-detection is SUBSTANTIVE.");
        eprintln!("  Activity share is the variable, and the sustained-structural-activity verdict's empirical leg HOLDS.");
    }
    eprintln!("    C truncated (W=120): k_c {}/{}, real CE {:.4}, null sd {:.3e}, distinct {}, z {}, p {}, spread-ok {}",
        c120.wl.k_carrier, c120.wl.m, c120.wl.real_ce, c120.wl.null_sd, c120.wl.null_distinct,
        zs(c120.wl.z()), pstr(&c120.wl), c120.wl.null_spread_ok());
    eprintln!("======================================================================\n");

    // ---------- EXPLORATORY, clearly labelled as NOT part of the pre-committed reading ----------
    // The pre-registration fixed W = 100/110/120 ("populated throughout"). Windows that CONTAIN
    // C's death events ask a DIFFERENT question, so they are reported separately and are not
    // allowed to stand in for the pre-committed result above.
    eprintln!("================ EXPLORATORY (POST-HOC, NOT PRE-REGISTERED) ================");
    eprintln!("  The pre-registered windows end at or before C's die-off, so C's only in-window structural");
    eprintln!("  events are 6 births at step 1. Windows that CONTAIN the deaths ask a different question.");
    eprintln!("  Reported for completeness; NOT substituted for the pre-committed reading.");
    eprintln!("  arm                          | k_c/M   | trans    | real CE  | null mean +- sd       | null min/max        | dist. | z       | p     | support  | spread | verdict");
    for w in [130u64, 150, 200, 400] {
        let cw = run(seed, true, false, false, 6, w);
        row(&format!("C  token  W={w}"), &cw.wl);
    }
    eprintln!("===========================================================================\n");
}
