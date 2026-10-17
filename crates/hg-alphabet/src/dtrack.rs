//! D-track run specs: the scaled substrate and the `rule_id`-permuted series.
//!
//! Committed in `docs/dtrack_preregistration.md` §D1.2 before any D-track
//! alphabet number existed. **Nothing here can change a measured value** — it
//! supplies initial conditions, rule text ordering and budgets, and the
//! signature, accounting, saturation and growth code is the A-track's,
//! unmodified (`tests/b0_instrument_pin.rs`).

use crate::btrack;
use crate::run::{Init, RunSpec};
use hg_rules::StepMode;

/// The D1.2a scaled substrate multiplier. 4 gives n = 960, a 16× scale-up of
/// B1's n = 60, with the crystal : ring node ratio held at 3 : 2 so the
/// substrate is a scaled version of B1's rather than a differently shaped one.
///
/// It is run **despite failing D1.2's own `demand ≤ 10% of n` requirement**:
/// `influx` has LHS arity 1 and matches universally, so its demand is extensive
/// and its fraction is scale-invariant (measured 98.3% → 99.9% across this very
/// span). No row measured here may be described as low-contention.
pub const SCALE: usize = 4;

pub fn scaled_substrate() -> Init {
    Init::CompositeChurnRing {
        w: 6 * SCALE,
        h: 6 * SCALE,
        arity: 3,
        fill: 5.0,
        ring: 24 * SCALE * SCALE,
    }
}

/// Split B1's k = 4 file back into its four rule bodies, in `rule_id` order.
/// `parts[0]` is the text preceding the first header (`rules/glider.toml` opens
/// with a comment block), so rule `i` is `parts[i + 1]`.
fn bodies() -> Vec<String> {
    let all = btrack::ruleset(4);
    let parts: Vec<&str> = all.split("[[rules]]").collect();
    assert_eq!(parts.len(), 5, "the k=4 file must contain exactly four rule headers");
    parts[1..].iter().map(|p| format!("[[rules]]{p}")).collect()
}

/// **D1.2b — the substituted control.** The same rule *sets* B1 used, with the
/// `rule_id` assignment REVERSED, so the conflict resolver's tie-break
/// `(Reverse(priority), rule_id, sorted bound nodes)` picks a different winner
/// while rules, substrate, instrument and horizons are held exactly fixed.
///
/// Nesting is by **suffix**, so set `k` is still a subset of set `k+1` and every
/// row stays comparable to B1's:
///
/// ```text
/// k=1 {glider}   k=2 {influx, glider}   k=3 {churn, influx, glider}
/// k=4 {transfer, churn, influx, glider}
/// ```
pub fn reversed_ruleset(count: usize) -> String {
    assert!((1..=4).contains(&count), "the committed series is 1..=4");
    let b = bodies();
    // `bodies()` is in forward order [glider, influx, churn, transfer]; the
    // suffix of length `count` reversed is the committed order.
    let mut parts: Vec<String> = b[..count].to_vec();
    parts.reverse();
    parts.join("\n")
}

/// Rule names in the order the compiled rule ids take, for the reversed series.
pub fn reversed_rule_names(count: usize) -> Vec<&'static str> {
    let mut v = btrack::rule_names(count);
    v.reverse();
    v
}

// ------------------------------------------------------------------ D1.3 arms

/// Arm A — B1's baseline, reproduced verbatim. The anchor every other arm is
/// read against; if this does not reproduce B1's published numbers the
/// comparison is void.
pub fn baseline_by_events(k: usize) -> RunSpec {
    btrack::series_by_events(k)
}
pub fn baseline_by_steps(k: usize) -> RunSpec {
    btrack::series_by_steps(k)
}

