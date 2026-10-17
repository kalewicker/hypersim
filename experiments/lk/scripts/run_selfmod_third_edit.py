#!/usr/bin/env python3
"""Freeze and run a single prospective third-edit panel; earlier stages use frozen v19b."""
import argparse,concurrent.futures,datetime,json,pathlib,shutil,subprocess,tarfile
from run_selfmod_financing import load,sha,write
from run_selfmod_second_edit_semantic import invoke,command
from audit_selfmod_birth import graph_stocks
from selfmod_second_edit_common import reconstruct
ROOT=pathlib.Path(__file__).resolve().parents[1];CONTRACT=load(ROOT/'docs/selfmod_v20_third_edit_contract.json');SEEDS=CONTRACT['seeds'];ARMS=CONTRACT['arms']
def freeze(out,pre):
 out.mkdir(parents=True,exist_ok=False);ref=out/'reference';ref.mkdir();prior=ROOT/'output/selfmod_v19b_second_edit_20260928';done=load(prior/'completion_verification.json');assert done['passed'] and all(sha(prior/f)==h for f,h in done['artifacts_sha256'].items())
 for f in ['audit.json','manifest.json','status.json','completion_verification.json','replay_verification.json','instrument_provenance.json','contract.json']:shutil.copy2(prior/f,ref/('v19b_'+f))
 for name in ['v18_manifest.json','v18_instrument_provenance.json','v11_audit.json','v19_failed_status.json']:shutil.copy2(prior/'reference'/name,ref/name)
 shutil.copy2(prior/'selfmod-second-edit-semantic',out/'v19b-reference-binary');assert sha(out/'v19b-reference-binary')==load(prior/'instrument_provenance.json')['artifacts_sha256']['selfmod-second-edit-semantic']
 v11=ROOT/'output/selfmod_v11_spectrum_20260926'
 for role in ['initial_prune','second_edit','third_edit']:
  e=CONTRACT[role];src=v11/'mutations'/(e['id']+'.json');assert sha(src)==load(ref/'v11_audit.json')['evidence_sha256']['mutations/'+src.name] and all(load(src)[k]==v for k,v in e.items());shutil.copy2(src,out/(role+'_mutation.json'))
 if not pre:
  eng=ROOT/'output/selfmod_v20_third_edit_preflight_v2_20260928'
  for f in ['audit.json','engineering_verification.json','source_archive_verification.json','missing_case_verification.json','test_verification.json']:
   assert load(eng/f).get('passed',load(eng/f).get('audit_passed',False));shutil.copy2(eng/f,ref/('engineering_'+f))
 inventory={}
 for f in sorted((ROOT/'output').glob('selfmod*/manifest.json')):
  d=load(f);used=sorted(set(d.get('seeds',[]))|set(d.get('world_seeds',[])))
  if not pre:assert not set(SEEDS)&set(used),f
  inventory[str(f.relative_to(ROOT))]=dict(sha256=sha(f),seeds=used)
 write(out/'fresh_seed_inventory.json',dict(selected_seeds=SEEDS,prior_manifests=inventory,no_overlap=not pre,engineering_only=pre));write(out/'reference_provenance.json',dict(reference_sha256={str(f.relative_to(out)):sha(f) for f in sorted(ref.rglob('*')) if f.is_file()},reference_binary_sha256=sha(out/'v19b-reference-binary')))
 shutil.copy2(ROOT/'target/release/selfmod-third-edit',out/'selfmod-third-edit');shutil.copy2(ROOT/'docs/selfmod_v20_third_edit_contract.json',out/'contract.json');shutil.copy2(ROOT/'docs/selfmod_v20_third_edit_preregistration.md',out/'preregistration.md');write(out/'composite_description.json',json.loads(subprocess.check_output([str(out/'selfmod-third-edit'),'--describe'],text=True)))
 sources=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'provenance.json']+list((ROOT/'docs').glob('*preregistration.md'))+list((ROOT/'docs').glob('*contract.json'))
 for folder in ['crates','bins','configs','rules','fixtures','scripts','.cargo','vendor']:sources.extend(f for f in (ROOT/folder).rglob('*') if f.is_file() and not set(f.relative_to(ROOT).parts)&{'target','__pycache__'})
 with tarfile.open(out/'source.tar.gz','w:gz') as t:
  for f in sorted(sources):t.add(f,arcname=str(f.relative_to(ROOT)),recursive=False)
 write(out/'instrument_provenance.json',dict(frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),sources_sha256={str(f.relative_to(ROOT)):sha(f) for f in sorted(sources)},artifacts_sha256={f:sha(out/f) for f in ['selfmod-third-edit','source.tar.gz','contract.json','preregistration.md','composite_description.json','reference_provenance.json','fresh_seed_inventory.json']},rustc=subprocess.check_output(['rustc','--version'],text=True).strip()))
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);p.add_argument('--preflight',action='store_true');a=p.parse_args();pre=a.preflight;subprocess.run(['cargo','build','--offline','--locked','--release','-p','hg-selfmod','--bin','selfmod-third-edit'],cwd=ROOT,check=True);out=a.output.resolve();freeze(out,pre);seeds=[7,42] if pre else SEEDS;horizon=2000 if pre else 10000;base=load(out/'reference/v19b_manifest.json')['configs'][0]
 write(out/'manifest.json',dict(protocol=CONTRACT['protocol'],engineering_preflight=pre,seeds=seeds,arms=ARMS,horizon=horizon,contract=CONTRACT,configs=[dict(base,seed=s,steps=10000) for s in seeds],max_parallel_processes=4,requested_trajectories=len(seeds)*2));write(out/'status.json',dict(state='running',trajectories_completed=0));old=out/'v19b-reference-binary';binary=out/'selfmod-third-edit';counts=dict(initial_prefix_steps=0,establishment_steps=0,second_prefix_steps=0,second_establishment_steps=0,third_prefix_steps=0,reference_steps=0);routes=[];tasks=[];completed=[]
 def route(seed,kind):
  r=dict(seed=seed,kind=kind);routes.append(r);write(out/f'seed_{seed}/routing.json',r)
 def preparation(seed):
  d=out/f'seed_{seed}';d.mkdir();r=command(old,['--prefix',seed,d/'initial_prefix'],d/'initial_prefix.log');local=dict(initial_prefix_steps=r['prefix_steps'],establishment_steps=0,second_prefix_steps=0,second_establishment_steps=0,third_prefix_steps=0,reference_steps=0)
  if not r['triggered']:return seed,'initial_untriggered',local,None
  if len(graph_stocks(load(d/'initial_prefix/prune_initial.json')))>8:return seed,'inadmissible',local,None
  invoke(old,d/'initial_prefix/prune_initial.json',0,10000,d/'establishment');local['establishment_steps']=10000;sel=load(d/'initial_prefix/prune_birth.json')['selection'];ready=reconstruct(d/'establishment',sel,10000)['focal'];write(d/'readiness.json',ready)
  if not ready['durability']['durable_success']:return seed,'not_ready',local,None
  r=command(old,['--second',d/'establishment/final.json',d/'second_prefix'],d/'second_prefix.log');local['second_prefix_steps']=r['search_steps']
  if not r['triggered']:return seed,'second_untriggered',local,None
  invoke(old,d/'second_prefix/edited_initial.json',0,10000,d/'second_establishment');local['second_establishment_steps']=10000;second=load(d/'second_prefix/edited_birth.json')['selection'];ready=reconstruct(d/'second_establishment',second,10000,second['child'])['focal'];write(d/'second_readiness.json',ready)
  if not ready['durability']['durable_success']:return seed,'second_not_ready',local,None
  r=command(binary,['--third',d/'second_establishment/final.json',second['child'],d/'third_prefix'],d/'third_prefix.log');local['third_prefix_steps']=r['search_steps']
  if not r['triggered']:return seed,'third_untriggered',local,None
  if pre:
   invoke(old,d/'second_establishment/final.json',20000,r['search_steps'],d/'v19b_third_reference');local['reference_steps']=r['search_steps'];fork=load(d/'third_prefix/fork.json');assert list(json.loads(x) for x in (d/'v19b_third_reference/rows.jsonl').read_text().splitlines())==list(json.loads(x) for x in (d/'third_prefix/rows.jsonl').read_text().splitlines())+[fork['faithful_birth']]
   for suffix in ['', '_allocator']:assert sha(d/f'v19b_third_reference/final{suffix}.json')==sha(d/f'third_prefix/faithful_initial{suffix}.json')
  return seed,'triggered',local,second['child']
 try:
  with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
   for seed,kind,local,founder in pool.map(preparation,seeds):
    for k,v in local.items():counts[k]+=v
    route(seed,kind)
    if kind=='triggered':tasks.extend((seed,arm) for arm in ARMS)
    print(f'{seed}: {kind}',flush=True)
  def branch(pair):
   seed,arm=pair;d=out/f'seed_{seed}';dest=d/arm;invoke(binary,d/f'third_prefix/{arm}_initial.json',0,horizon,dest)
   if pre:
    invoke(binary,d/f'third_prefix/{arm}_initial.json',0,1000,d/(arm+'_split_a'));invoke(binary,d/(arm+'_split_a/final.json'),1000,1000,d/(arm+'_split_b'));assert (dest/'rows.jsonl').read_bytes()==(d/(arm+'_split_a/rows.jsonl')).read_bytes()+(d/(arm+'_split_b/rows.jsonl')).read_bytes()
    for suffix in ['', '_allocator']:
     assert sha(dest/f'checkpoint_1000{suffix}.json')==sha(d/(arm+f'_split_a/final{suffix}.json'));assert sha(dest/f'final{suffix}.json')==sha(d/(arm+f'_split_b/final{suffix}.json'))
   return pair
  with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
   for pair in pool.map(branch,tasks):completed.append(pair);write(out/'status.json',dict(state='running',trajectories_completed=len(completed)));print(f'{pair}: {len(completed)}/{len(tasks)}',flush=True)
  advancing=sum(v for k,v in counts.items() if k!='reference_steps')+len(completed)*horizon;counts.update(continuation_steps=len(completed)*horizon,scientific_advancing_steps=0 if pre else advancing,engineering_advancing_steps=advancing+counts['reference_steps']+len(completed)*horizon if pre else 0,trajectories_completed=len(completed),fixed_seed_denominator=len(seeds),routing=routes);write(out/'summary.json',counts)
  if pre:write(out/'engineering_verification.json',dict(passed=True,exact_splits=len(completed),frozen_v19b_prior_stages_by_construction=True,faithful_third_phase_parity=True,engineering_advancing_steps=counts['engineering_advancing_steps'],excluded_from_science=True))
  write(out/'status.json',dict(state='complete',trajectories_completed=len(completed)))
 except Exception as e:write(out/'status.json',dict(state='failed',error=repr(e),trajectories_completed=len(completed)));raise
if __name__=='__main__':main()
