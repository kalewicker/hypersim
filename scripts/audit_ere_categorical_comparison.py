#!/usr/bin/env python3
"""Independent receipt/math accounting, not physical/RNG/compiler replay."""
import argparse
import copy
import json
import math
from pathlib import Path
import tempfile
import shutil
import time
import tomllib
import check_ere_categorical_comparison_plan as p
import audit_ere_categorical_adapter as cat

a=p.a
ROOT=Path(__file__).resolve().parents[1]
if ROOT.name=='source_snapshots':
    ROOT=Path(a.load(ROOT.parent/'snapshot.json')['repository'])
PREREG='output/ere_categorical_comparison_preregistration_20261001'
PREREG_SEAL='dcf4ec69f2ea647f1f7eeda9185ae526788a553639fd490395da7f98933b5d57'
CONFIG_SHA='b26d94f1d889018ff4baa79f81f68a82c1d8c838a0375e080e3de99bb81d4328'
EXTRA={'crates/hg-train/src/ere/categorical_comparison.rs','crates/hg-train/src/bin/ere-categorical-comparison.rs',
       'scripts/run_ere_categorical_comparison.py','scripts/audit_ere_categorical_comparison.py',
       'scripts/test_audit_ere_categorical_comparison.py','docs/ere_categorical_comparison_harness_v1.md'}
MODULE='crates/hg-train/src/ere.rs';MANIFEST='crates/hg-train/Cargo.toml'
PARENT=ROOT/'output/ere_complete_short_training_continuation_20260930'
PARENT_SEAL='dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132'
RUNTIME=ROOT/'output/ere_runtime50k_signal_panel_20260930'
RUNTIME_SEAL='0099ecae4b7487e9ac975f73d813e7cbde0de430470dc66198bbafd7df16488c'

def config():
    c=a.load(ROOT/'configs/ere_categorical_comparison_v1.json');p.validate(c)
    assert a.digest(ROOT/'configs/ere_categorical_comparison_v1.json')==CONFIG_SHA
    return c

def registration(base, c, check_historical=False):
    """Exact new learning registration, all other in-scope sources unchanged."""
    if check_historical:
        a.verify(ROOT/PREREG,expected=PREREG_SEAL,count=9)
        for pin in c['lineage'].values():a.verify(ROOT/pin['path'],expected=pin['root_seal'],count=pin['sealed_entries'])
    adapter=ROOT/c['lineage']['adapter']['path'];old=a.load(adapter/'manifest.json')['sources']
    excluded=set(c['excluded_feature_sources']);names=set(old)-excluded|set(p.SOURCES)|EXTRA
    assert len(old)==467 and len(names)==472
    for n,h in old.items():
        if n not in excluded|{MODULE,MANIFEST}:assert a.digest(base/n)==h,f'ERE/shared source drift: {n}'
    before=(adapter/'source_snapshots'/MODULE).read_text();anchor='#[cfg(feature = "ere-complete-signal")]\npub mod categorical;\n'
    assert before.count(anchor)==1 and (base/MODULE).read_text()==before.replace(anchor,anchor+'#[cfg(feature = "ere-complete-signal")]\npub mod categorical_comparison;\n',1)
    before=tomllib.loads((adapter/'source_snapshots'/MANIFEST).read_text());after=tomllib.loads((base/MANIFEST).read_text())
    assert after['bin'].pop()==dict(name='ere-categorical-comparison',path='src/bin/ere-categorical-comparison.rs',**{'required-features':['ere-complete-signal']})
    assert before==after,'Cargo changed beyond exact comparison binary registration'
    for n,h in a.load(ROOT/PREREG/'manifest.json')['sources'].items():assert a.digest(base/n)==h,'frozen plan modified'
    assert all((base/n).is_file() for n in names)
    train={str(n.relative_to(base)) for n in (base/'crates/hg-train').rglob('*') if n.is_file() and n.suffix in ('.rs','.toml')}
    assert train=={n for n in names if n.startswith('crates/hg-train/')},'unregistered learning source'
    return dict(source_names=sorted(names),historical_adapter_entries=467,excluded_feature='evolving-minds',
        excluded_paths=sorted(excluded),unchanged_in_scope_adapter_paths=460,
        registration={n:dict(before_sha256=old[n],after_sha256=a.digest(base/n)) for n in (MODULE,MANIFEST)})

