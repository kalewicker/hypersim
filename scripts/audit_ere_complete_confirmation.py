#!/usr/bin/env python3
"""Streaming accounting audit of a prospectively fixed champion confirmation."""
import argparse
import copy
import importlib.util
import json
import math
from pathlib import Path
import sys
import tempfile
import audit_ere_complete_admission as a

sys.dont_write_bytecode=True
PROFILE='E2-complete-fixed-champion-confirmation-v1'
SEED=0xE655000001
PARENT='dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132'
CHAMPION='14cf0b13540167b3a3b20082db71ddd9bbd761a5f5b0d81083dd6d719e24988f'
ARMS=['champion','untrained','random']
EXTRA={'configs/ere_complete_champion_confirmation_v1.json','docs/ere_complete_champion_confirmation_v1.md',
       'scripts/audit_ere_complete_confirmation.py','scripts/test_audit_ere_complete_confirmation.py',
       'scripts/audit_ere_complete_admission.py','scripts/audit_ere_complete_scaling.py'}

def design():
    return dict(profile=PROFILE,master_seed=SEED,draws=512,physics_seeds=list(range(SEED+100000,SEED+100004)),
        arms=ARMS,horizon=100,parameters=183,work=50000,results=50000,wall_seconds=3600,cell_wall_seconds=30,
        output_cap_bytes=2*1024**3,optimizer_generations=0,maximum_native_steps=614400,champion_selection='sealed_final_generation_parameters')

def schedule():
    return [dict(id=f'd{i}-{name}-p{p}',stage='held_out',draw=i,arm=arm,emission_seed=SEED+2000+i,
        random_parameter_seed=SEED+3000+i,physics_seed=SEED+100000+p,horizon=100)
        for i in range(512) for arm,name in enumerate(ARMS) for p in range(4)]

def check_manifest(m):
    assert m['profile']==PROFILE and m['design']==design()
    assert all(type(m['design'][k]) is type(v) for k,v in design().items())
    assert m['scheduled_cells']==6144 and m['optimizer_generations']==0
    assert 0<=m['recorded_cells']<=6144 and m['inflight_native_work_unknown'] is False
    assert 0<=m['attempted_steps']<=614400 and 0<=m['advancing_steps']<=m['attempted_steps']

