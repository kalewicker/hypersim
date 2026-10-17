use hg_selfmod::birth::bounded::{financing, Policy};
use std::path::PathBuf;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        eprintln!("usage: selfmod-finance FINAL POLICY OFFSET STEPS FRESH_OUT");
        std::process::exit(2);
    }
    match financing::extend(&PathBuf::from(&args[0]), Policy::parse(&args[1]).expect("policy"), args[2].parse().expect("offset"), args[3].parse().expect("steps"), &PathBuf::from(&args[4])) {
        Ok(_) => println!("complete"),
        Err(error) => { eprintln!("extension failed: {error}"); std::process::exit(1); }
    }
}
