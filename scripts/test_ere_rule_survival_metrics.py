import copy
from pathlib import Path
import unittest
import ere_rule_survival_metrics as m
import verify_ere_rule_lifecycle as old

ERRORS=(AssertionError,ValueError,KeyError,TypeError,IndexError)
ROOT=Path('output/ere_rule_lifecycle_fixture_development_20261003/native')
def load(arm):return old.integrity.load(ROOT/f'{old.SEEDS[0]}-{arm}.json')
def measured(arm):
    r=load(arm);old.audit_receipt(r)
    return m.cell(r,old.candidate(False,10)['rcf'],[row['step'] for row in r['timeline'] for _ in range(row['paid_edits'])])


class SurvivalTests(unittest.TestCase):
    def test_actual_replacement_eight_does_not_exceed_eight(self):
        c=measured('live_replacement');r=m.cohort([c])
        self.assertEqual(c['genotype_spells'],[dict(start=1,end=9),dict(start=17,end=None)])
        self.assertEqual(c['installed_spells'],[dict(start=2,end=10),dict(start=18,end=None)])
        self.assertEqual(r['genotype_first_half_life']['half_life'],8);self.assertEqual(r['installed_first_half_life']['half_life'],8)
        self.assertEqual(r['mean_actual_region_touch_interarrival'],8.);self.assertFalse(r['literal_lifetime_ratio_pass'])
        self.assertIsNone(r['root_half_life']['half_life'])

    def test_actual_one_shot_censoring_and_missing_interval(self):
        c=measured('live_paid');r=m.cohort([c]);self.assertEqual(c['genotype_first_duration']['lower_bound'],23)
        self.assertEqual(c['installed_first_duration']['lower_bound'],22);self.assertIsNone(r['genotype_first_half_life']['half_life'])
        self.assertIsNone(r['mean_actual_region_touch_interarrival']);self.assertFalse(r['literal_lifetime_ratio_pass'])

    def test_fixed_denominator_missing_birth_and_no_ordinary_firing(self):
        c=measured('live_replacement');missing=measured('live_unpaid');r=m.cohort([c,missing])
        self.assertEqual(r['fixed_cohort'],2);self.assertEqual(r['eligible_members'],1)
        self.assertTrue(r['genotype_first_half_life']['missing_births']);self.assertFalse(r['literal_lifetime_ratio_pass'])

    def test_bounds_with_ambiguous_and_exact_censoring(self):
        death=lambda x:dict(lower_bound=x,observed_death_age=x,right_censored=False)
        censored=lambda x:dict(lower_bound=x,observed_death_age=None,right_censored=True)
        r=m.half_life_bounds([death(8),death(8),censored(3),censored(3)])
        self.assertIsNone(r['half_life']);self.assertEqual((r['lower_bound'],r['upper_bound']),(3,8))
        r=m.half_life_bounds([death(8),death(8),censored(12),censored(12)])
        self.assertEqual(r['half_life'],8)
        r=m.half_life_bounds([censored(23)]*3);self.assertIsNone(r['half_life']);self.assertIsNone(r['upper_bound'])

    def test_repair_touch_count_cannot_change_reference_exposure_comparator(self):
        c=measured('live_replacement');dense=copy.deepcopy(c);dense['touch_steps']=list(range(1,18));dense['touch_intervals']=[1]*16
        r=m.cohort([dense],reference_exposure_interval=8.)
        self.assertTrue(r['literal_lifetime_ratio_pass']);self.assertFalse(r['reference_exposure_lifetime_ratio_pass'])
        self.assertFalse(r['sustained_claim']);self.assertFalse(r['scientific_acceptance'])

    def test_initialized_priority_only_timeline_and_touch_refusals(self):
        r=load('live_replacement')
        with self.assertRaises(ERRORS):m.cell(r,r['initial_all_candidates'][0]['rcf'],[1,9,17])
        bad=copy.deepcopy(r);target=old.candidate(False,10)['rcf']
        for row in bad['timeline']:
            for c in row['genotype']:
                if c['rcf']==target:c['structural_rcf']=r['initial_all_candidates'][0]['structural_rcf']
        with self.assertRaises(ERRORS):m.cell(bad,target,[1,9,17])
        bad=copy.deepcopy(r);bad['timeline'][5]['step']=6
        with self.assertRaises(ERRORS):m.cell(bad,target,[1,9,17])
        for touches in ([9,1],[0,9],[1,25],[1,float('nan')]):
            with self.assertRaises(ERRORS):m.cell(r,target,touches)


if __name__=='__main__':unittest.main()
