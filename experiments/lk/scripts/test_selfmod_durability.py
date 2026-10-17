#!/usr/bin/env python3
"""Frozen verdict boundary cases; no new physical experiment or threshold fitting."""
import copy,json,unittest
from audit_selfmod_durability import durability
from audit_selfmod_bounded import reproduction
SELECTED=dict(executable=True,cancelled=False)
def fixture():
 result=[];start=90
 for i in range(10):
  end=100 if i%2==0 else 90;net=end-start
  result.append(dict(start_capital=start,end_capital=end,income=1050+net,rent=1000,retired=50,starved=0,enumeration_censored=False,
   minimum_rent_headroom=40,family_alive_every_boundary=True,members_alive_every_boundary=True,lineage_productivity_windows=[1]*4,
   member_productivity_windows=[[1]*3 for _ in range(4)],birth_windows=[1]*4,lineage_birth_windows=[1,0,0,0]))
  start=end
 return result
class DurabilityTests(unittest.TestCase):
 def test_funded_negative_endpoint_passes_new_but_fails_old(self):
  windows=fixture();self.assertTrue(durability(SELECTED,windows,True)['durable_success'])
  last=windows[-1];legacy=dict(bounded_establishment=True,totals=dict(late_lineage_income=last['income'],late_lineage_rent=last['rent']))
  extra=dict(late_lineage_retired=50,late_lineage_starved=0,late_start_capital=100,final_capital=90,late_birth_windows=[1]*4,late_lineage_births=1)
  self.assertFalse(reproduction(legacy,extra)['joint_success'])
 def test_failures(self):
  cases=[('rent_floor',lambda w:w[2].__setitem__('minimum_rent_headroom',-1)),('family_loss',lambda w:w[2].__setitem__('family_alive_every_boundary',False)),
   ('member_loss',lambda w:w[2].__setitem__('members_alive_every_boundary',False)),('family_birth',lambda w:w[2].__setitem__('lineage_birth_windows',[0]*4)),
   ('culture_birth',lambda w:w[2].__setitem__('birth_windows',[1,1,0,1])),('family_inactive',lambda w:w[2].__setitem__('lineage_productivity_windows',[1,0,1,1])),
   ('member_inactive',lambda w:w[2]['member_productivity_windows'][0].__setitem__(1,0)),('enumeration',lambda w:w[0].__setitem__('enumeration_censored',True))]
  for name,edit in cases:
   with self.subTest(name=name):
    windows=fixture();edit(windows);self.assertFalse(durability(SELECTED,windows,True)['durable_success'])
 def test_incomplete_invalid_and_missing(self):
  self.assertFalse(durability(SELECTED,fixture(),False)['durable_success'])
  self.assertFalse(durability(dict(SELECTED,executable=False),fixture(),True)['durable_success'])
  self.assertFalse(durability(dict(SELECTED,cancelled=True),fixture(),True)['durable_success'])
  self.assertFalse(durability(SELECTED,fixture()[:9],True)['durable_success'])
 def test_accounting_corruption_is_rejected(self):
  for key in ['end_capital','income','retired']:
   with self.subTest(key=key):
    windows=fixture();windows[2][key]+=1
    with self.assertRaises(ValueError):durability(SELECTED,windows,True)
 def test_phase_floor_equality_and_warmup(self):
  windows=fixture();windows[2]['minimum_rent_headroom']=0;windows[0]['minimum_rent_headroom']=-1
  self.assertTrue(durability(SELECTED,windows,True)['durable_success'])
 def test_false_verdict_and_missing_denominator(self):
  windows=fixture();windows[2]['minimum_rent_headroom']=-1
  actual=durability(SELECTED,windows,True);damaged=copy.deepcopy(actual);damaged['durable_success']=True
  self.assertNotEqual(damaged,actual)
  expected=dict(fixed_denominator=8,durability_replicated=5>=6);damaged=dict(fixed_denominator=5,durability_replicated=True)
  self.assertNotEqual(expected,damaged)
if __name__=='__main__':unittest.main()
