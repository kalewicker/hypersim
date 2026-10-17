#!/usr/bin/env python3
"""Single frozen bounded timing panel; no fitting and no native retry."""
import argparse
import difflib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import verify_ere_resource_timed_bridge as v

ROOT=Path(__file__).resolve().parents[1]
ARCHIVES=dict(primary='ere_defense_allocation_20261007',correction='ere_defense_allocation_accounting_v2_20261007',completion_v3='ere_defense_allocation_completion_v3_20261007',completion_v4='ere_defense_allocation_completion_v4_20261007',adapter='ere_resource_adapter_accounting_v2_20261008',failed_adapter='ere_resource_adapter_20261008')
ROOTS={**v.a.ROOTS,'adapter':v.ADAPTER_ROOT,'failed_adapter':v.FAILED_ADAPTER_ROOT}
COUNTS={**v.a.COUNTS,'adapter':688,'failed_adapter':681}
EXTRA=['configs/ere_resource_timed_bridge_v1.json','docs/ere_resource_timed_bridge_v1.md',
    'configs/ere_resource_learning_proposal_v1.json','docs/ere_resource_learning_proposal_v1.md',
    'crates/hg-sim-core/src/ere/resource_timed_bridge.rs','crates/hg-train/src/ere/resource_timed_policy.rs',
    'crates/hg-train/tests/ere_resource_timed_bridge.rs','scripts/verify_ere_resource_timed_bridge.py',
    'scripts/test_verify_ere_resource_timed_bridge.py','scripts/run_ere_resource_timed_bridge.py','scripts/recheck_ere_resource_timed_bridge.py',
    'docs/ere_resource_timed_bridge_preflight_v2.md']
FAILED=ROOT/'output/ere_resource_timed_bridge_20261008'
FAILED_ROOT='560ea51646703e3b3c10bd68e57b0e3637701796a1787767362b315316716e82'
def failed_preservation():
    v.a.verify_seal(FAILED,FAILED_ROOT,695);old=v.a.load(FAILED/'manifest.json')
    assert old['status']=='failed' and old['native_launched'] is False and old['native_advances']==old['seal_applications']==old['optimizer_updates']==0
    changed=[n for n,h in old['sources'].items() if v.a.digest(ROOT/n)!=h]
    assert changed==['scripts/run_ere_resource_timed_bridge.py','scripts/verify_ere_resource_timed_bridge.py'],changed
    name='scripts/verify_ere_resource_timed_bridge.py';frozen=(FAILED/'source_snapshots'/name).read_text();live=(ROOT/name).read_text()
    assert live==frozen.replace('slots.extend(dict(id=len(slots)+i,refusal=name) for i,name in enumerate(REFUSALS))','slots.extend([dict(id=len(slots)+i,refusal=name) for i,name in enumerate(REFUSALS)])')
    return dict(root=FAILED_ROOT,entries=695,status='failed',native_launched=False,changed_helpers=changed,
        verifier_diff=''.join(difflib.unified_diff(frozen.splitlines(True),live.splitlines(True))),binary_sha256=old['binary_sha256'])
def save(p,x):p.write_text(json.dumps(x,indent=2,sort_keys=True)+'\n')
def preservation():
    for name,archive in ARCHIVES.items():v.a.verify_seal(ROOT/'output'/archive,ROOTS[name],COUNTS[name])
    old=v.a.load(ROOT/'output'/ARCHIVES['adapter']/'manifest.json')['sources'];changed=[n for n,h in old.items() if v.a.digest(ROOT/n)!=h]
    exports={'crates/hg-sim-core/src/ere.rs':'pub mod resource_timed_bridge;\n','crates/hg-train/src/ere.rs':'pub mod resource_timed_policy;\n'}
    assert changed==list(exports),changed
    for name,export in exports.items():
        live=(ROOT/name).read_text();assert live.count(export)==1 and live.replace(export,'',1)==(ROOT/'output'/ARCHIVES['adapter']/'source_snapshots'/name).read_text()
    return dict(archives={k:dict(root=ROOTS[k],entries=COUNTS[k]) for k in ARCHIVES},unchanged_prior_bodies=len(old)-2,additive_exports=exports)
