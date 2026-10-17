//! Prospective parent-funded reserve calibration; legacy physics remains fixed.
use super::{config::ENERGY, family::copy_cost, mutation::MUTATION_LAWS};
pub const FUNDING_PROFILE: &str = "evolving-minds-d0-funding-v1";
pub fn validate_endowment(endowment: u8) -> Result<(), String> {
    if [12, 24].contains(&endowment) {
        Ok(())
    } else {
        Err("funding fixed endowment denominator".into())
    }
}
/// Change only the identified native energy fields, never gene IDs or unrelated prices.
/// Also accepts a single immutable-template-derived settlement rule.
pub(crate) fn funded_text(text: &str, endowment: u8) -> Result<String, String> {
    validate_endowment(endowment)?;
    if endowment == 12 {
        return Ok(text.into());
    }
    let mut value: toml::Value = text.parse::<toml::Value>().map_err(|e| e.to_string())?;
    for rule in value["rules"].as_array_mut().ok_or("funding rules")? {
        let name = rule["name"].as_str().ok_or("funding rule name")?.to_owned();
        let (reserve, length) = if let Some(n) = name.strip_prefix("reserve_") {
            (true, n.parse::<usize>().map_err(|e| e.to_string())?)
        } else if let Some(n) = name.strip_prefix("transaction_copy_") {
            (false, n.parse::<usize>().map_err(|e| e.to_string())?)
        } else {
            continue;
        };
        if !(1..=8).contains(&length) {
            return Err("funding length".into());
        }
        let cost = copy_cost(length);
        let guard_var = if reserve { "r" } else { "escrow" };
        let old_guard = if reserve { 13.0 + cost } else { 12.0 + cost };
        let mut guards = 0;
        for g in rule["lhs"]["node_property_guards"]
            .as_array_mut()
            .ok_or("funding guards")?
        {
            if g["var"].as_str() == Some(guard_var)
                && g["property"].as_integer() == Some(ENERGY as i64)
            {
                if g["value"].as_float() != Some(old_guard)
                    || g["op"].as_str() != Some(if reserve { "gte" } else { "eq" })
                {
                    return Err("funding immutable guard".into());
                }
                g["value"] = toml::Value::Float(old_guard + 12.0);
                guards += 1;
            }
        }
        let mut effects = 0;
        for e in rule["rhs"]["property_effects"]
            .as_array_mut()
            .ok_or("funding effects")?
        {
            if e["key"].as_integer() != Some(ENERGY as i64) {
                continue;
            }
            let variable = e["var"].as_str().ok_or("funding effect var")?;
            let expected = if reserve && variable == "r" {
                Some(("subtract", 13.0 + cost))
            } else if reserve && variable == "escrow" {
                Some(("set", 12.0 + cost))
            } else if !reserve && variable == "child" {
                Some(("set", 12.0))
            } else {
                None
            };
            if let Some((op, old)) = expected {
                if e["op"].as_str() != Some(op) || e["value"].as_float() != Some(old) {
                    return Err("funding immutable effect".into());
                }
                e["value"] = toml::Value::Float(old + 12.0);
                effects += 1;
            }
        }
        if guards != 1 || effects != if reserve { 2 } else { 1 } {
            return Err("funding field coverage".into());
        }
    }
    toml::to_string(&value).map_err(|e| e.to_string())
}
pub fn funding_base_text(endowment: u8) -> Result<String, String> {
    funded_text(MUTATION_LAWS, endowment)
}
pub(crate) fn funding_law_text(
    schedule: &super::choice::ChoiceSchedule,
    endowment: u8,
) -> Result<String, String> {
    let old = super::mutation::mutation_law_text(schedule);
    Ok(format!(
        "{}{}",
        funding_base_text(endowment)?,
        &old[MUTATION_LAWS.len()..]
    ))
}
pub(crate) fn funding_descendant_text(endowment: u8) -> Result<String, String> {
    funded_text(&super::neighborhood::descendant_reserve_text()?, endowment)
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FundingRecipe {
    pub endowment: u8,
    pub recipe: super::neighborhood::NeighborhoodRecipe,
}
impl FundingRecipe {
    pub fn name(&self) -> String {
        format!("b{}_{}", self.endowment, self.recipe.name())
    }
}
pub fn funding_recipes() -> Vec<FundingRecipe> {
    [12, 24]
        .into_iter()
        .flat_map(|endowment| {
            super::neighborhood::neighborhood_recipes()
                .into_iter()
                .map(move |recipe| FundingRecipe { endowment, recipe })
        })
        .collect()
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FundingOutcome {
    pub endowment: u8,
    pub outcome: super::neighborhood::NeighborhoodOutcome,
}
