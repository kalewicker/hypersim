//! Bounded policy and program edits over the shared world's native chemistry.
//! Diagnostic compartments only; no scientific campaign entry point.
use super::*;
pub mod world;
pub const PROFILE: &str = "cumulative-editor-regulation-readiness-v1";
const ENABLED: u32 = 5;
const MASK: u32 = 6;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub genes: Vec<(i64, i64)>,
    pub next_serial: i64,
    pub enabled: i64,
    pub mask: i64,
}

#[derive(Clone)]
pub struct RegulatedCell {
    core: Cell,
    reservoir: i64,
    initial_reservoir: i64,
    endowment: i64,
    grants: u64,
    alive: bool,
    attempts: u64,
    baseline: Program,
    pub frames: Vec<Value>,
}

impl RegulatedCell {
    pub fn from_case(spec: &Value, seed: u64, offset: usize) -> Result<Self, String> {
        let stages: Vec<i64> =
            serde_json::from_value(spec["stages"].clone()).map_err(|e| e.to_string())?;
        let arm = serde_json::from_value(spec["arm"].clone()).map_err(|e| e.to_string())?;
        let mut core = Cell::with_offset(&stages, arm, seed, offset)?;
        let enabled = spec["enabled"].as_i64().ok_or("enabled")?;
        let mask = spec["mask"].as_i64().ok_or("mask")?;
        core.graph
            .get_node_mut(core.editor)
            .unwrap()
            .set_property(ENABLED, PropertyValue::Int(enabled));
        core.graph
            .get_node_mut(core.editor)
            .unwrap()
            .set_property(MASK, PropertyValue::Int(mask));
        let endowment = spec["capital"].as_i64().ok_or("capital")?;
        let reservoir = spec["reservoir"].as_i64().ok_or("reservoir")?;
        core.environment = 0;
        core.initial = endowment;
        set_capital(&mut core, endowment);
        if let Some(serial) = spec.get("next_serial") {
            core.next_serial = serial.as_i64().ok_or("next serial")?;
        }
        let baseline = Program {
            genes: stages
                .iter()
                .enumerate()
                .map(|(i, &s)| (i as i64, s))
                .collect(),
            next_serial: core.next_serial,
            enabled,
            mask,
        };
        let s = Self {
            core,
            reservoir,
            initial_reservoir: reservoir,
            endowment,
            grants: 0,
            alive: true,
            attempts: 0,
            baseline,
            frames: Vec::new(),
        };
        s.validate()?;
        Ok(s)
    }
    pub fn program(&self) -> Result<Program, String> {
        Ok(Program {
            genes: self
                .core
                .genes()?
                .iter()
                .map(|(id, _, s)| (*id, *s))
                .collect(),
            next_serial: self.core.next_serial,
            enabled: int(&self.core.graph, self.core.editor, ENABLED)?,
            mask: int(&self.core.graph, self.core.editor, MASK)?,
        })
    }
    pub fn state(&self) -> Result<Value, String> {
        Ok(
            json!({"profile":PROFILE,"core":self.core.state()?,"program":self.program()?,"baseline":self.baseline,
            "reservoir":self.reservoir,"initial_reservoir":self.initial_reservoir,"endowment":self.endowment,
            "grants":self.grants,"alive":self.alive,"attempts":self.attempts}),
        )
    }
    pub fn validate(&self) -> Result<(), String> {
        let c = &self.core;
        let p = self.program()?;
        let valid = |p: &Program| {
            !p.genes.is_empty()
                && p.genes.len() <= 8
                && (0..=32).contains(&p.next_serial)
                && (0..=1).contains(&p.enabled)
                && (0..=31).contains(&p.mask)
                && p.genes
                    .iter()
                    .all(|&(id, s)| id >= 0 && id < p.next_serial && (-1..=4).contains(&s))
                && p.genes.windows(2).all(|w| w[0].0 < w[1].0)
        };
        if !valid(&p)
            || !valid(&self.baseline)
            || c.tick > 16
            || self.attempts > 32
            || c.rewrites > 6000
            || c.word_pos >= 1u128 << 68
            || !(0..=64).contains(&self.endowment)
            || !(0..=512).contains(&self.initial_reservoir)
            || self.reservoir < 0
            || self.reservoir > self.initial_reservoir
            || self.reservoir % 16 != 0
            || self.initial_reservoir % 16 != 0
            || self.grants > 16
            || self.initial_reservoir - self.reservoir != 16 * self.grants as i64
            || c.environment != 0
            || c.initial != self.endowment + 16 * self.grants as i64
            || c.capital()? < 0
            || c.dissipated < 0
            || c.transferred < 0
            || i128::from(c.capital()?) + i128::from(c.dissipated) + i128::from(c.transferred)
                != i128::from(c.initial)
            || c.graph.step() != c.tick
            || (!self.alive && c.capital()? != 0)
            || c.graph.node_count() != p.genes.len() + 2
            || c.graph.edge_count() != p.genes.len() + 1
            || int(&c.graph, c.editor, EDIT_MODE)? != 0
        {
            return Err("regulated domain/energy/schema".into());
        }
        let mut owned = BTreeSet::new();
        for (_, e) in c.graph.edges() {
            if e.nodes.len() != 2 || e.nodes[0] != c.wallet || !owned.insert(e.nodes[1]) {
                return Err("regulated ownership".into());
            }
            let label = c
                .graph
                .get_node(e.nodes[1])
                .ok_or("missing target")?
                .label
                .0;
            if !matches!((e.label.0, label), (OWN_GENE, GENE) | (OWN_EDITOR, EDITOR)) {
                return Err("regulated ownership role".into());
            }
        }
        if c.graph.get_node(c.wallet).map(|n| n.label.0) != Some(WALLET)
            || c.graph.get_node(c.editor).map(|n| n.label.0) != Some(EDITOR)
        {
            return Err("regulated wallet/editor".into());
        }
        for (id, n) in c.graph.nodes() {
            let mut keys: Vec<_> = n.properties.iter().map(|(k, _)| *k).collect();
            keys.sort();
            let expected = if id == c.wallet {
                vec![CAPITAL]
            } else if id == c.editor {
                vec![EDIT_MODE, ENABLED, MASK]
            } else {
                vec![STAGE, SERIAL]
            };
            if keys != expected || n.energy != 0. || n.birth_step > c.tick {
                return Err("regulated node fields".into());
            }
            let mut actual = c.graph.edges_incident_to(id).to_vec();
            let mut expect: Vec<_> = c
                .graph
                .edges()
                .filter(|(_, e)| e.nodes.contains(&id))
                .map(|(k, _)| k)
                .collect();
            actual.sort_by_key(|k| k.data().as_ffi());
            expect.sort_by_key(|k| k.data().as_ffi());
            if actual != expect {
                return Err("regulated incidence".into());
            }
        }
        for label in [OWN_GENE, OWN_EDITOR, OWN_PARCEL] {
            let mut actual = c.graph.edges_with_label(Label(label)).to_vec();
            let mut expect: Vec<_> = c
                .graph
                .edges()
                .filter(|(_, e)| e.label.0 == label)
                .map(|(k, _)| k)
                .collect();
            actual.sort_by_key(|k| k.data().as_ffi());
            expect.sort_by_key(|k| k.data().as_ffi());
            if actual != expect {
                return Err("regulated label index".into());
            }
        }
        Ok(())
    }
    fn opportunities(
        &self,
        external: bool,
        diagnostic: bool,
    ) -> Result<(Vec<(i64, Vec<i64>)>, Vec<(i64, String)>), String> {
        let p = self.program()?;
        let mut admitted = Vec::new();
        let mut refused = Vec::new();
        for family in 0..5 {
            let reason = if !external && !diagnostic && p.mask & (1 << family) == 0 {
                Some("masked")
            } else if family == 0 && p.genes.len() == 8 {
                Some("gene_cap")
            } else if family == 0 && p.next_serial == 32 {
                Some("serial_cap")
            } else if family == 2 && p.genes.len() == 1 {
                Some("last_gene")
            } else {
                None
            };
            let targets: Vec<_> = match family {
                0 | 1 => p
                    .genes
                    .iter()
                    .filter(|(_, s)| (0..4).contains(s))
                    .map(|(id, _)| *id)
                    .collect(),
                2 => p.genes.iter().map(|(id, _)| *id).collect(),
                3 => vec![1 - p.enabled],
                _ => (0..32).collect(),
            };
            if let Some(r) = reason {
                refused.push((family, r.to_string()));
            } else if targets.is_empty() {
                refused.push((family, "no_executable_gene".into()));
            } else {
                admitted.push((family, targets));
            }
        }
        Ok((admitted, refused))
    }
    fn rule(
        &self,
        family: i64,
        target: i64,
        external: bool,
        diagnostic: bool,
    ) -> Result<CompiledRule, String> {
        let p = self.program()?;
        let with_gene = family < 3;
        let vars = if with_gene {
            vec![("w", WALLET), ("e", EDITOR), ("g", GENE)]
        } else {
            vec![("w", WALLET), ("e", EDITOR)]
        };
        let links = if with_gene {
            vec![("w", "e", OWN_EDITOR), ("w", "g", OWN_GENE)]
        } else {
            vec![("w", "e", OWN_EDITOR)]
        };
        let mut r = base_rule(&format!("regulated-{family}-{target}"), &vars, &links);
        guard(&mut r, "w", CAPITAL, PropertyOp::Gte, 2);
        guard(
            &mut r,
            "e",
            ENABLED,
            PropertyOp::Eq,
            if external || diagnostic { p.enabled } else { 1 },
        );
        guard(&mut r, "e", MASK, PropertyOp::Eq, p.mask);
        guard(&mut r, "e", EDIT_MODE, PropertyOp::Eq, 0);
        if with_gene {
            guard(&mut r, "g", SERIAL, PropertyOp::Eq, target);
        }
        if family < 2 {
            guard(&mut r, "g", STAGE, PropertyOp::Gte, 0);
            guard(&mut r, "g", STAGE, PropertyOp::Lt, 4);
        }
        match family {
            0 => {
                r.rhs.nodes.push("c".into());
                r.rhs.node_labels.insert("c".into(), Label(GENE));
                let index = r.rhs.edges.len();
                r.rhs.edges.push(vec!["w".into(), "c".into()]);
                r.rhs.edge_labels.insert(index, Label(OWN_GENE));
                effect(
                    &mut r,
                    "c",
                    STAGE,
                    EffectOpDecl::CopyFrom {
                        source_var: "g".into(),
                        source_key: STAGE,
                    },
                );
                effect(&mut r, "c", STAGE, EffectOpDecl::Jitter { magnitude: 1. });
                effect(
                    &mut r,
                    "c",
                    SERIAL,
                    EffectOpDecl::Set {
                        value: PropertyValue::Int(p.next_serial),
                    },
                );
            }
            1 => effect(&mut r, "g", STAGE, EffectOpDecl::Jitter { magnitude: 1. }),
            2 => {
                r.rhs.nodes.retain(|v| v != "g");
                r.rhs.edges.truncate(1);
                r.rhs.edge_labels.retain(|i, _| *i == 0);
            }
            3 => effect(
                &mut r,
                "e",
                ENABLED,
                EffectOpDecl::Set {
                    value: PropertyValue::Int(target),
                },
            ),
            4 => effect(
                &mut r,
                "e",
                MASK,
                EffectOpDecl::Set {
                    value: PropertyValue::Int(target),
                },
            ),
            _ => return Err("operator family".into()),
        }
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
    pub fn edit(
        &mut self,
        external: bool,
        probe: Option<(i64, i64)>,
        reverse: bool,
    ) -> Result<bool, String> {
        self.invoke(external, probe, reverse, false)
    }
    fn invoke(
        &mut self,
        external: bool,
        probe: Option<(i64, i64)>,
        reverse: bool,
        diagnostic: bool,
    ) -> Result<bool, String> {
        self.validate()?;
        if self.attempts == 32 {
            return Err("editor attempt cap".into());
        }
        let before = self.state()?;
        let (ops, restrictions) = self.opportunities(external, diagnostic)?;
        let reason = if !self.alive {
            Some("dead")
        } else if !diagnostic
            && ((external && self.core.arm != Arm::External)
                || (!external && matches!(self.core.arm, Arm::External | Arm::Frozen)))
        {
            Some("arm")
        } else if !external && !diagnostic && self.program()?.enabled == 0 {
            Some("disabled")
        } else if self.core.capital()? < 2 {
            Some("unfunded")
        } else if ops.is_empty() {
            Some("no_opportunities")
        } else if probe.is_some_and(|(f, t)| !ops.iter().any(|(k, ts)| *k == f && ts.contains(&t)))
        {
            Some("probe_unavailable")
        } else {
            None
        };
        if let Some(reason) = reason {
            self.attempts += 1;
            self.frames.push(json!({"kind":"edit_refusal","before":before,"after":self.state()?,"external":external,"probe":probe,"diagnostic":diagnostic,"reason":reason,"opportunities":ops,"restrictions":restrictions}));
            return Ok(false);
        }
        let mut all = Vec::new();
        let mut input = ops.clone();
        if reverse {
            input.reverse();
        }
        for (family, mut targets) in input {
            if reverse {
                targets.reverse();
            }
            for target in targets {
                let r = self.rule(family, target, external, diagnostic)?;
                let bs = find_bindings(&self.core.graph.snapshot(), &r, 0);
                if bs.len() != 1 {
                    return Err("regulated binding enumeration".into());
                }
                all.push((family, target, r, bs[0].clone()));
            }
        }
        if all.len() > 64 {
            return Err("regulated binding cap".into());
        }
        all.sort_by_key(|(f, t, _, _)| (*f, *t));
        let selected = if let Some(pair) = probe {
            pair
        } else {
            let mut rng = self.core.rng();
            let family_index = rng.gen_range(0..ops.len());
            let targets = &ops[family_index].1;
            let target_index = rng.gen_range(0..targets.len());
            self.core.word_pos = rng.get_word_pos();
            (ops[family_index].0, targets[target_index])
        };
        self.attempts += 1;
        self.frames.push(json!({"kind":"edit_selection","before":before,"after":self.state()?,"external":external,"probe":probe,"diagnostic":diagnostic,
            "opportunities":ops,"restrictions":restrictions,"selected":selected,"binding_count":all.len()}));
        let (_, _, r, b) = all
            .into_iter()
            .find(|(f, t, _, _)| (*f, *t) == selected)
            .unwrap();
        self.core.apply(&r,&b,"regulated_edit",json!({"family":selected.0,"target":selected.1,"external":external,"probe":probe.is_some(),"diagnostic_restore":diagnostic}))?;
        if selected.0 == 0 {
            self.core.next_serial += 1;
        }
        self.core.dissipated += 2;
        self.frames.push(json!({"kind":"native_edit","receipts":std::mem::take(&mut self.core.receipts),"after":self.state()?}));
        self.validate()?;
        Ok(true)
    }
    pub fn tick(&mut self, reverse: bool) -> Result<(), String> {
        self.validate()?;
        if self.core.tick == 16 || self.attempts == 32 {
            return Err("regulated tick/attempt cap".into());
        }
        let before = self.state()?;
        if self.alive {
            let due = 1 + self.program()?.genes.len() as i64;
            if !charge(&mut self.core, due)? {
                self.alive = false;
            }
            if self.alive && self.reservoir >= 16 && self.core.capital()? >= 1 {
                let before = self.core.state()?;
                self.reservoir -= 16;
                self.grants += 1;
                self.core.initial += 16;
                self.core.environment = 16;
                self.core.receipts.push(
                    json!({"kind":"shared_grant","before":before,"after":self.core.state()?}),
                );
                process(&mut self.core, reverse)?;
            } else {
                idle(&mut self.core)?;
            }
        } else {
            idle(&mut self.core)?;
        }
        self.frames.push(json!({"kind":"resource_tick","before":before,"receipts":std::mem::take(&mut self.core.receipts),"after":self.state()?}));
        self.edit(false, None, reverse)?;
        Ok(())
    }
    pub fn birth(&mut self) -> Result<Option<Self>, String> {
        self.validate()?;
        let before = self.state()?;
        let cost = 3 + self.program()?.genes.len() as i64;
        if !self.alive || self.core.capital()? < cost + 8 + 24 {
            self.frames.push(json!({"kind":"birth_refusal","before":before,"after":self.state()?,"reason":if self.alive {"unfunded"} else {"dead"}}));
            return Ok(None);
        }
        if self.core.transferred != 0 {
            return Err("single diagnostic birth cap".into());
        }
        let inherited = self.program()?;
        let transmitted = if self.core.arm == Arm::Nonheritable {
            self.baseline.clone()
        } else {
            inherited.clone()
        };
        let spec = json!({"stages":transmitted.genes.iter().map(|(_,s)|*s).collect::<Vec<_>>(),"enabled":transmitted.enabled,"mask":transmitted.mask,"arm":self.core.arm,"capital":8,"reservoir":0});
        let mut child = Self::from_case(&spec, self.core.seed.wrapping_add(1), 0)?;
        let ids: Vec<_> = child.core.genes()?.iter().map(|(_, id, _)| *id).collect();
        for (id, (serial, _)) in ids.iter().zip(&transmitted.genes) {
            child
                .core
                .graph
                .get_node_mut(*id)
                .unwrap()
                .set_property(SERIAL, PropertyValue::Int(*serial));
        }
        child.core.next_serial = transmitted.next_serial;
        child.core.id = self.core.id + 1;
        child.core.parent = Some(self.core.id);
        child.core.generation = self.core.generation + 1;
        child.core.tick = self.core.tick;
        child.core.graph.set_step(child.core.tick);
        let ids: Vec<_> = child.core.graph.nodes().map(|(id, _)| id).collect();
        for id in ids {
            child.core.graph.get_node_mut(id).unwrap().birth_step = child.core.tick;
        }
        child.baseline = self.baseline.clone();
        child.validate()?;
        let money = self.core.capital()?;
        set_capital(&mut self.core, money - cost - 8);
        self.core.dissipated += cost;
        self.core.transferred += 8;
        self.frames.push(json!({"kind":"birth","before":before,"after":self.state()?,"child":child.state()?,"copy_cost":cost,"inherited":inherited,"transmitted":transmitted}));
        self.validate()?;
        Ok(Some(child))
    }
    pub fn checkpoint(&self) -> Result<Value, String> {
        self.validate()?;
        Ok(json!({"state":self.state()?,"graph":self.core.graph}))
    }
    pub fn restore(v: Value) -> Result<Self, String> {
        if v.as_object().map(|o| o.len()) != Some(2) {
            return Err("regulated checkpoint fields".into());
        }
        let s = &v["state"];
        let t = &s["core"];
        let mut graph: HypergraphStore =
            serde_json::from_value(v["graph"].clone()).map_err(|e| e.to_string())?;
        let allocator =
            serde_json::from_value(t["allocator"].clone()).map_err(|e| e.to_string())?;
        graph
            .restore_allocator_order(&allocator)
            .map_err(str::to_string)?;
        let u = |k: &str| t[k].as_u64().ok_or(format!("regulated checkpoint {k}"));
        let i = |k: &str| t[k].as_i64().ok_or(format!("regulated checkpoint {k}"));
        let baseline: Program =
            serde_json::from_value(s["baseline"].clone()).map_err(|e| e.to_string())?;
        let core = Cell {
            graph,
            wallet: serde_json::from_value(t["wallet"].clone()).map_err(|e| e.to_string())?,
            editor: serde_json::from_value(t["editor"].clone()).map_err(|e| e.to_string())?,
            id: u("cell")?,
            parent: serde_json::from_value(t["parent"].clone()).map_err(|e| e.to_string())?,
            generation: u("generation")? as u32,
            arm: serde_json::from_value(t["arm"].clone()).map_err(|e| e.to_string())?,
            seed: u("seed")?,
            word_pos: t["word_pos"]
                .as_str()
                .ok_or("cursor")?
                .parse()
                .map_err(|_| "cursor parse")?,
            environment: i("environment")?,
            dissipated: i("dissipated")?,
            transferred: i("transferred")?,
            initial: i("initial")?,
            next_serial: i("next_serial")?,
            tick: u("tick")?,
            rewrites: u("rewrites")? as usize,
            baseline: Genome {
                stages: baseline.genes.iter().map(|(_, s)| *s).collect(),
                editor_mode: 0,
            },
            receipts: Vec::new(),
        };
        let p = Self {
            core,
            reservoir: s["reservoir"].as_i64().ok_or("bank")?,
            initial_reservoir: s["initial_reservoir"].as_i64().ok_or("initial bank")?,
            endowment: s["endowment"].as_i64().ok_or("endowment")?,
            grants: s["grants"].as_u64().ok_or("grants")?,
            alive: s["alive"].as_bool().ok_or("alive")?,
            attempts: s["attempts"].as_u64().ok_or("attempts")?,
            baseline,
            frames: Vec::new(),
        };
        p.validate()?;
        if p.state()? != *s {
            return Err("regulated graph/state projection".into());
        }
        Ok(p)
    }
}

pub fn fixture() -> Result<Value, String> {
    let start = native_work();
    let contract: Value = serde_json::from_str(include_str!("../../regulation_contract.json"))
        .map_err(|e| e.to_string())?;
    let seed = contract["development_seed"].as_u64().unwrap();
    let mut cases = Vec::new();
    for spec in contract["cases"].as_array().unwrap() {
        let mut c = RegulatedCell::from_case(spec, seed, 0)?;
        let initial = c.state()?;
        if spec["pre_edit"] == true {
            c.edit(false, None, false)?;
        }
        for _ in 0..spec["ticks"].as_u64().unwrap() {
            c.tick(false)?;
        }
        let child = if spec["birth"] == true {
            if let Some(mut child) = c.birth()? {
                let initial = child.state()?;
                child.edit(child.core.arm == Arm::External, None, false)?;
                Some(json!({"initial":initial,"frames":child.frames,"final":child.state()?}))
            } else {
                None
            }
        } else {
            None
        };
        cases.push(json!({"name":spec["name"],"initial":initial,"frames":c.frames,"final":c.state()?,"child":child}));
    }
    let spec =
        json!({"stages":[0],"enabled":1,"mask":31,"arm":"endogenous","capital":40,"reservoir":0});
    let mut disabled = RegulatedCell::from_case(&spec, seed, 0)?;
    let initial = disabled.state()?;
    disabled.edit(false, Some((3, 0)), false)?;
    let disabled_start = disabled.state()?;
    let mut restored = disabled.clone();
    restored.frames.clear();
    disabled.edit(false, Some((0, 0)), false)?;
    let mut sham = RegulatedCell::from_case(&spec, seed, 0)?;
    sham.edit(false, Some((4, 31)), false)?;
    sham.edit(false, Some((0, 0)), false)?;
    restored.invoke(false, Some((3, 1)), false, true)?;
    restored.edit(false, Some((0, 0)), false)?;
    let causal = json!({"initial":initial,"disabled":{"frames":disabled.frames,"final":disabled.state()?},"sham":{"frames":sham.frames,"final":sham.state()?},
        "restored":{"initial":disabled_start,"frames":restored.frames,"final":restored.state()?}});
    let spec = json!({"stages":[0,1,2],"enabled":1,"mask":20,"arm":"endogenous","capital":40,"reservoir":16});
    let mut a = RegulatedCell::from_case(&spec, seed, 0)?;
    let initial = a.state()?;
    // One normal resource-before-edit tick, then declared authored transitions.
    // Avoid an autonomous edit here: the setup is a native chemistry witness.
    let before = a.state()?;
    charge(&mut a.core, 4)?;
    let grant = a.core.state()?;
    a.reservoir -= 16;
    a.grants += 1;
    a.core.initial += 16;
    a.core.environment = 16;
    a.core
        .receipts
        .push(json!({"kind":"shared_grant","before":grant,"after":a.core.state()?}));
    process(&mut a.core, false)?;
    a.frames.push(json!({"kind":"resource_tick","before":before,"receipts":std::mem::take(&mut a.core.receipts),"after":a.state()?}));
    a.edit(false, Some((2, 1)), false)?;
    a.edit(false, Some((4, 17)), false)?;
    a.edit(false, Some((0, 0)), false)?;
    let setup = json!({"initial":initial,"frames":a.frames,"final":a.state()?});
    let checkpoint = a.checkpoint()?;
    let mut b = RegulatedCell::restore(checkpoint.clone())?;
    a.frames.clear();
    let child = a.birth()?.ok_or("hole inheritance funding")?;
    let child_b = b.birth()?.ok_or("restored hole inheritance funding")?;
    if child.state()? != child_b.state()? {
        return Err("regulated child continuation".into());
    }
    for _ in 0..2 {
        a.tick(false)?;
        b.tick(true)?;
    }
    if a.state()? != b.state()? || a.frames != b.frames {
        return Err("regulated continuation parity".into());
    }
    let continuation = json!({"setup":setup,"checkpoint":checkpoint,"original":a.frames,"restored":b.frames,"final":a.state()?,"restored_final":b.state()?,"child":child.state()?});
    let spec = &contract["cases"][4];
    let mut normal = RegulatedCell::from_case(spec, seed, 0)?;
    let mut reverse = normal.clone();
    let mut offset = RegulatedCell::from_case(spec, seed, 7)?;
    let normal_initial = normal.state()?;
    let offset_initial = offset.state()?;
    for _ in 0..2 {
        normal.tick(false)?;
        reverse.tick(true)?;
        offset.tick(true)?;
    }
    if normal.state()? != reverse.state()?
        || normal.frames != reverse.frames
        || normal.program()? != offset.program()?
        || normal.core.capital()? != offset.core.capital()?
        || normal.core.word_pos != offset.core.word_pos
    {
        return Err("regulated ordering/allocation parity".into());
    }
    Ok(
        json!({"profile":PROFILE,"contract":contract,"cases":cases,"causal":causal,"continuation":continuation,
        "order":{"initial":normal_initial,"normal":normal.frames,"reverse":reverse.frames,"final":normal.state()?},"offset":{"initial":offset_initial,"frames":offset.frames,"final":offset.state()?},
        "native_rewrites":native_work()-start,"engineering_fixture_pass":true,"readiness_gate_complete":false,"scientific_acceptance":false,"evolution_demonstrated":false,"campaign_launch_authorized":false}),
    )
}
