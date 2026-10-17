use hg_cumulative_readiness::{native_work, population::regulation::world::journal::campaign};
use std::path::Path;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = if args.len() == 2 {
        campaign::run(Path::new(&args[0]), Path::new(&args[1]))
    } else {
        Err("usage: cumulative-campaign DEVELOPMENT_REQUEST FRESH_JOURNAL".into())
    };
    match result {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::json!({"error":error,"native_rewrites":native_work()})
            );
            std::process::exit(1);
        }
    }
}
