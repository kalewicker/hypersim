//! Lawful communication controls through complete paid family lifecycles.
use super::{
    choice::{ChoiceSchedule, CueMode},
    family::{family_law_text, family_program, FamilySimulation},
};
use serde::{Deserialize, Serialize};
pub const LIFECYCLE_PROFILE: &str = "evolving-minds-d0-lifecycle-controls-v1";
pub const LIFECYCLE_LAWS: &str = include_str!("../../../../rules/evolving_minds/lifecycle_v1.toml");
pub const LIFECYCLE_STEPS: u64 = 44;
pub const COMMUNICATION_ENABLED: u32 = 242016;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LifecycleArm {
    Sham,
    Freeze,
    Rescue,
    Release,
}
impl LifecycleArm {
    pub fn name(self) -> &'static str {
        match self {
            Self::Sham => "sham",
            Self::Freeze => "freeze",
            Self::Rescue => "rescue",
            Self::Release => "release",
        }
    }
    pub fn law(self, tick: u64) -> &'static str {
        if tick <= 3 {
            return "communication_sham";
        }
        match self {
            Self::Sham => "communication_sham",
            Self::Freeze => "communication_freeze",
            Self::Rescue => "communication_rescue",
            Self::Release if tick <= 8 => "communication_freeze",
            Self::Release => "communication_sham",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleState {
    pub arm: LifecycleArm,
}
pub fn lifecycle_law_text(s: &ChoiceSchedule) -> String {
    let text = family_law_text(s);
    let start = text.find("\n[[rules]]\nname = \"world_").unwrap();
    format!("{LIFECYCLE_LAWS}{}", &text[start..])
}
impl FamilySimulation {
    pub fn new_lifecycle(arm: LifecycleArm) -> Result<Self, String> {
        Self::configure_lifecycle(
            family_program("cue")?,
            17,
            CueMode::Informative,
            LifecycleState { arm },
        )
    }
    pub fn fork_lifecycle(bytes: &[u8], arm: LifecycleArm) -> Result<Self, String> {
        let mut s = Self::resume(bytes)?;
        if s.rows.len() != 3
            || s.lifecycle.as_ref().map(|v| v.arm) != Some(LifecycleArm::Sham)
            || s.mutation.is_some()
        {
            return Err("lifecycle fork requires sham-origin tick 3".into());
        }
        s.lifecycle.as_mut().unwrap().arm = arm;
        s.validate()?;
        Ok(s)
    }
}
