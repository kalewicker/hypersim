#!/usr/bin/env python3
"""Post-hoc read-only mapping of a sealed sampler/Engine archive. No Engine/RNG draws."""
import argparse
import collections
import json
import math
from pathlib import Path
import shutil
import struct
import time
import audit_ere_complete_direction as d

ROOT=Path(__file__).resolve().parents[1]
INPUT_SEAL='ba88c696014fa42ca612f4c20baadb1818cc6c5757fd41594c87a1e4bc1a185a'
PROFILE='E2-sampler-reward-map-readonly-v1'
GROUPS=[('parent',0,168),('operator',168,181),('role',181,183)]
NATIVE=['crates/hg-sim-core/src/ere/valid_grammar.rs','crates/hg-sim-core/src/ere/signal.rs',
        'crates/hg-sim-core/src/ere/semantic.rs','crates/hg-sim-core/src/ere/coherent.rs',
        'crates/hg-meta/src/program.rs','crates/hg-meta/src/schema.rs','crates/hg-train/src/optim.rs']
CHECKERS=['scripts/audit_ere_complete_direction.py','scripts/audit_ere_complete_minibatch.py',
          'scripts/audit_ere_complete_feasibility_training.py','scripts/audit_ere_complete_confirmation.py',
          'scripts/audit_ere_complete_admission.py','scripts/audit_ere_complete_scaling.py']

def f32(x):return struct.unpack('<f',struct.pack('<f',x))[0]

def softmax(theta,support):
    assert support and len(set(support))==len(support) and all(type(i) is int and 0<=i<len(theta) for i in support)
    assert all(math.isfinite(x) for x in theta)
    m=max(theta[i] for i in support);ws=[math.exp(theta[i]-m) for i in support];z=sum(ws)
    return {i:w/z for i,w in zip(support,ws)}

def unpack(hex_bytes,tag):
    raw=bytes.fromhex(hex_bytes)
    if not raw.startswith(tag):
        assert raw.startswith(b'E1RCF2') and len(raw)>10
        return [raw]
    pos=len(tag);assert len(raw)>=pos+4;n=int.from_bytes(raw[pos:pos+4],'little');pos+=4;parts=[]
    assert n in (1,2)
    for _ in range(n):
        assert pos+4<=len(raw);size=int.from_bytes(raw[pos:pos+4],'little');pos+=4
        assert size>10 and pos+size<=len(raw);parts.append(raw[pos:pos+size]);pos+=size
    assert pos==len(raw) and all(p.startswith(b'E1RCF2') for p in parts)
    return parts

def eligible(rcf):
    # Program RCF contains executable canonical candidates. Zeroing priority
    # matches signal::structural; a second distinct body implies an active,
    # structurally distinct proposal in this two-allele complete grammar.
    parts=unpack(rcf,b'E2-PROGRAM\0');bodies={v[:6]+b'\0'*4+v[10:] for v in parts}
    return len(bodies)==2

