#!/usr/bin/env python3
"""Fixed-parameter direction accounting; not independent physics/RNG replay."""
import argparse
import copy
import json
import math
import struct
from pathlib import Path
import tempfile
import audit_ere_complete_minibatch as b

a=b.a;c=b.c;f=b.f
PROFILE='E2-complete-direction-reliability-v1'
LATEST='49a2bca085a7f7619c2bde3b5a25992a8fc16d8418d97165a2db3fb9db698806'
EXTRA={'configs/ere_complete_direction_v1.json','docs/ere_complete_direction_v1.md',
       'scripts/audit_ere_complete_direction.py','scripts/test_audit_ere_complete_direction.py'}
ARMS=['base','forward_a','reverse_a','forward_b','reverse_b']

def design(development=False):
    seed=0xE658D00001 if development else 0xE658000001
    blocks,k,draws=(1,2,2) if development else (8,4,16)
    return dict(profile=PROFILE,mode='development' if development else 'scientific',master_seed=seed,blocks=blocks,contexts=2,
        batch_size=k,validation_draws=draws,validation_physics=4,candidates=16,parameters=183,sigma=1.,alpha=0.05000000074505806,
        horizon=100,work=50000,results=50000,wall_seconds=900 if development else 3600,cell_wall_seconds=30,
        output_cap_bytes=2*1024**3,maximum_native_steps=blocks*2*(32*k+20*draws)*100,objective='mean-constraint-utility-v1',
        statistical_unit='paired_estimator_block',zero_direction_alignment=0,parameter_panel='zero-and-alternating-quarter-v1',
        validation_arms=ARMS,optimizer_generations=0,precheck_evaluations=0,full_training_authorized=False)

def schedule(d):
    seed=d['master_seed'];k=d['batch_size'];q=[]
    for r in range(d['blocks']):
        for ctx in range(2):
            for e in range(2):
                for p in range(8):
                    for sign in (1,-1):
                        for draw in range(k):
                            q.append(dict(id=f'estimate-r{r}-c{ctx}-e{e}-p{p}-s{sign}-b{draw}',stage='estimation',block=r,
                                context=ctx,estimate=e,pair=p,sign=sign,batch_draw=draw,
                                emission_seed=seed+10000+1000*r+100*ctx+40*e+k*p+draw,
                                physics_seed=seed+100000+100*r+10*ctx+4*e+draw%4,horizon=100))
    for r in range(d['blocks']):
        for ctx in range(2):
            for draw in range(d['validation_draws']):
                for arm,name in enumerate(ARMS):
                    for physics in range(4):
                        q.append(dict(id=f'validate-r{r}-c{ctx}-d{draw}-{name}-p{physics}',stage='validation',block=r,
                            context=ctx,draw=draw,arm=arm,physics_index=physics,emission_seed=seed+20000+1000*r+100*ctx+draw,
                            physics_seed=seed+200000+100*r+10*ctx+physics,horizon=100))
    return q

def contexts():return [[0.]*183,[.25 if i%2==0 else -.25 for i in range(183)]]

def alignment(theta,first,second):
    assert len(theta)==len(first)==len(second)==183
    f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
    x=[f32(u)-f32(t) for u,t in zip(first,theta)];y=[f32(u)-f32(t) for u,t in zip(second,theta)]
    nx=math.sqrt(sum(v*v for v in x));ny=math.sqrt(sum(v*v for v in y));degenerate=nx==0 or ny==0
    return dict(score=0. if degenerate else max(-1.,min(1.,sum(a*b for a,b in zip(x,y))/(nx*ny))),
        degenerate=degenerate,norm_a=nx,norm_b=ny)

def arm_parameters(theta,first,second,h):
    return [theta,first,[h.f32(h.f32(2*h.f32(t))-h.f32(u)) for t,u in zip(theta,first)],second,[h.f32(h.f32(2*h.f32(t))-h.f32(u)) for t,u in zip(theta,second)]]

def shared_basis(first,second):
    assert first['block']==second['block'] and first['context']==second['context']
    assert first['estimate']==0 and second['estimate']==1 and first['theta_before']==second['theta_before'] and first['eps']==second['eps']

