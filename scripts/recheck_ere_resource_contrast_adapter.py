#!/usr/bin/env python3
"""Frozen zero-native contrast preflight reproduction, no training entry point."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import verify_ere_resource_contrast_adapter as v
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);p.add_argument('--expected-root')
    args=p.parse_args();root=args.archive.resolve();v.old.a.verify_seal(root,args.expected_root)
    frozen=root/'source_snapshots/scripts/recheck_ere_resource_contrast_adapter.py'
    if Path(__file__).resolve()!=frozen:
        command=[sys.executable,'-B',str(frozen),str(root)]
        if args.expected_root:command+=['--expected-root',args.expected_root]
        subprocess.run(command,check=True);return
    assert v.old.a.load(root/'manifest.json')['status']=='complete'
    audit=v.audit(root);assert audit==v.old.a.load(root/'audit.json')
    with tempfile.TemporaryDirectory(prefix='ere-contrast-recheck-') as path:
        tmp=Path(path);shutil.copytree(root/'inputs',tmp/'inputs')
        env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_CONTRAST_PREFLIGHT=str(tmp))
        for extra in [[],['saved_contrast_fixture','--ignored','--exact']]:
            subprocess.run([str(root/'test_binary')]+extra,cwd=tmp,env=env,check=True,capture_output=True,text=True,timeout=60)
        assert v.old.a.load(tmp/'contrast_records.json')==v.old.a.load(root/'contrast_records.json')
        env['ERE_RESOURCE_CONTRAST_PREFLIGHT']=str(root)
        subprocess.run([sys.executable,'-B',str(root/'source_snapshots/scripts/test_verify_ere_resource_contrast_adapter.py'),'-v'],
                       cwd=tmp,env=env,check=True,capture_output=True,text=True,timeout=60)
    v.old.a.verify_seal(root,args.expected_root)
    print(json.dumps(dict(status='verified',seal_sha256=v.old.a.digest(root/'output_sha256.json'),new_native_advances=0,new_optimizer_updates=0,**audit),indent=2))
if __name__=='__main__':main()