def exact(got,expected):p.same(got,expected)
def numeric(got,expected,tol=2e-12):
    if isinstance(expected,dict):
        assert isinstance(got,dict) and set(got)==set(expected)
        for k,v in expected.items():numeric(got[k],v,tol)
    elif isinstance(expected,list):
        assert isinstance(got,list) and len(got)==len(expected)
        for x,y in zip(got,expected):numeric(x,y,tol)
    elif isinstance(expected,float):cat.close(got,expected,tol)
    else:exact(got,expected)

def matched(got,theta,raw):
    expected=p.matched_step(theta,raw);numeric(got,expected)
    cat.bits(got['forward'],expected['forward']);cat.bits(got['reverse'],expected['reverse'])
    return expected

def support(catlog,base,vectors):
    _,groups,_=cat.mapping.catalogue_model(catlog)
    bp=cat.mapping.probabilities(catlog,base,groups)['group_mass'];posts=[cat.mapping.probabilities(catlog,v,groups)['group_mass'] for v in vectors]
    rows=[];mass=[0.]*9;mins=[math.inf]*9;maxs=[0.]*9
    for key,g in groups.items():
        s=dict(zip(('parent','operator','role','production_index'),(*key,0)));ratios=[]
        b=cat.score(catlog,base,s)
        for i,v in enumerate(vectors):
            post=cat.score(catlog,v,s);w=math.exp(post['log_probability']-b['log_probability'])
            assert math.exp(-6*.100001)-1e-12<=w<=math.exp(6*.100001)+1e-12
            # Independent factorized probabilities versus saved log-probability ratio.
            cat.close(w,posts[i][key]/bp[key])
            mass[i]+=b['probability']*g['count']*w;mins[i]=min(mins[i],w);maxs[i]=max(maxs[i],w)
            ratios.append(dict(base_log_probability=b['log_probability'],proposal_log_probability=post['log_probability'],weight=w))
        rows.append(dict(selection=s,production_count=g['count'],ratios=ratios))
    assert len(rows)==3232 and all(abs(x-1)<1e-12 for x in mass)
    return dict(legal_leaves=18956,groups=rows,mass=mass,minimum=mins,maximum=maxs)

def estimate(rec, qs, theta, rows, catalogue, native, h, k):
    first=qs[0];method=first['method'];size=16*k
    assert rec['block']==first['block'] and rec['context']==first['context'] and rec['estimate']==first['estimate'] and rec['method']==method
    cat.bits(rec['theta_before'],theta);assert rec['evaluation_ids']==[q['id'] for q in qs] and type(rec['complete']) is bool
    assert len(qs)==size;full=all(rows[q['id']]['status'] in ('Complete','Null') for q in qs)
    if method=='es':
        assert type(rec['noise_seed']) is int and rec['noise_seed']==first['noise_seed'] and len(rec['eps'])==8
        for noise in rec['eps']:native.vector(noise)
        scores=[]
        for i,noise in enumerate(rec['eps']):
            for sign in (1,-1):
                candidate=[p.f32(t+p.f32(sign*x)) for t,x in zip(theta,noise)];batch=[]
                for q in qs[(2*i+(sign==-1))*k:(2*i+(sign==-1)+1)*k]:
                    row=rows[q['id']]
                    if row['status']!='NotRun':cat.bits(row['parameters'],candidate)
                    if row['status'] in ('Complete','Null'):batch.append(row['evaluation']['score'])
                scores.append(batch)
        if not rec['complete']:
            assert rec['raw'] is None and rec['matched'] is None;return None
        assert full
        means,fits=cat.d.b.batch_fits(scores,k,h)
        raw=native.update(theta,rec['eps'],fits,8,h)
        numeric(rec['raw'],dict(candidate_means=means,optimizer_pair_scores=[fits[i:i+2] for i in range(0,16,2)],theta_after=raw),tol=2e-7)
        cat.bits(rec['raw']['theta_after'],raw)
    else:
        assert rec['eps']==[] and rec['noise_seed'] is None
        samples=[]
        for q in qs:
            row=rows[q['id']]
            if row['status']!='NotRun':cat.bits(row['parameters'],theta)
            if row['status'] in ('Complete','Null'):
                samples.append(dict(id=q['id'],emission_seed=q['emission_seed'],physics_seed=q['physics_seed'],selection=cat.selection(row['emission']),
                    native_score=row['evaluation']['score'],reason=row['evaluation']['reason']))
        if not rec['complete']:
            assert rec['raw'] is None and rec['matched'] is None;return None
        assert full;batch=dict(theta=theta,samples=samples,expected_draws=size,alpha=p.f32(.05));cat.check_batch(rec['raw'],catalogue,batch)
        raw=rec['raw']['theta_after']
    return matched(rec['matched'],theta,raw)

