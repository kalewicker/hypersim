#!/usr/bin/env python3
"""Run the fixed two-paid-edit engineering panel once with feature-scoped provenance."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_rule_choice_learning as previous
import verify_ere_rule_composition as verify

a=verify.integrity
ROOT=previous.ROOT
EXTRA=['configs/ere_rule_composition_v1.json','docs/ere_rule_composition_v1.md',
       'scripts/run_ere_rule_composition.py','scripts/verify_ere_rule_composition.py',
       'scripts/test_verify_ere_rule_composition.py','crates/hg-train/tests/ere_rule_composition.rs']
LEARNING=ROOT/'output/ere_rule_choice_learning_trial_v2_20261004'
LEARNING_SEAL='94ee4da611d6a1232f573770b102b6f573494cd9571b99df67cf014963317b63'


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def source_names():return sorted(set(previous.source_names())|set(EXTRA))


def preservation():
    a.verify(LEARNING,expected=LEARNING_SEAL,count=758)
    old=a.load(LEARNING/'manifest.json')['sources']
    changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    assert changed==['crates/hg-sim-core/src/ere.rs','crates/hg-train/src/ere.rs'],'unexpected active source drift'
    for name in changed:
        current=(ROOT/name).read_text();frozen=(LEARNING/'source_snapshots'/name).read_text()
        assert current.count('pub mod rule_composition;\n')==1
        assert current.replace('pub mod rule_composition;\n','',1)==frozen,'not an export-only extension'
    a.verify(previous.ADAPTER,expected=previous.ADAPTER_SEAL,count=283)
    a.verify(previous.FAILED,expected=previous.FAILED_SEAL)
    lifecycle=previous.previous.historical_check()
    return dict(learning_seal_sha256=LEARNING_SEAL,learning_entries=758,
        prior_active_source_matches=len(old)-2,export_only_drift=changed,
        adapter_seal_sha256=previous.ADAPTER_SEAL,failed_preflight_seal_sha256=previous.FAILED_SEAL,
        lifecycle=lifecycle)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True)
    out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'requires fresh project output/'
    out.mkdir();start=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for name in names:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
    design=a.load(ROOT/EXTRA[0])
    manifest=dict(profile=verify.PROFILE,status='running',classification='two_edit_engineering_fixture',
        design=design,sources=sources,training_updates=0,training_admitted=False,scientific_acceptance=False,
        full_training_authorized=False,independent_physics_replay=False,
        maximum_native_attempts=9504,native_work_unknown=True)
    save(out/'manifest.json',manifest)
    launched=False
    try:
        assert design==verify.DESIGN,'prospective design mismatch'
        save(out/'feature_scope_before.json',previous.features())
        save(out/'historical_preservation.json',preservation())
        env=dict(os.environ,ERE_RULE_COMPOSITION_OUTPUT=str(out/'native'))
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere',
                 '--test','ere_rule_composition','--','--nocapture']
        launched=True
        native=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=300)
        save(out/'native_test_receipt.json',dict(command=command,returncode=native.returncode,stdout=native.stdout,stderr=native.stderr))
        assert native.returncode==0,'fixture failed; retained, no automatic retry'
        result=verify.audit_panel(out/'native');save(out/'audit.json',result)
        env=dict(os.environ,ERE_RULE_COMPOSITION_AUDIT_INPUT=str(out/'native'))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_composition.py','-v']
        tests=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=90)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=tests.returncode,stdout=tests.stdout,stderr=tests.stderr))
        assert tests.returncode==0,'independent accounting/controls failed'
        command=['cargo','check','--offline','--locked','-p','hg-train','--no-default-features']
        check=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'feature_off_check.json',dict(command=command,returncode=check.returncode,stdout=check.stdout,stderr=check.stderr))
        assert check.returncode==0,'feature-off regression'
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'measurement source drift'
        after=previous.features();save(out/'feature_scope_after.json',after)
        assert after==a.load(out/'feature_scope_before.json'),'feature drift'
        assert preservation()==a.load(out/'historical_preservation.json')
        manifest.update(status='complete',native_work_unknown=False,native_attempts=9504,advancing_steps=9504,
            native_test_groups=2,python_test_groups=4,detached_native_tamper_rejections=24,
            detached_policy_tamper_rejections=4,executed_binary_sha256=a.digest(out/'native/executed_test_binary'),
            elapsed_seconds=time.time()-start)
        save(out/'manifest.json',manifest)
    except Exception as error:
        manifest.update(status='failed',error=repr(error),native_launched=launched,
            native_work_unknown=launched,native_attempts=None if launched else 0,elapsed_seconds=time.time()-start)
        save(out/'manifest.json',manifest);a.seal(out);raise
    a.seal(out)
    print(json.dumps(dict(output=str(out),status='complete',native_attempts=9504,training_updates=0,
        accounting_pass=True,novel_fires=result['novel_fires'],seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
