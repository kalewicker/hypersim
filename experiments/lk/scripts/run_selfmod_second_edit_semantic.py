#!/usr/bin/env python3
"""Freeze and run the prospective second-edit panel with audited readiness."""
import argparse,concurrent.futures,datetime,json,pathlib,shutil,subprocess,tarfile
from run_selfmod_financing import load,sha,write
from audit_selfmod_birth import graph_stocks
from selfmod_second_edit_common import reconstruct
ROOT=pathlib.Path(__file__).resolve().parents[1]
CONTRACT=load(ROOT/'docs/selfmod_v19b_second_edit_contract.json');SEEDS=CONTRACT['seeds'];ARMS=CONTRACT['arms']
def invoke(binary,initial,offset,steps,out):
 with out.with_suffix('.log').open('x') as log:subprocess.run([str(binary),str(initial),str(offset),str(steps),str(out)],stdout=log,stderr=subprocess.STDOUT,check=True)
def command(binary,args,log):
 result=subprocess.run([str(binary)]+[str(x) for x in args],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);log.write_text(result.stdout);result.check_returncode();return json.loads(result.stdout.strip().splitlines()[-1])
def freeze(out,pre):
 out.mkdir(parents=True,exist_ok=False);ref=out/'reference';ref.mkdir();prior=ROOT/'output/selfmod_v18_variants_20260928'
 assert load(prior/'audit.json')['audit_passed'] and load(prior/'completion_verification.json')['passed'] and load(prior/'replay_verification.json')['passed']
 for f in ['audit.json','manifest.json','status.json','completion_verification.json','replay_verification.json','instrument_provenance.json']:shutil.copy2(prior/f,ref/('v18_'+f))
 failed=ROOT/'output/selfmod_v19_second_edit_20260928'
 assert load(failed/'status.json')['state']=='failed' and load(failed/'status.json')['trajectories_completed']==0
 for f in ['manifest.json','status.json','instrument_provenance.json','contract.json']:shutil.copy2(failed/f,ref/('v19_failed_'+f))
 interruption=ROOT/'output/selfmod_v19_interruption_verification_20260928/interrupted_run_verification.json'
 assert load(interruption)['completed_stages_verified'] and not load(interruption)['full_assay_complete']
 shutil.copy2(interruption,ref/'v19_interrupted_run_verification.json')
 correction=ROOT/'output/selfmod_v19_semantic_target_engineering_20260928/verification.json'
 assert load(correction)['passed'] and load(correction)['object_steps']==0 and load(correction)['both_native_orders_verified']
 assert all(sha(ROOT/f)==h for f,h in load(correction)['preserved_raw_artifacts_sha256'].items())
 shutil.copy2(correction,ref/'v19_semantic_target_verification.json')
 v11=ROOT/'output/selfmod_v11_spectrum_20260926';a=load(v11/'audit.json');assert a['audit_passed'];shutil.copy2(v11/'audit.json',ref/'v11_audit.json')
 for role in ['initial_prune','second_edit']:
  evidence=CONTRACT[role];source=v11/'mutations'/(evidence['id']+'.json');assert sha(source)==a['evidence_sha256']['mutations/'+source.name] and all(load(source)[k]==v for k,v in evidence.items());shutil.copy2(source,out/(role+'_mutation.json'))
 shutil.copy2(prior/'selfmod-variants',out/'v18-reference-binary');assert sha(out/'v18-reference-binary')==load(prior/'instrument_provenance.json')['binary_sha256']
 if not pre:
  engineering=ROOT/'output/selfmod_v19b_second_edit_preflight_20260928'
  for f in ['audit.json','engineering_verification.json','missing_case_verification.json','source_archive_verification.json','semantic_target_verification.json']:assert load(engineering/f).get('passed',load(engineering/f).get('audit_passed',False))
  for f in ['audit.json','engineering_verification.json','missing_case_verification.json','source_archive_verification.json','semantic_target_verification.json','manifest.json','instrument_provenance.json']:shutil.copy2(engineering/f,ref/('engineering_'+f))
 inventory={}
 for f in sorted((ROOT/'output').glob('selfmod*/manifest.json')):
  d=load(f);used=sorted(set(d.get('seeds',[]))|set(d.get('world_seeds',[])))
  if not pre:assert not set(used)&set(SEEDS),f
  inventory[str(f.relative_to(ROOT))]=dict(sha256=sha(f),seeds=used)
 write(out/'fresh_seed_inventory.json',dict(selected_seeds=SEEDS,prior_manifests=inventory,no_overlap=not pre,engineering_only=pre))
 write(out/'reference_provenance.json',dict(reference_sha256={str(f.relative_to(out)):sha(f) for f in sorted(ref.rglob('*')) if f.is_file()},mutation_sha256={role:sha(out/(role+'_mutation.json')) for role in ['initial_prune','second_edit']},reference_binary_sha256=sha(out/'v18-reference-binary')))
 shutil.copy2(ROOT/'target/release/selfmod-second-edit-semantic',out/'selfmod-second-edit-semantic');shutil.copy2(ROOT/'docs/selfmod_v19b_second_edit_contract.json',out/'contract.json');shutil.copy2(ROOT/'docs/selfmod_v19b_second_edit_preregistration.md',out/'preregistration.md')
 write(out/'composite_description.json',json.loads(subprocess.check_output([str(out/'selfmod-second-edit-semantic'),'--describe'],text=True)))
 sources=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'provenance.json']+list((ROOT/'docs').glob('*preregistration.md'))+list((ROOT/'docs').glob('*contract.json'))
 for folder in ['crates','bins','configs','rules','fixtures','scripts','.cargo','vendor']:sources.extend(f for f in (ROOT/folder).rglob('*') if f.is_file() and not set(f.relative_to(ROOT).parts)&{'target','__pycache__'})
 with tarfile.open(out/'source.tar.gz','w:gz') as t:
  for f in sorted(sources):t.add(f,arcname=str(f.relative_to(ROOT)),recursive=False)
 write(out/'instrument_provenance.json',dict(frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),sources_sha256={str(f.relative_to(ROOT)):sha(f) for f in sorted(sources)},artifacts_sha256={f:sha(out/f) for f in ['selfmod-second-edit-semantic','source.tar.gz','contract.json','preregistration.md','composite_description.json','reference_provenance.json','fresh_seed_inventory.json']},rustc=subprocess.check_output(['rustc','--version'],text=True).strip()))
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);p.add_argument('--preflight',action='store_true');args=p.parse_args();pre=args.preflight
 subprocess.run(['cargo','build','--offline','--locked','--release','-p','hg-selfmod','--bin','selfmod-second-edit-semantic'],cwd=ROOT,check=True)
 out=args.output.resolve();freeze(out,pre);seeds=[7,42] if pre else SEEDS;horizon=2000 if pre else 10000;base=load(out/'reference/v18_manifest.json')['configs'][0]
 write(out/'manifest.json',dict(protocol=CONTRACT['protocol'],engineering_preflight=pre,seeds=seeds,arms=ARMS,horizon=horizon,contract=CONTRACT,configs=[dict(base,seed=s,steps=10000) for s in seeds],max_parallel_processes=4,requested_trajectories=len(seeds)*2))
 write(out/'status.json',dict(state='running',trajectories_completed=0));routing=[];tasks=[];completed=[];first_steps=0;establishment_steps=0;search_steps=0;reference_steps=0
 try:
  valid=[]
  for seed in seeds:
   directory=out/f'seed_{seed}';directory.mkdir();result=command(out/'selfmod-second-edit-semantic',['--prefix',seed,directory/'initial_prefix'],directory/'initial_prefix.log');first_steps+=result['prefix_steps']
   if not result['triggered']:route=dict(seed=seed,kind='initial_untriggered');routing.append(route);write(directory/'routing.json',route);continue
   if len(graph_stocks(load(directory/'initial_prefix/prune_initial.json')))>8:route=dict(seed=seed,kind='inadmissible');routing.append(route);write(directory/'routing.json',route);continue
   valid.append(seed)
  def establish(seed):
   directory=out/f'seed_{seed}';invoke(out/'selfmod-second-edit-semantic',directory/'initial_prefix/prune_initial.json',0,10000,directory/'establishment');return seed
  with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
   for seed in pool.map(establish,valid):establishment_steps+=10000;print(f'{seed}: establishment complete',flush=True)
  for seed in valid:
   directory=out/f'seed_{seed}';selected=load(directory/'initial_prefix/prune_birth.json')['selection'];readiness=reconstruct(directory/'establishment',selected,10000)['focal'];write(directory/'readiness.json',readiness)
   if pre:
    reference=directory/'v18_reference';reference.mkdir();old=command(out/'v18-reference-binary',['--prefix',seed,reference/'prefix'],reference/'prefix.log');reference_steps+=old['prefix_steps']
    for f in ['initial.json','initial_allocator.json','prefix.jsonl','fork.json']+[g+'_'+part+'.json' for g in ['faithful','prune'] for part in ['birth','initial','initial_allocator']]:assert sha(directory/'initial_prefix'/f)==sha(reference/'prefix'/f),f
    with (reference/'establishment.log').open('x') as log:subprocess.run([str(out/'v18-reference-binary'),str(reference/'prefix/prune_initial.json'),'replace8','0','10000',str(reference/'establishment')],stdout=log,stderr=subprocess.STDOUT,check=True)
    reference_steps+=10000
    for f in ['rows.jsonl','final.json','final_allocator.json']+[f'checkpoint_{s}{suffix}.json' for s in range(1000,10001,1000) for suffix in ['', '_allocator']]:assert sha(directory/'establishment'/f)==sha(reference/'establishment'/f),f
   if not readiness['durability']['durable_success']:route=dict(seed=seed,kind='not_ready');routing.append(route);write(directory/'routing.json',route);continue
   result=command(out/'selfmod-second-edit-semantic',['--second',directory/'establishment/final.json',directory/'second_prefix'],directory/'second_prefix.log');search_steps+=result['search_steps']
   if not result['triggered']:route=dict(seed=seed,kind='second_untriggered');routing.append(route);write(directory/'routing.json',route);continue
   if pre:
    ref=directory/'v18_reference';dest=ref/'second_search'
    with (ref/'second_search.log').open('x') as log:subprocess.run([str(out/'v18-reference-binary'),str(directory/'establishment/final.json'),'replace8','10000',str(result['search_steps']),str(dest)],stdout=log,stderr=subprocess.STDOUT,check=True)
    reference_steps+=result['search_steps'];faithful=load(directory/'second_prefix/faithful_birth.json')['row'];expected=(directory/'second_prefix/rows.jsonl').read_bytes()+json.dumps(faithful,separators=(',',':')).encode()+b'\n'
    assert list(json.loads(x) for x in expected.splitlines())==list(json.loads(x) for x in (dest/'rows.jsonl').read_bytes().splitlines())
    for suffix in ['', '_allocator']:assert sha(directory/f'second_prefix/faithful_initial{suffix}.json')==sha(dest/f'final{suffix}.json')
   route=dict(seed=seed,kind='triggered');routing.append(route);write(directory/'routing.json',route);tasks.extend((seed,arm) for arm in ARMS)
  def branch(pair):
   seed,arm=pair;directory=out/f'seed_{seed}';src=directory/f'second_prefix/{arm}_initial.json';dest=directory/arm;invoke(out/'selfmod-second-edit-semantic',src,0,horizon,dest)
   if pre:
    invoke(out/'selfmod-second-edit-semantic',src,0,1000,directory/(arm+'_split_a'));invoke(out/'selfmod-second-edit-semantic',directory/(arm+'_split_a/final.json'),1000,1000,directory/(arm+'_split_b'))
    assert (dest/'rows.jsonl').read_bytes()==(directory/(arm+'_split_a/rows.jsonl')).read_bytes()+(directory/(arm+'_split_b/rows.jsonl')).read_bytes()
    for suffix in ['', '_allocator']:
     assert sha(dest/f'checkpoint_1000{suffix}.json')==sha(directory/(arm+f'_split_a/final{suffix}.json'));assert sha(dest/f'final{suffix}.json')==sha(directory/(arm+f'_split_b/final{suffix}.json'))
   return dict(seed=seed,arm=arm)
  with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
   for result in pool.map(branch,tasks):completed.append(result);write(out/'status.json',dict(state='running',trajectories_completed=len(completed)));print(f'{result}: {len(completed)}/{len(tasks)}',flush=True)
  counts=dict(initial_prefix_steps=first_steps,establishment_steps=establishment_steps,second_prefix_steps=search_steps,continuation_steps=len(completed)*horizon,scientific_advancing_steps=0 if pre else first_steps+establishment_steps+search_steps+len(completed)*horizon,engineering_advancing_steps=first_steps+establishment_steps+search_steps+reference_steps+len(completed)*horizon*2 if pre else 0,reference_steps=reference_steps,trajectories_completed=len(completed),fixed_seed_denominator=len(seeds),routing=sorted(routing,key=lambda r:seeds.index(r['seed'])))
  write(out/'summary.json',counts)
  if pre:write(out/'engineering_verification.json',dict(passed=True,exact_splits=len(completed),frozen_v18_initial_readiness_second_birth_parity=True,engineering_advancing_steps=counts['engineering_advancing_steps'],excluded_from_science=True))
  write(out/'status.json',dict(state='complete',trajectories_completed=len(completed)))
 except Exception as e:write(out/'status.json',dict(state='failed',error=repr(e),trajectories_completed=len(completed)));raise
if __name__=='__main__':main()
