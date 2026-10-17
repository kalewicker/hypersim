#!/usr/bin/env python3
"""Default zero-native preflight; explicit separate approved launch branch."""
import argparse
import difflib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import verify_ere_resource_learning as v

ROOT=Path(__file__).resolve().parents[1]
ARCHIVES=dict(primary='ere_defense_allocation_20261007',correction='ere_defense_allocation_accounting_v2_20261007',completion_v3='ere_defense_allocation_completion_v3_20261007',completion_v4='ere_defense_allocation_completion_v4_20261007',adapter='ere_resource_adapter_accounting_v2_20261008',failed_adapter='ere_resource_adapter_20261008',bridge='ere_resource_timed_bridge_preflight_v2_20261008',failed_bridge='ere_resource_timed_bridge_20261008')
EXTRA=['configs/ere_resource_learning_runner_v1.json','docs/ere_resource_learning_runner_v1.md',
    'crates/hg-train/src/ere/resource_learning.rs','crates/hg-train/src/bin/ere-resource-learning.rs','crates/hg-train/tests/ere_resource_learning.rs',
    'scripts/verify_ere_resource_learning.py','scripts/test_verify_ere_resource_learning.py','scripts/run_ere_resource_learning.py','scripts/recheck_ere_resource_learning.py']
BIN_STANZA='\n[[bin]]\nname = "ere-resource-learning"\npath = "src/bin/ere-resource-learning.rs"\nrequired-features = ["ere"]\n'
def save(p,x):p.write_text(json.dumps(x,indent=2,sort_keys=True)+'\n')
def preservation():
    for name,archive in ARCHIVES.items():v.a.verify_seal(ROOT/'output'/archive,v.ROOTS[name],v.COUNTS[name])
    parent=ROOT/'output'/ARCHIVES['bridge'];old=v.a.load(parent/'manifest.json')['sources'];changed=[n for n,h in old.items() if v.a.digest(ROOT/n)!=h]
    assert changed==['Cargo.lock','crates/hg-train/Cargo.toml','crates/hg-train/src/ere.rs'],changed
    frozen=(parent/'source_snapshots/crates/hg-train/Cargo.toml').read_text()
    expected=frozen.replace('"dep:hg-rules"]','"dep:hg-rules", "dep:flate2"]',1).replace('[dependencies]\n','[dependencies]\nflate2 = { version = "=1.1.9", optional = true }\n',1)+BIN_STANZA
    assert (ROOT/'crates/hg-train/Cargo.toml').read_text()==expected
    frozen_lock=(parent/'source_snapshots/Cargo.lock').read_text();start=frozen_lock.index('name = "hg-train"');end=frozen_lock.index('\n[[package]]',start)
    expected_lock=frozen_lock[:start]+frozen_lock[start:end].replace(' "csv",\n',' "csv",\n "flate2",\n',1)+frozen_lock[end:]
    assert (ROOT/'Cargo.lock').read_text()==expected_lock,'unregistered lockfile drift'
    export='pub mod resource_learning;\n';live=(ROOT/'crates/hg-train/src/ere.rs').read_text();assert live.count(export)==1 and live.replace(export,'',1)==(parent/'source_snapshots/crates/hg-train/src/ere.rs').read_text()
    return dict(archives={k:dict(root=v.ROOTS[k],entries=v.COUNTS[k]) for k in ARCHIVES},unchanged_prior_bodies=len(old)-3,registered_changes=changed,compressor_version='1.1.9')
