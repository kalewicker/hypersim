#!/usr/bin/env python3
import copy
import os
from pathlib import Path
import unittest
import evolving_minds_descendant_routes as routes

class RouteDiagnosisTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        bundled=Path(__file__).parent/'input_binding.json'
        if bundled.exists():
            binding=routes.common.data(bundled);native=Path(binding['native_directory']);audit=Path(binding['audited_directory'])
        else:
            native=Path(os.environ['EVOLVING_MINDS_HANDOFF_FIXTURE']);audit=Path(os.environ['EVOLVING_MINDS_HANDOFF_AUDIT_FIXTURE'])
        cls.results=routes.common.data(audit/'independent_replay.json')['results']
        cls.a=routes.common.load(native/'admission_reference.py','diagnosis_test_admission')
        cls.d=routes.common.load(native/'d0_reference.py','diagnosis_test_d0')
        cls.cells={name:routes.decompose_cell(value,cls.a,cls.d) for name,value in cls.results.items()}
        cls.source=cls.results['world401_source']

    def refuses(self,change):
        result=copy.deepcopy(self.source);change(result)
        with self.assertRaises((ValueError,KeyError)):routes.decompose_cell(result,self.a,self.d)

    def test_all_cases_requests_lifetimes_and_payments_reconcile(self):
        summary=routes.summarize(self.cells)
        self.assertEqual(summary['full_organism_records'],113)
        self.assertEqual(summary['focused_descendant_requests'],31)
        self.assertEqual({k:v['requests'] for k,v in summary['categories'].items()},dict(zip(routes.CATEGORIES,[12,5,10,4])))
        self.assertEqual(summary['mean_paired_difference'],-536)
        for key in routes.FLAGS:
            self.assertIs(summary[key],routes.FLAGS[key])
        self.assertFalse(summary['causal_effect_of_individual_edit_identified'])
        for cell in self.cells.values():
            self.assertEqual(sum(x['direct_child_harvested'] for x in cell['incoming_route_partition'].values()),cell['descendant_yield'])
            self.assertEqual(sum(len(c['root_ids']) for c in cell['classes'].values()),cell['full_lifetimes_verified'])
            self.assertTrue(all(r['subtree']['nonadditive'] for r in cell['routes']))

    def test_missing_and_duplicate_requests_refused(self):
        self.refuses(lambda r:r['records'].pop())
        self.refuses(lambda r:r['records'].append(copy.deepcopy(r['records'][0])))
    def test_truncated_horizon_and_tick_gap_refused(self):
        self.refuses(lambda r:r['opportunity']['ticks'].pop())
        self.refuses(lambda r:r['opportunity']['ticks'][0].__setitem__('tick',2))
    def test_missing_founder_refused(self):
        self.refuses(lambda r:r['opportunity']['lifetimes'].pop(0))
    def test_missing_child_and_false_identity_refused(self):
        def remove(r):
            child=next(q['child_id'] for q in r['records'] if q['child_id'] is not None)
            r['opportunity']['lifetimes']=[m for m in r['opportunity']['lifetimes'] if m['id']!=child]
        self.refuses(remove)
        self.refuses(lambda r:r['records'][0].__setitem__('child_id',99999))
    def test_canonical_parent_and_proposal_bytes_refused(self):
        self.refuses(lambda r:r['records'][0]['proposed_bytes'].__setitem__(0,255))
        self.refuses(lambda r:r['records'][0]['parent_bytes'].__setitem__(0,255))
    def test_forged_edit_target_old_new_refused(self):
        self.refuses(lambda r:r['records'][0].__setitem__('target_id',102))
        self.refuses(lambda r:r['records'][0].__setitem__('old_source',7))
        self.refuses(lambda r:r['records'][0].__setitem__('new_source',3))
    def test_free_request_and_refund_forgery_refused(self):
        self.refuses(lambda r:r['records'][0].__setitem__('request_charge',0))
        self.refuses(lambda r:r['records'][0].__setitem__('refunded',29))
    def test_lifetime_request_counter_and_charge_refused(self):
        self.refuses(lambda r:r['opportunity']['lifetimes'][0].__setitem__('requests',0))
        self.refuses(lambda r:r['opportunity']['lifetimes'][0].__setitem__('copying_charge',0))
    def test_parent_genotype_and_class_censoring_refused(self):
        self.refuses(lambda r:r['records'][0]['parent_program']['instructions'][0].__setitem__('source',7))
        self.refuses(lambda r:r['opportunity']['lifetimes'][0].__setitem__('right_censored_at_64',True))
    def test_forged_founder_and_ancestry_refused(self):
        self.refuses(lambda r:r['opportunity']['lifetimes'][-1].__setitem__('founder',0))
        self.refuses(lambda r:r['opportunity']['lifetimes'][-1].__setitem__('parent',99999))
    def test_lifetime_death_and_completion_bounds_refused(self):
        self.refuses(lambda r:r['opportunity']['lifetimes'][0].__setitem__('death_tick',65))
        self.refuses(lambda r:r['opportunity']['lifetimes'][0].__setitem__('completion_tick',0))
    def test_missing_case_or_focused_route_refused(self):
        cells=copy.deepcopy(self.cells);cells.pop('world409_source')
        with self.assertRaisesRegex(ValueError,'denominator'):routes.summarize(cells)
        cells=copy.deepcopy(self.cells);cell=cells['world401_source'];cell['routes'].remove(next(r for r in cell['routes'] if r['stage']=='descendant'))
        with self.assertRaisesRegex(ValueError,'complete31'):routes.summarize(cells)
    def test_changed_operator_cannot_hide_focused_failure(self):
        cells=copy.deepcopy(self.cells);next(r for r in cells['world401_source']['routes'] if r['stage']=='descendant')['operator']='faithful'
        with self.assertRaisesRegex(ValueError,'complete31'):routes.summarize(cells)
    def test_proposed_edit_is_not_installed_after_rejection(self):
        rejected=[r for cell in self.cells.values() for r in cell['routes'] if r['category']=='syntax_rejected']
        self.assertTrue(rejected)
        self.assertTrue(all(r['proposed_bytes_changed'] and not r['class_installed'] and r['class_first_install_tick'] is None and not r['subtree']['root_ids'] for r in rejected))
    def test_same_tick_class_installations_are_one_cohort(self):
        for cell in self.cells.values():
            for cls in cell['classes'].values():
                for r in cell['routes']:
                    if r['child_id'] in cls['first_installation_cohort']:
                        self.assertTrue(r['first_installation_cohort']);self.assertEqual(r['class_first_install_tick'],r['tick'])

class BoundaryIntervalTests(unittest.TestCase):
    def test_class_outlives_original_root_without_censoring_it(self):
        roots=[dict(born_tick=7,death_tick=20),dict(born_tick=15,death_tick=None)]
        self.assertEqual(routes.boundary_intervals(roots),[dict(first_observed_tick=7,last_observed_tick=64)])
        self.assertEqual(roots[0]['death_tick'],20)
    def test_extinction_gap_and_reintroduction_are_retained(self):
        roots=[dict(born_tick=1,death_tick=3),dict(born_tick=5,death_tick=7)]
        self.assertEqual(routes.boundary_intervals(roots),[dict(first_observed_tick=1,last_observed_tick=2),dict(first_observed_tick=5,last_observed_tick=6)])
    def test_terminal_death_has_no_boundary_survival(self):
        self.assertEqual(routes.boundary_intervals([dict(born_tick=63,death_tick=64)]),[dict(first_observed_tick=63,last_observed_tick=63)])
        self.assertEqual(routes.boundary_intervals([]),[])

if __name__=='__main__':unittest.main()
