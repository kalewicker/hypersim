import json
from pathlib import Path
import sys
from compact_evidence import source_inventory, sha, require
from audit_scientific_admission import audit


def recheck(package,seal):
    source_inventory(package,seal);pins=json.loads((package/'source_sha256.json').read_text())
    require(all(sha((package/'source_snapshots'/n).read_bytes())==h for n,h in pins.items()),'frozen source pins')
    result=audit(package);require(result==json.loads((package/'audit.json').read_text()),'saved independent audit')
    m=json.loads((package/'manifest.json').read_text());require(m['status']=='complete' and all(m[k]==v for k,v in result.items()),'manifest/audit identity')
    return dict(seal_sha256=seal,source_files=len(pins),**result)


if __name__=='__main__':print(json.dumps(recheck(Path(sys.argv[1]),sys.argv[2]),indent=2,sort_keys=True))
