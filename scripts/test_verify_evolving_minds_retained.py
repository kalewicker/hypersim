#!/usr/bin/env python3
import copy,json,os,shutil,tempfile,unittest,hashlib
from pathlib import Path
import verify_evolving_minds_retained as r
class RetainedAuditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=Path(os.environ['EVOLVING_MINDS_RETAINED_FIXTURE']);cls.v=r.load(cls.root/'stream_reference.py','retained_test_storage');cls.cfg=r.cases()[3]['config']
    def tamper(self,file,edit,rehash=False):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'segment';shutil.copytree(self.root/'fault_before_commit',p);q=p/file;x=r.data(q);edit(x);q.write_text(json.dumps(x))
            if rehash:
                m=r.data(p/'commit_0008.json');m['attestation_sha256']=hashlib.sha256(q.read_bytes()).hexdigest();(p/'commit_0008.json').write_text(json.dumps(m))
            with self.assertRaises((ValueError,KeyError)):self.v.inspect_segment(p,funded_config=self.cfg,retained_policy=r.POLICY)
    def test_wrong_header_policy(self):self.tamper('segment.json',lambda x:x.__setitem__('retained_policy','redraw'))
    def test_wrong_checkpoint_policy_even_with_rehashed_attestation(self):self.tamper('state_0008.json',lambda x:x['native']['retained'].__setitem__('policy','redraw'),True)
    def test_legacy_profile(self):self.tamper('segment.json',lambda x:x.__setitem__('profile','evolving-minds-d0-funded-streaming-v1'))
    def test_new_sampler_genome_forgery(self):
        # Rehashed metadata cannot replace the independently regenerated first sample.
        cfg=r.cases()[3]['config'];p=self.root/'baseline_invalid_immediate';chain=self.v.inspect_segment(p,funded_config=cfg,retained_policy=r.POLICY)
        commit=copy.deepcopy(chain[-1]['commits'][1]);commit['state']['native']['retained']['samples']['1']['proposed_bytes'][0]^=1
        modules=tuple(r.load(self.root/n,'ret_test_'+str(i)) for i,n in enumerate(['admission_reference.py','replay.py','d0_reference.py','schedule_reference.py']))
        with self.assertRaisesRegex(ValueError,'regenerated retained'):self.v.physical_audit(chain,commit,modules,(self.root/'b24_base_rules.toml').read_bytes(),funded_config=cfg,retained_policy=r.POLICY)
    def test_missing_retained_sample(self):
        cfg=r.cases()[3]['config'];p=self.root/'baseline_invalid_immediate';chain=self.v.inspect_segment(p,funded_config=cfg,retained_policy=r.POLICY);commit=copy.deepcopy(chain[-1]['commits'][1]);commit['state']['native']['retained']['samples'].pop('1')
        modules=tuple(r.load(self.root/n,'ret_missing_'+str(i)) for i,n in enumerate(['admission_reference.py','replay.py','d0_reference.py','schedule_reference.py']))
        with self.assertRaisesRegex(ValueError,'regenerated retained'):self.v.physical_audit(chain,commit,modules,(self.root/'b24_base_rules.toml').read_bytes(),funded_config=cfg,retained_policy=r.POLICY)
    def test_forged_scientific_flag(self):
        status=r.data(self.root/'run_status.json');status['evolution_enabled']=True
        with self.assertRaises(ValueError):self.v.boundary(status,r.PROFILE)
    def test_import_does_not_write_bytecode(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'a.py';p.write_text('VALUE=24\n');r.load(p,'retained_bytecode');self.assertFalse((p.parent/'__pycache__').exists())
if __name__=='__main__':unittest.main()
