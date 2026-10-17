//! EvaluatorObserver: plugs the evaluator into the step loop via
//! hg-observe. Window cadence is driven by on_window_complete.

use crate::archive::NoveltyArchive;
use crate::classify::{classify, interestingness};
use crate::gates::GateEngine;
use crate::thresholds::ARCHIVE_CAPACITY;
use crate::window::{WindowComputer, WindowFeatures};
use crate::EvalState;
use hg_core::GraphSnapshot;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_schema::GateStatus;

pub struct EvaluatorObserver {
    pub state: EvalState,
    computer: WindowComputer,
    gates: GateEngine,
    archive: NoveltyArchive,
    pub archive_evictions: usize,
}

impl EvaluatorObserver {
    pub fn new(window_len: u64, fixture_gate: GateStatus) -> Self {
        Self {
            state: EvalState::default(),
            computer: WindowComputer::new(window_len),
            gates: GateEngine::new(fixture_gate),
            archive: NoveltyArchive::new(ARCHIVE_CAPACITY),
            archive_evictions: 0,
        }
    }

    /// The latest window's features (None before the first window).
    pub fn latest(&self) -> Option<&WindowFeatures> {
        self.state.features_history.last()
    }

    pub fn archive(&self) -> &NoveltyArchive {
        &self.archive
    }

    pub fn gates(&self) -> &GateEngine {
        &self.gates
    }

    pub fn gates_mut(&mut self) -> &mut GateEngine {
        &mut self.gates
    }

    /// Process one window snapshot; returns the features.
    pub fn process_window(&mut self, snapshot: &GraphSnapshot<'_>) -> WindowFeatures {
        let classes = hg_core::compute_wl_class_map_snapshot(snapshot, crate::thresholds::WL_RADIUS);
        self.process_window_with_classes(snapshot, &classes)
    }

    /// Same, with the once-per-window WL class map supplied by the caller.
    pub fn process_window_with_classes(
        &mut self,
        snapshot: &GraphSnapshot<'_>,
        classes: &ahash::AHashMap<hg_core::NodeId, hg_core::WlHash>,
    ) -> WindowFeatures {
        let f = self.computer.observe_with_classes(snapshot, classes);
        let regime = classify(&f);
        let score = interestingness(&f);

        self.archive.begin_window();
        for sig in &f.new_signatures {
            self.archive.add(*sig);
        }

        self.gates.on_window(&f, regime, score);

        self.state.current_regime = regime;
        self.state.regime_history.push(regime);
        if let Some(pi) = f.pi_normalized {
            self.state.pi_history.push(pi);
        }
        self.state.interestingness_history.push(score);
        self.state.features_history.push(f.clone());
        self.state.gate_statuses = self.gates.statuses();
        self.state.science_unlocked = self.gates.science_unlocked();
        f
    }
}

impl Observer for EvaluatorObserver {
    fn on_step_end(
        &mut self,
        _snapshot: &GraphSnapshot<'_>,
        _stats: &StepStats,
        _events: &[RewriteEvent],
    ) {
    }

    fn on_window_complete(&mut self, snapshot: &GraphSnapshot<'_>, _window: u64) {
        self.process_window(snapshot);
    }

    fn on_run_end(&mut self, _snapshot: &GraphSnapshot<'_>) {}
}
