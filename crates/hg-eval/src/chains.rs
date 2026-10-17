//! Chain census (plan v3 §3): the primary EXP-L9 readout. Exact, cheap,
//! snapshot-based, offline — maximal paths of degree-2, generation > 0
//! nodes. Under the channelization hypothesis (F-LK5) the refinement
//! front extrudes 1D filaments; those filaments ARE these chains, so
//! their count and length distribution is the direct morphological
//! readout the sampled wormhole rate can only approximate. Divergence
//! between the two is signal, not error (v3 hazard 2).
//!
//! Generation is node-property key 6 (docs/property_registry.md); the
//! census requires the capped rule family's inheritance effects (or at
//! least an IC that initializes the key) to see anything.

use hg_core::{HypergraphStore, NodeId, PropertyValue};
use hg_schema::{ChainRow, ClaimType};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Node-property key holding the subdivision generation (Float).
pub const GENERATION_KEY: u32 = 6;

/// Census of one snapshot. Lengths are node counts of maximal paths.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChainCensus {
    pub chains_count: u32,
    pub chain_length_p50: f32,
    pub chain_length_p95: f32,
    pub chain_length_max: u32,
}

fn percentile(sorted: &[u32], p: f64) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let ix = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[ix.min(sorted.len() - 1)] as f32
}

/// Exact chain census of a store: connected components of the subgraph
/// induced on nodes with total degree 2 AND generation > 0. Within that
/// subgraph every node has degree ≤ 2, so components are simple paths
/// (or, degenerately, cycles — counted the same way; a cycle of
/// filament is still filament).
pub fn chain_census(store: &HypergraphStore) -> ChainCensus {
    // Degree = incident hyperedge count; adjacency over all co-membered
    // nodes. Built by iteration (never trusting a deserialized index).
    let mut degree: HashMap<NodeId, u32> = HashMap::new();
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for (_, edge) in store.edges() {
        let mut ends: Vec<NodeId> = edge.nodes.to_vec();
        ends.sort_unstable();
        ends.dedup();
        for &u in &ends {
            *degree.entry(u).or_default() += 1;
            for &v in &ends {
                if v != u {
                    adjacency.entry(u).or_default().push(v);
                }
            }
        }
    }
    let eligible: HashSet<NodeId> = store
        .nodes()
        .filter(|(id, data)| {
            degree.get(id).copied() == Some(2)
                && matches!(data.get_property(GENERATION_KEY),
                            Some(PropertyValue::Float(g)) if g > 0.0)
        })
        .map(|(id, _)| id)
        .collect();

    let mut seen: HashSet<NodeId> = HashSet::new();
    let mut lengths: Vec<u32> = Vec::new();
    for &start in &eligible {
        if seen.contains(&start) {
            continue;
        }
        // Flood the component within the eligible subgraph.
        let mut stack = vec![start];
        seen.insert(start);
        let mut size = 0u32;
        while let Some(u) = stack.pop() {
            size += 1;
            for v in adjacency.get(&u).into_iter().flatten() {
                if eligible.contains(v) && seen.insert(*v) {
                    stack.push(*v);
                }
            }
        }
        lengths.push(size);
    }
    lengths.sort_unstable();
    ChainCensus {
        chains_count: lengths.len() as u32,
        chain_length_p50: percentile(&lengths, 0.50),
        chain_length_p95: percentile(&lengths, 0.95),
        chain_length_max: lengths.last().copied().unwrap_or(0),
    }
}

/// Census every snapshot in `run_dir/snapshots/` (ascending step) and
/// write `run_dir/chains.csv` (Appendix B schema). Returns the rows.
pub fn census_run_dir(run_dir: &Path) -> std::io::Result<Vec<ChainRow>> {
    let snapshots = crate::cone::load_snapshots(&run_dir.join("snapshots"))?;
    let rows: Vec<ChainRow> = snapshots
        .iter()
        .map(|(step, store)| {
            let c = chain_census(store);
            ChainRow {
                snapshot_step: *step,
                chains_count: c.chains_count,
                chain_length_p50: c.chain_length_p50,
                chain_length_p95: c.chain_length_p95,
                chain_length_max: c.chain_length_max,
                claim_type: ClaimType::Estimated,
            }
        })
        .collect();
    let run_id = run_dir
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut csv: hg_output::CsvWriter<ChainRow> =
        hg_output::CsvWriter::create(&run_dir.join("chains.csv"), 1, &run_id, 0, "")?;
    for row in &rows {
        csv.write_row(row)?;
    }
    csv.flush()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hg_core::{HyperEdge, Label, NodeData};
    use smallvec::smallvec;

    fn node(store: &mut HypergraphStore, generation: f64) -> NodeId {
        let mut d = NodeData::new(Label(0), 0);
        d.set_property(GENERATION_KEY, PropertyValue::Float(generation));
        store.add_node(d)
    }

    fn link(store: &mut HypergraphStore, a: NodeId, b: NodeId) {
        store.add_edge(HyperEdge::new(Label(0), smallvec![a, b]));
    }

    /// A 5-link path whose 4 interior nodes carry generation > 0: one
    /// chain of length 4. Endpoints (degree 1 / generation 0) excluded.
    #[test]
    fn single_chain_counted_once_with_exact_length() {
        let mut s = HypergraphStore::new();
        let ends: Vec<NodeId> = (0..6)
            .map(|i| node(&mut s, if i == 0 || i == 5 { 0.0 } else { i as f64 }))
            .collect();
        for w in ends.windows(2) {
            link(&mut s, w[0], w[1]);
        }
        let c = chain_census(&s);
        assert_eq!(c.chains_count, 1);
        assert_eq!(c.chain_length_max, 4);
        assert_eq!(c.chain_length_p50, 4.0);
    }

    /// generation = 0 nodes never join a chain even at degree 2 (the
    /// original lattice must be invisible to the census).
    #[test]
    fn generation_zero_nodes_excluded() {
        let mut s = HypergraphStore::new();
        let a = node(&mut s, 0.0);
        let b = node(&mut s, 0.0);
        let c_ = node(&mut s, 0.0);
        link(&mut s, a, b);
        link(&mut s, b, c_);
        assert_eq!(chain_census(&s), ChainCensus::default());
    }

    /// A branch point (degree 3) splits chains: Y of three 2-chains
    /// around a hub → the hub is ineligible, three separate chains.
    #[test]
    fn branch_points_split_chains() {
        let mut s = HypergraphStore::new();
        let hub = node(&mut s, 1.0);
        let mut lengths = Vec::new();
        for _ in 0..3 {
            let m = node(&mut s, 1.0);
            let tip = node(&mut s, 1.0);
            link(&mut s, hub, m);
            link(&mut s, m, tip);
            lengths.push(m);
        }
        let c = chain_census(&s);
        // hub has degree 3 (out), each m degree 2, each tip degree 1.
        assert_eq!(c.chains_count, 3);
        assert_eq!(c.chain_length_max, 1);
    }
}
