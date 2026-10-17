#!/usr/bin/env python3
"""One source-frozen paired graph-aware/blind learning trial. No native retry."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_rule_relational as previous
import verify_ere_rule_relational_learning as verify

a=verify.a
ROOT=previous.ROOT
FIXTURE=ROOT/'output/ere_rule_relational_development_20261004'
FIXTURE_SEAL='58949cc2b129ae8cc845c3068fb5eb5d8ca7a5ffc8d88f711f6384b4fda17bcd'
EXTRA=['configs/ere_rule_relational_learning_v1.json','docs/ere_rule_relational_learning_v1.md',
    'scripts/run_ere_rule_relational_learning.py','scripts/verify_ere_rule_relational_learning.py',
    'scripts/test_verify_ere_rule_relational_learning.py','crates/hg-train/tests/ere_rule_relational_learning.rs']


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def source_names():return sorted(set(previous.source_names())|set(EXTRA))


def preservation():
    a.verify(FIXTURE,expected=FIXTURE_SEAL,count=643)
    old=a.load(FIXTURE/'manifest.json')['sources'];changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    assert changed==['crates/hg-train/src/ere.rs'],'unexpected prior active source drift'
    name=changed[0];current=(ROOT/name).read_text();frozen=(FIXTURE/'source_snapshots'/name).read_text()
    export='pub mod rule_relational_learning;\n'
    assert current.count(export)==1 and current.replace(export,'',1)==frozen,'not export-only extension'
    a.verify(previous.LEARNING,expected=previous.LEARNING_SEAL,count=1445)
    chain=previous.previous
    a.verify(chain.corrected.INPUT,expected=chain.corrected.INPUT_SEAL,count=617)
    a.verify(chain.SUPPLEMENT,expected=chain.SUPPLEMENT_SEAL,count=11)
    a.verify(chain.previous.LEARNING,expected=chain.previous.LEARNING_SEAL,count=758)
    choice=chain.previous.previous
    a.verify(choice.ADAPTER,expected=choice.ADAPTER_SEAL,count=283)
    a.verify(choice.FAILED,expected=choice.FAILED_SEAL)
    lifecycle=choice.previous.historical_check()
    return dict(fixture_seal_sha256=FIXTURE_SEAL,fixture_entries=643,prior_active_source_matches=len(old)-1,
        export_only_drift=changed,shared_learning_seal_sha256=previous.LEARNING_SEAL,
        original_composition_seal_sha256=chain.corrected.INPUT_SEAL,corrected_supplement_seal_sha256=chain.SUPPLEMENT_SEAL,
        choice_learning_seal_sha256=chain.previous.LEARNING_SEAL,adapter_seal_sha256=choice.ADAPTER_SEAL,
        failed_preflight_seal_sha256=choice.FAILED_SEAL,lifecycle=lifecycle)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True)
    out=p.parse_args().output.resolve();assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for name in names:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
    design=a.load(ROOT/EXTRA[0])
    m=dict(profile=verify.PROFILE,status='running',classification='bounded_paired_relational_learning',
        design=design,sources=sources,bounded_learning_authorized=True,full_training_authorized=False,training_admitted=False,
        scientific_acceptance=False,independent_physics_replay=False,maximum_native_attempts=42768,
        native_launched=False,native_work_unknown=False)
    save(out/'manifest.json',m)
    try:
        assert design==verify.DESIGN,'prospective design mismatch'
        before=previous.features();save(out/'feature_scope_before.json',before)
        history=preservation();save(out/'historical_preservation.json',history)
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_rule_relational_learning',
            'matched_policy_update_refusals','--','--exact','--nocapture']
        r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'optimizer_preflight_test.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
        assert r.returncode==0,'policy/update preflight failed before native work'
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_relational_learning.py','-v']
        r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=90)
        save(out/'auditor_preflight_test.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
        assert r.returncode==0,'synthetic accounting/tamper preflight failed before native work'
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'prelaunch source drift'
        env=dict(os.environ,ERE_RULE_RELATIONAL_LEARNING_OUTPUT=str(out/'native'))
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_rule_relational_learning',
            'fixed_native_relational_learning_trial','--','--exact','--nocapture']
        m.update(native_launched=True,native_work_unknown=True);save(out/'manifest.json',m)
        r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=540)
        save(out/'native_test_receipt.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
        assert r.returncode==0,'fixed trial failed; retained without retry'
        result=verify.audit(out/'native');save(out/'audit.json',result)
        env=dict(os.environ,ERE_RULE_RELATIONAL_LEARNING_AUDIT_INPUT=str(out/'native'))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_relational_learning.py','-v']
        r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=180)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
        assert r.returncode==0,'independent native/decision/update accounting failed'
        command=['cargo','check','--offline','--locked','-p','hg-train','--no-default-features']
        r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'feature_off_check.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'measurement source drift'
        after=previous.features();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert history==after
        m.update(status='complete',native_work_unknown=False,native_attempts=42768,advancing_steps=42768,completed_updates=72,
            learning_criterion_pass=result['learning_criterion_pass'],pre_emission_decisions_checked=864,
            executed_binary_sha256=a.digest(out/'native/executed_test_binary'),elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),native_work_unknown=m['native_launched'],native_attempts=None if m['native_launched'] else 0,elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out)
    print(json.dumps(dict(output=str(out),status='complete',native_attempts=42768,completed_updates=72,
        accounting_pass=True,learning_criterion_pass=result['learning_criterion_pass'],seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
