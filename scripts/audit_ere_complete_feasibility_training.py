#!/usr/bin/env python3
"""Frozen feasibility-aware ES accounting audit, not physics/RNG replay."""
import argparse
import copy
import json
import math
from pathlib import Path
import tempfile
import audit_ere_complete_confirmation as c

a=c.a
PROFILE='E2-complete-feasibility-training-v1'
SEED=0xE656000001
EXTRA={'configs/ere_complete_feasibility_training_v1.json','docs/ere_complete_feasibility_training_v1.md',
       'scripts/audit_ere_complete_feasibility_training.py','scripts/test_audit_ere_complete_feasibility_training.py'}

def design():
    return dict(profile=PROFILE,master_seed=SEED,horizon=100,parameters=183,initial_parameter_value=0.,generations=20,candidates=16,
        sigma=1.,alpha=0.05000000074505806,held_out_draws=512,training_physics=list(range(SEED+100000,SEED+100004)),
        held_out_physics=list(range(SEED+200000,SEED+200004)),work=50000,results=50000,wall_seconds=3600,cell_wall_seconds=30,
        output_cap_bytes=2*1024**3,maximum_native_steps=646400,objective='feasible-first-reuse-v1',
        champion_selection='final_generation_parameters',precheck_evaluations=0,full_training_authorized=False)

def schedule():
    training=[dict(id=f'train-{g}-{p}-{sign}',stage='training',generation=g,pair=p,sign=sign,
        emission_seed=SEED+5000+8*g+p,physics_seed=SEED+100000+(g+p)%4,horizon=100)
        for g in range(20) for p in range(8) for sign in (1,-1)]
    held=[dict(id=f'd{i}-{name}-p{p}',stage='held_out',draw=i,arm=arm,
        emission_seed=SEED+2000+i,random_parameter_seed=SEED+3000+i,physics_seed=SEED+200000+p,horizon=100)
        for i in range(512) for arm,name in enumerate(c.ARMS) for p in range(4)]
    return training+held

def utility(score,reason):
    if score is not None:
        assert type(score) in (int,float) and 0<=score<=1 and reason is None
        return 1.+score
    assert reason=='resource_cap','unsupported null outcome'
    return 0.

def optimizer_fits(scores,h):
    assert len(scores)==16
    keys=[1.+h.f32(x) if x is not None else 0. for x in scores]
    assert all(x is None or 0<=x<=1 for x in scores)
    levels=sorted(set(keys))
    return [float(levels.index(k)) for k in keys]

def statistics(native_means,utility_means,counts,complete):
    s=c.statistics(native_means);s['improvement_supported'] &= complete
    known=all(x is not None for d in utility_means for x in d)
    diffs=[d[0]-d[1] for d in utility_means if d[0] is not None and d[1] is not None]
    wins=sum(x>0 for x in diffs);losses=sum(x<0 for x in diffs)
    primary=sum(diffs)/512 if known else None
    n=wins+losses
    p=sum(c.math.comb(n,k) for k in range(wins,n+1))/2**n if known else None
    s.update(profile=PROFILE,measurement_complete=complete,
        constraint_utility=dict(all_outcomes_known=known,fixed_denominator=512,
        arm_means={name:sum(d[arm] for d in utility_means)/512 if known else None for arm,name in enumerate(c.ARMS)},
        champion_minus_untrained=primary,champion_minus_random=sum(d[0]-d[2] for d in utility_means)/512 if known else None,
        wins=wins,losses=losses,ties=len(diffs)-wins-losses,complete_pairs=len(diffs),one_sided_paired_sign_p=p,
        improvement_supported=complete and known and counts[0]>=counts[1] and primary>0 and p<.05,draw_means=utility_means),
        feasibility=dict(denominator_per_arm=2048,valid_counts=counts,rates=[n/2048 for n in counts] if known else None,
            champion_minus_untrained=(counts[0]-counts[1])/2048 if known else None))
    return s

def check_manifest(m):
    assert m['profile']==PROFILE and m['design']==design()
    assert all(type(m['design'][k]) is type(v) for k,v in design().items())
    assert m['scheduled_cells']==6464 and m['full_training_authorized'] is False
    assert 0<=m['recorded_cells']<=6464 and 0<=m['optimizer_generations']<=20
    assert m['inflight_native_work_unknown'] is False
    assert 0<=m['advancing_steps']<=m['attempted_steps']<=646400

