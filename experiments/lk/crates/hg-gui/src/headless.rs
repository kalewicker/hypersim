//! Background headless runs launched from the GUI (live builds).
//!
//! Each run gets its own thread, `SharedState` (for progress display
//! only — nothing is drawn from it), and `RunControl` (so it can be
//! stopped individually). The runs are completely independent of the
//! canvas-attached live controller: launching one never disturbs the
//! run being watched.

use hg_app_api::{RunControl, SharedState};
use hg_sim_core::{run_controlled, SimConfig};
use std::sync::Arc;
use std::thread::JoinHandle;

pub struct HeadlessRun {
    pub label: String,
    pub run_id: String,
    pub shared: SharedState,
    pub control: Arc<RunControl>,
    handle: Option<JoinHandle<Result<String, String>>>,
    /// Final one-line outcome once the thread is reaped: Ok("Completed —
    /// <run_dir>") / Ok("Interrupted — …") or Err(error message).
    pub outcome: Option<Result<String, String>>,
}

impl HeadlessRun {
    pub fn finished(&self) -> bool {
        self.outcome.is_some()
    }
}

#[derive(Default)]
pub struct HeadlessRunManager {
    pub runs: Vec<HeadlessRun>,
}

impl HeadlessRunManager {
    /// Spawn `config` as a background headless run. The effective run id
    /// is pinned up front so the UI can show it while the run is live.
    pub fn launch(&mut self, label: String, mut config: SimConfig) {
        config.run_id = config.effective_run_id();
        let run_id = config.run_id.clone();
        let shared = SharedState::new();
        let control = Arc::new(RunControl::new());

        let t_shared = shared.clone();
        let t_control = control.clone();
        let handle = std::thread::spawn(move || {
            run_controlled(&config, Some(&t_shared), None, None, Some(&t_control))
                .map(|artifacts| {
                    format!(
                        "{} @ step {} — {}",
                        artifacts.summary.provenance.state,
                        artifacts.final_step,
                        artifacts.run_dir.display()
                    )
                })
                .map_err(|e| e.to_string())
        });

        self.runs.push(HeadlessRun {
            label,
            run_id,
            shared,
            control,
            handle: Some(handle),
            outcome: None,
        });
    }

    /// Reap finished threads into `outcome`. Call once per frame.
    pub fn poll(&mut self) {
        for run in &mut self.runs {
            if run.outcome.is_none()
                && run.handle.as_ref().is_some_and(|h| h.is_finished())
            {
                let joined = run.handle.take().unwrap().join();
                run.outcome = Some(match joined {
                    Ok(result) => result,
                    Err(_) => Err("run thread panicked".into()),
                });
            }
        }
    }

    pub fn any_active(&self) -> bool {
        self.runs.iter().any(|r| !r.finished())
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    pub fn stop(&mut self, index: usize) {
        if let Some(run) = self.runs.get(index) {
            run.control.request_stop();
        }
    }

    /// Remove a finished run's row. Ignored while it is still running —
    /// stopping is explicit, dismissal is only tidy-up.
    pub fn dismiss(&mut self, index: usize) {
        if self.runs.get(index).is_some_and(|r| r.finished()) {
            self.runs.remove(index);
        }
    }
}

impl Drop for HeadlessRunManager {
    /// Closing the workbench must never leave a headless manifest lying
    /// as Running: request stops and join so every runner seals its
    /// manifest (Interrupted) first — same contract as
    /// LiveSimController::drop.
    fn drop(&mut self) {
        for run in &self.runs {
            run.control.request_stop();
        }
        for run in &mut self.runs {
            if let Some(h) = run.handle.take() {
                let _ = h.join();
            }
        }
    }
}
