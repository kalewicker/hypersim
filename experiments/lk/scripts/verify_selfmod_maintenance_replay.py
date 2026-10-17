#!/usr/bin/env python3
"""Exactly replay both full V22 extensions for fixed first seed457."""
import argparse
import concurrent.futures
import json
import pathlib
from audit_selfmod_birth import load, sha, require
from run_selfmod_second_edit_semantic import invoke


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('output', type=pathlib.Path)
    out = p.parse_args().output.resolve()
    a = load(out / 'audit.json')
    require(a['audit_passed'] and not a['engineering_preflight'], 'Audited science')
    seed = load(out / 'manifest.json')['seeds'][0]
    dest = out / 'verification'
    dest.mkdir(exist_ok=False)
    script = pathlib.Path(__file__)
    (dest / script.name).write_bytes(script.read_bytes())

    def branch(arm):
        invoke(out / 'selfmod-maintenance', out / f'inputs/seed_{seed}/{arm}/final.json',
               10000, 40000, dest / arm)
        receipts = {}
        names = ['initial.json', 'initial_allocator.json', 'rows.jsonl', 'final.json', 'final_allocator.json']
        names += [f'checkpoint_{s}{suffix}.json' for s in range(11000, 50001, 1000) for suffix in ['', '_allocator']]
        for name in names:
            require(sha(dest / arm / name) == sha(out / f'seed_{seed}/{arm}/{name}'), 'Exact continuation '+arm+'/'+name)
            receipts[arm+'/'+name] = sha(dest / arm / name)
        print(arm+': exact 40000-step extension', flush=True)
        return receipts

    receipts = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        for result in pool.map(branch, ['faithful', 'edited']):
            receipts.update(result)
    with (out / 'replay_verification.json').open('x') as f:
        json.dump(dict(passed=True, seed=seed, offset=10000, extension_steps=40000,
            verification_advancing_steps=80000, excluded_from_science=True,
            fresh_initialization_replayed=False, binary_sha256=sha(out / 'selfmod-maintenance'),
            script_sha256=sha(script), exact_receipts=receipts), f, indent=2)
        f.write('\n')
    print('170 exact receipts; 80000 verification advances excluded from science')


if __name__ == '__main__':
    main()
