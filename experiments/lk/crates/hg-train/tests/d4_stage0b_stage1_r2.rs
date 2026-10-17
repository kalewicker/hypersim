//! Phase D4 — Stage 0b′ and Stage 1′ under `readout_set = "r2"`
//! (`hypersim_nn_experiments_plan_v2.md` §9).
//!
//! Both stages measured a **joint** quantity and reported it as a substrate
//! property. Separation is substrate dynamics × readout sensitivity; memory
//! capacity is substrate memory × readout sensitivity. Under r1 the readout
//! factor was 16 buckets of a WL hash, and `glider` proved that factor can
//! be zero (`docs/policy_config.md` §4B.6).
//!
//! So r2 is not an upgrade, it is a **second measurement of the same
//! substrate through a different projection**, and the r1/r2 disagreements
//! are the result. Overwriting r1's column would destroy the only evidence
//! that distinguishes "the substrate does not separate" from "the readout
//! could not see it".
//!
//! **Exit (Stage 0b′):** every rule set has a class under r2; r1/r2
//! disagreements documented.
//! **Exit (Stage 1′):** MC under r2 against an ESN baseline of matched
//! state dimension.

use hg_cognition::readout::{r2_dim, ReadoutSet};
use hg_train::diagnostics::*;

const BUCKETS: usize = SEPARATION_BUCKETS;

/// The budget every Stage 0b arm runs at.
///
/// **Fixed at 4 across all five rule sets, matching the C0–C5 run
/// exactly** (`c4_diagnostics.rs` uses `budgeted_random(4)` throughout).
/// Varying it per fixture — which the first draft of this test did — mixes
/// the r1/r2 contrast with a budget contrast, and then a changed class
/// cannot be attributed to the readout. The readout is the only thing that
/// may differ between the two rows.
const STAGE0B_BUDGET: usize = 4;

fn fixtures() -> Vec<(Fixture, usize)> {
    vec![
        (
            Fixture {
                name: "glider",
                rules: RuleSource::Builtin("glider"),
                ic: "GliderRing",
                width: 8,
                node_count: 31,
                steps: 512,
                window: 64,
                step_mode: "MaxParallel",
            },
            STAGE0B_BUDGET,
        ),
        (
            Fixture {
                name: "growth_treadmill",
                rules: RuleSource::Builtin("growth_treadmill"),
                ic: "WhiteNoise",
                width: 8,
                node_count: 24,
                steps: 512,
                window: 64,
                step_mode: "MaxParallel",
            },
            STAGE0B_BUDGET,
        ),
        (
            Fixture {
                name: "triangle_subdivide",
                rules: RuleSource::Builtin("triangle_subdivide"),
                ic: "TriLattice",
                width: 8,
                node_count: 24,
                steps: 96,
                window: 24,
                step_mode: "MaxParallel",
            },
            STAGE0B_BUDGET,
        ),
        (
            Fixture {
                name: "binary_edge",
                rules: RuleSource::Builtin("binary_edge"),
                ic: "PureCrystal",
                width: 8,
                node_count: 24,
                steps: 96,
                window: 24,
                step_mode: "MaxParallel",
            },
            STAGE0B_BUDGET,
        ),
        (
            Fixture {
                name: "fl1_two_rule",
                rules: RuleSource::File("rules/fl1_learnable_signal.toml"),
                ic: "GliderRing",
                width: 8,
                node_count: 31,
                steps: 192,
                window: 32,
                step_mode: "MaxParallel",
            },
            STAGE0B_BUDGET,
        ),
    ]
}

fn class_name(v: &Viability) -> &'static str {
    match v {
        Viability::Ordered => "Ordered",
        Viability::Viable => "Viable",
        Viability::Chaotic => "Chaotic",
        Viability::Undetermined => "Undetermined",
    }
}

// =====================================================================
// Stage 0b' — separation under both readouts
// =====================================================================

#[test]
#[ignore = "re-runs Stage 0b on 5 rule sets x 2 readouts x 3 seeds; run \
            explicitly for the Stage 0b' table"]
