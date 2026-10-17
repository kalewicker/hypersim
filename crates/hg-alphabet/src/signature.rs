//! The three committed event-signature definitions (A1.1).
//!
//! Definitions are frozen in `docs/atrack_a1_preregistration.md` §1, committed
//! at `a43ca3d` before any alphabet cardinality existed. Nothing here may be
//! changed without a new pre-registration.
//!
//! An **event** is one accepted rewrite. Its signature is a *(rule, local
//! structural context)* pair — never "which rule fired", which is bounded by
//! the rules file by construction and is the plan's named trap. The vacuous
//! quantity is computed anyway, as `rule_only`, so the contrast is visible in
//! the same table rather than asserted.

use ahash::AHashMap;
use hg_core::{HypergraphStore, NodeId, WlHash};
use hg_observe::RewriteEvent;
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

/// The seed the WL machinery folds into every hasher (`hg_core::hash::WL_SEED`).
/// FxHasher maps an all-zero input stream to 0 and 0 is its fixed point, so a
/// nonzero seed is what stops label-0/energy-0 structure collapsing.
const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// Radii swept. R = 0 is the no-context degenerate baseline and is NOT eligible
/// for selection (pre-registration §2).
pub const R_MAX: u8 = 4;
pub const N_RADII: usize = (R_MAX as usize) + 1;

pub type ClassMap = AHashMap<NodeId, WlHash>;
pub type DegreeMap = AHashMap<NodeId, i64>;

/// Degree sentinel for a node absent from the graph on one side of the rewrite.
pub const ABSENT: i64 = -1;

/// Domain separators, so concatenated sub-sequences can never alias.
const TAG_PRE: u64 = 0xA1_0000_0001;
const TAG_SURVIVORS: u64 = 0xA1_0000_0002;
const TAG_CREATED: u64 = 0xA1_0000_0003;
const TAG_DEGREE: u64 = 0xA1_0000_0004;

/// One event's signature under every committed definition, at every radius.
#[derive(Clone, Copy, Debug)]
pub struct EventSignatures {
    /// The vacuous definition: rule identity alone. Reported as the contrast,
    /// never as the measurement.
    pub rule_only: u64,
    /// S1 — context signature, per radius. PRIMARY.
    pub context: [u64; N_RADII],
    /// S2 — structural-delta signature, per radius.
    pub delta: [u64; N_RADII],
    /// S3 — degree signature. Radius-free.
    pub degree: u64,
}

/// The `rule_only` symbol a given rule id produces — the same value
/// `signatures` computes for `EventSignatures::rule_only`.
///
/// Added for B-track reporting (per-rule event share) and reads nothing it does
/// not already compute: it lets a report map an existing `rule_only` count back
/// to the rule that produced it. It cannot change a measured value.
pub fn rule_only_symbol(rule_id: usize) -> u64 {
    let mut h = FxHasher::default();
    SEED.hash(&mut h);
    rule_id.hash(&mut h);
    h.finish()
}

/// Incident-edge count for every live node.
pub fn degrees(store: &HypergraphStore) -> DegreeMap {
    store
        .nodes()
        .map(|(id, _)| (id, store.edges_incident_to(id).len() as i64))
        .collect()
}

/// WL class maps at every swept radius.
///
/// The obvious implementation calls `compute_wl_class_map(store, r)` once per
/// radius, which repeats rounds `0+1+2+3+4 = 10` times for the five radii. WL
/// refinement is progressive — round `r+1` is computed FROM round `r` — so the
/// whole sweep is four rounds if the intermediate colourings are kept. This is
/// that, and `class_maps_reference` plus the equivalence test below are what
/// keep it honest: the fast path must be bit-identical to the shipped WL
/// machinery, not merely similar to it.
pub fn class_maps(store: &HypergraphStore) -> Vec<ClassMap> {
    let mut out: Vec<ClassMap> = Vec::with_capacity(N_RADII);
    let mut current = hg_core::compute_wl_class_map(store, 0);
    out.push(current.clone());
    for _ in 1..=R_MAX {
        current = refine_once(store, &current);
        out.push(current.clone());
    }
    out
}

/// One WL refinement round, replicating `hg_core::compute_wl_class_map`'s
/// combine step exactly: neighbour hashes sorted, seed folded first, previous
/// colour folded before the neighbours.
fn refine_once(store: &HypergraphStore, current: &ClassMap) -> ClassMap {
    use smallvec::SmallVec;
    let mut next: ClassMap = ClassMap::default();
    for (id, _) in store.nodes() {
        let mut neighbor_hashes: SmallVec<[u64; 8]> = SmallVec::new();
        for edge_id in store.edges_incident_to(id) {
            if let Some(edge) = store.get_edge(*edge_id) {
                for &neighbor in &edge.nodes {
                    if neighbor != id {
                        if let Some(h) = current.get(&neighbor) {
                            neighbor_hashes.push(h.0);
                        }
                    }
                }
            }
        }
        neighbor_hashes.sort_unstable();
        let mut hasher = FxHasher::default();
        SEED.hash(&mut hasher);
        if let Some(prev) = current.get(&id) {
            prev.0.hash(&mut hasher);
        }
        for h in neighbor_hashes {
            h.hash(&mut hasher);
        }
        next.insert(id, WlHash(hasher.finish()));
    }
    next
}

