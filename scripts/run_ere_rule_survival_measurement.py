#!/usr/bin/env python3
"""Freeze a zero-native lifetime instrument against two preserved native archives."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
import ere_rule_survival_metrics as metrics
import run_ere_graph_encoder_confirmation as current
import run_ere_graph_encoder_learning as history
import verify_ere_rule_lifecycle as lifecycle
import verify_ere_graph_encoder_confirmation as confirmation

ROOT=current.ROOT;a=current.a
CONF=ROOT/'output/ere_graph_encoder_native_confirmation_20261007'
CONF_SEAL='2da2e2889dd687ba613beea610ab9fb93670dced83285e38519d2e4142c2408d'
LIFE=ROOT/'output/ere_rule_lifecycle_fixture_development_20261003'
LIFE_SEAL='0cdba8b56ae0d490c87f0ea6d6172fbce1ed27c81c18f24b37d9840e398b8bd5'
EXTRA=['scripts/ere_rule_survival_metrics.py','scripts/test_ere_rule_survival_metrics.py','scripts/run_ere_rule_survival_measurement.py','docs/ere_rule_survival_measurement_v1.md']
def save(path,value):current.save(path,value)


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',required=True,type=Path);out=parser.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh measurement archive required'
    out.mkdir();start=time.time();names=sorted(set(current.source_names())|set(EXTRA));pins={n:a.digest(ROOT/n) for n in names}
    for n in names:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    manifest=dict(profile=metrics.PROFILE,status='running',classification='retrospective_archive_only_measurement_engineering',sources=pins,
        native_attempts=0,optimizer_updates=0,scientific_acceptance=False,sustained_claim=False,training_admitted=False)
    save(out/'manifest.json',manifest)
    try:
        assert a.verify(CONF,expected=CONF_SEAL,count=759)==759
        preservation=history.historical_verify(LIFE,LIFE_SEAL,187);save(out/'historical_preservation.json',preservation)
        for n,h in a.load(CONF/'manifest.json')['sources'].items():assert a.digest(ROOT/n)==h
        seal_conf=a.load(CONF/'output_sha256.json');seal_life=a.load(LIFE/'output_sha256.json')
        shutil.copyfile(CONF/'output_sha256.json',out/'confirmation_source_seal.json');shutil.copyfile(LIFE/'output_sha256.json',out/'lifecycle_source_seal.json')
        (out/'inputs').mkdir();provenance=[];cells=[];groups={}
        for si,seed in enumerate(confirmation.p.native.SEEDS):
            group=[]
            for wi,context in enumerate(confirmation.p.native.CONTEXTS):
                i=324+si*36+wi*4;name=f'native/cell-{i:04}.json';src=CONF/name;assert a.digest(src)==seal_conf[name]
                dest=out/'inputs'/f'confirmation-cell-{i:04}.json';shutil.copyfile(src,dest);wrapper=a.load(dest)
                assert wrapper['decision']['policy']=='learned_graph' and wrapper['context']==context and wrapper['native_seed']==seed
                receipt=wrapper['native'];confirmation.p.audit_native(receipt)
                target=confirmation.p.native.candidate(False,context)['rcf'];touches=[r['step'] for r in receipt['timeline'] for _ in range(r['paid_edits'])]
                measured=metrics.cell(receipt,target,touches);measured.update(origin='confirmation',source_cell=i,native_seed=seed,context=context)
                cells.append(measured);group.append(measured);provenance.append(dict(archive=CONF.name,source=name,sha256=a.digest(src),saved=dest.relative_to(out).as_posix()))
            groups[f'learned_graph_seed_{seed}']=metrics.cohort(group)
        for arm in ['live_paid','live_replacement']:
            group=[]
            for seed in lifecycle.SEEDS:
                name=f'native/{seed}-{arm}.json';src=LIFE/name;assert a.digest(src)==seal_life[name]
                dest=out/'inputs'/f'lifecycle-{seed}-{arm}.json';shutil.copyfile(src,dest);receipt=a.load(dest);lifecycle.audit_receipt(receipt)
                target=lifecycle.candidate(False,10)['rcf'];touches=[r['step'] for r in receipt['timeline'] for _ in range(r['paid_edits'])]
                measured=metrics.cell(receipt,target,touches);measured.update(origin='lifecycle',arm=arm,native_seed=seed)
                cells.append(measured);group.append(measured);provenance.append(dict(archive=LIFE.name,source=name,sha256=a.digest(src),saved=dest.relative_to(out).as_posix()))
            groups[arm]=metrics.cohort(group,8. if arm=='live_replacement' else None)
        assert len(cells)==24 and all(not g['literal_lifetime_ratio_pass'] for g in groups.values())
        save(out/'input_provenance.json',provenance);save(out/'cells.json',cells);save(out/'cohorts.json',groups)
        args=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_ere_rule_survival_metrics.py','-v']
        r=subprocess.run(args,cwd=ROOT,capture_output=True,text=True,timeout=120)
        save(out/'test_receipt.json',dict(command=args,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        assert pins=={n:a.digest(ROOT/n) for n in names} and names==sorted(set(current.source_names())|set(EXTRA))
        assert a.verify(CONF,expected=CONF_SEAL,count=759)==759 and history.historical_verify(LIFE,LIFE_SEAL,187)==preservation
        manifest.update(status='complete',accounting_pass=True,receipts_checked=24,cohorts=4,elapsed_seconds=time.time()-start)
        save(out/'manifest.json',manifest)
    except Exception as error:
        manifest.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start);save(out/'manifest.json',manifest);a.seal(out);raise
    a.seal(out);print(json.dumps(dict(output=str(out),status='complete',receipts_checked=24,cohorts=groups,source_pins=len(pins),seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
