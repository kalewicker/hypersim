#!/usr/bin/env python3
"""Independent arithmetic checks; historical accounting, NOT new physics/RNG replay."""
import argparse
import copy
import json
import math
from pathlib import Path
import struct
import analyze_ere_sampler_reward as mapping

d=mapping.d
a=d.a
ROOT=Path(__file__).resolve().parents[1]
PROFILE='E2-direct-categorical-adapter-v1'
INPUT=ROOT/'output/ere_complete_direction_assay_20261001'
INPUT_SEAL=mapping.INPUT_SEAL
SOURCES=['crates/hg-train/src/ere/categorical.rs','crates/hg-train/src/bin/ere-categorical-adapter.rs',
         'docs/ere_categorical_adapter_v1.md','scripts/audit_ere_categorical_adapter.py',
         'scripts/test_audit_ere_categorical_adapter.py','scripts/verify_ere_categorical_adapter.py',
         'scripts/analyze_ere_sampler_reward.py','scripts/test_analyze_ere_sampler_reward.py']

def f32(x):return struct.unpack('<f',struct.pack('<f',x))[0]
def close(x,y,tol=2e-12):
    assert type(x) in (int,float) and math.isfinite(x) and abs(x-y)<=tol,(x,y)
def vec(xs,ys,tol=2e-12):
    assert len(xs)==len(ys)
    for x,y in zip(xs,ys):close(x,y,tol)
def bits(xs,ys):
    assert len(xs)==len(ys) and all(struct.pack('<f',x)==struct.pack('<f',y) for x,y in zip(xs,ys)),'f32 vector bits'
def softmax(theta,support):
    assert support and len(set(support))==len(support)
    maximum=max(theta[i] for i in support)
    weights={i:math.exp(theta[i]-maximum) for i in support};total=math.fsum(weights.values())
    return [weights.get(i,0.)/total for i in range(len(theta))]
def selection(e):
    assert e['role'] in ('Core','Proposal')
    return dict(parent=e['parent'],operator=e['operator'],role=0 if e['role']=='Core' else 1,production_index=e['production_index'])
def key(s):return tuple(s[k] for k in ('parent','operator','role','production_index'))

def score(cat,theta,s):
    theta=[f32(t) for t in theta];assert len(theta)==183 and all(math.isfinite(t) for t in theta)
    p,o,r,k=key(s);parent=cat['parents'][p];group=parent['groups'][o];count=len(group['roles'][r]['productions'])
    assert 0<=k<count
    os=parent['operator_support'];rs=group['role_support']
    pp=softmax(theta[:168],list(range(168)));po=softmax(theta[168:181],os);pr=softmax(theta[181:],rs)
    assert pp[p]>0 and po[o]>0 and pr[r]>0
    gradient=[-v for v in pp+po+pr];gradient[p]+=1;gradient[168+o]+=1;gradient[181+r]+=1
    logp=math.log(pp[p])+math.log(po[o])+math.log(pr[r])-math.log(count)
    return dict(profile=PROFILE,theta=theta,selection=s,operator_support=os,role_support=rs,production_count=count,
        parent_probabilities=pp,operator_probabilities=po,role_probabilities=pr,log_probability=logp,probability=math.exp(logp),gradient=gradient)

def check_score(got,cat,theta,s):
    expected=score(cat,theta,s);assert set(got)==set(expected)
    assert type(got['production_count']) is int and all(type(x) is int for name in ('operator_support','role_support') for x in got[name])
    assert set(got['selection'])==set(s) and all(type(x) is int for x in got['selection'].values())
    for name in ('profile','selection','operator_support','role_support','production_count'):assert got[name]==expected[name],name
    bits(got['theta'],expected['theta'])
    for name in ('parent_probabilities','operator_probabilities','role_probabilities','gradient'):vec(got[name],expected[name])
    for name in ('log_probability','probability'):close(got[name],expected[name])
    for start,end in ((0,168),(168,181),(181,183)):close(math.fsum(got['gradient'][start:end]),0.,1e-12)
    return expected

def utility(sample):
    score,reason=sample['native_score'],sample['reason']
    if score is None:
        assert reason=='resource_cap';return 0.
    assert reason is None and type(score) in (int,float) and math.isfinite(score) and 0<=score<=1
    return 1.+score

