#!/usr/bin/env python3
import copy
import math
import os
from pathlib import Path
import unittest
from unittest.mock import patch
import ere_resource_gradient_diagnostic as p
import verify_ere_resource_gradient_diagnostic as v

class GradientDiagnostic(unittest.TestCase):
    def root(self):return Path(os.environ['ERE_RESOURCE_GRADIENT_DIAGNOSTIC'])
    def data(self):return v.old.a.load(self.root()/'diagnostic.json')
    def reject(self,data):
        original=v.old.a.load
        with patch.object(v.old.a,'load',side_effect=lambda path:data if Path(path).name=='diagnostic.json' else original(path)):
            with self.assertRaises(AssertionError):v.audit(self.root())
    def test_full_diagnostic(self):
        result=v.audit(self.root());self.assertTrue(result['diagnostic_pass'])
        self.assertEqual(result['recorded_updates_checked'],800)
        self.assertEqual(result['sampled_actions_checked'],6400)
        self.assertEqual(result['derivative_checks'],44)
    def test_independent_enumeration_and_finite_differences(self):
        states=p.read_states(self.root())
        for probe in self.data()['probes']:
            a=p.expected(probe['theta'],probe['mode'],states)
            b=v.analytic(probe['theta'],probe['mode'],states)
            v.near(a['expected_gradient'],b['expected_gradient'])
            v.near(p.finite_difference(probe['theta'],probe['mode'],states),[40*g for g in b['expected_gradient']])
    def test_baseline_cancels_expectation_not_variance(self):
        states=p.read_states(self.root());theta=[0.]*4
        fixed=v.analytic(theta,'full',states,9.5);other=v.analytic(theta,'full',states,0.)
        v.near(fixed['expected_gradient'],other['expected_gradient'])
        self.assertNotEqual(fixed['conditional_variance'],other['conditional_variance'])
    def test_unpaid_zero_actor_credit(self):
        states=p.read_states(self.root());changed=copy.deepcopy(states)
        for s in changed[1::2]:s['utility']=[20,1]
        a=p.expected([0.]*4,'full',states);b=p.expected([0.]*4,'full',changed)
        self.assertEqual(a['expected_gradient'],b['expected_gradient'])
        self.assertEqual(a['conditional_variance'],b['conditional_variance'])
        self.assertEqual(a['funded_expected_mean'],b['funded_expected_mean'])
        for s in b['states'][1::2]:self.assertEqual(s['expected_gradient'],[0.]*4)
    def test_domains_and_aliasing(self):
        states=p.read_states(self.root())
        for theta in [[math.nan,0,0,0],[8.1,0,0,0]]:
            with self.assertRaises(AssertionError):p.expected(theta,'full',states)
        with self.assertRaises(AssertionError):p.expected([0.]*4,'unknown',states)
        paid=states[::2]
        self.assertEqual(v.rank([v.features(s,'full') for s in paid]),3)
        self.assertEqual(v.rank([v.features(s,'cues_masked') for s in paid]),1)
    def test_derivative_margin_variance_corruptions(self):
        for field,value in [('expected_gradient',[0]*4),('conditional_variance',[0]*4),
                            ('funded_utility_finite_difference',[0]*4)]:
            bad=self.data();bad['probes'][-1][field]=value;self.reject(bad)
        bad=self.data();bad['probes'][-1]['states'][0]['signed_optimal_margin']=-1;self.reject(bad)
        bad=self.data();bad['recorded_updates'][-1]['sampled_gradient']=[0]*4;self.reject(bad)
    def test_counts_and_admission_corruptions(self):
        for key in ['models','probes','recorded_updates']:
            bad=self.data();bad[key].pop();self.reject(bad)
        bad=self.data();bad['optimizer_updates']=1;self.reject(bad)
        bad=self.data();bad['funded_utility_gradient_to_update_gradient_scale']=20;self.reject(bad)
    def test_saved_update_action_and_credit_corruptions(self):
        original=v.old.a.load;target=self.root()/'inputs/update-0799.json';record=original(target)
        for field,value in [('action',1-record['samples'][0]['action']),('utility',20)]:
            bad=copy.deepcopy(record);bad['samples'][0][field]=value
            with patch.object(v.old.a,'load',side_effect=lambda path:bad if Path(path)==target else original(path)):
                with self.assertRaises(AssertionError):v.audit(self.root())
    def test_parent_root_corruption(self):
        original=v.old.a.digest
        with patch.object(v.old.a,'digest',side_effect=lambda path:'0'*64 if Path(path).name=='parent_seal.json' else original(path)):
            with self.assertRaises(AssertionError):v.custody(self.root())

if __name__=='__main__':unittest.main()
