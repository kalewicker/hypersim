#!/usr/bin/env python3
"""Register/freeze an approved historical-only audit and verify saved receipts once."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time

import audit_ere_categorical_comparison_archive as h

SOURCES = [
    'scripts/audit_ere_categorical_comparison_archive.py',
    'scripts/test_audit_ere_categorical_comparison_archive.py',
    'scripts/verify_ere_categorical_comparison_archive.py',
    'docs/ere_categorical_comparison_archive_audit_v1.md',
]


def write(path, value):
    path.write_text(json.dumps(value, indent=2))


def verify(out):
    out=Path(out).absolute()
    assert out.parent.resolve()==(h.ROOT/'output').resolve() and not out.exists(), 'fresh direct output child required'
    h.verify_measurement()
    hashes={n:h.a.digest(h.ROOT/n) for n in SOURCES}
    out.mkdir()
    for n, digest in hashes.items():
        dest=out/'source_snapshots'/n; dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(h.ROOT/n, dest); assert h.a.digest(dest)==digest
    write(out/'registration.json', dict(profile=h.PROFILE, repository=str(h.ROOT),
        postmeasurement=True, historical_measurement_only=True, frozen_before_checks=True,
        user_authorized_archive_only_policy=True,
        measured_archive=str(h.s.INPUT), measured_root=h.s.INPUT_ROOT, measured_entries=12883,
        ordered_implementation_archive=str(h.PRIOR), ordered_implementation_root=h.PRIOR_ROOT,
        sources=hashes, audit_timeout_seconds=900, regression_suite_timeout_seconds=180,
        current_checkout_identity_claim=False, current_code_admission_claim=False,
        native_evaluations_allocated=0, native_steps_allocated=0,
        grammar_compilations_allocated=0, training_updates_allocated=0))
    write(out/'reused_correction.json', h.s.PROOF)
    (out/'mechanical.diff').write_text(h.s.DIFF)
    prepared={n:h.a.digest(out/n) for n in h.a.files(out)}
    write(out/'prepared_sha256.json', prepared)
    m=dict(profile=h.PROFILE, status='registered_before_checks', postmeasurement=True,
        historical_measurement_only=True, current_checkout_identity_claim=False,
        current_code_admission_claim=False, training_admitted=False,
        native_evaluations_run=0, native_steps_run=0, grammar_compilations_run=0, training_updates_run=0)
    write(out/'manifest.json', m); h.a.seal(out)
    start=time.monotonic()
    try:
        controls={}
        suites=[('ordered', h.PRIOR/'source_snapshots/scripts', 'test_audit_ere_categorical_comparison_ordered', 9, 'detached_helper_semantic_rejections', 15),
                ('archive_policy', out/'source_snapshots/scripts', 'test_audit_ere_categorical_comparison_archive', 4, 'detached_source_rejections', 4)]
        for name, cwd, module, count, key, expected_controls in suites:
            done=subprocess.run([sys.executable, '-B', '-m', 'unittest', '-v', module], cwd=cwd,
                                capture_output=True, text=True, timeout=180)
            write(out/(name+'_regressions.json'), dict(command=done.args, returncode=done.returncode,
                  stdout=done.stdout, stderr=done.stderr))
            assert done.returncode==0, f'{name} frozen regressions failed; no retry'
            assert f'Ran {count} tests' in done.stderr and done.stderr.rstrip().endswith('OK')
            rejected=json.loads(done.stdout)[key]
            assert len(rejected)==len(set(rejected))==expected_controls
            controls[name]=dict(scope='copied helper/source fixtures, not full archive tamper audits', rejected=rejected)
        write(out/'detached_controls.json', controls)
        m.update(status='auditing_saved_receipts', passed_regression_groups=13)
        write(out/'manifest.json', m); h.a.seal(out)
        checked=subprocess.run([sys.executable, '-B', str(out/'source_snapshots/scripts/audit_ere_categorical_comparison_archive.py')],
                               cwd=h.ROOT, capture_output=True, text=True, timeout=900)
        write(out/'audit_process.json', dict(command=checked.args, returncode=checked.returncode,
              stdout=checked.stdout, stderr=checked.stderr))
        assert checked.returncode==0, 'frozen historical audit failed; no retry'
        result=json.loads(checked.stdout)
        assert result['profile']==h.PROFILE and result['archive_accounting_pass'] is True
        assert result['historical_measurement_only'] is True
        assert result['current_checkout_identity_claim'] is result['current_code_admission_claim'] is False
        assert result['original_frozen_audit_pass'] is False
        accounting=result['corrected_accounting']
        assert accounting['scientific_campaign_complete'] is True and accounting['recorded_cells']==12288
        assert accounting['training_admitted'] is False
        h.s.original.exact(result['reused_correction'], h.s.PROOF)
        assert all(result[k]==0 for k in ('native_evaluations_run','native_steps_run','grammar_compilations_run','training_updates_run'))
        write(out/'audit.json', result)
        h.verify_measurement()
        assert all(h.a.digest(h.ROOT/n)==digest for n,digest in hashes.items()), 'new policy live source drift'
        assert all(h.a.digest(out/n)==digest for n,digest in prepared.items()), 'new policy frozen input drift'
        m.update(status='complete', archive_accounting_pass=True, original_frozen_audit_pass=False,
            preserved_prior_failures=True, detached_helper_semantic_rejections=15,
            detached_source_rejections=4, support_categorical_direction=accounting['support_categorical_direction'],
            observed_checkout_drift_paths=[x['path'] for x in result['checkout_drift_after']['changes']])
    except Exception as error:
        m.update(status='failed', reason=f'{type(error).__name__}: {error}')
        if isinstance(error, subprocess.TimeoutExpired):
            write(out/'timeout.json', dict(command=error.cmd, timeout=error.timeout,
                  stdout=str(error.stdout), stderr=str(error.stderr)))
        raise
    finally:
        m['elapsed_seconds']=time.monotonic()-start
        write(out/'manifest.json', m); h.a.seal(out)
    h.a.verify(out)
    print(json.dumps(dict(archive=str(out), root_seal=h.a.digest(out/'output_sha256.json'),
                         sealed_entries=len(h.a.load(out/'output_sha256.json')), **m), indent=2))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    verify(parser.parse_args().output)
