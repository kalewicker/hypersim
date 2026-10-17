#!/usr/bin/env python3
"""Prospective schedule/math/source checks only. No native runner or launch."""
import argparse
import collections
import json
import math
from pathlib import Path
import shutil
import struct
import tomllib
import audit_ere_complete_admission as a

ROOT=Path(__file__).resolve().parents[1]
CONFIG=ROOT/'configs/ere_categorical_comparison_v1.json'
PROFILE='E2-categorical-versus-es-direction-v1'
SOURCES=['configs/ere_categorical_comparison_v1.json','docs/ere_categorical_comparison_v1.md',
    'scripts/check_ere_categorical_comparison_plan.py','scripts/test_check_ere_categorical_comparison_plan.py']

def f32(x):return struct.unpack('<f',struct.pack('<f',x))[0]
def ordered_sum(xs):
    total=0.
    for x in xs:total+=x
    return total
def same(got,expected):
    assert type(got) is type(expected)
    if isinstance(expected,dict):
        assert set(got)==set(expected)
        for k,v in expected.items():same(got[k],v)
    elif isinstance(expected,list):
        assert len(got)==len(expected)
        for g,e in zip(got,expected):same(g,e)
    else:assert got==expected
def contexts():return [[0.]*183,[.25 if i%2==0 else -.25 for i in range(183)]]
def validate(c):
    assert set(c)=={'profile','objective','statistical_unit','parameter_panel','parameters','contexts','methods','estimates_per_method',
        'raw_alpha','es_sigma','es_pairs','es_candidate_count','categorical_baseline','step_rule','step_radius','maximum_actual_step_norm',
        'maximum_absolute_coordinate_step','zero_centered_direction','validation_estimator','validation_utility_control_variate',
        'validation_validity_control_variate','weight_clipping','weight_self_normalization','weight_bound','validation_vectors','reverse_rule',
        'sign_deadband','significance_threshold','acceptance','native','resources','seed_domains','seed_strides','scientific','development','lineage','adapter_registration','excluded_feature_sources'}
    assert all(type(c[k]) is int for k in ('parameters','contexts','estimates_per_method','es_pairs','es_candidate_count'))
    assert all(type(c[k]) is float for k in ('raw_alpha','es_sigma','step_radius','maximum_actual_step_norm','maximum_absolute_coordinate_step',
        'validation_utility_control_variate','validation_validity_control_variate','sign_deadband','significance_threshold'))
    assert c['profile']==PROFILE and c['objective']=='mean-constraint-utility-v1'
    assert c['statistical_unit']=='paired_estimator_block' and c['parameter_panel']=='zero-and-alternating-quarter-v1'
    assert c['parameters']==183 and c['contexts']==2 and c['methods']==['es','categorical'] and c['estimates_per_method']==2
    assert c['es_pairs']==8 and c['es_candidate_count']==16 and c['es_sigma']==1. and c['raw_alpha']==f32(.05)
    assert c['step_rule']=='actual-f32-delta-group-centered-fixed-l2-v1' and c['step_radius']==.1
    assert c['maximum_actual_step_norm']==c['maximum_absolute_coordinate_step']==.100001
    assert c['categorical_baseline']=='leave-one-out-emission-mean-v1' and c['zero_centered_direction']=='retain_base_and_all_allocations'
    assert c['validation_estimator']=='base-leaf-importance-effect-fixed-control-variate-v1'
    assert c['validation_utility_control_variate']==c['validation_validity_control_variate']==1.
    assert c['weight_clipping'] is False and c['weight_self_normalization'] is False
    assert c['weight_bound']=='exp(plus_or_minus_6_times_maximum_absolute_coordinate_step)'
    assert c['validation_vectors']==['base','es_a','es_b','categorical_a','categorical_b','reverse_es_a','reverse_es_b','reverse_categorical_a','reverse_categorical_b']
    assert c['reverse_rule']=='f32(2_times_base_minus_forward)' and c['sign_deadband']==1e-12 and c['significance_threshold']==.05
    assert set(c['acceptance'])=={'complete_fixed_schedule_and_known_outcomes','categorical_alignment_mean_above_deadband_and_sign_p_below_threshold',
        'categorical_utility_effect_mean_above_deadband_and_sign_p_below_threshold','categorical_minus_es_effect_mean_above_deadband_and_sign_p_below_threshold',
        'categorical_utility_effect_above_deadband_in_both_contexts','categorical_weighted_validity_effect_at_least_minus_deadband','conjunction_only'}
    assert all(v is True for v in c['acceptance'].values())
    same(c['native'],dict(grammar_profile='E2-complete-valid-production-grammar-v1',ordinary_world='signal-fixed-world-v1',
        allocation_mode='metadata-isolated-v1',matcher_work_model='repeated-self-pair-feasibility-v1',step_mode='MaxParallel',max_pairings=1,
        match_budget_policy='Pooled',work=50000,results=50000,horizon=100,node_ceiling=2000,edge_ceiling=4000))
    same(c['resources'],dict(grammar_compilations=1,cell_wall_seconds=30,output_cap_bytes=2*1024**3,receipt_reserve_bytes=128*1024**2,
        final_reserve_bytes=16*1024**2,auditor_wall_seconds=900,optimizer_generations=0,training_updates=0,full_training_authorized=False,
        scientific_launch_requires_separate_approval=True))
    same(c['seed_domains'],dict(es_noise=10000000,estimation_emission=20000000,estimation_physics=40000000,validation_emission=60000000,validation_physics=80000000))
    same(c['seed_strides'],dict(block=1000000,context=100000,method=10000,estimate=1000))
    same(c['excluded_feature_sources'],[
        'crates/hg-sim-core/src/bin/evolving_minds.rs',
        'crates/hg-sim-core/src/evolving_minds/config.rs',
        'crates/hg-sim-core/src/evolving_minds/mod.rs',
        'crates/hg-sim-core/src/evolving_minds/runtime.rs',
        'crates/hg-sim-core/src/evolving_minds/source_inventory.rs'])
    assert set(c['lineage'])=={'direction','adapter','failed_adapter_packaging'}
    assert set(c['adapter_registration'])=={'crates/hg-train/Cargo.toml','crates/hg-train/src/ere.rs'}
    for name,pin in c['adapter_registration'].items():
        assert set(pin)=={'historical_sha256','registered_sha256','change'}
        assert pin['change']==('append_only_required_feature_offline_binary' if name.endswith('Cargo.toml') else 'add_only_feature_gated_categorical_module_export')
    for mode,seed,blocks,k,draws,wall in [('scientific',0xE66000000000,8,4,128,3600),('development',0xE66100000000,1,1,8,900)]:
        l=c[mode];n=16*k;est=blocks*2*4*n;val=blocks*2*draws*4
        same(l,dict(mode=mode,master_seed=seed,blocks=blocks,es_emissions_per_candidate=k,categorical_emissions_per_estimate=n,
            validation_emissions_per_block_context=draws,validation_physics_per_emission=4,direction_estimates=blocks*2*4,
            estimation_evaluations=est,validation_evaluations=val,total_evaluations=est+val,maximum_native_attempts=(est+val)*100,wall_seconds=wall))
    return c

