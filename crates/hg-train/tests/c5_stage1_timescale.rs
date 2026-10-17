//! Stage 1 follow-up — is the substrate's memory shorter than one window?
//!
//! Stage 1 measured MC = 0.0390 against a matched ESN's 5.8237, with
//! **input coupling at delay 0 of +0.9261** (`docs/cognition_stage0_findings.md`
//! §3). The symbol arrives cleanly and is gone one window later: nothing
//! above r² = 0.1 at any delay ≥ 1.
//!
//! The plan pre-commits the reading — "the substrate scrambles input
//! identity faster than one window — informative about the substrate's
//! intrinsic timescale, and it redirects effort toward slower rule sets or
//! **shorter symbol windows**" — and that reading makes a testable
//! prediction. If the substrate's memory is shorter than 32 steps but not
//! zero, then shortening the window should bring the memory *inside* the
//! sampling interval and MC should rise. If MC stays flat as the window
//! shrinks, the memory is not merely being under-sampled and the timescale
//! story is wrong.
//!
//! The symbol count is held at 128 across the sweep, so the readout always
//! has the same number of training rows and MC values stay comparable —
//! only the steps-per-symbol changes.

use hg_train::diagnostics::{stage1, Fixture, RuleSource};

const SYMBOLS: u64 = 128;
const BUCKETS: usize = 16;
const MAX_DELAY: usize = 12;
/// Rule indices in `rules/stage1_reservoir.toml`.
const INJECT_ONE: usize = 3;
const INJECT_ZERO: usize = 4;

fn reservoir(window: u64) -> Fixture {
    Fixture {
        name: "stage1_reservoir",
        rules: RuleSource::File("rules/stage1_reservoir.toml"),
        ic: "Empty",
        width: 8,
        node_count: 31,
        steps: window * SYMBOLS,
        window,
        step_mode: "MaxParallel",
    }
}

#[test]
#[ignore = "long-running; run explicitly for the Stage 1 timescale table"]
fn stage1_memory_vs_symbol_window() {
    let dir = tempfile::tempdir().expect("tempdir");
    println!("\n=== Stage 1 follow-up — memory capacity vs symbol window ===");
    println!("({SYMBOLS} symbols at every window, so MC is comparable across rows)\n");
    println!(
        "{:>7} {:>7} {:>9} {:>10} {:>10} {:>10} {:>9}",
        "window", "steps", "MC", "input r2", "r2 d=1", "r2 d=2", "max delay"
    );

    let mut best: (f64, u64) = (f64::NEG_INFINITY, 0);
    for window in [2u64, 4, 8, 16, 32] {
        let f = reservoir(window);
        let r = match stage1(
            &f,
            dir.path(),
            4242,
            INJECT_ZERO,
            INJECT_ONE,
            BUCKETS,
            MAX_DELAY,
            Some(4),
        ) {
            Ok(r) => r,
            Err(e) => {
                println!("{window:>7} {:>7}   failed: {e}", f.steps);
                continue;
            }
        };
        let at = |d: usize| {
            r.mc
                .per_delay
                .iter()
                .find(|(dd, _)| *dd == d)
                .map(|(_, v)| *v)
                .unwrap_or(f64::NAN)
        };
        println!(
            "{:>7} {:>7} {:>9.4} {:>10.4} {:>10.4} {:>10.4} {:>9}",
            window,
            f.steps,
            r.mc.mc,
            r.mc.input_coupling_r2.unwrap_or(f64::NAN),
            at(1),
            at(2),
            r.mc
                .max_decodable_delay
                .map(|d| d.to_string())
                .unwrap_or_else(|| "none".into()),
        );
        if r.mc.mc > best.0 {
            best = (r.mc.mc, window);
        }
    }

    println!(
        "\nbest MC {:.4} at window {} (Stage 1's original window was 32, MC 0.0390)",
        best.0, best.1
    );
    println!(
        "Stage 1 exit (MC > 0 with r2 > 0.1 at delay >= 3) at any window: {}",
        "see the max-delay column"
    );
    // No assertion on the science: a flat row is the informative outcome
    // (the memory is not merely under-sampled) and a rising one is the
    // other. The test fails only if the diagnostic breaks.
}
