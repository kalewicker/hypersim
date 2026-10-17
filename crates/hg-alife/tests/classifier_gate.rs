//! S2 gate (scope §10): `HypergraphMotifClassifier` distinguishes a hand-built
//! carrier node from substrate nodes, on label alone. Plus genome/motif helpers.

use hg_alife::genome::keys;
use hg_alife::{
    carrier_energy, is_carrier, Genome, HypergraphMotifClassifier, CARRIER_LABEL, CLASS_CARRIER,
    CLASS_SUBSTRATE,
};
use hg_causal::MotifClassifier;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use smallvec::SmallVec;

/// Build a substrate ring with one carrier attached to one substrate node.
fn store_with_one_carrier() -> (HypergraphStore, hg_core::NodeId, Vec<hg_core::NodeId>) {
    let mut s = HypergraphStore::new();
    let substrate: Vec<_> = (0..4)
        .map(|_| s.add_node(NodeData::new(Label(0), 0)))
        .collect();
    for w in substrate.windows(2) {
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[w[0], w[1]])));
    }
    // The carrier: a node with the reserved carrier label, occupying substrate[0].
    let carrier = s.add_node(NodeData::new(CARRIER_LABEL, 0));
    s.get_node_mut(carrier)
        .unwrap()
        .set_property(keys::ENERGY, PropertyValue::Float(10.0));
    s.add_edge(HyperEdge::new(
        Label(0),
        SmallVec::from_slice(&[substrate[0], carrier]),
    ));
    (s, carrier, substrate)
}

#[test]
fn classifier_distinguishes_carrier_from_substrate() {
    let (s, carrier, substrate) = store_with_one_carrier();
    let snap = s.snapshot();
    let mut clf = HypergraphMotifClassifier::new();

    assert_eq!(
        clf.classify(&snap, carrier),
        CLASS_CARRIER,
        "the carrier-labelled node must classify as CLASS_CARRIER"
    );
    for &n in &substrate {
        assert_eq!(
            clf.classify(&snap, n),
            CLASS_SUBSTRATE,
            "substrate nodes must classify as CLASS_SUBSTRATE"
        );
    }
}

/// S2.3 NEGATIVE gate: motif kind is STRUCTURAL (label-bearing WL class), not
/// state. Mutating a carrier's energy and genome properties — exactly what the
/// S3 biology does every step — must not change its classification. If the
/// classifier ever read state, it would drift under the ecology and lineage
/// reconstruction would silently go wrong.
#[test]
fn classification_invariant_under_energy_and_genome_change() {
    let (mut s, carrier, _substrate) = store_with_one_carrier();
    let mut clf = HypergraphMotifClassifier::new();
    let before = clf.classify(&s.snapshot(), carrier);
    assert_eq!(before, CLASS_CARRIER);

    // Drive the carrier through a range of energies and genome traits.
    for (energy, thr, sat) in [(0.0, 1.0, 2.0), (99.0, 50.0, 40.0), (-5.0, 0.0, 0.0)] {
        let d = s.get_node_mut(carrier).unwrap();
        d.set_property(keys::ENERGY, PropertyValue::Float(energy));
        d.set_property(keys::REPLICATION_THRESHOLD, PropertyValue::Float(thr));
        d.set_property(keys::SATIATION, PropertyValue::Float(sat));
        assert_eq!(
            clf.classify(&s.snapshot(), carrier),
            CLASS_CARRIER,
            "carrier classification changed under energy/genome mutation \
(energy={energy}) — the classifier is reading state, not structure"
        );
    }
}

#[test]
fn is_carrier_and_carrier_energy() {
    let (s, carrier, substrate) = store_with_one_carrier();
    assert!(is_carrier(s.get_node(carrier).unwrap()));
    assert!(!is_carrier(s.get_node(substrate[0]).unwrap()));

    assert_eq!(carrier_energy(s.get_node(carrier).unwrap()), Some(10.0));
    // A substrate node has no carrier energy even if it carried an energy prop.
    assert_eq!(carrier_energy(s.get_node(substrate[0]).unwrap()), None);
}

#[test]
fn genome_reads_traits_from_node() {
    let mut node = NodeData::new(CARRIER_LABEL, 0);
    node.set_property(keys::HARVEST_RATE, PropertyValue::Float(0.5));
    node.set_property(keys::REPLICATION_THRESHOLD, PropertyValue::Float(12.0));
    node.set_property(keys::SATIATION, PropertyValue::Float(8.0));
    // maintenance_cost deliberately absent ⟹ None (diagnosable, not silent 0).
    let g = Genome::from_node(&node);
    assert_eq!(g.harvest_rate, Some(0.5));
    assert_eq!(g.replication_threshold, Some(12.0));
    assert_eq!(g.satiation, Some(8.0));
    assert_eq!(g.maintenance_cost, None);
}