def estimate(cat,batch):
    t=[f32(x) for x in batch['theta']];samples=batch['samples'];n=batch['expected_draws'];assert n>=2 and len(samples)==n
    assert len({s['id'] for s in samples})==len({s['emission_seed'] for s in samples})==n
    u=[utility(s) for s in samples];g=[0.]*183;contributions=[]
    for i,s in enumerate(samples):
        b=math.fsum(x for j,x in enumerate(u) if j!=i)/(n-1);advantage=u[i]-b;r=score(cat,t,s['selection'])
        for j in range(183):g[j]+=advantage*r['gradient'][j]/n
        contributions.append(dict(sample=s,score=r,utility=u[i],baseline=b,advantage=advantage))
    alpha=f32(batch['alpha']);assert math.isfinite(alpha) and alpha>=0
    return dict(profile=PROFILE,objective='mean-constraint-utility-v1',baseline='leave-one-out-emission-mean-v1',
        theta_before=t,alpha=alpha,expected_draws=n,utility_mean=math.fsum(u)/n,contributions=contributions,gradient=g,
        theta_after=[f32(x+alpha*y) for x,y in zip(t,g)])

def check_batch(got,cat,batch):
    e=estimate(cat,batch);assert set(got)==set(e)
    for name in ('profile','objective','baseline','expected_draws'):assert got[name]==e[name]
    bits(got['theta_before'],e['theta_before']);bits(got['theta_after'],e['theta_after']);bits([got['alpha']],[e['alpha']])
    close(got['utility_mean'],e['utility_mean']);vec(got['gradient'],e['gradient'])
    assert len(got['contributions'])==len(e['contributions'])
    for g,x in zip(got['contributions'],e['contributions']):
        assert set(g)==set(x) and g['sample']==x['sample'];check_score(g['score'],cat,batch['theta'],x['sample']['selection'])
        assert type(g['sample']['emission_seed']) is int and type(g['sample']['physics_seed']) is int
        utility(g['sample'])
        for name in ('utility','baseline','advantage'):close(g[name],x[name])

def oracle_values(cat):
    leaves,_,_=mapping.catalogue_model(cat)
    return [dict(selection=dict(zip(('parent','operator','role','production_index'),k)),active=float(v['active']),eligible=float(v['eligible'])) for k,v in sorted(leaves.items())]

def oracle_expectation(cat,theta,values,feature):
    # Group feature averages and direct finite sum; no Rust score-gradient formula.
    grouped={}
    for v in values:
        k=key(v['selection'])[:3];grouped.setdefault(k,[]).append(v[feature])
    pp=softmax(theta[:168],list(range(168)));terms=[]
    for p,parent in enumerate(cat['parents']):
        po=softmax(theta[168:181],parent['operator_support'])
        for o in parent['operator_support']:
            pr=softmax(theta[181:],parent['groups'][o]['role_support'])
            for r in parent['groups'][o]['role_support']:
                xs=grouped[(p,o,r)];terms.append(pp[p]*po[o]*pr[r]*math.fsum(xs)/len(xs))
    return math.fsum(terms)

def source_ids():
    return [f'validate-r{r}-c{ctx}-d{draw}-base-p0' for r in range(8) for ctx in range(2) for draw in range(16)]+[
        'estimate-r0-c0-e0-p4-s1-b2','estimate-r2-c1-e1-p3-s-1-b0','estimate-r3-c0-e0-p1-s-1-b2',
        'estimate-r3-c1-e1-p4-s1-b2','estimate-r6-c0-e0-p6-s1-b1']

def request_from_receipts(cat,rows):
    byid={r['schedule']['id']:r for r in rows};assert set(byid)==set(source_ids())
    batches=[];contexts=d.contexts()
    for r in range(8):
        for ctx in range(2):
            samples=[]
            for draw in range(16):
                row=byid[f'validate-r{r}-c{ctx}-d{draw}-base-p0'];bits(row['parameters'],contexts[ctx]);q=row['schedule'];v=row['evaluation']
                samples.append(dict(id=q['id'],emission_seed=q['emission_seed'],physics_seed=q['physics_seed'],selection=selection(row['emission']),native_score=v['score'],reason=v['reason']))
            batches.append(dict(id=f'historical-r{r}-c{ctx}',theta=contexts[ctx],alpha=f32(.05),expected_draws=16,samples=samples))
    first=[v['selection'] for v in oracle_values(cat) if v['selection']['parent'] in (0,1)]
    chosen=[next(s for s in first if s['parent']==p) for p in (0,1)]
    for name,outcomes in [('all-cap',[(None,'resource_cap')]*2),('equal-valid',[(0.,None)]*2),
                           ('mixed-cap',[(None,'resource_cap'),(1.,None)]),('ordered-valid',[(0.,None),(1.,None)])]:
        samples=[dict(id=f'synthetic-{name}-{i}',emission_seed=i,physics_seed=0,selection=chosen[i],native_score=v,reason=reason) for i,(v,reason) in enumerate(outcomes)]
        batches.append(dict(id='synthetic-'+name,theta=contexts[0],alpha=f32(.05),expected_draws=2,samples=samples))
    caps=[]
    for id in source_ids()[256:]:
        row=byid[id];v=row['evaluation'];assert row['status']=='Null' and v['score'] is None and v['reason']=='resource_cap'
        caps.append(dict(id=id,theta=row['parameters'],emission=row['emission'],native_score=None,reason='resource_cap'))
    return dict(profile=PROFILE,purpose='engineering_receipt_reconstruction',catalogue_sha256=a.CATALOGUE,
        batches=batches,score_cases=caps,oracle_contexts=contexts,oracle_leaves=oracle_values(cat))

