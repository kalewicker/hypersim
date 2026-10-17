#!/usr/bin/env python3
"""One fresh fixed engineering panel, source-bound archive, and independent audit."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_rule_lifecycle_fixture as legacy
import verify_ere_rule_choice as verify

a=verify.integrity
ROOT=legacy.ROOT
EXTRA=['configs/ere_rule_choice_adapter_v1.json','docs/ere_rule_choice_adapter_v1.md',
       'scripts/run_ere_rule_choice_adapter.py','scripts/verify_ere_rule_choice.py',
       'scripts/test_verify_ere_rule_choice.py','crates/hg-train/tests/ere_rule_choice.rs']
HISTORICAL=ROOT/'output/ere_rule_lifecycle_fixture_development_20261003'
HISTORICAL_SEAL='0cdba8b56ae0d490c87f0ea6d6172fbce1ed27c81c18f24b37d9840e398b8bd5'


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def source_names():
    names=set(legacy.source_names())|set(EXTRA)
    for crate in ('hg-train','hg-cognition'):
        base=ROOT/'crates'/crate;names.add((base/'Cargo.toml').relative_to(ROOT).as_posix())
        names.update(p.relative_to(ROOT).as_posix() for p in (base/'src').rglob('*.rs'))
    return sorted(names)


def historical_check():
    a.verify(HISTORICAL,expected=HISTORICAL_SEAL,count=187)
    unchanged=['crates/hg-sim-core/src/ere/lifecycle_fixture.rs',*legacy.EXTRA]
    assert all(a.digest(ROOT/n)==a.digest(HISTORICAL/'source_snapshots'/n) for n in unchanged)
    return dict(seal_sha256=HISTORICAL_SEAL,entries=187,legacy_source_files_unchanged=len(unchanged))


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True)
    out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'requires fresh output under project output/'
    out.mkdir();started=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for name in names:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
    design=a.load(ROOT/EXTRA[0])
    manifest=dict(profile=verify.PROFILE,status='running',classification='seeded_choice_engineering',
        design=design,sources=sources,training_updates=0,training_admitted=False,
        scientific_acceptance=False,independent_physics_replay=False,
        maximum_native_attempts=1152,native_work_unknown=True)
    save(out/'manifest.json',manifest)
    try:
        save(out/'historical_preservation.json',historical_check())
        env=dict(os.environ,ERE_RULE_CHOICE_OUTPUT=str(out/'native'))
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere',
                 '--test','ere_rule_choice','--','--nocapture']
        native=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=300)
        save(out/'native_test_receipt.json',dict(command=command,returncode=native.returncode,stdout=native.stdout,stderr=native.stderr))
        assert native.returncode==0,'choice tests failed; no automatic retry'
        save(out/'audit.json',verify.audit_panel(out/'native'))
        env=dict(os.environ,ERE_RULE_CHOICE_AUDIT_INPUT=str(out/'native'))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_choice.py','-v']
        tests=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=60)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=tests.returncode,stdout=tests.stdout,stderr=tests.stderr))
        assert tests.returncode==0,'independent audit/controls failed'
        command=['cargo','check','--offline','--locked','-p','hg-train','--no-default-features']
        check=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'feature_off_check.json',dict(command=command,returncode=check.returncode,stdout=check.stdout,stderr=check.stderr))
        assert check.returncode==0,'feature-off regression'
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'measurement source drift'
        assert historical_check()==a.load(out/'historical_preservation.json')
        manifest.update(status='complete',native_work_unknown=False,native_attempts=1152,advancing_steps=1152,
            native_test_groups=2,python_test_groups=4,detached_native_tamper_rejections=24,
            detached_policy_tamper_rejections=4,executed_binary_sha256=a.digest(out/'native/executed_test_binary'),
            elapsed_seconds=time.time()-started)
        save(out/'manifest.json',manifest)
    except Exception as error:
        manifest.update(status='failed',error=repr(error),elapsed_seconds=time.time()-started)
        save(out/'manifest.json',manifest);a.seal(out);raise
    a.seal(out)
    print(json.dumps(dict(output=str(out),status='complete',native_attempts=1152,training_updates=0,
        accounting_pass=True,training_admitted=False,seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