def catalogue_model(cat):
    assert cat['params']==183 and len(cat['parents'])==168 and cat['engine_steps']==0
    leaves={};groups={};counts=collections.Counter();unique=set()
    for p,parent in enumerate(cat['parents']):
        assert parent['parent']==p and len(parent['groups'])==13
        for op,g in enumerate(parent['groups']):
            assert g['operator']==op and len(g['roles'])==2
            assert g['role_support']==[r for r,role in enumerate(g['roles']) if role['productions']]
            for role,rs in enumerate(g['roles']):
                assert rs['role']==role
                entries=[]
                for i,leaf in enumerate(rs['productions']):
                    assert leaf['index']==i
                    active=True if op==11 else False if op==12 else bool(p%2)
                    credit=eligible(leaf['rcf']);assert not credit or active
                    entry=dict(active=active,eligible=credit,signature=leaf['execution'],request=leaf['request'])
                    leaves[(p,op,role,i)]=entry;entries.append(entry);unique.add(leaf['execution'])
                    counts['leaves']+=1;counts['active_leaves']+=active;counts['eligible_leaves']+=credit
                if entries:
                    groups[(p,op,role)]=dict(count=len(entries),active=sum(x['active'] for x in entries)/len(entries),eligible=sum(x['eligible'] for x in entries)/len(entries),
                        signatures=dict(collections.Counter(x['signature'] for x in entries)))
        assert parent['operator_support']==[g['operator'] for g in parent['groups'] if g['role_support']]
    counts['unique_execution_signatures']=len(unique);counts['parents']=168
    counts['parent_operator_groups']=sum(bool(g['role_support']) for p in cat['parents'] for g in p['groups'])
    counts['parent_operator_role_groups']=len(groups)
    counts['single_role_groups']=sum(len(g['role_support'])==1 for p in cat['parents'] for g in p['groups'])
    counts['two_role_groups']=sum(len(g['role_support'])==2 for p in cat['parents'] for g in p['groups'])
    return leaves,groups,dict(counts)

def probabilities(cat,theta,groups):
    assert len(theta)==183
    p=softmax(theta[:168],list(range(168)));op={};role={};group_mass={};active=credit=0.
    op_marg=[0.]*13;role_marg=[0.]*2
    for parent in range(168):
        op[parent]=softmax(theta[168:181],cat['parents'][parent]['operator_support'])
        for operator,po in op[parent].items():
            role[(parent,operator)]=softmax(theta[181:],cat['parents'][parent]['groups'][operator]['role_support'])
            op_marg[operator]+=p[parent]*po
            for r,pr in role[(parent,operator)].items():
                key=(parent,operator,r);mass=p[parent]*po*pr;group_mass[key]=mass
                active+=mass*groups[key]['active'];credit+=mass*groups[key]['eligible'];role_marg[r]+=mass
    assert math.isclose(sum(group_mass.values()),1.,abs_tol=1e-12)
    return dict(parent=p,operator=op,role=role,group_mass=group_mass,active_mass=active,eligible_mass=credit,
        operator_marginal=op_marg,role_marginal=role_marg,parent_active_mass=sum(v for i,v in p.items() if i%2))

def expectation_gradient(prob,groups,feature):
    total=prob[feature+'_mass'];g=[0.]*183
    for p,pp in prob['parent'].items():
        parent_q=0.;op_q={}
        for op,po in prob['operator'][p].items():
            q=sum(pr*groups[(p,op,r)][feature] for r,pr in prob['role'][(p,op)].items());op_q[op]=q;parent_q+=po*q
            for r,pr in prob['role'][(p,op)].items():g[181+r]+=pp*po*pr*(groups[(p,op,r)][feature]-q)
        g[p]=pp*(parent_q-total)
        for op,po in prob['operator'][p].items():g[168+op]+=pp*po*(op_q[op]-parent_q)
    assert all(abs(sum(g[start:end]))<1e-12 for _,start,end in GROUPS)
    return g

def parent_cdf_switch(a,b):
    # Exact overlap for the SAME uniform draw in the first inverse-CDF pick.
    aa=bb=overlap=0.
    for i in range(len(a)):
        ae=aa+a[i];be=bb+b[i];overlap+=max(0.,min(ae,be)-max(aa,bb));aa=ae;bb=be
    return max(0.,min(1.,1.-overlap))

def distance(a,b):return sum(abs(x-y) for x,y in zip(a,b))/2

def execution_mass(prob,groups):
    mass=collections.defaultdict(float)
    for key,p in prob['group_mass'].items():
        for signature,n in groups[key]['signatures'].items():mass[signature]+=p*n/groups[key]['count']
    assert math.isclose(sum(mass.values()),1.,abs_tol=1e-12)
    return mass

