import copy
import math
import os
from pathlib import Path
import unittest
import verify_ere_graph_encoder as v
from test_verify_ere_rule_relational import observation

ERRORS=(AssertionError,ValueError,KeyError,TypeError,IndexError)


def samples(mode):
    rows=[]
    for c in v.previous.DEVELOPMENT:
        inp=v.make_input(mode,v.observe(observation(c)))
        actions=[v.previous.native.CHOICES[s].index(c[s]) for s in range(2)]
        rows.extend([dict(context=c,input=inp,actions=actions,reward=0.375)]*2)
    return rows


class EncoderTests(unittest.TestCase):
    def test_raw_input_invariance_and_fourteen_refusals(self):
        for c in v.previous.native.CONTEXTS:
            raw=observation(c);expected=v.observe(raw);reordered=copy.deepcopy(raw)
            reordered['nodes'].reverse();reordered['edges'].reverse()
            for _,edge in reordered['edges']:edge['nodes'].reverse()
            self.assertEqual(v.observe(reordered),expected)
            remapped=copy.deepcopy(raw)
            for row in remapped['nodes']:
                i=int(row[0][7:row[0].index('v')]);row[0]=f'NodeId({i+500}v9)'
            for _,edge in remapped['edges']:
                for n in edge['nodes']:n['idx']+=500;n['version']=9
            self.assertEqual(v.observe(remapped),expected)
            metadata=copy.deepcopy(raw);metadata['nodes'].append(['NodeId(999999v1)',dict(data=dict(label=v.previous.native.META+9,answer=c))])
            metadata.update(context=[30,31],features=[[0,0,1]]*2);self.assertEqual(v.observe(metadata),expected)
            for name in v.VARIANTS:self.assertEqual(v.observe(v.variant(reordered,c,name)),v.observe(v.variant(raw,c,name)))
        raw=observation([10,11]);mutations=[lambda r:r.update(step=1),lambda r:r['nodes'].pop(),
            lambda r:r['nodes'].append(copy.deepcopy(r['nodes'][0])),lambda r:r['nodes'][0][1]['data'].update(label=99),
            lambda r:r['nodes'][0].__setitem__(0,'bad'),lambda r:r['edges'][0][1].update(label=1),
            lambda r:r['edges'][0][1]['nodes'].__setitem__(1,copy.deepcopy(r['edges'][0][1]['nodes'][0])),
            lambda r:r['edges'].append(copy.deepcopy(r['edges'][0])),lambda r:r['edges'][0][1]['nodes'][0].update(idx=999999),
            lambda r:r['edges'][0][1]['nodes'][0].update(version=0),lambda r:r['edges'][0].__setitem__(0,'bad'),
            lambda r:r['edges'][0][1]['nodes'][0].update(extra=1),lambda r:r.update(step=True),
            lambda r:r['nodes'][0][1]['data'].update(label=True)]
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(raw);mutate(r)
            with self.subTest(i=i),self.assertRaises(ERRORS):v.observe(r)
        # Raw observation is not an active-path validator.
        raw['edges']=[];self.assertEqual(v.observe(raw)['adjacency'],[[False]*7 for _ in range(7)])
        mixed=observation([10,11]);mixed['nodes'].append(['NodeId(999999v1)',dict(data=dict(label=v.previous.native.META+1))])
        mixed['edges'][0][1]['nodes'][0]=dict(idx=999999,version=1)
        with self.assertRaises(ERRORS):v.observe(mixed)

    def test_forward_dual_gradients_all_parameters_and_blind_independence(self):
        theta=v.initialize();reference=None
        for c in v.previous.native.CONTEXTS:
            o=v.observe(observation(c))
            for mode in v.MODES:
                inp=v.make_input(mode,o);p,lp,g=v.statistics(theta,inp,[1,2])
                if mode=='blind':
                    if reference is None:reference=(p,lp,g)
                    self.assertEqual((p,lp,g),reference)
                for i in range(110):
                    hi=theta.copy();lo=theta.copy();hi[i]+=1e-6;lo[i]-=1e-6
                    fd=(v.statistics(hi,inp,[1,2],False)[1]-v.statistics(lo,inp,[1,2],False)[1])/2e-6
                    self.assertLess(abs(fd-g[i]),2e-8)
                for a in range(4):
                    for b in range(4):
                        draws=[math.fsum(p[s][:action])+p[s][action]/2 for s,action in enumerate([a,b])]
                        self.assertEqual(v.sample(p,draws),[a,b])

    def test_dead_zero_initialization_and_teacher_selected_update_refusals(self):
        theta=v.initialize()
        for mode in v.MODES:
            batch=samples(mode);inp=batch[0]['input'];p,_,g=v.statistics([0.]*110,inp,[1,2])
            self.assertEqual(p,[[0.25]*4]*2);self.assertEqual(g[:108],[0.]*108)
            next_theta=v.update(theta,mode,batch)
            self.assertGreater(math.fsum(v.statistics(next_theta,s['input'],s['actions'],False)[1] for s in batch),
                math.fsum(v.statistics(theta,s['input'],s['actions'],False)[1] for s in batch))
            if mode=='graph':self.assertNotEqual(next_theta[:104],theta[:104])
            zero=copy.deepcopy(batch)
            for s in zero:s['reward']=0.
            self.assertEqual(v.update(theta,mode,zero),theta)
            for mutate in (lambda r:r.pop(),lambda r:r[0].update(context=v.previous.RESERVED[0]),
                           lambda r:r[0].update(context=v.previous.DEVELOPMENT[1]),lambda r:r[0].update(reward=float('nan')),
                           lambda r:r[0].update(reward=-0.1),lambda r:r[0]['input'].update(mode='blind' if mode=='graph' else 'graph'),
                           lambda r:r[0].update(actions=[4,0])):
                r=copy.deepcopy(batch);mutate(r)
                with self.assertRaises(ERRORS):v.update(theta,mode,r)
            for rate in (0.,-1.,float('nan'),5.):
                with self.assertRaises(ERRORS):v.update(theta,mode,batch,rate)

    def test_parameter_mask_and_blind_bound_refusals(self):
        inp=v.make_input('graph',v.observe(observation([10,11])));theta=v.initialize()
        for bad in (theta[:109],[float('nan')]*110,[1e308]*110):
            with self.assertRaises(ERRORS):v.statistics(bad,inp,[0,0])
        bad=copy.deepcopy(inp);bad['mode']='blind'
        with self.assertRaises(ERRORS):v.statistics(theta,bad)
        for a in range(4):
            for b in range(4):
                value=math.fsum(0.375/9 if [a,b]==[v.previous.native.CHOICES[s].index(c[s]) for s in range(2)] else 0. for c in v.previous.native.CONTEXTS)
                self.assertLessEqual(value,1/24)

    @unittest.skipUnless(os.environ.get('ERE_GRAPH_ENCODER_AUDIT_INPUT'),'saved encoder panel only')
    def test_saved_complete_engineering_accounting(self):
        out=Path(os.environ['ERE_GRAPH_ENCODER_AUDIT_INPUT']);prior=Path(os.environ['ERE_GRAPH_ENCODER_PRIOR'])
        result=v.audit(out,prior);self.assertTrue(result['accounting_pass']);self.assertEqual(result['native_steps'],0)
        self.assertEqual(result['forward_mode_gradients_checked'],864);self.assertFalse(result['learned_encoder_performance_claim'])

    @unittest.skipUnless(os.environ.get('ERE_GRAPH_ENCODER_AUDIT_INPUT'),'saved encoder panel only')
    def test_sixteen_detached_panel_corruptions(self):
        out=Path(os.environ['ERE_GRAPH_ENCODER_AUDIT_INPUT']);panel=v.a.load(out/'encoder.json')
        raws=[v.a.load(out/'inputs'/f'world-{i}.json') for i in range(9)]
        mutations=[lambda r:r.update(native_steps=1),lambda r:r.update(training_campaign_launched=True),
            lambda r:r['theta'].__setitem__(0,1.),lambda r:r['records'].pop(),
            lambda r:r['records'][0].update(world=1),lambda r:r['records'][0].update(context=[10,21]),
            lambda r:r['records'][0].update(variant='lure_edge'),lambda r:r['records'][0].update(mode='blind'),
            lambda r:r['records'][0]['raw']['edges'].pop(),lambda r:r['records'][0]['input']['observation']['adjacency'][0].__setitem__(0,True),
            lambda r:r['records'][1]['input']['observation']['adjacency'][0].__setitem__(1,True),
            lambda r:r['records'][0]['probabilities'][0].__setitem__(0,1.),
            lambda r:r['records'][0]['actions'][0].update(actions=[1,0]),
            lambda r:r['records'][0]['actions'][0].update(log_probability=0.),
            lambda r:r['records'][0]['actions'][0]['score_gradient'].__setitem__(0,1.),
            lambda r:r['diagnostics'][0]['after'].__setitem__(0,1.)]
        for i,mutate in enumerate(mutations):
            r=copy.deepcopy(panel);mutate(r)
            with self.subTest(i=i),self.assertRaises(ERRORS):v.verify_panel(r,raws)


if __name__=='__main__':unittest.main()
