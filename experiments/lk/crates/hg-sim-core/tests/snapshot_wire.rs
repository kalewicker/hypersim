//! Snapshot wire-type round-trip (hypersim_fix_snapshot_wire_type.md).
//!
//! The precise ambiguity the old untagged encoding could not preserve:
//! `Int(5)` and `Float(5.0)` are indistinguishable bytes on disk. These
//! tests fail on the old encoding (deserialize errors outright) and pass
//! on the tagged wire type. We go through the FULL snapshot write/read
//! path — bincode(store) → SnapshotWriter → SnapshotReader → bincode.

use hg_core::{HypergraphStore, NodeData, Label, PropertyValue};
use hg_output::{SnapshotReader, SnapshotWriter};

const KEY: u32 = 1;

/// A store with one node per PropertyValue variant plus a property-free
/// node, round-tripped through the snapshot path, must preserve every
/// variant exactly (Int stays Int, Float stays Float — the ambiguity the
/// old encoding erased).
#[test]
fn property_variants_round_trip_through_snapshot() {
    let mut store = HypergraphStore::new();
    let mut int_node = NodeData::new(Label(1), 0);
    int_node.set_property(KEY, PropertyValue::Int(5));
    let int_id = store.add_node(int_node);

    let mut float_node = NodeData::new(Label(2), 0);
    float_node.set_property(KEY, PropertyValue::Float(5.0));
    let float_id = store.add_node(float_node);

    let mut bool_node = NodeData::new(Label(3), 0);
    bool_node.set_property(KEY, PropertyValue::Bool(true));
    let bool_id = store.add_node(bool_node);

    // Property-free node: absence must round-trip to absence.
    let bare_id = store.add_node(NodeData::new(Label(4), 0));

    // Full path: serialize -> write -> read -> deserialize.
    let dir = std::env::temp_dir().join("hypersim_snapshot_wire");
    let _ = std::fs::remove_dir_all(&dir);
    let mut w = SnapshotWriter::new(&dir).unwrap();
    let payload = bincode::serialize(&store).expect("serialize");
    let path = w.write_snapshot(store.step(), &payload).unwrap();
    let (_step, back) = SnapshotReader::read_snapshot(&path).expect("read (no DeserializeAny)");
    let restored: HypergraphStore = bincode::deserialize(&back).expect("deserialize store");

    let prop = |s: &HypergraphStore, id| s.get_node(id).unwrap().get_property(KEY);
    assert_eq!(prop(&restored, int_id), Some(PropertyValue::Int(5)), "Int stays Int");
    assert_eq!(prop(&restored, float_id), Some(PropertyValue::Float(5.0)), "Float stays Float");
    assert_eq!(prop(&restored, bool_id), Some(PropertyValue::Bool(true)), "Bool stays Bool");
    assert_eq!(prop(&restored, bare_id), None, "absence stays absent");

    // The ambiguity is genuinely resolved: Int(5) and Float(5.0) are NOT
    // equal after the round-trip (untagged made them the same bytes).
    assert_ne!(prop(&restored, int_id), prop(&restored, float_id));
}

/// Prior-version (v1) snapshot bytes → the migration policy: a version
/// mismatch warns and is treated as absent (read returns an error, so a
/// consumer gets no store). Property-free or not, version-gated uniformly.
#[test]
fn prior_version_snapshot_is_treated_as_absent() {
    let dir = std::env::temp_dir().join("hypersim_snapshot_v1");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("snap_v1.bin");

    // Hand-craft a valid v1 HGSS frame (magic, version=1, step, len, body).
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"HGSS");
    bytes.extend_from_slice(&1u16.to_le_bytes()); // prior version
    bytes.extend_from_slice(&7u64.to_le_bytes()); // step
    let body = b"opaque";
    bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    bytes.extend_from_slice(body);
    std::fs::write(&path, &bytes).unwrap();

    let err = SnapshotReader::read_snapshot(&path).unwrap_err();
    assert!(
        err.to_string().contains("unsupported snapshot version 1"),
        "v1 snapshot must be rejected (absent), got: {err}"
    );
}
