#!/usr/bin/env python3
"""Stage-bound engineering verification; no worker retry or native launch."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from verify_ere_categorical_comparison_bounded_v2 import (
    ROOT, FAILED, FAILED_ROOT, original_driver, accounting_only,
    preflight_controls, NEW_SOURCES as V2_SOURCES,
)
import traceback

FAILED_V2 = ROOT / 'output/ere_categorical_comparison_bounded_auditor_v2_20261002'
FAILED_V2_ROOT = 'f6c2a73caf4fb2013cfd43225024cb983b1836b68e64e37aecae60862749bd39'
NEW_SOURCES = V2_SOURCES + [
    'scripts/verify_ere_categorical_comparison_bounded_v3.py',
    'docs/ere_categorical_comparison_bounded_auditor_verification_v3.md',
]


def historical_plan_regression(driver, env, out):
    """Execute unchanged planning tests at their exact pre-harness source stage."""
    a = driver.helpers()[0].a
    config = a.load(driver.DEV / 'config.json')
    adapter = ROOT / config['lineage']['adapter']['path']
    prereg = ROOT / 'output/ere_categorical_comparison_preregistration_20261001'
    adapter_sources = a.load(adapter / 'manifest.json')['sources']
    planning_sources = a.load(prereg / 'manifest.json')['sources']
    with tempfile.TemporaryDirectory(prefix='ere-historical-plan-tests-') as directory:
        fixture = Path(directory)
        for archive, sources in ((adapter, adapter_sources), (prereg, planning_sources)):
            for name, digest in sources.items():
                target = fixture / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(archive / 'source_snapshots' / name, target)
                assert a.digest(target) == digest
        direction = config['lineage']['direction']['path']
        minimal = [(direction, name) for name in (
            'snapshot.json', 'catalogue.json',
            'source_snapshots/crates/hg-train/Cargo.toml',
            'source_snapshots/crates/hg-train/src/ere.rs',
        )] + [(config['lineage']['adapter']['path'], 'manifest.json')]
        for archive, name in minimal:
            target = fixture / archive / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / archive / name, target)
        test = fixture / 'scripts/test_check_ere_categorical_comparison_plan.py'
        done = subprocess.run([driver.sys.executable, '-B', str(test), '-v'],
                              cwd=fixture, env=dict(env, PYTHONPATH=str(fixture / 'scripts')),
                              capture_output=True, text=True, timeout=900)
        driver.save(out / 'historical_plan_fixture.json', dict(
            purpose='historical_stage_regression_not_current_launch_admission',
            adapter_sources=adapter_sources, planning_sources=planning_sources,
            minimal_lineage_files=[dict(path=archive+'/'+name,sha256=a.digest(ROOT/archive/name))
                                   for archive,name in minimal],
            tests_unmodified=True, assertions_unmodified=True, no_monkeypatch=True))
        (out / 'test_check_ere_categorical_comparison_plan.stdout.txt').write_text(done.stdout)
        (out / 'test_check_ere_categorical_comparison_plan.stderr.txt').write_text(done.stderr)
        return done


def verify(out):
    driver = original_driver()
    legacy, bounded = driver.helpers()
    a = legacy.a
    out = out.resolve()
    assert out.parent == (ROOT / 'output').resolve() and not out.exists(), 'fresh direct output child required'
    a.verify(FAILED, expected=FAILED_ROOT, count=16)
    a.verify(FAILED_V2, expected=FAILED_V2_ROOT, count=16)
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
    driver.save(out / 'registration.json', dict(repository=str(ROOT), purpose='storage_equivalence_engineering_verification_v3',
        frozen_before_continuation_checks=True, sources=sources,
        failed_receipt=dict(path=str(FAILED), root_seal=FAILED_ROOT, sealed_entries=16),
        correction='preserve v1/v2 failures; preflight shape and historical regression stage binding only',
        prior_verification_failure=dict(path=str(FAILED_V2),root_seal=FAILED_V2_ROOT,sealed_entries=16),
        reuse_all_four_measurements_without_repeat=True, worker_timeout_seconds=900,
        native_evaluations_authorized=False, scientific_launch_authorized=False, training_authorized=False,
        launch_bridge_implemented=False))
    initial = {n: a.digest(out / n) for n in sorted(a.files(out))}
    driver.save(out / 'registration_sha256.json', initial)
    manifest = dict(status='running', purpose='packaging_only_engineering_continuation',
        regression_stage_binding_corrected=True,
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
            (ROOT / 'scripts/test_audit_ere_categorical_comparison.py', 7),
            (out / 'source_snapshots/scripts/test_verify_ere_categorical_comparison_bounded_v2.py', 3),
        ]
        for path, count in test_paths:
            done = subprocess.run([sys.executable, '-B', str(path), '-v'], cwd=ROOT, env=env,
                                  capture_output=True, text=True, timeout=900)
            (out / (path.stem + '.stdout.txt')).write_text(done.stdout)
            (out / (path.stem + '.stderr.txt')).write_text(done.stderr)
            regressions.append(dict(test=path.name, returncode=done.returncode, expected_groups=count))
            assert done.returncode == 0 and f'Ran {count} tests' in done.stderr
        plan_done = historical_plan_regression(driver, env, out)
        regressions.append(dict(test='test_check_ere_categorical_comparison_plan.py',
                                returncode=plan_done.returncode, expected_groups=8,
                                scope='hermetic historical adapter/planning stage'))
        assert plan_done.returncode == 0 and 'Ran 8 tests' in plan_done.stderr
        controls = dict(development=driver.detached_controls(bounded), preflight=preflight_controls(bounded, driver.PREFLIGHT))
        assert len(controls['development']) == 8 and len(controls['preflight']) == 6
        driver.save(out / 'controls.json', controls)
        assert all(a.digest(out / n) == h for n, h in initial.items())
        assert all(a.digest(ROOT / n) == h for n, h in historical.items())
        assert all(a.digest(ROOT / n) == h for n, h in sources.items())
        a.verify(driver.DEV, expected=driver.DEV_ROOT, count=692)
        a.verify(driver.PREFLIGHT, expected=driver.PREFLIGHT_ROOT, count=679)
        a.verify(FAILED, expected=FAILED_ROOT, count=16)
        a.verify(FAILED_V2, expected=FAILED_V2_ROOT, count=16)
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
