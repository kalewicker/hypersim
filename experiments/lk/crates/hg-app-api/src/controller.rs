//! The `SimController` trait and shared read-model.
//!
//! Shared `Arc<RwLock<..>>` state is authoritative for the GUI; the lossy
//! event stream only nudges redraws. `SimProgress` alone is atomic (§9.2).

use crate::commands::SimCommand;
use crate::dto::{CompactGraphSnapshot, EvalState, KnowledgeGraphView, MetricsHistory};
use crate::events::{SimEvent, SimStatusEvent};
use crate::progress::SimProgress;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct SharedState {
    pub progress: Arc<SimProgress>,
    pub metrics_history: Arc<RwLock<MetricsHistory>>,
    pub compact_graph: Arc<RwLock<CompactGraphSnapshot>>,
    pub eval_state: Arc<RwLock<EvalState>>,
    pub knowledge_graph: Arc<RwLock<KnowledgeGraphView>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(SimProgress::default()),
            metrics_history: Arc::new(RwLock::new(MetricsHistory::default())),
            compact_graph: Arc::new(RwLock::new(CompactGraphSnapshot::default())),
            eval_state: Arc::new(RwLock::new(EvalState::default())),
            knowledge_graph: Arc::new(RwLock::new(KnowledgeGraphView::default())),
        }
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SharedState {
    fn clone(&self) -> Self {
        Self {
            progress: self.progress.clone(),
            metrics_history: self.metrics_history.clone(),
            compact_graph: self.compact_graph.clone(),
            eval_state: self.eval_state.clone(),
            knowledge_graph: self.knowledge_graph.clone(),
        }
    }
}

/// The controller boundary the GUI programs against. hg-sim-core provides
/// the live implementation; MockSimController serves fixtures.
pub trait SimController: Send {
    fn start(&mut self);
    fn send_command(&self, cmd: SimCommand);
    /// Drain one lossy event if available.
    fn try_recv_event(&self) -> Option<SimEvent>;
    /// Drain one durable status event if available.
    fn try_recv_status(&self) -> Option<SimStatusEvent>;
    fn shared_state(&self) -> SharedState;
}
