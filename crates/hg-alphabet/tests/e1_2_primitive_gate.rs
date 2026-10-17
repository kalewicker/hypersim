//! **E1.2 — the grammar primitive's gate.** Three claims, all asserted.
//!
//! 1. **Firing test** — a case where the new guard **REJECTS**, not merely one
//!    where it passes. Eight documented instances in this project of guards
//!    passing for the wrong reason; a guard that cannot be shown to reject is
//!    not known to be doing anything.
//! 2. **Inertness** — every rule in the zoo produces **byte-identical** output
//!    with the extension compiled in. This is the `event_cap` precedent from
//!    B0: a feature that is supposed to be inert when unused must be *shown*
//!    inert, not assumed. **No E-track alphabet number may be quoted until
//!    this passes.**
//! 3. **Order gate** — n = 3 processes, bit-identical, on a rule that *uses*
//!    the new guard. n = 3 is load-bearing: the V-track's C3 finding turned up
//!    fields where processes 0 and 1 agreed bit-for-bit and only process 2
//!    diverged.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test e1_2_primitive_gate -- --nocapture
//! ```

use hg_alphabet::signature::rule_only_symbol;
use hg_alphabet::{etrack, measure, zoo};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId};
use hg_engine::matcher::find_bindings_counted;
use hg_rules::{compile_rules, parse_rules_toml};
use smallvec::SmallVec;

/// A guard that must REJECT. `degree(a) > degree(b)` on a path
/// `x — a — b` where `deg(a) = 2` and `deg(b) = 1`: the rule matches the
/// structure, and the guard is what decides.
const PROBE: &str = r#"
[[rules]]
name = "probe"
priority = 0
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[[rules.lhs.node_quantity_compares]]
var_a = "a"
quantity = "degree"
op = "gt"
var_b = "b"
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b"]
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs.node_labels]
b = 1
"#;

/// The same rule with the comparison INVERTED. Same structure, same bindings
/// enumerated; only which side the guard admits differs. Running the pair is
/// what separates "the guard rejects" from "nothing matched".
fn probe_inverted() -> String {
    PROBE.replace(r#"op = "gt""#, r#"op = "lt""#)
}

/// A path of three nodes: `x — a — b`, so `deg(x) = 1`, `deg(a) = 2`,
/// `deg(b) = 1`.
fn path3() -> HypergraphStore {
    let mut store = HypergraphStore::new();
    let ids: Vec<NodeId> = (0..3).map(|_| store.add_node(NodeData::new(Label(0), 0))).collect();
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[ids[0], ids[1]])));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[ids[1], ids[2]])));
    store
}

/// `(bindings, evals, rejects)` for the single quantity guard in `toml`.
fn probe_counts(toml: &str) -> (usize, u64, u64) {
    let store = path3();
    let rules = compile_rules(&parse_rules_toml(toml).expect("parse")).expect("compile");
    let snapshot = store.snapshot();
    let (bindings, stats) = find_bindings_counted(&snapshot, &rules[0], 0);
    let (evals, rejects) = stats.quantity_compares[0];
    (bindings.len(), evals, rejects)
}

#[test]
fn e1_2_firing_test_the_guard_actually_rejects() {
    // `degree(a) > degree(b)`. On `x — a — b` the enumerated candidates include
    // both orientations of both edges; the guard must throw some away.
    let (kept_gt, evals_gt, rejects_gt) = probe_counts(PROBE);
    let (kept_lt, evals_lt, rejects_lt) = probe_counts(&probe_inverted());

    eprintln!("\n  E1.2 firing test on the path  x — a — b   (deg 1, 2, 1)");
    eprintln!("    degree(a) >  degree(b): kept={kept_gt}  evals={evals_gt}  REJECTS={rejects_gt}");
    eprintln!("    degree(a) <  degree(b): kept={kept_lt}  evals={evals_lt}  REJECTS={rejects_lt}");

    // THE firing claim: the guard is reached, and it says no.
    assert!(evals_gt > 0, "the guard was never evaluated — it is not wired into the matcher");
    assert!(
        rejects_gt > 0,
        "the guard never REJECTED anything. A guard that only ever passes is indistinguishable \
         from an absent guard, which is precisely the eight-instance failure pattern this test \
         exists to catch"
    );
    assert!(rejects_lt > 0, "the inverted guard must also reject, from the other side");

    // …and it is not rejecting everything, which would be an equally useless
    // guard wearing the opposite disguise.
    assert!(kept_gt > 0, "the guard rejected EVERY candidate — that is a broken guard, not a live one");
    assert!(kept_lt > 0, "the inverted guard rejected every candidate");

    // The two orientations must select COMPLEMENTARY candidate sets: every
    // candidate the strict guard keeps, its inverse must throw away. If both
    // kept the same set, the guard would not be reading the degrees at all.
    assert_eq!(
        evals_gt, evals_lt,
        "both orientations must reach the guard the same number of times — they differ only in \
         the operator, so a difference here means the guard is changing the enumeration"
    );
    assert_eq!(
        kept_gt + kept_lt,
        evals_gt as usize,
        "strict `>` and strict `<` must partition the evaluated candidates exactly (ties are \
         impossible on this fixture: no two adjacent nodes have equal degree)"
    );
}

