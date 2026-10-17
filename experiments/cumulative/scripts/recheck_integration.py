"""Known-root frozen integration audit and tests; zero native work."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path
def recheck(package,root):
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    def custody():
        if sha(package/"output_sha256.json")!=root:raise ValueError("known-root integration seal")
        seal=json.loads((package/"output_sha256.json").read_text());actual={str(p.relative_to(package)) for p in package.rglob("*") if p.is_file() and p!=package/"output_sha256.json"}
        if actual!=set(seal):raise ValueError("integration archive file set")
        for rel,h in seal.items():
            p=Path(rel)
            if p.is_absolute() or ".." in p.parts or sha(package/p)!=h:raise ValueError("integration custody "+rel)
        m=json.loads((package/"manifest.json").read_text())
        if m["status"]!="complete" or not m["engineering_fixture_pass"] or not m["shared_population_and_lineage_integration_complete"]:raise ValueError("integration engineering status")
        if any(m[f] is not False for f in ["readiness_gate_complete","scientific_acceptance","evolution_demonstrated","campaign_launch_authorized"]):raise ValueError("integration admission")
        frozen=json.loads((package/"request_freeze.json").read_text())
        if frozen["before_native_assays"] is not True or frozen["outcomes_feed_back"] is not False or any(sha(package/name)!=h for name,h in frozen["sha256"].items()):raise ValueError("measurement request freeze")
        return seal
    seal=custody();scripts=package/"source_snapshots/experiments/cumulative/scripts"
    for script in ["audit_versions.py","test_versions_audit.py"]:
        r=subprocess.run([sys.executable,"-B",str(scripts/script),str(package)],capture_output=True,text=True,timeout=60)
        if r.returncode:raise ValueError(r.stderr)
        if script=="audit_versions.py" and json.loads(r.stdout)!=json.loads((package/"audit.json").read_text()):raise ValueError("frozen integration audit differs")
    custody();return {"verified_entries":len(seal),"frozen_audit_and_tests_pass":True,"native_rewrites":0,"seal_sha256":root}
if __name__=="__main__":
    if len(sys.argv)!=3:raise SystemExit("usage: recheck_integration.py ARCHIVE EXPECTED_SEAL_SHA256")
    print(json.dumps(recheck(Path(sys.argv[1]).resolve(),sys.argv[2]),indent=2))
