//! Shared finite economy over local graph compartments. Engineering only.
//! Uses the original native graph instruction semantics without diagnostic birth resources.
use super::*;
pub mod regulation;
use std::collections::BTreeMap;

pub const PROFILE: &str = "cumulative-shared-population-readiness-v1";
const GOLDEN: u64 = 0x9e3779b97f4a7c15;

pub fn score(seed: u64, tick: u64, phase: u64, id: u64) -> u64 {
    let mut z = seed
        ^ tick.wrapping_mul(GOLDEN)
        ^ phase.wrapping_mul(0xd1b54a32d192ed03)
        ^ id.wrapping_mul(0x94d049bb133111eb);
    z = z.wrapping_add(GOLDEN);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[derive(Clone)]
struct Member {
    cell: Cell,
    site: Option<usize>,
    endowment: i64,
    grants: u64,
}

#[derive(Clone)]
pub struct Population {
    cells: BTreeMap<u64, Member>,
    arm: Arm,
    seed: u64,
    tick: u64,
    reservoir: i64,
    initial_reservoir: i64,
    parcels_per_tick: usize,
    sites: usize,
    next_id: u64,
    initial_energy: i64,
    births: Vec<Value>,
    pub rows: Vec<Value>,
}

fn set_capital(c: &mut Cell, value: i64) {
    c.graph
        .get_node_mut(c.wallet)
        .unwrap()
        .set_property(CAPITAL, PropertyValue::Int(value));
}
fn idle(c: &mut Cell) -> Result<(), String> {
    let before = c.state()?;
    c.tick += 1;
    c.graph.set_step(c.tick);
    c.receipts
        .push(json!({"kind":"idle_boundary","before":before,"after":c.state()?}));
    Ok(())
}
fn charge(c: &mut Cell, amount: i64) -> Result<bool, String> {
    let before = c.state()?;
    let capital = c.capital()?;
    if capital < amount {
        set_capital(c, 0);
        c.dissipated += capital;
        c.receipts.push(json!({"kind":"death","amount":capital,"storage_due":amount,"before":before,"after":c.state()?}));
        return Ok(false);
    }
    set_capital(c, capital - amount);
    c.dissipated += amount;
    c.receipts
        .push(json!({"kind":"storage","amount":amount,"before":before,"after":c.state()?}));
    Ok(true)
}
fn process(c: &mut Cell, reverse: bool) -> Result<(), String> {
    // The only escrow comes from an already debited shared reservoir.
    if c.environment != 16 {
        return Err("parcel escrow".into());
    }
    let before = c.state()?;
    c.environment = 0;
    let p = node(&mut c.graph, 0, &[]);
    edge(&mut c.graph, OWN_PARCEL, c.wallet, p);
    c.receipts
        .push(json!({"kind":"supply","before":before,"after":c.state()?}));
    for _ in 0..4 {
        let stage = c.graph.get_node(p).unwrap().label.0 as i64;
        let mut rules: Vec<_> = c
            .genes()?
            .into_iter()
            .filter(|(_, _, s)| (0..4).contains(s))
            .map(|(serial, _, s)| Cell::metabolic_rule(s, serial).map(|r| (serial, r)))
            .collect::<Result<_, _>>()?;
        if reverse {
            rules.reverse();
        }
        let mut candidates = Vec::new();
        for (serial, r) in rules {
            let bindings = find_bindings(&c.graph.snapshot(), &r, 0);
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
        let opportunities: Vec<_> = candidates.iter().map(|(s, _, _)| *s).collect();
        let (serial, r, b) = candidates.remove(0);
        c.apply(
            &r,
            &b,
            "metabolic",
            json!({"serial":serial,"stage":stage,"opportunities":opportunities}),
        )?;
        c.dissipated += 1;
    }
    let stage = c.graph.get_node(p).unwrap().label.0;
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
    let bindings = find_bindings(&c.graph.snapshot(), &r, 0);
    if bindings.len() != 1 {
        return Err("settlement binding".into());
    }
    c.apply(
        &r,
        &bindings[0],
        "settle",
        json!({"stage":stage,"payout":payout}),
    )?;
    c.dissipated += 16 - payout;
    c.tick += 1;
    c.graph.set_step(c.tick);
    c.receipts
        .push(json!({"kind":"boundary","after":c.state()?}));
    Ok(())
}
fn edit(c: &mut Cell, external: bool) -> Result<(), String> {
    if c.genes()?.len() >= MAX_GENES {
        return Err("editing gene bound".into());
    }
    let r = c.editor_rule()?;
    let mut bs = find_bindings(&c.graph.snapshot(), &r, 0);
    if bs.len() > MAX_GENES || bs.is_empty() {
        return Err("funded native editor unavailable".into());
    }
    bs.sort_by_key(|b| int(&c.graph, b.bound_nodes[2], SERIAL).unwrap());
    let opportunities: Vec<_> = bs
        .iter()
        .map(|b| int(&c.graph, b.bound_nodes[2], SERIAL).unwrap())
        .collect();
    let before = c.state()?;
    let mut rng = c.rng();
    let i = rng.gen_range(0..bs.len());
    c.word_pos = rng.get_word_pos();
    c.receipts.push(
        json!({"kind":"selection","before":before,"after":c.state()?,"opportunities":opportunities,
        "selected_serial":opportunities[i],"external":external}),
    );
    c.apply(
        &r,
        &bs[i],
        if external {
            "external_edit"
        } else {
            "endogenous_edit"
        },
        json!({"probe":false,"selected_serial":opportunities[i],"opportunities":opportunities}),
    )?;
    c.next_serial += 1;
    c.dissipated += 2;
    c.receipts
        .push(json!({"kind":"edit_boundary","after":c.state()?}));
    Ok(())
}

impl Population {
    pub fn from_case(case: &Value, seed: u64, offset: usize) -> Result<Self, String> {
        let arm: Arm = serde_json::from_value(case["arm"].clone()).map_err(|e| e.to_string())?;
        let genomes: Vec<Vec<i64>> =
            serde_json::from_value(case["genomes"].clone()).map_err(|e| e.to_string())?;
        let capital = case["capital"].as_i64().ok_or("capital")?;
        let reservoir = case["reservoir"].as_i64().ok_or("reservoir")?;
        let sites = case["sites"].as_u64().ok_or("sites")? as usize;
        let parcels_per_tick = case["parcels_per_tick"].as_u64().ok_or("parcels")? as usize;
        if genomes.is_empty()
            || genomes.len() > sites
            || sites > 8
            || capital < 0
            || capital > 64
            || reservoir < 0
            || reservoir > 512
            || reservoir % 16 != 0
            || parcels_per_tick > 8
        {
            return Err("population input domain".into());
        }
        let mut cells = BTreeMap::new();
        for (id, stages) in genomes.iter().enumerate() {
            let mut c = Cell::with_offset(stages, arm, score(seed, 0, 3, id as u64), offset)?;
            c.id = id as u64;
            c.environment = 0;
            c.initial = capital;
            set_capital(&mut c, capital);
            cells.insert(
                id as u64,
                Member {
                    cell: c,
                    site: Some(id),
                    endowment: capital,
                    grants: 0,
                },
            );
        }
        let s = Self {
            cells,
            arm,
            seed,
            tick: 0,
            reservoir,
            initial_reservoir: reservoir,
            parcels_per_tick,
            sites,
            next_id: genomes.len() as u64,
            initial_energy: reservoir + capital * genomes.len() as i64,
            births: Vec::new(),
            rows: Vec::new(),
        };
        s.validate()?;
        Ok(s)
    }
    pub fn state(&self) -> Result<Value, String> {
        let cells: Vec<_> = self
            .cells
            .values()
            .map(|m| {
                m.cell.state().map(
                    |s| json!({"state":s,"site":m.site,"endowment":m.endowment,"grants":m.grants}),
                )
            })
            .collect::<Result<_, _>>()?;
        Ok(
            json!({"profile":PROFILE,"arm":self.arm,"seed":self.seed,"tick":self.tick,"reservoir":self.reservoir,
            "initial_reservoir":self.initial_reservoir,"parcels_per_tick":self.parcels_per_tick,"sites":self.sites,
            "next_id":self.next_id,"initial_energy":self.initial_energy,"births":self.births,"cells":cells}),
        )
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.tick > 16
            || self.sites == 0
            || self.sites > 8
            || self.cells.len() > 32
            || self.next_id != self.cells.len() as u64
            || self.reservoir < 0
            || self.reservoir > self.initial_reservoir
            || self.reservoir % 16 != 0
            || self.parcels_per_tick > 8
        {
            return Err("population clock/cap/reservoir".into());
        }
        let mut occupied = BTreeSet::new();
        let mut energy = i128::from(self.reservoir);
        let mut grants = 0u64;
        for (&id, m) in &self.cells {
            let c = &m.cell;
            let genes = c.genes()?;
            if id != c.id
                || c.arm != self.arm
                || c.seed != score(self.seed, 0, 3, id)
                || c.next_serial != genes.len() as i64
                || c.graph.node_count() != genes.len() + 2
                || c.graph.edge_count() != genes.len() + 1
                || genes.is_empty()
                || genes.len() > MAX_GENES
                || genes
                    .iter()
                    .enumerate()
                    .any(|(i, (s, _, stage))| *s != i as i64 || !(-1..=4).contains(stage))
                || c.environment != 0
                || c.graph.step() != c.tick
                || c.tick > self.tick
                || c.word_pos >= (1u128 << 68)
                || c.rewrites > 6000
                || m.grants > 16
                || m.endowment < 0
                || m.endowment > 64
            {
                return Err("population cell domain".into());
            }
            if c.graph.get_node(c.wallet).map(|d| d.label.0) != Some(WALLET)
                || c.graph.get_node(c.editor).map(|d| d.label.0) != Some(EDITOR)
                || int(&c.graph, c.editor, EDIT_MODE)? != 0
                || c.baseline.editor_mode != 0
                || c.baseline.stages.is_empty()
                || c.baseline.stages.len() > MAX_GENES
                || c.baseline.stages.iter().any(|s| !(-1..=4).contains(s))
            {
                return Err("population editor/baseline".into());
            }
            let mut owned = BTreeSet::new();
            for (_, e) in c.graph.edges() {
                if e.nodes.len() != 2
                    || e.nodes[0] != c.wallet
                    || !owned.insert(e.nodes[1].data().as_ffi())
                {
                    return Err("population ownership".into());
                }
                let lab = c
                    .graph
                    .get_node(e.nodes[1])
                    .ok_or("missing endpoint")?
                    .label
                    .0;
                if !matches!((e.label.0, lab), (OWN_GENE, GENE) | (OWN_EDITOR, EDITOR)) {
                    return Err("ownership role".into());
                }
            }
            if owned.len() != genes.len() + 1 {
                return Err("unowned genome".into());
            }
            for (n, d) in c.graph.nodes() {
                let expected = if n == c.wallet {
                    vec![CAPITAL]
                } else if n == c.editor {
                    vec![EDIT_MODE]
                } else {
                    vec![STAGE, SERIAL]
                };
                let mut keys: Vec<_> = d.properties.iter().map(|(k, _)| *k).collect();
                keys.sort();
                if keys != expected || d.birth_step > c.tick || d.energy != 0. {
                    return Err("node schema".into());
                }
                let mut incident = c.graph.edges_incident_to(n).to_vec();
                let mut expected: Vec<_> = c
                    .graph
                    .edges()
                    .filter(|(_, e)| e.nodes.contains(&n))
                    .map(|(id, _)| id)
                    .collect();
                incident.sort_by_key(|id| id.data().as_ffi());
                expected.sort_by_key(|id| id.data().as_ffi());
                if incident != expected {
                    return Err("incidence index".into());
                }
            }
            for label in [OWN_GENE, OWN_EDITOR, OWN_PARCEL] {
                let mut indexed = c.graph.edges_with_label(Label(label)).to_vec();
                let mut expected: Vec<_> = c
                    .graph
                    .edges()
                    .filter(|(_, e)| e.label.0 == label)
                    .map(|(id, _)| id)
                    .collect();
                indexed.sort_by_key(|id| id.data().as_ffi());
                expected.sort_by_key(|id| id.data().as_ffi());
                if indexed != expected {
                    return Err("label index".into());
                }
            }
            let capital = c.capital()?;
            if capital < 0
                || c.dissipated < 0
                || c.transferred < 0
                || c.initial != m.endowment + 16 * m.grants as i64
                || i128::from(capital) + i128::from(c.dissipated) + i128::from(c.transferred)
                    != i128::from(c.initial)
            {
                return Err("cell energy".into());
            }
            if let Some(site) = m.site {
                if site >= self.sites || !occupied.insert(site) || c.tick != self.tick {
                    return Err("site occupancy/live clock".into());
                }
            } else if capital != 0 {
                return Err("dead-cell capital".into());
            }
            if let Some(parent) = c.parent {
                let p = self.cells.get(&parent).ok_or("parent missing")?;
                if parent >= id || c.generation != p.cell.generation + 1 || m.endowment != 8 {
                    return Err("ancestry".into());
                }
            } else if c.generation != 0 {
                return Err("founder generation".into());
            }
            let children = self
                .cells
                .values()
                .filter(|x| x.cell.parent == Some(id))
                .count();
            if c.transferred != 8 * children as i64 {
                return Err("copy transfer ledger".into());
            }
            energy += i128::from(capital) + i128::from(c.dissipated);
            grants += m.grants;
        }
        if energy != i128::from(self.initial_energy)
            || self.initial_energy
                != self.initial_reservoir
                    + self
                        .cells
                        .values()
                        .filter(|m| m.cell.parent.is_none())
                        .map(|m| m.endowment)
                        .sum::<i64>()
            || self.initial_reservoir - self.reservoir != 16 * grants as i64
            || self.births.len()
                != self
                    .cells
                    .values()
                    .filter(|m| m.cell.parent.is_some())
                    .count()
        {
            return Err("world conservation/genealogy".into());
        }
        let mut seen = BTreeSet::new();
        for b in &self.births {
            let id = b["child"].as_u64().ok_or("birth child")?;
            let m = self.cells.get(&id).ok_or("birth member")?;
            let inherited: Genome =
                serde_json::from_value(b["inherited"].clone()).map_err(|e| e.to_string())?;
            let transmitted: Genome =
                serde_json::from_value(b["transmitted"].clone()).map_err(|e| e.to_string())?;
            if !seen.insert(id)
                || b["parent"].as_u64() != m.cell.parent
                || b["generation"].as_u64() != Some(u64::from(m.cell.generation))
                || b["tick"].as_u64() != m.cell.graph.get_node(m.cell.wallet).map(|n| n.birth_step)
                || b["site"].as_u64().is_none_or(|s| s >= self.sites as u64)
                || b["copy_cost"].as_i64() != Some(3 + inherited.stages.len() as i64)
                || inherited.editor_mode != 0
                || inherited.stages.is_empty()
                || inherited.stages.len() > MAX_GENES
                || inherited.stages.iter().any(|s| !(-1..=4).contains(s))
                || transmitted
                    != if self.arm == Arm::Nonheritable {
                        m.cell.baseline.clone()
                    } else {
                        inherited
                    }
            {
                return Err("genealogy birth record".into());
            }
        }
        Ok(())
    }
    fn ranked(&self, ids: &[u64], phase: u64, reverse: bool) -> Vec<(u64, u64)> {
        let mut input = ids.to_vec();
        if reverse {
            input.reverse();
        }
        let mut v: Vec<_> = input
            .into_iter()
            .map(|id| (id, score(self.seed, self.tick, phase, id)))
            .collect();
        v.sort_by_key(|&(id, s)| (s, id));
        v
    }
    pub fn step(&mut self, reverse: bool) -> Result<(), String> {
        self.validate()?;
        if self.tick >= 16 {
            return Err("world tick bound".into());
        }
        let before = self.state()?;
        let ids: Vec<_> = self
            .cells
            .iter()
            .filter(|(_, m)| m.site.is_some())
            .map(|(&id, _)| id)
            .collect();
        for &id in &ids {
            let m = self.cells.get_mut(&id).unwrap();
            let due = 1 + m.cell.genome()?.stages.len() as i64;
            if !charge(&mut m.cell, due)? {
                m.site = None;
            }
        }
        let requestors: Vec<_> = ids
            .iter()
            .copied()
            .filter(|id| {
                self.cells[id].site.is_some() && self.cells[id].cell.capital().unwrap() >= 1
            })
            .collect();
        let ranking = self.ranked(&requestors, 0, reverse);
        let count = self
            .parcels_per_tick
            .min((self.reservoir / 16) as usize)
            .min(ranking.len());
        let winners: Vec<_> = ranking.iter().take(count).map(|(id, _)| *id).collect();
        for &id in &winners {
            let m = self.cells.get_mut(&id).unwrap();
            let c = &mut m.cell;
            let before = c.state()?;
            self.reservoir -= 16;
            m.grants += 1;
            c.initial += 16;
            c.environment += 16;
            c.receipts
                .push(json!({"kind":"shared_grant","before":before,"after":c.state()?}));
            process(c, reverse)?;
        }
        for &id in &ids {
            let c = &mut self.cells.get_mut(&id).unwrap().cell;
            if c.tick == self.tick {
                idle(c)?;
            }
        }
        let mut edit_refusals = Vec::new();
        if self.tick % 3 == 0 && matches!(self.arm, Arm::Endogenous | Arm::Nonheritable) {
            for &id in &ids {
                let m = self.cells.get_mut(&id).unwrap();
                if m.site.is_none() {
                    continue;
                }
                if m.cell.capital()? < 2 {
                    edit_refusals.push(json!({"cell":id,"reason":"unfunded"}));
                    continue;
                }
                edit(&mut m.cell, false)?;
            }
        }
        let parents: Vec<_> = ids
            .iter()
            .copied()
            .filter(|id| self.cells[id].site.is_some())
            .collect();
        let birth_ranking = self.ranked(&parents, 1, reverse);
        let mut refusals = Vec::new();
        for &(parent, _) in &birth_ranking {
            let c = &self.cells[&parent].cell;
            let cost = 3 + c.genome()?.stages.len() as i64;
            if c.capital()? < cost + 8 + 24 {
                refusals.push(json!({"cell":parent,"reason":"unfunded"}));
                continue;
            }
            let occupied: BTreeSet<_> = self.cells.values().filter_map(|m| m.site).collect();
            let Some(site) = (0..self.sites).find(|s| !occupied.contains(s)) else {
                refusals.push(json!({"cell":parent,"reason":"no_site"}));
                continue;
            };
            if self.cells.len() >= 32 {
                return Err("cell archive backstop".into());
            }
            let id = self.next_id;
            let c = &self.cells[&parent].cell;
            let inherited = c.genome()?;
            let genome = if self.arm == Arm::Nonheritable {
                c.baseline.clone()
            } else {
                inherited.clone()
            };
            let mut child = Cell::new(&genome.stages, self.arm, score(self.seed, 0, 3, id))?;
            child.id = id;
            child.parent = Some(parent);
            child.generation = c.generation + 1;
            child.environment = 0;
            child.initial = 8;
            set_capital(&mut child, 8);
            child.baseline = c.baseline.clone();
            child.tick = self.tick + 1;
            child.graph.set_step(child.tick);
            for (n, _) in child
                .graph
                .nodes()
                .map(|(n, d)| (n, d.clone()))
                .collect::<Vec<_>>()
            {
                child.graph.get_node_mut(n).unwrap().birth_step = child.tick;
            }
            let c = &mut self.cells.get_mut(&parent).unwrap().cell;
            let pre = c.state()?;
            let money = c.capital()?;
            set_capital(c, money - cost - 8);
            c.dissipated += cost;
            c.transferred += 8;
            let birth = json!({"parent":parent,"child":id,"generation":child.generation,"tick":self.tick+1,
                "site":site,"copy_cost":cost,"inherited":inherited,"transmitted":genome});
            self.births.push(birth.clone());
            c.receipts.push(json!({"kind":"shared_birth","before":pre,"after":c.state()?,"child":child.state()?,"birth":birth}));
            if self.arm == Arm::External {
                edit(&mut child, true)?;
            }
            self.cells.insert(
                id,
                Member {
                    cell: child,
                    site: Some(site),
                    endowment: 8,
                    grants: 0,
                },
            );
            self.next_id += 1;
        }
        self.tick += 1;
        self.validate()?;
        let mut operations = Vec::new();
        for (&id, m) in &mut self.cells {
            if !m.cell.receipts.is_empty() {
                operations.push(json!({"cell":id,"receipts":std::mem::take(&mut m.cell.receipts)}));
            }
        }
        self.rows.push(
            json!({"before":before,"resource_ranking":ranking,"resource_winners":winners,
            "birth_ranking":birth_ranking,"birth_refusals":refusals,"edit_refusals":edit_refusals,
            "operations":operations,"after":self.state()?}),
        );
        Ok(())
    }
    pub fn checkpoint(&self) -> Result<Value, String> {
        self.validate()?;
        let cells: Vec<_> = self
            .cells
            .values()
            .map(|m| {
                json!({"metadata":{"site":m.site,"endowment":m.endowment,"grants":m.grants},
            "state":m.cell.state().unwrap(),"graph":m.cell.graph,"baseline":m.cell.baseline})
            })
            .collect();
        Ok(json!({"state":self.state()?,"cells":cells}))
    }
    pub fn restore(v: Value) -> Result<Self, String> {
        if v.as_object().map(|o| o.len()) != Some(2)
            || v.get("state").is_none()
            || v.get("cells").is_none()
        {
            return Err("checkpoint fields".into());
        }
        let s = &v["state"];
        if s["profile"] != PROFILE {
            return Err("population checkpoint profile".into());
        }
        let mut cells = BTreeMap::new();
        for entry in v["cells"].as_array().ok_or("checkpoint cells")? {
            if entry.as_object().map(|o| o.len()) != Some(4)
                || entry["metadata"].as_object().map(|o| o.len()) != Some(3)
            {
                return Err("member checkpoint fields".into());
            }
            let t = &entry["state"];
            let mut graph: HypergraphStore =
                serde_json::from_value(entry["graph"].clone()).map_err(|e| e.to_string())?;
            let allocator =
                serde_json::from_value(t["allocator"].clone()).map_err(|e| e.to_string())?;
            graph
                .restore_allocator_order(&allocator)
                .map_err(str::to_string)?;
            let u = |k: &str| t[k].as_u64().ok_or(format!("checkpoint {k}"));
            let i = |k: &str| t[k].as_i64().ok_or(format!("checkpoint {k}"));
            let c = Cell {
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
                baseline: serde_json::from_value(entry["baseline"].clone())
                    .map_err(|e| e.to_string())?,
                receipts: Vec::new(),
            };
            if c.state()? != *t {
                return Err("checkpoint graph/state disagreement".into());
            }
            let meta = &entry["metadata"];
            let m = Member {
                cell: c,
                site: serde_json::from_value(meta["site"].clone()).map_err(|e| e.to_string())?,
                endowment: meta["endowment"].as_i64().ok_or("endowment")?,
                grants: meta["grants"].as_u64().ok_or("grants")?,
            };
            if cells.insert(m.cell.id, m).is_some() {
                return Err("duplicate member".into());
            }
        }
        let u = |k: &str| s[k].as_u64().ok_or(format!("world {k}"));
        let i = |k: &str| s[k].as_i64().ok_or(format!("world {k}"));
        let p = Self {
            cells,
            arm: serde_json::from_value(s["arm"].clone()).map_err(|e| e.to_string())?,
            seed: u("seed")?,
            tick: u("tick")?,
            reservoir: i("reservoir")?,
            initial_reservoir: i("initial_reservoir")?,
            parcels_per_tick: u("parcels_per_tick")? as usize,
            sites: u("sites")? as usize,
            next_id: u("next_id")?,
            initial_energy: i("initial_energy")?,
            births: serde_json::from_value(s["births"].clone()).map_err(|e| e.to_string())?,
            rows: Vec::new(),
        };
        p.validate()?;
        if p.state() != Ok(s.clone()) {
            return Err("population checkpoint mismatch".into());
        }
        Ok(p)
    }
}

pub fn fixture() -> Result<Value, String> {
    let start = native_work();
    let contract: Value = serde_json::from_str(include_str!("../population_contract.json"))
        .map_err(|e| e.to_string())?;
    let seed = contract["development_seed"].as_u64().unwrap();
    let mut cases = Vec::new();
    for case in contract["cases"].as_array().unwrap() {
        let mut p = Population::from_case(case, seed, 0)?;
        let initial = p.state()?;
        for _ in 0..case["ticks"].as_u64().unwrap() {
            p.step(false)?;
        }
        cases.push(json!({"name":case["name"],"initial":initial,"rows":p.rows,"final":p.state()?}));
    }
    let case = &contract["cases"][2];
    let mut a = Population::from_case(case, seed, 0)?;
    let mut reverse = a.clone();
    let mut offset = Population::from_case(case, seed, 7)?;
    for _ in 0..4 {
        a.step(false)?;
        reverse.step(true)?;
        offset.step(true)?;
    }
    if a.state()? != reverse.state()? || a.rows != reverse.rows {
        return Err("population order parity".into());
    }
    for (id, m) in &a.cells {
        let n = &offset.cells[id];
        if m.cell.capital()? != n.cell.capital()?
            || m.cell.genome()? != n.cell.genome()?
            || m.cell.word_pos != n.cell.word_pos
            || m.site != n.site
        {
            return Err("population allocation parity".into());
        }
    }
    let order = json!({"original":a.rows,"reverse":reverse.rows,"offset":offset.rows});
    let checkpoint = a.checkpoint()?;
    let mut b = Population::restore(checkpoint.clone())?;
    a.rows.clear();
    for _ in 0..3 {
        a.step(false)?;
        b.step(true)?;
    }
    if a.state()? != b.state()? || a.rows != b.rows {
        return Err("population continuation parity".into());
    }
    let work = native_work() - start;
    if work > 6000 {
        return Err("fixture native work bound".into());
    }
    Ok(
        json!({"profile":PROFILE,"contract":contract,"cases":cases,"order":order,
        "continuation":{"checkpoint":checkpoint,"original":a.rows,"restored":b.rows},"native_rewrites":work,
        "engineering_fixture_pass":true,"readiness_gate_complete":false,"scientific_acceptance":false,
        "evolution_demonstrated":false,"campaign_launch_authorized":false}),
    )
}
