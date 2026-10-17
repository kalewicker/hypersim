//! Explicit temporal intervention; no spatial or physiological changes.
use super::{admission::*, choice::mix64, funding::funded_text, *};
use serde::{Deserialize, Serialize};
pub const TRANSFER_ENV_PROFILE: &str = "evolving-minds-ta1-temporal-v1";
pub const TRANSFER_ENV_STREAM_PROFILE: &str = "evolving-minds-ta1-temporal-streaming-v1";
pub const MUTATION_STRESS_PROFILE: &str = "evolving-minds-ta1-mutation-stress-v1";
pub const MUTATION_STRESS_STREAM_PROFILE: &str = "evolving-minds-ta1-mutation-stress-streaming-v1";
pub const MUTATION_STRESS_POLICY: &str = "grammar-source-stress-v1";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemporalPattern {
    Shuffled,
    Alternating,
    Blocks4,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransferEnvironment {
    pub pattern: TemporalPattern,
    pub cue_mode: CueMode,
}
impl TransferEnvironment {
    pub fn validate_stress(&self, cfg: &FundedAdmissionConfig) -> Result<(), String> {
        cfg.validate()?;
        if cfg.endowment != 24
            || cfg.kernels[0] != cfg.kernels[1]
            || !matches!(
                cfg.kernels[0],
                MutationKernel::Faithful
                    | MutationKernel::SourceKernel
                    | MutationKernel::GrammarSourceKernel
            )
            || !(cfg.founders == [FundedProgram::Immediate; 2]
                || (cfg.founders == [FundedProgram::Delayed; 2]
                    && cfg.kernels == [MutationKernel::Faithful; 2]))
            || self.cue_mode != CueMode::Informative
            || self.pattern == TemporalPattern::Shuffled
        {
            return Err("fixed mutation-stress physiology/kernel envelope".into());
        }
        Ok(())
    }
    pub fn schedule(&self, seed: u64) -> ChoiceSchedule {
        let mut s = ChoiceSchedule::new(seed, self.cue_mode);
        let phase = mix64(seed ^ 0x5441315f50484153) & 1 != 0;
        match self.pattern {
            TemporalPattern::Shuffled => {}
            TemporalPattern::Alternating => {
                s.resources = (0..64).map(|t| (t % 2 == 1) ^ phase).collect()
            }
            TemporalPattern::Blocks4 => {
                s.resources = (0..64).map(|t| ((t / 4) % 2 == 1) ^ phase).collect()
            }
        }
        if self.cue_mode == CueMode::Informative {
            s.cues = s.resources.clone();
        }
        s
    }
    pub fn law_text(&self, cfg: &FundedAdmissionConfig) -> Result<String, String> {
        cfg.validate()?;
        let old = admission_law_text(&self.schedule(cfg.seed));
        Ok(format!(
            "{}{}",
            funded_text(ADMISSION_LAWS, cfg.endowment)?,
            &old[ADMISSION_LAWS.len()..]
        ))
    }
    pub fn validate(&self, cfg: &FundedAdmissionConfig) -> Result<(), String> {
        cfg.validate()?;
        if cfg.kernels != [MutationKernel::Faithful; 2] {
            return Err("frozen transfer requires faithful inheritance".into());
        }
        Ok(())
    }
}
