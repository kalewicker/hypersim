//! Multi-seed F2 replication — is the carrier/substrate boundary detectable across
//! the regime, or only at seed 12345?
//!
//! Pre-registration: `docs/validation_v2_multiseed_preregistration.md` (committed
//! 43314c2, before any multi-seed CE). F2 showed the boundary is distinguishable
//! from an arbitrary same-sized partition at seed 12345 (depth 2, 0/200, p<0.005).
//! F6.2 flagged that every CE number is single-seed on a ~32%-establishing regime.
//! This runs the identical F2 boundary test across seeds 1.. until 30 are included.
//!
//! Depth is fixed at 2 (F1 pre-registration); the 6b vacuum-augmented model is the
//! only one whose EI_macro is not structurally pinned. Nothing is tuned. The TPM is
//! built STREAMING here (only the previous class map is retained), which is memory-
//! cheap across 120 seeds; a fidelity anchor reproduces seed 12345's F2 result
//! through this streaming path to prove it matches F2's retain-all implementation.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, NodeId, WlHash};
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::collections::HashMap;

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const MIN_TRANSITIONS: u64 = 1000;
const STEPS: u64 = 6_000;
const WL_DEPTH: u8 = 2; // F1 pre-registration; re-selecting is forbidden
const N_PERM: usize = 200;
const TARGET_INCLUDED: usize = 30;
const MAX_ATTEMPTS: u64 = 120;
const ALPHA: f64 = 0.05;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "f7"
run_id = "f7"
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
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Number of distinct group-size-preserving partitions: C(m, k).
fn null_support(m: usize, k: usize) -> f64 {
    let k = k.min(m - k);
    let mut acc = 1.0f64;
    for i in 0..k {
        acc *= (m - i) as f64 / (i + 1) as f64;
    }
    acc
}

struct SeedResult {
    seed: u64,
    surviving: bool,
    transitions: u64,
    m: usize,
    k_carrier: usize,
    real_ce: f64,
    null_mean: f64,
    null_max: f64,
    null_ge_real: usize,
    support: f64,
}
impl SeedResult {
    /// Included iff the boundary test is well-defined: enough transitions and a
    /// non-degenerate carrier group (so the permutation null has real support).
    fn included(&self) -> bool {
        self.transitions >= MIN_TRANSITIONS && self.k_carrier >= 2 && self.k_carrier < self.m
    }
    fn p(&self) -> f64 {
        self.null_ge_real as f64 / N_PERM as f64
    }
    fn margin(&self) -> f64 {
        self.real_ce - self.null_mean
    }
}

/// CE for a TPM under a partition given by dense class index.
fn ce_for(tpm: &TpmAccumulator, group_of: &[usize], k: usize) -> f64 {
    let rows = tpm.normalized_rows();
    let micro = ei_from_tpm(&rows.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>());
    let macro_rows = coarse_grain(&rows, group_of, k);
    let mac = ei_from_tpm(&macro_rows);
    if micro.ei > 0.0 {
        mac.ei / micro.ei
    } else {
        0.0
    }
}