def check_result(request,result,cat,finite_differences=False):
    assert set(result)=={'profile','classification','native_steps_run','grammar_compilations','training_updates','training_admitted','catalogue_sha256','request_sha256','batches','score_cases','oracles'}
    assert result['profile']==PROFILE and result['classification']=='engineering_arithmetic_only'
    assert all(type(result[k]) is int and result[k]==0 for k in ('native_steps_run','grammar_compilations','training_updates'))
    assert result['training_admitted'] is False and result['catalogue_sha256']==a.CATALOGUE
    assert len(result['batches'])==len(request['batches'])
    for got,batch in zip(result['batches'],request['batches']):
        assert set(got)=={'id','estimate'} and got['id']==batch['id'];check_batch(got['estimate'],cat,batch)
    assert len(result['score_cases'])==len(request['score_cases'])
    for got,case in zip(result['score_cases'],request['score_cases']):
        assert set(got)=={'id','score','native_score','reason','utility'} and got['id']==case['id']
        assert got['native_score']==case['native_score'] and got['reason']==case['reason'];close(got['utility'],utility(case))
        check_score(got['score'],cat,case['theta'],selection(case['emission']))
    assert len(result['oracles'])==len(request['oracle_contexts'])
    maximum=0.;checked=0
    for got,t in zip(result['oracles'],request['oracle_contexts']):
        assert set(got)=={'theta','active','eligible'};bits(got['theta'],t)
        for feature in ('active','eligible'):
            v=got[feature];assert set(v)=={'mass','value','gradient'};close(v['mass'],1.,1e-12)
            close(v['value'],oracle_expectation(cat,t,request['oracle_leaves'],feature))
            assert len(v['gradient'])==183 and all(math.isfinite(x) for x in v['gradient'])
            for start,end in ((0,168),(168,181),(181,183)):close(math.fsum(v['gradient'][start:end]),0.,1e-12)
            # Independently enumerate E[feature*score], then finite-difference the
            # direct expectation: support values and derivative paths are distinct.
            expected=[0.]*183
            for leaf in request['oracle_leaves']:
                s=score(cat,t,leaf['selection']);w=s['probability']*leaf[feature]
                for j,g in enumerate(s['gradient']):expected[j]+=w*g
            vec(v['gradient'],expected)
            if finite_differences:
                for j in range(183):
                    plus=t[:];minus=t[:];plus[j]+=1e-4;minus[j]-=1e-4
                    fd=(oracle_expectation(cat,plus,request['oracle_leaves'],feature)-oracle_expectation(cat,minus,request['oracle_leaves'],feature))/(2e-4)
                    error=abs(fd-v['gradient'][j]);assert error<2e-9,(feature,j,error);maximum=max(maximum,error);checked+=1
    return dict(finite_difference_coordinates=checked,maximum_finite_difference_error=maximum)

