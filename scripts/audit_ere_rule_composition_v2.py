#!/usr/bin/env python3
"""Corrected receipt-only accounting of a sealed native-complete/old-audit-failed panel."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_rule_composition as historical
import verify_ere_rule_composition_v2 as verify

a=verify.integrity
ROOT=historical.ROOT
INPUT=ROOT/'output/ere_rule_composition_development_20261004'
INPUT_SEAL='8dff01746ecc81db6e3c3e1c3036e0de9cdf8af4050857d1150a8cbbe11ede71'
EXTRA=['docs/ere_rule_composition_archive_audit_v2.md','scripts/audit_ere_rule_composition_v2.py',
       'scripts/verify_ere_rule_composition_v2.py','scripts/test_verify_ere_rule_composition_v2.py',
       'scripts/verify_ere_rule_lifecycle.py','scripts/audit_ere_complete_admission.py',
       'scripts/verify_ere_complete_witness.py']


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def version_delta():
    old=(INPUT/'source_snapshots/scripts/verify_ere_rule_composition.py').read_text()
    new=(ROOT/'scripts/verify_ere_rule_composition_v2.py').read_text()
    new=new.replace("\nAUDITOR_PROFILE='E2-rule-composition-receipt-auditor-v2'",'',1)
    new=new.replace('auditor_profile=AUDITOR_PROFILE,new_native_steps=0,','',1)
    begin_old=old.index('    members=[e for e in edges.values()')
    end_old=old.index('    installed=initial',begin_old)
    begin_new=new.index('    right=link(META+0x40,allele,2)')
    end_new=new.index('    installed=initial',begin_new)
    assert old[:begin_old]==new[:begin_new] and old[end_old:]==new[end_new:],'non-proof predicate drift'
    return dict(proof_only_correction=True,original_checker_sha256=a.digest(INPUT/'source_snapshots/scripts/verify_ere_rule_composition.py'),
                corrected_checker_sha256=a.digest(ROOT/'scripts/verify_ere_rule_composition_v2.py'))


def preflight():
    a.verify(INPUT,expected=INPUT_SEAL,count=617)
    m=a.load(INPUT/'manifest.json')
    assert m['status']=='failed' and m['native_launched'] is True and m['error']=='AssertionError()'
    assert a.load(INPUT/'native_test_receipt.json')['returncode']==0
    assert a.load(INPUT/'native/manifest.json')['status']=='native_complete'
    assert m['sources']=={n:a.digest(ROOT/n) for n in m['sources']},'original native source drift'
    assert m['sources']=={n:a.digest(INPUT/'source_snapshots'/n) for n in m['sources']}
    version=version_delta();history=historical.preservation();features=historical.previous.features()
    return dict(input_seal_sha256=INPUT_SEAL,input_entries=617,original_lifecycle_status='failed',
        original_native_status='native_complete',original_native_test_returncode=0,
        original_source_matches=len(m['sources']),original_binary_sha256=a.digest(INPUT/'native/executed_test_binary'),
        version=version,historical_preservation=history,feature_scope=features)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True)
    out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh supplemental output required'
    out.mkdir();start=time.time();sources={n:a.digest(ROOT/n) for n in EXTRA}
    for name in EXTRA:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
    m=dict(profile=verify.AUDITOR_PROFILE,status='running',classification='archive_only_corrected_accounting',
        input=str(INPUT),input_seal_sha256=INPUT_SEAL,sources=sources,new_native_steps=0,training_updates=0,
        original_native_rerun=False,training_admitted=False,scientific_acceptance=False,independent_physics_replay=False)
    save(out/'manifest.json',m)
    try:
        save(out/'preflight.json',preflight())
        result=verify.audit_panel(INPUT/'native');save(out/'audit.json',result)
        env=dict(os.environ,ERE_RULE_COMPOSITION_AUDIT_INPUT=str(INPUT/'native'))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_composition_v2.py','-v']
        tests=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=90)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=tests.returncode,stdout=tests.stdout,stderr=tests.stderr))
        assert tests.returncode==0,'corrected accounting/tamper regression failed'
        assert sources=={n:a.digest(ROOT/n) for n in EXTRA},'supplement source drift'
        assert preflight()==a.load(out/'preflight.json'),'native/feature/historical evidence drift'
        m.update(status='complete',accounting_pass=True,native_steps_checked=9504,cells=396,
            python_test_groups=5,detached_native_tamper_rejections=24,detached_policy_tamper_rejections=4,
            elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out)
    print(json.dumps(dict(output=str(out),status='complete',accounting_pass=True,cells=396,
        native_steps_checked=9504,new_native_steps=0,seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
