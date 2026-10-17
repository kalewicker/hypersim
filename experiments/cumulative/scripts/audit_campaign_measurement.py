"""Independent census/version reconstruction and exact 13-role registry validation."""
import copy
import json
import sys
from pathlib import Path
from audit_population import score
from audit_versions import reconstruction
from audit_readiness import require


def expected(cases, contract):
    samples, requests = [], []
    def request(role, stages):
        if not stages:
            return None
        for item in requests:
            if item['role'] == role and item['stages'] == stages:
                return item['id']
        requests.append(dict(id=len(requests), role=role, stages=stages[:]))
        return len(requests)-1
    for case in cases:
        for tick in contract['design']['census_ticks']:
            snapshot = next((r['after'] for r in case['rows'] if r['after']['tick'] == tick), None) if tick else case['initial']
            records = {m['id']: dict(initial=m['state'], frames=[]) for m in case['initial']['cells']}
            for row in case['rows']:
                if row['after']['tick'] > tick:
                    break
                for op in row['operations']:
                    if op['phase'] == 'newborn':
                        records[op['cell']] = dict(initial=op['initial'], frames=[])
                    records[op['cell']]['frames'].extend(op['frames'])
            states = {m['id']: m['state'] for m in snapshot['cells']} if snapshot else {}
            ranking = sorted([[i, score(snapshot['seed'], tick, 2, i)] for i in states], key=lambda x: (x[1], x[0]))
            for slot in range(contract['design']['slots_per_census']):
                s = dict(source=case['name'], census=tick, slot=slot, ranking=ranking)
                if snapshot is None:
                    s['status'] = 'unmeasured'
                elif slot >= len(ranking):
                    s['status'] = 'empty'
                else:
                    ident = ranking[slot][0]; lineage = reconstruction(states, records, ident)
                    stages = [stage for serial, stage in states[ident]['program']['genes']]
                    s.update(cell=ident, status='living' if states[ident]['alive'] else 'dead', lineage=lineage,
                             endpoint_assays={r: request(r, stages) for r in ['intact', 'sham', 'restored']})
                    for pair in lineage['pairs']:
                        add_roles(lineage, pair)
                        pair['assays'] = {r: request(r, pair['programs'][r]) for r in contract['candidate']['roles']}
                        pair['inadmissible_empty_ablation'] = None in pair['assays'].values()
                        del pair['programs']
                samples.append(s)
    return dict(profile='cumulative-campaign-measurement-readiness-v1', samples=samples), dict(profile='cumulative-campaign-adapter-readiness-v1', assays=requests)


def add_roles(lineage, pair):
    first = next(e for e in lineage['events'] if e['id'] == pair['earlier'])
    second = next(e for e in lineage['events'] if e['id'] == pair['later'])
    require(first['stage'] != second['stage'] and first['stage'] in [1,2,3] and second['stage'] in [1,2,3], 'distinct novel pair')
    vector = pair['programs']['intact']
    later, both = vector[:], vector[:]
    for i in range(len(vector)):
        if vector[i] == second['stage']:
            later[i] = both[i] = -1
        elif vector[i] == first['stage']:
            both[i] = -1
    pair['programs'].update(padded_remove_later=later, padded_remove_both=both)


def check(cases, contract, measurement, registry):
    wanted = expected(cases, contract)
    require((measurement, registry) == wanted, 'complete census lineage and 13-role request identity')
    return dict(samples=len(measurement['samples']), requests=len(registry['assays']),
                retained_pairs=sum(len(s.get('lineage', {}).get('pairs', [])) for s in measurement['samples']))


def worlds(adapter):
    from audit_campaign_adapter import recover
    from compact_evidence import sha
    c = json.loads((adapter/'contract.json').read_text()); cases = []
    for arm in ['endogenous', 'external', 'nonheritable', 'frozen']:
        raw = (adapter/'inputs'/('world_'+arm+'.json')).read_bytes()
        r = recover(adapter/'journals'/('world_'+arm), json.loads(raw), sha(raw), c, [])
        require(r['status'] == 'complete' and not r['unknown'] and not r['staging'], 'complete saved development world')
        cases.append(dict(name=arm, initial=r['initial']['state'], rows=r['rows'], final=r['checkpoint']['state']))
    return cases


