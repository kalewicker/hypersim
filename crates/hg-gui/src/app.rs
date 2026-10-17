//! The workbench app. Mock mode always compiles (hg-app-api only); the
//! "live" feature adds a real simulation backend plus experiment loading
//! (builtin dropdown + TOML file picker).

use crate::canvas::{ColorMode, GraphView};
use crate::tabs::TabRegistry;
use eframe::egui;
use hg_app_api::{MockSimController, SimCommand, SimController, SimStatusEvent};

#[cfg_attr(feature = "live", allow(dead_code))] // mock kept selectable for tests
enum Backend {
    Mock(Box<MockSimController>),
    #[cfg(feature = "live")]
    Live(Box<hg_sim_core::LiveSimController>),
}

impl Backend {
    fn controller(&mut self) -> &mut dyn SimController {
        match self {
            Backend::Mock(c) => c.as_mut(),
            #[cfg(feature = "live")]
            Backend::Live(c) => c.as_mut(),
        }
    }
}

pub struct HypersimApp {
    backend: Backend,
    tabs: TabRegistry,
    color_mode: ColorMode,
    status_lines: Vec<String>,
    started: bool,
    graph_view: GraphView,
    /// Last load/parse/run error, shown in a dismissable red bar. Loading
    /// a bad config must never crash the GUI.
    last_error: Option<String>,
    /// Speed multiplier vs SPEED_BASE_HZ; None = uncapped ("Max").
    speed_mult: Option<f32>,
    #[cfg(feature = "live")]
    headless: crate::headless::HeadlessRunManager,
    #[cfg(feature = "live")]
    selected_builtin: usize,
    #[cfg(feature = "live")]
    builtin_seed: u64,
    #[cfg(feature = "live")]
    builtin_node_count: usize,
}

impl HypersimApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(feature = "live")]
        let backend = Backend::Live(Box::new(hg_sim_core::LiveSimController::new(default_live_config())));
        #[cfg(not(feature = "live"))]
        let backend = Backend::Mock(Box::new(MockSimController::new()));

        Self {
            backend,
            tabs: TabRegistry::default(),
            color_mode: ColorMode::Label,
            status_lines: Vec::new(),
            started: false,
            graph_view: GraphView::default(),
            last_error: None,
            speed_mult: None,
            #[cfg(feature = "live")]
            headless: crate::headless::HeadlessRunManager::default(),
            #[cfg(feature = "live")]
            selected_builtin: 0,
            #[cfg(feature = "live")]
            builtin_seed: 42,
            #[cfg(feature = "live")]
            builtin_node_count: 30,
        }
    }

    /// Swap in a fresh controller for `config` and start it — the single
    /// start path for the default run, the builtin dropdown, and the TOML
    /// picker. Dropping the old controller seals any still-open manifest
    /// (Interrupted) before the new run begins.
    #[cfg(feature = "live")]
    fn start_experiment(&mut self, config: hg_sim_core::SimConfig) {
        self.backend = Backend::Live(Box::new(hg_sim_core::LiveSimController::new(config)));
        self.backend.controller().start();
        self.apply_speed(); // fresh controller = fresh RunControl (uncapped)
        self.started = true;
        self.status_lines.clear();
        self.graph_view.reset();
    }

    /// Push the current speed selection to the active controller.
    fn apply_speed(&mut self) {
        let steps_per_sec = self
            .speed_mult
            .map_or(f32::INFINITY, |m| m * SPEED_BASE_HZ);
        self.backend
            .controller()
            .send_command(SimCommand::SetSpeed(steps_per_sec));
    }
}

/// Steps/sec at speed ×1. The multiplier presets scale this.
const SPEED_BASE_HZ: f32 = 60.0;
const SPEED_CHOICES: [(&str, Option<f32>); 6] = [
    ("×¼ (15/s)", Some(0.25)),
    ("×½ (30/s)", Some(0.5)),
    ("×1 (60/s)", Some(1.0)),
    ("×2 (120/s)", Some(2.0)),
    ("×4 (240/s)", Some(4.0)),
    ("Max (uncapped)", None),
];

