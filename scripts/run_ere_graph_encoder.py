#!/usr/bin/env python3
"""Freeze and verify one zero-native raw-graph encoder engineering panel."""
import argparse
import difflib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import tomllib
import run_ere_rule_relational_learning as previous
import verify_ere_graph_encoder as verify

a=verify.a;ROOT=previous.ROOT;INPUT=ROOT/'output'/verify.INPUT_NAME
EXTRA=['configs/ere_graph_encoder_v1.json','docs/ere_graph_encoder_v1.md',
    'scripts/run_ere_graph_encoder.py','scripts/verify_ere_graph_encoder.py',
    'scripts/test_verify_ere_graph_encoder.py','crates/hg-train/tests/ere_graph_encoder.rs']
ARCHIVES=[
    ('ere_rule_relational_development_20261004',643,'58949cc2b129ae8cc845c3068fb5eb5d8ca7a5ffc8d88f711f6384b4fda17bcd'),
    ('ere_rule_composition_learning_trial_20261004',1445,'e6b6ef1faa22f7c2a1af0787154fc7b13a3811bc038e4f4ac9b35babc5ba262f'),
    ('ere_rule_composition_development_20261004',617,'8dff01746ecc81db6e3c3e1c3036e0de9cdf8af4050857d1150a8cbbe11ede71'),
    ('ere_rule_composition_archive_audit_v2_20261004',11,'6c1e8fb8a746c60919ab220f75e5cd46198b762a329a981c8c1c47fdd775163c'),
    ('ere_rule_choice_learning_trial_v2_20261004',758,'94ee4da611d6a1232f573770b102b6f573494cd9571b99df67cf014963317b63'),
    ('ere_rule_choice_adapter_development_20261004',283,'2d8e6c7378c9f73a3cdfee9c791c636981d8a4ddd70a1166408f8a6a4a95b5d2'),
    ('ere_rule_lifecycle_fixture_development_20261003',187,'0cdba8b56ae0d490c87f0ea6d6172fbce1ed27c81c18f24b37d9840e398b8bd5'),
    ('ere_rule_choice_learning_trial_20261004',None,'c4059b1c62715a413add601eece8172da44ae384821a0354a814cce4b050091f')]


def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True,allow_nan=False)+'\n')


def inactive(name):
    return name=='crates/hg-engine/src/checked.rs' or name.startswith('crates/hg-engine/src/checked/') or name.startswith('crates/hg-sim-core/src/constructive_chemistry/') or name=='crates/hg-sim-core/src/bin/constructive_chemistry_cc1.rs'


def source_names():return sorted(n for n in set(previous.source_names())|set(EXTRA) if not inactive(n))


def feature_scope():
    result=previous.previous.features()
    lines=[line for line in result['stdout'].splitlines() if line.startswith('hg-engine v')]
    assert lines and all('checked-transitions' not in line for line in lines)
    result.update(checked_transitions_enabled=False,constructive_chemistry_enabled=False)
    return result


def remove_exact(current,text):
    assert current.count(text)==1,'unexpected declaration multiplicity';return current.replace(text,'',1)