def independent_report(measurement, registry, results, cases, contract):
    """Score arithmetic and world decisions independent of the prospective evaluator."""
    table = {}
    for result in results:
        i = result['request']['id']
        require(type(i) is int and 0 <= i < len(registry['assays']) and i not in table and result['request'] == registry['assays'][i], 'unique exact result identity')
        require(len(result['native']['results']) == 1, 'single native result envelope')
        n = result['native']['results'][0]
        require(n['status'] in ['complete','unfunded_copy'] and type(n['score']) is int, 'native score domain')
        table[i] = n
    rows, decisions = [], []
    c = contract['candidate']
    def functional(p):
        if p['empty_ablation'] or not p['all_assays_funded']: return False
        v = p['scores']; cost = c['acquisition_cost']; gain = c['minimum_paid_incremental_score_gain']
        return (bool(p['earlier']) and bool(p['later']) and p['earlier'] != p['later'] and
                len(p['transmissions']) == len(p['active_descendant_generations']) == 2 and
                all(t >= c['minimum_paid_transmissions_each'] for t in p['transmissions']) and
                all(len(set(g)) >= c['minimum_active_descendant_generations_each'] for g in p['active_descendant_generations']) and
                v['earlier_after'] >= v['earlier_before']+cost+gain and v['later_after'] >= v['later_before']+cost+gain and
                all(v['intact'] > v[r] for r in ['remove_earlier','remove_later','padded_remove_earlier']) and
                v['sham'] == v['restored'] == v['intact'] and
                v['intact']-v['padded_remove_later'] >= cost+gain and
                v['padded_remove_earlier']-v['padded_remove_both'] <= cost and
                v['intact']+v['padded_remove_both'] > v['padded_remove_later']+v['padded_remove_earlier'])
    for s in measurement['samples']:
        row = {k:s[k] for k in ['source','census','slot','status']}; row['pairs'] = []
        row['endpoint'] = {}
        for role, i in s.get('endpoint_assays', {}).items():
            row['endpoint'][role] = dict(score=table[i]['score'],status=table[i]['status']) if i in table else None
            if i is not None and i not in table: row['status'] = 'unmeasured'
        for pair in s.get('lineage', {}).get('pairs', []):
            p = {k:copy.deepcopy(pair[k]) for k in ['earlier','later','origins','transmissions','active_descendant_generations']}
            p['scores'] = {role:table[i]['score'] if i in table else None for role,i in pair['assays'].items()}
            p['empty_ablation'] = pair['inadmissible_empty_ablation']
            p['all_assays_funded'] = all(i in table and table[i]['status']=='complete' for i in pair['assays'].values())
            p['measurement_status'] = 'complete'
            if not p['all_assays_funded']: p['measurement_status'] = 'unfunded'
            if any(i not in table for i in pair['assays'].values()): p['measurement_status'] = 'unmeasured'
            if p['empty_ablation']: p['measurement_status'] = 'inadmissible_empty_ablation'
            row['pairs'].append(p)
        rows.append(row)
    for case in cases:
        selected = [s for s in rows if s['source']==case['name']]; arm = case['initial']['arm']
        require(len(selected)==10, 'fixed world census slots')
        pairs = [p for s in selected for p in s['pairs']]
        unknown = case['final']['tick']!=16 or any(s['status'] not in ['living','dead','empty'] for s in selected) or any(p['measurement_status']=='unmeasured' for p in pairs)
        good = [p for p in pairs if functional(p)]
        qualified = any(c['origins'][arm] is not None and p['origins']==[c['origins'][arm]]*2 for p in good)
        status = 'unknown' if unknown else 'mechanical_control_violation' if good and arm in ['nonheritable','frozen'] else 'complete'
        decisions.append(dict(world=case['name'],arm=arm,status=status,qualified=None if unknown else qualified))
    return dict(profile=measurement['profile'], scope='development_only', samples=rows, worlds=decisions)


