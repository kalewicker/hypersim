//! V9 / Phase 1 — validate the metabolic instrument: is the closure-defect estimator
//! on the energy channel a VALIDATED statistic, or only a discriminating one?
//!
//! Pre-registration: docs/validation_v9_phase1_preregistration.md (19ebd70), committed
//! before any metabolic score existed. Burn-in criterion: N0 (d853f94).
//!
//! Micro state is DYNAMICAL, never a level (the energy-band partition is ~98%
//! definitional and is excluded everywhere):
//!     micro = (label, bucket(delta_t), bucket(delta_{t-1}), ... up to depth R)
//! An INERT carrier has delta == 0 forever, so it occupies exactly one micro class and
//! cannot be resolved -- the correct answer, not a defect.
//!
//! Depth R is fixed BY RULE on ARM A ONLY (the known F8 baseline, not the thing under
//! test): the smallest R in {1,2,3} for which arm A satisfies k>=2, k<M, support>100
//! AND the realised-spread guard. Then frozen for all arms and seeds. (The F1 lesson:
//! re-selecting depth after seeing results is the forbidden move.)
//!
//! Two quantities on the SAME TPM and null, directions pre-committed:
//!   CE on the metabolic channel  -- detected iff real EXCEEDS null (p counts null >= real)
//!   closure defect H(micro|macro) -- a partition that EXPLAINS the dynamics gives a LOWER
//!                                    defect, so its p counts null <= real (opposite side)
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{HypergraphStore, NodeId, PropertyValue, WlHash};
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

fn config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "n1"
run_id = "n1"
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

pub fn arm_spec(a: Arm) -> (bool, bool, bool, usize) { // sterile, motile, diffuse, markers
    match a {
        Arm::A => (false, false, false, 1),
        Arm::B => (true, true, false, 6),
        Arm::BPrime => (true, true, true, 6),
        Arm::C => (true, false, false, 6),
        Arm::D => (true, false, true, 6),
        Arm::Inert => (true, false, false, 6), // rules stripped below
    }
}

