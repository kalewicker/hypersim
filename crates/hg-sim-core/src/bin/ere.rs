//! Explicit ERE experiment entry point; no implicit training campaigns.
use hg_sim_core::ere::experiments;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = experiments::cli(&args[1..]);
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
