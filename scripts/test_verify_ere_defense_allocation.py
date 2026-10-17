"""Construction and archived receipt tamper tests; no unregistered native work."""
import copy
import os
from pathlib import Path
import unittest
import verify_ere_defense_allocation as v
class AllocationTests(unittest.TestCase):
    def output(self):
        path=os.environ.get('ERE_DEFENSE_ALLOCATION_AUDIT_INPUT')
        if not path:self.skipTest('postflight artifacts not requested')
        return Path(path)
    def test_design_and_native_catalogues(self):
        self.assertEqual(v.DESIGN['maximum_native_advances'],16*32)
        for action in v.ACTIONS:
            raw,compiled=v.editor_catalogue(action);self.assertEqual(len(raw),9)
            self.assertTrue(all(r['meta_visible'] and r['pattern']['node_label_ceiling'] is None for r in compiled))
            self.assertEqual([r['enabled'] for r in raw[-3:]],[action=='defend',action=='harvest',action=='harvest'])
            self.assertEqual([r['priority'] for r in raw[-3:]],[500]*3)
    def test_import_closure(self):
        import run_ere_defense_allocation as r
        closure=r.previous.primitive.dependencies('scripts/verify_ere_defense_allocation.py')
        self.assertLessEqual(closure,set(r.names()))
        self.assertIn('scripts/verify_ere_complete_gates.py',closure)
    def test_all_cells_and_censoring(self):
        result=v.audit(self.output());self.assertEqual(len(result['cells']),16)
        self.assertIsInstance(result['diagnostic_pass'],bool)
        for c in result['cells']:
            d=c['measured']['genotype_first_duration'];self.assertIsNotNone(d)
            if c['hazard_events']:self.assertEqual(d['observed_death_age'],11)
            else:self.assertEqual(d,dict(lower_bound=27,observed_death_age=None,right_censored=True))
        self.assertFalse(result['training_admitted']);self.assertFalse(result['scientific_acceptance'])
        self.assertEqual(len(result['task_observations']),8)
        self.assertIsInstance(result['existing_encoder_observation_alias'],bool)
    def test_no_free_value_unpaid(self):
        out=self.output()
        for i in [2,3,6,7,10,11,14,15]:
            c=v.audit_receipt(v.a.load(out/'native'/f'cell-{i:04}.json'))
            self.assertEqual(c['allocation_payments'],0);self.assertEqual(c['harvested_credits'],0)
            self.assertEqual(c['absorbed_threats'],0);self.assertEqual(c['final_wallet_units'],0.)
    def test_resource_observation_and_native_tampering(self):
        out=self.output();source=v.a.load(out/'native/cell-0000.json');attacks=[]
        def attack(name,fn):r=copy.deepcopy(source);fn(r);attacks.append((name,r))
        attack('short horizon',lambda r:r.update(completed_steps=31))
        attack('free wallet refill',lambda r:r['timeline'][14]['state'].update(wallet_units=1.))
        attack('escrow double spend',lambda r:r['timeline'][16]['state'].update(defense_fuel=1.))
        attack('free credits',lambda r:r['timeline'][14]['state'].update(harvested_credits=12))
        attack('forecast removed',lambda r:r['timeline'][0]['state'].update(forecast=0))
        attack('bounty relabeled',lambda r:r['timeline'][14]['state'].update(bounty=12))
        attack('decision after allocation',lambda r:r['allocation_observation'].update(step=14))
        attack('action leaked observation',lambda r:r['allocation_observation'].update(action='defend'))
        attack('damage inadmissible',lambda r:next(x for x in r['timeline'][16]['usage']['rules'] if r['table_catalogue'][r['timeline'][16]['pre_step_table']][x['rule_id']]['name']=='prevention_damage').update(results=0))
        attack('blanket META ceiling',lambda r:r['table_catalogue'][0][-5]['pattern'].update(node_label_ceiling=v.META))
        attack('remove payment',lambda r:r['timeline'][14].update(events=[e for e in r['timeline'][14]['events'] if e['kind']!='allocate_defend']))
        attack('wrong attribution',lambda r:next(e for e in r['timeline'][17]['events'] if e['kind']=='world').update(rcf='00'))
        attack('sustained inflation',lambda r:r.update(sustained_claim=True))
        for name,r in attacks:
            with self.subTest(name=name):
                with self.assertRaises((AssertionError,KeyError,IndexError,StopIteration)):v.audit_receipt(r)
if __name__=='__main__':unittest.main()
