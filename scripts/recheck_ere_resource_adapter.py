#!/usr/bin/env python3
"""Self-contained recheck: frozen verifier, tests, binary and copied inputs."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import verify_ere_resource_adapter as v

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);args=p.parse_args();root=args.archive.resolve()
    # When called from the live helper, dispatch into the sealed source snapshot.
    frozen=root/'source_snapshots/scripts/recheck_ere_resource_adapter.py'
    if Path(__file__).resolve()!=frozen:
        v.verify_seal(root)
        r=subprocess.run([sys.executable,'-B',str(frozen),str(root)],check=True)
        return
    entries=v.verify_seal(root);m=v.load(root/'manifest.json');assert m['status']=='complete'
    result=v.audit(root);assert result==v.load(root/'audit.json')
    with tempfile.TemporaryDirectory(prefix='ere-resource-adapter-') as tmp:
        tmp=Path(tmp);shutil.copytree(root/'inputs',tmp/'inputs')
        env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_ADAPTER_ARCHIVE=str(tmp))
        for extra in [[],['fixed_receipt_fixture','--ignored','--exact']]:
            r=subprocess.run([str(root/'test_binary')]+extra,cwd=tmp,env=env,capture_output=True,text=True,timeout=60)
            assert r.returncode==0,r.stdout+r.stderr
        assert v.load(tmp/'adapter_records.json')==v.load(root/'adapter_records.json')
        env['ERE_RESOURCE_ADAPTER_ARCHIVE']=str(root)
        r=subprocess.run([sys.executable,'-B',str(root/'source_snapshots/scripts/test_verify_ere_resource_adapter.py'),'-v'],cwd=tmp,env=env,capture_output=True,text=True,timeout=60)
        assert r.returncode==0,r.stdout+r.stderr
    v.verify_seal(root)
    print(json.dumps(dict(status='verified',entries=entries,seal_sha256=v.digest(root/'output_sha256.json'),binary_reproduction=True,rust_groups=4,python_groups=6,**result),indent=2))
if __name__=='__main__':main()
