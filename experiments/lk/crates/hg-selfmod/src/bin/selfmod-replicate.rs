use hg_selfmod::birth::bounded::{self, replication, Policy};
use std::path::PathBuf;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [out] => replication::run(&PathBuf::from(out), false),
        [flag, out] if flag == "--preflight" => replication::run(&PathBuf::from(out), true),
        [flag, seed, out] if flag == "--verify-prefix" => {
            replication::verify_prefix(seed.parse().expect("invalid seed"), &PathBuf::from(out))
        }
        [flag, birth, policy, out] if flag == "--replay" => bounded::replay(
            &PathBuf::from(birth),
            Policy::parse(policy).expect("unknown policy"),
            &PathBuf::from(out),
        ),
        _ => {
            eprintln!("usage: selfmod-replicate [--preflight] FRESH_OUT | --verify-prefix SEED FRESH_OUT | --replay BIRTH POLICY FRESH_OUT");
            std::process::exit(2);
        }
    };
    match result {
        Ok(_) => println!("complete"),
        Err(e) => {
            eprintln!("replication failed: {e}");
            std::process::exit(1);
        }
    }
}
