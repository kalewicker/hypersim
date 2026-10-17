//! hg-app-api: GUI/CLI-facing plumbing (v4 §2.1).
//!
//! `SimCommand`, the lossy `SimEvent` / durable `SimStatusEvent` split
//! (§9.1), `SimProgress` (§9.2), the `SimController` trait, and every
//! read-only GUI-facing DTO. This is the crate hg-gui's mock-data shell
//! depends on — nothing more.

pub mod commands;
pub mod control;
pub mod events;
pub mod progress;
pub mod controller;
pub mod dto;
pub mod mock;

pub use commands::SimCommand;
pub use control::RunControl;
pub use events::{SimEvent, SimStatusEvent, EventChannels};
pub use progress::SimProgress;
pub use controller::{SharedState, SimController};
pub use dto::*;
pub use mock::MockSimController;

// The DTOs expose these identifier types in public fields; re-export them
// so hg-app-api consumers (the GUI) never need a direct hg-core edge.
pub use hg_core::{EdgeId, Label, NodeId};
