#!/usr/bin/env python3
"""Frozen ERE scaling provenance/accounting audit, not physics replay."""
import argparse
import copy
import json
import math
from pathlib import Path
import shutil
import sys
import tempfile
import audit_ere_complete_admission as a

sys.dont_write_bytecode=True
PROFILE='E2-complete-scaling-v2'
SIGNAL='crates/hg-sim-core/src/ere/signal.rs'
ENGINEERING='a988f131bebbf6c2e14393bf6c2226737908c628ca7111a1ca3ff1c66b4657fd'
EXTRA=a.EXTRA | {'crates/hg-train/src/ere/scaling.rs','docs/ere_complete_scaling_trial_v2.md',
    'configs/ere_complete_scaling_v2.json','scripts/audit_ere_complete_scaling.py','scripts/test_audit_ere_complete_scaling.py'}

def design():
    return dict(profile=PROFILE,purpose='fixed_panel_scaling',horizons=[100,1000,10000],physics_seeds=[1361,1471],
        emission_seeds=list(range(0xE653000101,0xE653000107)),control_parents=[4,5],random_draws=4,parameters=[0.0]*183,work=50000,
        results=50000,wall_seconds=1800,cell_wall_seconds=120,output_cap_bytes=1024**3,
        optimizer_generations=0,held_out_evaluations=0)

def schedule(d):
    return [dict(id=f'p{p}-s{s}-h{h}',production=p,kind='draw' if p<4 else 'active' if p==4 else 'dormant',
        emission_seed=seed,physics_seed=s,horizon=h) for p,seed in enumerate(d['emission_seeds']) for s in d['physics_seeds'] for h in d['horizons']]

def bridge(old):
    marker=') -> Result<Value, String> {\n    let Some(p) = &emission.program else {'
    entry=''' ) -> Result<Value, String> {
    let mut result = evaluate_with_horizon(emission, seed, deadline, work, results, HORIZON)?;
    result.as_object_mut().unwrap().remove("horizon");
    if let Some(steps) = result["steps"].as_array_mut() {
        for step in steps {
            step.as_object_mut().unwrap().remove("nodes");
            step.as_object_mut().unwrap().remove("edges");
        }
    }
    Ok(result)
}
/// Versioned scaling API; legacy wrappers retain exact 100-step receipt shape.
pub fn evaluate_with_horizon(
    emission: &Emission,
    seed: u64,
    deadline: Instant,
    work: usize,
    results: usize,
    horizon: usize,
) -> Result<Value, String> {
    if !(2..=10_000).contains(&horizon) {
        return Err("scaling horizon must be in 2..=10000".into());
    }
    let Some(p) = &emission.program else {'''.lstrip()
    assert old.count(marker)==1
    result=old.replace(marker,entry)
    for before,after,count in [
        ('for step in 1..=HORIZON','for step in 1..=horizon',1),
        ('steps.len() == HORIZON','steps.len() == horizon',1),
        ('/ (HORIZON as f64).ln()','/ (horizon as f64).ln()',1),
        ('json!({"physics_seed":seed,"score":','json!({"horizon":horizon,"physics_seed":seed,"score":',2),
        ('steps.push(json!({"step":step','steps.push(json!({"nodes":s.node_count(),"edges":s.edge_count(),"step":step',1)]:
        assert result.count(before)==count
        result=result.replace(before,after)
    return result