def probability_comparison(cat,groups,theta,forward):
    theta=[f32(x) for x in theta];forward=[f32(x) for x in forward]
    base=probabilities(cat,theta,groups);post=probabilities(cat,forward,groups);delta=[b-a for a,b in zip(theta,forward)]
    gradients={feature:expectation_gradient(base,groups,feature) for feature in ['active','eligible']}
    group_norms={name:math.sqrt(sum(x*x for x in delta[start:end])) for name,start,end in GROUPS}
    energies={name:v*v/sum(x*x for x in delta) if any(delta) else 0. for name,v in group_norms.items()}
    base_exec=execution_mass(base,groups);post_exec=execution_mass(post,groups)
    return dict(base_active_mass=base['active_mass'],forward_active_mass=post['active_mass'],active_mass_delta=post['active_mass']-base['active_mass'],
        base_eligible_mass=base['eligible_mass'],forward_eligible_mass=post['eligible_mass'],eligible_mass_delta=post['eligible_mass']-base['eligible_mass'],
        parent_tv=distance(list(base['parent'].values()),list(post['parent'].values())),
        first_parent_cdf_switch_probability=parent_cdf_switch(list(base['parent'].values()),list(post['parent'].values())),
        leaf_selection_tv=sum(abs(base['group_mass'][key]-post['group_mass'][key]) for key in groups)/2,
        executable_signature_tv=sum(abs(base_exec[key]-post_exec[key]) for key in base_exec)/2,
        operator_marginal_tv=distance(base['operator_marginal'],post['operator_marginal']),role_marginal_tv=distance(base['role_marginal'],post['role_marginal']),
        group_step_norms=group_norms,group_step_energy_fractions=energies,
        active_gradient_dot_step=sum(a*b for a,b in zip(gradients['active'],delta)),
        eligible_gradient_dot_step=sum(a*b for a,b in zip(gradients['eligible'],delta)))

def summarize_comparisons(rows):
    keys=['active_mass_delta','eligible_mass_delta','parent_tv','first_parent_cdf_switch_probability','leaf_selection_tv','executable_signature_tv','operator_marginal_tv','role_marginal_tv']
    result={k:dict(mean=sum(r[k] for r in rows)/len(rows),minimum=min(r[k] for r in rows),maximum=max(r[k] for r in rows),
        positive=sum(r[k]>0 for r in rows),negative=sum(r[k]<0 for r in rows),zero=sum(r[k]==0 for r in rows)) for k in keys}
    result['mean_group_step_energy_fractions']={name:sum(r['group_step_energy_fractions'][name] for r in rows)/len(rows) for name,_,_ in GROUPS}
    return result