def schedule(c,mode):
    validate(c);l=c[mode];master=l['master_seed'];domains=c['seed_domains'];rows=[]
    for r in range(l['blocks']):
        for ctx in range(2):
            for m,method in enumerate(c['methods']):
                for e in range(2):
                    j=1000000*r+100000*ctx+10000*m+1000*e
                    if method=='es':
                        for p in range(8):
                            for sign in (1,-1):
                                for b in range(l['es_emissions_per_candidate']):
                                    i=4*p+b
                                    rows.append(dict(id=f'est-r{r}-c{ctx}-es-e{e}-p{p}-s{sign}-b{b}',stage='estimation',block=r,context=ctx,
                                        method=method,estimate=e,pair=p,sign=sign,batch_draw=b,noise_seed=master+domains['es_noise']+j,
                                        emission_seed=master+domains['estimation_emission']+j+i,physics_seed=master+domains['estimation_physics']+j+i,horizon=100))
                    else:
                        for i in range(l['categorical_emissions_per_estimate']):
                            rows.append(dict(id=f'est-r{r}-c{ctx}-categorical-e{e}-d{i}',stage='estimation',block=r,context=ctx,
                                method=method,estimate=e,draw=i,emission_seed=master+domains['estimation_emission']+j+i,
                                physics_seed=master+domains['estimation_physics']+j+i,horizon=100))
    for r in range(l['blocks']):
        for ctx in range(2):
            for d in range(l['validation_emissions_per_block_context']):
                for p in range(4):
                    j=1000000*r+100000*ctx
                    rows.append(dict(id=f'val-r{r}-c{ctx}-d{d}-p{p}',stage='validation',block=r,context=ctx,draw=d,physics_index=p,
                        sampled_arm='base',emission_seed=master+domains['validation_emission']+j+d,
                        physics_seed=master+domains['validation_physics']+j+4*d+p,horizon=100))
    return rows

