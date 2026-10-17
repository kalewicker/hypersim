//! Phase 9 exit: MockSimController serves a complete fixture with no
//! dependency beyond hg-app-api itself; event disciplines hold.

use hg_app_api::{
    events::{send_lossy, send_status},
    EventChannels, MockSimController, SimController, SimEvent, SimStatusEvent,
};

#[test]
fn mock_serves_a_complete_fixture() {
    let mut mock = MockSimController::new();
    mock.start();

    let shared = mock.shared_state();
    let hist = shared.metrics_history.read();
    assert_eq!(hist.rows.len(), mock.window_count());
    assert!(hist.rows.iter().all(|r| r.regime.is_some()));

    let eval = shared.eval_state.read();
    assert_eq!(eval.regime_history.len(), mock.window_count());

    let kg = shared.knowledge_graph.read();
    assert_eq!(kg.tasks.len(), 1);
    assert_eq!(kg.constructors.len(), 1);
    assert_eq!(kg.claims.len(), 1);

    assert!(shared.progress.step() > 0);

    // Durable status: run completion arrived.
    let mut completed = false;
    while let Some(ev) = mock.try_recv_status() {
        if matches!(ev, SimStatusEvent::RunCompleted { .. }) {
            completed = true;
        }
    }
    assert!(completed);
}

#[test]
fn lossy_channel_drops_on_full_without_panicking() {
    let ch = EventChannels::new();
    // Overfill: bound is 256; sends 300 — must not panic or block.
    for w in 0..300u64 {
        send_lossy(
            &ch.event_tx,
            SimEvent::WindowComplete {
                window: w,
                metrics: Box::default(),
            },
        );
    }
    let mut drained = 0;
    while ch.event_rx.try_recv().is_ok() {
        drained += 1;
    }
    assert_eq!(drained, 256, "bounded(256), drop-on-full");
}

#[test]
fn status_send_with_no_listener_is_a_normal_condition() {
    let ch = EventChannels::new();
    drop(ch.status_rx); // headless CLI: no GUI consuming status events
    // Must not panic — a disconnected receiver is expected (v4 §9.1).
    send_status(&ch.status_tx, SimStatusEvent::ScienceUnlocked);
    send_status(
        &ch.status_tx,
        SimStatusEvent::ErrorOccurred {
            message: "x".into(),
        },
    );
}

#[test]
fn progress_atomics_roundtrip() {
    let p = hg_app_api::SimProgress::new(1000);
    p.set_step(42);
    p.set_sim_hz(123.5);
    p.set_run_state_u8(2);
    assert_eq!(p.step(), 42);
    assert_eq!(p.max_steps(), 1000);
    assert_eq!(p.sim_hz(), 123.5);
    assert_eq!(p.run_state_u8(), 2);
}
