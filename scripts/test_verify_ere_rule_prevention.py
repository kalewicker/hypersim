"""Prospective construction/packaging and postflight tamper checks."""
import copy
import os
from pathlib import Path
import unittest
import verify_ere_rule_prevention as v

class PreventionTests(unittest.TestCase):
    def test_design_and_native_catalogue_construction(self):
        self.assertEqual(v.DESIGN['maximum_native_advances'],5*96)
        for arm in v.ARMS:
            raw,compiled=v.editor_catalogue(arm,[1,1]);self.assertEqual(len(compiled),9)
            self.assertTrue(all(c['meta_visible'] and c['pattern']['node_label_ceiling'] is None for c in compiled))
            self.assertEqual(raw[7]['name'],'prevention_damage');self.assertEqual(raw[7]['priority'],300)
            self.assertEqual(raw[8]['priority'],400)
            self.assertEqual(raw[8]['enabled'],arm!='disabled')
            self.assertTrue(all(r['enabled']==(arm!='no_hazard') for r in raw[3:7]))
            self.assertEqual(raw[8]['lhs']['nodes'],['a','b','root','token','threat','defense'])
            self.assertEqual(raw[8]['rhs']['edge_labels'],{'0':v.ROLE})

    def test_complete_import_closure(self):
        import run_ere_rule_prevention as run
        closure=run.dependencies()
        self.assertIn('scripts/verify_ere_complete_long_gates.py',closure)
        self.assertIn('scripts/verify_ere_complete_gates.py',closure)
        self.assertLessEqual(closure,set(run.names()))

    def output(self):
        value=os.environ.get('ERE_RULE_PREVENTION_AUDIT_INPUT')
        if not value:self.skipTest('postflight artifacts not requested')
        return Path(value)

    def test_all_native_control_ledgers(self):
        out=self.output();result=v.audit(out)
        self.assertEqual(result['native_advances'],480)
        self.assertIsInstance(result['mechanism_pass'],bool)
        paid=result['cells'][0];self.assertEqual(paid['measured']['genotype_spells'],[dict(start=5,end=None)])
        self.assertIsNone(paid['cohort']['genotype_first_half_life']['half_life'])
        self.assertFalse(paid['cohort']['reference_exposure_lifetime_ratio_pass'])
        self.assertTrue(paid['continuity']['class_lower_bounds_exceed16'])

    def test_payment_threat_and_counterfactual_tampering(self):
        source=v.a.load(self.output()/'native/cell-0000.json');attacks=[]
        def attack(name,mutator):
            r=copy.deepcopy(source);mutator(r);attacks.append((name,r))
        attack('missing horizon',lambda r:r.update(completed_steps=95))
        attack('fake free fuel',lambda r:r['timeline'][16]['state'].update(defense_fuel=4.))
        attack('erase pending threat',lambda r:r['timeline'][15]['state'].update(pending_threat=0))
        attack('no admitted damage binding',lambda r:next(x for x in r['timeline'][16]['usage']['rules'] if r['table_catalogue'][r['timeline'][16]['pre_step_table']][x['rule_id']]['name']=='prevention_damage').update(results=0))
        attack('priority bypass',lambda r:r['table_catalogue'][0][-1].update(priority=200))
        attack('global A2 exclusion',lambda r:r['table_catalogue'][0][-2]['pattern'].update(node_label_ceiling=v.META))
        attack('skip interception',lambda r:r['timeline'][16].update(events=[e for e in r['timeline'][16]['events'] if e['kind']!='absorb']))
        attack('zero payment primitive',lambda r:r['table_catalogue'][0][-1]['template']['property_effects'][-1]['op']['Subtract'].update(value={'Float':0.}))
        attack('wrong postbirth body',lambda r:r['timeline'][16].update(genotype=r['timeline'][0]['genotype']))
        attack('wrong installed attribution',lambda r:next(e for e in r['timeline'][17]['events'] if e['kind']=='world').update(rcf='00'))
        attack('non-distinct ordinary binding',lambda r:next(e for e in r['timeline'][17]['events'] if e['kind']=='world')['bound_nodes'].__setitem__(1,r['ordinary_nodes'][0]))
        attack('sustained claim inflation',lambda r:r.update(sustained_claim=True))
        for name,r in attacks:
            with self.subTest(name=name):
                with self.assertRaises((AssertionError,KeyError,IndexError,StopIteration)):v.audit_receipt(r)

    def test_no_threat_no_payment_and_exhaustion(self):
        out=self.output();no_hazard=v.audit_receipt(v.a.load(out/'native/cell-0004.json'))
        self.assertEqual(no_hazard['final_defense_fuel'],4.);self.assertEqual(no_hazard['issued_threats'],0)
        exhaustion=v.audit_receipt(v.a.load(out/'native/cell-0001.json'))
        self.assertEqual(exhaustion['absorbed_steps'],[16,32]);self.assertEqual(exhaustion['hazard_steps'],[48,64])
        self.assertEqual(exhaustion['measured']['genotype_first_duration']['observed_death_age'],43)
        self.assertEqual(exhaustion['measured']['installed_first_duration']['observed_death_age'],42)

if __name__=='__main__':unittest.main()
