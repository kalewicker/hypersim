#!/usr/bin/env python3
"""Freeze and run the preregistered v14 bounded birth panel."""
import argparse,datetime,hashlib,json,pathlib,shutil,subprocess,tarfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def load(p):return json.loads(p.read_text())
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);p.add_argument('--preflight',action='store_true');args=p.parse_args()
 prior=ROOT/'output'/('selfmod_v13_turnover_preflight_v2_20260927' if args.preflight else 'selfmod_v13_turnover_v2_20260927')
 a=load(prior/'audit.json');assert a['audit_passed'] and load(prior/'status.json')['state']=='complete'
 if not args.preflight:assert load(prior/'replay_verification.json')['passed']
 subprocess.run(['cargo','build','--offline','--locked','--release','-p','hg-selfmod','--bin','selfmod-bounded'],cwd=ROOT,check=True)
 out=args.output.resolve();out.mkdir(parents=True,exist_ok=False);inputs=out/'inputs';inputs.mkdir()
 for file in ['manifest.json','summary.json','status.json','audit.json','instrument_provenance.json','preregistration.md']:
  shutil.copy2(prior/file,inputs/file)
 if not args.preflight:shutil.copy2(prior/'replay_verification.json',inputs/'replay_verification.json')
 shutil.copy2(prior/'inputs/v11_selected_mutation.json',inputs/'v11_selected_mutation.json')
 for seed in ([7,42] if args.preflight else [11,17,23,31,47,61,83,101]):
  target=inputs/f'seed_{seed}';target.mkdir();source=prior/f'inputs/seed_{seed}/fork.json'
  assert sha(source)==a['evidence_sha256'][str(source.relative_to(prior))];shutil.copy2(source,target/'fork.json')
  for genotype in ['faithful','prune']:
   for mode in ['on','off']:
    arm=f'{genotype}_{mode}';(target/arm).mkdir()
    for file in ['birth.json','rows.jsonl','score.json','final.json']:
     source=prior/f'seed_{seed}/{arm}/{file}';assert sha(source)==a['evidence_sha256'][str(source.relative_to(prior))];shutil.copy2(source,target/arm/file)
 (out/'input_provenance.json').write_text(json.dumps(dict(prior_directory=str(prior),inputs_sha256={str(p.relative_to(out)):sha(p) for p in sorted(inputs.rglob('*')) if p.is_file()}),indent=2)+'\n')
 sources=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'provenance.json']+list((ROOT/'docs').glob('*preregistration.md'))
 for folder in ['crates','bins','configs','rules','fixtures','scripts','.cargo','vendor']:
  sources.extend(p for p in (ROOT/folder).rglob('*') if p.is_file() and not set(p.relative_to(ROOT).parts)&{'target','__pycache__'})
 with tarfile.open(out/'source.tar.gz','w:gz') as tar:
  for file in sorted(sources):tar.add(file,arcname=str(file.relative_to(ROOT)),recursive=False)
 shutil.copy2(ROOT/'target/release/selfmod-bounded',out/'selfmod-bounded')
 (out/'instrument_provenance.json').write_text(json.dumps(dict(frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
  binary_sha256=sha(out/'selfmod-bounded'),source_archive_sha256=sha(out/'source.tar.gz'),input_provenance_sha256=sha(out/'input_provenance.json'),
  sources_sha256={str(p.relative_to(ROOT)):sha(p) for p in sorted(sources)},engineering_preflight=args.preflight,
  rustc=subprocess.check_output(['rustc','--version'],text=True).strip()),indent=2)+'\n')
 command=[str(out/'selfmod-bounded')]+(['--preflight'] if args.preflight else [])+[str(out)]
 with (out/'run.log').open('x') as log:
  process=subprocess.Popen(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
  for line in process.stdout:print(line,end='',flush=True);log.write(line);log.flush()
  code=process.wait()
 if code:raise SystemExit(code)
 print(f'Frozen evidence: {out}',flush=True)
if __name__=='__main__':main()
