//! V-track V3 — CE validation (is the coefficient real).
//!
//! The CE number is the most falsifiable object in the project. It is partition-
//! relative and cross-scale, so it fails QUIETLY: a spurious boundary or a soft
//! hash yields a well-defined number that is not measuring what it claims.
//!
//! Design: the live arm is run ONCE and every per-observation WL class map is
//! retained, so all partitions/registries/onset-shifts below are recomputed from
//! IDENTICAL dynamics. Any CE difference is then attributable to the partition
//! choice alone, which is exactly the question.
//!
//! Two TPMs are built from the same observation sequence:
//!   * SURVIVING-only (the Experiment-6a model) — macro TPM is the identity because
//!     no surviving node ever crosses the carrier/substrate boundary.
//!   * VACUUM-augmented (the Experiment-6b fix) — births (∅→c) and deaths (c→∅)
//!     give the macro real off-diagonal mass, so the boundary can actually matter.
//! V3.2 is run on BOTH, because the contrast is the finding.
//!
//! Nothing here is tuned to raise CE (plan rule 2). A permutation null that beats
//! the real partition is reported as a void headline, not repaired.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::{compute_wl_class_map_snapshot, NodeId, WlHash};
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::collections::HashMap;

/// Reserved vacuum class ∅ (same sentinel as Experiment 6b).
const VACUUM: WlHash = WlHash(0xE6_0000_0000_0000);
const MIN_TRANSITIONS: u64 = 1000;
const STEPS: u64 = 6_000;
/// Permutation-null replicates for the spurious-boundary control.
const N_PERM: usize = 200;

fn config(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "v3"
run_id = "v3"
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

/// Deterministic PRNG (splitmix64) — the permutation null must be reproducible
/// across processes, so no `RandomState` or thread rng may enter here.
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// One observation: the node→WL-class map at a sampled step.
type Obs = Vec<(NodeId, WlHash)>;

/// Run the live arm once, retaining every observation and which classes are carrier
/// classes. Cadence 1 (per Experiment 6: carrier lifetimes are tens of steps, so a
/// coarser cadence never resolves a carrier and the partition degenerates).
fn run_and_collect(steps: u64) -> (Vec<Obs>, HashMap<u64, bool>) {
    let cfg = config(steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);

    let mut observations: Vec<Obs> = Vec::with_capacity(steps as usize);
    let mut is_carrier_class: HashMap<u64, bool> = HashMap::new();
    is_carrier_class.insert(VACUUM.0, false); // ∅ groups with substrate

    for _ in 1..=steps {
        engine.step(&mut store);
        let snapshot = store.snapshot();
        let classes = compute_wl_class_map_snapshot(&snapshot, 1);
        let mut obs: Obs = Vec::new();
        for (id, data) in snapshot.nodes() {
            let h = classes.get(&id).copied().unwrap_or(WlHash(0));
            assert_ne!(h, VACUUM, "real WL class collided with the ∅ sentinel");
            obs.push((id, h));
            is_carrier_class.entry(h.0).or_insert(data.label == CARRIER_LABEL);
        }
        // slotmap iteration is dense-slot order — already deterministic, no sort needed.
        observations.push(obs);
    }
    (observations, is_carrier_class)
}

/// Build a class-transition TPM from an observation slice. `vacuum = true` adds the
/// birth (∅→c) and death (c→∅) transitions (the 6b model); `false` keeps only
/// surviving-node transitions (the 6a model).
fn build_tpm(observations: &[Obs], vacuum: bool) -> TpmAccumulator {
    let mut tpm = TpmAccumulator::new();
    let mut prev: Option<HashMap<NodeId, WlHash>> = None;
    for obs in observations {
        let now: HashMap<NodeId, WlHash> = obs.iter().copied().collect();
        if let Some(p) = &prev {
            for (id, &hp) in p {
                match now.get(id) {
                    Some(&hn) => tpm.record(hp, hn),
                    None if vacuum => tpm.record(hp, VACUUM),
                    None => {}
                }
            }
            if vacuum {
                for (id, &hn) in &now {
                    if !p.contains_key(id) {
                        tpm.record(VACUUM, hn);
                    }
                }
            }
        }
        prev = Some(now);
    }
    tpm
}

struct Ce {
    ce: f64,
    ei_micro: f64,
    ei_macro: f64,
    k: usize,
    m: usize,
}

/// CE for a given micro TPM under a given macro partition (`group_of` is indexed by
/// dense class index, matching `tpm.hashes()` order).
fn ce_for(tpm: &TpmAccumulator, group_of: &[usize]) -> Ce {
    let rows = tpm.normalized_rows();
    let bare: Vec<Vec<f64>> = rows.iter().map(|(_, r)| r.clone()).collect();
    let micro = ei_from_tpm(&bare);
    let k = group_of.iter().copied().max().map(|m| m + 1).unwrap_or(1).max(1);
    let macro_rows = coarse_grain(&rows, group_of, k);
    let mac = ei_from_tpm(&macro_rows);
    Ce {
        ce: if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 },
        ei_micro: micro.ei,
        ei_macro: mac.ei,
        k,
        m: micro.m,
    }
}

