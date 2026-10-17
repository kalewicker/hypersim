#!/usr/bin/env python3
"""Run the fixed 432-step engineering test panel once into a fresh sealed archive."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

import verify_ere_rule_lifecycle as verify

a=verify.integrity
ROOT=next(p for p in Path(__file__).resolve().parents if (p/'output').is_dir() and (p/'Cargo.toml').is_file())
EXTRA=['configs/ere_rule_lifecycle_fixture_v1.json','docs/ere_rule_lifecycle_fixture_v1.md',
       'scripts/run_ere_rule_lifecycle_fixture.py','scripts/verify_ere_rule_lifecycle.py',
       'scripts/test_verify_ere_rule_lifecycle.py','scripts/audit_ere_complete_admission.py',
       'scripts/verify_ere_complete_witness.py','crates/hg-sim-core/tests/ere_rule_lifecycle.rs']


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def source_names():
    names=set(EXTRA)|{'Cargo.toml','Cargo.lock'}
    for crate in ['hg-core','hg-meta','hg-rules','hg-engine','hg-observe','hg-sim-core',
                  'hg-output','hg-schema','hg-eval','hg-causal','hg-report','hg-app-api']:
        base=ROOT/'crates'/crate;names.add((base/'Cargo.toml').relative_to(ROOT).as_posix())
        names.update(p.relative_to(ROOT).as_posix() for p in (base/'src').rglob('*.rs'))
    return sorted(names)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args();out=args.output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'requires fresh output under project output/'
    out.mkdir();started=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for name in names:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(ROOT/name,dest)
    design=a.load(ROOT/EXTRA[0])
    manifest=dict(profile=verify.PROFILE,status='running',classification='scripted_engineering_fixture',
        design=design,sources=sources,training_admitted=False,scientific_acceptance=False,
        independent_physics_replay=False,maximum_native_attempts=432,native_work_unknown=True)
    save(out/'manifest.json',manifest)
    try:
        env=dict(os.environ,ERE_LIFECYCLE_OUTPUT=str(out/'native'))
        command=['cargo','test','--offline','--locked','-p','hg-sim-core','--features','hg-meta',
                 '--test','ere_rule_lifecycle','--','--nocapture']
        native=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=300)
        save(out/'native_test_receipt.json',dict(command=command,returncode=native.returncode,
            stdout=native.stdout,stderr=native.stderr))
        assert native.returncode==0,'native fixture tests failed; no automatic retry'
        result=verify.audit_panel(out/'native');save(out/'audit.json',result)
        env=dict(os.environ,ERE_LIFECYCLE_AUDIT_INPUT=str(out/'native'))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_lifecycle.py','-v']
        tests=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=60)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=tests.returncode,
            stdout=tests.stdout,stderr=tests.stderr))
        assert tests.returncode==0,'independent reconstruction/controls failed'
        assert sources=={n:a.digest(ROOT/n) for n in names},'measurement source drift'
        assert names==source_names(),'measurement source set drift'
        manifest.update(status='complete',native_work_unknown=False,native_attempts=432,advancing_steps=432,
            native_test_groups=3,python_test_groups=4,detached_receipt_tamper_rejections=18,
            executed_binary_sha256=a.digest(out/'native/executed_test_binary'),
            elapsed_seconds=time.time()-started)
        save(out/'manifest.json',manifest)
    except Exception as error:
        manifest.update(status='failed',error=repr(error),elapsed_seconds=time.time()-started)
        save(out/'manifest.json',manifest);a.seal(out);raise
    a.seal(out)
    print(json.dumps(dict(output=str(out),status='complete',native_attempts=432,
        accounting_pass=True,training_admitted=False,seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
