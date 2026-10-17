//! Same-build diagnostic checkpoints for the complete-witness execution core.
//! This is not a general runner/output/evaluator restart format or admission.
use super::{EreConfig, EreRuntime, EventSink};
use bincode::Options;
use hg_core::HypergraphStore;
use hg_engine::{engine::EngineCheckpoint, Engine};
use serde::{Deserialize, Serialize};

const MAGIC: &[u8; 8] = b"ERECP001";
const LIMIT: u64 = 32 * 1024 * 1024;

// EreConfig's JSON skip/default attributes are intentional and unsuitable for
// direct bincode. Use its supported self-describing representation, retaining
// every effective option. All other runtime fields use tagged binary serde.
pub(super) mod config_wire {
    use super::*;
    pub fn serialize<S: serde::Serializer>(c: &EreConfig, s: S) -> Result<S::Ok, S::Error> {
        serde_json::to_string(c)
            .map_err(serde::ser::Error::custom)?
            .serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<EreConfig, D::Error> {
        serde_json::from_str(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize, Deserialize)]
struct State {
    step: u64,
    protocol: String,
    store: HypergraphStore,
    engine: EngineCheckpoint,
    runtime: EreRuntime,
}

/// Call at initialized step zero or after after_step (optionally close_window). Saves without
/// compiling, advancing RNG or graph, paying fuel, or reseeding anything.
pub fn encode(
    store: &HypergraphStore,
    engine: &Engine,
    runtime: &EreRuntime,
) -> Result<Vec<u8>, String> {
    if runtime.before.is_some() || !runtime.events.lock().map_err(|e| e.to_string())?.is_empty() {
        return Err("checkpoint requires a quiescent completed-step boundary".into());
    }
    if runtime.config.protocol()? != hg_meta::COMPLETE_PROTOCOL {
        return Err("checkpoint requires the explicit complete-witness profile".into());
    }
    let engine_state = engine.diagnostic_checkpoint()?;
    if engine_state
        .source
        .as_ref()
        .is_none_or(|s| s.kind != "GraphRuleSource-v1")
    {
        return Err("checkpoint requires a supported graph rule source".into());
    }
    // Serialize borrowed fields with the same shape as State, without cloning
    // runtime ledgers or its event sink. The deserializer owns all restored data.
    let payload = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(LIMIT)
        .serialize(&(
            store.step(),
            runtime.config.protocol()?,
            store,
            engine_state,
            runtime,
        ))
        .map_err(|e| e.to_string())?;
    let mut out = MAGIC.to_vec();
    out.extend((payload.len() as u64).to_le_bytes());
    out.extend(hg_meta::hash_bytes(&payload).to_le_bytes());
    out.extend(payload);
    Ok(out)
}

/// Restores the owned source/engine/runtime and exactly one runtime event sink.
/// Caller-owned observers are intentionally not restored and must be attached
/// explicitly. No rule source is reconstructed from the current graph.
pub fn decode(bytes: &[u8]) -> Result<(HypergraphStore, Engine, EreRuntime), String> {
    if bytes.len() < 24 || &bytes[..8] != MAGIC {
        return Err("invalid checkpoint header/version".into());
    }
    let length = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let checksum = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    if length > LIMIT
        || length != (bytes.len() - 24) as u64
        || hg_meta::hash_bytes(&bytes[24..]) != checksum
    {
        return Err("checkpoint length/checksum mismatch".into());
    }
    let state: State = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(LIMIT)
        .reject_trailing_bytes()
        .deserialize(&bytes[24..])
        .map_err(|e| e.to_string())?;
    if state.step != state.store.step()
        || state.protocol != hg_meta::COMPLETE_PROTOCOL
        || state.runtime.config.protocol()? != state.protocol
        || state.runtime.before.is_some()
    {
        return Err("checkpoint step/protocol/boundary mismatch".into());
    }
    let saved = state
        .engine
        .source
        .as_ref()
        .ok_or("checkpoint requires graph rule source")?;
    if saved.kind != "GraphRuleSource-v1" {
        return Err("unsupported checkpoint rule source".into());
    }
    let source: hg_meta::GraphRuleSource = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(LIMIT)
        .reject_trailing_bytes()
        .deserialize(&saved.payload)
        .map_err(|e| e.to_string())?;
    let mut engine = Engine::from_diagnostic_checkpoint(state.engine, Some(Box::new(source)))?;
    engine.add_observer(Box::new(EventSink(state.runtime.events.clone())));
    Ok((state.store, engine, state.runtime))
}
