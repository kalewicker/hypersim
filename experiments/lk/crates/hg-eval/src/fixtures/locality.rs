//! LK0 fixture generators (hypersim_leakage_plan_v1.md §3): static graphs
//! with known locality verdicts, used to calibrate the leakage
//! instruments before any live run is trusted. All seeded, all returning
//! a populated `HypergraphStore` (label-0 nodes, label-0 2-edges).
//!
//! | fixture              | expected verdict                            |
//! |----------------------|---------------------------------------------|
//! | `lattice_2d_tri(n)`  | detour_p50 = 2; cut_exponent ≈ 0.50          |
//! | `lattice_3d(n)`      | cut_exponent ≈ 0.67                          |
//! | `small_world(n,p,s)` | leakage metrics monotone in p                |
//! | `expander(n,d,s)`    | wormhole_frac high; cut_exponent > 0.9       |
//! | `ring(n)`            | detour = n−1 (documented degenerate); cut ≈ 0|
//! | `tree(b,depth)`      | cut_exponent → 1 (non-geometric, correctly)  |
//!
//! Sizing note (calibration finding, EXP-L1): an expander's detour is
//! ≈ log_{d−1}(n), so flagging wormholes at the default `detour_r_max`
//! = 12 requires log_{d−1}(n) > 12. At the plan's d = 4 that means
//! n > 3^12 ≈ 531k nodes — far past test budgets. The calibrated test
//! instantiation is `expander(16384, 3, seed)` (log₂ 16384 = 14 > 12);
//! recorded in docs/fixture_calibration.md rather than silently changed.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use smallvec::SmallVec;

/// Wormhole-registrability precondition for an expander (v2 §3.1,
/// F-LK3): the BFS ball at the detour cap must be a small fraction of
/// the graph, `n > 4 · d · (d−1)^(r_max−1)`, or detours can resolve
/// within the cap and the instrument silently under-ranges.
pub fn expander_registrable(n: usize, d: usize, r_max: usize) -> bool {
    let ball: f64 = 4.0 * d as f64 * ((d - 1) as f64).powi(r_max.saturating_sub(1) as i32);
    (n as f64) > ball
}

/// `expander`, with the registrability precondition CHECKED at
/// construction — parameter changes that under-range the instrument
/// become loud construction-time errors, not silent nulls.
pub fn expander_checked(n: usize, d: usize, r_max: usize, seed: u64) -> HypergraphStore {
    assert!(
        expander_registrable(n, d, r_max),
        "expander({n}, {d}) is NOT wormhole-registrable at detour cap {r_max}: \
         requires n > 4·d·(d−1)^(r_max−1) = {} (F-LK3; docs/fixture_calibration.md)",
        4.0 * d as f64 * ((d - 1) as f64).powi(r_max.saturating_sub(1) as i32)
    );
    expander(n, d, seed)
}

/// Registrability for the triangulated torus: its diameter is width/2
/// (both coordinates wrap, diagonal steps), which must exceed the cap
/// or NO edge can ever be a wormhole (the width-24 lesson).
pub fn torus_registrable(width: usize, r_max: usize) -> bool {
    width / 2 > r_max
}

/// `small_world`, with the torus registrability precondition checked.
pub fn small_world_checked(n: usize, p: f64, r_max: usize, seed: u64) -> HypergraphStore {
    assert!(
        torus_registrable(n, r_max),
        "small_world({n}, ..) is NOT wormhole-registrable at detour cap {r_max}: \
         torus diameter {} must exceed the cap (F-LK3)",
        n / 2
    );
    small_world(n, p, seed)
}

fn add_nodes(store: &mut HypergraphStore, n: usize) -> Vec<NodeId> {
    (0..n)
        .map(|_| store.add_node(NodeData::new(Label(0), 0)))
        .collect()
}

fn link(store: &mut HypergraphStore, a: NodeId, b: NodeId) {
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
}

/// Triangulated torus: n×n grid, periodic, with right / down / down-right
/// diagonal edges. Every edge's endpoints share a triangle → detour = 2
/// on every edge; cut_exponent ≈ (2−1)/2 = 0.5.
pub fn lattice_2d_tri(n: usize) -> HypergraphStore {
    let mut store = HypergraphStore::new();
    let ids = add_nodes(&mut store, n * n);
    let at = |x: usize, y: usize| ids[(y % n) * n + (x % n)];
    for y in 0..n {
        for x in 0..n {
            link(&mut store, at(x, y), at(x + 1, y));
            link(&mut store, at(x, y), at(x, y + 1));
            link(&mut store, at(x, y), at(x + 1, y + 1));
        }
    }
    store
}

