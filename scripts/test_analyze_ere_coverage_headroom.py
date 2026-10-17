import copy
import math
import unittest

import analyze_ere_coverage_headroom as review


class CoverageReviewTests(unittest.TestCase):
    def test_duplicate_not_independent_and_conflict_rejected(self):
        units = {}; key = (4, 11, 1, 0, 10)
        value = dict(utility=2., score=1., cap=False, reason=None, trace_sha256='a')
        review.add_observation(units, key, value, {'id': 'a'})
        review.add_observation(units, key, value, {'id': 'b'})
        self.assertEqual(len(units), 1)
        self.assertEqual(len(units[key]['receipts']), 2)
        bad = copy.deepcopy(value); bad['trace_sha256'] = 'changed'
        with self.assertRaisesRegex(AssertionError, 'conflicting'):
            review.add_observation(units, key, bad, {'id': 'c'})

    def test_missing_support_not_zero_filled_or_renormalized(self):
        keys = [(0, 0, 0, i) for i in range(3)]
        mass = dict(zip(keys, [.1, .4, .5]))
        leaves = {key: {'eligible': i != 2} for i, key in enumerate(keys)}
        means = {keys[0]: dict(utility=2., cap_rate=0.)}
        got = review.coverage_bounds(mass, means, leaves)
        self.assertAlmostEqual(got['observed_probability_mass'], .1)
        self.assertAlmostEqual(got['conditional_utility_lower'], .2)
        self.assertAlmostEqual(got['conditional_utility_upper'], 1.5)
        self.assertAlmostEqual(got['conditional_cap_upper'], .9)
        self.assertFalse(got['population_confidence_bound'])
        self.assertFalse(got['observed_support_renormalized'])

    def test_empty_and_complete_coverage(self):
        key = (0, 0, 0, 0); leaves = {key: dict(eligible=True)}
        got = review.coverage_bounds({key: 1.}, {}, leaves)
        self.assertEqual(got['conditional_utility_upper'], 2.)
        got = review.coverage_bounds({key: 1.}, {key: dict(utility=0., cap_rate=1.)}, leaves)
        self.assertEqual(got['conditional_cap_lower'], got['conditional_cap_upper'])
        self.assertEqual(got['conditional_utility_lower'], got['conditional_utility_upper'])

    def test_variance_separates_physics_from_trace(self):
        key = (0, 0, 0, 0)
        units = {key+(p,): {'outcome': dict(utility=1., score=0., cap=False,
                                          reason=None, trace_sha256=str(p))} for p in range(4)}
        got = review.variance_summary(units)
        self.assertEqual(got['leaves_with_multiple_physics_seeds'], 1)
        self.assertEqual(got['utility_varying_leaves'], 0)
        self.assertEqual(got['trace_varying_leaves'], 1)
        self.assertEqual(got['maximum_sample_utility_variance'], 0.)
        units[key+(3,)]['outcome'].update(utility=0., score=None, cap=True, reason='resource_cap')
        got = review.variance_summary(units)
        self.assertEqual(got['cap_varying_leaves'], 1)
        self.assertAlmostEqual(got['maximum_sample_utility_variance'], .25)

    def test_incomplete_groups_not_given_uniform_means(self):
        group = (0, 0, 0); groups = {group: dict(count=2)}
        means = {group+(0,): dict(utility=2., cap_rate=0.)}
        got = review.group_summary(means, groups)[0]
        self.assertFalse(got['complete'])
        self.assertIsNone(got['complete_empirical_uniform_utility'])
        self.assertIsNone(got['uniform_production_penalty'])
        means[group+(1,)] = dict(utility=0., cap_rate=1.)
        got = review.group_summary(means, groups)[0]
        self.assertTrue(got['complete'])
        self.assertEqual(got['uniform_production_penalty'], 1.)
        self.assertEqual(got['observed_cap_range'], 1.)

    def test_utility_refuses_unknown_and_keeps_caps(self):
        row = {'evaluation': dict(score=None, reason='resource_cap', proposal_fires=0, steps=[]),
               'constraint_utility': 0.}
        self.assertEqual(review.outcome(row)['utility'], 0.)
        row['evaluation']['reason'] = 'unknown'
        with self.assertRaises(AssertionError):
            review.outcome(row)

    def test_actual_grammar_probability_and_known_control_support(self):
        catalogue = review.a.load(review.ROOT/'output'/review.INPUTS[0][0]/'catalogue.json')
        leaves, groups, counts = review.mapping.catalogue_model(catalogue)
        self.assertEqual(counts['leaves'], 18956)
        self.assertEqual(groups[review.TARGET[:3]]['count'], 1)
        self.assertTrue(leaves[review.TARGET]['eligible'])
        for label, theta in review.policies():
            prob = review.mapping.probabilities(catalogue, theta, groups)
            self.assertTrue(math.isclose(math.fsum(prob['group_mass'].values()), 1., abs_tol=1e-12))
            if label == 'positive_control_selector_pm8':
                self.assertGreater(prob['group_mass'][review.TARGET[:3]], .9999)


if __name__ == '__main__':
    unittest.main()
