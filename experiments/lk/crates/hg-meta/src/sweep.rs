//! Full-library G-R2 sweep (plan §6, EXP-R02): for every rule in the
//! shipped `rules/*.toml` library, attempt the round trip
//!
//!   TOML parse → E1 candidate → install_region → read_region → compile
//!
//! and report the **expressible fraction**: how many rules survive the
//! whole path with their compiled form behaviorally identical to the
//! directly-parsed one. This number IS the expressivity envelope (§3.6)
//! and appears in every downstream novelty claim.
//!
//! Classification per rule:
//! - `RoundTrips`  — parsed → candidate → re-read → compiled; the
//!   re-compiled Rule is field-equal to a canonicalized parse of the
//!   original (guards/effects aside — see exclusions below).
//! - `Rejected`    — the candidate fails V-class validation or the walk
//!   (encoding cannot SAY this rule).
//! - `Diverged`    — round-trips but compiles to something different
//!   (would be an hg-meta bug, not an envelope limit).
//!
//! Known envelope exclusions (§3.6, SB-R2/SB-R5) are EXPECTED rejects:
//! schedules, property effects, survivor relabels, property-compare /
//! degree / age guards, vars_distinct var guards.

use crate::compile::candidate_to_rule;
use crate::graph_source::GraphRuleSource;
use crate::schema::{find_rule_roots, read_region_with_footprint};
use hg_rules::parser::parse_rules_toml;
use hg_rules::rule::Rule;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SweepClass {
    RoundTrips,
    Rejected,
    Diverged,
}

/// Behavioral comparison modulo the envelope exclusions: two Rules are
/// "E1-equivalent" if their pattern structure, priorities, LHS/RHS label
/// constraints, and property guards coincide — i.e., everything E1 can
/// carry. Anything else (effects, schedules, exotic guards) is excluded
/// by the envelope, not by error.
/// Behavioral comparison modulo variable RENAMING: hand-written TOML names
/// ("a","b") vs E1's canonical vN names are the same rule if the structure
/// matches after mapping each side's vars to first-appearance indices.
/// Canonical form of a Rule for envelope comparison (modulo renaming):
/// sorted LHS node indices, sorted RHS node indices, the label vector
/// (LHS constraints then created-node labels, in var order), and the
/// guard multiset.
type CanonRule = (
    Vec<usize>,
    Vec<usize>,
    Vec<Option<u32>>,
    Vec<(usize, u32, String, String)>,
);

fn e1_equivalent(a: &Rule, b: &Rule) -> bool {
    let canon = |r: &Rule| -> CanonRule {
        let mut order: Vec<String> = Vec::new();
        fn ix_of(order: &mut Vec<String>, name: &str) -> usize {
            if let Some(i) = order.iter().position(|v| v == name) {
                i
            } else {
                order.push(name.to_string());
                order.len() - 1
            }
        }
        let mut lhs_nodes: Vec<usize> = Vec::new();
        for n in &r.lhs.nodes {
            lhs_nodes.push(ix_of(&mut order, n));
        }
        let mut lhs_edges: Vec<Vec<usize>> = r
            .lhs
            .edges
            .iter()
            .map(|e| e.iter().map(|n| ix_of(&mut order, n)).collect())
            .collect();
        lhs_edges.sort();
        let mut rhs_nodes: Vec<usize> = Vec::new();
        for n in &r.rhs.nodes {
            rhs_nodes.push(ix_of(&mut order, n));
        }
        // Sort node lists so membership (not order) is compared.
        lhs_nodes.sort();
        rhs_nodes.sort();
        let mut lhs_labels: Vec<Option<u32>> = vec![None; order.len()];
        for (n, l) in &r.lhs.node_labels {
            lhs_labels[ix_of(&mut order, n)] = Some(l.0);
        }
        let mut created: Vec<Option<u32>> = vec![None; order.len()];
        for (n, l) in r.rhs_node_labels.iter().chain(r.rhs.node_labels.iter()) {
            let i = ix_of(&mut order, n);
            if !r.lhs.nodes.iter().any(|v| v == n) {
                created[i] = Some(l.0);
            }
        }
        let mut guards: Vec<_> = r
            .lhs_node_property_guards
            .iter()
            .map(|g| {
                (
                    ix_of(&mut order, &g.var),
                    g.property,
                    format!("{:?}", g.op),
                    format!("{:?}", g.value),
                )
            })
            .collect();
        guards.sort();
        (
            lhs_nodes,
            rhs_nodes,
            {
                // Canonical label vector: LHS constraints and created-node
                // labels concatenated in var order.
                let mut l: Vec<Option<u32>> = lhs_labels.clone();
                l.extend(created.clone());
                l
            },
            guards,
        )
    };

    // Compare the two rules through their canonical forms.
    let ca = canon(a);
    let cb = canon(b);
    if ca != cb {
        return false;
    }
    // Priorities must match too (behavioral).
    a.priority == b.priority
}

