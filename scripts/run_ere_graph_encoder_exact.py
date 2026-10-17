#!/usr/bin/env python3
"""One prospectively fixed zero-native optimizer diagnostic, sealed even if failed."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_graph_encoder_learning as previous
import verify_ere_graph_encoder_exact as v

ROOT=previous.ROOT;a=v.a;INPUT=ROOT/'output'/v.SOURCE
EXTRA=['configs/ere_graph_encoder_exact_v1.json','docs/ere_graph_encoder_exact_v1.md','scripts/run_ere_graph_encoder_exact.py',
    'scripts/verify_ere_graph_encoder_exact.py','scripts/test_verify_ere_graph_encoder_exact.py','crates/hg-train/tests/ere_graph_encoder_exact.rs']
def source_names():return sorted(set(previous.source_names())|set(EXTRA))
def save(path,value):previous.save(path,value)


def preservation():
    count=a.verify(INPUT,expected=v.SEAL,count=2920);old=a.load(INPUT/'manifest.json')['sources']
    changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    assert changed==['crates/hg-train/src/ere.rs'],'unexpected prior active source drift'
    current=(ROOT/changed[0]).read_text();frozen=(INPUT/'source_snapshots'/changed[0]).read_text();export='pub mod graph_encoder_exact;\n'
    assert current.count(export)==1 and current.replace(export,'',1)==frozen,'not export-only extension'
    history=a.load(INPUT/'historical_preservation.json')
    fixture_entries=a.verify(previous.FIXTURE,expected=previous.FIXTURE_SEAL,count=267)
    positive=previous.historical_verify(previous.previous.INPUT,previous.previous.verify.INPUT_SEAL,2900)
    assert positive==history['positive_control_archive']
    assert [c['final_graph']['theta'] for c in a.load(previous.previous.INPUT/'audit.json')['comparisons']]==v.prior.POSITIVE_THETAS
    historical={n:previous.historical_verify(ROOT/'output'/n,seal,c) for n,c,seal in previous.previous.ARCHIVES}
    assert historical==history['historical_archives']
    return dict(prior_root_seal=v.SEAL,prior_entries=count,prior_unchanged_source_pins=len(old)-1,additive_export=changed,
        engineering_entries=fixture_entries,engineering_root_seal=previous.FIXTURE_SEAL,historical_archives=historical,
        positive_control_archive=positive,positive_control_parameters_match=True,
        inherited_registered_differences=history['inherited_registered_differences'])


def command(out,name,args,env=None,timeout=300):
    r=subprocess.run(args,cwd=ROOT,env=env,capture_output=True,text=True,timeout=timeout)
    save(out/name,dict(command=args,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0,name


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for n in names:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    m=dict(profile=v.PROFILE,status='running',classification='offline_archive_reuse_optimizer_diagnostic',design=a.load(ROOT/EXTRA[0]),sources=sources,
        maximum_native_attempts=0,native_attempts=0,optimizer_launched=False,training_admitted=False,scientific_acceptance=False,
        independent_physics_replay=False,sustained_claim=False,full_training_authorized=False)
    save(out/'manifest.json',m)
    try:
        assert m['design']==v.DESIGN
        before=previous.previous.feature_scope();save(out/'feature_scope_before.json',before)
        history=preservation();save(out/'historical_preservation.json',history)
        shutil.copyfile(INPUT/'output_sha256.json',out/'source_output_sha256.json');(out/'inputs').mkdir()
        for i in list(range(144))+list(range(576,594)):shutil.copyfile(INPUT/'native'/f'cell-{i:04}.json',out/'inputs'/f'cell-{i:04}.json')
        for archive,entries in v.prior.INCIDENTAL.items():
            for n,h in entries.items():
                src=ROOT/'output'/archive/n;assert a.digest(src)==h
                dest=out/'incidental_snapshots'/archive/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dest)
        data,provenance=v.import_matrix(out);save(out/'input.json',data);save(out/'input_provenance.json',provenance)
        cargo=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_graph_encoder_exact']
        command(out,'rust_preflight.json',cargo+['exact_objective_finite_difference_and_refusals','--','--exact','--nocapture'])
        tests=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_graph_encoder_exact.py','-v']
        command(out,'python_preflight.json',tests)
        assert names==source_names() and sources=={n:a.digest(ROOT/n) for n in names},'prelaunch drift'
        m['optimizer_launched']=True;save(out/'manifest.json',m)
        command(out,'optimizer_receipt.json',cargo+['fixed_archive_exact_gradient_diagnostic','--','--exact','--nocapture'],dict(os.environ,ERE_GRAPH_ENCODER_EXACT_OUTPUT=str(out)))
        result=v.audit(out);save(out/'audit.json',result)
        command(out,'python_postflight.json',tests,dict(os.environ,ERE_GRAPH_ENCODER_EXACT_AUDIT_INPUT=str(out)))
        command(out,'feature_off_check.json',['cargo','check','--offline','--locked','-p','hg-train','--no-default-features'])
        assert names==source_names() and sources=={n:a.digest(ROOT/n) for n in names},'measurement drift'
        after=previous.previous.feature_scope();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert history==after
        m.update(status='complete',accounting_pass=True,completed_updates=200,checkpoint_results=result['checkpoint_results'],
            executed_binary_sha256=a.digest(out/'executed_test_binary'),elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start);save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out);print(json.dumps(dict(output=str(out),status='complete',accounting_pass=True,native_attempts=0,checkpoint_results=result['checkpoint_results'],seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
