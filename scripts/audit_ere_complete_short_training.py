#!/usr/bin/env python3
"""Frozen short-training accounting audit. Does not replay native physics/RNG."""
import argparse
import copy
import json
import math
import hashlib
from pathlib import Path
import shutil
import sys
import tempfile
import audit_ere_complete_admission as a
import audit_ere_complete_scaling as scaling

sys.dont_write_bytecode = True
PROFILE = 'E2-complete-short-training-v1'
CONTINUATION = 'E2-complete-short-training-continuation-v2'
PREFIX_SEAL = '813245dfa5ca432714a27e41a911fac75aef2951848867cc811a178b32d5ea5c'
SEED = 0xE654000001
PRE = [1601,1699,1801,1901]
TRAIN = [2003,2111,2203,2309]
HOLD = [2411,2503,2609,2707]
SIGNAL = 'crates/hg-sim-core/src/ere/signal.rs'
EXTRA = {'docs/ere_complete_short_training_v1.md', 'configs/ere_complete_short_training_v1.json',
         'docs/ere_complete_short_training_continuation_v2.md', 'configs/ere_complete_short_training_continuation_v2.json',
         'configs/ere_complete_receipts_v1.json', 'scripts/audit_ere_complete_short_training.py',
         'scripts/test_audit_ere_complete_short_training.py', 'scripts/audit_ere_complete_admission.py',
         'scripts/audit_ere_complete_scaling.py'}

def design():
    return dict(profile=PROFILE,master_seed=SEED,horizon=100,parameters=183,initial_parameter_value=0.,
        precheck_generations=5,precheck_pairs=4,generations=20,candidates=16,held_out_draws=64,
        precheck_physics=PRE,training_physics=TRAIN,held_out_physics=HOLD,sigma=1.,alpha=.05,
        minimum_valid_training_pairs=4,work=50000,results=50000,wall_seconds=3600,
        cell_wall_seconds=30,output_cap_bytes=512*1024**2)

def schedule():
    rows=[]
    def add(id,stage,seed,physics):
        rows.append(dict(id=id,stage=stage,emission_seed=seed,physics_seed=physics,horizon=100))
    for i in range(8):
        for s in PRE: add(f'baseline-{i}-{s}','precheck',SEED+2000+i,s)
    for active in ['true','false']:
        for s in PRE: add(f'control-{active}-{s}','precheck',SEED+3000,s)
    for g in range(5):
        for p in range(4):
            for sign in [1,-1]:
                for s in PRE[:2]: add(f'pre-es-{g}-{p}-{sign}-{s}','precheck',SEED+4000+4*g+p,s)
    for g in range(20):
        for p in range(8):
            for sign in [1,-1]:
                s=TRAIN[(g+p)%4]
                add(f'train-es-{g}-{p}-{sign}-{s}','training',SEED+5000+8*g+p,s)
    for i in range(64):
        for arm in ['champion','untrained','random']:
            for s in HOLD: add(f'hold-{i}-{arm}-{s}','held_out',SEED+6000+i,s)
    return rows

def repaired_source(old):
    source=scaling.bridge(old)
    before='        let before = s.step();\n        let stats = engine.step(&mut s);'
    assert source.count(before)==1
    return source.replace(before,'        let before = s.step();\n        capture.lock().unwrap().clear();\n        let stats = engine.step(&mut s);')

def wire_design(profile):
    # serde_json::Value widens the Rust f32 before serializing its number.
    return {**design(), 'profile':profile, 'alpha':0.05000000074505806}

