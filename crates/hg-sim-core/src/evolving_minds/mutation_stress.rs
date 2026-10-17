//! Fixed development comparison; never opens registered held-out transfer worlds.
use super::*;
use serde::{Deserialize, Serialize};
pub const MUTATION_STRESS_PANEL_PROFILE: &str = "evolving-minds-ta1-mutation-stress-development-v1";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MutationStressCase {
    pub name: String,
    pub arm: String,
    pub config: FundedAdmissionConfig,
    pub environment: TransferEnvironment,
    pub fault: StreamFault,
}
pub fn mutation_stress_cases() -> Result<Vec<MutationStressCase>, String> {
    let mut cases = vec![];
    for seed in [17, 23] {
        for pattern in [TemporalPattern::Alternating, TemporalPattern::Blocks4] {
            for (arm, founder, kernel) in [
                ("ancestor", FundedProgram::Delayed, MutationKernel::Faithful),
                (
                    "faithful",
                    FundedProgram::Immediate,
                    MutationKernel::Faithful,
                ),
                (
                    "source",
                    FundedProgram::Immediate,
                    MutationKernel::SourceKernel,
                ),
                (
                    "grammar",
                    FundedProgram::Immediate,
                    MutationKernel::GrammarSourceKernel,
                ),
            ] {
                cases.push(MutationStressCase {
                    name: format!("world{seed}_{pattern:?}_{arm}"),
                    arm: arm.into(),
                    config: FundedAdmissionConfig {
                        seed,
                        endowment: 24,
                        founders: [founder; 2],
                        kernels: [kernel; 2],
                        home_offsets: [0, 4],
                        rotation: 0,
                        reversed: false,
                        horizon: 64,
                        scheduler: "semantic-lottery-v1".into(),
                    },
                    environment: TransferEnvironment {
                        pattern,
                        cue_mode: CueMode::Informative,
                    },
                    fault: StreamFault::Normal,
                });
            }
        }
    }
    let mut recovery = cases[3].clone();
    recovery.name = "world17_Alternating_recovery".into();
    recovery.arm = "recovery".into();
    recovery.fault = StreamFault::CleanStop;
    cases.push(recovery);
    for c in &cases {
        c.environment.validate_stress(&c.config)?;
    }
    Ok(cases)
}
pub fn mutation_stress_registration() -> Result<serde_json::Value, String> {
    Ok(
        serde_json::json!({"profile":MUTATION_STRESS_PANEL_PROFILE,"development_only":true,"cases":mutation_stress_cases()?,
        "counts":{"worlds":2,"patterns":2,"primary_cells":16,"initial_cells":17,"segments":18,"ticks":1088,"native_advances_executed":50048,"published_states":154},
        "mutation_probability":"3/4","minimum_paid_attempts_per_functional_root":4,"minimum_success_fraction":"3/4",
        "required_functional_paid_transmissions":3,"minimum_active_offspring_gain":1,"minimum_net_resource_gain":8,
        "grammar_advance_requires_all_four_world_pattern_cells":true,"original_source_is_comparator_not_required_gate":true,
        "scientific_run_launched":false,"discovery_established":false,"transfer_established":false,
        "limits":{"panel_bytes":536870912,"native_seconds":3600,"audit_seconds":3600,"summed_advance_seconds":2400}}),
    )
}
pub fn mutation_stress_probe_registration() -> Result<serde_json::Value, String> {
    let all = mutation_stress_cases()?;
    let mut r = mutation_stress_registration()?;
    r["profile"] = "evolving-minds-ta1-mutation-stress-probe-v1".into();
    r["cases"] = serde_json::to_value(vec![all[3].clone(), all.last().unwrap().clone()])
        .map_err(|e| e.to_string())?;
    r["counts"] = serde_json::json!({"worlds":1,"patterns":1,"primary_cells":1,"initial_cells":2,"segments":3,"ticks":128,"native_advances_executed":5888,"published_states":19});
    Ok(r)
}