def check_schedule(c,mode):
    rows=schedule(c,mode);l=c[mode];assert len(rows)==l['total_evaluations'] and len({r['id'] for r in rows})==len(rows)
    assert collections.Counter(r['stage'] for r in rows)==dict(estimation=l['estimation_evaluations'],validation=l['validation_evaluations'])
    domains={name:set() for name in ('noise','est_emission','est_physics','val_emission','val_physics')}
    panels=[]
    for r in range(l['blocks']):
        for ctx in range(2):
            for method in c['methods']:
                for e in range(2):
                    panel=[x for x in rows if x['stage']=='estimation' and (x['block'],x['context'],x['method'],x['estimate'])==(r,ctx,method,e)]
                    assert len(panel)==l['categorical_emissions_per_estimate']
                    seeds={(x['emission_seed'],x['physics_seed']) for x in panel}
                    assert len(seeds)==(8*l['es_emissions_per_candidate'] if method=='es' else l['categorical_emissions_per_estimate'])
                    for old in panels:assert not (old & seeds)
                    panels.append(seeds)
                    for x in panel:
                        domains['est_emission'].add(x['emission_seed']);domains['est_physics'].add(x['physics_seed'])
                    if method=='es':
                        for p in range(8):
                            pair=[[ (x['emission_seed'],x['physics_seed']) for x in panel if x['pair']==p and x['sign']==s] for s in (1,-1)]
                            assert pair[0]==pair[1]
                        ns={x['noise_seed'] for x in panel};assert len(ns)==1 and not (domains['noise'] & ns);domains['noise']|=ns
    for x in rows:
        if x['stage']=='validation':domains['val_emission'].add(x['emission_seed']);domains['val_physics'].add(x['physics_seed'])
    for name,seeds in domains.items():
        assert all(type(s) is int and 0<=s<2**53 for s in seeds)
        for other,other_seeds in domains.items():
            if name!=other:assert not (seeds & other_seeds)
    return dict(mode=mode,counts={k:len(v) for k,v in domains.items()},rows=rows,seeds=set().union(*domains.values()))

def matched_step(base,raw):
    assert len(base)==len(raw)==183 and all(math.isfinite(x) for x in base+raw)
    base=[f32(x) for x in base];raw=[f32(x) for x in raw];delta=[r-b for r,b in zip(raw,base)];centered=[]
    for start,end in ((0,168),(168,181),(181,183)):
        mean=ordered_sum(delta[start:end])/(end-start);centered.extend(x-mean for x in delta[start:end])
    norm=math.sqrt(ordered_sum(x*x for x in centered));forward=base[:] if norm==0 else [f32(b+.1*x/norm) for b,x in zip(base,centered)]
    reverse=[f32(f32(2*b)-f) for b,f in zip(base,forward)]
    for vector in (forward,reverse):
        diff=[u-b for u,b in zip(vector,base)];assert max(abs(x) for x in diff)<=.100001 and math.sqrt(ordered_sum(x*x for x in diff))<=.100001
    return dict(raw_delta=delta,centered_delta=centered,centered_norm=norm,degenerate=norm==0,forward=forward,reverse=reverse)

