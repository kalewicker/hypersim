"""Independent zero-native audit of ownership transitions and spooled native law."""
import json
from pathlib import Path
import sys
from compact_evidence import Store, sha
from audit_durability import read
from audit_campaign_adapter import recover_values
from audit_readiness import require


class Journal:
    def __init__(self, store): self.root = store.root
    def iterdir(self): return (p for p in self.root.iterdir() if p.name != 'output_sha256.json')
    def __truediv__(self, key): return self.root / key


def ledger(root):
    owner = json.loads((root/'owner.json').read_text()); head=None; states={}; admissions=[]
    records=sorted((root/'events').iterdir())
    for i,p in enumerate(records):
        require(p.name==f'{i:08}.json' and not p.is_symlink(),'event file denominator')
        raw=p.read_bytes();e=json.loads(raw)
        require(e['owner']==owner['owner'] and e['sequence']==i and e['previous']==head,'independent spool chain')
        head=sha(raw);v=e['value'];unit=v['unit'];kind=v['kind']
        if kind=='reserved':
            require(unit==f'u{len(states):06}' and unit not in states,'unique owned unit')
            require(v['writer_byte_allowance']==32505856 and v['reservation']==117735424,'fixed bounded admission')
            a=v['admission'];require(a['charged_bytes']<=a['cap_bytes']==owner['cap_bytes'],'admission before native')
            require(a['charged_bytes']==a['retained_bytes']+a['pending_reservation_bytes']+a['control_reserve_bytes']+v['reservation']+1048576,'reservation arithmetic')
            admissions.append(a['charged_bytes']);states[unit]=dict(v)
        elif kind.startswith('restore'):
            require(states[unit]['kind']=='released','restore only verified retained evidence')
            order={None:'restore_intent','restore_intent':'restored','restored':'restore_release_intent','restore_release_intent':'restore_released'}
            require(order.get(states[unit].get('restore_state'))==kind,'restore transition sequence')
            states[unit]['restore_state']=kind
        else:
            order={'reserved':'sealed','sealed':'verified','verified':'release_intent','release_intent':'released'}
            require(order.get(states[unit]['kind'])==kind,'no release before verified durable intent')
            states[unit].update(v)
    return states,admissions


