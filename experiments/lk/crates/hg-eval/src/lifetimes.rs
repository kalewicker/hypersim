//! Handle-lifetime instrument (plan v5 §5, Phase R4): an OFFLINE
//! extension of the chain census — no live observer. Consumes a run's
//! persisted snapshots and writes `lifetimes.csv`.
//!
//! Identity (§1.1 handle-identity principle): a handle is the
//! topological object identified by the hash of its SORTED gen-0 anchor
//! set. A wormhole component is a connected component of the subgraph
//! induced on generation > 0 nodes (any degree — pump chains,
//! swap-regime extrusions, and Q2-style branched trees alike); its
//! anchors are the gen-0 nodes co-membered in an edge with any
//! component node. Edge set, length, and interior node ids are
//! REPRESENTATION, free to change: subdivision of a handle's interior
//! does not change its identity, and neither does partial fuse.
//!
//! Scope: handle_id is RUN-SCOPED. Gen-0 anchor NodeIds are stable
//! within a run (the generation guard makes gen-0 nodes non-removable,
//! and slotmap ids survive snapshot serialization), so the id is
//! well-defined across a run's snapshots; it is NOT portable across
//! runs. Cross-run comparison uses lifetime DISTRIBUTIONS. Under a
//! NodeId permutation the table is invariant up to a bijection of
//! handle_ids (tested).
//!
//! Two parallel throats between the same anchor pair share a handle_id
//! (a doubled handle is one topological handle); an anchor-free gen>0
//! component (a free-floating cycle) hashes the empty set — such
//! components are census-visible but identity-degenerate, and none has
//! ever been observed in the family.

use hg_core::{HypergraphStore, NodeId, PropertyValue};
use hg_schema::{ClaimType, LifetimeRow};
use rustc_hash::FxHasher;
use slotmap::Key;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::chains::GENERATION_KEY;

/// One handle's appearance in one snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct HandleObservation {
    pub handle_id: u64,
    pub nodes: u32,
    /// Edges incident to component nodes (interior + anchor edges).
    pub edges: u32,
    pub anchor_count: u32,
}

/// Exact handle census of one snapshot: components of the gen>0
/// subgraph, each reduced to (anchor-set hash, size measures).
pub fn handle_census(store: &HypergraphStore) -> Vec<HandleObservation> {
    let gen_pos: HashSet<NodeId> = store
        .nodes()
        .filter(|(_, d)| {
            matches!(d.get_property(GENERATION_KEY), Some(PropertyValue::Float(g)) if g > 0.0)
        })
        .map(|(id, _)| id)
        .collect();
    // Adjacency restricted to gen>0 nodes; anchors + incident-edge sets
    // gathered per node in the same pass.
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut anchors_of: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut edges_of: HashMap<NodeId, Vec<hg_core::EdgeId>> = HashMap::new();
    for (eid, edge) in store.edges() {
        let mut ends: Vec<NodeId> = edge.nodes.to_vec();
        ends.sort_unstable();
        ends.dedup();
        for &u in &ends {
            if !gen_pos.contains(&u) {
                continue;
            }
            edges_of.entry(u).or_default().push(eid);
            for &v in &ends {
                if v == u {
                    continue;
                }
                if gen_pos.contains(&v) {
                    adjacency.entry(u).or_default().push(v);
                } else {
                    anchors_of.entry(u).or_default().push(v);
                }
            }
        }
    }
    let mut seen: HashSet<NodeId> = HashSet::new();
    let mut out = Vec::new();
    for &start in &gen_pos {
        if seen.contains(&start) {
            continue;
        }
        let mut stack = vec![start];
        seen.insert(start);
        let mut nodes = 0u32;
        let mut anchors: Vec<NodeId> = Vec::new();
        let mut edges: HashSet<hg_core::EdgeId> = HashSet::new();
        while let Some(u) = stack.pop() {
            nodes += 1;
            anchors.extend(anchors_of.get(&u).into_iter().flatten());
            edges.extend(edges_of.get(&u).into_iter().flatten());
            for v in adjacency.get(&u).into_iter().flatten() {
                if seen.insert(*v) {
                    stack.push(*v);
                }
            }
        }
        anchors.sort_unstable();
        anchors.dedup();
        let mut h = FxHasher::default();
        0x9E37_79B9_7F4A_7C15u64.hash(&mut h);
        for a in &anchors {
            a.data().as_ffi().hash(&mut h);
        }
        out.push(HandleObservation {
            handle_id: h.finish(),
            nodes,
            edges: edges.len() as u32,
            anchor_count: anchors.len() as u32,
        });
    }
    out
}