/// Cubic torus with face diagonals: n×n×n, edges +x/+y/+z plus the three
/// face diagonals (+x+y, +x+z, +y+z). cut_exponent ≈ (3−1)/3 ≈ 0.67.
pub fn lattice_3d(n: usize) -> HypergraphStore {
    let mut store = HypergraphStore::new();
    let ids = add_nodes(&mut store, n * n * n);
    let at = |x: usize, y: usize, z: usize| ids[((z % n) * n + (y % n)) * n + (x % n)];
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                link(&mut store, at(x, y, z), at(x + 1, y, z));
                link(&mut store, at(x, y, z), at(x, y + 1, z));
                link(&mut store, at(x, y, z), at(x, y, z + 1));
                link(&mut store, at(x, y, z), at(x + 1, y + 1, z));
                link(&mut store, at(x, y, z), at(x + 1, y, z + 1));
                link(&mut store, at(x, y, z), at(x, y + 1, z + 1));
            }
        }
    }
    store
}

/// `lattice_2d_tri(n)` plus `round(p · E_lattice)` shortcut edges between
/// uniformly sampled distinct node pairs (seeded; duplicate pairs
/// re-drawn). The planted-wormhole population for sensitivity sweeps.
pub fn small_world(n: usize, p: f64, seed: u64) -> HypergraphStore {
    small_world_with_shortcuts(n, p, seed).0
}

/// `small_world`, also returning the planted shortcut EdgeIds — the
/// TrackedEdgeObserver's lineage seed (v2 §4): absolute counts for
/// EXP-L2 cannot come from sampling, so the planted set is tracked
/// exactly from birth.
pub fn small_world_with_shortcuts(
    n: usize,
    p: f64,
    seed: u64,
) -> (HypergraphStore, Vec<hg_core::EdgeId>) {
    let mut store = lattice_2d_tri(n);
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    let base_edges = store.edge_count();
    let shortcuts = (p * base_edges as f64).round() as usize;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut placed = std::collections::HashSet::new();
    let mut shortcut_ids = Vec::with_capacity(shortcuts);
    let mut guard = 0;
    while shortcut_ids.len() < shortcuts && guard < shortcuts * 100 + 100 {
        guard += 1;
        let a = ids[rng.gen_range(0..ids.len())];
        let b = ids[rng.gen_range(0..ids.len())];
        if a == b {
            continue;
        }
        let key = if a < b { (a, b) } else { (b, a) };
        if placed.insert(key) {
            let id = store.add_edge(hg_core::HyperEdge::new(
                hg_core::Label(0),
                smallvec::SmallVec::from_slice(&[a, b]),
            ));
            shortcut_ids.push(id);
        }
    }
    (store, shortcut_ids)
}

/// Random d-regular graph (union of d/2 seeded random Hamiltonian cycles
/// for even d; configuration model for odd d, self-loops and duplicate
/// pairs re-drawn best-effort). Locally tree-like → almost every edge's
/// alternative path has length ≈ log_{d−1}(n): the wormhole fixture.
pub fn expander(n: usize, d: usize, seed: u64) -> HypergraphStore {
    let mut store = HypergraphStore::new();
    let ids = add_nodes(&mut store, n);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    if d.is_multiple_of(2) {
        for _ in 0..d / 2 {
            let mut order: Vec<usize> = (0..n).collect();
            order.shuffle(&mut rng);
            for i in 0..n {
                link(&mut store, ids[order[i]], ids[order[(i + 1) % n]]);
            }
        }
    } else {
        // Configuration model: d stubs per node, shuffled and paired.
        // Self-pairs / duplicate edges are skipped (slight irregularity
        // is irrelevant to the fixture's verdict).
        assert!((n * d).is_multiple_of(2), "n*d must be even for a d-regular graph");
        let mut stubs: Vec<usize> = (0..n * d).map(|i| i / d).collect();
        stubs.shuffle(&mut rng);
        let mut seen = std::collections::HashSet::new();
        for pair in stubs.chunks(2) {
            let (a, b) = (pair[0], pair[1]);
            if a == b {
                continue;
            }
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                link(&mut store, ids[a], ids[b]);
            }
        }
    }
    store
}

/// Plain cycle. The documented degenerate case: detour = n−1 on every
/// edge (wormholes under the BFS cap), yet cut_exponent ≈ 0 — high girth
/// without path redundancy, disambiguated jointly with instrument 1.2.
pub fn ring(n: usize) -> HypergraphStore {
    let mut store = HypergraphStore::new();
    let ids = add_nodes(&mut store, n);
    for i in 0..n {
        link(&mut store, ids[i], ids[(i + 1) % n]);
    }
    store
}

/// Balanced b-ary tree of the given depth (root at depth 0). Cut scales
/// with volume → cut_exponent → 1: correctly classified non-geometric.
pub fn tree(b: usize, depth: usize) -> HypergraphStore {
    let mut store = HypergraphStore::new();
    let root = store.add_node(NodeData::new(Label(0), 0));
    let mut frontier = vec![root];
    for _ in 0..depth {
        let mut next = Vec::with_capacity(frontier.len() * b);
        for &parent in &frontier {
            for _ in 0..b {
                let child = store.add_node(NodeData::new(Label(0), 0));
                link(&mut store, parent, child);
                next.push(child);
            }
        }
        frontier = next;
    }
    store
}
