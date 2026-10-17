//! Hypersim GUI workbench entry point (live feature enabled).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    hg_gui::run()
}
