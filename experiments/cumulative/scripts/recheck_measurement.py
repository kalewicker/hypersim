"""Read-only, relocatable verification of the sealed measurement package."""
import json
from pathlib import Path
import sys
from compact_evidence import source_inventory, sha
from audit_campaign_measurement import audit


def recheck(package, seal):
    source_inventory(package,seal)
    pins=json.loads((package/'source_sha256.json').read_text())
    for name,wanted in pins.items():
        if sha((package/'source_snapshots'/name).read_bytes())!=wanted:
            raise ValueError('frozen source hash mismatch')
    result=audit(package)
    if result!=json.loads((package/'audit.json').read_text()):
        raise ValueError('published audit differs')
    manifest=json.loads((package/'manifest.json').read_text())
    if manifest['status']!='complete' or any(manifest[k]!=v for k,v in result.items()):
        raise ValueError('manifest differs')
    return dict(seal_sha256=seal,source_files=len(pins),new_native_rewrites=0,**result)


if __name__=='__main__':
    print(json.dumps(recheck(Path(sys.argv[1]),sys.argv[2]),indent=2,sort_keys=True))
