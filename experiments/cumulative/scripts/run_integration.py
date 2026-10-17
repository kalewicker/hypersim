"""Freeze bounded shared regulation and version measurement; retain every failure."""
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
from run_regulation import HISTORICAL,historical_custody
PARENT=ROOT/"output/cumulative_editor_regulation_readiness_20261009"
PARENT_ROOT="e98c55e2b9e4d53a3b0fdb9542b3dfe36dd4d23f7cca14c838a327fb0604a550"

def preservation():
    historical_custody()
    if digest(PARENT/"output_sha256.json")!=PARENT_ROOT:raise ValueError("editor parent root")
    seal=json.loads((PARENT/"output_sha256.json").read_text())
    actual={str(p.relative_to(PARENT)) for p in PARENT.rglob("*") if p.is_file() and p!=PARENT/"output_sha256.json"}
    if actual!=set(seal):raise ValueError("editor parent file set")
    for rel,h in seal.items():
        p=Path(rel)
        if p.is_absolute() or ".." in p.parts or digest(PARENT/p)!=h:raise ValueError("editor parent custody "+rel)
    m=json.loads((PARENT/"manifest.json").read_text())
    if m["status"]!="complete" or not m["engineering_fixture_pass"] or any(m[f] is not False for f in FALSE):raise ValueError("editor parent status")

