#!/usr/bin/env python3
"""One frozen 63-cell post-birth maintenance panel; retain every failure, never retry."""
import argparse
import ast
import json
import os
from pathlib import Path
import shutil
import sys
import time
import run_ere_graph_encoder_confirmation as prior
import run_ere_rule_survival_measurement as measurement
import verify_ere_rule_maintenance as v

ROOT=prior.ROOT;a=v.a
CONF=ROOT/'output'/v.SOURCE
SURV=ROOT/'output/ere_rule_survival_measurement_20261007'
SURV_SEAL='e9577a1c2143aa7b20273cf6594494f1a20ca838c1051a17491d58c8d03f2657'
EXTRA=['configs/ere_rule_maintenance_v1.json','docs/ere_rule_maintenance_v1.md','scripts/run_ere_rule_maintenance.py',
    'scripts/verify_ere_rule_maintenance.py','scripts/test_verify_ere_rule_maintenance.py','scripts/recheck_ere_rule_maintenance.py',
    'scripts/verify_ere_complete_long_gates.py','scripts/verify_ere_complete_gates.py','scripts/recheck_ere_rule_maintenance_closure_v2.py',
    'scripts/test_ere_rule_maintenance_packaging.py',
    'crates/hg-sim-core/tests/ere_rule_maintenance.rs','crates/hg-train/tests/ere_rule_maintenance.rs']
def dependency_names(entry='scripts/verify_ere_rule_maintenance.py'):
    """Pin the complete local static import closure, not just the top-level auditor."""
    seen=set();pending=[entry]
    while pending:
        n=pending.pop()
        if n in seen:continue
        seen.add(n)
        tree=ast.parse((ROOT/n).read_text(),filename=n)
        for node in ast.walk(tree):
            modules=[alias.name for alias in node.names] if isinstance(node,ast.Import) else [node.module] if isinstance(node,ast.ImportFrom) and node.level==0 and node.module else []
            for module in modules:
                candidate='scripts/'+module.replace('.','/')+'.py'
                if (ROOT/candidate).is_file() and candidate not in seen:pending.append(candidate)
    return seen

def names():return sorted(set(prior.source_names())|set(measurement.EXTRA)|set(EXTRA)|dependency_names())
def save(path,value):prior.save(path,value)
def command(out,name,args,env=None,timeout=300):prior.command(out,name,args,env,timeout)


