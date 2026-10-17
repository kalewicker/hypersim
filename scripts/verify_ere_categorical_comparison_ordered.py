#!/usr/bin/env python3
"""Freeze a separate postmeasurement correction, then run receipt-only checks once."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time

import audit_ere_categorical_comparison_ordered as s

SOURCES = [
    'scripts/audit_ere_categorical_comparison_ordered.py',
    'scripts/test_audit_ere_categorical_comparison_ordered.py',
    'scripts/verify_ere_categorical_comparison_ordered.py',
    'docs/ere_categorical_comparison_ordered_audit_v1.md',
]


def write(path, value):
    path.write_text(json.dumps(value, indent=2))


def verify(out, prior):
    out = Path(out).absolute()
    assert out.parent.resolve() == (s.ROOT/'output').resolve() and not out.exists(), 'fresh direct output child required'
    s.verify_input()
    prior = Path(prior).resolve()
    assert prior == s.ROOT/'output/ere_categorical_comparison_ordered_audit_20261003'
    prior_root = '85cd6236489bf9feee0a8b864bd0f0292c210af5f3fb3350ac9d97cce61598bc'
    s.a.verify(prior, expected=prior_root, count=10)
    assert s.a.load(prior/'manifest.json')['status'] == 'failed'
    assert not (prior/'audit_process.json').exists(), 'prior full audit must be unrun'
    prior_sources = s.a.load(prior/'registration.json')['sources']
    assert s.a.digest(s.ROOT/SOURCES[0]) == prior_sources[SOURCES[0]], 'arithmetic correction changed'
    old_test = (prior/'source_snapshots'/SOURCES[1]).read_text()
    assert old_test.count('s.original.matched(') == 2
    assert (s.ROOT/SOURCES[1]).read_text() == old_test.replace('s.original.matched(', 's.original.bounded.legacy.matched('), 'test changes beyond missing helper reference'
    hashes = {n:s.a.digest(s.ROOT/n) for n in SOURCES}
    out.mkdir()
    for n,h in hashes.items():
        dest = out/'source_snapshots'/n
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(s.ROOT/n, dest)
        assert s.a.digest(dest) == h
    write(out/'registration.json', dict(profile=s.PROFILE, repository=str(s.ROOT),
        verification_revision=2, preserved_failed_verification=dict(path=str(prior), root_seal=prior_root),
        input_archive=str(s.INPUT), input_root=s.INPUT_ROOT, input_entries=12883,
        original_failure_sha256=s.FAILED_SHA, sources=hashes,
        postmeasurement=True, frozen_before_checks=True,
        tests_timeout_seconds=180, audit_timeout_seconds=900,
        native_evaluations_allocated=0, native_steps_allocated=0,
        grammar_compilations_allocated=0, training_updates_allocated=0))
    write(out/'correction.json', s.PROOF)
    (out/'mechanical.diff').write_text(s.DIFF)
    prepared = {n:s.a.digest(out/n) for n in s.a.files(out)}
    write(out/'prepared_sha256.json', prepared)
    m = dict(profile=s.PROFILE, status='registered_before_checks', postmeasurement=True,
             native_evaluations_run=0, native_steps_run=0, grammar_compilations_run=0,
             training_updates_run=0, training_admitted=False)
    write(out/'manifest.json', m)
    s.a.seal(out)
    start = time.monotonic()
    try:
        scripts = out/'source_snapshots/scripts'
        tests = subprocess.run([sys.executable, '-B', '-m', 'unittest', '-v',
                                'test_audit_ere_categorical_comparison_ordered'],
                               cwd=scripts, capture_output=True, text=True, timeout=180)
        write(out/'regressions.json', dict(command=tests.args, returncode=tests.returncode,
                                         stdout=tests.stdout, stderr=tests.stderr))
        assert tests.returncode == 0, 'frozen regression failure; no retry'
        assert 'Ran 9 tests' in tests.stderr and tests.stderr.rstrip().endswith('OK'), 'registered group count'
        controls = json.loads(tests.stdout)['detached_helper_semantic_rejections']
        assert len(controls) == len(set(controls)) == 15, 'registered semantic control count'
        write(out/'detached_controls.json', dict(scope='copied in-memory helper-stage receipts, not whole archive audits',
                                               rejected=controls, unchanged_fixtures_passed=True))
        checked = subprocess.run([sys.executable, '-B', str(scripts/'audit_ere_categorical_comparison_ordered.py')],
                                 cwd=s.ROOT, capture_output=True, text=True, timeout=900)
        write(out/'audit_process.json', dict(command=checked.args, returncode=checked.returncode,
                                           stdout=checked.stdout, stderr=checked.stderr))
        assert checked.returncode == 0, 'frozen supplemental audit failure; no retry'
        result = json.loads(checked.stdout)
        s.original.exact(result['correction'], s.PROOF)
        assert result['corrected_accounting']['audit_pass'] is True
        assert result['corrected_accounting']['scientific_campaign_complete'] is True
        assert result['corrected_accounting']['recorded_cells'] == 12288
        assert result['original_frozen_audit_pass'] is False
        assert result['training_updates_run'] == result['native_steps_run'] == 0
        write(out/'audit.json', result)
        s.verify_input()
        s.a.verify(prior, expected=prior_root, count=10)
        assert all(s.a.digest(s.ROOT/n)==h for n,h in hashes.items()), 'supplemental live source drift'
        assert all(s.a.digest(out/n)==h for n,h in prepared.items()), 'supplemental prepared input drift'
        m.update(status='complete', corrected_accounting_pass=True, original_frozen_audit_pass=False,
                 original_failure_preserved=True, passed_regression_groups=9,
                 detached_helper_semantic_rejections=15,
                 support_categorical_direction=result['corrected_accounting']['support_categorical_direction'])
    except Exception as error:
        m.update(status='failed', reason=f'{type(error).__name__}: {error}')
        if isinstance(error, subprocess.TimeoutExpired):
            write(out/'timeout.json', dict(command=error.cmd, timeout=error.timeout,
                  stdout=str(error.stdout), stderr=str(error.stderr)))
        raise
    finally:
        m['elapsed_seconds'] = time.monotonic()-start
        write(out/'manifest.json', m)
        s.a.seal(out)
    s.a.verify(out)
    print(json.dumps(dict(archive=str(out), root_seal=s.a.digest(out/'output_sha256.json'),
                         sealed_entries=len(s.a.load(out/'output_sha256.json')), **m), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--prior-verification', required=True, type=Path)
    args = parser.parse_args()
    verify(args.output, args.prior_verification)
