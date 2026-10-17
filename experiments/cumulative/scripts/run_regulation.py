"""Freeze the bounded editor repertoire; preserve previous roots and failures."""
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path
from preservation_helpers import dependency_addition,strip_journal_main
from run_readiness import ROOT,PACKAGE,FALSE,digest,write,source_paths

HISTORICAL={
    "capacity":("cumulative_readiness_20261009","8c753671f1bca20029ca8d1f83142d42d7d5eac2027a37366264c5afaf4fa791"),
    "shared":("cumulative_shared_population_readiness_20261009","885748da9a20e93a5ef251352540dd5c3280df5932e04745cc664ff46bcd4c2f"),
    "lineage":("cumulative_lineage_intervention_readiness_20261009","d5b6609015c64792fad1159fd4d93d45b8dc697e5329a16e22243c52602cb34f"),
    "failed_development":("cumulative_lineage_development_unfunded_20261009","7c7692d7f0db52264658070550cf72d4922fbc5e0d755c6d8536c9b74b119add")}

def historical_custody():
    for role,(name,root) in HISTORICAL.items():
        package=ROOT/"output"/name
        if digest(package/"output_sha256.json")!=root:raise ValueError("historical root "+role)
        seal=json.loads((package/"output_sha256.json").read_text())
        actual={str(p.relative_to(package)) for p in package.rglob("*") if p.is_file() and p!=package/"output_sha256.json"}
        if actual!=set(seal):raise ValueError("historical file set "+role)
        for rel,h in seal.items():
            p=Path(rel)
            if p.is_absolute() or ".." in p.parts or digest(package/p)!=h:raise ValueError("historical custody "+role+":"+rel)
        manifest=json.loads((package/"manifest.json").read_text())
        if manifest["status"]!=("failed" if role=="failed_development" else "complete"):raise ValueError("historical status "+role)

