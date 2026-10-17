#!/usr/bin/env python3
"""Budget-matched minibatch accounting, not independent physics/RNG replay."""
import argparse
import copy
import json
import math
from pathlib import Path
import tempfile
import audit_ere_complete_feasibility_training as f

a=f.a;c=f.c
PROFILE='E2-complete-budget-matched-minibatch-v1'
SEED=0xE657000001;DEV_SEED=0xE657D00001
LATEST='f51ba18e5cabdedae4bab811e1cb6a3c86654b21355fbad8d07156f95f853432'
ARMS=['single','batch']
EXTRA={'configs/ere_complete_minibatch_v1.json','docs/ere_complete_minibatch_v1.md',
       'scripts/audit_ere_complete_minibatch.py','scripts/test_audit_ere_complete_minibatch.py'}

def design(development=False):
    seed=DEV_SEED if development else SEED
    reps,single,batch,size,draws=(1,2,1,2,2) if development else (8,20,5,4,64)
    cells=reps*2*single*16+draws*4*(1+2*reps)
    return dict(profile=PROFILE,mode='development' if development else 'scientific',master_seed=seed,replicates=reps,
        single_generations=single,batch_generations=batch,batch_size=size,candidates=16,parameters=183,sigma=1.,
        alpha=0.05000000074505806,horizon=100,held_out_draws=draws,held_out_physics=list(range(seed+200000,seed+200004)),
        objective='mean-constraint-utility-v1',statistical_unit='paired_training_replicate',maximum_native_steps=cells*100,
        wall_seconds=900 if development else 3600,cell_wall_seconds=30,output_cap_bytes=2*1024**3,work=50000,results=50000,
        precheck_evaluations=0,champion_selection='final_generation_parameters',full_training_authorized=False)

def schedule(d):
    q=[];seed=d['master_seed']
    for r in range(d['replicates']):
        for arm,name in enumerate(ARMS):
            k=1 if arm==0 else d['batch_size'];gs=d['single_generations'] if arm==0 else d['batch_generations']
            for g in range(gs):
                for p in range(8):
                    for sign in (1,-1):
                        for b in range(k):
                            pool=(8*g+p)*k+b
                            q.append(dict(id=f'train-r{r}-{name}-g{g}-p{p}-s{sign}-b{b}',stage='training',replicate=r,arm=arm,
                                generation=g,pair=p,sign=sign,batch_draw=b,pool_index=pool,
                                emission_seed=seed+10000+1000*r+pool,physics_seed=seed+100000+100*r+pool%4,horizon=100))
    for i in range(d['held_out_draws']):
        for p in range(4):q.append(dict(id=f'hold-untrained-d{i}-p{p}',stage='held_out',replicate=None,arm=2,draw=i,
            emission_seed=seed+20000+i,physics_seed=seed+200000+p,horizon=100))
    for r in range(d['replicates']):
        for i in range(d['held_out_draws']):
            for arm,name in enumerate(ARMS):
                for p in range(4):q.append(dict(id=f'hold-r{r}-{name}-d{i}-p{p}',stage='held_out',replicate=r,arm=arm,draw=i,
                    emission_seed=seed+20000+i,physics_seed=seed+200000+p,horizon=100))
    return q

def exact_schedule(got,expected):
    assert got==expected and len(got)==len(expected)
    for q,e in zip(got,expected):assert q.keys()==e.keys() and all(type(q[k]) is type(v) for k,v in e.items())

def batch_fits(scores,k,h):
    assert len(scores)==16 and k>0 and all(len(v)==k for v in scores),'incomplete candidate batch'
    assert all(x is None or type(x) in (int,float) and math.isfinite(x) and 0<=x<=1 for v in scores for x in v)
    means=[sum(0. if x is None else 1.+h.f32(x) for x in v)/k for v in scores]
    levels=sorted(set(means));return means,[float(levels.index(x)) for x in means]

