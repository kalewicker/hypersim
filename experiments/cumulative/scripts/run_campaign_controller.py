"""Publish the zero-native controller rehearsal and draft final campaign contract."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from run_readiness import ROOT, PACKAGE, FALSE, digest, write, source_paths
from run_campaign_adapter import custody
from compact_evidence import Store, source_inventory, sha, encode_json
from campaign_controller import Controller, PROFILE, initial, next_unit, transition
from audit_campaign_controller import audit_ledger
from audit_campaign_adapter import recover, recover_values
from audit_durability import read
from audit_spool import ledger, Journal


def owner(purpose,worlds,limits):
    return dict(profile=PROFILE,purpose=purpose,worlds=worlds,limits=limits,slots_per_world=10,batch_size=128,
                assay_work_bound=20,maximum_assays=78960,**FALSE)


def schedule(c,refs,registry,measurement,decision):
    """Rehearsal only: produce logical events from saved or synthetic evidence."""
    s=initial(c)
    def event(kind,value):
        nonlocal s
        s=transition(c,s,dict(kind=kind,value=value));return kind,value
    for ref in refs:
        yield event('intent',dict(unit=next_unit(c,s),native_ms_bound=10,elapsed_ms=s['elapsed_ms'],storage_charge_bytes=3144140800))
        yield event('receipt',dict(request_sha256=ref['request_sha256'],status='complete',work_lower=ref['work'],work_upper=ref['work'],native_ms=1,elapsed_ms=s['elapsed_ms']+1,evidence_sha256=ref['evidence_sha256']))
    yield event('registry',dict(sha256=registry['sha256'],measurement_sha256=measurement,count=registry['count'],slots=len(c['worlds'])*10))
    while s['phase']=='assays':
        unit=next_unit(c,s)
        yield event('intent',dict(unit=unit,native_ms_bound=10,elapsed_ms=s['elapsed_ms'],storage_charge_bytes=3144140800))
        work=registry['saved_work'] if 'saved_work' in registry else unit['work_bound']
        evidence=registry['saved_head'] if 'saved_head' in registry else sha(encode_json(dict(synthetic_range=unit)))
        yield event('receipt',dict(request_sha256=unit['request_sha256'],status='complete',work_lower=work,work_upper=work,native_ms=1,elapsed_ms=s['elapsed_ms']+1,evidence_sha256=evidence))
    yield event('close',dict(reason='complete',decision_sha256=decision))


def run(out):
    start=time.monotonic();out.mkdir(parents=True,exist_ok=False)
    for name in ['commands','controllers']:(out/name).mkdir()
    c=json.loads((PACKAGE/'controller_contract.json').read_text())
    manifest=dict(profile=c['profile'],status='running',engineering_controller_rehearsal_pass=False,native_rewrites=0,**FALSE);write(out/'manifest.json',manifest)
    def preserve():
        custody()
        prior=json.loads((PACKAGE/'measurement_aggregation_contract.json').read_text())
        for p in [prior[k] for k in ['parent','adapter_parent','storage_parent','reuse_native_parent']]+[c['parent']]:source_inventory(ROOT/p['archive'],p['seal_sha256'])
        old=json.loads((ROOT/c['parent']['archive']/'source_sha256.json').read_text())
        if any(digest(ROOT/n)!=h for n,h in old.items()):raise ValueError('previous source modified')
    def command(name,args):
        r=subprocess.run(args,capture_output=True,text=True,timeout=max(1,1200-(time.monotonic()-start)))
        write(out/'commands'/name,dict(exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr))
        if r.returncode:raise ValueError(r.stderr)
        return r.stdout
    try:
        preserve();pins={str(p.relative_to(ROOT)):digest(p) for p in source_paths()};write(out/'source_sha256.json',pins)
        for name in pins:
            dest=out/'source_snapshots'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/name,dest)
        write(out/'contract.json',c);frozen=out/'source_snapshots/experiments/cumulative/scripts'
        command('tests.json',[sys.executable,'-B',str(frozen/'test_campaign_controller.py')])
        parent=out/'measurement_parent';shutil.copytree(ROOT/c['parent']['archive'],parent)
        command('parent_recheck.json',[sys.executable,'-B',str(parent/'source_snapshots/experiments/cumulative/scripts/recheck_measurement.py'),str(parent),c['parent']['seal_sha256']])
        campaign=json.loads((parent/'campaign_contract.json').read_text());panel=json.loads((parent/'prospective_pending.json').read_text())['panel']
        draft=dict(profile='cumulative-final-campaign-draft-v1',status='draft_not_authorized',panel=panel,
                   physical_law_contract_sha256=digest(PACKAGE/'integration_contract.json'),prospective_contract=campaign,
                   claim_scope=dict(primary='cumulative functional evolution within this fixed world',secondary='paired endogenous versus external editing advantage',
                                    diagnostic_environment=512,diagnostic_cycles=4,assay_seed=2026100903,independent_unit='47 matched seed blocks',
                                    environmental_generalization=False,open_ended_evolution=False),
                   proposed_budget=c['virtual_resource_ceilings'],budget_approved=False,scientific_native_profile_implemented=False,live_dispatch_integrated=False,
                   stopping_rule='One funded fixed panel. Unknown execution stays charged and inconclusive; no retry, replacement seed or threshold relaxation. A valid negative closes this configuration.',
                   remaining=['Versioned native admission for exact frozen world identities and independent scientific journal auditor.',
                              'Live dispatch bridge enforcing real clock/spool reservations and verifying native evidence before receipt admission.',
                              'Explicit final scientific budget and launch authorization after the integrated controller passes its bounded development test.'],**FALSE)
        write(out/'final_campaign_draft.json',draft)
        info=json.loads((parent/'adapter_store.json').read_text());adapter_root=json.loads((parent/'contract.json').read_text())['adapter_parent']['seal_sha256']
        store=Store(parent/'adapter_evidence',info['index_sha256'],adapter_root);store.verify_all();ac=json.loads((store.root/'contract.json').read_text())
        bounds={'endogenous':250,'external':240,'nonheritable':250,'frozen':202};worlds=[];refs=[]
        for arm in campaign['design']['arms']:
            raw=(store.root/'inputs'/('world_'+arm+'.json')).read_bytes();r=recover(store.root/'journals'/('world_'+arm),json.loads(raw),sha(raw),ac,[])
            worlds.append(dict(world_id='development/'+arm,request_sha256=sha(raw),work_bound=bounds[arm]))
            refs.append(dict(request_sha256=sha(raw),evidence_sha256=r['head'],work=r['here_upper']))
        origin=parent/'native_origin';states,_=ledger(origin/'native_spool');unit,state=next(iter(states.items()))
        raw=(origin/'native_spool/requests'/(unit+'.json')).read_bytes();st=Store(origin/'native_spool/compact'/unit,state['index_sha256'],state['seal_sha256']);values,head,staging=read(Journal(st))
        items=json.loads((parent/'registry.json').read_text())['assays'];r=recover_values(values,head,staging,json.loads(raw),sha(raw),ac,items)
        registry=dict(sha256=digest(parent/'registry.json'),count=len(items),saved_work=r['here_upper'],saved_head=head)
        logical=sha(encode_json(dict(registry_sha256=registry['sha256'],start=0,end=len(items))))
        write(out/'saved_evidence_bindings.json',refs+[dict(request_sha256=logical,evidence_sha256=head,work=r['here_upper'])])
        ctl=Controller.create(out/'controllers/saved_development',owner('saved_development',worlds,c['virtual_resource_ceilings']))
        ctl.append_many(schedule(ctl.contract,refs,registry,digest(parent/'measurement.json'),digest(parent/'report.json')))
        print(json.dumps(dict(phase='saved_development',worlds=4,assays=9,new_native_rewrites=0)),flush=True)
        synthetic=[]
        for i in range(78960):
            number=i;stages=[]
            for _ in range(8):stages.append(number%6-1);number//=6
            synthetic.append(dict(id=i,role='intact',stages=stages))
        write(out/'synthetic_registry.json',dict(profile='explicitly_synthetic_controller_load_only',assays=synthetic))
        worlds=[dict(world_id=w['world_id'],request_sha256=sha(encode_json(w)),work_bound=bounds[w['arm']]) for w in panel['worlds']]
        refs=[dict(request_sha256=w['request_sha256'],evidence_sha256=sha(encode_json(dict(synthetic_world=w['world_id']))),work=w['work_bound']) for w in worlds]
        registry=dict(sha256=digest(out/'synthetic_registry.json'),count=78960)
        write(out/'synthetic_measurement.json',dict(scope='synthetic capacity only',worlds=188,slots=1880,assay_requests=78960))
        write(out/'synthetic_decision.json',dict(scope='synthetic control-flow closure only',scientific_interpretation=None,**FALSE))
        ctl=Controller.create(out/'controllers/full_scale',owner('synthetic_full_scale',worlds,c['virtual_resource_ceilings']))
        ctl.append_many(schedule(ctl.contract,refs,registry,digest(out/'synthetic_measurement.json'),digest(out/'synthetic_decision.json')))
        print(json.dumps(dict(phase='full_scale',worlds=188,assay_requests=78960,batches=617,new_native_rewrites=0)),flush=True)
        ctl=Controller.create(out/'controllers/interrupted',owner('synthetic_full_scale',worlds,c['virtual_resource_ceilings']))
        state=initial(ctl.contract);ctl.append('intent',dict(unit=next_unit(ctl.contract,state),native_ms_bound=10,elapsed_ms=0,storage_charge_bytes=3144140800))
        # Simulated process restart: surviving intent can be closed, never redispatched.
        ctl=Controller(ctl.root);ctl.append('close',dict(reason='operational_stop',decision_sha256=None))
        summaries={name:audit_ledger(out/'controllers'/name) for name in c['routes']};write(out/'controller_summaries.json',summaries)
        audited=json.loads(command('independent_audit.json',[sys.executable,'-B',str(frozen/'audit_campaign_controller.py'),str(out)]));write(out/'audit.json',audited)
        preserve()
        if any(digest(ROOT/n)!=h for n,h in pins.items()):raise ValueError('source changed during rehearsal')
        write(out/'preservation.json',dict(previous_archive_roots_preserved=16,previous_source_pins_preserved=168,historical_files_removed=0))
        manifest.update(audited,status='complete',source_files=len(pins),python_test_groups=8,elapsed_seconds=time.monotonic()-start,**FALSE);write(out/'manifest.json',manifest)
        if time.monotonic()-start>1200 or sum(p.stat().st_size for p in out.rglob('*') if p.is_file())>134217728:raise ValueError('workflow/output limit')
    except Exception as e:
        manifest.update(status='failed',error=str(e),engineering_controller_rehearsal_pass=False);write(out/'manifest.json',manifest)
        write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'})
        raise
    write(out/'output_sha256.json',{str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    print(json.dumps(dict(output=str(out),seal_sha256=digest(out/'output_sha256.json'),**manifest),indent=2))


if __name__=='__main__':run(Path(sys.argv[1]).resolve())
