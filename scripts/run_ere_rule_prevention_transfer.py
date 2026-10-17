#!/usr/bin/env python3
"""Freeze one 72-cell controller-transfer panel; no native retry or fitting."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys
import time
import run_ere_rule_prevention as primitive
import verify_ere_rule_prevention_transfer as v
ROOT=primitive.ROOT;a=v.a
INPUT=ROOT/'output/ere_rule_prevention_primitive_20261007'
INPUT_SEAL='7b70d40e9bfde731867632ea3ddadf788667797f270f86de665e1d95aa210b65'
RECHECK=ROOT/'output/ere_rule_prevention_recheck_20261007'
RECHECK_SEAL='b1006ebf58dd2d2db5ef3e5f2717ce299145465a94f3be66c26c53721bf974cb'
EXTRA=['configs/ere_rule_prevention_transfer_v1.json','docs/ere_rule_prevention_transfer_v1.md',
    'crates/hg-sim-core/src/ere/rule_prevention_transfer.rs','crates/hg-sim-core/tests/ere_rule_prevention_transfer.rs',
    'crates/hg-train/src/ere/rule_prevention_transfer.rs','crates/hg-train/tests/ere_rule_prevention_transfer.rs',
    'scripts/run_ere_rule_prevention_transfer.py','scripts/verify_ere_rule_prevention_transfer.py',
    'scripts/test_verify_ere_rule_prevention_transfer.py','scripts/recheck_ere_rule_prevention_transfer.py']
def names():
    closure=set()
    for entry in EXTRA[-4:-1]:closure |= primitive.dependencies(entry)
    return sorted(set(primitive.names())|set(EXTRA)|closure)
save=primitive.save;command=primitive.command

def preservation():
    assert a.verify(INPUT,expected=INPUT_SEAL,count=320)==320
    assert a.verify(RECHECK,expected=RECHECK_SEAL,count=3)==3
    old=a.load(INPUT/'manifest.json')['sources'];changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    expected=['crates/hg-sim-core/src/ere.rs','crates/hg-train/src/ere.rs'];assert changed==expected,'unregistered primitive body drift'
    export='pub mod rule_prevention_transfer;\n'
    for n in changed:
        live=(ROOT/n).read_text();frozen=(INPUT/'source_snapshots'/n).read_text()
        assert live.count(export)==1 and live.replace(export,'',1)==frozen,'not additive export only'
    history=a.load(INPUT/'historical_preservation_after.json')
    assert a.verify(primitive.INPUT,expected=history['primary_root'],count=380)==380
    assert a.verify(primitive.SUPPLEMENT,expected=history['supplement_root'],count=9)==9
    h=history['historical_preservation']
    assert a.verify(primitive.prior.CONF,expected=h['confirmation_root'],count=759)==759
    assert a.verify(primitive.prior.SURV,expected=h['survival_root'],count=302)==302
    chain=h['inherited_preservation']
    for n,r in chain['historical_archives'].items():assert primitive.prior.prior.historical.historical_verify(ROOT/'output'/n,r['seal_sha256'],r['entries'])==r
    r=chain['positive_control_archive'];assert primitive.prior.prior.historical.historical_verify(ROOT/'output/ere_rule_relational_learning_trial_20261004',r['seal_sha256'],r['entries'])==r
    for n,c,s in [('ere_graph_encoder_exact_gradient_20261007',445,v.previous.EXACT_SEAL),
        ('ere_graph_encoder_learning_trial_20261007',2920,'232178828b8de4fc2c33e30f6a3ccc7b25d489a5deeb7aad788367a1d2e9cd5b'),
        ('ere_graph_encoder_engineering_20261005',267,'2f6bbcde44f7bdbb646687b4a511ff6dfbc5484e293ff35d9860fa034b97e54d'),
        ('ere_rule_maintenance_recheck_20261007',2,'fc6be5e243c95e098cdf5c7268fc9b16b1078a09536e38e9c8690b4260d13bc5')]:assert a.verify(ROOT/'output'/n,expected=s,count=c)==c
    return dict(primitive_root=INPUT_SEAL,primitive_entries=320,recheck_root=RECHECK_SEAL,
        unchanged_prior_source_pins=len(old)-2,additive_exports=changed,inherited_preservation=history)

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();source_names=names();pins={n:a.digest(ROOT/n) for n in source_names}
    for n in source_names:
        dest=out/'source_snapshots'/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/n,dest)
    m=dict(profile=v.PROFILE,status='running',design=a.load(ROOT/EXTRA[0]),sources=pins,native_launched=False,native_work_unknown=False,
        native_advances=0,nonadvancing_seal_applications=0,optimizer_updates=0,classification='frozen_controller_transfer_engineering',
        scientific_acceptance=False,sustained_claim=False,training_admitted=False,independent_physics_replay=False)
    save(out/'manifest.json',m)
    try:
        assert m['design']==v.DESIGN
        before=primitive.prior.prior.historical.previous.feature_scope();save(out/'feature_scope_before.json',before)
        history=preservation();save(out/'historical_preservation.json',history)
        for src,dest in [('output_sha256.json','confirmation_source_seal.json'),('checkpoint_import.json','checkpoint_import.json'),('source_output_sha256.json','exact_source_seal.json')]:shutil.copyfile(primitive.prior.CONF/src,out/dest)
        v.imports(out)
        command(out,'rust_native_preflight.json',['cargo','test','--offline','--locked','-p','hg-sim-core','--features','hg-meta','--test','ere_rule_prevention_transfer','catalogue_and_deadline_refusals','--','--exact','--nocapture'],dict(os.environ,ERE_RULE_PREVENTION_TRANSFER_PREFLIGHT_OUTPUT=str(out)))
        catalogues=a.load(out/'catalogues.json');assert len(catalogues)==80
        for i,row in enumerate(catalogues):
            arm=v.ARMS[i//16];actions=[i%16//4,i%4];raw,compiled=v.editor_catalogue(arm,actions)
            assert row==dict(arm=arm,actions=actions,raw=raw,compiled=compiled),'native catalogue mismatch'
        cargo=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_rule_prevention_transfer']
        command(out,'rust_import_preflight.json',cargo+['schedule_and_import_refusals','--','--exact','--nocapture'],dict(os.environ,ERE_RULE_PREVENTION_TRANSFER_DECISION_PREFLIGHT=str(out)))
        prospective_slots=v.decisions(out,v.imports(out),reference_directory=primitive.INPUT/'native')
        save(out/'decision_preflight_audit.json',dict(verified_worlds=9,verified_policy_decisions=27,paired_draws=18,
            reference='sealed maintenance ordinary observations; zero new native work',slots=prospective_slots))
        tests=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_rule_prevention_transfer.py','-v']
        command(out,'python_preflight.json',tests)
        command(out,'binary_build.json',cargo+['--no-run','--message-format=json'])
        artifacts=[json.loads(s) for s in a.load(out/'binary_build.json')['stdout'].splitlines() if s.startswith('{')]
        bins=[r['executable'] for r in artifacts if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('name')=='ere_rule_prevention_transfer' and r.get('executable')]
        assert len(bins)==1;shutil.copyfile(bins[0],out/'prelaunch_test_binary');m['prelaunch_binary_sha256']=a.digest(out/'prelaunch_test_binary')
        probe='import sys,json;sys.path.insert(0,'+repr(str(out/'source_snapshots/scripts'))+');import verify_ere_rule_prevention_transfer;print(json.dumps({n:mod.__file__ for n,mod in sys.modules.items() if n.startswith(("verify_","audit_","ere_")) and hasattr(mod,"__file__")}))'
        command(out,'frozen_import_preflight.json',[sys.executable,'-B','-c',probe],dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1'))
        modules=json.loads(a.load(out/'frozen_import_preflight.json')['stdout'])
        for _,path in modules.items():
            path=Path(path).resolve();assert path.is_relative_to(out/'source_snapshots')
            n=path.relative_to(out/'source_snapshots').as_posix();assert pins[n]==a.digest(path)
        save(out/'frozen_module_provenance.json',modules)
        assert pins=={n:a.digest(ROOT/n) for n in names()},'prelaunch source drift'
        m.update(native_launched=True,native_work_unknown=True);save(out/'manifest.json',m)
        command(out,'native_receipt.json',cargo+['fixed_prevention_transfer_panel','--','--exact','--nocapture'],dict(os.environ,ERE_RULE_PREVENTION_TRANSFER_OUTPUT=str(out)),timeout=240)
        # Native work is known even if subsequent accounting fails.
        m.update(native_work_unknown=False,native_advances=6912,nonadvancing_seal_applications=72);save(out/'manifest.json',m)
        result=v.audit(out);save(out/'audit.json',result)
        command(out,'python_postflight.json',tests,dict(os.environ,ERE_RULE_PREVENTION_TRANSFER_AUDIT_INPUT=str(out)))
        command(out,'survival_tests.json',[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_ere_rule_survival_metrics.py','-v'])
        command(out,'feature_off_check.json',['cargo','check','--offline','--locked','-p','hg-train','--no-default-features'])
        assert pins=={n:a.digest(ROOT/n) for n in names()},'postflight source drift'
        after=primitive.prior.prior.historical.previous.feature_scope();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert after==history
        size=sum(x.stat().st_size for x in out.rglob('*') if x.is_file());assert size<v.DESIGN['output_limit_bytes']-262144
        m.update(status='complete',accounting_pass=True,transfer_pass=result['transfer_pass'],elapsed_seconds=time.time()-start,preseal_output_bytes=size)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start)
        if m['native_work_unknown']:m.update(native_advances=None,nonadvancing_seal_applications=None)
        save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out);print(json.dumps(dict(output=str(out),status='complete',transfer_pass=result['transfer_pass'],counts=result['useful_continuity_counts'],
        totals=result['totals'],source_pins=len(pins),entries=len(a.load(out/'output_sha256.json')),seal_sha256=a.digest(out/'output_sha256.json')),indent=2))
if __name__=='__main__':main()
