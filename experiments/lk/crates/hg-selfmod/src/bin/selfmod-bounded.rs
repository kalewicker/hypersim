use hg_selfmod::birth::bounded::{self, Policy};
use std::path::PathBuf;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [out] => bounded::run(&PathBuf::from(out), false),
        [flag, out] if flag == "--preflight" => bounded::run(&PathBuf::from(out), true),
        [flag, birth, policy, out] if flag == "--replay" => bounded::replay(
            &PathBuf::from(birth),
            Policy::parse(policy).expect("unknown policy"),
            &PathBuf::from(out),
        ),
        _ => {
            eprintln!(
                "usage: selfmod-bounded [--preflight] FRESH_OUT | --replay BIRTH POLICY FRESH_OUT"
            );
            std::process::exit(2);
        }
    };
    match result {
        Ok(_) => println!("complete"),
        Err(e) => {
            eprintln!("bounded assay failed: {e}");
            std::process::exit(1);
        }
    }
}
