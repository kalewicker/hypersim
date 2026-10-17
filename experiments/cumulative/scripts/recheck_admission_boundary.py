"""Admission fixture inventory with one exact oversized negative input.

The unchanged generic native-evidence reader caps every file at 32 MiB. This
fixture deliberately submits 32 MiB plus one byte to the admission CLI. Only that
named, all-space negative input gets the extra byte; native evidence limits do not.
This supplemental checker is read-only and never invokes the native binary.
"""
import json
from pathlib import Path
import sys
from compact_evidence import read_bounded, safe_name, sha, require, MAX_RECORD, MAX_INDEX, MAX_ENTRIES, MAX_FIXTURE
from audit_scientific_admission import audit


def recheck(package,seal):
    raw=read_bounded(package/'output_sha256.json',MAX_INDEX);require(sha(raw)==seal,'known admission seal')
    inventory=json.loads(raw);require('output_sha256.json' not in inventory and len(inventory)+1<=MAX_ENTRIES,'bounded inventory')
    expected={**inventory,'output_sha256.json':seal};actual=set();total=0
    for p in package.rglob('*'):
        require(not p.is_symlink(),'no symlink evidence')
        if p.is_file():actual.add(str(p.relative_to(package)))
    require(actual==set(expected),'complete admission inventory')
    special='requests/encoded_byte_cap.json'
    require(special in inventory,'declared oversized negative case')
    for name,wanted in expected.items():
        safe_name(name);data=read_bounded(package/name,MAX_RECORD+1 if name==special else MAX_RECORD)
        if name==special:require(len(data)==MAX_RECORD+1 and data==b' '*(MAX_RECORD+1),'exact oversized whitespace fixture only')
        require(sha(data)==wanted,'exact payload hash');total+=len(data)
    require(total<=MAX_FIXTURE,'total fixture cap')
    pins=json.loads((package/'source_sha256.json').read_text())
    require(all(sha((package/'source_snapshots'/n).read_bytes())==h for n,h in pins.items()),'frozen source identity')
    result=audit(package);require(result==json.loads((package/'audit.json').read_text()),'sealed audit identity')
    m=json.loads((package/'manifest.json').read_text());require(m['status']=='complete' and all(m[k]==v for k,v in result.items()),'manifest identity')
    return dict(seal_sha256=seal,source_files=len(pins),oversized_negative_input_only=special,
                supplemental_checker_sha256=sha(Path(__file__).read_bytes()),**result)


if __name__=='__main__':print(json.dumps(recheck(Path(sys.argv[1]),sys.argv[2]),indent=2,sort_keys=True))
