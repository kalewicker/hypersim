import copy
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import verify_ere_graph_encoder_confirmation as v
from test_verify_ere_rule_relational import observation

ERRORS=(AssertionError,ValueError,KeyError,TypeError,IndexError)
SOURCE=Path('output')/v.SOURCE
def synthetic():
    obs=[v.e.observe(observation(c)) for c in v.p.native.CONTEXTS];matrix=[]
    for c in v.p.native.CONTEXTS:
        correct=[v.p.native.CHOICES[s].index(c[s]) for s in range(2)]
        matrix.extend(0.375 if [x,y]==correct else 0. for x in range(4) for y in range(4))
    initial=v.e.initialize();g=v.p.endpoint('graph',initial,matrix,obs);b=v.p.endpoint('blind',initial,matrix,obs)
    checkpoints=[dict(checkpoint=12),dict(checkpoint=100,final_graph=g,final_blind=b,baseline=g)]
    controls=[dict(context=c,arm=arm,reward=0.375 if arm=='reverse_paid' else 0.) for c in v.p.RESERVED for arm in v.p.native.ARMS[1:]]
    return obs,matrix,checkpoints,controls


class ConfirmationTests(unittest.TestCase):
    def test_design_and_complete_schedule(self):
        self.assertEqual(v.a.load(Path('configs/ere_graph_encoder_confirmation_v1.json')),v.DESIGN)
        plan=list(v.schedule());self.assertEqual(len(plan),396)
        self.assertEqual([sum(r[0]==phase for r in plan) for phase in ['matrix','control','decision']],[288,36,72])
        for i,seed in enumerate(v.p.native.SEEDS):
            matrix=plan[i*144:(i+1)*144];self.assertTrue(all(r[1]==seed for r in matrix))
            for j,c in enumerate(v.p.native.CONTEXTS):
                self.assertEqual([r[4] for r in matrix[16*j:16*j+16]],[[x,y] for x in range(4) for y in range(4)])
                self.assertTrue(all(r[2]==c for r in matrix[16*j:16*j+16]))

    def test_byte_bound_import_and_tamper_refusals(self):
        with tempfile.TemporaryDirectory() as name:
            out=Path(name)
            for dest,src in [('source_output_sha256.json','output_sha256.json'),('checkpoint_import.json','checkpoints.json'),('matrix_import.json','input.json')]:shutil.copyfile(SOURCE/src,out/dest)
            seal=v.a.load(out/'source_output_sha256.json');provenance=dict(source_archive=v.SOURCE,source_root_seal=v.SEAL,checkpoint=100,
                imports={n:dict(source=s,sha256=seal[s]) for n,s in [('checkpoint_import.json','checkpoints.json'),('matrix_import.json','input.json')]})
            (out/'import_provenance.json').write_text(json.dumps(provenance));v.imports(out)
            for file in ['checkpoint_import.json','matrix_import.json','source_output_sha256.json','import_provenance.json']:
                original=(out/file).read_bytes();(out/file).write_text('{}')
                with self.subTest(file=file),self.assertRaises(ERRORS):v.imports(out)
                (out/file).write_bytes(original)

    def test_frozen_policies_and_invalid_parameters(self):
        c=v.a.load(SOURCE/'checkpoints.json');vectors=v.policies(c)
        self.assertEqual([m for m,t in vectors],['graph','blind','graph','blind']);self.assertEqual(vectors[0][1],vectors[3][1])
        self.assertEqual(vectors[2][1],v.e.initialize())
        for mutate in [lambda r:r.pop(),lambda r:r[1].update(checkpoint=12),lambda r:r[1]['final_graph'].update(theta=[0.]),
                       lambda r:r[1]['final_blind']['theta'].__setitem__(0,float('nan'))]:
            bad=copy.deepcopy(c);mutate(bad)
            with self.assertRaises(ERRORS):v.policies(bad)

    def test_decision_mask_draw_action_weight_refusals(self):
        o=v.e.observe(observation(v.p.native.CONTEXTS[0]));theta=v.e.initialize();draws=[0.125,0.875];states=[123,456]
        d,actions=v.decision('learned_graph','graph',theta,o,draws,states);r=dict(decision=d,actions=actions)
        v.verify_decision(r,'learned_graph','graph',theta,o,draws,states)
        for mutate in [lambda r:r['decision'].update(policy='trained_blind'),lambda r:r['decision'].update(mode='blind'),
                       lambda r:r['decision'].update(draws=[0.,0.]),lambda r:r['decision'].update(draw_states=[0,0]),
                       lambda r:r['decision']['input']['observation'].update(adjacency=[[False]*7 for _ in range(7)]),
                       lambda r:r['decision']['theta'].__setitem__(0,1.),lambda r:r['decision']['probabilities'][0].__setitem__(0,1.),
                       lambda r:r.update(actions=[(actions[0]+1)%4,actions[1]])]:
            bad=copy.deepcopy(r);mutate(bad)
            with self.assertRaises(ERRORS):v.verify_decision(bad,'learned_graph','graph',theta,o,draws,states)

    def test_negative_completion_and_endpoint_verdict_refusals(self):
        obs,matrix,c,controls=synthetic();vectors=v.policies(c)
        rows=[v.comparison(seed,vectors,matrix,controls,matrix,c,obs) for seed in v.p.native.SEEDS]
        self.assertFalse(v.verify_comparisons(rows,vectors,[matrix,matrix],[controls,controls],matrix,c,obs)[1])
        for mutate in [lambda r:r.pop(),lambda r:r[0].update(confirmation_pass=True),lambda r:r[0]['flags'].update(graph_reward_pass=True),
                       lambda r:r[0].update(native_seed=0),lambda r:r[0].update(graph_minus_initial=1.),
                       lambda r:r[0]['policies'][0]['endpoint'].update(balanced_expected_reward=1.),
                       lambda r:r[0]['controls'][0].update(reward=1.)]:
            bad=copy.deepcopy(rows);mutate(bad)
            with self.assertRaises(ERRORS):v.verify_comparisons(bad,vectors,[matrix,matrix],[controls,controls],matrix,c,obs)

    def test_wrapper_and_native_ledger_refusals(self):
        native=v.a.load(Path('output/ere_graph_encoder_learning_trial_20261007/native/cell-0000.json'))['native']
        r=dict(id=0,phase='matrix',native_seed=v.p.native.SEEDS[0],context=v.p.native.CONTEXTS[0],arm='live_paid',actions=[0,0],decision=None,native=native)
        args=(0,'matrix',v.p.native.SEEDS[0],v.p.native.CONTEXTS[0],'live_paid',[0,0]);v.verify_bindings(r,*args);v.p.audit_native(native)
        for mutate in [lambda r:r.update(id=1),lambda r:r.update(phase='decision'),lambda r:r.update(context=[10,21]),
                       lambda r:r.update(native_seed=0),lambda r:r.update(actions=[1,1]),lambda r:r['native'].update(arm='frozen_paid')]:
            bad=copy.deepcopy(r);mutate(bad)
            with self.assertRaises(ERRORS):v.verify_bindings(bad,*args)
        for mutate in [lambda r:r.update(adapter_reward=1.),lambda r:r['timeline'][1]['state'].update(fuel=9.),lambda r:r['timeline'].pop()]:
            bad=copy.deepcopy(native);mutate(bad)
            with self.assertRaises(ERRORS):v.p.audit_native(bad)

    @unittest.skipUnless(os.environ.get('ERE_GRAPH_ENCODER_CONFIRMATION_AUDIT_INPUT'),'measured receipts only')
    def test_measured_decision_endpoint_and_ledger_tampers(self):
        out=Path(os.environ['ERE_GRAPH_ENCODER_CONFIRMATION_AUDIT_INPUT']);c,data=v.imports(out);vectors=v.policies(c)
        directory=out/'native';r=v.a.load(directory/'cell-0324.json');n=r['native'];o=v.e.observe(n['initial_inventory']);d=r['decision']
        v.verify_decision(r,v.POLICIES[0],*vectors[0],o,d['draws'],d['draw_states'])
        bad=copy.deepcopy(r);bad['decision']['input']['observation']['adjacency']=[[False]*7 for _ in range(7)]
        with self.assertRaises(ERRORS):v.verify_decision(bad,v.POLICIES[0],*vectors[0],o,d['draws'],d['draw_states'])
        bad=copy.deepcopy(n);bad['timeline'][5]['paid_edits']+=1
        with self.assertRaises(ERRORS):v.p.audit_native(bad)
        rows=v.a.load(directory/'comparisons.json');base=[r for w in data['worlds'] for r in w['rewards']];obs=[v.e.observe(w['raw']) for w in data['worlds']]
        matrices=[[float(v.a.load(directory/f'cell-{i*144+j:04}.json')['native']['adapter_reward']) for j in range(144)] for i in range(2)]
        controls=[r['controls'] for r in rows];v.verify_comparisons(rows,vectors,matrices,controls,base,c,obs)
        for mutate in [lambda r:r[0].update(confirmation_pass=not r[0]['confirmation_pass']),
                       lambda r:r[0]['policies'][0]['endpoint'].update(balanced_expected_reward=0.)]:
            bad=copy.deepcopy(rows);mutate(bad)
            with self.assertRaises(ERRORS):v.verify_comparisons(bad,vectors,matrices,controls,base,c,obs)


if __name__=='__main__':unittest.main()
