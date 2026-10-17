use std::path::PathBuf;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (preflight, out) = match args.as_slice() {
        [out] => (false, PathBuf::from(out)),
        [flag, out] if flag == "--preflight" => (true, PathBuf::from(out)),
        _ => {
            eprintln!("usage: selfmod-spectrum [--preflight] FRESH_OUTPUT_DIR");
            std::process::exit(2);
        }
    };
    match hg_selfmod::spectrum::run(&out, preflight) {
        Ok(summary) => println!(
            "complete: {} cells, {} advancing steps",
            summary["mutation_cells"], summary["actual_advancing_steps"]
        ),
        Err(error) => {
            eprintln!("assay failed: {error}");
            std::process::exit(1);
        }
    }
}
