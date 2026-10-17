use hg_selfmod::birth::{turnover, Arm};
use std::path::PathBuf;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [out] => turnover::run(&PathBuf::from(out), false),
        [flag, out] if flag == "--preflight" => turnover::run(&PathBuf::from(out), true),
        [flag, fork, birth, arm, mode, out]
            if flag == "--replay" && matches!(mode.as_str(), "on" | "off") =>
        {
            turnover::replay(
                &PathBuf::from(fork),
                &PathBuf::from(birth),
                Arm::parse(arm).expect("unknown genotype"),
                mode == "on",
                &PathBuf::from(out),
            )
        }
        _ => {
            eprintln!("usage: selfmod-turnover [--preflight] FRESH_OUT | --replay FORK BIRTH GENOTYPE on|off FRESH_OUT");
            std::process::exit(2);
        }
    };
    match result {
        Ok(_) => println!("complete"),
        Err(e) => {
            eprintln!("turnover assay failed: {e}");
            std::process::exit(1);
        }
    }
}
