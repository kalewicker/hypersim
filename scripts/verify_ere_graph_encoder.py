#!/usr/bin/env python3
"""Independent raw-observation and forward-mode AD engineering verifier."""
import argparse
import json
import math
from pathlib import Path
import re
import verify_ere_rule_relational_learning as previous

a=previous.a
ROOT=Path(__file__).resolve().parents[1]
LABELS=[10,11,12,20,21,30,31]
PROFILE='E2-graph-encoder-engineering-v1'
INPUT_NAME='ere_rule_relational_learning_trial_20261004'
INPUT_SEAL='a0ccb05305ecaaa151be4bb110f6235b8c375aa50b5c3f93837dddb70f5f22a8'
SEED=993748720001
VARIANTS=['base','lure_edge','lure_chain']
MODES=['graph','blind']
DESIGN=dict(profile=PROFILE,ordinary_labels=LABELS,hidden_width=4,message_rounds=2,parameters=110,
    initialization_seed=SEED,initialization_scale=0.15,modes=MODES,topology_variants=VARIANTS,
    worlds=9,policy_records=54,joint_action_records=864,finite_difference_epsilon=1e-6,
    finite_difference_tolerance=2e-8,arithmetic_tolerance=2e-12,update_diagnostic_rate=0.001,
    batch_denominator=12,diagnostic_updates=2,maximum_native_attempts=0,
    training_campaign_launched=False,training_admitted=False,scientific_acceptance=False,independent_physics_replay=False)


def valid_key(key,kind):
    assert type(key) is str
    match=re.fullmatch(kind+r'\(([0-9]+)v([0-9]+)\)',key)
    assert match and int(match[1])<=2**64-1 and 0<int(match[2])<=2**64-1
    return key


def endpoint(v):
    assert type(v) is dict and set(v)=={'idx','version'}
    assert type(v['idx']) is int and 0<=v['idx']<=2**64-1
    assert type(v['version']) is int and 0<v['version']<=2**64-1
    return f"NodeId({v['idx']}v{v['version']})"


def observe(raw):
    assert type(raw['step']) is int and raw['step']==0
    ids={};ordinary={}
    for row in raw['nodes']:
        assert type(row) is list and len(row)==2
        key=valid_key(row[0],'NodeId');label=row[1]['data']['label']
        assert type(label) is int and 0<=label<=2**32-1 and key not in ids
        ids[key]=label
        if label<previous.native.META:
            assert label not in ordinary;ordinary[label]=key
    assert sorted(ordinary)==LABELS
    positions={ordinary[label]:i for i,label in enumerate(LABELS)}
    adjacent=[[False]*7 for _ in range(7)];seen=set()
    for row in raw['edges']:
        assert type(row) is list and len(row)==2
        key=valid_key(row[0],'EdgeId');assert key not in seen;seen.add(key)
        nodes=[endpoint(v) for v in row[1]['nodes']];assert all(n in ids for n in nodes)
        ords=[positions[n] for n in nodes if n in positions]
        if not ords:continue
        assert len(ords)==len(nodes)==2 and ords[0]!=ords[1] and not adjacent[ords[0]][ords[1]]
        assert type(row[1]['label']) is int and row[1]['label']==0
        adjacent[ords[0]][ords[1]]=adjacent[ords[1]][ords[0]]=True
    return dict(labels=LABELS,adjacency=adjacent)


def make_input(mode,o):
    assert mode in MODES
    return dict(mode=mode,observation=o if mode=='graph' else dict(labels=LABELS,adjacency=[[False]*7 for _ in range(7)]))


def initialize():
    state=SEED;theta=[]
    for _ in range(110):state,d=previous.next_draw(state);theta.append(0.15*(2*d-1))
    return theta


# Rust uses reverse-mode backpropagation. This separate implementation carries
# forward derivative vectors for every weight, avoiding the same backward code.
class Dual:
    def __init__(self,value,gradient=None):self.v=value;self.g=[0.]*110 if gradient is None else gradient
    def __add__(self,other):return Dual(self.v+other.v,[x+y for x,y in zip(self.g,other.g)])
    def __mul__(self,other):return Dual(self.v*other.v,[x*other.v+y*self.v for x,y in zip(self.g,other.g)])
    def tanh(self):
        value=math.tanh(self.v);assert math.isfinite(self.v)
        return Dual(value,[(1-value*value)*v for v in self.g])


