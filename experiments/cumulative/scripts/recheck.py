"""Zero-native frozen audit and corruption tests, authenticated by a known root."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(package, expected):
    if sha(package / "output_sha256.json") != expected:
        raise ValueError("known-root seal mismatch")
    seal = json.loads((package / "output_sha256.json").read_text())
    actual = {str(p.relative_to(package)) for p in package.rglob("*") if p.is_file() and p.name != "output_sha256.json"}
    if actual != set(seal):
        raise ValueError("archive file set mismatch")
    for rel, expected_hash in seal.items():
        path = Path(rel)
        if path.is_absolute() or ".." in path.parts or sha(package / path) != expected_hash:
            raise ValueError("archive custody " + rel)
    manifest = json.loads((package / "manifest.json").read_text())
    if manifest["status"] != "complete" or not manifest["engineering_fixture_pass"]:
        raise ValueError("package is not engineering-complete")
    return seal


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: recheck.py ARCHIVE EXPECTED_SEAL_SHA256")
    package = Path(sys.argv[1]).resolve()
    expected = sys.argv[2]
    seal = verify(package, expected)
    frozen = package / "source_snapshots/experiments/cumulative/scripts"
    for script in ["audit_readiness.py", "test_audit.py"]:
        result = subprocess.run([sys.executable,"-B",str(frozen / script),str(package / "fixture.json"),str(package / "contract.json")],
            capture_output=True,text=True,timeout=120)
        if result.returncode:
            raise ValueError(result.stderr)
        if script == "audit_readiness.py" and json.loads(result.stdout) != json.loads((package / "audit.json").read_text()):
            raise ValueError("frozen audit result differs")
    verify(package, expected)
    print(json.dumps({"verified_entries":len(seal),"frozen_audit_and_tests_pass":True,"native_rewrites":0,"seal_sha256":expected},indent=2))