def statistics(d,align,utilities,natives,counts,complete):
    blocks=d['blocks'];assert len(align)==len(utilities)==len(natives)==len(counts)==blocks
    assert all(len(v)==2 for data in (align,utilities,natives,counts) for v in data)
    assert all(len(v)==5 for data in (utilities,natives,counts) for r in data for v in r)
    known=all(x is not None for r in align for x in r) and all(x is not None for r in utilities for ctx in r for x in ctx)
    ba=[sum(x['score'] for x in r)/2 if all(x is not None for x in r) else None for r in align]
    effects=[[(u[1]+u[3])/2-u[0] if all(u[i] is not None for i in (0,1,3)) else None for u in r] for r in utilities]
    be=[sum(r)/2 if all(x is not None for x in r) else None for r in effects]
    def stats(v):
        xs=[x for x in v if x is not None];wins=sum(x>0 for x in xs);losses=sum(x<0 for x in xs)
        return dict(mean=sum(xs)/blocks if known else None,sign_p=b.sign_p(wins,losses) if known else None,
            wins=wins,losses=losses,ties=len(xs)-wins-losses,complete_blocks=len(xs))
    rel=stats(ba);use=stats(be);ce=[sum(r[ctx] for r in effects)/blocks if known else None for ctx in range(2)]
    valid=[sum(ctx[arm] for r in counts for ctx in r) for arm in range(5)]
    native_known=all(x is not None for r in natives for ctx in r for x in ctx)
    def means(v,ok):return [sum(sum(ctx[arm] for ctx in r) for r in v)/(2*blocks) if ok else None for arm in range(5)]
    um=means(utilities,known)
    support=d['mode']=='scientific' and complete and known and rel['mean']>0 and rel['sign_p']<.05 and use['mean']>0 and use['sign_p']<.05 and all(x>0 for x in ce) and valid[1]+valid[3]>=2*valid[0]
    return dict(profile=PROFILE,mode=d['mode'],measurement_complete=complete,statistical_unit='paired_estimator_block',fixed_denominator=blocks,
        all_outcomes_known=known,context_alignment=align,block_alignment=ba,reliability=rel,context_utility_effects=effects,
        context_mean_effects=ce,block_utility_effects=be,usefulness=use,utility_arm_means=um,native_arm_means=means(natives,native_known),
        all_validation_native_valid=native_known,valid_counts=valid,feasibility_denominator_per_arm=blocks*2*d['validation_draws']*4,
        forward_minus_reverse_descriptive=(um[1]+um[3]-um[2]-um[4])/2 if known else None,support_direction=support,independent_physics_replay=False)

def check_statistics(got,expected):
    f.check_statistics(got,expected)
    for key in ('reliability','usefulness'):
        p=expected[key]['sign_p']
        if p is not None:assert math.isclose(got[key]['sign_p'],p,rel_tol=1e-12,abs_tol=1e-300)

def manifest(m):
    d=design(m['design']['mode']=='development')
    assert m['profile']==PROFILE and m['design']==d and all(type(m['design'][k]) is type(v) for k,v in d.items())
    assert m['scheduled_cells']==d['maximum_native_steps']//100 and m['optimizer_generations']==0 and m['full_training_authorized'] is False
    assert 0<=m['advancing_steps']<=m['attempted_steps']<=d['maximum_native_steps'] and 0<=m['recorded_cells']<=m['scheduled_cells']
    assert 0<=m['direction_estimates']<=m['direction_starts']<=d['blocks']*4 and m['inflight_native_work_unknown'] is False
    return d

def estimate(rec,qs,theta,rows,native,h,k):
    assert len(qs)==16*k and rec['block']==qs[0]['block'] and rec['context']==qs[0]['context'] and rec['estimate']==qs[0]['estimate']
    assert rec['theta_before']==theta and len(rec['eps'])==8 and rec['evaluation_ids']==[q['id'] for q in qs]
    assert type(rec['update_applied']) is bool
    full=all(rows[q['id']]['status'] in ('Complete','Null') and rows[q['id']].get('constraint_utility') is not None for q in qs)
    scores=[]
    for p,noise in enumerate(rec['eps']):
        native.vector(noise)
        for sign in (1,-1):
            candidate=[h.f32(t+h.f32(sign*x)) for t,x in zip(theta,noise)];selected=qs[(2*p+(sign==-1))*k:(2*p+(sign==-1)+1)*k];v=[]
            assert all(q['pair']==p and q['sign']==sign and q['batch_draw']==draw for draw,q in enumerate(selected))
            for q in selected:
                row=rows[q['id']]
                if row['status']!='NotRun':native.vec_close(row['parameters'],candidate)
                if row['status'] in ('Complete','Null'):v.append(row['evaluation']['score'])
            scores.append(v)
    if not rec['update_applied']:
        assert rec['candidate_means'] is rec['optimizer_pair_scores'] is rec['theta_after'] is None
        return None
    assert full,'incomplete estimate update';means,fits=b.batch_fits(scores,k,h);f.check_statistics(rec,dict(candidate_means=means))
    assert rec['optimizer_pair_scores']==[fits[i:i+2] for i in range(0,16,2)]
    native.vec_close(rec['theta_after'],native.update(theta,rec['eps'],fits,8,h))
    return rec['theta_after']

