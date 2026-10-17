//! `MotifClassifier` seam (v4 §6.2): defined here, implemented by
//! hg-alife. `NullMotifClassifier` is the default for substrate-only runs.

use hg_core::{GraphSnapshot, NodeId};

pub trait MotifClassifier: Send {
    /// Classify the motif around `node`; 0 = unclassified.
    fn classify(&mut self, snapshot: &GraphSnapshot<'_>, node: NodeId) -> u32;
}

pub struct NullMotifClassifier;

impl MotifClassifier for NullMotifClassifier {
    fn classify(&mut self, _snapshot: &GraphSnapshot<'_>, _node: NodeId) -> u32 {
        0
    }
}
