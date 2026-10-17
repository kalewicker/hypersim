import copy
import os
from pathlib import Path
import unittest
import verify_ere_rule_composition_v2 as v


class CompositionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory=Path(os.environ['ERE_RULE_COMPOSITION_AUDIT_INPUT'])
        cls.rows={arm:v.integrity.load(cls.directory/v.filename(v.SEEDS[0],[10,11],arm,[1,1])) for arm in v.ARMS}

    def test_full_matrix(self):
        r=v.audit_panel(self.directory)
        self.assertEqual((r['cells'],r['native_steps_checked'],r['novel_fires']),(396,9504,324))
        self.assertEqual(sum(x['paid_edits'] for x in r['results']),522)
        self.assertEqual(sum(x['novel_classes'] for x in r['results']),522)

    def test_partial_classes_retire_and_need_two_payments(self):
        for actions in [[0,0],[1,0],[0,1],[1,2],[2,1]]:
            r=v.integrity.load(self.directory/v.filename(v.SEEDS[0],[10,11],'live_paid',actions))
            self.assertEqual(v.audit_receipt(r)['reward'],0.)
        for arm in ['live_paid','reverse_paid']:
            r=self.rows[arm];v.audit_receipt(r)
            partial=next(c for c in r['summary']['novel_classes'] if c['first_birth']==1)
            complete=next(c for c in r['summary']['novel_classes'] if c['first_birth']==5)
            self.assertEqual(partial['genotype_spells'],[dict(start=1,end=5)])
            self.assertEqual(partial['installed_spells'],[dict(start=2,end=6)])
            self.assertEqual(partial['non_meta_fire_steps'],[])
            self.assertEqual(complete['non_meta_fire_steps'],list(range(7,24,2)))
            self.assertIsNone(complete['genotype_spells'][0]['end'])
        self.assertEqual(v.audit_receipt(self.rows['one_fuel'])['paid_edits'],1)
        self.assertEqual(v.audit_receipt(self.rows['preseeded_lure'])['world_fires'],24)

    def test_slot_positions_are_not_global_variable_indices(self):
        r=self.rows['live_paid'];v.audit_receipt(r)
        edges=dict(r['initial_inventory']['edges'])
        def link(role,start,slot):
            es=[e for e in edges.values() if e['label']==role and v.node_key(e['nodes'][0])==start]
            self.assertEqual(len(es),1)
            return v.node_key(es[0]['nodes'][slot])
        root=v.node_key(r['root']);allele=link(v.META+0x43,root,1)
        left=link(v.META+0x40,allele,1)
        patterns=[v.node_key(e['nodes'][1]) for e in edges.values()
                  if e['label']==v.META+0x41 and v.node_key(e['nodes'][0])==left]
        endpoints={link(v.META+0x51,p,1) for p in patterns}
        self.assertEqual(len(endpoints),2)
        literal_nodes={link(v.META+0x70,var,1) for var in endpoints}
        self.assertIn(v.node_key(r['target_literals'][1]),literal_nodes)
        self.assertNotIn(v.node_key(r['target_literals'][0]),literal_nodes)

    def test_twenty_four_detached_native_tampers(self):
        cases=[
            ('live_paid',lambda r:r.update(training_admitted=True)),
            ('live_paid',lambda r:r.update(scientific_acceptance=True)),
            ('live_paid',lambda r:r.update(seed=0)),
            ('live_paid',lambda r:r.update(context=[10,21])),
            ('live_paid',lambda r:r.update(actions=[1,2])),
            ('live_paid',lambda r:r.update(actions=[True,1])),
            ('live_paid',lambda r:r.update(adapter_reward=1.)),
            ('live_paid',lambda r:r['timeline'].pop()),
            ('live_paid',lambda r:r['target_literals'].reverse()),
            ('live_paid',lambda r:r['timeline'][1]['state'].update(fuel=2.)),
            ('live_paid',lambda r:r['timeline'][5].update(paid_edits=0)),
            ('live_paid',lambda r:r['timeline'][7]['usage'].update(truncated=True)),
            ('live_paid',lambda r:r['timeline'][7].update(rng_after=-1)),
            ('live_paid',lambda r:r['timeline'][6].update(installed=copy.deepcopy(r['timeline'][2]['installed']))),
            ('live_paid',lambda r:r['timeline'][7]['events'][0].update(rcf='fake')),
            ('live_paid',lambda r:r['timeline'][1]['genotype'][1].update(structural_rcf='fake')),
            ('live_paid',lambda r:r['seeded_editor_rules'][2]['lhs_node_property_guards'][1].update(value=1)),
            ('live_paid',lambda r:r['seeded_editor_compiled'][2]['pattern']['edges'][0].update(ordered=False)),
            ('live_paid',lambda r:r['seeded_editor_compiled'][2]['template']['property_effects'][1]['op']['Subtract'].update(value={'Float':0.})),
            ('live_paid',lambda r:next(c for c in r['summary']['novel_classes'] if c['first_birth']==1).update(non_meta_fire_steps=[3])),
            ('live_paid',lambda r:next(c for c in r['summary']['novel_classes'] if c['first_birth']==5).update(first_non_meta_fire=5)),
            ('one_fuel',lambda r:r.update(adapter_reward=9/24)),
            ('frozen_paid',lambda r:r.update(adapter_reward=9/24)),
            ('preseeded_lure',lambda r:r['summary'].update(novel_class_count=1)),
        ]
        self.assertEqual(len(cases),24)
        for i,(arm,mutate) in enumerate(cases):
            bad=copy.deepcopy(self.rows[arm]);mutate(bad)
            with self.subTest(case=i),self.assertRaises((AssertionError,ValueError,KeyError,TypeError)):
                v.audit_receipt(bad)

    def test_shared_policy_and_four_tampers(self):
        policy=v.integrity.load(self.directory/'policy.json');v.audit_policy(policy)
        mutations=[lambda r:r.update(training_updates=1),
            lambda r:r['contexts'][0]['probabilities'][1].__setitem__(0,1.),
            lambda r:r['contexts'][0]['actions'][5].update(log_probability=0.),
            lambda r:r['contexts'][0]['actions'][5]['score_gradient'].__setitem__(0,1.)]
        for mutate in mutations:
            bad=copy.deepcopy(policy);mutate(bad)
            with self.assertRaises(AssertionError):v.audit_policy(bad)


if __name__=='__main__':unittest.main()
