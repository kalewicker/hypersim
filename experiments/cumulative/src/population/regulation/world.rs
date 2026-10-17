//! Integrated finite economy over regulated graph compartments. Development only.
use super::*;
pub mod journal;
use std::collections::BTreeMap;
pub const PROFILE: &str = "cumulative-regulated-world-lineage-readiness-v1";
#[derive(Clone)]
pub struct World {
    cells: BTreeMap<u64, (RegulatedCell, Option<usize>)>,
    seed: u64,
    arm: Arm,
    tick: u64,
    reservoir: i64,
    initial_reservoir: i64,
    sites: usize,
    parcels: usize,
    initial_energy: i64,
    births: Vec<Value>,
    pub rows: Vec<Value>,
}
impl World {
    pub fn from_case(spec: &Value, seed: u64, offset: usize) -> Result<Self, String> {
        let programs = spec["programs"].as_array().ok_or("programs")?;
        let sites = spec["sites"].as_u64().ok_or("sites")? as usize;
        let parcels = spec["parcels_per_tick"].as_u64().ok_or("parcels")? as usize;
        let reservoir = spec["reservoir"].as_i64().ok_or("reservoir")?;
        let capital = spec["capital"].as_i64().ok_or("capital")?;
        let arm = serde_json::from_value(spec["arm"].clone()).map_err(|e| e.to_string())?;
        if programs.is_empty()
            || programs.len() > sites
            || sites > 8
            || parcels > 8
            || !(0..=512).contains(&reservoir)
            || reservoir % 16 != 0
        {
            return Err("world input domain".into());
        }
        let mut cells = BTreeMap::new();
        for (id, p) in programs.iter().enumerate() {
            let s = json!({"stages":p["stages"],"enabled":p["enabled"],"mask":p["mask"],"capital":capital,"reservoir":0,"arm":arm});
            let mut c = RegulatedCell::from_case(&s, score(seed, 0, 3, id as u64), offset)?;
            c.core.id = id as u64;
            cells.insert(id as u64, (c, Some(id)));
        }
        let p = Self {
            cells,
            seed,
            arm,
            tick: 0,
            reservoir,
            initial_reservoir: reservoir,
            sites,
            parcels,
            initial_energy: reservoir + capital * programs.len() as i64,
            births: Vec::new(),
            rows: Vec::new(),
        };
        p.validate()?;
        Ok(p)
    }
    pub fn state(&self) -> Result<Value, String> {
        let cells = self
            .cells
            .iter()
            .map(|(&id, (c, site))| Ok(json!({"id":id,"site":site,"state":c.state()?})))
            .collect::<Result<Vec<Value>, String>>()?;
        Ok(
            json!({"profile":PROFILE,"seed":self.seed,"arm":self.arm,"tick":self.tick,"reservoir":self.reservoir,"initial_reservoir":self.initial_reservoir,"sites":self.sites,"parcels_per_tick":self.parcels,"initial_energy":self.initial_energy,"next_id":self.cells.len(),"births":self.births,"cells":cells}),
        )
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.tick > 16
            || self.cells.len() > 32
            || !(1..=8).contains(&self.sites)
            || self.parcels > 8
            || self.reservoir < 0
            || self.reservoir > self.initial_reservoir
            || self.reservoir % 16 != 0
        {
            return Err("world domain".into());
        }
        let mut energy = i128::from(self.reservoir);
        let mut grants = 0;
        let mut sites = BTreeSet::new();
        let mut founders = 0;
        for (index, (&id, (c, site))) in self.cells.iter().enumerate() {
            c.validate()?;
            if id != index as u64
                || c.core.id != id
                || c.core.arm != self.arm
                || c.core.seed != score(self.seed, 0, 3, id)
                || c.core.tick != self.tick
                || c.reservoir != 0
                || c.initial_reservoir != 16 * c.grants as i64
                || c.alive != site.is_some()
                || site.is_some_and(|s| s >= self.sites || !sites.insert(s))
            {
                return Err("world member identity/site/bank".into());
            }
            if let Some(parent) = c.core.parent {
                let p = &self.cells.get(&parent).ok_or("world parent")?.0;
                if parent >= id
                    || c.core.generation != p.core.generation + 1
                    || c.endowment != 8
                    || c.baseline != p.baseline
                {
                    return Err("world ancestry/baseline".into());
                }
            } else {
                if c.core.generation != 0 {
                    return Err("founder generation".into());
                }
                founders += c.endowment;
            }
            let children = self
                .cells
                .values()
                .filter(|(x, _)| x.core.parent == Some(id))
                .count();
            if c.core.transferred != 8 * children as i64 {
                return Err("world transfer ledger".into());
            }
            energy += i128::from(c.core.capital()?) + i128::from(c.core.dissipated);
            grants += c.grants;
        }
        if energy != i128::from(self.initial_energy)
            || self.initial_energy != self.initial_reservoir + founders
            || self.initial_reservoir - self.reservoir != 16 * grants as i64
            || self.births.len()
                != self
                    .cells
                    .values()
                    .filter(|(c, _)| c.core.parent.is_some())
                    .count()
        {
            return Err("shared world conservation".into());
        }
        for b in &self.births {
            let id = b["child"].as_u64().ok_or("birth child")?;
            let parent = b["parent"].as_u64().ok_or("birth parent")?;
            let c = &self.cells.get(&id).ok_or("birth child missing")?.0;
            if c.core.parent != Some(parent)
                || b["tick"].as_u64().ok_or("birth tick")? > self.tick
                || b["copy_cost"].as_i64()
                    != Some(
                        3 + serde_json::from_value::<Program>(b["inherited"].clone())
                            .map_err(|e| e.to_string())?
                            .genes
                            .len() as i64,
                    )
            {
                return Err("world birth log".into());
            }
        }
        Ok(())
    }
    fn ranking(&self, ids: &[u64], phase: u64, reverse: bool) -> Vec<(u64, u64)> {
        let mut ids = ids.to_vec();
        if reverse {
            ids.reverse();
        }
        let mut rows = ids
            .into_iter()
            .map(|id| (id, score(self.seed, self.tick, phase, id)))
            .collect::<Vec<_>>();
        rows.sort_by_key(|&(id, s)| (s, id));
        rows
    }
    pub fn step(&mut self, reverse: bool) -> Result<(), String> {
        self.advance(reverse, &[])
    }
    fn advance(&mut self, reverse: bool, probes: &[(i64, i64)]) -> Result<(), String> {
        self.validate()?;
        if self.tick == 16
            || self
                .cells
                .values()
                .any(|(c, _)| c.attempts + probes.len() as u64 + 1 > 32)
        {
            return Err("world tick/attempt cap".into());
        }
        let before = self.state()?;
        let ids = self.cells.keys().copied().collect::<Vec<_>>();
        let requestors = ids
            .iter()
            .copied()
            .filter(|id| {
                let c = &self.cells[id].0;
                c.alive && c.core.capital().unwrap() >= 2 + c.program().unwrap().genes.len() as i64
            })
            .collect::<Vec<_>>();
        let resources = self.ranking(&requestors, 0, reverse);
        let count = self
            .parcels
            .min((self.reservoir / 16) as usize)
            .min(resources.len());
        let winners = resources
            .iter()
            .take(count)
            .map(|r| r.0)
            .collect::<Vec<_>>();
        let mut operations = Vec::new();
        for &id in &ids {
            let (c, site) = self.cells.get_mut(&id).unwrap();
            let initial = c.state()?;
            c.frames.clear();
            if winners.contains(&id) {
                let before = c.state()?;
                self.reservoir -= 16;
                c.reservoir += 16;
                c.initial_reservoir += 16;
                c.frames.push(json!({"kind":"shared_allocation","before":before,"after":c.state()?,"amount":16}));
            }
            let before = c.state()?;
            if c.alive {
                let due = 1 + c.program()?.genes.len() as i64;
                if !charge(&mut c.core, due)? {
                    c.alive = false;
                    *site = None;
                }
                if c.alive && c.reservoir == 16 && c.core.capital()? >= 1 {
                    let before = c.core.state()?;
                    c.reservoir -= 16;
                    c.grants += 1;
                    c.core.initial += 16;
                    c.core.environment = 16;
                    c.core.receipts.push(
                        json!({"kind":"shared_grant","before":before,"after":c.core.state()?}),
                    );
                    process(&mut c.core, reverse)?;
                } else {
                    idle(&mut c.core)?;
                }
            } else {
                idle(&mut c.core)?;
            }
            c.frames.push(json!({"kind":"resource_tick","before":before,"after":c.state()?,"receipts":std::mem::take(&mut c.core.receipts)}));
            operations.push(json!({"phase":"resource","cell":id,"initial":initial,"frames":std::mem::take(&mut c.frames),"final":c.state()?}));
        }
        for &id in &ids {
            let c = &mut self.cells.get_mut(&id).unwrap().0;
            let initial = c.state()?;
            c.frames.clear();
            if id == 0 && !probes.is_empty() {
                for &p in probes {
                    c.edit(false, Some(p), reverse)?;
                }
            } else if self.tick % 3 == 0 {
                c.edit(false, None, reverse)?;
            }
            operations.push(json!({"phase":"edit","cell":id,"initial":initial,"frames":std::mem::take(&mut c.frames),"final":c.state()?}));
        }
        let parents = ids
            .iter()
            .copied()
            .filter(|id| self.cells[id].0.alive)
            .collect::<Vec<_>>();
        let births = self.ranking(&parents, 1, reverse);
        let mut refusals = Vec::new();
        for &(parent, _) in &births {
            let c = &self.cells[&parent].0;
            let cost = 3 + c.program()?.genes.len() as i64;
            let occupied = self
                .cells
                .values()
                .filter_map(|(_, s)| *s)
                .collect::<BTreeSet<_>>();
            let site = (0..self.sites).find(|s| !occupied.contains(s));
            let reason = if c.core.capital()? < cost + 8 + 24 {
                Some("unfunded")
            } else if site.is_none() {
                Some("no_site")
            } else if self.cells.len() == 32 {
                Some("cell_cap")
            } else {
                None
            };
            if let Some(reason) = reason {
                refusals.push(json!({"cell":parent,"reason":reason}));
                continue;
            }
            let id = self.cells.len() as u64;
            let inherited = c.program()?;
            let transmitted = if self.arm == Arm::Nonheritable {
                c.baseline.clone()
            } else {
                inherited.clone()
            };
            let spec = json!({"stages":transmitted.genes.iter().map(|g|g.1).collect::<Vec<_>>(),"enabled":transmitted.enabled,"mask":transmitted.mask,"capital":8,"reservoir":0,"arm":self.arm});
            let mut child = RegulatedCell::from_case(&spec, score(self.seed, 0, 3, id), 0)?;
            let gids = child.core.genes()?.iter().map(|g| g.1).collect::<Vec<_>>();
            for (gid, (serial, _)) in gids.into_iter().zip(&transmitted.genes) {
                child
                    .core
                    .graph
                    .get_node_mut(gid)
                    .unwrap()
                    .set_property(SERIAL, PropertyValue::Int(*serial));
            }
            child.core.next_serial = transmitted.next_serial;
            child.baseline = c.baseline.clone();
            child.core.id = id;
            child.core.parent = Some(parent);
            child.core.generation = c.core.generation + 1;
            child.core.tick = self.tick + 1;
            child.core.graph.set_step(self.tick + 1);
            let handles = child
                .core
                .graph
                .nodes()
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            for h in handles {
                child.core.graph.get_node_mut(h).unwrap().birth_step = self.tick + 1;
            }
            child.validate()?;
            let c = &mut self.cells.get_mut(&parent).unwrap().0;
            let initial = c.state()?;
            let capital = c.core.capital()?;
            set_capital(&mut c.core, capital - cost - 8);
            c.core.dissipated += cost;
            c.core.transferred += 8;
            let record = json!({"parent":parent,"child":id,"tick":self.tick+1,"site":site.unwrap(),"copy_cost":cost,"inherited":inherited,"transmitted":transmitted});
            self.births.push(record.clone());
            let child_initial = child.state()?;
            operations.push(json!({"phase":"birth","cell":parent,"initial":initial,"frames":[{"kind":"shared_birth","before":initial,"after":c.state()?,"child":child_initial,"birth":record}],"final":c.state()?}));
            if self.arm == Arm::External {
                child.edit(true, None, reverse)?;
            }
            operations.push(json!({"phase":"newborn","cell":id,"initial":child_initial,"frames":std::mem::take(&mut child.frames),"final":child.state()?}));
            self.cells.insert(id, (child, site));
        }
        self.tick += 1;
        self.validate()?;
        self.rows.push(json!({"before":before,"after":self.state()?,"resource_ranking":resources,"resource_winners":winners,"birth_ranking":births,"birth_refusals":refusals,"operations":operations}));
        Ok(())
    }
    pub fn checkpoint(&self) -> Result<Value, String> {
        self.validate()?;
        Ok(
            json!({"state":self.state()?,"cells":self.cells.iter().map(|(&id,(c,site))|Ok(json!({"id":id,"site":site,"checkpoint":c.checkpoint()?}))).collect::<Result<Vec<Value>,String>>()?}),
        )
    }
    pub fn restore(cp: Value) -> Result<Self, String> {
        if cp.as_object().map(|m| m.len()) != Some(2) {
            return Err("world checkpoint fields".into());
        }
        let s = &cp["state"];
        let mut cells = BTreeMap::new();
        for e in cp["cells"].as_array().ok_or("world checkpoint cells")? {
            let id = e["id"].as_u64().ok_or("id")?;
            let site = serde_json::from_value(e["site"].clone()).map_err(|e| e.to_string())?;
            if cells
                .insert(id, (RegulatedCell::restore(e["checkpoint"].clone())?, site))
                .is_some()
            {
                return Err("duplicate world cell".into());
            }
        }
        let u = |k: &str| s[k].as_u64().ok_or(format!("world {k}"));
        let i = |k: &str| s[k].as_i64().ok_or(format!("world {k}"));
        let p = Self {
            cells,
            seed: u("seed")?,
            arm: serde_json::from_value(s["arm"].clone()).map_err(|e| e.to_string())?,
            tick: u("tick")?,
            reservoir: i("reservoir")?,
            initial_reservoir: i("initial_reservoir")?,
            sites: u("sites")? as usize,
            parcels: u("parcels_per_tick")? as usize,
            initial_energy: i("initial_energy")?,
            births: serde_json::from_value(s["births"].clone()).map_err(|e| e.to_string())?,
            rows: Vec::new(),
        };
        p.validate()?;
        if p.state()? != *s {
            return Err("world checkpoint projection".into());
        }
        Ok(p)
    }
}
pub fn fixture() -> Result<Value, String> {
    let start = native_work();
    let contract: Value = serde_json::from_str(include_str!("../../../integration_contract.json"))
        .map_err(|e| e.to_string())?;
    let seed = contract["development_seed"].as_u64().unwrap();
    let mut cases = Vec::new();
    for spec in contract["cases"].as_array().unwrap() {
        let mut p = World::from_case(spec, seed, 0)?;
        let initial = p.state()?;
        for tick in 0..spec["ticks"].as_u64().unwrap() {
            let probes = if spec.get("probe_schedule").is_some() {
                serde_json::from_value(spec["probe_schedule"][tick as usize].clone())
                    .map_err(|e| e.to_string())?
            } else {
                Vec::new()
            };
            p.advance(false, &probes)?;
        }
        cases.push(json!({"name":spec["name"],"initial":initial,"rows":p.rows,"final":p.state()?}));
    }
    let spec = contract["cases"].as_array().unwrap().last().unwrap();
    let mut a = World::from_case(spec, seed, 0)?;
    let mut b = a.clone();
    let mut offset = World::from_case(spec, seed, 7)?;
    let initial = a.state()?;
    let offset_initial = offset.state()?;
    for tick in 0..2 {
        let probes =
            serde_json::from_value::<Vec<(i64, i64)>>(spec["probe_schedule"][tick].clone())
                .unwrap();
        a.advance(false, &probes)?;
        b.advance(true, &probes)?;
        offset.advance(true, &probes)?;
    }
    if a.state()? != b.state()? || a.rows != b.rows {
        return Err("world candidate order".into());
    }
    for (&id, (c, site)) in &a.cells {
        let (d, s) = &offset.cells[&id];
        if c.program()? != d.program()?
            || c.core.capital()? != d.core.capital()?
            || c.core.word_pos != d.core.word_pos
            || site != s
        {
            return Err("world allocation projection".into());
        }
    }
    let order = json!({"initial":initial,"original":a.rows,"reverse":b.rows,"final":a.state()?,"offset_initial":offset_initial,"offset":offset.rows,"offset_final":offset.state()?});
    let checkpoint = a.checkpoint()?;
    let mut restored = World::restore(checkpoint.clone())?;
    a.rows.clear();
    for _ in 0..2 {
        a.step(false)?;
        restored.step(true)?;
    }
    if a.state()? != restored.state()? || a.rows != restored.rows {
        return Err("world continuation".into());
    }
    let work = native_work() - start;
    if work > 6000 {
        return Err("world fixture work cap".into());
    }
    Ok(
        json!({"profile":PROFILE,"contract":contract,"cases":cases,"order":order,"continuation":{"checkpoint":checkpoint,"original":a.rows,"restored":restored.rows,"final":a.state()?},"native_rewrites":work,"readiness_gate_complete":false,"scientific_acceptance":false,"evolution_demonstrated":false,"campaign_launch_authorized":false}),
    )
}
