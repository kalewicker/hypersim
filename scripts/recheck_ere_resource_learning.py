#!/usr/bin/env python3
"""Frozen zero-native preflight recheck; a separate --campaign mode audits saved work only."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import verify_ere_resource_learning as v
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);p.add_argument('--campaign',action='store_true');args=p.parse_args();root=args.archive.resolve()
    v.a.verify_seal(root);frozen=root/'source_snapshots/scripts/recheck_ere_resource_learning.py'
    if Path(__file__).resolve()!=frozen:subprocess.run([sys.executable,'-B',str(frozen),str(root)]+(['--campaign'] if args.campaign else []),check=True);return
    assert v.a.load(root/'manifest.json')['status']=='complete'
    if args.campaign:result=v.audit_campaign(root)
    else:
        result=v.audit_preflight(root)
        with tempfile.TemporaryDirectory(prefix='resource-learning-recheck-') as path:
            tmp=Path(path);shutil.copytree(root/'inputs',tmp/'inputs');env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_LEARNING_PREFLIGHT=str(tmp))
            for extra in [[],['saved_credit_and_arithmetic_fixture','--ignored','--exact']]:subprocess.run([str(root/'test_binary')]+extra,cwd=tmp,env=env,check=True,capture_output=True,timeout=60)
            assert v.a.load(tmp/'arithmetic.json')==v.a.load(root/'arithmetic.json')
            for i in range(8):assert v.a.digest(tmp/f'compressed-state-{i}.json.gz')==v.a.digest(root/f'compressed-state-{i}.json.gz')
            schedule=subprocess.run([str(root/'campaign_binary'),'schedule'],cwd=tmp,env=env,check=True,capture_output=True,text=True,timeout=60)
            assert json.loads(schedule.stdout)==v.a.load(root/'schedule.json')
            env['ERE_RESOURCE_LEARNING_PREFLIGHT']=str(root)
            subprocess.run([sys.executable,'-B',str(root/'source_snapshots/scripts/test_verify_ere_resource_learning.py'),'-v'],cwd=tmp,env=env,check=True,capture_output=True,timeout=60)
    assert result==v.a.load(root/'audit.json');v.a.verify_seal(root)
    print(json.dumps(dict(status='verified',seal_sha256=v.a.digest(root/'output_sha256.json'),new_native_advances=0,new_optimizer_updates=0,**result),indent=2))
if __name__=='__main__':main()