def run(out):
    start=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    contract=json.loads((PACKAGE/"regulation_contract.json").read_text());limits=contract["limits"]
    manifest={"profile":contract["profile"],"scope":contract["scope"],"status":"running",
        "engineering_fixture_pass":False,"native_work_unknown":True,**FALSE}
    write(out/"manifest.json",manifest);native_seconds=0;test_work=0
    def command(name,argv,native=False):
        nonlocal native_seconds
        remaining=limits["build_test_audit_seconds"]-(time.monotonic()-start)
        if native:remaining=min(remaining,limits["native_seconds"]-native_seconds)
        if remaining<=0:raise RuntimeError("aggregate deadline")
        before=time.monotonic()
        try:
            result=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,timeout=remaining,env={**os.environ,"PYTHONDONTWRITEBYTECODE":"1"})
        except subprocess.TimeoutExpired as e:
            write(out/(name+".json"),{"argv":argv,"timed_out":True,"stdout":str(e.stdout or ""),"stderr":str(e.stderr or "")});raise
        elapsed=time.monotonic()-before
        if native:native_seconds+=elapsed
        write(out/(name+".json"),{"argv":argv,"exit_code":result.returncode,"seconds":elapsed,"stdout":result.stdout,"stderr":result.stderr})
        print(json.dumps({"phase":name,"exit_code":result.returncode}),flush=True)
        if result.returncode:raise RuntimeError(name+" failed: "+result.stderr[-2000:])
        return result
    try:
        historical_custody()
        parent=ROOT/"output"/HISTORICAL["lineage"][0]
        old_pins=json.loads((parent/"source_sha256.json").read_text())
        allowed={"experiments/cumulative/README.md","experiments/cumulative/src/main.rs",
            "experiments/cumulative/src/population.rs","experiments/cumulative/scripts/run_lineage.py"}
        allowed.update({"experiments/cumulative/Cargo.toml","experiments/cumulative/Cargo.lock","experiments/cumulative/scripts/run_population.py","experiments/cumulative/scripts/run_lineage.py","experiments/cumulative/scripts/run_regulation.py"})
        changed=[rel for rel,h in old_pins.items() if digest(ROOT/rel)!=h]
        if not set(changed)<=allowed:raise RuntimeError("unexpected parent source changes "+str(changed))
        dependency_addition(PACKAGE,parent)
        old=(parent/"source_snapshots/experiments/cumulative/src/population.rs").read_bytes()
        if (PACKAGE/"src/population.rs").read_bytes().replace(b"pub mod regulation;\n",b"",1)!=old:raise RuntimeError("population body changed")
        old_main=(parent/"source_snapshots/experiments/cumulative/src/main.rs").read_bytes()
        new_main=strip_journal_main((PACKAGE/"src/main.rs").read_bytes()).replace(b'            "regulation-fixture",\n',b'',1).replace(b'        } else if command == "regulation-fixture" {\n            hg_cumulative_readiness::population::regulation::fixture()?\n',b'',1)
        new_main=new_main.replace(b'            "integration-fixture",\n',b'',1).replace(b'        } else if command == "integration-fixture" {\n            hg_cumulative_readiness::population::regulation::world::fixture()?\n',b'',1)
        if new_main!=old_main:raise RuntimeError("original dispatch changed")
        pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/"source_sha256.json",pins)
        for rel in pins:
            destination=out/"source_snapshots"/rel;destination.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/rel,destination)
        write(out/"contract.json",contract)
        write(out/"historical_roots.json",{"roots":HISTORICAL,"allowed_additive_source_changes":changed,
            "original_cell_population_and_dispatch_bodies_unchanged":True,"earlier_archives_are_separate_immutable_evidence":True})
        command("format",["cargo","fmt","--manifest-path",str(PACKAGE/"Cargo.toml"),"--check"])
        for name,count in [("readiness",7),("population",6),("lineage",5),("regulation",6)]:
            r=command("rust_"+name,["cargo","test","--offline","--locked","--manifest-path",str(PACKAGE/"Cargo.toml"),"--test",name,"--","--nocapture","--test-threads=1"],native=True)
            counters=[int(x) for x in re.findall(r"native_test_rewrites=(\d+)",r.stdout)]
            if len(counters)!=count:raise RuntimeError("missing test work counters "+name)
            test_work+=sum(counters)
            if test_work>limits["native_rewrites"]:raise RuntimeError("aggregate native work cap")
        command("build",["cargo","build","--offline","--locked","--manifest-path",str(PACKAGE/"Cargo.toml"),"--bin","cumulative-readiness"])
        binary=out/"cumulative-readiness";shutil.copy2(PACKAGE/"target/debug/cumulative-readiness",binary)
        command("native",[str(binary),"regulation-fixture",str(out/"fixture.json")],native=True)
        frozen=out/"source_snapshots/experiments/cumulative/scripts"
        r=command("independent_audit",[sys.executable,"-B",str(frozen/"audit_regulation.py"),str(out/"fixture.json"),str(out/"contract.json")])
        audit=json.loads(r.stdout);write(out/"audit.json",audit)
        command("regulation_auditor_tests",[sys.executable,"-B",str(frozen/"test_regulation_audit.py"),str(out/"fixture.json"),str(out/"contract.json")])
        for name,script in [("capacity","test_audit.py"),("shared","test_population_audit.py")]:
            package=ROOT/"output"/HISTORICAL[name][0]
            command(name+"_auditor_regression",[sys.executable,"-B",str(frozen/script),str(package/"fixture.json"),str(package/"contract.json")])
        command("lineage_auditor_regression",[sys.executable,"-B",str(frozen/"test_lineage_audit.py"),str(parent)])
        if {rel:digest(ROOT/rel) for rel in pins}!=pins:raise RuntimeError("source changed during execution")
        historical_custody();total=test_work+audit["native_rewrites"]
        if total>limits["native_rewrites"]:raise RuntimeError("aggregate native work cap")
        write(out/"preservation.json",{"all_pinned_sources_unchanged":True,"source_files":len(pins),"all_four_historical_roots_unchanged":True})
        manifest.update(status="complete",engineering_fixture_pass=True,native_work_unknown=False,
            source_files=len(pins),binary_sha256=digest(binary),native_rewrites=total,test_native_rewrites=test_work,
            fixture_native_rewrites=audit["native_rewrites"],native_seconds=native_seconds,elapsed_seconds=time.monotonic()-start,
            rust_test_groups=24,python_test_groups=34,cases=len(audit["cases"]),native_families=audit["native_families"],
            independent_policy_binding_and_budget_audit=True,independent_rng_replay=False,independent_allocator_replay=False,
            shared_population_and_lineage_integration_complete=False)
        write(out/"manifest.json",manifest)
        size=sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
        if size>limits["output_bytes"]:raise RuntimeError("output cap")
    except Exception as e:
        manifest.update(status="failed",engineering_fixture_pass=False,error=str(e),test_native_rewrites=test_work,elapsed_seconds=time.monotonic()-start)
        write(out/"manifest.json",manifest)
        write(out/"output_sha256.json",{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file() and p!=out/"output_sha256.json"});raise
    seal={str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file()};write(out/"output_sha256.json",seal)
    print(json.dumps({"output":str(out),"seal_sha256":digest(out/"output_sha256.json"),"sealed_entries":len(seal),
        "source_files":len(pins),"native_rewrites":total,"payload_bytes":size,"elapsed_seconds":manifest["elapsed_seconds"],"engineering_fixture_pass":True,**FALSE},indent=2))

if __name__=="__main__":
    if len(sys.argv)!=2:raise SystemExit("usage: run_regulation.py NEW_OUTPUT_DIRECTORY")
    run(Path(sys.argv[1]).resolve())
