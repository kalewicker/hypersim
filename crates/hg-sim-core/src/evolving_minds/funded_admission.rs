//! Separately versioned, parameter-bound funded semantic admission.
use super::{admission::*, choice::*, d0::*, family::family_program, funding::*, mutation::*};
use serde::{Deserialize, Serialize};
pub const FUNDED_ADMISSION_PROFILE: &str = "evolving-minds-d0-funded-admission-v1";
pub const FUNDED_STREAM_PROFILE: &str = "evolving-minds-d0-funded-streaming-v1";
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FundedProgram {
    Delayed,
    Immediate,
    Coin,
}
impl FundedProgram {
    pub fn program(self) -> Result<D0Program, String> {
        let mut p = family_program("delayed_cue")?;
        p.instructions
            .iter_mut()
            .find(|g| g.id == 103)
            .ok_or("funded authored instruction")?
            .source = match self {
            Self::Delayed => 2,
            Self::Immediate => 0,
            Self::Coin => 1,
        };
        p.validate()?;
        Ok(p)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FundedAdmissionConfig {
    pub seed: u64,
    pub endowment: u8,
    pub founders: [FundedProgram; 2],
    pub kernels: [MutationKernel; 2],
    pub home_offsets: [u8; 2],
    pub rotation: u8,
    pub reversed: bool,
    pub horizon: u64,
    pub scheduler: String,
}
impl FundedAdmissionConfig {
    pub fn validate(&self) -> Result<(), String> {
        validate_endowment(self.endowment)?;
        if self.rotation >= 8
            || self.home_offsets.iter().any(|h| *h >= 8)
            || self.home_offsets[0] == self.home_offsets[1]
            || self.horizon != 64
            || self.scheduler != "semantic-lottery-v1"
        {
            return Err("funded admission parameter envelope".into());
        }
        self.programs()?;
        Ok(())
    }
    pub fn programs(&self) -> Result<[D0Program; 2], String> {
        Ok([self.founders[0].program()?, self.founders[1].program()?])
    }
    pub fn homes(&self) -> [i64; 2] {
        self.home_offsets.map(|h| ((h + self.rotation) % 8) as i64)
    }
    // Coordinate adapter only: physiology and founder kernels come from the bound config.
    pub(crate) fn coordinate_recipe(&self) -> AdmissionRecipe {
        AdmissionRecipe {
            case: "faithful_disjoint".into(),
            rotation: self.rotation,
            reversed: self.reversed,
        }
    }
    pub fn law_text(&self) -> Result<String, String> {
        self.validate()?;
        let s = ChoiceSchedule::new(self.seed, CueMode::Informative);
        let old = admission_law_text(&s);
        Ok(format!(
            "{}{}",
            funded_text(ADMISSION_LAWS, self.endowment)?,
            &old[ADMISSION_LAWS.len()..]
        ))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FundedAdmissionCase {
    pub name: String,
    pub config: FundedAdmissionConfig,
}
pub fn funded_admission_cases() -> Vec<FundedAdmissionCase> {
    let mut cases = vec![];
    for (name, program) in [
        ("delayed_b24", FundedProgram::Delayed),
        ("immediate_b24", FundedProgram::Immediate),
        ("coin_b24", FundedProgram::Coin),
    ] {
        cases.push(FundedAdmissionCase {
            name: name.into(),
            config: FundedAdmissionConfig {
                seed: 17,
                endowment: 24,
                founders: [program; 2],
                kernels: [MutationKernel::Faithful; 2],
                home_offsets: [0, 4],
                rotation: 0,
                reversed: false,
                horizon: 64,
                scheduler: "semantic-lottery-v1".into(),
            },
        });
    }
    let mut add = |name: &str, index: usize, edit: fn(&mut FundedAdmissionConfig)| {
        let mut c = cases[index].config.clone();
        edit(&mut c);
        cases.push(FundedAdmissionCase {
            name: name.into(),
            config: c,
        })
    };
    add("source_b24", 0, |c| {
        c.kernels = [MutationKernel::SourceKernel; 2]
    });
    add("mixed_reject_b24", 1, |c| {
        c.kernels[0] = MutationKernel::Invalid
    });
    add("mixed_unreachable_b24", 1, |c| {
        c.kernels[0] = MutationKernel::MissingPort
    });
    add("immediate_b12", 1, |c| c.endowment = 12);
    add("source_s23_b24", 3, |c| c.seed = 23);
    add("immediate_r3_forward", 1, |c| c.rotation = 3);
    add("immediate_r3_reverse", 8, |c| c.reversed = true);
    cases
}
