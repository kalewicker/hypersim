#!/usr/bin/env python3
"""Post-assay descriptive deduplication/ranking. Does not change v11 verdicts."""
import argparse
import collections
import hashlib
import json
import pathlib
import statistics


def load(path):
    return json.loads(path.read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=pathlib.Path)
    args = parser.parse_args()
    out = args.output
    audit = load(out / 'audit.json')
    if not audit['audit_passed'] or audit['engineering_preflight']:
        raise ValueError('complete audited scientific panel required')
    mutations = {p.stem: load(p) for p in (out/'mutations').glob('*.json')}
    novel = [c for c in audit['candidates'] if c['changed_executable']]
    leads = [c for c in novel if c['replicated_lead']]
    groups = collections.defaultdict(list)
    for c in leads:
        groups[(c['member'], mutations[c['id']]['mutant_class_hash'])].append(c)
    distinct = []
    for (member, class_hash), group in groups.items():
        c = group[0]
        distinct.append(dict(representative=c['id'], equivalent_draws=[x['id'] for x in group],
                             member=member, mutant_class_hash=class_hash,
                             decoded_spec=mutations[c['id']]['decoded_spec'],
                             functional_world_seeds=c['functional_world_seeds'],
                             resource_gated_cells=c['resource_gated_cells'],
                             mean_parent_paired_rate_delta=statistics.mean(x['parent_paired_rate_delta'] for x in c['paired'])))
    distinct.sort(key=lambda x: x['mean_parent_paired_rate_delta'], reverse=True)
    shortlist = []
    # Post-result interpretation examples, not additional scientific tests.
    for id_ in ('op11_member0_draw1', 'op07_member1_draw0', 'op12_member1_draw0'):
        c = next(c for c in leads if c['id'] == id_)
        member, records = c['member'], []
        for paired in c['paired']:
            seed = paired['seed']
            cycle = load(out/'cells'/f'{id_}_seed_{seed}.json')['cycle']
            parent = load(out/'controls'/f'seed_{seed}.json')['parent']
            late = len(cycle['rows'])//2
            records.append(dict(seed=seed, mean_productive_rate=paired['score']['productive_rate'],
                                parent_productive_rate=paired['parent_productive_rate'],
                                late_productive_delta=(sum(r['fates'][member]['productive'] for r in cycle['rows'][late:])-
                                                       sum(r['fates'][member]['productive'] for r in parent['rows'][late:]))/late,
                                late_productive_by_member=[sum(r['fates'][m]['productive'] for r in cycle['rows'][late:]) for m in range(3)],
                                final_world_nodes=cycle['rows'][-1]['world_nodes'],
                                focus_creation_spend=sum(r['creation_spend'][member] for r in cycle['rows'])))
        shortlist.append(dict(id=id_, member=member, records=records,
                              mean_productive_rate=statistics.mean(r['mean_productive_rate'] for r in records),
                              mean_parent_productive_rate=statistics.mean(r['parent_productive_rate'] for r in records),
                              mean_late_productive_delta=statistics.mean(r['late_productive_delta'] for r in records)))
    report = dict(description='Post-result descriptive analysis; primary thresholds/panel unchanged.',
                  audit_sha256=hashlib.sha256((out/'audit.json').read_bytes()).hexdigest(),
                  script_sha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
                  candidates=180, executable_candidates=sum(c['compiled'] for c in audit['candidates']),
                  unchanged_executable_candidates=sum(c['compiled'] and not c['changed_executable'] for c in audit['candidates']),
                  changed_executable_candidates=len(novel),
                  distinct_changed_variants=len({(c['member'], mutations[c['id']]['mutant_class_hash']) for c in novel}),
                  malformed_candidates=sum(not c['decoded'] for c in audit['candidates']),
                  functional_cells=sum(len(c['functional_world_seeds']) for c in novel),
                  enumeration_censored_cycles=sum(c['enumeration_censored_cells'] for c in audit['candidates']),
                  replicated_lead_draws=len(leads), distinct_lead_variants=len(distinct),
                  lead_operators=len({c['operator'] for c in leads}),
                  resource_gated_lead_cells=sum(sum(p['score']['resource_gate_observations'] > 0 for p in c['paired']
                                                       if p['score']['bounded_functional_candidate']) for c in leads),
                  distinct_leads_ranked_by_paired_full_rate=distinct, shortlist=shortlist)
    with (out/'interpretation.json').open('x') as file:
        json.dump(report, file, indent=2)
        file.write('\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ('distinct_leads_ranked_by_paired_full_rate', 'shortlist')}, indent=2))


if __name__ == '__main__':
    main()
