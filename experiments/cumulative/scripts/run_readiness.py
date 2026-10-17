"""Freeze and run the bounded development fixture; preserve failed packages."""
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PACKAGE = ROOT / "experiments/cumulative"
FALSE = {k: False for k in ["readiness_gate_complete", "scientific_acceptance", "evolution_demonstrated", "campaign_launch_authorized"]}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def source_paths():
    # Full first-party build closure, including the patched allocator.
    dirs = [PACKAGE] + [ROOT / "experiments/lk/crates" / n for n in
        ["hg-core", "hg-rules", "hg-engine", "hg-observe", "hg-schema", "hg-parallel"]]
    dirs += [ROOT / "experiments/lk/vendor/slotmap"]
    result = {ROOT / "experiments/lk/Cargo.toml", ROOT / "experiments/lk/Cargo.lock"}
    for directory in dirs:
        for p in directory.rglob("*"):
            if p.is_file() and not any(part in ["target", "__pycache__", ".git"] for part in p.relative_to(directory).parts):
                result.add(p)
    return sorted(result)


def run(out):
    start = time.monotonic()
    out.mkdir(parents=True, exist_ok=False)
    contract = json.loads((PACKAGE / "contract.json").read_text())
    limits = contract["limits"]
    sources = {str(p.relative_to(ROOT)): digest(p) for p in source_paths()}
    write(out / "source_sha256.json", sources)
    for rel in sources:
        destination = out / "source_snapshots" / rel
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / rel, destination)
    write(out / "contract.json", contract)
    manifest = {"profile":contract["profile"],"status":"running","scope":contract["scope"],
                "source_files":len(sources),"native_rewrites":0,"engineering_fixture_pass":False,
                "native_work_unknown":True,**FALSE}
    write(out / "manifest.json", manifest)

    def command(name, argv, cap=None):
        remaining = limits["build_test_audit_seconds"] - (time.monotonic() - start)
        if remaining <= 0:
            raise RuntimeError("aggregate workflow deadline")
        before = time.monotonic()
        completed = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True,
            timeout=min(remaining, cap or remaining), env={**os.environ,"PYTHONDONTWRITEBYTECODE":"1"})
        receipt = {"argv":argv,"exit_code":completed.returncode,"seconds":time.monotonic()-before,
                   "stdout":completed.stdout,"stderr":completed.stderr}
        write(out / (name + ".json"), receipt)
        if completed.returncode:
            raise RuntimeError(name + " failed: " + completed.stderr[-2000:])
        return completed

    try:
        command("format", ["cargo","fmt","--manifest-path",str(PACKAGE / "Cargo.toml"),"--check"])
        tests = command("rust_tests", ["cargo","test","--offline","--locked","--manifest-path",str(PACKAGE / "Cargo.toml"),
            "--test","readiness","--","--nocapture","--test-threads=1"])
        counters = [int(x) for x in re.findall(r"native_test_rewrites=(\d+)",tests.stdout)]
        if len(counters) != 7:
            raise RuntimeError("missing native test work counters")
        test_work = sum(counters)
        command("build", ["cargo","build","--offline","--locked","--manifest-path",str(PACKAGE / "Cargo.toml"),"--bin","cumulative-readiness"])
        binary = out / "cumulative-readiness"
        shutil.copy2(PACKAGE / "target/debug/cumulative-readiness", binary)
        manifest["binary_sha256"] = digest(binary)
        write(out / "manifest.json", manifest)
        command("native", [str(binary),"fixture",str(out / "fixture.json")], limits["native_seconds"])
        frozen = out / "source_snapshots/experiments/cumulative/scripts"
        result = command("independent_audit", [sys.executable,"-B",str(frozen / "audit_readiness.py"),str(out / "fixture.json"),str(out / "contract.json")])
        audit = json.loads(result.stdout)
        write(out / "audit.json", audit)
        command("auditor_tests", [sys.executable,"-B",str(frozen / "test_audit.py"),str(out / "fixture.json"),str(out / "contract.json")])
        if {rel:digest(ROOT/rel) for rel in sources} != sources:
            raise RuntimeError("source changed during execution")
        write(out / "preservation.json", {"all_pinned_sources_unchanged":True,"source_files":len(sources)})
        total_work = test_work + audit["native_rewrites"]
        if total_work > limits["native_rewrites"]:
            raise RuntimeError("aggregate test + fixture native work cap")
        manifest.update(status="complete", engineering_fixture_pass=True, native_work_unknown=False,
            native_rewrites=total_work, test_native_rewrites=test_work, fixture_native_rewrites=audit["native_rewrites"],
            elapsed_seconds=time.monotonic()-start, rust_test_groups=7, python_test_groups=8,
            measurement_boundary="independent integer law/binding audit; paired native checkpoint replay; no independent RNG/allocator replay")
        write(out / "manifest.json", manifest)
        size = sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
        if size > limits["output_bytes"]:
            raise RuntimeError("output cap")
    except Exception as e:
        manifest.update(status="failed",engineering_fixture_pass=False,error=str(e),elapsed_seconds=time.monotonic()-start)
        write(out / "manifest.json", manifest)
        write(out / "output_sha256.json", {str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file() and p.name!="output_sha256.json"})
        raise
    seal = {str(p.relative_to(out)):digest(p) for p in sorted(out.rglob("*")) if p.is_file()}
    write(out / "output_sha256.json", seal)
    root = digest(out / "output_sha256.json")
    print(json.dumps({"output":str(out),"seal_sha256":root,"sealed_entries":len(seal),"native_rewrites":total_work,
                      "elapsed_seconds":manifest["elapsed_seconds"],"payload_bytes":size,"engineering_fixture_pass":True,**FALSE},indent=2))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: run_readiness.py NEW_OUTPUT_DIRECTORY")
    run(Path(sys.argv[1]).resolve())
