"""Prospective decision rules on synthetic outcomes only; zero scientific/native evidence."""
import copy,json,sys,unittest
from pathlib import Path
from plan_campaign import statistics,panel,candidate,classify_world,analyze_outcomes
C=json.loads((Path(sys.argv.pop(1))/'contract.json').read_text())
def pair(origin='endogenous'):
    return {'earlier':'event-a','later':'event-b','scores':dict(zip(C['candidate']['roles'],[0,7,7,22,22,-6,7,-10,22,22,22,2,-10])),'all_assays_funded':True,'origins':[origin,origin],'transmissions':[2,2],'active_descendant_generations':[[1,2],[1,2]],'measurement_status':'complete'}
def outcomes(full=0,external=0):
    n=47;return {'blocks':n,'worlds':[{'block':i,'arm':a,'status':'complete','qualified':i<(full if a=='endogenous' else external if a=='external' else 0)} for i in range(n) for a in C['design']['arms']]}
class Planning(unittest.TestCase):
    def test_01_exact_joint_sizing_not_detection_or_separate_power(self):
        s=statistics(C);self.assertEqual(s['selected_blocks'],47);rows={r['blocks']:r for r in s['rows'] if r['scenario']=='target'};self.assertLess(rows[46]['joint_power']['value'],.8);self.assertGreaterEqual(rows[47]['joint_power']['value'],.8);self.assertEqual(rows[47]['prevalence_successes_required'],16)
    def test_02_same_functional_rule_can_pass_external_arm(self):
        self.assertTrue(candidate(pair(),'endogenous',C));self.assertTrue(candidate(pair('external'),'external',C));self.assertFalse(candidate(pair('external'),'endogenous',C))
    def test_03_two_independent_benefits_are_not_dependency(self):
        p=pair();p['scores'].update(intact=20,sham=20,restored=20,padded_remove_later=10,padded_remove_earlier=10,padded_remove_both=0);self.assertFalse(candidate(p,'endogenous',C))
    def test_04_generation_duplicates_and_missing_assay_do_not_pass(self):
        p=pair();p['active_descendant_generations'][1]=[1,1];self.assertFalse(candidate(p,'endogenous',C));p=pair();p['scores'].pop('padded_remove_both');w={'status':'complete','samples':[{'status':'living','pairs':[p]}]+[{'status':'empty'}]*9};self.assertIsNone(classify_world(w,'endogenous',C)['qualified'])
    def test_05_unknown_world_retains_full_denominator(self):
        o=outcomes(47);o['worlds'][0].update(status='partial',qualified=None);r=analyze_outcomes(o,C);self.assertEqual(r['world_denominator'],188);self.assertEqual(r['denominator'],47);self.assertEqual(r['status'],'inconclusive_operational_or_measurement_failure');self.assertFalse(r['cumulative_evolution'])
    def test_06_missing_duplicate_or_smaller_panel_is_rejected(self):
        for edit in [lambda o:o['worlds'].pop(),lambda o:o['worlds'].__setitem__(0,copy.deepcopy(o['worlds'][1])),lambda o:o.update(blocks=46)]:
            o=outcomes();edit(o)
            with self.assertRaises(ValueError):analyze_outcomes(o,C)
    def test_07_equal_external_success_does_not_establish_selfmod_advantage(self):
        r=analyze_outcomes(outcomes(47,47),C);self.assertTrue(r['cumulative_evolution']);self.assertFalse(r['self_modification_advantage'])
    def test_08_exact_prevalence_boundary_and_valid_negative(self):
        self.assertFalse(analyze_outcomes(outcomes(15),C)['cumulative_evolution']);self.assertTrue(analyze_outcomes(outcomes(16),C)['cumulative_evolution']);self.assertEqual(analyze_outcomes(outcomes(),C)['status'],'valid_negative')
    def test_09_positive_mechanical_control_invalidates_interpretation(self):
        p=pair('nonheritable');w={'status':'complete','samples':[{'status':'living','pairs':[p]}]+[{'status':'empty'}]*9};self.assertEqual(classify_world(w,'nonheritable',C)['status'],'mechanical_control_violation');o=outcomes(47);o['worlds'][2]['qualified']=True;self.assertEqual(analyze_outcomes(o,C)['status'],'invalid_mechanical_control')
    def test_10_seed_panel_is_new_fixed_and_arm_matched(self):
        p=panel(C,47);self.assertEqual(len(p['worlds']),188);self.assertEqual(p['sampling_slots'],1880);self.assertEqual(len({x['seed'] for x in p['worlds']}),47);self.assertFalse({x['seed'] for x in p['worlds']}&set(C['design']['development_seeds']));self.assertTrue(all(not p[f] for f in ['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']))
if __name__=='__main__':unittest.main()
