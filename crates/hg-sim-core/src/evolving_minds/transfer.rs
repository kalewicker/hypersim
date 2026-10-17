//! TA-1 Gate 0 composes the unchanged paid handoff lifecycle.
use super::*;
use serde::{Deserialize, Serialize};

pub const TRANSFER_GATE0_PROFILE: &str = "evolving-minds-ta1-gate0-development-v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransferGate0Config {
    pub profile: String,
    pub development_only: bool,
    pub worlds: Vec<u64>,
    pub horizon: u64,
    pub required_faithful_active_transmissions: u64,
    pub minimum_active_offspring_gain: u64,
    pub minimum_net_resource_gain: u64,
}

impl Default for TransferGate0Config {
    fn default() -> Self {
        Self {
            profile: TRANSFER_GATE0_PROFILE.into(),
            development_only: true,
            worlds: vec![17, 23],
            horizon: 64,
            required_faithful_active_transmissions: 3,
            minimum_active_offspring_gain: 1,
            minimum_net_resource_gain: 8,
        }
    }
}

impl TransferGate0Config {
    pub fn validate(&self) -> Result<(), String> {
        if self != &Self::default() {
            return Err("TA-1 Gate0 v1 requires the complete fixed development contract".into());
        }
        Ok(())
    }

    pub fn cases(&self) -> Result<Vec<HandoffCase>, String> {
        self.validate()?;
        let mut rows = vec![];
        for (seed, arm, supplied, fault) in [
            (17, "useful", SuppliedEdit::Useful, StreamFault::Normal),
            (17, "ancestor", SuppliedEdit::Unchanged, StreamFault::Normal),
            (23, "useful", SuppliedEdit::Useful, StreamFault::Normal),
            (23, "ancestor", SuppliedEdit::Unchanged, StreamFault::Normal),
            (17, "recovery", SuppliedEdit::Useful, StreamFault::CleanStop),
        ] {
            rows.push(HandoffCase {
                name: format!("world{seed}_{arm}"),
                arm: arm.into(),
                config: FundedAdmissionConfig {
                    seed,
                    endowment: 24,
                    founders: [FundedProgram::Delayed; 2],
                    kernels: [MutationKernel::Faithful; 2],
                    home_offsets: [0, 4],
                    rotation: 0,
                    reversed: false,
                    horizon: self.horizon,
                    scheduler: "semantic-lottery-v1".into(),
                },
                handoff: HandoffConfig {
                    policy: HANDOFF_POLICY.into(),
                    supplied,
                    descendant_kernel: MutationKernel::Faithful,
                },
                fault,
            });
        }
        for row in &rows {
            row.handoff.validate(&row.config)?;
        }
        Ok(rows)
    }

    pub fn registration(&self) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "config": self, "cases": self.cases()?,
            "counts": {"main_worlds":2,"main_cells":4,"initial_cells":5,
                "segments":6,"ticks":320,"native_advances_executed":14720,
                "published_states":46},
            "scientific_run_launched":false,"transfer_established":false,
            "discovery_established":false,"cumulative_reuse_established":false,
            "limits":{"panel_bytes":536870912,"native_seconds":3600,
                "audit_seconds":3600,"summed_advance_seconds":2400}
        }))
    }
}
