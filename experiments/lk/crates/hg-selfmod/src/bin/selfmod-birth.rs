use hg_selfmod::birth::{self, Arm};
use std::path::PathBuf;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [out] => birth::run(&PathBuf::from(out), false),
        [flag, out] if flag == "--preflight" => birth::run(&PathBuf::from(out), true),
        [flag, fork, arm, out] if flag == "--replay" => birth::replay(
            &PathBuf::from(fork),
            Arm::parse(arm).expect("unknown arm"),
            &PathBuf::from(out),
        ),
        _ => {
            eprintln!("usage: selfmod-birth [--preflight] FRESH_OUT | --replay FORK ARM FRESH_OUT");
            std::process::exit(2);
        }
    };
    match result {
        Ok(_) => println!("complete"),
        Err(e) => {
            eprintln!("assay failed: {e}");
            std::process::exit(1);
        }
    }
}
