use hg_selfmod::birth::bounded::conflict_attribution;
use std::path::Path;
fn main() {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 4 {
        eprintln!("usage: selfmod-conflict-attribution INITIAL OFFSET STEPS OUT");
        std::process::exit(2);
    }
    match conflict_attribution::extend(
        Path::new(&a[0]),
        a[1].parse().expect("offset"),
        a[2].parse().expect("steps"),
        Path::new(&a[3]),
    ) {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!("failed: {e}");
            std::process::exit(1);
        }
    }
}
