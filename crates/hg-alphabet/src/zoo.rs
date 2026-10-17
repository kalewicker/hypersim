//! The rule zoo under measurement, with the budgets committed in
//! `docs/atrack_a1_preregistration.md` §7.
//!
//! Every spec here is bare substrate. The bounded-churn substrate is included
//! because it is the most interesting case in the zoo — flat `n(t)` with full
//! identity turnover — and because A4.3 asks for its curve separately.

use crate::run::{Init, RunSpec};
use hg_rules::StepMode;

/// The N0-v2 derived churn burn-in. The M1 correction is binding: churn must be
/// scored after burn-in, because its startup transients dominate the class
/// count (~215 of 231 classes there were startup-only).
pub const CHURN_BURN_IN: u64 = 900;

/// The builtin rule sources, embedded from the same files `hg_rules::builtins`
/// embeds — so the zoo measures the shipped rules, not a copy that can drift.
fn builtin(name: &str) -> String {
    match name {
        "triangle_subdivide" => include_str!("../../../rules/triangle_subdivide.toml"),
        "binary_edge" => include_str!("../../../rules/binary_edge.toml"),
        "tetrahedral" => include_str!("../../../rules/tetrahedral.toml"),
        "glider" => include_str!("../../../rules/glider.toml"),
        "growth_treadmill" => include_str!("../../../rules/growth_treadmill.toml"),
        other => panic!("unknown builtin {other}"),
    }
    .to_string()
}

pub fn bounded_churn_toml() -> &'static str {
    include_str!("../../../rules/bounded_churn.toml")
}

/// The calibration pair (A2.2): known-poor versus known-rich.
pub fn glider() -> RunSpec {
    RunSpec::new(
        "glider",
        &builtin("glider"),
        Init::GliderRing { n: 24 },
        StepMode::SequentialRandom,
    )
    .steps(2000)
    .node_cap(usize::MAX)
}

pub fn triangle_subdivide() -> RunSpec {
    RunSpec::new(
        "triangle_subdivide",
        &builtin("triangle_subdivide"),
        Init::PureCrystal {
            w: 6,
            h: 6,
            arity: 3,
            seed_label1: false,
        },
        StepMode::MaxParallel,
    )
    .steps(400)
    .node_cap(20_000)
}

pub fn binary_edge() -> RunSpec {
    RunSpec::new(
        "binary_edge",
        &builtin("binary_edge"),
        Init::PureCrystal {
            w: 6,
            h: 6,
            arity: 2,
            seed_label1: false,
        },
        StepMode::MaxParallel,
    )
    .steps(400)
    .node_cap(20_000)
}

/// **Reduced step budget, and why.** The committed budget was 400 steps / 20000
/// nodes, but `tetrahedral` grows by exactly ONE node per step (all K4s in the
/// structure overlap, so node-disjoint conflict resolution admits one firing),
/// while the matcher's cost of enumerating K4 subgraphs rises ~3x per +20 steps
/// (measured: 0.02 s at 10 steps, 15.4 s at 80). The node cap can therefore
/// never bind and the run does not terminate in practice.
///
/// The budget is cut on RUNTIME, which is not a result-dependent choice — and
/// in this case it demonstrably cannot bias the verdict, because `|A|` equals
/// `n_events` exactly at every probed budget: the run sits on the hard
/// `|A| <= n_events` ceiling, so no budget makes its alphabet look larger or
/// smaller relative to what it could show. The stop reason is published with
/// the number.
pub fn tetrahedral() -> RunSpec {
    RunSpec::new(
        "tetrahedral",
        &builtin("tetrahedral"),
        Init::K4,
        StepMode::MaxParallel,
    )
    .steps(120)
    .node_cap(20_000)
}

pub fn growth_treadmill() -> RunSpec {
    RunSpec::new(
        "growth_treadmill",
        &builtin("growth_treadmill"),
        Init::PureCrystal {
            w: 6,
            h: 6,
            arity: 3,
            seed_label1: true,
        },
        StepMode::MaxParallel,
    )
    .steps(400)
    .node_cap(20_000)
}

/// The report's negative control: the same rule on a random substrate, which is
/// where the 304-class node registry came from.
pub fn growth_treadmill_on_noise() -> RunSpec {
    RunSpec::new(
        "growth_treadmill@noise",
        &builtin("growth_treadmill"),
        Init::WhiteNoise {
            nodes: 36,
            edges: 72,
            max_arity: 3,
            label_range: 2,
            seed: 42,
            seed_label1: true,
        },
        StepMode::MaxParallel,
    )
    .steps(400)
    .node_cap(20_000)
}

pub fn bounded_churn() -> RunSpec {
    RunSpec::new(
        "bounded_churn",
        bounded_churn_toml(),
        Init::ChurnCrystal {
            w: 6,
            h: 6,
            arity: 3,
            fill: 5.0,
        },
        StepMode::MaxParallel,
    )
    .steps(2400)
    .node_cap(usize::MAX)
    .burn_in(CHURN_BURN_IN)
}

/// The full zoo, in report order.
pub fn all() -> Vec<RunSpec> {
    vec![
        glider(),
        triangle_subdivide(),
        binary_edge(),
        tetrahedral(),
        growth_treadmill(),
        growth_treadmill_on_noise(),
        bounded_churn(),
    ]
}