/// Fold a snapshot sequence into lifetime rows. Returns the rows
/// (birth-step order, ties by handle_id hex) and the resolution bound
/// (max inter-snapshot gap; 0 for a single snapshot).
pub fn lifetimes_from_snapshots(
    snapshots: &[(u64, HypergraphStore)],
) -> (Vec<LifetimeRow>, u64) {
    let bound = snapshots
        .windows(2)
        .map(|w| w[1].0 - w[0].0)
        .max()
        .unwrap_or(0);
    struct Agg {
        birth: u64,
        last_seen: u64,
        death: Option<u64>,
        max_length: u32,
        max_edges: u32,
        anchor_count: u32,
    }
    let mut aggs: BTreeMap<u64, Agg> = BTreeMap::new();
    for (step, store) in snapshots {
        // Merge doubled handles (same anchor set) within a snapshot.
        let mut present: HashMap<u64, HandleObservation> = HashMap::new();
        for ob in handle_census(store) {
            let e = present.entry(ob.handle_id).or_insert_with(|| HandleObservation {
                handle_id: ob.handle_id,
                nodes: 0,
                edges: 0,
                anchor_count: ob.anchor_count,
            });
            e.nodes += ob.nodes;
            e.edges += ob.edges;
            e.anchor_count = e.anchor_count.max(ob.anchor_count);
        }
        for (id, ob) in &present {
            let a = aggs.entry(*id).or_insert(Agg {
                birth: *step,
                last_seen: *step,
                death: None,
                max_length: 0,
                max_edges: 0,
                anchor_count: 0,
            });
            a.last_seen = *step;
            // A handle re-appearing after a recorded death re-opens: the
            // instrument reports the OUTER envelope (birth = first seen,
            // death = first absent after LAST seen) — churn at the same
            // site is measured elsewhere (churn counter), not resolved
            // below snapshot cadence here.
            a.death = None;
            a.max_length = a.max_length.max(ob.nodes);
            a.max_edges = a.max_edges.max(ob.edges);
            a.anchor_count = a.anchor_count.max(ob.anchor_count);
        }
        for (id, a) in aggs.iter_mut() {
            if a.death.is_none() && !present.contains_key(id) {
                a.death = Some(*step);
            }
        }
    }
    let mut rows: Vec<LifetimeRow> = aggs
        .into_iter()
        .map(|(id, a)| LifetimeRow {
            handle_id: format!("{id:016x}"),
            birth_step: a.birth,
            death_step: a.death,
            max_length: a.max_length,
            max_edges: a.max_edges,
            anchor_count: a.anchor_count,
            resolution_bound_steps: bound,
            claim_type: ClaimType::Estimated,
        })
        .collect();
    rows.sort_by(|a, b| {
        a.birth_step
            .cmp(&b.birth_step)
            .then_with(|| a.handle_id.cmp(&b.handle_id))
    });
    (rows, bound)
}

