#!/usr/bin/env python3
"""Independent bounded receipt/accounting/proposal audit; no native replay."""
import argparse
import json
import math
from pathlib import Path
import time
import ere_categorical_feasible_development as d

a=d.a;cat=d.cat;ROOT=d.ROOT
legacy=d.mathcheck.history.s.original
BoundedRows=legacy.BoundedRows
CONFIG_SHA='0150569bc1dbbb9922c47faa53073203c7b4a1206be945e5660e80fd22f3454f'
IMMUTABLE={'snapshot.json','source_registration.json','build_receipt.json','executed_binary',
           'config.json','schedule.json','contexts.json','request.json','initial_manifest.json',
           'reference_catalogue.json','seed_inventory.json','launch_preflight.json'}


def numeric(got,expected):legacy.numeric(got,expected)


def support(catalogue,base,vectors):
    _,groups,_=cat.mapping.catalogue_model(catalogue)
    bp=cat.mapping.probabilities(catalogue,base,groups)['group_mass']
    posts=[cat.mapping.probabilities(catalogue,v,groups)['group_mass'] for v in vectors]
    rows=[];mass=[0.]*len(vectors);mins=[math.inf]*len(vectors);maxs=[0.]*len(vectors)
    for key,g in groups.items():
        s=dict(zip(('parent','operator','role','production_index'),(*key,0)))
        b=cat.score(catalogue,base,s);ratios=[]
        for i,v in enumerate(vectors):
            post=cat.score(catalogue,v,s);w=math.exp(post['log_probability']-b['log_probability'])
            assert math.exp(-6*.100001)-1e-12<=w<=math.exp(6*.100001)+1e-12
            cat.close(w,posts[i][key]/bp[key]);mass[i]+=b['probability']*g['count']*w
            mins[i]=min(mins[i],w);maxs[i]=max(maxs[i],w)
            ratios.append(dict(base_log_probability=b['log_probability'],proposal_log_probability=post['log_probability'],weight=w))
        rows.append(dict(selection=s,production_count=g['count'],ratios=ratios))
    assert len(rows)==3232 and all(abs(x-1)<1e-12 for x in mass)
    return dict(legal_leaves=18956,groups=rows,mass=mass,minimum=mins,maximum=maxs)


def estimate(rec,qs,rows,catalogue):
    first=qs[0];theta=d.contexts()[first['context']]
    assert set(rec)==set('context estimate theta_before evaluation_ids complete request proposal'.split())
    d.same(rec['context'],first['context']);d.same(rec['estimate'],first['estimate'])
    cat.bits(rec['theta_before'],theta);d.same(rec['evaluation_ids'],[q['id'] for q in qs])
    assert type(rec['complete']) is bool
    full=all(rows[q['id']]['status'] in ('Complete','Null') and rows[q['id']]['constraint_utility'] is not None for q in qs)
    if not rec['complete']:
        assert rec['request'] is None and rec['proposal'] is None
        return None
    assert full,'proposal constructed from unfinished/unknown panel'
    panels={}
    for stage,size,name in [('utility_estimation',16,'utility'),('risk_estimation',512,'risk')]:
        selected=[q for q in qs if q['stage']==stage];assert len(selected)==size
        samples=[]
        for q in selected:
            row=rows[q['id']];cat.bits(row['parameters'],theta)
            samples.append(dict(id=q['id'],emission_seed=q['emission_seed'],physics_seed=q['physics_seed'],
                selection=cat.selection(row['emission']),native_score=row['evaluation']['score'],reason=row['evaluation']['reason']))
        panels[name]=dict(stage=stage,expected_draws=size,samples=samples)
    expected=dict(theta=theta,**panels)
    # Exact request provenance, not trust in a stage tag or copied native sample.
    d.same(rec['request'],expected)
    d.mathcheck.check_case(expected,rec['proposal'],catalogue)
    return rec['proposal']['theta_after']


def preflight_reference(ref,snap,out):
    assert type(ref) is dict and set(ref)=={'path','root_seal'}
    pre=Path(ref['path']);assert pre.is_absolute() and pre.resolve()!=Path(out).resolve()
    assert pre.parent.resolve()==(ROOT/'output').resolve() and not pre.is_symlink()
    a.verify(pre,expected=ref['root_seal'])
    req=a.load(pre/'request.json');assert req['launch_authorized'] is False
    prior=a.load(pre/'snapshot.json')
    d.same(prior['sources'],snap['sources']);assert prior['binary_sha256']==snap['binary_sha256']
    d.same(a.load(pre/'manifest.json'),d.initial());d.same(a.load(pre/'initial_manifest.json'),d.initial())
    assert a.load(pre/'audit.json')['audit_pass'] is True
    assert a.load(pre/'audit.json')['classification']=='zero_work_preflight'
    controls=a.load(pre/'guard_verification.json')
    assert controls['engineering_pass'] is True and controls['zero_native_work'] is True
    assert controls['check_only_pass'] is True and len(controls['refusals'])==6
    assert controls['regression_groups']==6
    assert len(controls['auditor_rejections'])==4 and all(q['rejected'] is True for q in controls['auditor_rejections'])
    d.same(a.load(pre/'schedule.json'),d.schedule())
    assert a.digest(pre/'config.json')==CONFIG_SHA
    for q in d.schedule():
        d.same(a.load(pre/'cells'/(q['id']+'.json')),dict(schedule=q,status='NotRun',reason='not_started',evaluation=None))


