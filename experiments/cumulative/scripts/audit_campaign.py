"""Independent exact multinomial convolution and resource/custody audit; zero native."""
import hashlib,json,math,sys
from collections import defaultdict
from pathlib import Path
from audit_readiness import require,FALSE_FLAGS
from audit_durability import recover,sha

def evidence(package):
    root=package/'evidence';seal=json.loads((root/'parent_output_sha256.json').read_text());contract=json.loads((package/'contract.json').read_text())
    require(sha((root/'parent_output_sha256.json').read_bytes())==contract['parent']['seal_sha256'],'known parent root')
    copies=json.loads((root/'copy_map.json').read_text())
    for rel,original in copies.items():require(sha((root/rel).read_bytes())==seal[original],'authenticated evidence copy '+rel)
    worlds=recover(root/'world_complete',json.loads((root/'world_input.json').read_text()),sha((root/'world_input.json').read_bytes()))
    assays=recover(root/'assay_complete',json.loads((root/'assay_input.json').read_text()),sha((root/'assay_input.json').read_bytes()))
    require(worlds['status']==assays['status']=='complete' and worlds['lower']==151 and assays['lower']==324,'complete saved development denominator')
    return {'world_journal_bytes':sum(p.stat().st_size for p in (root/'world_complete').iterdir()),'assay_journal_bytes':sum(p.stat().st_size for p in (root/'assay_complete').iterdir()),'world_cases':len(worlds['cases']),'assays':len(assays['assays']),'average_world_ticks':sum(len(c['rows']) for c in worlds['cases'])/len(worlds['cases']),'world_command_seconds':json.loads((root/'world_command.json').read_text())['seconds'],'assay_command_seconds':json.loads((root/'assay_command.json').read_text())['seconds'],'native_rewrites':0,'development_worlds_are_not_independent_replicates':True,'timing_and_size_are_observations_not_future_upper_bounds':True}