/// Run the instrument over `run_dir/snapshots/` and write
/// `run_dir/lifetimes.csv`. Returns the rows.
pub fn lifetimes_run_dir(run_dir: &Path) -> std::io::Result<Vec<LifetimeRow>> {
    let snapshots = crate::cone::load_snapshots(&run_dir.join("snapshots"))?;
    let (rows, bound) = lifetimes_from_snapshots(&snapshots);
    let run_id = run_dir
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    // The cadence bound rides the header provenance line (v5 Appendix B)
    // as well as every row's resolution_bound_steps column.
    let mut csv: hg_output::CsvWriter<LifetimeRow> = hg_output::CsvWriter::create(
        &run_dir.join("lifetimes.csv"),
        1,
        &run_id,
        0,
        &format!("offline,cadence_bound_steps={bound}"),
    )?;
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

    /// v5 §5 required test 1+2: a scripted birth → interior refinement →
    /// death sequence yields ONE row with correct windows, and interior
    /// subdivision does not change handle_id.
    #[test]
    fn scripted_birth_refinement_death_is_one_row() {
        // Shared anchor substrate: S and N with bulk spurs, ids fixed
        // across the whole sequence (as in a real run).
        let mut base = HypergraphStore::new();
        let s = node(&mut base, 0.0);
        let n = node(&mut base, 0.0);
        let a = node(&mut base, 0.0);
        let b = node(&mut base, 0.0);
        link(&mut base, s, a);
        link(&mut base, n, b);

        // t=100: bare lattice edge S–N (no handle).
        let mut t1 = base.clone();
        link(&mut t1, s, n);
        // t=200: born — S–w–N.
        let mut t2 = base.clone();
        let w = node(&mut t2, 1.0);
        link(&mut t2, s, w);
        link(&mut t2, w, n);
        // t=300: interior refined — S–w–w2–N (same anchors).
        let mut t3 = base.clone();
        let w1 = node(&mut t3, 1.0);
        let w2 = node(&mut t3, 2.0);
        link(&mut t3, s, w1);
        link(&mut t3, w1, w2);
        link(&mut t3, w2, n);
        // t=400: dead — restored S–N.
        let mut t4 = base.clone();
        link(&mut t4, s, n);

        let id2 = handle_census(&t2)[0].handle_id;
        let id3 = handle_census(&t3)[0].handle_id;
        assert_eq!(id2, id3, "interior refinement must not change handle_id (§1.1)");

        let snaps = vec![(100, t1), (200, t2), (300, t3), (400, t4)];
        let (rows, bound) = lifetimes_from_snapshots(&snaps);
        assert_eq!(bound, 100);
        assert_eq!(rows.len(), 1, "exactly one handle: {rows:?}");
        let r = &rows[0];
        assert_eq!(r.birth_step, 200);
        assert_eq!(r.death_step, Some(400));
        assert_eq!(r.max_length, 2, "refined interior is the max");
        assert_eq!(r.max_edges, 3);
        assert_eq!(r.anchor_count, 2);
        assert_eq!(r.resolution_bound_steps, 100);
    }

    /// A handle open at run end has an empty death_step.
    #[test]
    fn open_handle_has_no_death() {
        let mut t = HypergraphStore::new();
        let s = node(&mut t, 0.0);
        let n = node(&mut t, 0.0);
        let w = node(&mut t, 1.0);
        link(&mut t, s, w);
        link(&mut t, w, n);
        let (rows, _) = lifetimes_from_snapshots(&[(100, t.clone()), (200, t)]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].death_step, None);
    }

    /// v5 §5 required test 3: NodeId-permutation invariance — the same
    /// morphological history built in a different insertion order yields
    /// the SAME table up to a bijection of handle_ids (ids are
    /// run-scoped; everything else must match exactly).
    #[test]
    fn lifetime_table_invariant_under_node_permutation() {
        // Two handles with different histories, built by `build(order)`.
        // order = false inserts anchors in reverse.
        let build = |rev: bool| {
            let mut base = HypergraphStore::new();
            let mk = |st: &mut HypergraphStore, k: usize| -> Vec<NodeId> {
                let mut v: Vec<NodeId> = (0..k).map(|_| node(st, 0.0)).collect();
                if rev {
                    v.reverse();
                }
                v
            };
            let anchors = mk(&mut base, 4); // s1 n1 s2 n2
            let (s1, n1, s2, n2) = (anchors[0], anchors[1], anchors[2], anchors[3]);
            // t=10: handle A alive (s1–w–n1).
            let mut t1 = base.clone();
            let w = node(&mut t1, 1.0);
            link(&mut t1, s1, w);
            link(&mut t1, w, n1);
            // t=20: A refined + handle B born (s2–v–n2).
            let mut t2 = base.clone();
            let w1 = node(&mut t2, 1.0);
            let w2 = node(&mut t2, 2.0);
            link(&mut t2, s1, w1);
            link(&mut t2, w1, w2);
            link(&mut t2, w2, n1);
            let v = node(&mut t2, 1.0);
            link(&mut t2, s2, v);
            link(&mut t2, v, n2);
            // t=30: A dead, B alive.
            let mut t3 = base.clone();
            let v3 = node(&mut t3, 1.0);
            link(&mut t3, s2, v3);
            link(&mut t3, v3, n2);
            lifetimes_from_snapshots(&[(10, t1), (20, t2), (30, t3)]).0
        };
        let fwd = build(false);
        let bwd = build(true);
        let strip = |rows: &[LifetimeRow]| -> Vec<(u64, Option<u64>, u32, u32, u32, u64)> {
            let mut v: Vec<_> = rows
                .iter()
                .map(|r| {
                    (
                        r.birth_step,
                        r.death_step,
                        r.max_length,
                        r.max_edges,
                        r.anchor_count,
                        r.resolution_bound_steps,
                    )
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(strip(&fwd), strip(&bwd), "table content must be permutation-invariant");
        // And the id bijection is consistent: distinct ids stay distinct.
        assert_eq!(
            fwd.iter().map(|r| &r.handle_id).collect::<HashSet<_>>().len(),
            bwd.iter().map(|r| &r.handle_id).collect::<HashSet<_>>().len(),
        );
    }
}
