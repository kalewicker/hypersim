//! hg-train: the learning layer's outer loop (NN plan §2.2).
//!
//! Owns the optimizers, the campaign runner, the seed schedule, the
//! campaign CSV and report, and champion serialization. It drives
//! everything through `hg-sim-core`'s rollout API and **never imports
//! `hg-engine`**, so v4 §2.3's "only `hg-sim-core` imports `hg-engine`"
//! holds unchanged.
//!
//! The output discipline it exists to preserve: v4's invariant is one CSV
//! and one report per run, and a campaign is ~10 000 rollouts. Producing
//! 10 000 reports would destroy that discipline rather than honour it, so a
//! campaign emits exactly **one** CSV and **one** report, and rollouts emit
//! nothing at all (`docs/campaign_output.md` §1).

pub mod campaign;
pub mod config;
pub mod diagnostics;
pub mod fitness;
pub mod optim;
pub mod prechecks;
pub mod screen;
pub mod report;
pub mod ridge;
pub mod row;
pub mod stage2;

pub use campaign::{run_campaign, CampaignOutcome, TrainError};
pub use config::{Algo, SeedRotation, TrainingConfig};
pub use fitness::{FitnessBreakdown, FitnessFn, HeldOut};
pub use row::CampaignRow;