def cell(row,e,c,h):
    q=row['schedule']; horizon=q['horizon']; v=row['evaluation']
    h.check_membership(c,e,full=True)
    assert e['emission_seed']==q['emission_seed'] and e['kernel']['training_admitted'] is False
    assert e['kernel']['advancing_engine_steps']==0 and not e['kernel']['codec_reencoded'] and not e['kernel']['metadata_recounted']
    if row['status']=='NotRun':
        assert v is None and row['reason']=='global_resource_ceiling'
        return 0,0,False
    assert 0<=row['elapsed_seconds']
    if row['status']=='Incomplete':
        assert v is None and row['partial_steps_unknown'] is True and isinstance(row['reason'],str)
        return 0,0,False
    assert row['status'] in ('Complete','Null') and v['horizon']==horizon and v['physics_seed']==q['physics_seed']
    assert v['runtime_match_budget']==dict(policy='Pooled',work=50000,results=50000)
    classes=e['classes']
    ids=[i for i,b in enumerate(v['candidate_execution']) if classes['active'] and classes['core']!=classes['proposal'] and b==classes['proposal_execution']]
    assert ids==v['credited_rule_ids']
    steps=v['steps']; fires=0; failure=None
    assert 0<len(steps)<=horizon and v['attempted_steps']==len(steps) and v['advancing_steps']==sum(s['advanced'] for s in steps)
    for i,s in enumerate(steps,1):
        assert s['step']==i and len(s['rule_ids'])<=1
        assert type(s['advanced']) is int and s['advanced'] in (0,1)
        assert type(s['nodes']) is int and type(s['edges']) is int and s['nodes']>=0 and s['edges']>=0
        assert all(type(n) is int and 0<=n<len(v['candidate_execution']) for n in s['rule_ids'])
        assert len(s['world_hash'])==16 and all(x in '0123456789abcdef' for x in s['world_hash'])
        h.check_allowances(s['usage'],50000,50000)
        assert s['usage']['truncated']==any(r['truncated'] for r in s['usage']['rules'])
        cap=s['usage']['truncated'] or s['nodes']>2000 or s['edges']>4000
        assert s['cap']==cap
        fires+=sum(n in ids for n in s['rule_ids'])
        if cap or not s['valid_family'] or s['advanced']!=1:
            assert i==len(steps)
            failure='resource_cap' if cap else 'invalid_family' if not s['valid_family'] else 'nonadvancing'
    assert v['proposal_fires']==fires
    full=len(steps)==horizon and failure is None
    assert full or failure is not None
    assert (v['score'] is not None)==full and v['reason']==failure
    assert (row['status']=='Complete')==full
    h.close(v['score'],math.log1p(max(fires-1,0))/math.log(horizon) if full else None)
    return len(steps),v['advancing_steps'],full

def summary(rows,d):
    groups={str(h):dict(scheduled=12,complete=0,null=0,incomplete=0,not_run=0) for h in d['horizons']}
    attempts=advances=complete=terminal=0
    for r in rows:
        groups[str(r['schedule']['horizon'])][{'Complete':'complete','Null':'null','Incomplete':'incomplete','NotRun':'not_run'}[r['status']]]+=1
        if r['evaluation'] is not None:
            terminal+=1;attempts+=r['evaluation']['attempted_steps'];advances+=r['evaluation']['advancing_steps']
        complete+=r['status']=='Complete'
    prefixes=True; controls=True; triples=[]
    for p in range(6):
        for seed in d['physics_seeds']:
            trio=[r for r in rows if r['schedule']['production']==p and r['schedule']['physics_seed']==seed]
            equal=True
            for short,long in zip(trio,trio[1:]):
                x,y=short['evaluation'],long['evaluation']
                equal &= x is not None and y is not None and len(x['steps'])<=len(y['steps']) and x['steps']==y['steps'][:len(x['steps'])]
            prefixes &= equal
            triples.append(dict(production=p,physics_seed=seed,prefix_equal=equal))
            if p>=4:
                controls &= all(r['status']=='Complete' and r['evaluation']['proposal_fires']==(r['schedule']['horizon'] if p==4 else 0) for r in trio)
    return dict(by_horizon=groups,attempted_steps=attempts,advancing_steps=advances,complete_cells=complete,
        terminal_native_cells=terminal,measurement_complete=terminal==36,prefixes_equal=prefixes,controls_pass=controls),triples

