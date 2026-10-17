#!/usr/bin/env python3
"""Regenerate the first eligible fresh v17 prefix and replay four complete bounded arms."""
import argparse,concurrent.futures,json,pathlib,subprocess
from audit_selfmod_birth import load,sha,require
from run_selfmod_financing import invoke

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve();m=load(out/'manifest.json');audit=load(out/'audit.json')
 require(audit['audit_passed'] and not m['engineering_preflight'],'audited fresh scientific panel')
 candidates=[r['seed'] for r in audit['forks'] if r['cap_admissible']];dest=out/'verification';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes());receipts={}
 if not candidates:
  report=dict(passed=False,unavailable_reason='no cap-admissible triggered seed',verification_advancing_steps=0,excluded_from_scientific_panel=True)
 else:
  seed=candidates[0];target=dest/'prefix'
  result=subprocess.run([str(out/'selfmod-durability'),'--prefix',str(seed),str(target)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,check=True);(dest/'prefix.log').write_text(result.stdout);prefix_steps=json.loads(result.stdout.strip().splitlines()[-1])['prefix_steps']
  files=['initial.json','initial_allocator.json','prefix.jsonl','fork.json']+[f'{g}_{f}.json' for g in ['faithful','prune'] for f in ['birth','initial','initial_allocator']]
  for f in files:require(sha(target/f)==sha(out/f'seed_{seed}'/f),'fresh complete prefix '+f);receipts['prefix/'+f]=sha(target/f)
  print(f'seed {seed}: exact {prefix_steps}-step fresh prefix, paid births and allocator orders',flush=True)
  def replay(arm):
   target=dest/arm;genotype=arm.split('_')[0];invoke(out/'selfmod-durability',out/f'seed_{seed}/{genotype}_initial.json',arm,0,10000,target)
   files=['initial.json','initial_allocator.json','rows.jsonl','final.json','final_allocator.json']+[f'checkpoint_{s}{suffix}.json' for s in range(1000,10001,1000) for suffix in ['', '_allocator']];result={}
   for f in files:require(sha(target/f)==sha(out/f'seed_{seed}/{arm}/{f}'),'exact fresh continuation '+arm+'/'+f);result[arm+'/'+f]=sha(target/f)
   print(arm+': exact 10000-step continuation and complete checkpoints',flush=True);return result
  arms=[g+'_'+p for g in ['faithful','prune'] for p in ['vacancy8','replace8']]
  with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
   for result in pool.map(replay,arms):receipts.update(result)
  report=dict(passed=True,seed=seed,verification_advancing_steps=40000+prefix_steps,excluded_from_scientific_panel=True,binary_sha256=sha(out/'selfmod-durability'),script_sha256=sha(script),exact_receipts=receipts)
 with (out/'replay_verification.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
if __name__=='__main__':main()
