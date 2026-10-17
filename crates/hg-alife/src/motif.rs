//! Motif kinds and carrier identification.
//!
//! A carrier is a hypergraph **node** distinguished by its `Label` (scope §2).
//! Motif kind is a *structural* category (label-derived); energy and genome are
//! *state* and do not enter the classification (scope §5.2).

use crate::genome::{keys, read_float};
use hg_core::{Label, NodeData};

/// The motif taxonomy. All three variants are defined; only `Carrier` is
/// implemented for Experiment 5 (scope §11 Q3). `Catalyst`/`Shell` are reserved
/// for later phases and carry no behaviour yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotifKind {
    Carrier,
    Catalyst,
    Shell,
}

/// The reserved carrier node label. Carriers are created by `bootstrap_seed`
/// and `replicate` in `alife_base.toml` with THIS label; the rule file hardcodes
/// the numeric value and this constant is the single source of truth. Chosen
/// high to avoid collision with substrate labels (0 and small ints).
pub const CARRIER_LABEL: Label = Label(100);

/// Motif class ids returned by the classifier and stored in `Attribute.motif_class`.
/// `0` matches `NullMotifClassifier` (substrate / unclassified), so a
/// substrate-only run and an ALife run agree on the substrate class id.
pub const CLASS_SUBSTRATE: u32 = 0;
/// A carrier-labelled WL class.
pub const CLASS_CARRIER: u32 = 1;

impl MotifKind {
    /// The `Attribute.motif_class` id for this kind.
    pub fn class_id(self) -> u32 {
        match self {
            MotifKind::Carrier => CLASS_CARRIER,
            // Reserved kinds have no distinct class id yet; they fold into
            // substrate until a later phase gives them structure.
            MotifKind::Catalyst | MotifKind::Shell => CLASS_SUBSTRATE,
        }
    }
}

/// Is this node a carrier? Structural test on the label only.
pub fn is_carrier(node: &NodeData) -> bool {
    node.label == CARRIER_LABEL
}

/// A carrier's energy (property key 1) as `f64`; `None` for a non-carrier or a
/// carrier missing/mis-typing its energy property.
pub fn carrier_energy(node: &NodeData) -> Option<f64> {
    if is_carrier(node) {
        read_float(node, keys::ENERGY)
    } else {
        None
    }
}