def audit(out,integrity=True,verify_prerequisites=True):
    out=Path(out)
    if integrity: a.verify(out)
    m=a.load(out/'manifest.json')
    assert m['profile']==PROFILE and m['training_admitted'] is False and m['full_training_authorized'] is False
    assert all(m[k]==0 for k in ('optimizer_evaluation_steps','precheck_evaluation_steps','held_out_evaluation_steps'))
    assert m['design']==design() and m['schedule']==schedule(design()) and m['scheduled_cells']==36
    assert m['grammar_compilations']==1
    evidence=m['admission']; hist=evidence['historical_evidence']
    assert evidence['profile']==PROFILE and evidence['scaling_execution_authorized'] is True
    assert evidence['training_admitted'] is False and evidence['full_training_authorized'] is False
    assert evidence['implementation_sources_verified']==230 and hist['historical_scopes_preserved'] is True
    assert hist['training_admitted'] is False and hist['full_training_authorized'] is False
    assert hist['prior_original_frozen_checker_pass'] is False and hist['prior_corrected_accounting_pass'] is True
    assert set(hist['receipts'])==set(a.SPECS)
    for name,(seal,sha,count) in a.SPECS.items():
        p=hist['receipts'][name];assert p['seal_sha256']==sha and p['seal_file']==seal and p['sealed_files_verified']==count and p['exact_file_coverage'] is True
        if verify_prerequisites: a.verify(Path(p['archive']),seal,sha,count)
    engineering=evidence['engineering_prerequisite'];assert engineering['seal_sha256']==ENGINEERING and engineering['sealed_files_verified']==247
    if verify_prerequisites: a.verify(Path(engineering['archive']),expected=ENGINEERING,count=247)
    runtime=Path(hist['receipts']['runtime']['archive'])
    pins={k:v for k,v in a.load(runtime/'manifest.json')['snapshot']['sources'].items() if a.pinned(k)}
    assert len(pins)==230
    snap=m['snapshot'];assert snap['frozen_before_measurement'] is True and set(snap['sources'])==set(pins)|EXTRA and len(snap['sources'])==244
    assert a.digest(out/'executed_binary')==snap['binary_sha256']
    for name,sha in snap['sources'].items():
        assert a.safe(name) and a.digest(out/'source_snapshots'/name)==sha
        if name in pins and name!=SIGNAL: assert sha==pins[name]
    old=(runtime/'source_snapshots'/SIGNAL).read_text(); current=(out/'source_snapshots'/SIGNAL).read_text()
    assert current==bridge(old)
    proof=evidence['native_source_bridge']
    assert proof==dict(source=SIGNAL,historical_sha256=pins[SIGNAL],current_sha256=snap['sources'][SIGNAL],exact_registered_transform=True)
    assert a.load(out/'source_snapshots/configs/ere_complete_scaling_v2.json')==design()
    assert a.digest(out/'catalogue.json')==a.CATALOGUE
    catalogue=a.load(out/'catalogue.json'); emissions=a.load(out/'emissions.json'); assert len(emissions)==6
    h=a.helper(runtime)
    for p,e in enumerate(emissions):
        h.check_membership(catalogue,e,full=True)
        assert e['emission_seed']==design()['emission_seeds'][p]
        if p>=4: assert e['parent']==design()['control_parents'][p-4] and e['operator']==(11 if p==4 else 12) and e['role']=='Proposal' and e['production_index']==0
    assert emissions[4]['classes']['active'] is True and emissions[5]['classes']['active'] is False
    assert all(emissions[4]['classes'][k]==emissions[5]['classes'][k] for k in ('core','proposal','proposal_execution'))
    assert emissions[4]['initial_world_hash']==emissions[5]['initial_world_hash']
    expected=schedule(design())
    assert {p.name for p in (out/'cells').iterdir()}=={q['id']+'.json' for q in expected}
    rows=[a.load(out/'cells'/(q['id']+'.json')) for q in expected]
    for q,r in zip(expected,rows):
        assert r['schedule']==q
        cell(r,emissions[q['production']],catalogue,h)
    derived,triples=summary(rows,design()); s=a.load(out/'summary.json')
    assert s['profile']==PROFILE and s['scheduled_cells']==36 and s['training_admitted'] is False and s['full_training_authorized'] is False
    assert s['counts_cover_receipted_cells_only'] is True and s['unknown_partial_work_cells']==sum(r['status']=='Incomplete' for r in rows)
    for k,v in derived.items(): assert s[k]==v,(k,s[k],v)
    for k in ('attempted_steps','advancing_steps','terminal_native_cells','measurement_complete'): assert m[k]==derived[k]
    assert m['recorded_cells']==sum(r['status']!='NotRun' for r in rows) and m['inflight_native_work_unknown'] is False
    assert m['status']==('complete' if derived['measurement_complete'] else 'incomplete')
    assert 0<=s['grammar_compile_seconds']<=s['elapsed_seconds'] and m['elapsed_seconds']==s['elapsed_seconds']
    pass_=derived['complete_cells']==36 and derived['prefixes_equal'] and derived['controls_pass'] and s['elapsed_seconds']<1800 and all(r.get('elapsed_seconds',float('inf'))<120 for r in rows)
    assert s['native_scaling_checks_pass']==pass_ and m['native_scaling_checks_pass']==pass_
    assert sum((out/p).stat().st_size for p in a.files(out))<1024**3
    metrics=[]
    for r in rows:
        v=r['evaluation']; steps=v['steps'] if v else []
        metrics.append(dict(id=r['schedule']['id'],status=r['status'],seconds=r.get('elapsed_seconds'),score=v['score'] if v else None,
            reason=v['reason'] if v else r.get('reason'),attempted_steps=v['attempted_steps'] if v else None,
            peak_work=max((x['usage']['work'] for x in steps),default=0),peak_results=max((x['usage']['results'] for x in steps),default=0),
            peak_nodes=max((x['nodes'] for x in steps),default=0),peak_edges=max((x['edges'] for x in steps),default=0),
            node_growth_ratio=steps[-1]['nodes']/max(steps[0]['nodes'],1) if steps else None,
            edge_growth_ratio=steps[-1]['edges']/max(steps[0]['edges'],1) if steps else None))
    return dict(audit_pass=True,scaling_panel_pass=pass_,training_admitted=False,full_training_authorized=False,
        independent_physics_replay=False,frozen_sources=244,implementation_sources=230,**derived,triples=triples,metrics=metrics)

