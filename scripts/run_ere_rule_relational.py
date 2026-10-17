#!/usr/bin/env python3
"""One frozen relational engineering panel; no learning or native retries."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_rule_composition_learning as previous
import verify_ere_rule_relational as verify

a=verify.integrity
ROOT=previous.ROOT
LEARNING=ROOT/'output/ere_rule_composition_learning_trial_20261004'
LEARNING_SEAL='e6b6ef1faa22f7c2a1af0787154fc7b13a3811bc038e4f4ac9b35babc5ba262f'
EXTRA=['configs/ere_rule_relational_v1.json','docs/ere_rule_relational_v1.md',
    'scripts/run_ere_rule_relational.py','scripts/verify_ere_rule_relational.py',
    'scripts/test_verify_ere_rule_relational.py','crates/hg-train/tests/ere_rule_relational.rs']


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def source_names():return sorted(set(previous.source_names())|set(EXTRA))


def preservation():
    a.verify(LEARNING,expected=LEARNING_SEAL,count=1445)
    old=a.load(LEARNING/'manifest.json')['sources']
    changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    assert changed==['crates/hg-sim-core/src/ere.rs','crates/hg-train/src/ere.rs'],'unexpected active source drift'
    for n in changed:
        current=(ROOT/n).read_text();frozen=(LEARNING/'source_snapshots'/n).read_text();export='pub mod rule_relational;\n'
        assert current.count(export)==1 and current.replace(export,'',1)==frozen,'not export-only extension'
    # Verify every historical output rather than call older live guards whose
    # exact export assumptions intentionally predate this new version.
    a.verify(previous.corrected.INPUT,expected=previous.corrected.INPUT_SEAL,count=617)
    a.verify(previous.SUPPLEMENT,expected=previous.SUPPLEMENT_SEAL,count=11)
    choice=previous.previous.previous
    a.verify(previous.previous.LEARNING,expected=previous.previous.LEARNING_SEAL,count=758)
    a.verify(choice.ADAPTER,expected=choice.ADAPTER_SEAL,count=283)
    a.verify(choice.FAILED,expected=choice.FAILED_SEAL)
    lifecycle=choice.previous.historical_check()
    return dict(learning_seal_sha256=LEARNING_SEAL,learning_entries=1445,prior_active_source_matches=len(old)-2,
        export_only_drift=changed,original_composition_seal_sha256=previous.corrected.INPUT_SEAL,
        corrected_supplement_seal_sha256=previous.SUPPLEMENT_SEAL,choice_learning_seal_sha256=previous.previous.LEARNING_SEAL,
        adapter_seal_sha256=choice.ADAPTER_SEAL,failed_preflight_seal_sha256=choice.FAILED_SEAL,lifecycle=lifecycle)


def features():return previous.previous.previous.features()


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True)
    out=p.parse_args().output.resolve();assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for name in names:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
    design=a.load(ROOT/EXTRA[0])
    m=dict(profile=verify.PROFILE,status='running',classification='relation_sensitive_engineering_fixture',
        design=design,sources=sources,training_updates=0,full_training_authorized=False,training_admitted=False,
        scientific_acceptance=False,independent_physics_replay=False,maximum_native_attempts=9504,
        native_launched=False,native_work_unknown=False)
    save(out/'manifest.json',m)
    try:
        assert design==verify.DESIGN,'prospective design mismatch'
        before=features();save(out/'feature_scope_before.json',before)
        history=preservation();save(out/'historical_preservation.json',history)
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_rule_relational',
            'graph_observation_gradients_and_refusals','--','--exact','--nocapture']
        pre=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'observation_preflight_test.json',dict(command=command,returncode=pre.returncode,stdout=pre.stdout,stderr=pre.stderr))
        assert pre.returncode==0,'observation/arithmetic preflight failed before native work'
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_relational.py','-v']
        pre=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=90)
        save(out/'auditor_preflight_test.json',dict(command=command,returncode=pre.returncode,stdout=pre.stdout,stderr=pre.stderr))
        assert pre.returncode==0,'synthetic auditor preflight failed before native work'
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'prelaunch source drift'
        env=dict(os.environ,ERE_RULE_RELATIONAL_OUTPUT=str(out/'native'))
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_rule_relational',
            'registered_native_relational_panel','--','--exact','--nocapture']
        m.update(native_launched=True,native_work_unknown=True);save(out/'manifest.json',m)
        r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=360)
        save(out/'native_test_receipt.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
        assert r.returncode==0,'native panel failed; retained, no retry'
        result=verify.audit_panel(out/'native');save(out/'audit.json',result)
        env=dict(os.environ,ERE_RULE_RELATIONAL_AUDIT_INPUT=str(out/'native'))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_relational.py','-v']
        r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=90)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
        assert r.returncode==0,'independent ledger/policy/observation audit failed'
        command=['cargo','check','--offline','--locked','-p','hg-train','--no-default-features']
        r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'feature_off_check.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'measurement source drift'
        after=features();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert history==after
        m.update(status='complete',native_work_unknown=False,native_attempts=9504,advancing_steps=9504,
            executed_binary_sha256=a.digest(out/'native/executed_test_binary'),elapsed_seconds=time.time()-start,
            accounting_pass=True,graph_probe_mean_reward=result['policy']['graph_probe_mean_reward'],
            category_only_upper_bound=1/24)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),native_work_unknown=m['native_launched'],native_attempts=None if m['native_launched'] else 0,elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out)
    print(json.dumps(dict(output=str(out),status='complete',native_attempts=9504,training_updates=0,
        accounting_pass=True,policy=result['policy'],seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
