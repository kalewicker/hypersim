//! G3 supplement — confound diagnostics for the sterile-token comparison.
//!
//! The frozen comparison (`g3_sterile_token.rs`) returned token 0/30 vs carrier
//! 30/30. Before that is read as "replication boundary survives", rule 4 (report
//! confounds) requires quantifying the two things that could make it an artifact:
//!
//!  (D1) POWER: the token is dead for ~97% of the run, so its classes touch few
//!       transitions. If the token's non-detection is just "too little signal",
//!       that is a duration/power confound, not evidence about replication. We
//!       measure the token-touching transition fraction and the turnover duration,
//!       head-to-head with the carrier.
//!  (D2) DETERMINISM: at the pre-registered full-grid bloom (N=36) the token run is
//!       seed-independent, so the 30 token rows are one measurement repeated. We
//!       confirm this and show the token ALSO fails to separate at smaller blooms
//!       (N=4..25), where marker placement IS seed-dependent — ruling out an N=36
//!       determinism artifact.
//!
//! Scoring is the same verbatim F8 code; nothing here changes the pipeline or the
//! pre-registered primary. These are supporting measurements only.
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
const WL_DEPTH: u8 = 2;
const N_PERM: usize = 200;
const ALPHA: f64 = 0.05;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "g3r"
run_id = "g3r"
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
    if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 }
}

struct Diag {
    m: usize,
    k_carrier: usize,
    transitions: u64,
    carrier_touching: u64, // transitions with >=1 endpoint a carrier/token class
    last_alive_step: u64,  // last step with >0 carriers (turnover duration)
    real_ce: f64,
    null_mean: f64,
    null_ge_real: usize,
    support: f64,
}

fn run_diag(seed: u64, sterile: bool, n_markers: usize) -> Diag {
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
    let mut last_alive_step = 0u64;
    // to count carrier-touching transitions we need each endpoint's carrier-ness;
    // record raw (hp,hn) pairs and classify after is_carrier_class is complete.
    let mut pairs: Vec<(u64, u64)> = Vec::new();

    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let classes = compute_wl_class_map_snapshot(&snap, WL_DEPTH);
        let mut now: HashMap<NodeId, WlHash> = HashMap::new();
        let mut live = 0usize;
        for (id, data) in snap.nodes() {
            let h = classes.get(&id).copied().unwrap_or(WlHash(0));
            let is_c = data.label == CARRIER_LABEL;
            if is_c {
                live += 1;
            }
            now.insert(id, h);
            is_carrier_class.entry(h.0).or_insert(is_c);
        }
        if live > 0 {
            last_alive_step = step;
        }
        if let Some(p) = &prev {
            for (id, &hp) in p {
                match now.get(id) {
                    Some(&hn) => {
                        tpm.record(hp, hn);
                        pairs.push((hp.0, hn.0));
                    }
                    None => {
                        tpm.record(hp, VACUUM);
                        pairs.push((hp.0, VACUUM.0));
                    }
                }
            }
            for (id, &hn) in &now {
                if !p.contains_key(id) {
                    tpm.record(VACUUM, hn);
                    pairs.push((VACUUM.0, hn.0));
                }
            }
        }
        prev = Some(now);
    }

    let carrier_touching = pairs
        .iter()
        .filter(|(a, b)| {
            *is_carrier_class.get(a).unwrap_or(&false) || *is_carrier_class.get(b).unwrap_or(&false)
        })
        .count() as u64;

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
    let k_carrier = tpm.hashes().iter().filter(|h| *is_carrier_class.get(&h.0).unwrap_or(&false)).count();
    let real_ce = if k >= 2 { ce_for(&tpm, &group_of, k) } else { 0.0 };

    let canon = tpm.canonical_order();
    let mut rng = 0x5EED_F7_u64 ^ seed;
    let mut null_ge = 0usize;
    let mut acc = 0.0f64;
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
            if c >= real_ce {
                null_ge += 1;
            }
        }
    }

    Diag {
        m,
        k_carrier,
        transitions: tpm.total_transitions(),
        carrier_touching,
        last_alive_step,
        real_ce,
        null_mean: acc / N_PERM as f64,
        null_ge_real: null_ge,
        support: null_support(m.max(1), k_carrier.max(1)),
    }
}

#[test]
fn g3_supplement_confounds() {
    // ---- D1: power / duration, head-to-head at seed 12345 ----
    let car = run_diag(12345, false, 1);
    let tok = run_diag(12345, true, 36);
    eprintln!("\n================ D1: POWER / DURATION CONFOUND (seed 12345) ================");
    eprintln!("  arm      | last-alive step | carrier-touching / total transitions | fraction");
    eprintln!("  ---------+-----------------+--------------------------------------+---------");
    eprintln!(
        "  carrier  | {:>15} | {:>16} / {:>8} | {:.3}",
        car.last_alive_step, car.carrier_touching, car.transitions,
        car.carrier_touching as f64 / car.transitions as f64
    );
    eprintln!(
        "  token    | {:>15} | {:>16} / {:>8} | {:.3}",
        tok.last_alive_step, tok.carrier_touching, tok.transitions,
        tok.carrier_touching as f64 / tok.transitions as f64
    );
    eprintln!("  Reading: the carrier sustains turnover for the whole run and its classes touch a");
    eprintln!("  large share of transitions; the sterile token self-extinguishes early and touches");
    eprintln!("  few. Non-detection of the token is therefore CONFOUNDED with turnover duration —");
    eprintln!("  but on a static substrate replication is the ONLY thing that sustains a population,");
    eprintln!("  so duration and replication cannot be separated here (a churn substrate could).");

    // ---- D2: determinism + robustness across bloom sizes ----
    eprintln!("\n================ D2: TOKEN NON-DETECTION vs BLOOM SIZE (robustness) ================");
    eprintln!("  N=36 is the pre-registered primary (deterministic). N=4..25 have seed-dependent");
    eprintln!("  marker placement, so seed variation there is REAL — a check the null result isn't");
    eprintln!("  an N=36 determinism artifact.\n");
    eprintln!("  N   | seed | M   | k | real CE | null mean | p     | support | detected?");
    eprintln!("  ----+------+-----+---+---------+-----------+-------+---------+----------");
    for &n in &[4usize, 9, 16, 25, 36] {
        for seed in [1u64, 2, 3] {
            let d = run_diag(seed, true, n);
            let p = d.null_ge_real as f64 / N_PERM as f64;
            let det = if d.k_carrier >= 2 && p < ALPHA { "YES" } else { "no" };
            eprintln!(
                "  {:>2} | {:>4} | {:>3} | {} | {:>7.4} | {:>9.4} | {:>5.3} | {:>7.1e} | {}",
                n, seed, d.m, d.k_carrier, d.real_ce, d.null_mean, p, d.support, det
            );
        }
    }
    eprintln!("=====================================================================================\n");
}
