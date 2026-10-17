#!/usr/bin/env python3
"""Freeze zero-native resource adapter verification on immutable receipts."""
import argparse
import difflib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import verify_ere_resource_adapter as v

ROOT=Path(__file__).resolve().parents[1]
ARCHIVES=dict(primary='ere_defense_allocation_20261007',correction='ere_defense_allocation_accounting_v2_20261007',
    completion_v3='ere_defense_allocation_completion_v3_20261007',completion_v4='ere_defense_allocation_completion_v4_20261007')
NEW=['configs/ere_resource_adapter_v1.json','docs/ere_resource_adapter_v1.md',
    'crates/hg-train/src/ere/resource_adapter.rs','crates/hg-train/tests/ere_resource_adapter.rs',
    'scripts/verify_ere_resource_adapter.py','scripts/test_verify_ere_resource_adapter.py',
    'scripts/run_ere_resource_adapter.py','scripts/recheck_ere_resource_adapter.py',
    'docs/ere_resource_adapter_accounting_v2.md']
FAILED=ROOT/'output/ere_resource_adapter_20261008'
FAILED_ROOT='a34244da71bc81d656e1979b55ac0fa75ee0ec6cd4689ab0d1135028aec38c38'
def failed_preservation():
    v.verify_seal(FAILED,FAILED_ROOT,681)
    old=v.load(FAILED/'manifest.json');assert old['status']=='failed'
    changed=[n for n,h in old['sources'].items() if v.digest(ROOT/n)!=h]
    assert changed==['scripts/run_ere_resource_adapter.py','scripts/test_verify_ere_resource_adapter.py','scripts/verify_ere_resource_adapter.py'],changed
    name='scripts/verify_ere_resource_adapter.py';frozen=(FAILED/'source_snapshots'/name).read_text();live=(ROOT/name).read_text()
    expected=frozen.replace("r'NodeId((0|[1-9][0-9]*)v([1-9][0-9]*))'", "r'NodeId\\((0|[1-9][0-9]*)v([1-9][0-9]*)\\)'")
    expected=expected.replace('return int(m[2]), int(m[3])','return int(m[1]), int(m[2])')
    expected=expected.replace("r'EdgeId((0|[1-9][0-9]*)v([1-9][0-9]*))'", "r'EdgeId\\((0|[1-9][0-9]*)v([1-9][0-9]*)\\)'")
    assert live==expected,'unregistered verifier change'
    return dict(root=FAILED_ROOT,entries=681,status='failed',changed_helpers=changed,
        verifier_diff=''.join(difflib.unified_diff(frozen.splitlines(True),live.splitlines(True))),binary_sha256=old['binary_sha256'])
def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
def preservation():
    for name,archive in ARCHIVES.items():v.verify_seal(ROOT/'output'/archive,v.ROOTS[name],v.COUNTS[name])
    frozen=ROOT/'output'/ARCHIVES['correction'];old=v.load(frozen/'manifest.json')['sources']
    changed=[n for n,h in old.items() if v.digest(ROOT/n)!=h]
    assert changed==['crates/hg-train/src/ere.rs'],changed
    export='pub mod resource_adapter;\n';live=(ROOT/changed[0]).read_text()
    assert live.count(export)==1 and live.replace(export,'',1)==(frozen/'source_snapshots'/changed[0]).read_text()
    return dict(archives={k:dict(seal=v.ROOTS[k],entries=v.COUNTS[k]) for k in ARCHIVES},unchanged_prior_bodies=len(old)-1,additive_export=changed[0])
