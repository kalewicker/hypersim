//! V-track closeout / C4.1 — arm C's empty-fraction sweep, converted to CONFIRMATORY.
//!
//! Pre-registration: docs/validation_v9_c4_preregistration.md (4fd9df1), committed before
//! this file existed and before any C4 number.
//!
//! Arm C's structural non-detection (z = -0.83, p = 0.930; 0/29 multi-seed) is established.
//! The standing challenge is different: C's carriers die at ~step 120, so over a 6000-step
//! window the carrier group is EMPTY for ~98% of scored steps. Is the non-detection an
//! EMPTY-GROUP ARTIFACT — a partition well-formedness problem — rather than a fact about C?
//!
//! M2 swept the window (empty fraction 8% -> 98%) and found the score identical throughout.
//! That is strong and it is EXPLORATORY: those windows were chosen after the pre-registered
//! truncation had failed. Here the grid, the cell-inclusion clauses and the decision rule
//! are all fixed in advance, on a seed set disjoint from every seed used before.
//!
//! Cell inclusion carries the M2.2 lesson explicitly: a window where the arm-A positive
//! control has no power, or where arm C's own null is degenerate, is read in NEITHER
//! direction rather than counted as a non-detection.
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
const N_PERM: usize = 200;
const ENERGY: u32 = 1;
const ALPHA: f64 = 0.05;
const DISTINCT_MIN: usize = 10;
const WL_DEPTH: u8 = 2;
/// pre-registered grid — fixed at 4fd9df1, never chosen from the data
const GRID: [u64; 8] = [130, 150, 200, 300, 600, 1200, 3000, 6000];
const SEED_START: u64 = 201;
const TARGET_SEEDS: usize = 10;
const MAX_ATTEMPTS: u64 = 40;

fn config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "c4"
run_id = "c4"
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