def sign_p(wins,losses):
    n=wins+losses;return sum(math.comb(n,k) for k in range(wins,n+1))/2**n

def statistics(d,utilities,natives,baseline,counts,complete):
    reps=d['replicates'];draws=d['held_out_draws']
    assert len(utilities)==len(natives)==len(counts)==reps and len(baseline)==draws
    assert all(len(v)==draws and all(len(x)==2 for x in v) for v in utilities+natives)
    def aggregate(values):
        return [[sum(x[a] for x in v)/draws if all(x[a] is not None for x in v) else None for a in range(2)] for v in values]
    us=aggregate(utilities);ns=aggregate(natives)
    known=all(x is not None for v in us for x in v) and all(x is not None for x in baseline)
    diffs=[r[1]-r[0] for r in us if all(x is not None for x in r)]
    wins=sum(x>0 for x in diffs);losses=sum(x<0 for x in diffs)
    mean=sum(diffs)/reps if known else None;p=sign_p(wins,losses) if known else None
    totals=[sum(r[a] for r in counts) for a in range(2)]
    native_known=all(x is not None for r in ns for x in r)
    native_diffs=[r[1]-r[0] for r in ns if all(x is not None for x in r)]
    return dict(profile=PROFILE,mode=d['mode'],measurement_complete=complete,statistical_unit='paired_training_replicate',
        fixed_denominator=reps,draws_per_replicate=draws,all_utility_outcomes_known=known,
        replicate_utility_means=us,replicate_native_means=ns,batch_minus_single_utility=mean,utility_sign_p=p,
        wins=wins,losses=losses,ties=len(diffs)-wins-losses,complete_pairs=len(diffs),
        method_utility_means=[sum(r[a] for r in us)/reps if known else None for a in range(2)],
        untrained_utility_mean=sum(baseline)/draws if all(x is not None for x in baseline) else None,
        feasible_counts=totals,feasibility_denominator_per_method=reps*draws*4,
        batch_minus_single_native=sum(native_diffs)/reps if native_known else None,
        native_sign_p=sign_p(sum(x>0 for x in native_diffs),sum(x<0 for x in native_diffs)) if native_known else None,
        all_comparison_native_valid=native_known,
        support_batching=d['mode']=='scientific' and complete and known and totals[1]>=totals[0] and mean>0 and p<.05,
        independent_physics_replay=False)

def check_statistics(got,expected):
    f.check_statistics(got,expected)
    for k in ('utility_sign_p','native_sign_p'):
        if expected[k] is not None:assert math.isclose(got[k],expected[k],rel_tol=1e-12,abs_tol=1e-300),k

def manifest(m):
    dev=m['design']['mode']=='development';d=design(dev)
    assert m['profile']==PROFILE and m['design']==d and all(type(m['design'][k]) is type(v) for k,v in d.items())
    assert m['scheduled_cells']==d['maximum_native_steps']//100 and m['full_training_authorized'] is False
    assert 0<=m['advancing_steps']<=m['attempted_steps']<=d['maximum_native_steps']
    assert 0<=m['optimizer_generations']<=m['started_generations']<=d['replicates']*(d['single_generations']+d['batch_generations'])
    assert m['inflight_native_work_unknown'] is False and 0<=m['recorded_cells']<=m['scheduled_cells']
    return d

def cell(row,q,catalogue,native,h):
    exact_schedule([row['schedule']],[q]);n,advance,score=native.native(row,catalogue,h)
    if row['status'] in ('Complete','Null'):
        reason=row['evaluation']['reason']
        expected=f.utility(score,reason) if score is not None or reason=='resource_cap' else None
        h.close(row['constraint_utility'],expected)
    elif row['status']=='Incomplete':assert row['constraint_utility'] is None
    return n,advance,score

