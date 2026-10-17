//! B-track rule compositions and run specs.
//!
//! Committed in `docs/btrack_preregistration.md` §1 before any B-track
//! cardinality existed. The composition was fixed by a liveness probe reporting
//! fire counts only (`tests/b1_liveness_probe.rs`), mirroring
//! `docs/policy_config.md` §2.5.
//!
//! **Nothing in this module can change a measured value.** It supplies rule
//! text, initial conditions and budgets; the signature, accounting, saturation
//! and growth code is the A-track's, unmodified, and `tests/b0_instrument_pin.rs`
//! proves it by reproducing the A-track's published numbers.

use crate::run::{Init, RunSpec};
use hg_rules::StepMode;

/// The primary horizon: equal accepted rewrites per row. `|A| ≤ n_events` is a
/// hard bound, so this is what holds the binding resource constant across rows
/// whose event rates differ ~17×.
pub const EVENT_BUDGET: u64 = 20_000;
/// The secondary horizon: equal steps per row.
pub const STEP_BUDGET: u64 = 2_000;
/// Step ceiling for the equal-event rows; the 1-rule row emits one event per
/// step, so it needs `EVENT_BUDGET` steps to fill its budget.
const STEP_CEILING: u64 = 40_000;

// ---------------------------------------------------------------- rule text
//
// PRIORITIES ARE FLATTENED TO 0 throughout. `MaxParallel` resolves conflicts by
// `(Reverse(priority), rule_id, sorted bound nodes)`, so a standing priority
// difference would carry a systematic preference among rules and confound "which
// rule fired" with "which rule the resolver preferred". This is the same edit,
// for the same stated reason, that `rules/wn_three.toml` already makes. It is
// the only edit made to any rule.

/// `rules/glider.toml`, priority already 0.
const GLIDER: &str = include_str!("../../../rules/glider.toml");

/// `influx` from `rules/bounded_churn.toml` (priority 0 as shipped).
const INFLUX: &str = r#"
[[rules]]
name = "influx"
priority = 0
[rules.lhs]
nodes = ["s"]
edges = []
[rules.lhs.node_labels]
s = 0
[[rules.lhs.node_property_guards]]
var = "s"
property = 1
op = "lt"
value = 20.0
[rules.rhs]
nodes = ["s"]
edges = []
[[rules.rhs.property_effects]]
var = "s"
key = 1
op = "add"
value = 1.0
"#;

/// `churn` from `rules/bounded_churn.toml`, priority flattened 2 → 0.
const CHURN: &str = r#"
[[rules]]
name = "churn"
priority = 0
[rules.lhs]
nodes = ["a", "old", "b"]
edges = [["a", "old"], ["old", "b"]]
[rules.lhs.node_labels]
a = 0
old = 0
b = 0
[[rules.lhs.node_property_guards]]
var = "old"
property = 1
op = "gte"
value = 20.0
[[rules.lhs.node_property_guards]]
var = "old"
property = 22
op = "eq"
value = false
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "old", "b"]
[rules.rhs]
nodes = ["a", "fresh", "b"]
edges = [["a", "fresh"], ["fresh", "b"]]
[rules.rhs.node_labels]
fresh = 0
[[rules.rhs.property_effects]]
var = "fresh"
key = 1
op = "set"
value = 0.0
[[rules.rhs.property_effects]]
var = "fresh"
key = 22
op = "set"
value = false
"#;

/// `transfer` from `rules/conservation_soak.toml`: a conservative 10-unit move
/// across a 2-edge, node-count neutral. Its companion `seed_cycle` is a
/// fire-once creation rule and is deliberately NOT included — a rule that fires
/// exactly once contributes exactly one symbol and would inflate the rule count
/// without contributing alphabet.
const TRANSFER: &str = r#"
[[rules]]
name = "transfer"
priority = 0
[rules.lhs]
nodes = ["src", "dst"]
edges = [["src", "dst"]]
[[rules.lhs.node_property_guards]]
var = "src"
property = 1
op = "gt"
value = 10.0
[rules.rhs]
nodes = ["src", "dst"]
edges = [["src", "dst"]]
[[rules.rhs.property_effects]]
var = "src"
key = 1
op = "subtract"
value = 10.0
[[rules.rhs.property_effects]]
var = "dst"
key = 1
op = "add"
value = 10.0
"#;

/// The nested series. Set `k` is a subset of set `k+1` by construction, so the
/// series is a refinement rather than four unrelated files.
pub fn ruleset(count: usize) -> String {
    let mut parts: Vec<&str> = vec![GLIDER];
    if count >= 2 {
        parts.push(INFLUX);
    }
    if count >= 3 {
        parts.push(CHURN);
    }
    if count >= 4 {
        parts.push(TRANSFER);
    }
    assert!((1..=4).contains(&count), "the committed series is 1..=4");
    parts.join("\n")
}

/// Rule names in the order the compiled rule ids will take, so a report can map
/// a `rule_only` symbol back to a name.
pub fn rule_names(count: usize) -> Vec<&'static str> {
    let all = ["glider", "influx", "churn", "transfer"];
    all[..count].to_vec()
}

pub const COUNTS: [usize; 4] = [1, 2, 3, 4];

/// The substrate, fixed across the whole series (pre-registration §1.1).
pub fn substrate() -> Init {
    Init::CompositeChurnRing {
        w: 6,
        h: 6,
        arity: 3,
        fill: 5.0,
        ring: 24,
    }
}