/// Arm B — the scaled substrate, everything else identical to arm A.
pub fn scaled_by_events(k: usize) -> RunSpec {
    RunSpec::new(
        &format!("scaled k={k} @{}ev", btrack::EVENT_BUDGET),
        &btrack::ruleset(k),
        scaled_substrate(),
        StepMode::MaxParallel,
    )
    .steps(40_000)
    .node_cap(usize::MAX)
    .event_cap(btrack::EVENT_BUDGET)
}
pub fn scaled_by_steps(k: usize) -> RunSpec {
    RunSpec::new(
        &format!("scaled k={k} @{}st", btrack::STEP_BUDGET),
        &btrack::ruleset(k),
        scaled_substrate(),
        StepMode::MaxParallel,
    )
    .steps(btrack::STEP_BUDGET)
    .node_cap(usize::MAX)
}

/// Arm C — `rule_id` reversed, on B1's own substrate. The only difference from
/// arm A is which rule wins a conflict.
pub fn reversed_by_events(k: usize) -> RunSpec {
    RunSpec::new(
        &format!("reversed k={k} @{}ev", btrack::EVENT_BUDGET),
        &reversed_ruleset(k),
        btrack::substrate(),
        StepMode::MaxParallel,
    )
    .steps(40_000)
    .node_cap(usize::MAX)
    .event_cap(btrack::EVENT_BUDGET)
}
pub fn reversed_by_steps(k: usize) -> RunSpec {
    RunSpec::new(
        &format!("reversed k={k} @{}st", btrack::STEP_BUDGET),
        &reversed_ruleset(k),
        btrack::substrate(),
        StepMode::MaxParallel,
    )
    .steps(btrack::STEP_BUDGET)
    .node_cap(usize::MAX)
}

// --------------------------------------------------------------------- D1.5

/// D1.5's horizon, **reduced from B1's and recorded as a reduction**.
///
/// `swap`'s LHS is two INDEPENDENT 2-edges, so candidate enumeration is
/// quadratic in edge count: ~85² ≈ 7 000 pairs/step on B1's crystal but
/// ~1 650² ≈ 2.7 M pairs/step on the scaled one, a ~380× cost increase per step.
/// B1's 2 000-step and 40 000-step ceilings are not reachable there in any
/// useful time (a first attempt was abandoned after 37 minutes).
///
/// This is legitimate because **D1.5 asks a fire-count question** — does
/// `A_rule_only` reach 5, i.e. does every rule fire at least once — and that is
/// exactly what C3.4's own probe answered in 500 steps. No alphabet verdict is
/// read at this horizon, and none is quoted against B1's rows.
pub const D1_5_STEPS: u64 = 50;

/// **D1.5's substrate is `SCALE = 2` (n = 240), not D1.2a's `SCALE = 4`** — a
/// recorded reduction, forced rather than chosen.
///
/// At `SCALE = 4` the crystal has ~1 650 2-edges and `swap`'s two-independent-
/// edge LHS enumerates ~2.7 M candidate pairs per step. `swap` is **not runnable
/// there at all**: three attempts were abandoned, the last after 10 minutes of
/// CPU and ~10 GB resident without completing 50 steps.
///
/// The reduction does not weaken what D1.5 tests. D1.2's probe established that
/// the demand fraction is scale-INVARIANT, so n = 240 tests the same claim as
/// n = 960 would; and D1.5 is a fire-count question, settled in single-digit
/// steps. That `swap` is computationally unrunnable at n = 960 is itself a
/// second, independent reason the k = 5 row cannot be rescued by scaling.
pub const D1_5_SCALE: usize = 2;

pub fn d1_5_substrate() -> Init {
    Init::CompositeChurnRing {
        w: 6 * D1_5_SCALE,
        h: 6 * D1_5_SCALE,
        arity: 3,
        fill: 5.0,
        ring: 24 * D1_5_SCALE * D1_5_SCALE,
    }
}