def generation(rec,qs,theta,rows,native,h,k):
    assert len(qs)==16*k and rec['replicate']==qs[0]['replicate'] and rec['arm']==qs[0]['arm'] and rec['generation']==qs[0]['generation']
    assert rec['theta_before']==theta and len(rec['eps'])==8
    assert rec['evaluation_ids']==[q['id'] for q in qs]
    full=all(rows[q['id']]['status'] in ('Complete','Null') and rows[q['id']].get('constraint_utility') is not None for q in qs)
    scores=[]
    for pair,noise in enumerate(rec['eps']):
        native.vector(noise)
        for sign in (1,-1):
            values=[];candidate=[h.f32(t+h.f32(sign*x)) for t,x in zip(theta,noise)]
            selected=qs[(pair*2+(sign==-1))*k:(pair*2+(sign==-1)+1)*k]
            assert all(q['pair']==pair and q['sign']==sign and q['batch_draw']==b for b,q in enumerate(selected))
            for q in selected:
                row=rows[q['id']]
                if row['status']!='NotRun':native.vec_close(row['parameters'],candidate)
                if row['status'] in ('Complete','Null'):values.append(row['evaluation']['score'])
            scores.append(values)
    if not rec['update_applied']:
        assert rec['candidate_means'] is rec['optimizer_pair_scores'] is rec['theta_after'] is None
        return theta,False
    assert full,'update applied to incomplete candidate batches'
    means,fits=batch_fits(scores,k,h)
    f.check_statistics(rec,dict(candidate_means=means))
    assert rec['optimizer_pair_scores']==[fits[i:i+2] for i in range(0,16,2)]
    native.vec_close(rec['theta_after'],native.update(theta,rec['eps'],fits,8,h))
    return rec['theta_after'],True