def check_statistics(got,expected):
    def equal(g,v,key):
        if type(v) is float:
            assert type(g) is float and math.isclose(g,v,rel_tol=1e-12,
                abs_tol=1e-300 if key=='one_sided_paired_sign_p' else 1e-12),key
        elif isinstance(v,dict):
            assert isinstance(g,dict) and g.keys()==v.keys(),key
            for k,x in v.items():equal(g[k],x,k)
        elif isinstance(v,list):
            assert isinstance(g,list) and len(g)==len(v),key
            for a,b in zip(g,v):equal(a,b,key)
        else:assert type(g) is type(v) and g==v,key
    for k,v in expected.items():equal(got[k],v,k)

def cell(row,q,catalogue,native,h):
    assert row['schedule']==q
    counts=native.native(row,catalogue,h)
    if row['status'] in ('Complete','Null'):
        expected=utility(counts[2],row['evaluation']['reason'])
        h.close(row['constraint_utility'],expected)
    elif row['status']=='Incomplete':assert row['constraint_utility'] is None
    return counts

def panel(rows,catalogue,h):
    assert len(rows)==64 and [r['seed'] for r in rows]==list(range(SEED+1000,SEED+1064))
    for r in rows:
        h.check_membership(catalogue,r['selection'],full=True)
        assert r['selection']['emission_seed']==r['seed'] and r['signature']==r['selection']['signature']

def generation(rec,g,theta,rows,native,h,catalogue,initial):
    assert rec['generation']==g and rec['theta_before']==theta and rec['update_applied'] is True
    assert len(rec['eps'])==8
    fits=[];scores=[];ids=[]
    for p,eps in enumerate(rec['eps']):
        native.vector(eps)
        for sign in (1,-1):
            q=schedule()[g*16+p*2+(sign==-1)];row=rows[q['id']];ids.append(q['id'])
            assert row['status'] in ('Complete','Null')
            native.vec_close(row['parameters'],[h.f32(t+h.f32(sign*x)) for t,x in zip(theta,eps)])
            scores.append(row['evaluation']['score'])
    assert rec['evaluation_ids']==ids and rec['native_scores']==scores
    fits=optimizer_fits(scores,h)
    assert rec['optimizer_pair_scores']==[fits[i:i+2] for i in range(0,16,2)]
    native.vec_close(rec['theta_after'],native.update(theta,rec['eps'],fits,8,h))
    panel(rec['distribution'],catalogue,h);h.close(rec['tv_from_initial'],h.distance(initial,rec['distribution']))
    return rec['theta_after']

