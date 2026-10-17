#!/usr/bin/env python3
"""One prospectively frozen shared composition learning trial; no native retries."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_rule_composition as previous
import audit_ere_rule_composition_v2 as corrected
import verify_ere_rule_composition_learning as verify

a=verify.a
ROOT=previous.ROOT
SUPPLEMENT=ROOT/'output/ere_rule_composition_archive_audit_v2_20261004'
SUPPLEMENT_SEAL='6c1e8fb8a746c60919ab220f75e5cd46198b762a329a981c8c1c47fdd775163c'
EXTRA=['configs/ere_rule_composition_learning_v1.json','docs/ere_rule_composition_learning_v1.md',
    'scripts/run_ere_rule_composition_learning.py','scripts/verify_ere_rule_composition_learning.py',
    'scripts/test_verify_ere_rule_composition_learning.py','crates/hg-train/tests/ere_rule_composition_learning.rs']


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def source_names():return sorted(set(previous.source_names())|set(corrected.EXTRA)|set(EXTRA))


def preservation():
    a.verify(corrected.INPUT,expected=corrected.INPUT_SEAL,count=617)
    a.verify(SUPPLEMENT,expected=SUPPLEMENT_SEAL,count=11)
    old=a.load(corrected.INPUT/'manifest.json')['sources']
    changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    assert changed==['crates/hg-train/src/ere.rs'],'unexpected active historical source drift'
    name=changed[0];current=(ROOT/name).read_text();frozen=(corrected.INPUT/'source_snapshots'/name).read_text()
    export='pub mod rule_composition_learning;\n'
    assert current.count(export)==1 and current.replace(export,'',1)==frozen,'not an additive export'
    supplemental=a.load(SUPPLEMENT/'manifest.json')['sources']
    assert supplemental=={n:a.digest(ROOT/n) for n in supplemental},'corrected proof/auditor drift'
    a.verify(previous.LEARNING,expected=previous.LEARNING_SEAL,count=758)
    learning=previous.previous
    a.verify(learning.ADAPTER,expected=learning.ADAPTER_SEAL,count=283)
    a.verify(learning.FAILED,expected=learning.FAILED_SEAL)
    lifecycle=learning.previous.historical_check()
    return dict(original_seal_sha256=corrected.INPUT_SEAL,original_entries=617,
        supplemental_seal_sha256=SUPPLEMENT_SEAL,supplemental_entries=11,
        original_source_matches=len(old)-1,export_only_drift=changed,corrected_source_matches=len(supplemental),
        learning_seal_sha256=previous.LEARNING_SEAL,adapter_seal_sha256=learning.ADAPTER_SEAL,
        failed_preflight_seal_sha256=learning.FAILED_SEAL,lifecycle=lifecycle)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True)
    out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for name in names:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
    design=a.load(ROOT/EXTRA[0])
    m=dict(profile=verify.PROFILE,status='running',classification='bounded_shared_composition_learning',
        design=design,sources=sources,bounded_learning_authorized=True,full_training_authorized=False,
        training_admitted=False,scientific_acceptance=False,independent_physics_replay=False,
        maximum_native_attempts=28944,native_work_unknown=True,native_launched=False)
    save(out/'manifest.json',m)
    try:
        assert design==verify.DESIGN,'prospective design mismatch'
        before=previous.previous.features();save(out/'feature_scope_before.json',before)
        history=preservation();save(out/'historical_preservation.json',history)
        # Synthetic/update tests only: no fixture launches before frozen campaign.
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test',
            'ere_rule_composition_learning','fixed_composition_update_refusals','--','--exact','--nocapture']
        check=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'optimizer_preflight_test.json',dict(command=command,returncode=check.returncode,stdout=check.stdout,stderr=check.stderr))
        assert check.returncode==0,'optimizer preflight failed before native work'
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_composition_learning.py','-v']
        check=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=60)
        save(out/'auditor_preflight_test.json',dict(command=command,returncode=check.returncode,stdout=check.stdout,stderr=check.stderr))
        assert check.returncode==0,'auditor synthetic preflight failed before native work'
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'prelaunch source drift'
        env=dict(os.environ,ERE_RULE_COMPOSITION_LEARNING_OUTPUT=str(out/'native'))
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test',
            'ere_rule_composition_learning','fixed_native_composition_learning_trial','--','--exact','--nocapture']
        m['native_launched']=True;save(out/'manifest.json',m)
        receipt=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=420)
        save(out/'native_test_receipt.json',dict(command=command,returncode=receipt.returncode,stdout=receipt.stdout,stderr=receipt.stderr))
        assert receipt.returncode==0,'trial failed; preserved, no retry'
        result=verify.audit(out/'native');save(out/'audit.json',result)
        env=dict(os.environ,ERE_RULE_COMPOSITION_LEARNING_AUDIT_INPUT=str(out/'native'))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_composition_learning.py','-v']
        receipt=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=120)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=receipt.returncode,stdout=receipt.stdout,stderr=receipt.stderr))
        assert receipt.returncode==0,'independent accounting/tamper check failed'
        command=['cargo','check','--offline','--locked','-p','hg-train','--no-default-features']
        check=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'feature_off_check.json',dict(command=command,returncode=check.returncode,stdout=check.stdout,stderr=check.stderr))
        assert check.returncode==0,'feature-off regression'
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'measurement source drift'
        after=previous.previous.features();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert history==after
        m.update(status='complete',native_work_unknown=False,native_attempts=28944,advancing_steps=28944,
            completed_updates=72,learning_criterion_pass=result['learning_criterion_pass'],
            executed_binary_sha256=a.digest(out/'native/executed_test_binary'),elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),native_work_unknown=m['native_launched'],
            native_attempts=None if m['native_launched'] else 0,elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out)
    print(json.dumps(dict(output=str(out),status='complete',native_attempts=28944,completed_updates=72,
        accounting_pass=True,learning_criterion_pass=result['learning_criterion_pass'],
        full_training_authorized=False,seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
