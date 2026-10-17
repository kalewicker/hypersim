//! The two event streams (v4 §9.1).
//!
//! `SimEvent`: lossy, bounded(256), try_send/drop-on-full — shared state is
//! authoritative, dropping any of these is harmless.
//! `SimStatusEvent`: durable, unbounded, at-most-a-handful-per-run.
//! Send failures must not panic and must not assume a listener exists —
//! every send site is `let _ = tx.send(..)` / `try_send`; the disk-backed
//! manifest is the authoritative status record.

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use hg_core::WlHash;
use hg_schema::{GateName, GateStatus, MetricsRow, Regime, RunSummary};

#[derive(Clone, Debug)]
pub enum SimEvent {
    WindowComplete { window: u64, metrics: Box<MetricsRow> },
    RegimeChanged { window: u64, old: Regime, new: Regime },
    GateUpdated { gate: GateName, status: GateStatus },
    ConstructorConfirmed { hash: WlHash, fidelity: f32, is_replicator: bool },
}

#[derive(Clone, Debug)]
pub enum SimStatusEvent {
    ErrorOccurred { message: String },
    RunCompleted { summary: Box<RunSummary> },
    ScienceUnlocked,
    ScienceLocked { reason: String },
}

pub const SIM_EVENT_BOUND: usize = 256;

/// Both channels, wired with the prescribed disciplines.
pub struct EventChannels {
    pub event_tx: Sender<SimEvent>,
    pub event_rx: Receiver<SimEvent>,
    pub status_tx: Sender<SimStatusEvent>,
    pub status_rx: Receiver<SimStatusEvent>,
}

impl EventChannels {
    pub fn new() -> Self {
        let (event_tx, event_rx) = bounded(SIM_EVENT_BOUND);
        let (status_tx, status_rx) = unbounded();
        Self { event_tx, event_rx, status_tx, status_rx }
    }
}

impl Default for EventChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// Lossy send: drop on full or disconnected, never panic.
pub fn send_lossy(tx: &Sender<SimEvent>, event: SimEvent) {
    let _ = tx.try_send(event);
}

/// Durable-channel send: a disconnected receiver is a normal, expected
/// condition (headless CLI) — never unwrap (v4 §9.1).
pub fn send_status(tx: &Sender<SimStatusEvent>, event: SimStatusEvent) {
    let _ = tx.send(event);
}