/// Run one seed: build the vacuum-augmented TPM at depth 2 STREAMING, then compute
/// the real carrier/substrate CE and its group-size-preserving permutation null.
fn run_seed(seed: u64) -> SeedResult {
    let cfg = config(seed);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut tpm = TpmAccumulator::new();
    let mut is_carrier_class: HashMap<u64, bool> = HashMap::new();
    is_carrier_class.insert(VACUUM.0, false);
    let mut prev: Option<HashMap<NodeId, WlHash>> = None;
    let mut final_carriers = 0usize;

    for _ in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let classes = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut now: HashMap<NodeId, WlHash> = HashMap::new();
        final_carriers = 0;
        for (id, data) in snap.nodes() {
            let h = classes.get(&id).copied().unwrap_or(WlHash(0));
            let is_c = data.label == CARRIER_LABEL;
            if is_c {
                final_carriers += 1;
            }
            now.insert(id, h);
            is_carrier_class.entry(h.0).or_insert(is_c);
        }
        if let Some(p) = &prev {
            for (id, &hp) in p {
                match now.get(id) {
                    Some(&hn) => tpm.record(hp, hn),
                    None => tpm.record(hp, VACUUM), // died
                }
            }
            for (id, &hn) in &now {
                if !p.contains_key(id) {
                    tpm.record(VACUUM, hn); // born
                }
            }
        }
        prev = Some(now);
    }

    // Real partition: carrier classes vs substrate classes (∅ with substrate),
    // compacted so K reflects groups actually present, indexed by DENSE id.
    let mut compact: HashMap<usize, usize> = HashMap::new();
    let group_of: Vec<usize> = tpm
        .hashes()
        .iter()
        .map(|h| {
            let raw = *is_carrier_class.get(&h.0).unwrap_or(&false) as usize;
            let next = compact.len();
            *compact.entry(raw).or_insert(next)
        })
        .collect();
    let k = compact.len().max(1);
    let m = tpm.hashes().len();
    let k_carrier = tpm
        .hashes()
        .iter()
        .filter(|h| *is_carrier_class.get(&h.0).unwrap_or(&false))
        .count();

    let real_ce = if k >= 2 { ce_for(&tpm, &group_of, k) } else { 0.0 };

    // Permutation null in canonical (hash) order so it is process-stable (F3).
    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7_u64 ^ seed;
    let mut null_ge = 0usize;
    let mut acc = 0.0f64;
    let mut null_max = f64::NEG_INFINITY;
    if k >= 2 {
        for _ in 0..N_PERM {
            let mut labels: Vec<usize> = canon.iter().map(|&i| group_of[i]).collect();
            for i in (1..labels.len()).rev() {
                let j = (splitmix(&mut rng) % (i as u64 + 1)) as usize;
                labels.swap(i, j);
            }
            let mut permuted = vec![0usize; group_of.len()];
            for (pos, &i) in canon.iter().enumerate() {
                permuted[i] = labels[pos];
            }
            let c = ce_for(&tpm, &permuted, k);
            acc += c;
            null_max = null_max.max(c);
            if c >= real_ce {
                null_ge += 1;
            }
        }
    }

    SeedResult {
        seed,
        surviving: final_carriers > 0,
        transitions: tpm.total_transitions(),
        m,
        k_carrier,
        real_ce,
        null_mean: if N_PERM > 0 { acc / N_PERM as f64 } else { 0.0 },
        null_max: if null_max.is_finite() { null_max } else { 0.0 },
        null_ge_real: null_ge,
        support: null_support(m.max(1), k_carrier.max(1)),
    }
}

/// Exact two-sided binomial sign-test p-value for `k` of `n` under p = 0.5.
fn sign_test_p(k: usize, n: usize) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let ln_c = |n: usize, k: usize| -> f64 {
        (1..=k).map(|i| ((n - k + i) as f64).ln() - (i as f64).ln()).sum()
    };
    let ln_half_n = (n as f64) * 0.5f64.ln();
    let tail = |from: usize| -> f64 { (from..=n).map(|i| (ln_c(n, i) + ln_half_n).exp()).sum::<f64>() };
    let extreme = k.max(n - k);
    (2.0 * tail(extreme)).min(1.0)
}

