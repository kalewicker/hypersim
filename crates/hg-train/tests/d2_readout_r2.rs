//! Phase D2 exit criteria — the probe readout (`docs/policy_config.md` §4B).
//!
//! `hypersim_nn_experiments_plan_v2.md` §10, Phase D2:
//!
//! > Exit: r1 reproduces C0–C5 numbers exactly; `glider`'s r2 readout is
//! > non-constant.
//!
//! The two halves are load-bearing in opposite directions. r1 must not move
//! or every published Stage 0a and Stage 0b number moves with it and no
//! r1/r2 disagreement is attributable to anything. r2 must move on `glider`
//! or the whole readout-richness argument (§7) is unsupported and `glider`
//! stays `Undetermined` for a reason nobody has established.

use hg_cognition::readout::{self, ReadoutSet};
use hg_sim_core::{run_rollout, RolloutRequest, SimConfig};
use hg_train::diagnostics::{Fixture, RuleSource};

const STEPS: u64 = 128;
const WINDOW: u64 = 8;
const BUCKETS: usize = 16;
const TOP_N: usize = 16;
const N_LABELS: usize = 8;

fn fixture(name: &'static str, rules: &'static str, ic: &'static str, nodes: u64) -> Fixture {
    Fixture {
        name,
        rules: RuleSource::File(rules),
        ic,
        width: 8,
        node_count: nodes,
        steps: STEPS,
        window: WINDOW,
        step_mode: "MaxParallel",
    }
}

fn glider() -> Fixture {
    fixture("glider", "rules/glider.toml", "GliderRing", 31)
}

fn rows(cfg: &SimConfig, seed: u64, set: ReadoutSet) -> Vec<Vec<f32>> {
    run_rollout(RolloutRequest {
        config: cfg,
        seed,
        policy: Box::new(hg_observe::policy::SelectAllPolicy),
        horizon_steps: STEPS,
        probe: Some(Box::new(readout::probe(set, BUCKETS, TOP_N, N_LABELS))),
    })
    .expect("rollout runs")
    .probe_rows
}

/// How many of a readout's columns actually vary across the run.
///
/// This is `readout_distinct_states`' column-wise refinement
/// (`docs/instrument_registry.md` §2.7): a readout that emits one state
/// cannot separate two trajectories whatever they do, and a *column* that
/// never varies contributes nothing to any distance computed from it.
fn varying_columns(rows: &[Vec<f32>]) -> usize {
    if rows.len() < 2 {
        return 0;
    }
    let dim = rows[0].len();
    (0..dim)
        .filter(|j| rows.iter().any(|r| (r[*j] - rows[0][*j]).abs() > 1e-9))
        .count()
}

/// Distinct whole-row states.
fn distinct_states(rows: &[Vec<f32>]) -> usize {
    let mut seen: Vec<&Vec<f32>> = Vec::new();
    for r in rows {
        if !seen.iter().any(|s| {
            s.len() == r.len() && s.iter().zip(r.iter()).all(|(a, b)| (a - b).abs() <= 1e-9)
        }) {
            seen.push(r);
        }
    }
    seen.len()
}

// =====================================================================
// r1 is frozen
// =====================================================================

#[test]
fn r1_reproduces_the_c0_c5_readout_exactly() {
    // `hg_train::diagnostics::wl_bucket_probe` is the function every C0–C5
    // Stage 0b and Stage 1 number was measured through. It now delegates to
    // `readout::r1`; this asserts the delegation is byte-identical rather
    // than merely similar, on real rollout data.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = glider().sim_config(dir.path(), 4242);

    let via_readout = rows(&cfg, 4242, ReadoutSet::R1);
    let via_diagnostics = run_rollout(RolloutRequest {
        config: &cfg,
        seed: 4242,
        policy: Box::new(hg_observe::policy::SelectAllPolicy),
        horizon_steps: STEPS,
        probe: Some(Box::new(hg_train::diagnostics::wl_bucket_probe(BUCKETS))),
    })
    .expect("rollout runs")
    .probe_rows;

    assert!(!via_readout.is_empty(), "the probe must actually have run");
    assert_eq!(via_readout.len(), via_diagnostics.len());
    for (i, (a, b)) in via_readout.iter().zip(&via_diagnostics).enumerate() {
        assert_eq!(a, b, "r1 diverged from the C0-C5 readout at window {i}");
    }
}

#[test]
fn r2_contains_r1_as_its_first_block() {
    // Not a convenience: it is what makes an r1/r2 disagreement diagnosable.
    // If the two readouts shared no columns, a changed verdict could be the
    // new blocks *or* a changed old block, and there would be no way to
    // tell which.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = glider().sim_config(dir.path(), 4242);
    let one = rows(&cfg, 4242, ReadoutSet::R1);
    let two = rows(&cfg, 4242, ReadoutSet::R2);

    assert_eq!(one.len(), two.len());
    for (a, b) in one.iter().zip(&two) {
        assert_eq!(&b[..BUCKETS], &a[..], "r2's first block must be r1 verbatim");
        assert_eq!(b.len(), readout::r2_dim(BUCKETS, TOP_N, N_LABELS));
    }
}

// =====================================================================
// r2 unblinds glider
// =====================================================================