def importance_effect(weights,utilities,validities):
    assert len(weights)==len(utilities)==len(validities)>0
    assert all(math.isfinite(x) for xs in (weights,utilities,validities) for x in xs)
    assert all(math.exp(-6*.100001)-1e-12<=w<=math.exp(6*.100001)+1e-12 for w in weights)
    assert all(0<=u<=2 for u in utilities) and all(0<=v<=1 for v in validities)
    n=len(weights)
    return dict(utility_effect=ordered_sum((u-1)*(w-1) for u,w in zip(utilities,weights))/n,
        validity_effect=ordered_sum((v-1)*(w-1) for v,w in zip(validities,weights))/n)

def statistics(c,mode,align,effects,validity,complete):
    validate(c);assert type(complete) is bool;n=c[mode]['blocks'];assert len(align)==len(effects)==len(validity)==n
    assert all(len(r)==2 and all(len(ctx)==2 for ctx in r) for data in (align,effects,validity) for r in data)
    assert all(len(v)==2 for data in (effects,validity) for r in data for ctx in r for v in ctx)
    values=[v for r in align for ctx in r for v in ctx]+[v for data in (effects,validity) for r in data for ctx in r for m in ctx for v in m]
    assert all(v is None or type(v) in (int,float) and math.isfinite(v) for v in values)
    known=complete and all(v is not None for v in values);eps=c['sign_deadband'];alpha=c['significance_threshold']
    def summarize(xs):
        wins=sum(x>eps for x in xs);losses=sum(x< -eps for x in xs);total=wins+losses
        return dict(mean=ordered_sum(xs)/n if known else None,sign_p=sum(math.comb(total,k) for k in range(wins,total+1))/2**total if known else None,
            wins=wins,losses=losses,ties=len(xs)-wins-losses,fixed_denominator=n)
    if known:
        context=[[ [ordered_sum(effects[r][ctx][m])/2 for m in range(2)] for ctx in range(2)] for r in range(n)]
        cr=[ordered_sum(row[1] for row in r)/2 for r in context];dr=[ordered_sum(row[1]-row[0] for row in r)/2 for r in context]
        lr=[ordered_sum(row[1] for row in r)/2 for r in align]
        cm=[ordered_sum(context[r][ctx][1] for r in range(n))/n for ctx in range(2)]
        vf=ordered_sum(validity[r][ctx][1][e] for r in range(n) for ctx in range(2) for e in range(2))/(n*4)
    else:cr=dr=lr=[];cm=[None,None];vf=None
    summaries={k:summarize(v) for k,v in [('categorical_utility',cr),('categorical_minus_es',dr),('categorical_alignment',lr)]}
    support=mode=='scientific' and known and all(v['mean']>eps and v['sign_p']<alpha for v in summaries.values()) and all(x>eps for x in cm) and vf>=-eps
    return dict(mode=mode,statistical_unit='paired_estimator_block',fixed_denominator=n,all_outcomes_known=known,
        block_categorical_effects=cr if known else [None]*n,block_contrasts=dr if known else [None]*n,block_alignment=lr if known else [None]*n,
        confirmatory=summaries,categorical_context_effects=cm,categorical_weighted_validity_effect=vf,support_categorical_direction=support)

