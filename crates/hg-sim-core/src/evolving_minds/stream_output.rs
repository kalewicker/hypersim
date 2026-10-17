//! Append-only evidence with immutable, hash-bound quiescent commits.
use super::{
    admission::*, descendant::*, family::*, funded_admission::*, handoff::*, retained::*,
    source_inventory::SOURCES, stream::*, transfer_environment::*,
};
use hg_output::RunManifest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StreamFault {
    Normal,
    CleanStop,
    TornPayload,
    TornCheckpoint,
    BeforeCommit,
    AfterCommit,
    Quota,
}
impl StreamFault {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::CleanStop => "clean_stop",
            Self::TornPayload => "torn_payload",
            Self::TornCheckpoint => "torn_checkpoint",
            Self::BeforeCommit => "before_commit",
            Self::AfterCommit => "after_commit",
            Self::Quota => "quota",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParentCommit {
    pub directory: String,
    pub commit: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentHeader {
    pub profile: String,
    pub dynamics_profile: String,
    pub recipe: AdmissionRecipe,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funded_config: Option<FundedAdmissionConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff_config: Option<HandoffConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descendant_config: Option<DescendantConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_environment: Option<TransferEnvironment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_stress_policy: Option<String>,
    pub start_tick: u64,
    pub parent: Option<ParentCommit>,
    pub fault: StreamFault,
    pub payload_limit: u64,
    pub law_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishedCommit {
    pub profile: String,
    pub cursor: EvidenceCursor,
    pub checkpoint: String,
    pub checkpoint_sha256: String,
    pub attestation: String,
    pub attestation_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StreamResources {
    pub advance_seconds: f64,
    pub checkpoint_seconds: f64,
    pub io_seconds: f64,
    pub resume_seconds: f64,
    pub payload_bytes: u64,
    pub total_bytes: u64,
    pub retained_entries_peak: usize,
    pub retained_rows_peak: usize,
    pub retained_rounds_peak: usize,
    pub retained_requests_peak: usize,
    pub retained_lineage_peak: usize,
    pub drained_history_empty: bool,
    pub checkpoints: Vec<(u64, usize)>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentReport {
    pub profile: String,
    pub recipe: AdmissionRecipe,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funded_config: Option<FundedAdmissionConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff_config: Option<HandoffConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descendant_config: Option<DescendantConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_environment: Option<TransferEnvironment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_stress_policy: Option<String>,
    pub fault: StreamFault,
    pub start_tick: u64,
    pub native_final_tick: u64,
    pub committed_final_tick: u64,
    pub state: String,
    pub stop_reason: Option<String>,
    pub resources: StreamResources,
}
fn bytes<T: Serialize>(x: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(x).map_err(|e| e.to_string())
}
fn put(p: &Path, b: &[u8]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
        .map_err(|e| e.to_string())?;
    f.write_all(b).map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())
}
fn sync_dir(p: &Path) -> Result<(), String> {
    File::open(p)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}
fn read(p: &Path, cap: usize) -> Result<Vec<u8>, String> {
    let m = fs::symlink_metadata(p).map_err(|e| e.to_string())?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > cap as u64 {
        return Err("stream evidence file type/ceiling".into());
    }
    fs::read(p).map_err(|e| e.to_string())
}
fn size(p: &Path) -> Result<u64, String> {
    let mut n = 0;
    for e in fs::read_dir(p).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let m = fs::symlink_metadata(e.path()).map_err(|e| e.to_string())?;
        if m.file_type().is_symlink() {
            return Err("stream symlink evidence".into());
        }
        n += if m.is_dir() {
            size(&e.path())?
        } else {
            m.len()
        };
    }
    Ok(n)
}
fn flags(ok: bool, reason: Option<&str>) -> serde_json::Value {
    serde_json::json!({"profile":STREAM_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"reproductive_fairness_accepted":false,"stop_reason":reason})
}
fn meta(name: &str) -> RunManifest {
    let mut m = RunManifest::default();
    m.run_id = name.into();
    m.experiment_name = STREAM_PROFILE.into();
    m.seed = 17;
    m.max_steps = 2944;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "46 native passes; immutable committed evidence segments".into();
    m.build_features = vec!["evolving-minds".into()];
    m.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    m.rustc_version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    m
}
fn commit_name(tick: u64) -> String {
    format!("commit_{tick:04}.json")
}
fn publish(
    p: &Path,
    s: &FamilySimulation,
    cursor: EvidenceCursor,
    fault: Option<&StreamFault>,
    resources: &mut StreamResources,
) -> Result<Option<PublishedCommit>, String> {
    let start = Instant::now();
    let cp = s.live_checkpoint(cursor.clone())?;
    let att = bytes(&s.live_attestation(cursor.clone())?)?;
    resources.checkpoint_seconds += start.elapsed().as_secs_f64();
    if att.len() > LIVE_LIMIT {
        return Err("stream attestation ceiling".into());
    }
    let checkpoint = format!("checkpoint_{:04}.bin", cursor.tick);
    let attestation = format!("state_{:04}.json", cursor.tick);
    let io = Instant::now();
    if fault == Some(&StreamFault::TornCheckpoint) {
        put(
            &p.join(format!("{checkpoint}.pending")),
            &cp[..cp.len() / 2],
        )?;
        resources.io_seconds += io.elapsed().as_secs_f64();
        return Ok(None);
    }
    put(&p.join(&checkpoint), &cp)?;
    put(&p.join(&attestation), &att)?;
    let marker = PublishedCommit {
        profile: s.stream_profile().into(),
        cursor,
        checkpoint,
        checkpoint_sha256: sha256(&cp),
        attestation,
        attestation_sha256: sha256(&att),
    };
    let marker_bytes = bytes(&marker)?;
    let name = commit_name(marker.cursor.tick);
    let temporary = p.join(format!("{name}.pending"));
    if fault == Some(&StreamFault::BeforeCommit) {
        put(&temporary, &marker_bytes[..marker_bytes.len() / 2])?;
        resources.io_seconds += io.elapsed().as_secs_f64();
        return Ok(None);
    }
    put(&temporary, &marker_bytes)?;
    fs::hard_link(&temporary, p.join(name))
        .map_err(|e| format!("exclusive commit publication: {e}"))?;
    sync_dir(p)?;
    fs::remove_file(temporary).map_err(|e| e.to_string())?;
    sync_dir(p)?;
    resources.checkpoints.push((marker.cursor.tick, cp.len()));
    resources.io_seconds += io.elapsed().as_secs_f64();
    Ok(Some(marker))
}
struct Loaded {
    header: SegmentHeader,
    marker: PublishedCommit,
    marker_name: String,
    marker_sha256: String,
    checkpoint: Vec<u8>,
    attestation: serde_json::Value,
}
fn load_cut(p: &Path, cut: Option<&str>, seen: &mut BTreeSet<PathBuf>) -> Result<Loaded, String> {
    if seen.len() >= 8
        || fs::symlink_metadata(p)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
    {
        return Err("stream chain depth/path".into());
    }
    let absolute = p.canonicalize().map_err(|e| e.to_string())?;
    if !seen.insert(absolute) {
        return Err("stream parent cycle".into());
    }
    if size(p)? > 32 * 1024 * 1024 {
        return Err("stream segment output ceiling".into());
    }
    for e in fs::read_dir(p).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        if !e.file_type().map_err(|e| e.to_string())?.is_file() {
            return Err("stream symlink evidence".into());
        }
    }
    let raw = read(&p.join("segment.json"), 65536)?;
    let header: SegmentHeader = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    let (profile, dynamics, laws) = if let Some(c) = &header.funded_config {
        c.validate()?;
        if header.recipe != c.coordinate_recipe() {
            return Err("funded segment coordinates".into());
        }
        (
            if header.mutation_stress_policy.is_some() {
                MUTATION_STRESS_STREAM_PROFILE
            } else if header.transfer_environment.is_some() {
                TRANSFER_ENV_STREAM_PROFILE
            } else if header.descendant_config.is_some() {
                DESCENDANT_STREAM_PROFILE
            } else if header.handoff_config.is_some() {
                HANDOFF_STREAM_PROFILE
            } else if header.retained_policy.is_some() {
                RETAINED_STREAM_PROFILE
            } else {
                FUNDED_STREAM_PROFILE
            },
            if header.mutation_stress_policy.is_some() {
                MUTATION_STRESS_PROFILE
            } else if header.transfer_environment.is_some() {
                TRANSFER_ENV_PROFILE
            } else if header.descendant_config.is_some() {
                DESCENDANT_PROFILE
            } else if header.handoff_config.is_some() {
                HANDOFF_PROFILE
            } else if header.retained_policy.is_some() {
                RETAINED_PROFILE
            } else {
                FUNDED_ADMISSION_PROFILE
            },
            if let Some(e) = &header.transfer_environment {
                if header.mutation_stress_policy.is_some() {
                    e.validate_stress(c)?;
                } else {
                    e.validate(c)?;
                }
                e.law_text(c)?
            } else {
                c.law_text()?
            },
        )
    } else {
        header.recipe.validate()?;
        (
            STREAM_PROFILE,
            ADMISSION_PROFILE,
            admission_law_text(&super::choice::ChoiceSchedule::new(
                17,
                super::choice::CueMode::Informative,
            )),
        )
    };
    if header
        .mutation_stress_policy
        .as_deref()
        .is_some_and(|p| p != MUTATION_STRESS_POLICY || header.transfer_environment.is_none())
    {
        return Err("stress header policy binding".into());
    }
    if header.transfer_environment.is_some()
        && (header.funded_config.is_none()
            || header.handoff_config.is_some()
            || header.descendant_config.is_some()
            || header.retained_policy.is_some())
    {
        return Err("transfer header configuration conflict".into());
    }
    if let Some(d) = &header.descendant_config {
        d.validate(
            header
                .funded_config
                .as_ref()
                .ok_or("descendant funded header")?,
        )?;
        if header.handoff_config.is_some() || header.retained_policy.is_some() {
            return Err("descendant header policy conflict".into());
        }
    }
    if let Some(h) = &header.handoff_config {
        h.validate(
            header
                .funded_config
                .as_ref()
                .ok_or("handoff funded header")?,
        )?;
        if header.retained_policy.is_some() {
            return Err("conflicting header policies".into());
        }
    }
    if header
        .retained_policy
        .as_ref()
        .is_some_and(|p| p != RETAINED_POLICY || header.funded_config.is_none())
    {
        return Err("retained segment policy".into());
    }
    if header.profile != profile
        || header.dynamics_profile != dynamics
        || header.start_tick > 64
        || header.start_tick % 8 != 0
        || header.law_sha256 != sha256(laws.as_bytes())
        || header.payload_limit
            != if header.fault == StreamFault::Quota {
                262144
            } else {
                16 * 1024 * 1024
            }
    {
        return Err("stream segment header".into());
    }
    let mut parent_state = None;
    if let Some(parent) = &header.parent {
        let loaded = load_cut(Path::new(&parent.directory), Some(&parent.commit), seen)?;
        if loaded.marker_sha256 != parent.sha256
            || loaded.marker.cursor.tick != header.start_tick
            || loaded.header.recipe != header.recipe
            || loaded.header.retained_policy != header.retained_policy
            || loaded.header.handoff_config != header.handoff_config
            || loaded.header.descendant_config != header.descendant_config
            || loaded.header.transfer_environment != header.transfer_environment
            || loaded.header.mutation_stress_policy != header.mutation_stress_policy
            || loaded.header.funded_config != header.funded_config
            || loaded.header.profile != header.profile
        {
            return Err("stream parent linkage".into());
        }
        parent_state = Some(loaded.attestation);
    } else if header.start_tick != 0 {
        return Err("stream missing parent".into());
    }
    let mut names = vec![];
    for e in fs::read_dir(p).map_err(|e| e.to_string())? {
        let name = e
            .map_err(|e| e.to_string())?
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name.starts_with("commit_") && name.ends_with(".json") {
            if name.len() != 16 || !name.is_ascii() || name[7..11].parse::<u64>().ok().is_none() {
                return Err("stream commit filename".into());
            }
            names.push(name)
        }
    }
    names.sort();
    let target = cut
        .map(str::to_string)
        .or_else(|| names.last().cloned())
        .ok_or("stream no published commit")?;
    if !names.contains(&target) {
        return Err("stream missing selected commit".into());
    }
    names.retain(|n| n <= &target);
    let log = read(&p.join("ticks.jsonl"), 16 * 1024 * 1024)?;
    if log.len() as u64 > header.payload_limit {
        return Err("stream payload ceiling".into());
    }
    let mut previous = None;
    let mut last = None;
    let mut last_tick = header.start_tick;
    for name in names {
        let raw_marker = read(&p.join(&name), 65536)?;
        let marker: PublishedCommit =
            serde_json::from_slice(&raw_marker).map_err(|e| e.to_string())?;
        marker.cursor.validate()?;
        let c = &marker.cursor;
        if marker.profile != header.profile
            || c.segment_sha256 != sha256(&raw)
            || c.previous_commit_sha256 != previous
            || c.tick < header.start_tick
            || c.local_ticks != c.tick - header.start_tick
            || c.offset > log.len() as u64
            || name != commit_name(c.tick)
            || marker.checkpoint != format!("checkpoint_{:04}.bin", c.tick)
            || marker.attestation != format!("state_{:04}.json", c.tick)
            || sha256(&log[..c.offset as usize]) != c.log_sha256
            || last.is_none() && c.tick != header.start_tick
            || last.is_some() && c.tick != last_tick + 8
        {
            return Err("stream commit cursor/chain/prefix".into());
        }
        let prefix = &log[..c.offset as usize];
        if !prefix.is_empty() && prefix.last() != Some(&b'\n') {
            return Err("stream incomplete committed record".into());
        }
        let lines: Vec<_> = if prefix.is_empty() {
            vec![]
        } else {
            prefix[..prefix.len() - 1].split(|b| *b == b'\n').collect()
        };
        if lines.len() != c.local_ticks as usize {
            return Err("stream committed tick count".into());
        }
        for (i, line) in lines.iter().enumerate() {
            if line.len() > LIVE_LIMIT {
                return Err("stream tick payload ceiling".into());
            }
            let b: TickBundle = serde_json::from_slice(line).map_err(|e| e.to_string())?;
            let tick = header.start_tick + i as u64 + 1;
            if b.profile != header.profile
                || b.tick != tick
                || b.row.tick != tick
                || b.row.engine_step != 46 * tick
                || b.entries.len() != 46
                || b.round.tick != tick
                || b.entries.iter().enumerate().any(|(pass, e)| {
                    e.tick != tick
                        || e.pass as usize != pass
                        || e.step != (tick - 1) * 46 + pass as u64 + 1
                })
            {
                return Err("stream committed bundle clocks".into());
            }
        }
        let cp = read(&p.join(&marker.checkpoint), LIVE_LIMIT + 24)?;
        let att = read(&p.join(&marker.attestation), LIVE_LIMIT)?;
        if sha256(&cp) != marker.checkpoint_sha256 || sha256(&att) != marker.attestation_sha256 {
            return Err("stream published checkpoint/state hash".into());
        }
        let attestation: serde_json::Value =
            serde_json::from_slice(&att).map_err(|e| e.to_string())?;
        if attestation["native"]["profile"] != header.profile
            || attestation["native"]["funded_config"]
                != serde_json::to_value(&header.funded_config).map_err(|e| e.to_string())?
        {
            return Err("stream checkpoint header parameters".into());
        }
        if attestation["native"]["transfer_environment"]
            != serde_json::to_value(&header.transfer_environment).map_err(|e| e.to_string())?
        {
            return Err("transfer checkpoint header binding".into());
        }
        if attestation["native"]["mutation_stress_policy"]
            != serde_json::to_value(&header.mutation_stress_policy).map_err(|e| e.to_string())?
        {
            return Err("stress checkpoint header binding".into());
        }
        if header.retained_policy.as_deref() != attestation["native"]["retained"]["policy"].as_str()
        {
            return Err("retained checkpoint header policy".into());
        }

        if attestation["native"]["descendant"]["config"]
            != serde_json::to_value(&header.descendant_config).map_err(|e| e.to_string())?
        {
            return Err("descendant checkpoint header binding".into());
        }
        if attestation["native"]["handoff_config"]
            != serde_json::to_value(&header.handoff_config).map_err(|e| e.to_string())?
        {
            return Err("handoff checkpoint header binding".into());
        }
        if last.is_none() {
            if let Some(parent) = &parent_state {
                let mut a = parent["native"].clone();
                let mut b = attestation["native"].clone();
                a.as_object_mut()
                    .ok_or("stream parent native state")?
                    .remove("cursor");
                b.as_object_mut()
                    .ok_or("stream child native state")?
                    .remove("cursor");
                if a != b || parent["frame"] != attestation["frame"] {
                    return Err("stream child native continuity".into());
                }
            }
        }
        last_tick = c.tick;
        previous = Some(sha256(&raw_marker));
        last = Some(Loaded {
            header: header.clone(),
            marker,
            marker_name: name,
            marker_sha256: previous.clone().unwrap(),
            checkpoint: cp,
            attestation,
        });
    }
    last.ok_or_else(|| "stream empty commit inventory".into())
}
fn execute(
    p: &Path,
    mut s: FamilySimulation,
    header: SegmentHeader,
    resume_seconds: f64,
) -> Result<SegmentReport, String> {
    fs::create_dir(p).map_err(|e| format!("exclusive stream directory: {e}"))?;
    if s.funded_config() != header.funded_config.as_ref()
        || s.stream_profile() != header.profile
        || s.retained_state().map(|r| &r.policy) != header.retained_policy.as_ref()
        || s.handoff_config() != header.handoff_config.as_ref()
        || s.descendant_config() != header.descendant_config.as_ref()
        || s.transfer_environment() != header.transfer_environment.as_ref()
        || s.mutation_stress_policy() != header.mutation_stress_policy.as_deref()
    {
        return Err("stream execution parameter binding".into());
    }
    let mut manifest = meta(&header.recipe.name());
    manifest.experiment_name = header.profile.clone();
    manifest.seed = header.funded_config.as_ref().map_or(17, |c| c.seed);
    let mut m = manifest.create(p).map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    m.final_step = header.start_tick * 46;
    m.persist().map_err(|e| e.to_string())?;
    let mut resources = StreamResources {
        resume_seconds,
        drained_history_empty: true,
        ..Default::default()
    };
    let mut last: Option<PublishedCommit> = None;
    let mut stop_reason = None;
    let mut native_final_tick = header.start_tick;
    let result: Result<(), String> = (|| {
        let header_bytes = bytes(&header)?;
        put(&p.join("segment.json"), &header_bytes)?;
        put(&p.join("initial.json"), &bytes(&s.final_frame())?)?;
        put(&p.join("initial_lineage.json"), &bytes(&s.lineage)?)?;
        put(&p.join("schedule.json"), &bytes(&s.schedule)?)?;
        put(&p.join("rules.toml"), s.law_text().as_bytes())?;
        let mut log = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(p.join("ticks.jsonl"))
            .map_err(|e| e.to_string())?;
        log.sync_all().map_err(|e| e.to_string())?;
        let mut digest = Sha256::new();
        let mut offset = 0;
        let cursor = EvidenceCursor {
            segment_sha256: sha256(&header_bytes),
            tick: header.start_tick,
            step: header.start_tick * 46,
            local_ticks: 0,
            offset: 0,
            log_sha256: sha256(b""),
            previous_commit_sha256: None,
        };
        last = publish(p, &s, cursor, None, &mut resources)?;
        let start = Instant::now();
        while s.completed_tick() < 64 {
            if start.elapsed().as_secs() >= 120 {
                return Err("stream advancing wall ceiling".into());
            }
            let advance = Instant::now();
            s.advance_tick()?;
            resources.advance_seconds += advance.elapsed().as_secs_f64();
            native_final_tick = s.completed_tick();
            resources.retained_entries_peak = resources.retained_entries_peak.max(s.journal.len());
            resources.retained_rows_peak = resources.retained_rows_peak.max(s.rows.len());
            resources.retained_lineage_peak = resources.retained_lineage_peak.max(s.lineage.len());
            let a = s.admission.as_ref().unwrap();
            resources.retained_rounds_peak = resources.retained_rounds_peak.max(a.rounds.len());
            resources.retained_requests_peak =
                resources.retained_requests_peak.max(a.records.len());
            let bundle = s.drain_stream_tick()?;
            resources.drained_history_empty &= s.rows.is_empty()
                && s.journal.is_empty()
                && s.admission.as_ref().unwrap().rounds.is_empty()
                && s.admission.as_ref().unwrap().records.is_empty();
            let mut data = bytes(&bundle)?;
            data.push(b'\n');
            if data.len() > LIVE_LIMIT {
                return Err("stream tick payload ceiling".into());
            }
            if offset + data.len() as u64 > header.payload_limit {
                stop_reason = Some("declared segment payload quota".to_string());
                break;
            }
            let io = Instant::now();
            if header.fault == StreamFault::TornPayload && native_final_tick == 9 {
                log.write_all(&data[..data.len() / 2])
                    .map_err(|e| e.to_string())?;
                log.sync_all().map_err(|e| e.to_string())?;
                resources.io_seconds += io.elapsed().as_secs_f64();
                stop_reason = Some("injected half-written tick-9 payload".into());
                break;
            }
            log.write_all(&data).map_err(|e| e.to_string())?;
            digest.update(&data);
            offset += data.len() as u64;
            resources.io_seconds += io.elapsed().as_secs_f64();
            if native_final_tick % 8 == 0 {
                log.flush().map_err(|e| e.to_string())?;
                log.sync_all().map_err(|e| e.to_string())?;
                let previous = last
                    .as_ref()
                    .map(|marker| {
                        read(&p.join(commit_name(marker.cursor.tick)), 65536).map(|b| sha256(&b))
                    })
                    .transpose()?;
                let cursor = EvidenceCursor {
                    segment_sha256: sha256(&header_bytes),
                    tick: native_final_tick,
                    step: native_final_tick * 46,
                    local_ticks: native_final_tick - header.start_tick,
                    offset,
                    log_sha256: format!("{:x}", digest.clone().finalize()),
                    previous_commit_sha256: previous,
                };
                let fault = if native_final_tick == 16
                    && [StreamFault::TornCheckpoint, StreamFault::BeforeCommit]
                        .contains(&header.fault)
                {
                    Some(&header.fault)
                } else {
                    None
                };
                let marker = publish(p, &s, cursor, fault, &mut resources)?;
                if marker.is_none() {
                    stop_reason = Some(format!("injected {} at tick 16", header.fault.name()));
                    break;
                }
                last = marker;
                if header.fault == StreamFault::AfterCommit && native_final_tick == 16 {
                    put(
                        &p.join("manifest_at_crash.json"),
                        &read(&p.join("manifest.json"), 65536)?,
                    )?;
                    stop_reason = Some(
                        "injected after published tick-16 commit before manifest progress".into(),
                    );
                    break;
                }
                m.final_step = native_final_tick * 46;
                m.persist().map_err(|e| e.to_string())?;
                if header.fault == StreamFault::CleanStop
                    && native_final_tick
                        == if header.handoff_config.is_some()
                            || header.descendant_config.is_some()
                            || header.transfer_environment.is_some()
                        {
                            24
                        } else {
                            8
                        }
                {
                    stop_reason = Some(format!(
                        "declared stop after committed tick {native_final_tick}"
                    ));
                    break;
                }
            }
            if size(p)? > 32 * 1024 * 1024 {
                return Err("stream segment output ceiling".into());
            }
        }
        Ok(())
    })();
    let end = last.as_ref().map_or(header.start_tick, |c| c.cursor.tick);
    if let Err(e) = &result {
        stop_reason = Some(e.clone())
    }
    let complete = result.is_ok() && stop_reason.is_none() && end == 64;
    if complete {
        m.seal_completed(end * 46).map_err(|e| e.to_string())?;
    } else {
        m.seal_interrupted(end * 46).map_err(|e| e.to_string())?;
    }
    put(
        &p.join("run_status.json"),
        &bytes(&{
            let mut f = flags(complete, stop_reason.as_deref());
            f["profile"] = header.profile.clone().into();
            f
        })?,
    )?;
    resources.payload_bytes = p
        .join("ticks.jsonl")
        .metadata()
        .map(|m| m.len())
        .unwrap_or(0);
    resources.total_bytes = size(p)?;
    let report = SegmentReport {
        profile: header.profile,
        recipe: header.recipe,
        funded_config: header.funded_config,
        retained_policy: header.retained_policy,
        handoff_config: header.handoff_config,
        descendant_config: header.descendant_config,
        transfer_environment: header.transfer_environment,
        mutation_stress_policy: header.mutation_stress_policy,
        fault: header.fault,
        start_tick: header.start_tick,
        native_final_tick,
        committed_final_tick: end,
        state: if complete { "Completed" } else { "Interrupted" }.into(),
        stop_reason,
        resources,
    };
    put(&p.join("segment_report.json"), &bytes(&report)?)?;
    result?;
    Ok(report)
}
pub fn write_stream_segment(
    p: &Path,
    recipe: AdmissionRecipe,
    fault: StreamFault,
) -> Result<SegmentReport, String> {
    let s = FamilySimulation::new_stream(recipe.clone())?;
    let header = SegmentHeader {
        profile: STREAM_PROFILE.into(),
        dynamics_profile: ADMISSION_PROFILE.into(),
        recipe,
        funded_config: None,
        retained_policy: None,
        handoff_config: None,
        descendant_config: None,
        transfer_environment: None,
        mutation_stress_policy: None,
        start_tick: 0,
        parent: None,
        payload_limit: if fault == StreamFault::Quota {
            262144
        } else {
            16 * 1024 * 1024
        },
        fault,
        law_sha256: sha256(admission_law_text(&s.schedule).as_bytes()),
    };
    execute(p, s, header, 0.0)
}
pub fn write_funded_stream_segment(
    p: &Path,
    config: FundedAdmissionConfig,
    fault: StreamFault,
) -> Result<SegmentReport, String> {
    config.validate()?;
    let s = FamilySimulation::new_funded_stream(config.clone())?;
    let header = SegmentHeader {
        profile: FUNDED_STREAM_PROFILE.into(),
        dynamics_profile: FUNDED_ADMISSION_PROFILE.into(),
        recipe: config.coordinate_recipe(),
        funded_config: Some(config),
        retained_policy: None,
        handoff_config: None,
        descendant_config: None,
        transfer_environment: None,
        mutation_stress_policy: None,
        start_tick: 0,
        parent: None,
        payload_limit: if fault == StreamFault::Quota {
            262144
        } else {
            16 * 1024 * 1024
        },
        fault,
        law_sha256: sha256(s.law_text().as_bytes()),
    };
    execute(p, s, header, 0.0)
}
pub fn write_transfer_stream_segment(
    p: &Path,
    config: FundedAdmissionConfig,
    environment: TransferEnvironment,
    fault: StreamFault,
) -> Result<SegmentReport, String> {
    let s = FamilySimulation::new_transfer_stream(config.clone(), environment.clone())?;
    let header = SegmentHeader {
        profile: TRANSFER_ENV_STREAM_PROFILE.into(),
        dynamics_profile: TRANSFER_ENV_PROFILE.into(),
        recipe: config.coordinate_recipe(),
        funded_config: Some(config),
        transfer_environment: Some(environment),
        mutation_stress_policy: None,
        retained_policy: None,
        handoff_config: None,
        descendant_config: None,
        start_tick: 0,
        parent: None,
        payload_limit: 16 * 1024 * 1024,
        law_sha256: sha256(s.law_text().as_bytes()),
        fault,
    };
    execute(p, s, header, 0.0)
}
pub fn write_mutation_stress_stream_segment(
    p: &Path,
    config: FundedAdmissionConfig,
    environment: TransferEnvironment,
    fault: StreamFault,
) -> Result<SegmentReport, String> {
    let s = FamilySimulation::new_mutation_stress_stream(config.clone(), environment.clone())?;
    let header = SegmentHeader {
        profile: MUTATION_STRESS_STREAM_PROFILE.into(),
        dynamics_profile: MUTATION_STRESS_PROFILE.into(),
        recipe: config.coordinate_recipe(),
        funded_config: Some(config),
        transfer_environment: Some(environment),
        mutation_stress_policy: Some(MUTATION_STRESS_POLICY.into()),
        retained_policy: None,
        handoff_config: None,
        descendant_config: None,
        start_tick: 0,
        parent: None,
        payload_limit: 16 * 1024 * 1024,
        law_sha256: sha256(s.law_text().as_bytes()),
        fault,
    };
    execute(p, s, header, 0.0)
}
pub fn write_retained_stream_segment(
    p: &Path,
    config: FundedAdmissionConfig,
    fault: StreamFault,
) -> Result<SegmentReport, String> {
    config.validate()?;
    let s = FamilySimulation::new_retained_stream(config.clone())?;
    let header = SegmentHeader {
        profile: RETAINED_STREAM_PROFILE.into(),
        dynamics_profile: RETAINED_PROFILE.into(),
        recipe: config.coordinate_recipe(),
        funded_config: Some(config),
        retained_policy: Some(RETAINED_POLICY.into()),
        handoff_config: None,
        descendant_config: None,
        transfer_environment: None,
        mutation_stress_policy: None,
        start_tick: 0,
        parent: None,
        payload_limit: if fault == StreamFault::Quota {
            262144
        } else {
            16 * 1024 * 1024
        },
        fault,
        law_sha256: sha256(s.law_text().as_bytes()),
    };
    execute(p, s, header, 0.0)
}
pub fn write_handoff_stream_segment(
    p: &Path,
    config: FundedAdmissionConfig,
    handoff: HandoffConfig,
    fault: StreamFault,
) -> Result<SegmentReport, String> {
    let s = FamilySimulation::new_handoff_stream(config.clone(), handoff.clone())?;
    let header = SegmentHeader {
        profile: HANDOFF_STREAM_PROFILE.into(),
        dynamics_profile: HANDOFF_PROFILE.into(),
        recipe: config.coordinate_recipe(),
        funded_config: Some(config),
        retained_policy: None,
        handoff_config: Some(handoff),
        descendant_config: None,
        transfer_environment: None,
        mutation_stress_policy: None,
        start_tick: 0,
        parent: None,
        payload_limit: if fault == StreamFault::Quota {
            262144
        } else {
            16 * 1024 * 1024
        },
        fault,
        law_sha256: sha256(s.law_text().as_bytes()),
    };
    execute(p, s, header, 0.0)
}
pub fn write_descendant_stream_segment(
    p: &Path,
    config: FundedAdmissionConfig,
    descendant: DescendantConfig,
    fault: StreamFault,
) -> Result<SegmentReport, String> {
    let s = FamilySimulation::new_descendant_stream(config.clone(), descendant.clone())?;
    let header = SegmentHeader {
        profile: DESCENDANT_STREAM_PROFILE.into(),
        dynamics_profile: DESCENDANT_PROFILE.into(),
        recipe: config.coordinate_recipe(),
        funded_config: Some(config),
        retained_policy: None,
        handoff_config: None,
        descendant_config: Some(descendant),
        transfer_environment: None,
        mutation_stress_policy: None,
        start_tick: 0,
        parent: None,
        payload_limit: if fault == StreamFault::Quota {
            262144
        } else {
            16 * 1024 * 1024
        },
        fault,
        law_sha256: sha256(s.law_text().as_bytes()),
    };
    execute(p, s, header, 0.0)
}
pub fn resume_stream_segment(source: &Path, p: &Path) -> Result<SegmentReport, String> {
    let start = Instant::now();
    let loaded = load_cut(source, None, &mut BTreeSet::new())?;
    let s = FamilySimulation::resume_live(
        &loaded.checkpoint,
        &loaded.marker.cursor,
        &loaded.attestation,
    )?;
    if s.funded_config() != loaded.header.funded_config.as_ref()
        || s.retained_state().map(|r| &r.policy) != loaded.header.retained_policy.as_ref()
        || s.handoff_config() != loaded.header.handoff_config.as_ref()
        || s.descendant_config() != loaded.header.descendant_config.as_ref()
    {
        return Err("resumed stream header/config mismatch".into());
    }
    let resume_seconds = start.elapsed().as_secs_f64();
    let header = SegmentHeader {
        profile: loaded.header.profile,
        dynamics_profile: loaded.header.dynamics_profile,
        recipe: loaded.header.recipe,
        funded_config: loaded.header.funded_config,
        retained_policy: loaded.header.retained_policy,
        handoff_config: loaded.header.handoff_config,
        descendant_config: loaded.header.descendant_config,
        transfer_environment: loaded.header.transfer_environment,
        mutation_stress_policy: loaded.header.mutation_stress_policy,
        start_tick: loaded.marker.cursor.tick,
        parent: Some(ParentCommit {
            directory: source
                .canonicalize()
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned(),
            commit: loaded.marker_name,
            sha256: loaded.marker_sha256,
        }),
        fault: StreamFault::Normal,
        payload_limit: 16 * 1024 * 1024,
        law_sha256: loaded.header.law_sha256,
    };
    execute(p, s, header, resume_seconds)
}
pub fn write_stream_panel(p: &Path) -> Result<(), String> {
    fs::create_dir(p).map_err(|e| format!("exclusive stream panel: {e}"))?;
    let mut m = meta("segmented_evidence_resource_pilot")
        .create(p)
        .map_err(|e| e.to_string())?;
    m.max_steps = 9 * 2944;
    m.start().map_err(|e| e.to_string())?;
    let result: Result<(), String> = (|| {
        for (name, b) in [
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_streaming.py").as_slice(),
            ),
            (
                "admission_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_admission.py").as_slice(),
            ),
            (
                "replay.py",
                include_bytes!("../../../../scripts/verify_evolving_minds.py").as_slice(),
            ),
            (
                "d0_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_d0.py").as_slice(),
            ),
            (
                "schedule_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_choice.py").as_slice(),
            ),
            ("base_rules.toml", ADMISSION_LAWS.as_bytes()),
            (
                "contract.md",
                include_bytes!(
                    "../../../../experiments/evolving_minds/contracts/d0_streaming_v1.md"
                )
                .as_slice(),
            ),
        ] {
            put(&p.join(name), b)?
        }
        let sources: std::collections::BTreeMap<_, _> = SOURCES.iter().copied().collect();
        put(&p.join("sources.json"), &bytes(&sources)?)?;
        let mut reports = vec![];
        for case in ["faithful_disjoint", "mixed_competing", "mixed_settlement"] {
            let report = write_stream_segment(
                &p.join(format!("baseline_{case}")),
                AdmissionRecipe {
                    case: case.into(),
                    rotation: 0,
                    reversed: false,
                },
                StreamFault::Normal,
            )?;
            if report.state != "Completed" {
                return Err("unexpected baseline interruption".into());
            }
            reports.push(report);
            m.final_step += 2944;
            m.persist().map_err(|e| e.to_string())?;
        }
        for fault in [
            StreamFault::CleanStop,
            StreamFault::TornPayload,
            StreamFault::TornCheckpoint,
            StreamFault::BeforeCommit,
            StreamFault::AfterCommit,
            StreamFault::Quota,
        ] {
            let name = fault.name();
            let source = p.join(format!("fault_{name}"));
            let report = write_stream_segment(
                &source,
                AdmissionRecipe {
                    case: "mixed_settlement".into(),
                    rotation: 0,
                    reversed: false,
                },
                fault.clone(),
            )?;
            if report.state != "Interrupted" {
                return Err("missing fixed injected interruption".into());
            }
            let expected = if fault == StreamFault::AfterCommit {
                16
            } else {
                8
            };
            if report.committed_final_tick != expected {
                return Err(format!(
                    "fixed recovery cut differs for {name}: {}",
                    report.committed_final_tick
                ));
            }
            reports.push(report);
            let continued = resume_stream_segment(&source, &p.join(format!("recovered_{name}")))?;
            if continued.state != "Completed" {
                return Err("unexpected recovery interruption".into());
            }
            reports.push(continued);
            m.final_step += 2944;
            m.persist().map_err(|e| e.to_string())?;
            if size(p)? > 256 * 1024 * 1024 - 1024 * 1024 {
                return Err("stream panel output ceiling".into());
            }
        }
        put(&p.join("summary.json"), &bytes(&reports)?)?;
        put(&p.join("run_status.json"), &bytes(&flags(true, None))?)?;
        put(&p.join("report.md"),b"# Segmented evidence engineering pilot\n\nThree complete baselines and six declared interruption/recovery chains. Commit markers bind full native evidence prefixes and live checkpoints. Scientific and broad fairness gates remain false.\n")?;
        m.seal_completed(m.final_step).map_err(|e| e.to_string())
    })();
    if let Err(e) = &result {
        m.seal_interrupted(m.final_step)
            .map_err(|e| e.to_string())?;
        if !p.join("run_status.json").exists() {
            put(&p.join("run_status.json"), &bytes(&flags(false, Some(e)))?)?
        }
    }
    result
}