def logits(theta,inp,derivatives=True):
    assert len(theta)==110 and all(type(t) is float and math.isfinite(t) for t in theta)
    assert inp['mode'] in MODES and inp['observation']['labels']==LABELS
    adjacency=inp['observation']['adjacency']
    assert len(adjacency)==7 and all(len(row)==7 and all(type(v) is bool for v in row) for row in adjacency)
    assert all(not adjacency[i][i] and all(adjacency[i][j]==adjacency[j][i] for j in range(7)) for i in range(7))
    if inp['mode']=='blind':assert not any(any(row) for row in adjacency)
    if derivatives:
        weights=[Dual(t,[float(i==j) for j in range(110)]) for i,t in enumerate(theta)]
        add=lambda x,y:x+y;mul=lambda x,y:x*y;tanh=lambda x:x.tanh()
    else:
        weights=theta;add=lambda x,y:x+y;mul=lambda x,y:x*y;tanh=lambda x:math.tanh(x) if math.isfinite(x) else (_ for _ in ()).throw(ValueError('overflow'))
    hidden=[[tanh(add(weights[i*4+k],weights[28+k])) for k in range(4)] for i in range(7)]
    for r in range(2):
        offset=32+36*r;updated=[]
        for i in range(7):
            row=[]
            for k in range(4):
                z=weights[offset+32+k]
                for j in range(4):
                    z=add(z,mul(hidden[i][j],weights[offset+j*4+k]))
                    for n in range(7):
                        if adjacency[i][n]:z=add(z,mul(hidden[n][j],weights[offset+16+j*4+k]))
                row.append(tanh(z))
            updated.append(row)
        hidden=updated
    rows=[]
    for s in range(2):
        row=[weights[108]]
        for label in previous.native.CHOICES[s][1:]:
            i=LABELS.index(label);z=weights[109]
            for k in range(4):z=add(z,mul(weights[104+k],hidden[i][k]))
            row.append(z)
        rows.append(row)
    return rows


def statistics(theta,inp,actions=None,derivatives=True):
    rows=logits(theta,inp,derivatives);values=[[v.v if derivatives else v for v in row] for row in rows]
    p=[];normalizers=[]
    for row in values:
        maximum=max(row);assert all(math.isfinite(v) and math.isfinite(v-maximum) for v in row)
        total=math.fsum(math.exp(v-maximum) for v in row);p.append([math.exp(v-maximum)/total for v in row]);normalizers.append((maximum,math.log(total)))
    if actions is None:return p
    assert len(actions)==2 and all(type(a) is int and 0<=a<4 for a in actions)
    lp=math.fsum(values[s][actions[s]]-normalizers[s][0]-normalizers[s][1] for s in range(2));assert math.isfinite(lp)
    g=None if not derivatives else [math.fsum((int(actions[s]==a)-p[s][a])*rows[s][a].g[i] for s in range(2) for a in range(4)) for i in range(110)]
    return p,lp,g


def sample(p,draws):
    assert len(draws)==2 and all(type(d) is float and math.isfinite(d) and 0<=d<1 for d in draws)
    actions=[]
    for row,d in zip(p,draws):
        total=0.;chosen=None
        for i,q in enumerate(row):
            total+=q
            if d<total:chosen=i;break
        if chosen is None:chosen=max(i for i,q in enumerate(row) if q>0)
        actions.append(chosen)
    return actions


def update(theta,mode,samples,rate=0.001):
    assert type(rate) is float and math.isfinite(rate) and 0<rate<=4
    assert len(samples)==12 and all(sum(s['context']==c for s in samples)==2 for c in previous.DEVELOPMENT)
    terms=[[] for _ in range(110)]
    for s in samples:
        assert s['input']['mode']==mode and type(s['reward']) is float and math.isfinite(s['reward']) and 0<=s['reward']<=1
        _,_,g=statistics(theta,s['input'],s['actions'])
        for i,v in enumerate(g):terms[i].append(rate*s['reward']*v/12)
    result=[theta[i]+math.fsum(t) for i,t in enumerate(terms)];statistics(result,samples[0]['input'],derivatives=False);return result


def variant(raw,context,name):
    import copy
    raw=copy.deepcopy(raw);lures=[label for choices in previous.native.CHOICES for label in choices[1:] if label not in context]
    ids={v['data']['label']:key for key,v in raw['nodes']}
    def ref(label):
        match=re.fullmatch(r'NodeId\(([0-9]+)v([0-9]+)\)',ids[label]);return dict(idx=int(match[1]),version=int(match[2]))
    pairs={'base':[],'lure_edge':[(0,1)],'lure_chain':[(0,1),(1,2)]}[name]
    for i,(x,y) in enumerate(pairs):raw['edges'].append([f'EdgeId({999001+i}v1)',dict(label=0,nodes=[ref(lures[x]),ref(lures[y])])])
    return raw


