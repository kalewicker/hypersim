//! G3 — the sterile-token comparison (V3 / G-track).
//!
//! Pre-registration: `docs/validation_v3_sterile_token_preregistration.md`
//! (committed f03c1b9, BEFORE this file that computes CE existed). The matched
//! config is FROZEN there: F8 verbatim + N=36 planted markers - the `replicate`
//! rule. Nothing about the bloom size, run length, or window is chosen here.
//!
//! The SCORING (WL depth 2, TpmAccumulator, canonical_order, size-preserving
//! permutation null) is copied VERBATIM from `f7_multiseed_boundary.rs` (rule 3:
//! no pipeline changes). The carrier arm (single marker, replication on) is run
//! through the identical scoring in THIS file and MUST reproduce F8's seed-12345
//! result (CE 0.1633, 0/200) — that anchor proves the scoring did not drift, so
//! the head-to-head is a like-for-like comparison whose only difference is the
//! variable under test: replication present vs absent.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, HypergraphStore, NodeId, PropertyValue, WlHash};
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::collections::HashMap;

const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const STEPS: u64 = 6_000;
const WL_DEPTH: u8 = 2; // F1 pre-registration
const N_PERM: usize = 200;
const ALPHA: f64 = 0.05;
const TOKEN_MARKERS: usize = 36; // FROZEN by G2 pre-registration (matches k=4, peak=36)
const SEEDS: std::ops::RangeInclusive<u64> = 1..=30; // reuse F8's set (G3.1)

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "g3"
run_id = "g3"
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

// ---- planting (from the G1 probe; N=1 == production marker) ----
fn plant_markers(store: &mut HypergraphStore, seed: u64, n: usize) {
    use rand::{Rng, SeedableRng};
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    if ids.is_empty() {
        return;
    }
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0x00CA_111E);
    let mut planted = 0usize;
    let mut used = std::collections::HashSet::new();
    let cap = n.min(ids.len());
    while planted < cap {
        let pick = ids[rng.gen_range(0..ids.len())];
        if used.insert(pick) {
            if let Some(d) = store.get_node_mut(pick) {
                d.set_property(hg_alife::keys::CARRIER_MARKER, PropertyValue::Bool(true));
                planted += 1;
            }
        }
    }
}

// ---- splitmix + null_support + ce_for: VERBATIM from f7_multiseed_boundary.rs ----
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn null_support(m: usize, k: usize) -> f64 {
    let k = k.min(m - k);
    let mut acc = 1.0f64;
    for i in 0..k {
        acc *= (m - i) as f64 / (i + 1) as f64;
    }
    acc
}

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

