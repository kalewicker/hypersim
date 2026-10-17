//! Experiment 6c — population-level causal emergence (carrier-count dynamics).
//!
//! Pre-registration: `docs/experiment6c_population_ce_preregistration.md`. Micro state = carrier
//! count N(t); micro TPM = N(t)→N(t+Δ); macro = N binned into K equal-width bands; CE =
//! EI_macro/EI_micro. Measures the MAGNITUDE of replicator causal power at the population level
//! (6b gave the sign at the node level). Self-contained (own TpmAccumulator via the exposed
//! ei_from_tpm/coarse_grain); the core observer is untouched (exit-criterion 7).
//!
//! Committed predictions: P1 CE_bare=0 & CE_live>0; P2 report magnitude, don't target CE>1;
//! F-A if EI_micro≈0 (count causally trivial). Nothing tuned.
#![cfg(feature = "alife")]

use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::WlHash;
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);
const MIN_TRANSITIONS: u64 = 1000;

fn config(alife: bool, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "exp6c"
run_id = "exp6c"
seed = 12345
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
enabled = false
[alife]
enabled = {alife}
seed_carrier_marker = {alife}
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

/// Record the carrier-count series (sampled every `cadence` steps).
struct Counter {
    cadence: u64,
    step: u64,
    series: Arc<Mutex<Vec<usize>>>,
}
impl Observer for Counter {
    fn on_step_end(&mut self, snap: &hg_core::GraphSnapshot<'_>, _s: &StepStats, _e: &[RewriteEvent]) {
        self.step += 1;
        if self.step % self.cadence == 0 {
            let n = snap.nodes().filter(|(_, d)| d.label == CARRIER).count();
            self.series.lock().unwrap().push(n);
        }
    }
}

fn count_series(alife: bool, steps: u64, cadence: u64) -> Vec<usize> {
    let cfg = config(alife, steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100000);
    let series = Arc::new(Mutex::new(Vec::new()));
    engine.add_observer(Box::new(Counter { cadence, step: 0, series: series.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    Arc::try_unwrap(series).unwrap().into_inner().unwrap()
}

struct Ce {
    ce: f64,
    ei_micro: f64,
    ei_macro: f64,
    micro_states: usize,
    macro_k: usize,
    transitions: u64,
}

/// Build the N→N' TPM from a count series and coarse-grain N into `k` equal-width bands.
fn population_ce(series: &[usize], k: usize) -> Ce {
    let mut tpm = TpmAccumulator::new();
    for w in series.windows(2) {
        tpm.record(WlHash(w[0] as u64), WlHash(w[1] as u64));
    }
    let rows = tpm.normalized_rows();
    let micro: Vec<Vec<f64>> = rows.iter().map(|(_, r)| r.clone()).collect();
    let mi = ei_from_tpm(&micro);

    // Equal-width bands over the observed range, with COMPACT group indices.
    let hashes = tpm.hashes();
    let counts: Vec<u64> = hashes.iter().map(|h| h.0).collect();
    let (lo, hi) = (
        *counts.iter().min().unwrap_or(&0),
        *counts.iter().max().unwrap_or(&0),
    );
    let span = (hi - lo).max(1) as f64;
    let mut compact = std::collections::HashMap::new();
    let group_of: Vec<usize> = counts
        .iter()
        .map(|&c| {
            let raw = (((c - lo) as f64 / span) * (k as f64 - 1e-9)).floor() as usize;
            let next = compact.len();
            *compact.entry(raw).or_insert(next)
        })
        .collect();
    let kk = compact.len().max(1);
    let ma = ei_from_tpm(&coarse_grain(&rows, &group_of, kk));
    Ce {
        ce: if mi.ei > 0.0 { ma.ei / mi.ei } else { 0.0 },
        ei_micro: mi.ei,
        ei_macro: ma.ei,
        micro_states: mi.m,
        macro_k: kk,
        transitions: tpm.total_transitions(),
    }
}

#[test]
fn experiment6c_population_causal_emergence() {
    let steps = 20_000u64;
    let cadences = [1u64, 4, 16];
    let ks = [2usize, 4];

    eprintln!("\n============ EXPERIMENT 6c — population-level CE (carrier-count dynamics) ============");
    eprintln!("seed 12345, turnover regime, {steps} steps, gate {MIN_TRANSITIONS} transitions\n");
    eprintln!("  Δ   K | arm  | CE       EI_micro  EI_macro | micro_states macro_K | transitions");
    eprintln!("  ------+------+-----------------------------+----------------------+------------");

    let mut any_live = None;
    for &cad in &cadences {
        let live_series = count_series(true, steps, cad);
        let bare_series = count_series(false, steps, cad);
        for &k in &ks {
            let live = population_ce(&live_series, k);
            let bare = population_ce(&bare_series, k);
            eprintln!(
                "  {cad:>2} {k:>2} | live | {:.4}   {:.4}    {:.4}  | {:>6}       {:>3}    | {}",
                live.ce, live.ei_micro, live.ei_macro, live.micro_states, live.macro_k, live.transitions
            );
            eprintln!(
                "        | bare | {:.4}   {:.4}    {:.4}  | {:>6}       {:>3}    | {}",
                bare.ce, bare.ei_micro, bare.ei_macro, bare.micro_states, bare.macro_k, bare.transitions
            );
            // F-C: a bare substrate has N ≡ 0 ⟹ one micro state ⟹ CE = 0.
            assert_eq!(bare.micro_states, 1, "F-C: bare arm has a single count state (N≡0)");
            assert_eq!(bare.ce, 0.0, "F-C: bare CE is the degenerate 0");
            if cad == 1 && k == 4 {
                any_live = Some(live.ce);
            }
        }
    }
    eprintln!("====================================================================================\n");

    // P1: the live population-count dynamics is non-degenerate (many states, gate cleared).
    let live1 = population_ce(&count_series(true, steps, 1), 4);
    assert!(live1.transitions >= MIN_TRANSITIONS, "F-B: gate cleared");
    assert!(live1.micro_states > 1, "P1: live count dynamics is non-degenerate");

    let ce = any_live.unwrap();
    eprintln!("RESULT: population CE (Δ=1,K=4) = {ce:.4}; EI_micro = {:.4} over {} count states.", live1.ei_micro, live1.micro_states);
    if live1.ei_micro < 1e-6 {
        eprintln!("  ⟹ F-A: steady-state count is causally trivial (EI_micro≈0) — replicator dynamics lives");
        eprintln!("    at the individual/lineage level, not the population count.");
    } else if ce > 1.0 {
        eprintln!("  ⟹ population-level CAUSAL EMERGENCE (CE>1): the carrying-capacity attractor is a macro-");
        eprintln!("    causal structure clearing the conservative bar. NOT tuned (Finding F1).");
    } else {
        eprintln!("  ⟹ 0<CE<1: population-count dynamics is non-degenerate but sub-emergent — a magnitude for");
        eprintln!("    replicator causal power, below the conservative bar (consistent with 6b's sign).");
    }
}

/// F-D: population CE is reproducible at the pinned seed.
#[test]
fn experiment6c_is_deterministic() {
    let a = count_series(true, 6000, 1);
    let b = count_series(true, 6000, 1);
    assert_eq!(a, b, "carrier-count series is bit-reproducible");
    let (ca, cb) = (population_ce(&a, 4), population_ce(&b, 4));
    assert_eq!(ca.ce.to_bits(), cb.ce.to_bits(), "population CE bit-reproducible");
}
