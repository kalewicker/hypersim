#!/usr/bin/env python3
"""Freeze and execute one five-cell native paid prevention capacity fixture."""
import argparse
import ast
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import run_ere_rule_maintenance as prior
import verify_ere_rule_prevention as v

ROOT=prior.ROOT;a=v.a
INPUT=ROOT/'output/ere_rule_maintenance_trial_20261007'
SUPPLEMENT=ROOT/'output/ere_rule_maintenance_closure_v2_20261007'
SUPPLEMENT_SEAL='9309163be350189e241aa4d9593f985f29b2351571630d0c61487b13f07020b3'
PACKAGING_BASELINE='0f24959c2126bb930a6308107dbde53c7f722dcffcca6d4d397f0349db0dbd2e'
EXTRA=['configs/ere_rule_prevention_primitive_v1.json','docs/ere_rule_prevention_primitive_v1.md',
    'crates/hg-sim-core/tests/ere_rule_prevention.rs','scripts/run_ere_rule_prevention.py','scripts/verify_ere_rule_prevention.py',
    'scripts/test_verify_ere_rule_prevention.py','scripts/recheck_ere_rule_prevention.py']
def dependencies(entry='scripts/verify_ere_rule_prevention.py'):
    seen=set();pending=[entry]
    while pending:
        n=pending.pop()
        if n in seen:continue
        seen.add(n)
        for node in ast.walk(ast.parse((ROOT/n).read_text(),filename=n)):
            modules=[a.name for a in node.names] if isinstance(node,ast.Import) else [node.module] if isinstance(node,ast.ImportFrom) and node.level==0 and node.module else []
            for module in modules:
                name='scripts/'+module.replace('.','/')+'.py'
                if (ROOT/name).is_file() and name not in seen:pending.append(name)
    return seen
def names():return sorted(set(prior.names())|set(EXTRA)|dependencies())
def save(path,value):prior.save(path,value)
def command(out,name,args,env=None,timeout=180):prior.command(out,name,args,env,timeout)

