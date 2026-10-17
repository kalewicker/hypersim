#!/usr/bin/env python3
"""Freeze the fixed v18 named-variant comparison or engineering checks."""
import argparse,concurrent.futures,datetime,json,pathlib,shutil,subprocess,tarfile
from run_selfmod_financing import sha,load,write
def invoke(binary,source,arm,offset,steps,dest):
 with dest.with_suffix(".log").open("x") as log:
  subprocess.run([str(binary),str(source),"replace8",str(offset),str(steps),str(dest)],stdout=log,stderr=subprocess.STDOUT,check=True)
from audit_selfmod_birth import graph_stocks
ROOT=pathlib.Path(__file__).resolve().parents[1]
CONTRACT=load(ROOT/"docs/selfmod_v18_variants_contract.json")
SEEDS=CONTRACT["seeds"]
VARIANTS=CONTRACT["variants"]
ARMS=[v+"_replace8" for v in VARIANTS]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);p.add_argument('--preflight',action='store_true');args=p.parse_args();pre=args.preflight
 subprocess.run(['cargo','build','--offline','--locked','--release','-p','hg-selfmod','--bin','selfmod-variants'],cwd=ROOT,check=True)
 prior=ROOT/'output/selfmod_v17_durability_20260927';a=load(prior/'audit.json');assert a['audit_passed'] and load(prior/'status.json')['state']=='complete' and load(prior/'replay_verification.json')['passed'] and load(prior/'completion_verification.json')['passed']
 out=args.output.resolve();out.mkdir(parents=True,exist_ok=False);reference=out/'reference';reference.mkdir()
 for f in ['audit.json','manifest.json','status.json','replay_verification.json','instrument_provenance.json','preregistration.md','completion_verification.json']:
  shutil.copy2(prior/f,reference/('v17_'+f))
 v11=ROOT/'output/selfmod_v11_spectrum_20260926';v11audit=load(v11/'audit.json');assert v11audit['audit_passed'];shutil.copy2(v11/'audit.json',reference/'v11_audit.json')
 for variant,evidence in CONTRACT['named_variants'].items():
  src=v11/'mutations'/(evidence['id']+'.json');assert sha(src)==v11audit['evidence_sha256']['mutations/'+src.name]
  original=load(src);assert all(original[k]==v for k,v in evidence.items());shutil.copy2(src,out/(variant+'_mutation.json'))
 shutil.copy2(prior/'selfmod-durability',out/'v17-reference-binary');assert sha(out/'v17-reference-binary')==load(prior/'instrument_provenance.json')['binary_sha256']
 if not pre:
  engineering=ROOT/'output/selfmod_v18_variants_preflight_20260928';assert load(engineering/'audit.json')['audit_passed'] and load(engineering/'engineering_verification.json')['passed']
  assert load(engineering/'missing_case_verification.json')['passed'] and load(engineering/'source_archive_verification.json')['passed']
  for f in ['audit.json','engineering_verification.json','instrument_provenance.json','manifest.json','status.json','missing_case_verification.json','source_archive_verification.json']:shutil.copy2(engineering/f,reference/('engineering_'+f))
 history={}
 for manifest in sorted((ROOT/'output').glob('selfmod*/manifest.json')):
  d=load(manifest);used=set(d.get('seeds',[]))|set(d.get('world_seeds',[]))
  if not pre:assert not used&set(SEEDS),(manifest,used&set(SEEDS))
  history[str(manifest.relative_to(ROOT))]=dict(sha256=sha(manifest),seeds=sorted(used))
 write(out/'fresh_seed_inventory.json',dict(selected_seeds=SEEDS,prior_manifests=history,no_overlap=not pre,engineering_only=pre))
 write(out/'reference_provenance.json',dict(reference_sha256={str(f.relative_to(out)):sha(f) for f in sorted(reference.rglob('*')) if f.is_file()},named_v11_sha256={v:sha(out/(v+'_mutation.json')) for v in CONTRACT['named_variants']},reference_binary_sha256=sha(out/'v17-reference-binary')))
 sources=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'provenance.json',ROOT/'docs/selfmod_v18_variants_contract.json']+list((ROOT/'docs').glob('*preregistration.md'))
 for folder in ['crates','bins','configs','rules','fixtures','scripts','.cargo','vendor']:
  sources.extend(f for f in (ROOT/folder).rglob('*') if f.is_file() and not set(f.relative_to(ROOT).parts)&{'target','__pycache__'})
 with tarfile.open(out/'source.tar.gz','w:gz') as tar:
  for f in sorted(sources):tar.add(f,arcname=str(f.relative_to(ROOT)),recursive=False)
 shutil.copy2(ROOT/'target/release/selfmod-variants',out/'selfmod-variants');shutil.copy2(ROOT/'docs/selfmod_v18_variants_preregistration.md',out/'preregistration.md')
 shutil.copy2(ROOT/'docs/selfmod_v18_variants_contract.json',out/'contract.json')
 write(out/'instrument_provenance.json',dict(frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),binary_sha256=sha(out/'selfmod-variants'),source_archive_sha256=sha(out/'source.tar.gz'),contract_sha256=sha(out/'contract.json'),sources_sha256={str(f.relative_to(ROOT)):sha(f) for f in sorted(sources)},reference_provenance_sha256=sha(out/'reference_provenance.json'),seed_inventory_sha256=sha(out/'fresh_seed_inventory.json'),rustc=subprocess.check_output(['rustc','--version'],text=True).strip()))
 seeds=[7,42] if pre else SEEDS;steps=2000 if pre else 10000;base=load(reference/'v17_manifest.json')['configs'][0]
 write(out/'manifest.json',dict(protocol=CONTRACT['protocol'],engineering_preflight=pre,seeds=seeds,arms=ARMS,horizon=steps,physical_scoring_horizon=10000,warmup_steps=2000,window_steps=1000,subwindow_steps=250,prefix_limit=256,limit=8,replication_threshold=6,stock_floor='rent_times_live_family_copies',future_mutation=False,requested_trajectories=len(seeds)*4,max_parallel_processes=4,configs=[dict(base,seed=s,steps=10000) for s in seeds],named_variants=CONTRACT['named_variants']))
 completed=[];missing=[];prefix_steps=0;tasks=[];prefix_results=[];reference_prefix_steps=0
 write(out/'status.json',dict(state='running',trajectories_completed=0))
 try:
  for seed in seeds:
   dest=out/f'seed_{seed}';command=[str(out/'selfmod-variants'),'--prefix',str(seed),str(dest)]
   result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);(out/f'prefix_seed_{seed}.log').write_text(result.stdout);result.check_returncode();receipt=json.loads(result.stdout.strip().splitlines()[-1]);prefix_steps+=receipt['prefix_steps'];prefix_results.append(receipt)
   if not receipt['triggered']:
    missing.extend(dict(seed=seed,arm=arm,reason='untriggered') for arm in ARMS);continue
   if pre:
    baseline=out/f'v17_seed_{seed}'
    reference_result=subprocess.run([str(out/'v17-reference-binary'),'--prefix',str(seed),str(baseline)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,check=True)
    (out/f'v17_seed_{seed}.log').write_text(reference_result.stdout);reference_prefix_steps+=json.loads(reference_result.stdout.strip().splitlines()[-1])['prefix_steps']
    for f in ['initial.json','initial_allocator.json','prefix.jsonl','fork.json']+[g+'_'+v+'.json' for g in ['faithful','prune'] for v in ['birth','initial','initial_allocator']]:assert sha(dest/f)==sha(baseline/f),f
   for arm in ARMS:
    genotype,policy=arm.split('_');population=len(graph_stocks(load(dest/f'{genotype}_initial.json')))
    if policy in ['vacancy8','replace8'] and population>8:
     path=dest/arm;path.mkdir();write(path/'inadmissible.json',dict(seed=seed,arm=arm,population=population,limit=8));missing.append(dict(seed=seed,arm=arm,reason='initial population exceeds bound'));continue
    tasks.append((seed,arm))
  write(out/'prefix_results.json',prefix_results)
  def task(pair):
   seed,arm=pair;parent=out/f'seed_{seed}';dest=parent/arm;src=parent/(arm.split('_')[0]+'_initial.json')
   invoke(out/'selfmod-variants',src,arm,0,steps,dest)
   if pre:
    invoke(out/'selfmod-variants',src,arm,0,1000,parent/(arm+'_split_a'))
    invoke(out/'selfmod-variants',parent/(arm+'_split_a/final.json'),arm,1000,1000,parent/(arm+'_split_b'))
    assert (dest/'rows.jsonl').read_bytes()==(parent/(arm+'_split_a/rows.jsonl')).read_bytes()+(parent/(arm+'_split_b/rows.jsonl')).read_bytes()
    for suffix in ['', '_allocator']:
     assert sha(dest/f'final{suffix}.json')==sha(parent/(arm+f'_split_b/final{suffix}.json'))
     assert sha(dest/f'checkpoint_1000{suffix}.json')==sha(parent/(arm+f'_split_a/final{suffix}.json'))
   return dict(seed=seed,arm=arm)
  with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
   futures=[pool.submit(task,pair) for pair in tasks]
   for f in concurrent.futures.as_completed(futures):
    completed.append(f.result());write(out/'status.json',dict(state='running',trajectories_completed=len(completed),continuation_steps=len(completed)*steps,shared_prefix_steps=prefix_steps));print(f'{completed[-1]}: {len(completed)}/{len(tasks)}',flush=True)
  write(out/'summary.json',dict(trajectories_completed=len(completed),continuation_steps=len(completed)*steps,shared_prefix_steps=prefix_steps,scientific_advancing_steps=0 if pre else len(completed)*steps+prefix_steps,engineering_advancing_steps=len(completed)*steps*2+prefix_steps+reference_prefix_steps if pre else 0,fixed_seed_denominator=len(seeds),missing=missing))
  if pre:write(out/'engineering_verification.json',dict(passed=True,branches=len(completed),exact_split_rows_checkpoints_final_allocator=True,engineering_advancing_steps=len(completed)*steps*2+prefix_steps+reference_prefix_steps,reference_prefix_steps=reference_prefix_steps,frozen_v17_prefix_parity=True,excluded_from_science=True))
  write(out/'status.json',dict(state='complete',trajectories_completed=len(completed),continuation_steps=len(completed)*steps,shared_prefix_steps=prefix_steps));print('Frozen evidence: '+str(out),flush=True)
 except Exception as e:
  write(out/'status.json',dict(state='failed',error=repr(e),trajectories_completed=len(completed)));raise
if __name__=='__main__':main()
