//! **Reproducibility order gate at n = 3 processes** (binding rule 5).
//!
//! Alphabet enumeration is set-and-hash work, which is exactly where
//! index-order contamination lives. Two findings in this project bound how this
//! must be checked:
//!
//! - **N1.3** — a TPM registry's ORDER differed across processes while every
//!   scalar agreed. Scalars agreeing is not evidence that the structure does.
//! - **C3** — a second hazard the first fix did not touch: entropies summed
//!   over `HashMap::values()` iterate in per-PROCESS order, and float addition
//!   is non-associative, so the same counts differ in the last ulps. It was
//!   caught only at **n = 3**, where processes 0 and 1 agreed bit-for-bit and
//!   only process 2 diverged. n = 2 is not a reproducibility test.
//!
//! This crate is built to be immune by construction — every container is a
//! `BTreeMap` or a sorted `Vec` — so the gate should pass. The RAW
//! (HashMap-order) sum is computed alongside as a **diagnostic**, so the
//! guarantee reads as the conditional it is: reproducible *because* we sum in
//! sorted order, not by luck.

use hg_alphabet::alphabet::SymbolStats;
use hg_alphabet::{measure, zoo};
use std::collections::HashMap;
use std::process::Command;

/// C3.2: n >= 3 from the start, not two.
const N_PROC: usize = 3;

/// Entropy summed in whatever order a `HashMap` yields — the hazard, reproduced
/// deliberately so the gate has something to be a gate against.
fn entropy_raw_hashmap_order(s: &SymbolStats) -> f64 {
    // Re-derive the counts through a std `HashMap`, whose iteration order is
    // seeded per PROCESS by `RandomState` — this is the hazard, reproduced on
    // purpose so the gate has something to be a gate against.
    let mut m: HashMap<u64, u64> = HashMap::new();
    for sym in s.sorted_symbols() {
        m.insert(sym, s.count_of(sym));
    }
    let total = s.total_events() as f64;
    let mut h = 0.0f64;
    for (_k, &c) in m.iter() {
        if c == 0 {
            continue;
        }
        let p = c as f64 / total;
        h -= p * p.log2();
    }
    h
}

/// A fingerprint dense enough that an order-dependent bug has to show up in it:
/// cardinalities and entropies at every radius for every variant, the SORTED
/// symbol-list digest (N1.3's lesson — check the structure, not only scalars),
/// and the growth exponent.
fn fingerprint() -> String {
    let mut out = String::new();
    let specs = [
        zoo::glider().steps(300),
        zoo::growth_treadmill().steps(30),
        zoo::bounded_churn().steps(120).burn_in(0),
    ];
    for spec in specs {
        let a = measure(&spec).full;
        for (variant, stats) in [("s1", &a.context), ("s2", &a.delta)] {
            for (r, s) in stats.iter().enumerate() {
                out.push_str(&format!(
                    "{}.{variant}.{r}.card={}\n{}.{variant}.{r}.H_sorted={:016x}\n{}.{variant}.{r}.H_raw={:016x}\n{}.{variant}.{r}.symdigest={:016x}\n",
                    spec.name,
                    s.cardinality(),
                    spec.name,
                    s.entropy_bits().to_bits(),
                    spec.name,
                    entropy_raw_hashmap_order(s).to_bits(),
                    spec.name,
                    digest(&s.sorted_symbols()),
                ));
            }
        }
        let f = hg_alphabet::fit_growth(&a.context[2].curve_by_event(200));
        out.push_str(&format!(
            "{}.b={:016x}\n{}.n_events={}\n",
            spec.name,
            f.exponent.to_bits(),
            spec.name,
            a.n_events()
        ));
    }
    out
}

/// `DefaultHasher::new()` is SipHash with FIXED zero keys, so it is
/// deterministic across processes — unlike `RandomState`, which is exactly the
/// hazard this file is about. Using it here keeps the digest a property of the
/// symbol list and nothing else.
fn digest(v: &[u64]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    0x9E37_79B9_7F4A_7C15u64.hash(&mut h);
    v.len().hash(&mut h);
    for x in v {
        x.hash(&mut h);
    }
    h.finish()
}

fn field(digest_text: &str, key: &str) -> Vec<String> {
    digest_text
        .lines()
        .filter(|l| l.contains(key))
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn a_alphabet_order_gate_at_three_processes() {
    // CHILD
    if let Ok(path) = std::env::var("A_ORDER_PROBE_OUT") {
        std::fs::write(&path, fingerprint()).expect("child could not write fingerprint");
        return;
    }
    // PARENT
    let exe = std::env::current_exe().expect("current_exe");
    let dir = std::env::temp_dir();
    let paths: Vec<std::path::PathBuf> = (0..N_PROC)
        .map(|i| dir.join(format!("a_order_probe_{i}.txt")))
        .collect();
    for p in &paths {
        let st = Command::new(&exe)
            .args(["a_alphabet_order_gate_at_three_processes", "--exact", "--nocapture"])
            .env("A_ORDER_PROBE_OUT", p)
            .status()
            .expect("failed to spawn child process");
        assert!(st.success(), "child process failed");
    }
    let ds: Vec<String> = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).expect("child wrote no fingerprint"))
        .collect();
    assert!(ds.iter().all(|d| !d.trim().is_empty()), "a child fingerprint is empty");

    eprintln!("\n================ A-TRACK — ALPHABET ORDER GATE, n = {N_PROC} PROCESSES ================");
    eprintln!("  N1.3: scalars agreeing is not evidence the STRUCTURE agrees — the sorted symbol");
    eprintln!("        list is digested too, not only the cardinalities.");
    eprintln!("  C3:   checked at n >= 3, because processes 0 and 1 have agreed bit-for-bit while");
    eprintln!("        only process 2 diverged.\n");

    // THE GATE — every sorted-order field bit-identical across all processes.
    for key in [".card=", ".H_sorted=", ".symdigest=", ".b=", ".n_events="] {
        let cols: Vec<Vec<String>> = ds.iter().map(|d| field(d, key)).collect();
        assert!(!cols[0].is_empty(), "fingerprint has no `{key}` fields — the gate is measuring nothing");
        for i in 1..cols.len() {
            assert_eq!(
                cols[0], cols[i],
                "ORDER GATE FAILED on `{key}`: process 0 and process {i} disagree"
            );
        }
        eprintln!("  {key:<14} bit-identical across {N_PROC} processes: yes ({} fields)", cols[0].len());
    }

    // DIAGNOSTIC — the raw HashMap-order sum. If this also agrees, say so as a
    // fact about this environment, NOT as a guarantee: the guarantee is that we
    // sum in sorted order.
    let raw: Vec<Vec<String>> = ds.iter().map(|d| field(d, ".H_raw=")).collect();
    let raw_agrees = raw.windows(2).all(|w| w[0] == w[1]);
    eprintln!(
        "\n  DIAGNOSTIC .H_raw= (summed in HashMap order): agrees across processes = {raw_agrees}"
    );
    eprintln!("  Either way the guarantee is conditional: this crate is reproducible BECAUSE every");
    eprintln!("  container is a BTreeMap or a sorted Vec, not because HashMap order happened to hold.");
    eprintln!("=====================================================================================\n");
}