fn build_rules(cfg: &SimConfig, arm: Arm, churn: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if arm == Arm::Inert {
        // INERT: keep only bootstrap_seed — a carrier exists and no economy rule touches it.
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

/// Quantised signed magnitude of an energy delta. Bounded and recurrent (H0.1 closure).
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

#[derive(Clone, Default)]
pub struct Score {
    pub m: usize, pub k_carrier: usize, pub real: f64,
    pub null_mean: f64, pub null_sd: f64, pub null_min: f64, pub null_max: f64, pub null_p95: f64,
    pub null_ge: usize, pub null_distinct: usize, pub support: f64, pub transitions: u64,
}
impl Score {
    pub fn p(&self) -> f64 { self.null_ge as f64 / N_PERM as f64 }
    pub fn z(&self) -> f64 {
        if self.k_carrier < 2 || self.null_sd <= 0.0 { f64::NAN } else { (self.real - self.null_mean) / self.null_sd }
    }
    /// realised-spread guard, applied FROM BIRTH (never retrofitted)
    pub fn spread_ok(&self) -> bool {
        self.null_sd > 0.0 && (self.null_max - self.null_min) > 0.0 && self.null_distinct >= DISTINCT_MIN
    }
    pub fn resolved(&self) -> bool { self.k_carrier >= 2 && self.k_carrier < self.m && self.support > 100.0 }
    pub fn detects(&self) -> bool { self.resolved() && self.spread_ok() && self.p() < ALPHA }
}

/// Permutation-scored statistic. `higher_is_detection` picks the tail:
/// CE counts null >= real; the closure defect counts null <= real (opposite side).
fn score_tpm(
    tpm: &TpmAccumulator,
    is_c: &HashMap<u64, bool>,
    seed: u64,
    stat: &dyn Fn(&TpmAccumulator, &[usize], usize) -> f64,
    higher_is_detection: bool,
) -> Score {
    let mut compact: HashMap<usize, usize> = HashMap::new();
    let group_of: Vec<usize> = tpm.hashes().iter().map(|h| {
        let raw = *is_c.get(&h.0).unwrap_or(&false) as usize;
        let n = compact.len();
        *compact.entry(raw).or_insert(n)
    }).collect();
    let k = compact.len().max(1);
    let m = tpm.hashes().len();
    let k_carrier = tpm.hashes().iter().filter(|h| *is_c.get(&h.0).unwrap_or(&false)).count();
    let real = if k >= 2 { stat(tpm, &group_of, k) } else { 0.0 };
    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7 ^ seed;
    let (mut ge, mut sum, mut sumsq, mut mn, mut mx) = (0usize, 0.0f64, 0.0f64, f64::MAX, f64::MIN);
    let mut seen: HashSet<u64> = HashSet::new();
    let mut vals: Vec<f64> = Vec::with_capacity(N_PERM);
    if k >= 2 {
        for _ in 0..N_PERM {
            let mut labels: Vec<usize> = canon.iter().map(|&i| group_of[i]).collect();
            for i in (1..labels.len()).rev() {
                let j = (splitmix(&mut rng) % (i as u64 + 1)) as usize;
                labels.swap(i, j);
            }
            let mut perm = vec![0usize; group_of.len()];
            for (pos, &i) in canon.iter().enumerate() { perm[i] = labels[pos]; }
            let c = stat(tpm, &perm, k);
            sum += c; sumsq += c * c; mn = mn.min(c); mx = mx.max(c);
            seen.insert(c.to_bits());
            vals.push(c);
            if (higher_is_detection && c >= real) || (!higher_is_detection && c <= real) { ge += 1; }
        }
    }
    let n = N_PERM as f64;
    let mean = sum / n;
    let var = (sumsq / n - mean * mean).max(0.0);
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Score {
        m, k_carrier, real, null_mean: mean, null_sd: var.sqrt(),
        null_min: if mn == f64::MAX { 0.0 } else { mn },
        null_max: if mx == f64::MIN { 0.0 } else { mx },
        null_p95: vals.get((0.95 * n) as usize).copied().unwrap_or(0.0),
        null_ge: ge, null_distinct: seen.len(),
        support: null_support(m, k_carrier), transitions: tpm.total_transitions(),
    }
}

/// Entropy with the terms summed in ascending KEY order (C3.1). Deterministic across
/// processes; `-m.values().map(..).sum()` is not, because `HashMap` iteration order is
/// per-process and float addition is non-associative.
fn h_sorted<K: Ord + Copy + std::hash::Hash>(m: &HashMap<K, f64>, tot: f64) -> f64 {
    if tot <= 0.0 { return 0.0; }
    let mut ks: Vec<K> = m.keys().copied().collect();
    ks.sort_unstable();
    -ks.iter().map(|k| { let p = m[k] / tot; if p > 0.0 { p * p.log2() } else { 0.0 } }).sum::<f64>()
}

/// Closure defect H(micro transition | macro transition) for a given grouping.
fn closure_defect(tpm: &TpmAccumulator, g: &[usize], _k: usize) -> f64 {
    // joint counts over (macro_from, macro_to) x (micro_from, micro_to)
    let rows = tpm.normalized_rows();
    let counts = tpm.counts_dense();
    let mut joint: HashMap<(usize, usize, usize, usize), f64> = HashMap::new();
    let mut macro_c: HashMap<(usize, usize), f64> = HashMap::new();
    for (i, (_, _row)) in rows.iter().enumerate() {
        for (j, &c) in counts[i].iter().enumerate() {
            if c == 0 { continue; }
            let cf = c as f64;
            let (gi, gj) = (g[i], g[j]);
            *joint.entry((gi, gj, i, j)).or_insert(0.0) += cf;
            *macro_c.entry((gi, gj)).or_insert(0.0) += cf;
        }
    }
    let tot: f64 = macro_c.values().sum(); // integer-valued counts: exact in f64, order-safe
    if tot <= 0.0 { return 0.0; }
    // C3.1: the entropy terms are summed in ascending KEY order. HashMap iteration order is
    // seeded per PROCESS by RandomState and float addition is non-associative, so summing in
    // iteration order makes this estimator differ in the last ulps between processes even
    // when recording is sorted (measured at n = 3 in c3_closure_defect_order_gate.rs).
    let h_macro = h_sorted(&macro_c, tot);
    let h_joint = h_sorted(&joint, tot);
    (h_joint - h_macro).max(0.0)
}

pub struct ArmRun {
    pub ce: Score,
    pub cd: Score,
    pub last_alive: u64,
    pub final_pop: usize,
    /// index-dependent reproducibility fingerprint (N1.3): ordered class hashes + row sums
    pub fingerprint: u64,
}

/// SORTED-order TPM recording (C3.1 / N1.3): the registry order is then reproducible
/// across processes, which is what makes an index-dependent fingerprint meaningful.
fn record_sorted(tpm: &mut TpmAccumulator, prev: &Option<HashMap<NodeId, u64>>, now: &HashMap<NodeId, u64>) {
    if let Some(p) = prev {
        let mut carry: Vec<(NodeId, u64)> = p.iter().map(|(k, v)| (*k, *v)).collect();
        let mut born: Vec<(NodeId, u64)> = now.iter()
            .filter(|(id, _)| !p.contains_key(id)).map(|(k, v)| (*k, *v)).collect();
        carry.sort_unstable();
        born.sort_unstable();
        for (id, hp) in carry { tpm.record(WlHash(hp), WlHash(now.get(&id).copied().unwrap_or(VACUUM.0))); }
        for (_, hn) in born { tpm.record(VACUUM, WlHash(hn)); }
    }
}

/// Build the metabolic TPM for one arm at depth R, scoring only steps > burn_in.
pub fn run_arm(seed: u64, arm: Arm, churn: bool, depth: usize, steps: u64, burn_in: u64) -> ArmRun {
    let cfg = config(seed, steps);
    let rules = build_rules(&cfg, arm, churn);
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

    for step in 1..=steps {
        engine.step(&mut store);
        let snap = store.snapshot();
        let mut now: HashMap<NodeId, u64> = HashMap::new();
        let mut live = 0usize;
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            if is_carrier { live += 1; }
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            let hv = hist.entry(id).or_default();
            hv.insert(0, bucket(delta));
            hv.truncate(depth);
            if hv.len() == depth {
                let cls = micro_class(d.label.0 as i64, hv);
                now.insert(id, cls);
                is_c.entry(cls).or_insert(is_carrier);
            }
            prev_e.insert(id, e);
        }
        if live > 0 { last_alive = step; }
        final_pop = live;
        if step > burn_in {
            // C3.1 / N1.3: SORTED-order recording. Iterating the per-step node maps in
            // HashMap order gives each PROCESS a different class-registry order, which is
            // exactly what makes `fingerprint` below non-reproducible. The scalars are
            // unaffected to printed precision, but an index-dependent statistic is not.
            record_sorted(&mut tpm, &prev_cls, &now);
        }
        prev_cls = Some(now);
    }

    let ce = score_tpm(&tpm, &is_c, seed, &ce_for, true);
    let cd = score_tpm(&tpm, &is_c, seed, &closure_defect, false);
    // index-dependent fingerprint: ordered class hashes AND per-class row sums (the §6 lesson —
    // a scalar coefficient can agree while an index-dependent statistic diverges)
    let mut fp = 0xCBF2_9CE4_8422_2325u64;
    let counts = tpm.counts_dense();
    for (i, h) in tpm.hashes().iter().enumerate() {
        fp = (fp ^ h.0).wrapping_mul(0x1000_0000_01B3);
        let rs: f64 = counts[i].iter().map(|&c| c as f64).sum();
        fp = (fp ^ rs.to_bits()).wrapping_mul(0x1000_0000_01B3);
    }
    ArmRun { ce, cd, last_alive, final_pop, fingerprint: fp }
}

fn pstr(s: &Score) -> String {
    if !s.resolved() { "  n/a".into() }
    else if s.null_ge == 0 { "<.005".into() }
    else { format!("{:.3}", s.p()) }
}
fn zs(x: f64) -> String { if x.is_nan() { "    n/a".into() } else { format!("{x:7.2}") } }

fn row(name: &str, s: &Score) {
    eprintln!("  {:<24} | {:>3}/{:<4}| {:>8} | {:>8.4} | {:>8.4} | {:>8.4} | {:>8.4} | {:>9.2e} | {:>4} | {} | {:>5} | {:.2e} | {:>6}",
        name, s.k_carrier, s.m, s.transitions, s.real, s.null_mean, s.null_p95, s.null_max,
        s.null_sd, s.null_distinct, zs(s.z()), pstr(s), s.support,
        if s.spread_ok() { "ok" } else { "DEGEN" });
}

#[test]
fn n1_metabolic_instrument_validation() {
    let seed = 12345;
    eprintln!("\n================ V9 / PHASE 1 — VALIDATING THE METABOLIC INSTRUMENT (seed {seed}) ================");

    // ---------- depth R fixed BY RULE on ARM A ONLY (pre-registered) ----------
    eprintln!("\nDEPTH SELECTION — the pre-registered rule, evaluated on ARM A ONLY (the known F8 baseline):");
    eprintln!("  R = smallest depth in {{1,2,3}} with arm A: k>=2, k<M, support>100, and spread-ok");
    eprintln!("  R | k_c/M   | support  | null sd    | distinct | qualifies");
    let mut chosen: Option<usize> = None;
    for r in 1..=3usize {
        let a = run_arm(seed, Arm::A, false, r, STEPS, 0);
        let q = a.ce.resolved() && a.ce.spread_ok();
        eprintln!("  {r} | {:>3}/{:<4}| {:.2e} | {:>10.2e} | {:>8} | {}",
            a.ce.k_carrier, a.ce.m, a.ce.support, a.ce.null_sd, a.ce.null_distinct, q);
        if q && chosen.is_none() { chosen = Some(r); }
    }
    let depth = match chosen {
        Some(r) => r,
        None => { eprintln!("\n  NO DEPTH QUALIFIES — the instrument is unresolvable and Phase 2 does not proceed."); return; }
    };
    eprintln!("  => R = {depth}, now FROZEN for every arm and seed (re-selecting later is the forbidden move).");

    // ---------- N1.1 per-arm null with realised spread ----------
    let arms: [(&str, Arm); 6] = [
        ("A  replicating", Arm::A), ("B  motile static", Arm::B), ("B' motile diffuse", Arm::BPrime),
        ("C  token", Arm::C), ("D  stationary", Arm::D), ("inert (CONTROL)", Arm::Inert),
    ];
    let runs: Vec<(&str, ArmRun)> = arms.iter().map(|(n, a)| (*n, run_arm(seed, *a, false, depth, STEPS, 0))).collect();

    eprintln!("\n================ N1.1 — CE ON THE METABOLIC CHANNEL, null with realised spread ================");
    eprintln!("  arm                      | k_c/M   | trans    | real     | null mean| null p95 | null max | null sd   | dist | z       | p     | support  | spread");
    for (n, r) in &runs { row(n, &r.ce); }

    eprintln!("\n================ N1.1 — CLOSURE DEFECT (lower = macro explains micro; p counts null <= real) ================");
    eprintln!("  arm                      | k_c/M   | trans    | real     | null mean| null p95 | null max | null sd   | dist | z       | p     | support  | spread");
    for (n, r) in &runs { row(n, &r.cd); }

    // ---------- N1.3 inert zero-point as a measured band ----------
    eprintln!("\n================ N1.3 — INERT ZERO-POINT (calibrated, not eyeballed) ================");
    let inert = &runs.iter().find(|(n, _)| n.starts_with("inert")).unwrap().1;
    eprintln!("  inert carrier: k_carrier {} / M {}  => resolved: {}  (an inert carrier has delta==0 forever,",
        inert.ce.k_carrier, inert.ce.m, inert.ce.resolved());
    eprintln!("  so it occupies ONE micro class and cannot be resolved — the correct answer, not a defect)");
    eprintln!("  CE real {:.4}, spread-ok {}, detects {}", inert.ce.real, inert.ce.spread_ok(), inert.ce.detects());

    // ---------- N1.3 two-process reproducibility ----------
    eprintln!("\n================ N1.3 — TWO-PROCESS REPRODUCIBILITY (index-dependent fingerprint) ================");
    for (n, r) in &runs {
        eprintln!("  {:<24} fingerprint {:#018x}  CE {:.6}  CD {:.6}", n, r.fingerprint, r.ce.real, r.cd.real);
    }
    eprintln!("  (cross-process comparison is asserted by n1_reproducibility.rs, which re-execs this binary)");

    // ---------- Does the closure defect DEGENERATE in this framing? (demonstrated, not asserted) ----------
    // H(micro|macro) = H(micro,macro) - H(macro). Here the macro group is a deterministic
    // FUNCTION of the micro class, so H(micro,macro) = H(micro) and the statistic collapses to
    // H(micro) - H(macro). H(micro) is invariant under label permutation, so the permutation test
    // on the closure defect is EXACTLY a permutation test on -H(macro).
    eprintln!("\n================ IS THE CLOSURE DEFECT DEGENERATE AS A WITHIN-CHANNEL STATISTIC? ================");
    eprintln!("  arm                      | CD (real) | H(micro) | H(macro) | H(micro)-H(macro) | identical?");
    for (n, r) in &runs {
        let a = run_arm_entropies(seed, arm_of(n), depth, STEPS);
        eprintln!("  {:<24} | {:>9.4} | {:>8.4} | {:>8.4} | {:>17.4} | {}",
            n, r.cd.real, a.0, a.1, a.0 - a.1, ((a.0 - a.1) - r.cd.real).abs() < 1e-9);
    }

    eprintln!("\n  NOTE: no property verdict is drawn here — Phase 1 validates the instrument only (rule 1).\n");
}

fn arm_of(name: &str) -> Arm {
    if name.starts_with("A ") { Arm::A }
    else if name.starts_with("B  ") { Arm::B }
    else if name.starts_with("B'") { Arm::BPrime }
    else if name.starts_with("C ") { Arm::C }
    else if name.starts_with("D ") { Arm::D }
    else { Arm::Inert }
}

/// (H(micro transition), H(macro transition)) under the TRUE partition, for the
/// degeneracy demonstration above.
fn run_arm_entropies(seed: u64, arm: Arm, depth: usize, steps: u64) -> (f64, f64) {
    let cfg = config(seed, steps);
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
    for _ in 1..=steps {
        engine.step(&mut store);
        let snap = store.snapshot();
        let mut now: HashMap<NodeId, u64> = HashMap::new();
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            let e = match d.get_property(ENERGY) { Some(PropertyValue::Float(v)) => v, _ => 0.0 };
            let delta = e - prev_e.get(&id).copied().unwrap_or(e);
            let hv = hist.entry(id).or_default();
            hv.insert(0, bucket(delta));
            hv.truncate(depth);
            if hv.len() == depth {
                let cls = micro_class(d.label.0 as i64, hv);
                now.insert(id, cls);
                is_c.entry(cls).or_insert(is_carrier);
            }
            prev_e.insert(id, e);
        }
        record_sorted(&mut tpm, &prev_cls, &now); // C3.1 sorted-order recording
        prev_cls = Some(now);
    }
    let mut compact: HashMap<usize, usize> = HashMap::new();
    let g: Vec<usize> = tpm.hashes().iter().map(|h| {
        let raw = *is_c.get(&h.0).unwrap_or(&false) as usize;
        let n = compact.len();
        *compact.entry(raw).or_insert(n)
    }).collect();
    let counts = tpm.counts_dense();
    let mut micro_c: HashMap<(usize, usize), f64> = HashMap::new();
    let mut macro_c: HashMap<(usize, usize), f64> = HashMap::new();
    for (i, rowv) in counts.iter().enumerate() {
        for (j, &c) in rowv.iter().enumerate() {
            if c == 0 { continue; }
            *micro_c.entry((i, j)).or_insert(0.0) += c as f64;
            *macro_c.entry((g[i], g[j])).or_insert(0.0) += c as f64;
        }
    }
    let tot: f64 = micro_c.values().sum();
    (h_sorted(&micro_c, tot), h_sorted(&macro_c, tot)) // C3.1 sorted summation order
}
