import copy
import os
from pathlib import Path
import unittest
import verify_ere_rule_choice as v


class ChoiceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory=Path(os.environ['ERE_RULE_CHOICE_AUDIT_INPUT'])
        cls.rows={arm:v.integrity.load(cls.directory/v.filename(v.SEEDS[0],10,arm,1)) for arm in v.ARMS}

    def test_full_panel(self):
        r=v.audit_panel(self.directory)
        self.assertEqual((r['cells'],r['native_steps_checked'],r['novel_fires']),(48,1152,66))
        self.assertEqual(sum(x['paid_edits'] for x in r['results']),24)
        self.assertEqual(sum(x['novel_classes'] for x in r['results']),24)

    def test_all_contexts_nonmatching_births_and_controls(self):
        for seed,context,arm,action in v.schedule():
            r=v.integrity.load(self.directory/v.filename(seed,context,arm,action));result=v.audit_receipt(r)
            self.assertEqual(result['reward'],11/24 if arm=='live_paid' and v.ACTIONS[action]==context else 0.)
            if arm=='live_paid' and action and v.ACTIONS[action]!=context:
                self.assertEqual(result['novel_classes'],1)
                self.assertEqual(result['non_meta_novel_fires'],0)

    def test_twenty_four_detached_native_tampers(self):
        cases=[
            ('live_paid',lambda r:r.update(training_admitted=True)),
            ('live_paid',lambda r:r.update(scientific_acceptance=True)),
            ('live_paid',lambda r:r.update(seed=0)),
            ('live_paid',lambda r:r.update(context=20)),
            ('live_paid',lambda r:r.update(action=2)),
            ('live_paid',lambda r:r.update(action=True)),
            ('live_paid',lambda r:r.update(adapter_reward=1.)),
            ('live_paid',lambda r:r['timeline'].pop()),
            ('live_paid',lambda r:r['timeline'][1]['state'].update(fuel=1.)),
            ('live_paid',lambda r:r['timeline'][1].update(paid_edits=0)),
            ('live_paid',lambda r:r['timeline'][3]['usage'].update(truncated=True)),
            ('live_paid',lambda r:r['timeline'][3].update(rng_after=-1)),
            ('live_paid',lambda r:r['timeline'][2].update(installed=copy.deepcopy(r['timeline'][0]['installed']))),
            ('live_paid',lambda r:r['timeline'][3]['events'][0].update(rcf='fake')),
            ('live_paid',lambda r:r['timeline'][1]['genotype'][1].update(structural_rcf='fake')),
            ('live_paid',lambda r:r['initial_all_candidates'].pop()),
            ('live_paid',lambda r:r['timeline'][1]['events'][0]['edges_created'][0]['edge'].update(label=1)),
            ('live_paid',lambda r:r['seeded_editor_compiled'][1]['pattern']['edges'][0].update(ordered=False)),
            ('live_paid',lambda r:r['seeded_editor_compiled'][1]['template']['property_effects'][1]['op']['Subtract'].update(value={'Float':0.})),
            ('live_paid',lambda r:r['summary']['novel_classes'][0]['genotype_spells'][0].update(end=24)),
            ('live_paid',lambda r:r['summary']['novel_classes'][0].update(first_birth=3)),
            ('frozen_paid',lambda r:r['summary']['novel_classes'][0].update(first_non_meta_fire=3)),
            ('preseeded_lure',lambda r:r['summary'].update(novel_class_count=1)),
            ('preseeded_lure',lambda r:r.update(adapter_reward=0.5)),
        ]
        self.assertEqual(len(cases),24)
        for i,(arm,mutate) in enumerate(cases):
            bad=copy.deepcopy(self.rows[arm]);mutate(bad)
            with self.subTest(case=i),self.assertRaises((AssertionError,KeyError,ValueError,TypeError)):
                v.audit_receipt(bad)

    def test_policy_arithmetic_and_four_tampers(self):
        policy=v.integrity.load(self.directory/'policy.json');v.audit_policy(policy)
        mutations=[lambda r:r.update(training_updates=1),
            lambda r:r['contexts'][0]['probabilities'].__setitem__(0,1.),
            lambda r:r['contexts'][0]['actions'][1].update(log_probability=0.),
            lambda r:r['contexts'][0]['actions'][1]['score_gradient'].__setitem__(0,1.)]
        for mutate in mutations:
            bad=copy.deepcopy(policy);mutate(bad)
            with self.assertRaises(AssertionError):v.audit_policy(bad)


if __name__=='__main__':unittest.main()
