"""Regression tests for paid useful paths and fixed Gate0 decision boundaries."""
import copy
import os
from pathlib import Path
import time
import unittest

import verify_evolving_minds_transfer as ta


def fixture(useful=True, depth=3):
    members=[];records=[]
    for founder in [1,2]:
        parent=founder
        for generation in range(depth+2 if useful else 1):
            bid=founder if generation==0 else founder+generation*10
            program={'source':2 if generation==0 else 0}
            m=dict(id=bid,parent=None if generation==0 else parent,
                born_tick=generation,program=program,initial_reserve=64 if generation==0 else 24,
                completion_tick=generation+5,attempts=1,successes=1,failures=0,
                payable_action_opportunities=1,harvested=80 if useful else 8,
                development_charge=5,attempt_charge=1,maintenance_charge=1,
                request_charge=0,copying_charge=0,refunds=0,
                child_endowment_transferred=0)
            if generation:
                records.append(dict(parent=parent,child_id=bid,tick=generation,
                    settled=True,proposed_program=program,request_charge=1,
                    copying_charge=5,refunded=0,target_id=103 if generation==1 else None,
                    old_source=2 if generation==1 else None,new_source=0 if generation==1 else None))
                previous=next(x for x in members if x['id']==parent)
                previous['request_charge']+=1;previous['copying_charge']+=5
            members.append(m);parent=bid
    costs=sum(m[k] for m in members for k in [
        'development_charge','attempt_charge','maintenance_charge','request_charge','copying_charge'])
    return dict(opportunity=dict(lifetimes=members,ticks=[{}]*64,
                    primary_descendant_harvested=sum(m['harvested'] for m in members if m['parent'])),
                records=records,final_row=dict(harvested=sum(m['harvested'] for m in members),dissipated=costs))


class Gate0Tests(unittest.TestCase):
    def test_installation_does_not_count_as_faithful_transmission(self):
        short=ta.endpoints(fixture(depth=2),True)
        self.assertTrue(all(f['max_active_faithful_transmissions']==2 and not f['persistence_pass'] for f in short['families']))
        enough=ta.endpoints(fixture(depth=3),True)
        self.assertTrue(all(f['persistence_pass'] and len(f['witness_path'])==4 for f in enough['families']))

    def test_attempt_without_success_breaks_functional_chain(self):
        result=fixture();m=result['opportunity']['lifetimes'][2]
        m['successes']=0;m['failures']=1
        measured=ta.endpoints(result,True)
        self.assertFalse(measured['families'][0]['persistence_pass'])
        self.assertTrue(measured['families'][1]['persistence_pass'])
        self.assertEqual(measured['active_completed_offspring'],8)

    def test_siblings_are_not_successive_transmissions(self):
        result=fixture()
        for m in result['opportunity']['lifetimes']:
            if m['born_tick']>1:
                founder=m['id']%10;m['parent']=founder+10
                next(r for r in result['records'] if r['child_id']==m['id'])['parent']=m['parent']
        measured=ta.endpoints(result,True)
        self.assertTrue(all(f['max_active_faithful_transmissions']==1 for f in measured['families']))

    def test_paid_edges_and_complete_ancestry_are_required(self):
        for field,value in [('copying_charge',0),('refunded',24),('request_charge',0)]:
            bad=fixture();bad['records'][0][field]=value
            with self.assertRaisesRegex(ValueError,'paid native birth'):ta.endpoints(bad,True)
        bad=fixture();bad['records'].pop()
        with self.assertRaisesRegex(ValueError,'every child'):ta.endpoints(bad,True)

    def test_net_endpoint_counts_all_real_costs_once(self):
        result=fixture();before=ta.endpoints(result,True)
        for m in result['opportunity']['lifetimes']:
            m['refunds']=1000;m['child_endowment_transferred']=1000
        self.assertEqual(before,ta.endpoints(result,True))
        result['final_row']['dissipated']-=1
        with self.assertRaisesRegex(ValueError,'net resource reconciliation'):ta.endpoints(result,True)

    def test_fixed_denominator_recovery_and_failure_stop(self):
        results={c['name']:fixture(c['arm']!='ancestor') for c in ta.cases()}
        decision=ta.decision(results)
        self.assertTrue(decision['gate0_feasibility_pass'])
        self.assertFalse(decision['scientific_acceptance'])
        self.assertFalse(decision['next_gate_automatically_authorized'])
        bad=copy.deepcopy(results);bad['world17_recovery']['records'][0]['copying_charge']=0
        with self.assertRaisesRegex(ValueError,'recovery exact'):ta.decision(bad)
        bad=copy.deepcopy(results);bad.pop('world23_ancestor')
        with self.assertRaisesRegex(ValueError,'denominator'):ta.decision(bad)
        bad=copy.deepcopy(results);bad['world23_useful']=fixture(depth=2)
        self.assertFalse(ta.decision(bad)['gate0_feasibility_pass'])


@unittest.skipUnless(os.environ.get('EVOLVING_MINDS_TRANSFER_FIXTURE'),
                     'requires generated TA Gate0 development fixture')
class NativeReplayTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root=Path(os.environ['EVOLVING_MINDS_TRANSFER_FIXTURE'])
        cls.root=root
        cls.v=ta.load(root/'stream_reference.py','ta_test_stream')
        cls.modules=tuple(ta.load(root/name,'ta_test_'+str(i)) for i,name in enumerate([
            'admission_reference.py','replay.py','d0_reference.py','schedule_reference.py']))
        cls.observer=ta.load(root/'opportunity_reference.py','ta_test_observer')
        cls.shared=ta.load(root/'shared_replay.py','ta_test_shared')
        cls.case=ta.cases()[0]
        cls.chain=cls.v.inspect_segment(root/cls.case['name'],funded_config=cls.case['config'],
                                      handoff_config=cls.case['handoff'])

    def shared_replay(self,chain):
        return self.shared.optimized_cell(chain,self.modules,
            (self.root/'b24_base_rules.toml').read_bytes(),self.case['config'],None,
            self.v,self.observer,time.monotonic(),handoff_config=self.case['handoff'])

    def test_omitted_publication_is_refused(self):
        chain=copy.deepcopy(self.chain);chain[0]['commits'].pop(1)
        with self.assertRaisesRegex(ValueError,'all nine'):self.shared_replay(chain)

    def test_intermediate_physical_state_is_reconstructed(self):
        chain=copy.deepcopy(self.chain)
        chain[0]['commits'][1]['state']['frame']['nodes']=[]
        with self.assertRaises(ValueError):self.shared_replay(chain)

    def test_forged_paid_proposal_is_refused(self):
        chain=copy.deepcopy(self.chain)
        records=[r for b in chain[0]['commits'][1]['bundles'] for r in b['records']]
        self.assertTrue(records)
        records[0]['copying_charge']=0
        with self.assertRaises(ValueError):self.shared_replay(chain)


if __name__=='__main__':
    unittest.main()
