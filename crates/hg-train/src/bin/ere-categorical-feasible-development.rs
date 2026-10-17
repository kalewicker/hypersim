//! Explicit isolated development dispatch; no ordinary training route.
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command, out, root] if command == "run-prepared" =>
            hg_train::ere::categorical_feasible_development::run(std::path::Path::new(out), root),
        [command, out, root] if command == "check-prepared" =>
            hg_train::ere::categorical_feasible_development::check_prepared(std::path::Path::new(out), root)
                .map(|v| println!("{v}")),
        _ => Err("usage: ere-categorical-feasible-development check-prepared DIRECTORY PREPARED_SHA256 (zero native work) | run-prepared DIRECTORY PREPARED_SHA256 (separate approval required)".into()),
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