def audit(package):
    from compact_evidence import Store, sha
    from audit_spool import ledger, Journal
    from audit_durability import read
    from audit_campaign_adapter import recover_values
    load = lambda name: json.loads((package/name).read_text())
    c=load('contract.json'); campaign=load('campaign_contract.json'); compact=load('adapter_store.json')
    store=Store(package/'adapter_evidence',compact['index_sha256'],c['adapter_parent']['seal_sha256']); store.verify_all()
    cases=worlds(store.root)
    measurement, registry = load('measurement.json'), load('registry.json')
    stats=check(cases,campaign,measurement,registry)
    native_package=package/'native_origin' if 'reuse_native_parent' in c else package
    if 'reuse_native_parent' in c:
        from compact_evidence import source_inventory
        source_inventory(native_package,c['reuse_native_parent']['seal_sha256'])
        require((package/'registry.json').read_bytes()==(native_package/'registry.json').read_bytes(), 'unchanged pre-native registry')
    states,admissions=ledger(native_package/'native_spool'); results=[]; work=0
    require(len(states)==(len(registry['assays'])+127)//128, 'complete fixed assay batch count')
    for index,(unit,state) in enumerate(states.items()):
        require(state['kind']=='released', 'retained compact native evidence')
        raw=(native_package/'native_spool/requests'/(unit+'.json')).read_bytes(); request=json.loads(raw)
        require(sha(raw)==state['request_sha256'] and request['registry']['sha256']==sha((package/'registry.json').read_bytes()), 'frozen registry and request')
        require(request['source_identity']==dict(binary_sha256=sha((native_package/'cumulative-campaign').read_bytes()),source_sha256=sha((native_package/'source_sha256.json').read_bytes())), 'native source identity')
        require(request['unit']=='assays' and request['source'] is None and request['fault'] is None and request['spec'] is None, 'only scheduled fresh assays')
        budget=json.loads((native_package/'contract.json').read_text())['limits']['native_rewrites']
        require(request['range']==dict(start=index*128,end=min((index+1)*128,len(registry['assays']))) and request['native_budget']==budget and request['execution_allowance']==budget-work, 'fixed batches and aggregate funding')
        compact=Store(native_package/'native_spool/compact'/unit,state['index_sha256'],state['seal_sha256']);compact.verify_all()
        values,head,staging=read(Journal(compact))
        recovered=recover_values(values,head,staging,request,sha(raw),json.loads((store.root/'contract.json').read_text()),registry['assays'])
        require(recovered['status']=='complete' and not staging and not recovered['unknown'], 'complete native assay denominator')
        receipt=json.loads((native_package/'native_spool/requests'/(unit+'.receipt.json')).read_text())
        require(not receipt['timed_out'] and receipt['returncode']==0, 'successful bounded child')
        work+=recovered['here_upper']; results.extend(recovered['results'])
    require(len(results)==len(registry['assays']), 'all frozen requests measured')
    require(load('report.json')==independent_report(measurement,registry,results,cases,campaign), 'independent prospective decision')
    prospective=load('prospective_pending.json')
    from plan_campaign import panel
    wanted=panel(campaign,47)
    require(prospective['panel']==wanted and len(prospective['samples'])==1880, 'untouched fixed scientific panel')
    expected_slots=[dict(world_id=w['world_id'],census=t,slot=s,status='not_started') for w in wanted['worlds'] for t in campaign['design']['census_ticks'] for s in range(2)]
    require(prospective['samples']==expected_slots, 'all unexecuted slots retained as unknown')
    require(all(not load('manifest.json')[k] for k in ['readiness_gate_complete','scientific_acceptance','evolution_demonstrated','campaign_launch_authorized']), 'no scientific admission')
    new_work=0 if 'reuse_native_parent' in c else work
    return dict(engineering_measurement_pass=True,**stats,native_work_lower=new_work,native_work_upper=new_work,assay_evidence_rewrites=work,
                scientific_worlds_executed=0,scientific_slots_pending=1880,maximum_admission_charge_bytes=max(admissions))


if __name__=='__main__':
    print(json.dumps(audit(Path(sys.argv[1])),indent=2,sort_keys=True))
