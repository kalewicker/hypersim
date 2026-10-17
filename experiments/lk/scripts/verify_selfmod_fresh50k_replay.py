#!/usr/bin/env python3
"""Replay fixed first selected world from ordinary initialization, any actual route."""
import argparse,concurrent.futures,json,pathlib
from audit_selfmod_birth import load,sha,require,graph_stocks
from selfmod_second_edit_common import reconstruct
from run_selfmod_fresh50k import invoke,command,ARMS

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);out=p.parse_args().output.resolve();a=load(out/'audit.json');require(a['audit_passed'] and not a['engineering_preflight'],'Audited science')
 seed=load(out/'manifest.json')['seeds'][0];original=out/f'seed_{seed}';dest=out/'verification';dest.mkdir(exist_ok=False);script=pathlib.Path(__file__);(dest/script.name).write_bytes(script.read_bytes());receipts={};advances=0;binary=out/'selfmod-reversible-edit';old=out/'v19b-reference-binary'
 def compare(folder):
  expected={f.name for f in (original/folder).iterdir() if f.is_file()};actual={f.name for f in (dest/folder).iterdir() if f.is_file()};require(expected==actual,'Exact replay inventory '+folder)
  for name in sorted(expected):
   require(sha(dest/folder/name)==sha(original/folder/name),'Exact replay '+folder+'/'+name);receipts[folder+'/'+name]=sha(dest/folder/name)
 p1=command(old,['--prefix',seed,dest/'initial_prefix'],dest/'initial_prefix.log');advances+=p1['prefix_steps'];compare('initial_prefix');kind='initial_untriggered'
 if p1['triggered']:
  kind='inadmissible'
  if len(graph_stocks(load(dest/'initial_prefix/prune_initial.json')))<=8:
   invoke(old,dest/'initial_prefix/prune_initial.json',0,10000,dest/'establishment');advances+=10000;compare('establishment');selected=load(dest/'initial_prefix/prune_birth.json')['selection'];ready=reconstruct(dest/'establishment',selected,10000)['focal'];require(ready==load(original/'readiness.json'),'First replay readiness');kind='not_ready'
   if ready['durability']['durable_success']:
    p2=command(old,['--second',dest/'establishment/final.json',dest/'second_prefix'],dest/'second_prefix.log');advances+=p2['search_steps'];compare('second_prefix');kind='second_untriggered'
    if p2['triggered']:
     invoke(old,dest/'second_prefix/edited_initial.json',0,10000,dest/'second_establishment');advances+=10000;compare('second_establishment');second=load(dest/'second_prefix/edited_birth.json')['selection'];ready=reconstruct(dest/'second_establishment',second,10000,second['child'])['focal'];require(ready==load(original/'second_readiness.json'),'Second replay readiness');kind='second_not_ready'
     if ready['durability']['durable_success']:
      p3=command(binary,['--third',dest/'second_establishment/final.json',second['child'],dest/'third_prefix'],dest/'third_prefix.log');advances+=p3['search_steps'];compare('third_prefix');kind='third_untriggered'
      if p3['triggered']:
       kind='triggered'
       def branch(arm):
        invoke(binary,dest/f'third_prefix/{arm}_initial.json',0,50000,dest/arm);compare(arm);print(arm+': exact 50000-step replay',flush=True)
       with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:list(pool.map(branch,ARMS))
       advances+=100000
 require(load(original/'routing.json')==dict(seed=seed,kind=kind),'Exact fixed first-world route, no replacement')
 require(advances<=120768,'Replay budget')
 report=dict(passed=True,seed=seed,route=kind,fresh_initialization_replayed=True,verification_advancing_steps=advances,excluded_from_science=True,script_sha256=sha(script),binary_sha256=sha(binary),reference_binary_sha256=sha(old),exact_receipts=receipts)
 with (out/'replay_verification.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps({k:v for k,v in report.items() if k!='exact_receipts'},indent=2));print('Exact receipts:',len(receipts))
if __name__=='__main__':main()