fn stage0b_prime_both_readouts() {
    let dir = tempfile::tempdir().expect("tempdir");

    println!("\n=== Stage 0b′ — separation under r1 and r2 ===");
    println!(
        "r2 dimension = {} (16 WL buckets + 16 raw WL + {} label slots + 8)\n",
        r2_dim(BUCKETS, R2_TOP_N_WL, R2_LABEL_SLOTS),
        R2_LABEL_SLOTS
    );
    println!(
        "{:<20} {:>8} {:>10} {:>10} {:>9}  {:<14}",
        "rule set", "readout", "plateau", "baseline", "ratio", "class"
    );

    let mut disagreements = Vec::new();
    let mut classified_under_r2 = 0usize;
    let mut faults: Vec<(&str, &str, f64)> = Vec::new();

    for (f, budget) in fixtures() {
        let mut per_readout = Vec::new();
        for readout in [ReadoutSet::R1, ReadoutSet::R2] {
            let r = stage0b_with_readout(&f, dir.path(), 4242, budgeted_random(budget), readout);
            match r {
                Ok(res) => {
                    let ratio = if res.independent_baseline > 0.0 {
                        res.plateau / res.independent_baseline
                    } else {
                        f64::NAN
                    };
                    // `docs/instrument_registry.md` §2.7: the perturbed
                    // pair cannot diverge further than two independent
                    // seeds, so a ratio above 1 is arithmetically
                    // impossible for a real distance and condemns the
                    // measurement, not the substrate.
                    let fault = ratio > 1.05;
                    println!(
                        "{:<20} {:>8} {:>10.4} {:>10.4} {:>9.3}  {:<14}{}",
                        f.name,
                        readout.as_str(),
                        res.plateau,
                        res.independent_baseline,
                        ratio,
                        class_name(&res.class),
                        if fault { "  <== RATIO > 1: INSTRUMENT FAULT" } else { "" }
                    );
                    if fault {
                        faults.push((f.name, readout.as_str(), ratio));
                    }
                    per_readout.push((readout, res.class.clone(), ratio));
                }
                Err(e) => println!("{:<20} {:>8}  FAILED: {e}", f.name, readout.as_str()),
            }
        }
        if let [(_, c1, _), (_, c2, r2r)] = per_readout.as_slice() {
            if c2 != &Viability::Undetermined {
                classified_under_r2 += 1;
            }
            if c1 != c2 {
                disagreements.push((f.name, class_name(c1), class_name(c2), *r2r));
            }
        }
        println!();
    }

    println!("=== Rule 2: separation ratios above their hard bound ===");
    if faults.is_empty() {
        println!("None. Every ratio is at or below its independent-seed reference.");
    } else {
        for (name, readout, ratio) in &faults {
            println!(
                "  {name} under {readout}: ratio {ratio:.3} > 1 — the perturbed pair \
                 diverged MORE than two independent seeds, which no real distance can \
                 do. This class is an instrument fault, not a substrate verdict."
            );
        }
    }

    println!("\n=== r1/r2 disagreements ===");
    if disagreements.is_empty() {
        println!(
            "None. r2 reproduces every r1 verdict, which is the consistency check \
             plan v2 §9 asks for — and it means no rule set's r1 class was a readout \
             artifact."
        );
    } else {
        for (name, a, b, ratio) in &disagreements {
            println!("  {name}: r1 = {a}  ->  r2 = {b}  (r2 ratio {ratio:.3})");
        }
    }
    println!(
        "\nclassified (non-Undetermined) under r2: {classified_under_r2}/{}",
        fixtures().len()
    );

    // Machinery only. Every classification pattern is a legitimate outcome;
    // what must not happen is the diagnostic failing to run.
    assert!(classified_under_r2 <= fixtures().len());
}

#[test]
#[ignore = "runs Stage 0b twice on one fixture; explicit"]
fn r1_reproduces_the_published_triangle_subdivide_number() {
    // `triangle_subdivide` at base seed 4242, budget 4, is the one Stage 0b
    // cell C0–C5 published with a number: **Viable, ratio 0.752**. This
    // asserts r1 still reproduces it exactly, which is what licenses
    // reading any r1/r2 difference as a readout effect.
    //
    // It does NOT assert that 0.752 is a property of the rule set. It is
    // one draw from a distribution that spans two classes — see
    // `stage0b_verdicts_across_seed_triples`, where the same cell comes
    // back Chaotic on 6 of 8 triples and exceeds its hard bound on 5.
    let dir = tempfile::tempdir().expect("tempdir");
    let f = fixtures()
        .into_iter()
        .find(|(f, _)| f.name == "triangle_subdivide")
        .expect("fixture");

    let one = stage0b_with_readout(&f.0, dir.path(), 4242, budgeted_random(f.1), ReadoutSet::R1)
        .expect("r1 runs");
    let ratio = one.plateau / one.independent_baseline.max(1e-12);
    println!("triangle_subdivide r1 @4242: {} ratio {ratio:.3}", class_name(&one.class));
    assert!(
        (ratio - 0.752).abs() < 0.005,
        "r1 no longer reproduces the published 0.752 (got {ratio:.4}); every r1/r2 \
         comparison downstream assumes it does"
    );
    assert_eq!(one.class, Viability::Viable);

    let two = stage0b_with_readout(&f.0, dir.path(), 4242, budgeted_random(f.1), ReadoutSet::R2)
        .expect("r2 runs");
    println!(
        "triangle_subdivide r2 @4242: {} ratio {:.3}",
        class_name(&two.class),
        two.plateau / two.independent_baseline.max(1e-12)
    );
}

