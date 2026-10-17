#!/usr/bin/env python3
"""Preserve the frozen handoff audit and repair only its resource packaging."""
import argparse
import ast
import copy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import time
import evolving_minds_opportunity as common
PROFILE='evolving-minds-d0-paid-handoff-audit-continuation-v1'
INPUT_SEAL='289591b6e0fcdf864a0618c2ac3477a04eeb83c233822ac15ee4f4bcdaff5981'
FLAGS=dict(common.FLAGS,power_adequacy_established=False,ready_for_scientific_launch=False)

def ast_hash(node):return hashlib.sha256(ast.dump(node,include_attributes=False).encode()).hexdigest()

def repaired_function(module,source):
    tree=ast.parse(source);original=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='verify');derived=copy.deepcopy(original)
    anchors=[n for n in ast.walk(derived) if isinstance(n,ast.Assign) and len(n.targets)==1 and ast.unparse(n.targets[0])=='resources[directory.name]' and ast.unparse(n.value)=='dict(total_bytes=actual, **resource)']
    common.require(len(anchors)==1,'unique frozen resource packaging expression')
    anchors[0].value.keywords[0].arg='observed_total_bytes'
    restored=copy.deepcopy(derived);anchor=next(n for n in ast.walk(restored) if isinstance(n,ast.Assign) and len(n.targets)==1 and ast.unparse(n.targets[0])=='resources[directory.name]');anchor.value.keywords[0].arg='total_bytes'
    common.require(ast_hash(restored)==ast_hash(original),'all original audit statements restored exactly')
    namespace=dict(module.__dict__);exec(compile(ast.fix_missing_locations(ast.Module(body=[derived],type_ignores=[])),'<handoff-resource-packaging-repair>','exec'),namespace)
    return namespace['verify'],dict(original_ast_sha256=ast_hash(original),derived_ast_sha256=ast_hash(derived),restored_ast_sha256=ast_hash(restored),original_ast_restored_exactly=True,metadata_keyword_renames=1,physical_predicates_changed=0,original_checker_sha256=hashlib.sha256(source.encode()).hexdigest())

def put(path,value):
    with path.open('x') as f:json.dump(value,f,indent=2);f.write('\n')

def analyze(source,destination):
    source=Path(source).resolve();destination=Path(destination);start=time.monotonic();before,size=common.sealed_input(source,INPUT_SEAL)
    destination.mkdir();manifest=dict(profile=PROFILE,state='Running',start_time=datetime.now(timezone.utc).isoformat(),native_advances=0,published_states_target=109)
    (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    try:
        paths=[('audit_continuation.py',Path(__file__)),('tests.py',Path(__file__).parent/'test_evolving_minds_handoff_audit_continuation.py'),('contract.md',Path(__file__).parent.parent/'experiments/evolving_minds/contracts/d0_paid_handoff_audit_continuation_v1.md'),('evolving_minds_opportunity.py',source/'opportunity_reference.py')]
        frozen={}
        for name,path in paths:
            raw=path.read_bytes();(destination/name).write_bytes(raw);frozen[name]=hashlib.sha256(raw).hexdigest()
        put(destination/'input_binding.json',dict(profile=PROFILE,**FLAGS,input_directory=str(source),input_seal=INPUT_SEAL,input_files=len(before),input_bytes=size,input_sha256=before,continuation_sources_sha256=frozen))
        original=common.load(source/'checker.py','handoff_original_checker');raw=(source/'checker.py').read_text();native_sources=common.data(source/'sources.json')
        common.require(raw==native_sources['scripts/verify_evolving_minds_handoff.py'],'original frozen checker source binding')
        verify,derivation=repaired_function(original,raw);put(destination/'derivation.json',derivation)
        result=verify(source);put(destination/'independent_replay.json',result)
        common.require(result['audit_pass'] and result['published_states_reconstructed']==109 and len(result['results'])==12,'complete continuation replay denominator')
        after,_=common.inventory(source);common.require(before==after,'native input preserved byte-identically')
        put(destination/'read_only_audit.json',dict(profile=PROFILE,**FLAGS,input_files=len(before),exact_inventory_and_hashes_preserved=True,input_sha256=before,elapsed_seconds=time.monotonic()-start))
        common.require(time.monotonic()-start<=3600 and common.inventory(destination)[1]<=256*1024**2,'continuation resource caps')
        put(destination/'run_status.json',dict(profile=PROFILE,**FLAGS,engineering_execution_complete=True,audit_pass=True,native_advances=0,published_states_reconstructed=109))
        manifest.update(state='Completed',end_time=datetime.now(timezone.utc).isoformat(),published_states_reconstructed=109)
        (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        print(json.dumps(dict(profile=PROFILE,audit_pass=True,native_advances=0,**FLAGS,decision=result['decision'],audit_seconds=result['audit_seconds']),indent=2))
    except Exception as error:
        manifest.update(state='Interrupted',end_time=datetime.now(timezone.utc).isoformat());(destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');(destination/'failure.txt').write_text(repr(error)+'\n');raise

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('source');parser.add_argument('destination');args=parser.parse_args();analyze(args.source,args.destination)
