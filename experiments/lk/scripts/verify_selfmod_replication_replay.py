#!/usr/bin/env python3
"""Regenerate a fresh prefix and exactly replay its four bounded continuations."""
import argparse,json,pathlib,subprocess
from audit_selfmod_birth import load,sha,require,graph_stocks

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve();m=load(out/'manifest.json')
 require(not m['engineering_preflight'] and m['horizon']==2000,'scientific panel required')
 candidates=[seed for seed in m['seeds'] if (out/f'seed_{seed}/fork.json').exists() and len(graph_stocks(load(out/f'seed_{seed}/faithful_birth.json')['initial']))<=8]
 dest=out/'verification';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes())
 if not candidates:
  report=dict(passed=False,unavailable_reason='no cap-admissible triggered seed',excluded_from_scientific_panel=True,verification_advancing_steps=0)
 else:
  seed=candidates[0];receipts={};target=dest/'prefix'
  with (dest/'prefix.log').open('x') as log:subprocess.run([str(out/'selfmod-replicate'),'--verify-prefix',str(seed),str(target)],stdout=log,stderr=subprocess.STDOUT,check=True)
  for file in ['initial.json','prefix.jsonl','fork.json','faithful_birth.json','prune_birth.json']:
   require(sha(target/file)==sha(out/f'seed_{seed}'/file),'fresh prefix '+file);receipts['prefix/'+file]=sha(target/file)
  prefix_steps=load(target/'fork.json')['pending']['absolute_step'];print(f'seed {seed}: exact fresh initialization, {prefix_steps}-step prefix, fork and both paid births',flush=True)
  for genotype in ['faithful','prune']:
   for policy in ['vacancy8','replace8']:
    arm=genotype+'_'+policy;target=dest/arm
    with (dest/f'{arm}.log').open('x') as log:
     subprocess.run([str(out/'selfmod-replicate'),'--replay',str(out/f'seed_{seed}/{genotype}_birth.json'),policy,str(target)],stdout=log,stderr=subprocess.STDOUT,check=True)
    for file in ['birth.json','rows.jsonl','score.json','reproduction.json','final.json']:
     require(sha(target/file)==sha(out/f'seed_{seed}/{arm}/{file}'),'bounded replay '+arm+'/'+file);receipts[arm+'/'+file]=sha(target/file)
    print(arm+': exact 2000-step bounded continuation, verdicts and final state',flush=True)
  report=dict(passed=True,seed=seed,verification_advancing_steps=8000+prefix_steps,verification_prefix_steps=prefix_steps,verification_continuation_steps=8000,
   excluded_from_scientific_panel=True,binary_sha256=sha(out/'selfmod-replicate'),script_sha256=sha(script),exact_receipts=receipts)
 with (out/'replay_verification.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
if __name__=='__main__':main()
