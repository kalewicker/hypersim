//! hg-gui: the Hypersim workbench.
//!
//! Mock-data shell by default (hg-app-api only); live simulation behind
//! the "live" Cargo feature.

pub mod app;
pub mod camera;
pub mod canvas;
#[cfg(feature = "live")]
pub mod headless;
pub mod layout;
pub mod tabs;

pub use app::HypersimApp;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Hypersim",
        options,
        Box::new(|cc| Ok(Box::new(HypersimApp::new(cc)))),
    )
    .map_err(|e| -> Box<dyn std::error::Error> { format!("eframe: {e}").into() })
}
