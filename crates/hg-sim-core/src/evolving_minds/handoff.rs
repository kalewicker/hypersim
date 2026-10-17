//! Constant supplied founder proposals, admitted through native paid reservation.
use super::{admission::*, d0::*, funded_admission::*, mutation::*};
use serde::{Deserialize, Serialize};
pub const HANDOFF_PROFILE: &str = "evolving-minds-d0-paid-handoff-v1";
pub const HANDOFF_STREAM_PROFILE: &str = "evolving-minds-d0-handoff-streaming-v1";
pub const HANDOFF_POLICY: &str = "paid-beneficial-edit-v1";
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SuppliedEdit {
    Useful,
    Unchanged,
    Invalid,
    Unreachable,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandoffConfig {
    pub policy: String,
    pub supplied: SuppliedEdit,
    pub descendant_kernel: MutationKernel,
}
impl HandoffConfig {
    pub fn validate(&self, c: &FundedAdmissionConfig) -> Result<(), String> {
        c.validate()?;
        if self.policy != HANDOFF_POLICY
            || c.founders != [FundedProgram::Delayed; 2]
            || !matches!(
                self.descendant_kernel,
                MutationKernel::Faithful | MutationKernel::SourceKernel
            )
            || c.kernels != [self.descendant_kernel; 2]
        {
            return Err("paid handoff policy/founder/kernel binding".into());
        }
        Ok(())
    }
    pub(crate) fn apply(&self, a: &mut AdmissionState, parent: i64) -> Result<(), String> {
        if ![1, 2].contains(&parent) {
            return Ok(());
        }
        let r = a
            .records
            .iter_mut()
            .find(|r| r.parent == parent && !r.settled)
            .ok_or("handoff paid native request")?;
        if r.parent_program != FundedProgram::Delayed.program()?
            || r.request_charge != 1.0
            || r.copying_charge != 0.0
            || r.refunded != 0.0
            || r.child_id.is_some()
        {
            return Err("handoff founder paid boundary".into());
        }
        let mut q = r.parent_program.clone();
        let edit = match self.supplied {
            SuppliedEdit::Useful => Some((103, 0)),
            SuppliedEdit::Unreachable => Some((103, 7)),
            SuppliedEdit::Invalid => Some((
                q.instructions
                    .iter()
                    .find(|g| g.op == D0Op::Halt)
                    .ok_or("handoff halt")?
                    .id,
                1,
            )),
            SuppliedEdit::Unchanged => None,
        };
        r.target_id = None;
        r.old_source = None;
        r.new_source = None;
        if let Some((id, source)) = edit {
            let g = q
                .instructions
                .iter_mut()
                .find(|g| g.id == id)
                .ok_or("handoff instruction")?;
            r.target_id = Some(id);
            r.old_source = Some(g.source);
            r.new_source = Some(source);
            g.source = source;
        }
        r.operator = "supplied_founder".into();
        r.stream = format!("{}:{:?}", HANDOFF_POLICY, self.supplied);
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
