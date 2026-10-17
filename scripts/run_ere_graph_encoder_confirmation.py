#!/usr/bin/env python3
"""One fixed native confirmation of an imported checkpoint; no training/retry."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys
import time
import run_ere_graph_encoder_exact as previous
import run_ere_graph_encoder_learning as historical
import verify_ere_graph_encoder_confirmation as v

ROOT=previous.ROOT;a=v.a;INPUT=ROOT/'output'/v.SOURCE
EXTRA=['configs/ere_graph_encoder_confirmation_v1.json','docs/ere_graph_encoder_confirmation_v1.md',
    'scripts/run_ere_graph_encoder_confirmation.py','scripts/verify_ere_graph_encoder_confirmation.py',
    'scripts/test_verify_ere_graph_encoder_confirmation.py','crates/hg-train/tests/ere_graph_encoder_confirmation.rs']
def source_names():return sorted(set(previous.source_names())|set(EXTRA))
def save(path,value):previous.save(path,value)
def command(out,name,args,env=None,timeout=300):previous.command(out,name,args,env,timeout)


def preservation():
    count=a.verify(INPUT,expected=v.SEAL,count=445);old=a.load(INPUT/'manifest.json')['sources']
    changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    assert changed==['crates/hg-train/src/ere.rs'],'unregistered previous source drift'
    export='pub mod graph_encoder_confirmation;\n';current=(ROOT/changed[0]).read_text();frozen=(INPUT/'source_snapshots'/changed[0]).read_text()
    assert current.count(export)==1 and current.replace(export,'',1)==frozen,'not export-only change'
    history=a.load(INPUT/'historical_preservation.json')
    assert a.verify(ROOT/'output'/v.exact.SOURCE,expected=v.exact.SEAL,count=2920)==2920
    assert a.verify(historical.FIXTURE,expected=historical.FIXTURE_SEAL,count=267)==267
    for name,r in history['historical_archives'].items():assert historical.historical_verify(ROOT/'output'/name,r['seal_sha256'],r['entries'])==r
    r=history['positive_control_archive'];assert historical.historical_verify(ROOT/'output/ere_rule_relational_learning_trial_20261004',r['seal_sha256'],r['entries'])==r
    return dict(source_entries=count,source_root_seal=v.SEAL,prior_unchanged_source_pins=len(old)-1,additive_export=changed,
        native_learning_archive_unchanged=True,engineering_archive_unchanged=True,inherited_preservation=history)


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',required=True,type=Path);out=parser.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for n in names:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    m=dict(profile=v.PROFILE,status='running',design=a.load(ROOT/EXTRA[0]),sources=sources,classification='frozen_encoder_native_transfer_confirmation',
        maximum_native_attempts=9504,native_launched=False,native_work_unknown=False,optimizer_updates=0,
        training_admitted=False,scientific_acceptance=False,sustained_claim=False,independent_physics_replay=False,full_training_authorized=False)
    save(out/'manifest.json',m)
    try:
        assert m['design']==v.DESIGN
        before=historical.previous.feature_scope();save(out/'feature_scope_before.json',before)
        history=preservation();save(out/'historical_preservation.json',history)
        for dest,source in [('source_output_sha256.json','output_sha256.json'),('checkpoint_import.json','checkpoints.json'),('matrix_import.json','input.json')]:shutil.copyfile(INPUT/source,out/dest)
        seal=a.load(out/'source_output_sha256.json');save(out/'import_provenance.json',dict(source_archive=v.SOURCE,source_root_seal=v.SEAL,checkpoint=100,
            imports={name:dict(source=source,sha256=seal[source]) for name,source in [('checkpoint_import.json','checkpoints.json'),('matrix_import.json','input.json')]}))
        v.imports(out)
        for archive,entries in v.p.INCIDENTAL.items():
            for n,h in entries.items():
                src=ROOT/'output'/archive/n;assert a.digest(src)==h
                dest=out/'incidental_snapshots'/archive/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dest)
        cargo=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_graph_encoder_confirmation']
        command(out,'rust_preflight.json',cargo+['frozen_import_refusals','--','--exact','--nocapture'])
        tests=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_graph_encoder_confirmation.py','-v']
        command(out,'python_preflight.json',tests)
        assert names==source_names() and sources=={n:a.digest(ROOT/n) for n in names},'prelaunch source drift'
        m.update(native_launched=True,native_work_unknown=True);save(out/'manifest.json',m)
        command(out,'native_receipt.json',cargo+['fixed_native_confirmation','--','--exact','--nocapture'],dict(os.environ,ERE_GRAPH_ENCODER_CONFIRMATION_OUTPUT=str(out)),timeout=240)
        result=v.audit(out);save(out/'audit.json',result)
        command(out,'python_postflight.json',tests,dict(os.environ,ERE_GRAPH_ENCODER_CONFIRMATION_AUDIT_INPUT=str(out)))
        command(out,'feature_off_check.json',['cargo','check','--offline','--locked','-p','hg-train','--no-default-features'])
        assert names==source_names() and sources=={n:a.digest(ROOT/n) for n in names},'measurement source drift'
        after=historical.previous.feature_scope();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert after==history
        size=sum(x.stat().st_size for x in out.rglob('*') if x.is_file());assert size<128*1024*1024-262144,'output limit exceeded'
        m.update(status='complete',accounting_pass=True,native_work_unknown=False,native_attempts=9504,advancing_steps=9504,confirmation_pass=result['confirmation_pass'],
            executed_binary_sha256=a.digest(out/'native/executed_test_binary'),elapsed_seconds=time.time()-start,pre_emission_decisions_checked=72)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),native_work_unknown=m['native_launched'],native_attempts=None if m['native_launched'] else 0,elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out);print(json.dumps(dict(output=str(out),status='complete',accounting_pass=True,native_attempts=9504,confirmation_pass=result['confirmation_pass'],seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
