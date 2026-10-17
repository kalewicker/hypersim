//! Explicit launch of a prospectively frozen comparison; never ordinary training.
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command, out, root] if command == "run-prepared" =>
            hg_train::ere::categorical_comparison_bridge::run(std::path::Path::new(out), root),
        [command, out, root] if command == "check-prepared" => {
            hg_train::ere::categorical_comparison_bridge::check_prepared(std::path::Path::new(out), root)
                .map(|receipt| println!("{receipt}"))
        },
        [command, catalogue, request, out] if command=="math-fixture" =>
            hg_train::ere::categorical_comparison_bridge::offline_fixture(std::path::Path::new(catalogue),std::path::Path::new(request),std::path::Path::new(out)),
        _ => Err("usage: ere-categorical-comparison-bridge run-prepared PREPARED_DIRECTORY PREPARED_ROOT_SHA256 | check-prepared PREPARED_DIRECTORY PREPARED_ROOT_SHA256 (zero native work) | math-fixture CATALOGUE REQUEST FRESH_RESULT (zero native work)".into()),
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
