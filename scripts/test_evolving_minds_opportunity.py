#!/usr/bin/env python3
"""Adversarial accounting and real archived native replay checks."""
import copy
from fractions import Fraction
import json
import os
from pathlib import Path
import tempfile
import unittest

import evolving_minds_opportunity as o


class OpportunityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root = Path(os.environ.get('EVOLVING_MINDS_FUNDED_ADMISSION_FIXTURE',
            str(Path(__file__).resolve().parents[1] / 'output/evolving_minds_funded_admission_development_20261003')))
        cls.f = o.load(cls.root / 'checker.py', 'opportunity_test_denominator')
        cls.v = o.load(cls.root / 'stream_reference.py', 'opportunity_test_stream')
        cls.modules = tuple(o.load(cls.root / n, 'opportunity_test_' + name) for n, name in
            [('admission_reference.py', 'admission'), ('replay.py', 'replay'), ('d0_reference.py', 'd0'), ('schedule_reference.py', 'schedule')])

    def evidence(self, case=1):
        cfg = self.f.cases()[case]['config']
        chain = self.v.inspect_segment(self.root / ('baseline_' + self.f.cases()[case]['name']), funded_config=cfg)
        return self.v.flatten(chain), cfg

    def test_exact_sign_tail_and_strict_eight_world_criterion(self):
        self.assertEqual(o.sign_tail(8, 7), Fraction(9, 256))
        self.assertEqual(o.sign_tail(8, 8), Fraction(1, 256))
        design = o.power_design()
        self.assertEqual(design['required_positives'], 8)
        self.assertAlmostEqual(design['scenarios'][3]['one_comparison_sign_count_probability'], .8 ** 8)
        self.assertFalse(design['empirical_ecological_power_estimated'])
        self.assertFalse(design['comparison_independence_assumed'])

    def test_joint_bound_does_not_assume_comparison_independence(self):
        for row in o.power_design()['scenarios']:
            p = row['one_comparison_sign_count_probability']
            self.assertLessEqual(row['two_comparison_joint_lower_bound'], p * p + 1e-15)
            self.assertGreaterEqual(row['two_comparison_joint_upper_bound'], p * p - 1e-15)

    def test_invalid_sign_domain(self):
        for n, k in [(0, 0), (8, 9), (True, 1), (8, -1)]:
            with self.assertRaises(ValueError):
                o.sign_tail(n, k)

    def test_truncated_and_duplicate_passes_rejected(self):
        evidence, cfg = self.evidence()
        for entries in [evidence['journal'][:-1], [evidence['journal'][0]] + evidence['journal'][:-1]]:
            bad = copy.deepcopy(evidence)
            bad['journal'] = entries
            with self.assertRaises(ValueError):
                o.LifetimeObserver(self.modules[1], self.modules[2], bad, cfg)

    def test_founder_contamination_and_double_charged_escrow_rejected(self):
        evidence, cfg = self.evidence()
        observer = o.LifetimeObserver(self.modules[1], self.modules[2], evidence, cfg)
        # A balanced zero-event tick isolates partition and root-balance failures.
        observer.evidence = copy.deepcopy(evidence)
        observer.evidence['rows'][0] = {k: 0 for k in ['development_paid', 'development_failures',
            'attempts', 'successes', 'failures', 'harvested', 'births', 'deaths', 'dissipated']}
        observer.reconcile_tick({}, 1)
        observer.members[1]['harvested'] = 8.0
        with self.assertRaisesRegex(ValueError, 'per-tick lifetime'):
            observer.reconcile_tick({}, 1)
        observer.members[1]['harvested'] = 0.0
        observer.members[1]['request_debits'] = 29.0
        with self.assertRaisesRegex(ValueError, 'organism root energy balance'):
            observer.reconcile_tick({}, 1)

    def test_wrong_full_denominator_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            o.write_json(p / 'registered_cases.json', self.f.cases()[:-1])
            o.write_json(p / 'registered_faults.json', self.f.FAULTS)
            with self.assertRaises(ValueError):
                self.f.denominator(p, self.f.cases())

    def test_seal_detects_changed_and_extra_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            (p / 'evidence').write_bytes(b'original')
            o.write_json(p / 'output_sha256.json', dict(**o.FLAGS, files_sha256={'evidence': o.digest(b'original')}))
            pin = o.digest((p / 'output_sha256.json').read_bytes())
            o.sealed_input(p, pin)
            (p / 'evidence').write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'inventory/hash'):
                o.sealed_input(p, pin)
            (p / 'evidence').write_bytes(b'original')
            (p / 'unregistered').write_bytes(b'extra')
            with self.assertRaisesRegex(ValueError, 'inventory/hash'):
                o.sealed_input(p, pin)

    def test_scientific_flag_forgery_rejected_even_with_rehashed_seal(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            bad = dict(o.FLAGS, evolution_enabled=True, files_sha256={})
            o.write_json(p / 'output_sha256.json', bad)
            with self.assertRaisesRegex(ValueError, 'research boundary'):
                o.sealed_input(p, o.digest((p / 'output_sha256.json').read_bytes()))

    def test_import_does_not_write_bytecode(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / 'reference.py'
            p.write_text('VALUE = 24\n')
            o.load(p, 'opportunity_no_bytecode')
            self.assertFalse((p.parent / '__pycache__').exists())

    def test_actual_b12_births_have_zero_descendant_yield_and_incomplete_lifetimes(self):
        case = self.f.cases()[6]
        result = o.analyze_cell(self.root, case['config'], self.root / ('baseline_' + case['name']), self.modules, self.v)['opportunity']
        self.assertEqual(result['descendant_denominator'], 8)
        self.assertEqual(result['descendants_completed'], 0)
        self.assertEqual(result['descendants_died_incomplete'], 8)
        self.assertEqual(result['primary_descendant_harvested'], 0)
        self.assertEqual(result['founder_harvested'], 192)
        self.assertEqual(result['total_harvested'], 192)
        self.assertEqual(result['totals']['payable_action_opportunities'], 24)

    def test_actual_b24_yield_excludes_founders_and_preserves_censored_lifetimes(self):
        case = self.f.cases()[1]
        result = o.analyze_cell(self.root, case['config'], self.root / ('baseline_' + case['name']), self.modules, self.v)['opportunity']
        self.assertEqual(result['descendant_denominator'], 16)
        self.assertEqual(result['descendants_live_at_64'], 3)
        self.assertEqual(result['founder_harvested'], 192)
        self.assertEqual(result['primary_descendant_harvested'], 1384)
        self.assertEqual(result['total_harvested'], 1576)
        self.assertEqual(result['totals']['eligible_parent_ticks'], 16)
        self.assertEqual(result['totals']['offered_targets'], 28)
        self.assertEqual(result['totals']['payable_action_opportunities'], 197)


if __name__ == '__main__':
    unittest.main()
