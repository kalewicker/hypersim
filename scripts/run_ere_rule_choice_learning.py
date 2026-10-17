#!/usr/bin/env python3
"""Launch exactly one fixed tiny native learning trial into a fresh sealed archive."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_rule_choice_adapter as previous
import verify_ere_rule_choice_learning as verify

a=verify.a
ROOT=previous.ROOT
EXTRA=['configs/ere_rule_choice_learning_v2.json','docs/ere_rule_choice_learning_v2.md',
       'configs/ere_rule_choice_learning_v1.json','docs/ere_rule_choice_learning_v1.md',
       'scripts/run_ere_rule_choice_learning.py','scripts/verify_ere_rule_choice_learning.py',
       'scripts/test_verify_ere_rule_choice_learning.py','crates/hg-train/tests/ere_rule_choice_learning.rs']
ADAPTER=ROOT/'output/ere_rule_choice_adapter_development_20261004'
ADAPTER_SEAL='2d8e6c7378c9f73a3cdfee9c791c636981d8a4ddd70a1166408f8a6a4a95b5d2'
FAILED=ROOT/'output/ere_rule_choice_learning_trial_20261004'
FAILED_SEAL='c4059b1c62715a413add601eece8172da44ae384821a0354a814cce4b050091f'


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def inactive(name):
    return name.startswith('crates/hg-sim-core/src/evolving_minds/') or name=='crates/hg-sim-core/src/bin/evolving_minds.rs'


def source_names():return sorted(n for n in set(previous.source_names())|set(EXTRA) if not inactive(n))


def features():
    command=['cargo','tree','--offline','--locked','-p','hg-train','--features','ere',
             '--prefix','none','--format','{p} features={f}']
    r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=30)
    assert r.returncode==0,'feature inventory failed'
    rows=[line for line in r.stdout.splitlines() if line.startswith('hg-sim-core v')]
    assert rows and all(set(line.split('features=')[1].split(' ')[0].split(','))=={'cognition','hg-meta'} for line in rows)
    return dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr,
                sim_core_features=['cognition','hg-meta'],evolving_minds_enabled=False)


def preservation():
    a.verify(ADAPTER,expected=ADAPTER_SEAL,count=283)
    old=a.load(ADAPTER/'manifest.json')['sources']
    changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    active=[n for n in changed if not inactive(n)]
    assert active==['crates/hg-train/src/ere.rs'],'unexpected active adapter-body drift'
    name=active[0];current=(ROOT/name).read_text();frozen=(ADAPTER/'source_snapshots'/name).read_text()
    assert current.count('pub mod rule_choice_learning;\n')==1
    assert current.replace('pub mod rule_choice_learning;\n','',1)==frozen,'not an export-only extension'
    a.verify(FAILED,expected=FAILED_SEAL)
    assert not (FAILED/'native').exists() and not (FAILED/'native_test_receipt.json').exists()
    return dict(adapter_seal_sha256=ADAPTER_SEAL,adapter_entries=283,
        prior_active_source_matches=sum(not inactive(n) for n in old)-1,export_only_drift=active,
        inactive_historical_drift=[n for n in changed if inactive(n)],
        failed_preflight_seal_sha256=FAILED_SEAL,failed_preflight_native_launched=False,
        lifecycle=previous.historical_check())


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True)
    out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'requires fresh output under project output/'
    out.mkdir();start=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for name in names:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
    design=a.load(ROOT/EXTRA[0])
    manifest=dict(profile=verify.PROFILE,status='running',classification='bounded_learning_mechanism_trial',
        design=design,sources=sources,bounded_learning_authorized=True,full_training_authorized=False,
        training_admitted=False,scientific_acceptance=False,independent_physics_replay=False,
        maximum_native_attempts=12960,native_work_unknown=True)
    save(out/'manifest.json',manifest)
    try:
        assert design==verify.DESIGN,'prospective design mismatch'
        save(out/'feature_scope_before.json',features())
        save(out/'historical_preservation.json',preservation())
        env=dict(os.environ,ERE_RULE_CHOICE_LEARNING_OUTPUT=str(out/'native'))
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere',
                 '--test','ere_rule_choice_learning','--','--nocapture']
        native=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=300)
        save(out/'native_test_receipt.json',dict(command=command,returncode=native.returncode,stdout=native.stdout,stderr=native.stderr))
        assert native.returncode==0,'fixed trial failed; preserved, no automatic retry'
        result=verify.audit(out/'native');save(out/'audit.json',result)
        env=dict(os.environ,ERE_RULE_CHOICE_LEARNING_AUDIT_INPUT=str(out/'native'))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_choice_learning.py','-v']
        tests=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=90)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=tests.returncode,stdout=tests.stdout,stderr=tests.stderr))
        assert tests.returncode==0,'independent accounting/controls failed'
        command=['cargo','check','--offline','--locked','-p','hg-train','--no-default-features']
        check=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'feature_off_check.json',dict(command=command,returncode=check.returncode,stdout=check.stdout,stderr=check.stderr))
        assert check.returncode==0,'feature-off regression'
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'measurement source drift'
        after=features();save(out/'feature_scope_after.json',after)
        assert after==a.load(out/'feature_scope_before.json'),'feature drift'
        # Disabled-feature work may continue; no active pin is waived.
        historical_after=preservation();save(out/'historical_preservation_after.json',historical_after)
        before=a.load(out/'historical_preservation.json')
        assert {k:v for k,v in historical_after.items() if k!='inactive_historical_drift'}=={k:v for k,v in before.items() if k!='inactive_historical_drift'}
        manifest.update(status='complete',native_work_unknown=False,native_attempts=12960,advancing_steps=12960,
            completed_updates=72,learning_criterion_pass=result['learning_criterion_pass'],
            native_test_groups=2,python_test_groups=4,detached_tamper_rejections=22,
            executed_binary_sha256=a.digest(out/'native/executed_test_binary'),elapsed_seconds=time.time()-start)
        save(out/'manifest.json',manifest)
    except Exception as error:
        manifest.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start)
        save(out/'manifest.json',manifest);a.seal(out);raise
    a.seal(out)
    print(json.dumps(dict(output=str(out),status='complete',native_attempts=12960,completed_updates=72,
        accounting_pass=True,learning_criterion_pass=result['learning_criterion_pass'],
        full_training_authorized=False,seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
