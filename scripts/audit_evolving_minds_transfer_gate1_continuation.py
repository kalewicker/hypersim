#!/usr/bin/env python3
"""Pinned read-only audit continuation: rename a shadowing local, preserve all checks."""
import argparse
import ast
import copy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sys
import time
sys.dont_write_bytecode=True
import evolving_minds_opportunity as common

PROFILE='evolving-minds-ta1-gate1-audit-continuation-v1'
NATIVE_SEAL='f23ef0ab8142399006e280f52332104a776b2842948602847f3e563650741661'
FAILED_AUDIT_SEAL='2cc5e546c29d08713ed81ec2a3bb829142124c1a0913a3edfc988fb700e2d7af'

def ast_hash(node):
    return hashlib.sha256(ast.dump(node,include_attributes=False).encode()).hexdigest()

def derive(module):
    tree=ast.parse(Path(module.__file__).read_text())
    original=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='verify')
    common.require(not any(isinstance(n,ast.Name) and n.id=='_registered_cell' for n in ast.walk(original)),
                   'fresh continuation local name')
    class Rename(ast.NodeTransformer):
        def visit_Name(self,node):
            if node.id=='case':node.id='_registered_cell'
            return node
        def visit_Call(self,node):
            if isinstance(node.func,ast.Name) and node.func.id=='case':
                node.args=[self.visit(a) for a in node.args]
                node.keywords=[self.visit(k) for k in node.keywords]
                return node
            return self.generic_visit(node)
    derived=Rename().visit(copy.deepcopy(original))
    restored=copy.deepcopy(derived)
    count=0
    for node in ast.walk(restored):
        if isinstance(node,ast.Name) and node.id=='_registered_cell':node.id='case';count+=1
    common.require(count>0 and ast_hash(restored)==ast_hash(original),'exact original verifier AST restoration')
    namespace=dict(module.__dict__)
    code=ast.fix_missing_locations(ast.Module(body=[derived],type_ignores=[]))
    exec(compile(code,'<ta1-pinned-local-rename-continuation>','exec'),namespace)
    return namespace['verify'],dict(original_ast_sha256=ast_hash(original),
        derived_ast_sha256=ast_hash(derived),restored_ast_sha256=ast_hash(restored),
        original_ast_restored_exactly=True,renamed_local_references=count,
        physical_predicates_changed=0,thresholds_changed=0,native_advances=0)

def sealed(root,pin):
    files,size=common.inventory(root)
    common.require(files.get('output_sha256.json')==pin,'pinned input identity')
    payload=dict(files);payload.pop('output_sha256.json')
    common.require(common.data(root/'output_sha256.json')['files_sha256']==payload,'complete sealed input')
    return files,size

def run(native,failed,destination):
    native,failed,destination=map(lambda x:Path(x).resolve(),[native,failed,destination])
    for root in [native,failed]:common.require(destination!=root and root not in destination.parents,'separate audit destination')
    before=sealed(native,NATIVE_SEAL);old=sealed(failed,FAILED_AUDIT_SEAL)
    common.require(common.data(failed/'manifest.json')['state']=='Interrupted' and
        common.data(failed/'failure.json')['error']=='''TypeError("'dict' object is not callable")''',
        'preserved declared packaging failure')
    destination.mkdir();started=time.monotonic()
    def put(name,value):
        with (destination/name).open('x') as f:json.dump(value,f,indent=2,sort_keys=True,allow_nan=False);f.write('\n')
    manifest=dict(experiment_name=PROFILE,state='Running',max_steps=0,final_step=0,
        start_time=datetime.now(timezone.utc).isoformat(),end_time=None)
    put('manifest.json',manifest)
    try:
        (destination/'continuation.py').write_bytes(Path(__file__).read_bytes())
        put('input_binding.json',dict(native_directory=str(native),failed_audit_directory=str(failed),
            native_seal_sha256=NATIVE_SEAL,failed_audit_seal_sha256=FAILED_AUDIT_SEAL,
            native_files=len(before[0]),native_bytes=before[1],failed_audit_files=len(old[0])))
        checker=common.load(native/'checker.py','ta1_frozen_packaging_checker')
        verify,derivation=derive(checker);put('derivation.json',derivation)
        result=verify(native)
        result['native_seal_sha256']=NATIVE_SEAL
        put('independent_replay.json',result)
        common.require(sealed(native,NATIVE_SEAL)==before and sealed(failed,FAILED_AUDIT_SEAL)==old,'all original inputs preserved')
        common.require(time.monotonic()-started<=3600,'continuation audit time cap')
        put('run_status.json',dict(profile=PROFILE,engineering_execution_complete=True,audit_pass=True,
            gate1_development_feasibility_pass=result['decision']['gate1_development_feasibility_pass'],
            native_advances=0,transfer_established=False,discovery_established=False,
            cumulative_reuse_established=False,**checker.FLAGS))
        manifest['state']='Completed'
        print(json.dumps(dict(audit_pass=True,decision=result['decision'],receipt=str(destination)),indent=2))
    except Exception as error:
        manifest['state']='Interrupted';put('failure.json',dict(error=repr(error),audit_pass=False,native_advances=0));raise
    finally:
        manifest['end_time']=datetime.now(timezone.utc).isoformat()
        (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        files,_=common.inventory(destination)
        put('output_sha256.json',dict(profile=PROFILE,files_sha256=files,development_only=True,scientific_run_launched=False))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['native','failed_audit','destination']:parser.add_argument(name)
    args=parser.parse_args();run(args.native,args.failed_audit,args.destination)
