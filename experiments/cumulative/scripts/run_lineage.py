"""Freeze bounded lineage measurement, fix requests before assays, preserve failures."""
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
from recheck import verify

SHARED=ROOT/"output/cumulative_shared_population_readiness_20261009"
SHARED_ROOT="885748da9a20e93a5ef251352540dd5c3280df5932e04745cc664ff46bcd4c2f"
CAPACITY=ROOT/"output/cumulative_readiness_20261009"
CAPACITY_ROOT="8c753671f1bca20029ca8d1f83142d42d7d5eac2027a37366264c5afaf4fa791"
FAILED=ROOT/"output/cumulative_lineage_development_unfunded_20261009"
FAILED_ROOT="7c7692d7f0db52264658070550cf72d4922fbc5e0d755c6d8536c9b74b119add"


def verify_failed():
    if digest(FAILED/"output_sha256.json")!=FAILED_ROOT:raise ValueError("failed development seal root")
    seal=json.loads((FAILED/"output_sha256.json").read_text())
    actual={str(p.relative_to(FAILED)) for p in FAILED.rglob("*") if p.is_file() and p.name!="output_sha256.json"}
    if actual!=set(seal) or any(digest(FAILED/rel)!=h for rel,h in seal.items()):raise ValueError("failed development custody")
    if json.loads((FAILED/"manifest.json").read_text())["status"]!="failed":raise ValueError("failed development reclassified")


