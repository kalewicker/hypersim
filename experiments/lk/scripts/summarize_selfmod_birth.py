#!/usr/bin/env python3
"""Post-result explanation of v12 criteria; never changes the primary verdict."""
import argparse
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
    audit, replay = load(out/'audit.json'), load(out/'replay_verification.json')
    if not audit['audit_passed'] or audit['engineering_preflight'] or not replay['passed']:
        raise ValueError('audited scientific panel and exact replay required')
    records = []
    parent_class = load(out/'v11_selected_mutation.json')['parent_class_hash']
    for paired in audit['paired']:
        seed = paired['seed']
        for arm, score in paired['scores'].items():
            directory = out/f'seed_{seed}'/arm
            birth = load(directory/'birth.json')
            final = load(directory/'final.json')
            selected = birth['selection']
            genes = {g['id']: g for g in final['genes']}
            ids = []
            for i, node in enumerate(final['rules']['nodes']):
                if node['value'] is not None and node['value']['label'] == 1000:
                    ids.append((node['version'] << 32) | i)
            r1_ancestral_copies = sum(genes[id_]['member'] == 0 and not genes[id_]['lineage'] for id_ in ids)
            original_class_copies = sum(genes[id_]['member'] == 0 and genes[id_]['class_hash'] == parent_class for id_ in ids)
            totals = score['totals']
            reasons = []
            if not selected['cancelled']:
                if not selected['executable']: reasons.append('initial_child_invalid')
                if not score['complete']: reasons.append('incomplete')
                if totals['enumeration_skips']: reasons.append('enumeration_unmeasured')
                if not score['whole_cycle_persistent']: reasons.append('cycle_not_persistent')
                if not score['final_lineage_copies']: reasons.append('lineage_lost')
                if not all(totals['late_lineage_windows']): reasons.append('late_lineage_gap')
                if totals['late_lineage_income'] < totals['late_lineage_rent']: reasons.append('late_income_below_rent')
            records.append(dict(seed=seed, arm=arm, established=score['bounded_establishment'],
                                final_lineage_copies=score['final_lineage_copies'],
                                late_income_minus_rent=totals['late_lineage_income']-totals['late_lineage_rent'],
                                late_lineage_windows=totals['late_lineage_windows'], failure_reasons=reasons,
                                parent_death_step=totals['parent_death_step'], child_death_step=totals['child_death_step'],
                                other_R1_ancestry_copies=r1_ancestral_copies,
                                original_R1_class_copies=original_class_copies,
                                final_nodes=sum(n['value'] is not None for n in final['world']['nodes']),
                                final_edges=sum(e['value'] is not None for e in final['world']['edges']),
                                lineage_per_copy_rate=score['lineage_per_copy_rate'],
                                lineage_productive_rate=score['lineage_productive_rate']))
    aggregates = {}
    for arm in ('faithful', 'prune', 'unscreened', 'refund'):
        group = [r for r in records if r['arm'] == arm]
        measured = [r['lineage_per_copy_rate'] for r in group if r['lineage_per_copy_rate'] is not None]
        aggregates[arm] = dict(mean_per_copy_productive_rate=statistics.mean(measured) if measured else None,
                               measured_rate_denominator=len(measured),
                               original_R1_class_absent_seeds=[r['seed'] for r in group if not r['original_R1_class_copies']],
                               alive_but_late_income_below_rent=[r['seed'] for r in group if r['final_lineage_copies'] and r['late_income_minus_rent'] < 0])
    report = dict(description='Post-result failure attribution and ancestry descriptions. Primary v12 verdicts unchanged.',
                  audit_sha256=hashlib.sha256((out/'audit.json').read_bytes()).hexdigest(),
                  script_sha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
                  aggregates=aggregates, records=records)
    post = out/'post_analysis'
    post.mkdir(exist_ok=False)
    (post/pathlib.Path(__file__).name).write_bytes(pathlib.Path(__file__).read_bytes())
    with (out/'interpretation.json').open('x') as file:
        json.dump(report, file, indent=2)
        file.write('\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
