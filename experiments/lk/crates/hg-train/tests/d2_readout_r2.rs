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

/// A **positional** readout: node labels in canonical `NodeId` order.
///
/// Not a coarse-graining — it is (almost) the identity, and it is here only
/// as a control. If this varies while every invariant readout is constant,
/// the trajectory is a sequence of *isomorphic* graphs and the constancy is
/// a theorem rather than a resolution limit.
fn positional_probe() -> impl FnMut(&hg_core::GraphSnapshot<'_>, u64) -> Vec<f32> + Send {
    // Slot iteration order, which is stable across windows because this
    // fixture neither creates nor deletes a node — 31 in, 31 out, every
    // window. That makes column `j` the same node throughout, which is
    // exactly what "positional" has to mean here.
    move |snap, _w| snap.nodes().map(|(_, d)| d.label.0 as f32).collect()
}

#[test]
fn glider_is_a_trajectory_through_isomorphic_graphs() {
    // **The D2 exit criterion, resolved against plan v2 risk 3.**
    //
    // r2 does NOT unblind `glider`, and the reason is stronger than "the
    // readout is still too coarse". The run does real work — 8 accepted
    // rewrites in every window, 48 attempted — while every aggregate is
    // bit-identical across all 16 windows: node count, edge count,
    // activity, and `type_histogram_entropy` to sixteen digits, with
    // `type_histogram_jsd` exactly 0.
    //
    // That is the signature of a trajectory through *isomorphic* graphs.
    // The rule translates a label around a symmetric ring, and each step
    // yields a graph isomorphic to the last. **Every isomorphism-invariant
    // readout is constant on it, provably** — r1, r2, and the whole
    // evaluator metric family are isomorphism-invariant by construction, so
    // no amount of added richness reaches it. Only a positional readout
    // does, and a positional readout is not a coarse-graining.
    //
    // Plan v2 risk 3 pre-registers the response: `glider` is **excluded**
    // from separation analysis with that stated as its reason, rather than
    // carried as `Undetermined` indefinitely. This test is that exclusion,
    // written as an assertion so it cannot quietly lapse.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = glider().sim_config(dir.path(), 4242);

    // 1. Every invariant readout is constant.
    let r2_rows = rows(&cfg, 4242, ReadoutSet::R2);
    assert_eq!(
        distinct_states(&r2_rows),
        1,
        "if r2 ever separates glider, this exclusion must be revisited —          re-derive it, do not simply re-run Stage 0b'"
    );
    assert_eq!(varying_columns(&r2_rows), 0);

    // 2. The graph is nonetheless changing, every single window.
    let m = run_rollout(RolloutRequest {
        config: &cfg,
        seed: 4242,
        policy: Box::new(hg_observe::policy::SelectAllPolicy),
        horizon_steps: STEPS,
        probe: Some(Box::new(positional_probe())),
    })
    .expect("rollout runs");
    assert!(
        m.windows.iter().all(|w| w.accepted_rewrites > 0),
        "glider must actually be rewriting, or this is inertness rather          than invisibility"
    );

    // 3. A positional readout separates what every invariant readout cannot.
    let positional = m.probe_rows;
    assert!(
        distinct_states(&positional) > 1,
        "the positional control must vary — if it does not, glider really is          inert and the C0-C5 'invisible, not inert' reading is wrong"
    );

    println!(
        "glider: r2 resolves {} state(s); a positional readout resolves {}          over {} windows, with {} rewrites accepted per window",
        distinct_states(&r2_rows),
        distinct_states(&positional),
        positional.len(),
        m.windows[0].accepted_rewrites,
    );
}

#[test]
fn the_evaluators_own_metrics_are_constant_on_glider_too() {
    // Corroboration from an independent instrument. If only *our* readouts
    // were constant, the honest reading would be that we built two blind
    // ones. The evaluator is isomorphism-invariant for its own reasons and
    // agrees exactly, which is what makes the invariance argument a
    // property of the trajectory rather than of `readout.rs`.
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = glider().sim_config(dir.path(), 4242);
    let m = run_rollout(RolloutRequest {
        config: &cfg,
        seed: 4242,
        policy: Box::new(hg_observe::policy::SelectAllPolicy),
        horizon_steps: STEPS,
        probe: None,
    })
    .expect("rollout runs");

    let w = &m.windows;
    assert!(w.len() >= 8);
    for row in w {
        assert_eq!(row.n_nodes, w[0].n_nodes);
        assert_eq!(row.n_edges, w[0].n_edges);
        assert_eq!(row.accepted_rewrites, w[0].accepted_rewrites);
        assert_eq!(row.type_histogram_entropy, w[0].type_histogram_entropy);
        assert_eq!(row.type_histogram_jsd, Some(0.0));
    }
}

// =====================================================================
// The readout is a measurement instrument and carries its own guard
// =====================================================================

#[test]
fn every_r2_column_is_non_negative() {
    // `stage0b` compares readout rows with Jensen-Shannon divergence, which
    // normalizes each vector by its sum and treats the components as
    // probability mass. A negative component makes that meaningless: the
    // normalizer includes it while the entropy terms skip it, so every
    // other column is silently rescaled.
    //
    // r2's degree-skew column is the only one that could go negative, and
    // this is the assertion that keeps it from doing so.
    let dir = tempfile::tempdir().expect("tempdir");
    for (name, rules, ic, nodes) in [
        ("glider", "rules/glider.toml", "GliderRing", 31u64),
        ("wn_three", "rules/wn_three.toml", "WhiteNoise", 24),
        ("fl1", "rules/fl1_learnable_signal.toml", "GliderRing", 31),
    ] {
        let cfg = fixture(name, rules, ic, nodes).sim_config(dir.path(), 4242);
        for (w, row) in rows(&cfg, 4242, ReadoutSet::R2).iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                assert!(
                    *v >= 0.0,
                    "{name} window {w} column {j} = {v} is negative; JSD over this \
                     readout would silently rescale every other column"
                );
            }
        }
    }
}

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
