//! The full-library G-R2 sweep (EXP-R02): every rule in every
//! `rules/*.toml` file through the round trip; the sweep unit is the
//! RULE, not the file — multi-rule files contribute one entry each.
//!
//! The library lives at the workspace root (`rules/`); paths are resolved
//! via CARGO_MANIFEST_DIR.

use crate::sweep::{classify_file, SweepClass};

const LIB_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../rules");

#[test]
fn gr2_full_library_sweep() {
    let mut entries: Vec<_> = std::fs::read_dir(LIB_DIR)
        .expect("rules/ directory present")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "toml").unwrap_or(false))
        .collect();
    entries.sort();

    assert!(entries.len() >= 20, "library inventory shrank?");

    let mut total = 0usize;
    let mut round_trips = 0usize;
    let mut rejected = 0usize;
    let mut diverged = 0usize;
    let mut reasons: Vec<(String, String)> = Vec::new();
    let mut rt_names: Vec<String> = Vec::new();

    for p in &entries {
        let file = p.file_stem().unwrap().to_string_lossy().to_string();
        let src = std::fs::read_to_string(p).expect("rule file readable");
        for (rule_name, class, why) in classify_file(&src) {
            total += 1;
            let label = if file.contains(&rule_name) || rule_name.starts_with('(') {
                format!("{file}:{rule_name}")
            } else {
                format!("{file}/{rule_name}")
            };
            match class {
                SweepClass::RoundTrips => {
                    round_trips += 1;
                    rt_names.push(label);
                }
                SweepClass::Rejected => {
                    rejected += 1;
                    reasons.push((label, why.unwrap_or("?").to_string()));
                }
                SweepClass::Diverged => {
                    diverged += 1;
                    reasons.push((label, why.unwrap_or("diverged").to_string()));
                }
            }
        }
    }

    println!("\n=== G-R2 full-library sweep (per-rule) ===");
    println!("rules          : {total}");
    println!(
        "round-trips    : {round_trips} ({:.1}%)",
        100.0 * round_trips as f64 / total as f64
    );
    println!("envelope reject: {rejected}");
    println!("DIVERGED       : {diverged}");
    for (n, r) in &reasons {
        println!("  {n}: {r}");
    }
    println!("expressible ({round_trips}): {}", rt_names.join(", "));

    // A divergence is an hg-meta BUG, not an envelope limit — always fatal.
    assert_eq!(
        diverged, 0,
        "round-trip divergences must be fixed, not counted"
    );
    // The envelope number itself is REPORTED, not asserted against a target
    // (plan §6 G-R2). Pin only that it's sane:
    assert!(
        round_trips >= 1,
        "no rule in the entire library is E1-expressible?"
    );
}