def preservation():
    assert a.verify(CONF,expected=v.SOURCE_SEAL,count=759)==759
    assert a.verify(SURV,expected=SURV_SEAL,count=302)==302
    pins=a.load(SURV/'manifest.json')['sources']
    changed=[n for n,h in pins.items() if a.digest(ROOT/n)!=h]
    assert changed==['crates/hg-sim-core/src/ere.rs','crates/hg-train/src/ere.rs'],'unregistered body drift'
    export='pub mod rule_maintenance;\n'
    for n in changed:
        live=(ROOT/n).read_text();frozen=(SURV/'source_snapshots'/n).read_text()
        assert live.count(export)==1 and live.replace(export,'',1)==frozen,'not additive export only'
    for n,h in a.load(CONF/'manifest.json')['sources'].items():
        if n not in changed:assert a.digest(ROOT/n)==h
    archives=[('ere_graph_encoder_exact_gradient_20261007',445,v.EXACT_SEAL),
        ('ere_graph_encoder_learning_trial_20261007',2920,'232178828b8de4fc2c33e30f6a3ccc7b25d489a5deeb7aad788367a1d2e9cd5b'),
        ('ere_graph_encoder_engineering_20261005',267,'2f6bbcde44f7bdbb646687b4a511ff6dfbc5484e293ff35d9860fa034b97e54d')]
    for n,c,h in archives:assert a.verify(ROOT/'output'/n,expected=h,count=c)==c
    history=a.load(CONF/'historical_preservation.json')['inherited_preservation']
    for n,r in history['historical_archives'].items():assert prior.historical.historical_verify(ROOT/'output'/n,r['seal_sha256'],r['entries'])==r
    r=history['positive_control_archive'];assert prior.historical.historical_verify(ROOT/'output/ere_rule_relational_learning_trial_20261004',r['seal_sha256'],r['entries'])==r
    return dict(confirmation_root=v.SOURCE_SEAL,confirmation_entries=759,survival_root=SURV_SEAL,survival_entries=302,
        unchanged_prior_pins=len(pins)-2,additive_exports=changed,inherited_preservation=history)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();source_names=names();pins={n:a.digest(ROOT/n) for n in source_names}
    for n in source_names:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    m=dict(profile=v.PROFILE,status='running',design=a.load(ROOT/EXTRA[0]),sources=pins,native_launched=False,
        native_work_unknown=False,panel_native_advances=0,panel_initialization_applications=0,
        preregistered_preflight_advances=1104,preregistered_preflight_seal_applications=7,
        optimizer_updates=0,training_admitted=False,scientific_acceptance=False,sustained_claim=False,independent_physics_replay=False)
    save(out/'manifest.json',m)
    try:
        assert m['design']==v.DESIGN
        before=prior.historical.previous.feature_scope();save(out/'feature_scope_before.json',before)
        history=preservation();save(out/'historical_preservation.json',history)
        for src,dest in [('output_sha256.json','confirmation_source_seal.json'),('checkpoint_import.json','checkpoint_import.json'),('source_output_sha256.json','exact_source_seal.json')]:
            shutil.copyfile(CONF/src,out/dest)
        v.imports(out)
        for archive,entries in prior.v.p.INCIDENTAL.items():
            for n,h in entries.items():
                src=ROOT/'output'/archive/n;assert a.digest(src)==h
                dest=out/'incidental_snapshots'/archive/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dest)
        fixture=out/'native_preflight';fixture.mkdir()
        command(out,'native_preflight_receipt.json',['cargo','test','--offline','--locked','-p','hg-sim-core','--features','hg-meta','--test','ere_rule_maintenance','--','--nocapture'],
            dict(os.environ,ERE_RULE_MAINTENANCE_PREFLIGHT=str(fixture)),timeout=120)
        assert a.files(fixture)=={f'{arm}.json' for arm in v.ARMS}
        preflight=[v.audit_receipt(a.load(fixture/f'{arm}.json')) for arm in v.ARMS]
        save(out/'preflight_audit.json',dict(controls=preflight,parity_worlds=9,parity_advances=432,control_advances=672,seal_applications=7))
        cargo=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_rule_maintenance']
        command(out,'rust_import_preflight.json',cargo+['schedule_and_import_refusals','--','--exact','--nocapture'])
        tests=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_maintenance.py','-v']
        env=dict(os.environ,ERE_RULE_MAINTENANCE_PREFLIGHT=str(fixture))
        command(out,'python_preflight.json',tests,env)
        command(out,'packaging_preflight.json',[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_ere_rule_maintenance_packaging.py','-v'])
        command(out,'binary_build.json',cargo+['--no-run','--message-format=json'])
        artifacts=[json.loads(s) for s in a.load(out/'binary_build.json')['stdout'].splitlines() if s.startswith('{')]
        bins=[r['executable'] for r in artifacts if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('name')=='ere_rule_maintenance' and r.get('executable')]
        assert len(bins)==1
        shutil.copyfile(bins[0],out/'prelaunch_test_binary');m['prelaunch_binary_sha256']=a.digest(out/'prelaunch_test_binary')
        assert names()==source_names and pins=={n:a.digest(ROOT/n) for n in source_names},'prelaunch drift'
        m.update(native_launched=True,native_work_unknown=True,preflight_advances=1104,preflight_initialization_applications=7);save(out/'manifest.json',m)
        command(out,'native_panel_receipt.json',cargo+['fixed_maintenance_panel','--','--exact','--nocapture'],dict(os.environ,ERE_RULE_MAINTENANCE_OUTPUT=str(out)),timeout=240)
        result=v.audit(out);save(out/'audit.json',result)
        command(out,'python_postflight.json',tests,dict(env,ERE_RULE_MAINTENANCE_AUDIT_INPUT=str(out)))
        command(out,'survival_tests.json',[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_ere_rule_survival_metrics.py','-v'])
        command(out,'feature_off_check.json',['cargo','check','--offline','--locked','-p','hg-train','--no-default-features'])
        assert names()==source_names and pins=={n:a.digest(ROOT/n) for n in source_names},'postflight drift'
        after=prior.historical.previous.feature_scope();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert history==after
        size=sum(x.stat().st_size for x in out.rglob('*') if x.is_file());assert size<v.DESIGN['output_limit_bytes']-524288
        m.update(status='complete',accounting_pass=True,native_work_unknown=False,panel_native_advances=6048,panel_initialization_applications=63,
            executed_binary_sha256=result['executed_binary_sha256'],learned_paid_restoration_pass=result['learned_paid_restoration_pass'],
            oracle_capacity_pass=result['oracle_capacity_pass'],elapsed_seconds=time.time()-start,preseal_output_bytes=size)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),native_work_unknown=m['native_launched'],panel_native_advances=None if m['native_launched'] else 0,
            elapsed_seconds=time.time()-start);save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out)
    print(json.dumps(dict(output=str(out),status='complete',accounting_pass=True,seal_sha256=a.digest(out/'output_sha256.json'),entries=len(a.load(out/'output_sha256.json')),
        learned_target_births=result['learned_target_births'],learned_nontrivial_worlds=result['learned_nontrivial_worlds'],
        learned_paid_restoration_pass=result['learned_paid_restoration_pass'],oracle_capacity_pass=result['oracle_capacity_pass'],totals=result['totals']),indent=2))


if __name__=='__main__':main()
