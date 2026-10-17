"""Prelaunch refusal tests plus artifact-backed native ledger tampering."""
import copy
import os
from pathlib import Path
import unittest
import verify_ere_rule_maintenance as v

class MaintenanceAuditTests(unittest.TestCase):
    def test_fixed_design_and_catalogues(self):
        self.assertEqual(v.DESIGN['cells']*v.DESIGN['horizon'],6048)
        self.assertEqual(v.DESIGN['target_role'],0xfff00050)
        self.assertEqual(len(v.operation_catalogue('Seal')),14)
        self.assertEqual(len(v.operation_catalogue('RepairSealed')),14)
        raw,compiled=v.editor_catalogue('learned_protected',[1,1])
        self.assertEqual(len(raw),7)
        self.assertTrue(all(c['pattern']['node_label_ceiling']==v.META and not c['meta_visible'] for c in compiled[3:]))
        self.assertEqual(len(v.editor_catalogue('learned_no_hazard',[0,0])[0]),3)

    def fixtures(self):
        path=os.environ.get('ERE_RULE_MAINTENANCE_PREFLIGHT')
        if not path:self.skipTest('artifact-backed fixtures not requested')
        return {arm:v.a.load(Path(path)/f'{arm}.json') for arm in v.ARMS}

    def test_all_native_controls(self):
        for arm,r in self.fixtures().items():
            result=v.audit_receipt(r)
            self.assertEqual(result['arm'],arm)
        paid=v.audit_receipt(self.fixtures()['learned_exposed_repair'])
        self.assertEqual(paid['measured']['genotype_first_duration']['observed_death_age'],11)
        self.assertEqual(paid['measured']['installed_first_duration']['observed_death_age'],10)
        self.assertEqual(len(paid['measured']['genotype_spells']),5)

    def test_ledger_and_attribution_tampering(self):
        source=self.fixtures()['learned_exposed_repair']
        attacks=[]
        def attack(name,mutator):
            r=copy.deepcopy(source);mutator(r);attacks.append((name,r))
        attack('missing cell horizon',lambda r:r.update(completed_steps=95))
        attack('seed substitution',lambda r:r.update(seed=v.SEED+1))
        attack('changed summary',lambda r:r['summary'].update(repair_payments=5))
        attack('free fuel',lambda r:r['timeline'][17]['state'].update(repair_fuel=8.))
        attack('merge recovery spell',lambda r:r['timeline'][16].update(genotype=r['timeline'][5]['genotype']))
        attack('erase payment',lambda r:r['timeline'][1].update(paid_edits=0))
        attack('wrong pre-step table',lambda r:r['timeline'][17].update(pre_step_table=0))
        attack('remove lesion',lambda r:r['timeline'][16].update(events=[e for e in r['timeline'][16]['events'] if e['kind']!='hazard']))
        attack('wrong class attribution',lambda r:next(e for e in r['timeline'][7]['events'] if e['kind']=='world').update(rcf='00'))
        attack('non-distinct world binding',lambda r:next(e for e in r['timeline'][7]['events'] if e['kind']=='world')['bound_nodes'].__setitem__(1,r['ordinary_nodes'][0]))
        attack('tamper operation cost',lambda r:r['table_catalogue'][0][2]['template']['property_effects'][0]['op']['Subtract'].update(value={'Float':0.}))
        attack('wrong sealed role',lambda r:r['initialization'].update(role=v.ROLE+1))
        attack('tamper final literal',lambda r:r['final_inventory']['nodes'][0][1]['data'].update(label=666))
        for name,r in attacks:
            with self.subTest(name=name):
                with self.assertRaises((AssertionError,KeyError,IndexError,StopIteration)):v.audit_receipt(r)

    def test_frozen_missing_installation_not_credit(self):
        r=v.audit_receipt(self.fixtures()['learned_frozen'])
        self.assertTrue(r['measured']['genotype_spells'])
        self.assertEqual(r['measured']['installed_spells'],[])
        self.assertFalse(r['measured']['nontrivial'])

    def test_full_panel_when_requested(self):
        path=os.environ.get('ERE_RULE_MAINTENANCE_AUDIT_INPUT')
        if not path:self.skipTest('full panel not requested')
        result=v.audit(Path(path));self.assertEqual(result['native_advances'],6048)
        self.assertFalse(result['sustained_claim'])

if __name__=='__main__':unittest.main()
