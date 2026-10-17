//! Bounded authored controls of information-dependent paid resource choice.
use super::{
    config::*,
    runtime::{checksum, float, int, validate_material_profile, Capture},
    Birth, Frame, Simulation,
};
use bincode::Options;
use hg_core::{HypergraphStore, PropertyValue};
use hg_engine::{engine::EngineCheckpoint, Engine};
use hg_observe::StepStats;
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, MatchBudgetPolicy, StepMode};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};

pub const CHOICE_PROFILE: &str = "evolving-minds-choice-v1";
pub const CHOICE_LAWS: &str = include_str!("../../../../rules/evolving_minds/choice_v1.toml");
pub const CLOCK: u32 = 241001;
pub const RESOURCE: u32 = 241002;
pub const CUE: u32 = 241003;
pub const COIN: u32 = 241004;
pub const SENSOR_COIN: u32 = 241005;
pub const POLICY: u32 = 241006;
pub const ATTEMPT: u32 = 241007;
pub const CHOICE: u32 = 241008;
pub const CHOICE_PHASES: [Phase; 11] = [
    Phase::Environment,
    Phase::World,
    Phase::Sense,
    Phase::Signal,
    Phase::Development,
    Phase::Attempt,
    Phase::Settle,
    Phase::Maintenance,
    Phase::Birth,
    Phase::Death,
    Phase::Cleanup,
];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Controller {
    Inactive,
    Constant0,
    Constant1,
    Random,
    Reactive,
}
impl Controller {
    pub fn code(self) -> i64 {
        match self {
            Self::Inactive => 0,
            Self::Constant0 => 1,
            Self::Constant1 => 2,
            Self::Random => 3,
            Self::Reactive => 4,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Constant0 => "constant0",
            Self::Constant1 => "constant1",
            Self::Random => "random",
            Self::Reactive => "reactive",
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CueMode {
    Informative,
    Permuted,
    Withheld,
}
impl CueMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Informative => "informative",
            Self::Permuted => "permuted",
            Self::Withheld => "withheld",
        }
    }
}

