#!/usr/bin/env python3
"""One frozen native graph-encoder learning trial; no outcome-dependent retry."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_graph_encoder as previous
import verify_ere_graph_encoder_learning as verify

a=verify.a;ROOT=previous.ROOT
FIXTURE=ROOT/'output/ere_graph_encoder_engineering_20261005'
FIXTURE_SEAL='2f6bbcde44f7bdbb646687b4a511ff6dfbc5484e293ff35d9860fa034b97e54d'
EXTRA=['configs/ere_graph_encoder_learning_v1.json','docs/ere_graph_encoder_learning_v1.md',
    'scripts/run_ere_graph_encoder_learning.py','scripts/verify_ere_graph_encoder_learning.py',
    'scripts/test_verify_ere_graph_encoder_learning.py','crates/hg-train/tests/ere_graph_encoder_learning.rs']


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def source_names():return sorted(set(previous.source_names())|set(EXTRA))


def historical_verify(root,expected,count):
    # Preserve exact old seals/entries. Explicitly account for separately pinned
    # Finder metadata added afterward; do not make a general hidden-file waiver.
    assert a.digest(root/'output_sha256.json')==expected
    entries=a.load(root/'output_sha256.json');actual=a.files(root)-{'output_sha256.json'}
    assert set(entries)<=actual
    if count is not None:assert len(entries)==count
    for name,h in entries.items():assert a.safe(name) and a.digest(root/name)==h
    incidental={name:a.digest(root/name) for name in sorted(actual-set(entries))}
    verify.verify_incidental(root.name,incidental)
    return dict(entries=len(entries),seal_sha256=expected,sealed_entries_unchanged=True,
        original_exact_membership_pass=not bool(incidental),separately_pinned_incidental_files=incidental)


def preservation():
    entries=a.verify(FIXTURE,expected=FIXTURE_SEAL,count=267)
    old=a.load(FIXTURE/'manifest.json')['sources'];changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    assert changed==['crates/hg-train/src/ere.rs'],'unexpected engineering source drift'
    name=changed[0];current=(ROOT/name).read_text();frozen=(FIXTURE/'source_snapshots'/name).read_text();export='pub mod graph_encoder_learning;\n'
    assert current.count(export)==1 and current.replace(export,'',1)==frozen,'not export-only extension'
    positive=historical_verify(previous.INPUT,previous.verify.INPUT_SEAL,2900)
    controls=a.load(previous.INPUT/'audit.json')['comparisons'];assert [c['final_graph']['theta'] for c in controls]==verify.POSITIVE_THETAS
    history={name:historical_verify(ROOT/'output'/name,seal,count) for name,count,seal in previous.ARCHIVES}
    return dict(engineering_seal=FIXTURE_SEAL,engineering_entries=entries,prior_unchanged_source_pins=len(old)-1,additive_export=changed,
        positive_control_source_seal=previous.verify.INPUT_SEAL,positive_control_parameters_match=True,positive_control_archive=positive,historical_archives=history,
        inherited_registered_differences=a.load(FIXTURE/'historical_preservation.json')['registered_additive_differences'])


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for name in names:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
    design=a.load(ROOT/EXTRA[0]);m=dict(profile=verify.PROFILE,status='running',classification='bounded_native_graph_encoder_learning',design=design,sources=sources,
        bounded_learning_authorized=True,full_training_authorized=False,training_admitted=False,scientific_acceptance=False,independent_physics_replay=False,
        maximum_native_attempts=42768,native_launched=False,native_work_unknown=False)
    save(out/'manifest.json',m)
    try:
        assert design==verify.DESIGN,'prospective design mismatch'
        before=previous.feature_scope();save(out/'feature_scope_before.json',before);history=preservation();save(out/'historical_preservation.json',history)
        for archive,entries in verify.INCIDENTAL.items():
            for name,h in entries.items():
                path=ROOT/'output'/archive/name;assert a.digest(path)==h
                dest=out/'incidental_snapshots'/archive/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(path,dest)
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_graph_encoder_learning','endpoint_baseline_and_update_refusals','--','--exact','--nocapture']
        r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'optimizer_preflight_test.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_graph_encoder_learning.py','-v']
        r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=180)
        save(out/'auditor_preflight_test.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'prelaunch source drift'
        env=dict(os.environ,ERE_GRAPH_ENCODER_LEARNING_OUTPUT=str(out/'native'))
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_graph_encoder_learning','fixed_native_graph_encoder_learning_trial','--','--exact','--nocapture']
        m.update(native_launched=True,native_work_unknown=True);save(out/'manifest.json',m)
        r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=540)
        save(out/'native_test_receipt.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0,'native trial failed; retained without retry'
        result=verify.audit(out/'native');save(out/'audit.json',result)
        env=dict(os.environ,ERE_GRAPH_ENCODER_LEARNING_AUDIT_INPUT=str(out/'native'))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_graph_encoder_learning.py','-v']
        r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=240)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0,'independent/tamper accounting failed'
        command=['cargo','check','--offline','--locked','-p','hg-train','--no-default-features']
        r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'feature_off_check.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'measurement source drift'
        after=previous.feature_scope();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert history==after
        m.update(status='complete',accounting_pass=True,native_work_unknown=False,native_attempts=42768,advancing_steps=42768,completed_updates=72,
            pre_emission_decisions_checked=864,learning_criterion_pass=result['learning_criterion_pass'],executed_binary_sha256=a.digest(out/'native/executed_test_binary'),elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),native_work_unknown=m['native_launched'],native_attempts=None if m['native_launched'] else 0,elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out);print(json.dumps(dict(output=str(out),status='complete',native_attempts=42768,completed_updates=72,accounting_pass=True,learning_criterion_pass=result['learning_criterion_pass'],seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