def audit(out):
    out=Path(out);a.verify(out);m=a.load(out/'manifest.json');check_manifest(m)
    plan=a.load(out/'schedule.json');assert plan==schedule() and a.digest(out/'schedule.json')==m['schedule_sha256']
    assert {p.name for p in (out/'cells').iterdir()}=={q['id']+'.json' for q in plan}
    if 'snapshot_sha256' not in m:
        assert m['status']=='blocked_before_evaluation' and m['recorded_cells']==m['attempted_steps']==m['advancing_steps']==0
        assert m['training_admitted'] is False and m['optimizer_generations']==0
        assert all(a.load(out/'cells'/(q['id']+'.json'))==dict(schedule=q,status='NotRun',reason='not_started',evaluation=None) for q in plan)
        return dict(audit_pass=True,classification='zero_work_refusal',training_complete=False)
    assert a.digest(out/'snapshot.json')==m['snapshot_sha256']
    snap=a.load(out/'snapshot.json');assert snap['frozen_before_measurement']
    proof=a.load(out/'parent_receipt.json');parent=Path(proof['parent_archive'])
    assert proof['parent_seal']==c.PARENT and proof['sealed_files_verified']==1690 and proof['native_sources_verified']==230
    assert proof['champion_sha256']==c.CHAMPION and proof['prior_improvement_supported'] is False and proof['prior_sign_p']==.109375
    a.verify(parent,expected=c.PARENT,count=1690);assert a.digest(parent/'champion.json')==c.CHAMPION
    prior=a.load(out/'parent_audit.json');assert prior['audit_pass'] and prior['training_complete'] and prior['precheck_pass']
    pins={k:v for k,v in a.load(parent/'manifest.json')['snapshot']['sources'].items() if a.pinned(k)};assert len(pins)==230
    train={str(p.relative_to(out/'source_snapshots')) for p in (out/'source_snapshots/crates/hg-train').rglob('*') if p.is_file() and p.suffix in ('.rs','.toml')}
    assert set(snap['sources'])==set(pins)|train|c.EXTRA|EXTRA
    assert a.files(out/'source_snapshots')==set(snap['sources'])
    for n,sha in snap['sources'].items():
        assert a.safe(n) and a.digest(out/'source_snapshots'/n)==sha
        if n in pins:assert sha==pins[n]
    for n in ('scripts/audit_ere_complete_admission.py','scripts/audit_ere_complete_scaling.py'):
        assert snap['sources'][n]==a.load(parent/'manifest.json')['snapshot']['sources'][n]
    assert a.digest(out/'executed_binary')==snap['binary_sha256']
    assert a.load(out/'source_snapshots/configs/ere_complete_feasibility_training_v1.json')==design()
    assert a.digest(out/'catalogue.json')==a.CATALOGUE and m['grammar_compilations']==1
    catalogue=a.load(out/'catalogue.json');native,h=c.helpers(parent)
    attempts=advances=recorded=unknown=valid=null=0;notrun=False;peaks=[0,0,0,0];native_means=[];utility_means=[];counts=[0,0,0]
    def checked(q):
        nonlocal attempts,advances,recorded,unknown,valid,null,notrun
        row=a.load(out/'cells'/(q['id']+'.json'));n,advance,score=cell(row,q,catalogue,native,h)
        attempts+=n;advances+=advance
        if row['status']=='NotRun':notrun=True
        else:
            assert not notrun,'fixed sequence gap';recorded+=1;unknown+=row['status']=='Incomplete'
            valid+=row['status']=='Complete';null+=row['status']=='Null'
            if row['evaluation'] is not None:
                for s in row['evaluation']['steps']:
                    for i,v in enumerate((s['usage']['work'],s['usage']['results'],s['nodes'],s['edges'])):peaks[i]=max(peaks[i],v)
        return row,score
    theta=[0.]*183;initial=a.load(out/'initial_distribution.json');panel(initial,catalogue,h)
    training_rows={};gen_count=0;training_null=0;max_tv=0.
    for g in range(20):
        rows={}
        for q in plan[g*16:(g+1)*16]:
            row,_=checked(q);rows[q['id']]=row;training_null+=row['status']=='Null'
        training_rows.update(rows)
        path=out/'generations'/f'train-{g}.json'
        if path.exists():
            assert gen_count==g;rec=a.load(path);theta=generation(rec,g,theta,rows,native,h,catalogue,initial)
            gen_count+=1;max_tv=max(max_tv,rec['tv_from_initial'])
    assert {p.name for p in (out/'generations').iterdir()}=={f'train-{g}.json' for g in range(gen_count)}
    assert gen_count==m['optimizer_generations']
    if (out/'champion.json').exists():
        champion=a.load(out/'champion.json');assert gen_count==20 and champion==dict(parameters=theta,selection='final_generation_parameters',generation=20)
    vectors=set()
    for i in range(512):
        nm=[];um=[]
        for arm in range(3):
            scores=[];us=[];emission=parameters=None
            for p in range(4):
                q=plan[320+i*12+arm*4+p];row,score=checked(q);scores.append(score);us.append(row.get('constraint_utility'))
                counts[arm]+=score is not None
                if row['status']!='NotRun':
                    if emission is None:emission=row['emission'];parameters=row['parameters']
                    else:assert row['emission']==emission and row['parameters']==parameters
                    if arm==0:assert row['parameters']==theta
                    if arm==1:assert row['parameters']==[0.]*183
            nm.append(sum(scores)/4 if all(x is not None for x in scores) else None)
            um.append(sum(us)/4 if all(x is not None for x in us) else None)
            if arm==2 and parameters is not None:vectors.add(tuple(parameters));assert any(x!=0 for x in parameters)
        native_means.append(nm);utility_means.append(um)
    complete=m['status']=='complete'
    assert (recorded,attempts,advances,unknown)==(m['recorded_cells'],m['attempted_steps'],m['advancing_steps'],m['unknown_partial_work_cells'])
    if complete:
        assert recorded==6464 and unknown==0 and gen_count==20 and len(vectors)==512 and m['live_sources_rechecked'] and m['training_admitted']
        assert 0<=m['grammar_compile_seconds']<=m['elapsed_seconds']<3600
    else:assert m['status'] in ('incomplete','blocked_before_evaluation') and isinstance(m['reason'],str)
    expected=statistics(native_means,utility_means,counts,complete);summary=a.load(out/'summary.json')
    check_statistics(summary,expected)
    assert summary['attempted_steps']==attempts and summary['advancing_steps']==advances
    assert sum((out/p).stat().st_size for p in a.files(out))<2*1024**3
    return dict(audit_pass=True,classification='feasibility_aware_training',training_complete=complete,recorded_cells=recorded,
        valid_evaluations=valid,null_evaluations=null,training_null_evaluations=training_null,held_out_null_evaluations=null-training_null,
        optimizer_generations=gen_count,attempted_steps=attempts,advancing_steps=advances,frozen_sources=len(snap['sources']),
        peak_work=peaks[0],peak_results=peaks[1],peak_nodes=peaks[2],peak_edges=peaks[3],max_training_distribution_tv=max_tv,
        native_reuse_improvement_supported=summary['improvement_supported'],constraint_utility=summary['constraint_utility'],
        feasibility=summary['feasibility'],independent_physics_replay=False)

