use std::io::Write;
fn main() {
    if std::env::args().nth(1).as_deref() == Some("journal-run") {
        if let Err(e) = hg_cumulative_readiness::population::regulation::world::journal::cli() {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    let result = (|| -> Result<(), String> {
        let mut args = std::env::args().skip(1);
        let command = args.next().ok_or("missing fixture command")?;
        if ![
            "fixture",
            "population-fixture",
            "lineage-witness",
            "lineage-assays",
            "regulation-fixture",
            "integration-fixture",
        ]
        .contains(&command.as_str())
        {
            return Err("usage: cumulative-readiness COMMAND [REQUESTS_JSON] NEW_JSON".into());
        }
        let requests = if command == "lineage-assays" {
            let input = args.next().ok_or("missing assay request path")?;
            Some(
                serde_json::from_slice::<serde_json::Value>(
                    &std::fs::read(input).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };
        let path = args.next().ok_or("missing fresh output path")?;
        if args.next().is_some() {
            return Err("unexpected argument".into());
        }
        let mut out = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        let value = if command == "fixture" {
            hg_cumulative_readiness::fixture()?
        } else if command == "population-fixture" {
            hg_cumulative_readiness::population::fixture()?
        } else if command == "lineage-witness" {
            hg_cumulative_readiness::lineage::witness()?
        } else if command == "regulation-fixture" {
            hg_cumulative_readiness::population::regulation::fixture()?
        } else if command == "integration-fixture" {
            hg_cumulative_readiness::population::regulation::world::fixture()?
        } else {
            hg_cumulative_readiness::lineage::assays(requests.as_ref().unwrap())?
        };
        serde_json::to_writer_pretty(&mut out, &value).map_err(|e| e.to_string())?;
        out.write_all(b"\n").map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
