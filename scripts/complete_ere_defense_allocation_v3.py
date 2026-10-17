#!/usr/bin/env python3
"""Recheck sealed corrected accounting and pin two historical regression inputs."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_defense_allocation as native_run
ROOT=native_run.ROOT;a=native_run.a
PRIMARY=ROOT/'output/ere_defense_allocation_20261007'
PRIMARY_SEAL='bf034c9962c153f4465d8a831de0ad9107c48613f8b9c8635f2ec98ea04414a1'
CORRECTION=ROOT/'output/ere_defense_allocation_accounting_v2_20261007'
CORRECTION_SEAL='b8ddf8f15f4cb899c1b340031c3180143b3e5967855ecc462b16f385c0e32739'
LIFE=ROOT/'output/ere_rule_lifecycle_fixture_development_20261003'
LIFE_SEAL='0cdba8b56ae0d490c87f0ea6d6172fbce1ed27c81c18f24b37d9840e398b8bd5'
save=native_run.save
def command(out,name,args,env,cwd=None):
    r=subprocess.run(args,cwd=cwd or out,env=env,capture_output=True,text=True,timeout=90)
    save(out/name,dict(command=args,cwd=str(cwd or out),returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    assert r.returncode==0,r.stderr+r.stdout
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists()
    assert a.verify(PRIMARY,expected=PRIMARY_SEAL,count=341)==341
    assert a.verify(CORRECTION,expected=CORRECTION_SEAL,count=323)==323
    m=a.load(CORRECTION/'manifest.json');assert m['status']=='failed'
    assert a.load(CORRECTION/'test_receipt.json')['returncode']==0 and a.load(CORRECTION/'survival_tests.json')['returncode']!=0
    for n,h in m['sources'].items():assert a.digest(ROOT/n)==a.digest(CORRECTION/'source_snapshots'/n)==h
    historical=native_run.previous.primitive.prior.prior.historical.historical_verify(LIFE,LIFE_SEAL,187)
    assert a.digest(ROOT/'output/ere_rule_survival_measurement_20261007/lifecycle_source_seal.json')==LIFE_SEAL
    history=native_run.preservation();out.mkdir();start=time.time()
    wrapper_pins={}
    for n in ['scripts/complete_ere_defense_allocation_v3.py','docs/ere_defense_allocation_completion_v3.md']:
        wrapper_pins[n]=a.digest(ROOT/n);dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    seal=a.load(LIFE/'output_sha256.json');references={}
    for arm in ['live_paid','live_replacement']:
        name=f'native/993748713473-{arm}.json';assert a.digest(LIFE/name)==seal[name]
        dest=out/'output'/LIFE.name/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(LIFE/name,dest)
        references[str(dest.relative_to(out))]=seal[name]
    shutil.copyfile(LIFE/'output_sha256.json',out/'lifecycle_source_seal.json')
    save(out/'historical_inputs.json',dict(lifecycle_root=LIFE_SEAL,entries=187,registered_history=historical,files=references))
    save(out/'historical_preservation.json',history)
    result=dict(profile='E2-defense-allocation-completion-v3',status='running',primary_root=PRIMARY_SEAL,primary_entries=341,
        correction_root=CORRECTION_SEAL,correction_entries=323,frozen_source_pins=len(m['sources']),sources=wrapper_pins,
        native_advances=0,optimizer_updates=0,scientific_acceptance=False,sustained_claim=False,training_admitted=False,independent_physics_replay=False)
    save(out/'manifest.json',result)
    scripts=CORRECTION/'source_snapshots/scripts';env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_DEFENSE_ALLOCATION_AUDIT_INPUT=str(PRIMARY))
    try:
        query='import sys,json;sys.path.insert(0,'+repr(str(scripts))+');import verify_ere_defense_allocation_v2 as v;r=v.audit(v.Path('+repr(str(PRIMARY))+'));v.close(r,v.a.load(v.Path('+repr(str(CORRECTION/'audit.json'))+')));print(json.dumps(r))'
        command(out,'verification.json',[sys.executable,'-B','-c',query],env)
        audit=json.loads(a.load(out/'verification.json')['stdout']);save(out/'audit.json',audit)
        command(out,'test_receipt.json',[sys.executable,'-B','-m','unittest','discover','-s',str(scripts),'-p','test_verify_ere_defense_allocation_v2.py','-v'],env)
        command(out,'survival_tests.json',[sys.executable,'-B','-m','unittest','discover','-s',str(scripts),'-p','test_ere_rule_survival_metrics.py','-v'],env)
        command(out,'feature_off_check.json',['cargo','check','--offline','--locked','-p','hg-sim-core','--no-default-features'],env,cwd=ROOT)
        assert a.verify(PRIMARY,expected=PRIMARY_SEAL,count=341)==341 and a.verify(CORRECTION,expected=CORRECTION_SEAL,count=323)==323
        assert history==native_run.preservation()
        for n,h in m['sources'].items():assert a.digest(ROOT/n)==h
        for n,h in wrapper_pins.items():assert a.digest(ROOT/n)==h
        for n,h in references.items():assert a.digest(out/n)==h
        result.update(status='complete',accounting_pass=True,diagnostic_pass=audit['diagnostic_pass'],elapsed_seconds=time.time()-start)
    except Exception as error:
        result.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start);save(out/'manifest.json',result);a.seal(out);raise
    save(out/'manifest.json',result);a.seal(out)
    print(json.dumps(dict(status='complete',diagnostic_pass=audit['diagnostic_pass'],paid_utilities=audit['paid_utilities'],
        entries=len(a.load(out/'output_sha256.json')),seal_sha256=a.digest(out/'output_sha256.json')),indent=2))
if __name__=='__main__':main()
