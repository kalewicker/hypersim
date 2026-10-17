//! The eight gates (docs/gate_formulas.md). Every predicate here traces to
//! that document; constants to crate::thresholds.

use crate::thresholds::*;
use crate::window::WindowFeatures;
use hg_schema::{GateName, GateStatus, Regime};

pub struct GateEngine {
    fixture_gate: GateStatus,
    white_noise: GateStatus,
    pi: GateStatus,
    generative: GateStatus,
    treadmill: GateStatus,
    mobile: GateStatus,
    completeness: GateStatus,
    pi_history: Vec<f64>,
    regime_history: Vec<Regime>,
    novelty_history: Vec<f64>,
}

impl GateEngine {
    /// `fixture_gate_result` comes from running the compiled-in truth table
    /// (gate 1 is evaluated eagerly at run start).
    pub fn new(fixture_gate_result: GateStatus) -> Self {
        Self {
            fixture_gate: fixture_gate_result,
            white_noise: GateStatus::Pending,
            pi: GateStatus::Pending,
            generative: GateStatus::Pending,
            treadmill: GateStatus::Pending,
            mobile: GateStatus::Pending,
            completeness: GateStatus::Pending,
            pi_history: Vec::new(),
            regime_history: Vec::new(),
            novelty_history: Vec::new(),
        }
    }

    /// Evaluate the per-window gates.
    pub fn on_window(&mut self, f: &WindowFeatures, regime: Regime, interestingness: f64) {
        if let Some(pi) = f.pi_normalized {
            self.pi_history.push(pi);
        }
        self.regime_history.push(regime);
        self.novelty_history.push(f.new_type_bin_rate);

        // 2. White-noise gate.
        let is_noise = matches!(regime, Regime::WhiteNoiseSoup | Regime::SterileChaos);
        self.white_noise = if is_noise && interestingness >= INTERESTINGNESS_NOISE_MAX {
            GateStatus::Fail
        } else if self.white_noise != GateStatus::Fail {
            GateStatus::Pass
        } else {
            GateStatus::Fail
        };

        // 3. PI gate: warmup then stability.
        if self.pi != GateStatus::Fail {
            if self.pi_history.len() >= PI_WARMUP_WINDOWS {
                let jumped = self
                    .pi_history
                    .windows(2)
                    .any(|w| (w[1] - w[0]).abs() > PI_JUMP_MAX);
                self.pi = if jumped { GateStatus::Fail } else { GateStatus::Pass };
            } else {
                self.pi = GateStatus::Pending;
            }
        }

        // 4. Generative gate: sustained GenerativeCandidate with novelty.
        if self.generative != GateStatus::Fail && self.generative != GateStatus::Pass {
            let n = self.regime_history.len();
            if n >= SUSTAIN_WINDOWS {
                let recent_regimes = &self.regime_history[n - SUSTAIN_WINDOWS..];
                let recent_novelty = &self.novelty_history[n - SUSTAIN_WINDOWS..];
                if recent_regimes.iter().all(|r| *r == Regime::GenerativeCandidate) {
                    self.generative = if recent_novelty.iter().all(|&r| r > 0.0) {
                        GateStatus::Pass
                    } else {
                        GateStatus::Fail // regime claims generative, novelty collapsed
                    };
                }
            }
        }

        // 5. Treadmill gate: stuck = TREADMILL_STUCK_WINDOWS of pure treadmill.
        if self.treadmill != GateStatus::Fail {
            let n = self.regime_history.len();
            if n >= TREADMILL_STUCK_WINDOWS {
                let recent = &self.regime_history[n - TREADMILL_STUCK_WINDOWS..];
                let stuck = recent.iter().all(|r| *r == Regime::GrowthTreadmill)
                    && self.novelty_history[n - TREADMILL_STUCK_WINDOWS..]
                        .iter()
                        .all(|&r| r == 0.0);
                self.treadmill = if stuck { GateStatus::Fail } else { GateStatus::Pass };
            } else {
                self.treadmill = GateStatus::Pending;
            }
        }

        // 6. Mobile gate.
        // Once observed, mobility stays observed.
        let mobile_seen = f.active_persistent_structures >= 1 || self.mobile == GateStatus::Pass;
        self.mobile = if mobile_seen {
            GateStatus::Pass
        } else if f.has_prev {
            GateStatus::Fail
        } else {
            GateStatus::Pending
        };
    }

    /// 8\. Completeness gate, evaluated at run end: every `[computed]`
    /// field populated at least once. The caller reports which were.
    pub fn finalize_completeness(&mut self, all_computed_fields_populated: bool) {
        self.completeness = if all_computed_fields_populated {
            GateStatus::Pass
        } else {
            GateStatus::Fail
        };
    }

    pub fn statuses(&self) -> Vec<(GateName, GateStatus)> {
        vec![
            (GateName::Fixture, self.fixture_gate),
            (GateName::WhiteNoise, self.white_noise),
            (GateName::Pi, self.pi),
            (GateName::Generative, self.generative),
            (GateName::Treadmill, self.treadmill),
            (GateName::Mobile, self.mobile),
            // 7. Controller: reserved — permanently Pending in the
            // substrate-only build (docs/gate_formulas.md).
            (GateName::Controller, GateStatus::Pending),
            (GateName::Completeness, self.completeness),
        ]
    }

    /// Structurally unreachable until the cognition layer supplies gate 7 —
    /// intended (docs/gate_formulas.md).
    pub fn science_unlocked(&self) -> bool {
        self.statuses().iter().all(|(_, s)| *s == GateStatus::Pass)
    }
}