def review_prefix(prefix,verify_prerequisites=True):
    prefix=Path(prefix)
    a.verify(prefix,expected=PREFIX_SEAL,count=1662)
    original=prefix/'source_snapshots/scripts/audit_ere_complete_short_training.py'
    text=original.read_text()
    before="m['design']==design()"
    after="m['design']=={**design(), 'alpha':0.05000000074505806}"
    assert text.count(before)==1
    corrected=text.replace(before,after)
    # Execute only the exact pinned, archived checker with this one numeric fix.
    scope={'__name__':'archived_float_only_checker','__file__':str(original)}
    exec(compile(corrected,str(original), 'exec'),scope)
    result=scope['audit'](prefix,verify_prerequisites=verify_prerequisites)
    assert result['audit_pass'] and result['precheck_pass'] and result['classification']=='fresh_admission_audit_failed'
    m=a.load(prefix/'manifest.json')
    assert m['status']=='incomplete' and m['recorded_cells']==120 and m['training_admitted'] is False
    assert m['counts']==dict(precheck=dict(attempted_steps=12000,advancing_steps=12000),training=dict(attempted_steps=0,advancing_steps=0),held_out=dict(attempted_steps=0,advancing_steps=0))
    assert 'frozen precheck audit failed' in m['reason']
    return dict(audit_pass=True,precheck_pass=True,prefix_archive=str(prefix.resolve()),prefix_seal=PREFIX_SEAL,
        original_frozen_checker_pass=False,postmeasurement_accounting_pass=True,physics_replayed=False,
        original_checker_sha256=a.digest(original),corrected_checker_sha256=hashlib.sha256(corrected.encode()).hexdigest(),
        correction=dict(before=before,after=after,sites=1),counts=m['counts'])

def vector(v):
    assert len(v)==183 and all(type(x) in (int,float) and math.isfinite(x) for x in v)

def vec_close(xs,ys):
    vector(xs);vector(ys)
    assert all(abs(x-y)<2e-7 for x,y in zip(xs,ys)), 'parameter arithmetic'

def native(row,catalogue,h):
    q=row['schedule']
    if row['status']=='NotRun':
        assert row['evaluation'] is None and row['reason']=='not_started'
        return 0,0,None
    assert 0<=row['elapsed_seconds']
    e=row['emission'];vector(row['parameters'])
    h.check_membership(catalogue,e,full=True)
    assert e['emission_seed']==q['emission_seed'] and e['kernel']['training_admitted'] is False
    assert e['kernel']['advancing_engine_steps']==0 and not e['kernel']['codec_reencoded'] and not e['kernel']['metadata_recounted']
    if row['status']=='Incomplete':
        assert row['evaluation'] is None and row['partial_steps_unknown'] is True and isinstance(row['reason'],str)
        return 0,0,None
    assert row['status'] in ('Complete','Null')
    v=row['evaluation'];assert v['horizon']==100 and v['physics_seed']==q['physics_seed']
    assert v['runtime_match_budget']==dict(policy='Pooled',work=50000,results=50000)
    classes=e['classes']
    credit=[i for i,b in enumerate(v['candidate_execution']) if classes['active'] and classes['core']!=classes['proposal'] and b==classes['proposal_execution']]
    assert credit==v['credited_rule_ids']
    steps=v['steps'];assert 0<len(steps)<=100 and v['attempted_steps']==len(steps)
    fires=0;reason=None
    for i,s in enumerate(steps,1):
        assert s['step']==i and type(s['advanced']) is int and s['advanced'] in (0,1)
        assert len(s['rule_ids'])<=1 and all(type(n) is int and 0<=n<len(v['candidate_execution']) for n in s['rule_ids'])
        assert type(s['nodes']) is int and type(s['edges']) is int and s['nodes']>=0 and s['edges']>=0
        assert len(s['world_hash'])==16 and all(x in '0123456789abcdef' for x in s['world_hash'])
        h.check_allowances(s['usage'],50000,50000)
        assert s['usage']['truncated']==any(r['truncated'] for r in s['usage']['rules'])
        cap=s['usage']['truncated'] or s['nodes']>2000 or s['edges']>4000
        assert s['cap']==cap
        if s['usage']['truncated']: assert s['advanced']==0 and s['rule_ids']==[]
        fires+=sum(n in credit for n in s['rule_ids'])
        if cap or not s['valid_family'] or s['advanced']!=1:
            assert i==len(steps)
            reason='resource_cap' if cap else 'invalid_family' if not s['valid_family'] else 'nonadvancing'
    full=len(steps)==100 and reason is None
    assert full or reason is not None
    assert v['proposal_fires']==fires and v['advancing_steps']==sum(s['advanced'] for s in steps)
    assert v['reason']==reason and (row['status']=='Complete')==full
    h.close(v['score'],math.log1p(max(fires-1,0))/math.log(100) if full else None)
    assert row['elapsed_seconds']<30
    return len(steps),v['advancing_steps'],v['score']

