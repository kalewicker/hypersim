#!/usr/bin/env python3
"""Historical receipt accounting; sealed sources bind evidence, live drift is descriptive."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import time

sys.dont_write_bytecode = True
PROFILE = 'E2-categorical-comparison-historical-archive-audit-v1'
PRIOR_NAME = 'ere_categorical_comparison_ordered_audit_v2_20261003'
PRIOR_ROOT = 'd69addbb3e6e681c3929a4329316386610308a0ef80316411f701e024f69003d'
FIRST_ROOT = '85cd6236489bf9feee0a8b864bd0f0292c210af5f3fb3350ac9d97cce61598bc'
ROOT = next(p for p in Path(__file__).resolve().parents
            if (p/'output'/PRIOR_NAME/'output_sha256.json').is_file())
PRIOR = ROOT/'output'/PRIOR_NAME
assert hashlib.sha256((PRIOR/'output_sha256.json').read_bytes()).hexdigest() == PRIOR_ROOT
prior_pins = json.loads((PRIOR/'output_sha256.json').read_text())
for name, h in prior_pins.items():
    if name.startswith('source_snapshots/scripts/'):
        file = PRIOR/name
        assert not file.is_symlink() and hashlib.sha256(file.read_bytes()).hexdigest() == h
sys.path.insert(0, str(PRIOR/'source_snapshots/scripts'))
import audit_ere_categorical_comparison_ordered as s

a = s.a
assert Path(s.__file__).resolve() == (PRIOR/'source_snapshots/scripts/audit_ere_categorical_comparison_ordered.py').resolve()


def regular_path(base, name):
    assert a.safe(name), 'unsafe source name'
    file = Path(base)
    assert not file.is_symlink(), 'symlink source root'
    for part in Path(name).parts:
        file = file/part
        assert not file.is_symlink(), 'symlink source path'
    assert file.is_file(), 'missing source file'
    return file


def verify_snapshot_sources(base, sources):
    for name, expected in sources.items():
        assert isinstance(expected, str) and len(expected) == 64
        assert a.digest(regular_path(base, name)) == expected, 'sealed snapshot source drift'


def verify_measurement():
    a.verify(s.INPUT, expected=s.INPUT_ROOT, count=12883)
    assert a.digest(s.INPUT/'failed_audit.json') == s.FAILED_SHA
    assert not (s.INPUT/'audit.json').exists(), 'original failure must remain preserved'
    snap = a.load(s.INPUT/'snapshot.json')
    assert len(snap['sources']) == 481
    verify_snapshot_sources(s.FROZEN, snap['sources'])
    assert a.digest(s.INPUT/'executed_binary') == snap['binary_sha256']
    a.verify(PRIOR, expected=PRIOR_ROOT, count=12)
    a.verify(ROOT/'output/ere_categorical_comparison_ordered_audit_20261003', expected=FIRST_ROOT, count=10)
    assert a.load(PRIOR/'manifest.json')['status'] == 'failed'
    assert not (PRIOR/'audit.json').exists()
    return snap


def checkout_drift(sources, repository):
    changes = []
    for name, expected in sorted(sources.items()):
        try:
            observed = a.digest(regular_path(repository, name))
            if observed != expected:
                changes.append(dict(path=name, status='changed', measurement_sha256=expected,
                                    observed_sha256=observed))
        except (AssertionError, OSError) as error:
            changes.append(dict(path=name, status='unavailable_or_unsafe', measurement_sha256=expected,
                                observed_sha256=None, reason=str(error)))
    return dict(captured_unix_seconds=time.time(), checked_sources=len(sources),
                matching_sources=len(sources)-len(changes), changes=changes,
                current_checkout_identity_claim=False, current_code_admission_claim=False,
                scope='point-in-time live file observations, not acceptance conditions')


def audit():
    snap = verify_measurement()
    before = checkout_drift(snap['sources'], ROOT)
    # Reuse the entire unchanged frozen bridge auditor with the pinned three-sum
    # reference correction. Never call s.audit(): its old live-equality policy
    # remains preserved and is intentionally not this historical-only policy.
    result = s.original.audit(s.INPUT)
    verify_measurement()
    after = checkout_drift(snap['sources'], ROOT)
    return dict(profile=PROFILE, postmeasurement=True, historical_measurement_only=True,
                archive_accounting_pass=result['audit_pass'], corrected_accounting=result,
                original_frozen_audit_pass=False, original_failure_preserved=True,
                prior_failed_verifications_preserved=True,
                input_archive=str(s.INPUT), input_root=s.INPUT_ROOT,
                reused_correction=s.PROOF, summary=a.load(s.INPUT/'summary.json'),
                checkout_drift_before=before, checkout_drift_after=after,
                current_checkout_identity_claim=False, current_code_admission_claim=False,
                native_evaluations_run=0, native_steps_run=0, grammar_compilations_run=0,
                training_updates_run=0, training_admitted=False,
                independent_physics_replay=False, independent_rng_replay=False)


if __name__ == '__main__':
    argparse.ArgumentParser(description=__doc__).parse_args()
    print(json.dumps(audit(), indent=2))
