#!/usr/bin/env python3
"""Four exact bounded replays, excluding 8000 verification steps."""
import argparse,json,pathlib,subprocess
from audit_selfmod_birth import load,sha,require
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);args=p.parse_args();out=args.output.resolve()
 require(load(out/'manifest.json')['horizon']==2000 and not load(out/'manifest.json')['engineering_preflight'],'scientific panel required')
 dest=out/'verification';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes());receipts={}
 for genotype in ['faithful','prune']:
  for policy in ['vacancy8','replace8']:
   arm=genotype+'_'+policy;target=dest/arm
   with (dest/f'{arm}.log').open('x') as log:
    subprocess.run([str(out/'selfmod-bounded'),'--replay',str(out/f'inputs/seed_11/{genotype}_on/birth.json'),policy,str(target)],stdout=log,stderr=subprocess.STDOUT,check=True)
   receipts[arm]={}
   for file in ['birth.json','rows.jsonl','score.json','final.json','reproduction.json']:
    require(sha(target/file)==sha(out/f'seed_11/{arm}/{file}'),'replay '+arm+'/'+file);receipts[arm][file]=sha(target/file)
   print(arm+': exact birth, 2000 rows, verdicts and final state',flush=True)
 with (out/'replay_verification.json').open('x') as f:json.dump(dict(passed=True,seed=11,verification_advancing_steps=8000,excluded_from_scientific_panel=True,
  binary_sha256=sha(out/'selfmod-bounded'),script_sha256=sha(script),exact_receipts=receipts),f,indent=2);f.write('\n')
if __name__=='__main__':main()
