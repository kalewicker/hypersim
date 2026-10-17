#!/usr/bin/env python3
"""Helper regressions plus a postmeasurement, read-only archive regression."""
import copy
from pathlib import Path
import unittest
from unittest.mock import patch
import audit_ere_valid_optimizer_signal as a

class Helpers(unittest.TestCase):
    def fixture(self):
        entry={'execution':'bytes','rcf':'class','request':'ChangePriority'}
        group={'roles':[{'productions':[entry]},{'productions':[]}],'role_support':[0]}
        parent={'groups':[group],'operator_support':[0],'root':{'idx':1,'version':1}}
        c={'parents':[parent]}
        e={'grammar_profile':'E2-complete-valid-production-grammar-v1','parent':0,'operator':0,'role':'Core',
           'production_index':0,'production_count':1,'operator_support':[0],'role_support':[0],
           'signature':'bytes','kernel_attempts':1,'resampled':False,'parent_fallback':False,
           'kernel':{'request':'ChangePriority','role':'Core','committed':True},
           'classes':{'rcf':'class'},'status':'Applied','admitted':True,'root':{'idx':1,'version':1}}
        return c,e
    def test_membership_valid(self):
        c,e=self.fixture();a.check_membership(c,e,full=True)
    def test_membership_fails_closed(self):
        c,e=self.fixture()
        for field,value in [('parent',168),('production_index',1),('role_support',[]),('signature','wrong'),('parent_fallback',True),('resampled',True),('kernel_attempts',2)]:
            with self.subTest(field=field):
                bad=copy.deepcopy(e);bad[field]=value
                with self.assertRaises((AssertionError,IndexError,KeyError,ValueError)):
                    a.check_membership(c,bad,full=True)
    def test_request_mismatch(self):
        c,e=self.fixture();e['kernel']['request']='Activate'
        with self.assertRaises(AssertionError):a.check_membership(c,e,full=True)
    def test_tied_ranks_flat_and_average(self):
        self.assertEqual(a.ranks([0.,0.,None]),[0.,0.,0.])
        self.assertEqual(a.ranks([1.,1.,2.]),[-.25,-.25,.5])
    def test_tv_denominator_fixed(self):
        initial=[{'signature':str(i)} for i in range(64)]
        final=copy.deepcopy(initial);final[0]['signature']='other'
        self.assertEqual(a.distance(initial,final),1/64)
        final=copy.deepcopy(initial)
        for i in range(4):final[i]['signature']='other'
        self.assertEqual(a.distance(initial,final),.0625)
    def test_registered_seeds_disjoint(self):
        self.assertNotEqual(a.SEED,0xE452390001)
        self.assertFalse(set(a.PHYSICS)&{101,211,307,401})
    def test_archive_multiple_rows_catalogue_scope(self):
        archive=Path(__file__).resolve().parents[1]/'output/ere_valid_optimizer_signal_panel_20260929'
        if not (archive/'summary.json').exists():
            self.skipTest('Completed panel archive is not present')
        with patch.object(a,'check_membership',wraps=a.check_membership) as membership:
            result=a.audit(archive)
        self.assertTrue(result['audit_pass'])
        self.assertEqual(result['rollouts'],120)
        self.assertEqual(membership.call_count,120+6*64)
        catalogue=membership.call_args_list[0].args[0]
        self.assertIn('parents',catalogue)
        self.assertTrue(all(call.args[0] is catalogue for call in membership.call_args_list))

if __name__=='__main__':unittest.main()
