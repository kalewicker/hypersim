#!/usr/bin/env python3
"""Ancestry and founder-family criterion boundary tests."""
import copy,types,unittest
from selfmod_second_edit_common import descendants,view,row_view,mark_floor
from audit_selfmod_durability import durability
from test_selfmod_durability import fixture,SELECTED
class SecondEditTests(unittest.TestCase):
 def test_cousins_and_older_prune_family_excluded(self):
  genes={9:dict(parent=1),1:dict(parent=None),3:dict(parent=1),20:dict(parent=9),31:dict(parent=20)}
  self.assertEqual(descendants(genes,9),{9,20,31})
 def test_cycle_missing_parent_and_unknown_founder_rejected(self):
  for genes,founder in [({1:dict(parent=None),2:dict(parent=3),3:dict(parent=2)},1),({1:dict(parent=None),2:dict(parent=99)},1),({1:dict(parent=None)},9)]:
   with self.assertRaises(ValueError):descendants(genes,founder)
 def test_view_preserves_raw_genealogy_and_culture(self):
  state=types.SimpleNamespace(genes={1:dict(lineage=True,member=0),2:dict(lineage=True,member=0),3:dict(lineage=False,member=1),4:dict(lineage=False,member=2)},balances={1:dict(capital=60),2:dict(capital=50),3:dict(capital=30),4:dict(capital=30)},cfg=dict(rent=1))
  before=copy.deepcopy(state);focal=view(state,{2});w={};mark_floor(w,focal)
  self.assertEqual(w['minimum_rent_headroom'],49);self.assertTrue(w['members_alive_every_boundary']);self.assertEqual(state.genes,before.genes)
 def test_older_family_cannot_rescue_missing_second_family(self):
  state=types.SimpleNamespace(genes={1:dict(lineage=True,member=0),3:dict(lineage=False,member=1),4:dict(lineage=False,member=2)},balances={1:dict(capital=90),3:dict(capital=30),4:dict(capital=30)},cfg=dict(rent=1))
  windows=fixture();self.assertTrue(durability(SELECTED,windows,True)['durable_success'])
  for w in windows[2:]:mark_floor(w,view(state,{2}))
  self.assertFalse(durability(SELECTED,windows,True)['durable_success'])
 def test_row_view_never_changes_raw_primary_flags(self):
  raw=dict(roots=[dict(id=1,lineage=True),dict(id=2,lineage=True)],births=[dict(child=5,gene=dict(lineage=True)),dict(child=6,gene=dict(lineage=True))]);before=copy.deepcopy(raw)
  mapped=row_view(raw,{2,5});self.assertFalse(mapped['roots'][0]['lineage']);self.assertTrue(mapped['births'][0]['gene']['lineage']);self.assertFalse(mapped['births'][1]['gene']['lineage']);self.assertEqual(raw,before)
if __name__=='__main__':unittest.main()