#[test]
fn f7_multiseed_boundary_replication() {
    // FIDELITY ANCHOR: seed 12345 through THIS streaming path must reproduce F2's
    // retain-all result (CE 0.1633, 0/200). If the streaming TPM build diverged,
    // every per-seed number below would be suspect.
    let anchor = run_seed(12345);
    eprintln!(
        "\nFIDELITY ANCHOR seed 12345: CE={:.4}  k_carrier={}  M={}  null≥real {}/{N_PERM}  (F2: 0.1633, 0/200)",
        anchor.real_ce, anchor.k_carrier, anchor.m, anchor.null_ge_real
    );
    assert!(
        (anchor.real_ce - 0.1633).abs() < 0.01 && anchor.null_ge_real == 0,
        "streaming TPM diverged from F2's retain-all build (CE {:.4}, null≥real {}) — multi-seed numbers not trustworthy",
        anchor.real_ce, anchor.null_ge_real
    );

    let mut results: Vec<SeedResult> = Vec::new();
    let mut attempts = 0u64;
    for seed in 1..=MAX_ATTEMPTS {
        attempts += 1;
        results.push(run_seed(seed));
        if results.iter().filter(|r| r.included()).count() >= TARGET_INCLUDED {
            break;
        }
    }

    let surviving = results.iter().filter(|r| r.surviving).count();
    let included: Vec<&SeedResult> = results.iter().filter(|r| r.included()).collect();
    let n = included.len();

    eprintln!("\n================ MULTI-SEED F2 REPLICATION (WL depth {WL_DEPTH}, seeds 1..{attempts}) ================");
    eprintln!("attempts {attempts}, surviving {surviving} ({:.0}%), included {n} ({:.0}%)",
        100.0 * surviving as f64 / attempts as f64, 100.0 * n as f64 / attempts as f64);
    eprintln!("(inclusion = transitions≥{MIN_TRANSITIONS} AND carrier_classes≥2, i.e. the boundary test is well-defined)\n");

    eprintln!("  seed | surv | trans  |  M  | k_c | real CE | null mean | null max | p       | support");
    eprintln!("  -----+------+--------+-----+-----+---------+-----------+----------+---------+---------");
    for r in &results {
        let p = if r.included() {
            if r.null_ge_real == 0 { "<.005".to_string() } else { format!("{:.3}", r.p()) }
        } else {
            "  —  ".to_string()
        };
        let inc = if r.included() { "" } else { "   (excluded)" };
        eprintln!(
            "  {:>4} | {:>4} | {:>6} | {:>3} | {:>3} | {:>7.4} | {:>9.4} | {:>8.4} | {:>7} | {:.2e}{}",
            r.seed, if r.surviving { "yes" } else { "no" }, r.transitions, r.m, r.k_carrier,
            r.real_ce, r.null_mean, r.null_max, p, r.support, inc
        );
    }

    assert!(n >= 10, "vacuous: only {n} included seeds from {attempts} attempts");

    // ---- Pre-committed aggregate decision rule ----
    let detect = included.iter().filter(|r| r.p() < ALPHA).count();
    let f = detect as f64 / n as f64;
    let k_pos = included.iter().filter(|r| r.margin() > 0.0).count();
    let sign_p = sign_test_p(k_pos, n);
    let reversals: Vec<&&SeedResult> = included.iter().filter(|r| r.margin() < 0.0).collect();

    let ce_med = {
        let mut v: Vec<f64> = included.iter().map(|r| r.real_ce).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    let min_support = included.iter().map(|r| r.support).fold(f64::INFINITY, f64::min);

    eprintln!("\n---------------- aggregate ----------------");
    eprintln!("  included n = {n}   median real CE = {ce_med:.4}   min null support = {min_support:.2e}");
    eprintln!("  PRIMARY   fraction with permutation p<{ALPHA}: {detect}/{n} = {:.2}", f);
    eprintln!("  SECONDARY cross-seed sign test (real CE > null mean): {k_pos}/{n}, binomial p = {sign_p:.2e}");
    if reversals.is_empty() {
        eprintln!("  REVERSALS none — every included seed's real boundary beats its own null mean");
    } else {
        eprintln!("  REVERSALS {} seed(s) where an arbitrary partition beat the real boundary:", reversals.len());
        for r in &reversals {
            eprintln!("    seed {}: real {:.4} < null mean {:.4} (Δ {:+.4})", r.seed, r.real_ce, r.null_mean, r.margin());
        }
    }

    eprintln!("\n================ VERDICT ================");
    if f >= 0.90 {
        eprintln!("  ROBUSTLY DETECTABLE ACROSS THE REGIME (f={:.2} ≥ 0.90).", f);
        eprintln!("  The F2 boundary result generalises; seed 12345 was representative. The");
        eprintln!("  carrier/substrate boundary carries partition-specific causal signal as a");
        eprintln!("  property of the establishing regime, not of one seed.");
    } else if f >= 0.50 {
        eprintln!("  DETECTABLE IN THE MAJORITY (0.50 ≤ f={:.2} < 0.90).", f);
        eprintln!("  F2 partially generalises. The excepted seeds are listed above; detectability");
        eprintln!("  is real but not universal across the regime.");
    } else {
        eprintln!("  NOT REPRESENTATIVE (f={:.2} < 0.50).", f);
        eprintln!("  Seed 12345's clean separation does not generalise; the boundary's");
        eprintln!("  DETECTABILITY is itself seed-dependent. A finding, not a failure.");
    }
    eprintln!("  (Magnitude is out of scope — F2.4: CE magnitude is registry-dominated. This");
    eprintln!("   measurement is about detectability only.)");
    eprintln!("========================================================================\n");
}
