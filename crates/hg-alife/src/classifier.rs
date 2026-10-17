//! `HypergraphMotifClassifier` — the `MotifClassifier` seam implementation.
//!
//! Classifies on **label-bearing WL class only** (scope §5.2): a carrier-labelled
//! node gets [`CLASS_CARRIER`], everything else [`CLASS_SUBSTRATE`]. It does NOT
//! read energy or genome — motif kind is structural, energy is state. Injected
//! into `CausalObserver` in place of `NullMotifClassifier` by `hg-sim-core`.

use crate::motif::{is_carrier, CLASS_CARRIER, CLASS_SUBSTRATE};
use hg_causal::MotifClassifier;
use hg_core::{GraphSnapshot, NodeId};

#[derive(Default)]
pub struct HypergraphMotifClassifier;

impl HypergraphMotifClassifier {
    pub fn new() -> Self {
        Self
    }
}

impl MotifClassifier for HypergraphMotifClassifier {
    fn classify(&mut self, snapshot: &GraphSnapshot<'_>, node: NodeId) -> u32 {
        match snapshot.get_node(node) {
            Some(data) if is_carrier(data) => CLASS_CARRIER,
            // Missing node or substrate label ⟹ substrate/unclassified.
            _ => CLASS_SUBSTRATE,
        }
    }
}