def validation(rec,r,ctx,qs,rows,catalogue,vectors,draws):
    assert rec['block']==r and rec['context']==ctx and len(rec['terms'])==draws
    ue=[[] for _ in vectors];ve=[[] for _ in vectors];base=[];native=[]
    for d in range(draws):
        panel=qs[4*d:4*d+4];rr=[rows[q['id']] for q in panel];assert all(x['status'] in ('Complete','Null') for x in rr)
        assert all(x['emission']==rr[0]['emission'] for x in rr),'physics repeats changed emission'
        u=p.ordered_sum(x['constraint_utility'] for x in rr)/4;v=sum(x['status']=='Complete' for x in rr)/4
        s=cat.selection(rr[0]['emission']);b=cat.score(catalogue,vectors[0],s);ws=[]
        for i,t in enumerate(vectors):
            post=cat.score(catalogue,t,s);w=math.exp(post['log_probability']-b['log_probability'])
            effect=p.importance_effect([w],[u],[v]);ue[i].append(effect['utility_effect']);ve[i].append(effect['validity_effect'])
            ws.append(dict(base_log_probability=b['log_probability'],proposal_log_probability=post['log_probability'],weight=w))
        numeric(rec['terms'][d],dict(draw=d,selection=s,evaluation_ids=[q['id'] for q in panel],utility=u,validity=v,weights=ws))
        base.append(u);native.extend(x['evaluation']['score'] for x in rr)
    u=[p.ordered_sum(x)/draws for x in ue];v=[p.ordered_sum(x)/draws for x in ve]
    numeric(rec['utility_effects'],u);numeric(rec['validity_effects'],v)
    numeric(rec['observed_base_mean_utility'],p.ordered_sum(base)/draws)
    numeric(rec['base_native_mean'],p.ordered_sum(native)/(draws*4) if all(x is not None for x in native) else None)
    return u,v

