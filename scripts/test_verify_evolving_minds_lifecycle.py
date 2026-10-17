#!/usr/bin/env python3
import copy,hashlib,json,os,shutil,tempfile,unittest
from pathlib import Path
import verify_evolving_minds_lifecycle as audit
class LifecycleAuditTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.root=Path(os.environ['EVOLVING_MINDS_LIFECYCLE_FIXTURE']);cls.base=(cls.root/'base_rules.toml').read_bytes();cls.c=audit.load(cls.root/'replay.py','lifecycle_test_replay');cls.d=audit.load(cls.root/'d0_reference.py','lifecycle_test_d0');cls.s=audit.load(cls.root/'schedule_reference.py','lifecycle_test_schedule')
 def tamper(self,edit,arm='Release'):
  with tempfile.TemporaryDirectory() as tmp:
   target=Path(tmp)/'run';shutil.copytree(self.root/arm.lower(),target);edit(target)
   with self.assertRaises((ValueError,KeyError,IndexError)):audit.verify_cell(target,self.c,self.d,self.s,self.base,'cue','Informative',arm)
 def journal(self,edit):
  def mutate(p):
   f=p/'journal.jsonl';es=[json.loads(x) for x in f.read_text().splitlines()];edit(es);f.write_text(''.join(json.dumps(e)+'\n' for e in es))
  self.tamper(mutate)
 def test_clean_panel_is_read_only(self):
  hashes=lambda:{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in self.root.rglob('*') if p.is_file()};before=hashes();self.assertTrue(audit.verify(self.root)['audit_pass']);self.assertEqual(before,hashes())
 def test_missing_control(self):self.journal(lambda es:next(e for e in es if e['tick']==4 and e['pass']==4)['events'].clear())
 def test_release_one_tick_early(self):self.journal(lambda es:next(e for e in es if e['tick']==8 and e['pass']==4)['events'][0].__setitem__('rule','communication_sham'))
 def test_missing_maintenance(self):self.journal(lambda es:next(e for e in es if e['pass']==15 and e['events'])['events'].clear())
 def test_missing_birth(self):self.journal(lambda es:next(e for e in es if e['pass']==16 and e['events'])['events'].clear())
 def test_missing_cleanup(self):self.journal(lambda es:next(e for e in es if e['pass']>=18 and e['events'])['events'].clear())
 def test_wrong_fork_graph(self):
  def edit(p):
   f=p/'fork.json';g=json.loads(f.read_text());next(v for _,n in g['nodes'] if n['label']==self.c.ROOT for k,v in n['properties'] if k==self.c.ENERGY)['Float']+=1;f.write_text(json.dumps(g))
  self.tamper(edit)
 def test_wrong_control_state(self):self.tamper(lambda p:(p/'control_state.json').write_text('{"arm":"Sham"}'))
 def test_foreign_owner_law(self):
  def edit(p):
   f=p/'rules.toml';s=f.read_text();s=s.replace('"key" = 242016','"key" = 240001',1);f.write_text(s)
  self.tamper(edit)
 def test_bad_checkpoint(self):
  def edit(p):
   f=p/'release.checkpoint';b=bytearray(f.read_bytes());b[-1]^=1;f.write_bytes(b)
  self.tamper(edit)
 def test_training_boundary(self):
  def edit(p):
   f=p/'run_status.json';s=json.loads(f.read_text());s['training_admitted']=True;f.write_text(json.dumps(s))
  self.tamper(edit)
 def test_fairness_boundary(self):
  def edit(p):
   f=p/'run_status.json';s=json.loads(f.read_text());s['reproductive_fairness_accepted']=True;f.write_text(json.dumps(s))
  self.tamper(edit)
if __name__=='__main__':unittest.main()
