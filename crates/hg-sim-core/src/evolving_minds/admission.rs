//! Supplied ecological arbitration; native rules alone change physical material.
use super::{
    choice::{mix64, ChoiceSchedule},
    config::*,
    d0::*,
    family::*,
    mutation::*,
    runtime::{float, int},
};
use hg_core::HypergraphStore;
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const ADMISSION_PROFILE: &str = "evolving-minds-d0-admission-v1";
pub const ADMISSION_LAWS: &str = include_str!("../../../../rules/evolving_minds/admission_v1.toml");
pub const ADMISSION_STEPS: u64 = 46;
pub const LOCAL_ID: u32 = 242018;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionRecipe {
    pub case: String,
    pub rotation: u8,
    pub reversed: bool,
}
impl AdmissionRecipe {
    pub fn name(&self) -> String {
        format!(
            "{}_r{}_{}",
            self.case,
            self.rotation,
            if self.reversed { "reverse" } else { "forward" }
        )
    }
    pub fn validate(&self) -> Result<(), String> {
        if !["faithful_disjoint", "mixed_competing", "mixed_settlement"]
            .contains(&self.case.as_str())
            || ![0, 3].contains(&self.rotation)
        {
            Err("admission fixed recipe".into())
        } else {
            Ok(())
        }
    }
    pub fn programs(&self) -> Result<[D0Program; 2], String> {
        Ok([
            family_program("cue")?,
            family_program(if self.case == "mixed_competing" {
                "delayed_cue"
            } else {
                "cue"
            })?,
        ])
    }
    pub fn homes(&self) -> [i64; 2] {
        [
            self.rotation as i64,
            (self.rotation as i64 + if self.case == "mixed_competing" { 2 } else { 4 }) % 8,
        ]
    }
    pub fn logical(&self, physical: i64) -> i64 {
        (physical - self.rotation as i64 + 8) % 8
    }
    pub fn kernel(&self, founder: i64) -> MutationKernel {
        if self.case == "mixed_settlement" {
            if founder == 1 {
                MutationKernel::Invalid
            } else {
                MutationKernel::SourceKernel
            }
        } else {
            MutationKernel::Faithful
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct AdmissionOffer {
    pub parent: i64,
    pub home: i64,
    pub target: i64,
    pub length: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionRound {
    pub tick: u64,
    pub stream_seed: u64,
    pub offers: Vec<AdmissionOffer>,
    pub raw_draws: Vec<u64>,
    pub permutation: Vec<usize>,
    pub selected: Vec<usize>,
}
/// Unbiased bounded draw; outcome-independent rejection is only modulo-bias removal.
pub fn lottery(
    seed: u64,
    tick: u64,
    mut offers: Vec<AdmissionOffer>,
) -> Result<AdmissionRound, String> {
    offers.sort();
    if offers.windows(2).any(|x| x[0] == x[1])
        || offers.iter().any(|o| {
            o.parent <= 0
                || !(0..8).contains(&o.home)
                || !(0..8).contains(&o.target)
                || o.home == o.target
                || !(1..=8).contains(&o.length)
        })
    {
        return Err("admission offer schema/duplicate".into());
    }
    let stream_seed = mix64(seed ^ 0x41444d495353494f ^ tick);
    let mut state = stream_seed;
    let mut raw_draws = vec![];
    let mut permutation: Vec<_> = (0..offers.len()).collect();
    for i in (1..offers.len()).rev() {
        let n = (i + 1) as u64;
        let threshold = n.wrapping_neg() % n;
        let x = loop {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^= z >> 31;
            raw_draws.push(z);
            if z >= threshold {
                break z;
            }
        };
        permutation.swap(i, (x % n) as usize)
    }
    let mut parents = BTreeSet::new();
    let mut sites = BTreeSet::new();
    let mut selected = vec![];
    for &i in &permutation {
        let o = &offers[i];
        if parents.contains(&o.parent) || sites.contains(&o.home) || sites.contains(&o.target) {
            continue;
        }
        parents.insert(o.parent);
        sites.extend([o.home, o.target]);
        selected.push(i)
    }
    Ok(AdmissionRound {
        tick,
        stream_seed,
        offers,
        raw_draws,
        permutation,
        selected,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdmissionState {
    pub recipe: AdmissionRecipe,
    pub rounds: Vec<AdmissionRound>,
    pub records: Vec<MutationRecord>,
}
pub fn admission_law_text(s: &ChoiceSchedule) -> String {
    let x = family_law_text(s);
    let start = x.find("\n[[rules]]\nname = \"world_").unwrap();
    format!("{ADMISSION_LAWS}{}", &x[start..])
}
fn specialized(
    mut law: toml::Value,
    name: String,
    parent: i64,
    target: Option<i64>,
    request: Option<i64>,
) -> Result<CompiledRule, String> {
    law["name"] = name.into();
    let guards = law["lhs"]["node_property_guards"]
        .as_array_mut()
        .ok_or("admission guards")?;
    for (var, key, value) in [
        ("r", BIRTH_ID, Some(parent)),
        ("target", 241009, target),
        ("escrow", REQUEST, request),
    ] {
        if let Some(v) = value {
            guards.push(
                toml::Value::try_from(
                    serde_json::json!({"var":var,"property":key,"op":"eq","value":v}),
                )
                .map_err(|e| e.to_string())?,
            )
        }
    }
    let text = toml::to_string(
        &toml::Value::try_from(BTreeMap::from([("rules", vec![law])]))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(
        compile_rules(&parse_rules_toml(&text).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
            .remove(0),
    )
}
impl AdmissionState {
    pub fn new(recipe: AdmissionRecipe) -> Self {
        Self {
            recipe,
            rounds: vec![],
            records: vec![],
        }
    }
    pub fn reserve_rules(
        &mut self,
        store: &HypergraphStore,
        lineage: &BTreeMap<i64, FamilyBirth>,
        seed: u64,
        tick: u64,
    ) -> Result<Vec<CompiledRule>, String> {
        self.reserve_rules_funded(store, lineage, seed, tick, 12)
    }
    pub(crate) fn reserve_rules_funded(
        &mut self,
        store: &HypergraphStore,
        lineage: &BTreeMap<i64, FamilyBirth>,
        seed: u64,
        tick: u64,
        endowment: u8,
    ) -> Result<Vec<CompiledRule>, String> {
        let sites: BTreeMap<_, _> = store
            .nodes()
            .filter(|(_, n)| n.label.0 == SITE)
            .map(|(h, n)| Ok((int(n, 241009)?, (h, n))))
            .collect::<Result<_, String>>()?;
        let mut offers = vec![];
        for b in lineage.values().filter(|b| b.death_tick.is_none()) {
            let r = store
                .nodes()
                .find(|(_, n)| n.label.0 == ROOT && int(n, BIRTH_ID).ok() == Some(b.id))
                .ok_or("admission root")?
                .1;
            let length = b.program.instructions.len();
            if int(r, STATUS)? != 2
                || int(r, AGE)? < 2
                || float(r, ENERGY)? < 1.0 + endowment as f64 + copy_cost(length)
            {
                continue;
            }
            for target in [(b.site + 7) % 8, (b.site + 1) % 8] {
                if sites[&target].1.get_property(OCCUPIED)
                    == Some(hg_core::PropertyValue::Bool(false))
                {
                    offers.push(AdmissionOffer {
                        parent: b.id,
                        home: self.recipe.logical(b.site),
                        target: self.recipe.logical(target),
                        length,
                    })
                }
            }
        }
        let round = lottery(seed, tick, offers)?;
        if round.selected.is_empty() {
            self.rounds.push(round);
            return Ok(vec![]);
        }
        let base: toml::Value = super::funding::funded_text(ADMISSION_LAWS, endowment)?
            .parse::<toml::Value>()
            .map_err(|e| e.to_string())?;
        let mut rules = vec![];
        for &index in &round.selected {
            let o = &round.offers[index];
            let target = (o.target + self.recipe.rotation as i64) % 8;
            let law = base["rules"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["name"].as_str() == Some(&format!("reserve_{}", o.length)))
                .unwrap()
                .clone();
            rules.push(specialized(
                law,
                format!("reserve_selected_{}_{}", o.parent, target),
                o.parent,
                Some(target),
                None,
            )?)
        }
        self.rounds.push(round);
        Ok(rules)
    }
    pub fn admit(
        &mut self,
        store: &HypergraphStore,
        parent: i64,
        p: D0Program,
        seed: u64,
        tick: u64,
        lineage: &BTreeMap<i64, FamilyBirth>,
    ) -> Result<(), String> {
        let mut founder = parent;
        while let Some(a) = lineage
            .get(&founder)
            .ok_or("admission founder ancestry")?
            .parent
        {
            founder = a;
        }
        self.admit_origin(store, parent, p, seed, tick, founder)
    }
    pub(crate) fn admit_origin(
        &mut self,
        store: &HypergraphStore,
        parent: i64,
        p: D0Program,
        seed: u64,
        tick: u64,
        founder: i64,
    ) -> Result<(), String> {
        self.admit_origin_kernel(
            store,
            parent,
            p,
            seed,
            tick,
            founder,
            self.recipe.kernel(founder),
        )
    }
    pub(crate) fn admit_origin_kernel(
        &mut self,
        store: &HypergraphStore,
        parent: i64,
        p: D0Program,
        seed: u64,
        tick: u64,
        founder: i64,
        kernel: MutationKernel,
    ) -> Result<(), String> {
        if ![1, 2].contains(&founder) {
            return Err("admission founder identity".into());
        }
        let (handle, e) = store
            .nodes()
            .find(|(_, n)| {
                n.label.0 == ESCROW
                    && int(n, OWNER).ok() == Some(parent)
                    && int(n, TX_STATUS).ok() == Some(0)
            })
            .ok_or("admission escrow")?;
        let id = int(e, REQUEST)?;
        if self.records.iter().any(|r| r.request_id == id) {
            return Err("admission reused request".into());
        }
        let site = store
            .edges()
            .find(|(_, e)| e.label.0 == PLACE && e.nodes.contains(&handle))
            .and_then(|(_, e)| {
                e.nodes
                    .iter()
                    .find_map(|h| store.get_node(*h).filter(|n| n.label.0 == SITE))
            })
            .ok_or("admission target")?;
        let stream_seed = mix64(seed ^ 0x4d55544154494f4e ^ id as u64);
        let mut sampler = MutationState::new(kernel, stream_seed);
        let (q, target, old, new, raw) = sampler.proposal(&p)?;
        let error = q.validate().err();
        self.records.push(MutationRecord {
            request_id: id,
            tick,
            parent,
            site: int(site, 241009)?,
            operator: kernel.name().into(),
            version: 1,
            stream: format!("splitmix64-request-local-v1:{stream_seed}"),
            draw_start: 0,
            draw_end: sampler.draw_index,
            raw_draws: raw,
            target_id: target,
            old_source: old,
            new_source: new,
            parent_bytes: raw_program_bytes(&p),
            proposed_bytes: raw_program_bytes(&q),
            parent_program: p,
            material_reachable: error.is_none() && shape(&q, q.instructions.len()).is_ok(),
            syntax_valid: error.is_none(),
            invalid_reason: error,
            proposed_program: q,
            request_charge: 1.0,
            copying_charge: 0.0,
            refunded: 0.0,
            child_id: None,
            settled: false,
            settlement_rule: String::new(),
        });
        self.records.sort_by_key(|r| r.request_id);
        Ok(())
    }
    pub fn settlement_rules(&mut self) -> Result<Vec<CompiledRule>, String> {
        self.settlement_rules_funded(12)
    }
    pub(crate) fn settlement_rules_funded(
        &mut self,
        endowment: u8,
    ) -> Result<Vec<CompiledRule>, String> {
        if self.records.iter().all(|r| r.settled) {
            return Ok(vec![]);
        }
        let base: toml::Value = super::funding::funded_text(ADMISSION_LAWS, endowment)?
            .parse::<toml::Value>()
            .map_err(|e| e.to_string())?;
        let mut result = vec![];
        for r in self.records.iter_mut().filter(|r| !r.settled) {
            let name = if r.syntax_valid {
                format!("transaction_copy_{}", r.proposed_program.instructions.len())
            } else {
                "transaction_reject".into()
            };
            let mut law = base["rules"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["name"].as_str() == Some(&name))
                .unwrap()
                .clone();
            if r.syntax_valid {
                if let Some(id) = r.target_id {
                    let pos = r
                        .proposed_program
                        .instructions
                        .iter()
                        .position(|g| g.id == id)
                        .ok_or("admission gene ID")?;
                    let effect = law["rhs"]["property_effects"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|f| {
                            f["var"].as_str() == Some(&format!("cg{pos}"))
                                && f["key"].as_integer() == Some(SRC as i64)
                        })
                        .ok_or("admission copy effect")?;
                    let t = effect.as_table_mut().unwrap();
                    t.remove("source_var");
                    t.remove("source_key");
                    t.insert("op".into(), "set".into());
                    t.insert("value".into(), (r.new_source.unwrap() as i64).into());
                }
            }
            let rule = specialized(
                law,
                format!("transaction_selected_{}", r.request_id),
                r.parent,
                Some(r.site),
                Some(r.request_id),
            )?;
            r.settlement_rule = rule.name.clone();
            result.push(rule)
        }
        Ok(result)
    }
    pub fn settle(&mut self, request: i64, child: Option<i64>) -> Result<(), String> {
        self.settle_funded(request, child, 12)
    }
    pub(crate) fn settle_funded(
        &mut self,
        request: i64,
        child: Option<i64>,
        endowment: u8,
    ) -> Result<(), String> {
        let r = self
            .records
            .iter_mut()
            .find(|r| r.request_id == request)
            .ok_or("admission unknown settlement")?;
        if r.settled || r.syntax_valid != child.is_some() || child.is_some_and(|c| c != request) {
            return Err("admission settlement outcome".into());
        }
        r.child_id = child;
        r.settled = true;
        if child.is_some() {
            r.copying_charge = copy_cost(r.proposed_program.instructions.len())
        } else {
            r.refunded = endowment as f64 + copy_cost(r.proposed_program.instructions.len())
        }
        Ok(())
    }
    pub fn validate(&self, store: &HypergraphStore) -> Result<(), String> {
        self.validate_funded(store, 12)
    }
    pub(crate) fn validate_funded(
        &self,
        store: &HypergraphStore,
        endowment: u8,
    ) -> Result<(), String> {
        let mut seen = BTreeSet::new();
        let mut escrows = BTreeSet::new();
        for r in &self.records {
            if !seen.insert(r.request_id)
                || r.request_id != 3 + self.recipe.logical(r.site) + 8 * r.tick as i64
            {
                return Err("admission record identity".into());
            }
        }
        for (h, e) in store.nodes().filter(|(_, n)| n.label.0 == ESCROW) {
            let id = int(e, REQUEST)?;
            if !escrows.insert(id) {
                return Err("admission duplicate escrow".into());
            }
            let r = self
                .records
                .iter()
                .find(|r| r.request_id == id)
                .ok_or("admission unrecorded escrow")?;
            if e.energy != 0.0
                || e.properties.len() != 5
                || int(e, OWNER)? != r.parent
                || int(e, GENOME_LENGTH)? != r.proposed_program.instructions.len() as i64
                || float(e, ENERGY)?
                    != if r.settled {
                        0.0
                    } else {
                        endowment as f64 + copy_cost(r.proposed_program.instructions.len())
                    }
                || int(e, TX_STATUS)?
                    != if !r.settled {
                        0
                    } else if r.child_id.is_some() {
                        1
                    } else {
                        2
                    }
            {
                return Err("admission escrow material".into());
            }
            let mut links = vec![];
            for (_, edge) in store.edges().filter(|(_, e)| e.nodes.contains(&h)) {
                if edge.nodes.len() != 2 {
                    return Err("admission escrow arity".into());
                }
                let other = *edge.nodes.iter().find(|x| **x != h).unwrap();
                let n = store.get_node(other).ok_or("admission dangling")?;
                if !match edge.label.0 {
                    FUNDS => n.label.0 == ROOT && int(n, BIRTH_ID)? == r.parent,
                    PLACE => n.label.0 == SITE && int(n, 241009)? == r.site,
                    _ => false,
                } {
                    return Err("admission foreign escrow".into());
                }
                links.push(edge.label.0)
            }
            links.sort();
            if links != [FUNDS, PLACE] {
                return Err("admission escrow roles".into());
            }
        }
        if self
            .records
            .iter()
            .any(|r| !r.settled && !escrows.contains(&r.request_id))
        {
            return Err("admission lost pending escrow".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hg_engine::Engine;
    use hg_rules::{MatchBudgetPolicy, StepMode};
    #[test]
    fn native_mixed_length_parallel_reservation_and_copy() {
        // Controlled native fixture: develop both founders before opening offers.
        // This is separate from the fixed ecology and makes no viability claim.
        let sim = FamilySimulation::new_admission(AdmissionRecipe {
            case: "mixed_competing".into(),
            rotation: 0,
            reversed: false,
        })
        .unwrap();
        let mut store = sim.store().clone();
        let rules = compile_rules(&parse_rules_toml(ADMISSION_LAWS).unwrap()).unwrap();
        let mut engine = Engine::new(vec![], StepMode::MaxParallel, 17, 1_000_000);
        engine.set_match_budget_policy(1_000_000, 100_000, MatchBudgetPolicy::Pooled);
        for prefix in [
            "development_",
            "development_",
            "development_",
            "development_",
            "development_",
            "maintenance_",
            "maintenance_",
            "identity_",
        ] {
            engine.replace_static_rules(
                rules
                    .iter()
                    .filter(|r| r.name.starts_with(prefix))
                    .cloned()
                    .collect(),
            );
            let stats = engine.step(&mut store);
            assert_eq!(stats.property_effects_skipped, 0);
        }
        let mut state = AdmissionState::new(sim.admission.as_ref().unwrap().recipe.clone());
        let initial: f64 = store
            .nodes()
            .filter(|(_, n)| [ROOT, SITE].contains(&n.label.0))
            .map(|(_, n)| float(n, ENERGY).unwrap())
            .sum();
        let reserve = state.reserve_rules(&store, &sim.lineage, 17, 1).unwrap();
        assert_eq!(reserve.len(), 2);
        engine.replace_static_rules(reserve);
        assert_eq!(engine.step(&mut store).accepted_rewrites, 2);
        for parent in [1, 2] {
            state
                .admit(
                    &store,
                    parent,
                    sim.lineage[&parent].program.clone(),
                    17,
                    1,
                    &sim.lineage,
                )
                .unwrap();
        }
        assert_eq!(
            state
                .records
                .iter()
                .map(|r| r.proposed_program.instructions.len())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([3, 5])
        );
        state.validate(&store).unwrap();
        engine.replace_static_rules(state.settlement_rules().unwrap());
        assert_eq!(engine.step(&mut store).accepted_rewrites, 2);
        for i in 0..state.records.len() {
            let id = state.records[i].request_id;
            state.settle(id, Some(id)).unwrap();
        }
        state.validate(&store).unwrap();
        engine.replace_static_rules(
            rules
                .iter()
                .filter(|r| r.name == "escrow_dispose")
                .cloned()
                .collect(),
        );
        assert_eq!(engine.step(&mut store).accepted_rewrites, 2);
        state.validate(&store).unwrap();
        assert!(!store.nodes().any(|(_, n)| n.label.0 == ESCROW));
        assert_eq!(store.nodes().filter(|(_, n)| n.label.0 == ROOT).count(), 4);
        let final_energy: f64 = store
            .nodes()
            .filter(|(_, n)| [ROOT, SITE].contains(&n.label.0))
            .map(|(_, n)| float(n, ENERGY).unwrap())
            .sum();
        assert_eq!(initial - final_energy, 2.0 + copy_cost(3) + copy_cost(5));
        for (_, n) in store
            .nodes()
            .filter(|(_, n)| n.label.0 == ROOT && int(n, BIRTH_ID).unwrap() > 2)
        {
            assert_eq!(int(n, AGE).unwrap(), 0);
            assert_eq!(int(n, CURSOR).unwrap(), 0);
            assert_eq!(int(n, STATUS).unwrap(), 0);
            assert_eq!(float(n, ENERGY).unwrap(), 12.0);
        }
    }
}