def audit(out, verify_seal=True):
    start=time.monotonic();out=Path(out)
    if verify_seal:a.verify(out)
    c=a.load(out/'config.json');p.validate(c);assert a.digest(out/'config.json')==CONFIG_SHA
    a.verify(ROOT/PREREG,expected=PREREG_SEAL,count=9)
    for pin in c['lineage'].values():a.verify(ROOT/pin['path'],expected=pin['root_seal'],count=pin['sealed_entries'])
    a.verify(PARENT,expected=PARENT_SEAL,count=1690);a.verify(RUNTIME,expected=RUNTIME_SEAL,count=460)
    req=a.load(out/'request.json');assert set(req)=={'profile','mode','launch_authorized','full_training_authorized','setup_elapsed_seconds'}
    assert req['profile']==p.PROFILE and req['mode'] in ('development','scientific') and type(req['launch_authorized']) is bool and req['full_training_authorized'] is False
    mode=req['mode'];l=c[mode];plan=p.schedule(c,mode);exact(a.load(out/'schedule.json'),plan);exact(a.load(out/'contexts.json'),p.contexts())
    prepared=a.load(out/'prepared_sha256.json');immutable=set(a.load(out/'snapshot.json')['sources'])
    expected={f'source_snapshots/{n}' for n in immutable}|{'snapshot.json','source_registration.json','build_receipt.json','executed_binary','config.json','schedule.json','contexts.json','request.json','initial_manifest.json','reference_catalogue.json','development_gate.json'}
    assert set(prepared)==expected
    for n,h in prepared.items():assert a.safe(n) and a.digest(out/n)==h,'prepared immutable input changed'
    snap=a.load(out/'snapshot.json');assert snap['frozen_before_measurement'] is True and snap['build_features']==['ere-complete-signal'] and snap['default_features'] is False
    assert Path(snap['repository']).resolve()==ROOT.resolve()
    receipt=registration(out/'source_snapshots',c);exact(a.load(out/'source_registration.json'),receipt)
    assert set(snap['sources'])==set(receipt['source_names']) and a.files(out/'source_snapshots')==set(snap['sources'])
    for n,h in snap['sources'].items():assert a.digest(out/'source_snapshots'/n)==h
    assert a.digest(out/'executed_binary')==snap['binary_sha256']
    gate=a.load(out/'development_gate.json')
    if mode=='scientific' and req['launch_authorized']:
        assert isinstance(gate,dict) and set(gate)=={'path','root_seal'}
        dev=Path(gate['path']);assert dev.resolve()!=out.resolve();a.verify(dev,expected=gate['root_seal'])
        prior=audit(dev)
        assert prior['classification']=='bounded_development' and prior['campaign_complete'] and prior['unknown_partial_work_cells']==0
        assert a.load(dev/'snapshot.json')['sources']==snap['sources'] and len(a.load(dev/'audit.json')['detached_tamper_rejections'])==6
    else:assert gate is None
    build=a.load(out/'build_receipt.json');assert build['command']==['cargo','build','--offline','--release','-p','hg-train','--no-default-features','--features','ere-complete-signal','--bin','ere-categorical-comparison']
    assert build['source_hashes_before']==build['source_hashes_after']==snap['sources'] and build['binary_sha256']==snap['binary_sha256'] and build['returncode']==0
    m=a.load(out/'manifest.json');initial=a.load(out/'initial_manifest.json')
    assert m['profile']==initial['profile']==p.PROFILE and m['mode']==initial['mode']==mode
    assert m['scheduled_cells']==len(plan) and m['optimizer_generations']==m['training_updates']==0 and m['training_admitted'] is False and m['full_training_authorized'] is False
    assert m['inflight_native_work_unknown'] is False,'in-flight unknown work blocks final accounting'
    assert type(m['recorded_cells']) is int and 0<=m['recorded_cells']<=len(plan)
    assert all(type(m[k]) is int and m[k]>=0 for k in ('attempted_steps','advancing_steps','unknown_partial_work_cells','grammar_compilations','direction_starts','direction_estimates','receipt_bytes','conditional_leaf_checks','optimizer_generations','training_updates'))
    assert a.files(out/'cells')=={q['id']+'.json' for q in plan}
    catalogue=a.load(out/'reference_catalogue.json');assert a.digest(out/'reference_catalogue.json')==a.CATALOGUE
    if (out/'catalogue.json').exists():assert a.digest(out/'catalogue.json')==a.CATALOGUE and m['grammar_compilations']==1
    native,h=cat.d.c.helpers(PARENT);rows={};attempts=advances=unknown=valid=null=0;prefix=False
    for q in plan:
        row=a.load(out/'cells'/(q['id']+'.json'));cat.d.b.exact_schedule([row['schedule']],[q])
        ns,adv,score=cat.d.b.cell(row,q,catalogue,native,h);attempts+=ns;advances+=adv
        if row['status']=='NotRun':prefix=True
        else:
            assert not prefix,'fixed native sequence gap';unknown+=row['status']=='Incomplete';valid+=row['status']=='Complete';null+=row['status']=='Null'
        if q['stage']=='validation' and row['status']!='NotRun':cat.bits(row['parameters'],p.contexts()[q['context']])
        rows[q['id']]=row
        assert time.monotonic()-start<900,'auditor wall ceiling'
    assert (m['recorded_cells'],m['attempted_steps'],m['advancing_steps'],m['unknown_partial_work_cells'])==(valid+null+unknown,attempts,advances,unknown)
    assert 0<=advances<=attempts<=l['maximum_native_attempts']
    checked_emissions=sum(q['stage']=='estimation' or q.get('physics_index')==0 for q in plan[:m['recorded_cells']])
    assert type(m['conditional_leaf_checks']) is int and checked_emissions<=m['conditional_leaf_checks']<=checked_emissions+1
    n=l['blocks'];align=[[[None]*2 for _ in range(2)] for _ in range(n)];effects=[[[[None]*2 for _ in range(2)] for _ in range(2)] for _ in range(n)];vs=copy.deepcopy(effects)
    estimation=[q for q in plan if q['stage']=='estimation'];efiles=set();dfiles=set();vfiles=set();directions={};starts=updates=0;closed=False;size=l['categorical_emissions_per_estimate'];offset=0
    for r in range(n):
        for ctx in range(2):
            forwards=[];reverses=[]
            for method in ('es','categorical'):
                for e in range(2):
                    qs=estimation[offset:offset+size];offset+=size;name=f'r{r}-c{ctx}-{method}-e{e}.json';path=out/'estimates'/name
                    if path.exists():
                        assert not closed,'estimate follows unfinished estimate';efiles.add(name);starts+=1
                        rec=a.load(path);step=estimate(rec,qs,p.contexts()[ctx],rows,catalogue,native,h,l['es_emissions_per_candidate'])
                        if step is None:closed=True
                        else:updates+=1;forwards.append(step['forward']);reverses.append(step['reverse'])
                    else:assert all(rows[q['id']]['status']=='NotRun' for q in qs);closed=True
            name=f'r{r}-c{ctx}.json';path=out/'directions'/name
            if path.exists():
                assert len(forwards)==4;dfiles.add(name);d=a.load(path);vectors=[p.contexts()[ctx]]+forwards+reverses
                for x,y in zip(d['vectors'],vectors):cat.bits(x,y)
                assert len(d['vectors'])==9 and d['block']==r and d['context']==ctx
                al=[cat.d.alignment(vectors[0],vectors[a],vectors[b]) for a,b in ((1,2),(3,4))]
                numeric(d['alignment'],al);numeric(d['full_support'],support(catalogue,vectors[0],vectors))
                align[r][ctx]=[x['score'] for x in al];directions[(r,ctx)]=vectors
    assert a.files(out/'estimates')==efiles and a.files(out/'directions')==dfiles and (starts,updates)==(m['direction_starts'],m['direction_estimates'])
    for r in range(n):
        for ctx in range(2):
            qs=[q for q in plan if q['stage']=='validation' and (q['block'],q['context'])==(r,ctx)]
            if any(rows[q['id']]['status']!='NotRun' for q in qs):assert updates==l['direction_estimates'] and len(directions)==n*2,'validation before all proposals'
            name=f'r{r}-c{ctx}.json';path=out/'validation'/name
            if path.exists():
                vfiles.add(name);u,v=validation(a.load(path),r,ctx,qs,rows,catalogue,directions[(r,ctx)],l['validation_emissions_per_block_context'])
                for method in range(2):
                    for e in range(2):effects[r][ctx][method][e]=u[1+2*method+e];vs[r][ctx][method][e]=v[1+2*method+e]
    assert a.files(out/'validation')==vfiles
    complete=m['status']=='complete'
    if complete:
        assert req['launch_authorized'] and m['recorded_cells']==len(plan) and unknown==0 and updates==l['direction_estimates'] and len(vfiles)==n*2
        assert m['live_sources_rechecked'] is True and 0<=m['elapsed_seconds']<l['wall_seconds']
        assert m['conditional_leaf_checks']==l['estimation_evaluations']+l['validation_evaluations']//4
    else:assert m['status'] in ('zero_work_preflight','incomplete','blocked_before_evaluation')
    if not req['launch_authorized']:
        assert m['status']=='zero_work_preflight' and m==initial and attempts==updates==starts==m['grammar_compilations']==0
    numeric(a.load(out/'summary.json'),p.statistics(c,mode,align,effects,vs,complete))
    assert sum((out/n).stat().st_size for n in a.files(out))<c['resources']['output_cap_bytes']
    assert time.monotonic()-start<900,'auditor wall ceiling'
    return dict(audit_pass=True,classification='zero_work_preflight' if not req['launch_authorized'] else 'bounded_development' if mode=='development' else 'fixed_parameter_categorical_comparison',
        campaign_complete=complete,scientific_campaign_complete=complete and mode=='scientific',support_categorical_direction=a.load(out/'summary.json')['support_categorical_direction'],
        recorded_cells=m['recorded_cells'],valid_evaluations=valid,null_evaluations=null,unknown_partial_work_cells=unknown,
        attempted_steps=attempts,advancing_steps=advances,direction_estimates=updates,grammar_compilations=m['grammar_compilations'],
        training_updates=0,training_admitted=False,independent_physics_replay=False,independent_rng_replay=False,frozen_sources=len(snap['sources']))