def inspect_rows(inp,cat,leaves):
    native,h=d.c.helpers(Path(d.a.load(inp/'parent_receipt.json')['parent_archive']))
    plan=d.schedule(d.design());stage_counts=collections.defaultdict(collections.Counter);small={};cases=[];observed=set();valid_by_leaf=collections.defaultdict(set)
    for q in plan:
        row=d.a.load(inp/'cells'/(q['id']+'.json'));d.b.cell(row,q,cat,native,h)
        e=row['emission'];v=row['evaluation'];key=(e['parent'],e['operator'],0 if e['role']=='Core' else 1,e['production_index']);leaf=leaves[key];observed.add(key)
        assert e['classes']['active']==leaf['active'] and bool(v['credited_rule_ids'])==leaf['eligible']
        assert e['signature']==leaf['signature']
        s=stage_counts[q['stage']];s['cells']+=1;s['valid' if row['status']=='Complete' else 'caps']+=1
        s['active']+=leaf['active'];s['credit_eligible']+=leaf['eligible'];s['credited_fires_positive']+=v['proposal_fires']>0;s['credited_fires_at_least_two']+=v['proposal_fires']>=2
        if v['score'] is not None:
            s['positive_score']+=v['score']>0;s['zero_score']+=v['score']==0
            if v['score']==0:
                label='inactive' if not leaf['active'] else 'active_structurally_identical' if not leaf['eligible'] else 'eligible_no_fires' if v['proposal_fires']==0 else 'eligible_one_fire'
                s['zero_'+label]+=1
            valid_by_leaf[key].add(v['score'])
        if q['stage']=='validation':
            small[(q['block'],q['context'],q['draw'],q['arm'],q['physics_index'])]=dict(emission=e,score=v['score'],utility=row['constraint_utility'],fires=v['proposal_fires'],credit=v['credited_rule_ids'],id=q['id'])
    changes=collections.defaultdict(collections.Counter);all_equal=True;observed_context=[[0.]*5 for _ in range(2)]
    for r in range(8):
        for ctx in range(2):
            for draw in range(16):
                panels=[]
                for arm in range(5):
                    rows=[small[(r,ctx,draw,arm,p)] for p in range(4)];all_equal &= len({v['score'] for v in rows})==1
                    panels.append((rows[0],sum(v['utility'] for v in rows)/4))
                base,bu=panels[0]
                for arm,(row,u) in enumerate(panels):
                    observed_context[ctx][arm]+=u/128
                    if arm==0:continue
                    c=changes[(ctx,arm)];c['draws']+=1;changed=row['emission']['signature']!=base['emission']['signature'];c['signature_changed']+=changed
                    c['parent_changed']+=row['emission']['parent']!=base['emission']['parent'];c['operator_changed']+=row['emission']['operator']!=base['emission']['operator'];c['role_changed']+=row['emission']['role']!=base['emission']['role']
                    c['active_changed']+=row['emission']['classes']['active']!=base['emission']['classes']['active'];c['utility_changed']+=u!=bu;c['wins']+=u>bu;c['losses']+=u<bu
                    if changed or u!=bu:
                        def info(v):return {k:v[k] for k in ['id','score','fires','credit']}|{k:v['emission'][k] for k in ['parent','operator','role','production_index']}|dict(active=v['emission']['classes']['active'])
                        cases.append(dict(block=r,context=ctx,draw=draw,arm=ARMS[arm],utility_delta=u-bu,base=info(base),post=info(row)))
    return dict(stage_counts={k:dict(v) for k,v in stage_counts.items()},catalogue_observed_unique_leaves=len(observed),
        observed_valid_unique_leaves=len(valid_by_leaf),observed_leaf_score_variation=sum(len(v)>1 for v in valid_by_leaf.values()),
        all_validation_physics_scores_equal=all_equal,context_utility_arm_means=observed_context,
        validation_comparisons=[dict(context=ctx,arm=ARMS[arm],**dict(v)) for (ctx,arm),v in sorted(changes.items())],cases=cases),observed

ARMS=d.ARMS