def distribution(panel,catalogue,h):
    assert len(panel)==64 and [r['seed'] for r in panel]==list(range(SEED+1000,SEED+1064))
    for r in panel:
        h.check_membership(catalogue,r['selection'],full=True)
        assert r['selection']['emission_seed']==r['seed'] and r['signature']==r['selection']['signature']

def update(theta,eps,fits,pairs,h):
    shaped=h.ranks(fits)
    result=theta[:]
    for p,e in enumerate(eps):
        d=h.f32(shaped[2*p]-shaped[2*p+1])
        c=h.f32(h.f32(h.f32(.05)/h.f32(float(pairs)))*d)
        if d: result=[h.f32(t+h.f32(c*x)) for t,x in zip(result,e)]
    return result

def generations(out,rows,catalogue,h,pre):
    count,pairs=(5,4) if pre else (20,8)
    label='pre' if pre else 'train'
    theta=[0.]*183;initial=a.load(out/'initial_distribution.json');distribution(initial,catalogue,h)
    valid_total=0;max_tv=0.;completed=0
    for g in range(count):
        path=out/'generations'/f'{label}-{g}.json'
        if not path.exists(): break
        rec=a.load(path);assert rec['generation']==g and rec['theta_before']==theta
        assert len(rec['eps'])==pairs
        for e in rec['eps']: vector(e)
        fits=[];expected_ids=[];valid=0
        for p,eps in enumerate(rec['eps']):
            arms=[]
            for sign in [1,-1]:
                vals=[]
                for s in PRE[:2] if pre else [TRAIN[(g+p)%4]]:
                    id=f'{label}-es-{g}-{p}-{sign}-{s}';expected_ids.append(id);row=rows[id]
                    assert row['status'] in ('Complete','Null')
                    expected=[h.f32(t+h.f32(sign*x)) for t,x in zip(theta,eps)]
                    vec_close(row['parameters'],expected)
                    vals.append(row['evaluation']['score'])
                arms.append(vals)
            good=all(x is not None for arm in arms for x in arm)
            pair=[h.f32(h.mean(arm)) for arm in arms] if good else [None,None]
            assert rec['pair_scores'][p]==pair
            fits.extend(pair);valid+=good
        assert rec['evaluation_ids']==expected_ids and rec['valid_pairs']==valid
        apply=pre or valid>=4
        assert rec['update_applied']==apply
        expected=update(theta,rec['eps'],fits,pairs,h) if apply else theta
        vec_close(rec['theta_after'],expected);theta=rec['theta_after']
        distribution(rec['distribution'],catalogue,h)
        tv=h.distance(initial,rec['distribution']);h.close(rec['tv_from_initial'],tv)
        max_tv=max(max_tv,tv);valid_total+=valid;completed+=1
        if not apply: break
    assert {p.name for p in (out/'generations').glob(f'{label}-*.json')}=={f'{label}-{g}.json' for g in range(completed)}
    return theta,valid_total,max_tv,completed

