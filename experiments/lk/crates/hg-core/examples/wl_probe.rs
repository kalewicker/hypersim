//! S0.1 cross-process WL determinism probe.
//!
//! Builds a fixed fixture and prints its WL hashes. Run as two SEPARATE OS
//! processes and compare stdout: identical ⟹ the WL hasher is process-stable
//! (FxHasher + constant seed), differing ⟹ a RandomState-seeded hasher leaked
//! back in and every cross-run WL claim is invalid. A same-process rerun cannot
//! detect the failure — RandomState seeds once per process — so this MUST be
//! invoked twice from the shell (see `tests/wl_cross_process.rs`).

use hg_core::graph::{HyperEdge, HypergraphStore, Label, NodeData};
use hg_core::hash::{compute_wl_class_map, compute_wl_hash};
use hg_core::property::PropertyValue;
use smallvec::SmallVec;

fn fixture() -> HypergraphStore {
    let mut s = HypergraphStore::new();
    // Varied labels, properties, and energy-field values so the hash exercises
    // every input channel (label, energy field, structure) — and, by the S0
    // invariant, ignores the properties.
    let ids: Vec<_> = (0..12)
        .map(|i| {
            let n = s.add_node(NodeData::new(Label((i % 3) as u32), 0));
            let d = s.get_node_mut(n).unwrap();
            d.energy = (i as f32) * 0.25;
            d.set_property(1, PropertyValue::Float(i as f64));
            d.set_property(11, PropertyValue::Float(0.5 + i as f64));
            d.set_property(20, PropertyValue::Bool(i % 2 == 0));
            n
        })
        .collect();
    // A ring plus a few 3-ary hyperedges for non-trivial neighbourhoods.
    for w in ids.windows(2) {
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[w[0], w[1]])));
    }
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[ids[11], ids[0]])));
    s.add_edge(HyperEdge::new(Label(1), SmallVec::from_slice(&[ids[0], ids[4], ids[8]])));
    s.add_edge(HyperEdge::new(Label(1), SmallVec::from_slice(&[ids[1], ids[5], ids[9]])));
    s
}

fn main() {
    let s = fixture();
    // Per-node hashes at several radii.
    for radius in [1u8, 2, 3] {
        let mut per_node: Vec<u64> = s
            .nodes()
            .map(|(id, _)| compute_wl_hash(&s, id, radius).0)
            .collect();
        per_node.sort_unstable();
        // Fold into one digest so output is order-independent and compact.
        let digest = per_node.iter().fold(0xcbf29ce484222325u64, |acc, &h| {
            (acc ^ h).wrapping_mul(0x100000001b3)
        });
        // Also fold the batch class map, which shares the base-hash recipe.
        let mut map_hashes: Vec<u64> =
            compute_wl_class_map(&s, radius).into_values().map(|w| w.0).collect();
        map_hashes.sort_unstable();
        let map_digest = map_hashes.iter().fold(0xcbf29ce484222325u64, |acc, &h| {
            (acc ^ h).wrapping_mul(0x100000001b3)
        });
        println!("radius={radius} per_node_digest={digest:#018x} class_map_digest={map_digest:#018x}");
    }
}
