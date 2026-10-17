"""Adversarial journal tests: rebuild custody to exercise independent semantic guards."""
import copy,json,sys,tempfile,unittest
from pathlib import Path
from audit_durability import PROFILE,read,recover,summary,sha,checkpoint
PACKAGE=Path(sys.argv.pop(1)).resolve()
class Audit(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data={}
        for name in ['world_boundary','world_uncertain','world_torn','assay_boundary','assay_uncertain','world_continue']:
            raw=(PACKAGE/'inputs'/(name+'.json')).read_bytes();r=json.loads(raw);values,head,staging=read(PACKAGE/'journals'/name)
            cls.data[name]=(r,sha(raw),values,staging)
        r,h,_,_=cls.data['world_boundary'];cls.source=recover(PACKAGE/'journals/world_boundary',r,h)
    def changed(self,name,edit,source=None):
        r,h,original,staging=self.data[name];values=copy.deepcopy(original);edit(values)
        with tempfile.TemporaryDirectory(prefix='cumulative-audit-') as tmp:
            p=Path(tmp);head=None
            for i,v in enumerate(values):
                raw=(json.dumps({'profile':PROFILE,'sequence':i,'previous_sha256':head,'value':v},sort_keys=True,separators=(',',':'))+'\n').encode();(p/f'{i:08}.json').write_bytes(raw);head=sha(raw)
            for n in staging:(p/n).write_bytes((PACKAGE/'journals'/name/n).read_bytes())
            return recover(p,r,h,source)
    def rejects(self,name,edit,source=None):
        with self.assertRaises((ValueError,KeyError,IndexError,TypeError)):self.changed(name,edit,source)
    def test_01_known_prefix_retains_unreached_denominator(self):
        s=summary(self.source);self.assertFalse(s['native_work_unknown']);self.assertTrue(s['resumable_known_boundary']);self.assertEqual(len(s['slots']),96);self.assertIn('not_started',{v['status'] for v in s['slots']})
    def test_02_pending_work_has_funded_interval(self):
        r,h,_,_=self.data['world_uncertain'];v=recover(PACKAGE/'journals/world_uncertain',r,h);self.assertTrue(v['unknown']);self.assertEqual(v['upper']-v['lower'],v['pending']['work_bound']);self.assertFalse(summary(v)['resumable_known_boundary'])
    def test_03_torn_bytes_never_become_committed_work(self):
        r,h,_,_=self.data['world_torn'];v=recover(PACKAGE/'journals/world_torn',r,h);self.assertTrue(v['unknown']);self.assertEqual(len(v['staging']),1);self.assertEqual(v['lower'],self.data['world_uncertain'][2][-1]['work_before'])
        # A crash after publication can leave an identical staging hard-link.
        import shutil
        with tempfile.TemporaryDirectory() as tmp:
            dest=Path(tmp)/'journal';shutil.copytree(PACKAGE/'journals/world_boundary',dest);files=sorted(dest.iterdir());(dest/f'.staging-{len(files)-1:08}.json').write_bytes(files[-1].read_bytes());r,h,_,_=self.data['world_boundary'];linked=recover(dest,r,h);self.assertFalse(linked['unknown']);self.assertFalse(summary(linked)['resumable_known_boundary'])
    def test_04_reservation_cannot_be_shrunk(self):
        self.rejects('world_uncertain',lambda v:v[-1].update(work_bound=0))
    def test_05_committed_native_counter_cannot_be_changed(self):
        self.rejects('world_boundary',lambda v:v[-1].update(native_rewrites=0,work_total=0))
    def test_06_checkpoint_must_match_same_result_row(self):
        self.rejects('world_boundary',lambda v:v[-1]['checkpoint']['state'].update(reservoir=0))
    def test_07_partial_cannot_be_promoted_to_complete(self):
        self.rejects('world_boundary',lambda v:v.append({'kind':'complete','work_total':self.source['lower'],'work_here':self.source['lower']}))
    def test_08_assay_global_identity_and_admission(self):
        self.rejects('assay_boundary',lambda v:v[-1].update(index=0))
        self.rejects('assay_boundary',lambda v:v[-1]['native'].update(scientific_acceptance=True))
    def test_09_continuation_cannot_drop_ancestry(self):
        self.rejects('world_continue',lambda v:v[1]['cases'][0].update(rows=[]),self.source)
    def test_10_unknown_source_cannot_continue(self):
        r,h,_,_=self.data['world_uncertain'];bad=recover(PACKAGE/'journals/world_uncertain',r,h)
        self.rejects('world_continue',lambda v:None,bad)
if __name__=='__main__':unittest.main()
