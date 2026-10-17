//! hg-schema: pure, serializable data only — no logic, no open-ended traits.
//!
//! Owns the types whose shape is essentially frozen once defined:
//! `Regime` (11 variants, v4 §7), `GateStatus`/`GateName`, `MetricsRow`
//! (the CSV row shape), and `RunSummary` (the report content shape).
//! Depends on `serde` only.

pub mod regime;
pub mod gate;
pub mod leakage;
pub mod metrics;
pub mod substrate;
pub mod summary;

pub use regime::Regime;
pub use gate::{GateStatus, GateName};
pub use leakage::{ChainRow, GuardRow, LifetimeRow, ClaimType, ConeRow, ConeStatus, LeakagePhase, LeakageRow, TrackedRow, TwinRow};
pub use metrics::MetricsRow;
pub use substrate::SubstrateRow;
pub use summary::{RunSummary, RunProvenance};
pub mod ere;
pub use ere::{EreMetricsRow, EreSubstrateMetrics, RuleReplicatorKind};
