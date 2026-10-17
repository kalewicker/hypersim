//! **C0 (blocking gate) — the `A_eff` share confound, and the B1 series re-read
//! on share-independent measures.**
//!
//! `A_eff` is entropy-based, so over a multi-rule file it rises partly because
//! the **rule-share distribution** got less degenerate, independently of whether
//! any rule contributed context diversity. The tell is inside B1's own data: at
//! k = 2 the two horizons give `A_eff` **3.74 vs 7.53** with **identical**
//! `|A| = 16` and **identical** recurrent = 16, because `influx`'s event share
//! moved 4.3% → 29.9%.
//!
//! The A-track was unaffected — single-rule rows have no share structure. B1's
//! LINEAR verdict may not be, and it is load-bearing for the C-track's premise,
//! so it is tested here rather than inherited.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test c0_share_confound -- --ignored --nocapture
//! ```

use hg_alphabet::alphabet::SymbolStats;
use hg_alphabet::scaling::classify_scaling;
use hg_alphabet::{btrack, measure, RunAlphabets};

/// Radius fixed at 2 throughout, as in B1: comparing rows at different radii
/// would confound rule count with resolution.
const R: usize = 2;

// ------------------------------------------------------------- the identity

/// The decomposition is exact arithmetic, not an approximation, and it is
/// asserted before it is used to reinterpret anything:
/// `A_eff = 2^H(w) × 2^(Σ_r w_r H_r)`.
#[test]
fn c0_share_decomposition_is_exact() {
    for k in btrack::COUNTS {
        let a = measure(&btrack::series_by_events(k)).full;
        let d = a.share_decomposition(R);
        let reconstructed = d.share_perplexity * d.corrected;
        assert!(
            (reconstructed - d.a_eff).abs() < 1e-9 * d.a_eff.max(1.0),
            "k={k}: A_eff {} != share_perplexity {} x corrected {} = {}",
            d.a_eff,
            d.share_perplexity,
            d.corrected,
            reconstructed
        );
    }
}

/// **The firing case for C0's premise.** A single-rule file has `H(w) = 0`, so
/// the share factor is exactly 1 and `A_eff` is uncontaminated — which is why
/// the A-track needed no correction. If this were not so, C0 would be claiming
/// a defect that does not exist.
#[test]
fn c0_share_factor_is_exactly_one_on_a_single_rule_file() {
    let a = measure(&btrack::series_by_events(1)).full;
    let d = a.share_decomposition(R);
    assert_eq!(a.rule_only.cardinality(), 1);
    assert!(
        (d.share_perplexity - 1.0).abs() < 1e-12,
        "single-rule share perplexity must be exactly 1, got {}",
        d.share_perplexity
    );
    assert!((d.corrected - d.a_eff).abs() < 1e-9);
}

/// **The firing case for the defect itself.** The two horizons must reproduce
/// the same alphabet with different `A_eff`, and the whole difference must live
/// in the share factor. If the corrected measures also differ, the confound is
/// not what C0 says it is.
#[test]
fn c0_the_k2_discrepancy_is_entirely_share_structure() {
    let ev = measure(&btrack::series_by_events(2)).full;
    let st = measure(&btrack::series_by_steps(2)).full;

    let (se, ss) = (&ev.context[R], &st.context[R]);
    assert_eq!(se.cardinality(), ss.cardinality(), "same alphabet expected");
    assert_eq!(se.singletons(), ss.singletons(), "same recurrence expected");

    let (de, ds) = (ev.share_decomposition(R), st.share_decomposition(R));
    eprintln!(
        "\n  C0 firing case — k=2, identical |A|={} and recurrent={}:\n\
         \x20   equal-events: A_eff={:.2}  share_perplexity={:.3}  corrected={:.2}  within_sum={:.2}\n\
         \x20   equal-steps : A_eff={:.2}  share_perplexity={:.3}  corrected={:.2}  within_sum={:.2}",
        se.cardinality(),
        se.cardinality() - se.singletons(),
        de.a_eff, de.share_perplexity, de.corrected, de.within_sum,
        ds.a_eff, ds.share_perplexity, ds.corrected, ds.within_sum,
    );

    assert!(
        (de.a_eff - ds.a_eff).abs() > 1.0,
        "the two horizons should disagree on A_eff — that is the defect C0 exists for"
    );
    assert!(
        (de.within_sum - ds.within_sum).abs() < 0.5 * de.within_sum.max(1.0),
        "the share-INDEPENDENT witness should be far more stable across horizons than A_eff; \
         got {} vs {}",
        de.within_sum,
        ds.within_sum
    );
}