/// The unoptimised sweep: one independent call into the shipped WL machinery
/// per radius. Kept as the reference the fast path is tested against.
pub fn class_maps_reference(store: &HypergraphStore) -> Vec<ClassMap> {
    (0..=R_MAX)
        .map(|r| hg_core::compute_wl_class_map(store, r))
        .collect()
}

fn class_of(map: &ClassMap, node: NodeId) -> u64 {
    map.get(&node).map(|h| h.0).unwrap_or(0)
}

/// Compute all signatures for one event.
///
/// `pre` / `pre_deg` describe `G_t`, the graph at the START of the step — which
/// is exactly the state `Engine::collect_candidates` matched every binding
/// against, so "pre-rewrite graph" has an unambiguous referent. `post` /
/// `post_deg` describe `G_{t+1}`, the joint post-state of the whole step; see
/// the pre-registration's stated limitation for S2/S3.
pub fn signatures(
    event: &RewriteEvent,
    pre: &[ClassMap],
    post: &[ClassMap],
    pre_deg: &DegreeMap,
    post_deg: &DegreeMap,
) -> EventSignatures {
    debug_assert_eq!(pre.len(), N_RADII);
    debug_assert_eq!(post.len(), N_RADII);

    let rule_only = {
        let mut h = FxHasher::default();
        SEED.hash(&mut h);
        event.rule_id.hash(&mut h);
        h.finish()
    };

    let mut context = [0u64; N_RADII];
    let mut delta = [0u64; N_RADII];

    for r in 0..N_RADII {
        // S1 — (rule, pre-state classes of the bound nodes in LHS-var order).
        // Var order is canonical by construction (the compiled pattern fixes
        // variable indices), so this is an ordered sequence, not a multiset.
        let mut h = FxHasher::default();
        SEED.hash(&mut h);
        event.rule_id.hash(&mut h);
        TAG_PRE.hash(&mut h);
        event.binding.bound_nodes.len().hash(&mut h);
        for &v in &event.binding.bound_nodes {
            class_of(&pre[r], v).hash(&mut h);
        }
        context[r] = h.finish();

        // S2 — the same pre-state prefix, then the post-state classes of the
        // surviving bound nodes and of the created nodes. Both post-state
        // collections are UNORDERED, so both are sorted before hashing
        // (binding rule 5).
        let mut survivors: Vec<u64> = event
            .binding
            .bound_nodes
            .iter()
            .filter(|v| !event.nodes_deleted.contains(v))
            .map(|&v| class_of(&post[r], v))
            .collect();
        survivors.sort_unstable();
        let mut created: Vec<u64> = event
            .nodes_created
            .iter()
            .map(|&w| class_of(&post[r], w))
            .collect();
        created.sort_unstable();

        let mut h = FxHasher::default();
        SEED.hash(&mut h);
        event.rule_id.hash(&mut h);
        TAG_PRE.hash(&mut h);
        event.binding.bound_nodes.len().hash(&mut h);
        for &v in &event.binding.bound_nodes {
            class_of(&pre[r], v).hash(&mut h);
        }
        TAG_SURVIVORS.hash(&mut h);
        survivors.len().hash(&mut h);
        for s in &survivors {
            s.hash(&mut h);
        }
        TAG_CREATED.hash(&mut h);
        created.len().hash(&mut h);
        for c in &created {
            c.hash(&mut h);
        }
        delta[r] = h.finish();
    }

    // S3 — (rule, sorted multiset of (pre-degree, post-degree) over the touched
    // nodes). The pair strictly refines "degree change", which it determines;
    // the refinement is declared in the pre-registration, not discovered here.
    let mut touched: Vec<NodeId> = Vec::new();
    let push = |v: NodeId, acc: &mut Vec<NodeId>| {
        if !acc.contains(&v) {
            acc.push(v);
        }
    };
    for &v in &event.binding.bound_nodes {
        push(v, &mut touched);
    }
    for &v in &event.nodes_created {
        push(v, &mut touched);
    }
    // The engine's one documented locality-of-effect exception (v4 §8.1):
    // surviving endpoints of edges removed by dangling-edge cleanup.
    for &v in &event.cleanup_neighbors {
        push(v, &mut touched);
    }
    let mut pairs: Vec<(i64, i64)> = touched
        .iter()
        .map(|v| {
            (
                pre_deg.get(v).copied().unwrap_or(ABSENT),
                post_deg.get(v).copied().unwrap_or(ABSENT),
            )
        })
        .collect();
    // Sorted on CONTENT (the degree pairs), never on NodeId — node ids encode
    // allocation order and must not reach a published statistic.
    pairs.sort_unstable();

    let degree = {
        let mut h = FxHasher::default();
        SEED.hash(&mut h);
        event.rule_id.hash(&mut h);
        TAG_DEGREE.hash(&mut h);
        pairs.len().hash(&mut h);
        for (a, b) in &pairs {
            a.hash(&mut h);
            b.hash(&mut h);
        }
        h.finish()
    };

    EventSignatures {
        rule_only,
        context,
        delta,
        degree,
    }
}