def verify_panel(panel,raws):
    assert panel['profile']==PROFILE and panel['native_steps']==0 and panel['training_campaign_launched'] is False and panel['scientific_acceptance'] is False
    theta=initialize();previous.close(panel['theta'],theta);assert len(panel['records'])==54 and len(raws)==9
    records=panel['records'];index=0;blind=None;graphs=[]
    for world,c in enumerate(previous.native.CONTEXTS):
        for name in VARIANTS:
            raw=variant(raws[world],c,name);o=observe(raw)
            for mode in MODES:
                r=records[index];index+=1
                assert r['world']==world and r['context']==c and r['variant']==name and r['mode']==mode and r['raw']==raw
                inp=make_input(mode,o);assert r['observation']==o and r['input']==inp
                z=logits(theta,inp);values=[[t.v for t in row] for row in z]
                p=statistics(theta,inp,derivatives=False);previous.close(r['probabilities'],p)
                if mode=='blind':
                    if blind is None:blind=inp,p
                    assert (inp,p)==blind
                elif name=='base':graphs.append(p)
                assert len(r['actions'])==16
                for i,ar in enumerate(r['actions']):
                    actions=[i//4,i%4];assert ar['actions']==actions
                    lp=math.fsum(values[s][actions[s]]-max(values[s])-math.log(math.fsum(math.exp(v-max(values[s])) for v in values[s])) for s in range(2))
                    g=[math.fsum((int(actions[s]==a)-p[s][a])*z[s][a].g[j] for s in range(2) for a in range(4)) for j in range(110)]
                    draws=[math.fsum(p[s][:action])+p[s][action]/2 for s,action in enumerate(actions)]
                    previous.close(ar,dict(actions=actions,draws=draws,sampled_actions=sample(p,ar['draws']),log_probability=lp,score_gradient=g))
    assert len({str(p) for p in graphs})==9,'fixed initial network must be connectivity-sensitive'
    assert len({str(records[6*i]['observation']) for i in range(9)})==9
    assert len(panel['diagnostics'])==2
    for mode,diagnostic in zip(MODES,panel['diagnostics']):
        samples=[]
        for r in records:
            if r['variant']=='base' and r['mode']==mode and r['context'] in previous.DEVELOPMENT:
                actions=[previous.native.CHOICES[s].index(r['context'][s]) for s in range(2)]
                samples.extend([dict(context=r['context'],input=r['input'],actions=actions,reward=0.375)]*2)
        after=update(theta,mode,samples)
        before_lp=math.fsum(statistics(theta,s['input'],s['actions'],False)[1]/12 for s in samples)
        after_lp=math.fsum(statistics(after,s['input'],s['actions'],False)[1]/12 for s in samples)
        assert after_lp>before_lp
        previous.close(diagnostic,dict(mode=mode,before=theta,after=after,mean_log_probability_before=before_lp,mean_log_probability_after=after_lp,synthetic_teacher_selected=True,native_steps=0))
    return dict(policy_records=54,joint_action_records=864,forward_mode_gradients_checked=864,graph_base_outputs_distinct=9,
        blind_inputs_world_independent=True,diagnostic_updates=2,training_campaign_launched=False,native_steps=0)


def audit(directory,input_root):
    directory=Path(directory);input_root=Path(input_root);a.verify(input_root,expected=INPUT_SEAL,count=2900)
    m=a.load(directory/'manifest.json');assert m['design']==DESIGN and m['maximum_native_attempts']==0
    for key in ('training_campaign_launched','training_admitted','scientific_acceptance','independent_physics_replay'):assert m[key] is False
    raws=[];rewards=[]
    for world,c in enumerate(previous.native.CONTEXTS):
        row=a.load(input_root/'native'/f'cell-{world*16:04}.json');raw=row['native']['initial_inventory'];raws.append(raw)
        assert a.load(directory/'inputs'/f'world-{world}.json')==raw
        for i in range(16):
            cell=a.load(input_root/'native'/f'cell-{world*16+i:04}.json');result=previous.native.audit_receipt(cell['native'])
            assert cell['native']['initial_inventory']==raw;rewards.append(float(result['reward']))
    panel=a.load(directory/'encoder.json');result=verify_panel(panel,raws)
    endpoints={}
    for mode in MODES:
        expected=[]
        for world in range(9):
            p=panel['records'][world*6+MODES.index(mode)]['probabilities']
            expected.append(math.fsum(p[0][x]*p[1][y]*rewards[world*16+x*4+y] for x in range(4) for y in range(4)))
        endpoints[mode]=dict(world_expected_rewards=expected,balanced_expected_reward=math.fsum(v/9 for v in expected))
    assert endpoints['blind']['balanced_expected_reward']<=1/24+2e-12
    old=a.load(input_root/'audit.json');assert old['accounting_pass'] and old['learning_criterion_pass']
    controls=[dict(policy_seed=c['policy_seed'],graph_balanced=c['final_graph']['balanced_expected_reward'],blind_balanced=c['final_blind']['balanced_expected_reward']) for c in old['comparisons']]
    return dict(result,accounting_pass=True,profile=PROFILE,archive_native_action_cells_rechecked=144,
        untrained_archived_endpoints=endpoints,uniform_baseline=9/384,graph_blind_ceiling=1/24,
        prior_learned_controls=controls,synthetic_topologies_native_validated=False,
        learned_encoder_performance_claim=False,training_admitted=False,scientific_acceptance=False,independent_physics_replay=False)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path);p.add_argument('--input',type=Path,required=True)
    args=p.parse_args();print(json.dumps(audit(args.directory,args.input),indent=2))