def command(out,name,args,env,cwd):
    r=subprocess.run(args,cwd=cwd,env=env,capture_output=True,text=True,timeout=90)
    save(out/name,dict(command=args,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    assert r.returncode==0,r.stdout+r.stderr
    return r.stdout
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    history=preservation();failed=failed_preservation();old=v.load(ROOT/'output'/ARCHIVES['correction']/'manifest.json')['sources']
    names=set(old)|set(NEW)|{'Cargo.toml','Cargo.lock'}
    for path in (ROOT/'crates').rglob('*'):
        if path.is_file() and 'target' not in path.parts and (path.suffix=='.rs' or path.name=='Cargo.toml'):names.add(path.relative_to(ROOT).as_posix())
    names=sorted(names);pins={n:v.digest(ROOT/n) for n in names};out.mkdir();start=time.time()
    for n in names:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    inputs=out/'inputs';inputs.mkdir();imports={}
    for key,archive in ARCHIVES.items():shutil.copyfile(ROOT/'output'/archive/'output_sha256.json',inputs/f'{key}_seal.json')
    for i in range(16):
        name=f'native/cell-{i:04}.json';dest=inputs/f'cell-{i:04}.json';shutil.copyfile(ROOT/'output'/ARCHIVES['primary']/name,dest)
        imports[dest.relative_to(out).as_posix()]=dict(archive='primary',path=name)
    for name,local in [('audit.json','allocation_audit.json'),('manifest.json','allocation_completion_manifest.json')]:
        shutil.copyfile(ROOT/'output'/ARCHIVES['completion_v4']/name,inputs/local)
        imports['inputs/'+local]=dict(archive='completion_v4',path=name)
    save(out/'preservation_before.json',history)
    save(out/'accounting_correction.json',failed)
    m=dict(profile='E2-resource-token-adapter-engineering-v1',status='running',sources=pins,imported_files=imports,
        accounting_revision=2,failed_primary_root=FAILED_ROOT,
        native_advances=0,optimizer_updates=0,scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)
    save(out/'manifest.json',m)
    env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_ADAPTER_ARCHIVE=str(out))
    try:
        cargo=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_resource_adapter']
        stdout=command(out,'binary_build.json',cargo+['--no-run','--message-format=json'],env,ROOT)
        rows=[json.loads(s) for s in stdout.splitlines() if s.startswith('{')]
        bins=[r['executable'] for r in rows if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('name')=='ere_resource_adapter' and r.get('executable')]
        assert len(bins)==1;shutil.copyfile(bins[0],out/'test_binary');(out/'test_binary').chmod(0o755);m['binary_sha256']=v.digest(out/'test_binary');assert m['binary_sha256']==failed['binary_sha256'];save(out/'manifest.json',m)
        command(out,'rust_tests.json',[str(out/'test_binary'),'--nocapture'],env,out)
        command(out,'receipt_fixture.json',[str(out/'test_binary'),'fixed_receipt_fixture','--ignored','--exact','--nocapture'],env,out)
        scripts=out/'source_snapshots/scripts'
        stdout=command(out,'verification.json',[sys.executable,'-B',str(scripts/'verify_ere_resource_adapter.py'),str(out)],env,out)
        audit=json.loads(stdout);save(out/'audit.json',audit)
        command(out,'python_tests.json',[sys.executable,'-B',str(scripts/'test_verify_ere_resource_adapter.py'),'-v'],env,out)
        command(out,'feature_off.json',['cargo','check','--offline','--locked','-p','hg-train','--no-default-features'],env,ROOT)
        assert pins=={n:v.digest(ROOT/n) for n in names},'live source drift'
        assert preservation()==history;save(out/'preservation_after.json',history)
        assert failed_preservation()==failed;save(out/'failed_preservation_after.json',failed)
        size=sum(p.stat().st_size for p in out.rglob('*') if p.is_file());assert size<128*1024*1024-262144
        m.update(status='complete',engineering_pass=audit['engineering_pass'],elapsed_seconds=time.time()-start,preseal_output_bytes=size)
    except Exception as error:
        m.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start);save(out/'manifest.json',m);seal(out);raise
    save(out/'manifest.json',m);seal(out)
    print(json.dumps(dict(output=str(out),status=m['status'],source_pins=len(pins),entries=v.verify_seal(out),seal_sha256=v.digest(out/'output_sha256.json')),indent=2))
def seal(out):
    save(out/'output_sha256.json',{p.relative_to(out).as_posix():v.digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name!='output_sha256.json'})
if __name__=='__main__':main()
