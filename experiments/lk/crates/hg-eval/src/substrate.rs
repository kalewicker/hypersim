//! Substrate-only metric variants (hg-alife S0.2, scope §9 Q3).
//!
//! When carriers become hypergraph nodes, they inflate `n_nodes`, `d_eff`, and
//! the regime classifier — a carrier bloom would read as the *substrate*
//! `Exploding` (classify.rs fires on `growth_ratio`/`n_nodes`). These functions
//! recompute the structural metrics and the regime over the **substrate
//! subgraph only** (nodes NOT bearing the carrier label, and the edges among
//! them), by building a filtered store and running the *existing* metric
//! functions on it unchanged — no metric logic is duplicated.
//!
//! hg-eval stays ALife-agnostic: the carrier label is passed in as a plain
//! `Label`. `hg-sim-core` (which knows `hg_alife::CARRIER_LABEL`) supplies it.

use crate::classify::classify;
use crate::geometry::effective_dimension;
use crate::window::{WindowComputer, WindowFeatures};
use ahash::AHashMap;
use hg_core::{GraphSnapshot, HyperEdge, HypergraphStore, Label, NodeId};
use hg_schema::Regime;
use smallvec::SmallVec;

/// Build the substrate subgraph as a standalone store: every node whose label
/// is NOT `carrier_label`, plus every edge all of whose endpoints survive.
/// An occupancy edge (substrate↔carrier) is dropped because one endpoint (the
/// carrier) is excluded — exactly the intent. Node labels, energy, generation,
/// and properties are preserved, so substrate WL classes are faithful.
pub fn substrate_subgraph(snapshot: &GraphSnapshot<'_>, carrier_label: Label) -> HypergraphStore {
    let mut out = HypergraphStore::with_step(snapshot.step());
    let mut remap: AHashMap<NodeId, NodeId> = AHashMap::default();
    for (id, data) in snapshot.nodes() {
        if data.label == carrier_label {
            continue;
        }
        let new_id = out.add_node(data.clone());
        remap.insert(id, new_id);
    }
    for (_eid, edge) in snapshot.edges() {
        if edge.nodes.iter().all(|n| remap.contains_key(n)) {
            let mapped: SmallVec<[NodeId; 8]> = edge.nodes.iter().map(|n| remap[n]).collect();
            out.add_edge(HyperEdge::new(edge.label, mapped));
        }
    }
    out
}

/// Stateless substrate-only structural metrics (single snapshot).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SubstrateStructural {
    pub n_nodes: usize,
    pub n_edges: usize,
    pub mean_degree: Option<f64>,
    pub d_eff: Option<f64>,
}

/// Compute the substrate-only structural metrics. `deff_seed` is the same seed
/// the full-graph `effective_dimension` would use for this window.
pub fn substrate_structural(
    snapshot: &GraphSnapshot<'_>,
    carrier_label: Label,
    deff_seed: u64,
) -> SubstrateStructural {
    let sub = substrate_subgraph(snapshot, carrier_label);
    let snap = sub.snapshot();
    let n_nodes = snap.node_count();
    let n_edges = snap.edge_count();
    let mean_degree = if n_nodes > 0 {
        let deg_sum: usize = snap.node_ids().map(|id| snap.edges_incident_to(id).len()).sum();
        Some(deg_sum as f64 / n_nodes as f64)
    } else {
        None
    };
    SubstrateStructural {
        n_nodes,
        n_edges,
        mean_degree,
        d_eff: effective_dimension(&snap, deff_seed),
    }
}

/// Observe the substrate subgraph into a substrate-dedicated `WindowComputer`
/// and classify it. The caller owns `computer` and threads it across windows so
/// growth_ratio/novelty are measured on the substrate's OWN history — never
/// contaminated by carrier births. Returns the substrate features and regime.
pub fn substrate_regime(
    computer: &mut WindowComputer,
    snapshot: &GraphSnapshot<'_>,
    carrier_label: Label,
) -> (WindowFeatures, Regime) {
    let sub = substrate_subgraph(snapshot, carrier_label);
    let features = computer.observe(&sub.snapshot());
    let regime = classify(&features);
    (features, regime)
}
