//! `hg-alife` — the concrete ALife layer: genomes, carriers, and the causal-layer
//! seam that lets carriers be seen as first-class hypergraph structure.
//!
//! **This crate contains almost no dynamics** (scope §1.4). The biology is a
//! rewrite-rule file (`rules/alife_base.toml`, added in S3); this crate is
//! bookkeeping around physics that runs in the engine: genome encoding, a motif
//! classifier, and — later — an observer, a lineage tree, and ecology metrics.
//!
//! Dependency discipline (scope §1.2): this crate depends on `hg-core`,
//! `hg-observe`, `hg-causal`, `hg-schema` — **never `hg-engine`**. The only
//! seam through which anything above the engine changes the graph is the rewrite
//! system, so ALife dynamics are rules, not code here.

pub mod classifier;
pub mod genome;
pub mod lineage;
pub mod metrics;
pub mod motif;
pub mod observer;

pub use classifier::HypergraphMotifClassifier;
pub use genome::{keys, read_float, trait_bounds, Genome, TraitBounds, GENOME_TRAIT_KEYS};
pub use lineage::LineageTree;
pub use metrics::{population, EcologyRow, EcologyWindow};
pub use motif::{
    carrier_energy, is_carrier, MotifKind, CARRIER_LABEL, CLASS_CARRIER, CLASS_SUBSTRATE,
};
pub use observer::AlifeObserver;

/// The biology as a TOML rule string (scope §4.1). `hg-sim-core` parses this
/// into the engine's rule set when an ALife run is configured. Kept here, with
/// the crate that owns the ecology, rather than in the runner.
pub fn alife_base_toml() -> &'static str {
    include_str!("../rules/alife_base.toml")
}

/// Errors surfaced by the ALife bookkeeping layer. Kept minimal until a stage
/// needs a variant (lineage reconstruction and metrics arrive in S5).
#[derive(Debug, thiserror::Error)]
pub enum AlifeError {
    /// A `RewriteEvent` attributed to `replicate` did not carry the expected
    /// parent/child node structure — the lineage tree cannot place the birth.
    #[error("malformed replication event: {0}")]
    MalformedReplication(String),
}
