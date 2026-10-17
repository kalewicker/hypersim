"""Exact additive dispatch and SHA dependency guards for live historical launchers."""
import tomllib
JOURNAL_PRELUDE=b'''    if std::env::args().nth(1).as_deref() == Some("journal-run") {
        if let Err(e) = hg_cumulative_readiness::population::regulation::world::journal::cli() {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
'''
def strip_journal_main(raw):
    return raw.replace(JOURNAL_PRELUDE,b"",1)

def dependency_addition(package,parent):
    old=parent/"source_snapshots/experiments/cumulative"
    if (package/"Cargo.toml").read_bytes().replace(b'sha2 = "0.10.9"\n',b'',1)!=(old/"Cargo.toml").read_bytes():raise ValueError("only declared SHA dependency addition allowed")
    before=tomllib.loads((old/"Cargo.lock").read_text())["package"];after=tomllib.loads((package/"Cargo.lock").read_text())["package"]
    key=lambda p:(p["name"],p["version"],p.get("source"));a={key(p):p for p in after};b={key(p):p for p in before}
    added=set(a)-set(b)
    if {k[0] for k in added}!={"sha2","digest","crypto-common","block-buffer","generic-array","typenum","cpufeatures"}:raise ValueError("unexpected dependency closure addition")
    for k,p in b.items():
        candidate=a.get(k)
        if k[0]=="hg-cumulative-readiness":
            candidate=dict(candidate);candidate["dependencies"]=[v for v in candidate["dependencies"] if v!="sha2"]
        if candidate!=p:raise ValueError("existing resolved dependency changed "+k[0])
    return sorted(k[0] for k in added)
