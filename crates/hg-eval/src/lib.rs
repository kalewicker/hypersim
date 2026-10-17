//! hg-eval: the scientific evaluator — regime classification, gate
//! enforcement, interestingness (v4 §2.3; docs/metric_formulas.md,
//! docs/gate_formulas.md, docs/fixture_definitions.md).

pub mod campaign;
pub mod cone;
pub mod geometry;
pub mod twin;
pub mod window;
pub mod classify;
pub mod gates;
pub mod archive;
pub mod chains;
pub mod fixtures;
pub mod observer;
pub mod eval_state;
pub mod thresholds;
pub mod substrate;
pub mod restoration;
pub mod lifetimes;

pub use geometry::effective_dimension;
pub use window::{WindowComputer, WindowFeatures};
pub use classify::{classify, interestingness};
pub use gates::GateEngine;
pub use archive::NoveltyArchive;
pub use fixtures::*;
pub use observer::EvaluatorObserver;
pub use eval_state::EvalState;
pub use substrate::{substrate_regime, substrate_structural, substrate_subgraph, SubstrateStructural};
