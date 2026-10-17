//! Versioned executor journal publication; same durable wire protocol.
use super::admission::sha;
use serde_json::{json, Value};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
const PROFILE: &str = "cumulative-durable-journal-readiness-v1";
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub struct Writer {
    dir: PathBuf,
    sequence: usize,
    head: Option<String>,
    bytes: usize,
    limit: usize,
}
impl Writer {
    pub fn new(dir: &Path, limit: usize) -> Result<Self, String> {
        fs::create_dir(dir).map_err(error)?;
        File::open(dir.parent().ok_or("journal parent")?)
            .map_err(error)?
            .sync_all()
            .map_err(error)?;
        Ok(Self {
            dir: dir.to_path_buf(),
            sequence: 0,
            head: None,
            bytes: 0,
            limit,
        })
    }
    pub fn publish(&mut self, value: Value, torn: bool) -> Result<(), String> {
        let v = json!({"profile":PROFILE,"sequence":self.sequence,"previous_sha256":self.head,"value":value});
        let mut bytes = serde_json::to_vec(&v).map_err(error)?;
        bytes.push(b'\n');
        if self.sequence >= 2048 || self.bytes + bytes.len() > self.limit {
            return Err("journal storage/record cap".into());
        }
        let staging = self.dir.join(format!(".staging-{:08}.json", self.sequence));
        let target = self.dir.join(format!("{:08}.json", self.sequence));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)
            .map_err(error)?;
        if torn {
            file.write_all(&bytes[..bytes.len() / 2]).map_err(error)?;
            file.sync_all().map_err(error)?;
            File::open(&self.dir)
                .map_err(error)?
                .sync_all()
                .map_err(error)?;
            return Err("declared torn-result interruption".into());
        }
        file.write_all(&bytes).map_err(error)?;
        file.sync_all().map_err(error)?;
        fs::hard_link(&staging, &target).map_err(error)?;
        File::open(&self.dir)
            .map_err(error)?
            .sync_all()
            .map_err(error)?;
        fs::remove_file(&staging).map_err(error)?;
        File::open(&self.dir)
            .map_err(error)?
            .sync_all()
            .map_err(error)?;
        self.head = Some(sha(&bytes));
        self.sequence += 1;
        self.bytes += bytes.len();
        Ok(())
    }
}
