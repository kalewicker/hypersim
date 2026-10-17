//! MockSimController — serves a complete fixture without any dependency
//! beyond hg-app-api itself (Phase 9 exit criterion). The GUI's mock-data
//! shell runs against this.

use crate::commands::SimCommand;
use crate::controller::{SharedState, SimController};
use crate::dto::*;
use crate::events::{send_lossy, send_status, EventChannels, SimEvent, SimStatusEvent};
use hg_schema::{GateName, GateStatus, MetricsRow, Regime, RunSummary};

pub struct MockSimController {
    shared: SharedState,
    channels: EventChannels,
    windows: Vec<MetricsRow>,
    playhead: usize,
    started: bool,
}

impl MockSimController {
    pub fn new() -> Self {
        let shared = SharedState::new();
        let windows = Self::fixture_windows();
        shared.progress.set_max_steps(windows.len() as u64 * 512);
        {
            let mut kg = shared.knowledge_graph.write();
            *kg = Self::fixture_kg();
        }
        Self {
            shared,
            channels: EventChannels::new(),
            windows,
            playhead: 0,
            started: false,
        }
    }

    /// A deterministic, plausible 12-window "growth then generative" run.
    fn fixture_windows() -> Vec<MetricsRow> {
        (0..12u64)
            .map(|w| {
                let mut m = MetricsRow::default();
                m.window_id = w;
                m.step_start = w * 512;
                m.step_end = (w + 1) * 512 - 1;
                m.n_nodes = 30 + (w as usize) * 9;
                m.n_edges = 40 + (w as usize) * 12;
                m.accepted_rewrites = 100 + (w as usize) * 7;
                m.attempted_matches = 900 + (w as usize) * 11;
                m.accepted_per_step = m.accepted_rewrites as f64 / 512.0;
                m.mean_degree = Some(2.6 + 0.05 * w as f64);
                m.activity = Some(0.2 + 0.01 * w as f64);
                m.pi_normalized = Some(0.8 - 0.01 * w as f64);
                m.pi_sample_size = Some(m.n_nodes);
                m.pi_sample_strategy = Some("full".into());
                m.type_histogram_entropy = Some(1.0 + 0.1 * w as f64);
                m.dominant_type_fraction = Some(0.6 - 0.02 * w as f64);
                m.type_histogram_jsd = Some(0.05);
                m.new_type_bin_rate = Some(if w > 6 { 0.004 } else { 0.0 });
                m.interestingness_score = Some(0.4 + 0.03 * w as f64);
                m.regime = Some(
                    if w == 0 {
                        Regime::Unknown
                    } else if w <= 6 {
                        Regime::GrowthTreadmill
                    } else {
                        Regime::GenerativeCandidate
                    }
                    .as_str()
                    .to_string(),
                );
                m
            })
            .collect()
    }

    fn fixture_kg() -> KnowledgeGraphView {
        KnowledgeGraphView {
            tasks: vec![TaskLatticeRow {
                id: 0,
                input_hash: 0xA11CE,
                output_hash: 0xB0B,
                status: "Confirmed".into(),
                certainty: "Empirical".into(),
                witness_count: 5,
            }],
            constructors: vec![ConstructorRow {
                wl_hash: 0xC0FFEE,
                fidelity: 0.93,
                confirmed_cycles: 6,
                is_replicator: true,
                replicator_kind: "trivial".into(),
                certainty: "Law".into(),
            }],
            claims: vec![ClaimRow {
                subject: "0000000000c0ffee".into(),
                predicate: "catalyzes".into(),
                object: "task_0".into(),
                certainty: "Model".into(),
                step: 7,
            }],
            ei_results: vec![crate::EiResultRow {
                strategy: "functional".into(),
                ei_micro: 0.8113,
                ei_macro: 1.0,
                ce_coefficient: 1.2325,
                transition_count: 4096,
                cadence_steps: 128,
                class_registry_size: 64,
            }],
        }
    }

    /// Advance the playhead one window (the GUI's scrub control calls this).
    pub fn advance(&mut self) {
        if self.playhead >= self.windows.len() {
            return;
        }
        let row = self.windows[self.playhead].clone();
        self.playhead += 1;

        self.shared.progress.set_step(row.step_end + 1);
        {
            let mut hist = self.shared.metrics_history.write();
            hist.rows.push(row.clone());
        }
        {
            let mut eval = self.shared.eval_state.write();
            let regime = match row.regime.as_deref() {
                Some("GrowthTreadmill") => Regime::GrowthTreadmill,
                Some("GenerativeCandidate") => Regime::GenerativeCandidate,
                _ => Regime::Unknown,
            };
            let old = eval.current_regime;
            eval.current_regime = regime;
            eval.regime_history.push(regime);
            if let Some(pi) = row.pi_normalized {
                eval.pi_history.push(pi);
            }
            eval.gate_statuses = GateName::ALL
                .iter()
                .map(|g| (*g, GateStatus::Pending))
                .collect();
            if old != regime {
                send_lossy(
                    &self.channels.event_tx,
                    SimEvent::RegimeChanged {
                        window: row.window_id,
                        old,
                        new: regime,
                    },
                );
            }
        }
        send_lossy(
            &self.channels.event_tx,
            SimEvent::WindowComplete {
                window: row.window_id,
                metrics: Box::new(row),
            },
        );

        if self.playhead == self.windows.len() {
            send_status(
                &self.channels.status_tx,
                SimStatusEvent::RunCompleted {
                    summary: Box::new(RunSummary::new()),
                },
            );
        }
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}

impl Default for MockSimController {
    fn default() -> Self {
        Self::new()
    }
}

impl SimController for MockSimController {
    fn start(&mut self) {
        self.started = true;
        while self.playhead < self.windows.len() {
            self.advance();
        }
    }

    fn send_command(&self, _cmd: SimCommand) {}

    fn try_recv_event(&self) -> Option<SimEvent> {
        self.channels.event_rx.try_recv().ok()
    }

    fn try_recv_status(&self) -> Option<SimStatusEvent> {
        self.channels.status_rx.try_recv().ok()
    }

    fn shared_state(&self) -> SharedState {
        self.shared.clone()
    }
}
