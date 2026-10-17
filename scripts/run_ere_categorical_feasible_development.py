#!/usr/bin/env python3
"""Freeze/preflight only by default. Native execution requires separate approval."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import audit_ere_categorical_feasible_development as v

d=v.d;a=v.a;ROOT=d.ROOT


def write(path,value):path.write_text(json.dumps(value,indent=2))


def live(hashes):
    for n,h in hashes.items():assert a.digest(ROOT/n)==h,'source changed during frozen preparation: '+n


def command(out,name,args,cwd=None,timeout=600,env=None):
    done=subprocess.run(args,cwd=cwd or ROOT,capture_output=True,text=True,timeout=timeout,env=env)
    write(out/(name+'.json'),dict(command=args,returncode=done.returncode,stdout=done.stdout,stderr=done.stderr))
    assert done.returncode==0,name+' failed; preserve archive, no retry'
    return done


def detached_audit(out):
    checked=subprocess.run([sys.executable,'-B',str(out/'source_snapshots/scripts/audit_ere_categorical_feasible_development.py'),str(out)],cwd=ROOT,capture_output=True,text=True,timeout=900)
    if checked.returncode!=0:
        write(out/'failed_audit.json',dict(returncode=checked.returncode,stdout=checked.stdout,stderr=checked.stderr));a.seal(out)
        raise AssertionError('frozen audit failed; no retry; failed_audit.json preserved')
    return json.loads(checked.stdout)


def verify_guards(out):
    binary=out/'executed_binary';root=a.digest(out/'prepared_sha256.json')
    before=a.load(out/'output_sha256.json')
    check=command(out,'check_only',[str(binary),'check-prepared',str(out),root],timeout=900)
    got=json.loads(check.stdout)
    assert got==dict(prepared_checked=True,native_evaluations_run=0,native_steps_run=0,grammar_compilations_run=0,training_updates=0)
    refusal=[]
    # Same executable and preparation, unapproved launch must refuse before work.
    process=subprocess.run([str(binary),'run-prepared',str(out),root],capture_output=True,text=True,timeout=60)
    assert process.returncode!=0 and 'explicit native development approval required' in process.stderr
    refusal.append(dict(name='unapproved_native_launch',rejected=True,stderr=process.stderr))
    process=subprocess.run([str(binary),'check-prepared',str(out),'0'*64],capture_output=True,text=True,timeout=60)
    assert process.returncode!=0 and 'prepared root' in process.stderr
    refusal.append(dict(name='wrong_prepared_root',rejected=True,stderr=process.stderr))
    # Disposable *copies*: mutation never touches a sealed historical/preflight file.
    for name,change in [('schedule_drift',lambda p:write(p/'schedule.json',[])),
                        ('config_drift',lambda p:write(p/'config.json',dict(d.design(),risk_draws=513))),
                        ('request_drift',lambda p:write(p/'request.json',dict(a.load(p/'request.json'),launch_authorized=True))),
                        ('source_snapshot_drift',lambda p:write(p/'source_snapshots'/d.CONFIG,{}))]:
        with tempfile.TemporaryDirectory(prefix='ere-feasible-guard-') as tmp:
            clone=Path(tmp)/'copy';shutil.copytree(out,clone)
            change(clone)
            process=subprocess.run([str(binary),'check-prepared',str(clone),root],capture_output=True,text=True,timeout=60)
            assert process.returncode!=0 and 'prepared input drift' in process.stderr,name+' not rejected at immutable guard'
            assert not (clone/'catalogue.json').exists()
            d.same(a.load(clone/'manifest.json'),d.initial())
            refusal.append(dict(name=name,rejected=True,stderr=process.stderr))
    # Check-only and refusals have not changed any prior prepared/archive bytes.
    for n,h in before.items():assert a.digest(out/n)==h,'guard command mutated original receipt'
    d.same(a.load(out/'manifest.json'),d.initial());assert not (out/'catalogue.json').exists()
    auditor_rejections=[]
    for name,change in [('training_admission',lambda p:write(p/'manifest.json',dict(d.initial(),training_admitted=True))),
                        ('invented_attempts',lambda p:write(p/'manifest.json',dict(d.initial(),attempted_steps=1))),
                        ('inflight_unknown',lambda p:write(p/'manifest.json',dict(d.initial(),inflight_native_work_unknown=True))),
                        ('invented_proposal',lambda p:write(p/'estimates/c0-e0.json',dict(context=0,estimate=0,
                            theta_before=d.contexts()[0],evaluation_ids=[q['id'] for q in d.schedule()[:528]],
                            complete=True,request=None,proposal=None)))]:
        with tempfile.TemporaryDirectory(prefix='ere-feasible-audit-') as tmp:
            clone=Path(tmp)/'copy';shutil.copytree(out,clone);change(clone);a.seal(clone)
            process=subprocess.run([sys.executable,'-B',str(out/'source_snapshots/scripts/audit_ere_categorical_feasible_development.py'),str(clone)],cwd=ROOT,capture_output=True,text=True,timeout=900)
            assert process.returncode!=0 and 'AssertionError' in process.stderr,name+' accepted'
            auditor_rejections.append(dict(name=name,rejected=True,stderr=process.stderr))
    for n,h in before.items():assert a.digest(out/n)==h,'detached audit mutated original receipt'
    return dict(engineering_pass=True,zero_native_work=True,check_only_pass=True,regression_groups=6,
        refusals=refusal,auditor_rejections=auditor_rejections)


def prepare(out,execute=False,preflight=None):
    out=Path(out).absolute()
    assert out.parent.resolve()==(ROOT/'output').resolve() and not out.exists(),'fresh direct output child required'
    assert type(execute) is bool and (execute==(preflight is not None)),'execute requires explicit source-identical verified preflight'
    start=time.monotonic();d.same(a.load(ROOT/d.CONFIG),d.design())
    reg=d.registration(ROOT,verify_lineage=True);seeds=d.seed_inventory()
    a.verify(v.legacy.PARENT,expected=v.legacy.PARENT_SEAL,count=1690)
    a.verify(v.legacy.RUNTIME,expected=v.legacy.RUNTIME_SEAL,count=460)
    hashes={n:a.digest(ROOT/n) for n in reg['source_names']}
    out.mkdir()
    for directory in ('cells','estimates','directions','validation'):(out/directory).mkdir()
    for n,h in hashes.items():
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(ROOT/n,dest);assert a.digest(dest)==h
    write(out/'source_registration.json',reg)
    try:
        build=command(out,'release_build',d.BUILD);live(hashes)
        binary=ROOT/'target/release'/d.BIN;shutil.copy2(binary,out/'executed_binary')
        snap=dict(sources=hashes,binary_sha256=a.digest(out/'executed_binary'),frozen_before_measurement=True,
            build_features=['ere-complete-signal'],default_features=False,repository=str(ROOT))
        write(out/'snapshot.json',snap)
        write(out/'build_receipt.json',dict(command=d.BUILD,returncode=0,source_hashes_before=hashes,
            source_hashes_after=hashes,binary_sha256=snap['binary_sha256'],seed_collision_check=seeds,
            stdout=build.stdout,stderr=build.stderr))
        ref=None
        if execute:
            pre=Path(preflight).absolute();a.verify(pre)
            ref=dict(path=str(pre),root_seal=a.digest(pre/'output_sha256.json'))
            v.preflight_reference(ref,snap,out)
        write(out/'launch_preflight.json',ref);write(out/'seed_inventory.json',seeds)
        shutil.copyfile(ROOT/d.CONFIG,out/'config.json')
        shutil.copyfile(d.BASELINE/'catalogue.json',out/'reference_catalogue.json')
        plan=d.schedule();write(out/'schedule.json',plan);write(out/'contexts.json',d.contexts())
        for q in plan:write(out/'cells'/(q['id']+'.json'),dict(schedule=q,status='NotRun',reason='not_started',evaluation=None))
        write(out/'initial_manifest.json',d.initial());write(out/'manifest.json',d.initial());write(out/'summary.json',d.summary(False))
        req=dict(profile=d.PROFILE,mode='development',launch_authorized=execute,full_training_authorized=False,
            setup_elapsed_seconds=time.monotonic()-start)
        assert req['setup_elapsed_seconds']<900;write(out/'request.json',req)
        immutable=v.IMMUTABLE|{'source_snapshots/'+n for n in hashes}
        write(out/'prepared_sha256.json',{n:a.digest(out/n) for n in sorted(immutable)})
        live(hashes);a.seal(out)
        zero=detached_audit(out)
        assert zero['classification']=='zero_work_preflight' and zero['recorded_cells']==0
        if not execute:
            command(out,'rust_tests',['cargo','test','--offline','--locked','-p','hg-train','--no-default-features','--features','ere-complete-signal','--lib','ere::categorical_feasible_development::tests'])
            live(hashes)
            command(out,'adapter_tests',['cargo','test','--offline','--locked','-p','hg-train','--no-default-features','--features','ere-complete-signal','--lib','ere::categorical_feasible::tests'])
            command(out,'categorical_tests',['cargo','test','--offline','--locked','-p','hg-train','--no-default-features','--features','ere-complete-signal','--lib','ere::categorical::tests'])
            command(out,'feature_disabled',['cargo','check','--offline','--locked','-p','hg-train','--no-default-features','--lib'])
            command(out,'python_tests',[sys.executable,'-B','-m','unittest','-v','test_ere_categorical_feasible_development'],cwd=out/'source_snapshots/scripts',timeout=180)
            live(hashes);a.seal(out)
            guards=verify_guards(out);write(out/'guard_verification.json',guards);a.seal(out)
        else:
            d.same(d.seed_inventory(exclude=out),seeds);live(hashes)
            process=subprocess.run([str(out/'executed_binary'),'run-prepared',str(out),a.digest(out/'prepared_sha256.json')],cwd=ROOT)
            m=a.load(out/'manifest.json')
            if process.returncode!=0 and m['status'] not in ('incomplete','blocked_before_evaluation'):
                m.update(status='blocked_before_evaluation' if m['recorded_cells']==0 else 'incomplete',reason=f'native process exit {process.returncode}')
                write(out/'manifest.json',m);a.seal(out)
        live(hashes);result=detached_audit(out)
        write(out/'audit.json',result);a.seal(out);a.verify(out)
        print(json.dumps(dict(archive=str(out),root_seal=a.digest(out/'output_sha256.json'),**result),indent=2))
        return out
    except BaseException as error:
        write(out/'preparation_failure.json',dict(error_type=type(error).__name__,reason=str(error),automatic_retry=False))
        a.seal(out);raise


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--execute',action='store_true',help='Only after separate approval; runs 2,176 native evaluations')
    parser.add_argument('--preflight',type=Path,help='Required with --execute; verified source/binary-identical zero-work archive')
    args=parser.parse_args();prepare(args.output,args.execute,args.preflight)