def tamper_tests(out):
    """Detached controls, never editing the measured archive or resealing it."""
    cases=[('summary.json',lambda x:x.update(support_categorical_direction=True)),
           ('manifest.json',lambda x:x.update(training_admitted=True)),
           ('manifest.json',lambda x:x.update(attempted_steps=x['attempted_steps']+1)),
           ('schedule.json',lambda x:x[0].update(physics_seed=x[0]['physics_seed']+1)),
           ('request.json',lambda x:x.update(full_training_authorized=True)),
           ('snapshot.json',lambda x:x.update(default_features=True))]
    with tempfile.TemporaryDirectory() as directory:
        clone=Path(directory)/'control';shutil.copytree(out,clone);passed=[]
        for name,mutate in cases:
            original=(clone/name).read_bytes();x=a.load(clone/name);mutate(x);(clone/name).write_text(json.dumps(x))
            try:audit(clone,verify_seal=False)
            except (AssertionError,KeyError,ValueError,TypeError,IndexError):passed.append(name)
            else:raise AssertionError(f'tamper accepted: {name}')
            finally:(clone/name).write_bytes(original)
    return passed

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('archive',type=Path);parser.add_argument('--tamper-tests',action='store_true');args=parser.parse_args()
    result=audit(args.archive)
    if args.tamper_tests:result['detached_tamper_rejections']=tamper_tests(args.archive)
    print(json.dumps(result,indent=2))