def helpers(parent):
    path=parent/'source_snapshots/scripts/audit_ere_complete_short_training.py'
    spec=importlib.util.spec_from_file_location('pinned_short_accounting',path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    runtime=Path(a.load(parent/'manifest.json')['receipt_paths']['runtime'])
    return module,a.helper(runtime)

def statistics(draws):
    assert len(draws)==512 and all(len(d)==3 for d in draws)
    all_valid=all(x is not None for d in draws for x in d)
    differences=[d[0]-d[1] for d in draws if d[0] is not None and d[1] is not None]
    random=[d[0]-d[2] for d in draws if d[0] is not None and d[2] is not None]
    wins=sum(x>0 for x in differences);losses=sum(x<0 for x in differences);n=wins+losses
    primary=sum(differences)/512 if all_valid else None
    p=sum(math.comb(n,k) for k in range(wins,n+1))/2**n if all_valid else None
    return dict(fixed_denominator=512,all_valid=all_valid,complete_primary_pairs=len(differences),complete_secondary_pairs=len(random),
        arm_means={name:sum(d[arm] for d in draws)/512 if all_valid else None for arm,name in enumerate(ARMS)},
        champion_minus_untrained=primary,champion_minus_random=sum(random)/512 if all_valid else None,
        wins=wins,losses=losses,ties=len(differences)-wins-losses,one_sided_paired_sign_p=p,
        improvement_supported=all_valid and primary>0 and p<.05,draw_means=draws)

def check_statistics(summary,expected,complete):
    expected={**expected,'improvement_supported':expected['improvement_supported'] and complete}
    def equal(got,v,key):
        if type(v) is float:
            tolerance=1e-300 if key=='one_sided_paired_sign_p' else 1e-12
            assert type(got) is float and math.isclose(got,v,rel_tol=1e-12,abs_tol=tolerance),key
        elif isinstance(v,dict):
            assert isinstance(got,dict) and got.keys()==v.keys(),key
            for k,value in v.items():equal(got[k],value,key)
        elif isinstance(v,list):
            assert isinstance(got,list) and len(got)==len(v),key
            for g,value in zip(got,v):equal(g,value,key)
        else:assert type(got) is type(v) and got==v,key
    for k,v in expected.items():
        equal(summary[k],v,k)

def check_digest(path,expected):
    assert a.digest(path)==expected,'file digest'

def cell(row,q,theta,catalogue,native,h):
    assert row['schedule']==q
    if row['status']=='NotRun':return native.native(row,catalogue,h)
    expected=theta if q['arm']==0 else [0.]*183 if q['arm']==1 else row['parameters']
    assert row['parameters']==expected
    if q['arm']==2:assert any(x!=0 for x in row['parameters'])
    return native.native(row,catalogue,h)

def audit(out):
    out=Path(out);a.verify(out)
    m=a.load(out/'manifest.json');check_manifest(m)
    plan=a.load(out/'schedule.json');assert plan==schedule() and a.digest(out/'schedule.json')==m['schedule_sha256']
    assert {p.name for p in (out/'cells').iterdir()}=={q['id']+'.json' for q in plan}
    if 'snapshot_sha256' not in m:
        assert m['status']=='blocked_before_evaluation' and m['recorded_cells']==m['attempted_steps']==m['advancing_steps']==0
        assert all(a.load(out/'cells'/(q['id']+'.json'))==dict(schedule=q,status='NotRun',reason='not_started',evaluation=None) for q in plan)
        return dict(audit_pass=True,classification='zero_work_refusal',confirmation_complete=False)
    assert a.digest(out/'snapshot.json')==m['snapshot_sha256']
    snap=a.load(out/'snapshot.json');assert snap['frozen_before_measurement']
    proof=a.load(out/'parent_receipt.json');parent=Path(proof['parent_archive'])
    assert proof['parent_seal']==PARENT and proof['sealed_files_verified']==1690 and proof['champion_sha256']==CHAMPION and proof['native_sources_verified']==230
    assert proof['prior_improvement_supported'] is False and proof['prior_sign_p']==.109375
    a.verify(parent,expected=PARENT,count=1690)
    assert a.digest(out/'champion.json')==a.digest(parent/'champion.json')==CHAMPION
    theta=a.load(out/'champion.json')['parameters'];native,h=helpers(parent);native.vector(theta)
    pins={k:v for k,v in a.load(parent/'manifest.json')['snapshot']['sources'].items() if a.pinned(k)}
    assert len(pins)==230
    train={str(p.relative_to(out/'source_snapshots')) for p in (out/'source_snapshots/crates/hg-train').rglob('*') if p.is_file() and p.suffix in ('.rs','.toml')}
    assert set(snap['sources'])==set(pins)|train|EXTRA and a.files(out/'source_snapshots')==set(snap['sources'])
    check_digest(out/'executed_binary',snap['binary_sha256'])
    for name,sha in snap['sources'].items():
        assert a.safe(name),name
        check_digest(out/'source_snapshots'/name,sha)
        if name in pins:assert sha==pins[name],name
    for name in ('scripts/audit_ere_complete_admission.py','scripts/audit_ere_complete_scaling.py'):
        assert snap['sources'][name]==a.load(parent/'manifest.json')['snapshot']['sources'][name]
    assert a.load(out/'source_snapshots/configs/ere_complete_champion_confirmation_v1.json')==design()
    if not (out/'catalogue.json').exists():
        assert m['status']=='blocked_before_evaluation' and m['recorded_cells']==0
        return dict(audit_pass=True,classification='incomplete_setup',confirmation_complete=False)
    prior=a.load(out/'parent_audit.json');assert prior['audit_pass'] and prior['training_complete'] and prior['precheck_pass'] and prior['held_out_improvement_supported'] is False
    assert a.digest(out/'catalogue.json')==a.CATALOGUE and m['grammar_compilations']==1
    catalogue=a.load(out/'catalogue.json');draws=[];recorded=unknown=attempts=advances=full=null=0;notrun=False;vectors=set();peakwork=peakresults=peaknodes=peakedges=0
    for i in range(512):
        means=[]
        for arm in range(3):
            values=[];emission=parameters=None
            for p in range(4):
                q=plan[i*12+arm*4+p];row=a.load(out/'cells'/(q['id']+'.json'))
                n,advanced,score=cell(row,q,theta,catalogue,native,h)
                if row['status']=='NotRun':notrun=True
                else:
                    assert not notrun,'fixed sequence has gaps'
                    recorded+=1;unknown+=row['status']=='Incomplete';full+=row['status']=='Complete';null+=row['status']=='Null'
                    if emission is None:emission=row['emission'];parameters=row['parameters']
                    else:assert emission==row['emission'] and parameters==row['parameters'],'physics replicas changed genotype'
                    if row['evaluation'] is not None:
                        for step in row['evaluation']['steps']:
                            peakwork=max(peakwork,step['usage']['work']);peakresults=max(peakresults,step['usage']['results'])
                            peaknodes=max(peaknodes,step['nodes']);peakedges=max(peakedges,step['edges'])
                attempts+=n;advances+=advanced;values.append(score)
            means.append(sum(values)/4 if all(x is not None for x in values) else None)
            if arm==2 and parameters is not None:vectors.add(tuple(parameters))
        draws.append(means)
    assert (recorded,attempts,advances,unknown)==(m['recorded_cells'],m['attempted_steps'],m['advancing_steps'],m['unknown_partial_work_cells'])
    complete=m['status']=='complete'
    if complete:
        assert recorded==6144 and unknown==0 and len(vectors)==512 and m['live_sources_rechecked']
        assert 0<=m['grammar_compile_seconds']<=m['elapsed_seconds']<3600
    else:assert m['status'] in ('incomplete','blocked_before_evaluation') and isinstance(m['reason'],str)
    expected=statistics(draws);s=a.load(out/'summary.json');check_statistics(s,expected,complete)
    assert s['profile']==PROFILE and s['measurement_complete']==complete and s['attempted_steps']==attempts and s['advancing_steps']==advances
    assert sum((out/p).stat().st_size for p in a.files(out))<2*1024**3
    return dict(audit_pass=True,classification='fixed_champion_confirmation' if complete else 'incomplete_confirmation',
        confirmation_complete=complete,valid_evaluations=full,null_evaluations=null,recorded_cells=recorded,
        attempted_steps=attempts,advancing_steps=advances,frozen_sources=len(snap['sources']),
        improvement_supported=s['improvement_supported'],champion_minus_untrained=s['champion_minus_untrained'],
        champion_minus_random=s['champion_minus_random'],one_sided_paired_sign_p=s['one_sided_paired_sign_p'],
        peak_work=peakwork,peak_results=peakresults,peak_nodes=peaknodes,peak_edges=peakedges,
        independent_physics_replay=False)

def tamper_tests(out):
    # Every control mutates detached data, never a source/archive hard link.
    out=Path(out);m=a.load(out/'manifest.json');s=a.load(out/'summary.json');proof=a.load(out/'parent_receipt.json');parent=Path(proof['parent_archive'])
    native,h=helpers(parent);catalogue=a.load(out/'catalogue.json');theta=a.load(out/'champion.json')['parameters']
    q=schedule()[0];row=a.load(out/'cells'/(q['id']+'.json'))
    accepted=0
    def rejects(f):
        nonlocal accepted
        try:f()
        except (AssertionError,KeyError,ValueError,IndexError):accepted+=1;return
        raise AssertionError('tamper accepted')
    for key,value in [('profile','wrong'),('scheduled_cells',6143),('optimizer_generations',1),('attempted_steps',614401)]:
        bad=copy.deepcopy(m);bad[key]=value;rejects(lambda:check_manifest(bad))
    for key,value in [('horizon',1000),('work',400000),('draws',64),('master_seed',SEED+1)]:
        bad=copy.deepcopy(m);bad['design'][key]=value;rejects(lambda:check_manifest(bad))
    for case in ['seed','score','credit','cap','allowance','membership','parameters','event']:
        bad=copy.deepcopy(row)
        if case=='seed':bad['schedule']['physics_seed']+=1
        elif case=='score':bad['evaluation']['score']=.12345
        elif case=='credit':bad['evaluation']['proposal_fires']+=1
        elif case=='cap':bad['evaluation']['steps'][-1]['cap']=not bad['evaluation']['steps'][-1]['cap']
        elif case=='allowance':bad['evaluation']['steps'][0]['usage']['rules'][0]['work_allowance']+=1
        elif case=='membership':bad['emission']['production_index']=999999
        elif case=='parameters':bad['parameters'][0]+=1
        else:bad['evaluation']['steps'][0]['rule_ids']=[999999]
        rejects(lambda:cell(bad,q,theta,catalogue,native,h))
    expected=statistics(s['draw_means'])
    for key,value in [('fixed_denominator',64),('champion_minus_untrained',123.),('one_sided_paired_sign_p',.98765),('wins',999),('improvement_supported',not s['improvement_supported'])]:
        bad=copy.deepcopy(s);bad[key]=value;rejects(lambda:check_statistics(bad,expected,True))
    # Test the same digest/seal primitive on bounded temporary files.
    with tempfile.TemporaryDirectory(prefix='ere-confirm-integrity-') as tmp:
        root=Path(tmp);file=root/'input';file.write_bytes(b'frozen');sha=a.digest(file);a.seal(root)
        file.write_bytes(b'changed');rejects(lambda: a.verify(root))
        rejects(lambda: check_digest(file,sha))
    return accepted

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('out',type=Path);parser.add_argument('--tamper-tests',action='store_true');parser.add_argument('--seal',action='store_true')
    args=parser.parse_args();result=audit(args.out)
    if args.tamper_tests:result['tamper_rejections']=tamper_tests(args.out)
    if args.seal:
        (args.out/'audit.json').write_text(json.dumps(result,indent=2));a.seal(args.out);audit(args.out)
    print(json.dumps(result,indent=2))
