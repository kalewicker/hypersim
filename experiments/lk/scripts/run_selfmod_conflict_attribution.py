#!/usr/bin/env python3
"""Run four frozen retrospective targets, with archived controls and exact parity."""
import argparse,datetime,json,pathlib,subprocess
from audit_selfmod_birth import load,sha,require

def save(path,x):
 with path.open('x') as f:json.dump(x,f,indent=2);f.write('\n')
def run(binary,initial,offset,steps,dest):
 with dest.with_suffix('.log').open('x') as log:
  subprocess.run([str(binary),str(initial),str(offset),str(steps),str(dest)],stdout=log,stderr=subprocess.STDOUT,check=True)
def main():
 p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);out=p.parse_args().output.resolve();manifest=load(out/'manifest.json');seal=load(out/'execution_seal.json');contract=load(out/'contract.json');receipts={};completed=0
 for file,digest in seal['files_sha256'].items():require(sha(out/file)==digest,'Pre-execution sealed input '+file)
 try:
  for entry in manifest['targets']:
   folder=out/entry['name'];initial=folder/'input/initial.json';offset=entry['offset'];steps=entry['steps']
   run(out/'reference-native',initial,offset,steps,folder/'control')
   expected=folder/'input/expected_rows.jsonl';require(sha(folder/'control/rows.jsonl')==sha(expected),'Reference archive slice '+entry['name'])
   for native,archived in [('initial.json','initial.json'),('initial_allocator.json','initial_allocator.json'),('final.json','expected_final.json'),('final_allocator.json','expected_final_allocator.json'),(f'checkpoint_{offset+steps}.json','expected_final.json'),(f'checkpoint_{offset+steps}_allocator.json','expected_final_allocator.json')]:
    require(sha(folder/'control'/native)==sha(folder/'input'/archived),'Reference endpoint '+entry['name']+'/'+native)
   run(out/'selfmod-conflict-attribution',initial,offset,steps,folder/'observed')
   for f in sorted((folder/'control').iterdir()):
    if f.is_file():require(sha(f)==sha(folder/'observed'/f.name),'Observed native parity '+entry['name']+'/'+f.name);receipts[entry['name']+'/'+f.name]=sha(f)
   require(load(folder/'observed/status.json')==dict(state='complete',offset=offset,steps_completed=steps),'Complete fixed target')
   completed+=1;print(entry['name']+': reference/archive and observed/control exact',flush=True)
  advances=sum(e['steps']*2 for e in manifest['targets']);require(advances==8000<=contract['max_target_advancing_steps'],'Engineering target budget')
  save(out/'native_parity.json',dict(passed=True,targets_completed=completed,target_advancing_steps=advances,new_scientific_advancing_steps=0,exact_observed_native_receipts=receipts,runner_sha256=sha(pathlib.Path(__file__)),reference_binary_sha256=sha(out/'reference-native'),observer_binary_sha256=sha(out/'selfmod-conflict-attribution')))
  save(out/'status.json',dict(state='complete',targets_completed=completed,engineering_target_advancing_steps=advances,new_scientific_advancing_steps=0))
 except Exception as e:
  save(out/'status.json',dict(state='failed',targets_completed=completed,error=str(e),evidence_retained=True));raise
if __name__=='__main__':main()