def audit(out,verify_seal=True):
    start=time.monotonic();out=Path(out)
    if verify_seal:a.verify(out)
    c=a.load(out/'config.json');d.same(c,d.design());assert a.digest(out/'config.json')==CONFIG_SHA
    req=a.load(out/'request.json')
    assert set(req)==set('profile mode launch_authorized full_training_authorized setup_elapsed_seconds'.split())
    assert req['profile']==d.PROFILE and req['mode']=='development' and type(req['launch_authorized']) is bool and req['full_training_authorized'] is False
    assert type(req['setup_elapsed_seconds']) is float and 0<=req['setup_elapsed_seconds']<900
    plan=d.schedule();d.check_schedule(a.load(out/'schedule.json'));d.same(a.load(out/'contexts.json'),d.contexts())
    snap=a.load(out/'snapshot.json');prepared=a.load(out/'prepared_sha256.json')
    assert snap['frozen_before_measurement'] is True and snap['build_features']==['ere-complete-signal'] and snap['default_features'] is False
    assert Path(snap['repository']).resolve()==ROOT.resolve()
    d.registration(out/'source_snapshots',verify_lineage=True)
    registration=d.registration(out/'source_snapshots');d.same(a.load(out/'source_registration.json'),registration)
    assert set(snap['sources'])==set(registration['source_names']) and a.files(out/'source_snapshots')==set(snap['sources'])
    assert set(prepared)==IMMUTABLE|{'source_snapshots/'+n for n in snap['sources']}
    for n,h in prepared.items():assert a.safe(n) and a.digest(out/n)==h,'immutable prepared input drift'
    for n,h in snap['sources'].items():assert a.digest(out/'source_snapshots'/n)==h
    assert a.digest(out/'executed_binary')==snap['binary_sha256']
    build=a.load(out/'build_receipt.json');assert build['command']==d.BUILD and build['returncode']==0
    assert build['source_hashes_before']==build['source_hashes_after']==snap['sources'] and build['binary_sha256']==snap['binary_sha256']
    seeds=a.load(out/'seed_inventory.json');d.same(seeds,build['seed_collision_check'])
    # History can grow after measurement: validate recorded files, not a moving
    # equality gate. Launcher/pre-launch independently rechecks current inventory.
    oldseeds=set()
    for pin in seeds['prior_schedule_files']:
        path=ROOT/'output'/pin['directory'];assert a.digest(path/'schedule.json')==pin['schedule_sha256']
        for n,h in pin['metadata_sha256'].items():assert a.digest(path/n)==h
        for q in a.load(path/'schedule.json'):
            if isinstance(q,dict):oldseeds.update(q[k] for k in ('emission_seed','physics_seed','noise_seed','random_parameter_seed') if q.get(k) is not None)
    assert len(oldseeds)==seeds['explicit_seed_identities'] and not oldseeds&d.check_schedule(plan)
    assert seeds['collision'] is False and seeds['implicit_rng_collision_claim'] is False
    ref=a.load(out/'launch_preflight.json')
    if req['launch_authorized']:preflight_reference(ref,snap,out)
    else:assert ref is None
    m=a.load(out/'manifest.json');d.same(a.load(out/'initial_manifest.json'),d.initial())
    assert m['profile']==d.PROFILE and m['mode']=='development' and m['scheduled_cells']==2176
    for k in ('recorded_cells','attempted_steps','advancing_steps','unknown_partial_work_cells','grammar_compilations','conditional_leaf_checks','direction_starts','direction_estimates','receipt_bytes','optimizer_generations','training_updates'):
        assert type(m[k]) is int and m[k]>=0
    assert m['optimizer_generations']==m['training_updates']==0 and m['full_training_authorized'] is False and m['training_admitted'] is False
    assert m['inflight_native_work_unknown'] is False,'inflight unknown work cannot be finalized'
    assert 0<=m['direction_estimates']<=m['direction_starts']<=4 and m['recorded_cells']<=2176
    assert a.files(out/'cells')=={q['id']+'.json' for q in plan}
    catalogue=a.load(out/'reference_catalogue.json');assert a.digest(out/'reference_catalogue.json')==a.CATALOGUE
    if (out/'catalogue.json').exists():assert a.digest(out/'catalogue.json')==a.CATALOGUE and m['grammar_compilations']==1
    assert m['grammar_compilations'] in (0,1)
    a.verify(legacy.PARENT,expected=legacy.PARENT_SEAL,count=1690)
    a.verify(legacy.RUNTIME,expected=legacy.RUNTIME_SEAL,count=460)
    native,h=cat.d.c.helpers(legacy.PARENT);rows=BoundedRows(out/'cells')
    attempts=advances=unknown=valid=null=bytes_count=0;notrun=False;stopped=False
    for q in plan:
        row=rows.read_unregistered(q['id']);d.same(row['schedule'],q)
        n,adv,score=cat.d.b.cell(row,q,catalogue,native,h);attempts+=n;advances+=adv
        if row['status']=='NotRun':
            notrun=True;d.same(row,dict(schedule=q,status='NotRun',reason='not_started',evaluation=None))
        else:
            assert not notrun and not stopped,'sequence gap or work after unknown outcome'
            assert m['grammar_compilations']==1 and (out/'catalogue.json').exists()
            cat.bits(row['parameters'],d.contexts()[q['context']])
            unknown+=row['status']=='Incomplete';valid+=row['status']=='Complete';null+=row['status']=='Null'
            stopped=row.get('constraint_utility') is None
            bytes_count+=(out/'cells'/(q['id']+'.json')).stat().st_size
        rows[q['id']]=row
        assert time.monotonic()-start<900,'audit wall ceiling'
    assert (m['recorded_cells'],m['attempted_steps'],m['advancing_steps'],m['unknown_partial_work_cells'],m['receipt_bytes'])==(valid+null+unknown,attempts,advances,unknown,bytes_count)
    assert 0<=advances<=attempts<=217600
    checks=sum(q['stage']!='validation' or q['physics_index']==0 for q in plan[:m['recorded_cells']])
    assert checks<=m['conditional_leaf_checks']<=checks+1
    efiles=set();dfiles=set();vfiles=set();directions={};started=completed=0;closed=False
    for ctx in range(2):
        forwards=[]
        for est in range(2):
            qs=[q for q in plan[:2112] if (q['context'],q['estimate'])==(ctx,est)]
            name=f'c{ctx}-e{est}.json';path=out/'estimates'/name
            if path.exists():
                assert not closed;efiles.add(name);started+=1
                step=estimate(a.load(path),qs,rows,catalogue)
                if step is None:closed=True
                else:completed+=1;forwards.append(step)
            else:
                assert all(rows[q['id']]['status']=='NotRun' for q in qs);closed=True
        name=f'c{ctx}.json';path=out/'directions'/name
        if path.exists():
            assert len(forwards)==2;dfiles.add(name);rec=a.load(path)
            assert set(rec)=={'context','vectors','alignment','full_support'} and rec['context']==ctx
            vectors=[d.contexts()[ctx]]+forwards;assert len(rec['vectors'])==3
            for x,y in zip(rec['vectors'],vectors):cat.bits(x,y)
            numeric(rec['alignment'],cat.d.alignment(vectors[0],vectors[1],vectors[2]))
            numeric(rec['full_support'],support(catalogue,vectors[0],vectors));directions[ctx]=vectors
    assert a.files(out/'estimates')==efiles and a.files(out/'directions')==dfiles
    assert (started,completed)==(m['direction_starts'],m['direction_estimates'])
    points=[]
    for ctx in range(2):
        qs=[q for q in plan[2112:] if q['context']==ctx]
        if any(rows[q['id']]['status']!='NotRun' for q in qs):assert completed==4 and len(directions)==2,'validation before all proposals'
        name=f'c{ctx}.json';path=out/'validation'/name
        if path.exists():
            vfiles.add(name)
            u,v=legacy.validation(a.load(path),0,ctx,qs,rows,catalogue,directions[ctx],8)
            points.append(dict(context=ctx,utility_effects=u,validity_effects=v))
    assert a.files(out/'validation')==vfiles
    complete=m['status']=='complete'
    if complete:
        assert req['launch_authorized'] and m['recorded_cells']==2176 and unknown==0 and not stopped
        assert completed==4 and len(vfiles)==2 and m['conditional_leaf_checks']==2128
        assert m['live_sources_rechecked'] is True and 0<=m['grammar_compile_seconds']<=m['elapsed_seconds']<900
        assert m['reason'] is None
    else:assert m['status'] in ('zero_work_preflight','incomplete','blocked_before_evaluation')
    if m['status']=='zero_work_preflight':d.same(m,d.initial())
    if not req['launch_authorized']:d.same(m,d.initial());assert attempts==started==completed==m['grammar_compilations']==0
    numeric(a.load(out/'summary.json'),d.summary(complete,points))
    assert sum((out/n).stat().st_size for n in a.files(out))<c['output_cap_bytes']
    assert time.monotonic()-start<900,'audit wall ceiling'
    return dict(audit_pass=True,profile=d.PROFILE,source_registration_verified=True,
        classification='zero_work_preflight' if m['status']=='zero_work_preflight' else 'bounded_development',
        campaign_complete=complete,scientific_support=False,recorded_cells=m['recorded_cells'],
        valid_evaluations=valid,null_evaluations=null,unknown_partial_work_cells=unknown,
        attempted_steps=attempts,advancing_steps=advances,direction_estimates=completed,
        grammar_compilations=m['grammar_compilations'],conditional_leaf_checks=m['conditional_leaf_checks'],
        training_updates=0,training_admitted=False,frozen_sources=len(snap['sources']),
        independent_physics_replay=False,independent_rng_replay=False,unseen_cap_feasibility_certificate=False)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('archive',type=Path)
    args=parser.parse_args();print(json.dumps(audit(args.archive),indent=2))
