"""Read-only relocatable binding fixture recheck."""
import json
from pathlib import Path
import sys
from compact_evidence import source_inventory,require
from audit_scientific_binding_fixture import audit


def recheck(out,seal):
    source_inventory(out,seal);result=audit(out)
    require(result==json.loads((out/'audit.json').read_text()),'identical independent binding audit')
    m=json.loads((out/'manifest.json').read_text());require(m['status']=='complete' and all(m[k]==v for k,v in result.items()),'binding manifest identity')
    return dict(seal_sha256=seal,**result)


if __name__=='__main__':print(json.dumps(recheck(Path(sys.argv[1]).resolve(),sys.argv[2]),indent=2,sort_keys=True))
