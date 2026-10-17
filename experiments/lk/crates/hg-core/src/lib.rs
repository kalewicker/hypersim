//! hg-core: Core hypergraph data structures for Hypersim (v4 §2.1).
//!
//! The hypergraph itself: `NodeId`/`EdgeId` (generational, via slotmap),
//! `HypergraphStore`, `GraphSnapshot`, graph generators, the deterministic
//! WL neighbourhood hash, and `PropertyValue`. No internal dependencies.

pub mod graph;
pub mod hash;
pub mod generators;
pub mod property;
pub mod snapshot;
pub mod wire;

pub use graph::{HypergraphStore, NodeData, HyperEdge, Label, NodeId, EdgeId, Properties};
pub use hash::{compute_wl_hash, compute_wl_hash_snapshot, compute_wl_class_map, compute_wl_class_map_snapshot, WlHash};
pub use generators::*;
pub use property::{PropertyValue, PropertyOp};
pub use snapshot::GraphSnapshot;
