//! Threshold constants — single source of truth is
//! docs/metric_formulas.md; these values must match that table.

pub const GROWTH_EXPLODE_MIN: f64 = 1.5;
pub const NODE_CAP: usize = 100_000;
pub const GROWTH_MIN: f64 = 1.01;
/// Novelty boundary (plan default 0.01, NOT 0): "invents nothing"
/// regimes tolerate `rate <= min`, GenerativeCandidate requires
/// `rate > min`. Strictly-zero was a spec deviation that broke
/// GrowthTreadmill for any rule with growing-degree hubs — WL radius-1
/// classes are degree-sensitive, so an ever-growing hub mints ~1 fresh
/// bin per window forever (a constant trickle, not real invention).
pub const NEW_TYPE_BIN_RATE_MIN: f64 = 0.01;
pub const JSD_NOISE_MIN: f64 = 0.15;
pub const PI_NOISE_MAX: f64 = 0.2;
pub const ACTIVITY_CHAOS_MIN: f64 = 0.5;
pub const MEAN_DEGREE_FRAG_MAX: f64 = 0.5;
pub const DOMINANT_MONOCULTURE_MIN: f64 = 0.9;
pub const INTERESTINGNESS_NOISE_MAX: f64 = 0.5;
pub const SUSTAIN_WINDOWS: usize = 3;
pub const TREADMILL_STUCK_WINDOWS: usize = 8;
pub const PI_WARMUP_WINDOWS: usize = 4;
pub const PI_JUMP_MAX: f64 = 0.5;
pub const PI_SAMPLE_CAP: usize = 4096;
pub const ARCHIVE_CAPACITY: usize = 10_000;
pub const WL_RADIUS: u8 = 1;