/// The k = 5 row on the SCALED substrate. `swap` is appended last, exactly as
/// C3.4 composed it, so the only thing that changed is the substrate size.
pub fn scaled_k5_by_events() -> RunSpec {
    RunSpec::new(
        &format!("scaled k=5 @{}ev", btrack::EVENT_BUDGET),
        &format!("{}\n{}", btrack::ruleset(4), btrack::SWAP),
        scaled_substrate(),
        StepMode::MaxParallel,
    )
    .steps(D1_5_STEPS)
    .node_cap(usize::MAX)
    .event_cap(btrack::EVENT_BUDGET)
}
// ----------------------------------------------------------------- D2 arms
//
// The conservation control's matched treatment. Both arms differ from `swap` in
// EXACTLY ONE property, and between them they isolate the two halves of the
// C-track diagnosis ("WL classes are pure degree structure BECAUSE the
// substrate is unlabelled"): arm E varies the rule, arm F varies the substrate.

/// Arm E — `rewire`, node-count-neutral and **degree-changing**.
pub const REWIRE: &str = include_str!("../../../rules/rewire.toml");

/// Arm E′ — the ONE recorded post-hoc deviation, corrected for the absorbing
/// state that killed arm E at the D2.3 gate. Reported separately, never as the
/// pre-registered arm. See the header of `rules/rewire_bounded.toml`.
pub const REWIRE_BOUNDED: &str = include_str!("../../../rules/rewire_bounded.toml");

/// The C-track's substrate for `swap`, reused unchanged so arm E differs from
/// the `swap` control in the rule and nothing else.
pub fn bare_crystal() -> Init {
    Init::PureCrystal { w: 6, h: 6, arity: 3, seed_label1: false }
}

/// The substrate `cyclic3` was pre-registered on, reused rather than invented so
/// that no new free choice enters arm F.
pub fn labelled_crystal() -> Init {
    Init::LabelledCrystal { w: 6, h: 6, arity: 3, states: 3, seed: 42, periodic: false }
}

/// Arm E at a given horizon: `rewire` on the bare crystal.
pub fn rewire_at(steps: u64) -> RunSpec {
    RunSpec::new(
        &format!("rewire@{steps}st"),
        REWIRE,
        bare_crystal(),
        StepMode::MaxParallel,
    )
    .steps(steps)
    .node_cap(usize::MAX)
}

/// Arm E′ at a given horizon.
pub fn rewire_bounded_at(steps: u64) -> RunSpec {
    RunSpec::new(
        &format!("rewire_bounded@{steps}st"),
        REWIRE_BOUNDED,
        bare_crystal(),
        StepMode::MaxParallel,
    )
    .steps(steps)
    .node_cap(usize::MAX)
}

/// Arm F at a given horizon: `swap`, **untouched**, on a labelled substrate.
pub fn swap_labelled_at(steps: u64) -> RunSpec {
    RunSpec::new(
        &format!("swap-labelled@{steps}st"),
        btrack::SWAP,
        labelled_crystal(),
        StepMode::MaxParallel,
    )
    .steps(steps)
    .node_cap(usize::MAX)
}

pub fn scaled_k5_by_steps() -> RunSpec {
    RunSpec::new(
        &format!("scaled k=5 @{D1_5_STEPS}st"),
        &format!("{}\n{}", btrack::ruleset(4), btrack::SWAP),
        scaled_substrate(),
        StepMode::MaxParallel,
    )
    .steps(D1_5_STEPS)
    .node_cap(usize::MAX)
}

/// The k = 5 composition with `swap` promoted to `rule_id` 0. C3.4 showed
/// starvation runs BOTH ways at n = 60, which is what ruled out "`swap` is
/// simply weak"; the same check has to be available at n = 960 or the D1.5
/// zero is again a zero that does not explain itself.
pub fn scaled_k5_swap_first() -> RunSpec {
    RunSpec::new(
        &format!("scaled k=5 swap-FIRST @{D1_5_STEPS}st"),
        &format!("{}\n{}", btrack::SWAP, btrack::ruleset(4)),
        scaled_substrate(),
        StepMode::MaxParallel,
    )
    .steps(D1_5_STEPS)
    .node_cap(usize::MAX)
}
