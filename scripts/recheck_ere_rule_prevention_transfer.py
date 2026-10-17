#!/usr/bin/env python3
"""Self-contained archive-only frozen prevention audit and tests; zero native work."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import os
import time
import audit_ere_complete_admission as a

def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',required=True,type=Path);p.add_argument('--seal',required=True)
    p.add_argument('--entries',required=True,type=int);p.add_argument('--output',required=True,type=Path);args=p.parse_args()
    root=Path(__file__).resolve().parents[1];source=args.input.resolve();out=args.output.resolve()
    assert source.is_relative_to(root/'output') and out.is_relative_to(root/'output') and not out.exists()
    assert a.verify(source,expected=args.seal,count=args.entries)==args.entries
    m=a.load(source/'manifest.json');assert m['status']=='complete' and m['accounting_pass'] is True
    for n,h in m['sources'].items():assert a.digest(source/'source_snapshots'/n)==h
    assert a.digest(source/'native/executed_test_binary')==m['prelaunch_binary_sha256']
    out.mkdir();start=time.time();script=source/'source_snapshots/scripts/verify_ere_rule_prevention_transfer.py'
    env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RULE_PREVENTION_TRANSFER_AUDIT_INPUT=str(source))
    command=[sys.executable,'-B',str(script),str(source),'--seal',args.seal,'--entries',str(args.entries)]
    r=subprocess.run(command,cwd=source,env=env,capture_output=True,text=True,timeout=90)
    save(out/'verification.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    tests=[sys.executable,'-B','-m','unittest','discover','-s',str(script.parent),'-p','test_verify_ere_rule_prevention_transfer.py','-v']
    t=subprocess.run(tests,cwd=source,env=env,capture_output=True,text=True,timeout=90)
    save(out/'test_receipt.json',dict(command=tests,returncode=t.returncode,stdout=t.stdout,stderr=t.stderr))
    assert a.verify(source,expected=args.seal,count=args.entries)==args.entries
    result=dict(profile='E2-rule-prevention-transfer-archive-recheck-v1',status='complete' if r.returncode==t.returncode==0 else 'failed',
        input=source.name,input_root=args.seal,input_entries=args.entries,source_pins=len(m['sources']),frozen_auditor=True,
        native_advances=0,optimizer_updates=0,scientific_acceptance=False,sustained_claim=False,training_admitted=False,
        elapsed_seconds=time.time()-start)
    save(out/'manifest.json',result);a.seal(out);assert r.returncode==t.returncode==0,r.stderr+t.stderr
    print(json.dumps(dict(result,seal_sha256=a.digest(out/'output_sha256.json'),entries=len(a.load(out/'output_sha256.json'))),indent=2))
if __name__=='__main__':main()