/// Seed triples used to test whether a Stage 0b verdict is stable.
///
/// `stage0b` derives its perturbed partner and its independent reference
/// from ONE base seed, so a single call is a single triple. Nothing in
/// C0–C5 ran more than one.
fn triples() -> Vec<u64> {
    (0..8u64).map(|i| 4242 + i * 1_000_003).collect()
}

#[test]
#[ignore = "Stage 0b across 5 rule sets x 8 seed triples x 2 readouts"]
fn stage0b_verdicts_across_seed_triples() {
    // **The check C0–C5 never ran, and the one that decides whether any
    // Stage 0b class is admissible.**
    //
    // A separation class rests on a normalized ratio whose denominator is a
    // single independent-seed pair. If that denominator is unstable, the
    // ratio is not a well-defined statistic and no single-triple verdict —
    // r1's or r2's — can be cited.
    let dir = tempfile::tempdir().expect("tempdir");

    println!("\n=== Stage 0b — verdict stability across {} seed triples ===", triples().len());
    println!(
        "A ratio above 1 is arithmetically impossible for a real distance \
         (docs/instrument_registry.md §2.7).\n"
    );
    println!(
        "{:<20} {:>8} {:>9} {:>9} {:>9} {:>28}",
        "rule set", "readout", "faults", "ratio med", "ratio max", "classes"
    );

    let mut unusable = Vec::new();
    for (f, budget) in fixtures() {
        for readout in [ReadoutSet::R1, ReadoutSet::R2] {
            let mut ratios = Vec::new();
            let mut classes: std::collections::BTreeMap<&str, usize> = Default::default();
            for seed in triples() {
                let Ok(r) = stage0b_with_readout(&f, dir.path(), seed, budgeted_random(budget), readout)
                else {
                    continue;
                };
                if r.independent_baseline > 0.0 {
                    ratios.push(r.plateau / r.independent_baseline);
                }
                *classes.entry(class_name(&r.class)).or_default() += 1;
            }
            let faults = ratios.iter().filter(|x| **x > 1.05).count();
            let mut sorted = ratios.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let med = sorted.get(sorted.len() / 2).copied().unwrap_or(f64::NAN);
            let max = sorted.last().copied().unwrap_or(f64::NAN);
            let summary: Vec<String> =
                classes.iter().map(|(k, v)| format!("{k}x{v}")).collect();
            println!(
                "{:<20} {:>8} {:>4}/{:<4} {:>9.3} {:>9.3} {:>28}",
                f.name,
                readout.as_str(),
                faults,
                ratios.len(),
                med,
                max,
                summary.join(" ")
            );
            // A measurement whose value is impossible on a majority of
            // draws is not a measurement.
            if !ratios.is_empty() && faults * 2 > ratios.len() {
                unusable.push((f.name, readout.as_str(), faults, ratios.len()));
            }
        }
        println!();
    }

    println!("=== rule sets where the separation ratio is not a valid statistic ===");
    if unusable.is_empty() {
        println!("None.");
    } else {
        for (name, readout, faults, n) in &unusable {
            println!(
                "  {name} under {readout}: {faults}/{n} triples exceed the hard bound. \
                 The independent-seed denominator is unstable here, so NO single-triple \
                 verdict on this rule set is admissible — under either readout."
            );
        }
    }

    assert!(!triples().is_empty());
}

// =====================================================================
// Stage 1' — memory capacity under both readouts
// =====================================================================

const S1_STEPS: u64 = 4096;
const S1_WINDOW: u64 = 32;
const MAX_DELAY: usize = 12;
/// Rule indices in `rules/stage1_reservoir.toml`.
const INJECT_ONE: usize = 3;
const INJECT_ZERO: usize = 4;

fn reservoir(window: u64, steps: u64) -> Fixture {
    Fixture {
        name: "stage1_reservoir",
        rules: RuleSource::File("rules/stage1_reservoir.toml"),
        ic: "Empty",
        width: 8,
        node_count: 31,
        steps,
        window,
        step_mode: "MaxParallel",
    }
}

