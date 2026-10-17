//! First D0 package: bounded native development, without reproduction or mutation.
use super::{
    choice::{ChoiceSchedule, CueMode, ATTEMPT, CHOICE, CLOCK, COIN, CUE, RESOURCE},
    config::*,
    runtime::{checksum, float, int, Capture},
    Frame,
};
use bincode::Options;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::{engine::EngineCheckpoint, Engine};
use hg_observe::StepStats;
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, MatchBudgetPolicy, StepMode};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

pub const D0_PROFILE: &str = "evolving-minds-d0-development-v1";
pub const D0_LAWS: &str = include_str!("../../../../rules/evolving_minds/d0_v1.toml");
pub const STATE: u32 = 0xE1010000;
pub const SLOT: u32 = STATE + 1;
pub const OWN_PART: u32 = STATE + 100;
pub const SLOT_MAP: u32 = STATE + 101;
pub const NEXT_INSTRUCTION: u32 = STATE + 102;
pub const CURSOR: u32 = 242001;
pub const STATUS: u32 = 242002;
pub const POSITION: u32 = 242003;
pub const INSTRUCTION_ID: u32 = 242004;
pub const DEST: u32 = 242005;
pub const SRC: u32 = 242006;
pub const INITIAL: u32 = 242007;
pub const USED: u32 = 242008;
pub const HAS_INPUT: u32 = 242009;
pub const BUFFER: u32 = 242010;
pub const HAS_ACTUATOR: u32 = 242011;
pub const D0_TICKS: u64 = 24;
pub const D0_STEPS: u64 = 15;
const PHASES: [(Phase, &str); 15] = [
    (Phase::Environment, "environment_"),
    (Phase::World, "world_"),
    (Phase::Sense, "sense_0"),
    (Phase::Sense, "sense_1"),
    (Phase::Signal, "prepare_2_"),
    (Phase::Signal, "prepare_3_"),
    (Phase::Signal, "prepare_4_"),
    (Phase::Signal, "prepare_5_"),
    (Phase::Signal, "prepare_6_"),
    (Phase::Signal, "prepare_7_"),
    (Phase::Signal, "commit_"),
    (Phase::Development, "development_"),
    (Phase::Attempt, "attempt_"),
    (Phase::Settle, "settle_"),
    (Phase::Maintenance, "maintenance_"),
];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum D0Op {
    State,
    Actuator,
    Wire,
    Halt,
}
impl D0Op {
    fn code(self) -> i64 {
        match self {
            Self::State => 0,
            Self::Actuator => 1,
            Self::Wire => 2,
            Self::Halt => 3,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct D0Instruction {
    pub id: u32,
    pub op: D0Op,
    pub slot: u8,
    pub source: u8,
    pub initial: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct D0Program {
    pub instructions: Vec<D0Instruction>,
}
impl D0Program {
    pub fn validate(&self) -> Result<(), String> {
        if self.instructions.is_empty() || self.instructions.len() > 8 {
            return Err("D0 length must be 1..8".into());
        }
        let mut ids = BTreeSet::new();
        for (i, g) in self.instructions.iter().enumerate() {
            if !ids.insert(g.id) || g.slot > 7 || g.source > 7 {
                return Err("D0 instruction identity/slot bounds".into());
            }
            let canonical = match g.op {
                D0Op::State => g.slot >= 2 && g.source == 0,
                D0Op::Actuator => g.slot >= 2 && g.source == 0 && !g.initial,
                D0Op::Wire => !g.initial,
                D0Op::Halt => {
                    g.slot == 0 && g.source == 0 && !g.initial && i + 1 == self.instructions.len()
                }
            };
            if !canonical {
                return Err("D0 typed parameter/order grammar".into());
            }
        }
        if self.instructions.last().unwrap().op != D0Op::Halt {
            return Err("D0 needs one final explicit Halt".into());
        }
        Ok(())
    }
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        // Fixed explicit schema, independent of arena handles and enum serialization.
        let mut bytes = b"HG-D0-1".to_vec();
        bytes.push(self.instructions.len() as u8);
        for g in &self.instructions {
            bytes.extend(g.id.to_le_bytes());
            bytes.extend([g.op.code() as u8, g.slot, g.source, g.initial as u8]);
        }
        Ok(bytes)
    }
}
fn instruction(id: u32, op: D0Op, slot: u8, source: u8, initial: bool) -> D0Instruction {
    D0Instruction {
        id,
        op,
        slot,
        source,
        initial,
    }
}
pub fn d0_program(name: &str) -> Result<D0Program, String> {
    use D0Op::*;
    let mut ops = match name {
        "cue" => vec![(Actuator, 2, 0, false), (Wire, 2, 0, false)],
        "coin" => vec![(Actuator, 2, 0, false), (Wire, 2, 1, false)],
        "constant0" | "constant1" => vec![
            (State, 2, 0, name == "constant1"),
            (Actuator, 3, 0, false),
            (Wire, 3, 2, false),
        ],
        "delayed_cue" => vec![
            (State, 2, 0, false),
            (Actuator, 3, 0, false),
            (Wire, 2, 0, false),
            (Wire, 3, 2, false),
        ],
        "missing_port" => vec![(Actuator, 2, 0, false), (Wire, 2, 7, false)],
        "duplicate_writer" => vec![
            (Actuator, 2, 0, false),
            (Wire, 2, 0, false),
            (Wire, 2, 1, false),
        ],
        "duplicate_slot" => vec![(State, 2, 0, false), (Actuator, 2, 0, false)],
        "second_actuator" => vec![(Actuator, 2, 0, false), (Actuator, 3, 0, false)],
        _ => return Err("unknown D0 fixture program".into()),
    };
    ops.push((Halt, 0, 0, false));
    Ok(D0Program {
        instructions: ops
            .into_iter()
            .enumerate()
            .map(|(i, (op, slot, source, initial))| {
                instruction(100 + i as u32, op, slot, source, initial)
            })
            .collect(),
    })
}
pub fn d0_law_text(schedule: &ChoiceSchedule) -> String {
    let choice = schedule.law_text();
    let start = choice
        .find("\n[[rules]]\nname = \"world_")
        .expect("shared frozen world generator");
    format!("{D0_LAWS}{}", &choice[start..])
}
fn boolean(n: &NodeData, k: u32) -> Result<bool, String> {
    match n.get_property(k) {
        Some(PropertyValue::Bool(v)) => Ok(v),
        _ => Err(format!("D0 Bool {k}")),
    }
}
fn energy(s: &HypergraphStore) -> Result<f64, String> {
    s.nodes()
        .filter(|(_, n)| [ROOT, SITE].contains(&n.label.0))
        .map(|(_, n)| float(n, ENERGY))
        .sum()
}
fn edge_key(label: u32, a: NodeId, b: NodeId) -> (u32, BTreeSet<NodeId>) {
    (label, BTreeSet::from([a, b]))
}

// Pure grammar projection used for validation; never creates phenotype or executes dynamics.
pub(crate) fn shape(
    program: &D0Program,
    cursor: usize,
) -> Result<(BTreeMap<u8, (u32, bool)>, Vec<(u8, u8)>), String> {
    let mut parts = BTreeMap::from([(0, (SENSOR, false)), (1, (SENSOR, false))]);
    let mut wires = vec![];
    let mut actuator = false;
    for g in program.instructions.iter().take(cursor) {
        match g.op {
            D0Op::State | D0Op::Actuator => {
                if parts.contains_key(&g.slot) || (g.op == D0Op::Actuator && actuator) {
                    return Err("D0 duplicate slot/actuator".into());
                }
                let label = if g.op == D0Op::State {
                    STATE
                } else {
                    actuator = true;
                    ACTUATOR
                };
                parts.insert(g.slot, (label, g.initial));
            }
            D0Op::Wire => {
                if !parts.contains_key(&g.source)
                    || !parts.contains_key(&g.slot)
                    || parts[&g.slot].0 == SENSOR
                    || g.source == g.slot
                    || wires.iter().any(|&(_, d)| d == g.slot)
                {
                    return Err("D0 missing/illegal port or multiple writer".into());
                }
                wires.push((g.source, g.slot));
            }
            D0Op::Halt => {}
        }
    }
    Ok((parts, wires))
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct D0Row {
    pub tick: u64,
    pub engine_step: u64,
    pub cursor: i64,
    pub status: i64,
    pub material_nodes: usize,
    pub development_paid: u64,
    pub development_failures: u64,
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
    development_paid: u64,
    development_failures: u64,
    attempts: u64,
    successes: u64,
    failures: u64,
}
#[derive(Serialize, Deserialize)]
struct Saved {
    profile: String,
    program: D0Program,
    schedule: ChoiceSchedule,
    initial_reserve: f64,
    store: HypergraphStore,
    allocator: hg_core::graph::GraphAllocationState,
    engine: EngineCheckpoint,
    tick: u64,
    frames: Vec<Frame>,
    rows: Vec<D0Row>,
    ledger: Ledger,
    law_identity: u64,
}
pub struct D0Simulation {
    pub program: D0Program,
    pub schedule: ChoiceSchedule,
    pub frames: Vec<Frame>,
    pub rows: Vec<D0Row>,
    store: HypergraphStore,
    engine: Engine,
    capture: Arc<Mutex<Vec<super::Event>>>,
    rules: Vec<CompiledRule>,
    initial_reserve: f64,
    tick: u64,
    ledger: Ledger,
    quiescent: bool,
}
impl D0Simulation {
    pub fn new(program: D0Program, seed: u64, mode: CueMode, reserve: f64) -> Result<Self, String> {
        program.validate()?;
        if !reserve.is_finite() || !(0.0..=512.0).contains(&reserve) {
            return Err("D0 founder reserve 0..512".into());
        }
        let mut store = HypergraphStore::new();
        let mut add = |label, props: Vec<(u32, PropertyValue)>| {
            let mut n = NodeData::new(Label(label), 0);
            for (k, v) in props {
                n.set_property(k, v)
            }
            store.add_node(n)
        };
        use PropertyValue::{Bool as B, Float as F, Int as I};
        let site = add(
            SITE,
            vec![
                (ENERGY, F(64.0)),
                (CLOCK, I(0)),
                (241009, I(0)),
                (RESOURCE, B(false)),
                (CUE, B(false)),
                (COIN, B(false)),
            ],
        );
        let root = add(
            ROOT,
            vec![
                (ENERGY, F(reserve)),
                (BIRTH_ID, I(1)),
                (AGE, I(0)),
                (CURSOR, I(0)),
                (STATUS, I(0)),
                (HAS_ACTUATOR, B(false)),
                (ATTEMPT, B(false)),
                (CHOICE, B(false)),
                (MAINTENANCE, F((11 + program.instructions.len()) as f64)),
            ],
        );
        let mut slots = vec![];
        for slot in 0..8 {
            slots.push(add(
                SLOT,
                vec![(OWNER, I(1)), (DEST, I(slot)), (USED, B(slot < 2))],
            ))
        }
        let mut sensors = vec![];
        for slot in 0..2 {
            sensors.push(add(
                SENSOR,
                vec![(OWNER, I(1)), (DEST, I(slot)), (SIGNAL, B(false))],
            ))
        }
        let mut genes = vec![];
        for (pos, g) in program.instructions.iter().enumerate() {
            genes.push(add(
                GENE,
                vec![
                    (OWNER, I(1)),
                    (POSITION, I(pos as i64)),
                    (INSTRUCTION_ID, I(g.id as i64)),
                    (OPCODE, I(g.op.code())),
                    (DEST, I(g.slot as i64)),
                    (SRC, I(g.source as i64)),
                    (INITIAL, B(g.initial)),
                ],
            ));
        }
        store.add_edge(HyperEdge::new(
            Label(OCCUPANCY),
            [root, site].into_iter().collect(),
        ));
        for &p in slots.iter().chain(sensors.iter()).chain(genes.iter()) {
            store.add_edge(HyperEdge::new(
                Label(OWN_PART),
                [root, p].into_iter().collect(),
            ));
        }
        for slot in 0..2 {
            store.add_edge(HyperEdge::new(
                Label(SLOT_MAP),
                [slots[slot], sensors[slot]].into_iter().collect(),
            ));
        }
        for pair in genes.windows(2) {
            store.add_edge(HyperEdge::new(
                Label(NEXT_INSTRUCTION),
                pair.iter().copied().collect(),
            ));
        }
        let schedule = ChoiceSchedule::new(seed, mode);
        let rules =
            compile_rules(&parse_rules_toml(&d0_law_text(&schedule)).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let mut engine = Engine::new(vec![], StepMode::MaxParallel, seed, 100000);
        engine.set_match_budget_policy(100000, 10000, MatchBudgetPolicy::Pooled);
        let capture = Arc::new(Mutex::new(vec![]));
        engine.add_observer(Box::new(Capture(capture.clone())));
        let frames = vec![Frame::new(&store, 0, None, vec![], StepStats::default(), 0)];
        let ledger = Ledger {
            initial: energy(&store)?,
            ..Default::default()
        };
        let sim = Self {
            program,
            schedule,
            frames,
            rows: vec![],
            store,
            engine,
            capture,
            rules,
            initial_reserve: reserve,
            tick: 0,
            ledger,
            quiescent: true,
        };
        sim.validate()?;
        Ok(sim)
    }
    pub fn store(&self) -> &HypergraphStore {
        &self.store
    }
    /// Read-only projection from the authoritative ordered instruction graph.
    pub fn graph_program(&self) -> Result<D0Program, String> {
        let mut genes: Vec<_> = self
            .store
            .nodes()
            .filter(|(_, n)| n.label.0 == GENE)
            .collect();
        genes.sort_by_key(|(_, n)| int(n, POSITION).unwrap_or(-1));
        let mut instructions = vec![];
        for (pos, (_, g)) in genes.iter().enumerate() {
            if int(g, POSITION)? != pos as i64 {
                return Err("D0 gene order".into());
            }
            let op = match int(g, OPCODE)? {
                0 => D0Op::State,
                1 => D0Op::Actuator,
                2 => D0Op::Wire,
                3 => D0Op::Halt,
                _ => return Err("D0 opcode".into()),
            };
            instructions.push(D0Instruction {
                id: u32::try_from(int(g, INSTRUCTION_ID)?).map_err(|_| "D0 identity")?,
                op,
                slot: u8::try_from(int(g, DEST)?).map_err(|_| "D0 slot")?,
                source: u8::try_from(int(g, SRC)?).map_err(|_| "D0 source")?,
                initial: boolean(g, INITIAL)?,
            });
        }
        let p = D0Program { instructions };
        p.validate()?;
        Ok(p)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.graph_program()?.canonical_bytes()? != self.program.canonical_bytes()? {
            return Err("D0 genotype/projection drift".into());
        }
        let roots: Vec<_> = self
            .store
            .nodes()
            .filter(|(_, n)| n.label.0 == ROOT)
            .collect();
        let sites: Vec<_> = self
            .store
            .nodes()
            .filter(|(_, n)| n.label.0 == SITE)
            .collect();
        if roots.len() != 1 || sites.len() != 1 {
            return Err("D0 founder/site inventory".into());
        }
        let (root, r) = roots[0];
        let (site, world) = sites[0];
        let cursor = usize::try_from(int(r, CURSOR)?).map_err(|_| "D0 cursor")?;
        let status = int(r, STATUS)?;
        if cursor > self.program.instructions.len()
            || !(0..=3).contains(&status)
            || (status == 2) != (cursor == self.program.instructions.len())
            || int(r, BIRTH_ID)? != 1
            || int(r, AGE)? < 0
            || int(r, AGE)? > 24
        {
            return Err("D0 execution state".into());
        }
        if status == 3
            && (cursor == self.program.instructions.len()
                || shape(&self.program, cursor + 1).is_ok())
        {
            return Err("D0 false failed instruction".into());
        }
        let (expected_parts, expected_wires) = shape(&self.program, cursor)?;
        let mut parts = BTreeMap::new();
        let mut slots = BTreeMap::new();
        let mut genes = BTreeMap::new();
        let mut wires = vec![];
        for (id, n) in self.store.nodes() {
            if n.energy != 0.0 || n.generation > self.store.step() {
                return Err("D0 hidden energy/generation".into());
            }
            let schema: Vec<u32> = match n.label.0 {
                ROOT => vec![
                    ENERGY,
                    BIRTH_ID,
                    AGE,
                    CURSOR,
                    STATUS,
                    HAS_ACTUATOR,
                    ATTEMPT,
                    CHOICE,
                    MAINTENANCE,
                ],
                SITE => vec![ENERGY, CLOCK, 241009, RESOURCE, CUE, COIN],
                SENSOR => vec![OWNER, DEST, SIGNAL],
                STATE | ACTUATOR => vec![OWNER, DEST, SIGNAL, BUFFER, HAS_INPUT],
                SLOT => vec![OWNER, DEST, USED],
                GENE => vec![OWNER, POSITION, INSTRUCTION_ID, OPCODE, DEST, SRC, INITIAL],
                CONNECTION => vec![OWNER],
                _ => return Err("D0 unsupported material label".into()),
            };
            if n.properties.len() != schema.len()
                || n.properties
                    .iter()
                    .map(|(k, _)| *k)
                    .collect::<BTreeSet<_>>()
                    != schema.into_iter().collect()
            {
                return Err("D0 material property schema".into());
            }
            if [ROOT, SITE].contains(&n.label.0) {
                if float(n, ENERGY)? < 0.0 {
                    return Err("D0 negative reserve".into());
                }
                continue;
            }
            if n.get_property(ENERGY).is_some() || int(n, OWNER)? != 1 {
                return Err("D0 part owner/hidden reserve".into());
            }
            match n.label.0 {
                SENSOR | STATE | ACTUATOR => {
                    let slot = int(n, DEST)?;
                    boolean(n, SIGNAL)?;
                    if parts.insert(slot, (id, n)).is_some() {
                        return Err("D0 duplicate material slot".into());
                    }
                    if n.label.0 != SENSOR {
                        boolean(n, BUFFER)?;
                        boolean(n, HAS_INPUT)?;
                    }
                }
                SLOT => {
                    if slots.insert(int(n, DEST)?, (id, n)).is_some() {
                        return Err("D0 duplicate map slot".into());
                    }
                    boolean(n, USED)?;
                }
                GENE => {
                    genes.insert(int(n, POSITION)?, id);
                }
                CONNECTION => wires.push(id),
                _ => return Err("D0 unsupported material label".into()),
            }
        }
        if slots.keys().copied().collect::<Vec<_>>() != (0..8).collect::<Vec<_>>()
            || parts.len() != expected_parts.len()
            || genes.len() != self.program.instructions.len()
            || wires.len() != expected_wires.len()
        {
            return Err("D0 bounded material inventory".into());
        }
        let mut expected = BTreeSet::from([edge_key(OCCUPANCY, root, site)]);
        for (_, (id, _)) in &slots {
            expected.insert(edge_key(OWN_PART, root, *id));
        }
        for (&slot, &(label, _)) in &expected_parts {
            let (id, n) = parts
                .get(&(slot as i64))
                .ok_or("D0 missing material slot")?;
            if n.label.0 != label {
                return Err("D0 material role".into());
            }
            let incoming = expected_wires.iter().any(|&(_, d)| d == slot);
            if label != SENSOR && boolean(n, HAS_INPUT)? != incoming {
                return Err("D0 input occupancy".into());
            }
            expected.insert(edge_key(OWN_PART, root, *id));
            expected.insert(edge_key(SLOT_MAP, slots[&(slot as i64)].0, *id));
        }
        for (&slot, (_, n)) in &slots {
            if boolean(n, USED)? != parts.contains_key(&slot) {
                return Err("D0 slot allocation".into());
            }
        }
        for pos in 0..genes.len() {
            expected.insert(edge_key(OWN_PART, root, genes[&(pos as i64)]));
            if pos > 0 {
                expected.insert(edge_key(
                    NEXT_INSTRUCTION,
                    genes[&((pos - 1) as i64)],
                    genes[&(pos as i64)],
                ));
            }
        }
        let actual: BTreeSet<_> = self
            .store
            .edges()
            .map(|(_, e)| (e.label.0, e.nodes.iter().copied().collect::<BTreeSet<_>>()))
            .collect();
        if actual.len() != self.store.edge_count()
            || self.store.edges().any(|(_, e)| e.nodes.len() != 2)
        {
            return Err("D0 duplicate/self/arity relation".into());
        }
        let mut unmatched = expected_wires.clone();
        for w in wires {
            expected.insert(edge_key(OWN_PART, root, w));
            let srcs: Vec<_> = parts
                .iter()
                .filter(|(_, (id, _))| actual.contains(&edge_key(SOURCE, *id, w)))
                .map(|(&s, _)| s as u8)
                .collect();
            let dsts: Vec<_> = parts
                .iter()
                .filter(|(_, (id, _))| actual.contains(&edge_key(TARGET, w, *id)))
                .map(|(&s, _)| s as u8)
                .collect();
            if srcs.len() != 1 || dsts.len() != 1 {
                return Err("D0 wire port references".into());
            }
            let pair = (srcs[0], dsts[0]);
            let i = unmatched
                .iter()
                .position(|p| *p == pair)
                .ok_or("D0 unprogrammed wire")?;
            unmatched.remove(i);
            expected.insert(edge_key(SOURCE, parts[&(pair.0 as i64)].0, w));
            expected.insert(edge_key(TARGET, w, parts[&(pair.1 as i64)].0));
        }
        if actual != expected {
            return Err("D0 relation grammar/ownership/order".into());
        }
        if float(r, MAINTENANCE)? != (self.store.node_count() - 1) as f64
            || boolean(r, HAS_ACTUATOR)? != parts.values().any(|(_, n)| n.label.0 == ACTUATOR)
            || (boolean(r, ATTEMPT)? && status != 2)
        {
            return Err("D0 upkeep/action grammar".into());
        }
        boolean(r, CHOICE)?;
        for key in [RESOURCE, CUE, COIN] {
            boolean(world, key)?;
        }
        if int(world, 241009)? != 0 || !(0..=24).contains(&int(world, CLOCK)?) {
            return Err("D0 world clock".into());
        }
        if self.store.node_count() > 40 || self.store.edge_count() > 64 {
            return Err("D0 expansion bound".into());
        }
        Ok(())
    }
    pub fn advance_tick(&mut self) -> Result<(), String> {
        if !self.quiescent || self.tick >= D0_TICKS {
            return Err("D0 completed tick boundary".into());
        }
        self.validate()?;
        self.quiescent = false;
        let mut work = 0;
        for (phase, prefix) in PHASES {
            let maintenance = if phase == Phase::Maintenance {
                let (_, r) = self.store.nodes().find(|(_, n)| n.label.0 == ROOT).unwrap();
                float(r, ENERGY)?.min(float(r, MAINTENANCE)?)
            } else {
                0.0
            };
            self.engine.replace_static_rules(
                self.rules
                    .iter()
                    .filter(|r| r.name.starts_with(prefix))
                    .cloned()
                    .collect(),
            );
            self.capture.lock().unwrap().clear();
            let stats = self.engine.step(&mut self.store);
            if self.engine.match_ceiling_binding() || stats.property_effects_skipped != 0 {
                return Err("D0 matcher ceiling/skipped effect".into());
            }
            let events = std::mem::take(&mut *self.capture.lock().unwrap());
            if phase == Phase::Development && events.len() > 1 {
                return Err("D0 multiple instructions per boundary".into());
            }
            if phase == Phase::Maintenance && events.len() != 1 {
                return Err("D0 maintenance coverage".into());
            }
            for e in &events {
                // Native rules recreate matched edges even when their relations survive.
                if e.vanished_energy != 0.0 || !e.deleted.is_empty() {
                    return Err("D0 unexpected node deletion".into());
                }
                match e.rule.as_str() {
                    "environment_influx" => self.ledger.influx += 8.0,
                    "development_wait" => {}
                    s if s.starts_with("development_") => {
                        self.ledger.dissipated += 1.0;
                        self.ledger.development_paid += 1;
                        if s == "development_fail" {
                            self.ledger.development_failures += 1
                        }
                    }
                    "attempt_commit" => {
                        self.ledger.dissipated += 1.0;
                        self.ledger.attempts += 1
                    }
                    "settle_success" => {
                        self.ledger.harvested += 8.0;
                        self.ledger.successes += 1
                    }
                    "settle_failure" => self.ledger.failures += 1,
                    s if s.starts_with("maintenance_") => self.ledger.dissipated += maintenance,
                    _ => {}
                }
            }
            self.validate()?;
            if energy(&self.store)?
                != self.ledger.initial + self.ledger.influx - self.ledger.dissipated
            {
                return Err("D0 conservation".into());
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
        }
        let (_, r) = self.store.nodes().find(|(_, n)| n.label.0 == ROOT).unwrap();
        if boolean(r, ATTEMPT)?
            || self.ledger.attempts != self.ledger.successes + self.ledger.failures
        {
            return Err("D0 unsettled action".into());
        }
        self.tick += 1;
        self.rows.push(D0Row {
            tick: self.tick,
            engine_step: self.store.step(),
            cursor: int(r, CURSOR)?,
            status: int(r, STATUS)?,
            material_nodes: self.store.node_count() - 1,
            development_paid: self.ledger.development_paid,
            development_failures: self.ledger.development_failures,
            attempts: self.ledger.attempts,
            successes: self.ledger.successes,
            failures: self.ledger.failures,
            harvested: self.ledger.harvested,
            influx: self.ledger.influx,
            dissipated: self.ledger.dissipated,
            total_energy: energy(&self.store)?,
            residual: 0.0,
            match_work: work,
        });
        self.quiescent = true;
        Ok(())
    }
    pub fn run(&mut self) -> Result<(), String> {
        while self.tick < D0_TICKS {
            self.advance_tick()?
        }
        Ok(())
    }
    pub fn checkpoint(&self) -> Result<Vec<u8>, String> {
        if !self.quiescent {
            return Err("D0 checkpoint boundary".into());
        }
        self.validate()?;
        let saved = Saved {
            profile: D0_PROFILE.into(),
            program: self.program.clone(),
            schedule: self.schedule.clone(),
            initial_reserve: self.initial_reserve,
            store: self.store.clone(),
            allocator: self.store.allocation_state(),
            engine: self.engine.diagnostic_checkpoint()?,
            tick: self.tick,
            frames: self.frames.clone(),
            rows: self.rows.clone(),
            ledger: self.ledger.clone(),
            law_identity: checksum(d0_law_text(&self.schedule).as_bytes()),
        };
        let payload = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&saved)
            .map_err(|e| e.to_string())?;
        if payload.len() > 8 * 1024 * 1024 {
            return Err("D0 checkpoint bound".into());
        }
        let mut bytes = b"HGEMD001".to_vec();
        bytes.extend((payload.len() as u64).to_le_bytes());
        bytes.extend(checksum(&payload).to_le_bytes());
        bytes.extend(payload);
        Ok(bytes)
    }
    pub fn resume(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 24
            || bytes.len() > 8 * 1024 * 1024 + 24
            || &bytes[..8] != b"HGEMD001"
            || u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize != bytes.len() - 24
            || checksum(&bytes[24..]) != u64::from_le_bytes(bytes[16..24].try_into().unwrap())
        {
            return Err("D0 checkpoint envelope".into());
        }
        let mut c: Saved = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(8 * 1024 * 1024)
            .reject_trailing_bytes()
            .deserialize(&bytes[24..])
            .map_err(|e| e.to_string())?;
        if c.profile != D0_PROFILE
            || c.schedule != ChoiceSchedule::new(c.schedule.seed, c.schedule.mode)
            || c.law_identity != checksum(d0_law_text(&c.schedule).as_bytes())
            || c.tick > D0_TICKS
            || c.store.step() != c.tick * D0_STEPS
            || c.frames.len() != 1 + c.tick as usize * D0_STEPS as usize
            || c.rows.len() != c.tick as usize
            || c.engine.source.is_some()
        {
            return Err("D0 checkpoint identity/coverage".into());
        }
        c.store
            .restore_allocation_state(c.allocator)
            .map_err(str::to_string)?;
        let mut sim = Self::new(
            c.program,
            c.schedule.seed,
            c.schedule.mode,
            c.initial_reserve,
        )?;
        if serde_json::to_vec(&sim.frames[0]).unwrap() != serde_json::to_vec(&c.frames[0]).unwrap()
        {
            return Err("D0 checkpoint founder identity".into());
        }
        // Verify all persisted frames, rows, ledger and engine state by exact native replay.
        while sim.tick < c.tick {
            sim.advance_tick()?
        }
        // Hash-map wire order is not canonical. Compare semantic execution state,
        // including RNG and opaque allocator continuation, rather than payload bytes.
        for (name, a, b) in [
            (
                "frames",
                serde_json::to_value(&sim.frames),
                serde_json::to_value(&c.frames),
            ),
            (
                "rows",
                serde_json::to_value(&sim.rows),
                serde_json::to_value(&c.rows),
            ),
            (
                "ledger",
                serde_json::to_value(&sim.ledger),
                serde_json::to_value(&c.ledger),
            ),
            (
                "material",
                serde_json::to_value(&sim.store),
                serde_json::to_value(&c.store),
            ),
            (
                "allocator",
                serde_json::to_value(sim.store.allocation_state()),
                serde_json::to_value(c.store.allocation_state()),
            ),
            (
                "engine",
                serde_json::to_value(sim.engine.diagnostic_checkpoint()?),
                serde_json::to_value(&c.engine),
            ),
        ] {
            if a.map_err(|e| e.to_string())? != b.map_err(|e| e.to_string())? {
                return Err(format!(
                    "D0 checkpoint {name} continuation/history mismatch"
                ));
            }
        }
        // Keep the exact persisted allocator and RNG continuation, after history validation.
        sim.store = c.store;
        sim.engine = Engine::from_diagnostic_checkpoint(c.engine, None)?;
        sim.capture = Arc::new(Mutex::new(vec![]));
        sim.engine
            .add_observer(Box::new(Capture(sim.capture.clone())));
        sim.validate()?;
        Ok(sim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn material_and_checkpoint_tampering_fail_closed() {
        for edit in 0..5 {
            let mut s =
                D0Simulation::new(d0_program("cue").unwrap(), 17, CueMode::Informative, 512.0)
                    .unwrap();
            s.run().unwrap();
            let id = s
                .store
                .nodes()
                .find(|(_, n)| {
                    n.label.0
                        == if edit == 0 {
                            GENE
                        } else if edit == 1 {
                            CONNECTION
                        } else {
                            ROOT
                        }
                })
                .unwrap()
                .0;
            let n = s.store.get_node_mut(id).unwrap();
            match edit {
                0 => n.set_property(OPCODE, PropertyValue::Int(2)),
                1 => n.set_property(OWNER, PropertyValue::Int(2)),
                2 => n.set_property(MAINTENANCE, PropertyValue::Float(0.0)),
                3 => n.energy = 1.0,
                _ => n.set_property(999999, PropertyValue::Bool(true)),
            }
            assert!(s.validate().is_err());
        }
        let mut s =
            D0Simulation::new(d0_program("cue").unwrap(), 17, CueMode::Informative, 512.0).unwrap();
        s.advance_tick().unwrap();
        let bytes = s.checkpoint().unwrap();
        let mut saved: Saved = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .deserialize(&bytes[24..])
            .unwrap();
        saved.rows[0].development_paid += 1;
        let payload = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&saved)
            .unwrap();
        let mut tampered = b"HGEMD001".to_vec();
        tampered.extend((payload.len() as u64).to_le_bytes());
        tampered.extend(checksum(&payload).to_le_bytes());
        tampered.extend(payload);
        assert!(matches!(D0Simulation::resume(&tampered),Err(s) if s.contains("rows")));
    }
}