def analyze(inp):
    d.a.verify(inp,expected=INPUT_SEAL,count=7686)
    old=d.a.load(inp/'audit.json');assert old['audit_pass'] and old['tamper_rejections']==40 and old['support_direction'] is False
    snap=d.a.load(inp/'snapshot.json')
    for name in NATIVE:assert d.a.digest(inp/'source_snapshots'/name)==snap['sources'][name]
    for name in CHECKERS:
        assert d.a.digest(inp/'source_snapshots'/name)==d.a.digest(ROOT/name)==snap['sources'][name]
    parent=Path(d.a.load(inp/'parent_receipt.json')['parent_archive']);d.a.verify(parent,expected=d.c.PARENT,count=1690)
    runtime=Path(d.a.load(parent/'manifest.json')['receipt_paths']['runtime']);d.a.verify(runtime,expected=d.a.SPECS['runtime'][1],count=d.a.SPECS['runtime'][2])
    cat=d.a.load(inp/'catalogue.json');assert d.a.digest(inp/'catalogue.json')==d.a.CATALOGUE
    leaves,groups,counts=catalogue_model(cat);obs,observed=inspect_rows(inp,cat,leaves);contexts=d.a.load(inp/'contexts.json');rows=[]
    base_features=[]
    for ctx in range(2):
        prob=probabilities(cat,contexts[ctx],groups)
        seen_mass=sum(prob['group_mass'][(p,o,r)]/groups[(p,o,r)]['count'] for p,o,r,i in observed)
        base_features.append(dict(context=ctx,active_mass=prob['active_mass'],credit_eligible_mass=prob['eligible_mass'],parent_active_mass=prob['parent_active_mass'],
            observed_leaf_selection_mass=seen_mass,operator_marginal=prob['operator_marginal'],role_marginal=prob['role_marginal'],
            eligible_gradient_group_norms={name:math.sqrt(sum(v*v for v in expectation_gradient(prob,groups,'eligible')[start:end])) for name,start,end in GROUPS}))
    for r in range(8):
        for ctx in range(2):
            vectors=d.a.load(inp/'directions'/f'r{r}-c{ctx}.json')['parameters']
            for arm in [1,3]:rows.append(dict(block=r,context=ctx,arm=ARMS[arm],**probability_comparison(cat,groups,contexts[ctx],vectors[arm])))
    # All rows/caps remain in their original archive/statistical denominator.
    d.a.verify(inp,expected=INPUT_SEAL,count=7686)
    d.a.verify(parent,expected=d.c.PARENT,count=1690)
    d.a.verify(runtime,expected=d.a.SPECS['runtime'][1],count=d.a.SPECS['runtime'][2])
    return dict(profile=PROFILE,classification='post_hoc_read_only_diagnostic',scientific_campaign=False,
        native_steps_run=0,grammar_compilations=0,training_updates=0,input_archive=str(inp.resolve()),input_seal=INPUT_SEAL,
        input_sources={name:snap['sources'][name] for name in NATIVE},catalogue=counts,base_probability_features=base_features,
        checker_sources={name:snap['sources'][name] for name in CHECKERS},
        helper_archives=[dict(path=str(parent),seal=d.c.PARENT,entries=1690),dict(path=str(runtime),seal=d.a.SPECS['runtime'][1],entries=d.a.SPECS['runtime'][2])],
        forward_probability_comparisons=rows,forward_probability_summary=summarize_comparisons(rows),
        context_probability_summaries=[summarize_comparisons([r for r in rows if r['context']==ctx]) for ctx in range(2)],
        receipt_diagnostics=obs,exact_native_expected_reward_available=False,
        limitations=['Activation/credit eligibility are exact sampler features, not expected native reward or cap probability.',
            'No new RNG samples, grammar compilation or Engine work; no native counterfactual replay.',
            'The CDF switch calculation concerns the first parent draw, not the full hierarchical emission coupling.',
            'Historical draw comparisons remain post-hoc and are not independent scientific tests.'])

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('input',type=Path);parser.add_argument('output',type=Path);args=parser.parse_args()
    assert not args.output.exists(),'fresh diagnostic output required';start=time.monotonic();result=analyze(args.input)
    args.output.mkdir();sources={}
    for name in ['scripts/analyze_ere_sampler_reward.py','scripts/test_analyze_ere_sampler_reward.py']:
        source=ROOT/name;target=args.output/'source_snapshots'/name;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,target);sources[name]=d.a.digest(source)
    (args.output/'analysis.json').write_text(json.dumps(result,indent=2))
    manifest=dict(profile=PROFILE,status='complete',classification='post_hoc_read_only_diagnostic',input_seal=INPUT_SEAL,
        native_steps_run=0,grammar_compilations=0,training_updates=0,elapsed_seconds=time.monotonic()-start,
        diagnostic_sources=sources,analysis_sha256=d.a.digest(args.output/'analysis.json'))
    (args.output/'manifest.json').write_text(json.dumps(manifest,indent=2));d.a.seal(args.output)
    print(json.dumps(dict(status='complete',native_steps_run=0,catalogue=result['catalogue'],base_probability_features=result['base_probability_features'],
        forward_probability_summary=result['forward_probability_summary'],receipt_stage_counts=result['receipt_diagnostics']['stage_counts']),indent=2))