def tamper_tests(out):
    out=Path(out);m=a.load(out/'manifest.json');s=a.load(out/'summary.json');parent=Path(a.load(out/'parent_receipt.json')['parent_archive'])
    native,h=c.helpers(parent);catalogue=a.load(out/'catalogue.json');initial=a.load(out/'initial_distribution.json')
    n=0
    def rejects(fn):
        nonlocal n
        try:fn()
        except (AssertionError,KeyError,ValueError,IndexError):n+=1;return
        raise AssertionError('tamper accepted')
    for k,v in [('scheduled_cells',6144),('full_training_authorized',True),('optimizer_generations',21)]:
        bad=copy.deepcopy(m);bad[k]=v;rejects(lambda:check_manifest(bad))
    for k,v in [('objective','native-reuse'),('master_seed',SEED+1),('horizon',1000),('alpha',.05),('work',400000)]:
        bad=copy.deepcopy(m);bad['design'][k]=v;rejects(lambda:check_manifest(bad))
    q=schedule()[0];row=a.load(out/'cells'/(q['id']+'.json'))
    for case in ['seed','score','credit','cap','allowance','membership','event','utility']:
        bad=copy.deepcopy(row)
        if case=='seed':bad['schedule']['physics_seed']+=1
        elif case=='score':bad['evaluation']['score']=.12345
        elif case=='credit':bad['evaluation']['proposal_fires']+=1
        elif case=='cap':bad['evaluation']['steps'][0]['cap']=True
        elif case=='allowance':bad['evaluation']['steps'][0]['usage']['rules'][0]['work_allowance']+=1
        elif case=='membership':bad['emission']['signature']='false'
        elif case=='event':bad['evaluation']['steps'][0]['rule_ids']=[999999]
        else:bad['constraint_utility']+=1
        rejects(lambda:cell(bad,q,catalogue,native,h))
    rec=a.load(out/'generations/train-0.json');rows={q['id']:a.load(out/'cells'/(q['id']+'.json')) for q in schedule()[:16]}
    for case in ['update','fit','score','candidate','selection']:
        bad=copy.deepcopy(rec);badrows=copy.deepcopy(rows)
        if case=='update':bad['theta_after'][0]+=.1
        elif case=='fit':bad['optimizer_pair_scores'][0][0]+=1
        elif case=='score':bad['native_scores'][0]=.12345
        elif case=='candidate':badrows[schedule()[0]['id']]['parameters'][0]+=.1
        else:bad['evaluation_ids'].reverse()
        rejects(lambda:generation(bad,0,[0.]*183,badrows,native,h,catalogue,initial))
    expected={k:v for k,v in s.items() if k not in ('attempted_steps','advancing_steps')}
    for case in ['denominator','native','utility','p','feasibility']:
        bad=copy.deepcopy(s)
        if case=='denominator':bad['fixed_denominator']=64
        elif case=='native':bad['champion_minus_untrained']=123.
        elif case=='utility':bad['constraint_utility']['champion_minus_untrained']=123.
        elif case=='p':bad['constraint_utility']['one_sided_paired_sign_p']=.987654
        else:bad['feasibility']['denominator_per_arm']=2047
        rejects(lambda:check_statistics(bad,expected))
    rejects(lambda:utility(None,'invalid_family'))
    with tempfile.TemporaryDirectory(prefix='ere-feasible-integrity-') as tmp:
        root=Path(tmp);p=root/'input';p.write_bytes(b'frozen');sha=a.digest(p);a.seal(root)
        p.write_bytes(b'changed');rejects(lambda:a.verify(root));rejects(lambda:c.check_digest(p,sha))
    return n

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('out',type=Path);parser.add_argument('--tamper-tests',action='store_true');parser.add_argument('--seal',action='store_true')
    args=parser.parse_args();result=audit(args.out)
    if args.tamper_tests:result['tamper_rejections']=tamper_tests(args.out)
    # Detailed utility draw means stay in summary.json, not duplicated in audit.
    result['constraint_utility'].pop('draw_means',None)
    if args.seal:
        (args.out/'audit.json').write_text(json.dumps(result,indent=2));a.seal(args.out);audit(args.out)
    print(json.dumps(result,indent=2))
