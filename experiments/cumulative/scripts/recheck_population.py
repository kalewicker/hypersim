"""Authenticate and audit the frozen population package with zero native work."""
import json
import subprocess
import sys
from pathlib import Path
from recheck import verify


def recheck(package, expected):
    seal = verify(package, expected)
    frozen = package / "source_snapshots/experiments/cumulative/scripts"
    for script in ["audit_population.py","test_population_audit.py"]:
        result = subprocess.run([sys.executable,"-B",str(frozen/script),str(package/"fixture.json"),str(package/"contract.json")],
            capture_output=True,text=True,timeout=60)
        if result.returncode:
            raise ValueError(result.stderr)
        if script=="audit_population.py" and json.loads(result.stdout)!=json.loads((package/"audit.json").read_text()):
            raise ValueError("frozen audit result differs")
    verify(package, expected)
    return {"verified_entries":len(seal),"frozen_audit_and_tests_pass":True,"native_rewrites":0,"seal_sha256":expected}


if __name__=="__main__":
    if len(sys.argv)!=3:
        raise SystemExit("usage: recheck_population.py ARCHIVE EXPECTED_SEAL_SHA256")
    print(json.dumps(recheck(Path(sys.argv[1]).resolve(),sys.argv[2]),indent=2))
