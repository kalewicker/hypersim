use super::config::*;
use bincode::Options;
use hg_core::{EdgeId, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::{engine::EngineCheckpoint, Engine};
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, MatchBudgetPolicy, StepMode};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub rule: String,
    pub bound: Vec<NodeId>,
    pub matched_edges: Vec<EdgeId>,
    pub created: Vec<NodeId>,
    pub deleted: Vec<NodeId>,
    pub edges_created: Vec<EdgeId>,
    pub edges_deleted: Vec<EdgeId>,
    pub vanished_energy: f64,
}
pub(crate) struct Capture(pub(crate) Arc<Mutex<Vec<Event>>>);
impl Observer for Capture {
    fn on_step_end(
        &mut self,
        _: &hg_core::GraphSnapshot<'_>,
        _: &StepStats,
        events: &[RewriteEvent],
    ) {
        *self.0.lock().unwrap() = events
            .iter()
            .map(|e| Event {
                rule: e.rule_name.clone(),
                bound: e.binding.bound_nodes.to_vec(),
                matched_edges: e.binding.matched_edges.to_vec(),
                created: e.nodes_created.clone(),
                deleted: e.nodes_deleted.clone(),
                edges_created: e.edges_created.clone(),
                edges_deleted: e.edges_deleted.clone(),
                vanished_energy: e.vanished_energy,
            })
            .collect();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub tick: u64,
    pub phase: Option<Phase>,
    pub step: u64,
    pub nodes: Vec<(NodeId, NodeData)>,
    pub edges: Vec<(EdgeId, HyperEdge)>,
    pub events: Vec<Event>,
    pub stats: StepStats,
    pub match_work: usize,
}
impl Frame {
    pub(crate) fn new(
        s: &HypergraphStore,
        tick: u64,
        phase: Option<Phase>,
        events: Vec<Event>,
        stats: StepStats,
        work: usize,
    ) -> Self {
        Self {
            tick,
            phase,
            step: s.step(),
            nodes: s.nodes().map(|(id, n)| (id, n.clone())).collect(),
            edges: s.edges().map(|(id, e)| (id, e.clone())).collect(),
            events,
            stats,
            match_work: work,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Row {
    pub tick: u64,
    pub engine_step: u64,
    pub live_organisms: usize,
    pub nodes: usize,
    pub edges: usize,
    pub births: u64,
    pub deaths: u64,
    pub developed: u64,
    pub harvested: f64,
    pub total_energy: f64,
    pub influx: f64,
    pub dissipated: f64,
    pub residual: f64,
    pub match_work: usize,
    pub accepted: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Birth {
    pub birth_id: i64,
    pub parent_id: Option<i64>,
    pub born_tick: u64,
    pub death_tick: Option<u64>,
    pub opcode: i64,
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
    evidence_bytes: u64,
}
#[derive(Serialize, Deserialize)]
struct Checkpoint {
    profile: String,
    config: Config,
    instruction: Instruction,
    tick: u64,
    store: HypergraphStore,
    allocator: hg_core::graph::GraphAllocationState,
    engine: EngineCheckpoint,
    ledger: Ledger,
    lineage: BTreeMap<i64, Birth>,
    frames: Vec<Frame>,
    rows: Vec<Row>,
    rules_identity: u64,
}

pub struct Simulation {
    pub(crate) config: Config,
    pub(crate) instruction: Instruction,
    pub(crate) store: HypergraphStore,
    engine: Engine,
    capture: Arc<Mutex<Vec<Event>>>,
    rules: Vec<CompiledRule>,
    pub(crate) tick: u64,
    ledger: Ledger,
    pub(crate) lineage: BTreeMap<i64, Birth>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) rows: Vec<Row>,
    quiescent: bool,
}

pub(crate) fn float(n: &NodeData, key: u32) -> Result<f64, String> {
    match n.get_property(key) {
        Some(PropertyValue::Float(v)) if v.is_finite() => Ok(v),
        _ => Err(format!("missing/non-finite Float property {key}")),
    }
}
pub(crate) fn int(n: &NodeData, key: u32) -> Result<i64, String> {
    match n.get_property(key) {
        Some(PropertyValue::Int(v)) => Ok(v),
        _ => Err(format!("missing Int property {key}")),
    }
}
fn boolean(n: &NodeData, key: u32) -> Result<bool, String> {
    match n.get_property(key) {
        Some(PropertyValue::Bool(v)) => Ok(v),
        _ => Err(format!("missing Bool property {key}")),
    }
}
fn link(s: &mut HypergraphStore, label: u32, a: NodeId, b: NodeId) {
    s.add_edge(HyperEdge::new(Label(label), [a, b].into_iter().collect()));
}
fn node(s: &mut HypergraphStore, label: u32, properties: &[(u32, PropertyValue)]) -> NodeId {
    let mut n = NodeData::new(Label(label), s.step());
    for &(k, v) in properties {
        n.set_property(k, v);
    }
    s.add_node(n)
}
pub(crate) fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
fn rules() -> Result<Vec<CompiledRule>, String> {
    compile_rules(&parse_rules_toml(RULES).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

impl Simulation {
    pub fn new(config: Config, instruction: Instruction) -> Result<Self, String> {
        config.validate()?;
        let rules = rules()?;
        let mut store = HypergraphStore::new();
        let sites: Vec<_> = (0..config.sites)
            .map(|_| {
                node(
                    &mut store,
                    SITE,
                    &[
                        (ENERGY, PropertyValue::Float(32.0)),
                        (OCCUPIED, PropertyValue::Bool(false)),
                    ],
                )
            })
            .collect();
        for i in 0..sites.len() {
            link(&mut store, NEIGHBOR, sites[i], sites[(i + 1) % sites.len()]);
        }
        let r = node(
            &mut store,
            ROOT,
            &[
                (ENERGY, PropertyValue::Float(24.0)),
                (BIRTH_ID, PropertyValue::Int(1)),
                (PARENT_ID, PropertyValue::Int(0)),
                (AGE, PropertyValue::Int(0)),
                (DEVELOPED, PropertyValue::Bool(false)),
                (MAINTENANCE, PropertyValue::Float(3.0)),
            ],
        );
        let s = node(
            &mut store,
            SENSOR,
            &[
                (OWNER, PropertyValue::Int(1)),
                (SIGNAL, PropertyValue::Bool(false)),
            ],
        );
        let g = node(
            &mut store,
            GENE,
            &[
                (OWNER, PropertyValue::Int(1)),
                (OPCODE, PropertyValue::Int(instruction.opcode())),
            ],
        );
        node(&mut store, COUNTER, &[(NEXT_ID, PropertyValue::Int(2))]);
        link(&mut store, OWN_SENSOR, r, s);
        link(&mut store, OWN_GENE, r, g);
        link(&mut store, OCCUPANCY, r, sites[0]);
        store
            .get_node_mut(sites[0])
            .unwrap()
            .set_property(OCCUPIED, PropertyValue::Bool(true));
        let mut engine = Engine::new(vec![], StepMode::MaxParallel, config.seed, 100_000);
        engine.set_match_budget_policy(
            config.match_work,
            config.match_results,
            MatchBudgetPolicy::Pooled,
        );
        let capture = Arc::new(Mutex::new(Vec::new()));
        engine.add_observer(Box::new(Capture(capture.clone())));
        let initial = energy(&store)?;
        let frames = vec![Frame::new(&store, 0, None, vec![], StepStats::default(), 0)];
        let lineage = BTreeMap::from([(
            1,
            Birth {
                birth_id: 1,
                parent_id: None,
                born_tick: 0,
                death_tick: None,
                opcode: instruction.opcode(),
            },
        )]);
        let sim = Self {
            config,
            instruction,
            store,
            engine,
            capture,
            rules,
            tick: 0,
            ledger: Ledger {
                initial,
                ..Ledger::default()
            },
            lineage,
            frames,
            rows: vec![],
            quiescent: true,
        };
        sim.validate_material()?;
        Ok(sim)
    }
    pub fn store(&self) -> &HypergraphStore {
        &self.store
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
    pub fn lineage(&self) -> &BTreeMap<i64, Birth> {
        &self.lineage
    }

    pub fn advance_tick(&mut self) -> Result<(), String> {
        if !self.quiescent {
            return Err("interrupted phase cannot continue without verified checkpoint".into());
        }
        if self.tick >= self.config.ticks {
            return Err("configured horizon already complete".into());
        }
        self.validate_material()?;
        self.quiescent = false;
        let mut work = 0;
        let mut accepted = 0;
        for phase in PHASES {
            let before = self.store.clone();
            self.engine.replace_static_rules(
                self.rules
                    .iter()
                    .filter(|r| r.name.starts_with(phase.prefix()))
                    .cloned()
                    .collect(),
            );
            let stats = self.engine.step(&mut self.store);
            if self.engine.match_ceiling_binding() {
                return Err(format!("matching ceiling at {phase:?}"));
            }
            if stats.property_effects_skipped != 0 {
                return Err("skipped physical effect".into());
            }
            let events = std::mem::take(&mut *self.capture.lock().unwrap());
            let phase_work = self.engine.match_budget_usage().work;
            work += phase_work;
            accepted += stats.accepted_rewrites;
            self.account(&before, phase, &events)?;
            self.frames.push(Frame::new(
                &self.store,
                self.tick + 1,
                Some(phase),
                events,
                stats,
                phase_work,
            ));
            self.ledger.evidence_bytes += serde_json::to_vec(self.frames.last().unwrap())
                .map_err(|e| e.to_string())?
                .len() as u64
                + 1;
            if self.ledger.evidence_bytes > self.config.output_bytes / 4 {
                return Err("trace retention ceiling".into());
            }
            self.validate_material()?;
            if energy(&self.store)?
                != self.ledger.initial + self.ledger.influx - self.ledger.dissipated
            {
                return Err(format!("energy ledger mismatch at {phase:?}"));
            }
        }
        self.tick += 1;
        self.rows.push(Row {
            tick: self.tick,
            engine_step: self.store.step(),
            live_organisms: self
                .store
                .nodes()
                .filter(|(_, n)| n.label.0 == ROOT)
                .count(),
            nodes: self.store.node_count(),
            edges: self.store.edge_count(),
            births: self.ledger.births,
            deaths: self.ledger.deaths,
            developed: self.ledger.developed,
            harvested: self.ledger.harvested,
            total_energy: energy(&self.store)?,
            influx: self.ledger.influx,
            dissipated: self.ledger.dissipated,
            residual: 0.0,
            match_work: work,
            accepted,
        });
        self.quiescent = true;
        Ok(())
    }
    pub fn run(&mut self) -> Result<(), String> {
        while self.tick < self.config.ticks {
            self.advance_tick()?;
        }
        Ok(())
    }
    fn account(
        &mut self,
        before: &HypergraphStore,
        phase: Phase,
        events: &[Event],
    ) -> Result<(), String> {
        if phase == Phase::Maintenance {
            let roots: Vec<_> = before.nodes().filter(|(_, n)| n.label.0 == ROOT).collect();
            let engaged: BTreeSet<_> = events
                .iter()
                .flat_map(|e| e.bound.iter().copied())
                .collect();
            if roots.len() != events.len() || roots.iter().any(|(id, _)| !engaged.contains(id)) {
                return Err("obligatory maintenance missed or duplicated".into());
            }
            for (id, n) in roots {
                let after = self
                    .store
                    .get_node(id)
                    .ok_or("maintenance removed anchor")?;
                if int(after, AGE)? != int(n, AGE)? + 1 {
                    return Err("maintenance age mismatch".into());
                }
                self.ledger.dissipated += float(n, ENERGY)?.min(float(n, MAINTENANCE)?);
            }
        }
        for event in events {
            if event.vanished_energy != 0.0 {
                return Err("unsettled energy deleted".into());
            }
            match event.rule.as_str() {
                "environment_influx" => self.ledger.influx += 8.0,
                "development_relay" => {
                    self.ledger.dissipated += 2.0;
                    self.ledger.developed += 1;
                }
                "action_intake" => self.ledger.harvested += 8.0,
                "birth_faithful" => {
                    self.ledger.dissipated += 2.0;
                    self.ledger.births += 1;
                    let child = event
                        .created
                        .iter()
                        .filter_map(|id| self.store.get_node(*id))
                        .find(|n| n.label.0 == ROOT)
                        .ok_or("birth missing child")?;
                    let id = int(child, BIRTH_ID)?;
                    let parent = int(child, PARENT_ID)?;
                    let p = self.lineage.get(&parent).ok_or("birth parent absent")?;
                    if p.death_tick.is_some() {
                        return Err("birth from retired parent".into());
                    }
                    let birth = Birth {
                        birth_id: id,
                        parent_id: Some(parent),
                        born_tick: self.tick + 1,
                        death_tick: None,
                        opcode: p.opcode,
                    };
                    if self.lineage.insert(id, birth).is_some() {
                        return Err("reused birth identity".into());
                    }
                }
                n if n.starts_with("death_") => {
                    self.ledger.deaths += 1;
                    let root = event
                        .bound
                        .iter()
                        .filter_map(|id| before.get_node(*id))
                        .find(|n| n.label.0 == ROOT)
                        .ok_or("death missing parent")?;
                    let b = self
                        .lineage
                        .get_mut(&int(root, BIRTH_ID)?)
                        .ok_or("death identity missing")?;
                    if b.death_tick.replace(self.tick + 1).is_some() {
                        return Err("duplicate death".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    /// Also rejects side channels, foreign ownership and stale topology before
    /// a malformed fixture can skip a cost by merely failing to match.
    pub fn validate_material(&self) -> Result<(), String> {
        validate_material_profile(
            &self.store,
            &self.config,
            &self.lineage,
            &self.frames[0],
            1,
            false,
        )
    }
    pub(crate) fn diagnostic_engine_checkpoint(&self) -> Result<EngineCheckpoint, String> {
        self.engine.diagnostic_checkpoint()
    }

    pub fn checkpoint(&self) -> Result<Vec<u8>, String> {
        if !self.quiescent || !self.capture.lock().unwrap().is_empty() {
            return Err("checkpoint requires completed tick".into());
        }
        self.validate_material()?;
        let state = Checkpoint {
            profile: PROFILE.into(),
            config: self.config.clone(),
            instruction: self.instruction,
            tick: self.tick,
            store: self.store.clone(),
            allocator: self.store.allocation_state(),
            engine: self.engine.diagnostic_checkpoint()?,
            ledger: self.ledger.clone(),
            lineage: self.lineage.clone(),
            frames: self.frames.clone(),
            rows: self.rows.clone(),
            rules_identity: checksum(RULES.as_bytes()),
        };
        let payload = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&state)
            .map_err(|e| e.to_string())?;
        if payload.len() > 64 * 1024 * 1024 {
            return Err("checkpoint byte ceiling".into());
        }
        let mut bytes = b"HGEMCP01".to_vec();
        bytes.extend((payload.len() as u64).to_le_bytes());
        bytes.extend(checksum(&payload).to_le_bytes());
        bytes.extend(payload);
        Ok(bytes)
    }
    pub fn resume(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 24 || &bytes[..8] != b"HGEMCP01" {
            return Err("checkpoint header/version".into());
        }
        let length = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        if length > 64 * 1024 * 1024
            || length as usize != bytes.len() - 24
            || checksum(&bytes[24..]) != u64::from_le_bytes(bytes[16..24].try_into().unwrap())
        {
            return Err("checkpoint length/checksum".into());
        }
        let mut c: Checkpoint = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(64 * 1024 * 1024)
            .reject_trailing_bytes()
            .deserialize(&bytes[24..])
            .map_err(|e| e.to_string())?;
        c.config.validate()?;
        if c.profile != PROFILE
            || c.rules_identity != checksum(RULES.as_bytes())
            || c.tick > c.config.ticks
            || c.store.step() != c.tick * PHASES.len() as u64
            || c.rows.len() != c.tick as usize
            || c.frames.len() != 1 + c.tick as usize * PHASES.len()
            || c.engine.source.is_some()
        {
            return Err("checkpoint profile/clock/coverage mismatch".into());
        }
        c.store
            .restore_allocation_state(c.allocator)
            .map_err(str::to_string)?;
        let mut engine = Engine::from_diagnostic_checkpoint(c.engine, None)?;
        let capture = Arc::new(Mutex::new(Vec::new()));
        engine.add_observer(Box::new(Capture(capture.clone())));
        let sim = Self {
            config: c.config,
            instruction: c.instruction,
            store: c.store,
            engine,
            capture,
            rules: rules()?,
            tick: c.tick,
            ledger: c.ledger,
            lineage: c.lineage,
            frames: c.frames,
            rows: c.rows,
            quiescent: true,
        };
        sim.validate_material()?;
        if energy(&sim.store)? != sim.ledger.initial + sim.ledger.influx - sim.ledger.dissipated {
            return Err("checkpoint ledger mismatch".into());
        }
        Ok(sim)
    }
}
fn edge_key(label: u32, a: NodeId, b: NodeId) -> (u32, NodeId, NodeId) {
    if a < b {
        (label, a, b)
    } else {
        (label, b, a)
    }
}
fn energy(s: &HypergraphStore) -> Result<f64, String> {
    s.nodes()
        .filter(|(_, n)| [ROOT, SITE, DEAD].contains(&n.label.0))
        .try_fold(0.0, |acc, (_, n)| Ok(acc + float(n, ENERGY)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_owner_guard_has_both_rejection_and_firing_controls() {
        let mut sim = Simulation::new(Config::default(), Instruction::BuildRelay).unwrap();
        sim.advance_tick().unwrap();
        let connection = sim
            .store
            .nodes()
            .find(|(_, n)| n.label.0 == CONNECTION)
            .unwrap()
            .0;
        sim.store
            .get_node_mut(connection)
            .unwrap()
            .set_property(OWNER, PropertyValue::Int(999));
        sim.engine.replace_static_rules(
            sim.rules
                .iter()
                .filter(|r| r.name.starts_with("signal_"))
                .cloned()
                .collect(),
        );
        assert_eq!(sim.engine.step(&mut sim.store).accepted_rewrites, 0);
        assert!(sim.validate_material().is_err());
        sim.store
            .get_node_mut(connection)
            .unwrap()
            .set_property(OWNER, PropertyValue::Int(1));
        assert_eq!(sim.engine.step(&mut sim.store).accepted_rewrites, 1);
    }

    #[test]
    fn missing_cost_bad_signal_and_hidden_reserve_fail_before_execution() {
        for defect in 0..3 {
            let mut sim = Simulation::new(Config::default(), Instruction::BuildRelay).unwrap();
            let label = if defect == 0 { ROOT } else { SENSOR };
            let id = sim
                .store
                .nodes()
                .find(|(_, n)| n.label.0 == label)
                .unwrap()
                .0;
            let (key, v) = match defect {
                0 => (MAINTENANCE, PropertyValue::Float(0.0)),
                1 => (SIGNAL, PropertyValue::Int(1)),
                _ => (ENERGY, PropertyValue::Float(100.0)),
            };
            sim.store.get_node_mut(id).unwrap().set_property(key, v);
            assert!(sim.advance_tick().is_err());
            assert_eq!(sim.store.step(), 0);
        }
    }
}

/// Shared fixed-price relay topology validation; profile-specific properties
/// and genotype ranges are checked by their respective runners.
pub(crate) fn validate_material_profile(
    store: &HypergraphStore,
    config: &Config,
    lineage: &BTreeMap<i64, Birth>,
    initial: &Frame,
    max_opcode: i64,
    all_relays: bool,
) -> Result<(), String> {
    if store.node_count() > config.max_nodes || store.edge_count() > config.max_edges {
        return Err("material ceiling".into());
    }
    let mut live = BTreeMap::new();
    let mut sites = BTreeSet::new();
    let mut counters = 0;
    for (id, n) in store.nodes() {
        if n.energy != 0.0 {
            return Err("unexpected energy field side channel".into());
        }
        if ![ROOT, SITE, DEAD].contains(&n.label.0) && n.get_property(ENERGY).is_some() {
            return Err("unaccounted internal reserve".into());
        }
        match n.label.0 {
            ROOT => {
                let bid = int(n, BIRTH_ID)?;
                if live.insert(bid, id).is_some() {
                    return Err("duplicate live identity".into());
                }
                let entry = lineage.get(&bid).ok_or("unknown live identity")?;
                if entry.death_tick.is_some() {
                    return Err("retired identity alive".into());
                }
                if int(n, PARENT_ID)? != entry.parent_id.unwrap_or(0) || int(n, AGE)? < 0 {
                    return Err("invalid anchor ancestry/age".into());
                }
                boolean(n, DEVELOPED)?;
                let m = float(n, MAINTENANCE)?;
                if m != 3.0 && m != 5.0 {
                    return Err("mutable physical price".into());
                }
                if float(n, ENERGY)? < 0.0 {
                    return Err("negative organism reserve".into());
                }
            }
            SITE => {
                sites.insert(id);
                boolean(n, OCCUPIED)?;
                if float(n, ENERGY)? < 0.0 {
                    return Err("negative site reserve".into());
                }
            }
            SENSOR | ACTUATOR => {
                int(n, OWNER)?;
                boolean(n, SIGNAL)?;
            }
            GENE => {
                int(n, OWNER)?;
                let op = int(n, OPCODE)?;
                if !(0..=max_opcode).contains(&op) {
                    return Err("invalid program opcode".into());
                }
            }
            CONNECTION => {
                int(n, OWNER)?;
            }
            COUNTER => {
                counters += 1;
                if int(n, NEXT_ID)? != lineage.len() as i64 + 1 {
                    return Err("birth counter mismatch".into());
                }
            }
            DEAD => {
                if float(n, ENERGY)? != 0.0 {
                    return Err("unsettled tombstone".into());
                }
            }
            _ => return Err("unknown material label".into()),
        }
    }
    if sites.len() != config.sites || counters != 1 {
        return Err("world scaffold changed".into());
    }
    let mut occupancy = BTreeMap::new();
    let mut owned: BTreeMap<i64, BTreeMap<u32, NodeId>> = BTreeMap::new();
    for (id, n) in store
        .nodes()
        .filter(|(_, n)| [SENSOR, GENE, CONNECTION, ACTUATOR].contains(&n.label.0))
    {
        let owner = int(n, OWNER)?;
        if !live.contains_key(&owner)
            || owned
                .entry(owner)
                .or_default()
                .insert(n.label.0, id)
                .is_some()
        {
            return Err("orphan/duplicate owned material".into());
        }
    }
    let mut expected = BTreeSet::new();
    for (&owner, &root) in &live {
        let parts = owned.get(&owner).ok_or("anchor missing seed")?;
        let sensor = *parts.get(&SENSOR).ok_or("missing sensor")?;
        let gene = *parts.get(&GENE).ok_or("missing instruction")?;
        if int(store.get_node(gene).unwrap(), OPCODE)? != lineage[&owner].opcode {
            return Err("unfaithful genotype".into());
        }
        let r = store.get_node(root).unwrap();
        let built = boolean(r, DEVELOPED)? && (all_relays || lineage[&owner].opcode == 1);
        if parts.len() != if built { 4 } else { 2 }
            || float(r, MAINTENANCE)? != if built { 5.0 } else { 3.0 }
        {
            return Err("phenotype/price mismatch".into());
        }
        expected.insert(edge_key(OWN_SENSOR, root, sensor));
        expected.insert(edge_key(OWN_GENE, root, gene));
        if built {
            let c = *parts.get(&CONNECTION).ok_or("missing connection")?;
            let a = *parts.get(&ACTUATOR).ok_or("missing actuator")?;
            expected.insert(edge_key(OWN_CONNECTION, root, c));
            expected.insert(edge_key(OWN_ACTUATOR, root, a));
            expected.insert(edge_key(SOURCE, sensor, c));
            expected.insert(edge_key(TARGET, c, a));
        }
    }
    let mut actual = BTreeSet::new();
    let mut neighbor_count = 0;
    let scaffold: BTreeSet<_> = initial
        .edges
        .iter()
        .filter(|(_, e)| e.label.0 == NEIGHBOR)
        .map(|(_, e)| edge_key(NEIGHBOR, e.nodes[0], e.nodes[1]))
        .collect();
    for (_, e) in store.edges() {
        if e.nodes.len() != 2
            || e.nodes[0] == e.nodes[1]
            || e.nodes.iter().any(|id| store.get_node(*id).is_none())
        {
            return Err("invalid edge endpoints".into());
        }
        if !actual.insert(edge_key(e.label.0, e.nodes[0], e.nodes[1])) {
            return Err("duplicate relation".into());
        }
        if e.label.0 == OCCUPANCY {
            let root = e
                .nodes
                .iter()
                .find(|id| store.get_node(**id).unwrap().label.0 == ROOT)
                .ok_or("occupancy without anchor")?;
            let site = e
                .nodes
                .iter()
                .find(|id| sites.contains(id))
                .ok_or("occupancy without site")?;
            if occupancy.insert(*site, *root).is_some() {
                return Err("multiply occupied site".into());
            }
            expected.insert(edge_key(OCCUPANCY, *root, *site));
        } else if e.label.0 == NEIGHBOR {
            if e.nodes.iter().any(|id| !sites.contains(id)) {
                return Err("invalid world adjacency".into());
            }
            if !scaffold.contains(&edge_key(NEIGHBOR, e.nodes[0], e.nodes[1])) {
                return Err("world scaffold changed".into());
            }
            neighbor_count += 1;
            expected.insert(edge_key(NEIGHBOR, e.nodes[0], e.nodes[1]));
        }
    }
    if actual != expected || occupancy.len() != live.len() || neighbor_count != sites.len() {
        return Err("ownership/topology mismatch".into());
    }
    for site in sites {
        if boolean(store.get_node(site).unwrap(), OCCUPIED)? != occupancy.contains_key(&site) {
            return Err("occupancy flag mismatch".into());
        }
    }
    Ok(())
}