def direction(rec,r,ctx,theta,first,second,h):
    assert rec['block']==r and rec['context']==ctx and rec['selection']=='both_fixed_estimates_no_validation_selection'
    f.check_statistics(rec,dict(alignment=alignment(theta,first,second)))
    # Reversal must be the exact saved-vector f32 construction, not another tuned step.
    expected=arm_parameters(theta,first,second,h)
    assert len(rec['parameters'])==5
    for got,params in zip(rec['parameters'],expected):
        assert [h.f32(x) for x in got]==[h.f32(x) for x in params]
    return rec['alignment'],rec['parameters']

def audit(out):
    out=Path(out);a.verify(out);m=a.load(out/'manifest.json');d=manifest(m);plan=schedule(d)
    b.exact_schedule(a.load(out/'schedule.json'),plan);assert a.digest(out/'schedule.json')==m['schedule_sha256']
    assert {p.name for p in (out/'cells').iterdir()}=={q['id']+'.json' for q in plan}
    if 'snapshot_sha256' not in m:
        assert m['status']=='blocked_before_evaluation' and m['recorded_cells']==m['attempted_steps']==m['advancing_steps']==m['direction_starts']==m['direction_estimates']==m['grammar_compilations']==0 and m['training_admitted'] is False
        assert not list((out/'estimates').iterdir()) and not list((out/'directions').iterdir())
        assert all(a.load(out/'cells'/(q['id']+'.json'))==dict(schedule=q,status='NotRun',reason='not_started',evaluation=None) for q in plan)
        s=a.load(out/'summary.json');assert not s['measurement_complete'] and not s['support_direction'] and s['fixed_denominator']==d['blocks'] and s['usefulness']['mean'] is None
        return dict(audit_pass=True,classification='zero_work_refusal',scientific_campaign_complete=False)
    assert a.digest(out/'snapshot.json')==m['snapshot_sha256'];snap=a.load(out/'snapshot.json');assert snap['frozen_before_measurement'] is True
    proof=a.load(out/'parent_receipt.json');parent=Path(proof['parent_archive']);a.verify(parent,expected=c.PARENT,count=1690)
    assert proof['parent_seal']==c.PARENT and proof['sealed_files_verified']==1690 and proof['native_sources_verified']==230 and proof['champion_sha256']==c.CHAMPION
    assert proof['prior_improvement_supported'] is False and proof['prior_sign_p']==.109375
    latest=a.load(out/'latest_receipt.json');assert latest['root_seal']==LATEST and latest['sealed_files_verified']==10152 and latest['scope']=='diagnostic lineage, not improvement'
    a.verify(Path(latest['archive']),expected=LATEST,count=10152);la=a.load(Path(latest['archive'])/'audit.json')
    assert la['audit_pass'] and la['scientific_campaign_complete'] and la['support_batching'] is False
    pins={n:v for n,v in a.load(parent/'manifest.json')['snapshot']['sources'].items() if a.pinned(n)};assert len(pins)==230
    train={str(p.relative_to(out/'source_snapshots')) for p in (out/'source_snapshots/crates/hg-train').rglob('*') if p.is_file() and p.suffix in ('.rs','.toml')}
    assert set(snap['sources'])==set(pins)|train|c.EXTRA|f.EXTRA|b.EXTRA|EXTRA and a.files(out/'source_snapshots')==set(snap['sources'])
    for n,sha in snap['sources'].items():
        assert a.safe(n) and a.digest(out/'source_snapshots'/n)==sha
        if n in pins:assert sha==pins[n]
    for n in ('scripts/audit_ere_complete_admission.py','scripts/audit_ere_complete_scaling.py'):assert snap['sources'][n]==a.load(parent/'manifest.json')['snapshot']['sources'][n]
    assert a.digest(out/'executed_binary')==snap['binary_sha256']
    assert a.load(out/'source_snapshots/configs/ere_complete_direction_v1.json')==dict(scientific=design(),development=design(True))
    if not (out/'catalogue.json').exists():
        assert m['status']=='blocked_before_evaluation' and m['recorded_cells']==m['attempted_steps']==m['direction_starts']==0
        assert all(a.load(out/'cells'/(q['id']+'.json'))['status']=='NotRun' for q in plan)
        assert not list((out/'estimates').iterdir()) and not list((out/'directions').iterdir())
        if (out/'parent_audit.json').exists():
            prior=a.load(out/'parent_audit.json');assert prior['audit_pass'] and prior['training_complete'] and prior['precheck_pass']
        if (out/'contexts.json').exists():assert a.load(out/'contexts.json')==contexts()
        s=a.load(out/'summary.json');assert not s['support_direction'] and not s['measurement_complete'] and s['usefulness']['mean'] is None
        return dict(audit_pass=True,classification='incomplete_setup',scientific_campaign_complete=False)
    prior=a.load(out/'parent_audit.json');assert prior['audit_pass'] and prior['training_complete'] and prior['precheck_pass']
    theta=contexts();assert a.load(out/'contexts.json')==theta
    assert a.digest(out/'catalogue.json')==a.CATALOGUE and m['grammar_compilations']==1;catalogue=a.load(out/'catalogue.json');native,h=c.helpers(parent)
    attempts=advances=recorded=unknown=valid=null=0;peaks=[0]*4;notrun=False
    def checked(q):
        nonlocal attempts,advances,recorded,unknown,valid,null,notrun
        row=a.load(out/'cells'/(q['id']+'.json'));n,adv,score=b.cell(row,q,catalogue,native,h);attempts+=n;advances+=adv
        if row['status']=='NotRun':notrun=True
        else:
            assert not notrun,'fixed sequence gap';recorded+=1;unknown+=row['status']=='Incomplete';valid+=row['status']=='Complete';null+=row['status']=='Null'
            if row['evaluation'] is not None:
                assert 0<=row['elapsed_seconds']<d['cell_wall_seconds']
                for s in row['evaluation']['steps']:
                    for i,v in enumerate((s['usage']['work'],s['usage']['results'],s['nodes'],s['edges'])):peaks[i]=max(peaks[i],v)
        return row,score
    offset=starts=updates=estimation_null=0;efiles=set();dfiles=set();directions={};align=[[None]*2 for _ in range(d['blocks'])];estimate_prefix_closed=False
    for r in range(d['blocks']):
        for ctx in range(2):
            forwards=[];noises=[]
            for e in range(2):
                qs=plan[offset:offset+16*d['batch_size']];offset+=16*d['batch_size'];rows={}
                for q in qs:
                    row,_=checked(q);rows[q['id']]=row;estimation_null+=row['status']=='Null'
                name=f'r{r}-c{ctx}-e{e}.json';path=out/'estimates'/name
                if path.exists():
                    assert not estimate_prefix_closed,'estimate starts after an unfinished/missing estimate'
                    rec=a.load(path);starts+=1;efiles.add(name);forward=estimate(rec,qs,theta[ctx],rows,native,h,d['batch_size'])
                    noises.append(rec['eps']);updates+=forward is not None;forwards.append(forward)
                    if forward is None:estimate_prefix_closed=True
                    if e==1:
                        assert len(noises)==2 and forwards[0] is not None,'shared perturbation basis'
                        shared_basis(a.load(out/'estimates'/f'r{r}-c{ctx}-e0.json'),rec)
                else:
                    assert all(row['status']=='NotRun' for row in rows.values());forwards.append(None);estimate_prefix_closed=True
            name=f'r{r}-c{ctx}.json';path=out/'directions'/name
            if path.exists():
                assert all(x is not None for x in forwards);dfiles.add(name);rec=a.load(path)
                align[r][ctx],directions[(r,ctx)]=direction(rec,r,ctx,theta[ctx],*forwards,h)
    assert {p.name for p in (out/'estimates').iterdir()}==efiles and {p.name for p in (out/'directions').iterdir()}==dfiles
    assert (starts,updates)==(m['direction_starts'],m['direction_estimates'])
    us=[[[None]*5 for _ in range(2)] for _ in range(d['blocks'])];ns=copy.deepcopy(us);counts=[[[0]*5 for _ in range(2)] for _ in range(d['blocks'])]
    for r in range(d['blocks']):
        for ctx in range(2):
            utilities=[[] for _ in range(5)];scores=[[] for _ in range(5)];context_full=True
            for draw in range(d['validation_draws']):
                for arm in range(5):
                    emission=None
                    for _ in range(4):
                        q=plan[offset];offset+=1;row,score=checked(q);scores[arm].append(score);u=row.get('constraint_utility');utilities[arm].append(u);counts[r][ctx][arm]+=score is not None
                        context_full &= row['status'] in ('Complete','Null') and u is not None
                        if row['status']!='NotRun':
                            assert len(directions)==d['blocks']*2 and updates==d['blocks']*4,'validation before all fixed directions'
                            assert row['parameters']==directions[(r,ctx)][arm]
                            if emission is None:emission=row['emission']
                            else:assert row['emission']==emission
            if context_full:
                for arm in range(5):
                    us[r][ctx][arm]=sum(utilities[arm])/(d['validation_draws']*4)
                    ns[r][ctx][arm]=sum(scores[arm])/(d['validation_draws']*4) if all(x is not None for x in scores[arm]) else None
    assert offset==len(plan) and (recorded,attempts,advances,unknown)==(m['recorded_cells'],m['attempted_steps'],m['advancing_steps'],m['unknown_partial_work_cells'])
    complete=m['status']=='complete'
    if complete:
        assert recorded==len(plan) and unknown==0 and updates==d['blocks']*4 and m['live_sources_rechecked'] and m['training_admitted']
        assert 0<=m['grammar_compile_seconds']<=m['elapsed_seconds']<d['wall_seconds']
    else:assert m['status'] in ('incomplete','blocked_before_evaluation') and isinstance(m['reason'],str)
    summary=a.load(out/'summary.json');expected=statistics(d,align,us,ns,counts,complete);check_statistics(summary,expected)
    assert summary['attempted_steps']==attempts and summary['advancing_steps']==advances
    assert sum((out/n).stat().st_size for n in a.files(out))<d['output_cap_bytes']
    return dict(audit_pass=True,classification='bounded_development' if d['mode']=='development' else 'fixed_parameter_direction_assay',
        campaign_complete=complete,scientific_campaign_complete=complete and d['mode']=='scientific',support_direction=summary['support_direction'],
        recorded_cells=recorded,valid_evaluations=valid,null_evaluations=null,estimation_null_evaluations=estimation_null,
        attempted_steps=attempts,advancing_steps=advances,direction_estimates=updates,optimizer_generations=0,frozen_sources=len(snap['sources']),
        peak_work=peaks[0],peak_results=peaks[1],peak_nodes=peaks[2],peak_edges=peaks[3],statistical_unit='paired_estimator_block',
        fixed_denominator=d['blocks'],reliability=summary['reliability'],usefulness=summary['usefulness'],independent_physics_replay=False)

