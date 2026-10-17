#!/usr/bin/env python3
"""Exactly regenerate first eligible v19b readiness/second fork and both full branches."""
import argparse,concurrent.futures,json,pathlib
from audit_selfmod_birth import load,sha,require
from run_selfmod_second_edit_semantic import invoke,command,ARMS
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve();a=load(out/'audit.json');require(a['audit_passed'] and not a['engineering_preflight'],'audited science')
 dest=out/'verification';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes());receipts={}
 if not a['forks']:report=dict(passed=False,unavailable_reason='no ready triggered seed',verification_advancing_steps=0,excluded_from_science=True)
 else:
  seed=a['forks'][0]['seed'];original=out/f'seed_{seed}';binary=out/'selfmod-second-edit-semantic';p1=command(binary,['--prefix',seed,dest/'initial_prefix'],dest/'initial_prefix.log')
  def compare(folder,files):
   for f in files:require(sha(dest/folder/f)==sha(original/folder/f),'exact receipt '+folder+'/'+f);receipts[folder+'/'+f]=sha(dest/folder/f)
  prefix_files=['initial.json','initial_allocator.json','prefix.jsonl','fork.json']+[g+'_'+part+'.json' for g in ['faithful','prune'] for part in ['birth','initial','initial_allocator']]
  compare('initial_prefix',prefix_files)
  invoke(binary,dest/'initial_prefix/prune_initial.json',0,10000,dest/'establishment')
  continuation_files=['initial.json','initial_allocator.json','rows.jsonl','final.json','final_allocator.json']+[f'checkpoint_{s}{suffix}.json' for s in range(1000,10001,1000) for suffix in ['', '_allocator']]
  compare('establishment',continuation_files);p2=command(binary,['--second',dest/'establishment/final.json',dest/'second_prefix'],dest/'second_prefix.log')
  compare('second_prefix',['initial.json','initial_allocator.json','rows.jsonl','fork.json']+[g+'_'+part+'.json' for g in ARMS for part in ['birth','initial','initial_allocator']])
  compare('second_prefix',['edited_targeting.json'])
  def branch(arm):invoke(binary,dest/f'second_prefix/{arm}_initial.json',0,10000,dest/arm);compare(arm,continuation_files);print(arm+': exact 10000-step replay',flush=True)
  with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:list(pool.map(branch,ARMS))
  report=dict(passed=True,seed=seed,verification_advancing_steps=30000+p1['prefix_steps']+p2['search_steps'],excluded_from_science=True,script_sha256=sha(script),binary_sha256=sha(binary),exact_receipts=receipts)
 with (out/'replay_verification.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps({k:v for k,v in report.items() if k!='exact_receipts'},indent=2))
if __name__=='__main__':main()
