//! Rolling evaluator state (shared with the GUI via hg-sim-core).

use crate::window::WindowFeatures;
use hg_schema::{GateName, GateStatus, Regime};

#[derive(Default, Clone, Debug)]
pub struct EvalState {
    pub current_regime: Regime,
    pub regime_history: Vec<Regime>,
    pub pi_history: Vec<f64>,
    pub interestingness_history: Vec<f64>,
    pub features_history: Vec<WindowFeatures>,
    pub gate_statuses: Vec<(GateName, GateStatus)>,
    pub science_unlocked: bool,
}
