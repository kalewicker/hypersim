"""Bounded complete scientific archive verification, with no native execution."""
import hashlib
import json
from pathlib import Path
import sys
from compact_evidence import read_bounded,require,safe_name
from scientific_launch_binding import parse,sha,CONTRACT
from audit_scientific_campaign import audit


def recheck(out,seal):
    raw=read_bounded(out/'output_sha256.json',64*1024**2);require(sha(raw)==seal,'known scientific archive seal')
    inventory=parse(raw);require(len(inventory)<=500000 and 'output_sha256.json' not in inventory,'scientific inventory denominator')
    actual=set();total=0
    for p in out.rglob('*'):
        require(not p.is_symlink(),'no symlink evidence')
        if p.is_file():actual.add(str(p.relative_to(out)))
    require(actual==set(inventory)|{'output_sha256.json'},'complete scientific archive inventory')
    for name,wanted in inventory.items():
        safe_name(name);p=out/name;total+=p.stat().st_size
        require(p.stat().st_size<=64*1024**2 and total<=CONTRACT['limits']['storage_bytes'],'scientific evidence byte ceilings')
        h=hashlib.sha256()
        with p.open('rb') as f:
            for chunk in iter(lambda:f.read(1048576),b''):h.update(chunk)
        require(h.hexdigest()==wanted,'exact scientific payload hash')
    result=audit(out);require(result==parse((out/'audit.json').read_bytes()),'identical independent scientific audit')
    m=parse((out/'manifest.json').read_bytes());require(m['status']=='complete' and all(m[k]==v for k,v in result.items()),'scientific run status authority')
    return dict(seal_sha256=seal,**result)


if __name__=='__main__':print(json.dumps(recheck(Path(sys.argv[1]).resolve(),sys.argv[2]),indent=2,sort_keys=True))
