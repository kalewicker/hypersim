"""Relocatable, read-only executor archive audit."""
import json
from pathlib import Path
import sys
from compact_evidence import source_inventory, require
from audit_scientific_executor import audit
from test_scientific_executor import run


def recheck(out,seal):
    source_inventory(out,seal)
    result=audit(out)
    require(result==json.loads((out/'audit.json').read_text()),'audit identity')
    require(run(out)==json.loads((out/'auditor_tests.json').read_text()),'mutation test identity')
    m=json.loads((out/'manifest.json').read_text())
    require(m['status']=='complete' and all(m[k]==v for k,v in result.items()),'manifest identity')
    return dict(seal_sha256=seal,**result)


if __name__=='__main__':print(json.dumps(recheck(Path(sys.argv[1]),sys.argv[2]),indent=2,sort_keys=True))
