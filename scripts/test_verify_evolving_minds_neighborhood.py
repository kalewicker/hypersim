#!/usr/bin/env python3
import copy,json,os,shutil,tempfile,unittest
from pathlib import Path
import verify_evolving_minds_neighborhood as n
class NeighborhoodAuditTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.root=Path(os.environ['EVOLVING_MINDS_NEIGHBORHOOD_FIXTURE']);cls.cell=cls.root/'cue_unchanged';cls.modules=[n.load(cls.root/f,k) for f,k in [('mutation_reference.py','nt_mut'),('replay.py','nt_replay'),('d0_reference.py','nt_d0'),('schedule_reference.py','nt_schedule')]];cls.recipe=n.read(cls.cell/'recipe.json');cls.base=(cls.root/'base_rules.toml').read_bytes()
 def tamper(self,file,edit):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'cell';shutil.copytree(self.cell,p);f=p/file;x=n.read(f);edit(x);f.write_text(json.dumps(x));a,c,d,s=self.modules
   with self.assertRaises((ValueError,KeyError,IndexError)):a.verify_cell(p,c,d,s,self.base,'cue','Informative','Faithful',neighborhood=self.recipe)
 def test_free_child(self):self.tamper('mutations.json',lambda x:x[0].__setitem__('copying_charge',0))
 def test_changed_first_proposal(self):self.tamper('mutations.json',lambda x:x[0]['proposed_program']['instructions'][1].__setitem__('source',1))
 def test_forged_lifetime(self):self.tamper('lifetimes.json',lambda x:x['2'].__setitem__('maintenance_paid',0))
 def test_forged_turnover(self):self.tamper('outcome.json',lambda x:x.__setitem__('descendants',999))
 def test_false_scientific_claim(self):self.tamper('run_status.json',lambda x:x.__setitem__('evolution_enabled',True))
 def test_missing_request(self):self.tamper('mutations.json',lambda x:x.pop())
 def test_extra_founder(self):self.tamper('lineage.json',lambda x:x['2'].__setitem__('parent',None))
 def test_complete_enumeration(self):
  rs=n.recipes(self.modules[2]);self.assertEqual(len(rs),58);self.assertEqual(len({n.name(r) for r in rs}),58)
if __name__=='__main__':unittest.main()