pub fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
fn balanced(seed: u64) -> Vec<bool> {
    let mut order: Vec<_> = (0..64).collect();
    let mut state = seed;
    for i in (1..64).rev() {
        let n = (i + 1) as u64;
        let threshold = n.wrapping_neg() % n;
        let draw = loop {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let x = mix64(state);
            if x >= threshold {
                break x;
            }
        };
        order.swap(i, (draw % n) as usize);
    }
    let mut bits = vec![false; 64];
    for i in &order[..32] {
        bits[*i] = true;
    }
    bits
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChoiceSchedule {
    pub seed: u64,
    pub mode: CueMode,
    pub resources: Vec<bool>,
    pub cues: Vec<bool>,
    pub coins: Vec<Vec<bool>>,
}
impl ChoiceSchedule {
    pub fn new(seed: u64, mode: CueMode) -> Self {
        let resources = balanced(seed ^ 0x5245534f55524345);
        let cues = match mode {
            CueMode::Informative => resources.clone(),
            CueMode::Permuted => balanced(seed ^ 0x4355455f5045524d),
            CueMode::Withheld => vec![false; 64],
        };
        let coins = (0..8)
            .map(|site| {
                (0..64)
                    .map(|t| mix64(seed ^ 0x434f494e5f535452 ^ ((site as u64) << 32) ^ t) & 1 != 0)
                    .collect()
            })
            .collect();
        Self {
            seed,
            mode,
            resources,
            cues,
            coins,
        }
    }
    pub fn law_text(&self) -> String {
        let mut text = CHOICE_LAWS.to_string();
        for site in 0..8 {
            for t in 0..64 {
                text+=&format!("\n[[rules]]\nname = \"world_{site}_{t}\"\npriority = 0\n[rules.lhs]\nnodes = [\"site\"]\nedges = []\n[rules.lhs.node_labels]\nsite = {SITE}\n[[rules.lhs.node_property_guards]]\nvar = \"site\"\nproperty = {CLOCK}\nop = \"eq\"\nvalue = {t}\n[[rules.lhs.node_property_guards]]\nvar = \"site\"\nproperty = 241009\nop = \"eq\"\nvalue = {site}\n[rules.rhs]\nnodes = [\"site\"]\nedges = []\n");
                for (key, v) in [
                    (RESOURCE, self.resources[t]),
                    (CUE, self.cues[t]),
                    (COIN, self.coins[site][t]),
                ] {
                    text+=&format!("[[rules.rhs.property_effects]]\nvar = \"site\"\nkey = {key}\nop = \"set\"\nvalue = {v}\n");
                }
                text+=&format!("[[rules.rhs.property_effects]]\nvar = \"site\"\nkey = {CLOCK}\nop = \"add\"\nvalue = 1\n");
            }
        }
        text
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ChoiceRow {
    pub tick: u64,
    pub engine_step: u64,
    pub live_organisms: usize,
    pub births: u64,
    pub deaths: u64,
    pub developed: u64,
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub harvested: f64,
    pub influx: f64,
    pub dissipated: f64,
    pub total_energy: f64,
    pub residual: f64,
    pub match_work: usize,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Ledger {
    initial: f64,
    influx: f64,
    dissipated: f64,
    harvested: f64,
    births: u64,
    deaths: u64,
    developed: u64,
    attempts: u64,
    successes: u64,
    failures: u64,
    trace_bytes: u64,
}
#[derive(Serialize, Deserialize)]
struct Saved {
    profile: String,
    controller: Controller,
    schedule: ChoiceSchedule,
    store: HypergraphStore,
    allocator: hg_core::graph::GraphAllocationState,
    engine: EngineCheckpoint,
    tick: u64,
    lineage: BTreeMap<i64, Birth>,
    frames: Vec<Frame>,
    rows: Vec<ChoiceRow>,
    ledger: Ledger,
    law_identity: u64,
}
pub struct ChoiceSimulation {
    controller: Controller,
    pub schedule: ChoiceSchedule,
    pub(crate) store: HypergraphStore,
    engine: Engine,
    capture: Arc<Mutex<Vec<super::Event>>>,
    rules: Vec<CompiledRule>,
    tick: u64,
    pub lineage: BTreeMap<i64, Birth>,
    pub frames: Vec<Frame>,
    pub rows: Vec<ChoiceRow>,
    ledger: Ledger,
    quiescent: bool,
}
fn config(seed: u64) -> Config {
    Config {
        seed,
        sites: 8,
        ticks: 64,
        ..Config::default()
    }
}
fn total(store: &HypergraphStore) -> Result<f64, String> {
    store
        .nodes()
        .filter(|(_, n)| [ROOT, SITE, DEAD].contains(&n.label.0))
        .map(|(_, n)| float(n, ENERGY))
        .sum()
}
fn boolean(n: &hg_core::NodeData, key: u32) -> Result<bool, String> {
    match n.get_property(key) {
        Some(PropertyValue::Bool(v)) => Ok(v),
        _ => Err(format!("required Bool {key}")),
    }
}
impl ChoiceSimulation {
    pub fn new(controller: Controller, seed: u64, mode: CueMode) -> Result<Self, String> {
        let base = Simulation::new(config(seed), Instruction::BuildRelay)?;
        let mut store = base.store().clone();
        let mut site = 0;
        let ids: Vec<_> = store.nodes().map(|(id, _)| id).collect();
        for id in ids {
            let n = store.get_node_mut(id).unwrap();
            match n.label.0 {
                ROOT => {
                    n.set_property(POLICY, PropertyValue::Int(controller.code()));
                    n.set_property(ATTEMPT, PropertyValue::Bool(false));
                    n.set_property(CHOICE, PropertyValue::Bool(false));
                }
                GENE => n.set_property(OPCODE, PropertyValue::Int(controller.code())),
                SENSOR => n.set_property(SENSOR_COIN, PropertyValue::Bool(false)),
                SITE => {
                    for key in [RESOURCE, CUE, COIN] {
                        n.set_property(key, PropertyValue::Bool(false));
                    }
                    n.set_property(CLOCK, PropertyValue::Int(0));
                    n.set_property(241009, PropertyValue::Int(site));
                    site += 1;
                }
                _ => {}
            }
        }
        let schedule = ChoiceSchedule::new(seed, mode);
        let rules =
            compile_rules(&parse_rules_toml(&schedule.law_text()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let mut engine = Engine::new(vec![], StepMode::MaxParallel, seed, 100000);
        engine.set_match_budget_policy(100000, 10000, MatchBudgetPolicy::Pooled);
        let capture = Arc::new(Mutex::new(Vec::new()));
        engine.add_observer(Box::new(Capture(capture.clone())));
        let frames = vec![Frame::new(&store, 0, None, vec![], StepStats::default(), 0)];
        let mut lineage = base.lineage().clone();
        lineage.get_mut(&1).unwrap().opcode = controller.code();
        let ledger = Ledger {
            initial: total(&store)?,
            ..Ledger::default()
        };
        let sim = Self {
            controller,
            schedule,
            store,
            engine,
            capture,
            rules,
            tick: 0,
            lineage,
            frames,
            rows: vec![],
            ledger,
            quiescent: true,
        };
        sim.validate()?;
        Ok(sim)
    }
    pub fn validate(&self) -> Result<(), String> {
        validate_material_profile(
            &self.store,
            &config(self.schedule.seed),
            &self.lineage,
            &self.frames[0],
            4,
            true,
        )?;
        for (_, n) in self.store.nodes() {
            match n.label.0 {
                ROOT => {
                    if int(n, POLICY)? != self.controller.code()
                        || self.lineage[&int(n, BIRTH_ID)?].opcode != self.controller.code()
                    {
                        return Err("controller identity changed".into());
                    }
                    boolean(n, ATTEMPT)?;
                    boolean(n, CHOICE)?;
                }
                SENSOR => {
                    boolean(n, SENSOR_COIN)?;
                }
                SITE => {
                    for key in [RESOURCE, CUE, COIN] {
                        boolean(n, key)?;
                    }
                    let clock = int(n, CLOCK)?;
                    if !(0..=64).contains(&clock) || !(0..8).contains(&int(n, 241009)?) {
                        return Err("world clock/slot".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub fn advance_tick(&mut self) -> Result<(), String> {
        if !self.quiescent || self.tick >= 64 {
            return Err("invalid choice tick boundary".into());
        }
        self.validate()?;
        self.quiescent = false;
        let mut work = 0;
        for phase in CHOICE_PHASES {
            let before = self.store.clone();
            self.engine.replace_static_rules(
                self.rules
                    .iter()
                    .filter(|r| r.name.starts_with(phase.prefix()))
                    .cloned()
                    .collect(),
            );
            // A native matching early exit must not replay the previous phase's capture.
            self.capture.lock().unwrap().clear();
            let stats = self.engine.step(&mut self.store);
            if self.engine.match_ceiling_binding() || stats.property_effects_skipped != 0 {
                return Err("choice match ceiling/skipped effect".into());
            }
            let events = std::mem::take(&mut *self.capture.lock().unwrap());
            if phase == Phase::Maintenance {
                let roots: Vec<_> = before.nodes().filter(|(_, n)| n.label.0 == ROOT).collect();
                if roots.len() != events.len() {
                    return Err("maintenance coverage".into());
                }
                for (id, n) in roots {
                    if events.iter().filter(|e| e.bound.contains(&id)).count() != 1
                        || int(
                            self.store.get_node(id).ok_or("missing maintained root")?,
                            AGE,
                        )? != int(n, AGE)? + 1
                    {
                        return Err("maintenance engagement".into());
                    }
                    self.ledger.dissipated += float(n, ENERGY)?.min(float(n, MAINTENANCE)?);
                }
            }
            for e in &events {
                if e.vanished_energy != 0.0 {
                    return Err("unsettled deletion".into());
                }
                match e.rule.as_str() {
                    "environment_influx" => self.ledger.influx += 8.0,
                    "development_relay" => {
                        self.ledger.dissipated += 2.0;
                        self.ledger.developed += 1;
                    }
                    "attempt_commit" => {
                        self.ledger.dissipated += 1.0;
                        self.ledger.attempts += 1;
                    }
                    "settle_success" => {
                        self.ledger.harvested += 8.0;
                        self.ledger.successes += 1;
                    }
                    "settle_failure" => self.ledger.failures += 1,
                    "birth_faithful" => {
                        self.ledger.dissipated += 2.0;
                        self.ledger.births += 1;
                        let child = e
                            .created
                            .iter()
                            .filter_map(|id| self.store.get_node(*id))
                            .find(|n| n.label.0 == ROOT)
                            .ok_or("child absent")?;
                        let id = int(child, BIRTH_ID)?;
                        let parent = int(child, PARENT_ID)?;
                        if !self.lineage.contains_key(&parent)
                            || self.lineage[&parent].death_tick.is_some()
                        {
                            return Err("invalid parent".into());
                        }
                        if self
                            .lineage
                            .insert(
                                id,
                                Birth {
                                    birth_id: id,
                                    parent_id: Some(parent),
                                    born_tick: self.tick + 1,
                                    death_tick: None,
                                    opcode: self.controller.code(),
                                },
                            )
                            .is_some()
                        {
                            return Err("duplicate birth".into());
                        }
                    }
                    name if name.starts_with("death_") => {
                        self.ledger.deaths += 1;
                        let root = e
                            .bound
                            .iter()
                            .filter_map(|id| before.get_node(*id))
                            .find(|n| n.label.0 == ROOT)
                            .ok_or("dead root absent")?;
                        if self
                            .lineage
                            .get_mut(&int(root, BIRTH_ID)?)
                            .ok_or("death identity")?
                            .death_tick
                            .replace(self.tick + 1)
                            .is_some()
                        {
                            return Err("duplicate death".into());
                        }
                    }
                    _ => {}
                }
            }
            self.validate()?;
            if total(&self.store)?
                != self.ledger.initial + self.ledger.influx - self.ledger.dissipated
            {
                return Err("choice ledger".into());
            }
            work += self.engine.match_budget_usage().work;
            self.frames.push(Frame::new(
                &self.store,
                self.tick + 1,
                Some(phase),
                events,
                stats,
                self.engine.match_budget_usage().work,
            ));
            self.ledger.trace_bytes += serde_json::to_vec(self.frames.last().unwrap())
                .map_err(|e| e.to_string())?
                .len() as u64
                + 1;
            if self.ledger.trace_bytes > 16 * 1024 * 1024 {
                return Err("choice trace ceiling".into());
            }
        }
        self.tick += 1;
        if self.ledger.attempts != self.ledger.successes + self.ledger.failures
            || self
                .store
                .nodes()
                .filter(|(_, n)| n.label.0 == ROOT)
                .any(|(_, n)| boolean(n, ATTEMPT).ok() != Some(false))
        {
            return Err("unsettled attempt".into());
        }
        self.rows.push(ChoiceRow {
            tick: self.tick,
            engine_step: self.store.step(),
            live_organisms: self
                .store
                .nodes()
                .filter(|(_, n)| n.label.0 == ROOT)
                .count(),
            births: self.ledger.births,
            deaths: self.ledger.deaths,
            developed: self.ledger.developed,
            attempts: self.ledger.attempts,
            successes: self.ledger.successes,
            failures: self.ledger.failures,
            harvested: self.ledger.harvested,
            influx: self.ledger.influx,
            dissipated: self.ledger.dissipated,
            total_energy: total(&self.store)?,
            residual: 0.0,
            match_work: work,
        });
        self.quiescent = true;
        Ok(())
    }
    pub fn run(&mut self) -> Result<(), String> {
        while self.tick < 64 {
            self.advance_tick()?;
        }
        Ok(())
    }
    pub fn checkpoint(&self) -> Result<Vec<u8>, String> {
        if !self.quiescent {
            return Err("choice checkpoint requires completed tick".into());
        }
        self.validate()?;
        let state = Saved {
            profile: CHOICE_PROFILE.into(),
            controller: self.controller,
            schedule: self.schedule.clone(),
            store: self.store.clone(),
            allocator: self.store.allocation_state(),
            engine: self.engine.diagnostic_checkpoint()?,
            tick: self.tick,
            lineage: self.lineage.clone(),
            frames: self.frames.clone(),
            rows: self.rows.clone(),
            ledger: self.ledger.clone(),
            law_identity: checksum(self.schedule.law_text().as_bytes()),
        };
        let payload = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&state)
            .map_err(|e| e.to_string())?;
        if payload.len() > 32 * 1024 * 1024 {
            return Err("choice checkpoint size".into());
        }
        let mut bytes = b"HGEMCH01".to_vec();
        bytes.extend((payload.len() as u64).to_le_bytes());
        bytes.extend(checksum(&payload).to_le_bytes());
        bytes.extend(payload);
        Ok(bytes)
    }
    pub fn resume(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 24
            || bytes.len() > 32 * 1024 * 1024 + 24
            || &bytes[..8] != b"HGEMCH01"
            || u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize != bytes.len() - 24
            || checksum(&bytes[24..]) != u64::from_le_bytes(bytes[16..24].try_into().unwrap())
        {
            return Err("choice checkpoint envelope".into());
        }
        let mut c: Saved = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(32 * 1024 * 1024)
            .reject_trailing_bytes()
            .deserialize(&bytes[24..])
            .map_err(|e| e.to_string())?;
        if c.profile != CHOICE_PROFILE
            || c.schedule != ChoiceSchedule::new(c.schedule.seed, c.schedule.mode)
            || c.law_identity != checksum(c.schedule.law_text().as_bytes())
            || c.tick > 64
            || c.store.step() != c.tick * 11
            || c.frames.len() != 1 + c.tick as usize * 11
            || c.rows.len() != c.tick as usize
            || c.engine.source.is_some()
        {
            return Err("choice checkpoint identity/coverage".into());
        }
        c.store
            .restore_allocation_state(c.allocator)
            .map_err(str::to_string)?;
        let mut engine = Engine::from_diagnostic_checkpoint(c.engine, None)?;
        let capture = Arc::new(Mutex::new(Vec::new()));
        engine.add_observer(Box::new(Capture(capture.clone())));
        let rules =
            compile_rules(&parse_rules_toml(&c.schedule.law_text()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let sim = Self {
            controller: c.controller,
            schedule: c.schedule,
            store: c.store,
            engine,
            capture,
            rules,
            tick: c.tick,
            lineage: c.lineage,
            frames: c.frames,
            rows: c.rows,
            ledger: c.ledger,
            quiescent: true,
        };
        sim.validate()?;
        if total(&sim.store)? != sim.ledger.initial + sim.ledger.influx - sim.ledger.dissipated {
            return Err("choice checkpoint ledger".into());
        }
        Ok(sim)
    }
}

pub fn resume_choice(dir: &Path, checkpoint: &Path) -> Result<(), String> {
    let size = std::fs::metadata(checkpoint)
        .map_err(|e| e.to_string())?
        .len();
    if size > 32 * 1024 * 1024 + 24 {
        return Err("choice checkpoint file ceiling".into());
    }
    let mut sim = ChoiceSimulation::resume(&std::fs::read(checkpoint).map_err(|e| e.to_string())?)?;
    std::fs::create_dir(dir).map_err(|e| e.to_string())?;
    sim.run()?;
    std::fs::write(
        dir.join("frames.json"),
        serde_json::to_vec(&sim.frames).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        dir.join("rows.json"),
        serde_json::to_vec(&sim.rows).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        dir.join("organisms.json"),
        serde_json::to_vec_pretty(&sim.lineage).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
