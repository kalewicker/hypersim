"""Zero-native known-root controller recheck, including relocated evidence."""
import json
from pathlib import Path
import sys
from compact_evidence import source_inventory, sha, require
from audit_campaign_controller import audit


def recheck(package,seal):
    source_inventory(package,seal)
    pins=json.loads((package/'source_sha256.json').read_text())
    require(all(sha((package/'source_snapshots'/n).read_bytes())==h for n,h in pins.items()),'frozen source identity')
    result=audit(package)
    require(result==json.loads((package/'audit.json').read_text()),'published independent result')
    manifest=json.loads((package/'manifest.json').read_text())
    require(manifest['status']=='complete' and all(manifest[k]==v for k,v in result.items()),'manifest matches audit')
    return dict(seal_sha256=seal,source_files=len(pins),**result)


if __name__=='__main__':print(json.dumps(recheck(Path(sys.argv[1]),sys.argv[2]),indent=2,sort_keys=True))