def tamper_tests(out):
    cases=['horizon','seed','budget','training','schedule','denominator','counts','source','binary','bridge','membership','score','fires','credit','step','allowance','graph_cap','prefix_hash','summary','controls','missing','extra','catalogue','elapsed']
    for case in cases:
        with tempfile.TemporaryDirectory(prefix='ere-scaling-audit-') as tmp:
            dst=Path(tmp)/'receipt';shutil.copytree(out,dst)
            m=a.load(dst/'manifest.json'); s=a.load(dst/'summary.json')
            q=m['schedule'][2];p=dst/'cells'/(q['id']+'.json');r=a.load(p);v=r['evaluation']
            if case=='horizon': m['design']['horizons'][-1]=9999
            elif case=='seed': r['schedule']['physics_seed']+=1
            elif case=='budget': v['runtime_match_budget']['work']=400000
            elif case=='training': m['training_admitted']=True
            elif case=='schedule': m['schedule'].pop()
            elif case=='denominator': s['by_horizon']['10000']['scheduled']=11
            elif case=='counts': m['optimizer_evaluation_steps']=1
            elif case=='source': (dst/'source_snapshots/crates/hg-train/src/optim.rs').write_text('stale')
            elif case=='binary': (dst/'executed_binary').write_bytes(b'wrong')
            elif case=='bridge': m['admission']['native_source_bridge']['exact_registered_transform']=False
            elif case=='membership':
                es=a.load(dst/'emissions.json');es[0]['production_count']+=1;(dst/'emissions.json').write_text(json.dumps(es))
            elif case=='score': v['score']=.1234
            elif case=='fires': v['proposal_fires']+=1
            elif case=='credit': v['credited_rule_ids']=[-1]
            elif case=='step': v['steps'][0]['step']=2
            elif case=='allowance': v['steps'][0]['usage']['rules'][0]['work_allowance']+=1
            elif case=='graph_cap': v['steps'][0]['nodes']=2001
            elif case=='prefix_hash': v['steps'][0]['world_hash']='0123456789abcdef'
            elif case=='summary': s['complete_cells']=0
            elif case=='controls': s['controls_pass']=not s['controls_pass']
            elif case=='missing': p.unlink()
            elif case=='extra': (dst/'cells/unregistered.json').write_text('{}')
            elif case=='catalogue': (dst/'catalogue.json').write_text('{}')
            elif case=='elapsed': s['elapsed_seconds']=1801
            if case!='missing': p.write_text(json.dumps(r))
            (dst/'manifest.json').write_text(json.dumps(m));(dst/'summary.json').write_text(json.dumps(s))
            try: audit(dst,integrity=False,verify_prerequisites=False)
            except (AssertionError,KeyError,ValueError,OSError,TypeError): continue
            raise AssertionError('tamper accepted: '+case)
    return len(cases)

def main():
    assert __debug__, 'do not use -O'
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--tamper-tests',action='store_true');p.add_argument('--seal',action='store_true')
    args=p.parse_args();result=audit(args.output);result['tamper_rejections']=tamper_tests(args.output) if args.tamper_tests else 0
    if args.seal:
        (args.output/'audit.json').write_text(json.dumps(result,indent=2));a.seal(args.output);audit(args.output)
    print(json.dumps({k:v for k,v in result.items() if k not in ('metrics','triples')}))

if __name__=='__main__': main()