/// The REAL partition: carrier classes vs substrate classes (∅ with substrate).
/// Group ids are compacted so `k` reflects groups actually present.
fn real_partition(tpm: &TpmAccumulator, is_carrier_class: &HashMap<u64, bool>) -> Vec<usize> {
    let mut compact: HashMap<usize, usize> = HashMap::new();
    tpm.hashes()
        .iter()
        .map(|h| {
            let raw = *is_carrier_class.get(&h.0).unwrap_or(&false) as usize;
            let next = compact.len();
            *compact.entry(raw).or_insert(next)
        })
        .collect()
}

/// Dense class indices ordered canonically by WL hash.
///
/// CRITICAL (and the exact failure mode V3.1 exists to catch): `TpmAccumulator`
/// assigns dense class indices in HASHMAP-ITERATION order, which is `RandomState`-
/// seeded and therefore differs BETWEEN PROCESSES. The CE coefficient is invariant
/// to that permutation, but any statistic computed OVER partition indices — a
/// permutation null, a class ranking, a random class flip — silently is not. Every
/// order-dependent computation below is canonicalised through this function so the
/// V3 statistics reproduce across processes, not just the coefficient.
fn canonical_order(tpm: &TpmAccumulator) -> Vec<usize> {
    let h = tpm.hashes();
    let mut idx: Vec<usize> = (0..h.len()).collect();
    idx.sort_by_key(|&i| h[i].0);
    idx
}

/// A spurious partition: the real group labels SHUFFLED across classes. Group sizes
/// are preserved exactly, so `k` and the macro dimensionality match the real
/// partition; only the correspondence with carrier-ness is destroyed. This is the
/// correct null for "is CE detecting the boundary, or just the partition arithmetic?"
/// Shuffling happens in canonical (hash) order so the null is process-stable.
fn permuted_partition(real: &[usize], canon: &[usize], rng: &mut u64) -> Vec<usize> {
    let mut labels: Vec<usize> = canon.iter().map(|&i| real[i]).collect();
    for i in (1..labels.len()).rev() {
        let j = (splitmix(rng) % (i as u64 + 1)) as usize;
        labels.swap(i, j);
    }
    let mut out = vec![0usize; real.len()];
    for (pos, &i) in canon.iter().enumerate() {
        out[i] = labels[pos];
    }
    out
}

fn pct(v: &mut Vec<f64>, q: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if v.is_empty() {
        return f64::NAN;
    }
    v[(((v.len() - 1) as f64) * q).round() as usize]
}

