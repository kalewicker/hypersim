use hg_selfmod::birth::bounded::{durability,financing,Policy};
use std::path::PathBuf;
fn main() {
    let args:Vec<_>=std::env::args().skip(1).collect();
    let result=match args.as_slice() {
        [flag,seed,out] if flag=="--prefix" => durability::prefix(seed.parse().expect("seed"),&PathBuf::from(out)),
        [input,policy,offset,steps,out] => financing::extend(&PathBuf::from(input),Policy::parse(policy).expect("policy"),offset.parse().expect("offset"),steps.parse().expect("steps"),&PathBuf::from(out)),
        _ => {eprintln!("usage: selfmod-durability --prefix SEED OUT | INITIAL POLICY OFFSET STEPS OUT");std::process::exit(2);}
    };
    match result {Ok(value)=>println!("{value}"),Err(e)=>{eprintln!("failed: {e}");std::process::exit(1);}}
}