def command(out,name,args,env,cwd,check=True):
    r=subprocess.run(args,cwd=cwd,env=env,capture_output=True,text=True,timeout=90)
    save(out/name,dict(command=args,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    if check:assert r.returncode==0,r.stdout+r.stderr
    return r
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    before=preservation();failed=failed_preservation();old=v.a.load(ROOT/'output'/ARCHIVES['adapter']/'manifest.json')['sources'];names=sorted(set(old)|set(EXTRA));pins={n:v.a.digest(ROOT/n) for n in names}
    out.mkdir();start=time.time()
    for n in names:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    inputs=out/'inputs';inputs.mkdir();imports={}
    for key,archive in ARCHIVES.items():shutil.copyfile(ROOT/'output'/archive/'output_sha256.json',inputs/f'{key}_seal.json')
    for i in range(16):
        local=f'inputs/cell-{i:04}.json';origin=f'native/cell-{i:04}.json';shutil.copyfile(ROOT/'output'/ARCHIVES['primary']/origin,out/local);imports[local]=dict(archive='primary',path=origin)
    for archive,origin,local in [('completion_v4','audit.json','allocation_audit.json'),('adapter','audit.json','adapter_audit.json'),('adapter','manifest.json','adapter_manifest.json')]:
        shutil.copyfile(ROOT/'output'/ARCHIVES[archive]/origin,inputs/local);imports['inputs/'+local]=dict(archive=archive,path=origin)
    save(out/'preservation_before.json',before)
    save(out/'preflight_correction.json',failed)
    old_kernel=(ROOT/'crates/hg-sim-core/src/ere/defense_allocation.rs').read_text();new_kernel=(ROOT/EXTRA[4]).read_text()
    (out/'kernel_fork.diff').write_text(''.join(difflib.unified_diff(old_kernel.splitlines(True),new_kernel.splitlines(True),fromfile='frozen-defense-allocation-v1',tofile='resource-timed-bridge-v1')))
    m=dict(profile=v.PROFILE,status='running',preflight_revision=2,failed_preflight_root=FAILED_ROOT,sources=pins,imported_files=imports,native_launched=False,native_work_unknown=False,native_advances=0,seal_applications=0,optimizer_updates=0,scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)
    save(out/'manifest.json',m);env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_TIMED_BRIDGE_ARCHIVE=str(out));scripts=out/'source_snapshots/scripts'
    try:
        cargo=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_resource_timed_bridge']
        result=command(out,'binary_build.json',cargo+['--no-run','--message-format=json'],env,ROOT)
        rows=[json.loads(s) for s in result.stdout.splitlines() if s.startswith('{')];bins=[r['executable'] for r in rows if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('name')=='ere_resource_timed_bridge' and r.get('executable')]
        assert len(bins)==1;shutil.copyfile(bins[0],out/'test_binary');(out/'test_binary').chmod(0o755);m['binary_sha256']=v.a.digest(out/'test_binary');assert m['binary_sha256']==failed['binary_sha256'];save(out/'manifest.json',m)
        command(out,'rust_preflight.json',[str(out/'test_binary'),'--nocapture'],env,out)
        tests=[sys.executable,'-B',str(scripts/'test_verify_ere_resource_timed_bridge.py'),'-v'];command(out,'python_preflight.json',tests,env,out)
        for package in ['hg-train','hg-sim-core']:command(out,f'{package}_feature_off.json',['cargo','check','--offline','--locked','-p',package,'--no-default-features'],env,ROOT)
        assert pins=={n:v.a.digest(ROOT/n) for n in names} and preservation()==before
        m.update(native_launched=True,native_work_unknown=True);save(out/'manifest.json',m)
        result=command(out,'native_receipt.json',[str(out/'test_binary'),'fixed_timed_bridge_fixture','--ignored','--exact','--nocapture'],env,out,check=False)
        statuses=v.a.load(out/'native/status.json');m['native_work_unknown']=any(s['status']=='running' for s in statuses)
        m['native_advances']=sum(s.get('failure',s).get('native_advances',0) for s in statuses);m['seal_applications']=sum(s.get('failure',s).get('seal_applications',0) for s in statuses);save(out/'manifest.json',m)
        assert result.returncode==0,result.stdout+result.stderr
        query=[sys.executable,'-B',str(scripts/'verify_ere_resource_timed_bridge.py'),str(out)]
        result=command(out,'verification.json',query,env,out);audit=json.loads(result.stdout);save(out/'audit.json',audit)
        command(out,'python_postflight.json',tests,env,out)
        assert pins=={n:v.a.digest(ROOT/n) for n in names};assert preservation()==before;save(out/'preservation_after.json',before)
        assert failed_preservation()==failed;save(out/'failed_preflight_preservation_after.json',failed)
        size=sum(p.stat().st_size for p in out.rglob('*') if p.is_file());assert size<128*1024*1024-262144
        m.update(status='complete',engineering_pass=True,elapsed_seconds=time.time()-start,preseal_output_bytes=size)
    except Exception as error:
        m.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start);save(out/'manifest.json',m);seal(out);raise
    save(out/'manifest.json',m);seal(out)
    print(json.dumps(dict(status=m['status'],source_pins=len(pins),entries=v.a.verify_seal(out),seal_sha256=v.a.digest(out/'output_sha256.json'),native_advances=m['native_advances'],seal_applications=m['seal_applications']),indent=2))
def seal(out):save(out/'output_sha256.json',{p.relative_to(out).as_posix():v.a.digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name!='output_sha256.json'})
if __name__=='__main__':main()
