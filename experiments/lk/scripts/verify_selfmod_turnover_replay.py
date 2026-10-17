#!/usr/bin/env python3
"""Replay two v13 OFF branches independently; exclude 4000 verification steps."""
import argparse, json, pathlib, subprocess
from audit_selfmod_birth import load,sha,require

def main():
    p=argparse.ArgumentParser(description=__doc__); p.add_argument('output',type=pathlib.Path); args=p.parse_args(); out=args.output.resolve()
    m=load(out/'manifest.json'); require(not m['engineering_preflight'] and m['horizon']==2000,'scientific panel required')
    dest=out/'verification'; dest.mkdir(exist_ok=False); script=pathlib.Path(__file__); (dest/script.name).write_bytes(script.read_bytes())
    receipts={}
    for genotype in ['faithful','prune']:
        arm=genotype+'_off'; target=dest/arm
        with (dest/f'{arm}.log').open('x') as log:
            subprocess.run([str(out/'selfmod-turnover'),'--replay',str(out/'inputs/seed_11/fork.json'),
                str(out/f'inputs/seed_11/{genotype}/birth.json'),genotype,'off',str(target)],stdout=log,stderr=subprocess.STDOUT,check=True)
        receipts[arm]={}
        for file in ['birth.json','rows.jsonl','score.json','final.json']:
            require(sha(target/file)==sha(out/'seed_11'/arm/file),'exact replay '+arm+'/'+file); receipts[arm][file]=sha(target/file)
        print(arm+': all 2000 steps, birth/final snapshot and score exactly reproduced',flush=True)
    with (out/'replay_verification.json').open('x') as f:
        json.dump(dict(passed=True,seed=11,verification_advancing_steps=4000,excluded_from_scientific_panel=True,
            binary_sha256=sha(out/'selfmod-turnover'),script_sha256=sha(script),exact_receipts=receipts),f,indent=2);f.write('\n')
if __name__=='__main__': main()
