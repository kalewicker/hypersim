#!/usr/bin/env python3
"""Freeze fresh-seed v15, or regenerate the v14 engineering preflight exactly."""
import argparse,datetime,hashlib,json,pathlib,shutil,subprocess,tarfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
SEEDS=[131,137,149,157,163,173,181,191]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def load(p):return json.loads(p.read_text())
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);p.add_argument('--preflight',action='store_true');args=p.parse_args()
 prior=ROOT/'output'/('selfmod_v14_bounded_preflight_20260927' if args.preflight else 'selfmod_v14_bounded_20260927')
 audit=load(prior/'audit.json');assert audit['audit_passed'] and load(prior/'status.json')['state']=='complete'
 preflight=ROOT/'output/selfmod_v15_replication_preflight_20260927'
 if not args.preflight:
  assert load(prior/'replay_verification.json')['passed']
  assert load(preflight/'status.json')['state']=='complete' and load(preflight/'audit.json')['audit_passed'] and load(preflight/'summary.json')['historical_preflight_exact']
 subprocess.run(['cargo','build','--offline','--locked','--release','-p','hg-selfmod','--bin','selfmod-replicate'],cwd=ROOT,check=True)
 out=args.output.resolve();out.mkdir(parents=True,exist_ok=False);reference=out/'reference';reference.mkdir()
 for file in ['manifest.json','summary.json','status.json','audit.json','instrument_provenance.json','preregistration.md']:
  shutil.copy2(prior/file,reference/file)
 if not args.preflight:
  shutil.copy2(prior/'replay_verification.json',reference/'replay_verification.json')
  engineering=reference/'v15_preflight';engineering.mkdir()
  for file in ['audit.json','status.json','instrument_provenance.json','manifest.json','summary.json']:shutil.copy2(preflight/file,engineering/file)
 evidence=prior/'inputs/v11_selected_mutation.json';assert sha(evidence)==audit['evidence_sha256']['inputs/v11_selected_mutation.json']
 shutil.copy2(evidence,out/'v11_selected_mutation.json')
 if args.preflight:
  for seed in [7,42]:
   dest=reference/f'seed_{seed}';dest.mkdir()
   for genotype in ['faithful','prune']:
    for policy in ['on','off','vacancy8','replace8']:
     arm=genotype+'_'+policy;(dest/arm).mkdir()
     for file in ['birth.json','rows.jsonl','score.json','reproduction.json','final.json']:
      source=prior/f'seed_{seed}/{arm}/{file}';assert sha(source)==audit['evidence_sha256'][str(source.relative_to(prior))];shutil.copy2(source,dest/arm/file)
 else:
  history={}
  for manifest in sorted((ROOT/'output').glob('selfmod*/manifest.json')):
   data=load(manifest);used=set(data.get('seeds',[]))|set(data.get('world_seeds',[]));overlap=used&set(SEEDS)
   assert not overlap,(manifest,overlap)
   history[str(manifest.relative_to(ROOT))]=dict(sha256=sha(manifest),seeds=sorted(used))
  (out/'fresh_seed_inventory.json').write_text(json.dumps(dict(selected_seeds=SEEDS,prior_manifests=history,no_overlap=True),indent=2)+'\n')
 (out/'reference_provenance.json').write_text(json.dumps(dict(prior_directory=str(prior),reference_sha256={str(p.relative_to(out)):sha(p) for p in sorted(reference.rglob('*')) if p.is_file()},
  selected_v11_evidence_sha256=sha(out/'v11_selected_mutation.json')),indent=2)+'\n')
 sources=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'provenance.json']+list((ROOT/'docs').glob('*preregistration.md'))
 for folder in ['crates','bins','configs','rules','fixtures','scripts','.cargo','vendor']:
  sources.extend(p for p in (ROOT/folder).rglob('*') if p.is_file() and not set(p.relative_to(ROOT).parts)&{'target','__pycache__'})
 with tarfile.open(out/'source.tar.gz','w:gz') as tar:
  for file in sorted(sources):tar.add(file,arcname=str(file.relative_to(ROOT)),recursive=False)
 shutil.copy2(ROOT/'target/release/selfmod-replicate',out/'selfmod-replicate')
 (out/'instrument_provenance.json').write_text(json.dumps(dict(frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),binary_sha256=sha(out/'selfmod-replicate'),source_archive_sha256=sha(out/'source.tar.gz'),
  reference_provenance_sha256=sha(out/'reference_provenance.json'),fresh_seed_inventory_sha256=None if args.preflight else sha(out/'fresh_seed_inventory.json'),
  sources_sha256={str(p.relative_to(ROOT)):sha(p) for p in sorted(sources)},engineering_preflight=args.preflight,rustc=subprocess.check_output(['rustc','--version'],text=True).strip()),indent=2)+'\n')
 command=[str(out/'selfmod-replicate')]+(['--preflight'] if args.preflight else [])+[str(out)]
 with (out/'run.log').open('x') as log:
  process=subprocess.Popen(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
  for line in process.stdout:print(line,end='',flush=True);log.write(line);log.flush()
  code=process.wait()
 if code:raise SystemExit(code)
 print(f'Frozen evidence: {out}',flush=True)
if __name__=='__main__':main()