def independent_statistics(c):
    i=c['inference'];maximum=c['design']['replicate_search_max'];alpha=i['prevalence_alpha'];paired_alpha=i['advantage_alpha'];floor=i['prevalence_floor'];effect=i['minimum_observed_paired_advantage'];target=i['joint_power_target'];critical={};null=[1];thresholds={}
    for n in range(1,maximum+1):
        next_null=[0]*(n+1)
        for k,v in enumerate(null):next_null[k]+=v*(floor['denominator']-floor['numerator']);next_null[k+1]+=v*floor['numerator']
        null=next_null;running=0;selected=n+1
        for k in range(n,-1,-1):
            running+=null[k]
            if running*alpha['denominator']<=alpha['numerator']*floor['denominator']**n:selected=k
        critical[n]=selected
        for d in range(n+1):
            coefficients=[1]
            for j in range(d):coefficients.append(coefficients[-1]*(d-j)//(j+1))
            tail=2**d
            for b in range(d+1):
                thresholds[n,d,b]=(2*b-d)*effect['denominator']>=n*effect['numerator'] and tail*paired_alpha['denominator']<=paired_alpha['numerator']*2**d;tail-=coefficients[b]
    result={}
    for scenario in i['scenarios']:
        w=scenario['weights'];total=sum(w.values());dist={(0,0,0):1}
        for n in range(1,maximum+1):
            new=defaultdict(int)
            for (k,b,x),v in dist.items():
                for key,weight in [((k+1,b,x),w['both']),((k+1,b+1,x),w['endogenous_only']),((k,b,x+1),w['external_only']),((k,b,x),w['neither'])]:
                    if weight:new[key]+=v*weight
            dist=new;require(sum(dist.values())==total**n,'complete multinomial probability mass');prevalence=advantage=joint=0
            for (k,b,x),v in dist.items():
                primary=k>=critical[n];paired=thresholds[n,b+x,b]
                if primary:prevalence+=v
                if paired:advantage+=v
                if primary and paired:joint+=v
            result[scenario['name'],n]={'prevalence_successes_required':critical[n],'prevalence_power':(prevalence,total**n),'advantage_power':(advantage,total**n),'joint_power':(joint,total**n),'joint_power_target_met':joint*target['denominator']>=total**n*target['numerator']}
    selected=next((n for n in range(1,maximum+1) if result['target',n]['joint_power_target_met']),None)
    return result,selected

def audit(package):
    c=json.loads((package/'contract.json').read_text());r=json.loads((package/'plan.json').read_text());e=evidence(package);require(e==json.loads((package/'evidence_summary.json').read_text()),'independent evidence metrics')
    for obj in [c,r,r['panel']]:
        for f in FALSE_FLAGS:require(obj[f] is False,'planning/science admission')
    require(r['native_rewrites']==0 and r['whole_campaign_feasibility_pass'] is False and r['campaign_preregistration_complete'] is False and r['campaign_budget_approved'] is False,'no funded/operational campaign claim')
    expected,n=independent_statistics(c);require(n==r['selected_blocks']==r['statistics']['selected_blocks'] and r['statistics']['selection_uses_scientific_outcomes'] is False,'prospective exact joint power choice')
    require(len(r['statistics']['rows'])==len(expected),'all sensitivity cells retained')
    for row in r['statistics']['rows']:
        v=expected[row['scenario'],row['blocks']]
        for k in ['prevalence_power','advantage_power','joint_power']:
            num,den=v[k];require(row[k]=={'numerator':num,'denominator':den,'value':num/den},'independent exact probability '+k)
        require(row['prevalence_successes_required']==v['prevalence_successes_required'] and row['joint_power_target_met']==v['joint_power_target_met'],'exact decision threshold')
    d=c['design'];w=d['world'];worlds=r['panel']['worlds'];require(len(worlds)==n*len(d['arms']) and [(s['block'],s['arm']) for s in worlds]==[(i,a) for i in range(n) for a in d['arms']],'complete ordered independent blocks')
    seedset=set()
    for i in range(n):
        seed=int(hashlib.sha256((d['seed_domain']+'/'+str(i)).encode()).hexdigest()[:16],16);require(seed not in seedset and seed not in d['development_seeds'],'new unreplaced seed');seedset.add(seed)
        for j,arm in enumerate(d['arms']):
            x=worlds[i*len(d['arms'])+j];require(x=={'block':i,'arm':arm,'seed':seed,'world_id':f'block-{i:03}/{arm}','programs':[{'stages':w['founder_stages'],'enabled':int(arm in ['endogenous','nonheritable']),'mask':31 if arm in ['endogenous','nonheritable'] else 0} for _ in range(w['founders'])],'capital':w['capital'],'reservoir':w['reservoir'],'parcels_per_tick':w['parcels_per_tick'],'sites':w['sites'],'ticks':w['ticks']},'matched arm founders and exact proposed panel')
    samples=n*len(d['arms'])*len(d['census_ticks'])*d['slots_per_census'];require(samples==r['panel']['sampling_slots'],'sampling denominator')
    # Separate live-site edit bound from retained-cell unknown reservation.
    parcel_total=min(w['reservoir']//16,w['ticks']*w['parcels_per_tick']);reaction_bound=parcel_total*(4+1);edit_bound=w['sites']*((w['ticks']+d['world']['edit_every_ticks']-1)//d['world']['edit_every_ticks']);birth_bound=w['retained_cells']-w['founders'];native={'endogenous':reaction_bound+edit_bound,'external':reaction_bound+birth_bound,'nonheritable':reaction_bound+edit_bound,'frozen':reaction_bound};reserves={a:5*w['parcels_per_tick']+w['retained_cells']+(w['sites'] if a=='external' else 0) for a in d['arms']};novel=set(w['executable_stages'])-set(w['founder_stages']);pairs=len(novel)*(len(novel)-1)//2;per_world=len(d['census_ticks'])*d['slots_per_census']*(3+pairs*len(c['candidate']['roles']));requests=per_world*len(worlds)*d['diagnostic_contexts'];world_work=n*sum(native.values());reserved=n*sum(reserves.values());assay_work=requests*4*5;cost=r['resources']
    require(cost['native_upper_per_arm']==native and cost['single_unpaired_intent_upper_per_arm']==reserves and cost['world_native_upper']==world_work and cost['one_unknown_action_reserve_per_world']==reserved and cost['maximum_novel_stages_per_path']==len(novel) and cost['maximum_pairs_per_sample']==pairs and cost['planned_slots']==samples and cost['uncached_request_upper_per_world']==per_world and cost['uncached_assay_request_upper']==requests and cost['assay_native_upper']==assay_work and cost['sufficient_conservative_native_reservation']==world_work+reserved+assay_work,'independent structural work and complete assay envelope')
    require(cost['guaranteed_rewrites_within_existing_envelope']==(world_work+reserved+assay_work<=c['existing_envelope']['native_rewrites']) and cost['full_request_bound_within_current_single_request_schema']==(requests<=c['existing_envelope']['assays_per_native_request']),'no budget or denominator truncation')
    p=cost['observed_scale_projection'];require(p['certified_storage_or_time_upper_bound'] is False and p['world_size_population_and_program_not_matched'] is True,'no extrapolation promoted to a bound')
    require(p['world_journal_bytes']==math.ceil(e['world_journal_bytes']/e['world_cases']*len(worlds)*w['ticks']/e['average_world_ticks']) and p['assay_journal_bytes']==math.ceil(e['assay_journal_bytes']/e['assays']*requests) and p['world_command_seconds']==e['world_command_seconds']/e['world_cases']*len(worlds)*w['ticks']/e['average_world_ticks'] and p['assay_command_seconds']==e['assay_command_seconds']/e['assays']*requests,'authenticated observed-scale projection')
    return {'profile':c['profile'],'engineering_planning_pass':True,'independent_exact_multinomial_convolution':True,'selected_blocks':n,'proposed_worlds':len(worlds),'prospective_slots':samples,'uncached_assay_request_upper':requests,'sufficient_conservative_native_reservation':world_work+reserved+assay_work,'statistical_scenario_cells':len(expected),'native_rewrites':0,'whole_campaign_feasibility_pass':False,'campaign_budget_approved':False,'campaign_preregistration_complete':False,**{f:False for f in FALSE_FLAGS}}
if __name__=='__main__':
    print(json.dumps(audit(Path(sys.argv[1])),indent=2,sort_keys=True))
