use hg_selfmod::birth::bounded::{second_edit, durability, financing, Policy};
use std::path::PathBuf;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [flag] if flag == "--describe" => Ok(second_edit::describe_semantic()),
        [flag, seed, out] if flag == "--prefix" => durability::prefix(seed.parse().expect("seed"), &PathBuf::from(out)),
        [flag, input, out] if flag == "--second" => second_edit::second_prefix_semantic(&PathBuf::from(input), &PathBuf::from(out)),
        [flag, input, out] if flag == "--semantic-regression" => second_edit::semantic_regression(&PathBuf::from(input), &PathBuf::from(out)),
        [input, offset, steps, out] => financing::extend(&PathBuf::from(input), Policy::Replace8, offset.parse().expect("offset"), steps.parse().expect("steps"), &PathBuf::from(out)),
        _ => { eprintln!("usage: selfmod-second-edit-semantic --describe | --prefix SEED OUT | --second INITIAL OUT | INITIAL OFFSET STEPS OUT"); std::process::exit(2); }
    };
    match result { Ok(v) => println!("{v}"), Err(e) => { eprintln!("failed: {e}"); std::process::exit(1); } }
}
