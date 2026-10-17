//! `Observer` and `ObserverChain` — the engine's observation seam.
//!
//! Three lifecycle methods (v4 §13 Phase 3): step end (with the step's
//! rewrite events), window completion, and run end. Observers are read-only
//! by construction — they receive `GraphSnapshot`, never the store.

use crate::event::RewriteEvent;
use crate::stats::StepStats;
use hg_core::GraphSnapshot;

pub trait Observer: Send {
    fn on_step_end(
        &mut self,
        _snapshot: &GraphSnapshot<'_>,
        _stats: &StepStats,
        _events: &[RewriteEvent],
    ) {
    }

    fn on_window_complete(&mut self, _snapshot: &GraphSnapshot<'_>, _window: u64) {}

    fn on_run_end(&mut self, _snapshot: &GraphSnapshot<'_>) {}
}

/// Delegates to N members in push order, for all three lifecycle methods.
#[derive(Default)]
pub struct ObserverChain {
    observers: Vec<Box<dyn Observer>>,
}

impl ObserverChain {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn add(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }

    pub fn len(&self) -> usize {
        self.observers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.observers.is_empty()
    }

    pub fn on_step_end(
        &mut self,
        snapshot: &GraphSnapshot<'_>,
        stats: &StepStats,
        events: &[RewriteEvent],
    ) {
        for obs in &mut self.observers {
            obs.on_step_end(snapshot, stats, events);
        }
    }

    pub fn on_window_complete(&mut self, snapshot: &GraphSnapshot<'_>, window: u64) {
        for obs in &mut self.observers {
            obs.on_window_complete(snapshot, window);
        }
    }

    pub fn on_run_end(&mut self, snapshot: &GraphSnapshot<'_>) {
        for obs in &mut self.observers {
            obs.on_run_end(snapshot);
        }
    }
}
