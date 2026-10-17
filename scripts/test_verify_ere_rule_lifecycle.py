import copy
import os
from pathlib import Path
import unittest

import verify_ere_rule_lifecycle as v


class LifecycleAccountingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory=Path(os.environ['ERE_LIFECYCLE_AUDIT_INPUT'])
        cls.rows={arm:v.integrity.load(cls.directory/f'{v.SEEDS[0]}-{arm}.json') for arm in v.ARMS}

    def test_complete_native_panel(self):
        result=v.audit_panel(self.directory)
        self.assertTrue(result['accounting_pass'])
        self.assertEqual(result['native_steps_checked'],432)
        self.assertEqual(sum(x['non_meta_novel_fires'] for x in result['results']),54)

    def test_independent_canonical_identity_and_priority_exclusion(self):
        core=v.candidate(True,10);proposal=v.candidate(False,10);old=v.candidate(False,99)
        self.assertNotEqual(core['rcf'],proposal['rcf'])
        self.assertNotEqual(old['structural_rcf'],proposal['structural_rcf'])
        raw=dict(labels=[10,11,12],lhs=[[0,1],[0,2]],rhs=[[0,1],[1,2]],priority=1)
        self.assertEqual(v.encode(raw,(0,1,2),False).hex(),proposal['execution'])
        self.assertEqual(v.summary(self.rows['preseeded_lure']['initial_all_candidates'],
                                   self.rows['preseeded_lure']['timeline'])['novel_class_count'],0)

    def test_retirement_reappearance_and_censoring(self):
        c=self.rows['live_replacement']['summary']['novel_classes'][0]
        self.assertEqual(c['genotype_spells'],[dict(start=1,end=9),dict(start=17,end=None)])
        self.assertEqual(c['installed_spells'],[dict(start=2,end=10),dict(start=18,end=None)])
        self.assertIn(9,c['non_meta_fire_steps'])
        self.assertTrue(self.rows['live_paid']['summary']['replacement_interval_unmeasured'])

    def test_eighteen_detached_receipt_tampers_reject(self):
        def reject(arm,mutate):
            bad=copy.deepcopy(self.rows[arm]);mutate(bad)
            with self.assertRaises((AssertionError,KeyError,ValueError)):
                v.audit_receipt(bad)
        cases=[
            ('live_paid',lambda r:r.update(training_admitted=True)),
            ('live_paid',lambda r:r.update(scientific_acceptance=True)),
            ('live_paid',lambda r:r.update(seed=0)),
            ('live_paid',lambda r:r['timeline'].pop()),
            ('live_paid',lambda r:r['timeline'][1]['state'].update(fuel=1.)),
            ('live_paid',lambda r:r['timeline'][1].update(paid_edits=0)),
            ('live_paid',lambda r:r['timeline'][3]['usage'].update(truncated=True)),
            ('live_paid',lambda r:r['timeline'][3].update(rng_after=-1)),
            ('live_paid',lambda r:r['timeline'][2].update(installed=copy.deepcopy(r['timeline'][0]['installed']))),
            ('live_paid',lambda r:r['timeline'][3]['events'][0].update(rcf='fake')),
            ('live_paid',lambda r:r['timeline'][1]['genotype'][1].update(structural_rcf='fake')),
            ('live_paid',lambda r:r['timeline'][1]['events'][0]['edges_created'][0]['edge'].update(label=1)),
            ('live_paid',lambda r:r['seeded_editor_compiled'][1]['pattern']['edges'][0].update(ordered=False)),
            ('live_paid',lambda r:r['seeded_editor_compiled'][1]['template']['property_effects'][1]['op']['Subtract'].update(value={'Float':0.})),
            ('live_replacement',lambda r:r['summary']['novel_classes'][0]['genotype_spells'][0].update(end=24)),
            ('live_paid',lambda r:r['summary']['novel_classes'][0]['genotype_spells'][0].update(end=24)),
            ('frozen_paid',lambda r:r['summary']['novel_classes'][0].update(first_non_meta_fire=3)),
            ('preseeded_lure',lambda r:r['summary'].update(novel_class_count=1)),
        ]
        for i,(arm,mutate) in enumerate(cases):
            with self.subTest(case=i,arm=arm):reject(arm,mutate)


if __name__=='__main__':unittest.main()
