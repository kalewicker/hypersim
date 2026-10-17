#!/usr/bin/env python3
"""Independent offline arithmetic checks; synthetic outcomes are not native evidence."""
import copy
import math
import struct
import audit_ere_categorical_comparison_archive as history

a=history.a; cat=history.s.cat; ROOT=history.ROOT
PROFILE='E2-categorical-feasibility-proposal-v2'


def ordered(xs):
    total=0.
    for x in xs:total+=x
    return total


def dot(xs,ys):return ordered(x*y for x,y in zip(xs,ys))
def norm(xs):return math.sqrt(dot(xs,xs))
def center(xs):
    result=list(xs)
    for start,end in ((0,168),(168,181),(181,183)):
        mean=ordered(xs[start:end])/(end-start)
        for i in range(start,end):result[i]-=mean
    return result


def cone(u,v):
    un,vn=norm(u),norm(v)
    if un<=1e-12 or vn<=1e-12:return None
    uu=[x/un for x in u];vv=[x/vn for x in v];alignment=dot(uu,vv)
    if alignment>=.1:d=uu
    else:
        perpendicular=[x-alignment*y for x,y in zip(uu,vv)];pn=norm(perpendicular)
        if pn<=1e-12:return None
        tangent=math.sqrt(1.-.1*.1)
        d=[tangent*x/pn+.1*y for x,y in zip(perpendicular,vv)]
    return None if dot(u,d)<=0 else d


def fixture(catalogue):
    leaves=cat.oracle_values(catalogue)
    def panel(stage,n,offset,label):
        samples=[]
        for i in range(n):
            selection=copy.deepcopy(leaves[(i*293+offset)%len(leaves)]['selection'])
            capped=selection['parent']>=84 if label=='mixed' else label=='all_cap'
            score=None if capped else (selection['operator']%3)/2
            if label=='equal_valid':score=0.
            samples.append(dict(id=f'synthetic-{stage}-{offset}-{i}',emission_seed=offset+2*i,
                physics_seed=offset+2*i+1,selection=selection,native_score=score,
                reason='resource_cap' if capped else None))
        return dict(stage=stage,expected_draws=n,samples=samples)
    cases=[]
    for ctx,theta in enumerate(history.s.original.p.contexts()):
        for label in ('mixed','all_valid','all_cap'):
            cases.append(dict(id=f'context{ctx}-{label}',theta=theta,
                utility=panel('utility_estimation',64,10000,'mixed'),
                risk=panel('risk_estimation',512,100000,label)))
    return dict(profile=PROFILE,purpose='engineering_synthetic_only',cases=cases)


