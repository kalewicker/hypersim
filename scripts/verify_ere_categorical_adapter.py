#!/usr/bin/env python3
"""Frozen engineering replay/oracle verification; never launches native work."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import time
import audit_ere_categorical_adapter as v

def write(p,value):p.write_text(json.dumps(value,indent=2))
def freeze_binary(binary,target):
    # copyfile alone drops the executable mode and prevents any invocation.
    shutil.copy2(binary,target)
    assert target.stat().st_mode & 0o111,'source binary must be executable'
def run(binary,out):
    binary=Path(binary).resolve();out=Path(out)
    assert not out.exists(),'fresh engineering output required'
    v.a.verify(v.INPUT,expected=v.INPUT_SEAL,count=7686)
    assert binary.is_file()
    out.mkdir();start=time.monotonic();snap=v.a.load(v.INPUT/'snapshot.json');sources={}
    for name in sorted(set(snap['sources'])|set(v.SOURCES)):
        src=v.ROOT/name;dst=out/'source_snapshots'/name;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst);sources[name]=v.a.digest(dst)
    freeze_binary(binary,out/'executed_binary');shutil.copyfile(v.INPUT/'catalogue.json',out/'catalogue.json')
    (out/'source_receipts').mkdir();rows=[]
    for id in v.source_ids():
        dst=out/'source_receipts'/(id+'.json');shutil.copyfile(v.INPUT/'cells'/(id+'.json'),dst);rows.append(v.a.load(dst))
    request=v.request_from_receipts(v.a.load(out/'catalogue.json'),rows);write(out/'request.json',request)
    manifest=dict(profile=v.PROFILE,status='prepared',classification='engineering_arithmetic_only',input_archive=str(v.INPUT),input_seal=v.INPUT_SEAL,
        sources=sources,binary_sha256=v.a.digest(out/'executed_binary'),frozen_before_adapter_invocation=True,
        native_steps_run=0,grammar_compilations=0,training_updates=0,training_admitted=False,artifacts={})
    write(out/'manifest.json',manifest)
    try:
        subprocess.run([str(out.resolve()/'executed_binary'),str(out/'catalogue.json'),str(out/'request.json'),str(out/'result.json')],check=True)
        manifest['status']='complete';manifest['artifacts']={name:v.a.digest(out/name) for name in ('catalogue.json','request.json','result.json')};write(out/'manifest.json',manifest)
        audit=v.audit(out,sealed=False,tamper=True)
        assert all(v.a.digest(v.ROOT/name)==h for name,h in sources.items()),'sources changed during engineering invocation'
        assert v.a.digest(binary)==manifest['binary_sha256'],'binary changed during engineering invocation'
        v.a.verify(v.INPUT,expected=v.INPUT_SEAL,count=7686)
        write(out/'audit.json',audit);manifest['elapsed_seconds']=time.monotonic()-start;manifest['artifacts']['audit.json']=v.a.digest(out/'audit.json');write(out/'manifest.json',manifest)
        v.a.seal(out);v.a.verify(out)
        print(json.dumps(dict(status='complete',root_seal=v.a.digest(out/'output_sha256.json'),**audit),indent=2))
    except BaseException as e:
        manifest['status']='failed_engineering';manifest['error']=str(e);write(out/'manifest.json',manifest);v.a.seal(out);raise

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('binary',type=Path);parser.add_argument('output',type=Path);args=parser.parse_args();run(args.binary,args.output)
