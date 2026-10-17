use hg_selfmod::birth::bounded::{third_edit, financing, Policy};
use std::path::PathBuf;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [flag] if flag == "--describe" => Ok(third_edit::describe()),
        [flag, input, founder, out] if flag == "--third" => third_edit::prefix(&PathBuf::from(input), founder.parse().expect("founder"), &PathBuf::from(out)),
        [input, offset, steps, out] => financing::extend(&PathBuf::from(input), Policy::Replace8, offset.parse().expect("offset"), steps.parse().expect("steps"), &PathBuf::from(out)),
        _ => { eprintln!("usage: selfmod-third-edit --describe | --third INITIAL FOUNDER OUT | INITIAL OFFSET STEPS OUT"); std::process::exit(2); }
    };
    match result { Ok(v) => println!("{v}"), Err(e) => { eprintln!("failed: {e}"); std::process::exit(1); } }
}
