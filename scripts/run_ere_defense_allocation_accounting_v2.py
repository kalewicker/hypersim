#!/usr/bin/env python3
"""Separately frozen accounting-only correction of the native-complete allocation archive."""
import argparse
import difflib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_defense_allocation as native_run
ROOT=native_run.ROOT;a=native_run.a
INPUT=ROOT/'output/ere_defense_allocation_20261007'
SEAL='bf034c9962c153f4465d8a831de0ad9107c48613f8b9c8635f2ec98ea04414a1'
EXTRA=['scripts/verify_ere_defense_allocation_v2.py','scripts/test_verify_ere_defense_allocation_v2.py',
    'scripts/run_ere_defense_allocation_accounting_v2.py','docs/ere_defense_allocation_accounting_v2.md']
HELPER="def legacy_projection(raw):\n    assert type(raw['step']) is int and raw['step']==13\n    ids={};ordinary={}\n    for row in raw['nodes']:\n        assert type(row) is list and len(row)==2\n        key=previous.encoder.valid_key(row[0],'NodeId');label=row[1]['data']['label']\n        assert type(label) is int and 0<=label<=2**32-1 and key not in ids\n        ids[key]=label\n        if label<META:\n            assert label not in ordinary;ordinary[label]=key\n    assert sorted(ordinary)==previous.encoder.LABELS\n    positions={ordinary[label]:i for i,label in enumerate(previous.encoder.LABELS)}\n    adjacent=[[False]*7 for _ in range(7)];seen=set()\n    for row in raw['edges']:\n        assert type(row) is list and len(row)==2\n        key=previous.encoder.valid_key(row[0],'EdgeId');assert key not in seen;seen.add(key)\n        nodes=[previous.encoder.endpoint(v) for v in row[1]['nodes']];assert all(n in ids for n in nodes)\n        ords=[positions[n] for n in nodes if n in positions]\n        if not ords:continue\n        assert len(ords)==len(nodes)==2 and ords[0]!=ords[1] and not adjacent[ords[0]][ords[1]]\n        assert type(row[1]['label']) is int and row[1]['label']==0\n        adjacent[ords[0]][ords[1]]=adjacent[ords[1]][ords[0]]=True\n    return dict(labels=previous.encoder.LABELS,adjacency=adjacent)"
def names():return sorted(set(native_run.names())|set(EXTRA))
save=native_run.save
def command(out,name,args,env):
    r=subprocess.run(args,cwd=out,env=env,capture_output=True,text=True,timeout=90)
    save(out/name,dict(command=args,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    assert r.returncode==0,r.stderr+r.stdout
def correction():
    old=(INPUT/'source_snapshots/scripts/verify_ere_defense_allocation.py').read_text()
    fixed=old.replace('\ndef audit(out):','\n# Read-only versioned projection, not a change to the initialization-only encoder.\n'+HELPER+'\n\ndef audit(out):',1)
    fixed=fixed.replace("legacy=[previous.encoder.observe(o['inventory']) for o in observations]","legacy=[legacy_projection(o['inventory']) for o in observations]\n    for index,o in zip(range(0,16,2),legacy):close(o,previous.encoder.observe(a.load(directory/f'cell-{index:04}.json')['initial_inventory']))",1)
    fixed=fixed.replace("task_observations=task_inputs,existing_encoder_observation_alias=legacy_alias,","task_observations=task_inputs,existing_encoder_observation_alias=legacy_alias,existing_encoder_initialization_only=True,",1)
    assert fixed==(ROOT/EXTRA[0]).read_text(),'unregistered correction beyond time-scoped projection'
    return dict(primary_root=SEAL,primary_entries=341,
        original_auditor_sha256=a.digest(INPUT/'source_snapshots/scripts/verify_ere_defense_allocation.py'),
        corrected_auditor_sha256=a.digest(ROOT/EXTRA[0]),diff=''.join(difflib.unified_diff(old.splitlines(True),fixed.splitlines(True),fromfile='frozen-v1',tofile='projection-v2')),
        scope='step13-only projection and initial-view cross-check; no native/criterion/utility changes')
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists()
    assert a.verify(INPUT,expected=SEAL,count=341)==341
    primary=a.load(INPUT/'manifest.json');assert primary['status']=='failed' and primary['error']=='AssertionError()'
    assert primary['native_work_unknown'] is False and primary['native_advances']==512 and primary['nonadvancing_seal_applications']==16
    assert a.load(INPUT/'native/manifest.json')['status']=='complete'
    for n,h in primary['sources'].items():assert a.digest(ROOT/n)==a.digest(INPUT/'source_snapshots'/n)==h
    assert a.digest(INPUT/'native/executed_test_binary')==primary['prelaunch_binary_sha256']
    record=correction();history=native_run.preservation()
    out.mkdir();start=time.time();pins={n:a.digest(ROOT/n) for n in names()}
    for n in pins:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    m=dict(profile='E2-defense-allocation-accounting-correction-v2',status='running',input=INPUT.name,input_root=SEAL,
        input_entries=341,sources=pins,primary_status='failed',primary_native_advances=512,
        native_advances=0,optimizer_updates=0,scientific_acceptance=False,sustained_claim=False,training_admitted=False,independent_physics_replay=False)
    save(out/'manifest.json',m);save(out/'correction.json',record);save(out/'historical_preservation.json',history)
    script=out/'source_snapshots/scripts/verify_ere_defense_allocation_v2.py'
    env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_DEFENSE_ALLOCATION_AUDIT_INPUT=str(INPUT))
    try:
        probe='import sys,json;sys.path.insert(0,'+repr(str(script.parent))+');import verify_ere_defense_allocation_v2 as v;print(json.dumps({n:mod.__file__ for n,mod in sys.modules.items() if n.startswith(("verify_","audit_","ere_")) and hasattr(mod,"__file__")}))'
        command(out,'frozen_import_preflight.json',[sys.executable,'-B','-c',probe],env)
        modules=json.loads(a.load(out/'frozen_import_preflight.json')['stdout'])
        for path in modules.values():
            path=Path(path).resolve();assert path.is_relative_to(out/'source_snapshots')
            n=path.relative_to(out/'source_snapshots').as_posix();assert pins[n]==a.digest(path)
        save(out/'frozen_module_provenance.json',modules)
        query='import sys,json;sys.path.insert(0,'+repr(str(script.parent))+');import verify_ere_defense_allocation_v2 as v;print(json.dumps(v.audit(v.Path('+repr(str(INPUT))+'))))'
        command(out,'verification.json',[sys.executable,'-B','-c',query],env)
        result=json.loads(a.load(out/'verification.json')['stdout']);save(out/'audit.json',result)
        tests=[sys.executable,'-B','-m','unittest','discover','-s',str(script.parent),'-p','test_verify_ere_defense_allocation_v2.py','-v']
        command(out,'test_receipt.json',tests,env)
        command(out,'survival_tests.json',[sys.executable,'-B','-m','unittest','discover','-s',str(script.parent),'-p','test_ere_rule_survival_metrics.py','-v'],env)
        native_run.command(out,'feature_off_check.json',['cargo','check','--offline','--locked','-p','hg-sim-core','--no-default-features'])
        assert a.verify(INPUT,expected=SEAL,count=341)==341
        assert pins=={n:a.digest(ROOT/n) for n in names()} and history==native_run.preservation()
        m.update(status='complete',accounting_pass=True,diagnostic_pass=result['diagnostic_pass'],elapsed_seconds=time.time()-start)
    except Exception as error:
        m.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start);save(out/'manifest.json',m);a.seal(out);raise
    save(out/'manifest.json',m);a.seal(out)
    print(json.dumps(dict(status=m['status'],diagnostic_pass=result['diagnostic_pass'],paid_utilities=result['paid_utilities'],
        adaptive_mean=result['adaptive_mean'],source_pins=len(pins),entries=len(a.load(out/'output_sha256.json')),seal_sha256=a.digest(out/'output_sha256.json')),indent=2))
if __name__=='__main__':main()
