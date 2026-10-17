#!/usr/bin/env python3
import copy
import os
from pathlib import Path
import time
import unittest
import evolving_minds_descendant_behavior as b

class BehaviorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        bundle=Path(__file__).parent/'input_binding.json'
        cls.native=Path(b.common.data(bundle)['native_directory']) if bundle.exists() else Path(os.environ.get('EVOLVING_MINDS_DESCENDANT_FIXTURE','output/evolving_minds_descendant_development_20261004')).resolve()
        v=b.common.load(cls.native/'stream_reference.py','bt_stream')
        a=b.common.load(cls.native/'admission_reference.py','bt_admission')
        c=b.common.load(cls.native/'replay.py','bt_replay')
        d=b.common.load(cls.native/'d0_reference.py','bt_d0')
        s=b.common.load(cls.native/'schedule_reference.py','bt_schedule')
        o=b.common.load(cls.native/'opportunity_reference.py','bt_observer')
        cls.d=d;cls.a=a
        collected=[];o.LifetimeObserver=b.observer_type(o.LifetimeObserver,collected)
        cfg=dict(seed=17,endowment=24,founders=['Delayed']*2,kernels=['Faithful']*2,home_offsets=[0,4],rotation=0,reversed=False,horizon=64,scheduler='semantic-lottery-v1')
        h=dict(policy='first-paid-descendant-by24-v1',target_id=103,new_source=1)
        chain=v.inspect_segment(cls.native/'world17_recovery_resumed',funded_config=cfg,descendant_config=h)
        final,physical,opportunity,count=b.routes.shared.optimized_cell(chain,(a,c,d,s),(cls.native/'b24_base_rules.toml').read_bytes(),cfg,None,v,o,time.monotonic(),descendant_config=h)
        cls.result=dict(records=final['admission']['records'],opportunity=opportunity,final_row=physical['final_row'],target_state=chain[-1]['commits'][-1]['state']['native']['descendant'])
        cls.traces=collected[0].behavior
        b.validate_trace(cls.result,cls.traces)
        cls.root=opportunity['lifetimes'][0]
        cls.trace=cls.traces[str(cls.root['id'])]

    def reject_trace(self,edit):
        x=copy.deepcopy(self.traces);edit(x)
        with self.assertRaises((ValueError,KeyError)):b.validate_trace(self.result,x)

    def test_complete_recovery_trace_and_routes(self):
        cell=b.routes.decompose_cell(self.result,self.a,self.d)
        self.assertEqual(cell['full_lifetimes_verified'],len(self.traces))
        self.assertEqual(sum(len(x) for x in self.traces.values()),sum(m['live_start_ticks'] for m in self.result['opportunity']['lifetimes']))
    def test_missing_root_refused(self):self.reject_trace(lambda x:x.pop(next(iter(x))))
    def test_missing_tick_refused(self):self.reject_trace(lambda x:x[next(iter(x))].pop())
    def test_duplicate_tick_refused(self):self.reject_trace(lambda x:x[next(iter(x))].append(copy.deepcopy(x[next(iter(x))][-1])))
    def test_altered_attempt_refused(self):
        self.reject_trace(lambda x:next(r for r in x['1'] if r['attempt_choice'] is not None).__setitem__('attempt_choice',None))
    def test_altered_harvest_refused(self):self.reject_trace(lambda x:x['1'][0].__setitem__('harvested',99))
    def test_altered_funding_refused(self):self.reject_trace(lambda x:x['1'][0].__setitem__('funded',not x['1'][0]['funded']))
    def test_altered_payment_refused(self):self.reject_trace(lambda x:x['1'][0].__setitem__('maintenance',999))
    def test_equal_finite_trace_never_universal(self):
        r=b.compare_roots(self.root,self.root,self.trace,self.trace)
        self.assertTrue(r['finite_behavior_equal']);self.assertFalse(r['universal_equivalence'])
    def test_same_totals_different_choices_refused_as_equivalence(self):
        t=copy.deepcopy(self.trace)
        next(r for r in t if r['attempt_choice'] is not None)['attempt_choice']^=True
        r=b.compare_roots(self.root,self.root,t,self.trace)
        self.assertFalse(r['finite_behavior_equal']);self.assertIn('attempt_choice',r['first_divergence']['fields'])
    def test_internal_difference_is_separate(self):
        t=copy.deepcopy(self.trace);t[0]['internal_slots']['2']=not t[0]['internal_slots'].get('2',False)
        r=b.compare_roots(self.root,self.root,t,self.trace)
        self.assertTrue(r['finite_behavior_equal']);self.assertEqual(r['internal_state_different_ticks'],1)
    def test_death_boundary_difference_refused_as_equivalence(self):
        target=copy.deepcopy(self.root);target['death_tick']+=1
        self.assertFalse(b.compare_roots(target,self.root,self.trace,self.trace)['finite_behavior_equal'])
    def test_missing_child_not_installed(self):self.assertFalse(b.compare_roots(None,self.root,[],self.trace)['finite_behavior_equal'])
    def test_canonical_forgery_refused(self):
        r=copy.deepcopy(self.result);r['records'][0]['proposed_bytes'][0]^=1
        with self.assertRaises(ValueError):b.routes.decompose_cell(r,self.a,self.d)
    def test_missing_parent_refused(self):
        r=copy.deepcopy(self.result);r['opportunity']['lifetimes'][-1]['parent']=999999
        with self.assertRaisesRegex(ValueError,'parent edge'):b.class_propagation(r,1,self.d)
    def test_root_class_end_before_death_boundary(self):
        self.assertEqual(b.routes.boundary_intervals([dict(born_tick=17,death_tick=33)]),[dict(first_observed_tick=17,last_observed_tick=32)])

