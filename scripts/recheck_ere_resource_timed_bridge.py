#!/usr/bin/env python3
"""Recheck a copied sealed panel; no new physics and no live input fallback."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import verify_ere_resource_timed_bridge as v
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);args=p.parse_args();root=args.archive.resolve()
    frozen=root/'source_snapshots/scripts/recheck_ere_resource_timed_bridge.py'
    v.a.verify_seal(root)
    if Path(__file__).resolve()!=frozen:subprocess.run([sys.executable,'-B',str(frozen),str(root)],check=True);return
    m=v.a.load(root/'manifest.json');assert m['status']=='complete';r=v.audit(root);assert r==v.a.load(root/'audit.json')
    env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_TIMED_BRIDGE_ARCHIVE=str(root))
    tests=[sys.executable,'-B',str(root/'source_snapshots/scripts/test_verify_ere_resource_timed_bridge.py'),'-v']
    result=subprocess.run(tests,cwd=root,env=env,capture_output=True,text=True,timeout=60);assert result.returncode==0,result.stdout+result.stderr
    # Normal groups never advance native physics; never run the ignored fixture.
    result=subprocess.run([str(root/'test_binary')],cwd=root,env=env,capture_output=True,text=True,timeout=60);assert result.returncode==0,result.stdout+result.stderr
    v.a.verify_seal(root)
    print(json.dumps(dict(status='verified',seal_sha256=v.a.digest(root/'output_sha256.json'),entries=len(v.a.load(root/'output_sha256.json')),audit=r,new_native_advances=0,new_optimizer_updates=0),indent=2))
if __name__=='__main__':main()
