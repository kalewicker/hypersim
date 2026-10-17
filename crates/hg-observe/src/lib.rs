//! hg-observe: the engine's extension-point vocabulary (v4 §2.1).
//!
//! `Observer`, `RewriteEvent`, `ObserverChain`, `Binding`, `StepStats`, and
//! `RuleSelectionPolicy`. Depends on `hg-core` (for `GraphSnapshot`,
//! `NodeId`) and `hg-schema` only — never on `hg-engine`; the engine
//! depends on this crate, not the reverse.

pub mod balls;
pub mod binding;
pub mod event;
pub mod locality;
pub mod observer;
pub mod policy;
pub mod stats;
pub mod tracked;

pub use balls::{ball_profiles, fit_loglog, growth_band, mean_profiles, BallProfile};
pub use binding::Binding;
pub use event::RewriteEvent;
pub use locality::{
    compute_leakage, detour_census, edge_detour, DetourCensus, LeakageComputation,
    LeakageSettings, LocalityObserver, DETOUR_HIST_BINS,
};
pub use observer::{Observer, ObserverChain};
pub use policy::RuleSelectionPolicy;
pub use tracked::{ChurnCounter, EnergyVanishedCounter, RuleFireCounter, SharedTracked, TrackedEdgeObserver};
pub use stats::StepStats;