def check_registration(c):
    validate(c)
    old=ROOT/c['lineage']['direction']['path'];snapshot=a.load(old/'snapshot.json');changed=[];excluded={}
    for name,h in snapshot['sources'].items():
        if name in c['excluded_feature_sources']:
            excluded[name]=dict(historical_sha256=h,observed_live_sha256=a.digest(ROOT/name));continue
        if a.digest(ROOT/name)!=h:changed.append(name)
    assert len(snapshot['sources'])==459 and len(excluded)==5
    assert set(changed)==set(c['adapter_registration']),f'unregistered ERE/shared source drift: {changed}'
    for name,pin in c['adapter_registration'].items():
        assert snapshot['sources'][name]==pin['historical_sha256'] and a.digest(ROOT/name)==pin['registered_sha256']
    name='crates/hg-train/Cargo.toml';before=tomllib.loads((old/'source_snapshots'/name).read_text());after=tomllib.loads((ROOT/name).read_text())
    assert after['bin'].pop()==dict(name='ere-categorical-adapter',path='src/bin/ere-categorical-adapter.rs',**{'required-features':['ere-complete-signal']})
    assert before==after,'manifest changed beyond offline binary registration'
    name='crates/hg-train/src/ere.rs';before=(old/'source_snapshots'/name).read_text();anchor='#[cfg(feature = "ere-complete-signal")]\npub mod direction;\n'
    assert before.count(anchor)==1 and (ROOT/name).read_text()==before.replace(anchor,anchor+'#[cfg(feature = "ere-complete-signal")]\npub mod categorical;\n',1)
    assert '#[cfg(feature = "evolving-minds")]\npub mod evolving_minds;' in (ROOT/'crates/hg-sim-core/src/lib.rs').read_text()
    sim=tomllib.loads((ROOT/'crates/hg-sim-core/Cargo.toml').read_text())
    assert sim['features']['evolving-minds']==[]
    assert 'evolving-minds' not in after['features']['ere-complete-signal']
    adapter_sources=a.load(ROOT/c['lineage']['adapter']['path']/'manifest.json')['sources']
    assert len(adapter_sources)==467
    for name,h in adapter_sources.items():
        if name not in excluded:assert a.digest(ROOT/name)==h,f'adapter/ERE source drift: {name}'
    return dict(historical_source_entries=459,unchanged_live_entries=452,exact_adapter_registration=c['adapter_registration'],
        adapter_source_entries=467,adapter_live_entries_matching=462,
        excluded_feature='evolving-minds',excluded_feature_observations=excluded,source_scope='prospective ERE plan only; not native launch receipt',legacy_guards_modified=False)

def check(c,verify_lineage=False):
    validate(c);s=check_schedule(c,'scientific');dev=check_schedule(c,'development');assert not (s['seeds']&dev['seeds'])
    if verify_lineage:
        for name,pin in c['lineage'].items():a.verify(ROOT/pin['path'],expected=pin['root_seal'],count=pin['sealed_entries'])
        adapter=a.load(ROOT/c['lineage']['adapter']['path']/'audit.json');assert adapter['audit_pass'] and adapter['classification']=='engineering_arithmetic_only'
        direction=a.load(ROOT/c['lineage']['direction']['path']/'audit.json');assert direction['audit_pass'] and direction['support_direction'] is False
    return dict(plan_valid=True,classification='prospective_plan_only',native_steps_run=0,grammar_compilations=0,training_updates=0,
        native_harness_implemented=False,development_ready=False,scientific_ready=False,scientific_launch_authorized=False,training_admitted=False,
        scientific={k:v for k,v in s.items() if k not in ('rows','seeds')},development={k:v for k,v in dev.items() if k not in ('rows','seeds')},
        lineage_seals_verified=verify_lineage,registration=check_registration(c))

def freeze(out):
    out=Path(out);assert not out.exists(),'fresh preregistration output required';c=a.load(CONFIG);result=check(c,verify_lineage=True)
    out.mkdir();sources={}
    for name in SOURCES:
        dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest);sources[name]=a.digest(dest)
    assert all(a.digest(ROOT/name)==h for name,h in sources.items()),'plan changed while freezing'
    check_registration(c)
    def write(name,value):(out/name).write_text(json.dumps(value,indent=2))
    write('validation.json',result);write('scientific_schedule.json',schedule(c,'scientific'));write('development_schedule.json',schedule(c,'development'));write('contexts.json',contexts())
    write('manifest.json',dict(profile=PROFILE,status='preregistered_not_implemented',classification='prospective_plan_only',sources=sources,
        artifacts={n:a.digest(out/n) for n in ['validation.json','scientific_schedule.json','development_schedule.json','contexts.json']},
        native_steps_run=0,grammar_compilations=0,training_updates=0,scientific_launch_authorized=False,training_admitted=False))
    a.seal(out);a.verify(out);print(json.dumps(dict(root_seal=a.digest(out/'output_sha256.json'),sealed_entries=a.verify(out),**result),indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path);parser.add_argument('--verify-lineage',action='store_true');args=parser.parse_args()
    if args.output:freeze(args.output)
    else:print(json.dumps(check(a.load(CONFIG),args.verify_lineage),indent=2))
