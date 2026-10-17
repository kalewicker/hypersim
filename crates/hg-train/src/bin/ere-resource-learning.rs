use hg_train::ere::resource_learning as r;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let result=match args.get(1).map(String::as_str) {
        Some("schedule") if args.len()==2=>serde_json::to_writer(std::io::stdout(),&r::schedule()).map_err(|e|e.to_string()),
        Some("campaign") if args.len()==5=> {
            let ack=args[3].parse::<u64>().unwrap_or(0);let bootstrap=args[4].parse::<u64>().unwrap_or(u64::MAX);
            r::campaign(std::path::Path::new(&args[2]),ack,bootstrap).and_then(|m|serde_json::to_writer(std::io::stdout(),&m).map_err(|e|e.to_string()))
        },
        _=>Err("usage: ere-resource-learning schedule | campaign OUTPUT ACK_NATIVE_ADVANCES BOOTSTRAP_BYTES".into()),
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
