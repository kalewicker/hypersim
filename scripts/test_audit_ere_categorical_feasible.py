import copy
import json
import os
from pathlib import Path
import unittest
import audit_ere_categorical_feasible as v


class FeasibleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.out=Path(os.environ['ERE_FEASIBLE_ENGINEERING_ARCHIVE'])
        cls.request=v.a.load(cls.out/'request.json');cls.result=v.a.load(cls.out/'result.json')
        cls.catalogue=v.a.load(cls.out/'catalogue.json')

    def test_independent_complete_math_and_exact_proposal_bits(self):
        self.assertTrue(v.check_result(self.request,self.result,self.catalogue)['engineering_arithmetic_pass'])

    def test_informative_panel_produces_nonzero_candidate(self):
        active=[q['proposal'] for q in self.result['cases'] if q['proposal']['selected_candidate'] is not None]
        self.assertTrue(active,'engineering fixture must exercise selection, not only holds')
        for q in active:
            selected=q['candidates'][q['selected_candidate']]
            self.assertLessEqual(selected['observed_cap_risk_effect'],0.)
            self.assertGreater(selected['utility_linear_gain'],0.)
            self.assertEqual(len(q['candidates']),9)

    def test_no_caps_and_all_caps_hold_base_without_replacement(self):
        for case,got in zip(self.request['cases'],self.result['cases']):
            if case['id'].endswith('all_valid') or case['id'].endswith('all_cap'):
                self.assertIsNone(got['proposal']['selected_candidate'])
                v.cat.bits(got['proposal']['theta_after'],case['theta'])
                self.assertEqual(got['proposal']['candidates'],[])
                self.assertEqual(len(got['proposal']['risk_input_validation']['contributions']),512)

    def test_geometry_conflict_zero_and_inward_margin(self):
        self.assertIsNone(v.cone([-1.,0.],[1.,0.]))
        self.assertIsNone(v.cone([0.,0.],[1.,0.]))
        d=v.cone([0.,1.],[1.,0.]);self.assertAlmostEqual(d[0],.1)
        self.assertAlmostEqual(v.norm(d),1.)
        self.assertEqual(v.ordered([1e16,1.,-1e16]),0.)

    def test_detached_result_semantic_controls(self):
        changes=[
            lambda x:x.update(training_admitted=True),
            lambda x:x['cases'][0]['proposal'].update(validation_used=True),
            lambda x:x['cases'][0]['proposal'].update(unseen_cap_feasibility_certificate=True),
            lambda x:x['cases'][0]['proposal'].update(risk_cap_count=999),
            lambda x:x['cases'][0]['proposal']['centered_validity_gradient'].__setitem__(0,1.),
            lambda x:x['cases'][0]['proposal'].update(cone_margin=.01),
            lambda x:x['cases'][0]['proposal'].update(selected_candidate=999),
            lambda x:x['cases'][0]['proposal']['theta_after'].__setitem__(0,99.),
            lambda x:x['cases'][0]['proposal']['risk_fixed_baselines'].__setitem__(0,99.),
            lambda x:x['cases'][0]['proposal'].update(support_categorical_direction=True),
        ]
        v.check_result(self.request,self.result,self.catalogue)
        for change in changes:
            bad=copy.deepcopy(self.result);change(bad)
            with self.assertRaises((AssertionError,IndexError,KeyError)):
                v.check_result(self.request,bad,self.catalogue)
        print(json.dumps({'detached_result_controls':len(changes)}))


if __name__=='__main__':unittest.main()