def command(out,name,args,env,cwd,timeout=90):
    r=subprocess.run(args,cwd=cwd,env=env,capture_output=True,text=True,timeout=timeout)
    save(out/name,dict(command=args,cwd=str(cwd),returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    assert r.returncode==0,r.stdout+r.stderr
    return r.stdout
def prepare(out):
    parent=ROOT/'output'/ARCHIVES['bridge'];old=v.a.load(parent/'manifest.json')['sources'];names=sorted(set(old)|set(EXTRA));pins={n:v.a.digest(ROOT/n) for n in names}
    out.mkdir(parents=True,exist_ok=True)
    for n in names:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    inputs=out/'inputs';inputs.mkdir();imports={}
    for key,archive in ARCHIVES.items():shutil.copyfile(ROOT/'output'/archive/'output_sha256.json',inputs/f'{key}_seal.json')
    for i in range(16):
        local=f'inputs/cell-{i:04}.json';origin=f'native/cell-{i:04}.json';shutil.copyfile(ROOT/'output'/ARCHIVES['primary']/origin,out/local);imports[local]=dict(archive='primary',path=origin)
    for i in range(8):
        local=f'inputs/state-{i}.json';origin=f'native/cell-{i:04}.json';shutil.copyfile(parent/origin,out/local);imports[local]=dict(archive='bridge',path=origin)
    shutil.copyfile(ROOT/'output'/ARCHIVES['completion_v4']/'audit.json',inputs/'allocation_audit.json');imports['inputs/allocation_audit.json']=dict(archive='completion_v4',path='audit.json')
    m=dict(profile=v.PREFLIGHT,status='running',sources=pins,imported_files=imports,binaries={},native_advances=0,optimizer_updates=0,launch_authorized=False,scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)
    save(out/'manifest.json',m);return m
def compile_binaries(out,env):
    rows=command(out,'binary_build.json',['cargo','build','--offline','--locked','-p','hg-train','--features','ere','--bin','ere-resource-learning','--message-format=json'],env,ROOT)
    items=[json.loads(line) for line in rows.splitlines() if line.startswith('{')];bins=[i['executable'] for i in items if i.get('reason')=='compiler-artifact' and i.get('target',{}).get('name')=='ere-resource-learning' and i.get('executable')];assert len(bins)==1
    shutil.copyfile(bins[0],out/'campaign_binary');(out/'campaign_binary').chmod(0o755)
    rows=command(out,'test_build.json',['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_resource_learning','--no-run','--message-format=json'],env,ROOT)
    items=[json.loads(line) for line in rows.splitlines() if line.startswith('{')];bins=[i['executable'] for i in items if i.get('reason')=='compiler-artifact' and i.get('target',{}).get('name')=='ere_resource_learning' and i.get('executable')];assert len(bins)==1
    shutil.copyfile(bins[0],out/'test_binary');(out/'test_binary').chmod(0o755)
    return {n:v.a.digest(out/n) for n in ['campaign_binary','test_binary']}
def preflight(out):
    start=time.time();history=preservation()
    # Live implementation checks happen in disposable zero-native scratch before the final evidence package is frozen.
    with tempfile.TemporaryDirectory(prefix='resource-learning-preflight-') as tmp:
        stage=Path(tmp);m=prepare(stage);save(stage/'preservation_before.json',history)
        env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_LEARNING_PREFLIGHT=str(stage))
        m['binaries']=compile_binaries(stage,env);save(stage/'manifest.json',m)
        command(stage,'rust_tests.json',[str(stage/'test_binary'),'--nocapture'],env,stage)
        text=command(stage,'schedule_receipt.json',[str(stage/'campaign_binary'),'schedule'],env,stage);(stage/'schedule.json').write_text(text)
        assert v.a.load(stage/'schedule.json')==v.schedule();save(stage/'draw_hashes.json',v.draw_hashes())
        command(stage,'arithmetic_receipt.json',[str(stage/'test_binary'),'saved_credit_and_arithmetic_fixture','--ignored','--exact','--nocapture'],env,stage)
        scripts=stage/'source_snapshots/scripts'
        result=command(stage,'verification.json',[sys.executable,'-B',str(scripts/'verify_ere_resource_learning.py'),str(stage)],env,stage);audit=json.loads(result);save(stage/'audit.json',audit)
        command(stage,'python_tests.json',[sys.executable,'-B',str(scripts/'test_verify_ere_resource_learning.py'),'-v'],env,stage)
        for package in ['hg-train','hg-sim-core']:command(stage,f'{package}_feature_off.json',['cargo','check','--offline','--locked','-p',package,'--no-default-features'],env,ROOT)
        assert preservation()==history;assert all(v.a.digest(ROOT/n)==h for n,h in m['sources'].items());save(stage/'preservation_after.json',history)
        size=sum(p.stat().st_size for p in stage.rglob('*') if p.is_file());assert size<268435456-262144
        m.update(status='complete',engineering_pass=True,elapsed_seconds=time.time()-start,preseal_output_bytes=size)
        save(stage/'manifest.json',m);shutil.copytree(stage,out)
    seal(out);return dict(status='complete',output=str(out),entries=v.a.verify_seal(out),source_pins=len(m['sources']),seal_sha256=v.a.digest(out/'output_sha256.json'),**audit)
def launch(out,source,expected,authorization,ack):
    assert ack==238080 and authorization.strip(),'explicit human approval note and exact native budget required'
    source=source.resolve();v.a.verify_seal(source,expected);assert v.a.load(source/'manifest.json')['status']=='complete';v.audit_preflight(source)
    history=preservation();old=v.a.load(source/'manifest.json');assert all(v.a.digest(ROOT/n)==h for n,h in old['sources'].items()),'live source drift since preflight'
    out.mkdir();start=time.time()
    for path in ['source_snapshots','inputs'] :shutil.copytree(source/path,out/path)
    for path in ['campaign_binary','test_binary','schedule.json','draw_hashes.json','arithmetic.json'] :shutil.copyfile(source/path,out/path)
    for i in range(8):shutil.copyfile(source/f'compressed-state-{i}.json.gz',out/f'compressed-state-{i}.json.gz')
    (out/'campaign_binary').chmod(0o755);(out/'test_binary').chmod(0o755)
    m={**old,'profile':v.PROFILE,'status':'running','launch_authorized':True,'authorization_note':authorization,'preflight_root':expected,'native_work_unknown':True,'native_advances':0,'seal_applications':0,'optimizer_updates':0}
    save(out/'manifest.json',m);save(out/'preservation_before.json',history)
    env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1');bootstrap=sum(p.stat().st_size for p in out.rglob('*') if p.is_file())
    try:
        command(out,'native_receipt.json',[str(out/'campaign_binary'),'campaign',str(out),str(ack),str(bootstrap)],env,out,timeout=7210)
        native=v.a.load(out/'campaign/manifest.json');m.update(native_work_unknown=native['native_work_unknown'],native_advances=native['native_advances'],seal_applications=native['seal_applications'],optimizer_updates=native['optimizer_updates'],native_status=native['status']);save(out/'manifest.json',m)
        text=command(out,'verification.json',[sys.executable,'-B',str(out/'source_snapshots/scripts/verify_ere_resource_learning.py'),str(out),'--campaign'],env,out,timeout=7200)
        audit=json.loads(text);save(out/'audit.json',audit)
        assert preservation()==history;assert all(v.a.digest(ROOT/n)==h for n,h in m['sources'].items());save(out/'preservation_after.json',history)
        assert sum(p.stat().st_size for p in out.rglob('*') if p.is_file())<1073741824-2097152
        m.update(status='complete',accounting_pass=True,local_learning_pass=audit['local_learning_pass'],elapsed_seconds=time.time()-start)
    except Exception as error:
        path=out/'campaign/manifest.json'
        if path.exists():
            native=v.a.load(path);m.update(native_status=native['status'],native_work_unknown=native['native_work_unknown'],native_advances=native['native_advances'],seal_applications=native['seal_applications'],optimizer_updates=native['optimizer_updates'])
        else:m['native_work_unknown']=True
        m.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start);save(out/'manifest.json',m);seal(out);raise
    save(out/'manifest.json',m);seal(out);return dict(status='complete',seal_sha256=v.a.digest(out/'output_sha256.json'),**audit)
def seal(out):save(out/'output_sha256.json',{p.relative_to(out).as_posix():v.a.digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name!='output_sha256.json'})
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);p.add_argument('--launch',action='store_true');p.add_argument('--preflight',type=Path);p.add_argument('--expected-preflight-seal');p.add_argument('--authorization-note',default='');p.add_argument('--acknowledge-native-advances',type=int,default=0);args=p.parse_args();out=args.output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    if args.launch:
        assert args.preflight is not None and args.expected_preflight_seal is not None
        result=launch(out,args.preflight,args.expected_preflight_seal,args.authorization_note,args.acknowledge_native_advances)
    else:
        assert not args.authorization_note and args.acknowledge_native_advances==0
        result=preflight(out)
    print(json.dumps(result,indent=2))
if __name__=='__main__':main()
