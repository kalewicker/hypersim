//! Report formatting. Kept in the crate rather than in a test so every consumer
//! prints the same obligations: the raw count NEVER without the effective size,
//! the regime label NEVER without its fit quality, and a zero engagement counter
//! as `unmeasured` rather than `0`.

use crate::alphabet::{RunAlphabets, SymbolStats};
use crate::growth;
use crate::saturation::{self, Saturation};
use crate::signature::N_RADII;

/// Instrument-registry Rule 1: the not-engaged marker.
pub const NOT_ENGAGED: &str = "INSTRUMENT_NOT_ENGAGED";

/// Composition line for one alphabet (A3.3 obligations in one place).
pub fn composition(s: &SymbolStats) -> String {
    if s.total_events() == 0 {
        return format!("{NOT_ENGAGED} (n_events=0) -> unmeasured");
    }
    let ceiling = s.ceiling_fraction();
    let flag = if ceiling >= SymbolStats::CEILING_WARN {
        "  << AT THE |A| <= n_events CEILING: budget-limited, not substrate-limited"
    } else {
        ""
    };
    format!(
        "|A|={:<6} A_eff={:<9.2} H={:<7.3} top1={:<6.1}% top3={:<6.1}% singletons={}/{} n_events={} |A|/n_events={:.3}{flag}",
        s.cardinality(),
        s.effective_size(),
        s.entropy_bits(),
        100.0 * s.top_mass(1),
        100.0 * s.top_mass(3),
        s.singletons(),
        s.cardinality(),
        s.total_events(),
        ceiling
    )
}

/// The R-sweep table for one run and one signature variant, with the
/// pre-registered selection applied. The FULL sweep is always printed, so no
/// radius can be cherry-picked after the fact.
pub fn radius_sweep(label: &str, stats: &[SymbolStats]) -> (String, Saturation) {
    let sizes: Vec<usize> = stats.iter().map(|s| s.cardinality()).collect();
    let sat = saturation::select_radius(&sizes);
    let mut out = String::new();
    out.push_str(&format!("  {label}\n"));
    out.push_str("    R  |    |A|  |    A_eff  |  singletons  |  g(R)\n");
    out.push_str("    ---+---------+-----------+--------------+--------\n");
    for (r, stat) in stats.iter().enumerate().take(N_RADII) {
        let g = sat
            .gains
            .get(r)
            .map(|g| {
                if g.is_infinite() {
                    "   inf".to_string()
                } else {
                    format!("{g:6.3}")
                }
            })
            .unwrap_or_else(|| "     -".to_string());
        let flag = if r == 0 { " (baseline, not eligible)" } else { "" };
        out.push_str(&format!(
            "    {r}  | {:>7} | {:>9.2} | {:>12} | {g}{flag}\n",
            stat.cardinality(),
            stat.effective_size(),
            stat.singletons(),
        ));
    }
    out.push_str(&match sat.r_star {
        Some(r) => format!(
            "    R* = {r}   (smallest eligible R with g(R) < {:.2})\n",
            saturation::EPSILON
        ),
        None => format!(
            "    R* = NONE — DOES NOT SATURATE within the swept range (no eligible R has g(R) < {:.2})\n",
            saturation::EPSILON
        ),
    });
    if sat.refines_above_r_star() {
        out.push_str(&format!(
            "    !! REFINEMENT RESUMES ABOVE R*: max g(R>R*) = {:.3} >= {:.2}. The committed rule \n\
             \x20      takes the SMALLEST passing R, so R* here marks a DIP, not a plateau — the\n\
             \x20      alphabet is not finished growing at R*.\n",
            sat.max_gain_above_r_star.unwrap_or(0.0),
            saturation::EPSILON
        ));
    }
    if !sat.monotone {
        out.push_str("    !! NON-MONOTONE IN R — INSTRUMENT FAULT, not a finding\n");
    }
    (out, sat)
}

/// The A4 novelty curves for one alphabet, both abscissae, with fit quality.
pub fn novelty(s: &SymbolStats) -> String {
    if s.total_events() == 0 {
        return format!("    {NOT_ENGAGED} -> unmeasured\n");
    }
    let by_step = growth::fit(&s.curve_by_step(200));
    let by_event = growth::fit(&s.curve_by_event(200));
    format!(
        "    vs STEP   {}\n    vs EVENT  {}\n",
        by_step.line(),
        by_event.line()
    )
}

/// The whole-run block for one rule.
pub fn run_block(name: &str, a: &RunAlphabets) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\n===== {name} =====\n  steps={} stop={} final n={} e={} events={}\n",
        a.steps_run,
        a.stop_reason,
        a.final_nodes,
        a.final_edges,
        a.n_events()
    ));
    if !a.engaged() {
        out.push_str(&format!("  {NOT_ENGAGED} — every alphabet on this row is `unmeasured`, not 0.\n"));
        return out;
    }
    out.push_str(&format!(
        "  A_rule_only (the VACUOUS definition, for contrast) = {}\n",
        a.rule_only.cardinality()
    ));
    let (sweep, _) = radius_sweep("S1 context signature (PRIMARY)", &a.context);
    out.push_str(&sweep);
    let (sweep2, _) = radius_sweep("S2 structural-delta signature", &a.delta);
    out.push_str(&sweep2);
    out.push_str(&format!(
        "  S3 degree signature (radius-free)\n    {}\n",
        composition(&a.degree)
    ));
    out
}