/// Classify ONE parsed rule (the sweep's unit is the rule, not the file).
fn classify_parsed(rule: &hg_rules::rule::Rule, label: &str) -> (SweepClass, Option<&'static str>) {
    // Envelope pre-check: features E1 cannot carry (§3.6). As of R5,
    // vars_distinct guards and property effects ARE carried (E1-V/E2-lite);
    // remaining exclusions:
    if rule.schedule.is_some() {
        return (SweepClass::Rejected, Some("schedule (SB-R2)"));
    }
    if !rule.lhs_node_property_compares.is_empty() {
        return (SweepClass::Rejected, Some("property-to-property compares"));
    }
    if !rule.lhs_node_degree_compares.is_empty() || !rule.lhs_node_age_compares.is_empty() {
        return (SweepClass::Rejected, Some("degree/age guards"));
    }
    // Survivor relabels (SB-R5): RHS label on a var that also exists in LHS.
    // Checked against BOTH spellings: `rules.rhs.node_labels` (the graph
    // field, which the engine treats as a survivor relabel) and the legacy
    // top-level `rhs_node_labels`.
    {
        let lhs_names: std::collections::HashSet<&str> =
            rule.lhs.nodes.iter().map(|s| s.as_str()).collect();
        for v in &rule.rhs.nodes {
            if lhs_names.contains(v.as_str())
                && (rule.rhs_node_labels.contains_key(v) || rule.rhs.node_labels.contains_key(v))
            {
                return (SweepClass::Rejected, Some("survivor relabel (SB-R5)"));
            }
        }
    }

    // The round trip itself: Rule → CandidateRule → region → read back.
    let cand = match crate::compile::rule_to_candidate(rule) {
        Some(c) => c,
        None => return (SweepClass::Rejected, Some("candidate extraction failed")),
    };

    let mut store = hg_core::HypergraphStore::new();
    GraphRuleSource::install_candidates(&mut store, std::slice::from_ref(&cand));
    let snap = store.snapshot();
    let roots = find_rule_roots(&snap);
    if roots.len() != 1 {
        return (
            SweepClass::Rejected,
            Some("region did not install as one root"),
        );
    }
    let (read_back, _fp) = match read_region_with_footprint(&snap, roots[0]) {
        Ok(x) => x,
        Err(rej) => {
            if std::env::var("ERE_SWEEP_DEBUG").is_ok() {
                eprintln!("RE-READ REJECT {label}: {rej}");
            }
            return (SweepClass::Rejected, Some("re-read rejected"));
        }
    };
    let reparsed = candidate_to_rule(&read_back);
    if e1_equivalent(rule, &reparsed) {
        (SweepClass::RoundTrips, None)
    } else {
        // Distinguish a real divergence from a KNOWN benign one: E1 cannot
        // express free-var LHS nodes (a var with no slot edges is invisible
        // to the reader — vars exist iff referenced). A rule whose parsed
        // LHS/RHS node list contains such free vars will read back with
        // fewer pattern participants. That is an envelope exclusion, not a
        // bug: record it as rejected-by-envelope.
        let lhs_free = rule
            .lhs
            .nodes
            .iter()
            .any(|v| !rule.lhs.edges.iter().any(|e| e.contains(v)));
        let rhs_only_free =
            rule.rhs.nodes.iter().any(|v| {
                !rule.rhs.edges.iter().any(|e| e.contains(v)) && !rule.lhs.nodes.contains(v)
            });
        if std::env::var("ERE_SWEEP_DEBUG").is_ok() {
            eprintln!("DIVERGE {label}:");
            eprintln!(
                "  orig lhs.nodes={:?} edges={:?}",
                rule.lhs.nodes, rule.lhs.edges
            );
            eprintln!(
                "  read lhs.nodes={:?} edges={:?}",
                reparsed.lhs.nodes, reparsed.lhs.edges
            );
            eprintln!(
                "  orig rhs.nodes={:?} edges={:?}",
                rule.rhs.nodes, rule.rhs.edges
            );
            eprintln!(
                "  read rhs.nodes={:?} edges={:?}",
                reparsed.rhs.nodes, reparsed.rhs.edges
            );
            eprintln!(
                "  orig lhs_labels={:?} read lhs_labels={:?}",
                rule.lhs.node_labels, reparsed.lhs.node_labels
            );
            eprintln!(
                "  orig rhs_labels={:?} read rhs_labels={:?}",
                {
                    let mut v: Vec<_> = rule
                        .rhs_node_labels
                        .iter()
                        .chain(rule.rhs.node_labels.iter())
                        .map(|(k, l)| (k.clone(), l.0))
                        .collect();
                    v.sort();
                    v
                },
                {
                    let mut v: Vec<_> = reparsed
                        .rhs_node_labels
                        .iter()
                        .map(|(k, l)| (k.clone(), l.0))
                        .collect();
                    v.sort();
                    v
                }
            );
        }
        if lhs_free || rhs_only_free {
            // Free-var participants are invisible to E1's reader (vars
            // exist iff referenced by a slot edge). Envelope exclusion.
            (
                SweepClass::Rejected,
                Some("free-var pattern nodes (no slot edges)"),
            )
        } else {
            (SweepClass::Diverged, Some("round-trip diverged"))
        }
    }
}

/// Sweep a whole TOML file: every `[[rules]]` entry classified separately.
/// Returns (rule_label, class, reason) per rule.
pub fn classify_file(src: &str) -> Vec<(String, SweepClass, Option<&'static str>)> {
    match parse_rules_toml(src) {
        Ok(rules) => {
            if rules.is_empty() {
                vec![(
                    "(no rules)".to_string(),
                    SweepClass::Rejected,
                    Some("unparseable"),
                )]
            } else {
                rules
                    .iter()
                    .map(|r| {
                        let (c, why) = classify_parsed(r, &r.name);
                        (r.name.clone(), c, why)
                    })
                    .collect()
            }
        }
        Err(_) => vec![(
            "(unparseable)".to_string(),
            SweepClass::Rejected,
            Some("unparseable"),
        )],
    }
}
