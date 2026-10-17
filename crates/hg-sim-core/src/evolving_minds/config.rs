use serde::{Deserialize, Serialize};

pub const PROFILE: &str = "evolving-minds-engineering-v1";
pub const RULES: &str = include_str!("../../../../rules/evolving_minds/engineering_v1.toml");

/// An authored calibration program, not an evolved controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    Idle,
    BuildRelay,
}
impl Instruction {
    pub fn opcode(self) -> i64 {
        match self {
            Self::Idle => 0,
            Self::BuildRelay => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub seed: u64,
    pub sites: usize,
    pub ticks: u64,
    pub max_nodes: usize,
    pub max_edges: usize,
    pub match_work: usize,
    pub match_results: usize,
    pub output_bytes: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 42,
            sites: 16,
            ticks: 64,
            max_nodes: 256,
            max_edges: 512,
            match_work: 100_000,
            match_results: 10_000,
            output_bytes: 64 * 1024 * 1024,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !(3..=64).contains(&self.sites)
            || !(1..=1024).contains(&self.ticks)
            || !(1..=100_000).contains(&self.max_nodes)
            || !(1..=200_000).contains(&self.max_edges)
            || self.match_work == 0
            || self.match_work > 1_000_000
            || self.match_results == 0
            || self.match_results > 100_000
            || self.output_bytes < 1024 * 1024
            || self.output_bytes > 256 * 1024 * 1024
        {
            return Err("config outside bounded engineering envelope".into());
        }
        if self.max_nodes < self.sites + 4 || self.max_edges < self.sites + 3 {
            return Err("initial fixture exceeds material ceilings".into());
        }
        Ok(())
    }
}

// Profile-local namespaces. Below META_BASE; no existing WL/ERE schema changed.
pub const ROOT: u32 = 0xE100_0000;
pub const SITE: u32 = ROOT + 1;
pub const SENSOR: u32 = ROOT + 2;
pub const GENE: u32 = ROOT + 3;
pub const CONNECTION: u32 = ROOT + 4;
pub const ACTUATOR: u32 = ROOT + 5;
pub const COUNTER: u32 = ROOT + 6;
pub const DEAD: u32 = ROOT + 7;
pub const OCCUPANCY: u32 = ROOT + 100;
pub const NEIGHBOR: u32 = ROOT + 101;
pub const OWN_SENSOR: u32 = ROOT + 102;
pub const OWN_GENE: u32 = ROOT + 103;
pub const OWN_CONNECTION: u32 = ROOT + 104;
pub const OWN_ACTUATOR: u32 = ROOT + 105;
pub const SOURCE: u32 = ROOT + 106;
pub const TARGET: u32 = ROOT + 107;
pub const ENERGY: u32 = 1;
pub const OWNER: u32 = 240_001;
pub const OPCODE: u32 = 240_002;
pub const DEVELOPED: u32 = 240_003;
pub const AGE: u32 = 240_004;
pub const SIGNAL: u32 = 240_005;
pub const OCCUPIED: u32 = 240_006;
pub const BIRTH_ID: u32 = 240_007;
pub const PARENT_ID: u32 = 240_008;
pub const NEXT_ID: u32 = 240_009;
pub const MAINTENANCE: u32 = 240_010;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Environment,
    Sense,
    Signal,
    Development,
    Action,
    Maintenance,
    Birth,
    Death,
    Cleanup,
    World,
    Attempt,
    Settle,
}
pub const PHASES: [Phase; 9] = [
    Phase::Environment,
    Phase::Sense,
    Phase::Signal,
    Phase::Development,
    Phase::Action,
    Phase::Maintenance,
    Phase::Birth,
    Phase::Death,
    Phase::Cleanup,
];
impl Phase {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Environment => "environment_",
            Self::Sense => "sense_",
            Self::Signal => "signal_",
            Self::Development => "development_",
            Self::Action => "action_",
            Self::Maintenance => "maintenance_",
            Self::Birth => "birth_",
            Self::Death => "death_",
            Self::Cleanup => "cleanup_",
            Self::World => "world_",
            Self::Attempt => "attempt_",
            Self::Settle => "settle_",
        }
    }
}