struct Result {
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
impl Result {
    fn included(&self) -> bool {
        self.transitions >= 1000 && self.k_carrier >= 2 && self.k_carrier < self.m
    }
    fn p(&self) -> f64 {
        self.null_ge_real as f64 / N_PERM as f64
    }
    fn margin(&self) -> f64 {
        self.real_ce - self.null_mean
    }
}

/// Build the streaming vacuum-augmented TPM (F8 lines 127-168, verbatim), then
/// score the carrier/substrate boundary with the F8 permutation null (F8 lines
/// 170-229, verbatim). The ONLY differences from F8's `run_seed` are the two
/// pre-registered setup knobs: `sterile` (drop replicate) and `n_markers`.
fn run_seed(seed: u64, sterile: bool, n_markers: usize) -> Result {
    let cfg = config(seed);
    let mut rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    if sterile {
        rules.retain(|r| r.name != "replicate");
    }
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    plant_markers(&mut store, seed, n_markers);
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
                    None => tpm.record(hp, VACUUM),
                }
            }
            for (id, &hn) in &now {
                if !p.contains_key(id) {
                    tpm.record(VACUUM, hn);
                }
            }
        }
        prev = Some(now);
    }

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

    Result {
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
fn g3_sterile_token_comparison() {
    // FIDELITY ANCHOR (rule 3): carrier arm (1 marker, replication on) through THIS
    // file's scoring must reproduce F8 at seed 12345 (CE 0.1633, 0/200). If it does
    // not, the scoring drifted and no head-to-head number is trustworthy.
    let anchor = run_seed(12345, false, 1);
    eprintln!(
        "\nFIDELITY ANCHOR seed 12345 carrier: CE={:.4} k={} M={} null≥real {}/{N_PERM}  (F8: 0.1633, 0/200)",
        anchor.real_ce, anchor.k_carrier, anchor.m, anchor.null_ge_real
    );
    assert!(
        (anchor.real_ce - 0.1633).abs() < 0.01 && anchor.null_ge_real == 0,
        "scoring drifted from F8 (CE {:.4}, null≥real {})",
        anchor.real_ce, anchor.null_ge_real
    );

    // Head-to-head on the pre-registered seed set. Carrier arm = F8 (1 marker,
    // replication). Token arm = FROZEN config (36 markers, no replicate).
    let mut rows: Vec<(Result, Result)> = Vec::new();
    for seed in SEEDS {
        let carrier = run_seed(seed, false, 1);
        let token = run_seed(seed, true, TOKEN_MARKERS);
        rows.push((carrier, token));
    }

    eprintln!("\n============================ G3: STERILE-TOKEN HEAD-TO-HEAD (WL depth {WL_DEPTH}) ============================");
    eprintln!("  carrier arm: 1 marker + replication (= F8).  token arm: {TOKEN_MARKERS} markers, NO replicate (frozen G2).\n");
    eprintln!("  seed | car M | car k | car CE | car p  | car null || tok M | tok k | tok CE | tok p  | tok null | tok support | separable?");
    eprintln!("  -----+-------+-------+--------+--------+----------++-------+-------+--------+--------+----------+-------------+-----------");
    let mut token_detect = 0usize;
    let mut token_incl = 0usize;
    let mut token_pos = 0usize;
    let mut both_incl = 0usize;
    let mut carrier_detect = 0usize;
    for (c, t) in &rows {
        let cp = if c.included() { if c.null_ge_real == 0 { "<.005".into() } else { format!("{:.3}", c.p()) } } else { "  —  ".into() };
        let tp = if t.included() { if t.null_ge_real == 0 { "<.005".into() } else { format!("{:.3}", t.p()) } } else { "  —  ".into() };
        // "separable?" per the plan: carrier detected but token NOT ⟹ replication-specific.
        let sep = if c.included() && t.included() {
            if c.p() < ALPHA && t.p() >= ALPHA { "YES (repl)" }
            else if c.p() < ALPHA && t.p() < ALPHA { "no (both)" }
            else { "—" }
        } else { "n/a" };
        if c.included() { carrier_detect += (c.p() < ALPHA) as usize; }
        if t.included() {
            token_incl += 1;
            token_detect += (t.p() < ALPHA) as usize;
            token_pos += (t.margin() > 0.0) as usize;
        }
        if c.included() && t.included() { both_incl += 1; }
        eprintln!(
            "  {:>4} | {:>5} | {:>5} | {:>6.4} | {:>6} | {:>8.4} || {:>5} | {:>5} | {:>6.4} | {:>6} | {:>8.4} | {:>11.3e} | {}",
            c.seed, c.m, c.k_carrier, c.real_ce, cp, c.null_mean,
            t.m, t.k_carrier, t.real_ce, tp, t.null_mean, t.support, sep
        );
    }

    let sign_p = sign_test_p(token_pos, token_incl);
    let tok_support_min = rows.iter().filter(|(_, t)| t.included()).map(|(_, t)| t.support).fold(f64::INFINITY, f64::min);
    let tok_support_max = rows.iter().filter(|(_, t)| t.included()).map(|(_, t)| t.support).fold(f64::NEG_INFINITY, f64::max);

    eprintln!("\n---------------------------- aggregate ----------------------------");
    eprintln!("  carrier included = 30, carrier p<{ALPHA}: {carrier_detect}/30  (F8 control, expect 30/30)");
    eprintln!("  token included   = {token_incl}/30   both-arms included = {both_incl}");
    eprintln!("  TOKEN fraction with permutation p<{ALPHA}: {token_detect}/{token_incl} = {:.2}", token_detect as f64 / token_incl.max(1) as f64);
    eprintln!("  TOKEN cross-seed sign test (real CE > null mean): {token_pos}/{token_incl}, binomial p = {sign_p:.2e}");
    eprintln!("  TOKEN null support at its resolution: [{tok_support_min:.3e}, {tok_support_max:.3e}]  (F0 fatal floor ~10)");

    let f_tok = token_detect as f64 / token_incl.max(1) as f64;
    eprintln!("\n============================ VERDICT ============================");
    if f_tok >= 0.90 && carrier_detect >= 27 {
        eprintln!("  MATTER-TURNOVER BOUNDARY. The sterile token (no replication) is detected as");
        eprintln!("  cleanly as the real carrier ({token_detect}/{token_incl} vs {carrier_detect}/30). Replication is NOT what the");
        eprintln!("  boundary detects; the object is birth-and-death-with-site-structure. The");
        eprintln!("  'replication boundary' name deflates. H_construction is not refuted.");
    } else if f_tok <= 0.10 && carrier_detect >= 27 {
        eprintln!("  REPLICATION BOUNDARY SURVIVES. The carrier separates ({carrier_detect}/30) but the sterile");
        eprintln!("  token does NOT ({token_detect}/{token_incl}). Replication-specific structure exists; removing");
        eprintln!("  replication removes the boundary's detectability. Strongest positive to date.");
    } else {
        eprintln!("  PARTIAL / SEED-DEPENDENT. token detect f={f_tok:.2}, carrier {carrier_detect}/30.");
        eprintln!("  Report the distribution; the honest claim is partial replication-specific structure.");
    }
    eprintln!("  (Still entangled, only a churn substrate can remove: both classes are the only");
    eprintln!("   ∅-touching classes. This track removes replication, not the ∅-touching construction.)");
    eprintln!("=================================================================================\n");

    // Sanity gates (not the finding — just guards on a well-formed comparison).
    assert_eq!(carrier_detect, 30, "carrier control arm should reproduce F8's 30/30");
    assert!(token_incl >= 10, "token comparison vacuous: only {token_incl} included");
    assert!(tok_support_min > 1000.0, "token null under-resolved (support {tok_support_min:.1}); F0 artifact risk");
}