class InheritanceTests(unittest.TestCase):
    def setUp(self):
        self.d=b.common.load(Path(__file__).resolve().parent.parent/'scripts/verify_evolving_minds_d0.py','synthetic_d0') if not (Path(__file__).parent/'input_binding.json').exists() else b.common.load(Path(b.common.data(Path(__file__).parent/'input_binding.json')['native_directory'])/'d0_reference.py','synthetic_d0')
        p=self.d.program('cue');q=self.d.program('coin')
        def root(i,parent,program,born,death):return dict(id=i,parent=parent,program=program,born_tick=born,death_tick=death,completion_tick=born+4,requests=1)
        self.result=dict(opportunity=dict(lifetimes=[root(1,None,p,0,20),root(2,1,q,5,22),root(3,2,q,12,30),root(4,1,q,15,40),root(5,3,q,25,None)]))
    def test_recurrence_is_not_inheritance(self):
        r=b.class_propagation(self.result,2,self.d)
        self.assertEqual(r['same_class_descendant_ids'],[3,5]);self.assertEqual(r['independent_class_roots_outside_target_lineage'],[4]);self.assertEqual(r['class_origin_ids'],[2,4]);self.assertEqual(r['max_inherited_generation'],2)
    def test_class_outlives_root_through_real_descendants(self):
        r=b.class_propagation(self.result,2,self.d)
        self.assertTrue(r['same_class_descendants_outlive_target']);self.assertFalse(r['target_root_censored']);self.assertTrue(r['inherited_lineage_present_at64'])
    def test_class_survival_from_independent_origin_not_transmission(self):
        self.result['opportunity']['lifetimes']=[m for m in self.result['opportunity']['lifetimes'] if m['id'] not in (3,5)]
        self.result['opportunity']['lifetimes'][-1]['death_tick']=None
        r=b.class_propagation(self.result,2,self.d)
        self.assertTrue(r['class_present_at64']);self.assertFalse(r['inherited_lineage_present_at64']);self.assertFalse(r['same_class_descendants_outlive_target'])
    def test_equal_hashes_do_not_bridge_changed_intermediate(self):
        self.result['opportunity']['lifetimes'][2]['program']=self.d.program('cue')
        r=b.class_propagation(self.result,2,self.d)
        self.assertEqual(r['same_class_descendant_ids'],[]);self.assertIn(5,r['class_origin_ids'])
    def test_duplicate_root_refused(self):
        self.result['opportunity']['lifetimes'].append(copy.deepcopy(self.result['opportunity']['lifetimes'][0]))
        with self.assertRaisesRegex(ValueError,'unique class'):b.class_propagation(self.result,2,self.d)

class PanelDenominatorTests(unittest.TestCase):
    def test_missing_case_or_recovery_cannot_enter_analysis(self):
        class Checker:
            def cases(self):return [dict(name='unchanged'),dict(name='substitution'),dict(name='recovery')]
        for omitted in ('unchanged','substitution','recovery'):
            results={x:dict() for x in ('unchanged','substitution','recovery') if x!=omitted}
            traces={x:dict() for x in ('unchanged','substitution','recovery')}
            with self.assertRaisesRegex(ValueError,'case/trace denominator'):
                b.analyze_results(dict(results=results),traces,Checker(),None,None)
    def test_missing_or_extra_trace_case_refused(self):
        class Checker:
            def cases(self):return [dict(name='unchanged')]
        for traces in ({},{'unchanged':{},'extra':{}}):
            with self.assertRaisesRegex(ValueError,'case/trace denominator'):
                b.analyze_results(dict(results={'unchanged':{}}),traces,Checker(),None,None)

if __name__=='__main__':unittest.main()
