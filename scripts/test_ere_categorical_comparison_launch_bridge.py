import argparse
import copy
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import traceback
import unittest

import ere_categorical_comparison_launch_bridge as b
import audit_ere_categorical_comparison_bridge as v
import run_ere_categorical_comparison_bridge as driver


class BridgeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.c = v.config()

    def test_registered_481_sources_and_470_identical_paths(self):
        r = b.registration(b.ROOT, self.c, check_historical=True)
        self.assertEqual(len(r['source_names']), 481)
        self.assertEqual(r['unchanged_historical_paths'], 470)
        self.assertFalse(r['full_source_identity_claim'])
        self.assertTrue(v.verify_bridge_body()['accounting_ast_identical_except_registered_bridge'])

    def fixture(self, directory, seed, authorized=False, status='zero_work_preflight'):
        path = Path(directory) / 'history'
        path.mkdir()
        m = driver.initial('scientific', 1)
        m['status'] = status
        if status != 'zero_work_preflight':
            m.update(recorded_cells=1, attempted_steps=100, grammar_compilations=1)
        driver.write(path / 'manifest.json', m)
        driver.write(path / 'request.json', dict(launch_authorized=authorized))
        driver.write(path / 'schedule.json', [dict(emission_seed=seed, physics_seed=seed+1, noise_seed=None)])
        return path

    def test_selected_mode_keeps_development_reserved_and_science_disjoint(self):
        seed = v.p.schedule(self.c, 'development')[0]['emission_seed']
        with tempfile.TemporaryDirectory() as directory:
            self.fixture(directory, seed, True, 'complete')
            self.assertEqual(len(b.seed_inventory(self.c, 'scientific', directory)['prior_schedule_files']), 1)
            with self.assertRaisesRegex(AssertionError, 'development.*collision'):
                b.seed_inventory(self.c, 'development', directory)

    def test_false_launch_proven_zero_work_does_not_reserve(self):
        seed = v.p.schedule(self.c, 'scientific')[0]['emission_seed']
        with tempfile.TemporaryDirectory() as directory:
            self.fixture(directory, seed, False)
            self.assertEqual(b.seed_inventory(self.c, 'scientific', directory)['prior_schedule_files'], [])

    def test_authorized_zero_work_reserves_entire_schedule(self):
        seed = v.p.schedule(self.c, 'scientific')[0]['emission_seed']
        with tempfile.TemporaryDirectory() as directory:
            path = self.fixture(directory, seed, True)
            with self.assertRaisesRegex(AssertionError, 'collision'):
                b.seed_inventory(self.c, 'scientific', directory)
            self.assertEqual(b.seed_inventory(self.c, 'scientific', directory, exclude=path)['prior_schedule_files'], [])

    def test_ambiguous_and_boolean_zero_metadata_never_skip(self):
        seed = v.p.schedule(self.c, 'scientific')[0]['emission_seed']
        with tempfile.TemporaryDirectory() as directory:
            path = self.fixture(directory, seed, False)
            (path / 'request.json').unlink()
            with self.assertRaisesRegex(AssertionError, 'collision'):
                b.seed_inventory(self.c, 'scientific', directory)
            driver.write(path / 'request.json', dict(launch_authorized=False))
            m = v.a.load(path / 'manifest.json'); m['recorded_cells'] = False
            driver.write(path / 'manifest.json', m)
            with self.assertRaisesRegex(AssertionError, 'collision'):
                b.seed_inventory(self.c, 'scientific', directory)

    def test_cross_role_collision_and_malformed_seed_reject(self):
        seed = v.p.schedule(self.c, 'scientific')[0]['noise_seed']
        with tempfile.TemporaryDirectory() as directory:
            path = self.fixture(directory, 7, True, 'complete')
            driver.write(path / 'schedule.json', [dict(physics_seed=seed)])
            with self.assertRaisesRegex(AssertionError, 'collision'):
                b.seed_inventory(self.c, 'scientific', directory)
            for bad in (True, -1, 1.2, 2**64):
                driver.write(path / 'schedule.json', [dict(noise_seed=bad)])
                with self.assertRaisesRegex(AssertionError, 'malformed explicit seed'):
                    b.seed_inventory(self.c, 'scientific', directory)

    def test_seed_history_symlink_and_metadata_drift_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            path = self.fixture(directory, 7, True, 'complete')
            before = b.seed_inventory(self.c, 'scientific', directory)
            driver.write(path / 'request.json', dict(launch_authorized=True, marker='changed'))
            with self.assertRaises(AssertionError):
                v.exact(before, b.seed_inventory(self.c, 'scientific', directory))
            (Path(directory) / 'link').symlink_to(path, target_is_directory=True)
            with self.assertRaisesRegex(AssertionError, 'symlink'):
                b.seed_inventory(self.c, 'scientific', directory)

    def test_source_drift_and_broadened_exports_reject(self):
        r = b.registration(b.ROOT, self.c)
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            for name in r['source_names']:
                target = base / name; target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(b.ROOT / name, target)
            for name in ('crates/hg-engine/src/engine.rs', b.MODULE, b.CARGO, b.NEW_NATIVE,
                         'scripts/audit_ere_categorical_comparison_bounded.py'):
                original = (base / name).read_bytes()
                marker = b'\n# unregistered fixture drift\n' if name == b.CARGO else b'\n// unregistered fixture drift\n'
                (base / name).write_bytes(original + marker)
                with self.assertRaises(AssertionError):
                    b.registration(base, self.c)
                (base / name).write_bytes(original)
            (base / 'crates/hg-train/src/unregistered.rs').write_text('// fixture')
            with self.assertRaisesRegex(AssertionError, 'unregistered learning source'):
                b.registration(base, self.c)

    def test_new_binary_offline_math_is_exactly_original(self):
        original = b.DEV / 'executed_binary'
        current = b.ROOT / 'target/release' / b.BIN
        self.assertTrue(current.is_file())
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            request = path / 'request.json'
            driver.write(request, dict(purpose='engineering_math_only', raw_vectors=[
                [v.p.f32(.02*math.sin(i+e)) for i in range(183)] for e in range(4)]))
            results = []
            for i, binary in enumerate((original, current)):
                result = path / f'result-{i}.json'
                done = subprocess.run([str(binary), 'math-fixture', str(b.DEV/'reference_catalogue.json'), str(request), str(result)], capture_output=True, text=True)
                self.assertEqual(done.returncode, 0, done.stderr)
                results.append(v.a.load(result))
            v.exact(*results)
            self.assertEqual(results[1]['native_steps_run'], 0)
            self.assertEqual(results[1]['grammar_compilations'], 0)
            self.assertFalse(results[1]['training_admitted'])

    def test_driver_refusals_precede_output_or_work(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(AssertionError, 'fresh comparison output'):
                driver.prepare(Path(directory), 'scientific')
            path = b.ROOT / 'output/never_created_bridge_unit_fixture'
            self.assertFalse(path.exists())
            with self.assertRaisesRegex(AssertionError, 'science-only'):
                driver.prepare(path, 'development')
            with self.assertRaisesRegex(AssertionError, 'explicit bridge preflight'):
                driver.prepare(path, 'scientific', execute=True)
            self.assertFalse(path.exists())

    def test_forged_gate_mode_roots_and_waivers_reject_early(self):
        value = b.gate()
        seeds = dict(selected_mode='scientific')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            driver.write(path / 'request.json', dict(launch_authorized=False))
            for mutation in (lambda x:x['development'].update(root_seal='0'*64),
                             lambda x:x['storage_verification'].update(root_seal='0'*64),
                             lambda x:x.update(waive_sources=True)):
                bad = copy.deepcopy(value); mutation(bad)
                with self.assertRaises(AssertionError):
                    b.validate_gate(bad, {}, {}, self.c, 'scientific', seeds, path)
            with self.assertRaisesRegex(AssertionError, 'science-only'):
                b.validate_gate(value, {}, {}, self.c, 'development', seeds, path)
            driver.write(path / 'request.json', dict(launch_authorized=True))
            with self.assertRaisesRegex(AssertionError, 'source-identical zero-work'):
                b.validate_gate(value, {}, {}, self.c, 'scientific', seeds, path)


def detached_verification(preflight):
    """Read-only parent; mutate/reseal disposable copies only, never run true launch."""
    preflight = preflight.resolve()
    root = v.a.digest(preflight / 'output_sha256.json')
    v.a.verify(preflight, expected=root)
    saved = v.a.load(preflight / 'audit.json')
    v.exact(saved, v.audit(preflight))
    assert saved['classification'] == 'zero_work_preflight' and saved['recorded_cells'] == 0
    guards = []
    binary = preflight / 'executed_binary'
    for command, digest in (('run-prepared', v.a.digest(preflight / 'prepared_sha256.json')),
                            ('check-prepared', '0'*64)):
        done = subprocess.run([str(binary), command, str(preflight), digest], capture_output=True, text=True)
        assert done.returncode != 0
        guards.append(dict(command=command, rejected=True, stderr=done.stderr))
        v.a.verify(preflight, expected=root)

    def repin(clone):
        pins = v.a.load(clone / 'prepared_sha256.json')
        driver.write(clone / 'prepared_sha256.json', {n:v.a.digest(clone/n) for n in pins})
        v.a.seal(clone)

    controls = [
        ('summary.json', lambda x:x.update(support_categorical_direction=True)),
        ('manifest.json', lambda x:x.update(training_admitted=True)),
        ('manifest.json', lambda x:x.update(attempted_steps=1)),
        ('schedule.json', lambda x:x[0].update(physics_seed=x[0]['physics_seed']+1)),
        ('request.json', lambda x:x.update(full_training_authorized=True)),
        ('snapshot.json', lambda x:x.update(default_features=True)),
        ('development_gate.json', lambda x:x['development'].update(root_seal='0'*64)),
        ('development_gate.json', lambda x:x['storage_verification'].update(root_seal='0'*64)),
        ('development_gate.json', lambda x:x.update(waive_sources=True)),
        ('seed_inventory.json', lambda x:x.update(selected_mode='development')),
        ('config.json', lambda x:x.update(step_radius=.2)),
        ('manifest.json', lambda x:x.update(inflight_native_work_unknown=True)),
    ]
    rejected = []
    with tempfile.TemporaryDirectory(prefix='ere-bridge-controls-') as directory:
        clone = Path(directory) / 'control'; shutil.copytree(preflight, clone)
        for index, (name, mutate) in enumerate(controls):
            original = (clone / name).read_bytes()
            value = v.a.load(clone / name); mutate(value); driver.write(clone / name, value)
            repin(clone)  # Exercise semantic checks, not just stale input hashes.
            try:
                v.audit(clone)
            except (AssertionError, KeyError, ValueError, TypeError, IndexError) as exc:
                rejected.append(dict(index=index, path=name, rejected=True, reason=str(exc)))
            else:
                raise AssertionError(f'bridge tamper accepted: {name}')
            finally:
                (clone / name).write_bytes(original); repin(clone)
        request = v.a.load(clone / 'request.json'); request['launch_authorized'] = True
        driver.write(clone / 'request.json', request)
        gate = b.gate(dict(path=str(preflight), root_seal=root)); driver.write(clone / 'development_gate.json', gate)
        repin(clone)
        # The only true-bit invocation is this check-only command. Never run-prepared.
        done = subprocess.run([str(clone/'executed_binary'), 'check-prepared', str(clone), v.a.digest(clone/'prepared_sha256.json')], capture_output=True, text=True, timeout=900)
        assert done.returncode == 0, done.stderr
        check = json.loads(done.stdout)
        v.exact(check, dict(bridge_preflight_checked=True, native_evaluations_run=0, native_steps_run=0, grammar_compilations_run=0, training_updates=0))
        assert v.a.load(clone/'manifest.json') == v.a.load(clone/'initial_manifest.json')
        assert not (clone/'catalogue.json').exists()
    v.a.verify(preflight, expected=root)
    b.registration(b.ROOT, v.config(), check_historical=True)
    return dict(engineering_pass=True, parent_preflight_root=root,
                source_stage=b.registration(b.ROOT, v.config()),
                accounting_ast=v.verify_bridge_body(), native_guard_rejections=guards,
                semantic_tamper_rejections=rejected, synthetic_check_only=check,
                native_evaluations_run=0, native_steps_run=0, grammar_compilations_run=0,
                training_updates=0, scientific_campaign_complete=False, training_admitted=False)


def verify_receipt(preflight, output):
    output = output.resolve(); preflight = preflight.resolve()
    assert output.parent == (b.ROOT/'output').resolve() and not output.exists(), 'fresh engineering output required'
    root = v.a.digest(preflight/'output_sha256.json'); v.a.verify(preflight, expected=root)
    snapshot = v.a.load(preflight/'snapshot.json')
    assert all(v.a.digest(b.ROOT/n)==h for n,h in snapshot['sources'].items())
    output.mkdir()
    driver.write(output/'registration.json', dict(profile=b.PROFILE, purpose='zero_native_bridge_verification',
        preflight=dict(path=str(preflight),root_seal=root), source_hashes=snapshot['sources'],
        frozen_checker=str(Path(__file__).resolve()), registered_before_checks=True,
        python_regression_groups=11, semantic_tamper_controls=12, true_launch_invocations=0))
    v.a.seal(output)
    manifest = dict(status='running', native_evaluations_run=0, native_steps_run=0,
                    grammar_compilations_run=0, training_updates=0, scientific_campaign_complete=False, training_admitted=False)
    try:
        done = subprocess.run([sys.executable, '-B', str(Path(__file__).resolve()), '-v'], cwd=b.ROOT, capture_output=True, text=True, timeout=900)
        (output/'regressions.stdout.txt').write_text(done.stdout); (output/'regressions.stderr.txt').write_text(done.stderr)
        assert done.returncode == 0 and 'Ran 11 tests' in done.stderr, done.stderr
        result = detached_verification(preflight)
        driver.write(output/'verification.json', result)
        assert all(v.a.digest(b.ROOT/n)==h for n,h in snapshot['sources'].items())
        manifest['status'] = 'complete'
    except Exception as exc:
        manifest.update(status='failed_engineering', failure=str(exc))
        (output/'failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        driver.write(output/'manifest.json',manifest);v.a.seal(output)
    print(json.dumps(dict(archive=str(output),sealed_entries=v.a.verify(output),root_seal=v.a.digest(output/'output_sha256.json'),**manifest),indent=2))


if __name__ == '__main__':
    if '--verify-preflight' in sys.argv:
        parser = argparse.ArgumentParser()
        parser.add_argument('--verify-preflight', type=Path, required=True)
        parser.add_argument('--receipt', type=Path, required=True)
        args = parser.parse_args();verify_receipt(args.verify_preflight,args.receipt)
    else:
        unittest.main()
