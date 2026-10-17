import copy
import os
from pathlib import Path
import unittest
import verify_ere_rule_choice_learning as v


class LearningTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory=Path(os.environ['ERE_RULE_CHOICE_LEARNING_AUDIT_INPUT'])
        cls.transitions=v.a.load(cls.directory/'transitions.json')
        cls.training=[]
        for seed_index in range(3):
            for i in range(seed_index*180+12,seed_index*180+156):
                r=v.a.load(cls.directory/f'cell-{i:04}.json')
                r['reward']=float(r.pop('native')['adapter_reward']);cls.training.append(r)

    def test_full_accounting_and_fixed_criterion(self):
        result=v.audit(self.directory)
        self.assertTrue(result['accounting_pass'])
        self.assertEqual(result['native_steps_checked'],12960)
        self.assertEqual(result['completed_updates'],72)
        # Do not assert a favorable endpoint: negative results remain complete.
        self.assertIsInstance(result['learning_criterion_pass'],bool)

    def test_fourteen_detached_update_sampling_tampers(self):
        mutations=[
            lambda t,r:t.pop(),
            lambda t,r:r.pop(),
            lambda t,r:t[0].update(skipped=True),
            lambda t,r:t[0].update(iteration=1),
            lambda t,r:t[0].update(first_cell=13),
            lambda t,r:t[0]['before_live'].__setitem__(0,1.),
            lambda t,r:t[0]['after_live'].__setitem__(0,1.),
            lambda t,r:t[0]['after_reuse'].__setitem__(0,1.),
            lambda t,r:r[0].update(draw=0.0),
            lambda t,r:r[0].update(draw_state=0),
            lambda t,r:r[0].update(action=(r[0]['action']+1)%4),
            lambda t,r:r[0]['score_gradient'].__setitem__(0,1.),
            lambda t,r:r[0].update(context=20),
            lambda t,r:r[1].update(reward=11/24),
        ]
        self.assertEqual(len(mutations),14)
        for i,mutate in enumerate(mutations):
            t=copy.deepcopy(self.transitions);r=copy.deepcopy(self.training);mutate(t,r)
            with self.subTest(case=i),self.assertRaises((AssertionError,ValueError,KeyError,IndexError)):
                v.verify_transitions(t,r)

    def test_six_detached_native_tampers(self):
        first=v.a.load(self.directory/'cell-0001.json')['native']
        reused=v.a.load(self.directory/'cell-0013.json')['native']
        mutations=[(first,lambda r:r.update(adapter_reward=1.)),
            (first,lambda r:r['timeline'].pop()),
            (first,lambda r:r['timeline'][3]['usage'].update(truncated=True)),
            (first,lambda r:r['timeline'][1]['state'].update(fuel=1.)),
            (reused,lambda r:r.update(adapter_reward=11/24)),
            (reused,lambda r:r['summary'].update(novel_class_count=1))]
        for row,mutate in mutations:
            bad=copy.deepcopy(row);mutate(bad)
            with self.assertRaises((AssertionError,ValueError,KeyError)):
                v.native.audit_receipt(bad)

    def test_independent_draws_zero_control_and_endpoint_math(self):
        policies=v.verify_transitions(self.transitions,self.training)
        for seed,_,reuse in policies:self.assertEqual(reuse,[0.]*12)
        endpoint=v.endpoint([0.]*12,[0.,11/24,0.,0.,0.,0.,11/24,0.,0.,0.,0.,11/24])
        self.assertAlmostEqual(endpoint['mean_expected_reward'],11/96)
        # Endpoint/criterion corruption rejects through the same strict comparator.
        bad=copy.deepcopy(endpoint);bad['mean_expected_reward']=0.5
        with self.assertRaises(AssertionError):v.close(bad,endpoint)
        bad=copy.deepcopy(endpoint);bad['contexts'][0]['matching_edit_probability']=1.
        with self.assertRaises(AssertionError):v.close(bad,endpoint)


if __name__=='__main__':unittest.main()
