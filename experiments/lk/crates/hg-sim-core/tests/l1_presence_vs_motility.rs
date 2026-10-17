//! V7 / L1.3-L1.4 — the presence-vs-motility comparison: is a carrier that is
//! PRESENT but does not MOVE, does not REPLICATE and does not TURN OVER still
//! detectable?
//!
//! Mechanism pre-committed in docs/validation_v7_l1_sustain_mechanism.md (01bf010);
//! the substrate was characterised bare as physics in l1_diffusion_substrate.rs
//! (8f6acf2, firewall holds). The reading rule below was fixed in that same doc
//! BEFORE any number here existed.
//!
//!   arm | replicate | relocate | diffuse
//!   ----+-----------+----------+--------
//!    A  |    yes    |    no    |   no     replicating baseline (F8)
//!    B  |    no     |   YES    |   no     K-track arm B (motile, 248 relocations)
//!    B' |    no     |   YES    |  YES     motile on D's OWN substrate (the clean control)
//!    D  |    no     |    no    |  YES     THE SPECIMEN: present, stationary
//!    C  |    no     |    no    |   no     the token, dead at 120
//!
//! D vs B' isolates MOTILITY (same substrate, differ in `relocate` alone) and is
//! the decisive contrast. D vs C isolates PRESENCE but is NOT single-variable —
//! sustaining a stationary carrier requires the energy-delivery change — which is
//! why D-vs-B' carries the verdict. Stated in advance, in L1.1.
//!
//! Per V7 rule 3 the detectability VERDICT may only be read on a partition that
//! passed the L2.1 Tier-1 criteria (l2_instrument_validation.rs). This harness
//! reports Tier-1 stationarity (TWTF) for each arm's own substrate alongside the
//! scores so the gate is checkable here.
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
experiment_name = "l1pm"
run_id = "l1pm"
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

/// byte-identical to h2_step3_split.rs / k1 / k2 / k3
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

/// byte-identical to the rule characterised bare in l1_diffusion_substrate.rs
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
struct Score { m: usize, k_carrier: usize, real_ce: f64, null_mean: f64, null_sd: f64, null_ge: usize, support: f64 }
impl Score {
    fn p(&self) -> f64 { self.null_ge as f64 / N_PERM as f64 }
    fn z(&self) -> f64 {
        if self.k_carrier < 2 || self.null_sd <= 0.0 { f64::NAN } else { (self.real_ce - self.null_mean) / self.null_sd }
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

struct Res {
    wl: Score, eb: Score,
    peak: usize, final_pop: usize, last_alive: u64,
    births: u64, deaths: u64, moves: u64,
    wl_twtf: f64, wl_classes: usize,
}

fn run(seed: u64, sterile: bool, motile: bool, diffuse: bool, n_markers: usize) -> Res {
    let cfg = config(seed);
    let rules = build_rules(&cfg, sterile, motile, diffuse);
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
    let mut prev_carriers: HashSet<NodeId> = HashSet::new();
    let mut site_prev: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let (mut peak, mut final_pop, mut last_alive) = (0usize, 0usize, 0u64);
    let (mut births, mut deaths, mut moves) = (0u64, 0u64, 0u64);
    // Tier-1 stationarity (H0.1 TWTF, R=3) on THIS arm's own substrate
    let mut wl_live: HashMap<u64, u64> = HashMap::new();
    let mut wl_src: HashMap<u64, u64> = HashMap::new();
    wl_live.insert(VACUUM.0, u64::MAX);

    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut wl_now: HashMap<NodeId, u64> = HashMap::new();
        let mut eb_now: HashMap<NodeId, u64> = HashMap::new();
        let mut carriers: HashSet<NodeId> = HashSet::new();
        let mut site_now: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut present: HashSet<u64> = HashSet::new();
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
            present.insert(wh);
            wl_c.entry(wh).or_insert(is_carrier);
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let eb = energy_band_class(d.label.0 as i64, e);
            eb_now.insert(id, eb);
            eb_c.entry(eb).or_insert(is_carrier);
        }
        for h in &present { *wl_live.entry(*h).or_insert(0) += 1; }
        births += carriers.difference(&prev_carriers).count() as u64;
        deaths += prev_carriers.difference(&carriers).count() as u64;
        for (id, s) in &site_now {
            if let Some(old) = site_prev.get(id) { if old != s { moves += 1; } }
        }
        site_prev = site_now;
        peak = peak.max(carriers.len());
        final_pop = carriers.len();
        if !carriers.is_empty() { last_alive = step; }
        prev_carriers = carriers;

        if let Some(p) = &wl_prev {
            for (id, &hp) in p {
                wl_tpm.record(WlHash(hp), WlHash(wl_now.get(id).copied().unwrap_or(VACUUM.0)));
                *wl_src.entry(hp).or_insert(0) += 1;
            }
            for (id, &hn) in &wl_now {
                if !p.contains_key(id) { wl_tpm.record(VACUUM, WlHash(hn)); *wl_src.entry(VACUUM.0).or_insert(0) += 1; }
            }
        }
        if let Some(p) = &eb_prev {
            for (id, &hp) in p { eb_tpm.record(WlHash(hp), WlHash(eb_now.get(id).copied().unwrap_or(VACUUM.0))); }
            for (id, &hn) in &eb_now { if !p.contains_key(id) { eb_tpm.record(VACUUM, WlHash(hn)); } }
        }
        wl_prev = Some(wl_now);
        eb_prev = Some(eb_now);
    }

