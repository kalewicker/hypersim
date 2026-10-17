#!/usr/bin/env python3
"""Zero-native scratch verification and sealed contrast preflight; no launch branch."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import verify_ere_resource_contrast_adapter as v
import run_ere_resource_learning as history

ROOT=Path(__file__).resolve().parents[1]
PARENT=ROOT/'output/ere_resource_learning_campaign_20261008'
DIAGNOSTIC=ROOT/'output/ere_resource_gradient_diagnostic_20261009'
def save(p,x):p.write_text(json.dumps(x,indent=2,sort_keys=True,allow_nan=False)+'\n')
def preservation():
    for key,name in history.ARCHIVES.items():v.old.a.verify_seal(ROOT/'output'/name,v.old.ROOTS[key],v.old.COUNTS[key])
    v.old.a.verify_seal(PARENT,v.CAMPAIGN,8968);v.old.a.verify_seal(DIAGNOSTIC,v.DIAGNOSTIC,838)
    pm=v.old.a.load(PARENT/'manifest.json');dm=v.old.a.load(DIAGNOSTIC/'manifest.json')
    changed=[n for n,h in pm['sources'].items() if v.old.a.digest(ROOT/n)!=h]
    assert changed==['crates/hg-train/src/ere.rs'],changed
    text=(ROOT/changed[0]).read_text();assert text.count(v.EXPORT)==1
    assert text.replace(v.EXPORT,'',1)==(PARENT/'source_snapshots'/changed[0]).read_text()
    assert all(v.old.a.digest(ROOT/n)==h for n,h in dm['sources'].items())
    return dict(campaign_root=v.CAMPAIGN,diagnostic_root=v.DIAGNOSTIC,
                historical_roots=v.old.ROOTS,unchanged_campaign_bodies=len(pm['sources'])-1,
                unchanged_diagnostic_bodies=len(dm['sources']),registered_changes=changed,
                exact_additive_export=v.EXPORT)
def command(stage,name,args,env,cwd,timeout=90):
    r=subprocess.run(args,cwd=cwd,env=env,capture_output=True,text=True,timeout=timeout)
    save(stage/name,dict(command=args,cwd=str(cwd),returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    assert r.returncode==0,r.stdout+r.stderr
    return r.stdout
def run(out):
    assert out.is_relative_to(ROOT/'output') and not out.exists()
    started=time.time();before=preservation();pm=v.old.a.load(PARENT/'manifest.json')
    with tempfile.TemporaryDirectory(prefix='ere-contrast-preflight-') as path:
        stage=Path(path);(stage/'inputs').mkdir()
        imports={f'inputs/cell-{i:04}.json':dict(archive='campaign',path=f'inputs/cell-{i:04}.json') for i in range(16)}
        imports['inputs/allocation_audit.json']=dict(archive='campaign',path='inputs/allocation_audit.json')
        for key,parent in [('campaign',PARENT),('diagnostic',DIAGNOSTIC)]:
            shutil.copyfile(parent/'output_sha256.json',stage/f'inputs/{key}_seal.json')
            for kind in ['manifest','audit']:
                imports[f'inputs/{key}_{kind}.json']=dict(archive=key,path=f'{kind}.json')
        for local,origin in imports.items():
            parent=PARENT if origin['archive']=='campaign' else DIAGNOSTIC
            shutil.copyfile(parent/origin['path'],stage/local)
        sources={}
        for name in sorted(set(pm['sources'])|set(v.EXTRA)):
            dest=stage/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True)
            shutil.copyfile(ROOT/name,dest);sources[name]=v.old.a.digest(dest)
        manifest=dict(profile=v.PROFILE,status='running',sources=sources,imported_files=imports,
                      native_advances=0,optimizer_updates=0,controller_fitting=False,launch_authorized=False,
                      scientific_acceptance=False,training_admitted=False,sustained_claim=False,independent_physics_replay=False)
        save(stage/'manifest.json',manifest);save(stage/'preservation_before.json',before)
        env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_CONTRAST_PREFLIGHT=str(stage))
        rows=command(stage,'test_build.json',['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_resource_contrast_adapter','--no-run','--message-format=json'],env,ROOT)
        artifacts=[json.loads(line) for line in rows.splitlines() if line.startswith('{')]
        bins=[a['executable'] for a in artifacts if a.get('reason')=='compiler-artifact' and a.get('target',{}).get('name')=='ere_resource_contrast_adapter' and a.get('executable')]
        assert len(bins)==1;shutil.copyfile(bins[0],stage/'test_binary');(stage/'test_binary').chmod(0o755)
        manifest['binary_sha256']=v.old.a.digest(stage/'test_binary');save(stage/'manifest.json',manifest)
        command(stage,'rust_tests.json',[str(stage/'test_binary'),'--nocapture'],env,stage)
        command(stage,'fixture.json',[str(stage/'test_binary'),'saved_contrast_fixture','--ignored','--exact','--nocapture'],env,stage)
        scripts=stage/'source_snapshots/scripts'
        output=command(stage,'verification.json',[sys.executable,'-B',str(scripts/'verify_ere_resource_contrast_adapter.py'),str(stage)],env,stage)
        audit=json.loads(output);save(stage/'audit.json',audit)
        command(stage,'python_tests.json',[sys.executable,'-B',str(scripts/'test_verify_ere_resource_contrast_adapter.py'),'-v'],env,stage)
        for target in ['ere_resource_adapter','ere_resource_timed_bridge','ere_resource_learning']:
            command(stage,f'{target}_regression.json',['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test',target],env,ROOT)
        for package in ['hg-train','hg-sim-core']:
            command(stage,f'{package}_feature_off.json',['cargo','check','--offline','--locked','-p',package,'--no-default-features'],env,ROOT)
        after=preservation();assert after==before;save(stage/'preservation_after.json',after)
        assert all(v.old.a.digest(ROOT/n)==h for n,h in sources.items())
        size=sum(p.stat().st_size for p in stage.rglob('*') if p.is_file());assert size<134217728-1048576
        manifest.update(status='complete',engineering_pass=True,elapsed_seconds=time.time()-started,preseal_bytes=size)
        save(stage/'manifest.json',manifest);out.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(stage,out)
    save(out/'output_sha256.json',{p.relative_to(out).as_posix():v.old.a.digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    return dict(output=str(out),entries=v.old.a.verify_seal(out),sources=len(sources),
                seal_sha256=v.old.a.digest(out/'output_sha256.json'),**audit)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);args=p.parse_args()
    print(json.dumps(run(args.output.resolve()),indent=2))