def prechecks(out,rows,catalogue,h):
    b=[rows[f'baseline-{i}-{s}']['evaluation']['score'] for i in range(8) for s in PRE]
    controls=[rows[f'control-{on}-{s}']['evaluation']['score'] for on in ['true','false'] for s in PRE]
    active=rows[f'control-true-{PRE[0]}']['emission'];dormant=rows[f'control-false-{PRE[0]}']['emission']
    for on,parent,op in [('true',4,11),('false',5,12)]:
        for s in PRE:
            e=rows[f'control-{on}-{s}']['emission']
            assert e['parent']==parent and e['operator']==op and e['role']=='Proposal' and e['production_index']==0
            assert rows[f'control-{on}-{s}']['parameters']==[0.]*183
    for i in range(8):
        for s in PRE: assert rows[f'baseline-{i}-{s}']['parameters']==[0.]*183
    assert active['classes']['active'] and not dormant['classes']['active']
    assert all(active['classes'][k]==dormant['classes'][k] for k in ('core','proposal','proposal_execution'))
    assert active['initial_world_hash']==dormant['initial_world_hash']
    _,valid,tv,n=generations(out,rows,catalogue,h,True);assert n==5
    baseline_mean=h.mean(b) if all(x is not None for x in b) else None
    sd=h.sd(b) if baseline_mean is not None else None
    gap=h.mean(controls[:4])-h.mean(controls[4:]) if all(x is not None for x in controls) else None
    panel=a.load(out/'initial_distribution.json')
    checks=dict(P1=all(x['signature']!=active['signature'] for x in panel),P2=tv>=.05 and valid==20,
        P3=gap is not None and sd is not None and gap>2*sd and gap>1e-12,
        P4=baseline_mean is not None and .02<baseline_mean<.98)
    envelope=all(rows[q['id']]['status']=='Complete' and rows[q['id']]['evaluation']['advancing_steps']==100 for q in schedule()[:120])
    expected=dict(checks=checks,runtime_envelope_pass=envelope,baseline_mean=baseline_mean,baseline_population_sd=sd,
        forced_gap=gap,valid_pairs=valid,max_distribution_tv=tv,precheck_pass=all(checks.values()) and envelope,rollouts=120,
        training_admitted=False)
    actual=a.load(out/'prechecks.json');assert actual.keys()==expected.keys()
    for k,v in expected.items():
        if isinstance(v,float): h.close(actual[k],v)
        else: assert actual[k]==v,k
    return expected['precheck_pass']

def sign_p(wins,losses):
    n=wins+losses
    return sum(math.comb(n,k) for k in range(wins,n+1))/2**n if n else 1.

def held_statistics(draws):
    all_valid=all(x is not None for d in draws for x in d.values())
    diffs=[d['champion']-d['untrained'] for d in draws if d['champion'] is not None and d['untrained'] is not None]
    random=[d['champion']-d['random'] for d in draws if d['champion'] is not None and d['random'] is not None]
    wins=sum(d>0 for d in diffs);losses=sum(d<0 for d in diffs)
    gap=sum(diffs)/64 if all_valid else None
    p=sign_p(wins,losses) if all_valid else None
    return dict(draws=draws,fixed_denominator=64,all_held_out_valid=all_valid,complete_primary_pairs=len(diffs),
        champion_minus_untrained=gap,champion_minus_random=sum(random)/64 if all_valid else None,
        primary_wins=wins,primary_losses=losses,primary_ties=len(diffs)-wins-losses,one_sided_paired_sign_p=p,
        held_out_improvement_supported=all_valid and gap>0 and p<.05,
        champion_selection='final_generation_parameters',independent_physics_replay=False)

