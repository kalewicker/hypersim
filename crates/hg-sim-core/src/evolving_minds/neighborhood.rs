//! Exhaustive paid first-offspring diagnostic. No selection or stochastic edits.
use super::{config::*, family::*, mutation::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const NEIGHBORHOOD_PROFILE: &str = "evolving-minds-d0-neighborhood-v1";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NeighborhoodRecipe {
    pub program: String,
    pub target_id: Option<u32>,
    pub new_source: Option<u8>,
}
impl NeighborhoodRecipe {
    pub fn edit(&self) -> Option<(u32, u8)> {
        self.target_id.zip(self.new_source)
    }
    pub fn name(&self) -> String {
        if let Some((id, source)) = self.edit() {
            format!("{}_g{}_s{}", self.program, id, source)
        } else {
            format!("{}_unchanged", self.program)
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if !["cue", "delayed_cue"].contains(&self.program.as_str())
            || self.target_id.is_some() != self.new_source.is_some()
        {
            return Err("neighborhood recipe".into());
        }
        let p = family_program(&self.program)?;
        if let Some((id, source)) = self.edit() {
            if source > 7
                || p.instructions
                    .iter()
                    .filter(|g| g.id == id && g.source != source)
                    .count()
                    != 1
            {
                return Err("neighborhood edit identity/slot".into());
            }
        }
        Ok(())
    }
}
pub fn neighborhood_recipes() -> Vec<NeighborhoodRecipe> {
    let mut all = vec![];
    for name in ["cue", "delayed_cue"] {
        all.push(NeighborhoodRecipe {
            program: name.into(),
            target_id: None,
            new_source: None,
        });
        let p = family_program(name).unwrap();
        for g in p.instructions {
            for source in 0..8 {
                if source != g.source {
                    all.push(NeighborhoodRecipe {
                        program: name.into(),
                        target_id: Some(g.id),
                        new_source: Some(source),
                    });
                }
            }
        }
    }
    all
}
pub(crate) fn descendant_reserve_text() -> Result<String, String> {
    let mut v: toml::Value = MUTATION_LAWS
        .parse::<toml::Value>()
        .map_err(|e| e.to_string())?;
    let rules = v["rules"].as_array_mut().ok_or("neighborhood law table")?;
    rules.retain(|r| {
        r["name"]
            .as_str()
            .is_some_and(|n| n.starts_with("reserve_"))
    });
    for r in rules {
        let guard: toml::Value = toml::from_str(&format!(
            "var=\"r\"\nproperty={}\nop=\"gte\"\nvalue=2",
            BIRTH_ID
        ))
        .map_err(|e| e.to_string())?;
        r["lhs"]["node_property_guards"]
            .as_array_mut()
            .ok_or("neighborhood reserve guards")?
            .push(guard);
    }
    toml::to_string(&v).map_err(|e| e.to_string())
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ChildLifetime {
    pub id: i64,
    pub born_tick: u64,
    pub death_tick: Option<u64>,
    pub death_cause: Option<String>,
    pub paid_construction_completed: bool,
    pub development_paid: u64,
    pub development_failures: u64,
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub harvested: f64,
    pub maintenance_paid: f64,
    pub last_age: i64,
    pub reserve_at_death: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NeighborhoodOutcome {
    pub recipe: NeighborhoodRecipe,
    pub syntax_valid: bool,
    pub material_reachable: bool,
    pub first_request_tick: Option<u64>,
    pub first_child: Option<ChildLifetime>,
    pub first_child_reproduced: bool,
    pub descendants: usize,
    pub live_inherited_at_64: usize,
    pub max_inherited_depth: usize,
    pub lineage_harvested: f64,
    pub sustained_turnover: bool,
    pub final_row: FamilyRow,
}
pub fn neighborhood_outcome(
    s: &FamilySimulation,
    recipe: NeighborhoodRecipe,
) -> Result<NeighborhoodOutcome, String> {
    let m = s
        .mutation
        .as_ref()
        .ok_or("neighborhood missing mutation state")?;
    let r = m
        .records
        .first()
        .ok_or("neighborhood no lawful founder request")?;
    let mut depth: BTreeMap<i64, usize> = BTreeMap::new();
    for (&id, b) in &s.lineage {
        depth.insert(id, b.parent.map_or(0, |p| depth[&p] + 1));
    }
    let first_child = r
        .child_id
        .and_then(|id| s.neighborhood_lifetimes.get(&id).cloned());
    let inherited: Vec<_> = s.lineage.values().filter(|b| b.id != 1).collect();
    let reproduced = r.child_id.is_some_and(|id| {
        m.records
            .iter()
            .any(|q| q.parent == id && q.child_id.is_some())
    });
    let live = inherited.iter().filter(|b| b.death_tick.is_none()).count();
    Ok(NeighborhoodOutcome {
        recipe,
        syntax_valid: r.syntax_valid,
        material_reachable: r.material_reachable,
        first_request_tick: Some(r.tick),
        first_child,
        first_child_reproduced: reproduced,
        descendants: inherited.len().saturating_sub(1),
        live_inherited_at_64: live,
        max_inherited_depth: depth.values().copied().max().unwrap_or(0),
        lineage_harvested: s.neighborhood_lifetimes.values().map(|c| c.harvested).sum(),
        sustained_turnover: reproduced && live > 0,
        final_row: s
            .rows
            .last()
            .ok_or("neighborhood missing final row")?
            .clone(),
    })
}
