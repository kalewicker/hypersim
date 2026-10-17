#!/usr/bin/env python3
"""Exactly replay the fixed first seed's four complete bounded extensions."""
import argparse,concurrent.futures,json,pathlib,subprocess
from audit_selfmod_birth import load,sha,require
from run_selfmod_financing import invoke

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve();m=load(out/'manifest.json')
 require(not m['engineering_preflight'] and load(out/'audit.json')['audit_passed'],'audited scientific diagnostic required')
 dest=out/'verification';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes());seed=m['seeds'][0]
 def replay(arm):
  target=dest/arm;invoke(out/'selfmod-finance',out/f'inputs/seed_{seed}/{arm}/final.json',arm,2000,8000,target)
  files=['initial.json','initial_allocator.json','rows.jsonl','final.json','final_allocator.json']+[f'checkpoint_{s}{suffix}.json' for s in range(3000,10001,1000) for suffix in ['', '_allocator']];receipts={}
  for file in files:
   require(sha(target/file)==sha(out/f'seed_{seed}/{arm}/{file}'),'exact extension '+arm+'/'+file);receipts[arm+'/'+file]=sha(target/file)
  print(arm+': exact 8000-step extension and eight checkpoints',flush=True);return receipts
 arms=[g+'_'+p for g in ['faithful','prune'] for p in ['vacancy8','replace8']];receipts={}
 with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
  for result in pool.map(replay,arms):receipts.update(result)
 with (out/'replay_verification.json').open('x') as f:json.dump(dict(passed=True,seed=seed,verification_advancing_steps=32000,excluded_from_diagnostic_panel=True,binary_sha256=sha(out/'selfmod-finance'),script_sha256=sha(script),exact_receipts=receipts),f,indent=2);f.write('\n')
if __name__=='__main__':main()