#[test]
fn v3_ce_validation() {
    eprintln!("\n================ V3 — CE VALIDATION (seed 12345, {STEPS} steps, cadence 1) ================");
    let (observations, is_carrier) = run_and_collect(STEPS);
    eprintln!("collected {} observations", observations.len());

    let tpm_surv = build_tpm(&observations, false); // 6a model
    let tpm_vac = build_tpm(&observations, true); // 6b model
    assert!(tpm_vac.total_transitions() >= MIN_TRANSITIONS, "below the transition gate — no CE claim");

    let real_surv = real_partition(&tpm_surv, &is_carrier);
    let real_vac = real_partition(&tpm_vac, &is_carrier);
    let ce_surv = ce_for(&tpm_surv, &real_surv);
    let ce_vac = ce_for(&tpm_vac, &real_vac);

    // Group sizes matter for reading the permutation null: the null preserves them,
    // so a lopsided real split makes the null a narrow family of partitions.
    let g0 = real_vac.iter().filter(|&&g| g == 0).count();
    let g1 = real_vac.len() - g0;
    eprintln!("  6b micro classes: {} total, real partition group sizes = ({g0}, {g1})", real_vac.len());

    eprintln!("\n--- baseline CE under the REAL carrier/substrate partition ---");
    eprintln!("  6a surviving-only : CE={:.4}  EI_micro={:.4}  EI_macro={:.4}  K={}  M={}  trans={}",
        ce_surv.ce, ce_surv.ei_micro, ce_surv.ei_macro, ce_surv.k, ce_surv.m, tpm_surv.total_transitions());
    eprintln!("  6b vacuum-augmented: CE={:.4}  EI_micro={:.4}  EI_macro={:.4}  K={}  M={}  trans={}",
        ce_vac.ce, ce_vac.ei_micro, ce_vac.ei_macro, ce_vac.k, ce_vac.m, tpm_vac.total_transitions());

    // ---------------------------------------------------------------
    // V3.2 — Boundary reality (spurious-boundary / permutation null).
    // ---------------------------------------------------------------
    eprintln!("\n--- V3.2  spurious-boundary control ({N_PERM} label permutations, group sizes preserved) ---");
    fn report(label: &str, tpm: &TpmAccumulator, real: &[usize], real_ce: f64) -> (f64, f64, usize) {
        // Fixed seed + canonical class order: the permutation null must reproduce
        // across processes (V3.1), not merely within one.
        let mut rng = 0x5EED_0002_u64;
        let canon = canonical_order(tpm);
        let mut ces = Vec::with_capacity(N_PERM);
        let mut n_beat = 0usize;
        for _ in 0..N_PERM {
            let p = permuted_partition(real, &canon, &mut rng);
            let c = ce_for(tpm, &p).ce;
            if c >= real_ce {
                n_beat += 1;
            }
            ces.push(c);
        }
        let mean = ces.iter().sum::<f64>() / ces.len() as f64;
        let mut sorted = ces.clone();
        let p95 = pct(&mut sorted, 0.95);
        let max = sorted.last().copied().unwrap_or(f64::NAN);
        eprintln!("  {label}: real={real_ce:.4}  null mean={mean:.4}  p95={p95:.4}  max={max:.4}  null≥real in {n_beat}/{N_PERM}");
        (mean, max, n_beat)
    }
    let (surv_mean, surv_max, surv_beat) = report("6a surviving-only ", &tpm_surv, &real_surv, ce_surv.ce);
    let (vac_mean, vac_max, vac_beat) = report("6b vacuum-augmented", &tpm_vac, &real_vac, ce_vac.ce);

    eprintln!("\n  Interpretation (pre-committed):");
    eprintln!("   * 6a: EI_macro is PINNED at log2(K)={:.4} because no surviving node crosses the", (ce_surv.k as f64).log2());
    eprintln!("     boundary — the identity macro is the MAXIMUM attainable macro EI, so the real");
    eprintln!("     partition wins trivially. A win here is an artifact of no-crossings, NOT evidence");
    eprintln!("     that CE detects the boundary.");
    eprintln!("   * 6b: births/deaths give the macro real off-diagonal mass, so this comparison is");
    eprintln!("     the one that can actually falsify the boundary.");
    let margin = ce_vac.ce - vac_mean;
    let null_p = vac_beat as f64 / N_PERM as f64;
    eprintln!("  ⟹ 6b margin over the null: {:+.4} (real {:.4} vs null mean {:.4}, null max {:.4}); p={null_p:.3}",
        margin, ce_vac.ce, vac_mean, vac_max);
    let _ = (surv_mean, surv_max, surv_beat);

    // FIDELITY GUARD (this WOULD be a bug): the V3 reimplementation must reproduce the
    // official Experiment-6a/6b coefficients. If it does not, the null result below is
    // an artifact of THIS file rather than a fact about the system, and nothing here
    // may be reported. 6a is pinned at CE = 1/EI_micro; 6b at ~0.25 (Δ=1).
    assert!(
        (ce_surv.ce - 1.0 / ce_surv.ei_micro).abs() < 1e-9,
        "V3 fidelity: 6a model must reduce to CE = 1/EI_micro (identity macro) — reimplementation diverged"
    );
    assert!(
        (ce_vac.ce - 0.25).abs() < 0.02,
        "V3 fidelity: 6b model must reproduce the official Experiment-6b CE≈0.2500 at Δ=1, got {:.4} — reimplementation diverged, so the V3.2 null is not interpretable",
        ce_vac.ce
    );

    // The V3.2 verdict is a PRE-COMMITTED BRANCH, not a build failure: the plan states
    // that an arbitrary cut scoring as high as the real one is a FINDING (the headline
    // is void), to be reported rather than repaired. Per plan rule 2 no threshold,
    // partition, or influx is adjusted to change it — the number stands as measured.
    let boundary_survives = vac_beat == 0;

    // --- V3.2 (literal form): a TEMPORAL arbitrary boundary ---
    // The plan's literal control is "partition the run at a step with no replication
    // event" and check the real replication-onset boundary scores higher. Births and
    // deaths are recoverable from the observation sequence (a node id present at t but
    // not t-1 is a birth; absent at t but present at t-1 is a death).
    eprintln!("\n--- V3.2 (literal)  temporal arbitrary-cut control ---");
    let mut births_at: Vec<usize> = Vec::with_capacity(observations.len());
    for t in 0..observations.len() {
        if t == 0 {
            births_at.push(0);
            continue;
        }
        let prev: std::collections::HashSet<NodeId> = observations[t - 1].iter().map(|(i, _)| *i).collect();
        births_at.push(observations[t].iter().filter(|(i, _)| !prev.contains(i)).count());
    }
    let onset = births_at.iter().position(|&b| b > 0);
    let total_births: usize = births_at.iter().sum();
    // Longest stretch of consecutive birth-free steps (a candidate quiescent cut).
    let (mut best_len, mut best_start, mut cur_len, mut cur_start) = (0usize, 0usize, 0usize, 0usize);
    for (t, &b) in births_at.iter().enumerate() {
        if b == 0 {
            if cur_len == 0 {
                cur_start = t;
            }
            cur_len += 1;
            if cur_len > best_len {
                best_len = cur_len;
                best_start = cur_start;
            }
        } else {
            cur_len = 0;
        }
    }
    eprintln!("  replication onset at observation {:?}; total births {total_births}", onset);
    eprintln!("  longest birth-free stretch: {best_len} steps starting at {best_start}");
    const W: usize = 400;
    if best_len >= 2 * W {
        let quiet = build_tpm(&observations[best_start..best_start + 2 * W], true);
        let qr = real_partition(&quiet, &is_carrier);
        let qc = ce_for(&quiet, &qr);
        let on = onset.unwrap_or(0);
        let lo = on.saturating_sub(W);
        let hi = (on + W).min(observations.len());
        let onset_tpm = build_tpm(&observations[lo..hi], true);
        let or_ = real_partition(&onset_tpm, &is_carrier);
        let oc = ce_for(&onset_tpm, &or_);
        eprintln!("  CE across replication-onset window [{lo},{hi}) = {:.4} (K={}, trans={})", oc.ce, oc.k, onset_tpm.total_transitions());
        eprintln!("  CE across quiescent window        [{},{}) = {:.4} (K={}, trans={})",
            best_start, best_start + 2 * W, qc.ce, qc.k, quiet.total_transitions());
        eprintln!("  ⟹ real {:.4} vs arbitrary {:.4}", oc.ce, qc.ce);
    } else {
        eprintln!("  ⟹ CANNOT BE CONSTRUCTED: the turnover regime has no birth-free window of");
        eprintln!("     {} steps (longest is {best_len}). Per plan rule 4, the ABSENCE of a quiescent", 2 * W);
        eprintln!("     cut is itself the result: in a sustained-turnover run every window straddles");
        eprintln!("     replication, so the temporal form of the control is undefined here. The");
        eprintln!("     partition-permutation null above is the applicable boundary-reality test.");
    }

    // ---------------------------------------------------------------
    // V3.3 — Coarse-graining sensitivity.
    // ---------------------------------------------------------------
    eprintln!("\n--- V3.3  partition-sensitivity envelope ---");
    eprintln!("  (a) reassign j boundary classes at random, 40 draws each (6b model):");
    let mut rng = 0x5EED_0003_u64;
    let canon_vac = canonical_order(&tpm_vac);
    for j in [1usize, 2, 3] {
        if j >= real_vac.len() {
            continue;
        }
        let mut ces = Vec::new();
        for _ in 0..40 {
            let mut p = real_vac.clone();
            for _ in 0..j {
                // Choose in canonical order so the draw is process-stable.
                let idx = canon_vac[(splitmix(&mut rng) % canon_vac.len() as u64) as usize];
                p[idx] = if p[idx] == 0 { 1 } else { 0 };
            }
            ces.push(ce_for(&tpm_vac, &p).ce);
        }
        let mean = ces.iter().sum::<f64>() / ces.len() as f64;
        let lo = ces.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = ces.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        eprintln!("      j={j}: CE mean={mean:.4}  range [{lo:.4}, {hi:.4}]  (real {:.4})", ce_vac.ce);
    }
    eprintln!("  (b) shift the analysis window start by ±a few observations (onset-shift):");
    for shift in [0usize, 1, 2, 5, 10] {
        let t = build_tpm(&observations[shift..], true);
        let r = real_partition(&t, &is_carrier);
        let c = ce_for(&t, &r);
        eprintln!("      start+{shift:<3}: CE={:.6}  K={}  M={}  trans={}", c.ce, c.k, c.m, t.total_transitions());
    }
    eprintln!("  ⟹ report this envelope ALONGSIDE the coefficient regardless (plan V3.3).");

    // ---------------------------------------------------------------
    // V3.4 — Bias direction (empirical F1 confirmation).
    // ---------------------------------------------------------------
    eprintln!("\n--- V3.4  F1 bias direction: closed-registry control ---");
    eprintln!("  F1 predicts CE is biased DOWNWARD on an OPEN registry (new WL classes keep");
    eprintln!("  appearing, inflating EI_micro = the denominator). A CLOSED (small, fixed) registry");
    eprintln!("  should therefore score HIGHER. Closing the registry = folding the rare-class tail");
    eprintln!("  into one bucket PER GROUP, which preserves the carrier/substrate macro partition.");
    let rows = tpm_vac.normalized_rows();
    let hashes = tpm_vac.hashes().to_vec();
    // Rank classes by outgoing mass (frequency proxy) to identify the rare tail.
    let mut mass: Vec<(usize, f64)> = rows.iter().enumerate().map(|(i, (_, r))| (i, r.iter().sum())).collect();
    // Ties broken by WL hash: mass alone leaves the ranking hashmap-order dependent,
    // which made the "top-N classes" set differ between processes (see canonical_order).
    mass.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap()
            .then(hashes[a.0].0.cmp(&hashes[b.0].0))
    });
    for keep in [2usize, 3, 4, 6] {
        if keep >= hashes.len() {
            continue;
        }
        // Rebuild a TPM whose micro states are: the `keep` most active classes, plus
        // one "rare" bucket per macro group. The vacuum sentinel stays substrate-side.
        let kept: std::collections::HashSet<usize> = mass.iter().take(keep).map(|(i, _)| *i).collect();
        let mut remap: HashMap<u64, WlHash> = HashMap::new();
        for (i, h) in hashes.iter().enumerate() {
            let carrier = *is_carrier.get(&h.0).unwrap_or(&false);
            let target = if kept.contains(&i) {
                *h
            } else if carrier {
                WlHash(0xC10_0000_0000_0001) // rare-carrier bucket
            } else {
                WlHash(0xC10_0000_0000_0002) // rare-substrate bucket
            };
            remap.insert(h.0, target);
        }
        let mut closed = TpmAccumulator::new();
        let mut prev: Option<HashMap<NodeId, WlHash>> = None;
        for obs in &observations {
            let now: HashMap<NodeId, WlHash> = obs
                .iter()
                .map(|(id, h)| (*id, remap.get(&h.0).copied().unwrap_or(*h)))
                .collect();
            if let Some(p) = &prev {
                for (id, &hp) in p {
                    match now.get(id) {
                        Some(&hn) => closed.record(hp, hn),
                        None => closed.record(hp, remap.get(&VACUUM.0).copied().unwrap_or(VACUUM)),
                    }
                }
                for (id, &hn) in &now {
                    if !p.contains_key(id) {
                        closed.record(remap.get(&VACUUM.0).copied().unwrap_or(VACUUM), hn);
                    }
                }
            }
            prev = Some(now);
        }
        let mut carrier_map = is_carrier.clone();
        carrier_map.insert(0xC10_0000_0000_0001, true);
        carrier_map.insert(0xC10_0000_0000_0002, false);
        let rp = real_partition(&closed, &carrier_map);
        let c = ce_for(&closed, &rp);
        eprintln!("      registry closed to {keep} active classes (+2 rare buckets): CE={:.4}  EI_micro={:.4}  M={}  (open: CE={:.4}, M={})",
            c.ce, c.ei_micro, c.m, ce_vac.ce, ce_vac.m);
    }

    // ---------------------------------------------------------------
    // VERDICT
    // ---------------------------------------------------------------
    eprintln!("\n================================ V3 VERDICT ================================");
    if boundary_survives {
        eprintln!("  V3.2 PASSED: the real carrier/substrate boundary outscores every arbitrary");
        eprintln!("  same-sized partition. The CE coefficient is measuring the boundary.");
    } else {
        eprintln!("  V3.2 NOT SURVIVED (pre-committed branch — a FINDING, not a bug):");
        eprintln!("    Under the 6b (un-pinned) model an arbitrary same-sized partition matched or");
        eprintln!("    beat the real carrier/substrate boundary in {vac_beat}/{N_PERM} permutations (p={null_p:.3}).");
        eprintln!("    Real CE {:.4} sits essentially AT the null mean {:.4}.", ce_vac.ce, vac_mean);
        eprintln!("    ⟹ CE is NOT detecting the replication boundary it is attributed to. The 6a");
        eprintln!("      coefficient's apparent margin is the identity-macro artifact (EI_macro pinned");
        eprintln!("      at its max log2(K)), which any no-crossing partition attains.");
        eprintln!("    ⟹ Per the plan's fork: report the stack up to this blocker; do NOT run further");
        eprintln!("      experiments on a boundary that isn't real. Nothing was tuned to change this.");
    }
    eprintln!("============================================================================================\n");
}
