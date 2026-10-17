//! Experimental one-shot sampler; material/energy changes execute native rules.
use super::{choice::ChoiceSchedule, config::*, d0::*, family::*, runtime::int};
use hg_core::{HypergraphStore, PropertyValue};
use serde::{Deserialize, Serialize};
pub const MUTATION_PROFILE: &str = "evolving-minds-d0-mutation-v1";
pub const MUTATION_LAWS: &str = include_str!("../../../../rules/evolving_minds/mutation_v1.toml");
pub const MUTATION_STEPS: u64 = 45;
pub const ESCROW: u32 = 0xE1020000;
pub const REQUEST: u32 = 242014;
pub const TX_STATUS: u32 = 242015;
pub const FUNDS: u32 = ESCROW + 100;
pub const PLACE: u32 = ESCROW + 101;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MutationKernel {
    Faithful,
    SourceKernel,
    Coin,
    Invalid,
    MissingPort,
    GrammarSourceKernel,
}
impl MutationKernel {
    pub fn name(self) -> &'static str {
        match self {
            Self::Faithful => "faithful",
            Self::SourceKernel => "source",
            Self::Coin => "coin",
            Self::Invalid => "invalid",
            Self::MissingPort => "missing_port",
            Self::GrammarSourceKernel => "grammar_source",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MutationRecord {
    pub request_id: i64,
    pub tick: u64,
    pub parent: i64,
    pub site: i64,
    pub operator: String,
    pub version: u32,
    pub stream: String,
    pub draw_start: u64,
    pub draw_end: u64,
    pub raw_draws: Vec<u64>,
    pub target_id: Option<u32>,
    pub old_source: Option<u8>,
    pub new_source: Option<u8>,
    pub parent_program: D0Program,
    pub proposed_program: D0Program,
    pub parent_bytes: Vec<u8>,
    pub proposed_bytes: Vec<u8>,
    pub syntax_valid: bool,
    pub invalid_reason: Option<String>,
    pub material_reachable: bool,
    pub request_charge: f64,
    pub copying_charge: f64,
    pub refunded: f64,
    pub child_id: Option<i64>,
    pub settled: bool,
    pub settlement_rule: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MutationState {
    pub kernel: MutationKernel,
    pub seed: u64,
    pub allocation_reversed: bool,
    pub rng_state: u64,
    pub draw_index: u64,
    pub records: Vec<MutationRecord>,
    pub pending: Option<i64>,
}
pub fn raw_program_bytes(p: &D0Program) -> Vec<u8> {
    let mut b = b"HG-D0-1".to_vec();
    b.push(p.instructions.len() as u8);
    for g in &p.instructions {
        b.extend(g.id.to_le_bytes());
        b.extend([
            match g.op {
                D0Op::State => 0,
                D0Op::Actuator => 1,
                D0Op::Wire => 2,
                D0Op::Halt => 3,
            },
            g.slot,
            g.source,
            g.initial as u8,
        ]);
    }
    b
}
pub fn mutation_law_text(s: &ChoiceSchedule) -> String {
    let text = family_law_text(s);
    let start = text.find("\n[[rules]]\nname = \"world_").unwrap();
    format!("{MUTATION_LAWS}{}", &text[start..])
}
impl MutationState {
    pub fn new(kernel: MutationKernel, seed: u64) -> Self {
        Self {
            kernel,
            seed,
            allocation_reversed: false,
            rng_state: seed ^ 0xD0B17A5E,
            draw_index: 0,
            records: vec![],
            pending: None,
        }
    }
    fn draw(&mut self, raw: &mut Vec<u64>) -> u64 {
        self.rng_state = self.rng_state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.rng_state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        self.draw_index += 1;
        raw.push(z);
        z
    }
    fn bounded(&mut self, n: u64, raw: &mut Vec<u64>) -> u64 {
        let threshold = n.wrapping_neg() % n;
        loop {
            let x = self.draw(raw);
            if x >= threshold {
                return x % n;
            }
        }
    }
    /// A pure semantic edit: no graph handles or outcome-dependent retry.
    pub fn proposal(
        &mut self,
        p: &D0Program,
    ) -> Result<(D0Program, Option<u32>, Option<u8>, Option<u8>, Vec<u64>), String> {
        p.validate()?;
        let mut q = p.clone();
        let mut raw = vec![];
        let mut edit = None;
        match self.kernel {
            MutationKernel::Faithful => {}
            MutationKernel::GrammarSourceKernel => {
                if self.bounded(4, &mut raw) != 0 {
                    let choices = super::grammar_mutation::grammar_source_candidates(p)?;
                    if !choices.is_empty() {
                        let (id, source, _) =
                            &choices[self.bounded(choices.len() as u64, &mut raw) as usize];
                        edit = Some((*id, *source));
                    }
                }
            }
            MutationKernel::SourceKernel => {
                if self.bounded(4, &mut raw) != 0 {
                    let mut ids: Vec<_> = p.instructions.iter().map(|g| g.id).collect();
                    ids.sort();
                    let id = ids[self.bounded(ids.len() as u64, &mut raw) as usize];
                    let g = p.instructions.iter().find(|g| g.id == id).unwrap();
                    let source = (g.source + 1 + self.bounded(7, &mut raw) as u8) % 8;
                    edit = Some((id, source));
                }
            }
            MutationKernel::Coin | MutationKernel::MissingPort | MutationKernel::Invalid => {
                let op = if self.kernel == MutationKernel::Invalid {
                    D0Op::Halt
                } else {
                    D0Op::Wire
                };
                let id = p
                    .instructions
                    .iter()
                    .filter(|g| g.op == op)
                    .map(|g| g.id)
                    .min()
                    .ok_or("mutation control target missing")?;
                edit = Some((
                    id,
                    if self.kernel == MutationKernel::MissingPort {
                        7
                    } else {
                        1
                    },
                ));
            }
        }
        if let Some((id, source)) = edit {
            let gs: Vec<_> = q.instructions.iter_mut().filter(|g| g.id == id).collect();
            if gs.len() != 1 {
                return Err("mutation semantic target ambiguity".into());
            }
            let g = gs.into_iter().next().unwrap();
            let old = g.source;
            g.source = source;
            Ok((q, Some(id), Some(old), Some(source), raw))
        } else {
            Ok((q, None, None, None, raw))
        }
    }
    pub(crate) fn admit(
        &mut self,
        store: &HypergraphStore,
        p: D0Program,
        tick: u64,
    ) -> Result<(), String> {
        self.admit_impl(store, p, tick, None)
    }
    pub(crate) fn admit_enumerated(
        &mut self,
        store: &HypergraphStore,
        p: D0Program,
        tick: u64,
        edit: Option<(u32, u8)>,
    ) -> Result<(), String> {
        self.admit_impl(store, p, tick, Some(edit))
    }
    fn admit_impl(
        &mut self,
        store: &HypergraphStore,
        p: D0Program,
        tick: u64,
        enumerated: Option<Option<(u32, u8)>>,
    ) -> Result<(), String> {
        if self.pending.is_some() {
            return Err("mutation duplicate admission".into());
        }
        let (_, e) = store
            .nodes()
            .find(|(_, n)| n.label.0 == ESCROW)
            .ok_or("mutation missing escrow")?;
        let id = int(e, REQUEST)?;
        if id != self.records.len() as i64 + 1 {
            return Err("mutation request order".into());
        }
        let parent = int(e, OWNER)?;
        let site = store
            .edges()
            .filter(|(_, edge)| edge.label.0 == PLACE)
            .flat_map(|(_, edge)| edge.nodes.iter())
            .filter_map(|h| store.get_node(*h))
            .find(|n| n.label.0 == SITE)
            .ok_or("mutation escrow target")?;
        let start = self.draw_index;
        let (q, target, old, new, raw) = if let Some(edit) = enumerated {
            p.validate()?;
            let mut q = p.clone();
            if let Some((id, source)) = edit {
                let genes: Vec<_> = q.instructions.iter_mut().filter(|g| g.id == id).collect();
                if genes.len() != 1 || source > 7 {
                    return Err("enumerated source identity/slot".into());
                }
                let g = genes.into_iter().next().unwrap();
                let old = g.source;
                if old == source {
                    return Err("enumerated edit is unchanged".into());
                }
                g.source = source;
                (q, Some(id), Some(old), Some(source), vec![])
            } else {
                (q, None, None, None, vec![])
            }
        } else {
            self.proposal(&p)?
        };
        let error = q.validate().err();
        let valid = error.is_none();
        let law = settlement_law(&q, target, new, valid)?;
        self.records.push(MutationRecord {
            request_id: id,
            tick,
            parent,
            site: int(site, 241009)?,
            operator: if let Some(edit) = enumerated {
                if edit.is_some() {
                    "enumerated_source"
                } else {
                    "unchanged_control"
                }
            } else {
                self.kernel.name()
            }
            .into(),
            version: 1,
            stream: if enumerated.is_some() {
                "enumerated-source-v1".into()
            } else {
                format!("splitmix64-source-v1:{}", self.seed)
            },
            draw_start: start,
            draw_end: self.draw_index,
            raw_draws: raw,
            target_id: target,
            old_source: old,
            new_source: new,
            parent_bytes: raw_program_bytes(&p),
            proposed_bytes: raw_program_bytes(&q),
            parent_program: p,
            material_reachable: valid && shape(&q, q.instructions.len()).is_ok(),
            proposed_program: q,
            syntax_valid: valid,
            invalid_reason: error,
            request_charge: 1.0,
            copying_charge: 0.0,
            refunded: 0.0,
            child_id: None,
            settled: false,
            settlement_rule: law,
        });
        self.pending = Some(id);
        Ok(())
    }
    pub(crate) fn record(&self) -> Result<&MutationRecord, String> {
        self.pending
            .and_then(|i| self.records.get(i as usize - 1))
            .ok_or_else(|| "mutation missing pending request".into())
    }
    pub(crate) fn settle_endowment(
        &mut self,
        child: Option<i64>,
        endowment: f64,
    ) -> Result<(), String> {
        let i = self.pending.ok_or("mutation no request settlement")? as usize - 1;
        let r = &mut self.records[i];
        if r.settled || r.syntax_valid != child.is_some() {
            return Err("mutation incorrect settlement".into());
        }
        r.child_id = child;
        r.copying_charge = if child.is_some() {
            copy_cost(r.proposed_program.instructions.len())
        } else {
            0.0
        };
        r.refunded = if child.is_none() {
            endowment + copy_cost(r.proposed_program.instructions.len())
        } else {
            0.0
        };
        r.settled = true;
        Ok(())
    }
    pub(crate) fn dispose(&mut self) -> Result<(), String> {
        if !self.record()?.settled {
            return Err("mutation premature disposal".into());
        }
        self.pending = None;
        Ok(())
    }
    pub(crate) fn validate_endowment(
        &self,
        store: &HypergraphStore,
        endowment: f64,
    ) -> Result<(), String> {
        let escrow: Vec<_> = store.nodes().filter(|(_, n)| n.label.0 == ESCROW).collect();
        if self.records.iter().enumerate().any(|(i, r)| {
            r.request_id != i as i64 + 1 || (!r.settled && Some(r.request_id) != self.pending)
        }) {
            return Err("mutation request coverage".into());
        }
        if self.pending.is_none() {
            if !escrow.is_empty() {
                return Err("mutation hidden escrow".into());
            }
            return Ok(());
        }
        if escrow.len() != 1 {
            return Err("mutation escrow coverage".into());
        }
        let (h, e) = escrow[0];
        let r = self.record()?;
        let expected = [
            (
                ENERGY,
                PropertyValue::Float(if r.settled {
                    0.0
                } else {
                    endowment + copy_cost(r.proposed_program.instructions.len())
                }),
            ),
            (OWNER, PropertyValue::Int(r.parent)),
            (
                GENOME_LENGTH,
                PropertyValue::Int(r.proposed_program.instructions.len() as i64),
            ),
            (REQUEST, PropertyValue::Int(r.request_id)),
            (
                TX_STATUS,
                PropertyValue::Int(if !r.settled {
                    0
                } else if r.child_id.is_some() {
                    1
                } else {
                    2
                }),
            ),
        ];
        if e.energy != 0.0
            || e.properties.len() != expected.len()
            || expected
                .iter()
                .any(|(k, v)| e.get_property(*k) != Some(v.clone()))
        {
            return Err("mutation escrow properties".into());
        }
        let mut links = vec![];
        for (_, edge) in store.edges().filter(|(_, edge)| edge.nodes.contains(&h)) {
            if edge.nodes.len() != 2 {
                return Err("mutation escrow relation arity".into());
            }
            let other = *edge.nodes.iter().find(|n| **n != h).unwrap();
            let n = store.get_node(other).unwrap();
            let good = match edge.label.0 {
                FUNDS => n.label.0 == ROOT && int(n, BIRTH_ID)? == r.parent,
                PLACE => n.label.0 == SITE && int(n, 241009)? == r.site,
                _ => false,
            };
            if !good {
                return Err("mutation foreign escrow relation".into());
            }
            links.push(edge.label.0)
        }
        links.sort();
        if links != [FUNDS, PLACE] {
            return Err("mutation escrow roles".into());
        }
        Ok(())
    }
}
fn settlement_law(
    q: &D0Program,
    target: Option<u32>,
    new: Option<u8>,
    valid: bool,
) -> Result<String, String> {
    let mut value: toml::Value = MUTATION_LAWS
        .parse::<toml::Value>()
        .map_err(|e| e.to_string())?;
    let rules = value
        .get_mut("rules")
        .and_then(|v| v.as_array_mut())
        .ok_or("mutation rule table")?;
    let name = if valid {
        format!("transaction_copy_{}", q.instructions.len())
    } else {
        "transaction_reject".into()
    };
    let mut law = rules
        .iter()
        .find(|r| r["name"].as_str() == Some(&name))
        .ok_or("mutation settlement template")?
        .clone();
    if valid {
        if let Some(id) = target {
            let position = q
                .instructions
                .iter()
                .position(|g| g.id == id)
                .ok_or("mutation semantic ID")?;
            let var = format!("cg{position}");
            let effects = law["rhs"]["property_effects"].as_array_mut().unwrap();
            let effect = effects
                .iter_mut()
                .find(|f| {
                    f["var"].as_str() == Some(&var) && f["key"].as_integer() == Some(SRC as i64)
                })
                .ok_or("mutation source copy")?;
            effect.as_table_mut().unwrap().remove("source_var");
            effect.as_table_mut().unwrap().remove("source_key");
            effect["op"] = toml::Value::String("set".into());
            effect
                .as_table_mut()
                .unwrap()
                .insert("value".into(), toml::Value::Integer(new.unwrap() as i64));
        }
    }
    let mut root = toml::map::Map::new();
    root.insert("rules".into(), toml::Value::Array(vec![law]));
    toml::to_string(&toml::Value::Table(root)).map_err(|e| e.to_string())
}
/// Complete syntactic/material source-edit neighborhood; no survival inference.
pub fn source_neighborhood(p: &D0Program) -> serde_json::Value {
    let mut rows = vec![];
    for g in &p.instructions {
        for source in 0..8 {
            if source == g.source {
                continue;
            }
            let mut q = p.clone();
            q.instructions
                .iter_mut()
                .find(|x| x.id == g.id)
                .unwrap()
                .source = source;
            let valid = q.validate().is_ok();
            rows.push(serde_json::json!({"target_id":g.id,"old_source":g.source,"new_source":source,"syntax_valid":valid,"material_reachable":valid&&shape(&q,q.instructions.len()).is_ok(),"proposed_bytes":raw_program_bytes(&q)}));
        }
    }
    serde_json::json!({"program":p,"scope":"grammar_projection_only","neighbors":rows})
}

// Read the first engineering envelope without guessing the added allocation recipe.
#[derive(Deserialize)]
pub(crate) struct LegacyMutationState {
    kernel: MutationKernel,
    seed: u64,
    rng_state: u64,
    draw_index: u64,
    records: Vec<MutationRecord>,
    pending: Option<i64>,
}
impl From<LegacyMutationState> for MutationState {
    fn from(m: LegacyMutationState) -> Self {
        Self {
            kernel: m.kernel,
            seed: m.seed,
            allocation_reversed: false,
            rng_state: m.rng_state,
            draw_index: m.draw_index,
            records: m.records,
            pending: m.pending,
        }
    }
}