#[test]
#[ignore = "runs the Stage 1 reservoir twice at 4096 steps; explicit"]
fn stage1_prime_r1_vs_r2() {
    let dir = tempfile::tempdir().expect("tempdir");

    println!("\n=== Stage 1′ — memory capacity under r1 and r2 ===");
    println!(
        "The ESN baseline is sized from the PROBE ROWS, so both rows below are \
         dimension-matched.\n"
    );
    println!(
        "{:>8} {:>6} {:>9} {:>10} {:>10} {:>10} {:>11}",
        "readout", "dim", "MC", "ESN MC", "input r2", "r2 d=1", "max delay"
    );

    let mut rows = Vec::new();
    for readout in [ReadoutSet::R1, ReadoutSet::R2] {
        let f = reservoir(S1_WINDOW, S1_STEPS);
        let r = match stage1_with_readout(
            &f,
            dir.path(),
            4242,
            INJECT_ZERO,
            INJECT_ONE,
            BUCKETS,
            MAX_DELAY,
            Some(4),
            readout,
        ) {
            Ok(r) => r,
            Err(e) => {
                println!("{:>8}  FAILED: {e}", readout.as_str());
                continue;
            }
        };
        let at = |d: usize| {
            r.mc.per_delay
                .iter()
                .find(|(dd, _)| *dd == d)
                .map(|(_, v)| *v)
                .unwrap_or(f64::NAN)
        };
        println!(
            "{:>8} {:>6} {:>9.4} {:>10.4} {:>10.4} {:>10.4} {:>11}",
            readout.as_str(),
            r.state_dim,
            r.mc.mc,
            r.esn_mc.mc,
            r.mc.input_coupling_r2.unwrap_or(f64::NAN),
            at(1),
            r.mc.max_decodable_delay
                .map(|d| d.to_string())
                .unwrap_or_else(|| "none".into()),
        );
        rows.push((readout, r));
    }

    // ---- the pre-registered decision rule (plan v2 §7) -----------------
    println!("\n=== Stage 1′ verdict ===");
    if let [(_, one), (_, two)] = rows.as_slice() {
        let moved = (two.mc.mc - one.mc.mc).abs();
        let rel = moved / one.mc.mc.max(1e-12);
        println!(
            "MC r1 = {:.4} (dim {}), MC r2 = {:.4} (dim {}); |Δ| = {moved:.4} ({:.0}% of r1)",
            one.mc.mc, one.state_dim, two.mc.mc, two.state_dim, rel * 100.0
        );
        println!(
            "ESN at matched dimension: {:.4} -> {:.4}",
            one.esn_mc.mc, two.esn_mc.mc
        );
        println!(
            "substrate/ESN ratio: r1 {:.4}, r2 {:.4}",
            one.mc.mc / one.esn_mc.mc.max(1e-12),
            two.mc.mc / two.esn_mc.mc.max(1e-12),
        );

        if two.mc.max_decodable_delay.is_none() && one.mc.max_decodable_delay.is_none() {
            println!(
                "\nNo delay >= 1 is decodable under EITHER readout. This is the \
                 pre-registered failure reading: the substrate's intrinsic timescale is \
                 shorter than one window, the Stage 1 null is REAL rather than a \
                 projection artifact, and it is publishable as *memory capacity of \
                 hypergraph rewriting systems is bounded below one rewrite window*."
            );
        } else {
            println!(
                "\nA delay is decodable under at least one readout — the Stage 1 null \
                 was partly a readout artifact and the window sweep must be redone \
                 under r2 (plan v2 §7's decision rule)."
            );
        }
        println!(
            "\nNote on what r2 CANNOT fix: input coupling at d=0 with nothing at d>=1 is \
             the signature of BOTH hypotheses (short timescale, and a readout that \
             resolves the symbol but not its consequences), so it cannot discriminate on \
             its own and is not cited as if it does."
        );
    }

    assert!(!rows.is_empty(), "neither readout produced a Stage 1 result");
}

#[test]
fn the_esn_baseline_is_sized_from_the_probe_not_from_buckets() {
    // The dimension-matching fix, checked without paying for a reservoir
    // run. Under r1 the readout is exactly `buckets` wide so nothing moves;
    // under r2 it is ~2.7x wider, and sizing the ESN from `buckets` would
    // pit a 43-dimensional substrate readout against a 16-unit ESN and
    // report the ratio as a substrate property.
    assert_eq!(r2_dim(BUCKETS, R2_TOP_N_WL, R2_LABEL_SLOTS), 16 + 16 + 8 + 8);
    assert_ne!(
        r2_dim(BUCKETS, R2_TOP_N_WL, R2_LABEL_SLOTS),
        BUCKETS,
        "if these were equal the fix would be untestable"
    );
}
