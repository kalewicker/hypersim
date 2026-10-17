"""Known-root, zero-native audit of the complete frozen regulation package."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

def recheck(package,root):
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    def custody():
        if sha(package/"output_sha256.json")!=root:raise ValueError("known-root regulation seal")
        seal=json.loads((package/"output_sha256.json").read_text())
        actual={str(p.relative_to(package)) for p in package.rglob("*") if p.is_file() and p!=package/"output_sha256.json"}
        if actual!=set(seal):raise ValueError("regulation archive file set")
        for rel,h in seal.items():
            p=Path(rel)
            if p.is_absolute() or ".." in p.parts or sha(package/p)!=h:raise ValueError("regulation custody "+rel)
        manifest=json.loads((package/"manifest.json").read_text())
        if manifest["status"]!="complete" or not manifest["engineering_fixture_pass"]:raise ValueError("incomplete regulation engineering")
        if any(manifest[k] is not False for k in ["readiness_gate_complete","scientific_acceptance","evolution_demonstrated","campaign_launch_authorized"]):raise ValueError("regulation admission")
        return seal
    seal=custody();frozen=package/"source_snapshots/experiments/cumulative/scripts"
    for script in ["audit_regulation.py","test_regulation_audit.py"]:
        result=subprocess.run([sys.executable,"-B",str(frozen/script),str(package/"fixture.json"),str(package/"contract.json")],capture_output=True,text=True,timeout=60)
        if result.returncode:raise ValueError(result.stderr)
        if script=="audit_regulation.py" and json.loads(result.stdout)!=json.loads((package/"audit.json").read_text()):raise ValueError("frozen regulation audit differs")
    custody()
    return {"verified_entries":len(seal),"frozen_audit_and_tests_pass":True,"native_rewrites":0,"seal_sha256":root}

if __name__=="__main__":
    if len(sys.argv)!=3:raise SystemExit("usage: recheck_regulation.py ARCHIVE EXPECTED_SEAL_SHA256")
    print(json.dumps(recheck(Path(sys.argv[1]).resolve(),sys.argv[2]),indent=2))