def tamper_tests(out):
    out=Path(out);m=a.load(out/'manifest.json');d=manifest(m);plan=schedule(d);parent=Path(a.load(out/'parent_receipt.json')['parent_archive']);native,h=c.helpers(parent);catalogue=a.load(out/'catalogue.json');n=0
    def rejects(fn):
        nonlocal n
        try:fn()
        except (AssertionError,KeyError,ValueError,IndexError):n+=1;return
        raise AssertionError('tamper accepted')
    for key,value in [('blocks',16),('horizon',10000),('parameter_panel','best-champion'),('master_seed',d['master_seed']+1),('statistical_unit','rollout'),('alpha',.05)]:
        bad=copy.deepcopy(m);bad['design'][key]=value;rejects(lambda:manifest(bad))
    for key,value in [('direction_starts',0),('optimizer_generations',1),('full_training_authorized',True)]:
        bad=copy.deepcopy(m);bad[key]=value;rejects(lambda:manifest(bad))
    q=plan[0];row=a.load(out/'cells'/(q['id']+'.json'))
    for case in ('seed','score','credit','cap','allowance','membership','event','utility','batch_draw_type'):
        bad=copy.deepcopy(row)
        if case=='seed':bad['schedule']['emission_seed']+=1
        elif case=='score':bad['evaluation']['score']=.1234567
        elif case=='credit':bad['evaluation']['proposal_fires']+=1
        elif case=='cap':bad['evaluation']['steps'][0]['cap']=True
        elif case=='allowance':bad['evaluation']['steps'][0]['usage']['rules'][0]['work_allowance']+=1
        elif case=='membership':bad['emission']['signature']='bad'
        elif case=='event':bad['evaluation']['steps'][0]['rule_ids']=[999999]
        elif case=='utility':bad['constraint_utility']+=1
        else:bad['schedule']['batch_draw']=False
        rejects(lambda:b.cell(bad,q,catalogue,native,h))
    qs=plan[:16*d['batch_size']];rows={q['id']:a.load(out/'cells'/(q['id']+'.json')) for q in qs};rec=a.load(out/'estimates/r0-c0-e0.json')
    for case in ('candidate','mean','fit','update','ids','partial','adaptive_context','update_type'):
        bad=copy.deepcopy(rec);rs=copy.deepcopy(rows)
        if case=='candidate':rs[qs[1]['id']]['parameters'][0]+=.1
        elif case=='mean':bad['candidate_means'][0]+=.1
        elif case=='fit':bad['optimizer_pair_scores'][0][0]+=1
        elif case=='update':bad['theta_after'][0]+=.1
        elif case=='ids':bad['evaluation_ids'].reverse()
        elif case=='partial':rs[qs[1]['id']]['status']='NotRun'
        elif case=='adaptive_context':bad['theta_before'][0]=.1
        else:bad['update_applied']=1
        rejects(lambda:estimate(bad,qs,contexts()[0],rs,native,h,d['batch_size']))
    rec=a.load(out/'directions/r0-c0.json');first=a.load(out/'estimates/r0-c0-e0.json')['theta_after'];second=a.load(out/'estimates/r0-c0-e1.json')['theta_after']
    first_rec=a.load(out/'estimates/r0-c0-e0.json');second_rec=a.load(out/'estimates/r0-c0-e1.json')
    bad=copy.deepcopy(second_rec);bad['eps'][0][0]+=.1;rejects(lambda:shared_basis(first_rec,bad))
    for case in ('cosine','reverse','selection','context','degenerate'):
        bad=copy.deepcopy(rec)
        if case=='cosine':bad['alignment']['score']+=.1
        elif case=='reverse':bad['parameters'][2][0]+=.1
        elif case=='selection':bad['selection']='best-validation'
        elif case=='context':bad['parameters'][0][0]+=.1
        else:bad['alignment']['degenerate']=not bad['alignment']['degenerate']
        rejects(lambda:direction(bad,0,0,contexts()[0],first,second,h))
    s=a.load(out/'summary.json');expected={k:v for k,v in s.items() if k not in ('attempted_steps','advancing_steps')}
    for case in ('denominator','unit','mean','p','support'):
        bad=copy.deepcopy(s)
        if case=='denominator':bad['fixed_denominator']=7168
        elif case=='unit':bad['statistical_unit']='rollout'
        elif case=='mean':bad['usefulness']['mean']=123.
        elif case=='p':bad['reliability']['sign_p']=.1234567
        else:bad['support_direction']=not bad['support_direction']
        rejects(lambda:check_statistics(bad,expected))
    rejects(lambda:b.batch_fits([[None]]*16,4,h))
    with tempfile.TemporaryDirectory(prefix='ere-direction-integrity-') as tmp:
        root=Path(tmp);p=root/'input';p.write_bytes(b'frozen');sha=a.digest(p);a.seal(root);p.write_bytes(b'changed')
        rejects(lambda:a.verify(root));rejects(lambda:c.check_digest(p,sha))
    return n

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('out',type=Path);parser.add_argument('--tamper-tests',action='store_true');parser.add_argument('--seal',action='store_true')
    args=parser.parse_args();result=audit(args.out)
    if args.tamper_tests:result['tamper_rejections']=tamper_tests(args.out)
    if args.seal:
        (args.out/'audit.json').write_text(json.dumps(result,indent=2));a.seal(args.out);audit(args.out)
    print(json.dumps(result,indent=2))
