//! One original proposal per founder; descendants inherit faithfully.
use super::{admission::*, choice::mix64, d0::*, funded_admission::*, mutation::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const RETAINED_PROFILE: &str = "evolving-minds-d0-retained-proposal-v1";
pub const RETAINED_STREAM_PROFILE: &str = "evolving-minds-d0-retained-streaming-v1";
pub const RETAINED_POLICY: &str = "founder-first-proposal-v1";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RetainedState {
    pub policy: String,
    pub samples: BTreeMap<i64, MutationRecord>,
}
impl Default for RetainedState {
    fn default() -> Self {
        Self {
            policy: RETAINED_POLICY.into(),
            samples: BTreeMap::new(),
        }
    }
}
impl RetainedState {
    pub(crate) fn validate(&self, config: &FundedAdmissionConfig, tick: u64) -> Result<(), String> {
        if self.policy != RETAINED_POLICY || self.samples.len() > 2 {
            return Err("retained policy/bound".into());
        }
        for (&founder, r) in &self.samples {
            if ![1, 2].contains(&founder) {
                return Err("retained founder identity".into());
            }
            let p = config.programs()?[(founder - 1) as usize].clone();
            let kernel = config.kernels[(founder - 1) as usize];
            let seed = mix64(config.seed ^ 0x4d55544154494f4e ^ r.request_id as u64);
            let mut sampler = MutationState::new(kernel, seed);
            let (q, target, old, new, raw) = sampler.proposal(&p)?;
            let error = q.validate().err();
            if r.parent != founder
                || r.tick == 0
                || r.tick > tick
                || !(0..8).contains(&r.site)
                || r.request_id
                    != 3 + config.coordinate_recipe().logical(r.site) + 8 * r.tick as i64
                || r.parent_program != p
                || r.parent_bytes != raw_program_bytes(&p)
                || r.proposed_program != q
                || r.proposed_bytes != raw_program_bytes(&q)
                || r.operator != kernel.name()
                || r.version != 1
                || r.stream != format!("splitmix64-request-local-v1:{seed}")
                || r.draw_start != 0
                || r.draw_end != sampler.draw_index
                || r.raw_draws != raw
                || r.target_id != target
                || r.old_source != old
                || r.new_source != new
                || r.syntax_valid != error.is_none()
                || r.invalid_reason != error
                || r.material_reachable
                    != (q.validate().is_ok() && shape(&q, q.instructions.len()).is_ok())
                || r.request_charge != 1.0
                || r.copying_charge != 0.0
                || r.refunded != 0.0
                || r.child_id.is_some()
                || r.settled
                || !r.settlement_rule.is_empty()
            {
                return Err("retained first proposal provenance".into());
            }
        }
        Ok(())
    }
    /// Called only after a native reserve admitted and charged a fresh request.
    pub(crate) fn apply(
        &mut self,
        admission: &mut AdmissionState,
        parent: i64,
    ) -> Result<(), String> {
        let r = admission
            .records
            .iter_mut()
            .find(|r| r.parent == parent && !r.settled)
            .ok_or("retained new native request")?;
        if ![1, 2].contains(&parent) {
            return Ok(());
        }
        if let Some(first) = self.samples.get(&parent) {
            if r.parent_program != first.parent_program {
                return Err("retained founder genome changed".into());
            }
            r.operator = "retained_founder".into();
            r.stream = format!("retained-founder-v1:{}", first.request_id);
            r.draw_start = 0;
            r.draw_end = 0;
            r.raw_draws.clear();
            r.target_id = first.target_id;
            r.old_source = first.old_source;
            r.new_source = first.new_source;
            r.proposed_program = first.proposed_program.clone();
            r.proposed_bytes = first.proposed_bytes.clone();
            r.syntax_valid = first.syntax_valid;
            r.invalid_reason = first.invalid_reason.clone();
            r.material_reachable = first.material_reachable;
        } else {
            self.samples.insert(parent, r.clone());
        }
        Ok(())
    }
}
pub fn retained_cases() -> Vec<FundedAdmissionCase> {
    let base = funded_admission_cases();
    let mut rows = vec![];
    for (name, index, kernel, reverse) in [
        ("source17_delayed", 3, MutationKernel::SourceKernel, false),
        ("source23_delayed", 7, MutationKernel::SourceKernel, false),
        ("faithful_immediate", 1, MutationKernel::Faithful, false),
        ("invalid_immediate", 1, MutationKernel::Invalid, false),
        (
            "unreachable_immediate",
            1,
            MutationKernel::MissingPort,
            false,
        ),
        ("source23_reverse", 7, MutationKernel::SourceKernel, true),
    ] {
        let mut c = base[index].config.clone();
        c.kernels = [kernel; 2];
        c.reversed = reverse;
        rows.push(FundedAdmissionCase {
            name: name.into(),
            config: c,
        });
    }
    rows
}