def run(out):
    start=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    contract=json.loads((PACKAGE/"integration_contract.json").read_text());limits=contract["limits"]
    manifest={"profile":contract["profile"],"scope":contract["scope"],"status":"running","engineering_fixture_pass":False,"native_work_unknown":True,**FALSE};write(out/"manifest.json",manifest)
    test_work=0;native_seconds=0
    def command(name,argv,native=False):
        nonlocal native_seconds
        remaining=limits["build_test_audit_seconds"]-(time.monotonic()-start)
        if native:remaining=min(remaining,limits["native_seconds"]-native_seconds)
        if remaining<=0:raise RuntimeError("aggregate deadline")
        before=time.monotonic()
        try:r=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,timeout=remaining,env={**os.environ,"PYTHONDONTWRITEBYTECODE":"1"})
        except subprocess.TimeoutExpired as e:
            write(out/(name+".json"),{"argv":argv,"timed_out":True,"stdout":str(e.stdout or ""),"stderr":str(e.stderr or "")});raise
        elapsed=time.monotonic()-before
        if native:native_seconds+=elapsed
        write(out/(name+".json"),{"argv":argv,"exit_code":r.returncode,"seconds":elapsed,"stdout":r.stdout,"stderr":r.stderr});print(json.dumps({"phase":name,"exit_code":r.returncode}),flush=True)
        if r.returncode:raise RuntimeError(name+" failed: "+r.stderr[-2000:])
        return r
    try:
        preservation();old_pins=json.loads((PARENT/"source_sha256.json").read_text())
        allowed={"experiments/cumulative/README.md","experiments/cumulative/src/main.rs","experiments/cumulative/src/population/regulation.rs","experiments/cumulative/scripts/run_regulation.py"}
        allowed.update({"experiments/cumulative/Cargo.toml","experiments/cumulative/Cargo.lock","experiments/cumulative/scripts/run_population.py","experiments/cumulative/scripts/run_lineage.py","experiments/cumulative/scripts/run_regulation.py"})
        changed=[rel for rel,h in old_pins.items() if digest(ROOT/rel)!=h]
        if not set(changed)<=allowed:raise RuntimeError("unexpected parent source change "+str(changed))
        dependency_addition(PACKAGE,PARENT)
        old=(PARENT/"source_snapshots/experiments/cumulative/src/population/regulation.rs").read_bytes()
        if (PACKAGE/"src/population/regulation.rs").read_bytes().replace(b"pub mod world;\n",b"",1)!=old:raise RuntimeError("editor kernel body changed")
        old_main=(PARENT/"source_snapshots/experiments/cumulative/src/main.rs").read_bytes()
        main=strip_journal_main((PACKAGE/"src/main.rs").read_bytes()).replace(b'            "integration-fixture",\n',b'',1).replace(b'        } else if command == "integration-fixture" {\n            hg_cumulative_readiness::population::regulation::world::fixture()?\n',b'',1)
        if main!=old_main:raise RuntimeError("original dispatch changed")
        pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/"source_sha256.json",pins)
        for rel in pins:
            target=out/"source_snapshots"/rel;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/rel,target)
        write(out/"contract.json",contract);write(out/"historical_roots.json",{"roots":HISTORICAL,"editor":PARENT_ROOT,"allowed_additive_source_changes":changed,"original_cell_population_lineage_and_editor_bodies_unchanged":True})
        command("format",["cargo","fmt","--manifest-path",str(PACKAGE/"Cargo.toml"),"--check"])
        for name,count in [("readiness",7),("population",6),("lineage",5),("regulation",6),("integration",6)]:
            r=command("rust_"+name,["cargo","test","--offline","--locked","--manifest-path",str(PACKAGE/"Cargo.toml"),"--test",name,"--","--nocapture","--test-threads=1"],native=True)
            counters=[int(x) for x in re.findall(r"native_test_rewrites=(\d+)",r.stdout)]
            if len(counters)!=count:raise RuntimeError("native test counters "+name)
            test_work+=sum(counters)
            if test_work>limits["native_rewrites"]:raise RuntimeError("aggregate test native work cap")
        command("build",["cargo","build","--offline","--locked","--manifest-path",str(PACKAGE/"Cargo.toml"),"--bin","cumulative-readiness"])
        binary=out/"cumulative-readiness";shutil.copy2(PACKAGE/"target/debug/cumulative-readiness",binary)
        command("native_worlds",[str(binary),"integration-fixture",str(out/"fixture.json")],native=True)
        frozen=out/"source_snapshots/experiments/cumulative/scripts"
        command("plan",[sys.executable,"-B",str(frozen/"plan_versions.py"),str(out/"fixture.json"),str(out/"contract.json"),str(out/"measurement.json"),str(out/"requests.json")])
        fixed={name:digest(out/name) for name in ["fixture.json","contract.json","measurement.json","requests.json"]}
        write(out/"request_freeze.json",{"sha256":fixed,"before_native_assays":True,"source_is_development":True,"scientific_prospective_sampling":False,"outcomes_feed_back":False})
        command("native_assays",[str(binary),"lineage-assays",str(out/"requests.json"),str(out/"assays.json")],native=True)
        if {name:digest(out/name) for name in fixed}!=fixed:raise RuntimeError("frozen measurement changed after assays")
        command("aggregate",[sys.executable,"-B",str(frozen/"plan_versions.py"),"aggregate",str(out/"measurement.json"),str(out/"assays.json"),str(out/"report.json")])
        r=command("independent_audit",[sys.executable,"-B",str(frozen/"audit_versions.py"),str(out)]);audit=json.loads(r.stdout);write(out/"audit.json",audit)
        command("version_auditor_tests",[sys.executable,"-B",str(frozen/"test_versions_audit.py"),str(out)])
        command("regulation_auditor_regression",[sys.executable,"-B",str(frozen/"test_regulation_audit.py"),str(PARENT/"fixture.json"),str(PARENT/"contract.json")])
        for name,script in [("capacity","test_audit.py"),("shared","test_population_audit.py")]:
            old=ROOT/"output"/HISTORICAL[name][0];command(name+"_auditor_regression",[sys.executable,"-B",str(frozen/script),str(old/"fixture.json"),str(old/"contract.json")])
        old=ROOT/"output"/HISTORICAL["lineage"][0];command("lineage_auditor_regression",[sys.executable,"-B",str(frozen/"test_lineage_audit.py"),str(old)])
        if {rel:digest(ROOT/rel) for rel in pins}!=pins:raise RuntimeError("source changed during execution")
        preservation();total=test_work+audit["native_rewrites"]
        if total>limits["native_rewrites"]:raise RuntimeError("aggregate native work cap")
        write(out/"preservation.json",{"all_pinned_sources_unchanged":True,"all_five_historical_roots_unchanged":True,"source_files":len(pins)})
        manifest.update(status="complete",engineering_fixture_pass=True,native_work_unknown=False,binary_sha256=digest(binary),source_files=len(pins),native_rewrites=total,test_native_rewrites=test_work,fixture_native_rewrites=audit["native_rewrites"],world_native_rewrites=audit["world_native_rewrites"],assay_native_rewrites=audit["assay_native_rewrites"],native_seconds=native_seconds,elapsed_seconds=time.monotonic()-start,rust_test_groups=30,python_test_groups=44,samples=audit["samples"],assays=audit["assays"],shared_population_and_lineage_integration_complete=True,independent_rng_replay=False,independent_allocator_replay=False,durable_campaign_routes_complete=False)
        write(out/"manifest.json",manifest);size=sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
        if size>limits["output_bytes"]:raise RuntimeError("output cap")
    except Exception as e:
        manifest.update(status="failed",engineering_fixture_pass=False,error=str(e),test_native_rewrites=test_work,elapsed_seconds=time.monotonic()-start);write(out/"manifest.json",manifest)
        write(out/"output_sha256.json",{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file() and p!=out/"output_sha256.json"});raise
    seal={str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file()};write(out/"output_sha256.json",seal)
    print(json.dumps({"output":str(out),"seal_sha256":digest(out/"output_sha256.json"),"sealed_entries":len(seal),"source_files":len(pins),"native_rewrites":total,"elapsed_seconds":manifest["elapsed_seconds"],"payload_bytes":size,"engineering_fixture_pass":True,**FALSE},indent=2))

if __name__=="__main__":
    if len(sys.argv)!=2:raise SystemExit("usage: run_integration.py NEW_OUTPUT_DIRECTORY")
    run(Path(sys.argv[1]).resolve())
