#!/usr/bin/env python3
import copy,hashlib,json,os,shutil,tempfile,unittest
from pathlib import Path
import verify_evolving_minds_streaming as v

class StreamingAuditTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.root=Path(os.environ['EVOLVING_MINDS_STREAMING_FIXTURE']);cls.cell=cls.root/'fault_before_commit'
 def changed(self,file,edit,binary=False):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'cell';shutil.copytree(self.cell,p);f=p/file
   x=bytearray(f.read_bytes()) if binary else json.loads(f.read_text());edit(x);f.write_bytes(x if binary else json.dumps(x).encode())
   with self.assertRaises((ValueError,KeyError,IndexError,json.JSONDecodeError)):v.inspect_segment(p)
 def test_published_corrupt_rejects_no_fallback(self):self.changed('checkpoint_0008.bin',lambda b:b.__setitem__(-1,b[-1]^1),True)
 def test_bad_marker(self):self.changed('commit_0008.json',lambda x:x['cursor'].__setitem__('offset',0))
 def test_forged_hash(self):self.changed('commit_0008.json',lambda x:x['cursor'].__setitem__('log_sha256','0'*64))
 def test_gap(self):self.changed('commit_0008.json',lambda x:x['cursor'].__setitem__('tick',16))
 def test_wrong_profile(self):self.changed('segment.json',lambda x:x.__setitem__('profile','wrong'))
 def test_cycle(self):self.changed('segment.json',lambda x:x.__setitem__('parent',dict(directory=str(self.cell),commit='commit_0008.json',sha256='0'*64)))
 def test_orphan_ignored_and_read_only(self):
  hashes=lambda:{str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in self.cell.iterdir()};before=hashes();x=v.inspect_segment(self.cell);self.assertEqual(x[-1]['marker']['cursor']['tick'],8);self.assertEqual(before,hashes());self.assertTrue((self.cell/'checkpoint_0016.bin').exists())
 def test_orphan_promotion_rejected(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'cell';shutil.copytree(self.cell,p);shutil.copyfile(p/'commit_0016.json.pending',p/'commit_0016.json')
   with self.assertRaises(ValueError):v.inspect_segment(p)
 def test_symlink(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'cell';shutil.copytree(self.cell,p);(p/'linked').symlink_to(p/'commit_0008.json')
   with self.assertRaises(ValueError):v.inspect_segment(p)
 def test_forged_live_lineage_with_rehashed_marker(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'cell';shutil.copytree(self.cell,p);state=json.loads((p/'state_0008.json').read_text());state['native']['live']['maximum_depth']+=1;b=json.dumps(state).encode();(p/'state_0008.json').write_bytes(b);m=json.loads((p/'commit_0008.json').read_text());m['attestation_sha256']=v.digest(b);(p/'commit_0008.json').write_text(json.dumps(m));chain=v.inspect_segment(p)
   with self.assertRaises(ValueError):v.audit_state(chain,chain[-1],v.flatten(chain))
 def model_tamper(self,edit):
  chain=v.inspect_segment(self.cell);commit=copy.deepcopy(chain[-1]);edit(commit)
  modules=tuple(v.load(self.root/f,n) for f,n in [('admission_reference.py','tamper_admission'),('replay.py','tamper_replay'),('d0_reference.py','tamper_d0'),('schedule_reference.py','tamper_schedule')])
  with self.assertRaises((ValueError,KeyError,IndexError)):v.physical_audit(chain,commit,modules,(self.root/'base_rules.toml').read_bytes())
 def test_free_native_request(self):
  self.model_tamper(lambda x:next(b for b in x['bundles'] if b['records'])['records'][0].__setitem__('request_charge',0.0))
 def test_missing_native_event(self):
  self.model_tamper(lambda x:next(e for b in x['bundles'] for e in b['entries'] if e['events'])['events'].pop())
 def test_forged_final_energy(self):
  self.model_tamper(lambda x:x['bundles'][-1]['row'].__setitem__('total_energy',0.0))
 def test_false_scientific_claim(self):
  x=json.loads((self.cell/'run_status.json').read_text());x['gate_pass']=True
  with self.assertRaises(ValueError):v.boundary(x)
if __name__=='__main__':unittest.main()