def preservation():
    assert a.verify(INPUT,expected=v.PRIMARY,count=380)==380
    assert a.verify(SUPPLEMENT,expected=SUPPLEMENT_SEAL,count=9)==9
    old=a.load(INPUT/'manifest.json')['sources'];changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    expected=['crates/hg-sim-core/src/ere.rs','scripts/run_ere_rule_maintenance.py'];assert changed==expected,'unregistered prior source drift'
    name=expected[0];export='pub mod rule_prevention;\n';live=(ROOT/name).read_text();frozen=(INPUT/'source_snapshots'/name).read_text()
    assert live.count(export)==1 and live.replace(export,'',1)==frozen,'not export-only extension'
    assert a.digest(ROOT/expected[1])==PACKAGING_BASELINE,'previous packaging correction changed'
    packaging=dict(frozen_sha256=old[expected[1]],postflight_live_sha256=PACKAGING_BASELINE,
        scope='previous turn static import closure packaging correction; native and criteria bodies unchanged')
    for n,h in a.load(SUPPLEMENT/'manifest.json')['supplemented_dependencies'].items():assert a.digest(ROOT/n)==h
    history=a.load(INPUT/'historical_preservation.json')
    assert a.verify(prior.CONF,expected=history['confirmation_root'],count=759)==759
    assert a.verify(prior.SURV,expected=history['survival_root'],count=302)==302
    chain=history['inherited_preservation']
    for n,r in chain['historical_archives'].items():assert prior.prior.historical.historical_verify(ROOT/'output'/n,r['seal_sha256'],r['entries'])==r
    r=chain['positive_control_archive'];assert prior.prior.historical.historical_verify(ROOT/'output/ere_rule_relational_learning_trial_20261004',r['seal_sha256'],r['entries'])==r
    for n,c,h in [('ere_graph_encoder_exact_gradient_20261007',445,v.previous.EXACT_SEAL),
        ('ere_graph_encoder_learning_trial_20261007',2920,'232178828b8de4fc2c33e30f6a3ccc7b25d489a5deeb7aad788367a1d2e9cd5b'),
        ('ere_graph_encoder_engineering_20261005',267,'2f6bbcde44f7bdbb646687b4a511ff6dfbc5484e293ff35d9860fa034b97e54d')]:assert a.verify(ROOT/'output'/n,expected=h,count=c)==c
    return dict(primary_root=v.PRIMARY,primary_entries=380,supplement_root=SUPPLEMENT_SEAL,supplement_entries=9,
        unchanged_prior_source_pins=len(old)-2,additive_export=name,registered_prior_packaging_delta=packaging,historical_preservation=history)

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();source_names=names();pins={n:a.digest(ROOT/n) for n in source_names}
    for n in source_names:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    m=dict(profile=v.PROFILE,status='running',design=a.load(ROOT/EXTRA[0]),sources=pins,native_launched=False,native_work_unknown=False,
        native_advances=0,nonadvancing_seal_applications=0,optimizer_updates=0,classification='scripted_engineering_fixture',
        scientific_acceptance=False,sustained_claim=False,training_admitted=False,independent_physics_replay=False,frozen_controller_panel_launched=False)
    save(out/'manifest.json',m)
    try:
        assert m['design']==v.DESIGN
        before=prior.prior.historical.previous.feature_scope();save(out/'feature_scope_before.json',before)
        history=preservation();save(out/'historical_preservation.json',history)
        shutil.copyfile(INPUT/'output_sha256.json',out/'maintenance_source_seal.json');shutil.copyfile(INPUT/'native/cell-0001.json',out/'repair_reference.json')
        seal=a.load(out/'maintenance_source_seal.json');assert a.digest(out/'repair_reference.json')==seal['native/cell-0001.json']
        cargo=['cargo','test','--offline','--locked','-p','hg-sim-core','--features','hg-meta','--test','ere_rule_prevention']
        command(out,'rust_preflight.json',cargo+['catalogue_and_deadline_refusals','--','--exact','--nocapture'],dict(os.environ,ERE_RULE_PREVENTION_PREFLIGHT_OUTPUT=str(out)))
        catalogues=a.load(out/'catalogues.json');assert len(catalogues)==5
        for arm,row in zip(v.ARMS,catalogues):
            raw,compiled=v.editor_catalogue(arm,[1,1]);assert row==dict(arm=arm,raw=raw,compiled=compiled),'native/independent catalogue mismatch'
        tests=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_prevention.py','-v']
        command(out,'python_preflight.json',tests)
        command(out,'binary_build.json',cargo+['--no-run','--message-format=json'])
        artifacts=[json.loads(s) for s in a.load(out/'binary_build.json')['stdout'].splitlines() if s.startswith('{')]
        bins=[r['executable'] for r in artifacts if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('name')=='ere_rule_prevention' and r.get('executable')]
        assert len(bins)==1;shutil.copyfile(bins[0],out/'prelaunch_test_binary');m['prelaunch_binary_sha256']=a.digest(out/'prelaunch_test_binary')
        # Import the actual frozen package before native launch. No live source fallback.
        probe='import sys,json;sys.path.insert(0,'+repr(str(out/'source_snapshots/scripts'))+');import verify_ere_rule_prevention;print(json.dumps({n:mod.__file__ for n,mod in sys.modules.items() if n.startswith(("verify_","audit_","ere_")) and hasattr(mod,"__file__")}))'
        command(out,'frozen_import_preflight.json',[sys.executable,'-B','-c',probe],dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1'))
        modules=json.loads(a.load(out/'frozen_import_preflight.json')['stdout'])
        for module,path in modules.items():
            path=Path(path).resolve();assert path.is_relative_to(out/'source_snapshots')
            n=path.relative_to(out/'source_snapshots').as_posix();assert pins[n]==a.digest(path)
        save(out/'frozen_module_provenance.json',modules)
        assert names()==source_names and pins=={n:a.digest(ROOT/n) for n in source_names},'prelaunch source drift'
        m.update(native_launched=True,native_work_unknown=True);save(out/'manifest.json',m)
        command(out,'native_receipt.json',cargo+['fixed_prevention_fixture','--','--exact','--nocapture'],dict(os.environ,ERE_RULE_PREVENTION_OUTPUT=str(out)),timeout=90)
        result=v.audit(out);save(out/'audit.json',result)
        command(out,'python_postflight.json',tests,dict(os.environ,ERE_RULE_PREVENTION_AUDIT_INPUT=str(out)))
        command(out,'survival_tests.json',[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_ere_rule_survival_metrics.py','-v'])
        command(out,'feature_off_check.json',['cargo','check','--offline','--locked','-p','hg-sim-core','--no-default-features'])
        assert names()==source_names and pins=={n:a.digest(ROOT/n) for n in source_names},'postflight source drift'
        after=prior.prior.historical.previous.feature_scope();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert after==history
        size=sum(x.stat().st_size for x in out.rglob('*') if x.is_file());assert size<v.DESIGN['output_limit_bytes']-262144
        m.update(status='complete',accounting_pass=True,native_work_unknown=False,native_advances=480,nonadvancing_seal_applications=5,
            mechanism_pass=result['mechanism_pass'],elapsed_seconds=time.time()-start,preseal_output_bytes=size)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),native_work_unknown=m['native_launched'],native_advances=None if m['native_launched'] else 0,
            elapsed_seconds=time.time()-start);save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out);print(json.dumps(dict(output=str(out),status='complete',mechanism_pass=result['mechanism_pass'],totals=result['totals'],
        source_pins=len(pins),entries=len(a.load(out/'output_sha256.json')),seal_sha256=a.digest(out/'output_sha256.json')),indent=2))

if __name__=='__main__':main()
