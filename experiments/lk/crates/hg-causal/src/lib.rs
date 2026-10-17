//! hg-causal: the causal language layer — attributes, tasks, constructors,
//! a certainty-tagged knowledge graph, and effective information (v4 §2.3).

pub mod types;
pub mod task_lattice;
pub mod constructor;
pub mod ei;
pub mod closure;
pub mod event_graph;
pub mod knowledge_graph;
pub mod motif;
pub mod observer;

pub use types::*;
pub use task_lattice::TaskLattice;
pub use constructor::ConstructorRegistry;
pub use closure::{estimate as estimate_closure, ClosureReport, ClosureSettings, WindowSample};
pub use event_graph::{CausalEventObserver, EventGraphLog, EventRecord};
pub use ei::{
    carrier_substrate_partition, coarse_grain, compute_ei_results, ei_from_tpm, partitions_for,
    shannon_entropy, ClassFeatures, EiDump,
    CoarseStrategy, EiComponents, EiResult, EiSettings, TpmAccumulator,
};
pub use knowledge_graph::KnowledgeGraph;
pub use motif::{MotifClassifier, NullMotifClassifier};
pub use observer::CausalObserver;