#[cfg(feature = "live")]
fn default_live_config() -> hg_sim_core::SimConfig {
    hg_sim_core::SimConfig::from_toml(&builtin_config_toml("binary_edge", 40, 42))
        .expect("default live config parses")
}

/// Per-builtin preset configs, mirroring configs/example_*.toml so GUI
/// runs behave like the documented demos. `node_count` only applies to
/// the WhiteNoise-seeded presets.
#[cfg(feature = "live")]
pub fn builtin_config_toml(name: &str, node_count: usize, seed: u64) -> String {
    match name {
        // Crystal seed, per configs/example_crystal.toml: subdivision
        // match enumeration compounds superlinearly with graph growth, so
        // both the pairing budget AND the step count stay small (the
        // example config runs 96 steps for this reason; 4096 steps of
        // triangle_subdivide burns CPU-hours and never finishes usefully).
        "triangle_subdivide" | "tetrahedral" => format!(
            r#"
experiment_name = "gui_{name}"
seed = {seed}

[rules]
source = "Builtin"
builtin_name = "{name}"

[initial]
condition_type = "PureCrystal"
width = 6
height = 6
arity = 3

[engine]
max_steps = 96
metrics_window_interval = 12
max_pairings_per_step = 4
thread_count = 4
"#
        ),
        // Glider: a purpose-built ring track (overlapping 3-edges, one
        // mark) under SequentialRandom — one hop per step. A generic
        // substrate has no track and the mark dead-ends immediately.
        "glider" => format!(
            r#"
experiment_name = "gui_{name}"
seed = {seed}

[rules]
source = "Builtin"
builtin_name = "{name}"

[initial]
condition_type = "GliderRing"
node_count = {node_count}

[engine]
step_mode = "SequentialRandom"
max_steps = 4096
metrics_window_interval = 512
max_pairings_per_step = 1
thread_count = 1
"#
        ),
        // Growth treadmill: same white-noise seed, but the PLAN-DEFAULT
        // window (512). The treadmill/monoculture boundary tolerates
        // new_type_bin_rate <= 0.01 per step, while a growing hub mints
        // ~1 fresh WL bin per window regardless of window length — short
        // windows inflate the per-step rate and make the regime flap.
        "growth_treadmill" => format!(
            r#"
experiment_name = "gui_{name}"
seed = {seed}

[rules]
source = "Builtin"
builtin_name = "{name}"

[initial]
condition_type = "WhiteNoise"
node_count = {node_count}
edge_count = {edge_count}
max_arity = 2
label_range = 3

[engine]
max_steps = 4096
metrics_window_interval = 512
max_pairings_per_step = 12
thread_count = 4
"#,
            edge_count = node_count * 4 / 3,
        ),
        // White-noise seed, per configs/example_binary_edge.toml. The
        // builtin rule labels are all 0..2, so label_range = 3 covers
        // every LHS.
        _ => format!(
            r#"
experiment_name = "gui_{name}"
seed = {seed}

[rules]
source = "Builtin"
builtin_name = "{name}"

[initial]
condition_type = "WhiteNoise"
node_count = {node_count}
edge_count = {edge_count}
max_arity = 2
label_range = 3

[engine]
max_steps = 4096
metrics_window_interval = 128
max_pairings_per_step = 12
thread_count = 4
"#,
            edge_count = node_count * 4 / 3,
        ),
    }
}

#[cfg(feature = "live")]
fn load_config_from_path(path: &std::path::Path) -> Result<hg_sim_core::SimConfig, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    hg_sim_core::SimConfig::from_toml(&content).map_err(|e| e.to_string())
}