def run(out):
    start=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    contract=json.loads((PACKAGE/"lineage_contract.json").read_text());limits=contract["limits"]
    manifest={"profile":contract["profile"],"status":"running","scope":contract["scope"],
        "engineering_fixture_pass":False,"native_work_unknown":True,**FALSE}
    write(out/"manifest.json",manifest)
    native_seconds=0

    def command(name,argv,native=False):
        nonlocal native_seconds
        remaining=limits["build_test_audit_seconds"]-(time.monotonic()-start)
        if native:remaining=min(remaining,limits["native_seconds"]-native_seconds)
        if remaining<=0:raise RuntimeError("aggregate workflow/native deadline")
        before=time.monotonic()
        try:
            result=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,timeout=remaining,
                env={**os.environ,"PYTHONDONTWRITEBYTECODE":"1"})
        except subprocess.TimeoutExpired as e:
            write(out/(name+".json"),{"argv":argv,"timed_out":True,"stdout":str(e.stdout or ""),"stderr":str(e.stderr or "")});raise
        elapsed=time.monotonic()-before
        if native:native_seconds+=elapsed
        write(out/(name+".json"),{"argv":argv,"exit_code":result.returncode,"seconds":elapsed,"stdout":result.stdout,"stderr":result.stderr})
        print(json.dumps({"phase":name,"exit_code":result.returncode}),flush=True)
        if result.returncode:raise RuntimeError(name+" failed: "+result.stderr[-2000:])
        return result

    try:
        verify(SHARED,SHARED_ROOT);verify(CAPACITY,CAPACITY_ROOT);verify_failed()
        parent_pins=json.loads((SHARED/"source_sha256.json").read_text())
        allowed={"experiments/cumulative/README.md","experiments/cumulative/src/lib.rs",
            "experiments/cumulative/src/main.rs","experiments/cumulative/scripts/run_population.py",
            "experiments/cumulative/src/population.rs"}
        allowed.update({"experiments/cumulative/Cargo.toml","experiments/cumulative/Cargo.lock","experiments/cumulative/scripts/run_population.py","experiments/cumulative/scripts/run_lineage.py","experiments/cumulative/scripts/run_regulation.py"})
        changed=[rel for rel,h in parent_pins.items() if digest(ROOT/rel)!=h]
        if not set(changed)<=allowed:raise RuntimeError("unexpected pinned source change "+str(changed))
        dependency_addition(PACKAGE,SHARED)
        old=(SHARED/"source_snapshots/experiments/cumulative/src/lib.rs").read_bytes()
        if (PACKAGE/"src/lib.rs").read_bytes().replace(b"pub mod lineage;\n",b"",1)!=old:
            raise RuntimeError("original cell or population body changed")
        old_population=(SHARED/"source_snapshots/experiments/cumulative/src/population.rs").read_bytes()
        if (PACKAGE/"src/population.rs").read_bytes().replace(b"pub mod regulation;\n",b"",1)!=old_population:
            raise RuntimeError("original population body changed")
        pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()}
        write(out/"source_sha256.json",pins)
        for rel in pins:
            target=out/"source_snapshots"/rel;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/rel,target)
        write(out/"contract.json",contract)
        inputs=out/"inputs/shared";inputs.mkdir(parents=True)
        for name in ["fixture.json","contract.json","audit.json","output_sha256.json"]:shutil.copy2(SHARED/name,inputs/name)
        write(out/"historical_roots.json",{"shared":SHARED_ROOT,"capacity":CAPACITY_ROOT,"failed_development":FAILED_ROOT,
            "allowed_additive_source_changes":changed,"original_cell_and_population_bodies_unchanged":True})
        shutil.copytree(FAILED,out/"retained_development_failure")
        command("format",["cargo","fmt","--manifest-path",str(PACKAGE/"Cargo.toml"),"--check"])
        test_work=0
        for name,count in [("readiness",7),("population",6),("lineage",5)]:
            r=command("rust_"+name,["cargo","test","--offline","--locked","--manifest-path",str(PACKAGE/"Cargo.toml"),"--test",name,"--","--nocapture","--test-threads=1"])
            counters=[int(x) for x in re.findall(r"native_test_rewrites=(\d+)",r.stdout)]
            if len(counters)!=count:raise RuntimeError("missing native work counters "+name)
            test_work+=sum(counters)
        command("build",["cargo","build","--offline","--locked","--manifest-path",str(PACKAGE/"Cargo.toml"),"--bin","cumulative-readiness"])
        binary=out/"cumulative-readiness";shutil.copy2(PACKAGE/"target/debug/cumulative-readiness",binary)
        frozen=out/"source_snapshots/experiments/cumulative/scripts"
        command("native_witness",[str(binary),"lineage-witness",str(out/"witness.json")],native=True)
        command("plan",[sys.executable,"-B",str(frozen/"plan_lineage.py"),str(inputs/"fixture.json"),str(out/"witness.json"),str(out/"contract.json"),str(out/"measurement.json"),str(out/"requests.json")])
        fixed={name:digest(out/name) for name in ["witness.json","measurement.json","requests.json","contract.json"]}
        write(out/"request_freeze.json",{"sha256":fixed,"before_native_assays":True,
            "source_is_archived_development":True,"scientific_prospective_sampling":False,"outcomes_feed_back":False})
        command("native_assays",[str(binary),"lineage-assays",str(out/"requests.json"),str(out/"assays.json")],native=True)
        if {name:digest(out/name) for name in fixed}!=fixed:raise RuntimeError("request changed after freeze")
        command("aggregate",[sys.executable,"-B",str(frozen/"plan_lineage.py"),"aggregate",str(out/"measurement.json"),str(out/"assays.json"),str(out/"report.json")])
        r=command("independent_audit",[sys.executable,"-B",str(frozen/"audit_lineage.py"),str(out)])
        audit=json.loads(r.stdout);write(out/"audit.json",audit)
        command("lineage_auditor_tests",[sys.executable,"-B",str(frozen/"test_lineage_audit.py"),str(out)])
        command("population_auditor_regression",[sys.executable,"-B",str(frozen/"test_population_audit.py"),str(inputs/"fixture.json"),str(inputs/"contract.json")])
        command("capacity_auditor_regression",[sys.executable,"-B",str(frozen/"test_audit.py"),str(CAPACITY/"fixture.json"),str(CAPACITY/"contract.json")])
        if {rel:digest(ROOT/rel) for rel in pins}!=pins:raise RuntimeError("source changed during execution")
        verify(SHARED,SHARED_ROOT);verify(CAPACITY,CAPACITY_ROOT);verify_failed()
        total=test_work+audit["native_rewrites"]
        if total>limits["native_rewrites"]:raise RuntimeError("aggregate native work cap")
        write(out/"preservation.json",{"all_pinned_sources_unchanged":True,"source_files":len(pins),"all_historical_roots_unchanged":True})
        manifest.update(status="complete",engineering_fixture_pass=True,native_work_unknown=False,binary_sha256=digest(binary),
            source_files=len(pins),native_rewrites=total,test_native_rewrites=test_work,fixture_native_rewrites=audit["native_rewrites"],
            native_seconds=native_seconds,elapsed_seconds=time.monotonic()-start,rust_test_groups=18,python_test_groups=25,
            samples=audit["samples"],assays=audit["assays"],independent_lineage_and_sampling=True,
            independent_rng_replay=False,independent_allocator_replay=False)
        write(out/"manifest.json",manifest)
        size=sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
        if size>limits["output_bytes"]:raise RuntimeError("output cap")
    except Exception as e:
        manifest.update(status="failed",engineering_fixture_pass=False,error=str(e),elapsed_seconds=time.monotonic()-start)
        write(out/"manifest.json",manifest)
        write(out/"output_sha256.json",{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file() and p!=out/"output_sha256.json"})
        raise
    seal={str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file()}
    write(out/"output_sha256.json",seal)
    print(json.dumps({"output":str(out),"seal_sha256":digest(out/"output_sha256.json"),"sealed_entries":len(seal),
        "source_files":len(pins),"native_rewrites":total,"payload_bytes":size,"elapsed_seconds":manifest["elapsed_seconds"],
        "engineering_fixture_pass":True,**FALSE},indent=2))


if __name__=="__main__":
    if len(sys.argv)!=2:raise SystemExit("usage: run_lineage.py NEW_OUTPUT_DIRECTORY")
    run(Path(sys.argv[1]).resolve())
