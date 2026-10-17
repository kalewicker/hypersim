"""Complete prospective census mapping. Callers must authenticate native histories first."""
import copy
from audit_population import score
from plan_versions import histories, descent
from plan_campaign import classify_world

PROFILE = 'cumulative-campaign-measurement-readiness-v1'
NATIVE_PROFILE = 'cumulative-campaign-adapter-readiness-v1'
ENDPOINT = ['intact', 'sham', 'restored']


def extend(lineage):
    """Preserve acquisition identities; pad every copy of the selected organization."""
    result = copy.deepcopy(lineage)
    events = {e['id']: e for e in result['events']}
    for pair in result['pairs']:
        a, b = (events[pair[k]]['stage'] for k in ['earlier', 'later'])
        if a == b or a not in [1, 2, 3] or b not in [1, 2, 3]:
            raise ValueError('distinct novel nonfounder organizations required')
        stages = pair['programs']['intact']
        pair['programs']['padded_remove_later'] = [-1 if s == b else s for s in stages]
        pair['programs']['padded_remove_both'] = [-1 if s in (a, b) else s for s in stages]
    return result


def plan(cases, contract):
    samples, requests, cache = [], [], {}
    if len({c['name'] for c in cases}) != len(cases):
        raise ValueError('duplicate world identity')
    def register(role, stages):
        if not stages:
            return None
        if len(stages) > 8 or any(type(s) is not int or not -1 <= s <= 4 for s in stages):
            raise ValueError('program domain')
        key = role, tuple(stages)
        if key not in cache:
            cache[key] = len(requests)
            requests.append(dict(id=len(requests), role=role, stages=stages[:]))
        return cache[key]
    for case in cases:
        for tick in contract['design']['census_ticks']:
            available = tick <= case['final']['tick']
            if available:
                frame, states, hs = histories(case, tick)
                ranking = sorted([[i, score(frame['seed'], tick, 2, i)] for i in states], key=lambda x: (x[1], x[0]))
            else:
                ranking = []
            for slot in range(contract['design']['slots_per_census']):
                sample = dict(source=case['name'], census=tick, slot=slot, ranking=ranking)
                if not available:
                    sample['status'] = 'unmeasured'
                elif slot >= len(ranking):
                    sample['status'] = 'empty'
                else:
                    leaf = ranking[slot][0]
                    lineage = extend(descent(states, hs, leaf))
                    if len(lineage['pairs']) > 3:
                        raise ValueError('complete novel pair bound')
                    stages = [s for _, s in states[leaf]['program']['genes']]
                    sample.update(cell=leaf, status='living' if states[leaf]['alive'] else 'dead', lineage=lineage,
                                  endpoint_assays={r: register(r, stages) for r in ENDPOINT})
                    for pair in lineage['pairs']:
                        pair['assays'] = {r: register(r, pair['programs'][r]) for r in contract['candidate']['roles']}
                        pair['inadmissible_empty_ablation'] = any(i is None for i in pair['assays'].values())
                        del pair['programs']
                samples.append(sample)
    if len(requests) > 78960:
        raise ValueError('global registry bound')
    return dict(profile=PROFILE, samples=samples), dict(profile=NATIVE_PROFILE, assays=requests)


def evaluate(measurement, registry, results, cases, contract):
    """Results are independently audited native global-result records, never bare scores."""
    items = registry['assays']; lookup = {}
    for result in results:
        i = result['request']['id']
        if type(i) is not int or not 0 <= i < len(items) or result['request'] != items[i] or i in lookup:
            raise ValueError('result identity or duplicate')
        envelope = result['native']
        if len(envelope['results']) != 1:
            raise ValueError('exactly one native result per global request')
        native = envelope['results'][0]
        if native['status'] not in ['complete', 'unfunded_copy'] or type(native['score']) is not int:
            raise ValueError('result status/score')
        lookup[i] = native
    rows = []
    for sample in measurement['samples']:
        row = {k: sample[k] for k in ['source', 'census', 'slot', 'status']}
        row['pairs'] = []
        endpoint_ids = sample.get('endpoint_assays', {})
        row['endpoint'] = {r: None if i not in lookup else dict(score=lookup[i]['score'], status=lookup[i]['status']) for r, i in endpoint_ids.items()}
        if any(i is not None and i not in lookup for i in endpoint_ids.values()):
            row['status'] = 'unmeasured'
        for pair in sample.get('lineage', {}).get('pairs', []):
            p = {k: copy.deepcopy(pair[k]) for k in ['earlier', 'later', 'origins', 'transmissions', 'active_descendant_generations']}
            ids = pair['assays']
            p['scores'] = {r: lookup[i]['score'] if i in lookup else None for r, i in ids.items()}
            p['all_assays_funded'] = all(i in lookup and lookup[i]['status'] == 'complete' for i in ids.values())
            p['empty_ablation'] = pair['inadmissible_empty_ablation']
            p['measurement_status'] = ('inadmissible_empty_ablation' if p['empty_ablation'] else
                                       'unmeasured' if any(i not in lookup for i in ids.values()) else
                                       'complete' if p['all_assays_funded'] else 'unfunded')
            row['pairs'].append(p)
        rows.append(row)
    worlds = []
    for case in cases:
        samples = [r for r in rows if r['source'] == case['name']]
        state = 'complete' if case['final']['tick'] == contract['design']['world']['ticks'] else 'partial'
        decision = classify_world(dict(status=state, samples=samples), case['initial']['arm'], contract)
        worlds.append(dict(world=case['name'], arm=case['initial']['arm'], **decision))
    return dict(profile=PROFILE, scope='development_only', samples=rows, worlds=worlds)
