//! Integrated engineering capacity only. No scientific campaign entry point.
pub mod lineage;
pub mod population;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyOp, PropertyValue};
use hg_engine::{apply_binding, find_bindings};
use hg_observe::Binding;
use hg_rules::{
    compile_rule, CompiledRule, EffectOpDecl, NodePropertyGuard, PropertyEffectDecl, Rule,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use slotmap::Key;
use smallvec::SmallVec;
use std::collections::BTreeSet;

pub const PROFILE: &str = "cumulative-readiness-v1";
pub const WALLET: u32 = 100;
pub const GENE: u32 = 200;
pub const EDITOR: u32 = 300;
pub const OWN_GENE: u32 = 1000;
pub const OWN_EDITOR: u32 = 1001;
pub const OWN_PARCEL: u32 = 1002;
pub const CAPITAL: u32 = 1;
pub const STAGE: u32 = 2;
pub const SERIAL: u32 = 3;
pub const EDIT_MODE: u32 = 4;
pub const VALUES: [i64; 5] = [0, 4, 8, 14, 16];
pub const MAX_GENES: usize = 8;
pub const MAX_NODES: usize = 16;
pub const MAX_EDGES: usize = 16;

thread_local! { static NATIVE_WORK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
pub fn native_work() -> usize {
    NATIVE_WORK.with(|n| n.get())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    Endogenous,
    External,
    Nonheritable,
    Frozen,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Genome {
    pub stages: Vec<i64>,
    pub editor_mode: i64,
}

#[derive(Clone)]
pub struct Cell {
    pub graph: HypergraphStore,
    pub wallet: NodeId,
    pub editor: NodeId,
    pub id: u64,
    pub parent: Option<u64>,
    pub generation: u32,
    pub arm: Arm,
    pub seed: u64,
    pub word_pos: u128,
    pub environment: i64,
    pub dissipated: i64,
    pub transferred: i64,
    pub initial: i64,
    pub next_serial: i64,
    pub tick: u64,
    pub rewrites: usize,
    pub baseline: Genome,
    pub receipts: Vec<Value>,
}

fn int(g: &HypergraphStore, n: NodeId, key: u32) -> Result<i64, String> {
    match g.get_node(n).and_then(|d| d.get_property(key)) {
        Some(PropertyValue::Int(v)) => Ok(v),
        _ => Err(format!("missing/wrong integer {key}")),
    }
}
fn node(g: &mut HypergraphStore, label: u32, props: &[(u32, i64)]) -> NodeId {
    let mut d = NodeData::new(Label(label), g.step());
    for &(key, value) in props {
        d.set_property(key, PropertyValue::Int(value));
    }
    g.add_node(d)
}
fn edge(g: &mut HypergraphStore, label: u32, a: NodeId, b: NodeId) {
    g.add_edge(HyperEdge::new(Label(label), SmallVec::from_slice(&[a, b])));
}
fn effect(r: &mut Rule, var: &str, key: u32, op: EffectOpDecl) {
    r.rhs_property_effects.push(PropertyEffectDecl {
        var: var.into(),
        key,
        op,
    });
}
fn guard(r: &mut Rule, var: &str, key: u32, op: PropertyOp, value: i64) {
    r.lhs_node_property_guards.push(NodePropertyGuard {
        var: var.into(),
        property: key,
        op,
        value: PropertyValue::Int(value),
    });
}
fn base_rule(name: &str, vars: &[(&str, u32)], links: &[(&str, &str, u32)]) -> Rule {
    let mut r = Rule {
        name: name.into(),
        ..Rule::default()
    };
    for &(v, l) in vars {
        r.lhs.nodes.push(v.into());
        r.rhs.nodes.push(v.into());
        r.lhs.node_labels.insert(v.into(), Label(l));
    }
    for &(a, b, l) in links {
        let ix = r.lhs.edges.len();
        r.lhs.edges.push(vec![a.into(), b.into()]);
        r.lhs.edge_labels.insert(ix, Label(l));
        r.rhs.edges.push(vec![a.into(), b.into()]);
        r.rhs.edge_labels.insert(ix, Label(l));
    }
    r
}

impl Cell {
    pub fn new(stages: &[i64], arm: Arm, seed: u64) -> Result<Self, String> {
        Self::with_offset(stages, arm, seed, 0)
    }
    pub fn with_offset(stages: &[i64], arm: Arm, seed: u64, offset: usize) -> Result<Self, String> {
        if stages.is_empty()
            || stages.len() > MAX_GENES
            || stages.iter().any(|s| !(-1..=4).contains(s))
            || offset > 32
        {
            return Err("genome/offset domain".into());
        }
        let mut graph = HypergraphStore::new();
        let dummy: Vec<_> = (0..offset).map(|_| node(&mut graph, 999, &[])).collect();
        for n in dummy {
            graph.remove_node(n);
        }
        let wallet = node(&mut graph, WALLET, &[(CAPITAL, 40)]);
        let editor = node(&mut graph, EDITOR, &[(EDIT_MODE, 0)]);
        edge(&mut graph, OWN_EDITOR, wallet, editor);
        for (serial, &stage) in stages.iter().enumerate() {
            let gene = node(&mut graph, GENE, &[(STAGE, stage), (SERIAL, serial as i64)]);
            edge(&mut graph, OWN_GENE, wallet, gene);
        }
        let baseline = Genome {
            stages: stages.to_vec(),
            editor_mode: 0,
        };
        let s = Self {
            graph,
            wallet,
            editor,
            id: 0,
            parent: None,
            generation: 0,
            arm,
            seed,
            word_pos: 0,
            environment: 512,
            dissipated: 0,
            transferred: 0,
            initial: 552,
            next_serial: stages.len() as i64,
            tick: 0,
            rewrites: 0,
            baseline,
            receipts: Vec::new(),
        };
        s.validate()?;
        Ok(s)
    }
    pub fn capital(&self) -> Result<i64, String> {
        int(&self.graph, self.wallet, CAPITAL)
    }
    fn genes(&self) -> Result<Vec<(i64, NodeId, i64)>, String> {
        let mut genes = Vec::new();
        for (n, d) in self.graph.nodes() {
            if d.label.0 == GENE {
                genes.push((int(&self.graph, n, SERIAL)?, n, int(&self.graph, n, STAGE)?));
            }
        }
        genes.sort_by_key(|(serial, _, _)| *serial);
        Ok(genes)
    }
    pub fn genome(&self) -> Result<Genome, String> {
        Ok(Genome {
            stages: self.genes()?.iter().map(|(_, _, s)| *s).collect(),
            editor_mode: int(&self.graph, self.editor, EDIT_MODE)?,
        })
    }
    pub fn state(&self) -> Result<Value, String> {
        let mut nodes: Vec<_> = self
            .graph
            .nodes()
            .map(|(id, d)| {
                json!({"id":id.data().as_ffi(),"label":d.label.0,
            "born":d.birth_step,"properties":d.sorted_properties()})
            })
            .collect();
        nodes.sort_by_key(|n| n["id"].as_u64().unwrap());
        let mut edges: Vec<_> = self
            .graph
            .edges()
            .map(|(id, e)| {
                json!({"id":id.data().as_ffi(),"label":e.label.0,
            "nodes":e.nodes.iter().map(|n|n.data().as_ffi()).collect::<Vec<_>>()})
            })
            .collect();
        edges.sort_by_key(|e| e["id"].as_u64().unwrap());
        Ok(
            json!({"cell":self.id,"parent":self.parent,"generation":self.generation,"arm":self.arm,
            "genome":self.genome()?,"capital":self.capital()?,"environment":self.environment,
            "dissipated":self.dissipated,"transferred":self.transferred,"initial":self.initial,
            "tick":self.tick,"rewrites":self.rewrites,"seed":self.seed,"word_pos":self.word_pos.to_string(),
            "next_serial":self.next_serial,"wallet":self.wallet,"editor":self.editor,
            "allocator":self.graph.allocator_order(),"graph_step":self.graph.step(),"nodes":nodes,"edges":edges}),
        )
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.graph.node_count() > MAX_NODES
            || self.graph.edge_count() > MAX_EDGES
            || self.genes()?.len() > MAX_GENES
        {
            return Err("cell cap".into());
        }
        if self.graph.get_node(self.wallet).map(|d| d.label.0) != Some(WALLET)
            || self.graph.get_node(self.editor).map(|d| d.label.0) != Some(EDITOR)
            || !(0..=1).contains(&int(&self.graph, self.editor, EDIT_MODE)?)
        {
            return Err("wallet/editor schema".into());
        }
        let genes = self.genes()?;
        if self.next_serial != genes.len() as i64
            || self.word_pos >= (1u128 << 68)
            || self.environment > 512
            || self.environment % 16 != 0
            || self.tick > 32
            || !matches!(self.initial, 520 | 552)
            || self.baseline.stages.is_empty()
            || self.baseline.stages.len() > MAX_GENES
            || self.baseline.stages.iter().any(|s| !(-1..=4).contains(s))
            || !(0..=1).contains(&self.baseline.editor_mode)
        {
            return Err("continuation/baseline domain".into());
        }
        let mut serials = BTreeSet::new();
        for (serial, _, stage) in &genes {
            if !serials.insert(*serial)
                || *serial < 0
                || *serial >= self.next_serial
                || !(-1..=4).contains(stage)
            {
                return Err("gene schema".into());
            }
        }
        let mut parcel = 0;
        for (n, d) in self.graph.nodes() {
            let expected = match d.label.0 {
                WALLET if n == self.wallet => vec![CAPITAL],
                EDITOR if n == self.editor => vec![EDIT_MODE],
                GENE => vec![STAGE, SERIAL],
                0..=4 => {
                    parcel += 1;
                    vec![]
                }
                _ => return Err("unexpected node".into()),
            };
            let mut keys: Vec<_> = d.properties.iter().map(|(k, _)| *k).collect();
            keys.sort();
            if keys != expected || d.energy != 0. || d.birth_step > self.tick {
                return Err("node properties/birth/energy".into());
            }
        }
        if parcel > 1 {
            return Err("parcel multiplicity".into());
        }
        let mut owned = BTreeSet::new();
        for (_, e) in self.graph.edges() {
            if e.nodes.len() != 2
                || e.nodes[0] != self.wallet
                || !owned.insert(e.nodes[1].data().as_ffi())
            {
                return Err("ownership shape".into());
            }
            let label = self
                .graph
                .get_node(e.nodes[1])
                .ok_or("missing endpoint")?
                .label
                .0;
            if !matches!(
                (e.label.0, label),
                (OWN_GENE, GENE) | (OWN_EDITOR, EDITOR) | (OWN_PARCEL, 0..=4)
            ) {
                return Err("ownership role".into());
            }
        }
        if owned.len() != self.graph.node_count() - 1 {
            return Err("unowned metadata/material".into());
        }
        if [
            self.capital()?,
            self.environment,
            self.dissipated,
            self.transferred,
        ]
        .iter()
        .any(|x| *x < 0 || *x > self.initial)
            || i128::from(self.capital()?)
                + i128::from(self.environment)
                + i128::from(self.dissipated)
                + i128::from(self.transferred)
                + i128::from(parcel * 16)
                != i128::from(self.initial)
        {
            return Err("energy conservation".into());
        }
        if self.graph.step() != self.tick || self.rewrites > 4000 {
            return Err("clock/work cap".into());
        }
        Ok(())
    }
    fn rng(&self) -> ChaCha8Rng {
        let mut r = ChaCha8Rng::seed_from_u64(self.seed);
        r.set_word_pos(self.word_pos);
        r
    }
    fn apply(
        &mut self,
        r: &CompiledRule,
        b: &Binding,
        kind: &str,
        extra: Value,
    ) -> Result<(), String> {
        let before = self.state()?;
        let mut rng = self.rng();
        let (event, applied, skipped) = apply_binding(&mut self.graph, r, b, &mut rng);
        NATIVE_WORK.with(|n| n.set(n.get() + 1));
        if skipped != 0 {
            return Err("native property effect skipped".into());
        }
        self.word_pos = rng.get_word_pos();
        self.rewrites += 1;
        self.receipts.push(
            json!({"kind":kind,"before":before,"rule":r.name,"bound":b.bound_nodes,
            "created":event.nodes_created,"deleted":event.nodes_deleted,"effects":applied,
            "extra":extra,"native_after":self.state()?}),
        );
        Ok(())
    }
    fn charge(&mut self, amount: i64, kind: &str) -> Result<(), String> {
        if amount < 0 || self.capital()? < amount {
            return Err(format!("unfunded {kind}"));
        }
        let before = self.state()?;
        let c = self.capital()?;
        self.graph
            .get_node_mut(self.wallet)
            .unwrap()
            .set_property(CAPITAL, PropertyValue::Int(c - amount));
        self.dissipated += amount;
        self.receipts
            .push(json!({"kind":kind,"amount":amount,"before":before,"after":self.state()?}));
        self.validate()
    }
    fn metabolic_rule(stage: i64, serial: i64) -> Result<CompiledRule, String> {
        let mut r = base_rule(
            &format!("metabolize-{serial}"),
            &[("w", WALLET), ("g", GENE), ("p", stage as u32)],
            &[("w", "g", OWN_GENE), ("w", "p", OWN_PARCEL)],
        );
        guard(&mut r, "g", SERIAL, PropertyOp::Eq, serial);
        guard(&mut r, "g", STAGE, PropertyOp::Eq, stage);
        guard(&mut r, "w", CAPITAL, PropertyOp::Gte, 1);
        r.rhs_node_labels
            .insert("p".into(), Label(stage as u32 + 1));
        effect(
            &mut r,
            "w",
            CAPITAL,
            EffectOpDecl::Subtract {
                value: PropertyValue::Int(1),
            },
        );
        compile_rule(&r).map_err(|e| e.to_string())
    }
    pub fn cycle(&mut self, reverse_rules: bool) -> Result<(), String> {
        self.validate()?;
        if self.environment < 16 {
            return Err("finite environment exhausted".into());
        }
        self.charge(1 + self.genes()?.len() as i64, "storage")?;
        let before = self.state()?;
        self.environment -= 16;
        let p = node(&mut self.graph, 0, &[]);
        edge(&mut self.graph, OWN_PARCEL, self.wallet, p);
        self.receipts
            .push(json!({"kind":"supply","before":before,"after":self.state()?}));
        for _ in 0..4 {
            let stage = self.graph.get_node(p).unwrap().label.0 as i64;
            let mut rules: Vec<_> = self
                .genes()?
                .into_iter()
                .filter(|(_, _, s)| (0..4).contains(s))
                .map(|(serial, _, s)| Self::metabolic_rule(s, serial).map(|r| (serial, r)))
                .collect::<Result<_, _>>()?;
            if reverse_rules {
                rules.reverse();
            }
            let mut candidates = Vec::new();
            for (serial, r) in rules {
                let bindings = find_bindings(&self.graph.snapshot(), &r, 0);
                if bindings.len() > MAX_GENES {
                    return Err("enumeration cap".into());
                }
                for b in bindings {
                    candidates.push((serial, r.clone(), b));
                }
            }
            candidates.sort_by_key(|(serial, _, _)| *serial);
            if candidates.is_empty() {
                break;
            }
            let opportunities: Vec<_> = candidates.iter().map(|(serial, _, _)| *serial).collect();
            let (serial, r, b) = candidates.remove(0);
            self.apply(
                &r,
                &b,
                "metabolic",
                json!({"serial":serial,"stage":stage,"opportunities":opportunities}),
            )?;
            self.dissipated += 1;
            self.validate()?;
        }
        let stage = self.graph.get_node(p).unwrap().label.0;
        let payout = VALUES[stage as usize];
        let mut r = base_rule(
            "settle",
            &[("w", WALLET), ("p", stage)],
            &[("w", "p", OWN_PARCEL)],
        );
        r.rhs.nodes.retain(|x| x != "p");
        r.rhs.edges.clear();
        r.rhs.edge_labels.clear();
        effect(
            &mut r,
            "w",
            CAPITAL,
            EffectOpDecl::Add {
                value: PropertyValue::Int(payout),
            },
        );
        let r = compile_rule(&r).map_err(|e| e.to_string())?;
        let bindings = find_bindings(&self.graph.snapshot(), &r, 0);
        if bindings.len() != 1 {
            return Err("settlement identity".into());
        }
        self.apply(
            &r,
            &bindings[0],
            "settle",
            json!({"stage":stage,"payout":payout}),
        )?;
        self.dissipated += 16 - payout;
        self.tick += 1;
        self.graph.set_step(self.tick);
        self.validate()?;
        self.receipts
            .push(json!({"kind":"boundary","after":self.state()?}));
        Ok(())
    }
    fn editor_rule(&self) -> Result<CompiledRule, String> {
        let mut r = base_rule(
            "duplicate-gene",
            &[("w", WALLET), ("e", EDITOR), ("g", GENE)],
            &[("w", "e", OWN_EDITOR), ("w", "g", OWN_GENE)],
        );
        guard(&mut r, "w", CAPITAL, PropertyOp::Gte, 2);
        guard(&mut r, "g", STAGE, PropertyOp::Gte, 0);
        guard(&mut r, "g", STAGE, PropertyOp::Lt, 4);
        r.rhs.nodes.push("c".into());
        r.rhs.node_labels.insert("c".into(), Label(GENE));
        let ix = r.rhs.edges.len();
        r.rhs.edges.push(vec!["w".into(), "c".into()]);
        r.rhs.edge_labels.insert(ix, Label(OWN_GENE));
        effect(
            &mut r,
            "c",
            STAGE,
            EffectOpDecl::CopyFrom {
                source_var: "g".into(),
                source_key: STAGE,
            },
        );
        let op = match int(&self.graph, self.editor, EDIT_MODE)? {
            0 => EffectOpDecl::Jitter { magnitude: 1. },
            1 => EffectOpDecl::Add {
                value: PropertyValue::Int(1),
            },
            _ => return Err("editor mode".into()),
        };
        effect(&mut r, "c", STAGE, op);
        effect(
            &mut r,
            "c",
            SERIAL,
            EffectOpDecl::Set {
                value: PropertyValue::Int(self.next_serial),
            },
        );
        effect(
            &mut r,
            "w",
            CAPITAL,
            EffectOpDecl::Subtract {
                value: PropertyValue::Int(2),
            },
        );
        compile_rule(&r).map_err(|e| e.to_string())
    }
    /// A supplied serial is permitted only for explicitly authored accessibility probes.
    /// Autonomous selection uses all admitted matches, independent of utility.
    pub fn edit(&mut self, external: bool, probe_serial: Option<i64>) -> Result<(), String> {
        self.validate()?;
        if (!external && matches!(self.arm, Arm::External | Arm::Frozen))
            || (external && self.arm != Arm::External)
        {
            return Err("arm forbids editing phase".into());
        }
        if self.genes()?.len() >= MAX_GENES
            || self.graph.node_count() + 1 > MAX_NODES
            || self.graph.edge_count() + 1 > MAX_EDGES
        {
            return Err("edit capacity bound".into());
        }
        let r = self.editor_rule()?;
        let mut bs = find_bindings(&self.graph.snapshot(), &r, 0);
        if bs.is_empty() || bs.len() > MAX_GENES {
            return Err("unfunded/unsupported edit".into());
        }
        bs.sort_by_key(|b| int(&self.graph, b.bound_nodes[2], SERIAL).unwrap());
        let opportunities: Vec<_> = bs
            .iter()
            .map(|b| int(&self.graph, b.bound_nodes[2], SERIAL).unwrap())
            .collect();
        let selected = if let Some(serial) = probe_serial {
            bs.iter()
                .position(|b| int(&self.graph, b.bound_nodes[2], SERIAL) == Ok(serial))
                .ok_or("probe absent")?
        } else {
            let before = self.state()?;
            let mut rng = self.rng();
            let i = rng.gen_range(0..bs.len());
            self.word_pos = rng.get_word_pos();
            self.receipts.push(json!({"kind":"selection","before":before,"after":self.state()?,
                "opportunities":opportunities,"selected_serial":opportunities[i],"external":external}));
            i
        };
        let b = bs.remove(selected);
        self.apply(&r,&b,if external{"external_edit"}else{"endogenous_edit"},
            json!({"probe":probe_serial.is_some(),"selected_serial":opportunities[selected],"opportunities":opportunities}))?;
        self.next_serial += 1;
        self.dissipated += 2;
        self.validate()?;
        self.receipts
            .push(json!({"kind":"edit_boundary","after":self.state()?}));
        Ok(())
    }
    pub fn reproduce(&mut self, child_id: u64) -> Result<Self, String> {
        self.validate()?;
        if child_id == self.id {
            return Err("child identity reused".into());
        }
        let inherited = self.genome()?;
        let cost = 2 + inherited.stages.len() as i64 + 1;
        if self.capital()? < cost + 8 {
            return Err("unfunded reproduction".into());
        }
        let before = self.state()?;
        let genome = if self.arm == Arm::Nonheritable {
            self.baseline.clone()
        } else {
            inherited.clone()
        };
        let mut child = Self::new(&genome.stages, self.arm, self.seed.wrapping_add(child_id))?;
        child
            .graph
            .get_node_mut(child.wallet)
            .unwrap()
            .set_property(CAPITAL, PropertyValue::Int(8));
        child
            .graph
            .get_node_mut(child.editor)
            .unwrap()
            .set_property(EDIT_MODE, PropertyValue::Int(genome.editor_mode));
        child.id = child_id;
        child.parent = Some(self.id);
        child.generation = self.generation + 1;
        child.initial = 520;
        child.baseline = self.baseline.clone();
        let capital = self.capital()?;
        self.graph
            .get_node_mut(self.wallet)
            .unwrap()
            .set_property(CAPITAL, PropertyValue::Int(capital - cost - 8));
        self.dissipated += cost;
        self.transferred += 8;
        // Each isolated diagnostic descendant receives a separately declared finite test environment.
        // Shared-world resource inheritance is intentionally not claimed here.
        child.validate()?;
        self.validate()?;
        self.receipts.push(
            json!({"kind":"birth","before":before,"after":self.state()?,"child":child.state()?,
            "copy_cost":cost,"inherited":inherited,"transmitted":genome}),
        );
        Ok(child)
    }
    pub fn checkpoint(&self) -> Result<Value, String> {
        self.validate()?;
        Ok(
            json!({"profile":PROFILE,"graph":self.graph,"allocator":self.graph.allocator_order(),
            "wallet":self.wallet,"editor":self.editor,"id":self.id,"parent":self.parent,"generation":self.generation,
            "arm":self.arm,"seed":self.seed,"word_pos":self.word_pos.to_string(),"environment":self.environment,
            "dissipated":self.dissipated,"transferred":self.transferred,"initial":self.initial,
            "next_serial":self.next_serial,"tick":self.tick,"rewrites":self.rewrites,"baseline":self.baseline}),
        )
    }
    pub fn restore(v: Value) -> Result<Self, String> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            profile: String,
            graph: HypergraphStore,
            allocator: (Vec<u32>, Vec<u32>),
            wallet: NodeId,
            editor: NodeId,
            id: u64,
            parent: Option<u64>,
            generation: u32,
            arm: Arm,
            seed: u64,
            word_pos: String,
            environment: i64,
            dissipated: i64,
            transferred: i64,
            initial: i64,
            next_serial: i64,
            tick: u64,
            rewrites: usize,
            baseline: Genome,
        }
        let s: Saved = serde_json::from_value(v).map_err(|e| e.to_string())?;
        if s.profile != PROFILE {
            return Err("checkpoint profile".into());
        }
        let mut graph = s.graph;
        graph
            .restore_allocator_order(&s.allocator)
            .map_err(str::to_string)?;
        let cell = Self {
            graph,
            wallet: s.wallet,
            editor: s.editor,
            id: s.id,
            parent: s.parent,
            generation: s.generation,
            arm: s.arm,
            seed: s.seed,
            word_pos: s.word_pos.parse().map_err(|_| "RNG cursor")?,
            environment: s.environment,
            dissipated: s.dissipated,
            transferred: s.transferred,
            initial: s.initial,
            next_serial: s.next_serial,
            tick: s.tick,
            rewrites: s.rewrites,
            baseline: s.baseline,
            receipts: Vec::new(),
        };
        cell.validate()?;
        Ok(cell)
    }
}

pub fn fixture() -> Result<Value, String> {
    let work_start = native_work();
    let contract: Value =
        serde_json::from_str(include_str!("../contract.json")).map_err(|e| e.to_string())?;
    let seed = contract["development_seed"].as_u64().unwrap();
    let mut cases = Vec::new();
    for case in contract["capacity_cases"].as_array().unwrap() {
        let stages: Vec<_> = case["stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect();
        let mut c = Cell::new(&stages, Arm::Frozen, seed)?;
        let initial = c.state()?;
        for _ in 0..12 {
            c.cycle(false)?;
        }
        cases.push(
            json!({"name":case["name"],"initial":initial,"final":c.state()?,"receipts":c.receipts}),
        );
    }
    let mut c = Cell::new(&[0], Arm::Endogenous, seed)?;
    // Fixed deterministic editor probe, externally requested; never discovery evidence.
    c.graph
        .get_node_mut(c.editor)
        .unwrap()
        .set_property(EDIT_MODE, PropertyValue::Int(1));
    let initial = c.state()?;
    c.edit(false, Some(0))?;
    c.edit(false, Some(1))?;
    if c.genome()?.stages != vec![0, 1, 2] {
        return Err("accessibility witness".into());
    }
    for _ in 0..12 {
        c.cycle(false)?;
    }
    cases.push(json!({"name":"paid_accessibility","initial":initial,"final":c.state()?,"receipts":c.receipts}));
    for (name, arm) in [
        ("endogenous", Arm::Endogenous),
        ("external", Arm::External),
        ("nonheritable", Arm::Nonheritable),
        ("frozen", Arm::Frozen),
    ] {
        let mut c = Cell::new(&[0], arm, seed)?;
        let initial = c.state()?;
        if matches!(arm, Arm::Endogenous | Arm::Nonheritable) {
            c.edit(false, None)?;
        }
        for _ in 0..4 {
            c.cycle(false)?;
        }
        let mut child = c.reproduce(1)?;
        if arm == Arm::External {
            child.edit(true, None)?;
        }
        for _ in 0..4 {
            child.cycle(false)?;
        }
        cases.push(json!({"name":format!("arm-{name}"),"initial":initial,"final":c.state()?,"receipts":c.receipts,
            "child_initial":child.receipts.first().map(|r|r["before"].clone()),"child_final":child.state()?,"child_receipts":child.receipts}));
    }
    let mut c = Cell::new(&[0, 1, 2], Arm::Endogenous, seed)?;
    let mut transmissions = Vec::new();
    for generation in 1..=3 {
        let initial = c.state()?;
        for _ in 0..4 {
            c.cycle(false)?;
        }
        let child = c.reproduce(generation)?;
        transmissions.push(json!({"initial":initial,"final":c.state()?,"receipts":c.receipts}));
        c = child;
    }
    // Checkpoint includes allocator continuation and both selection/jitter RNG draws.
    let mut a = Cell::new(&[0, 1], Arm::Endogenous, seed)?;
    a.edit(false, None)?;
    a.cycle(false)?;
    let saved = a.checkpoint()?;
    let mut b = Cell::restore(saved.clone())?;
    a.receipts.clear();
    a.edit(false, None)?;
    b.edit(false, None)?;
    a.cycle(false)?;
    b.cycle(false)?;
    if a.state()? != b.state()? || a.receipts != b.receipts {
        return Err("checkpoint continuation".into());
    }
    let mut normal = Cell::new(&[0, 1, 2, 0], Arm::Frozen, seed)?;
    let mut reverse = normal.clone();
    normal.cycle(false)?;
    reverse.cycle(true)?;
    if normal.state()? != reverse.state()? || normal.receipts != reverse.receipts {
        return Err("candidate order".into());
    }
    let mut offset = Cell::with_offset(&[0, 1, 2, 0], Arm::Frozen, seed, 7)?;
    offset.cycle(false)?;
    if normal.capital()? != offset.capital()?
        || normal.genome()? != offset.genome()?
        || normal.dissipated != offset.dissipated
    {
        return Err("allocation offset".into());
    }
    let actual_work = native_work() - work_start;
    if actual_work > 4000 {
        return Err("fixture work cap".into());
    }
    Ok(
        json!({"profile":PROFILE,"contract":contract,"cases":cases,"transmissions":transmissions,
        "continuation":{"checkpoint":saved,"receipts":a.receipts,"restored_receipts":b.receipts,"final":a.state()?,"restored_final":b.state()?},
        "order":{"receipts":normal.receipts,"reverse_receipts":reverse.receipts,"final":normal.state()?,"reverse_final":reverse.state()?},
        "offset":{"receipts":offset.receipts,"final":offset.state()?},"native_rewrites":actual_work,
        "engineering_fixture_pass":true,"readiness_gate_complete":false,"scientific_acceptance":false,
        "evolution_demonstrated":false,"campaign_launch_authorized":false}),
    )
}