    let total: u64 = wl_src.values().sum();
    let transient: u64 = wl_src.iter()
        .filter(|(h, _)| *wl_live.get(*h).unwrap_or(&0) < R_PERSIST)
        .map(|(_, c)| *c).sum();

    Res {
        wl: score(&wl_tpm, &wl_c, seed),
        eb: score(&eb_tpm, &eb_c, seed),
        peak, final_pop, last_alive, births, deaths, moves,
        wl_twtf: if total > 0 { transient as f64 / total as f64 } else { 0.0 },
        wl_classes: wl_live.len().saturating_sub(1),
    }
}

fn pstr(s: &Score) -> String {
    if s.k_carrier < 2 { "  n/a".into() }
    else if s.null_ge == 0 { "<.005".into() }
    else { format!("{:.3}", s.p()) }
}
fn zs(x: f64) -> String { if x.is_nan() { "    n/a".into() } else { format!("{x:7.2}") } }

#[test]
fn l1_presence_vs_motility() {
    let seed = 12345;
    //                  sterile, motile, diffuse, markers
    let a = run(seed, false, false, false, 1);
    let b = run(seed, true,  true,  false, 6);
    let bp = run(seed, true, true,  true,  6);
    let d = run(seed, true,  false, true,  6);
    let c = run(seed, true,  false, false, 6);
    let arms: [(&str, &Res); 5] = [
        ("A  replicating (F8)", &a),
        ("B  non-repl MOTILE (static)", &b),
        ("B' non-repl MOTILE (diffuse)", &bp),
        ("D  non-repl STATIONARY (diffuse)", &d),
        ("C  non-repl token (static)", &c),
    ];

    eprintln!("\n================ V7 / L1.3 — FOUR-WAY LADDER, REALIZED DEMOGRAPHY (seed {seed}) ================");
    eprintln!("  arm                              | peak | final | last-alive | births | deaths | RELOCATIONS");
    eprintln!("  ---------------------------------+------+-------+------------+--------+--------+------------");
    for (name, r) in arms {
        eprintln!("  {:<32} | {:>4} | {:>5} | {:>10} | {:>6} | {:>6} | {:>11}",
            name, r.peak, r.final_pop, r.last_alive, r.births, r.deaths, r.moves);
    }
    let d_stationary = d.moves == 0;
    let d_present = d.last_alive >= STEPS && d.final_pop > 0;
    let bp_moves = bp.moves > 0;
    eprintln!("\n  SPECIMEN CHECK — D must be PRESENT and STATIONARY, and B' must still MOVE (else D-vs-B' is degenerate):");
    eprintln!("    D present to step {STEPS}: {d_present} (final pop {});  D relocations == 0: {d_stationary}", d.final_pop);
    eprintln!("    B' still relocates: {bp_moves} ({} relocations)", bp.moves);

    eprintln!("\n================ L1.4 — SCORES, both partitions (z = (real-null_mean)/null_sd; DETECTABILITY only) ================");
    eprintln!("  arm                              || WL: k_c/M    | real CE  | z       | p     | support  | TWTF  || eb: k_c/M   | z       | p");
    eprintln!("  ---------------------------------++--------------+----------+---------+-------+----------+-------++-------------+---------+------");
    for (name, r) in arms {
        eprintln!("  {:<32} || {:>3}/{:<4}     | {:>8.4} | {} | {:>5} | {:.2e} | {:.3} || {:>3}/{:<4}    | {} | {:>5}",
            name, r.wl.k_carrier, r.wl.m, r.wl.real_ce, zs(r.wl.z()), pstr(&r.wl), r.wl.support, r.wl_twtf,
            r.eb.k_carrier, r.eb.m, zs(r.eb.z()), pstr(&r.eb));
    }

    eprintln!("\n  TIER-1 STATIONARITY (L2.1 criterion b, H0.1 TWTF<=0.5 at R=3) on each arm's OWN substrate:");
    for (name, r) in arms {
        eprintln!("    {:<32} WL classes {:>4}, TWTF {:.3}  => {}",
            name, r.wl_classes, r.wl_twtf, if r.wl_twtf <= 0.5 { "STATIONARY" } else { "EPHEMERALISED" });
    }

    // ---------- pre-committed reading (docs/validation_v7_l1_sustain_mechanism.md) ----------
    eprintln!("\n================ L1.4 — PRE-COMMITTED READING (rule fixed in 01bf010) ================");
    eprintln!("  WL detects:  A={} B={} B'={} D={} C={}", a.wl.detects(), b.wl.detects(), bp.wl.detects(), d.wl.detects(), c.wl.detects());
    eprintln!("  eb detects:  A={} B={} B'={} D={} C={}  (always-on calibrator; NOT biology-discriminating, K3)",
        a.eb.detects(), b.eb.detects(), bp.eb.detects(), d.eb.detects(), c.eb.detects());
    if !d_present || !d_stationary {
        eprintln!("  SPECIMEN INVALID — D is not both present and stationary; no verdict is read.");
    } else if d.wl.detects() && !c.wl.detects() {
        eprintln!("  => PRESENCE, NOT MOTILITY, IS LOAD-BEARING. D is present, stationary (0 relocations),");
        eprintln!("     non-replicating and non-turnover, and is STILL detectable, while the dead token C is not.");
        eprintln!("     A carrier is detectable merely by being structurally present across time — independent of");
        eprintln!("     replication, turnover AND motion. This is the strongest defensible form of the verdict.");
        if bp_moves {
            eprintln!("     The D-vs-B' contrast is non-degenerate (B' relocated {} times on the same substrate),", bp.moves);
            eprintln!("     so motility is isolated cleanly and is shown NOT to be the operative variable.");
        }
    } else if !d.wl.detects() && (b.wl.detects() || bp.wl.detects()) {
        eprintln!("  => MOTILITY WAS THE OPERATIVE VARIABLE. The stationary present carrier D is NOT detectable");
        eprintln!("     while the motile ones are. The property is structural ACTIVITY FROM MOTION, arm B's");
        eprintln!("     earlier detectability is reattributed to its relocations, and the verdict narrows toward");
        eprintln!("     construction. Per V7 rule 2 this is the finding: motion is NOT added back and 'present'");
        eprintln!("     is NOT redefined.");
    } else {
        eprintln!("  => MIXED/OTHER — read the table; do not force it into either branch.");
    }
    eprintln!("  (Verdict valid ONLY if L2 validated the WL partition at Tier 1 — see l2_instrument_validation.rs.)");
    eprintln!("======================================================================================\n");
}
