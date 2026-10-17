#!/usr/bin/env python3
"""Archive-only frozen-auditor recheck; no native execution or live-source substitution."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import time
import audit_ere_complete_admission as a

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',required=True,type=Path);p.add_argument('--seal',required=True)
    p.add_argument('--entries',type=int,required=True);p.add_argument('--output',required=True,type=Path);args=p.parse_args()
    root=Path(__file__).resolve().parents[1];source=args.input.resolve();out=args.output.resolve()
    assert source.is_relative_to(root/'output') and out.is_relative_to(root/'output') and not out.exists()
    a.verify(source,expected=args.seal,count=args.entries)
    m=a.load(source/'manifest.json');assert m['status']=='complete' and m['accounting_pass'] is True
    for name,h in m['sources'].items():assert a.digest(source/'source_snapshots'/name)==h
    assert a.digest(source/'native/executed_test_binary')==m['prelaunch_binary_sha256']==m['executed_binary_sha256']
    out.mkdir();start=time.time()
    command=[sys.executable,'-B',str(source/'source_snapshots/scripts/verify_ere_rule_maintenance.py'),str(source),'--seal',args.seal,'--entries',str(args.entries)]
    r=subprocess.run(command,cwd=source,capture_output=True,text=True,timeout=120)
    (out/'verification.json').write_text(json.dumps(dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr),indent=2)+'\n')
    result=dict(status='complete' if r.returncode==0 else 'failed',profile='E2-rule-maintenance-archive-recheck-v1',
        input=source.name,input_root_seal=args.seal,input_entries=args.entries,source_pins=len(m['sources']),frozen_auditor=True,
        native_advances=0,optimizer_updates=0,scientific_acceptance=False,sustained_claim=False,training_admitted=False,elapsed_seconds=time.time()-start)
    (out/'manifest.json').write_text(json.dumps(result,indent=2)+'\n');a.seal(out)
    assert r.returncode==0,r.stderr
    print(json.dumps(dict(result,seal_sha256=a.digest(out/'output_sha256.json')),indent=2))

if __name__=='__main__':main()