def audit(out):
    out=Path(out);a.verify(out);m=a.load(out/'manifest.json');d=manifest(m);plan=schedule(d)
    exact_schedule(a.load(out/'schedule.json'),plan);assert a.digest(out/'schedule.json')==m['schedule_sha256']
    assert {p.name for p in (out/'cells').iterdir()}=={q['id']+'.json' for q in plan}
    if 'snapshot_sha256' not in m:
        assert m['status']=='blocked_before_evaluation' and m['recorded_cells']==m['attempted_steps']==m['advancing_steps']==m['started_generations']==0
        assert m['training_admitted'] is False
        assert all(a.load(out/'cells'/(q['id']+'.json'))==dict(schedule=q,status='NotRun',reason='not_started',evaluation=None) for q in plan)
        s=a.load(out/'summary.json');assert s['support_batching'] is False and s['measurement_complete'] is False
        assert s['batch_minus_single_utility'] is s['utility_sign_p'] is None and s['fixed_denominator']==d['replicates']
        return dict(audit_pass=True,classification='zero_work_refusal',scientific_campaign_complete=False)
    assert a.digest(out/'snapshot.json')==m['snapshot_sha256'];snap=a.load(out/'snapshot.json');assert snap['frozen_before_measurement']
    proof=a.load(out/'parent_receipt.json');parent=Path(proof['parent_archive']);a.verify(parent,expected=c.PARENT,count=1690)
    assert proof['parent_seal']==c.PARENT and proof['sealed_files_verified']==1690 and proof['native_sources_verified']==230 and proof['champion_sha256']==c.CHAMPION
    assert proof['prior_improvement_supported'] is False and proof['prior_sign_p']==.109375
    latest=a.load(out/'latest_receipt.json');assert latest['root_seal']==LATEST and latest['sealed_files_verified']==6944 and latest['scope']=='diagnostic lineage, not improvement'
    a.verify(Path(latest['archive']),expected=LATEST,count=6944)
    oldaudit=a.load(Path(latest['archive'])/'audit.json');assert oldaudit['audit_pass'] and oldaudit['training_complete'] and not oldaudit['constraint_utility']['improvement_supported']
    pins={n:v for n,v in a.load(parent/'manifest.json')['snapshot']['sources'].items() if a.pinned(n)};assert len(pins)==230
    train={str(p.relative_to(out/'source_snapshots')) for p in (out/'source_snapshots/crates/hg-train').rglob('*') if p.is_file() and p.suffix in ('.rs','.toml')}
    assert set(snap['sources'])==set(pins)|train|c.EXTRA|f.EXTRA|EXTRA and a.files(out/'source_snapshots')==set(snap['sources'])
    for n,sha in snap['sources'].items():
        assert a.safe(n) and a.digest(out/'source_snapshots'/n)==sha
        if n in pins:assert sha==pins[n]
    for n in ('scripts/audit_ere_complete_admission.py','scripts/audit_ere_complete_scaling.py'):
        assert snap['sources'][n]==a.load(parent/'manifest.json')['snapshot']['sources'][n]
    assert a.digest(out/'executed_binary')==snap['binary_sha256']
    cfg=a.load(out/'source_snapshots/configs/ere_complete_minibatch_v1.json');assert cfg==dict(scientific=design(False),development=design(True))
    prior=a.load(out/'parent_audit.json');assert prior['audit_pass'] and prior['training_complete'] and prior['precheck_pass']
    if not (out/'catalogue.json').exists():
        assert m['status']=='blocked_before_evaluation' and m['recorded_cells']==m['attempted_steps']==m['advancing_steps']==m['started_generations']==0
        assert all(a.load(out/'cells'/(q['id']+'.json'))['status']=='NotRun' for q in plan)
        return dict(audit_pass=True,classification='incomplete_setup',scientific_campaign_complete=False)
    assert a.digest(out/'catalogue.json')==a.CATALOGUE and m['grammar_compilations']==1
    native,h=c.helpers(parent);catalogue=a.load(out/'catalogue.json')
    attempts=advances=recorded=unknown=valid=null=0;peaks=[0,0,0,0];notrun=False
    def checked(q):
        nonlocal attempts,advances,recorded,unknown,valid,null,notrun
        row=a.load(out/'cells'/(q['id']+'.json'));n,advance,score=cell(row,q,catalogue,native,h)
        attempts+=n;advances+=advance
        if row['status']=='NotRun':notrun=True
        else:
            assert not notrun,'fixed sequence gap';recorded+=1;unknown+=row['status']=='Incomplete';valid+=row['status']=='Complete';null+=row['status']=='Null'
            if row['evaluation'] is not None:
                for s in row['evaluation']['steps']:
                    for i,v in enumerate((s['usage']['work'],s['usage']['results'],s['nodes'],s['edges'])):peaks[i]=max(peaks[i],v)
        return row,score
    genfiles=set();champfiles=set();offset=0;started=completed=0;champions={};training_null=0
    for rep in range(d['replicates']):
        for arm,name in enumerate(ARMS):
            theta=[0.]*183;k=1 if arm==0 else d['batch_size'];gs=d['single_generations'] if arm==0 else d['batch_generations'];done=0
            for g in range(gs):
                qs=plan[offset:offset+16*k];offset+=16*k;rows={}
                for q in qs:
                    row,_=checked(q);rows[q['id']]=row;training_null+=row['status']=='Null'
                filename=f'r{rep}-{name}-g{g}.json';path=out/'generations'/filename
                if path.exists():
                    assert done==g,'generation start gap';genfiles.add(filename);started+=1
                    theta,full=generation(a.load(path),qs,theta,rows,native,h,k);completed+=full;done+=full
                else:assert all(r['status']=='NotRun' for r in rows.values())
            filename=f'r{rep}-{name}.json';path=out/'champions'/filename
            if path.exists():
                assert done==gs;champfiles.add(filename);champ=a.load(path)
                assert champ==dict(parameters=theta,replicate=rep,arm=arm,generation=gs,selection='final_generation_parameters')
                champions[(rep,arm)]=theta
    assert {p.name for p in (out/'generations').iterdir()}==genfiles and {p.name for p in (out/'champions').iterdir()}==champfiles
    assert (started,completed)==(m['started_generations'],m['optimizer_generations'])
    baseline=[];baseline_valid=0
    for i in range(d['held_out_draws']):
        us=[];emission=None
        for _ in range(4):
            q=plan[offset];offset+=1;row,score=checked(q);us.append(row.get('constraint_utility'));baseline_valid+=score is not None
            if row['status']!='NotRun':
                assert row['parameters']==[0.]*183 and len(champions)==2*d['replicates'],'baseline before all champions fixed'
                if emission is None:emission=row['emission']
                else:assert row['emission']==emission
        baseline.append(sum(us)/4 if all(x is not None for x in us) else None)
    utilities=[];natives=[];counts=[]
    for rep in range(d['replicates']):
        uv=[];nv=[];vc=[0,0]
        for i in range(d['held_out_draws']):
            ud=[];nd=[]
            for arm in range(2):
                values=[];us=[];emission=None
                for _ in range(4):
                    q=plan[offset];offset+=1;row,score=checked(q);values.append(score);us.append(row.get('constraint_utility'));vc[arm]+=score is not None
                    if row['status']!='NotRun':
                        assert row['parameters']==champions[(rep,arm)]
                        if emission is None:emission=row['emission']
                        else:assert row['emission']==emission
                nd.append(sum(values)/4 if all(x is not None for x in values) else None)
                ud.append(sum(us)/4 if all(x is not None for x in us) else None)
            uv.append(ud);nv.append(nd)
        utilities.append(uv);natives.append(nv);counts.append(vc)
    assert offset==len(plan) and (recorded,attempts,advances,unknown)==(m['recorded_cells'],m['attempted_steps'],m['advancing_steps'],m['unknown_partial_work_cells'])
    complete=m['status']=='complete'
    if complete:
        assert recorded==len(plan) and unknown==0 and completed==d['replicates']*(d['single_generations']+d['batch_generations']) and m['live_sources_rechecked'] and m['training_admitted']
        assert 0<=m['grammar_compile_seconds']<=m['elapsed_seconds']<d['wall_seconds']
    else:assert m['status'] in ('incomplete','blocked_before_evaluation') and isinstance(m['reason'],str)
    expected=statistics(d,utilities,natives,baseline,counts,complete);summary=a.load(out/'summary.json');check_statistics(summary,expected)
    if complete:assert summary['all_utility_outcomes_known']
    assert summary['attempted_steps']==attempts and summary['advancing_steps']==advances
    assert sum((out/p).stat().st_size for p in a.files(out))<d['output_cap_bytes']
    return dict(audit_pass=True,classification='bounded_development' if d['mode']=='development' else 'budget_matched_minibatch_trial',
        campaign_complete=complete,scientific_campaign_complete=complete and d['mode']=='scientific',support_batching=summary['support_batching'],
        recorded_cells=recorded,valid_evaluations=valid,null_evaluations=null,training_null_evaluations=training_null,
        attempted_steps=attempts,advancing_steps=advances,optimizer_generations=completed,frozen_sources=len(snap['sources']),
        peak_work=peaks[0],peak_results=peaks[1],peak_nodes=peaks[2],peak_edges=peaks[3],baseline_valid_evaluations=baseline_valid,
        statistical_unit='paired_training_replicate',fixed_denominator=d['replicates'],batch_minus_single_utility=summary['batch_minus_single_utility'],
        utility_sign_p=summary['utility_sign_p'],independent_physics_replay=False)