#[test]
fn gliders_r1_readout_is_exactly_constant() {
    // The premise of the whole §7 argument, measured rather than asserted.
    // `glider` is a conservative relabel on a symmetric ring: it moves a
    // label around a cycle without changing any node's degree, any edge, or
    // the multiset of WL classes — and r1's bucketed histogram is invariant
    // under precisely that.
    //
    // This is why `glider` is `Undetermined` under r1. It is NOT a
    // statement about glider's dynamics: it is a readout that is blind to
    // the only thing happening.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = glider().sim_config(dir.path(), 4242);
    let r = rows(&cfg, 4242, ReadoutSet::R1);

    assert!(r.len() >= 4, "need several windows to call anything constant");
    assert_eq!(
        distinct_states(&r),
        1,
        "glider's r1 readout must be exactly constant — if this fails the \
         §7 premise has changed and Stage 0b' needs re-deriving, not re-running"
    );
    assert_eq!(varying_columns(&r), 0);
}

#[test]
fn gliders_r2_readout_is_not_constant() {
    // **The D2 exit criterion.** Plan v2 risk 3 says outright that r2 may
    // fail here — "a conservative relabel on a symmetric ring may be
    // invisible to *any* coarse-graining that isn't the identity" — and
    // that if so, `glider` is excluded from separation analysis with that
    // stated as its reason rather than carried as `Undetermined` forever.
    //
    // So this test decides which of those two worlds we are in, and its
    // failure message says what to do in the other one.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = glider().sim_config(dir.path(), 4242);
    let r = rows(&cfg, 4242, ReadoutSet::R2);

    let distinct = distinct_states(&r);
    let varying = varying_columns(&r);
    println!(
        "glider under r2: {} windows, {distinct} distinct states, {varying} varying columns",
        r.len()
    );

    assert!(
        distinct >= 2,
        "r2 is still blind to glider ({distinct} distinct state over {} windows). \
         This is plan v2 risk 3 realized: glider must then be EXCLUDED from \
         separation analysis with that stated as its reason, not carried as \
         Undetermined — see docs/policy_config.md §4B.5",
        r.len()
    );
}

#[test]
fn what_actually_changes_in_a_glider_run() {
    // Diagnostic: r2 reports glider constant. Before concluding "the
    // readout is still too coarse", establish whether ANYTHING changes.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = glider().sim_config(dir.path(), 4242);
    let m = run_rollout(RolloutRequest {
        config: &cfg,
        seed: 4242,
        policy: Box::new(hg_observe::policy::SelectAllPolicy),
        horizon_steps: STEPS,
        probe: Some(Box::new(readout::probe(ReadoutSet::R2, BUCKETS, TOP_N, N_LABELS))),
    })
    .expect("rollout runs");
    println!("final_canonical_hash = {:#x}", m.final_canonical_hash);
    for w in &m.windows {
        println!(
            "w{:>3} nodes={} edges={} accepted={} attempted={} jsd={:?} entropy={:?} regime={:?}",
            w.window_id, w.n_nodes, w.n_edges, w.accepted_rewrites, w.attempted_matches,
            w.type_histogram_jsd, w.type_histogram_entropy, w.regime
        );
    }
}

#[test]
fn r2s_extra_blocks_are_what_carry_the_signal_on_glider() {
    // Attribution, not just existence. If r2 discriminated glider only
    // through its r1 block, the previous test would be passing for a reason
    // that contradicts `gliders_r1_readout_is_exactly_constant`, and one of
    // the two would be wrong.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = glider().sim_config(dir.path(), 4242);
    let r = rows(&cfg, 4242, ReadoutSet::R2);

    let r1_block_varies = r
        .iter()
        .any(|row| row[..BUCKETS].iter().zip(&r[0][..BUCKETS]).any(|(a, b)| (a - b).abs() > 1e-9));
    assert!(
        !r1_block_varies,
        "the r1 block inside r2 must be as constant as r1 itself"
    );

    let tail_varies = r.iter().any(|row| {
        row[BUCKETS..]
            .iter()
            .zip(&r[0][BUCKETS..])
            .any(|(a, b)| (a - b).abs() > 1e-9)
    });
    assert!(
        tail_varies,
        "every column that varies on glider must come from r2's new blocks"
    );
}

// =====================================================================
// The readout is a measurement instrument and carries its own guard
// =====================================================================

#[test]
fn r2_stays_dimension_stable_across_a_run() {
    // A probe whose width changes when a new label first appears produces a
    // ragged matrix, and both `kernel_held`'s SVD and Stage 1's ridge solve
    // reject it — after the rollout has been paid for.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = fixture("wn_three", "rules/wn_three.toml", "WhiteNoise", 24)
        .sim_config(dir.path(), 4242);
    let r = rows(&cfg, 4242, ReadoutSet::R2);
    assert!(!r.is_empty());
    let want = readout::r2_dim(BUCKETS, TOP_N, N_LABELS);
    for (i, row) in r.iter().enumerate() {
        assert_eq!(row.len(), want, "window {i} changed the readout's width");
        assert!(
            row.iter().all(|x| x.is_finite()),
            "window {i} produced a non-finite column; one NaN poisons every \
             SVD and ridge solve downstream and would look like a substrate property"
        );
    }
}

#[test]
fn r2_sees_more_than_r1_on_a_substrate_where_both_work() {
    // The consistency check plan v2 §9 asks for under Stage 0b': r2 must
    // not be *worse* on the rule sets r1 already classified, or the new
    // readout has traded one blind spot for another.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = fixture("wn_three", "rules/wn_three.toml", "WhiteNoise", 24)
        .sim_config(dir.path(), 4242);
    let one = rows(&cfg, 4242, ReadoutSet::R1);
    let two = rows(&cfg, 4242, ReadoutSet::R2);
    assert!(
        distinct_states(&two) >= distinct_states(&one),
        "r2 resolves {} states where r1 resolves {} — a superset readout \
         cannot resolve fewer",
        distinct_states(&two),
        distinct_states(&one)
    );
}
