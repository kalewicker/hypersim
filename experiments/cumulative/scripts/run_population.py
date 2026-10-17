"""Freeze one bounded shared-population engineering package; retain failures."""
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path
from preservation_helpers import dependency_addition,strip_journal_main
from run_readiness import ROOT, PACKAGE, FALSE, digest, write, source_paths
from recheck import verify

PARENT = ROOT / "output/cumulative_readiness_20261009"
PARENT_ROOT = "8c753671f1bca20029ca8d1f83142d42d7d5eac2027a37366264c5afaf4fa791"


def run(out):
    start = time.monotonic()
    out.mkdir(parents=True, exist_ok=False)
    contract = json.loads((PACKAGE / "population_contract.json").read_text())
    limits = contract["limits"]
    manifest = {"profile":contract["profile"], "status":"running", "scope":contract["scope"],
                "engineering_fixture_pass":False, "native_work_unknown":True, "native_rewrites":0, **FALSE}
    write(out / "manifest.json", manifest)

    def command(name, argv, cap=None):
        remaining = limits["build_test_audit_seconds"] - (time.monotonic()-start)
        if remaining <= 0:
            raise RuntimeError("aggregate workflow deadline")
        before = time.monotonic()
        try:
            result = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True,
                timeout=min(remaining, cap or remaining), env={**os.environ,"PYTHONDONTWRITEBYTECODE":"1"})
        except subprocess.TimeoutExpired as error:
            write(out / (name+".json"), {"argv":argv,"timed_out":True,
                "stdout":str(error.stdout or ""),"stderr":str(error.stderr or ""),"seconds":time.monotonic()-before})
            raise
        write(out / (name+".json"), {"argv":argv,"exit_code":result.returncode,
            "seconds":time.monotonic()-before,"stdout":result.stdout,"stderr":result.stderr})
        print(json.dumps({"phase":name,"exit_code":result.returncode}), flush=True)
        if result.returncode:
            raise RuntimeError(name+" failed: "+result.stderr[-2000:])
        return result

    try:
        verify(PARENT, PARENT_ROOT)
        parent_sources = json.loads((PARENT / "source_sha256.json").read_text())
        allowed = {"experiments/cumulative/README.md","experiments/cumulative/src/lib.rs","experiments/cumulative/src/main.rs"}
        allowed.update({"experiments/cumulative/Cargo.toml","experiments/cumulative/Cargo.lock","experiments/cumulative/scripts/run_population.py","experiments/cumulative/scripts/run_lineage.py","experiments/cumulative/scripts/run_regulation.py"})
        changed = [rel for rel,h in parent_sources.items() if digest(ROOT/rel)!=h]
        if not set(changed) <= allowed:
            raise RuntimeError("unexpected change to parent-pinned bodies: "+str(changed))
        dependency_addition(PACKAGE,PARENT)
        old_lib = (PARENT / "source_snapshots/experiments/cumulative/src/lib.rs").read_bytes()
        new_lib = (PACKAGE / "src/lib.rs").read_bytes()
        if new_lib.replace(b"pub mod lineage;\n", b"", 1).replace(b"pub mod population;\n", b"", 1) != old_lib:
            raise RuntimeError("original cell implementation changed")
        sources = {str(p.relative_to(ROOT)):digest(p) for p in source_paths()}
        write(out / "source_sha256.json", sources)
        for rel in sources:
            target = out / "source_snapshots" / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT/rel, target)
        write(out / "contract.json", contract)
        write(out / "parent_preservation.json", {"parent_archive":str(PARENT),"known_root":PARENT_ROOT,
            "verified_before_execution":True,"parent_pinned_files":len(parent_sources),
            "allowed_additive_source_changes":changed,"original_cell_body_unchanged":True})
        shutil.copy2(PARENT / "output_sha256.json", out / "parent_output_sha256.json")
        manifest["source_files"] = len(sources)
        command("format", ["cargo","fmt","--manifest-path",str(PACKAGE/"Cargo.toml"),"--check"])
        test_work = 0
        for name, count in [("readiness",7),("population",6)]:
            result = command("rust_"+name, ["cargo","test","--offline","--locked","--manifest-path",str(PACKAGE/"Cargo.toml"),
                "--test",name,"--","--nocapture","--test-threads=1"])
            counters = [int(x) for x in re.findall(r"native_test_rewrites=(\d+)",result.stdout)]
            if len(counters)!=count:
                raise RuntimeError("missing test work counters: "+name)
            test_work += sum(counters)
        command("build", ["cargo","build","--offline","--locked","--manifest-path",str(PACKAGE/"Cargo.toml"),"--bin","cumulative-readiness"])
        binary = out / "cumulative-readiness"
        shutil.copy2(PACKAGE/"target/debug/cumulative-readiness", binary)
        manifest["binary_sha256"] = digest(binary)
        write(out / "manifest.json", manifest)
        command("native", [str(binary),"population-fixture",str(out/"fixture.json")], limits["native_seconds"])
        frozen = out / "source_snapshots/experiments/cumulative/scripts"
        result = command("independent_audit", [sys.executable,"-B",str(frozen/"audit_population.py"),str(out/"fixture.json"),str(out/"contract.json")])
        audit = json.loads(result.stdout)
        write(out / "audit.json", audit)
        command("auditor_tests", [sys.executable,"-B",str(frozen/"test_population_audit.py"),str(out/"fixture.json"),str(out/"contract.json")])
        command("parent_auditor_regression", [sys.executable,"-B",str(frozen/"test_audit.py"),str(PARENT/"fixture.json"),str(PARENT/"contract.json")])
        if {rel:digest(ROOT/rel) for rel in sources}!=sources:
            raise RuntimeError("source changed during execution")
        verify(PARENT, PARENT_ROOT)
        write(out / "preservation.json", {"all_pinned_sources_unchanged":True,"source_files":len(sources),
            "parent_archive_unchanged":True,"parent_known_root":PARENT_ROOT})
        total_work = test_work+audit["native_rewrites"]
        if total_work>limits["native_rewrites"]:
            raise RuntimeError("aggregate test and fixture native work cap")
        manifest.update(status="complete",engineering_fixture_pass=True,native_work_unknown=False,
            native_rewrites=total_work,test_native_rewrites=test_work,fixture_native_rewrites=audit["native_rewrites"],
            elapsed_seconds=time.monotonic()-start,rust_test_groups=13,python_test_groups=16,
            scheduler_reconstructed=True,checkpoint_wire_and_indices_checked=True,
            independent_rng_replay=False,independent_allocator_replay=False,
            measurement_boundary="independent integer law, graph binding, scheduler and checkpoint wire audit; paired native continuation")
        write(out / "manifest.json", manifest)
        size = sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
        if size>limits["output_bytes"]:
            raise RuntimeError("output cap")
    except Exception as error:
        manifest.update(status="failed",engineering_fixture_pass=False,error=str(error),elapsed_seconds=time.monotonic()-start)
        write(out / "manifest.json", manifest)
        write(out / "output_sha256.json", {str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file() and p.name!="output_sha256.json"})
        raise
    seal = {str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file()}
    write(out / "output_sha256.json", seal)
    print(json.dumps({"output":str(out),"seal_sha256":digest(out/"output_sha256.json"),"sealed_entries":len(seal),
        "native_rewrites":total_work,"elapsed_seconds":manifest["elapsed_seconds"],"payload_bytes":size,
        "engineering_fixture_pass":True,**FALSE},indent=2))


if __name__=="__main__":
    if len(sys.argv)!=2:
        raise SystemExit("usage: run_population.py NEW_OUTPUT_DIRECTORY")
    run(Path(sys.argv[1]).resolve())
