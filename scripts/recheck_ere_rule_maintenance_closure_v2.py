#!/usr/bin/env python3
"""Frozen accounting with a separately sealed, historically byte-pinned dependency supplement."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import audit_ere_complete_admission as a

PRIMARY='4e1c6f359586c969d1c740527c7fe2ddcc4bfc99bd82b0cbdee91c9977db13ea'
HISTORY='ere_complete_long_gates_full_20260927'
HISTORY_SEAL='32bc8eb713b67d71569a94e8dc6ed6000784c3c89afd00ad67a583c6b9edf5c5'
HELPERS={'scripts/verify_ere_complete_long_gates.py':'023254fec9877d4154c5f257f831dfaa7e730110cf4c7fe90f80cdd4bf2b0798',
    'scripts/verify_ere_complete_gates.py':'92678b21663ae29d2710ac972670362dd2ee1ea8e225145973eb9a380172859a'}

def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',required=True,type=Path);p.add_argument('--output',required=True,type=Path);args=p.parse_args()
    root=Path(__file__).resolve().parents[1];source=args.input.resolve();out=args.output.resolve()
    assert source.is_relative_to(root/'output') and out.is_relative_to(root/'output') and not out.exists()
    assert a.verify(source,expected=PRIMARY,count=380)==380
    m=a.load(source/'manifest.json');assert m['status']=='complete' and m['accounting_pass'] is True
    pins=m['sources']
    for name,h in pins.items():assert a.digest(source/'source_snapshots'/name)==h
    assert a.digest(source/'native/executed_test_binary')==m['prelaunch_binary_sha256']==m['executed_binary_sha256']
    history=root/'output'/HISTORY;assert a.digest(history/'output_sha256.json')==HISTORY_SEAL
    old=a.load(history/'output_sha256.json')
    for n,h in HELPERS.items():
        assert old['source_snapshots/'+n]==h and a.digest(history/'source_snapshots'/n)==h
        assert a.digest(root/n)==h,'dependency changed since historical freeze'
        assert n not in pins,'only omitted dependencies may be supplemented'
    out.mkdir();start=time.time()
    for n in HELPERS:
        dest=out/'dependency_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(history/'source_snapshots'/n,dest)
    shutil.copyfile(Path(__file__),out/'recheck_script.py')
    shutil.copyfile(source/'output_sha256.json',out/'primary_source_seal.json')
    shutil.copyfile(history/'output_sha256.json',out/'historical_helper_source_seal.json')
    env=dict(os.environ,PYTHONPATH=str(out/'dependency_snapshots/scripts'),PYTHONDONTWRITEBYTECODE='1',
        ERE_RULE_MAINTENANCE_PREFLIGHT=str(source/'native_preflight'),ERE_RULE_MAINTENANCE_AUDIT_INPUT=str(source))
    script=source/'source_snapshots/scripts/verify_ere_rule_maintenance.py'
    command=[sys.executable,'-B',str(script),str(source),'--seal',PRIMARY,'--entries','380']
    r=subprocess.run(command,cwd=source,env=env,capture_output=True,text=True,timeout=120)
    save(out/'verification.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    tests=[sys.executable,'-B','-m','unittest','discover','-s',str(script.parent),'-p','test_verify_ere_rule_maintenance.py','-v']
    t=subprocess.run(tests,cwd=source,env=env,capture_output=True,text=True,timeout=120)
    save(out/'tamper_tests.json',dict(command=tests,returncode=t.returncode,stdout=t.stdout,stderr=t.stderr))
    # Inventory imported project modules, then refuse any live/unpinned source path.
    code='import sys,json; sys.path.insert(0,'+repr(str(script.parent))+'); import verify_ere_rule_maintenance; print(json.dumps({n:mod.__file__ for n,mod in sys.modules.items() if n.startswith(("verify_","audit_","run_","ere_")) and hasattr(mod,"__file__")}))'
    q=subprocess.run([sys.executable,'-B','-c',code],cwd=source,env=env,capture_output=True,text=True,timeout=30)
    assert q.returncode==0,q.stderr
    modules=json.loads(q.stdout);origins={}
    for module,path in modules.items():
        path=Path(path).resolve()
        if path.is_relative_to(source/'source_snapshots'):
            n=path.relative_to(source/'source_snapshots').as_posix();assert a.digest(path)==pins[n];origin='primary_snapshot'
        else:
            assert path.is_relative_to(out/'dependency_snapshots'),'live source import refused'
            n=path.relative_to(out/'dependency_snapshots').as_posix();assert a.digest(path)==HELPERS[n];origin='historically_pinned_supplement'
        origins[module]=dict(path=str(path),origin=origin,sha256=a.digest(path))
    assert {'verify_ere_complete_long_gates','verify_ere_complete_gates'}<=origins.keys()
    save(out/'module_provenance.json',origins)
    assert a.verify(source,expected=PRIMARY,count=380)==380
    result=dict(profile='E2-rule-maintenance-audit-closure-v2',status='complete' if r.returncode==t.returncode==0 else 'failed',
        primary_archive=source.name,primary_root_seal=PRIMARY,primary_entries=380,primary_unchanged=True,
        primary_source_pins=len(pins),supplemented_dependencies=HELPERS,historical_helper_archive=HISTORY,
        historical_helper_root=HISTORY_SEAL,historical_validation='root seal and selected helper entries only, not full archive re-audit',
        frozen_primary_auditor_unchanged=True,criteria_unchanged=True,all_imports_frozen=True,modules_verified=len(origins),
        native_advances=0,optimizer_updates=0,scientific_acceptance=False,sustained_claim=False,training_admitted=False,
        independent_physics_replay=False,elapsed_seconds=time.time()-start)
    save(out/'manifest.json',result);a.seal(out)
    assert r.returncode==t.returncode==0,r.stderr+t.stderr
    print(json.dumps(dict(result,seal_sha256=a.digest(out/'output_sha256.json'),entries=len(a.load(out/'output_sha256.json'))),indent=2))

if __name__=='__main__':main()