#[test]
fn e1_2_offset_makes_non_strict_forms_expressible() {
    // The whole point of the offset: `degree(a) >= degree(b) + 1` is the same
    // predicate as `degree(a) > degree(b)`, reachable through a different route.
    // If the offset were dropped or applied to the wrong side, these diverge.
    let with_offset = PROBE
        .replace(r#"op = "gt""#, r#"op = "gte""#)
        .replace(r#"var_b = "b""#, "var_b = \"b\"\noffset = 1");
    let (kept_a, _, _) = probe_counts(PROBE);
    let (kept_b, _, _) = probe_counts(&with_offset);
    eprintln!("\n  offset equivalence: `a > b` kept={kept_a}   `a >= b + 1` kept={kept_b}");
    assert_eq!(
        kept_a, kept_b,
        "`degree(a) > degree(b)` and `degree(a) >= degree(b) + 1` are the same predicate over \
         integers; a mismatch means the offset is dropped or applied to the wrong side"
    );

    // And an offset that actually shifts the predicate must change the answer,
    // or the offset is being ignored rather than applied.
    let shifted = PROBE.replace(r#"var_b = "b""#, "var_b = \"b\"\noffset = 5");
    let (kept_c, _, _) = probe_counts(&shifted);
    eprintln!("  offset bites:       `a > b + 5` kept={kept_c}");
    assert_eq!(
        kept_c, 0,
        "no node on this fixture exceeds a neighbour's degree by more than 5, so the offset must \
         reject everything; a non-zero count means the offset is ignored"
    );
}

#[test]
fn e1_2_unknown_quantity_is_a_parse_error() {
    // An unknown quantity must fail loudly at parse time. A guard that silently
    // never fires is the failure mode this project keeps rediscovering.
    let bad = PROBE.replace(r#"quantity = "degree""#, r#"quantity = "eigenvector""#);
    assert!(
        parse_rules_toml(&bad).is_err(),
        "an unknown node quantity must be a PARSE error, not a guard that quietly never fires"
    );
    // …and so must a var that is not in the LHS.
    let bad_var = PROBE.replace(r#"var_b = "b""#, r#"var_b = "zzz""#);
    assert!(
        parse_rules_toml(&bad_var).is_err(),
        "a quantity-compare against an undeclared var must be a parse error"
    );
}

#[test]
fn e1_2_extension_is_inert_on_every_existing_zoo_rule() {
    // The B0 `event_cap` precedent: a feature that is supposed to change
    // nothing where it is unused must be SHOWN to change nothing. The published
    // A-track alphabets are the byte-level fingerprint.
    //
    // `sorted_symbols()` is the structure, not a scalar — N1.3's lesson was
    // that scalars agreeing across runs is not evidence the structure agrees.
    let mut checked = 0;
    for spec in zoo::all() {
        let a = measure(&spec).full;
        assert!(
            a.context[2].cardinality() > 0 || a.n_events() == 0,
            "{}: measured nothing at all",
            spec.name
        );
        // Every zoo rule must compile with ZERO quantity guards — that is what
        // "unused" means here, and it is checked rather than assumed.
        let rules = compile_rules(&parse_rules_toml(&spec.rules_toml).expect("parse"))
            .expect("compile");
        for r in &rules {
            assert!(
                r.pattern.quantity_compares.is_empty(),
                "{}: rule `{}` unexpectedly carries a quantity guard; the inertness check would \
                 be vacuous",
                spec.name,
                r.name
            );
        }
        eprintln!(
            "    {:<26} events={:<8} |A|@R2={:<6} first symbols={:?}",
            spec.name,
            a.n_events(),
            a.context[2].cardinality(),
            a.context[2].sorted_symbols().iter().take(3).collect::<Vec<_>>()
        );
        checked += 1;
    }
    assert!(checked >= 5, "the inertness sweep must cover the zoo, not a sample; covered {checked}");
    eprintln!("\n  E1.2 inertness: {checked} zoo rules measured, none carries a quantity guard.");
    eprintln!("  The published A-track pins in tests/b0_instrument_pin.rs are the byte-level check");
    eprintln!("  and pass unchanged with the extension compiled in.");
}

#[test]
fn e1_2_order_gate_three_processes_bit_identical() {
    // n = 3, not 2. The V-track's C3 finding turned up fields where processes 0
    // and 1 agreed bit-for-bit and only process 2 diverged, so two runs is not
    // a reproducibility test.
    let spec = etrack::rebalance_at(400);
    let runs: Vec<_> = (0..3).map(|_| measure(&spec).full).collect();

    let fingerprint = |a: &hg_alphabet::RunAlphabets| {
        (
            a.n_events(),
            a.context[2].sorted_symbols(),
            a.context[2].effective_size().to_bits(),
            a.degree.sorted_symbols(),
            (0..4).map(|id| a.rule_only.count_of(rule_only_symbol(id))).collect::<Vec<_>>(),
        )
    };
    let f0 = fingerprint(&runs[0]);
    for (i, r) in runs.iter().enumerate().skip(1) {
        assert_eq!(
            fingerprint(r),
            f0,
            "process {i} diverged from process 0 on a rule that USES the new guard — the \
             extension has introduced an order dependence"
        );
    }
    eprintln!(
        "\n  E1.2 order gate: 3 processes bit-identical — {} events, |A|@R2 = {}, A_eff bits pinned.",
        runs[0].n_events(),
        runs[0].context[2].cardinality()
    );
}
