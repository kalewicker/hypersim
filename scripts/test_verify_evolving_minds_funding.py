#!/usr/bin/env python3
import copy,json,os,shutil,tempfile,unittest
from pathlib import Path
import verify_evolving_minds_funding as f
class FundingAuditTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.root=Path(os.environ['EVOLVING_MINDS_FUNDING_FIXTURE']);cls.modules=[f.load(cls.root/p,n) for p,n in [('mutation_reference.py','ft_mut'),('replay.py','ft_replay'),('d0_reference.py','ft_d0'),('schedule_reference.py','ft_schedule')]];cls.base=(cls.root/'b24_base_rules.toml').read_bytes()
 def tamper(self,file,edit,cell='b24_delayed_cue_g103_s0',raw=False):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'cell';shutil.copytree(self.root/cell,p);target=p/file
   if raw:
    old=target.read_text();new=edit(old);self.assertNotEqual(old,new);target.write_text(new)
   else:
    x=f.read(target);edit(x);target.write_text(json.dumps(x))
   a,c,d,s=self.modules;recipe=f.read(p/'recipe.json')
   with self.assertRaises((ValueError,KeyError,IndexError)):a.verify_cell(p,c,d,s,self.base,'delayed_cue','Informative','Faithful',neighborhood=recipe,endowment=24)
 def test_uncharged_copy(self):self.tamper('mutations.json',lambda x:x[0].__setitem__('copying_charge',0))
 def test_wrong_newborn_budget(self):self.tamper('run_status.json',lambda x:x.__setitem__('endowment',12))
 def test_inflated_rejection_refund(self):self.tamper('mutations.json',lambda x:x[0].__setitem__('refunded',30),cell='b24_delayed_cue_g104_s1')
 def test_hidden_parent_charge(self):self.tamper('mutations.json',lambda x:x[0].__setitem__('request_charge',0))
 def test_forged_construction(self):self.tamper('lifetimes.json',lambda x:x['2'].__setitem__('maintenance_paid',0))
 def test_false_gate(self):self.tamper('run_status.json',lambda x:x.__setitem__('gate_pass',True))
 def test_unfunded_guard(self):self.tamper('rules.toml',lambda x:x.replace('value = 30.0','value = 18.0',1),raw=True)
 def test_replaced_ancestry(self):self.tamper('lineage.json',lambda x:x['2'].__setitem__('parent',None))
 def test_full_denominator_and_omitted_cell(self):
  cells=f.recipes(self.modules[2]);self.assertEqual(len(cells),116);self.assertEqual(len({f.name(r) for r in cells}),116)
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp);(p/'registered_cells.json').write_text(json.dumps(cells))
   for r in cells[:-1]:(p/f.name(r)).mkdir()
   with self.assertRaises(ValueError):f.denominator(p,cells)
 def test_loader_does_not_write_bytecode(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'module.py';p.write_text('VALUE=24\n');self.assertEqual(f.load(p,'ft_readonly').VALUE,24);self.assertFalse((p.parent/'__pycache__').exists())
if __name__=='__main__':unittest.main()
