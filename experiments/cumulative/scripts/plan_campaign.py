"""Exact integer prospective sizing and conservative bounds; performs no native work."""
import hashlib,json,math,sys
from pathlib import Path
from functools import lru_cache
from fractions import Fraction
FLAGS=['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']
def ratio(v):return Fraction(v['numerator'],v['denominator'])
def probability(num,den):return {'numerator':num,'denominator':den,'value':num/den}
def statistics(contract):
    inference=contract['inference'];floor=ratio(inference['prevalence_floor']);a=ratio(inference['prevalence_alpha']);pa=ratio(inference['advantage_alpha']);effect=ratio(inference['minimum_observed_paired_advantage']);target=ratio(inference['joint_power_target']);maximum=contract['design']['replicate_search_max'];rows=[]
    @lru_cache(None)
    def critical(n):
        denom=floor.denominator**n
        return next((k for k in range(n+1) if sum(math.comb(n,i)*floor.numerator**i*(floor.denominator-floor.numerator)**(n-i) for i in range(k,n+1))*a.denominator<=a.numerator*denom),n+1)
    @lru_cache(None)
    def paired(n,d,b):
        return (2*b-d)*effect.denominator>=effect.numerator*n and sum(math.comb(d,j) for j in range(b,d+1))*pa.denominator<=pa.numerator*2**d
    for scenario in inference['scenarios']:
        w=scenario['weights'];both,full,external,neither=[w[k] for k in ['both','endogenous_only','external_only','neither']];den=sum(w.values())
        @lru_cache(None)
        def agreement(m,k):return sum(math.comb(m,j)*both**j*neither**(m-j) for j in range(max(0,k),m+1))
        for n in range(1,maximum+1):
            k=critical(n);joint=0;advantage=0
            for d in range(n+1):
                for b in range(d+1):
                    if not paired(n,d,b):continue
                    coefficient=math.comb(n,d)*math.comb(d,b)*full**b*external**(d-b)
                    joint+=coefficient*agreement(n-d,k-b);advantage+=coefficient*(both+neither)**(n-d)
            prevalence=sum(math.comb(n,j)*(both+full)**j*(external+neither)**(n-j) for j in range(k,n+1));total=den**n
            rows.append({'scenario':scenario['name'],'blocks':n,'prevalence_successes_required':k,'prevalence_power':probability(prevalence,total),'advantage_power':probability(advantage,total),'joint_power':probability(joint,total),'joint_power_target_met':joint*target.denominator>=target.numerator*total})
    selected=next((r['blocks'] for r in rows if r['scenario']=='target' and r['joint_power_target_met']),None)
    return {'rows':rows,'selected_blocks':selected,'selection_uses_scientific_outcomes':False}
def panel(contract,n):
    d=contract['design'];w=d['world'];rows=[];seeds=[]
    for i in range(n):
        seed=int.from_bytes(hashlib.sha256((d['seed_domain']+'/'+str(i)).encode('ascii')).digest()[:8],'big');seeds.append(seed)
        for arm in d['arms']:
            rows.append({'block':i,'arm':arm,'seed':seed,'world_id':f'block-{i:03}/{arm}','programs':[{'stages':w['founder_stages'],'enabled':int(arm not in ['external','frozen']),'mask':31 if arm not in ['external','frozen'] else 0} for _ in range(w['founders'])],'capital':w['capital'],'reservoir':w['reservoir'],'parcels_per_tick':w['parcels_per_tick'],'sites':w['sites'],'ticks':w['ticks']})
    if len(seeds)!=len(set(seeds)) or set(seeds)&set(d['development_seeds']):raise ValueError('fixed seed panel collision/development overlap')
    return {'status':'proposal_only','worlds':rows,'blocks':n,'world_count':len(rows),'sampling_slots':len(rows)*len(d['census_ticks'])*d['slots_per_census'],**{f:False for f in FLAGS}}
