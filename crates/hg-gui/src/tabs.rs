//! Dashboard tabs driven by a small internal registry (v4 §6.4), not
//! hardcoded panel wiring.

use egui_plot::{Legend, Line, Plot, PlotPoints};
use hg_app_api::SharedState;

pub type TabRenderer = fn(&mut egui::Ui, &SharedState);

pub struct TabRegistry {
    pub tabs: Vec<(&'static str, TabRenderer)>,
    pub selected: usize,
}

impl Default for TabRegistry {
    fn default() -> Self {
        Self {
            tabs: vec![
                ("Metrics", metrics_tab as TabRenderer),
                ("Regime & Gates", gates_tab),
                ("Knowledge Graph", kg_tab),
            ],
            selected: 0,
        }
    }
}

impl TabRegistry {
    pub fn show(&mut self, ui: &mut egui::Ui, shared: &SharedState) {
        ui.horizontal(|ui| {
            for (i, (name, _)) in self.tabs.iter().enumerate() {
                if ui.selectable_label(self.selected == i, *name).clicked() {
                    self.selected = i;
                }
            }
        });
        ui.separator();
        (self.tabs[self.selected].1)(ui, shared);
    }
}

fn metrics_tab(ui: &mut egui::Ui, shared: &SharedState) {
    let hist = shared.metrics_history.read();
    if hist.rows.is_empty() {
        ui.label("No windows yet.");
        return;
    }
    let series = |f: fn(&hg_schema::MetricsRow) -> Option<f64>| -> PlotPoints {
        hist.rows
            .iter()
            .enumerate()
            .filter_map(|(i, r)| f(r).map(|v| [i as f64, v]))
            .collect()
    };
    // Line names only render when a Legend is attached to the plot —
    // without it every series is an anonymous colored line.
    Plot::new("nodes_plot").height(140.0).legend(Legend::default()).show(ui, |p| {
        p.line(Line::new(series(|r| Some(r.n_nodes as f64))).name("n_nodes"));
        p.line(Line::new(series(|r| Some(r.n_edges as f64))).name("n_edges"));
    });
    Plot::new("pi_plot").height(140.0).legend(Legend::default()).show(ui, |p| {
        p.line(Line::new(series(|r| r.pi_normalized)).name("pi_normalized"));
        p.line(Line::new(series(|r| r.interestingness_score)).name("interestingness"));
    });
    Plot::new("entropy_plot").height(140.0).legend(Legend::default()).show(ui, |p| {
        p.line(Line::new(series(|r| r.type_histogram_entropy)).name("type_entropy"));
        p.line(Line::new(series(|r| r.type_histogram_jsd)).name("jsd"));
    });
    Plot::new("geometry_plot").height(140.0).legend(Legend::default()).show(ui, |p| {
        p.line(Line::new(series(|r| r.d_eff)).name("d_eff"));
    });
}

fn gates_tab(ui: &mut egui::Ui, shared: &SharedState) {
    let eval = shared.eval_state.read();
    ui.heading(format!("Regime: {}", eval.current_regime.as_str()));
    ui.add_space(6.0);
    egui::Grid::new("gates_grid").striped(true).show(ui, |ui| {
        ui.strong("Gate");
        ui.strong("Status");
        ui.end_row();
        for (gate, status) in &eval.gate_statuses {
            ui.label(gate.as_str());
            let color = match status {
                hg_schema::GateStatus::Pass => egui::Color32::GREEN,
                hg_schema::GateStatus::Fail => egui::Color32::RED,
                hg_schema::GateStatus::Pending => egui::Color32::YELLOW,
            };
            ui.colored_label(color, status.as_str());
            ui.end_row();
        }
    });
    ui.add_space(6.0);
    ui.label(format!(
        "Science unlocked: {}",
        if eval.science_unlocked { "yes" } else { "no (controller gate reserved)" }
    ));
    ui.add_space(8.0);
    ui.label("Regime history:");
    ui.horizontal_wrapped(|ui| {
        for r in eval.regime_history.iter().rev().take(40).rev() {
            ui.small(r.as_str());
        }
    });
}

fn kg_tab(ui: &mut egui::Ui, shared: &SharedState) {
    let kg = shared.knowledge_graph.read();
    ui.heading("Task lattice");
    egui::Grid::new("tasks_grid").striped(true).show(ui, |ui| {
        for row in ["id", "input", "output", "status", "certainty", "witnesses"] {
            ui.strong(row);
        }
        ui.end_row();
        for t in &kg.tasks {
            ui.label(t.id.to_string());
            ui.label(format!("{:012x}", t.input_hash));
            ui.label(format!("{:012x}", t.output_hash));
            ui.label(&t.status);
            ui.label(&t.certainty);
            ui.label(t.witness_count.to_string());
            ui.end_row();
        }
    });
    ui.add_space(8.0);
    ui.heading("Constructor ecology");
    egui::Grid::new("ctor_grid").striped(true).show(ui, |ui| {
        for row in ["pattern", "fidelity", "cycles", "replicator", "certainty"] {
            ui.strong(row);
        }
        ui.end_row();
        for c in &kg.constructors {
            ui.label(format!("{:012x}", c.wl_hash));
            ui.label(format!("{:.2}", c.fidelity));
            ui.label(c.confirmed_cycles.to_string());
            ui.label(&c.replicator_kind);
            ui.label(&c.certainty);
            ui.end_row();
        }
    });
    ui.add_space(8.0);
    ui.heading("EI / Causal emergence");
    if kg.ei_results.is_empty() {
        ui.label("No EI results yet (below the emission interval or minimum-transition gate).");
    } else {
        egui::Grid::new("ei_grid").striped(true).show(ui, |ui| {
            for h in ["strategy", "EI_micro", "EI_macro", "CE", "n", "cadence", "M(t)"] {
                ui.strong(h);
            }
            ui.end_row();
            for r in &kg.ei_results {
                ui.label(&r.strategy);
                ui.label(format!("{:.4}", r.ei_micro));
                ui.label(format!("{:.4}", r.ei_macro));
                ui.label(format!("{:.4}", r.ce_coefficient));
                ui.label(r.transition_count.to_string());
                ui.label(format!("{} steps", r.cadence_steps));
                ui.label(r.class_registry_size.to_string());
                ui.end_row();
            }
        });
    }
    ui.add_space(8.0);
    ui.heading(format!("Claims ({})", kg.claims.len()));
    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        for c in kg.claims.iter().take(200) {
            ui.small(format!(
                "[{}] {} {} {} ({})",
                c.step, c.subject, c.predicate, c.object, c.certainty
            ));
        }
    });
}