def tamper_checks(request,result,cat):
    mutations=[]
    def add(name,change):mutations.append((name,change))
    add('log_probability',lambda x:x['batches'][0]['estimate']['contributions'][0]['score'].__setitem__('log_probability',0.))
    add('probability',lambda x:x['batches'][0]['estimate']['contributions'][0]['score'].__setitem__('probability',1.))
    add('gradient',lambda x:x['batches'][0]['estimate']['gradient'].__setitem__(0,1.))
    add('score_gradient',lambda x:x['batches'][0]['estimate']['contributions'][0]['score']['gradient'].__setitem__(181,1.))
    add('mask',lambda x:x['batches'][0]['estimate']['contributions'][0]['score'].__setitem__('operator_support',[]))
    add('production_count',lambda x:x['batches'][0]['estimate']['contributions'][0]['score'].__setitem__('production_count',999))
    add('f32_update',lambda x:x['batches'][0]['estimate']['theta_after'].__setitem__(0,.3))
    add('baseline',lambda x:x['batches'][0]['estimate']['contributions'][0].__setitem__('baseline',3.))
    add('advantage',lambda x:x['batches'][0]['estimate']['contributions'][0].__setitem__('advantage',3.))
    add('cap_imputation',lambda x:x['score_cases'][0].__setitem__('native_score',0.))
    add('cap_drop',lambda x:x['score_cases'].pop())
    add('denominator',lambda x:x['batches'][0]['estimate'].__setitem__('expected_draws',15))
    add('missing_sample',lambda x:x['batches'][0]['estimate']['contributions'].pop())
    add('seed',lambda x:x['batches'][0]['estimate']['contributions'][0]['sample'].__setitem__('emission_seed',0))
    add('reason',lambda x:x['score_cases'][0].__setitem__('reason','crash'))
    add('oracle',lambda x:x['oracles'][0]['eligible'].__setitem__('value',.9))
    add('native_work_claim',lambda x:x.__setitem__('native_steps_run',1))
    add('admission_claim',lambda x:x.__setitem__('training_admitted',True))
    rejected=[]
    for name,change in mutations:
        bad=copy.deepcopy(result);change(bad)
        try:check_result(request,bad,cat)
        except (AssertionError,KeyError,ValueError,IndexError):rejected.append(name)
        else:raise AssertionError('tamper accepted: '+name)
    return rejected

def audit(out,sealed=True,tamper=False):
    out=Path(out)
    if sealed:a.verify(out)
    m=a.load(out/'manifest.json');assert m['profile']==PROFILE and m['status']=='complete' and m['classification']=='engineering_arithmetic_only'
    assert m['frozen_before_adapter_invocation'] is True
    assert all(type(m[k]) is int and m[k]==0 for k in ('native_steps_run','grammar_compilations','training_updates')) and m['training_admitted'] is False
    assert m['input_archive']==str(INPUT) and m['input_seal']==INPUT_SEAL
    a.verify(INPUT,expected=INPUT_SEAL,count=7686)
    snap=a.load(INPUT/'snapshot.json');assert set(m['sources'])==set(snap['sources'])|set(SOURCES)
    for name,h in m['sources'].items():assert a.safe(name) and a.digest(out/'source_snapshots'/name)==h
    # Executed Python arithmetic/accounting imports must be the frozen versions.
    for name in set(mapping.CHECKERS)|set(SOURCES[3:]):assert a.digest(ROOT/name)==m['sources'][name],name
    assert a.digest(out/'executed_binary')==m['binary_sha256']
    for name,h in m['artifacts'].items():assert a.digest(out/name)==h
    assert {'catalogue.json','request.json','result.json'}<=set(m['artifacts'])<= {'catalogue.json','request.json','result.json','audit.json'}
    cat=a.load(out/'catalogue.json');assert a.digest(out/'catalogue.json')==a.CATALOGUE
    ids=source_ids();assert {p.stem for p in (out/'source_receipts').iterdir()}==set(ids)
    parent=Path(a.load(INPUT/'parent_receipt.json')['parent_archive']);a.verify(parent,expected=d.c.PARENT,count=1690)
    runtime=Path(a.load(parent/'manifest.json')['receipt_paths']['runtime']);a.verify(runtime,expected=a.SPECS['runtime'][1],count=a.SPECS['runtime'][2])
    native,h=d.c.helpers(parent);schedule={q['id']:q for q in d.schedule(d.design())};rows=[]
    for id in ids:
        p=out/'source_receipts'/(id+'.json');assert a.digest(p)==a.digest(INPUT/'cells'/(id+'.json'))
        row=a.load(p);d.b.cell(row,schedule[id],cat,native,h);rows.append(row)
    request=a.load(out/'request.json');assert request==request_from_receipts(cat,rows)
    result=a.load(out/'result.json');assert result['request_sha256']==a.digest(out/'request.json')
    math_checks=check_result(request,result,cat,finite_differences=True)
    rejected=tamper_checks(request,result,cat) if tamper else []
    return dict(audit_pass=True,classification='engineering_arithmetic_only',native_steps_run=0,grammar_compilations=0,training_updates=0,
        training_admitted=False,native_receipts_rechecked=261,historical_batches=16,synthetic_batches=4,oracle_leaves=18956,
        tamper_rejections=len(rejected),tamper_controls=rejected,physics_replayed=False,rng_replayed=False,**math_checks)

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('output',type=Path);parser.add_argument('--tamper-tests',action='store_true');args=parser.parse_args()
    print(json.dumps(audit(args.output,tamper=args.tamper_tests),indent=2))
