#![cfg(feature = "hg-meta")]
//! Opt-in long gate producer. Default tests advance only development horizons.
#[allow(dead_code)]
#[path = "support/witness.rs"]
mod witness;
use flate2::{write::GzEncoder, Compression};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyOp, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_rules::{
    compile_rule, CompiledRule, EffectOpDecl, EpochPolicy, MatchBudgetPolicy, NodePropertyGuard,
    PropertyEffectDecl, Rule, StepMode,
};
use hg_schema::MetricsRow;
use hg_sim_core::ere::{
    meta_excluded_store, EreArm, EreConfig, EreRuntime, EreSeedFamily, MetaVocabularyConfig,
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

const PROFILE: &str = "E2-complete-long-horizon-gates-v1";
const CLOCK: u32 = 0xE5560001;
const MARKER: u32 = 0xE5560002;
const CSV: &str = "step,full_hash,world_hash,n_nodes,n_edges,attempted_matches,accepted_rewrites,nodes_created,nodes_deleted,edges_created,edges_deleted,property_effects_applied,property_effects_skipped,rng_before,rng_after,match_work,match_results,match_ceiling_binding\n";
type Res<T> = Result<T, String>;
fn hex(b: Vec<u8>) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn save(p: &Path, v: &Value) -> Res<()> {
    std::fs::write(p, serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn demand(ok: bool, why: &str) -> Res<()> {
    if ok {
        Ok(())
    } else {
        Err(why.into())
    }
}
fn candidate(kind: &str) -> CandidateRule {
    let proposal = kind == "proposal";
    CandidateRule {
        priority: i32::from(proposal),
        labels: vec![Some(10), Some(11), Some(12)],
        lhs: if proposal {
            vec![vec![0, 1], vec![0, 2]]
        } else {
            vec![vec![0, 1], vec![1, 2]]
        },
        rhs: if kind == "core" {
            vec![vec![0, 1], vec![0, 2]]
        } else {
            vec![vec![0, 1], vec![1, 2]]
        },
        ..Default::default()
    }
}
fn program_value(p: &RuleProgram) -> Value {
    let c = |c: &CandidateRule| json!({"priority":c.priority,"labels":c.labels,"lhs":c.lhs,"rhs":c.rhs,"guards":[],"rcf":hex(c.rcf_bytes()),"execution":hex(c.execution_bytes())});
    json!({"core":c(&p.core),"proposal":p.proposal.as_ref().map(c),"active":p.active,"rcf":hex(p.rcf_bytes()),"execution":hex(p.execution_bytes())})
}
#[derive(Clone)]
struct Cell {
    id: String,
    arm: EreArm,
    identity: bool,
    fuel: f64,
    owner: bool,
    root_loss: bool,
    horizon: u64,
    development: bool,
}
impl Cell {
    fn gr3(&self) -> bool {
        matches!(self.arm, EreArm::A0 | EreArm::A1)
    }
    fn payloads(&self) -> Vec<u64> {
        if self.development {
            vec![8, 16, 24, 56]
        } else {
            vec![2500, 5000, 7500, 9992]
        }
    }
    fn ownership(&self) -> u64 {
        if self.development {
            40
        } else {
            9000
        }
    }
    fn deletion(&self) -> u64 {
        if self.development {
            60
        } else {
            9996
        }
    }
    fn value(&self) -> Value {
        json!({"id":self.id,"gate":if self.gr3(){"G-R3"}else{"G-R5"},"arm":self.arm,
        "identity":self.identity,"repair_fuel":self.fuel,"ownership_loss":self.owner,"root_loss":self.root_loss,
        "horizon":self.horizon,"development_only":self.development,"payload_steps":self.payloads(),"ownership_step":self.ownership(),"root_delete_step":self.deletion(),
        "seed":if self.gr3(){42}else{984927043621u64},"step_mode":if self.gr3(){"SequentialRandom"}else{"MaxParallel"},"max_pairings":if self.gr3(){1}else{16}})
    }
}
fn matrix(development: bool, horizon: u64) -> Vec<Cell> {
    let mut cells = vec![];
    for identity in [true, false] {
        for arm in [EreArm::A0, EreArm::A1] {
            cells.push(Cell {
                id: format!(
                    "{}_{}",
                    if identity {
                        "identity_dormant"
                    } else {
                        "oscillator_active"
                    },
                    format!("{arm:?}")
                ),
                arm,
                identity,
                fuel: 0.,
                owner: false,
                root_loss: false,
                horizon: if development { horizon } else { 100000 },
                development,
            });
        }
    }
    for arm in [EreArm::A2, EreArm::A3] {
        for fuel in [0., 2., 8.] {
            for owner in [false, true] {
                cells.push(Cell {
                    id: format!("{arm:?}_fuel{fuel}_owner{owner}"),
                    arm,
                    identity: false,
                    fuel,
                    owner,
                    root_loss: false,
                    horizon: if development { horizon } else { 10000 },
                    development,
                });
            }
        }
        cells.push(Cell {
            id: format!("anchor_loss_{arm:?}"),
            arm,
            identity: false,
            fuel: 8.,
            owner: false,
            root_loss: true,
            horizon: if development { horizon } else { 10000 },
            development,
        });
    }
    cells
}
fn environment(c: &Cell) -> Vec<CompiledRule> {
    let effect = PropertyEffectDecl {
        var: "clock".into(),
        key: CLOCK,
        op: EffectOpDecl::Add {
            value: PropertyValue::Int(1),
        },
    };
    let mut tick = Rule {
        name: "environment_clock".into(),
        priority: 100,
        ..Default::default()
    };
    tick.lhs.nodes = vec!["clock".into()];
    tick.rhs.nodes = tick.lhs.nodes.clone();
    tick.lhs.node_labels.insert("clock".into(), Label(777));
    tick.rhs_property_effects.push(effect.clone());
    let mut raw = vec![tick];
    for (kind, step) in c
        .payloads()
        .into_iter()
        .map(|d| ("payload", d))
        .chain(c.owner.then_some(("ownership", c.ownership())))
        .chain(c.root_loss.then_some(("delete", c.deletion())))
    {
        let mut r = Rule {
            name: format!("environment_{kind}_{step}"),
            priority: 200,
            ..Default::default()
        };
        r.lhs.nodes = vec!["clock".into(), "root".into(), "side".into()];
        r.rhs.nodes = r.lhs.nodes.clone();
        r.lhs.node_labels.insert("clock".into(), Label(777));
        for (var, key, value) in [
            ("clock", CLOCK, (step - 1) as i64),
            ("root", MARKER, 0),
            ("side", MARKER, 1),
        ] {
            r.lhs_node_property_guards.push(NodePropertyGuard {
                var: var.into(),
                property: key,
                op: PropertyOp::Eq,
                value: PropertyValue::Int(value),
            });
        }
        let endpoints: Vec<String> = if kind == "payload" {
            vec!["root".into(), "side".into(), "root".into()]
        } else {
            vec!["root".into(), "side".into()]
        };
        r.lhs.edges.push(endpoints.clone());
        if kind != "payload" {
            r.lhs.edge_labels.insert(0, Label(E_OWNS));
        }
        if kind == "delete" {
            r.rhs.nodes.retain(|n| n != "root");
        } else {
            r.rhs.edges.push(endpoints);
        }
        r.rhs_property_effects.push(effect.clone());
        raw.push(r);
    }
    raw.iter()
        .map(|r| {
            let mut r = compile_rule(r).unwrap();
            r.meta_visible = c.arm == EreArm::A3;
            r.pattern.node_label_ceiling = (c.arm == EreArm::A2).then_some(META_BASE);
            for e in &mut r.pattern.edges {
                e.ordered = true;
            }
            r
        })
        .collect()
}
struct Fixture {
    store: HypergraphStore,
    engine: Engine,
    runtime: EreRuntime,
    root: NodeId,
    core: NodeId,
    clock: Option<NodeId>,
    instructions: Vec<NodeId>,
    expected: RuleProgram,
    motifs: Vec<[NodeId; 3]>,
    config: EreConfig,
}
fn fixture(c: &Cell) -> Fixture {
    let core = candidate(if c.identity { "identity" } else { "core" });
    let proposal = candidate(if c.identity { "identity" } else { "proposal" });
    #[derive(serde::Serialize)]
    struct Raw {
        rules: Vec<Rule>,
    }
    let text = |r: &CandidateRule| {
        toml::to_string(&Raw {
            rules: vec![r.to_rule("ecology".into())],
        })
        .unwrap()
    };
    let config = EreConfig {
        arm: c.arm,
        encoding_version: EncodingVersion::AlternativeFamiliesV3,
        epoch: if c.gr3() {
            EpochPolicy::Never
        } else {
            EpochPolicy::Fixed(1.try_into().unwrap())
        },
        match_budget_policy: MatchBudgetPolicy::Pooled,
        match_ceiling: 50000,
        rule_region_ceiling: 500,
        measure_family_maintenance: true,
        measure_ei: false,
        meta_seed_energy: 0.,
        seed_families: Some(vec![EreSeedFamily {
            core_toml: text(&core),
            proposal_toml: text(&proposal),
            active: !c.identity,
        }]),
        vocabulary: Some(MetaVocabularyConfig {
            protocol: COMPLETE_PROTOCOL.into(),
            instructions: if c.gr3() {
                vec![MetaInstructionSeed {
                    operation: MetaOperation::Seal,
                    fuel: 0.,
                }]
            } else {
                vec![
                    MetaInstructionSeed {
                        operation: MetaOperation::Seal,
                        fuel: 1.,
                    },
                    MetaInstructionSeed {
                        operation: MetaOperation::RepairSealed,
                        fuel: c.fuel,
                    },
                ]
            },
        }),
        ..Default::default()
    };
    let mut store = HypergraphStore::new();
    let mut motifs = vec![];
    for _ in 0..4 {
        let mut ids = vec![];
        for l in [10, 11, 12] {
            ids.push(store.add_node(NodeData::new(Label(l), 0)));
        }
        let m = [ids[0], ids[1], ids[2]];
        for pair in [[m[0], m[1]], [m[1], m[2]]] {
            store.add_edge(HyperEdge::new(Label(0), pair.into_iter().collect()));
        }
        motifs.push(m);
    }
    for _ in 0..4 {
        store.add_node(NodeData::new(Label(99), 0));
    }
    let ordinary: Vec<_> = store.nodes().map(|(id, _)| id).collect();
    for id in ordinary {
        store
            .get_node_mut(id)
            .unwrap()
            .set_property(1, PropertyValue::Float(0.0));
    }
    let mut engine = Engine::new(
        vec![core.compile(&MetaLimits::default(), false).unwrap()],
        if c.gr3() {
            StepMode::SequentialRandom
        } else {
            StepMode::MaxParallel
        },
        if c.gr3() { 42 } else { 984927043621 },
        if c.gr3() { 1 } else { 16 },
    );
    engine.set_match_budget_policy(50000, 50000, MatchBudgetPolicy::Pooled);
    let mut runtime = EreRuntime::install(&config, &mut store, &mut engine).unwrap();
    let root = store
        .nodes()
        .find(|(_, n)| n.label.0 == L_RULE_ROOT)
        .unwrap()
        .0;
    let core_id = store
        .edges()
        .find(|(_, e)| e.label.0 == E_CORE && e.nodes[0] == root)
        .unwrap()
        .1
        .nodes[1];
    let clock = if c.gr3() {
        None
    } else {
        store
            .get_node_mut(root)
            .unwrap()
            .set_property(MARKER, PropertyValue::Int(0));
        store
            .get_node_mut(core_id)
            .unwrap()
            .set_property(MARKER, PropertyValue::Int(1));
        let mut n = NodeData::new(Label(777), 0);
        n.set_property(CLOCK, PropertyValue::Int(0));
        let clock = store.add_node(n);
        runtime
            .install_diagnostic_environment(environment(c), &mut engine)
            .unwrap();
        Some(clock)
    };
    let mut instructions: Vec<_> = store
        .nodes()
        .filter(|(id, _)| read_complete_instruction(&store.snapshot(), *id).is_ok())
        .map(|(id, _)| id)
        .collect();
    instructions.sort();
    Fixture {
        store,
        engine,
        runtime,
        root,
        core: core_id,
        clock,
        instructions,
        expected: RuleProgram {
            core,
            proposal: Some(proposal),
            active: !c.identity,
        },
        motifs,
        config,
    }
}
struct Capture(Arc<Mutex<Option<Value>>>);
impl hg_observe::Observer for Capture {
    fn on_step_end(
        &mut self,
        _: &hg_core::GraphSnapshot<'_>,
        stats: &hg_observe::StepStats,
        events: &[hg_observe::RewriteEvent],
    ) {
        *self.0.lock().unwrap() = Some(
            json!({"stats":stats,"events":events.iter().map(|e|json!({"rule_id":e.rule_id,"rule_name":e.rule_name,"bound_nodes":e.binding.bound_nodes,"matched_edges":e.binding.matched_edges,"nodes_created":e.nodes_created,"nodes_deleted":e.nodes_deleted,"edges_created":e.edges_created,"edges_deleted":e.edges_deleted,"cleanup_neighbors":e.cleanup_neighbors,"vanished_energy":e.vanished_energy})).collect::<Vec<_>>() }),
        );
    }
}
// Full typed graph delta, not just the template's deletion list: this includes
// native dangling-edge cleanup. Only two bounded inventories are live at once.
fn delta(before: &Value, after: &Value) -> Value {
    let mut out = json!({});
    for kind in ["nodes", "edges"] {
        let a: BTreeMap<_, _> = before[kind]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r[0].as_str().unwrap(),
                    if kind == "nodes" {
                        &r[1]["data"]
                    } else {
                        &r[1]
                    },
                )
            })
            .collect();
        let b: BTreeMap<_, _> = after[kind]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r[0].as_str().unwrap(),
                    if kind == "nodes" {
                        &r[1]["data"]
                    } else {
                        &r[1]
                    },
                )
            })
            .collect();
        out[kind] = json!({"deleted":a.keys().filter(|k|!b.contains_key(*k)).collect::<Vec<_>>(),"upsert":b.iter().filter(|(k,v)|a.get(*k)!=Some(*v)).map(|(k,v)|json!([k,v])).collect::<Vec<_>>()});
    }
    out
}
fn motifs(s: &HypergraphStore, ms: &[[NodeId; 3]]) -> Value {
    json!(ms.iter().map(|m|{let mut edges:Vec<_>=s.edges().filter(|(_,e)|e.nodes.iter().all(|n|m.contains(n))).map(|(id,e)|json!([format!("{id:?}"),e])).collect();edges.sort_by_key(|e|e[0].as_str().unwrap().to_owned());
        let pair=|a,b|s.edges().any(|(_,e)|e.label.0==0&&e.nodes.len()==2&&e.nodes.contains(&a)&&e.nodes.contains(&b));
        json!({"nodes":m,"phase":if pair(m[0],m[1])&&pair(m[1],m[2]){"path"}else if pair(m[0],m[1])&&pair(m[0],m[2]){"star"}else{"invalid"},"edges":edges})}).collect::<Vec<_>>())
}
fn instructions(f: &Fixture) -> Value {
    json!(f
        .instructions
        .iter()
        .map(|id| {
            let op = read_complete_instruction(&f.store.snapshot(), *id).unwrap();
            json!({"id":id,"operation":op.operation,"fuel":op.fuel,"valid":true})
        })
        .collect::<Vec<_>>())
}
fn binding_probes(f: &Fixture, c: &Cell, step: u64) -> Res<Value> {
    let mut probes = vec![];
    if c.arm != EreArm::A2 {
        return Ok(json!(probes));
    }
    let saved = bincode::serialize(&f.store).unwrap();
    let rng = f.engine.rng_state_probe();
    for r in f.engine.rules().iter().filter(|r| {
        r.name == format!("environment_payload_{step}")
            || r.name == format!("environment_ownership_{step}")
            || r.name == format!("environment_delete_{step}")
    }) {
        let mut protected = r.clone();
        protected.pattern.match_work_limit = Some(50000);
        protected.pattern.match_result_limit = Some(50000);
        let (bindings, usage) =
            hg_engine::find_bindings_counted(&f.store.snapshot(), &protected, 0);
        let mut visible = protected.clone();
        visible.meta_visible = true;
        visible.pattern.node_label_ceiling = None;
        let (counter, counter_usage) =
            hg_engine::find_bindings_counted(&f.store.snapshot(), &visible, 0);
        demand(
            bindings.is_empty()
                && !usage.enumeration_truncated
                && !counter_usage.enumeration_truncated,
            "protected probe capped or bound",
        )?;
        demand(
            counter
                .iter()
                .any(|b| b.bound_nodes.contains(&f.root) && b.bound_nodes.contains(&f.core)),
            "counterfactual misses real target",
        )?;
        probes.push(json!({"rule":r.name,"protected_rule":protected,"visible_rule":visible,"protected_bindings":bindings.iter().map(|b|json!({"nodes":b.bound_nodes,"edges":b.matched_edges})).collect::<Vec<_>>(),"visible_bindings":counter.iter().map(|b|json!({"nodes":b.bound_nodes,"edges":b.matched_edges})).collect::<Vec<_>>(),"protected_usage":usage,"visible_usage":counter_usage,"clock":step-1,"root":f.root,"core":f.core,"read_only":true}));
    }
    demand(
        saved == bincode::serialize(&f.store).unwrap() && rng == f.engine.rng_state_probe(),
        "probe mutated store/RNG",
    )?;
    Ok(json!(probes))
}
struct Limits {
    start: Instant,
    out: PathBuf,
    bytes: u64,
    auditor_seconds: f64,
    wall: f64,
    output: u64,
}
impl Limits {
    fn check(&self, s: &HypergraphStore) -> Res<()> {
        demand(
            self.start.elapsed().as_secs_f64() < self.wall,
            "assay wall cap",
        )?;
        demand(
            s.node_count() <= 128 && s.edge_count() <= 512,
            "store size cap",
        )?;
        demand(self.bytes <= self.output, "assay output cap")
    }
    fn account(&mut self, n: u64) -> Res<()> {
        self.bytes = self.bytes.checked_add(n).ok_or("byte overflow")?;
        demand(self.bytes <= self.output, "assay output cap")
    }
}
struct Stream(GzEncoder<BufWriter<File>>, u64);
impl Stream {
    fn new(path: &Path) -> Res<Self> {
        Ok(Self(
            GzEncoder::new(
                BufWriter::new(File::create(path).map_err(|e| e.to_string())?),
                Compression::default(),
            ),
            0,
        ))
    }
    fn line(&mut self, v: &Value, lim: &mut Limits) -> Res<()> {
        let mut b = serde_json::to_vec(v).map_err(|e| e.to_string())?;
        b.push(b'\n');
        self.0.write_all(&b).map_err(|e| e.to_string())?;
        // Flush keeps partial JSONL recoverable and lets the aggregate cap see
        // emitted compressed bytes, rather than a whole-horizon buffer.
        self.0.flush().map_err(|e| e.to_string())?;
        let now = self
            .0
            .get_ref()
            .get_ref()
            .metadata()
            .map_err(|e| e.to_string())?
            .len();
        lim.account(now - self.1)?;
        self.1 = now;
        Ok(())
    }
    fn finish(self, lim: &mut Limits) -> Res<()> {
        let prior = self.1;
        let mut w = self.0.finish().map_err(|e| e.to_string())?;
        w.flush().map_err(|e| e.to_string())?;
        lim.account(w.get_ref().metadata().map_err(|e| e.to_string())?.len() - prior)
    }
}
fn csv(row: &Value) -> String {
    let s = &row["native"]["stats"];
    let vals = vec![
        row["step"].to_string(),
        row["full_hash"].as_str().unwrap().into(),
        row["world_hash"].as_str().unwrap().into(),
        row["n_nodes"].to_string(),
        row["n_edges"].to_string(),
        s["attempted_matches"].to_string(),
        s["accepted_rewrites"].to_string(),
        s["nodes_created"].to_string(),
        s["nodes_deleted"].to_string(),
        s["edges_created"].to_string(),
        s["edges_deleted"].to_string(),
        s["property_effects_applied"].to_string(),
        s["property_effects_skipped"].to_string(),
        row["rng_before"].as_str().unwrap().into(),
        row["rng_after"].as_str().unwrap().into(),
        row["usage"]["work"].to_string(),
        row["usage"]["results"].to_string(),
        row["ceiling"].to_string(),
    ];
    vals.join(",") + "\n"
}
fn boundary(c: &Cell, step: u64) -> bool {
    step == 0
        || step == c.horizon
        || step % 1000 == 0
        || c.payloads()
            .into_iter()
            .chain(c.owner.then_some(c.ownership()))
            .chain(c.root_loss.then_some(c.deletion()))
            .any(|d| step >= d - 1 && step <= d + 2)
}
fn expected_valid(c: &Cell, step: u64) -> bool {
    if c.arm != EreArm::A3 {
        return true;
    }
    if (c.owner && step >= c.ownership()) || (c.root_loss && step >= c.deletion()) {
        return false;
    }
    let lesions: Vec<_> = c.payloads().into_iter().filter(|d| *d <= step).collect();
    lesions.is_empty() || (step > *lesions.last().unwrap() && lesions.len() <= c.fuel as usize)
}
fn native_checks(c: &Cell, f: &Fixture, row: &Value, before: &Value) -> Res<()> {
    let step = row["step"].as_u64().unwrap();
    demand(
        row["family_valid"] == expected_valid(c, step)
            && row["original_program"] == expected_valid(c, step),
        "native family timeline",
    )?;
    demand(
        row["anchor_alive"] == !(c.arm == EreArm::A3 && c.root_loss && step >= c.deletion()),
        "native anchor timeline",
    )?;
    demand(
        (row["rng_before"] != row["rng_after"]) == c.gr3(),
        "native RNG consumption",
    )?;
    let events = row["native"]["events"].as_array().unwrap();
    let worlds = events
        .iter()
        .filter(|e| e["rule_name"].as_str().unwrap().starts_with("e1_"))
        .count();
    demand(
        worlds
            == if expected_valid(c, step - 1) {
                if c.gr3() {
                    1
                } else {
                    4
                }
            } else {
                0
            },
        "native ecological engagement",
    )?;
    let paid: Vec<_> = events
        .iter()
        .filter(|e| {
            e["rule_name"]
                .as_str()
                .unwrap()
                .starts_with("complete_encoded_meta_")
        })
        .collect();
    let repairs: Vec<_> = if c.arm == EreArm::A3 {
        c.payloads()
            .into_iter()
            .take(c.fuel as usize)
            .map(|d| d + 1)
            .collect()
    } else {
        vec![]
    };
    let repair = repairs.contains(&step);
    let seal = !c.gr3() && step == 1;
    demand(
        paid.len() == usize::from(seal || repair),
        "native paid event schedule",
    )?;
    for e in paid {
        demand(
            e["rule_name"].as_str().unwrap().contains("RepairSealed") == repair,
            "native paid opcode",
        )?;
        let core_edges: Vec<_> = e["edges_created"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| format!("EdgeId({}v{})", id["idx"], id["version"]))
            .collect();
        demand(
            row["delta"]["edges"]["upsert"]
                .as_array()
                .unwrap()
                .iter()
                .any(|edge| {
                    core_edges.contains(&edge[0].as_str().unwrap().to_owned())
                        && edge[1]["label"] == E_CORE
                        && edge[1]["nodes"] == json!([f.root, f.core, f.root])
                }),
            "paid operation did not target ordered CORE payload",
        )?;
    }
    for op in row["instructions"].as_array().unwrap() {
        let is_seal = op["operation"] == "Seal";
        let expected = if is_seal {
            0.
        } else {
            c.fuel - repairs.iter().filter(|d| **d <= step).count() as f64
        };
        demand(op["fuel"] == expected, "native fuel chronology")?;
    }
    if !c.gr3() {
        demand(row["clock"] == step, "native clock chronology")?;
    }
    for n in before["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| [10, 11, 12, 99].contains(&n[1]["data"]["label"].as_u64().unwrap()))
    {
        let id = n[0].as_str().unwrap();
        demand(
            !row["delta"]["nodes"]["deleted"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n == id)
                && !row["delta"]["nodes"]["upsert"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|n| n[0] == id),
            "native ordinary handle/data changed",
        )?;
    }
    Ok(())
}
fn run_cell(c: &Cell, mut f: Fixture, lim: &mut Limits, instrument: bool) -> Res<Value> {
    let dir = lim.out.join(&c.id);
    std::fs::create_dir(&dir).map_err(|e| e.to_string())?;
    let mut raw = Stream::new(&dir.join("steps.jsonl.gz"))?;
    let mut inventories = Stream::new(&dir.join("inventories.jsonl.gz"))?;
    let mut windows = Stream::new(&dir.join("windows.jsonl.gz"))?;
    let mut csv_file = if c.gr3() {
        Some(BufWriter::new(
            File::create(dir.join("trajectory.csv")).map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };
    if let Some(w) = csv_file.as_mut() {
        w.write_all(CSV.as_bytes()).map_err(|e| e.to_string())?;
        lim.account(CSV.len() as u64)?;
    }
    let native = Arc::new(Mutex::new(None));
    f.engine.add_observer(Box::new(Capture(native.clone())));
    let mut previous = witness::inventory(&f.store);
    inventories.line(&previous, lim)?;
    let mut revision = None;
    let mut roles = [0u64; 2];
    let mut probe_work = 0usize;
    let mut probe_results = 0usize;
    for step in 1..=c.horizon {
        lim.check(&f.store)?;
        let rng = f.engine.rng_state_probe();
        let full_before = format!("{:016x}", f.store.canonical_hash());
        let world_before = format!("{:016x}", meta_excluded_store(&f.store).canonical_hash());
        f.engine.compile_at_boundary(&f.store);
        let report = f.engine.compile_report().cloned();
        let table = if revision != Some(f.engine.rule_table_revision()) {
            revision = Some(f.engine.rule_table_revision());
            Some(f.engine.rules().to_vec())
        } else {
            None
        };
        let probes = binding_probes(&f, c, step)?;
        for p in probes.as_array().unwrap() {
            for k in ["protected_usage", "visible_usage"] {
                probe_work += p[k]["enumeration_work"].as_u64().unwrap() as usize;
            }
            probe_results += p["protected_bindings"].as_array().unwrap().len()
                + p["visible_bindings"].as_array().unwrap().len();
        }
        demand(
            probe_work <= 500000 && probe_results <= 500000,
            "cell probe budget",
        )?;
        f.runtime.before_step(&f.store, &mut f.engine)?;
        demand(
            rng == f.engine.rng_state_probe(),
            "compilation consumes RNG",
        )?;
        let stats = f.engine.step(&mut f.store);
        f.runtime
            .note_match_ceiling(f.engine.match_ceiling_binding());
        f.runtime.after_step(&f.store, &stats);
        let after = witness::inventory(&f.store);
        let n = match native.lock().unwrap().take() {
            Some(n) => n,
            None => {
                // A matching cap returns before Engine notifies observers.
                // Archive its native refusal stats, then reject below.
                demand(
                    f.engine.match_ceiling_binding() && f.store.step() != step,
                    "observer missing on advancing step",
                )?;
                json!({"stats":stats,"events":[]})
            }
        };
        let decoded = read_program(&f.store.snapshot(), f.root, &MetaLimits::default()).ok();
        let class_count = f
            .engine
            .rules()
            .iter()
            .filter(|r| r.name.starts_with("e1_"))
            .count();
        for event in n["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["rule_name"].as_str().unwrap().starts_with("e1_"))
        {
            let r = &f.engine.rules()[event["rule_id"].as_u64().unwrap() as usize];
            roles[usize::from(r.priority == 1)] += 1;
        }
        let row = json!({"step":step,"full_hash_before":full_before,"world_hash_before":world_before,"full_hash":format!("{:016x}",f.store.canonical_hash()),"world_hash":format!("{:016x}",meta_excluded_store(&f.store).canonical_hash()),"n_nodes":f.store.node_count(),"n_edges":f.store.edge_count(),
            "rng_before":format!("{rng:016x}"),"rng_after_compile":format!("{rng:016x}"),"rng_after":format!("{:016x}",f.engine.rng_state_probe()),"usage":f.engine.match_budget_usage(),"ceiling":f.engine.match_ceiling_binding(),"guards":f.engine.drain_guard_stats(),"revision":revision,"table":table,"compile_report":report,"native":n,
            "delta":delta(&previous,&after),"family_valid":decoded.is_some(),"original_program":decoded.as_ref()==Some(&f.expected),"execution":decoded.map(|p|hex(p.execution_bytes())),"anchor_alive":f.store.get_node(f.root).is_some(),"clock":f.clock.map(|id|f.store.get_node(id).unwrap().get_property(CLOCK)),"instructions":instructions(&f),"world_classes":class_count,"motifs":motifs(&f.store,&f.motifs),"binding_probes":probes});
        raw.line(&row, lim)?;
        if let Some(w) = csv_file.as_mut() {
            let b = csv(&row);
            w.write_all(b.as_bytes()).map_err(|e| e.to_string())?;
            lim.account(b.len() as u64)?;
        }
        if boundary(c, step) {
            inventories.line(&after, lim)?;
        }
        // Persist the row before rejecting it. Caps/nonadvancement are not a
        // scientific null and must never quietly shorten a passing receipt.
        demand(f.store.step() == step, "nonadvancing engine step")?;
        demand(!f.engine.match_ceiling_binding(), "engine matching cap")?;
        lim.check(&f.store)?;
        native_checks(c, &f, &row, &previous)?;
        previous = after;
        if instrument && (step % 128 == 0 || step == c.horizon) {
            let before = bincode::serialize(&f.store).unwrap();
            let rng = f.engine.rng_state_probe();
            let row =
                f.runtime
                    .close_window(&f.store, MetricsRow::default(), MetricsRow::default());
            windows.line(&json!({"end":step,"row":row}), lim)?;
            demand(
                before == bincode::serialize(&f.store).unwrap()
                    && rng == f.engine.rng_state_probe(),
                "close_window mutated store/RNG",
            )?;
        }
    }
    if let Some(mut w) = csv_file {
        w.flush().map_err(|e| e.to_string())?;
    }
    raw.finish(lim)?;
    inventories.finish(lim)?;
    windows.finish(lim)?;
    Ok(
        json!({"id":c.id,"completed_steps":f.store.step(),"world_core_events":roles[0],"world_proposal_events":roles[1],"probe_work":probe_work,"probe_results":probe_results,"native_complete":true,"native_checks_pass":true}),
    )
}
fn setup_value(c: &Cell, f: &mut Fixture) -> Value {
    let before = bincode::serialize(&f.store).unwrap();
    let rng = f.engine.rng_state_probe();
    f.engine.compile_at_boundary(&f.store);
    assert_eq!(before, bincode::serialize(&f.store).unwrap());
    assert_eq!(rng, f.engine.rng_state_probe());
    json!({"cell":c.value(),"config":f.config,"initial":witness::inventory(&f.store),"root":f.root,"core":f.core,"clock":f.clock,"instructions":instructions(f),"program":program_value(&f.expected),"motifs":f.motifs,"initial_table":f.engine.rules(),"initial_report":f.engine.compile_report(),"initial_rng":format!("{rng:016x}")})
}
fn paths(dir: &Path, out: &mut Vec<PathBuf>) -> Res<()> {
    for e in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let p = e.map_err(|e| e.to_string())?.path();
        if p.is_dir() {
            paths(&p, out)?;
        } else if p.is_file() {
            out.push(p);
        }
    }
    Ok(())
}
fn freeze(out: &Path, setups: &[Value]) -> Res<()> {
    let root = repo();
    let mut inputs = vec![
        root.join("Cargo.lock"),
        root.join("Cargo.toml"),
        root.join("docs/ere_complete_long_gates_v1.md"),
        root.join("docs/ere_complete_long_gates_v1.json"),
    ];
    for dir in [
        "crates/hg-core",
        "crates/hg-engine",
        "crates/hg-rules",
        "crates/hg-meta",
        "crates/hg-observe",
        "crates/hg-schema",
        "crates/hg-eval",
        "crates/hg-causal",
        "crates/hg-output",
        "crates/hg-report",
        "crates/hg-app-api",
        "crates/hg-sim-core/src",
        "crates/hg-sim-core/tests/support",
    ] {
        paths(&root.join(dir), &mut inputs)?;
    }
    for p in [
        "crates/hg-sim-core/Cargo.toml",
        "crates/hg-sim-core/tests/ere_complete_long_gates.rs",
        "scripts/verify_ere_complete_long_gates.py",
        "scripts/test_verify_ere_complete_long_gates.py",
        "scripts/verify_ere_complete_gates.py",
        "scripts/verify_ere_complete_witness.py",
    ] {
        inputs.push(root.join(p));
    }
    inputs.sort();
    inputs.dedup();
    let mut sources = vec![];
    for p in inputs {
        let rel = p.strip_prefix(&root).unwrap();
        let dest = out.join("source_snapshots").join(rel);
        std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::copy(&p, &dest).map_err(|e| e.to_string())?;
        sources.push(rel.to_str().unwrap().to_owned());
    }
    std::fs::copy(
        std::env::current_exe().unwrap(),
        out.join("executed_test_binary"),
    )
    .map_err(|e| e.to_string())?;
    let basis = root.join("output/ere_complete_gates_fresh_20260926");
    std::fs::create_dir(out.join("basis")).map_err(|e| e.to_string())?;
    for p in ["summary.json", "independent_audit.json", "run_status.json"] {
        std::fs::copy(basis.join(p), out.join("basis").join(p)).map_err(|e| e.to_string())?;
    }
    save(
        &out.join("manifest.json"),
        &json!({"profile":PROFILE,"protocol":COMPLETE_PROTOCOL,"family_version":10,"instruction_version":11,"sources":sources,"cells":setups,
        "development_only":setups[0]["cell"]["development_only"],"training_admitted":false,"format":"canonical UTF-8 JSONL compressed with deterministic gzip, plus uncompressed canonical CSV","frozen_before_measurement":true,"resources":serde_json::from_slice::<Value>(&std::fs::read(root.join("docs/ere_complete_long_gates_v1.json")).unwrap()).unwrap()["proposed_resources"]}),
    )?;
    auditor(out, "freeze", false, 900.)?;
    Ok(())
}
fn auditor(out: &Path, stage: &str, development: bool, seconds: f64) -> Res<f64> {
    let start = Instant::now();
    let mut cmd = std::process::Command::new("python3");
    cmd.arg("-B")
        .arg(out.join("source_snapshots/scripts/verify_ere_complete_long_gates.py"))
        .arg(out)
        .arg("--stage")
        .arg(stage)
        .arg("--seconds")
        .arg(seconds.to_string());
    if development {
        cmd.arg("--development");
    }
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            demand(status.success(), "frozen standalone auditor failed")?;
            return Ok(start.elapsed().as_secs_f64());
        }
        if start.elapsed().as_secs_f64() >= seconds {
            let _ = child.kill();
            let _ = child.wait();
            return Err("shared auditor time cap".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}
fn campaign(out: &Path, development: bool) -> Res<()> {
    std::fs::create_dir(out).map_err(|e| format!("fresh directory required: {e}"))?;
    let result = (|| {
        let cells = matrix(development, 64);
        let mut fixtures: Vec<_> = cells.iter().map(fixture).collect();
        let setups: Vec<_> = cells
            .iter()
            .zip(fixtures.iter_mut())
            .map(|(c, f)| setup_value(c, f))
            .collect();
        freeze(out, &setups)?;
        let mut files = vec![];
        paths(out, &mut files)?;
        let bytes = files.iter().map(|p| p.metadata().unwrap().len()).sum();
        let mut lim = Limits {
            start: Instant::now(),
            out: out.into(),
            bytes,
            auditor_seconds: 0.,
            wall: 3600.,
            output: 2147483648,
        };
        let mut receipts = vec![];
        for (i, (c, f)) in cells.iter().zip(fixtures).enumerate() {
            if i == 4 {
                for fixture in ["identity_dormant", "oscillator_active"] {
                    use std::io::Read;
                    let mut a = File::open(out.join(format!("{fixture}_A0/trajectory.csv")))
                        .map_err(|e| e.to_string())?;
                    let mut b = File::open(out.join(format!("{fixture}_A1/trajectory.csv")))
                        .map_err(|e| e.to_string())?;
                    let mut left = [0u8; 16384];
                    let mut right = [0u8; 16384];
                    loop {
                        let n = a.read(&mut left).map_err(|e| e.to_string())?;
                        let m = b.read(&mut right).map_err(|e| e.to_string())?;
                        demand(
                            n == m && left[..n] == right[..m],
                            "native G-R3 CSV divergence",
                        )?;
                        if n == 0 {
                            break;
                        }
                    }
                }
                save(
                    &out.join("progress.json"),
                    &json!({"G-R3":"native complete awaiting audit","G-R5":"not_started","cells":receipts}),
                )?;
                lim.auditor_seconds +=
                    auditor(out, "GR3", development, 900. - lim.auditor_seconds)?;
                save(
                    &out.join("stage_barrier.json"),
                    &json!({"G-R3":"native and frozen stage audit passed","G-R5":"not_started","native_cells":receipts,"advancing_steps_before_G_R5":if development{256}else{400000},"stage_report":"audit_GR3.json","development_only":development}),
                )?;
            }
            receipts.push(run_cell(c, f, &mut lim, true)?);
            save(
                &out.join("progress.json"),
                &json!({"cells":receipts,"accounted_bytes":lim.bytes,"elapsed_seconds":lim.start.elapsed().as_secs_f64()}),
            )?;
        }
        lim.auditor_seconds += auditor(out, "full", development, 900. - lim.auditor_seconds)?;
        let mut all = vec![];
        paths(out, &mut all)?;
        demand(
            all.iter().map(|p| p.metadata().unwrap().len()).sum::<u64>() <= lim.output,
            "final aggregate output cap",
        )?;
        save(
            &out.join("run_status.json"),
            &json!({"status":"complete","development_only":development,"gate_pass":!development,"training_admitted":false,"advancing_engine_steps":receipts.iter().map(|r|r["completed_steps"].as_u64().unwrap()).sum::<u64>(),"cells":receipts,"elapsed_seconds":lim.start.elapsed().as_secs_f64(),"auditor_seconds":lim.auditor_seconds,"long_run_launched":!development}),
        )?;
        lim.auditor_seconds += auditor(out, "seal", development, 900. - lim.auditor_seconds)?;
        Ok(())
    })();
    if let Err(e) = &result {
        let _ = save(
            &out.join("run_status.json"),
            &json!({"status":"failed_or_unmeasured","error":e,"gate_pass":false,"training_admitted":false,"development_only":development,"G-R5":if out.join("A2_fuel0_ownerfalse").exists(){"partial"}else{"not_started"}}),
        );
    }
    result
}
#[test]
fn short_horizons_and_instrumentation_noninterference() {
    for horizon in [8, 16, 64] {
        for c in matrix(true, horizon)
            .into_iter()
            .filter(|c| c.gr3() || horizon < 64)
        {
            let mut hashes = vec![];
            for instrument in [false, true] {
                let tmp = tempfile::tempdir().unwrap();
                let mut lim = Limits {
                    start: Instant::now(),
                    out: tmp.path().into(),
                    bytes: 0,
                    auditor_seconds: 0.,
                    wall: 60.,
                    output: 100000000,
                };
                let receipt = run_cell(&c, fixture(&c), &mut lim, instrument).unwrap();
                assert_eq!(receipt["completed_steps"], horizon);
                hashes.push(std::fs::read(tmp.path().join(&c.id).join("steps.jsonl.gz")).unwrap());
            }
            assert_eq!(
                hashes[0], hashes[1],
                "observer/window instrumentation changed trajectory"
            );
        }
    }
}
#[test]
fn observer_and_report_probe_noninterference() {
    for horizon in [8, 16, 64] {
        let c = matrix(true, horizon)
            .into_iter()
            .find(|c| c.arm == EreArm::A3 && c.fuel == 8. && c.owner)
            .unwrap();
        let mut a = fixture(&c);
        let mut b = fixture(&c);
        let captured = Arc::new(Mutex::new(None));
        a.engine.add_observer(Box::new(Capture(captured.clone())));
        for step in 1..=horizon {
            a.engine.compile_at_boundary(&a.store);
            let before = a
                .engine
                .compile_report()
                .map(|r| serde_json::to_value(r).unwrap());
            assert_eq!(
                before,
                a.engine
                    .compile_report()
                    .map(|r| serde_json::to_value(r).unwrap())
            );
            a.runtime.before_step(&a.store, &mut a.engine).unwrap();
            assert!(
                a.engine.compile_report().is_none(),
                "read-only accessor consumed/replaced runtime receipt"
            );
            b.runtime.before_step(&b.store, &mut b.engine).unwrap();
            let sa = a.engine.step(&mut a.store);
            let sb = b.engine.step(&mut b.store);
            assert_eq!(
                serde_json::to_value(&sa).unwrap(),
                serde_json::to_value(&sb).unwrap()
            );
            assert_eq!(
                captured.lock().unwrap().take().unwrap()["stats"],
                serde_json::to_value(&sa).unwrap()
            );
            a.runtime.after_step(&a.store, &sa);
            b.runtime.after_step(&b.store, &sb);
            if step % 8 == 0 {
                a.runtime
                    .close_window(&a.store, MetricsRow::default(), MetricsRow::default());
            }
            assert_eq!(witness::inventory(&a.store), witness::inventory(&b.store));
            assert_eq!(a.engine.rng_state_probe(), b.engine.rng_state_probe());
            assert_eq!(a.engine.match_budget_usage(), b.engine.match_budget_usage());
            assert_eq!(a.engine.drain_guard_stats(), b.engine.drain_guard_stats());
        }
    }
}
#[test]
fn resource_caps_fail_closed() {
    let c = matrix(true, 8).remove(0);
    let tmp = tempfile::tempdir().unwrap();
    let f = fixture(&c);
    let mut lim = Limits {
        start: Instant::now(),
        out: tmp.path().into(),
        bytes: 0,
        auditor_seconds: 0.,
        wall: 60.,
        output: 1,
    };
    assert!(run_cell(&c, f, &mut lim, true)
        .unwrap_err()
        .contains("output cap"));
    lim.output = 100000000;
    lim.wall = 0.;
    assert!(lim
        .check(&fixture(&c).store)
        .unwrap_err()
        .contains("wall cap"));
    let tmp = tempfile::tempdir().unwrap();
    lim.out = tmp.path().into();
    lim.wall = 60.;
    lim.bytes = 0;
    let mut capped = fixture(&c);
    capped
        .engine
        .set_match_budget_policy(1, 1, MatchBudgetPolicy::Pooled);
    assert!(run_cell(&c, capped, &mut lim, true)
        .unwrap_err()
        .contains("nonadvancing"));
    let mut large = HypergraphStore::new();
    for _ in 0..129 {
        large.add_node(NodeData::new(Label(99), 0));
    }
    assert!(lim.check(&large).unwrap_err().contains("store size cap"));
}
#[test]
#[ignore = "development-only frozen 64-step controls; never a long gate pass"]
fn long_gates_development_smoke() {
    let out = PathBuf::from(
        std::env::var_os("ERE_LONG_GATES_SMOKE_OUTPUT")
            .expect("fresh ERE_LONG_GATES_SMOKE_OUTPUT required"),
    );
    campaign(&out, true).unwrap();
}
#[test]
#[ignore = "requires separate explicit approval of the 540000-step resource proposal"]
fn long_gates_frozen_assay() {
    assert_eq!(
        std::env::var("ERE_LONG_GATES_APPROVED_STEPS").as_deref(),
        Ok("540000"),
        "full campaign requires separately approved budget"
    );
    let out = PathBuf::from(
        std::env::var_os("ERE_LONG_GATES_OUTPUT").expect("fresh ERE_LONG_GATES_OUTPUT required"),
    );
    campaign(&out, false).unwrap();
}