def tamper_tests(out):
    out=Path(out);m=a.load(out/'manifest.json');d=manifest(m);plan=schedule(d);catalogue=a.load(out/'catalogue.json');parent=Path(a.load(out/'parent_receipt.json')['parent_archive'])
    native,h=c.helpers(parent);n=0
    def rejects(fn):
        nonlocal n
        try:fn()
        except (AssertionError,KeyError,ValueError,IndexError):n+=1;return
        raise AssertionError('tamper accepted')
    for key,value in [('scheduled_cells',1),('full_training_authorized',True),('started_generations',0)]:
        bad=copy.deepcopy(m);bad[key]=value;rejects(lambda:manifest(bad))
    for key,value in [('batch_size',8),('horizon',1000),('statistical_unit','rollout'),('master_seed',d['master_seed']+1),('alpha',.05)]:
        bad=copy.deepcopy(m);bad['design'][key]=value;rejects(lambda:manifest(bad))
    q=plan[0];row=a.load(out/'cells'/(q['id']+'.json'))
    for case in ['seed','score','credit','cap','allowance','membership','event','utility','batch_draw_type']:
        bad=copy.deepcopy(row)
        if case=='seed':bad['schedule']['emission_seed']+=1
        elif case=='score':bad['evaluation']['score']=.12345
        elif case=='credit':bad['evaluation']['proposal_fires']+=1
        elif case=='cap':bad['evaluation']['steps'][0]['cap']=True
        elif case=='allowance':bad['evaluation']['steps'][0]['usage']['rules'][0]['work_allowance']+=1
        elif case=='membership':bad['emission']['signature']='bad'
        elif case=='event':bad['evaluation']['steps'][0]['rule_ids']=[999999]
        elif case=='utility':bad['constraint_utility']+=1
        else:bad['schedule']['batch_draw']=False
        rejects(lambda:cell(bad,q,catalogue,native,h))
    qs=[q for q in plan if q['stage']=='training' and q['replicate']==0 and q['arm']==1 and q['generation']==0]
    rows={q['id']:a.load(out/'cells'/(q['id']+'.json')) for q in qs};rec=a.load(out/'generations/r0-batch-g0.json')
    for case in ['candidate','mean','fit','update','ids','partial_batch']:
        bad=copy.deepcopy(rec);badrows=copy.deepcopy(rows)
        if case=='candidate':badrows[qs[1]['id']]['parameters'][0]+=.1
        elif case=='mean':bad['candidate_means'][0]+=.1
        elif case=='fit':bad['optimizer_pair_scores'][0][0]+=1
        elif case=='update':bad['theta_after'][0]+=.1
        elif case=='ids':bad['evaluation_ids'].reverse()
        else:badrows[qs[1]['id']]['status']='NotRun'
        rejects(lambda:generation(bad,qs,[0.]*183,badrows,native,h,d['batch_size']))
    s=a.load(out/'summary.json');expected={k:v for k,v in s.items() if k not in ('attempted_steps','advancing_steps')}
    for key,value in [('fixed_denominator',4096),('statistical_unit','rollout'),('batch_minus_single_utility',123.),('utility_sign_p',.1234567),('support_batching',not s['support_batching'])]:
        bad=copy.deepcopy(s);bad[key]=value;rejects(lambda:check_statistics(bad,expected))
    rejects(lambda:batch_fits([[None]]*16,4,h))
    with tempfile.TemporaryDirectory(prefix='ere-minibatch-integrity-') as tmp:
        root=Path(tmp);p=root/'input';p.write_bytes(b'frozen');sha=a.digest(p);a.seal(root);p.write_bytes(b'changed')
        rejects(lambda:a.verify(root));rejects(lambda:c.check_digest(p,sha))
    return n

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('out',type=Path);p.add_argument('--tamper-tests',action='store_true');p.add_argument('--seal',action='store_true')
    args=p.parse_args();result=audit(args.out)
    if args.tamper_tests:result['tamper_rejections']=tamper_tests(args.out)
    if args.seal:
        (args.out/'audit.json').write_text(json.dumps(result,indent=2));a.seal(args.out);audit(args.out)
    print(json.dumps(result,indent=2))