def preservation():
    entries=a.verify(INPUT,expected=verify.INPUT_SEAL,count=2900)
    old=a.load(INPUT/'manifest.json')['sources'];changed=[n for n,h in old.items() if a.digest(ROOT/n)!=h]
    expected=['Cargo.lock','crates/hg-engine/Cargo.toml','crates/hg-engine/src/lib.rs','crates/hg-sim-core/Cargo.toml','crates/hg-sim-core/src/lib.rs','crates/hg-train/src/ere.rs']
    assert changed==expected,'unregistered prior source body drift'
    diffs={}
    for name in changed:
        frozen=(INPUT/'source_snapshots'/name).read_text();current=(ROOT/name).read_text();stripped=current
        if name=='Cargo.lock':
            before=tomllib.loads(frozen);after=tomllib.loads(current)
            package=next(p for p in after['package'] if p['name']=='hg-engine');assert package['dependencies'].count('slotmap')==1
            package['dependencies'].remove('slotmap');assert before==after,'unexpected lock change'
        else:
            additions={
                'crates/hg-engine/Cargo.toml':['checked-transitions = ["dep:slotmap"]\n','slotmap = { workspace = true, optional = true }\n\n'],
                'crates/hg-engine/src/lib.rs':['\n/// Explicit, fail-closed graph patches; ordinary engine effects are unchanged.\n#[cfg(feature = "checked-transitions")]\npub mod checked;\n'],
                'crates/hg-sim-core/Cargo.toml':['constructive-chemistry = ["dep:sha2", "hg-engine/checked-transitions"]\n','\n[[bin]]\nname = "constructive-chemistry-cc1"\npath = "src/bin/constructive_chemistry_cc1.rs"\nrequired-features = ["constructive-chemistry"]\n'],
                'crates/hg-sim-core/src/lib.rs':['\n/// Scripted atomic chemistry engineering profile (no kinetic clock).\n#[cfg(feature = "constructive-chemistry")]\npub mod constructive_chemistry;\n'],
                'crates/hg-train/src/ere.rs':['pub mod graph_encoder;\n']}
            for text in additions[name]:stripped=remove_exact(stripped,text)
            assert stripped==frozen,'not exact registered additive-only drift'
        diffs[name]=''.join(difflib.unified_diff(frozen.splitlines(True),current.splitlines(True),fromfile='frozen/'+name,tofile='current/'+name))
    history={name:dict(entries=a.verify(ROOT/'output'/name,expected=seal,count=count),seal_sha256=seal) for name,count,seal in ARCHIVES}
    return dict(input_root_seal=verify.INPUT_SEAL,input_entries=entries,prior_unchanged_source_pins=len(old)-len(changed),registered_additive_differences=diffs,historical_archives=history)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);out=p.parse_args().output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(),'fresh project output required'
    out.mkdir();start=time.time();names=source_names();sources={n:a.digest(ROOT/n) for n in names}
    for name in names:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
    design=a.load(ROOT/EXTRA[0]);m=dict(profile=verify.PROFILE,status='running',design=design,sources=sources,classification='zero_native_encoder_engineering',maximum_native_attempts=0,native_attempts=0,training_campaign_launched=False,training_admitted=False,scientific_acceptance=False,independent_physics_replay=False)
    save(out/'manifest.json',m)
    try:
        assert design==verify.DESIGN
        before=feature_scope();save(out/'feature_scope_before.json',before)
        history=preservation();save(out/'historical_preservation.json',history)
        (out/'inputs').mkdir();inputs=[]
        for i in range(9):
            path=INPUT/'native'/f'cell-{i*16:04}.json';raw=a.load(path)['native']['initial_inventory'];save(out/'inputs'/f'world-{i}.json',raw)
            inputs.append(dict(world=i,source=str(path.relative_to(ROOT)),source_sha256=a.digest(path),saved_sha256=a.digest(out/'inputs'/f'world-{i}.json')))
        save(out/'input_provenance.json',dict(prior_root_seal=verify.INPUT_SEAL,inputs=inputs))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_graph_encoder.py','-v']
        r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=120)
        save(out/'python_preflight.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names()
        env=dict(os.environ,ERE_GRAPH_ENCODER_OUTPUT=str(out))
        command=['cargo','test','--offline','--locked','-p','hg-train','--features','ere','--test','ere_graph_encoder','--','--nocapture']
        r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=300)
        save(out/'rust_test_receipt.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        result=verify.audit(out,INPUT);save(out/'audit.json',result)
        env=dict(os.environ,ERE_GRAPH_ENCODER_AUDIT_INPUT=str(out),ERE_GRAPH_ENCODER_PRIOR=str(INPUT))
        command=[sys.executable,'-B','-m','unittest','discover','-s','scripts','-p','test_verify_ere_graph_encoder.py','-v']
        r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=180)
        save(out/'verification_test_receipt.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        command=['cargo','check','--offline','--locked','-p','hg-train','--no-default-features']
        r=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=300)
        save(out/'feature_off_check.json',dict(command=command,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0
        assert sources=={n:a.digest(ROOT/n) for n in names} and names==source_names(),'active source drift'
        after=feature_scope();save(out/'feature_scope_after.json',after);assert before==after
        after=preservation();save(out/'historical_preservation_after.json',after);assert history==after
        m.update(status='complete',accounting_pass=True,policy_records=54,joint_action_records=864,diagnostic_updates=2,executed_binary_sha256=a.digest(out/'executed_test_binary'),elapsed_seconds=time.time()-start)
        save(out/'manifest.json',m)
    except Exception as error:
        m.update(status='failed',error=repr(error),elapsed_seconds=time.time()-start);save(out/'manifest.json',m);a.seal(out);raise
    a.seal(out);print(json.dumps(dict(output=str(out),status='complete',accounting_pass=True,native_attempts=0,seal_sha256=a.digest(out/'output_sha256.json')),indent=2))


if __name__=='__main__':main()