impl eframe::App for HypersimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain events (lossy stream nudges redraws; shared state is
        // authoritative).
        while let Some(_ev) = self.backend.controller().try_recv_event() {}
        while let Some(status) = self.backend.controller().try_recv_status() {
            if let SimStatusEvent::ErrorOccurred { message } = &status {
                self.last_error = Some(message.clone());
            }
            self.status_lines.push(format!("{status:?}"));
        }

        #[cfg(feature = "live")]
        self.headless.poll();

        // Experiment loading is gated off while a run is live (§2.4 of the
        // GUI-improvements doc): never start a second run on top of one.
        let run_busy = matches!(
            self.backend.controller().shared_state().progress.run_state_u8(),
            1 | 2 // Running | Paused
        );

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Run", |ui| {
                    if ui.button("Start").clicked() {
                        self.backend.controller().start();
                        self.apply_speed();
                        self.started = true;
                    }
                    if ui.button("Pause").clicked() {
                        self.backend.controller().send_command(SimCommand::Pause);
                    }
                    if ui.button("Resume").clicked() {
                        self.backend.controller().send_command(SimCommand::Resume);
                    }
                    ui.menu_button("Speed", |ui| {
                        for (label, mult) in SPEED_CHOICES {
                            if ui
                                .selectable_label(self.speed_mult == mult, label)
                                .clicked()
                            {
                                self.speed_mult = mult;
                                self.apply_speed(); // takes effect at the next step
                                ui.close_menu();
                            }
                        }
                    });
                    if ui.button("Quit").clicked() {
                        self.backend.controller().send_command(SimCommand::Quit);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Experiment", |ui| {
                    self.experiment_menu(ui, run_busy);
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Fit to view (F)").clicked() {
                        self.graph_view.request_fit();
                        ui.close_menu();
                    }
                    ui.checkbox(
                        &mut self.graph_view.emphasize_edges,
                        "Emphasize edges",
                    );
                });
                ui.menu_button("Color", |ui| {
                    if ui.button("L — by label").clicked() {
                        self.color_mode = ColorMode::Label;
                    }
                    if ui.button("E — by energy (blue→red)").clicked() {
                        self.color_mode = ColorMode::Energy;
                    }
                    if ui.button("G — by generation").clicked() {
                        self.color_mode = ColorMode::Generation;
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let shared = self.backend.controller().shared_state();
                    let state = hg_output_state_name(shared.progress.run_state_u8());
                    ui.label(format!(
                        "{} — step {}/{} @ {:.0} hz",
                        state,
                        shared.progress.step(),
                        shared.progress.max_steps(),
                        shared.progress.sim_hz()
                    ));
                    #[cfg(feature = "live")]
                    ui.label("live");
                    #[cfg(not(feature = "live"))]
                    ui.label("mock");
                });
            });
        });

        if let Some(err) = self.last_error.clone() {
            egui::TopBottomPanel::top("error_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("✕").clicked() {
                        self.last_error = None;
                    }
                    ui.colored_label(egui::Color32::from_rgb(255, 95, 95), err);
                });
            });
        }

        let shared = self.backend.controller().shared_state();

        egui::SidePanel::right("dashboard")
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                self.tabs.show(ui, &shared);
            });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            match self.status_lines.last() {
                Some(line) => ui.small(line.as_str()),
                None => ui.small("no status events"),
            };
        });

        #[cfg(feature = "live")]
        if !self.headless.is_empty() {
            egui::TopBottomPanel::bottom("headless_runs").show(ctx, |ui| {
                self.headless_panel(ui);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let graph = shared.compact_graph.read().clone();
            self.graph_view.show(ui, &graph, self.color_mode);
        });

        #[cfg(feature = "live")]
        let background_active = self.headless.any_active();
        #[cfg(not(feature = "live"))]
        let background_active = false;
        if self.started || background_active {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

impl HypersimApp {
    #[cfg(feature = "live")]
    fn experiment_menu(&mut self, ui: &mut egui::Ui, run_busy: bool) {
        ui.add_enabled_ui(!run_busy, |ui| {
            ui.label("Builtin rule set:");
            for i in 0..hg_sim_core::BUILTIN_NAMES.len() {
                let name = hg_sim_core::BUILTIN_NAMES[i];
                if ui.selectable_label(self.selected_builtin == i, name).clicked() {
                    self.selected_builtin = i;
                    let toml = builtin_config_toml(name, self.builtin_node_count, self.builtin_seed);
                    match hg_sim_core::SimConfig::from_toml(&toml) {
                        Ok(config) => self.start_experiment(config),
                        Err(e) => self.last_error = Some(format!("builtin preset: {e}")),
                    }
                    ui.close_menu();
                }
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("seed");
                ui.add(egui::DragValue::new(&mut self.builtin_seed));
            });
            ui.horizontal(|ui| {
                ui.label("nodes");
                ui.add(egui::DragValue::new(&mut self.builtin_node_count).range(10..=2000));
            });
            ui.separator();
            if ui.button("Load config…").clicked() {
                ui.close_menu();
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("TOML config", &["toml"])
                    .pick_file()
                {
                    match load_config_from_path(&path) {
                        Ok(config) => self.start_experiment(config),
                        Err(e) => {
                            self.last_error =
                                Some(format!("Failed to load {}: {e}", path.display()));
                        }
                    }
                }
            }
        });
        if run_busy {
            ui.small("run in progress — Quit it (or let it finish) first");
        }

        // Headless launches are never gated on the live run: they get
        // their own thread, SharedState, and RunControl.
        ui.separator();
        ui.label("Headless (background):");
        let selected_name = hg_sim_core::BUILTIN_NAMES[self.selected_builtin];
        if ui
            .button(format!("Run \"{selected_name}\" headless"))
            .clicked()
        {
            ui.close_menu();
            let toml =
                builtin_config_toml(selected_name, self.builtin_node_count, self.builtin_seed);
            match hg_sim_core::SimConfig::from_toml(&toml) {
                Ok(config) => self.headless.launch(selected_name.to_string(), config),
                Err(e) => self.last_error = Some(format!("builtin preset: {e}")),
            }
        }
        if ui.button("Run config headless…").clicked() {
            ui.close_menu();
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("TOML config", &["toml"])
                .pick_file()
            {
                match load_config_from_path(&path) {
                    Ok(config) => {
                        let label = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "config".into());
                        self.headless.launch(label, config);
                    }
                    Err(e) => {
                        self.last_error = Some(format!("Failed to load {}: {e}", path.display()));
                    }
                }
            }
        }
    }

    /// One row per background run: label, progress, state, stop/dismiss.
    #[cfg(feature = "live")]
    fn headless_panel(&mut self, ui: &mut egui::Ui) {
        let mut dismiss: Option<usize> = None;
        for (i, run) in self.headless.runs.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.strong(&run.label);
                ui.small(&run.run_id);
                match &run.outcome {
                    None => {
                        let step = run.shared.progress.step();
                        let max = run.shared.progress.max_steps().max(1);
                        ui.add(
                            egui::ProgressBar::new(step as f32 / max as f32)
                                .desired_width(180.0)
                                .text(format!("{step}/{max}")),
                        );
                        ui.label(hg_output_state_name(run.shared.progress.run_state_u8()));
                        if ui.button("Stop").clicked() {
                            run.control.request_stop();
                        }
                    }
                    Some(Ok(line)) => {
                        ui.colored_label(egui::Color32::from_rgb(120, 220, 120), line);
                        if ui.button("✕").clicked() {
                            dismiss = Some(i);
                        }
                    }
                    Some(Err(e)) => {
                        ui.colored_label(egui::Color32::from_rgb(255, 95, 95), e);
                        if ui.button("✕").clicked() {
                            dismiss = Some(i);
                        }
                    }
                }
            });
        }
        if let Some(i) = dismiss {
            self.headless.dismiss(i);
        }
    }

    #[cfg(not(feature = "live"))]
    fn experiment_menu(&mut self, ui: &mut egui::Ui, _run_busy: bool) {
        ui.add_enabled_ui(false, |ui| {
            let _ = ui.button("Load config…");
        });
        ui.small("experiment loading requires the \"live\" build");
    }
}

/// Decodes the RunState discriminant published through SimProgress
/// without depending on hg-output (values per hg-output::RunState::as_u8).
fn hg_output_state_name(v: u8) -> &'static str {
    match v {
        1 => "Running",
        2 => "Paused",
        3 => "Interrupted",
        4 => "Completed",
        _ => "Created",
    }
}
