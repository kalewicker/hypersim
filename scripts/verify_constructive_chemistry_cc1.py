#!/usr/bin/env python3
"""Reproduce CC-1 engineering evidence in a new immutable output directory.

Build/tests have a shared 1200-second ceiling; the native fixture and semantic
audit each have a 300-second ceiling. Failures and partial receipts are retained.
Historical output inventories are verified without modifying those archives.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
CONTRACT='experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json'
FLAGS={f:False for f in ['training_admitted','scientific_acceptance','sustained_claim','evolution_enabled','scientific_run_launched','ready_for_scientific_launch','reproductive_fairness_accepted','power_adequacy_established']}

def hash_file(p):
    h=hashlib.sha256()
    with open(p,'rb') as f:
        while chunk:=f.read(1024*1024):h.update(chunk)
    return h.hexdigest()
def encoded(v):return json.dumps(v,sort_keys=True,separators=(',',':')).encode()
def now():return datetime.now(timezone.utc).isoformat()
def write(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2,sort_keys=True);f.write('\n')
def files(root):return {str(p.relative_to(root)):hash_file(p) for p in sorted(root.rglob('*')) if p.is_file()}

def run(output, reuse=None):
    output=Path(output).resolve();output.mkdir(parents=True,exist_ok=False);(output/'logs').mkdir()
    manifest=dict(run_id=output.name,experiment_name='constructive-chemistry-cc1-atomic-fixture-v1',state='Running',start_time=now(),seed=None,max_steps=2624,final_step=0,thread_count=[1,4],step_mode='scripted proposals; accepted revision only; no physical clock')
    write(output/'manifest.json',manifest)
    receipts=[];build_start=time.monotonic();cap=128*1024*1024
    def command(name,args,seconds):
        print(name,flush=True);begin=time.monotonic()
        with (output/'logs'/(name+'.log')).open('xb') as log:
            try:
                p=subprocess.run(args,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,timeout=seconds)
                code=p.returncode;error=None
            except subprocess.TimeoutExpired:
                code=None;error='timeout_cap_binding'
        receipts.append(dict(name=name,argv=args,exit_code=code,error=error,seconds=time.monotonic()-begin))
        if code!=0:raise RuntimeError(name+': '+str(error or code))
    def build_command(name,args):
        remaining=1200-(time.monotonic()-build_start)
        if remaining<=0:raise RuntimeError('shared build/test time cap bound')
        command(name,args,remaining)
    try:
        if reuse is None:
            build_command('core',['cargo','test','-p','hg-core'])
            build_command('engine_default',['cargo','test','-p','hg-engine'])
            build_command('engine_checked',['cargo','test','-p','hg-engine','--features','checked-transitions'])
            build_command('chemistry',['cargo','test','-p','hg-sim-core','--features','constructive-chemistry','--test','constructive_chemistry_cc1'])
            build_command('descendant',['cargo','test','-p','hg-sim-core','--features','evolving-minds','--test','evolving_minds_descendant'])
            build_command('ere_lifecycle',['cargo','test','-p','hg-sim-core','--features','hg-meta','--test','ere_rule_lifecycle'])
            build_command('binary',['cargo','build','-p','hg-sim-core','--features','constructive-chemistry','--bin','constructive-chemistry-cc1'])
            build_command('rust_version',['rustc','--version'])
            build_command('cargo_version',['cargo','--version'])
        else:
            reuse=Path(reuse).resolve()
            seal=json.loads((reuse/'output_sha256.json').read_bytes())
            actual=files(reuse);actual.pop('output_sha256.json')
            if actual!=seal['files_sha256']:raise RuntimeError('reused archive seal mismatch')
            former=json.loads((reuse/'validation_receipts.json').read_bytes())
            required=['core','engine_default','engine_checked','chemistry','descendant','ere_lifecycle','binary','rust_version','cargo_version']
            if any(not any(r['name']==name and r['exit_code']==0 for r in former) for name in required):raise RuntimeError('missing successful reused checks')
            old_bindings=json.loads((reuse/'source_bindings.json').read_bytes())
            changed=[path for path,h in old_bindings.items() if hash_file(ROOT/path)!=h]
            if changed!=['scripts/verify_constructive_chemistry_cc1.py']:raise RuntimeError('reuse requires only verifier correction: '+str(changed))
            for r in former:
                if r['name'] in required:
                    shutil.copy2(reuse/'logs'/(r['name']+'.log'),output/'logs'/(r['name']+'.log'))
                    receipts.append(dict(r,reused_from=str(reuse)))
            write(output/'reuse_receipt.json',dict(previous_archive=str(reuse),previous_seal_sha256=hash_file(reuse/'output_sha256.json'),successful_checks_reused=required,source_differences=changed,native_rust_config_contract_and_auditor_bytes_unchanged=True,previous_failure='payload inventory omitted the separately recorded seal file',**FLAGS))
        # Snapshot before final outcomes; do not bind an unfrozen working tree.
        frozen=output/'frozen_workspace';frozen.mkdir()
        source=[ROOT/'Cargo.toml',ROOT/'Cargo.lock']
        source += [p for folder in ['crates','rules','configs','bins'] for p in (ROOT/folder).rglob('*') if p.is_file() and p.suffix in ['.rs','.toml','.json'] and 'target' not in p.parts]
        source += [ROOT/CONTRACT,ROOT/'docs/constructive_chemistry_cc0_compatibility_spec_v1.md']
        source += [ROOT/'scripts'/name for name in ['audit_constructive_chemistry_cc1.py','test_constructive_chemistry_cc1_audit.py','verify_constructive_chemistry_cc1.py']]
        for p in sorted(set(source)):
            dest=frozen/p.relative_to(ROOT);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,dest)
        bindings=files(frozen);write(output/'source_bindings.json',bindings)
        binary=output/'constructive-chemistry-cc1';shutil.copy2(ROOT/'target/debug/constructive-chemistry-cc1',binary)
        c=json.loads((ROOT/CONTRACT).read_bytes());law={k:c[k] for k in ['law_version','schema','energy','atomicity']};write(output/'law.json',law)
        write(output/'provenance.json',dict(contract_sha256=hash_file(ROOT/CONTRACT),config_sha256=hash_file(ROOT/CONTRACT),law_sha256=hashlib.sha256(encoded(law)).hexdigest(),source_set_sha256=hashlib.sha256(encoded(bindings)).hexdigest(),binary_sha256=hash_file(binary),frozen_before_native_execution=True,law_hash_encoding='sorted compact JSON of law_version/schema/energy/atomicity',**FLAGS))
        # CC-0 previously verified the complete Evolving Minds archive set.
        old=json.loads((ROOT/'output/constructive_chemistry_cc0_20261005/prior_archive_inventory.json').read_bytes())
        prior={path:dict(receipt['files_sha256'],**{'output_sha256.json':receipt['seal_sha256']}) for path,receipt in old.items()}
        prior[str(ROOT/'output/constructive_chemistry_cc0_20261005')]=files(ROOT/'output/constructive_chemistry_cc0_20261005')
        if reuse is not None:prior[str(reuse)]=files(reuse)
        write(output/'prior_archive_inventory.json',prior)
        command('native',[str(binary),str(frozen/CONTRACT),str(output/'native.json')],300)
        command('independent_audit',[sys.executable,str(frozen/'scripts/audit_constructive_chemistry_cc1.py'),str(frozen/CONTRACT),str(output/'native.json'),'--output',str(output/'audit.json')],300)
        command('auditor_adversarial',[sys.executable,str(frozen/'scripts/test_constructive_chemistry_cc1_audit.py'),str(output/'native.json')],300)
        for path,inventory in prior.items():
            if files(Path(path))!=inventory:raise RuntimeError('historical archive changed: '+path)
        if files(frozen)!=bindings or any(hash_file(ROOT/path)!=h for path,h in bindings.items()):raise RuntimeError('source drift during verification')
        if hash_file(binary)!=hash_file(ROOT/'target/debug/constructive-chemistry-cc1'):raise RuntimeError('binary drift')
        audit=json.loads((output/'audit.json').read_bytes());native=json.loads((output/'native.json').read_bytes())
        provenance=json.loads((output/'provenance.json').read_bytes())
        if native['binary_sha256']!=provenance['binary_sha256']:raise RuntimeError('native binary binding')
        write(output/'preservation_receipt.json',dict(prior_archives_verified=len(prior),prior_files_verified=sum(map(len,prior.values())),exact_inventory_and_hashes_preserved=True,frozen_and_live_sources_unchanged=True,**FLAGS))
        write(output/'completion.json',dict(status='complete_engineering_only',gate_pass=audit['gate_pass'],cases=41,worker_variants=82,accepted_commits=audit['accepted_commits'],scientific_worlds=0,seeds=[],next_gate='CC-2 prospective kinetics/provenance/durable-continuation contract',**FLAGS))
        manifest.update(state='Completed',end_time=now(),final_step=audit['accepted_commits'])
    except Exception as e:
        manifest.update(state='Failed',end_time=now(),error=str(e))
        write(output/'failure.json',dict(status='incomplete_or_failed',error=str(e),all_existing_outputs_retained=True,gate_pass=False,**FLAGS))
    write(output/'validation_receipts.json',receipts)
    # manifest lifecycle is the only deliberate rewrite before the final seal.
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    payload=files(output);size=sum((output/p).stat().st_size for p in payload)
    seal=dict(files_sha256=payload,payload_bytes=size,file_count=len(payload),gate_pass=manifest['state']=='Completed',**FLAGS)
    write(output/'output_sha256.json',seal)
    if size+(output/'output_sha256.json').stat().st_size>cap:raise RuntimeError('total archive output cap bound; archive retained')
    print(json.dumps(dict(state=manifest['state'],output=str(output),files=len(payload)+1,payload_bytes=size,seal_sha256=hash_file(output/'output_sha256.json')),sort_keys=True),flush=True)
    if manifest['state']!='Completed':raise RuntimeError(manifest['error'])
    return output
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('new_output_directory');p.add_argument('--reuse-validated-run');args=p.parse_args();run(args.new_output_directory,args.reuse_validated_run)
