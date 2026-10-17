//! Versioned live execution state and externally committed evidence cursor.
use super::{admission::*, family::*, mutation::MutationRecord};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
pub const STREAM_PROFILE: &str = "evolving-minds-d0-streaming-v1";
pub const LIVE_LIMIT: usize = 2 * 1024 * 1024;
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveAncestor {
    pub founder: i64,
    pub depth: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveAncestry {
    pub members: BTreeMap<i64, LiveAncestor>,
    pub maximum_depth: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceCursor {
    pub segment_sha256: String,
    pub tick: u64,
    pub step: u64,
    pub local_ticks: u64,
    pub offset: u64,
    pub log_sha256: String,
    pub previous_commit_sha256: Option<String>,
}
impl EvidenceCursor {
    pub fn validate(&self) -> Result<(), String> {
        let hash = |s: &str| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        };
        if self.tick > 64
            || self.step != self.tick * 46
            || self.local_ticks > self.tick
            || self.offset > 16 * 1024 * 1024
            || !hash(&self.segment_sha256)
            || !hash(&self.log_sha256)
            || self
                .previous_commit_sha256
                .as_ref()
                .is_some_and(|h| !hash(h))
        {
            Err("stream evidence cursor".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TickBundle {
    pub profile: String,
    pub tick: u64,
    pub entries: Vec<FamilyStep>,
    pub row: FamilyRow,
    pub round: AdmissionRound,
    pub records: Vec<MutationRecord>,
    pub lineage_delta: Vec<FamilyBirth>,
}
