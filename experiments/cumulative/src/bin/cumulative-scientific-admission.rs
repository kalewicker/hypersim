//! Check-only binary: no world constructor, journal writer or execution mode.
#[path = "support/scientific_admission.rs"]
mod admission;
use std::fs;
use std::io::Read;
use std::path::Path;

fn run() -> Result<serde_json::Value, String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || args[0] != "--check" {
        return Err(
            "usage: cumulative-scientific-admission --check REQUEST; execution unavailable".into(),
        );
    }
    let p = Path::new(&args[1]);
    let meta = fs::symlink_metadata(p).map_err(|e| e.to_string())?;
    if !meta.file_type().is_file() || meta.len() > admission::MAX_BYTES as u64 {
        return Err("regular bounded request file required".into());
    }
    let mut raw = Vec::new();
    fs::File::open(p)
        .map_err(|e| e.to_string())?
        .take(admission::MAX_BYTES as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    let request = admission::parse(&raw)?;
    let mut report = admission::validate(&request)?;
    report["request_sha256"] = serde_json::json!(admission::sha(&raw));
    report["native_rewrites"] = serde_json::json!(hg_cumulative_readiness::native_work());
    Ok(report)
}
fn main() {
    match run() {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!(
                "{}",
                serde_json::json!({"error":e,"request_valid":false,"execution_enabled":false,"launch_authorized":false,"native_rewrites":hg_cumulative_readiness::native_work()})
            );
            std::process::exit(1);
        }
    }
}
