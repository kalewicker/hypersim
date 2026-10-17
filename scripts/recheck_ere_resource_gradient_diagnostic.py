#!/usr/bin/env python3
"""Self-contained reproduction of saved arithmetic, never physics or fitting."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import ere_resource_gradient_diagnostic as producer
import verify_ere_resource_gradient_diagnostic as verifier

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path)
    p.add_argument('--expected-root');args=p.parse_args();root=args.archive.resolve()
    verifier.old.a.verify_seal(root,args.expected_root)
    frozen=root/'source_snapshots/scripts/recheck_ere_resource_gradient_diagnostic.py'
    if Path(__file__).resolve()!=frozen:
        command=[sys.executable,'-B',str(frozen),str(root)]
        if args.expected_root:command+=['--expected-root',args.expected_root]
        subprocess.run(command,check=True);return
    assert verifier.old.a.load(root/'manifest.json')['status']=='complete'
    result=verifier.audit(root);assert result==verifier.old.a.load(root/'audit.json')
    assert producer.calculate(root)==verifier.old.a.load(root/'diagnostic.json')
    env=dict(os.environ,PYTHONPATH='',PYTHONDONTWRITEBYTECODE='1',ERE_RESOURCE_GRADIENT_DIAGNOSTIC=str(root))
    subprocess.run([sys.executable,'-B',str(root/'source_snapshots/scripts/test_ere_resource_gradient_diagnostic.py'),'-v'],
                   cwd=root,env=env,check=True,capture_output=True,text=True,timeout=60)
    verifier.old.a.verify_seal(root,args.expected_root)
    print(json.dumps(dict(status='verified',seal_sha256=verifier.old.a.digest(root/'output_sha256.json'),
                         new_native_advances=0,new_optimizer_updates=0,**result),indent=2))

if __name__=='__main__':main()
