#!/usr/bin/env python3
"""Packaging-only continuation; reuse immutable resource workers without retry."""
import argparse
import copy
import importlib.util
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import traceback

ROOT = Path(__file__).resolve().parents[1]
if ROOT.name == 'source_snapshots':
    ROOT = Path(json.loads((ROOT.parent / 'registration.json').read_text())['repository'])
FAILED = ROOT / 'output/ere_categorical_comparison_bounded_auditor_20261002'
FAILED_ROOT = '6b98587e9b151ec738bf32210ba0d60e7b657c27d87e50757c1e4c68a9b970d8'
NEW_SOURCES = [
    'scripts/verify_ere_categorical_comparison_bounded_v2.py',
    'scripts/test_verify_ere_categorical_comparison_bounded_v2.py',
    'docs/ere_categorical_comparison_bounded_auditor_continuation_v2.md',
]


def original_driver():
    path = FAILED / 'source_snapshots/scripts/measure_ere_categorical_comparison_bounded.py'
    spec = importlib.util.spec_from_file_location('frozen_bounded_measurement_driver', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    sys.path.insert(0, str(path.parent))
    return module


def accounting_only(saved, development):
    result = copy.deepcopy(saved)
    if development:
        controls = result.pop('detached_tamper_rejections')
        assert isinstance(controls, list) and len(controls) == 6
    else:
        assert 'detached_tamper_rejections' not in result, 'unexpected preflight shape'
    return result


def preflight_controls(b, out):
    cases = [
        ('summary.json', lambda x: x.update(support_categorical_direction=True)),
        ('manifest.json', lambda x: x.update(training_admitted=True)),
        ('manifest.json', lambda x: x.update(attempted_steps=x['attempted_steps'] + 1)),
        ('schedule.json', lambda x: x[0].update(physics_seed=x[0]['physics_seed'] + 1)),
        ('request.json', lambda x: x.update(full_training_authorized=True)),
        ('snapshot.json', lambda x: x.update(default_features=True)),
    ]
    results = []
    with tempfile.TemporaryDirectory(prefix='ere-bounded-preflight-controls-') as directory:
        clone = Path(directory) / 'control'
        shutil.copytree(out, clone)
        for index, (name, mutate) in enumerate(cases):
            original = (clone / name).read_bytes()
            value = b.a.load(clone / name)
            mutate(value)
            (clone / name).write_text(json.dumps(value))
            try:
                b.audit(clone, verify_seal=False)
            except (AssertionError, KeyError, ValueError, TypeError, IndexError) as exc:
                results.append(dict(index=index, path=name, rejected=True, reason=str(exc)))
            else:
                raise AssertionError(f'preflight tamper accepted: {index}')
            finally:
                (clone / name).write_bytes(original)
    return results


def verify(out):
    driver = original_driver()
    legacy, bounded = driver.helpers()
    a = legacy.a
    out = out.resolve()
    assert out.parent == (ROOT / 'output').resolve() and not out.exists(), 'fresh direct output child required'
    a.verify(FAILED, expected=FAILED_ROOT, count=16)
    failed_manifest = a.load(FAILED / 'manifest.json')
    assert failed_manifest['status'] == 'failed_engineering' and failed_manifest['failure'] == "'detached_tamper_rejections'"
    a.verify(driver.DEV, expected=driver.DEV_ROOT, count=692)
    a.verify(driver.PREFLIGHT, expected=driver.PREFLIGHT_ROOT, count=679)
    historical = a.load(driver.DEV / 'snapshot.json')['sources']
    assert all(a.digest(ROOT / n) == h for n, h in historical.items())
    old_sources = a.load(FAILED / 'registration.json')['sources']
    assert set(old_sources) == set(driver.SOURCES)
    sources = dict(old_sources, **{n: a.digest(ROOT / n) for n in NEW_SOURCES})
    assert all(a.digest(ROOT / n) == h for n, h in sources.items())
    bounded.verify_storage_only()
    out.mkdir()
    for name in sources:
        target = out / 'source_snapshots' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        origin = FAILED / 'source_snapshots' / name if name in old_sources else ROOT / name
        shutil.copy2(origin, target)
        assert a.digest(target) == sources[name]
    driver.save(out / 'registration.json', dict(repository=str(ROOT), purpose='packaging_only_engineering_continuation',
        frozen_before_continuation_checks=True, sources=sources,
        failed_receipt=dict(path=str(FAILED), root_seal=FAILED_ROOT, sealed_entries=16),
        correction='preflight accounting result has no development-only detached-control metadata',
        reuse_all_four_measurements_without_repeat=True, worker_timeout_seconds=900,
        native_evaluations_authorized=False, scientific_launch_authorized=False, training_authorized=False,
        launch_bridge_implemented=False))
    initial = {n: a.digest(out / n) for n in sorted(a.files(out))}
    driver.save(out / 'registration_sha256.json', initial)
    manifest = dict(status='running', purpose='packaging_only_engineering_continuation',
        registration_root=a.digest(out / 'registration_sha256.json'),
        native_evaluations_run=0, native_steps_run=0, grammar_compilations_run=0,
        training_updates=0, scientific_campaign_complete=False, training_admitted=False,
        launch_bridge_implemented=False, original_failure_preserved=True, repeated_resource_workers=0)
    driver.save(out / 'manifest.json', manifest)
    a.seal(out)
    try:
        measurements = {}
        for name in driver.WORKERS:
            assert (FAILED / (name + '.stderr.txt')).read_text() == ''
            x = a.load(FAILED / (name + '.stdout.txt'))
            assert x['worker'] == name and type(x['wall_seconds']) is float
            assert math.isfinite(x['wall_seconds']) and 0 < x['wall_seconds'] < 900
            assert type(x['peak_process_rss_bytes']) is int and x['peak_process_rss_bytes'] > 0
            measurements[name] = x
        eager = measurements['legacy-development']
        small = measurements['bounded-development']
        aresult = accounting_only(a.load(driver.DEV / 'audit.json'), True)
        bounded.exact(eager['result'], aresult)
        bounded.exact(small['result'], aresult)
        bounded.exact(measurements['bounded-preflight']['result'], accounting_only(a.load(driver.PREFLIGHT / 'audit.json'), False))
        assert small['peak_process_rss_bytes'] < eager['peak_process_rss_bytes']
        synthetic = measurements['bounded-stress-12288']
        assert synthetic['peak_process_rss_bytes'] < 512*1024*1024
        assert synthetic['result']['bindings'] == 12288 and synthetic['result']['reuse_repetitions'] == 64
        assert synthetic['result']['cached_receipt_objects'] == synthetic['result']['native_steps_run'] == 0
        driver.save(out / 'preserved_measurements.json', measurements)
        regressions = []
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', PYTHONPATH=str(driver.DEV / 'source_snapshots/scripts'))
        test_paths = [
            (out / 'source_snapshots/scripts/test_audit_ere_categorical_comparison_bounded.py', 7),
            (driver.DEV / 'source_snapshots/scripts/test_audit_ere_categorical_comparison.py', 7),
            (driver.DEV / 'source_snapshots/scripts/test_check_ere_categorical_comparison_plan.py', 8),
            (out / 'source_snapshots/scripts/test_verify_ere_categorical_comparison_bounded_v2.py', 3),
        ]
        for path, count in test_paths:
            done = subprocess.run([sys.executable, '-B', str(path), '-v'], cwd=ROOT, env=env,
                                  capture_output=True, text=True, timeout=900)
            (out / (path.stem + '.stdout.txt')).write_text(done.stdout)
            (out / (path.stem + '.stderr.txt')).write_text(done.stderr)
            regressions.append(dict(test=path.name, returncode=done.returncode, expected_groups=count))
            assert done.returncode == 0 and f'Ran {count} tests' in done.stderr
        controls = dict(development=driver.detached_controls(bounded), preflight=preflight_controls(bounded, driver.PREFLIGHT))
        assert len(controls['development']) == 8 and len(controls['preflight']) == 6
        driver.save(out / 'controls.json', controls)
        assert all(a.digest(out / n) == h for n, h in initial.items())
        assert all(a.digest(ROOT / n) == h for n, h in historical.items())
        assert all(a.digest(ROOT / n) == h for n, h in sources.items())
        a.verify(driver.DEV, expected=driver.DEV_ROOT, count=692)
        a.verify(driver.PREFLIGHT, expected=driver.PREFLIGHT_ROOT, count=679)
        a.verify(FAILED, expected=FAILED_ROOT, count=16)
        driver.save(out / 'verification.json', dict(engineering_pass=True,
            audit_results_exactly_equivalent=True, storage_only_ast=bounded.verify_storage_only(),
            preserved_historical_sources=len(historical), preserved_failed_receipt_root=FAILED_ROOT,
            preserved_input_roots=[driver.DEV_ROOT, driver.PREFLIGHT_ROOT],
            regressions=regressions, passed_regression_groups=25, detached_tamper_rejections=14,
            rss_reduction_fraction=1-small['peak_process_rss_bytes']/eager['peak_process_rss_bytes'],
            scientific_support_claim=False, launch_bridge_implemented=False))
        manifest['status'] = 'complete'
    except Exception as exc:
        manifest.update(status='failed_engineering', failure=str(exc))
        (out / 'failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        driver.save(out / 'manifest.json', manifest)
        a.seal(out)
    count = a.verify(out)
    print(json.dumps(dict(archive=str(out), root_seal=a.digest(out / 'output_sha256.json'), sealed_entries=count, **manifest), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    verify(parser.parse_args().output)