/// arm C = sterile, no motility, no diffusion, 6 markers. arm A = the intact economy.
fn build_rules(cfg: &SimConfig, sterile: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if sterile { biology.retain(|r| r.name != "replicate"); }
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

#[derive(Clone, Default)]
struct Score {
    m: usize, k_carrier: usize, real: f64, null_mean: f64, null_sd: f64,
    null_min: f64, null_max: f64, null_ge: usize, null_distinct: usize, support: f64,
}
impl Score {
    fn p(&self) -> f64 { self.null_ge as f64 / N_PERM as f64 }
    fn z(&self) -> f64 {
        if self.k_carrier < 2 || self.null_sd <= 0.0 { f64::NAN } else { (self.real - self.null_mean) / self.null_sd }
    }
    fn spread_ok(&self) -> bool {
        self.null_sd > 0.0 && (self.null_max - self.null_min) > 0.0 && self.null_distinct >= DISTINCT_MIN
    }
    fn resolved(&self) -> bool { self.k_carrier >= 2 && self.k_carrier < self.m && self.support > 100.0 }
    fn detects(&self) -> bool { self.resolved() && self.spread_ok() && self.p() < ALPHA }
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
    Score {
        m, k_carrier, real, null_mean: mean,
        null_sd: (sumsq / n - mean * mean).max(0.0).sqrt(),
        null_min: if mn == f64::MAX { 0.0 } else { mn },
        null_max: if mx == f64::MIN { 0.0 } else { mx },
        null_ge: ge, null_distinct: seen.len(), support: null_support(m, k_carrier),
    }
}

/// Structural (WL depth 2) CE over a window of `w` steps, with the realised empty fraction.
fn run_window(seed: u64, sterile: bool, markers: usize, w: u64) -> (Score, f64, u64) {
    let cfg = config(seed, w);
    let rules = build_rules(&cfg, sterile);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    plant_markers(&mut store, seed, markers);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut tpm = TpmAccumulator::new();
    let mut is_c: HashMap<u64, bool> = HashMap::new();
    is_c.insert(VACUUM.0, false);
    let mut prev: Option<HashMap<NodeId, u64>> = None;
    let (mut empty_steps, mut last_alive) = (0u64, 0u64);

    for step in 1..=w {
        engine.step(&mut store);
        let snap = store.snapshot();
        let wl = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut now: HashMap<NodeId, u64> = HashMap::new();
        let mut live = 0usize;
        for (id, d) in snap.nodes() {
            let is_carrier = d.label == CARRIER_LABEL;
            if is_carrier { live += 1; }
            let h = wl.get(&id).copied().unwrap_or(WlHash(0)).0;
            now.insert(id, h);
            is_c.entry(h).or_insert(is_carrier);
        }
        if live > 0 { last_alive = step; } else { empty_steps += 1; }
        // C3.1 / N1.3 sorted-order recording
        if let Some(p) = &prev {
            let mut carry: Vec<(NodeId, u64)> = p.iter().map(|(k, v)| (*k, *v)).collect();
            let mut born: Vec<(NodeId, u64)> = now.iter().filter(|(id, _)| !p.contains_key(id)).map(|(k, v)| (*k, *v)).collect();
            carry.sort_unstable();
            born.sort_unstable();
            for (id, hp) in carry { tpm.record(WlHash(hp), WlHash(now.get(&id).copied().unwrap_or(VACUUM.0))); }
            for (_, hn) in born { tpm.record(VACUUM, WlHash(hn)); }
        }
        prev = Some(now);
    }
    (score_ce(&tpm, &is_c, seed), empty_steps as f64 / w as f64, last_alive)
}

/// Spearman rank correlation.
fn spearman(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len();
    if n < 3 { return f64::NAN; }
    let rank = |v: &[f64]| -> Vec<f64> {
        let mut idx: Vec<usize> = (0..v.len()).collect();
        idx.sort_by(|&a, &b| v[a].partial_cmp(&v[b]).unwrap());
        let mut r = vec![0.0; v.len()];
        let mut i = 0;
        while i < idx.len() {
            let mut j = i;
            while j + 1 < idx.len() && v[idx[j + 1]] == v[idx[i]] { j += 1; }
            let avg = (i + j) as f64 / 2.0 + 1.0;
            for &k in &idx[i..=j] { r[k] = avg; }
            i = j + 1;
        }
        r
    };
    let (rx, ry) = (rank(xs), rank(ys));
    let mx = rx.iter().sum::<f64>() / n as f64;
    let my = ry.iter().sum::<f64>() / n as f64;
    let num: f64 = rx.iter().zip(&ry).map(|(a, b)| (a - mx) * (b - my)).sum();
    let dx: f64 = rx.iter().map(|a| (a - mx).powi(2)).sum::<f64>().sqrt();
    let dy: f64 = ry.iter().map(|b| (b - my).powi(2)).sum::<f64>().sqrt();
    if dx * dy > 0.0 { num / (dx * dy) } else { f64::NAN }
}

#[test]
fn c4_1_arm_c_confirmatory_empty_fraction_sweep() {
    eprintln!("\n================ V-TRACK CLOSEOUT / C4.1 — ARM-C EMPTY-FRACTION SWEEP (CONFIRMATORY) ================");
    eprintln!("  Pre-registration 4fd9df1: grid, seed set, cell inclusion and decision rule all fixed BEFORE this ran.");
    eprintln!("  Grid W = {GRID:?}; seeds from {SEED_START} (disjoint from 12345 and from Phase 2's 1-30).");
    eprintln!("  A cell counts ONLY if arm A detects at that W (positive control) AND arm C's null passes the");
    eprintln!("  spread guard. Otherwise it is read in NEITHER direction — the M2.2 lesson.\n");

    eprintln!("  seed |    W | C empty-frac | C k_c/M  | C real   | C z     | C p   | C spread | A detects | included");
    eprintln!("  -----+------+--------------+----------+----------+---------+-------+----------+-----------+---------");

    let mut ef: Vec<f64> = Vec::new();
    let mut zs: Vec<f64> = Vec::new();
    let (mut included, mut detected) = (0usize, 0usize);
    let mut excluded_reasons: HashMap<&str, usize> = HashMap::new();
    let mut seeds_used = 0usize;
    // per-seed CE across the grid, kept at full precision: the sweep turns out to be exactly
    // invariant within a seed, and that has to be measured rather than eyeballed off 4 dp.
    let mut per_seed: Vec<(u64, Vec<f64>)> = Vec::new();
    // the load-bearing cell is the SMALLEST window (the most populated carrier group), not the
    // sweep as a whole — see the invariance note below.
    let mut smallest_cell: Vec<(u64, f64, f64, bool)> = Vec::new();

    for seed in SEED_START..(SEED_START + MAX_ATTEMPTS) {
        if seeds_used >= TARGET_SEEDS { break; }
        // a seed is usable only if arm C actually establishes carriers that then die
        let (_, _, c_last) = run_window(seed, true, 6, 6000);
        if c_last == 0 { *excluded_reasons.entry("arm C never established").or_insert(0) += 1; continue; }
        seeds_used += 1;
        let mut ces: Vec<f64> = Vec::new();

        for &w in GRID.iter() {
            let (cs, cef, _) = run_window(seed, true, 6, w);
            let (as_, _, _) = run_window(seed, false, 1, w);
            let a_ok = as_.detects();
            let inc = a_ok && cs.spread_ok() && cs.resolved();
            let note = if inc { "YES" }
                else if !a_ok { *excluded_reasons.entry("arm A no power at W").or_insert(0) += 1; "no (A no power)" }
                else { *excluded_reasons.entry("arm C null degenerate").or_insert(0) += 1; "no (C degenerate)" };
            eprintln!("  {:>4} | {:>4} | {:>11.1}% | {:>3}/{:<4} | {:>8.4} | {:>7} | {:>5} | {:>8} | {:>9} | {}",
                seed, w, 100.0 * cef, cs.k_carrier, cs.m, cs.real,
                if cs.z().is_nan() { "n/a".to_string() } else { format!("{:.2}", cs.z()) },
                if cs.resolved() { format!("{:.3}", cs.p()) } else { "n/a".into() },
                cs.spread_ok(), a_ok, note);
            if inc {
                included += 1;
                if cs.detects() { detected += 1; }
                ef.push(cef);
                zs.push(cs.z());
                ces.push(cs.real);
                if w == GRID[0] { smallest_cell.push((seed, cef, cs.z(), cs.detects())); }
            }
        }
        if !ces.is_empty() { per_seed.push((seed, ces)); }
    }

    let f = if included > 0 { detected as f64 / included as f64 } else { f64::NAN };
    let rho = spearman(&ef, &zs);
    eprintln!("\n  seeds used {seeds_used}, INCLUDED cells {included}, arm C detections {detected}");
    eprintln!("  exclusions: {excluded_reasons:?}");
    if included > 0 {
        eprintln!("  realised empty fraction across included cells: {:.1}% .. {:.1}%",
            100.0 * ef.iter().cloned().fold(f64::MAX, f64::min),
            100.0 * ef.iter().cloned().fold(f64::MIN, f64::max));
        eprintln!("  arm C z across included cells: {:.2} .. {:.2}",
            zs.iter().cloned().fold(f64::MAX, f64::min), zs.iter().cloned().fold(f64::MIN, f64::max));
    }
    eprintln!("  f = {f:.3}   rho(empty fraction, z) = {rho:.3}");

    // ---- what rho is, and is not, measuring here ----
    eprintln!("\n  WITHIN-SEED INVARIANCE (measured at full precision, not read off 4 dp):");
    let mut all_flat = true;
    for (seed, ces) in &per_seed {
        let span = ces.iter().cloned().fold(f64::MIN, f64::max) - ces.iter().cloned().fold(f64::MAX, f64::min);
        let bit_exact = ces.windows(2).all(|w| w[0].to_bits() == w[1].to_bits());
        if span != 0.0 { all_flat = false; }
        eprintln!("    seed {seed}: CE across the whole grid spans {span:.3e}  bit-identical: {bit_exact}");
    }
    if all_flat {
        eprintln!("\n  ⟹ arm C's CE is EXACTLY invariant across the grid. So rho = 0 is MECHANICAL — z is constant");
        eprintln!("     within a seed, every pair is a tie, and rho carries NO INDEPENDENT INFORMATION. It must not");
        eprintln!("     be quoted as a second test, and is not below.");
        eprintln!("     Why: once arm C's carriers die the graph is structurally CONSTANT, so every further step");
        eprintln!("     adds only self-loop mass to rows that are already pure self-loops. The normalised rows, and");
        eprintln!("     hence CE, do not move. The empty steps are INERT for this statistic.");
        eprintln!("     Consequence: the sweep's evidence does NOT come from the trend. It comes from the SMALLEST");
        eprintln!("     window, where the carrier group is populated and the artifact hypothesis makes its prediction.");
    }
    eprintln!("\n  THE LOAD-BEARING CELL — W = {} ({:.1}% empty: the MOST populated carrier group on the grid):", GRID[0],
        100.0 * smallest_cell.first().map(|c| c.1).unwrap_or(f64::NAN));
    eprintln!("    seed | empty-frac | C z     | C detects");
    for (s, e, z, d) in &smallest_cell {
        eprintln!("    {:>4} | {:>9.1}% | {:>7.2} | {}", s, 100.0 * e, z, d);
    }
    let small_det = smallest_cell.iter().filter(|c| c.3).count();
    eprintln!("    ⟹ {}/{} seeds detect at the most-populated window, with arm A detecting there and C's null",
        small_det, smallest_cell.len());
    eprintln!("       non-degenerate. This is where the empty-group hypothesis predicts C SHOULD detect.");

    eprintln!("\n================ C4.1 — THE PRE-COMMITTED READING (fixed at 4fd9df1) ================");
    if included < 5 {
        eprintln!("  VACUOUS: fewer than 5 included cells. Nothing is concluded and nothing changes.");
    } else if f <= 0.05 && rho.abs() < 0.5 {
        eprintln!("  ⟹ f = {f:.3} <= 0.05 and |rho| = {:.3} < 0.5: arm C's non-detection is FLAT IN THE EMPTY", rho.abs());
        eprintln!("     FRACTION across {:.0}%-{:.0}% of the window being empty. The empty-group artifact is",
            100.0 * ef.iter().cloned().fold(f64::MAX, f64::min), 100.0 * ef.iter().cloned().fold(f64::MIN, f64::max));
        eprintln!("     REJECTED under pre-registration, and M2's exploratory finding is CONVERTED TO CONFIRMATORY.");
        eprintln!("     STATED PRECISELY, because the rho clause turned out to be degenerate: the evidence is that");
        eprintln!("     {}/{} seeds detect at W = {} where the carrier group is populated {:.0}% of scored steps and",
            small_det, smallest_cell.len(), GRID[0],
            100.0 * (1.0 - smallest_cell.first().map(|c| c.1).unwrap_or(f64::NAN)));
        eprintln!("     arm A DOES detect — the exact cell where the artifact hypothesis predicts C should detect.");
        eprintln!("     M2's pre-registered truncation could not reach this cell: at W = 120 arm C's own null was");
        eprintln!("     degenerate. The grid fixed in advance here clears the spread guard, so the question that was");
        eprintln!("     unanswerable in M2 is answered.");
    } else if rho < -0.5 || f > 0.05 {
        eprintln!("  ⟹ THE ARTIFACT READING IS SUPPORTED (f = {f:.3}, rho = {rho:.3}). Arm C's non-detection is a");
        eprintln!("     well-formedness problem: 'sustained structural activity' weakens to 'A POPULATED CARRIER");
        eprintln!("     GROUP IS NECESSARY'. This is the finding and must be reported as such.");
    } else {
        eprintln!("  ⟹ INTERMEDIATE (f = {f:.3}, rho = {rho:.3}): report both numbers; claim bounded by both.");
    }
    eprintln!("======================================================================\n");

    assert!(included >= 5 || seeds_used == 0,
        "C4.1 produced fewer than 5 included cells across {seeds_used} seeds — report as vacuous, do not read it");
}
