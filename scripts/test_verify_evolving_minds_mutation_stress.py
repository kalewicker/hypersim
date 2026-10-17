import copy
import os
from pathlib import Path
import time
import unittest
import verify_evolving_minds_mutation_stress as stress
import verify_evolving_minds_admission as a
import verify_evolving_minds_d0 as d
from test_verify_evolving_minds_transfer import fixture

def sample(useful=True):
    r=fixture(useful)
    for m in r['opportunity']['lifetimes']:
        m.update(attempts=4,successes=4,failures=0,payable_action_opportunities=4,attempt_charge=4,development_failures=0)
    for q in r['records']:q.update(syntax_valid=True,material_reachable=True)
    r['final_row']['dissipated']+=3*len(r['opportunity']['lifetimes'])
    return r

def results():
    return {c['name']:sample(c['arm']!='ancestor') for c in stress.cases()}

class StressTests(unittest.TestCase):
    def test_grammar_set_does_not_filter_material_failures(self):
        p=d.program('delayed_cue');next(g for g in p['instructions'] if g['id']==103)['source']=0
        choices=a.grammar_candidates(p);self.assertEqual(len(choices),14)
        self.assertTrue(any(not a.reachable(q) for _,_,q in choices))
        self.assertTrue(all(a.typed_valid(q) for _,_,q in choices))
        for seed in range(128):
            rng=a.Sampler(seed,'GrammarSourceKernel');q,target,old,new,raw,start=rng.proposal(p)
            record=dict(request_id=5,parent_program=p,raw_draws=raw,target_id=target,new_source=new)
            cert=a.grammar_certificate(record,d)
            self.assertEqual(target is not None,raw[0]%4!=0)
            self.assertEqual(cert['candidate_count'],14)
            self.assertEqual(cert['each_candidate_probability_denominator'],56)
    def test_empty_set_is_explicit_identity(self):
        p=dict(instructions=[d.program('delayed_cue')['instructions'][-1]])
        self.assertEqual(a.grammar_candidates(p),[])
        q,target,old,new,raw,start=a.Sampler(17,'GrammarSourceKernel').proposal(p)
        cert=a.grammar_certificate(dict(request_id=1,parent_program=p,raw_draws=raw,target_id=target,new_source=new),d)
        self.assertEqual(q,p);self.assertIsNone(target);self.assertEqual(len(raw),1)
        self.assertEqual(cert['identity_probability_denominator'],1)
    def test_behavior_can_persist_across_changed_genotypes(self):
        summary=stress.summarize(sample())
        self.assertTrue(all(f['persistence_pass'] for f in summary['families']))
        self.assertTrue(any(f['changed_genotype_edges']>0 for f in summary['families']))
    def test_function_quality_and_minimum_paid_exposure_are_required(self):
        r=sample();m=r['opportunity']['lifetimes'][2]
        m.update(successes=2,failures=2)
        self.assertFalse(stress.summarize(r)['families'][0]['persistence_pass'])
        r=sample()
        for m in r['opportunity']['lifetimes']:m.update(attempts=3,successes=3,payable_action_opportunities=3)
        self.assertFalse(any(f['persistence_pass'] for f in stress.summarize(r)['families']))
    def test_fixed_grammar_gate_does_not_select_best_kernel_after_outcomes(self):
        r=results();self.assertTrue(stress.decision(r)['development_advance_screen_pass'])
        for c in stress.cases():
            if c['arm']=='source':r[c['name']]=sample(False)
        decision=stress.decision(r)
        self.assertFalse(decision['original_source_retention_screen_pass'])
        self.assertTrue(decision['grammar_retention_screen_pass'])
        r['world23_Blocks4_grammar']=sample(False)
        self.assertFalse(stress.decision(r)['development_advance_screen_pass'])
        r.pop('world23_Blocks4_ancestor')
        with self.assertRaisesRegex(ValueError,'denominator'):stress.decision(r)
    def test_probe_recovery_never_claims_retention_or_science(self):
        r={c['name']:sample() for c in stress.cases(True)}
        decision=stress.decision(r,True)
        self.assertTrue(decision['probe_packaging_and_recovery_pass'])
        self.assertFalse(decision['development_screen_measured']);self.assertFalse(decision['grammar_retention_screen_pass'])
        r['world17_Alternating_recovery']['records'][0]['copying_charge']=0
        with self.assertRaisesRegex(ValueError,'recovery exact'):stress.decision(r,True)

@unittest.skipUnless(os.environ.get('EVOLVING_MINDS_STRESS_FIXTURE'),'requires native mutation-stress fixture')
class NativeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=Path(os.environ['EVOLVING_MINDS_STRESS_FIXTURE']);cls.v=stress.load(cls.root/'stream_reference.py','stress_test_stream')
        cls.modules=tuple(stress.load(cls.root/n,'stress_test_'+str(i)) for i,n in enumerate(['admission_reference.py','replay.py','d0_reference.py','schedule_reference.py']))
        cls.o=stress.load(cls.root/'opportunity_reference.py','stress_test_observer');cls.shared=stress.load(cls.root/'shared_replay.py','stress_test_shared')
        cls.cfg=stress.cases(True)[0]
        cls.chain=cls.v.inspect_segment(cls.root/cls.cfg['name'],funded_config=cls.cfg['config'],transfer_environment=cls.cfg['environment'],mutation_stress_policy=stress.POLICY)
    def replay(self,chain):
        return self.shared.optimized_cell(chain,self.modules,(self.root/'b24_base_rules.toml').read_bytes(),self.cfg['config'],None,self.v,self.o,time.monotonic(),transfer_environment=self.cfg['environment'],mutation_stress_policy=stress.POLICY)
    def test_policy_and_omitted_publication_refused(self):
        chain=copy.deepcopy(self.chain);chain[0]['commits'][1]['state']['native']['mutation_stress_policy']='unknown'
        with self.assertRaises(ValueError):self.replay(chain)
        chain=copy.deepcopy(self.chain);chain[0]['commits'].pop(1)
        with self.assertRaises(ValueError):self.replay(chain)
    def test_native_candidate_certificates_reconstruct_exactly(self):
        chain=self.chain
        records=[r for b in chain[0]['bundles'] for r in b['records'] if r['operator']=='grammar_source']
        expected=[self.modules[0].grammar_certificate(r,self.modules[2]) for r in records]
        actual=stress.data(self.root/'grammar_certificates.json')[self.cfg['name']]
        self.assertTrue(expected);self.assertEqual(actual,expected)
        forged=copy.deepcopy(actual);forged[0]['candidate_count']+=1
        self.assertNotEqual(forged,expected)

if __name__=='__main__':unittest.main()
