//! Phase 6 exit criteria (v4 §13): exactly one CSV per run; manifest.json
//! reflects current status even with zero event-channel listeners; binary
//! formats round-trip.

use hg_output::{
    AppendLogReader, AppendLogWriter, CsvWriter, RunManifest, RunState, SnapshotReader,
    SnapshotWriter,
};
use serde::Serialize;

#[derive(Serialize)]
struct Row {
    step: u64,
    value: f64,
    tag: Option<String>,
}

#[test]
fn csv_writer_is_generic_and_writes_header_from_field_names() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("metrics.csv");
    let mut w: CsvWriter<Row> = CsvWriter::create(&path, 1, "run_x", 42, "t0").unwrap();
    w.write_row(&Row { step: 0, value: 1.5, tag: None }).unwrap();
    w.write_row(&Row { step: 1, value: 2.5, tag: Some("a".into()) }).unwrap();
    w.flush().unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let mut lines = content.lines();
    assert!(lines.next().unwrap().starts_with("#hypersim_csv_v=1,run_id=run_x,seed=42"));
    assert_eq!(lines.next().unwrap(), "step,value,tag");
    assert_eq!(lines.next().unwrap(), "0,1.5,");
    assert_eq!(lines.next().unwrap(), "1,2.5,a");

    // Exactly one CSV in the run dir.
    let csvs = std::fs::read_dir(dir.path())
        .unwrap()
        .filter(|e| {
            e.as_ref().unwrap().path().extension().map(|x| x == "csv").unwrap_or(false)
        })
        .count();
    assert_eq!(csvs, 1);
}

#[test]
fn manifest_transitions_are_durable_with_zero_listeners() {
    let dir = tempfile::tempdir().unwrap();
    let mut m = RunManifest::default();
    m.run_id = "r1".into();
    m.experiment_name = "exp".into();
    m.seed = 7;
    m.max_steps = 100;
    let mut m = m.create(dir.path()).unwrap();

    // Created is already on disk.
    assert_eq!(RunManifest::load(dir.path()).unwrap().state(), RunState::Created);

    m.start().unwrap();
    assert_eq!(RunManifest::load(dir.path()).unwrap().state(), RunState::Running);

    m.pause().unwrap();
    assert_eq!(RunManifest::load(dir.path()).unwrap().state(), RunState::Paused);

    m.resume().unwrap();
    m.seal_completed(55).unwrap();
    let loaded = RunManifest::load(dir.path()).unwrap();
    assert_eq!(loaded.state(), RunState::Completed);
    assert_eq!(loaded.final_step, 55);
    assert!(loaded.end_time.is_some());
}

#[test]
fn invalid_transitions_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let mut m = RunManifest::default().create(dir.path()).unwrap();
    assert!(m.pause().is_err()); // Created -> Paused invalid
    m.start().unwrap();
    m.seal_completed(1).unwrap();
    assert!(m.resume().is_err()); // Completed -> Running invalid
    assert!(m.seal_interrupted(2).is_err()); // Completed -> Interrupted invalid
}

#[test]
fn run_state_u8_roundtrip() {
    for s in [
        RunState::Created,
        RunState::Running,
        RunState::Paused,
        RunState::Interrupted,
        RunState::Completed,
    ] {
        assert_eq!(RunState::from_u8(s.as_u8()), s);
    }
}

#[test]
fn snapshot_and_append_log_roundtrip_bytes() {
    let dir = tempfile::tempdir().unwrap();

    let mut sw = SnapshotWriter::new(&dir.path().join("snapshots")).unwrap();
    let payload = b"opaque pre-serialized snapshot bytes";
    let path = sw.write_snapshot(1234, payload).unwrap();
    let (step, back) = SnapshotReader::read_snapshot(&path).unwrap();
    assert_eq!(step, 1234);
    assert_eq!(back, payload);

    let log_path = dir.path().join("events.hgal");
    let mut lw = AppendLogWriter::create(&log_path).unwrap();
    lw.append(b"first").unwrap();
    lw.append(b"second record").unwrap();
    lw.flush().unwrap();
    let mut lr = AppendLogReader::open(&log_path).unwrap();
    assert_eq!(lr.next_record().unwrap().unwrap(), b"first");
    assert_eq!(lr.next_record().unwrap().unwrap(), b"second record");
    assert!(lr.next_record().unwrap().is_none());
}