def check_case(case,got,catalogue):
    assert set(got)==set('profile classification utility_input risk_input_validation risk_validity_values risk_fixed_baselines centered_utility_gradient centered_validity_gradient risk_cap_count direction cone_margin minimum_gradient_norm maximum_radius candidates selected_candidate theta_after hold_reason observed_cap_guard_only unseen_cap_feasibility_certificate validation_used native_steps_run grammar_compilations training_updates training_admitted'.split())
    assert got['profile']==PROFILE and got['classification']=='engineering_arithmetic_only'
    assert got['validation_used'] is False and got['unseen_cap_feasibility_certificate'] is False
    assert got['observed_cap_guard_only'] is True and got['training_admitted'] is False
    assert got['native_steps_run']==got['grammar_compilations']==got['training_updates']==0
    assert case['utility']['stage']=='utility_estimation' and case['risk']['stage']=='risk_estimation'
    samples=case['utility']['samples']+case['risk']['samples']
    assert len({s['id'] for s in samples})==len(samples)
    assert len({s[k] for s in samples for k in ('emission_seed','physics_seed')})==2*len(samples)
    theta=[cat.f32(x) for x in case['theta']]
    for name,panel,alpha in (('utility_input',case['utility'],cat.f32(.05)),('risk_input_validation',case['risk'],0.)):
        cat.check_batch(got[name],catalogue,dict(theta=theta,samples=panel['samples'],expected_draws=panel['expected_draws'],alpha=alpha))
    risk=case['risk'];n=risk['expected_draws'];values=[float(s['native_score'] is not None) for s in risk['samples']]
    scores=[cat.score(catalogue,theta,s['selection']) for s in risk['samples']]
    vg=[0.]*183;baselines=[]
    for i,score in enumerate(scores):
        baseline=1.;baselines.append(baseline)
        for j in range(183):vg[j]+=(values[i]-baseline)*score['gradient'][j]/n
    cat.vec(got['risk_validity_values'],values);cat.vec(got['risk_fixed_baselines'],baselines)
    cat.vec(got['centered_validity_gradient'],center(vg))
    cat.vec(got['centered_utility_gradient'],center(cat.estimate(catalogue,dict(theta=theta,
        samples=case['utility']['samples'],expected_draws=case['utility']['expected_draws'],alpha=cat.f32(.05)))['gradient']))
    # Proposal operations are reconstructed from the saved native f64 gradients,
    # already independently checked above. This avoids treating libm last-bit
    # differences as evidence of different score calculus while keeping each
    # subsequent f32 proposal operation bit-exact (no vector tolerance waiver).
    ug=got['centered_utility_gradient'];vg=got['centered_validity_gradient'];caps=values.count(0.)
    assert got['risk_cap_count']==caps and got['cone_margin']==.1 and got['maximum_radius']==.1
    assert got['minimum_gradient_norm']==1e-12
    reason=None;direction=None
    if caps==0:reason='no_observed_caps'
    elif caps==n or norm(vg)<=1e-12:reason='no_informative_validity_gradient'
    elif norm(ug)<=1e-12:reason='zero_utility_gradient'
    else:
        direction=cone(ug,vg)
        if direction is None:reason='utility_validity_conflict'
    if direction is None:assert got['direction'] is None and got['candidates']==[]
    else:cat.vec(got['direction'],direction);assert len(got['candidates'])==9
    selected=None
    for index,q in enumerate(got['candidates']):
        assert set(q)==set('index requested_radius theta_after actual_norm maximum_coordinate_step utility_linear_gain validity_linear_gain observed_cap_risk_effect risk_weights passes'.split())
        radius=.1/2**index;after=[cat.f32(t+radius*x) for t,x in zip(theta,direction)]
        cat.bits(q['theta_after'],after)
        delta=[x-t for x,t in zip(after,theta)];actual=norm(delta);maximum=max(abs(x) for x in delta)
        utility_gain=dot(ug,delta);validity_gain=dot(vg,delta);weights=[];effect=0.
        for sample,raw in zip(risk['samples'],got['risk_input_validation']['contributions']):
            post=cat.score(catalogue,after,sample['selection'])
            weight=math.exp(post['log_probability']-raw['score']['log_probability']);weights.append(weight)
            if sample['native_score'] is None:effect+=(weight-1.)/n
        for name,value in [('requested_radius',radius),('actual_norm',actual),('maximum_coordinate_step',maximum),
                           ('utility_linear_gain',utility_gain),('validity_linear_gain',validity_gain),('observed_cap_risk_effect',effect)]:cat.close(q[name],value)
        cat.vec(q['risk_weights'],weights);assert q['index']==index
        # Reconstruct exact decision from recorded, independently checked scalar
        # operations; no outcome-dependent adjustment to a zero guard.
        passes=q['actual_norm']>0 and q['actual_norm']<=.100001 and q['maximum_coordinate_step']<=.100001 and q['utility_linear_gain']>0 and q['validity_linear_gain']>=.1*norm(vg)*q['actual_norm']-1e-12 and q['observed_cap_risk_effect']<=0
        assert q['passes'] is passes
        if passes and selected is None:selected=index
    assert got['selected_candidate']==selected
    cat.bits(got['theta_after'],theta if selected is None else got['candidates'][selected]['theta_after'])
    assert got['hold_reason']==(reason or 'no_finite_candidate_passes' if selected is None else None)


def check_result(request,result,catalogue):
    assert set(result)==set('profile classification cases catalogue_sha256 request_sha256 native_steps_run grammar_compilations training_updates training_admitted'.split())
    assert result['profile']==PROFILE and result['classification']=='engineering_synthetic_only'
    assert result['native_steps_run']==result['grammar_compilations']==result['training_updates']==0
    assert result['training_admitted'] is False
    assert len(result['cases'])==len(request['cases'])
    for case,got in zip(request['cases'],result['cases']):
        assert set(got)=={'id','proposal'}
        assert got['id']==case['id'];check_case(case,got['proposal'],catalogue)
    return dict(engineering_arithmetic_pass=True,cases=len(request['cases']),
        native_steps_run=0,grammar_compilations=0,training_updates=0,training_admitted=False,
        unseen_cap_feasibility_certificate=False,validation_used=False,
        f32_proposal_reconstruction='bit-exact from independently checked native f64 gradient receipts')
