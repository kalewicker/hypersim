#!/usr/bin/env python3
"""Fresh frozen preparation; native execution requires explicit --execute."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import time
import audit_ere_categorical_comparison as v

ROOT=v.ROOT;a=v.a
BUILD=['cargo','build','--offline','--release','-p','hg-train','--no-default-features','--features','ere-complete-signal','--bin','ere-categorical-comparison']
def write(path,value):path.write_text(json.dumps(value,indent=2))
def live(hashes):
    assert all(a.digest(ROOT/n)==h for n,h in hashes.items()),'sources changed during preparation/build'
def initial(mode,count):
    return dict(profile=v.p.PROFILE,mode=mode,status='zero_work_preflight',reason=None,scheduled_cells=count,
        recorded_cells=0,attempted_steps=0,advancing_steps=0,unknown_partial_work_cells=0,
        inflight_native_work_unknown=False,grammar_compilations=0,optimizer_generations=0,training_updates=0,
        conditional_leaf_checks=0,
        direction_starts=0,direction_estimates=0,receipt_bytes=0,training_admitted=False,full_training_authorized=False,
        live_sources_rechecked=False,elapsed_seconds=0.)
def seed_collision_check(c):
    """Read explicit seeds in prior root schedule files; no random draws."""
    seeds=set();paths=[]
    for path in sorted((ROOT/'output').glob('*/schedule.json')):
        if path.is_symlink():raise AssertionError('symlink prior schedule')
        manifest=path.parent/'manifest.json'
        if manifest.exists():
            m=a.load(manifest)
            if m.get('status')=='zero_work_preflight' and m.get('recorded_cells')==m.get('attempted_steps')==m.get('direction_starts')==m.get('grammar_compilations')==0 and m.get('inflight_native_work_unknown') is False:
                continue # Allocated IDs with no native/RNG use do not consume seeds.
        schedule=a.load(path)
        if not isinstance(schedule,list):continue
        paths.append(dict(path=str(path.relative_to(ROOT)),sha256=a.digest(path)))
        for q in schedule:
            if isinstance(q,dict):
                for k in ('emission_seed','physics_seed','noise_seed'):
                    if type(q.get(k)) is int:seeds.add(q[k])
    for mode in ('development','scientific'):
        assert not (seeds & v.p.check_schedule(c,mode)['seeds']),'prior recorded seed collision; prospective revision required'
    return dict(prior_schedule_files=paths,explicit_seed_identities=len(seeds),collision=False,
        scope='explicit seeds in existing root schedule.json files only; not a proof about implicit historical RNG draws')
def prepare(out,mode,execute=False,development_receipt=None):
    out=Path(out).absolute();assert not out.exists(),'fresh comparison output required'
    assert mode in ('development','scientific') and type(execute) is bool
    # Never place outputs under immutable evidence or source directories.
    assert out.parent.resolve()==(ROOT/'output').resolve(),'direct fresh output/ child required'
    start=time.monotonic()
    c=v.config();receipt=v.registration(ROOT,c,check_historical=True)
    # Verify the native helper lineage too, before any compilation/simulation.
    a.verify(v.PARENT,expected=v.PARENT_SEAL,count=1690);a.verify(v.RUNTIME,expected=v.RUNTIME_SEAL,count=460)
    seeds=seed_collision_check(c)
    hashes={n:a.digest(ROOT/n) for n in receipt['source_names']}
    development_gate=None
    if mode=='scientific' and execute:
        assert development_receipt is not None,'science requires completed frozen development receipt'
        dev=Path(development_receipt).resolve();a.verify(dev);checked=v.audit(dev)
        assert checked['classification']=='bounded_development' and checked['campaign_complete'] and checked['unknown_partial_work_cells']==0
        assert a.load(dev/'snapshot.json')['sources']==hashes,'development source identity changed'
        assert len(a.load(dev/'audit.json')['detached_tamper_rejections'])==6,'development tamper controls required'
        development_gate=dict(path=str(dev),root_seal=a.digest(dev/'output_sha256.json'))
    built=subprocess.run(BUILD,cwd=ROOT,capture_output=True,text=True)
    assert built.returncode==0,built.stderr;live(hashes)
    binary=ROOT/'target/release/ere-categorical-comparison';assert binary.is_file()
    # Conservatively include source/lineage checks and Cargo build in setup wall.
    out.mkdir()
    for directory in ('cells','estimates','directions','validation'):(out/directory).mkdir()
    for n,h in hashes.items():
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest);assert a.digest(dest)==h
    shutil.copy2(binary,out/'executed_binary')
    snap=dict(sources=hashes,binary_sha256=a.digest(out/'executed_binary'),frozen_before_measurement=True,
        build_features=['ere-complete-signal'],default_features=False,repository=str(ROOT))
    build=dict(command=BUILD,returncode=0,source_hashes_before=hashes,source_hashes_after=hashes,binary_sha256=snap['binary_sha256'],
        stdout=built.stdout,stderr=built.stderr,seed_collision_check=seeds)
    write(out/'snapshot.json',snap);write(out/'source_registration.json',receipt);write(out/'build_receipt.json',build)
    write(out/'development_gate.json',development_gate)
    shutil.copyfile(ROOT/'configs/ere_categorical_comparison_v1.json',out/'config.json')
    shutil.copyfile(ROOT/c['lineage']['direction']['path']/'catalogue.json',out/'reference_catalogue.json')
    plan=v.p.schedule(c,mode);write(out/'schedule.json',plan);write(out/'contexts.json',v.p.contexts())
    for q in plan:write(out/'cells'/(q['id']+'.json'),dict(schedule=q,status='NotRun',reason='not_started',evaluation=None))
    m=initial(mode,len(plan));write(out/'initial_manifest.json',m);write(out/'manifest.json',m)
    n=c[mode]['blocks'];align=[[[None]*2 for _ in range(2)] for _ in range(n)];effects=[[[[None]*2 for _ in range(2)] for _ in range(2)] for _ in range(n)]
    write(out/'summary.json',v.p.statistics(c,mode,align,effects,effects,False))
    req=dict(profile=v.p.PROFILE,mode=mode,launch_authorized=execute,full_training_authorized=False,setup_elapsed_seconds=time.monotonic()-start)
    assert req['setup_elapsed_seconds']<c[mode]['wall_seconds'];write(out/'request.json',req);live(hashes)
    immutable={f'source_snapshots/{n}' for n in hashes}|{'snapshot.json','source_registration.json','build_receipt.json','executed_binary','config.json','schedule.json','contexts.json','request.json','initial_manifest.json','reference_catalogue.json','development_gate.json'}
    write(out/'prepared_sha256.json',{n:a.digest(out/n) for n in sorted(immutable)})
    a.seal(out)
    if execute:
        native=subprocess.run([str(out/'executed_binary'),'run-prepared',str(out),a.digest(out/'prepared_sha256.json')],cwd=ROOT)
        # A hard process failure stays failed, with any InFlight state intact.
        m=a.load(out/'manifest.json')
        if native.returncode!=0 and m['status'] not in ('incomplete','blocked_before_evaluation'):
            m.update(status='blocked_before_evaluation' if m['recorded_cells']==0 else 'incomplete',reason=f'native process exit {native.returncode}')
            write(out/'manifest.json',m);a.seal(out)
    # Run the frozen detached auditor, never the mutable workspace checker.
    audit_cmd=['python3','-B',str(out/'source_snapshots/scripts/audit_ere_categorical_comparison.py'),str(out)]
    if execute and mode=='development':audit_cmd.append('--tamper-tests')
    checked=subprocess.run(audit_cmd,cwd=ROOT,capture_output=True,text=True,timeout=900)
    if checked.returncode!=0:
        write(out/'failed_audit.json',dict(returncode=checked.returncode,stdout=checked.stdout,stderr=checked.stderr));a.seal(out)
        raise AssertionError('frozen audit failed; preserved failed_audit.json; no retry/reseal correction')
    result=json.loads(checked.stdout);write(out/'audit.json',result);a.seal(out);a.verify(out)
    print(json.dumps(dict(archive=str(out),root_seal=a.digest(out/'output_sha256.json'),**result),indent=2))
    return out

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--output',required=True,type=Path);parser.add_argument('--mode',choices=['development','scientific'],default='development')
    parser.add_argument('--execute',action='store_true',help='Only after separate user approval for this mode; performs native work')
    parser.add_argument('--development-receipt',type=Path,help='Required for science execution; complete, tamper-checked and source-identical')
    args=parser.parse_args();prepare(args.output,args.mode,args.execute,args.development_receipt)
