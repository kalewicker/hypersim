//! Render a RunSummary into the 11 canonical sections (v4 §11).
//!
//! Every section is present and non-empty — or explicitly says why not.

use hg_schema::{GateStatus, MetricsRow, RunSummary};
use std::path::Path;

pub const MEASURED: &str = "[Measured]";
pub const ESTIMATED: &str = "[Estimated]";

pub fn render_report(s: &RunSummary) -> String {
    let mut md = String::with_capacity(8192);
    md.push_str("# Hypersim Run Report\n\n");

    // 1. Manifest — provenance bookkeeping.
    md.push_str("## 1. Manifest\n\n");
    let p = &s.provenance;
    md.push_str(&format!("- **Run ID**: {}\n", nz(&p.run_id)));
    md.push_str(&format!("- **Experiment**: {}\n", nz(&p.experiment_name)));
    md.push_str(&format!("- **Seed**: {}\n", p.seed));
    md.push_str(&format!("- **State**: {}\n", nz(&p.state)));
    md.push_str(&format!("- **Thread count**: {}\n", p.thread_count));
    md.push_str(&format!("- **Hypersim version**: {}\n", nz(&p.hypersim_version)));
    md.push_str(&format!("- **Start**: {}\n", nz(&p.start_time)));
    md.push_str(&format!(
        "- **End**: {}\n",
        p.end_time.as_deref().unwrap_or("(not sealed)")
    ));
    md.push_str(&format!("- **Max steps**: {}\n", p.max_steps));
    md.push_str(&format!("- **Final step**: {}\n", p.final_step));
    md.push_str(&format!("- **Config hash**: {}\n\n", nz(&p.config_hash)));

    // 2. Run Parameters — the experimental configuration itself.
    // Restored as its own section (v4 §11): config visibility is a
    // reproducibility requirement; reproducing a run needs both 1 and 2.
    md.push_str("## 2. Run Parameters\n\n");
    if s.run_parameters.is_empty() {
        md.push_str("No parameters recorded — this run was configured programmatically without a serialized config.\n\n");
    } else {
        for (k, v) in &s.run_parameters {
            md.push_str(&format!("- **{k}**: {v}\n"));
        }
        md.push('\n');
    }

    // 3. Campaign Summary.
    md.push_str("## 3. Campaign Summary\n\n");
    match &s.campaign_summary {
        Some(c) => md.push_str(&format!("{c}\n\n")),
        None => md.push_str("Not applicable — single run, not part of a campaign.\n\n"),
    }

    // 4. Gate Status (+ Fixture Truth Table subsection).
    md.push_str("## 4. Gate Status\n\n");
    if s.gate_statuses.is_empty() {
        md.push_str("No gates evaluated — evaluator disabled for this run.\n\n");
    } else {
        md.push_str("| Gate | Status |\n|---|---|\n");
        for (name, status) in &s.gate_statuses {
            md.push_str(&format!("| {} | {} |\n", name.as_str(), status.as_str()));
        }
        md.push('\n');
        let unlocked = if s.science_unlocked { "yes" } else { "no" };
        md.push_str(&format!("Science unlocked: **{unlocked}**"));
        if let Some(reason) = &s.science_locked_reason {
            md.push_str(&format!(" — {reason}"));
        }
        md.push_str("\n\n");
    }
    md.push_str("### Fixture Truth Table\n\n");
    if s.fixture_truth_table.is_empty() {
        md.push_str("Truth table not run (evaluator disabled).\n\n");
    } else {
        md.push_str("| Fixture | Classified | Pass |\n|---|---|---|\n");
        for (name, regime, pass) in &s.fixture_truth_table {
            md.push_str(&format!(
                "| {name} | {regime} | {} |\n",
                if *pass { "✅" } else { "❌" }
            ));
        }
        md.push('\n');
    }

    // 5. Regime Summary.
    md.push_str("## 5. Regime Summary\n\n");
    if s.regime_summary.is_empty() {
        md.push_str("No windows classified — run produced no complete evaluation windows.\n\n");
    } else {
        md.push_str("| Regime | Windows |\n|---|---|\n");
        for (regime, count) in &s.regime_summary {
            md.push_str(&format!("| {} | {} |\n", regime.as_str(), count));
        }
        md.push('\n');
    }

    // 6. Evaluator Details.
    md.push_str("## 6. Evaluator Details\n\n");
    if s.evaluator_details.is_empty() {
        md.push_str("Evaluator disabled for this run.\n\n");
    } else {
        for (heading, body) in &s.evaluator_details {
            md.push_str(&format!("### {heading}\n\n{body}\n\n"));
        }
    }

    // 7. Causal Language Summary (+ Constructor Ecology subsection).
    md.push_str("## 7. Causal Language Summary\n\n");
    if s.causal_summary.is_empty() {
        md.push_str("Causal layer disabled for this run.\n\n");
    } else {
        for (k, v) in &s.causal_summary {
            md.push_str(&format!("- **{k}**: {v}\n"));
        }
        md.push('\n');
    }
    md.push_str("### Constructor Ecology\n\n");
    if s.constructor_ecology.is_empty() {
        md.push_str("No constructor candidates observed.\n\n");
    } else {
        for (k, v) in &s.constructor_ecology {
            md.push_str(&format!("- **{k}**: {v}\n"));
        }
        md.push('\n');
    }

    // 8. Final Metrics Table.
    md.push_str("## 8. Final Metrics Table\n\n");
    match &s.final_metrics {
        None => md.push_str("No final metrics — the run completed before the first window.\n\n"),
        Some(m) => render_metrics(&mut md, m),
    }

    // 8b. Locality & Leakage (LK5) — subsection so the 11 canonical
    // section numbers stay stable.
    md.push_str("### Locality & Leakage\n\n");
    if s.leakage_rows.is_empty() && s.leakage_summary.is_empty() {
        md.push_str("Leakage track disabled for this run.\n\n");
    } else {
        if !s.leakage_rows.is_empty() {
            md.push_str(
                "Phase-plane trajectory (v2 §1.5): all EXP verdicts are trajectories \
                 through (wormhole_frac, area_law_defect), never a single scalar's \
                 direction. The detour channel is valid only in the sparse phase. All \
                 [Estimated]; window-timescale quantities — step-level dynamics are \
                 unmeasured.\n\n",
            );
            md.push_str("| step | detour p50 | detour p95 | wormhole_frac | wh_count | wh_comp | cut_exponent | r² | area_law_defect | d_bfs | phase |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
            // Evolution table: at most 12 rows — first, last, and evenly
            // spaced between.
            let n = s.leakage_rows.len();
            let picks: Vec<usize> = if n <= 12 {
                (0..n).collect()
            } else {
                (0..12).map(|i| i * (n - 1) / 11).collect()
            };
            for &i in &picks {
                let r = &s.leakage_rows[i];
                let opt = |v: Option<f32>| {
                    v.map(|x| format!("{x:.3}")).unwrap_or_else(|| "—".into())
                };
                md.push_str(&format!(
                    "| {} | {} | {} | {:.4} | {} | {} | {} | {} | {} | {} | {} |\n",
                    r.step,
                    r.detour_p50,
                    r.detour_p95,
                    r.wormhole_frac,
                    r.wormhole_count_est.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
                    r.wormhole_components.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
                    opt(r.cut_exponent),
                    opt(r.cut_fit_r2),
                    opt(r.area_law_defect),
                    opt(r.d_bfs),
                    r.phase.as_str(),
                ));
            }
            md.push('\n');
        }
        for (k, v) in &s.leakage_summary {
            md.push_str(&format!("- **{k}**: {v}\n"));
        }
        if !s.leakage_summary.is_empty() {
            md.push('\n');
        }
    }

    // 9. Suspicious Metric Flags.
    md.push_str("## 9. Suspicious Metric Flags\n\n");
    if s.suspicious_flags.is_empty() {
        md.push_str("None detected.\n\n");
    } else {
        for flag in &s.suspicious_flags {
            md.push_str(&format!("- ⚠️ {flag}\n"));
        }
        md.push('\n');
    }

    // 10. Observations.
    md.push_str("## 10. Observations\n\n");
    if s.observations.is_empty() {
        md.push_str("No observations recorded.\n\n");
    } else {
        for o in &s.observations {
            md.push_str(&format!("- {o}\n"));
        }
        md.push('\n');
    }

    // 11. Recommended Next Actions.
    md.push_str("## 11. Recommended Next Actions\n\n");
    if s.recommended_actions.is_empty() {
        md.push_str("- Re-run with a different seed to check regime stability.\n\n");
    } else {
        for a in &s.recommended_actions {
            md.push_str(&format!("- {a}\n"));
        }
        md.push('\n');
    }

    md
}

fn render_metrics(md: &mut String, m: &MetricsRow) {
    md.push_str("| Metric | Value | Epistemic |\n|---|---|---|\n");
    let mut row = |k: &str, v: String, tag: &str| {
        md.push_str(&format!("| {k} | {v} | {tag} |\n"));
    };
    row("window_id", m.window_id.to_string(), MEASURED);
    row("steps", format!("{}..{}", m.step_start, m.step_end), MEASURED);
    row("n_nodes", m.n_nodes.to_string(), MEASURED);
    row("n_edges", m.n_edges.to_string(), MEASURED);
    row("accepted_rewrites", m.accepted_rewrites.to_string(), MEASURED);
    row("attempted_matches", m.attempted_matches.to_string(), MEASURED);
    row("accepted_per_step", format!("{:.6}", m.accepted_per_step), MEASURED);
    opt_row(&mut row, "mean_degree", m.mean_degree, MEASURED);
    opt_row(&mut row, "activity", m.activity, MEASURED);
    opt_row(&mut row, "entropy_rate", m.entropy_rate, MEASURED);
    opt_row(&mut row, "pi_normalized", m.pi_normalized, ESTIMATED);
    // A log-log ball-growth fit, not a direct readout — Estimated.
    opt_row(&mut row, "d_eff", m.d_eff, ESTIMATED);
    opt_row(&mut row, "type_histogram_entropy", m.type_histogram_entropy, MEASURED);
    opt_row(&mut row, "dominant_type_fraction", m.dominant_type_fraction, MEASURED);
    opt_row(&mut row, "type_histogram_jsd", m.type_histogram_jsd, MEASURED);
    opt_row(&mut row, "new_type_bin_rate", m.new_type_bin_rate, MEASURED);
    opt_row(&mut row, "interestingness_score", m.interestingness_score, ESTIMATED);
    // EI/CE: statistical estimates from finite transition counts.
    opt_row(&mut row, "ei_micro", m.ei_micro, ESTIMATED);
    opt_row(&mut row, "ei_macro", m.ei_macro, ESTIMATED);
    opt_row(&mut row, "ce_structural", m.ce_structural, ESTIMATED);
    opt_row(&mut row, "ce_energetic", m.ce_energetic, ESTIMATED);
    opt_row(&mut row, "ce_functional", m.ce_functional, ESTIMATED);
    opt_row(&mut row, "ce_best", m.ce_best, ESTIMATED);
    if let Some(strat) = &m.ce_best_strategy {
        row("ce_best_strategy", strat.clone(), ESTIMATED);
    }
    if let Some(v) = m.wl_class_count {
        row("wl_class_count", v.to_string(), MEASURED);
    }
    if let Some(v) = m.task_lattice_depth {
        row("task_lattice_depth", v.to_string(), MEASURED);
    }
    if let Some(v) = m.constructor_count {
        row("constructor_count", v.to_string(), MEASURED);
    }
    if let Some(v) = m.replicator_count {
        row("replicator_count", v.to_string(), MEASURED);
    }
    if let Some(r) = &m.regime {
        row("regime", r.clone(), ESTIMATED);
    }
    md.push('\n');
}

fn opt_row(row: &mut impl FnMut(&str, String, &str), k: &str, v: Option<f64>, tag: &str) {
    if let Some(v) = v {
        row(k, format!("{v:.6}"), tag);
    }
}

fn nz(s: &str) -> &str {
    if s.is_empty() {
        "(unset)"
    } else {
        s
    }
}

/// Convenience: gate helper for `GateStatus` display in caller code.
pub fn gate_str(g: GateStatus) -> &'static str {
    g.as_str()
}

pub fn write_report(summary: &RunSummary, path: &Path) -> std::io::Result<()> {
    std::fs::write(path, render_report(summary))
}