/// PRIMARY: equal event budget.
pub fn series_by_events(count: usize) -> RunSpec {
    RunSpec::new(
        &format!("k={count} @{EVENT_BUDGET}ev"),
        &ruleset(count),
        substrate(),
        StepMode::MaxParallel,
    )
    .steps(STEP_CEILING)
    .node_cap(usize::MAX)
    .event_cap(EVENT_BUDGET)
}

/// SECONDARY: equal step budget.
pub fn series_by_steps(count: usize) -> RunSpec {
    RunSpec::new(
        &format!("k={count} @{STEP_BUDGET}st"),
        &ruleset(count),
        substrate(),
        StepMode::MaxParallel,
    )
    .steps(STEP_BUDGET)
    .node_cap(usize::MAX)
}

// ------------------------------------------------------------------ B2 ladder

/// The B2 horizon ladder: the A-track's `growth_treadmill` spec exactly, varying
/// only the step cap. 400 is the A-track rung.
pub const B2_LADDER: [u64; 3] = [400, 1600, 6400];

pub fn growth_treadmill_at(steps: u64) -> RunSpec {
    crate::zoo::growth_treadmill().steps(steps).node_cap(usize::MAX)
}

// ------------------------------------------------------------------- C-track
//
// The two gap-rule mechanisms and their substrates, committed in
// `docs/ctrack_preregistration.md` §1 before either was measured.

/// C-track horizons: the A-track's flat-rule horizon and 4× it. **The C3.2 gap
/// verdict is read from exactly these two rungs**, as pre-registered.
pub const C_LADDER: [u64; 2] = [2_000, 8_000];

/// A third rung, added for **power, not for a verdict**: C3.3 reuses B2.4's
/// scale-invariance instrument, whose committed minimum span is 16×, and the
/// pre-registered 2000/8000 pair spans only 4× — so C3.3 could not return
/// anything but `INSUFFICIENT LADDER` on the pair. That is an inconsistency in
/// the C-track pre-registration, reported as one. This rung makes the span 16×
/// so the criterion can fire; it changes no threshold and the C3.2 verdict is
/// still read from the pre-registered pair alone.
pub const C_LADDER_LONG: u64 = 32_000;

pub const SWAP: &str = include_str!("../../../rules/swap.toml");
pub const CYCLIC3: &str = include_str!("../../../rules/cyclic3.toml");

/// `swap` on the A-track's `triangle_subdivide` substrate, bare.
pub fn swap_at(steps: u64) -> RunSpec {
    RunSpec::new(
        &format!("swap@{steps}st"),
        SWAP,
        Init::PureCrystal { w: 6, h: 6, arity: 3, seed_label1: false },
        StepMode::MaxParallel,
    )
    .steps(steps)
    .node_cap(usize::MAX)
}

/// `cyclic3` on the **pre-registered** substrate: 6×6, OPEN boundary.
/// Retained exactly as committed, because it is the configuration that failed
/// the C2 gate and that result stands.
pub fn cyclic3_at(steps: u64) -> RunSpec {
    RunSpec::new(
        &format!("cyclic3@{steps}st"),
        CYCLIC3,
        Init::LabelledCrystal { w: 6, h: 6, arity: 3, states: 3, seed: 42, periodic: false },
        StepMode::MaxParallel,
    )
    .steps(steps)
    .node_cap(usize::MAX)
}

/// `cyclic3` on a **torus with room for a spiral core** — a POST-HOC deviation
/// from the pre-registered substrate, recorded as one in
/// `docs/ctrack_findings.md` C2.
///
/// The deviation is to the SUBSTRATE, not to the rule: `cyclic3` has no
/// parameters and none were touched. Two mechanism defects in the original
/// choice are being corrected, both identifiable without reference to any
/// alphabet number:
///
/// 1. **An open boundary is a sink.** Waves propagate off the edge and are
///    destroyed, so the lattice drains and freezes. The cyclic-CA literature
///    the rule comes from runs on a torus.
/// 2. **6×6 has no room for a spiral core.** Self-organised spirals need a
///    closed loop carrying the full 3-state cycle around a defect; a lattice
///    six sites across cannot host one.
///
/// 24×24 periodic is chosen as the smallest power-of-two-ish lattice that
/// comfortably exceeds the cycle length in every direction. It is run ONCE.
pub fn cyclic3_torus_at(steps: u64) -> RunSpec {
    RunSpec::new(
        &format!("cyclic3-torus@{steps}st"),
        CYCLIC3,
        Init::LabelledCrystal { w: 24, h: 24, arity: 3, states: 3, seed: 42, periodic: true },
        StepMode::MaxParallel,
    )
    .steps(steps)
    .node_cap(usize::MAX)
}

/// C3.4 — the B1 series extended to k = 5 by appending a C-track rule, on the
/// same substrate with the same flattened priorities and the same horizons.
/// `swap` needs only 2-edges, which the `CompositeChurnRing` crystal has.
pub fn series_k5_by_events() -> RunSpec {
    RunSpec::new(
        &format!("k=5 @{EVENT_BUDGET}ev"),
        &format!("{}\n{}", ruleset(4), SWAP),
        substrate(),
        StepMode::MaxParallel,
    )
    .steps(STEP_CEILING)
    .node_cap(usize::MAX)
    .event_cap(EVENT_BUDGET)
}

pub fn series_k5_by_steps() -> RunSpec {
    RunSpec::new(
        &format!("k=5 @{STEP_BUDGET}st"),
        &format!("{}\n{}", ruleset(4), SWAP),
        substrate(),
        StepMode::MaxParallel,
    )
    .steps(STEP_BUDGET)
    .node_cap(usize::MAX)
}
