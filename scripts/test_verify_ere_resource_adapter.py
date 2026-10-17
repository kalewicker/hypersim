#!/usr/bin/env python3
import copy
import os
from pathlib import Path
import unittest
from unittest.mock import patch
import verify_ere_resource_adapter as v

class AdapterAudit(unittest.TestCase):
    def root(self):
        return Path(os.environ['ERE_RESOURCE_ADAPTER_ARCHIVE'])
    def test_full_archive(self):
        r=v.audit(self.root());self.assertTrue(r['engineering_pass']);self.assertEqual(r['derivatives'],384)
    def test_record_tampering(self):
        original=v.load;data=original(self.root()/'adapter_records.json')
        mutations=[('state',9),('mode','bogus'),('input',[1,1,0,0]),('probabilities',[.5,.5]),
            ('neutral',[.4,.6]),('utility',[8,3]),('expected_utility',20),('utility_gradient',[0,0,0,0]),
            ('boundary_sample',0),('observation',dict(wallet=0,forecast=0,bounty=2))]
        for key,value in mutations:
            bad=copy.deepcopy(data);bad['records'][0][key]=value
            with patch.object(v,'load',side_effect=lambda p:bad if Path(p).name=='adapter_records.json' else original(p)):
                with self.assertRaises(AssertionError,msg=key):v.audit(self.root())
        for key,value in [('action',1),('sample',1),('request','harvest'),('gradient',[0,0,0,0])]:
            bad=copy.deepcopy(data);bad['records'][0]['actions'][0][key]=value
            with patch.object(v,'load',side_effect=lambda p:bad if Path(p).name=='adapter_records.json' else original(p)):
                with self.assertRaises(AssertionError,msg=key):v.audit(self.root())
    def test_projection_refusals(self):
        self.assertEqual(v.node_key('NodeId(0v1)'),(0,1))
        for key in ['NodeId(01v1)','NodeId(1v0)','NodeId1v1','NodeId(-1v1)']:
            with self.assertRaises(AssertionError):v.node_key(key)
        raw=v.load(self.root()/'inputs/cell-0000.json')['allocation_observation']['inventory']
        for kind in ['step','missing','duplicate','wallet_type','pending','credits','linked']:
            bad=copy.deepcopy(raw)
            if kind=='step':bad['step']=0
            else:
                label={'wallet_type':4293949189,'pending':4293949188,'credits':4293949190}.get(kind,4293949189)
                row=next(r for r in bad['nodes'] if r[1]['data']['label']==label)
                if kind=='missing':bad['nodes'].remove(row)
                elif kind=='duplicate':bad['nodes'].append(copy.deepcopy(row))
                elif kind=='wallet_type':row[1]['data']['properties']=[[1,{'Int':1}]]
                elif kind=='pending':row[1]['data']['properties']=[[30468,{'Int':1}],[30469,{'Int':1}]]
                elif kind=='credits':row[1]['data']['properties']=[[30470,{'Int':2}],[30471,{'Int':1}]]
                else:row[1]['incident']=['EdgeId(99v1)']
            with self.assertRaises((AssertionError,KeyError),msg=kind):v.projection(bad)
    def test_projection_noninterference(self):
        raw=v.load(self.root()/'inputs/cell-0000.json')['allocation_observation']['inventory']
        want=v.projection(raw);raw=copy.deepcopy(raw)
        raw['scenario']='safe_high';raw['action']='harvest';raw['wallet_units']=99
        raw['nodes'].reverse();raw['edges'].reverse()
        for row in raw['nodes']:row[1]['data']['properties'].reverse()
        self.assertEqual(v.projection(raw),want)
    def test_fixed_denominators(self):
        original=v.load;data=original(self.root()/'adapter_records.json')
        for key,value in [('records',data['records'][:-1]),('theta',[0,0,0,0]),('native_advances',1),('optimizer_updates',1)]:
            bad=copy.deepcopy(data);bad[key]=value
            with patch.object(v,'load',side_effect=lambda p:bad if Path(p).name=='adapter_records.json' else original(p)):
                with self.assertRaises(AssertionError,msg=key):v.audit(self.root())
    def test_input_custody(self):
        original=v.digest
        with patch.object(v,'digest',side_effect=lambda p:'0'*64 if Path(p).name=='cell-0000.json' else original(p)):
            with self.assertRaises(AssertionError):v.audit(self.root())
if __name__=='__main__':unittest.main()