def audit(out,integrity=True,verify_prerequisites=True):
    out=Path(out)
    if integrity: a.verify(out)
    m=a.load(out/'manifest.json');assert m['profile'] in (PROFILE,CONTINUATION) and m['design']==wire_design(m['profile'])
    assert m['schedule']==schedule() and m['scheduled_cells']==1208 and m['full_training_authorized'] is False
    assert {p.name for p in (out/'cells').iterdir()}=={q['id']+'.json' for q in schedule()}
    rows={q['id']:a.load(out/'cells'/(q['id']+'.json')) for q in schedule()}
    for q in schedule(): assert rows[q['id']]['schedule']==q
    nonrun=[r['status']!='NotRun' for r in rows.values()]
    assert nonrun==sorted(nonrun,reverse=True), 'evaluation sequence has gaps'
    assert m['recorded_cells']==sum(nonrun) and m['inflight_native_work_unknown'] is False
    assert m['unknown_partial_work_cells']==sum(r['status']=='Incomplete' for r in rows.values())
    if 'snapshot' not in m:
        assert m['status']=='blocked_before_evaluation' and m['grammar_compilations']==0 and not any(nonrun)
        assert m['training_admitted'] is False and all(v==0 for d in m['counts'].values() for v in d.values())
        return dict(audit_pass=True,classification='zero_work_refusal',training_complete=False)
    proof=m['admission'];assert proof['profile']==PROFILE and proof['implementation_sources_verified']==230
    assert proof['user_authorized_training'] and proof['admitted_horizon']==100 and proof['admitted_generations']==20 and proof['admitted_candidates']==16
    hist=proof['historical_evidence'];assert hist['historical_scopes_preserved'] and hist['training_admitted'] is False
    assert hist['full_training_authorized'] is False and hist['prior_original_frozen_checker_pass'] is False and hist['prior_corrected_accounting_pass'] is True
    assert set(hist['receipts'])==set(a.SPECS)==set(m['receipt_paths'])
    for name,(seal,sha,count) in a.SPECS.items():
        e=hist['receipts'][name];assert e['seal_sha256']==sha and e['seal_file']==seal and e['sealed_files_verified']==count and e['exact_file_coverage']
        assert Path(e['archive'])==Path(m['receipt_paths'][name])
        if verify_prerequisites: a.verify(Path(e['archive']),seal,sha,count)
    runtime=Path(m['receipt_paths']['runtime']);h=a.helper(runtime)
    pins={k:v for k,v in a.load(runtime/'manifest.json')['snapshot']['sources'].items() if a.pinned(k)}
    assert len(pins)==230
    snap=m['snapshot'];assert snap['frozen_before_measurement']
    train_files={str(p.relative_to(out/'source_snapshots')).replace('\\','/') for p in (out/'source_snapshots/crates/hg-train').rglob('*') if p.is_file() and p.suffix in ('.rs','.toml')}
    assert set(snap['sources'])==set(pins)|train_files|EXTRA
    assert set(a.files(out/'source_snapshots'))==set(snap['sources'])
    assert a.digest(out/'executed_binary')==snap['binary_sha256']
    for name,sha in snap['sources'].items():
        assert a.safe(name) and a.digest(out/'source_snapshots'/name)==sha
        if name in pins and name!=SIGNAL: assert sha==pins[name],name
    old=(runtime/'source_snapshots'/SIGNAL).read_text()
    assert (out/'source_snapshots'/SIGNAL).read_text()==repaired_source(old)
    assert proof['native_source_bridge']==dict(source=SIGNAL,historical_sha256=pins[SIGNAL],current_sha256=snap['sources'][SIGNAL],exact_registered_transform=True)
    assert a.load(out/'source_snapshots/configs/ere_complete_short_training_v1.json')==design()
    if m['profile']==CONTINUATION:
        assert a.load(out/'source_snapshots/configs/ere_complete_short_training_continuation_v2.json')=={**design(),'profile':CONTINUATION}
        prefix=Path(m['precheck_prefix'])
        review=review_prefix(prefix,verify_prerequisites)
        assert a.load(out/'prefix_accounting_review.json')==review
        old_snap=a.load(prefix/'manifest.json')['snapshot']['sources']
        assert all(snap['sources'][p]==old_snap[p] for p in pins)
        assert m['precheck_steps_reused']==12000 and m['new_precheck_native_steps']==0
        for q in schedule()[:120]:
            p='cells/'+q['id']+'.json'
            assert a.digest(out/p)==a.digest(prefix/p)
        for p in ['prechecks.json','initial_distribution.json']+[f'generations/pre-{g}.json' for g in range(5)]:
            assert a.digest(out/p)==a.digest(prefix/p)
    if not (out/'catalogue.json').exists():
        assert not any(nonrun) and m['training_admitted'] is False
        return dict(audit_pass=True,classification='incomplete_setup',training_complete=False)
    assert a.digest(out/'catalogue.json')==a.CATALOGUE and m['grammar_compilations']==1
    catalogue=a.load(out/'catalogue.json');counts={stage:dict(attempted_steps=0,advancing_steps=0) for stage in ['precheck','training','held_out']}
    world=set()
    for r in rows.values():
        attempted,advanced,_=native(r,catalogue,h);stage=r['schedule']['stage']
        counts[stage]['attempted_steps']+=attempted;counts[stage]['advancing_steps']+=advanced
        if r['status']!='NotRun': world.add(r['emission']['initial_world_hash'])
    assert len(world)<=1 and counts==m['counts']
    assert counts['precheck']['attempted_steps']<=12000 and counts['training']['attempted_steps']<=32000 and counts['held_out']['attempted_steps']<=76800
    pass_=prechecks(out,rows,catalogue,h) if (out/'prechecks.json').exists() else False
    assert m['training_admitted']==(pass_ and m.get('precheck_audit_pass',False))
    if not pass_:
        assert all(r['status']=='NotRun' for r in list(rows.values())[120:])
        assert m['status']=='incomplete'
        return dict(audit_pass=True,classification='precheck_failure',training_complete=False,precheck_pass=False,counts=counts)
    theta,pairs,tv,n=generations(out,rows,catalogue,h,False)
    if m['status']=='prechecks':
        assert n==0 and all(r['status']=='NotRun' for r in list(rows.values())[120:])
        return dict(audit_pass=True,classification='fresh_prechecks_verified',training_complete=False,precheck_pass=True,counts=counts)
    if not m.get('precheck_audit_pass',False):
        assert m['status']=='incomplete' and n==0 and all(r['status']=='NotRun' for r in list(rows.values())[120:])
        return dict(audit_pass=True,classification='fresh_admission_audit_failed',training_complete=False,precheck_pass=True,counts=counts)
    assert m['precheck_audit_pass'] is True
    preaudit=a.load(out/'precheck_audit.json')
    assert preaudit['audit_pass'] and preaudit['precheck_pass'] and preaudit['classification']=='fresh_prechecks_verified'
    if m['status']!='complete':
        assert m['status']=='incomplete' and isinstance(m['reason'],str)
        return dict(audit_pass=True,classification='incomplete_training_campaign',training_complete=False,precheck_pass=True,counts=counts)
    assert n==20 and all(nonrun) and m['unknown_partial_work_cells']==0 and m['measurement_complete'] and m['live_sources_rechecked']
    champion=a.load(out/'champion.json');assert champion['parameters']==theta and champion['selection']=='final_generation_parameters' and champion['generation']==20
    assert champion['valid_pairs']==pairs;h.close(champion['max_distribution_tv'],tv)
    random=a.load(out/'random_parameters.json');assert len(random)==64
    for v in random: vector(v)
    assert len({tuple(v) for v in random})==64 and all(any(x!=0 for x in v) for v in random)
    draws=[]
    for i in range(64):
        d={}
        for arm in ['champion','untrained','random']:
            vals=[]
            for s in HOLD:
                row=rows[f'hold-{i}-{arm}-{s}'];assert row['status'] in ('Complete','Null')
                assert row['parameters']==(theta if arm=='champion' else [0.]*183 if arm=='untrained' else random[i])
                vals.append(row['evaluation']['score'])
            d[arm]=h.mean(vals) if all(x is not None for x in vals) else None
        draws.append(d)
    expected=held_statistics(draws);summary=a.load(out/'summary.json')
    for k,v in expected.items():
        if isinstance(v,float): h.close(summary[k],v)
        else: assert summary[k]==v,k
    assert summary['profile']==m['profile'] and summary['training_complete'] and summary['training_admitted'] and summary['full_training_authorized'] is False and summary['counts']==counts
    assert 0<=m['grammar_compile_seconds']<=m['elapsed_seconds']<3600
    assert sum((out/p).stat().st_size for p in a.files(out))<512*1024**2
    return dict(audit_pass=True,classification='completed_100_step_training',training_complete=True,precheck_pass=True,
        counts=counts,held_out_improvement_supported=expected['held_out_improvement_supported'],
        champion_minus_untrained=expected['champion_minus_untrained'],champion_minus_random=expected['champion_minus_random'],
        one_sided_paired_sign_p=expected['one_sided_paired_sign_p'],frozen_sources=len(snap['sources']),independent_physics_replay=False)

