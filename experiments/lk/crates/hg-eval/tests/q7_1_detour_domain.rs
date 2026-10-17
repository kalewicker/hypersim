//! Q7-1b: measure the wormhole census's DETECTION DOMAIN directly,
//! rather than assuming the pump formula holds everywhere.
//!
//! The track has used detour = 2L+2 since F-LK14. That was derived in
//! the PUMP, where the seed's own bulk edges are consumed into chains so
//! a return path must descend a SECOND chain. In the swap regime the
//! ambient lattice around a filament is intact, so the return path is
//! (walk back along the chain) + (lattice detour between the anchors) +
//! (walk out) ≈ L + 3 — a different domain for the same instrument.
//!
//! This probe buckets every filament edge by its chain's length and
//! reports the measured detour, so the domain is read off the data.
//!
//! Q7_RUN_DIRS = colon-separated run dirs.

use hg_core::{EdgeId, HypergraphStore, NodeId, PropertyValue};
use std::collections::{HashMap, HashSet};

const GENERATION: u32 = 6;
const R_MAX: usize = 12;

/// Map each filament node to the length of the chain it belongs to.
fn chain_of(store: &HypergraphStore) -> HashMap<NodeId, u32> {
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
        .filter(|(id, d)| {
            degree.get(id).copied() == Some(2)
                && matches!(d.get_property(GENERATION), Some(PropertyValue::Float(g)) if g > 0.0)
        })
        .map(|(id, _)| id)
        .collect();
    let mut out = HashMap::new();
    let mut seen: HashSet<NodeId> = HashSet::new();
    for &start in &eligible {
        if seen.contains(&start) {
            continue;
        }
        let mut members = vec![start];
        seen.insert(start);
        let mut stack = vec![start];
        while let Some(u) = stack.pop() {
            for v in adjacency.get(&u).into_iter().flatten() {
                if eligible.contains(v) && seen.insert(*v) {
                    members.push(*v);
                    stack.push(*v);
                }
            }
        }
        let len = members.len() as u32;
        for m in members {
            out.insert(m, len);
        }
    }
    out
}

#[test]
#[ignore]
fn q7_1_detour_domain() {
    let dirs = std::env::var("Q7_RUN_DIRS").expect("Q7_RUN_DIRS=dir1:dir2:…");
    println!("cell | chain_len L | n_edges | mean detour | max detour | > R_max({R_MAX})?");
    for dir in dirs.split(':').filter(|s| !s.is_empty()) {
        let path = std::path::Path::new(dir);
        let snaps = hg_eval::cone::load_snapshots(path.join("snapshots").as_path()).unwrap();
        // Use the snapshot with the longest chain (richest domain sample).
        let Some((_, store)) = snaps.iter().max_by_key(|(_, s)| {
            chain_of(s).values().copied().max().unwrap_or(0)
        }) else {
            continue;
        };
        let cmap = chain_of(store);
        if cmap.is_empty() {
            continue;
        }
        let snap = store.snapshot();
        // Bucket filament edges by their chain length.
        let mut buckets: HashMap<u32, Vec<f32>> = HashMap::new();
        let mut seen_edges: HashSet<EdgeId> = HashSet::new();
        for (&node, &len) in &cmap {
            for e in store.edges_incident_to(node) {
                if !seen_edges.insert(*e) {
                    continue;
                }
                let d = hg_observe::edge_detour(&snap, *e, R_MAX);
                buckets.entry(len).or_default().push(d);
            }
        }
        let name = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| dir.into());
        let mut keys: Vec<u32> = buckets.keys().copied().collect();
        keys.sort_unstable();
        for len in keys {
            let v = &buckets[&len];
            let finite: Vec<f32> = v.iter().copied().filter(|d| d.is_finite()).collect();
            let mean = finite.iter().sum::<f32>() / finite.len().max(1) as f32;
            let max = v.iter().copied().fold(0.0f32, f32::max);
            let over = v
                .iter()
                .filter(|d| !d.is_finite() || **d as usize > R_MAX)
                .count();
            println!(
                "{name} | {len} | {} | {mean:.1} | {} | {over}",
                v.len(),
                if max.is_finite() { format!("{max:.0}") } else { "inf".into() },
            );
        }
    }
}
