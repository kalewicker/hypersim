#!/usr/bin/env python3
"""Independent fixed-denominator retained-proposal native/recovery reconstruction."""
import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import time
import tomllib
PROFILE='evolving-minds-d0-retained-streaming-v1'
POLICY='founder-first-proposal-v1'
def require(ok,reason):
    if not ok:raise ValueError(reason)
def data(p):return json.loads(Path(p).read_text())
def load(p,name):
    spec=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(spec);old=sys.dont_write_bytecode
    try:sys.dont_write_bytecode=True;spec.loader.exec_module(m)
    finally:sys.dont_write_bytecode=old
    return m
def cases():
    rows=[]
    for name,seed,program,kernel,reverse in [
        ('source17_delayed',17,'Delayed','SourceKernel',False),('source23_delayed',23,'Delayed','SourceKernel',False),
        ('faithful_immediate',17,'Immediate','Faithful',False),('invalid_immediate',17,'Immediate','Invalid',False),
        ('unreachable_immediate',17,'Immediate','MissingPort',False),('source23_reverse',23,'Delayed','SourceKernel',True)]:
        rows.append(dict(name=name,config=dict(seed=seed,endowment=24,founders=[program]*2,kernels=[kernel]*2,
            home_offsets=[0,4],rotation=0,reversed=reverse,horizon=64,scheduler='semantic-lottery-v1')))
    return rows
def denominator(root):
    require(data(root/'registered_cases.json')==cases(),'registered retained denominator')
    require({p.name for p in root.iterdir() if p.is_dir()}=={'baseline_'+r['name'] for r in cases()}|{'fault_before_commit','recovered_before_commit'},'complete retained segment denominator')
