#!/usr/bin/env python3
"""Freeze and run the v12 natural-birth assay in a fresh directory."""
import argparse
import datetime
import hashlib
import json
import pathlib
import shutil
import subprocess
import tarfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=pathlib.Path)
    parser.add_argument('--preflight', action='store_true')
    args = parser.parse_args()
    subprocess.run(['cargo', 'build', '--offline', '--locked', '--release',
                    '-p', 'hg-selfmod', '--bin', 'selfmod-birth'], cwd=ROOT, check=True)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    sources = [ROOT/'Cargo.toml', ROOT/'Cargo.lock', ROOT/'provenance.json',
               ROOT/'docs/selfmod_v12_birth_preregistration.md', ROOT/'docs/selfmod_v11_findings.md',
               ROOT/'docs/selfmod_v11_spectrum_preregistration.md',
               ROOT/'docs/selfmod_v13_turnover_preregistration.md', ROOT/'docs/selfmod_v14_bounded_preregistration.md', ROOT/'docs/selfmod_v15_replication_preregistration.md']
    for folder in ('crates', 'bins', 'configs', 'rules', 'fixtures', 'scripts', '.cargo', 'vendor'):
        sources.extend(p for p in (ROOT/folder).rglob('*') if p.is_file() and
                       not set(p.relative_to(ROOT).parts) & {'target', '__pycache__'})
    hashes = {str(p.relative_to(ROOT)): digest(p) for p in sorted(sources)}
    archive = out/'source.tar.gz'
    with tarfile.open(archive, 'w:gz') as tar:
        for source in sorted(sources):
            tar.add(source, arcname=str(source.relative_to(ROOT)), recursive=False)
    binary = out/'selfmod-birth'
    shutil.copy2(ROOT/'target/release/selfmod-birth', binary)
    evidence = ROOT/'output/selfmod_v11_spectrum_20260926/mutations/op11_member0_draw1.json'
    shutil.copy2(evidence, out/'v11_selected_mutation.json')
    provenance = dict(frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                      engineering_preflight=args.preflight, binary_sha256=digest(binary),
                      source_archive_sha256=digest(archive), sources_sha256=hashes,
                      selected_v11_evidence_sha256=digest(evidence),
                      rustc=subprocess.check_output(['rustc', '--version'], text=True).strip(),
                      build_command='cargo build --offline --locked --release -p hg-selfmod --bin selfmod-birth')
    (out/'instrument_provenance.json').write_text(json.dumps(provenance, indent=2)+'\n')
    command = [str(binary)] + (['--preflight'] if args.preflight else []) + [str(out)]
    with (out/'run.log').open('x') as log:
        process = subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        for line in process.stdout:
            print(line, end='', flush=True)
            log.write(line)
            log.flush()
        code = process.wait()
    if code:
        raise SystemExit(code)
    print(f'Frozen evidence: {out}', flush=True)


if __name__ == '__main__':
    main()
