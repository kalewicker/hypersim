//! Snapshot and append-log binary formats (docs/schema_versions.md).
//!
//! Writers accept pre-serialized `&[u8]` — they never touch
//! `HypergraphStore` or any hg-core type (v4 §2.2). The caller serializes
//! whatever a "snapshot" conceptually is before handing over the bytes.

use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

const SNAP_MAGIC: &[u8; 4] = b"HGSS";
// v2 (2026-07-10): node properties are now tagged on disk. v1 snapshots
// that contained properties are unrecoverable in principle — untagged
// bincode wrote no type discriminant — so version mismatch warns and is
// treated as absent (docs/schema_versions.md).
const SNAP_VERSION: u16 = 2;
const LOG_MAGIC: &[u8; 4] = b"HGAL";
const LOG_VERSION: u16 = 1;

pub struct SnapshotWriter {
    dir: PathBuf,
}

impl SnapshotWriter {
    pub fn new(dir: &Path) -> std::io::Result<Self> {
        fs::create_dir_all(dir)?;
        Ok(Self { dir: dir.to_path_buf() })
    }

    pub fn write_snapshot(&mut self, step: u64, payload: &[u8]) -> std::io::Result<PathBuf> {
        let path = self.dir.join(format!("snap_{step:012}.bin"));
        let mut w = BufWriter::new(File::create(&path)?);
        w.write_all(SNAP_MAGIC)?;
        w.write_all(&SNAP_VERSION.to_le_bytes())?;
        w.write_all(&step.to_le_bytes())?;
        w.write_all(&(payload.len() as u64).to_le_bytes())?;
        w.write_all(payload)?;
        w.flush()?;
        Ok(path)
    }
}

pub struct SnapshotReader;

impl SnapshotReader {
    /// Returns (step, payload).
    pub fn read_snapshot(path: &Path) -> std::io::Result<(u64, Vec<u8>)> {
        let mut r = BufReader::new(File::open(path)?);
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if &magic != SNAP_MAGIC {
            return Err(std::io::Error::other("bad snapshot magic"));
        }
        let mut v = [0u8; 2];
        r.read_exact(&mut v)?;
        let version = u16::from_le_bytes(v);
        if version != SNAP_VERSION {
            // Migration policy: warn and treat as absent. Prior-version
            // snapshots are not auto-migrated (v1 property bytes carry no
            // type tags and are unrecoverable in principle).
            tracing::warn!(
                found = version,
                expected = SNAP_VERSION,
                "snapshot version mismatch — treating as absent (not auto-migrated)"
            );
            return Err(std::io::Error::other(format!(
                "unsupported snapshot version {version} (expected {SNAP_VERSION})"
            )));
        }
        let mut step_b = [0u8; 8];
        r.read_exact(&mut step_b)?;
        let mut len_b = [0u8; 8];
        r.read_exact(&mut len_b)?;
        let mut payload = vec![0u8; u64::from_le_bytes(len_b) as usize];
        r.read_exact(&mut payload)?;
        Ok((u64::from_le_bytes(step_b), payload))
    }
}

pub struct AppendLogWriter {
    inner: BufWriter<File>,
}

impl AppendLogWriter {
    pub fn create(path: &Path) -> std::io::Result<Self> {
        let mut inner = BufWriter::new(File::create(path)?);
        inner.write_all(LOG_MAGIC)?;
        inner.write_all(&LOG_VERSION.to_le_bytes())?;
        inner.flush()?;
        Ok(Self { inner })
    }

    pub fn append(&mut self, record: &[u8]) -> std::io::Result<()> {
        self.inner.write_all(&(record.len() as u32).to_le_bytes())?;
        self.inner.write_all(record)?;
        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub struct AppendLogReader {
    inner: BufReader<File>,
}

impl AppendLogReader {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let mut inner = BufReader::new(File::open(path)?);
        let mut magic = [0u8; 4];
        inner.read_exact(&mut magic)?;
        if &magic != LOG_MAGIC {
            return Err(std::io::Error::other("bad append-log magic"));
        }
        let mut v = [0u8; 2];
        inner.read_exact(&mut v)?;
        if u16::from_le_bytes(v) != LOG_VERSION {
            return Err(std::io::Error::other("unsupported append-log version"));
        }
        Ok(Self { inner })
    }

    /// Read the next record; Ok(None) at clean EOF.
    pub fn next_record(&mut self) -> std::io::Result<Option<Vec<u8>>> {
        let mut len_b = [0u8; 4];
        match self.inner.read_exact(&mut len_b) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        }
        let mut record = vec![0u8; u32::from_le_bytes(len_b) as usize];
        self.inner.read_exact(&mut record)?;
        Ok(Some(record))
    }
}
