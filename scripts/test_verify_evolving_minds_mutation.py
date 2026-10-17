#!/usr/bin/env python3
"""Adversarial checks of the independent one-shot transaction audit."""
import copy,hashlib,json,os,shutil,tempfile,unittest
from pathlib import Path
import verify_evolving_minds_mutation as audit
class MutationCheckerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.root=Path(os.environ['EVOLVING_MINDS_MUTATION_FIXTURE']);cls.base=(cls.root/'base_rules.toml').read_bytes();cls.c=audit.load(cls.root/'replay.py','mutation_test_replay');cls.d=audit.load(cls.root/'d0_reference.py','mutation_test_d0');cls.s=audit.load(cls.root/'schedule_reference.py','mutation_test_schedule')
 def tamper(self,edit,cell='source_cue',kernel='SourceKernel'):
  with tempfile.TemporaryDirectory() as tmp:
   target=Path(tmp)/'run';shutil.copytree(self.root/cell,target);edit(target)
   with self.assertRaises((ValueError,KeyError,IndexError)):
    audit.verify_cell(target,self.c,self.d,self.s,self.base,'cue','Informative',kernel)
 def records(self,edit,cell='source_cue',kernel='SourceKernel'):
  def change(p):
   path=p/'mutations.json';x=json.loads(path.read_text());edit(x);path.write_text(json.dumps(x))
  self.tamper(change,cell,kernel)
 def journal(self,edit):
  def change(p):
   path=p/'journal.jsonl';es=[json.loads(s) for s in path.read_text().splitlines()];edit(es);path.write_text(''.join(json.dumps(e)+'\n' for e in es))
  self.tamper(change)
 def test_clean_panel_is_read_only(self):
  hashes=lambda:{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in self.root.rglob('*') if p.is_file()};before=hashes();r=audit.verify(self.root);self.assertTrue(r['audit_pass']);self.assertEqual(r['cells_verified'],6);self.assertEqual(before,hashes())
 def test_missing_request(self):self.records(lambda rs:rs.pop(0))
 def test_reordered_requests(self):self.records(lambda rs:rs.reverse())
 def test_altered_draw(self):self.records(lambda rs:rs[0]['raw_draws'].__setitem__(0,rs[0]['raw_draws'][0]^1))
 def test_extra_redraw(self):self.records(lambda rs:rs[0]['raw_draws'].append(7))
 def test_semantic_target(self):self.records(lambda rs:rs[0].__setitem__('target_id',9999))
 def test_wrong_refund(self):self.records(lambda rs:rs[0].__setitem__('refunded',17.5),'invalid_cue','Invalid')
 def test_fallback_child(self):self.records(lambda rs:rs[0].__setitem__('child_id',2),'invalid_cue','Invalid')
 def test_unpaid_request(self):self.records(lambda rs:rs[0].__setitem__('request_charge',0.0))
 def test_wrong_copy_charge(self):self.records(lambda rs:rs[0].__setitem__('copying_charge',0.0),'coin_cue','Coin')
 def test_mutated_proposal(self):self.records(lambda rs:rs[0]['proposed_program']['instructions'][1].__setitem__('source',3))
 def test_missing_reservation_event(self):self.journal(lambda es:next(e for e in es if e['pass']==15 and e['events'])['events'].clear())
 def test_missing_disposal_event(self):self.journal(lambda es:next(e for e in es if e['pass']==17 and e['events'])['events'].clear())
 def test_missing_cleanup_event(self):self.journal(lambda es:next(e for e in es if e['pass']>=19 and e['events'])['events'].clear())
 def test_altered_stream_state(self):
  def edit(p):
   f=p/'mutation_state.json';s=json.loads(f.read_text());s['rng_state']^=1;f.write_text(json.dumps(s))
  self.tamper(edit)
 def test_free_settlement_law(self):
  def edit(rs):
   r=rs[0];r['settlement_rule']=r['settlement_rule'].replace('value = 12.0','value = 14.0',1)
  self.records(edit,'coin_cue','Coin')
 def test_bad_checkpoint(self):
  def edit(p):
   f=p/'midpoint.checkpoint';b=bytearray(f.read_bytes());b[-1]^=1;f.write_bytes(b)
  self.tamper(edit)
 def test_admission_boundary(self):
  def edit(p):
   f=p/'run_status.json';s=json.loads(f.read_text());s['evolution_enabled']=True;f.write_text(json.dumps(s))
  self.tamper(edit)
if __name__=='__main__':unittest.main()
