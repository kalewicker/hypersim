#!/usr/bin/env python3
import hashlib,json,os,shutil,tempfile,unittest
from pathlib import Path
import verify_evolving_minds_admission as a
class AdmissionAuditTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.root=Path(os.environ['EVOLVING_MINDS_ADMISSION_FIXTURE']);cls.cell=cls.root/'mixed_settlement_r0_forward';cls.base=(cls.root/'base_rules.toml').read_bytes();cls.c=a.load(cls.root/'replay.py','adm_test_replay');cls.d=a.load(cls.root/'d0_reference.py','adm_test_d0');cls.s=a.load(cls.root/'schedule_reference.py','adm_test_schedule');cls.recipe=json.loads((cls.cell/'recipe.json').read_text())
 def tamper(self,file,edit):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'cell';shutil.copytree(self.cell,p);f=p/file
   if file.endswith('jsonl'):
    x=[json.loads(l) for l in f.read_text().splitlines()];edit(x);f.write_text(''.join(json.dumps(l)+'\n' for l in x))
   else:x=json.loads(f.read_text());edit(x);f.write_text(json.dumps(x))
   with self.assertRaises((ValueError,KeyError,IndexError)):a.verify_cell(p,self.c,self.d,self.s,self.base,self.recipe)
 def test_clean_read_only(self):
  hashes=lambda:{str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in self.root.rglob('*') if f.is_file()};before=hashes();self.assertEqual(a.verify(self.root)['cells_verified'],12);self.assertEqual(before,hashes())
 def test_missing_offer(self):self.tamper('admission_state.json',lambda x:next(r for r in x['rounds'] if r['offers'])['offers'].pop())
 def test_forged_permutation(self):self.tamper('admission_state.json',lambda x:next(r for r in x['rounds'] if len(r['permutation'])>1)['permutation'].reverse())
 def test_forged_selection(self):self.tamper('admission_state.json',lambda x:next(r for r in x['rounds'] if r['selected'])['selected'].clear())
 def test_missing_request(self):self.tamper('admission_state.json',lambda x:x['records'].pop())
 def test_changed_draw(self):self.tamper('admission_state.json',lambda x:next(r for r in x['records'] if r['raw_draws'])['raw_draws'].__setitem__(0,0))
 def test_free_request(self):self.tamper('admission_state.json',lambda x:x['records'][0].__setitem__('request_charge',0.0))
 def test_forged_refund(self):self.tamper('admission_state.json',lambda x:next(r for r in x['records'] if not r['syntax_valid']).__setitem__('refunded',0.0))
 def test_duplicate_child_id(self):self.tamper('admission_state.json',lambda x:next(r for r in x['records'] if r['child_id']).__setitem__('child_id',1))
 def test_missing_native_reserve(self):self.tamper('journal.jsonl',lambda x:next(r for r in x if r['phase']=='reserve_' and len(r['events'])>1)['events'].pop())
 def test_missing_native_settle(self):self.tamper('journal.jsonl',lambda x:next(r for r in x if r['phase']=='transaction_' and len(r['events'])>1)['events'].pop())
 def test_missing_identity(self):self.tamper('journal.jsonl',lambda x:x[2]['events'].pop())
 def test_wrong_frontier(self):
  def edit(x):
   n=next(n for _,n in x['nodes'] if n['label']==self.c.SITE);next(v for k,v in n['properties'] if k==a.LOCAL)['Int']+=8
  self.tamper('initial.json',edit)
 def test_false_research_claim(self):self.tamper('run_status.json',lambda x:x.__setitem__('reproductive_fairness_accepted',True))
 def test_bad_checkpoint(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'cell';shutil.copytree(self.cell,p);f=p/'admission.checkpoint';x=bytearray(f.read_bytes());x[-1]^=1;f.write_bytes(x)
   with self.assertRaises(ValueError):a.verify_cell(p,self.c,self.d,self.s,self.base,self.recipe)
if __name__=='__main__':unittest.main()
