#!/usr/bin/env python3
import copy,hashlib,json,os,shutil,tempfile,unittest
from pathlib import Path
import verify_evolving_minds_arbitration as audit
class ArbitrationAuditTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.root=Path(os.environ['EVOLVING_MINDS_ARBITRATION_FIXTURE']);cls.base=(cls.root/'base_rules.toml').read_bytes();cls.c=audit.load(cls.root/'replay.py','arbitration_test_replay');cls.d=audit.load(cls.root/'d0_reference.py','arbitration_test_d0');cls.s=audit.load(cls.root/'schedule_reference.py','arbitration_test_schedule')
 def tamper(self,edit,local=False):
  recipe=dict(policy='Local' if local else 'Global',topology='Disjoint',rotation=0,reversed=False);name=('local' if local else 'global')+'_disjoint_r0_forward'
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'run';shutil.copytree(self.root/name,p);f=p/'result.json';r=json.loads(f.read_text());edit(r);f.write_text(json.dumps(r))
   with self.assertRaises((ValueError,KeyError,IndexError)):audit.verify_cell(p,self.c,self.d,self.s,self.base,recipe)
 def test_clean_panel_is_read_only(self):
  hashes=lambda:{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in self.root.rglob('*') if p.is_file()};before=hashes();self.assertEqual(audit.verify(self.root)['cells_verified'],64);self.assertEqual(before,hashes())
 def test_missing_offer(self):self.tamper(lambda r:r['offers'].pop())
 def test_fake_conflict(self):self.tamper(lambda r:r['offers'][0].__setitem__('counter_only_conflicts',[]))
 def test_wrong_winner(self):self.tamper(lambda r:r.__setitem__('accepted_parents',[2]))
 def test_changed_key(self):self.tamper(lambda r:r['offers'][0]['canonical_key'].__setitem__(0,0))
 def test_missing_paid_warmup(self):self.tamper(lambda r:r['frames'][1]['events'].clear())
 def test_duplicate_disjoint_birth(self):self.tamper(lambda r:r['frames'][4]['events'].append(copy.deepcopy(r['frames'][4]['events'][0])))
 def test_missing_local_birth(self):self.tamper(lambda r:r['frames'][4]['events'].pop(),True)
 def test_free_copy(self):self.tamper(lambda r:r.__setitem__('copying_dissipation',0.0))
 def test_changed_frontier(self):
  def edit(r):
   n=next(n for _,n in r['frames'][0]['nodes'] if n['label']==self.c.SITE);next(v for k,v in n['properties'] if k==audit.LOCAL)['Int']+=8
  self.tamper(edit,True)
 def test_hidden_offer(self):
  def edit(r):
   n=next(n for _,n in r['frames'][0]['nodes'] if n['label']==self.c.SITE);next(v for k,v in n['properties'] if k==audit.OFFER)['Bool']=True
  self.tamper(edit)
 def test_bad_child_reset(self):
  def edit(r):
   n=next(n for _,n in r['frames'][4]['nodes'] if n['label']==self.c.ROOT and self.c.value(n,self.c.BID,'Int')>2);next(v for k,v in n['properties'] if k==self.c.AGE)['Int']=9
  self.tamper(edit)
 def test_missing_summary_cell(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'panel';shutil.copytree(self.root,p);f=p/'summary.json';x=json.loads(f.read_text());x.pop();f.write_text(json.dumps(x))
   with self.assertRaises(ValueError):audit.verify(p)
if __name__=='__main__':unittest.main()
