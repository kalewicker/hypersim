//! One paid early descendant proposal per founder family; all other copies faithful.
use super::{admission::*, d0::*, funded_admission::*, handoff::*, mutation::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const DESCENDANT_PROFILE: &str = "evolving-minds-d0-paid-descendant-neighborhood-v1";
pub const DESCENDANT_STREAM_PROFILE: &str = "evolving-minds-d0-paid-descendant-streaming-v1";
pub const DESCENDANT_POLICY: &str = "first-paid-descendant-by24-v1";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DescendantConfig {
    pub policy: String,
    pub target_id: Option<u32>,
    pub new_source: Option<u8>,
}
impl DescendantConfig {
    pub fn validate(&self, cfg: &FundedAdmissionConfig) -> Result<(), String> {
        cfg.validate()?;
        if self.policy != DESCENDANT_POLICY
            || cfg.endowment != 24
            || cfg.founders != [FundedProgram::Delayed; 2]
            || cfg.kernels != [MutationKernel::Faithful; 2]
            || !matches!(
                (self.target_id, self.new_source),
                (None, None) | (Some(100..=104), Some(1..=7))
            )
        {
            return Err("descendant policy/config/source binding".into());
        }
        Ok(())
    }
    fn propose(&self, r: &mut MutationRecord) -> Result<(), String> {
        if r.parent_program != FundedProgram::Immediate.program()?
            || r.request_charge != 1.0
            || r.copying_charge != 0.0
            || r.refunded != 0.0
            || r.child_id.is_some()
            || r.settled
            || !r.settlement_rule.is_empty()
        {
            return Err("descendant immediate paid proposal boundary".into());
        }
        let mut q = r.parent_program.clone();
        r.target_id = self.target_id;
        r.old_source = self.target_id.map(|_| 0);
        r.new_source = self.new_source;
        if let (Some(id), Some(source)) = (self.target_id, self.new_source) {
            q.instructions
                .iter_mut()
                .find(|g| g.id == id)
                .ok_or("descendant instruction")?
                .source = source;
        }
        r.operator = "supplied_descendant".into();
        r.stream = format!(
            "{}:{}:{}",
            DESCENDANT_POLICY,
            self.target_id.map_or("unchanged".into(), |x| x.to_string()),
            self.new_source
                .map_or("unchanged".into(), |x| x.to_string())
        );
        r.draw_start = 0;
        r.draw_end = 0;
        r.raw_draws.clear();
        r.invalid_reason = q.validate().err();
        r.syntax_valid = r.invalid_reason.is_none();
        r.material_reachable = r.syntax_valid && shape(&q, q.instructions.len()).is_ok();
        r.proposed_bytes = raw_program_bytes(&q);
        r.proposed_program = q;
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DescendantState {
    pub config: DescendantConfig,
    pub targets: BTreeMap<i64, MutationRecord>,
}
impl DescendantState {
    pub(crate) fn validate(&self, cfg: &FundedAdmissionConfig, tick: u64) -> Result<(), String> {
        self.config.validate(cfg)?;
        if self.targets.len() > 2 {
            return Err("descendant target bound".into());
        }
        for (&founder, r) in &self.targets {
            if ![1, 2].contains(&founder)
                || r.parent <= 2
                || r.parent >= r.request_id
                || r.tick == 0
                || r.tick > 24
                || r.tick > tick
                || !(0..8).contains(&r.site)
                || r.request_id != 3 + cfg.coordinate_recipe().logical(r.site) + 8 * r.tick as i64
                || r.parent_bytes != raw_program_bytes(&r.parent_program)
                || r.version != 1
            {
                return Err("descendant first paid request provenance".into());
            }
            let mut expected = r.clone();
            self.config.propose(&mut expected)?;
            if expected != *r {
                return Err("descendant supplied receipt provenance".into());
            }
        }
        Ok(())
    }
    /// Call in semantic parent-ID order after all native reserves are admitted.
    pub(crate) fn apply(
        &mut self,
        a: &mut AdmissionState,
        parent: i64,
        founder: i64,
    ) -> Result<(), String> {
        if parent == founder {
            return HandoffConfig {
                policy: HANDOFF_POLICY.into(),
                supplied: SuppliedEdit::Useful,
                descendant_kernel: MutationKernel::Faithful,
            }
            .apply(a, parent);
        }
        let r = a
            .records
            .iter_mut()
            .find(|r| r.parent == parent && !r.settled)
            .ok_or("descendant native request")?;
        if r.tick <= 24 && !self.targets.contains_key(&founder) {
            self.config.propose(r)?;
            self.targets.insert(founder, r.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deadline_and_one_shot_do_not_retry_invalid_targets() {
        let mut cfg = super::super::descendant_cases()[0].config.clone();
        cfg.seed = 17;
        let config = DescendantConfig {
            policy: DESCENDANT_POLICY.into(),
            target_id: Some(104),
            new_source: Some(1),
        };
        let mut sim = super::super::FamilySimulation::new_descendant(cfg, config.clone()).unwrap();
        for _ in 0..24 {
            sim.advance_tick().unwrap();
        }
        let mut state = sim.descendant_state().unwrap().clone();
        let mut sample = state.targets[&1].clone();
        assert!(!sample.syntax_valid);
        let mut a = sim.admission.as_ref().unwrap().clone();
        sample.tick = 25;
        sample.parent += 1;
        sample.request_id += 8;
        a.records = vec![sample.clone()];
        let mut absent = DescendantState {
            config,
            targets: BTreeMap::new(),
        };
        absent.apply(&mut a, sample.parent, 1).unwrap();
        assert!(absent.targets.is_empty());
        assert_eq!(a.records[0], sample);
        sample.tick = 24;
        a.records = vec![sample.clone()];
        let before = state.clone();
        state.apply(&mut a, sample.parent, 1).unwrap();
        assert_eq!(state, before);
        assert_eq!(a.records[0], sample);
    }
}