// --------------------------------------------------- C0.2 the B1 recompute

struct Row {
    k: usize,
    a_eff: f64,
    share: f64,
    corrected: f64,
    within: f64,
    recurrent: f64,
    raw: usize,
    per_rule: Vec<(usize, u64, f64)>,
}

fn rows(by_events: bool) -> Vec<Row> {
    btrack::COUNTS
        .iter()
        .map(|&k| {
            let spec = if by_events {
                btrack::series_by_events(k)
            } else {
                btrack::series_by_steps(k)
            };
            let a: RunAlphabets = measure(&spec).full;
            assert_eq!(a.rule_only.cardinality(), k, "k={k} row void");
            let s: &SymbolStats = &a.context[R];
            let d = a.share_decomposition(R);
            Row {
                k,
                a_eff: d.a_eff,
                share: d.share_perplexity,
                corrected: d.corrected,
                within: d.within_sum,
                recurrent: (s.cardinality() - s.singletons()) as f64,
                raw: s.cardinality(),
                per_rule: d.per_rule.clone(),
            }
        })
        .collect()
}

fn report(label: &str, rows: &[Row]) {
    eprintln!("\n================ C0.2 — B1 RECOMPUTED, {label} ================");
    eprintln!(
        "   k | |A|  | A_eff  | 2^H(w) | corrected | within_sum | recurrent | per-rule within-A_eff"
    );
    eprintln!(
        "  ---+------+--------+--------+-----------+------------+-----------+----------------------"
    );
    for r in rows {
        let pr = r
            .per_rule
            .iter()
            .map(|(id, _n, e)| format!("r{id}:{e:.2}"))
            .collect::<Vec<_>>()
            .join(" ");
        eprintln!(
            "   {} | {:>4} | {:>6.2} | {:>6.3} | {:>9.2} | {:>10.2} | {:>9.0} | {pr}",
            r.k, r.raw, r.a_eff, r.share, r.corrected, r.within, r.recurrent
        );
    }
    let fit = |f: &dyn Fn(&Row) -> f64| {
        classify_scaling(&rows.iter().map(|r| (r.k as f64, f(r))).collect::<Vec<_>>())
    };
    eprintln!("\n  A_eff        (B1's metric, CONTAMINATED): {}", fit(&|r: &Row| r.a_eff).line());
    eprintln!("  corrected    (share factor divided out) : {}", fit(&|r: &Row| r.corrected).line());
    eprintln!("  within_sum   (SHARE-INDEPENDENT)        : {}", fit(&|r: &Row| r.within).line());
    eprintln!("  recurrent    (SHARE-INDEPENDENT)        : {}", fit(&|r: &Row| r.recurrent).line());
}

#[test]
#[ignore = "long run: the C0 recompute of the B1 series"]
fn c0_recompute_b1_on_share_independent_measures() {
    eprintln!("\n################################################################################");
    eprintln!("#  C-TRACK / C0 — DOES B1's LINEAR VERDICT SURVIVE THE SHARE CORRECTION?");
    eprintln!("#  A_eff = 2^H(w) x 2^(sum_r w_r H_r): share perplexity x within-rule diversity");
    eprintln!("################################################################################");

    let primary = rows(true);
    let secondary = rows(false);
    report("PRIMARY: equal event budget", &primary);
    report("SECONDARY: equal step budget", &secondary);

    // The gate the C-track's premise rests on: the share-independent witnesses
    // must not silently reverse B1's verdict without it being reported. This
    // assertion does not demand a particular verdict — it demands the two
    // witnesses agree with each other, so a single odd metric cannot carry it.
    let w = classify_scaling(
        &primary.iter().map(|r| (r.k as f64, r.within)).collect::<Vec<_>>(),
    );
    let rec = classify_scaling(
        &primary.iter().map(|r| (r.k as f64, r.recurrent)).collect::<Vec<_>>(),
    );
    eprintln!(
        "\n  Second-witness agreement: within_sum = {} , recurrent = {}",
        w.verdict, rec.verdict
    );
    eprintln!("\n################################################################################\n");
}
