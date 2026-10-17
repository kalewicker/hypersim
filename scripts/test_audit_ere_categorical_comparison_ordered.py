import copy
import json
import math
from pathlib import Path
import struct
import unittest

import audit_ere_categorical_comparison_ordered as s

SEMANTIC_REJECTIONS = []


def fixture(r=1, ctx=0, e=1):
    rec = s.a.load(s.INPUT / f'estimates/r{r}-c{ctx}-categorical-e{e}.json')
    schedule = s.a.load(s.INPUT / 'schedule.json')
    qs = [q for q in schedule if q['stage'] == 'estimation' and q['block'] == r
          and q['context'] == ctx and q['method'] == 'categorical' and q['estimate'] == e]
    rows = {q['id']: s.a.load(s.INPUT / 'cells' / (q['id']+'.json')) for q in qs}
    batch = dict(theta=s.original.p.contexts()[ctx], expected_draws=64, alpha=s.cat.f32(.05),
                 samples=[dict(id=q['id'], emission_seed=q['emission_seed'], physics_seed=q['physics_seed'],
                               selection=s.cat.selection(rows[q['id']]['emission']),
                               native_score=rows[q['id']]['evaluation']['score'],
                               reason=rows[q['id']]['evaluation']['reason']) for q in qs])
    return rec, qs, rows, batch


class OrderedTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalogue = s.a.load(s.INPUT / 'reference_catalogue.json')
        cls.rec, cls.qs, cls.rows, cls.batch = fixture()

    @classmethod
    def tearDownClass(cls):
        assert len(SEMANTIC_REJECTIONS) == len(set(SEMANTIC_REJECTIONS)) == 15
        print(json.dumps(dict(detached_helper_semantic_rejections=SEMANTIC_REJECTIONS)))

    def test_exact_three_sum_transform_and_no_source_edits(self):
        self.assertEqual(len(s.PROOF['substitutions']), 3)
        self.assertTrue(s.PROOF['ast_identical_except_three_sum_callees'])
        self.assertEqual(s.digest(Path(s.cat.__file__)), s.ADAPTER_SHA)
        with self.assertRaisesRegex(AssertionError, 'pinned adapter'):
            s.transform(Path(s.cat.__file__).read_bytes()+b'\n# drift\n')

    def test_cancellation_is_ordered_not_fsum(self):
        self.assertEqual(s.ordered_sum([1e16, 1., -1e16]), 0.)
        self.assertEqual(math.fsum([1e16, 1., -1e16]), 1.)
        self.assertEqual(s.ordered_sum(iter([0., .5, 1.5])), 2.)

    def test_saved_failure_reproduced_and_exactly_corrected(self):
        raw = self.rec['raw']
        old = s.ORIGINAL_ESTIMATE(self.catalogue, self.batch)
        with self.assertRaisesRegex(AssertionError, 'f32 vector bits'):
            s.cat.bits(raw['theta_after'], old['theta_after'])
        s.cat.check_batch(raw, self.catalogue, self.batch)

    def test_all_32_saved_64_emission_categorical_updates(self):
        for r in range(8):
            for ctx in range(2):
                for e in range(2):
                    rec, _, _, batch = fixture(r, ctx, e)
                    s.cat.check_batch(rec['raw'], self.catalogue, batch)

    def test_mixed_cap_and_equal_utility_complete_denominators(self):
        rec, _, _, batch = fixture(0, 0, 0)
        self.assertEqual(sum(q['native_score'] is None for q in batch['samples']), 1)
        s.cat.check_batch(rec['raw'], self.catalogue, batch)
        for sample in batch['samples']:
            sample.update(native_score=0., reason=None)
        raw = s.cat.estimate(self.catalogue, batch)
        self.assertEqual(raw['gradient'], [0.]*183)
        s.cat.bits(raw['theta_before'], raw['theta_after'])
        for sample in batch['samples']:
            sample.update(native_score=None, reason='resource_cap')
        raw = s.cat.estimate(self.catalogue, batch)
        self.assertEqual(raw['utility_mean'], 0.)
        self.assertEqual(len(raw['contributions']), 64)

    def test_signed_zero_and_single_ulp_still_reject(self):
        with self.assertRaisesRegex(AssertionError, 'f32 vector bits'):
            s.cat.bits([0.], [-0.])
        value = self.rec['raw']['theta_after'][0]
        bits = struct.unpack('<I', struct.pack('<f', value))[0]
        next_value = struct.unpack('<f', struct.pack('<I', bits+1))[0]
        raw = copy.deepcopy(self.rec['raw']); raw['theta_after'][0] = next_value
        with self.assertRaises(AssertionError):
            s.cat.check_batch(raw, self.catalogue, self.batch)
        SEMANTIC_REJECTIONS.append('one_ulp_update')

    def test_detached_raw_update_semantic_tampers(self):
        changes = [
            lambda x:x.update(alpha=.1),
            lambda x:x.update(expected_draws=63),
            lambda x:x.update(utility_mean=2.),
            lambda x:x['gradient'].__setitem__(0, 1.),
            lambda x:x['contributions'][0].update(baseline=9.),
            lambda x:x['contributions'][0].update(advantage=9.),
            lambda x:x['contributions'][0]['score']['gradient'].__setitem__(0, 1.),
            lambda x:x['contributions'][0]['sample'].update(native_score=None, reason='resource_cap'),
        ]
        s.cat.check_batch(self.rec['raw'], self.catalogue, self.batch)
        for index, change in enumerate(changes):
            raw = copy.deepcopy(self.rec['raw']); change(raw)
            with self.assertRaises((AssertionError, ValueError, IndexError, KeyError)):
                s.cat.check_batch(raw, self.catalogue, self.batch)
            SEMANTIC_REJECTIONS.append('raw_update_'+str(index))

    def test_matched_proposal_and_leaf_equality_controls(self):
        s.original.bounded.legacy.matched(self.rec['matched'], self.batch['theta'], self.rec['raw']['theta_after'])
        changed = copy.deepcopy(self.rec['matched']); changed['forward'][0] = .11
        with self.assertRaises(AssertionError):
            s.original.bounded.legacy.matched(changed, self.batch['theta'], self.rec['raw']['theta_after'])
        SEMANTIC_REJECTIONS.append('matched_forward')
        dev = s.original.bridge.DEV
        qs = [q for q in s.a.load(dev/'schedule.json') if q['stage']=='validation' and q['context']==0]
        rows = {q['id']:s.a.load(dev/'cells'/(q['id']+'.json')) for q in qs}
        rec = s.a.load(dev/'validation/r0-c0.json'); vectors = s.a.load(dev/'directions/r0-c0.json')['vectors']
        s.original.validation(rec, 0, 0, qs, rows, self.catalogue, vectors, 8)
        for field in ('utility_effects', 'validity_effects'):
            bad = copy.deepcopy(rec); bad[field][3] += .1
            with self.assertRaises(AssertionError):
                s.original.validation(bad, 0, 0, qs, rows, self.catalogue, vectors, 8)
            SEMANTIC_REJECTIONS.append(field)
        bad = copy.deepcopy(rec); bad['terms'][0]['weights'][3]['weight'] += .1
        with self.assertRaises(AssertionError):
            s.original.validation(bad, 0, 0, qs, rows, self.catalogue, vectors, 8)
        SEMANTIC_REJECTIONS.append('importance_weight')
        rows[qs[1]['id']]['emission']['extra_marker'] = True
        with self.assertRaisesRegex(AssertionError, 'physics repeats changed emission'):
            s.original.validation(rec, 0, 0, qs, rows, self.catalogue, vectors, 8)
        SEMANTIC_REJECTIONS.append('full_emission_repeat')

    def test_original_denominators_sign_rule_and_validity_guard(self):
        summary = s.a.load(s.INPUT/'summary.json')
        self.assertFalse(summary['support_categorical_direction'])
        self.assertEqual(summary['fixed_denominator'], 8)
        for value in summary['confirmatory'].values():
            self.assertEqual(value['sign_p'], 1/256)
        self.assertLess(summary['categorical_weighted_validity_effect'], -1e-12)
        align = [[[.5, .5] for _ in range(2)] for _ in range(8)]
        effects = [[[[0., 0.], [.01, .01]] for _ in range(2)] for _ in range(8)]
        validity = [[[[0., 0.], [0., 0.]] for _ in range(2)] for _ in range(8)]
        good = s.original.p.statistics(s.original.config(), 'scientific', align, effects, validity, True)
        self.assertTrue(good['support_categorical_direction'])
        validity[0][0][1][0] = -1e-5
        bad_validity = s.original.p.statistics(s.original.config(), 'scientific', align, effects, validity, True)
        self.assertFalse(bad_validity['support_categorical_direction'])
        bad = copy.deepcopy(summary); bad['support_categorical_direction'] = True
        with self.assertRaises(AssertionError):
            s.original.numeric(bad, summary)
        SEMANTIC_REJECTIONS.append('forged_support')


if __name__ == '__main__':
    unittest.main()
