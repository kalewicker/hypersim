//! Frozen authored genotypes, explicit interventions, and unexposed evaluation registration.
use super::{mutation::raw_program_bytes, *};
use serde::{Deserialize, Serialize};
pub const TRANSFER_GATE1_PROFILE: &str = "evolving-minds-ta1-gate1-development-v2";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferIntervention {
    pub source: u8,
    pub program: D0Program,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferCase {
    pub name: String,
    pub arm: String,
    pub config: FundedAdmissionConfig,
    pub environment: TransferEnvironment,
    pub fault: StreamFault,
    pub interventions: Vec<TransferIntervention>,
}
pub fn transfer_interventions(
    arm: &str,
) -> Result<(FundedProgram, Vec<TransferIntervention>), String> {
    let sources: &[u8] = match arm {
        "ancestor" => &[2],
        "useful" | "recovery" => &[0],
        "reversion" => &[0, 2],
        "sham" => &[0, 0],
        "restoration" => &[0, 2, 0],
        _ => return Err("unknown transfer arm".into()),
    };
    let mut p = FundedProgram::Delayed.program()?;
    let mut records = vec![];
    for &source in sources {
        p.instructions
            .iter_mut()
            .find(|g| g.id == 103)
            .ok_or("transfer g103")?
            .source = source;
        p.validate()?;
        records.push(TransferIntervention {
            source,
            bytes: raw_program_bytes(&p),
            program: p.clone(),
        });
    }
    let program = if sources.last() == Some(&2) {
        FundedProgram::Delayed
    } else {
        FundedProgram::Immediate
    };
    if p != program.program()? {
        return Err("transfer intervention genotype mismatch".into());
    }
    Ok((program, records))
}
fn case(
    seed: u64,
    pattern: TemporalPattern,
    cue_mode: CueMode,
    arm: &str,
) -> Result<TransferCase, String> {
    let (p, interventions) = transfer_interventions(arm)?;
    Ok(TransferCase {
        name: format!("world{seed}_{pattern:?}_{cue_mode:?}_{arm}"),
        arm: arm.into(),
        config: FundedAdmissionConfig {
            seed,
            endowment: 24,
            founders: [p; 2],
            kernels: [MutationKernel::Faithful; 2],
            home_offsets: [0, 4],
            rotation: 0,
            reversed: false,
            horizon: 64,
            scheduler: "semantic-lottery-v1".into(),
        },
        environment: TransferEnvironment { pattern, cue_mode },
        fault: if arm == "recovery" {
            StreamFault::CleanStop
        } else {
            StreamFault::Normal
        },
        interventions,
    })
}
pub fn transfer_gate1_cases() -> Result<Vec<TransferCase>, String> {
    let mut cases = vec![];
    for pattern in [TemporalPattern::Alternating, TemporalPattern::Blocks4] {
        for arm in ["ancestor", "useful", "reversion", "sham", "restoration"] {
            cases.push(case(17, pattern, CueMode::Informative, arm)?);
        }
    }
    for arm in ["ancestor", "useful"] {
        cases.push(case(
            17,
            TemporalPattern::Alternating,
            CueMode::Withheld,
            arm,
        )?);
    }
    cases.push(case(
        17,
        TemporalPattern::Alternating,
        CueMode::Informative,
        "recovery",
    )?);
    Ok(cases)
}
pub fn transfer_gate1_registration() -> Result<serde_json::Value, String> {
    Ok(
        serde_json::json!({"profile":TRANSFER_GATE1_PROFILE,"development_only":true,
        "cases":transfer_gate1_cases()?,"counts":{"main_worlds":1,"primary_cells":10,"diagnostic_cells":2,"initial_cells":13,"segments":14,"ticks":832,"native_advances_executed":38272,"published_states":118},
        "minimum_active_offspring_gain":1,"minimum_net_resource_gain":8,
        "required_active_faithful_transmissions":3,"transfer_established":false,"scientific_run_launched":false,
        "limits":{"panel_bytes":536870912,"native_seconds":3600,"audit_seconds":3600,"summed_advance_seconds":2400}}),
    )
}
pub fn transfer_gate1_evaluation_registration() -> Result<serde_json::Value, String> {
    use sha2::{Digest, Sha256};
    let mut blocks = vec![];
    for i in 0..8 {
        let label = format!("hypersim-ta1-temporal-evaluation-v1-{i:03}");
        let hash = Sha256::digest(label.as_bytes());
        let seed = u64::from_le_bytes(hash[..8].try_into().unwrap());
        let mut cells = vec![];
        for pattern in [TemporalPattern::Alternating, TemporalPattern::Blocks4] {
            for arm in ["ancestor", "useful", "reversion", "sham", "restoration"] {
                cells.push(case(seed, pattern, CueMode::Informative, arm)?);
            }
        }
        for arm in ["ancestor", "useful"] {
            cells.push(case(
                seed,
                TemporalPattern::Alternating,
                CueMode::Withheld,
                arm,
            )?);
        }
        blocks.push(serde_json::json!({"block":label,"seed":seed,"exposure":"registered_unexposed","cases":cells}));
    }
    Ok(
        serde_json::json!({"profile":"evolving-minds-ta1-gate1-evaluation-proposal-v1","blocks":blocks,"cells":96,"native_advances":282624,"published_states":864,
        "launch_enabled":false,"scientific_run_launched":false,"transfer_established":false,
        "budget_approved":false,"power_adequacy_established":false,
        "acceptance_proposal":"all eight blocks in each primary temporal family meet development effect and persistence floors; exact sham/restoration and reversion controls; diagnostics excluded",
        "scientific_budget":null,"execution_command_available":false}),
    )
}
