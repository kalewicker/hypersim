#!/usr/bin/env python3
"""Prospective engineering receipt; no native launch or archive replacement."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import platform
import resource
import shutil
import subprocess
import sys
import tempfile
import time
import traceback

ROOT = Path(__file__).resolve().parents[1]
if ROOT.name == 'source_snapshots':
    ROOT = Path(json.loads((ROOT.parent / 'registration.json').read_text())['repository'])
DEV = ROOT / 'output/ere_categorical_comparison_development_20261002'
DEV_ROOT = '3a9d48f7059dd910a8e008a566237cbf03935790629902f1b9305d496ecf3682'
PREFLIGHT = ROOT / 'output/ere_categorical_comparison_preflight_20261002'
PREFLIGHT_ROOT = 'bf2daa0d582fa05bc2bc9d8ea6c75bdc97991a1abd1dd48dea6be31b0da395b8'
SOURCES = [
    'scripts/audit_ere_categorical_comparison_bounded.py',
    'scripts/test_audit_ere_categorical_comparison_bounded.py',
    'scripts/measure_ere_categorical_comparison_bounded.py',
    'docs/ere_categorical_comparison_bounded_auditor_v1.md',
]
WORKERS = ('legacy-development', 'bounded-development', 'bounded-preflight', 'bounded-stress-12288')


def helpers():
    # The original auditor and dependencies come from the preserved run, not
    # mutable live imports. New auditor/tests resolve beside this frozen driver.
    sys.path.insert(0, str(DEV / 'source_snapshots/scripts'))
    import audit_ere_categorical_comparison as legacy
    import audit_ere_categorical_comparison_bounded as bounded
    return legacy, bounded


def deep_size(value):
    seen = set()

    def visit(x):
        if id(x) in seen:
            return 0
        seen.add(id(x))
        result = sys.getsizeof(x)
        if isinstance(x, dict):
            result += sum(visit(k) + visit(v) for k, v in x.items())
        elif isinstance(x, (list, tuple)):
            result += sum(visit(v) for v in x)
        return result
    return visit(value)


def stress(b):
    plan = b.a.load(DEV / 'schedule.json')
    assert len(plan) == 192
    native, h = b.cat.d.c.helpers(b.PARENT)
    catalogue = b.a.load(DEV / 'reference_catalogue.json')

    class RepeatedRows(b.BoundedRows):
        def _read(self, key):
            assert isinstance(key, str) and key.startswith('repeat-')
            i = int(key.removeprefix('repeat-'))
            assert key == f'repeat-{i}' and 0 <= i < 12288
            return super()._read(plan[i % 192]['id'])

    rows = RepeatedRows(DEV / 'cells')
    for i in range(12288):
        key = f'repeat-{i}'
        q = plan[i % 192]
        row = rows.read_unregistered(key)
        b.cat.d.b.exact_schedule([row['schedule']], [q])
        b.cat.d.b.cell(row, q, catalogue, native, h)
        rows[key] = row
    assert rows._pending is None and len(rows) == 12288
    for i in range(12288):
        assert rows[f'repeat-{i}']['status'] == 'Complete'
    return dict(purpose='synthetic_storage_stress_not_scientific_schedule',
                bindings=len(rows), distinct_historical_native_receipts=192,
                reuse_repetitions=64, cached_receipt_objects=0,
                retained_binding_deep_bytes=deep_size(rows.bindings),
                receipt_reads=rows.reads, receipt_bytes_read=rows.bytes_read,
                historical_native_trace_checks_with_repetition=12288,
                native_evaluations_run=0, native_steps_run=0, grammar_compilations_run=0,
                training_updates=0, scientific_campaign_complete=False, training_admitted=False)


def worker(name):
    start = time.monotonic()
    legacy, bounded = helpers()
    bounded.verify_storage_only()
    if name == 'legacy-development':
        result = legacy.audit(DEV)
    elif name == 'bounded-development':
        result = bounded.audit(DEV)
    elif name == 'bounded-preflight':
        result = bounded.audit(PREFLIGHT)
    else:
        assert name == 'bounded-stress-12288'
        legacy.a.verify(DEV, expected=DEV_ROOT, count=692)
        result = stress(bounded)
    # macOS returns bytes; Unix/Linux convention is KiB.
    raw = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return dict(worker=name, wall_seconds=time.monotonic() - start,
                peak_process_rss_bytes=raw if sys.platform == 'darwin' else raw * 1024,
                ru_maxrss_raw=raw, ru_maxrss_unit='bytes' if sys.platform == 'darwin' else 'KiB',
                result=result)


def detached_controls(b):
    cases = [
        ('summary.json', lambda x: x.update(support_categorical_direction=True)),
        ('manifest.json', lambda x: x.update(training_admitted=True)),
        ('manifest.json', lambda x: x.update(attempted_steps=x['attempted_steps'] + 1)),
        ('schedule.json', lambda x: x[0].update(physics_seed=x[0]['physics_seed'] + 1)),
        ('request.json', lambda x: x.update(full_training_authorized=True)),
        ('snapshot.json', lambda x: x.update(default_features=True)),
    ]
    first = b.a.load(DEV / 'schedule.json')[0]['id']
    name = f'cells/{first}.json'
    cases += [
        (name, lambda x: x['evaluation']['steps'][0].update(advanced=True)),
        (name, lambda x: x.update(constraint_utility=99.)),
    ]
    results = []
    with tempfile.TemporaryDirectory(prefix='ere-bounded-controls-') as directory:
        clone = Path(directory) / 'control'
        shutil.copytree(DEV, clone)
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
                raise AssertionError(f'tamper accepted: {index} {name}')
            finally:
                (clone / name).write_bytes(original)
    return results


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def measure(out):
    legacy, bounded = helpers()
    a = legacy.a
    out = out.resolve()
    assert out.parent == (ROOT / 'output').resolve() and not out.exists(), 'fresh direct output child required'
    a.verify(DEV, expected=DEV_ROOT, count=692)
    a.verify(PREFLIGHT, expected=PREFLIGHT_ROOT, count=679)
    historical = a.load(DEV / 'snapshot.json')['sources']
    assert all(a.digest(ROOT / n) == h for n, h in historical.items()), 'historical live source drift'
    bounded.verify_storage_only()
    sources = {name: a.digest(ROOT / name) for name in SOURCES}
    out.mkdir()
    for name in SOURCES:
        target = out / 'source_snapshots' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / name, target)
        assert a.digest(target) == sources[name]
    save(out / 'registration.json', dict(purpose='storage_equivalence_engineering_only',
        repository=str(ROOT), sources=sources, frozen_before_resource_measurement=True,
        native_development=dict(path=str(DEV), root_seal=DEV_ROOT, sealed_entries=692),
        zero_work_preflight=dict(path=str(PREFLIGHT), root_seal=PREFLIGHT_ROOT, sealed_entries=679),
        legacy_sha256=bounded.LEGACY_SHA256, workers=list(WORKERS),
        worker_timeout_seconds=900, stress_bindings=12288, stress_peak_rss_cap_bytes=512*1024*1024,
        platform=platform.platform(), python=sys.version, executable=sys.executable,
        native_evaluations_authorized=False, scientific_launch_authorized=False,
        training_authorized=False, launch_bridge_implemented=False))
    initial = {n: a.digest(out / n) for n in sorted(a.files(out))}
    save(out / 'registration_sha256.json', initial)
    registration_root = a.digest(out / 'registration_sha256.json')
    manifest = dict(status='running', purpose='storage_equivalence_engineering_only',
        registration_root=registration_root, native_evaluations_run=0, native_steps_run=0,
        grammar_compilations_run=0, training_updates=0, scientific_campaign_complete=False,
        training_admitted=False, launch_bridge_implemented=False)
    save(out / 'manifest.json', manifest)
    a.seal(out)
    frozen_driver = out / 'source_snapshots/scripts/measure_ere_categorical_comparison_bounded.py'
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', PYTHONPATH=str(DEV / 'source_snapshots/scripts'))
    try:
        measurements = {}
        for name in WORKERS:
            done = subprocess.run([sys.executable, '-B', str(frozen_driver), '--worker', name],
                                  cwd=ROOT, env=env, capture_output=True, text=True, timeout=900)
            (out / (name + '.stdout.txt')).write_text(done.stdout)
            (out / (name + '.stderr.txt')).write_text(done.stderr)
            assert done.returncode == 0, f'worker failed: {name}: {done.stderr}'
            measurements[name] = json.loads(done.stdout)
        eager = measurements['legacy-development']
        small = measurements['bounded-development']
        saved = copy.deepcopy(a.load(DEV / 'audit.json'))
        assert len(saved.pop('detached_tamper_rejections')) == 6
        assert eager['result'] == small['result'] == saved, 'development result not exactly equivalent'
        zero = copy.deepcopy(a.load(PREFLIGHT / 'audit.json'))
        assert len(zero.pop('detached_tamper_rejections')) == 6
        assert measurements['bounded-preflight']['result'] == zero
        assert small['peak_process_rss_bytes'] < eager['peak_process_rss_bytes'], 'development RSS did not improve'
        assert measurements['bounded-stress-12288']['peak_process_rss_bytes'] < 512*1024*1024, 'stress RSS cap'
        save(out / 'measurements.json', measurements)
        regressions = []
        test_paths = [
            out / 'source_snapshots/scripts/test_audit_ere_categorical_comparison_bounded.py',
            DEV / 'source_snapshots/scripts/test_audit_ere_categorical_comparison.py',
            DEV / 'source_snapshots/scripts/test_check_ere_categorical_comparison_plan.py',
        ]
        for path in test_paths:
            done = subprocess.run([sys.executable, '-B', str(path), '-v'], cwd=ROOT, env=env,
                                  capture_output=True, text=True, timeout=900)
            (out / (path.stem + '.stdout.txt')).write_text(done.stdout)
            (out / (path.stem + '.stderr.txt')).write_text(done.stderr)
            regressions.append(dict(test=path.name, returncode=done.returncode))
            assert done.returncode == 0, f'regression failure: {path.name}'
        controls = detached_controls(bounded)
        save(out / 'controls.json', controls)
        assert all(a.digest(out / n) == h for n, h in initial.items()), 'registered input changed'
        assert all(a.digest(ROOT / n) == h for n, h in historical.items()), 'historical source changed'
        assert all(a.digest(ROOT / n) == h for n, h in sources.items()), 'new source changed'
        a.verify(DEV, expected=DEV_ROOT, count=692)
        a.verify(PREFLIGHT, expected=PREFLIGHT_ROOT, count=679)
        save(out / 'verification.json', dict(engineering_pass=True,
            audit_results_exactly_equivalent=True, storage_only_ast=bounded.verify_storage_only(),
            preserved_historical_sources=len(historical), preserved_input_roots=[DEV_ROOT, PREFLIGHT_ROOT],
            regressions=regressions, detached_tamper_rejections=len(controls),
            rss_reduction_fraction=1-small['peak_process_rss_bytes']/eager['peak_process_rss_bytes'],
            scientific_support_claim=False, launch_bridge_implemented=False))
        manifest['status'] = 'complete'
    except Exception as exc:
        manifest.update(status='failed_engineering', failure=str(exc))
        (out / 'failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        save(out / 'manifest.json', manifest)
        a.seal(out)
    count = a.verify(out)
    print(json.dumps(dict(archive=str(out), root_seal=a.digest(out / 'output_sha256.json'),
                         sealed_entries=count, **manifest), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--worker', choices=WORKERS)
    args = parser.parse_args()
    assert (args.output is None) != (args.worker is None), 'choose output or isolated worker'
    if args.worker:
        print(json.dumps(worker(args.worker)))
    else:
        measure(args.output)
