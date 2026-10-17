#!/usr/bin/env python3
"""Freeze v13 instrument and v12 inputs before the ON/OFF comparison."""
import argparse, datetime, hashlib, json, pathlib, shutil, subprocess, tarfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def load(p): return json.loads(p.read_text())
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=pathlib.Path); parser.add_argument('--preflight',action='store_true'); args=parser.parse_args()
    prior=ROOT/'output'/('selfmod_v12_birth_preflight_v2_20260926' if args.preflight else 'selfmod_v12_birth_v2_20260926')
    assert load(prior/'status.json')['state']=='complete' and load(prior/'audit.json')['audit_passed']
    if not args.preflight: assert load(prior/'replay_verification.json')['passed']
    subprocess.run(['cargo','build','--offline','--locked','--release','-p','hg-selfmod','--bin','selfmod-turnover'],cwd=ROOT,check=True)
    out=args.output.resolve(); out.mkdir(parents=True,exist_ok=False)
    inputs=out/'inputs'; inputs.mkdir()
    for file in ['manifest.json','status.json','summary.json','audit.json','instrument_provenance.json','preregistration.md','v11_selected_mutation.json']:
        shutil.copy2(prior/file,inputs/file)
    if not args.preflight: shutil.copy2(prior/'replay_verification.json',inputs/'replay_verification.json')
    for seed in ([7,42] if args.preflight else [11,17,23,31,47,61,83,101]):
        dest=inputs/f'seed_{seed}'; dest.mkdir(); shutil.copy2(prior/f'seed_{seed}/fork.json',dest/'fork.json')
        for arm in ['faithful','prune']:
            (dest/arm).mkdir()
            for file in ['birth.json','rows.jsonl','score.json','final.json']:
                source=prior/f'seed_{seed}'/arm/file
                assert sha(source)==load(prior/'audit.json')['evidence_sha256'][str(source.relative_to(prior))]
                shutil.copy2(source,dest/arm/file)
    # Contract and immutable input inventory exist before any advancing step.
    (out/'input_provenance.json').write_text(json.dumps(dict(prior_directory=str(prior),
        prior_protocol='selfmod-v12-birth-v1',inputs_sha256={str(p.relative_to(out)):sha(p) for p in sorted(inputs.rglob('*')) if p.is_file()}),indent=2)+'\n')
    sources=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'provenance.json',ROOT/'docs/selfmod_v13_turnover_preregistration.md', ROOT/'docs/selfmod_v14_bounded_preregistration.md', ROOT/'docs/selfmod_v15_replication_preregistration.md',ROOT/'docs/selfmod_v12_birth_preregistration.md',ROOT/'docs/selfmod_v11_spectrum_preregistration.md']
    for folder in ['crates','bins','configs','rules','fixtures','scripts','.cargo','vendor']:
        sources.extend(p for p in (ROOT/folder).rglob('*') if p.is_file() and not set(p.relative_to(ROOT).parts)&{'target','__pycache__'})
    with tarfile.open(out/'source.tar.gz','w:gz') as tar:
        for p in sorted(sources): tar.add(p,arcname=str(p.relative_to(ROOT)),recursive=False)
    shutil.copy2(ROOT/'target/release/selfmod-turnover',out/'selfmod-turnover')
    (out/'instrument_provenance.json').write_text(json.dumps(dict(frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        binary_sha256=sha(out/'selfmod-turnover'),source_archive_sha256=sha(out/'source.tar.gz'),
        sources_sha256={str(p.relative_to(ROOT)):sha(p) for p in sorted(sources)},input_provenance_sha256=sha(out/'input_provenance.json'),
        engineering_preflight=args.preflight,rustc=subprocess.check_output(['rustc','--version'],text=True).strip()),indent=2)+'\n')
    command=[str(out/'selfmod-turnover')]+(['--preflight'] if args.preflight else [])+[str(out)]
    with (out/'run.log').open('x') as log:
        p=subprocess.Popen(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
        for line in p.stdout: print(line,end='',flush=True); log.write(line); log.flush()
        code=p.wait()
    if code: raise SystemExit(code)
    print(f'Frozen evidence: {out}',flush=True)
if __name__=='__main__': main()