def verify(root,extra_segment=None):
    start=time.monotonic();root=Path(root)
    require(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')),'retained archive path')
    require(sum(p.stat().st_size for p in root.rglob('*') if p.is_file())<=256*1024*1024,'retained panel cap')
    denominator(root);sources=data(root/'sources.json')
    for name,path in [('checker.py','scripts/verify_evolving_minds_retained.py'),('stream_reference.py','scripts/verify_evolving_minds_streaming.py'),
        ('admission_reference.py','scripts/verify_evolving_minds_admission.py'),('funding_reference.py','scripts/verify_evolving_minds_funding.py'),
        ('opportunity_reference.py','scripts/evolving_minds_opportunity.py'),('replay.py','scripts/verify_evolving_minds.py'),
        ('d0_reference.py','scripts/verify_evolving_minds_d0.py'),('schedule_reference.py','scripts/verify_evolving_minds_choice.py'),
        ('contract.md','experiments/evolving_minds/contracts/d0_retained_proposal_v1.md'),('base_rules.toml','rules/evolving_minds/admission_v1.toml')]:
        require((root/name).read_text()==sources[path],'frozen retained source binding')
    v=load(root/'stream_reference.py','retained_stream');a=load(root/'admission_reference.py','retained_adm');c=load(root/'replay.py','retained_replay');d=load(root/'d0_reference.py','retained_d0');s=load(root/'schedule_reference.py','retained_schedule');f=load(root/'funding_reference.py','retained_funding');o=load(root/'opportunity_reference.py','retained_opportunity')
    modules=(a,c,d,s);base=(root/'base_rules.toml').read_bytes();b24=(root/'b24_base_rules.toml').read_bytes()
    require(hashlib.sha256(base).hexdigest()==a.LAW_SHA and tomllib.loads(b24.decode())==f.funded_laws(base,24),'independently derived unchanged funded laws')
    configs={'baseline_'+r['name']:r['config'] for r in cases()};cfg=cases()[3]['config']
    configs.update(fault_before_commit=cfg,recovered_before_commit=cfg)
    evidence={};results={};reports=[];count=0;advances=0
    for name,config in configs.items():
        p=root/name;chain=v.inspect_segment(p,funded_config=config,retained_policy=POLICY);seg=chain[-1]
        m=data(p/'manifest.json');status=data(p/'run_status.json');report=data(p/'segment_report.json');v.boundary(status,PROFILE)
        faulted=name=='fault_before_commit';tick=seg['marker']['cursor']['tick'];expected_tick=8 if faulted else 64
        require(report['profile']==PROFILE and report['retained_policy']==POLICY and report['funded_config']==config and
            m['experiment_name']==PROFILE and m['seed']==config['seed'] and m['max_steps']==2944 and m['final_step']==46*expected_tick and
            m['state']==report['state']==('Interrupted' if faulted else 'Completed') and tick==expected_tick and
            report['committed_final_tick']==tick and report['native_final_tick']==(16 if faulted else 64) and
            report['start_tick']==seg['header']['start_tick'] and report['fault']==seg['header']['fault']==('BeforeCommit' if faulted else 'Normal') and
            status['engineering_execution_complete'] is (not faulted),'actual retained segment scope/status')
        resources=report['resources']
        require(resources['drained_history_empty'] and resources['retained_entries_peak']==46 and resources['retained_rows_peak']==resources['retained_rounds_peak']==1 and resources['retained_requests_peak']<=8 and resources['retained_lineage_peak']<=16 and resources['advance_seconds']<=120 and resources['payload_bytes']==(p/'ticks.jsonl').stat().st_size,'bounded retained resources')
        require(resources['checkpoints']==[[x['marker']['cursor']['tick'],x['checkpoint_bytes']] for x in seg['commits']],'checkpoint resource denominator')
        if faulted:require((p/'commit_0016.json.pending').exists() and not (p/'commit_0016.json').exists(),'retained pending publication preserved')
        final=None;physics=None
        for commit in seg['commits']:
            final,physics=v.physical_audit(chain,commit,modules,b24,funded_config=config,retained_policy=POLICY);count+=1
        reports.append(report);advances+=(report['native_final_tick']-report['start_tick'])*46
        if not faulted:
            observer=o.LifetimeObserver(c,d,final,config);c.apply=observer.apply
            try:
                v.physical_audit(chain,seg['commits'][-1],modules,b24,funded_config=config,retained_policy=POLICY)
                opportunity=observer.finish()
            finally:c.apply=observer.orig_apply
            evidence[name]=final;results[name]=dict(retained=physics['retained'],opportunity=opportunity)
    require(data(root/'summary.json')==reports,'retained segment summaries')
    require(evidence['recovered_before_commit']==evidence['baseline_invalid_immediate'],'retained complete recovery identity')
    x=copy.deepcopy(evidence['baseline_source23_delayed']);y=copy.deepcopy(evidence['baseline_source23_reverse'])
    for e in [x,y]:
        e.pop('journal');e['admission'].pop('recipe')
        for row in e['rows']:row.pop('match_work')
    require(x==y and results['baseline_source23_delayed']==results['baseline_source23_reverse'],'retained allocation semantic/endpoint identity')
    invalid=evidence['baseline_invalid_immediate']['admission']['records']
    require(len(invalid)>2 and all(not r['syntax_valid'] and r['request_charge']==1 and r['refunded']==29 and r['copying_charge']==0 for r in invalid),'repeated retained native rejects')
    require(results['baseline_unreachable_immediate']['opportunity']['descendants_grammar_failed']>=2,'retained paid unreachable development')
    status=data(root/'run_status.json');v.boundary(status,PROFILE);require(status['engineering_execution_complete'],'retained panel completion')
    m=data(root/'manifest.json');require(m['state']=='Completed' and m['experiment_name']==PROFILE and m['final_step']==m['max_steps']==20608,'real retained root manifest')
    require(data(root/'execution_counts.json')==dict(segments=8,complete_chains=7,complete_committed_chain_advances=20608,native_advances_executed=advances,unpublished_or_repeated_tail_advances=368),'retained actual execution count')
    require(advances==20976,'fixed retained execution horizon')
    if extra_segment is not None:
        chain=v.inspect_segment(extra_segment,funded_config=cfg,retained_policy=POLICY)
        require(chain[-1]['marker']['cursor']['tick']==64 and chain[-1]['header']['start_tick']==8,'fresh retained process horizon')
        for commit in chain[-1]['commits']:
            extra,_=v.physical_audit(chain,commit,modules,b24,funded_config=cfg,retained_policy=POLICY);count+=1
        require(extra==evidence['baseline_invalid_immediate'],'fresh retained process exact identity')
    return dict(profile=PROFILE,audit_pass=True,development_only=True,scientific_run_launched=False,gate_pass=False,training_admitted=False,evolution_enabled=False,reproductive_fairness_accepted=False,
        baselines_verified=6,complete_chains_verified=7,segments_verified=8,published_states_reconstructed=count,native_panel_advances=advances,
        fresh_process_continuation_verified=extra_segment is not None,audit_seconds=time.monotonic()-start,
        source_sha256={n:hashlib.sha256(raw.encode()).hexdigest() for n,raw in sources.items()},results=results)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('directory');p.add_argument('--extra-segment');args=p.parse_args();print(json.dumps(verify(args.directory,args.extra_segment),indent=2))
