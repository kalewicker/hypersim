//! **C3.4 diagnosis probe — why the k = 5 row is VOID.** Fire counts only.
//!
//! `swap` satisfies every B1.1 criterion for a fifth rule: live, perpetual and
//! node-count-neutral. It still cannot join the k = 4 composition, and the
//! reason is compositional rather than a defect in the rule.
//!
//! **`swap` binds 4 nodes per firing and the composite substrate's crystal has
//! exactly 36.** It fires 9 times per step alone — 9 × 4 = 36, the whole
//! crystal — so under `MaxParallel` node-disjointness `swap` and the
//! crystal-resident rules are **mutually exclusive**, and `rule_id` order (the
//! conflict resolver's tie-break) decides which side wins outright.
//!
//! Read the two orderings against each other: with `swap` last it fires 0 and
//! the others run normally; with `swap` first it fires 4500 and `churn` and
//! `transfer` drop to 0 while `influx` falls 8242 → 315. That is starvation in
//! both directions, not a dead rule.
//!
//! **One arm here is NOT starvation and is labelled so:** in
//! `glider + transfer + swap`, `transfer` fires 0 despite having a *lower*
//! `rule_id` than `swap`. Its guard needs property 1 > 10 and without `influx`
//! the fill never leaves 5.0 — the rule has no candidates at all. A zero is not
//! self-explaining, which is exactly why the arms are run side by side.

use hg_alphabet::signature::rule_only_symbol;
use hg_alphabet::{btrack, measure, RunSpec};
use hg_rules::StepMode;

const GLIDER: &str = include_str!("../../../rules/glider.toml");
const TRANSFER_ONLY: &str = r#"
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

fn probe(label: &str, toml: String, names: &[&str]) {
    let spec = RunSpec::new(label, &toml, btrack::substrate(), StepMode::MaxParallel)
        .steps(500)
        .node_cap(usize::MAX);
    let a = measure(&spec).full;
    eprintln!("\n  === {label} ===  events={}", a.n_events());
    for (id, n) in names.iter().enumerate() {
        let c = a.rule_only.count_of(rule_only_symbol(id));
        // A bare zero does not say WHY. The arms below are what disambiguate
        // "starved by node-disjointness" from "no candidates at all".
        eprintln!(
            "     rule_id {id}  {n:<10} fired={c:<8} {}",
            if c == 0 { "<< 0 firings" } else { "" }
        );
    }
}

#[test]
#[ignore]
fn probe_swap_starvation() {
    probe(
        "k=5 as composed (swap is rule_id 4, last)",
        format!("{}\n{}", btrack::ruleset(4), btrack::SWAP),
        &["glider", "influx", "churn", "transfer", "swap"],
    );
    probe(
        "swap ALONE on the composite substrate",
        btrack::SWAP.to_string(),
        &["swap"],
    );
    probe(
        "glider + swap (no influx)",
        format!("{GLIDER}\n{}", btrack::SWAP),
        &["glider", "swap"],
    );
    probe(
        "glider + transfer + swap (no influx, no churn)",
        format!("{GLIDER}\n{TRANSFER_ONLY}\n{}", btrack::SWAP),
        &["glider", "transfer", "swap"],
    );
    probe(
        "swap FIRST, then the k=4 set (swap is rule_id 0)",
        format!("{}\n{}", btrack::SWAP, btrack::ruleset(4)),
        &["swap", "glider", "influx", "churn", "transfer"],
    );
}