def audit(package):
    c=json.loads((package/'contract.json').read_text());tests=json.loads((package/'fault_tests.json').read_text())
    expected=set(c['fault_cases']+c['rejections']+['stress_full_roundtrip'])
    require({x['case'] for x in tests}==expected and len(tests)==14,'fixed saved-evidence test denominator')
    peak=0
    for t in tests:
        root=package/'cases'/t['case'];states,admitted=ledger(root);peak=max([peak]+admitted)
        if t['case']=='cap_before_dispatch':require(not states and not t['native_dispatched'],'cap fails before reservation/work');continue
        state=states[t.get('unit','u000000')]
        if t['case'] in c['fault_cases']+['stress_full_roundtrip']:
            require(state['kind']=='released' and state['restore_state']=='restore_released','recovered archive and restore release')
            store=Store(root/'compact'/state['unit'],state['index_sha256'],state['seal_sha256']);store.verify_all()
            require(not (root/'raw'/state['unit']).exists() and not (root/'restore'/state['unit']).exists(),'only retained compact evidence remains')
        elif t['case'] in ['pending_native_no_restore','torn_native_no_restore']:
            require(state['kind']=='released' and state.get('restore_state') is None,'unsafe restore refused before lease')
            store=Store(root/'compact'/state['unit'],state['index_sha256'],state['seal_sha256']);store.verify_all()
            values,_,staging=read(Journal(store))
            require(bool(staging) if t['case']=='torn_native_no_restore' else values[-1]['kind']=='intent','retained uncertain/torn evidence')
        elif t['case']=='reserved_writer_no_retry':require(state['kind']=='reserved','no transition or redispatch of unknown writer')
        elif t['case']=='partial_compact_no_overwrite':require(state['kind']=='sealed' and not (root/'compact'/state['unit']/'index.json').exists(),'partial compact retained')
        elif t['case']=='corrupt_compact_no_release':require(state['kind']=='sealed' and (root/'raw'/state['unit']).exists(),'raw preserved after compact corruption')
        elif t['case']=='foreign_raw_no_release':require(state['kind']=='release_intent' and (root/'raw'/state['unit']/'foreign.txt').exists(),'foreign file refuses cleanup')
        elif t['case']=='symlink_raw_no_release':require(state['kind']=='reserved' and t['test_link_removed'],'symlink never sealed or released')
        elif t['case']=='modified_restore_no_release':require(state['restore_state']=='restore_release_intent' and (root/'restore'/state['unit']).exists(),'changed restore retained')
        if 'capacity' in t: require(t['capacity']['charged_bytes']<=t['capacity']['cap_bytes'],'failed routes remain charged within cap')
    root=package/'native_spool';states,admitted=ledger(root);peak=max([peak]+admitted)
    require(len(states)==3,'exact native route denominator')
    routes=json.loads((package/'native_routes.json').read_text());adapter=json.loads((package/'adapter_contract.json').read_text());results={};total=0
    binary=sha((package/'cumulative-campaign').read_bytes());sources=sha((package/'source_sha256.json').read_bytes())
    for route in c['native_routes']:
        name=route['name'];unit=routes[name]['unit'];state=states[unit]
        require(state['kind']=='released' and not (root/'raw'/unit).exists(),'native evidence safely compacted')
        request_raw=(root/'requests'/(unit+'.json')).read_bytes();request=json.loads(request_raw)
        require(request==json.loads((package/'native_inputs'/(name+'.json')).read_text()) and sha(request_raw)==state['request_sha256'],'pre-dispatch request identity')
        require(request['source_identity']=={'binary_sha256':binary,'source_sha256':sources} and request['seed']==2026101001 and request['purpose']=='development','no scientific seed or changed executable')
        require(request['native_budget']==250 and request['execution_allowance']==250-total and request['journal_byte_allowance']==32505856,'aggregate native and storage funding')
        require(request['fault']==route['fault'] and request['spec']==adapter['world_spec'],'unchanged matched development law')
        store=Store(root/'compact'/unit,state['index_sha256'],state['seal_sha256']);store.verify_all()
        values,head,staging=read(Journal(store))
        result=recover_values(values,head,staging,request,sha(request_raw),adapter,[],results.get(route.get('source')))
        require(result['status']==('partial' if route['fault'] else 'complete') and not result['unknown'],'exact declared interruption status')
        receipt=json.loads((root/'requests'/(unit+'.receipt.json')).read_text())
        require(not receipt['timed_out'] and receipt['returncode']==(1 if route['fault'] else 0),'expected native child result')
        require(routes[name]['lower']==routes[name]['upper']==result['here_upper'] and routes[name]['head']==head,'independent native work receipt')
        results[name]=result;total+=result['here_upper']
    require(results['complete']['rows']==results['continue']['rows'] and results['complete']['checkpoint']==results['continue']['checkpoint'],'byte-exact native continuation through compact/restore')
    require(total==144<=250,'bounded native work')
    require(states[routes['boundary']['unit']]['restore_state']=='restore_released' and not any((root/'restore').iterdir()),'continuation restore lease released')
    return {'engineering_spool_pass':True,'saved_evidence_checks':14,'native_units':3,'native_work_lower':total,'native_work_upper':total,
            'maximum_admission_charge_bytes':peak,'bounded_spool_cap_bytes':268435456,'exact_native_continuation':True,
            'storage_spool_development_integration_pass':True,'production_scientific_campaign_admitted':False,
            'readiness_gate_complete':False,'scientific_acceptance':False,'evolution_demonstrated':False,'campaign_launch_authorized':False}


if __name__=='__main__': print(json.dumps(audit(Path(sys.argv[1])),indent=2,sort_keys=True))