def tamper_tests(out):
    out=Path(out)
    cases=['schedule','design','training_flag','counts','source','binary','coverage','bridge','precheck',
        'score','cap','credit','membership','update','champion','heldout','random','seed']
    for case in cases:
        with tempfile.TemporaryDirectory(prefix='ere-short-audit-') as tmp:
            dst=Path(tmp)/'receipt';shutil.copytree(out,dst)
            m=a.load(dst/'manifest.json')
            if case=='schedule':m['schedule'].pop()
            elif case=='design':m['design']['horizon']=10000
            elif case=='training_flag':m['training_admitted']=not m['training_admitted']
            elif case=='counts':m['counts']['training']['attempted_steps']+=1
            elif case=='source':(dst/'source_snapshots/crates/hg-train/src/optim.rs').write_text('stale')
            elif case=='binary':(dst/'executed_binary').write_bytes(b'stale')
            elif case=='coverage':m['snapshot']['sources'].pop(SIGNAL)
            elif case=='bridge':
                p=dst/'source_snapshots'/SIGNAL;p.write_text(p.read_text().replace('        capture.lock().unwrap().clear();\n',''))
                m['snapshot']['sources'][SIGNAL]=a.digest(p);m['admission']['native_source_bridge']['current_sha256']=a.digest(p)
            elif case=='precheck':
                p=dst/'prechecks.json';v=a.load(p);v['checks']['P2']=not v['checks']['P2'];p.write_text(json.dumps(v))
            elif case in ['score','cap','credit','membership','seed']:
                p=dst/'cells'/f'control-true-{PRE[0]}.json';v=a.load(p)
                if case=='score':v['evaluation']['score']=.123
                elif case=='cap':v['evaluation']['steps'][-1]['cap']=True
                elif case=='credit':v['evaluation']['credited_rule_ids']=[]
                elif case=='membership':v['emission']['production_index']=999999
                else:v['schedule']['physics_seed']+=1
                p.write_text(json.dumps(v))
            elif case=='update':
                p=next((dst/'generations').glob('pre-*.json'));v=a.load(p);v['theta_after'][0]+=1;p.write_text(json.dumps(v))
            elif case=='champion':
                if not (dst/'champion.json').exists():continue
                p=dst/'champion.json';v=a.load(p);v['parameters'][0]+=1;p.write_text(json.dumps(v))
            elif case=='heldout':
                if not (dst/'summary.json').exists():continue
                p=dst/'summary.json';v=a.load(p);v['champion_minus_untrained']=123;p.write_text(json.dumps(v))
            elif case=='random':
                if not (dst/'random_parameters.json').exists():continue
                p=dst/'random_parameters.json';v=a.load(p);v[0]=[0.]*183;p.write_text(json.dumps(v))
            (dst/'manifest.json').write_text(json.dumps(m));a.seal(dst)
            try:audit(dst,verify_prerequisites=False)
            except (AssertionError,KeyError,ValueError,FileNotFoundError,IndexError):continue
            raise AssertionError('tamper accepted: '+case)
    return len(cases) if (out/'summary.json').exists() else len(cases)-3

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('out',type=Path);parser.add_argument('--tamper-tests',action='store_true');parser.add_argument('--seal',action='store_true');parser.add_argument('--admission-check',action='store_true');parser.add_argument('--review-prefix',action='store_true')
    args=parser.parse_args();result=review_prefix(args.out) if args.review_prefix else audit(args.out,integrity=not args.admission_check)
    if args.admission_check: assert result['classification']=='fresh_prechecks_verified' and result['precheck_pass']
    if args.tamper_tests:result['tamper_rejections']=tamper_tests(args.out)
    if args.seal:
        (args.out/'audit.json').write_text(json.dumps(result,indent=2));a.seal(args.out);audit(args.out)
    print(json.dumps(result,indent=2))