def resources(contract,n,evidence):
    d=contract['design'];w=d['world'];ticks=w['ticks'];arms=d['arms'];cellcap=w['retained_cells'];sites=w['sites'];cadence=len(range(0,ticks,w['edit_every_ticks']));parcels=min(w['reservoir']//16,w['parcels_per_tick']*ticks);metabolic=5*parcels;births=cellcap-w['founders'];per_arm={};reservation_arm={}
    for arm in arms:
        per_arm[arm]=metabolic+(sites*cadence if arm in ['endogenous','nonheritable'] else births if arm=='external' else 0)
        reservation_arm[arm]=5*w['parcels_per_tick']+cellcap+(sites if arm=='external' else 0)
    novel=len(set(w['executable_stages'])-set(w['founder_stages']));pairs=math.comb(novel,2);samples=len(d['census_ticks'])*d['slots_per_census'];roles=len(contract['candidate']['roles']);requests_per_world=samples*(3+pairs*roles);world_count=n*len(arms);requests=requests_per_world*world_count*d['diagnostic_contexts'];world_work=n*sum(per_arm.values());unknown_reservations=n*sum(reservation_arm.values());assay_work=requests*20
    # Empirical scale projections are explicitly not certified upper bounds.
    world_bytes=evidence['world_journal_bytes']/evidence['world_cases'];world_seconds=evidence['world_command_seconds']/evidence['world_cases'];assay_bytes=evidence['assay_journal_bytes']/evidence['assays'];assay_seconds=evidence['assay_command_seconds']/evidence['assays']
    return {'native_upper_per_arm':per_arm,'single_unpaired_intent_upper_per_arm':reservation_arm,'world_native_upper':world_work,'one_unknown_action_reserve_per_world':unknown_reservations,'maximum_novel_stages_per_path':novel,'maximum_pairs_per_sample':pairs,'slots_per_world':samples,'planned_slots':samples*world_count,'uncached_request_upper_per_world':requests_per_world,'uncached_assay_request_upper':requests,'assay_native_upper':assay_work,'sufficient_conservative_native_reservation':world_work+unknown_reservations+assay_work,'guaranteed_rewrites_within_existing_envelope':world_work+unknown_reservations+assay_work<=contract['existing_envelope']['native_rewrites'],'full_request_bound_within_current_single_request_schema':requests<=contract['existing_envelope']['assays_per_native_request'],'observed_scale_projection':{'world_journal_bytes':math.ceil(world_bytes*world_count*ticks/evidence['average_world_ticks']),'assay_journal_bytes':math.ceil(assay_bytes*requests),'world_command_seconds':world_seconds*world_count*ticks/evidence['average_world_ticks'],'assay_command_seconds':assay_seconds*requests,'certified_storage_or_time_upper_bound':False,'assay_requests_are_uncached_upper_bound':True,'world_size_population_and_program_not_matched':True},'requests_can_be_cached_only_by_frozen_role_and_exact_program':True,'no_low_novelty_assumption_used_to_fund_richer_outcomes':True}
def functional_pair(pair,contract):
    c=contract['candidate'];scores=pair['scores'];roles=c['roles']
    if any(scores.get(k) is None for k in roles) or pair.get('empty_ablation',False) or not pair['all_assays_funded']:return False
    if any(type(scores[k]) is not int for k in roles):raise ValueError('native integer assay scores required')
    if not pair.get('earlier') or not pair.get('later') or pair['earlier']==pair['later']:return False
    margin=[scores['earlier_after']-scores['earlier_before']-c['acquisition_cost'],scores['later_after']-scores['later_before']-c['acquisition_cost']]
    later_with=scores['intact']-scores['padded_remove_later'];later_without=scores['padded_remove_earlier']-scores['padded_remove_both'];floor=c['minimum_paid_incremental_score_gain']
    return len(pair['transmissions'])==len(pair['active_descendant_generations'])==2 and min(pair['transmissions'])>=c['minimum_paid_transmissions_each'] and min(len(set(g)) for g in pair['active_descendant_generations'])>=c['minimum_active_descendant_generations_each'] and min(margin)>=floor and all(scores['intact']>scores[k] for k in ['remove_earlier','remove_later','padded_remove_earlier']) and scores['intact']==scores['sham']==scores['restored'] and later_with-c['acquisition_cost']>=floor and later_without-c['acquisition_cost']<=0 and later_with>later_without
def candidate(pair,arm,contract):
    expected=contract['candidate']['origins'][arm]
    return expected is not None and pair['origins']==[expected,expected] and functional_pair(pair,contract)
def classify_world(world,arm,contract):
    if world['status']!='complete':return {'status':'unknown','qualified':None}
    if len(world['samples'])!=len(contract['design']['census_ticks'])*contract['design']['slots_per_census']:raise ValueError('world sampling denominator')
    if any(s['status'] not in ['living','dead','empty'] for s in world['samples']) or any(p.get('measurement_status') not in ['complete','unfunded','inadmissible_empty_ablation'] for s in world['samples'] for p in s.get('pairs',[])):return {'status':'unknown','qualified':None}
    pairs=[p for s in world['samples'] for p in s.get('pairs',[])]
    if any(p['measurement_status']=='complete' and any(p['scores'].get(r) is None for r in contract['candidate']['roles']) for p in pairs):return {'status':'unknown','qualified':None}
    qualifying=any(candidate(p,arm,contract) for p in pairs);mechanical=arm in ['nonheritable','frozen'] and any(functional_pair(p,contract) for p in pairs)
    return {'status':'mechanical_control_violation' if mechanical else 'complete','qualified':qualifying}
def analyze_outcomes(outcomes,contract):
    n=outcomes['blocks'];arms=contract['design']['arms'];entries=outcomes['worlds']
    if n!=statistics(contract)['selected_blocks'] or type(n) is not int:raise ValueError('fixed prospective block count')
    if any(r['qualified'] is not None and type(r['qualified']) is not bool for r in entries):raise ValueError('typed world qualification')
    if len(entries)!=n*len(arms) or {(r['block'],r['arm']) for r in entries}!={(i,a) for i in range(n) for a in arms}:raise ValueError('all fixed independent world slots required')
    if any(r['status']=='mechanical_control_violation' or (r['qualified'] is True and r['arm'] in ['nonheritable','frozen']) for r in entries):return {'status':'invalid_mechanical_control','cumulative_evolution':False,'self_modification_advantage':False,**{f:False for f in FLAGS}}
    if any(r['status']!='complete' or r['qualified'] is None for r in entries):return {'status':'inconclusive_operational_or_measurement_failure','cumulative_evolution':False,'self_modification_advantage':False,'denominator':n,'world_denominator':len(entries),'unknown_worlds':sum(r['status']!='complete' or r['qualified'] is None for r in entries),**{f:False for f in FLAGS}}
    values={(r['block'],r['arm']):r['qualified'] for r in entries};x=sum(values[i,'endogenous'] for i in range(n));b=sum(values[i,'endogenous'] and not values[i,'external'] for i in range(n));c=sum(not values[i,'endogenous'] and values[i,'external'] for i in range(n));d=b+c;f=ratio(contract['inference']['prevalence_floor']);den=f.denominator**n;num=sum(math.comb(n,j)*f.numerator**j*(f.denominator-f.numerator)**(n-j) for j in range(x,n+1));p=Fraction(num,den);pp=Fraction(sum(math.comb(d,j) for j in range(b,d+1)),2**d);a=ratio(contract['inference']['prevalence_alpha']);pa=ratio(contract['inference']['advantage_alpha']);effect=ratio(contract['inference']['minimum_observed_paired_advantage']);primary=p<=a;advantage=primary and pp<=pa and Fraction(b-c,n)>=effect
    return {'status':'cumulative_with_selfmod_advantage' if advantage else 'cumulative_without_demonstrated_selfmod_advantage' if primary else 'valid_negative','cumulative_evolution':primary,'self_modification_advantage':advantage,'denominator':n,'world_denominator':len(entries),'endogenous_successes':x,'external_successes':sum(values[i,'external'] for i in range(n)),'endogenous_only':b,'external_only':c,'prevalence_p':probability(num,den),'advantage_p':probability(pp.numerator,pp.denominator),**{f:False for f in FLAGS}}
def plan(contract,evidence):
    stats=statistics(contract);n=stats['selected_blocks'];proposed=panel(contract,n) if n else None;cost=resources(contract,n,evidence) if n else None
    return {'profile':contract['profile'],'engineering_planning_complete':True,'selected_blocks':n,'statistics':stats,'panel':proposed,'resources':cost,'campaign_budget_approved':False,'whole_campaign_feasibility_pass':False,'campaign_preregistration_complete':False,'remaining':['scientific campaign budget unapproved','generalized native world/measurement/journal adapter absent','two padded interaction roles not admitted by current native request schema','time/storage bounds for full population/program and complete intervention denominator unmeasured','independent confirmation environment policy unresolved'],'native_rewrites':0,**{f:False for f in FLAGS}}
if __name__=='__main__':
    c,e=[json.loads(Path(p).read_text()) for p in sys.argv[1:3]];result=plan(c,e)
    with Path(sys.argv[3]).open('x') as out:json.dump(result,out,indent=2,sort_keys=True);out.write('\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ['statistics','panel','resources']},indent=2))
