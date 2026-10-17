//! Gate vocabulary — pure data shared by the evaluator, the app layer, and
//! the report (v4 §9.1 references `GateName`/`GateStatus`).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GateStatus {
    Pass,
    Fail,
    #[default]
    Pending,
}

impl GateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            GateStatus::Pass => "Pass",
            GateStatus::Fail => "Fail",
            GateStatus::Pending => "Pending",
        }
    }
}

/// The eight gates (docs/gate_formulas.md). `Controller` is reserved and
/// permanently `Pending` in the substrate-only build.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateName {
    Fixture,
    WhiteNoise,
    Pi,
    Generative,
    Treadmill,
    Mobile,
    Controller,
    Completeness,
}

impl GateName {
    pub const ALL: [GateName; 8] = [
        GateName::Fixture,
        GateName::WhiteNoise,
        GateName::Pi,
        GateName::Generative,
        GateName::Treadmill,
        GateName::Mobile,
        GateName::Controller,
        GateName::Completeness,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            GateName::Fixture => "Fixture",
            GateName::WhiteNoise => "WhiteNoise",
            GateName::Pi => "Pi",
            GateName::Generative => "Generative",
            GateName::Treadmill => "Treadmill",
            GateName::Mobile => "Mobile",
            GateName::Controller => "Controller",
            GateName::Completeness => "Completeness",
        }
    }
}
