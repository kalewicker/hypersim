//! hg-alphabet — the substrate's realised event alphabet (A-track).
//!
//! **Substrate-track measurement, characterised as physics, zero biology.** The
//! output belongs in the substrate characterisation report and the rule zoo,
//! beside the existing regime classifications — not in a methodology track.
//!
//! The quantity is the **realised event space**: the distinct
//! *(rule, local structural context)* signatures a run actually produces. One
//! rule firing into many distinct neighbourhoods generates many distinct
//! events; the same rule firing into a homogeneous substrate generates few.
//! That version can genuinely return either answer, which "which rule fired"
//! — bounded by the rules file by construction — cannot.
//!
//! Definitions, the radius selection rule, the composition and growth formulas,
//! and the calibration bars are frozen in
//! `docs/atrack_a1_preregistration.md`, committed at `a43ca3d` before any
//! cardinality was computed.
//!
//! Module map:
//! - [`signature`] — the three committed signature definitions (A1.1)
//! - [`alphabet`] — cardinality, composition, effective size, novelty curve (A3)
//! - [`saturation`] — the pre-registered radius selection rule (A1.2)
//! - [`growth`] — the novelty-curve regime fit (A4)
//! - [`run`] — the measurement harness
//! - [`zoo`] — the rule zoo under measurement, with committed budgets
//! - [`report`] — formatting that enforces the reporting obligations

pub mod alphabet;
pub mod btrack;
pub mod dtrack;
pub mod etrack;
pub mod growth;
pub mod report;
pub mod run;
pub mod saturation;
pub mod scaling;
pub mod signature;
pub mod zoo;

pub use alphabet::{RunAlphabets, StopReason, SymbolStats};
pub use growth::{fit as fit_growth, GrowthFit, Regime};
pub use run::{measure, Init, RunOutcome, RunSpec};
pub use saturation::{select_radius, Saturation, EPSILON};
pub use scaling::{classify_ceiling, classify_scaling, Ceiling, Scaling};
pub use signature::{degrees, signatures, EventSignatures, N_RADII, R_MAX};
