//! hg-output: CSV writing, the run manifest, run-state transitions, and
//! snapshot/append-log binary formats.
//!
//! This crate has **no internal dependencies** (v4 §2.2) — deliberate, not
//! an oversight. `CsvWriter<T>` never needs to know `MetricsRow` exists;
//! `RunState`/`RunManifest` are defined here; `SnapshotWriter`/
//! `AppendLogWriter` accept pre-serialized `&[u8]`. The orchestrating crate
//! (`hg-sim-core`) supplies concrete, typed values in this crate's plain
//! vocabulary.

pub mod csv_writer;
pub mod manifest;
pub mod binlog;

pub use csv_writer::CsvWriter;
pub use manifest::{RunManifest, RunState, ManifestError};
pub use binlog::{AppendLogWriter, AppendLogReader, SnapshotWriter, SnapshotReader};
