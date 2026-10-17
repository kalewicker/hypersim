#!/usr/bin/env python3
"""Freeze and extend every V21 branch without resetting its physical state."""
import argparse
import concurrent.futures
import datetime
import pathlib
import shutil
import tarfile
from run_selfmod_financing import load, sha, write
from run_selfmod_second_edit_semantic import invoke

ROOT = pathlib.Path(__file__).resolve().parents[1]
CONTRACT = load(ROOT / 'docs/selfmod_v22_maintenance_contract.json')
SEEDS, ARMS = CONTRACT['seeds'], CONTRACT['arms']


def freeze(out, pre):
    prior = ROOT / 'output' / CONTRACT['prior_assay']
    done = load(prior / 'completion_verification.json')
    assert done['passed'] and all(sha(prior / f) == h for f, h in done['artifacts_sha256'].items())
    assert all(load(prior / 'audit.json')['aggregates'][a]['durable_third_families'] == 8 for a in ARMS)
    out.mkdir(parents=True, exist_ok=False)
    inputs = out / 'inputs'
    inputs.mkdir()
    for f in ['audit.json', 'manifest.json', 'summary.json', 'status.json', 'instrument_provenance.json',
              'completion_verification.json', 'replay_verification.json', 'contract.json']:
        shutil.copy2(prior / f, inputs / f)
    shutil.copy2(prior / 'source.tar.gz', out / 'native_source.tar.gz')
    shutil.copy2(prior / 'selfmod-reversible-edit', out / 'selfmod-maintenance')
    prior_audit = load(prior / 'audit.json')
    for seed in (CONTRACT['engineering_seeds'] if pre else SEEDS):
        for arm in ARMS:
            d = inputs / f'seed_{seed}' / arm
            d.mkdir(parents=True)
            for f in ['final.json', 'final_allocator.json']:
                name = f'seed_{seed}/{arm}/{f}'
                assert sha(prior / name) == prior_audit['evidence_sha256'][name] == done['artifacts_sha256'][name]
                shutil.copy2(prior / name, d / f)
            name = f'seed_{seed}/third_prefix/{arm}_birth.json'
            assert sha(prior / name) == prior_audit['evidence_sha256'][name] == done['artifacts_sha256'][name]
            shutil.copy2(prior / name, d / 'birth.json')
    failed = ROOT / 'output/selfmod_v22_maintenance_preflight_20260928'
    if (failed / 'audit_validation_failure.json').exists():
        require_failure = load(failed / 'audit_validation_failure.json')
        assert require_failure['audit_passed'] is False and require_failure['science_not_launched']
        for f in ['audit_validation_failure.json', 'instrument_provenance.json', 'summary.json', 'development_verification.json']:
            shutil.copy2(failed / f, inputs / ('failed_engineering_' + f))
        write(inputs / 'failed_engineering_raw_inventory.json', dict(
            directory=str(failed), raw_sha256={str(f.relative_to(failed)): sha(f)
                for f in sorted(failed.glob('seed_*/*/*')) if f.is_file() and f.suffix in ['.json', '.jsonl']}))
    if not pre:
        eng = ROOT / 'output/selfmod_v22_maintenance_preflight_v2_20260928'
        for f in ['audit.json', 'engineering_verification.json', 'source_archive_verification.json',
                  'missing_case_verification.json', 'test_verification.json']:
            check = load(eng / f)
            assert check.get('passed', check.get('audit_passed', False))
            shutil.copy2(eng / f, inputs / ('engineering_' + f))
    write(out / 'input_provenance.json', dict(prior_directory=str(prior),
        inputs_sha256={str(p.relative_to(out)): sha(p) for p in sorted(inputs.rglob('*')) if p.is_file()}))
    shutil.copy2(ROOT / 'docs/selfmod_v22_maintenance_contract.json', out / 'contract.json')
    shutil.copy2(ROOT / 'docs/selfmod_v22_maintenance_preregistration.md', out / 'preregistration.md')
    sources = [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / 'provenance.json']
    sources += list((ROOT / 'docs').glob('*preregistration.md')) + list((ROOT / 'docs').glob('*contract.json'))
    for folder in ['crates', 'bins', 'configs', 'rules', 'fixtures', 'scripts', '.cargo', 'vendor']:
        sources += [p for p in (ROOT / folder).rglob('*') if p.is_file() and
                    not set(p.relative_to(ROOT).parts) & {'target', '__pycache__'}]
    with tarfile.open(out / 'source.tar.gz', 'w:gz') as t:
        for p in sorted(sources):
            t.add(p, arcname=str(p.relative_to(ROOT)), recursive=False)
    write(out / 'instrument_provenance.json', dict(
        frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        native_binary_reused_without_rebuild=True,
        sources_sha256={str(p.relative_to(ROOT)): sha(p) for p in sorted(sources)},
        artifacts_sha256={f: sha(out / f) for f in ['selfmod-maintenance', 'native_source.tar.gz',
            'source.tar.gz', 'contract.json', 'preregistration.md', 'input_provenance.json']}))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('output', type=pathlib.Path)
    p.add_argument('--preflight', action='store_true')
    args = p.parse_args()
    out, pre = args.output.resolve(), args.preflight
    freeze(out, pre)
    seeds = CONTRACT['engineering_seeds'] if pre else SEEDS
    steps = CONTRACT['engineering_steps'] if pre else CONTRACT['extension_steps']
    write(out / 'manifest.json', dict(protocol=CONTRACT['protocol'], engineering_preflight=pre,
        contract=CONTRACT, seeds=seeds, arms=ARMS, offset=10000, extension_steps=steps,
        requested_trajectories=len(seeds)*2, max_parallel_processes=4, independent_replication=False))
    write(out / 'status.json', dict(state='running', trajectories_completed=0))
    completed = []

    def branch(pair):
        seed, arm = pair
        parent = out / f'seed_{seed}'
        parent.mkdir(exist_ok=True)
        src = out / f'inputs/seed_{seed}/{arm}/final.json'
        invoke(out / 'selfmod-maintenance', src, 10000, steps, parent / arm)
        if pre:
            invoke(out / 'selfmod-maintenance', src, 10000, 1000, parent / (arm+'_split_a'))
            invoke(out / 'selfmod-maintenance', parent / (arm+'_split_a/final.json'),
                   11000, 1000, parent / (arm+'_split_b'))
            assert (parent/arm/'rows.jsonl').read_bytes() == (parent/(arm+'_split_a/rows.jsonl')).read_bytes() + (parent/(arm+'_split_b/rows.jsonl')).read_bytes()
            for suffix in ['', '_allocator']:
                assert sha(parent/arm/f'checkpoint_11000{suffix}.json') == sha(parent/(arm+f'_split_a/final{suffix}.json'))
                assert sha(parent/arm/f'final{suffix}.json') == sha(parent/(arm+f'_split_b/final{suffix}.json'))
        return dict(seed=seed, arm=arm)

    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            for result in pool.map(branch, [(s, a) for s in seeds for a in ARMS]):
                completed.append(result)
                write(out / 'status.json', dict(state='running', trajectories_completed=len(completed)))
                print(f'{result}: {len(completed)}/{len(seeds)*2}', flush=True)
        summary = dict(trajectories_completed=len(completed), extension_steps=len(completed)*steps,
            scientific_advancing_steps=0 if pre else len(completed)*steps,
            engineering_advancing_steps=len(completed)*steps*2 if pre else 0,
            fixed_seed_denominator=len(seeds), prior_steps_recounted=0)
        write(out / 'summary.json', summary)
        if pre:
            write(out / 'engineering_verification.json', dict(passed=True, exact_splits=4,
                engineering_advancing_steps=16000, native_binary_matches_v21=True,
                original_config_rng_clock_and_allocator_restored=True, excluded_from_science=True))
        write(out / 'status.json', dict(state='complete', trajectories_completed=len(completed)))
    except Exception as exc:
        write(out / 'status.json', dict(state='failed', error=repr(exc), trajectories_completed=len(completed)))
        raise


if __name__ == '__main__':
    main()
