"""Zero-physics construction and archive-backed adversarial transfer tests."""
import copy
import os
from pathlib import Path
import unittest
from unittest.mock import patch
import verify_ere_rule_prevention_transfer as v
class TransferTests(unittest.TestCase):
    def output(self):
        value=os.environ.get('ERE_RULE_PREVENTION_TRANSFER_AUDIT_INPUT')
        if not value:self.skipTest('postflight artifacts not requested')
        return Path(value)
    def test_design_catalogue_and_import_closure(self):
        import run_ere_rule_prevention_transfer as r
        self.assertEqual(v.DESIGN['maximum_native_advances'],72*96)
        self.assertEqual(len(v.CONDITIONS),8)
        for arm in v.ARMS:
            for a in range(4):
                for b in range(4):
                    raw,compiled=v.editor_catalogue(arm,[a,b]);self.assertEqual(len(raw),9)
                    self.assertEqual(raw[1]['enabled'],a!=0);self.assertEqual(raw[2]['enabled'],b!=0)
                    self.assertTrue(all(c['meta_visible'] and c['pattern']['node_label_ceiling'] is None for c in compiled))
        closure=r.primitive.dependencies('scripts/verify_ere_rule_prevention_transfer.py')
        self.assertIn('scripts/verify_ere_complete_long_gates.py',closure)
        self.assertIn('scripts/verify_ere_complete_gates.py',closure)
        self.assertLessEqual(closure,set(r.names()))
    def test_fixed_denominator_and_censoring(self):
        result=v.audit(self.output());self.assertEqual(len(result['cells']),72)
        for cohort in result['cohorts'].values():self.assertEqual(cohort['fixed_cohort'],9)
        self.assertIsInstance(result['transfer_pass'],bool)
        self.assertFalse(result['scientific_acceptance']);self.assertFalse(result['training_admitted'])
        # Exact value assertions belong to the verdict, not the accounting test.
        for condition in ['learned_paid','oracle_paid']:
            cohort=result['cohorts'][condition]
            self.assertIsNone(cohort['genotype_first_half_life']['half_life'])
            self.assertFalse(cohort['reference_exposure_lifetime_ratio_pass'])
    def test_native_payment_threat_tampering(self):
        source=v.a.load(self.output()/'native/cell-0007.json');attacks=[]
        def attack(name,fn):r=copy.deepcopy(source);fn(r);attacks.append((name,r))
        attack('horizon',lambda r:r.update(completed_steps=95))
        attack('free fuel',lambda r:r['timeline'][16]['state'].update(defense_fuel=4.))
        attack('pending erased',lambda r:r['timeline'][15]['state'].update(pending_threat=0))
        attack('damage inadmissible',lambda r:next(x for x in r['timeline'][16]['usage']['rules'] if r['table_catalogue'][r['timeline'][16]['pre_step_table']][x['rule_id']]['name']=='prevention_damage').update(results=0))
        attack('A2 blanket',lambda r:r['table_catalogue'][0][-2]['pattern'].update(node_label_ceiling=v.META))
        attack('wrong body',lambda r:r['timeline'][16].update(genotype=r['timeline'][0]['genotype']))
        attack('wrong attribution',lambda r:next(e for e in r['timeline'][17]['events'] if e['kind']=='world').update(rcf='00'))
        attack('nondistinct',lambda r:next(e for e in r['timeline'][17]['events'] if e['kind']=='world')['bound_nodes'].__setitem__(1,r['ordinary_nodes'][0]))
        attack('claim inflation',lambda r:r.update(sustained_claim=True))
        attack('out of range action',lambda r:r.update(actions=[4,1]))
        for name,r in attacks:
            with self.subTest(name=name):
                with self.assertRaises((AssertionError,KeyError,IndexError,StopIteration)):v.audit_receipt(r)
    def test_paired_decision_tampering(self):
        out=self.output();policies=v.imports(out);source=v.a.load(out/'native/decisions.json');real=v.a.load
        edits=[lambda d:d[0].update(stream_after=0),lambda d:d[0]['draws'].__setitem__(0,0.),
            lambda d:d[0]['policies'][0].update(actions=[0,0]),lambda d:d[0]['policies'][1].update(policy_index=0),
            lambda d:d.pop(),lambda d:d[0]['policies'][0]['input'].update(mode='blind')]
        for fn in edits:
            d=copy.deepcopy(source);fn(d)
            def load(path):return d if Path(path)==out/'native/decisions.json' else real(path)
            with patch.object(v.a,'load',side_effect=load):
                with self.assertRaises((AssertionError,KeyError,IndexError)):v.decisions(out,policies)
    def test_missing_birth_not_removed(self):
        # Pure instrument test: nine slots remain even when one target never appears.
        m=dict(genotype_first_duration=None,installed_first_duration=None,root_duration=dict(lower_bound=96,observed_death_age=None,right_censored=True),touch_intervals=[4],nontrivial=False)
        cohort=v.survival.cohort([m]*9,16.)
        self.assertEqual(cohort['fixed_cohort'],9);self.assertEqual(cohort['eligible_members'],0)
        self.assertTrue(cohort['genotype_first_half_life']['missing_births']);self.assertFalse(cohort['reference_exposure_lifetime_ratio_pass'])
if __name__=='__main__':unittest.main()
