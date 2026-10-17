//! Phase C1 cost bench: what one `select` call costs at the
//! 512-candidate cap.
//!
//! The plan's exit criterion is *feature extraction + scoring ≤ 10% of step
//! wallclock*. That ratio cannot be computed here — `hg-cognition` does not
//! depend on `hg-engine` and has no step to measure against. What this
//! bench produces is the numerator, in absolute time, at the cap and below
//! it. The ratio is closed out in Phase C2, where `run_rollout` can time a
//! real step loop with and without a policy installed; this number is what
//! that comparison is checked against.
//!
//! `wl_hash_cache` is the load-bearing case: it contrasts a pool whose
//! bindings share nodes (cache hits) with one whose bindings are spread
//! across the graph (cache misses), which is the difference the per-step
//! memoization exists to make.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use hg_cognition::features::{FeatureExtractor, CAND_SCORE_CAP};
use hg_cognition::testkit::{candidate_pool, matched, node_ids, ring_store};
use hg_cognition::{MlpScorer, Parameterized, RandomSubset};
use hg_observe::{Binding, RuleSelectionPolicy};

const N_RULES: usize = 6;
const WL_BUCKETS: usize = 16;
const HIDDEN: usize = 16;

fn mlp(budget: usize, seed: u64) -> MlpScorer {
    let mut p = MlpScorer::new(N_RULES, WL_BUCKETS, HIDDEN, Some(budget), seed);
    let n = p.n_params();
    // Deterministic non-trivial genome: an all-zero one would let the
    // ReLU short-circuit and understate the forward pass.
    let theta: Vec<f32> = (0..n)
        .map(|i| ((i as f32) * 0.037).sin() * 0.5)
        .collect();
    p.set_params(&theta);
    p
}

/// Full `select` at and below the candidate cap.
fn bench_select(c: &mut Criterion) {
    let store = ring_store(4096, 64);
    let snap = store.snapshot();

    let mut group = c.benchmark_group("mlp_select");
    for &n_cands in &[64usize, 256, CAND_SCORE_CAP, CAND_SCORE_CAP * 4] {
        let cands = candidate_pool(&store, 2, n_cands, N_RULES, 3);
        group.bench_with_input(BenchmarkId::from_parameter(n_cands), &cands, |b, cands| {
            // iter_batched with a fresh scorer per run: the policy's WL
            // cache is keyed on the snapshot step, which does not advance
            // in a bench, so a reused scorer would time a warm cache. The
            // setup cost (weight alloc + set_params) is excluded from the
            // measurement.
            b.iter_batched_ref(
                || mlp(16, 1),
                |p| black_box(p.select(&snap, black_box(cands))),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// The control: what selection costs when it does no scoring at all. The
/// gap between this and `mlp_select` is the whole price of learning.
fn bench_random_baseline(c: &mut Criterion) {
    let store = ring_store(4096, 64);
    let snap = store.snapshot();
    let cands = candidate_pool(&store, 2, CAND_SCORE_CAP, N_RULES, 3);

    c.bench_function("random_subset_select_512", |b| {
        let mut p = RandomSubset::new(Some(16), 1);
        b.iter(|| black_box(p.select(&snap, black_box(&cands))));
    });
}

/// Feature extraction alone, separated from the forward pass.
fn bench_features(c: &mut Criterion) {
    let store = ring_store(4096, 64);
    let snap = store.snapshot();
    let cands = candidate_pool(&store, 0, CAND_SCORE_CAP, N_RULES, 3);

    c.bench_function("features_512x_arity3", |b| {
        b.iter(|| {
            // A FRESH extractor per iteration, deliberately. The WL cache
            // is keyed on the snapshot's step counter, and a bench that
            // reuses one extractor against one fixed-step snapshot measures
            // a warm cache forever after its first iteration — which is not
            // a step, it is a step and then 99 free replays.
            let mut fx = FeatureExtractor::new(N_RULES, WL_BUCKETS);
            fx.begin_step(&snap, &cands);
            for cand in &cands {
                black_box(fx.features(&snap, cand));
            }
        });
    });
}

/// Cache hits vs misses: 512 bindings over 8 shared nodes against 512
/// bindings over 1536 distinct nodes.
fn bench_wl_cache(c: &mut Criterion) {
    let store = ring_store(4096, 64);
    let snap = store.snapshot();
    let ids = node_ids(&store);

    let shared: Vec<Binding> = (0..CAND_SCORE_CAP)
        .map(|i| matched(i % N_RULES, 0, &ids[0..3]))
        .collect();
    let spread: Vec<Binding> = (0..CAND_SCORE_CAP)
        .map(|i| matched(i % N_RULES, 0, &ids[i * 3..i * 3 + 3]))
        .collect();

    let mut group = c.benchmark_group("wl_hash_cache");
    for (name, cands) in [("shared_nodes", &shared), ("distinct_nodes", &spread)] {
        group.bench_function(name, |b| {
            b.iter(|| {
                // Fresh per iteration: see the note in bench_features. With
                // a reused extractor both arms measure a warm cache and the
                // comparison reports nothing.
                let mut fx = FeatureExtractor::new(N_RULES, WL_BUCKETS);
                fx.begin_step(&snap, cands);
                for cand in cands.iter() {
                    black_box(fx.features(&snap, cand));
                }
            });
        });
    }
    group.finish();
}

/// Per-node ball walks vs one whole-graph refinement pass.
///
/// `compute_wl_class_map_snapshot` is documented as bit-identical to
/// calling `compute_wl_hash` per node, but O(radius · E) for the whole
/// graph instead of one ball walk each. The evaluator and causal layer
/// already share it per window. Whether the extractor should too is a
/// question about where the crossover sits, and this is the measurement
/// that answers it.
fn bench_wl_strategy(c: &mut Criterion) {
    let store = ring_store(4096, 64);
    let snap = store.snapshot();
    let ids = node_ids(&store);

    let mut group = c.benchmark_group("wl_strategy");
    group.bench_function("class_map_whole_graph", |b| {
        b.iter(|| black_box(hg_core::compute_wl_class_map_snapshot(&snap, 1)));
    });
    for &n in &[128usize, 512, 1536] {
        group.bench_with_input(BenchmarkId::new("per_node_ball", n), &n, |b, &n| {
            b.iter(|| {
                for &id in ids.iter().take(n) {
                    black_box(hg_core::compute_wl_hash_snapshot(&snap, id, 1));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_select,
    bench_random_baseline,
    bench_features,
    bench_wl_cache,
    bench_wl_strategy
);
criterion_main!(benches);
