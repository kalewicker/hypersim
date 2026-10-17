#!/usr/bin/env python3
"""Frozen synthetic engineering verification; never compiles a grammar or runs physics."""
import argparse
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import audit_ere_categorical_feasible as v

NEW={
    'crates/hg-train/src/ere/categorical_feasible.rs',
    'crates/hg-train/src/bin/ere-categorical-feasible.rs',
    'scripts/audit_ere_categorical_feasible.py',
    'scripts/test_audit_ere_categorical_feasible.py',
    'scripts/verify_ere_categorical_feasible.py',
    'docs/ere_categorical_feasible_v1.md',
    'docs/ere_categorical_feasible_v2.md',
}
BIN='ere-categorical-feasible'
BASE=['cargo','--offline','--locked']


def write(path,value):path.write_text(json.dumps(value,indent=2))


def run(out):
    out=Path(out).absolute()
    assert not out.exists() and out.parent.resolve()==(v.ROOT/'output').resolve(),'fresh direct output child required'
    v.history.verify_measurement()
    prior=v.ROOT/'output/ere_categorical_feasible_engineering_20261003'
    prior_root='316ff752f98cf0e855524585ccf8b1f8bfcfc7197ed1609f41f4b2dd43a821f2'
    v.a.verify(prior,expected=prior_root,count=1145)
    assert v.a.load(prior/'manifest.json')['status']=='failed_engineering'
    hist=v.a.load(v.history.s.INPUT/'snapshot.json')['sources']
    assert v.a.digest(v.ROOT/'crates/hg-train/src/ere/categorical.rs')==hist['crates/hg-train/src/ere/categorical.rs'],'old categorical adapter changed'
    before=(v.history.s.FROZEN/'crates/hg-train/src/ere.rs').read_text()
    anchor='pub mod categorical_comparison_bridge;\n'
    assert before.count(anchor)==1
    assert (v.ROOT/'crates/hg-train/src/ere.rs').read_text()==before.replace(anchor,anchor+'#[cfg(feature = "ere-complete-signal")]\npub mod categorical_feasible;\n')
    cargo_append='\n[[bin]]\nname = "ere-categorical-feasible"\npath = "src/bin/ere-categorical-feasible.rs"\nrequired-features = ["ere-complete-signal"]\n'
    assert (v.ROOT/'crates/hg-train/Cargo.toml').read_text()==(v.history.s.FROZEN/'crates/hg-train/Cargo.toml').read_text()+cargo_append
    rust={str(p.relative_to(v.ROOT)) for p in (v.ROOT/'crates').rglob('*') if p.is_file() and p.suffix in ('.rs','.toml') and 'target' not in p.parts}
    support=set(v.a.load(v.ROOT/'output/ere_categorical_comparison_historical_audit_20261003/registration.json')['sources'])
    names=set(hist)|rust|NEW|support|{'Cargo.toml','Cargo.lock'}
    hashes={n:v.a.digest(v.ROOT/n) for n in sorted(names)}
    changed={n:dict(measured_sha256=h,current_sha256=hashes[n]) for n,h in hist.items() if hashes[n]!=h}
    assert set(changed)=={'Cargo.lock','crates/hg-sim-core/Cargo.toml','crates/hg-train/Cargo.toml','crates/hg-train/src/ere.rs'},'unexpected historical/shared source drift'
    out.mkdir()
    for n,h in hashes.items():
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(v.ROOT/n,dest);assert v.a.digest(dest)==h
    write(out/'registration.json',dict(profile=v.PROFILE,classification='engineering_synthetic_only',sources=hashes,
        frozen_before_build_and_invocation=True,historical_current_source_differences=changed,
        preserved_failed_prototype=dict(path=str(prior),root_seal=prior_root),
        new_sources=sorted(NEW),native_evaluations_allocated=0,native_steps_allocated=0,grammar_compilations_allocated=0,
        training_updates_allocated=0,validation_outcomes_used=False,automatic_retry=False))
    m=dict(profile=v.PROFILE,status='registered_before_checks',native_steps_run=0,grammar_compilations=0,training_updates=0,training_admitted=False)
    write(out/'manifest.json',m);v.a.seal(out);start=time.monotonic()
    def command(name,args):
        done=subprocess.run(args,cwd=v.ROOT,capture_output=True,text=True,timeout=600)
        write(out/(name+'.json'),dict(command=args,returncode=done.returncode,stdout=done.stdout,stderr=done.stderr))
        assert done.returncode==0,name+' failed; no retry'
        assert all(v.a.digest(v.ROOT/n)==h for n,h in hashes.items()),'source drift during build/check'
        return done
    try:
        command('rust_tests',['cargo','test','--offline','--locked','-p','hg-train','--no-default-features','--features','ere-complete-signal','--lib','ere::categorical_feasible::tests'])
        command('feature_disabled',['cargo','check','--offline','--locked','-p','hg-train','--no-default-features','--lib'])
        command('release_build',['cargo','build','--offline','--locked','--release','-p','hg-train','--no-default-features','--features','ere-complete-signal','--bin',BIN])
        shutil.copy2(v.ROOT/'target/release'/BIN,out/'executed_binary')
        binary_hash=v.a.digest(out/'executed_binary')
        shutil.copyfile(v.history.s.INPUT/'reference_catalogue.json',out/'catalogue.json')
        catalogue=v.a.load(out/'catalogue.json');request=v.fixture(catalogue);write(out/'request.json',request)
        frozen_inputs={n:v.a.digest(out/n) for n in v.a.files(out) if n not in {'output_sha256.json','manifest.json'}}
        write(out/'prepared_sha256.json',frozen_inputs);m['status']='frozen_before_offline_invocation';write(out/'manifest.json',m);v.a.seal(out)
        command('offline_invocation',[str(out/'executed_binary'),str(out/'catalogue.json'),str(out/'request.json'),str(out/'result.json')])
        result=v.a.load(out/'result.json')
        assert result['catalogue_sha256']==v.a.digest(out/'catalogue.json')
        assert result['request_sha256']==v.a.digest(out/'request.json')
        scripts=out/'source_snapshots/scripts';env=dict(os.environ,ERE_FEASIBLE_ENGINEERING_ARCHIVE=str(out))
        tests=subprocess.run([sys.executable,'-B','-m','unittest','-v','test_audit_ere_categorical_feasible'],cwd=scripts,env=env,capture_output=True,text=True,timeout=180)
        write(out/'python_tests.json',dict(returncode=tests.returncode,stdout=tests.stdout,stderr=tests.stderr))
        assert tests.returncode==0,'frozen Python tests failed; no retry'
        assert 'Ran 5 tests' in tests.stderr and json.loads(tests.stdout)['detached_result_controls']==10
        # Independently recompute with the frozen checker too, not the mutable copy.
        checked=subprocess.run([sys.executable,'-B','-c',
            'import json,sys; import audit_ere_categorical_feasible as v; from pathlib import Path; p=Path(sys.argv[1]); print(json.dumps(v.check_result(v.a.load(p/"request.json"),v.a.load(p/"result.json"),v.a.load(p/"catalogue.json"))))',str(out)],cwd=scripts,capture_output=True,text=True,timeout=180)
        write(out/'audit_process.json',dict(returncode=checked.returncode,stdout=checked.stdout,stderr=checked.stderr));assert checked.returncode==0,'frozen checker failed; no retry'
        audit=json.loads(checked.stdout)
        # Disposable synthetic request mutations; no measured archive is edited.
        controls=[('validation_stage',lambda x:x['cases'][0]['risk'].update(stage='validation')),
            ('cross_panel_id',lambda x:x['cases'][0]['risk']['samples'][0].update(id=x['cases'][0]['utility']['samples'][0]['id'])),
            ('cross_role_seed',lambda x:x['cases'][0]['risk']['samples'][0].update(physics_seed=x['cases'][0]['utility']['samples'][0]['emission_seed'])),
            ('missing_draw',lambda x:x['cases'][0]['risk'].update(expected_draws=513)),
            ('unknown_cap',lambda x:x['cases'][0]['risk']['samples'][0].update(native_score=None,reason='unknown_partial_work')),
            ('base_bounds',lambda x:x['cases'][0]['theta'].__setitem__(0,8.1)),
            ('scientific_purpose',lambda x:x.update(purpose='scientific_campaign'))]
        refusals=[];(out/'refusal_controls').mkdir()
        for name,change in controls:
            bad=copy.deepcopy(request);change(bad);path=out/'refusal_controls'/(name+'.json');write(path,bad);target=out/'refusal_controls'/(name+'-result.json')
            done=subprocess.run([str(out/'executed_binary'),str(out/'catalogue.json'),str(path),str(target)],capture_output=True,text=True,timeout=60)
            assert done.returncode!=0 and not target.exists(),name+' did not refuse before result'
            refusals.append(dict(name=name,rejected=True,stderr=done.stderr))
        old_result_hash=v.a.digest(out/'result.json')
        done=subprocess.run([str(out/'executed_binary'),str(out/'catalogue.json'),str(out/'request.json'),str(out/'result.json')],capture_output=True,text=True,timeout=60)
        assert done.returncode!=0 and v.a.digest(out/'result.json')==old_result_hash
        refusals.append(dict(name='overwrite',rejected=True,stderr=done.stderr));write(out/'binary_refusals.json',refusals)
        assert all(v.a.digest(out/n)==h for n,h in frozen_inputs.items())
        assert all(v.a.digest(v.ROOT/n)==h for n,h in hashes.items())
        assert v.a.digest(out/'executed_binary')==binary_hash
        v.history.verify_measurement()
        v.a.verify(prior,expected=prior_root,count=1145)
        v.a.verify(v.ROOT/'output/ere_categorical_comparison_historical_audit_20261003',expected='a33b236aaa4d2ade164f54e067be03f0a1313088cfef537ecfec1d8133f2e3e9',count=14)
        write(out/'audit.json',dict(**audit,passed_rust_groups=5,passed_python_groups=5,detached_result_rejections=10,binary_refusals=len(refusals),
            historical_roots_preserved=True,frozen_source_count=len(hashes),binary_sha256=binary_hash,
            selected_candidates={q['id']:q['proposal']['selected_candidate'] for q in result['cases']}))
        m.update(status='complete',engineering_arithmetic_pass=True,scientific_support=False,
            unseen_cap_feasibility_certificate=False,validation_used=False,source_count=len(hashes),binary_sha256=binary_hash)
    except Exception as error:
        m.update(status='failed_engineering',reason=f'{type(error).__name__}: {error}');raise
    finally:
        m['elapsed_seconds']=time.monotonic()-start;write(out/'manifest.json',m);v.a.seal(out)
    v.a.verify(out);print(json.dumps(dict(root_seal=v.a.digest(out/'output_sha256.json'),sealed_entries=len(v.a.load(out/'output_sha256.json')),**m),indent=2))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',required=True,type=Path);run(parser.parse_args().output)
