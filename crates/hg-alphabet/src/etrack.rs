//! E-track run specs: the self-balancing rule the grammar previously could not
//! express.
//!
//! Committed in `docs/etrack_preregistration.md` before any E-track number
//! existed. **Nothing here can change a measured value** — it supplies rule
//! text and budgets; the signature, accounting, saturation and growth code is
//! the A-track's, unmodified (`tests/b0_instrument_pin.rs`).

use crate::dtrack;
use crate::run::RunSpec;
use hg_rules::StepMode;

/// `rebalance` — moves an incidence from the higher-degree of two matched nodes
/// to the lower-degree one, at fixed `n` and fixed edge count. Zero constants.
pub const REBALANCE: &str = include_str!("../../../rules/rebalance.toml");

/// The same bare crystal `swap`, `rewire` and `rewire_bounded` were all
/// characterised on, so the arm differs from its control in the rule and
/// nothing else.
///
/// **It is not scaled, and that is deliberate.** E1.1 established that a
/// var-to-var guard is a POST-BINDING filter: it never prunes enumeration, only
/// what enumeration produced. This LHS is two independent 2-edges, so candidate
/// enumeration is quadratic in edge count — the same shape that made `swap`
/// intractable at n = 960 (E4.2). The arm is measured at the D2.4 horizons on
/// the 6×6 crystal and nowhere larger.
pub fn rebalance_at(steps: u64) -> RunSpec {
    RunSpec::new(
        &format!("rebalance@{steps}st"),
        REBALANCE,
        dtrack::bare_crystal(),
        StepMode::MaxParallel,
    )
    .steps(steps)
    .node_cap(usize::MAX)
}
