//! F3.2 — cross-process stability of INDEX-DEPENDENT statistics over a TPM.
//!
//! `TpmAccumulator` assigns dense class indices in `record()` call order, and
//! callers routinely drive those calls from `HashMap` iteration, which is
//! `RandomState`-seeded per process. The resulting index permutation is invisible
//! to CE — `coarse_grain` indexes `group_of` by dense id and `ei_from_tpm` is
//! permutation-invariant — so **a test that checks CE cannot catch this**. That is
//! precisely how it survived undetected: in the V-track a permutation-null p-value
//! and a "top-N most active classes" ranking both drifted across processes while
//! every coefficient reproduced to the last digit.
//!
//! This test therefore computes statistics that RANK and PERMUTE:
//!   (a) the class ranking produced via `canonical_order` (the blessed path), and
//!   (b) a deterministic-PRNG permutation null over a two-group partition,
//! and asserts both are byte-identical across two separate OS processes. A
//! same-process rerun could not detect a per-process hash seed, so — following
//! `hg-core/tests/wl_cross_process.rs` — this re-execs ITSELF.
//!
//! It also pins the control: CE is reported alongside and is expected to match
//! even if the ranking does not, which is the whole point.

use hg_causal::{coarse_grain, ei_from_tpm, TpmAccumulator};
use hg_core::WlHash;
use std::collections::HashMap;
use std::process::Command;

/// Deterministic PRNG so the permutation itself contributes no nondeterminism —
/// any cross-process difference must come from index assignment.
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Build a TPM whose `record()` calls are DRIVEN BY HASHMAP ITERATION — i.e. the
/// hazardous pattern, deliberately reproduced so the test has something to catch.
/// If `canonical_order` is ever dropped from the statistics below, this fixture
/// makes them differ across processes and the test fails.
fn build_tpm() -> TpmAccumulator {
    let mut edges: HashMap<u64, Vec<u64>> = HashMap::new();
    for i in 0..24u64 {
        edges.insert(i, vec![(i * 7 + 1) % 24, (i * 13 + 5) % 24, i]);
    }
    let mut tpm = TpmAccumulator::new();
    // Iterating a HashMap here is the point: dense-index assignment order becomes
    // process-dependent.
    for (from, tos) in &edges {
        for to in tos {
            tpm.record(WlHash(*from), WlHash(*to));
        }
    }
    tpm
}

fn statistics() -> String {
    let tpm = build_tpm();
    let mut out = String::new();

    // (a) RANKING via the blessed canonical path.
    let order = tpm.canonical_order();
    let hashes = tpm.hashes();
    let ranked: Vec<String> = order.iter().map(|&i| format!("{:016x}", hashes[i].0)).collect();
    out.push_str(&format!("ranked={}\n", ranked.join(",")));

    // A two-group partition defined by HASH VALUE (deterministic), indexed by dense id.
    let group_of: Vec<usize> = hashes.iter().map(|h| (h.0 % 2) as usize).collect();
    let rows = tpm.normalized_rows();
    let micro = ei_from_tpm(&rows.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>());
    let mac = ei_from_tpm(&coarse_grain(&rows, &group_of, 2));
    let ce = if micro.ei > 0.0 { mac.ei / micro.ei } else { 0.0 };

    // (b) PERMUTATION NULL — shuffled in canonical order so it is process-stable.
    let mut rng = 0x5EED_F3_u64;
    let mut hits = 0usize;
    let mut acc = 0.0f64;
    for _ in 0..64 {
        let mut labels: Vec<usize> = order.iter().map(|&i| group_of[i]).collect();
        for i in (1..labels.len()).rev() {
            let j = (splitmix(&mut rng) % (i as u64 + 1)) as usize;
            labels.swap(i, j);
        }
        let mut permuted = vec![0usize; group_of.len()];
        for (pos, &i) in order.iter().enumerate() {
            permuted[i] = labels[pos];
        }
        let m = ei_from_tpm(&coarse_grain(&rows, &permuted, 2));
        let c = if micro.ei > 0.0 { m.ei / micro.ei } else { 0.0 };
        acc += c;
        if c >= ce {
            hits += 1;
        }
    }
    out.push_str(&format!("null_hits={hits}\n"));
    out.push_str(&format!("null_mean={:.12}\n", acc / 64.0));
    // Control: CE is permutation-invariant and is expected to match regardless —
    // it is precisely what CANNOT catch this class of bug.
    out.push_str(&format!("ce_control={ce:.12}\n"));
    out
}

#[test]
fn index_dependent_statistics_are_cross_process_stable() {
    // Child leg: write statistics to the requested file and exit.
    if let Ok(path) = std::env::var("TPM_STAT_OUT") {
        std::fs::write(&path, statistics()).expect("child writes statistics file");
        std::process::exit(0);
    }

    let exe = std::env::current_exe().expect("current test exe");
    let base = std::env::temp_dir();
    let pid = std::process::id();
    let run_child = |tag: &str| -> String {
        let out = base.join(format!("hypersim_tpm_stat_{pid}_{tag}.txt"));
        let _ = std::fs::remove_file(&out);
        let status = Command::new(&exe)
            .arg("--exact")
            .arg("index_dependent_statistics_are_cross_process_stable")
            .env("TPM_STAT_OUT", &out)
            .status()
            .expect("spawn child");
        assert!(status.success(), "child {tag} exited non-zero");
        let s = std::fs::read_to_string(&out).expect("read child statistics");
        let _ = std::fs::remove_file(&out);
        s
    };

    let a = run_child("a");
    let b = run_child("b");
    assert!(!a.trim().is_empty(), "child produced no statistics");
    assert_eq!(
        a, b,
        "F3.2 FAILURE: an index-dependent statistic over a TPM differs between two \
processes.\nprocess a:\n{a}\nprocess b:\n{b}\nDense class indices are assigned in \
record() order; route every ranking/permutation/sampling through \
TpmAccumulator::canonical_order()."
    );
}

/// The canonical order is genuinely sorted by WL hash, and is a permutation of
/// exactly the dense index set (no dropped or duplicated classes).
#[test]
fn canonical_order_is_a_sorted_permutation() {
    let tpm = build_tpm();
    let order = tpm.canonical_order();
    let hashes = tpm.hashes();
    assert_eq!(order.len(), hashes.len(), "canonical_order must cover every class");
    let mut seen = order.clone();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), hashes.len(), "canonical_order must be a permutation");
    assert!(
        order.windows(2).all(|w| hashes[w[0]].0 <= hashes[w[1]].0),
        "canonical_order must be sorted by WL hash"
    );
}
