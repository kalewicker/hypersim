#!/usr/bin/env python3
import copy,json,os,shutil,tempfile,unittest,hashlib
from pathlib import Path
import verify_evolving_minds_funded_admission as f
class FundedAdmissionAuditTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.root=Path(os.environ['EVOLVING_MINDS_FUNDED_ADMISSION_FIXTURE']);cls.v=f.load(cls.root/'stream_reference.py','fa_test_stream');cls.cfg=f.cases()[4]['config']
 def tamper(self,file,edit,rehash=False):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'segment';shutil.copytree(self.root/'fault_before_commit',p);target=p/file;x=f.data(target);edit(x);target.write_text(json.dumps(x))
   if rehash:
    marker=p/'commit_0008.json';m=f.data(marker);m['attestation_sha256']=hashlib.sha256(target.read_bytes()).hexdigest();marker.write_text(json.dumps(m))
   with self.assertRaises((ValueError,KeyError,IndexError)):self.v.inspect_segment(p,funded_config=self.cfg)
 def test_wrong_header_budget(self):self.tamper('segment.json',lambda x:x['funded_config'].__setitem__('endowment',12))
 def test_wrong_kernel(self):self.tamper('segment.json',lambda x:x['funded_config']['kernels'].__setitem__(0,'Faithful'))
 def test_wrong_checkpoint_parameters_even_after_rehash(self):self.tamper('state_0008.json',lambda x:x['native']['funded_config'].__setitem__('seed',23),True)
 def test_bad_published_cursor(self):self.tamper('commit_0008.json',lambda x:x['cursor'].__setitem__('offset',1))
 def test_legacy_profile(self):self.tamper('segment.json',lambda x:x.__setitem__('profile','evolving-minds-d0-streaming-v1'))
 def test_bad_native_checkpoint_envelope(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'segment';shutil.copytree(self.root/'fault_before_commit',p);q=p/'checkpoint_0008.bin';b=bytearray(q.read_bytes());b[:8]=b'HGEMS001';q.write_bytes(b);m=f.data(p/'commit_0008.json');m['checkpoint_sha256']=hashlib.sha256(b).hexdigest();(p/'commit_0008.json').write_text(json.dumps(m))
   with self.assertRaises(ValueError):self.v.inspect_segment(p,funded_config=self.cfg)
 def test_pending_marker_never_promoted(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'segment';shutil.copytree(self.root/'fault_before_commit',p);shutil.copy(p/'commit_0016.json.pending',p/'commit_0016.json')
   with self.assertRaises((ValueError,json.JSONDecodeError)):self.v.inspect_segment(p,funded_config=self.cfg)
 def test_false_scientific_flag(self):
  status=f.data(self.root/'run_status.json');status['evolution_enabled']=True
  with self.assertRaises(ValueError):self.v.boundary(status,f.PROFILE)
 def test_omitted_baseline(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp);rows=f.cases();(p/'registered_cases.json').write_text(json.dumps(rows));(p/'registered_faults.json').write_text(json.dumps(f.FAULTS));self.assertEqual(len(rows),10)
   for case in rows[:-1]:(p/f"baseline_{case['name']}").mkdir()
   for n in f.FAULT_NAMES:
    for prefix in ['fault','recovered']:(p/f'{prefix}_{n}').mkdir()
   with self.assertRaises(ValueError):f.denominator(p,rows)
 def test_imports_are_read_only(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'a.py';p.write_text('VALUE=24\n');f.load(p,'fa_read_only');self.assertFalse((p.parent/'__pycache__').exists())
if __name__=='__main__':unittest.main()
