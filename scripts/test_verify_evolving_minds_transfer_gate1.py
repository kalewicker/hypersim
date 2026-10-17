import copy
import os
from pathlib import Path
import time
import unittest
import verify_evolving_minds_transfer_gate1 as ta
import verify_evolving_minds_admission as admission
import verify_evolving_minds_choice as choice
from test_verify_evolving_minds_transfer import fixture

def results():
    rows={}
    for c in ta.cases():
        useful=c['arm'] not in ['ancestor','reversion']
        r=fixture(useful)
        if useful:
            for m in r['opportunity']['lifetimes']:m['program']['source']=0
        rows[c['name']]=r
    return rows

class Gate1Tests(unittest.TestCase):
    def test_schedule_balance_cue_relationship_and_shift(self):
        for pattern in ['Alternating','Blocks4']:
            e=dict(pattern=pattern,cue_mode='Informative')
            table=admission.temporal_schedule(17,e,choice)
            self.assertEqual(sum(table['resources']),32)
            self.assertEqual(table['cues'],table['resources'])
            self.assertEqual(table['coins'],choice.schedule(17,'Informative')['coins'])
            e['cue_mode']='Withheld';masked=admission.temporal_schedule(17,e,choice)
            self.assertEqual(masked['resources'],table['resources']);self.assertFalse(any(masked['cues']))

    def test_causal_intervention_bytes(self):
        ancestor,_=ta.intervention('ancestor');p,reversion=ta.intervention('reversion')
        self.assertEqual(p,ancestor)
        _,useful=ta.intervention('useful');_,restore=ta.intervention('restoration');_,sham=ta.intervention('sham')
        self.assertEqual(useful[-1],restore[-1]);self.assertEqual(useful[-1],sham[-1])
        self.assertNotEqual(reversion[-1]['bytes'],useful[-1]['bytes'])

    def test_controls_are_exact_and_do_not_establish_transfer(self):
        r=results();decision=ta.decision(r)
        self.assertTrue(decision['gate1_development_feasibility_pass']);self.assertFalse(decision['transfer_established'])
        r[ta.case(17,'Blocks4','Informative','sham')['name']]['records'][0]['new_source']=3
        with self.assertRaisesRegex(ValueError,'causal control exact'):ta.decision(r)

    def test_diagnostic_is_not_a_primary_failure_and_missing_cells_refused(self):
        r=results();r[ta.case(17,'Alternating','Withheld','useful')['name']]=fixture(False)
        self.assertTrue(ta.decision(r)['gate1_development_feasibility_pass'])
        r.pop(ta.case(17,'Blocks4','Informative','reversion')['name'])
        with self.assertRaisesRegex(ValueError,'denominator'):ta.decision(r)

    def test_founder_initialization_is_not_counted_as_inheritance(self):
        r=fixture(True,depth=1)
        for m in r['opportunity']['lifetimes']:m['program']['source']=0
        self.assertTrue(all(f['max_active_faithful_transmissions']==2 and not f['persistence_pass'] for f in ta.endpoints(r)['families']))

    def test_evaluation_registration_remains_unexposed(self):
        reg=ta.evaluation_registration();self.assertEqual(reg['cells'],96)
        self.assertFalse(reg['launch_enabled']);self.assertIsNone(reg['scientific_budget'])
        seeds=[b['seed'] for b in reg['blocks']]
        self.assertEqual(len(set(seeds)),8);self.assertNotIn(17,seeds)
        self.assertTrue(all(b['exposure']=='registered_unexposed' for b in reg['blocks']))

@unittest.skipUnless(os.environ.get('EVOLVING_MINDS_TRANSFER_GATE1_FIXTURE'),'requires Gate1 native fixture')
class NativeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=Path(os.environ['EVOLVING_MINDS_TRANSFER_GATE1_FIXTURE'])
        cls.v=ta.load(cls.root/'stream_reference.py','tg1_stream')
        cls.modules=tuple(ta.load(cls.root/n,'tg1_'+str(i)) for i,n in enumerate(['admission_reference.py','replay.py','d0_reference.py','schedule_reference.py']))
        cls.shared=ta.load(cls.root/'shared_replay.py','tg1_shared');cls.o=ta.load(cls.root/'opportunity_reference.py','tg1_observer')
        cls.case=ta.cases()[1]
        cls.chain=cls.v.inspect_segment(cls.root/cls.case['name'],funded_config=cls.case['config'],transfer_environment=cls.case['environment'])
    def replay(self,chain):
        return self.shared.optimized_cell(chain,self.modules,(self.root/'b24_base_rules.toml').read_bytes(),self.case['config'],None,self.v,self.o,time.monotonic(),transfer_environment=self.case['environment'])
    def test_environment_attestation_forgery_refused(self):
        chain=copy.deepcopy(self.chain);chain[0]['commits'][1]['state']['native']['transfer_environment']['pattern']='Blocks4'
        with self.assertRaisesRegex(ValueError,'temporal configuration'):self.replay(chain)
    def test_intermediate_graph_and_missing_publication_refused(self):
        chain=copy.deepcopy(self.chain);chain[0]['commits'][1]['state']['frame']['nodes']=[]
        with self.assertRaises(ValueError):self.replay(chain)
        chain=copy.deepcopy(self.chain);chain[0]['commits'].pop(1)
        with self.assertRaises(ValueError):self.replay(chain)

if __name__=='__main__':unittest.main()
