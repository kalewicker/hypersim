#!/usr/bin/env python3
"""Freeze and run continuous v16 extensions or exact split engineering checks."""
import argparse,concurrent.futures,datetime,hashlib,json,pathlib,shutil,subprocess,tarfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
SEEDS=[131,137,149,157,163,173,181,191]
ARMS=[g+'_'+p for g in ['faithful','prune'] for p in ['on','off','vacancy8','replace8']]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def load(p):return json.loads(p.read_text())
def write(p,data):
 tmp=p.with_suffix('.tmp');tmp.write_text(json.dumps(data,indent=2)+'\n');tmp.replace(p)
def invoke(binary,source,arm,offset,steps,dest):
 with dest.with_suffix('.log').open('x') as log:
  subprocess.run([str(binary),str(source),arm.split('_')[1],str(offset),str(steps),str(dest)],stdout=log,stderr=subprocess.STDOUT,check=True)
def freeze(out,prior,pre):
 audit=load(prior/'audit.json');assert audit['audit_passed'] and load(prior/'status.json')['state']=='complete'
 if not pre:assert load(prior/'replay_verification.json')['passed']
 out.mkdir(parents=True,exist_ok=False);inputs=out/'inputs';inputs.mkdir()
 for file in ['audit.json','manifest.json','summary.json','status.json','instrument_provenance.json','preregistration.md']:
  shutil.copy2(prior/file,inputs/file)
 if not pre:
  shutil.copy2(prior/'replay_verification.json',inputs/'replay_verification.json')
  engineering=ROOT/'output/selfmod_v16_financing_preflight_20260927'
  checks=load(engineering/'engineering_verification.json');assert checks['passed'] and checks['branches']==16
  assert load(engineering/'audit.json')['audit_passed']
  shutil.copy2(engineering/'engineering_verification.json',inputs/'engineering_verification.json')
  shutil.copy2(engineering/'instrument_provenance.json',inputs/'engineering_instrument_provenance.json')
  shutil.copy2(engineering/'audit.json',inputs/'engineering_audit.json')
 for seed in ([7,42] if pre else SEEDS):
  for arm in ARMS:
   dest=inputs/f'seed_{seed}/{arm}';dest.mkdir(parents=True)
   for file in ['birth.json','final.json','score.json','reproduction.json']:
    src=prior/f'seed_{seed}/{arm}/{file}';assert sha(src)==audit['evidence_sha256'][f'seed_{seed}/{arm}/{file}'];shutil.copy2(src,dest/file)
 write(out/'input_provenance.json',dict(prior_directory=str(prior),inputs_sha256={str(p.relative_to(out)):sha(p) for p in sorted(inputs.rglob('*')) if p.is_file()}))
 sources=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'provenance.json']+list((ROOT/'docs').glob('*preregistration.md'))
 for folder in ['crates','bins','configs','rules','fixtures','scripts','.cargo','vendor']:
  sources.extend(p for p in (ROOT/folder).rglob('*') if p.is_file() and not set(p.relative_to(ROOT).parts)&{'target','__pycache__'})
 with tarfile.open(out/'source.tar.gz','w:gz') as tar:
  for p in sorted(sources):tar.add(p,arcname=str(p.relative_to(ROOT)),recursive=False)
 shutil.copy2(ROOT/'target/release/selfmod-finance',out/'selfmod-finance')
 shutil.copy2(ROOT/'docs/selfmod_v16_financing_preregistration.md',out/'preregistration.md')
 write(out/'instrument_provenance.json',dict(frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),binary_sha256=sha(out/'selfmod-finance'),source_archive_sha256=sha(out/'source.tar.gz'),input_provenance_sha256=sha(out/'input_provenance.json'),sources_sha256={str(p.relative_to(ROOT)):sha(p) for p in sorted(sources)},rustc=subprocess.check_output(['rustc','--version'],text=True).strip()))
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);p.add_argument('--preflight',action='store_true');args=p.parse_args();pre=args.preflight
 subprocess.run(['cargo','build','--offline','--locked','--release','-p','hg-selfmod','--bin','selfmod-finance'],cwd=ROOT,check=True)
 out=args.output.resolve();prior=ROOT/'output'/('selfmod_v15_replication_preflight_20260927' if pre else 'selfmod_v15_replication_20260927');freeze(out,prior,pre)
 seeds=[7,42] if pre else SEEDS;offset=64 if pre else 2000;steps=128 if pre else 8000
 write(out/'manifest.json',dict(protocol='selfmod-v16-financing-v1',engineering_preflight=pre,seeds=seeds,arms=ARMS,offset=offset,extension_steps=steps,requested_trajectories=len(seeds)*8,max_parallel_processes=4,window_steps=1000,segment_steps=2000,prior_primary_changed=False,independent_replication=False))
 write(out/'status.json',dict(state='running',trajectories_completed=0));completed=[]
 def task(seed,arm):
  parent=out/f'seed_{seed}';parent.mkdir(exist_ok=True);dest=parent/arm;src=out/f'inputs/seed_{seed}/{arm}/final.json'
  invoke(out/'selfmod-finance',src,arm,offset,steps,dest)
  if pre:
   invoke(out/'selfmod-finance',src,arm,offset,64,parent/(arm+'_split_a'))
   invoke(out/'selfmod-finance',parent/(arm+'_split_a/final.json'),arm,offset+64,64,parent/(arm+'_split_b'))
   assert (dest/'rows.jsonl').read_bytes()==(parent/(arm+'_split_a/rows.jsonl')).read_bytes()+(parent/(arm+'_split_b/rows.jsonl')).read_bytes()
   assert sha(dest/'final.json')==sha(parent/(arm+'_split_b/final.json'))
   assert sha(dest/'final_allocator.json')==sha(parent/(arm+'_split_b/final_allocator.json'))
  return dict(seed=seed,arm=arm)
 try:
  with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
   futures=[pool.submit(task,s,a) for s in seeds for a in ARMS]
   for f in concurrent.futures.as_completed(futures):
    completed.append(f.result());write(out/'status.json',dict(state='running',trajectories_completed=len(completed),new_advancing_steps=len(completed)*steps));print(f"{completed[-1]}: {len(completed)}/{len(futures)}",flush=True)
  if pre:
   write(out/'engineering_verification.json',dict(passed=True,branches=len(completed),exact_split_rows_and_final=True,engineering_advancing_steps=len(completed)*256,excluded_from_science=True,evidence_sha256={str(p.relative_to(out)):sha(p) for p in sorted(out.glob('seed_*/*/*')) if p.is_file()}))
  write(out/'status.json',dict(state='complete',trajectories_completed=len(completed),new_advancing_steps=len(completed)*steps,excluded_split_steps=len(completed)*128 if pre else 0))
 except Exception as e:
  write(out/'status.json',dict(state='failed',error=repr(e),trajectories_completed=len(completed)));raise
 print('Frozen evidence: '+str(out),flush=True)
if __name__=='__main__':main()
