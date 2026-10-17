//! LiveSimController: the real hg_app_api::SimController — a sim thread
//! driving run_controlled, a command channel for Start, and a RunControl
//! for stop/pause (polled by the runner at step boundaries).

use crate::config::SimConfig;
use crate::runner::run_controlled;
use hg_app_api::events::send_status;
use hg_app_api::{
    EventChannels, RunControl, SharedState, SimCommand, SimController, SimEvent, SimStatusEvent,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::Arc;
use std::thread;

pub struct LiveSimController {
    config: SimConfig,
    shared: SharedState,
    channels: EventChannels,
    control: Arc<RunControl>,
    command_tx: Sender<SimCommand>,
    command_rx: Option<Receiver<SimCommand>>,
    sim_thread: Option<thread::JoinHandle<()>>,
}

impl LiveSimController {
    pub fn new(config: SimConfig) -> Self {
        let (command_tx, command_rx) = unbounded();
        let shared = SharedState::new();
        shared.progress.set_max_steps(config.engine.max_steps);
        Self {
            config,
            shared,
            channels: EventChannels::new(),
            control: Arc::new(RunControl::new()),
            command_tx,
            command_rx: Some(command_rx),
            sim_thread: None,
        }
    }

    pub fn join(&mut self) {
        if let Some(h) = self.sim_thread.take() {
            let _ = h.join();
        }
    }
}

impl SimController for LiveSimController {
    fn start(&mut self) {
        if self.sim_thread.is_some() {
            return;
        }
        let Some(cmd_rx) = self.command_rx.take() else { return };
        let config = self.config.clone();
        let shared = self.shared.clone();
        let event_tx = self.channels.event_tx.clone();
        let status_tx = self.channels.status_tx.clone();
        let control = self.control.clone();

        self.sim_thread = Some(thread::spawn(move || {
            // Wait for Start; Quit (or channel drop) before start aborts.
            loop {
                match cmd_rx.recv() {
                    Ok(SimCommand::Start) => break,
                    Ok(SimCommand::Quit) | Err(_) => return,
                    Ok(_) => continue,
                }
            }
            let result = run_controlled(
                &config,
                Some(&shared),
                Some(&event_tx),
                Some(&status_tx),
                Some(&control),
            );
            if let Err(e) = result {
                send_status(
                    &status_tx,
                    SimStatusEvent::ErrorOccurred {
                        message: e.to_string(),
                    },
                );
            }
        }));
        let _ = self.command_tx.send(SimCommand::Start);
    }

    fn send_command(&self, cmd: SimCommand) {
        match cmd {
            // Mid-run commands act through RunControl (the runner polls it
            // each step); the channel only matters before the run starts.
            SimCommand::Pause => self.control.request_pause(),
            SimCommand::Resume => self.control.request_resume(),
            SimCommand::Quit => {
                self.control.request_stop();
                let _ = self.command_tx.send(SimCommand::Quit); // unblock pre-start wait
            }
            // Target steps/sec; non-finite or <= 0 uncaps. Polled at step
            // boundaries like pause/stop.
            SimCommand::SetSpeed(v) => self.control.set_speed_steps_per_sec(v),
            // Step / Reset: not yet implemented (user guide §14.4).
            other => {
                let _ = self.command_tx.send(other);
            }
        }
    }

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

impl Drop for LiveSimController {
    /// Closing the workbench must never leave manifest.json lying: request
    /// a stop, unblock a pre-start wait, and join so the runner seals the
    /// manifest (Interrupted) before the process exits.
    fn drop(&mut self) {
        self.control.request_stop();
        let _ = self.command_tx.send(SimCommand::Quit);
        self.join();
    }
}
