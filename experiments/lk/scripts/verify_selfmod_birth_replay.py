#!/usr/bin/env python3
"""Two preregistered cross-process replays; excluded from scientific counts."""
import argparse
import hashlib
import json
import pathlib
import subprocess


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=pathlib.Path)
    args = parser.parse_args()
    out = args.output.resolve()
    root = out/'verification'
    root.mkdir(exist_ok=False)
    source = pathlib.Path(__file__)
    (root/source.name).write_bytes(source.read_bytes())
    manifest = json.loads((out/'manifest.json').read_text())
    if manifest['engineering_preflight'] or manifest['horizon'] != 2000:
        raise ValueError('scientific 2000-step panel required')
    comparison = {}
    for arm in ('faithful', 'prune'):
        target = root/arm
        command = [str(out/'selfmod-birth'), '--replay', str(out/'seed_11/fork.json'), arm, str(target)]
        with (root/f'{arm}.log').open('x') as log:
            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=True)
        expected = out/'seed_11'/arm
        comparison[arm] = {}
        for name in ('birth.json', 'rows.jsonl', 'score.json', 'final.json'):
            actual_hash, expected_hash = digest(target/name), digest(expected/name)
            if actual_hash != expected_hash:
                raise ValueError(f'replay divergence: {arm}/{name}')
            comparison[arm][name] = actual_hash
        print(f'{arm}: all 2000 steps, initial/final states and verdict exactly reproduced', flush=True)
    report = dict(passed=True, seed=11, verification_advancing_steps=4000, excluded_from_scientific_panel=True,
                  binary_sha256=digest(out/'selfmod-birth'), script_sha256=digest(source), exact_receipts=comparison)
    with (out/'replay_verification.json').open('x') as file:
        json.dump(report, file, indent=2)
        file.write('\n')


if __name__ == '__main__':
    main()
