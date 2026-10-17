#!/usr/bin/env python3
import os
from pathlib import Path
import unittest
from unittest.mock import patch
import verify_ere_resource_contrast_adapter as v

class Contrast(unittest.TestCase):
    def root(self):return Path(os.environ['ERE_RESOURCE_CONTRAST_PREFLIGHT'])
    def data(self):return v.old.a.load(self.root()/'contrast_records.json')
    def reject(self,data):
        original=v.old.a.load
        with patch.object(v.old.a,'load',side_effect=lambda p:data if Path(p).name=='contrast_records.json' else original(p)):
            with self.assertRaises(AssertionError):v.audit(self.root())
    def test_full(self):
        result=v.audit(self.root());self.assertEqual(result['derivatives'],768);self.assertEqual(result['full_logit_sign_checks'],8)
    def test_feature_mask_and_wallet(self):
        for w in [0,1]:
            o=dict(wallet=w,forecast=0,bounty=2)
            self.assertEqual(v.features(o,'full'),[1,-1,-1,1])
            self.assertEqual(v.features(o,'cues_masked'),[1,0,0,0])
        bad=self.data();bad['records'][0]['input']=[1,1,1,1];self.reject(bad)
        bad=self.data();bad['records'][3]['contributions']=[[0]*4]*2;self.reject(bad)
    def test_scores_credit_and_requests(self):
        bad=self.data();bad['records'][0]['scores'][0]=[0]*4;self.reject(bad)
        bad=self.data();bad['records'][0]['requests'][0]['decision']['step']=14;self.reject(bad)
        bad=self.data();bad['records'][8]['requests'][0]['decision']['request']='defend';self.reject(bad)
        bad=self.data();bad['records'][8]['contributions'][0]=[1]*4;self.reject(bad)
    def test_gradient_sign_and_domain(self):
        bad=self.data();bad['batches'][0]['directions'][0]['actual']=-.0375;self.reject(bad)
        for o in [dict(wallet=2,forecast=0,bounty=2),dict(wallet=1,forecast=2,bounty=2),dict(wallet=1,forecast=1,bounty=3)]:
            with self.assertRaises(AssertionError):v.features(o,'cues_masked')
    def test_counts_and_flags(self):
        for key in ['records','batches']:
            bad=self.data();bad[key].pop();self.reject(bad)
        bad=self.data();bad['native_advances']=1;self.reject(bad)
        bad=self.data();bad['records'][0]['utility']=[20,20];self.reject(bad)
    def test_custody_corruption(self):
        original=v.old.a.digest
        with patch.object(v.old.a,'digest',side_effect=lambda p:'0'*64 if Path(p).name=='campaign_seal.json' else original(p)):
            with self.assertRaises(AssertionError):v.custody(self.root())
if __name__=='__main__':unittest.main()
