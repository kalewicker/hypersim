use hg_sim_core::constructive_chemistry::{fixture::run_contract, sha};
use serde_json::{json, Value};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: constructive-chemistry-cc1 CONTRACT NEW_OUTPUT_JSON".into());
    }
    let bytes = std::fs::read(&args[1])?;
    let contract: Value = serde_json::from_slice(&bytes)?;
    let mut result = run_contract(&contract);
    result["contract_sha256"] = json!(sha(&bytes));
    let exe = std::env::current_exe()?;
    result["binary_sha256"] = json!(sha(&std::fs::read(exe)?));
    let encoded = serde_json::to_vec(&result)?;
    if encoded.len() > 134_217_728 {
        return Err("total output cap".into());
    }
    // create_new prevents accidental replacement of a retained run.
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    file.write_all(&encoded)?;
    file.sync_all()?;
    println!("41 cases / 82 variants retained; {} bytes", encoded.len());
    Ok(())
}
