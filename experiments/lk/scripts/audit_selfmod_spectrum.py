#!/usr/bin/env python3
"""Independent v11 accounting/gate/verdict audit; does not reimplement matching."""
import argparse
import collections
import copy
import hashlib
import json
import math
import pathlib
import tarfile

FATE = ('generated', 'truncated', 'rejected', 'sterile', 'productive')
OPERATORS = ('jitter_guard', 'mutate_label', 'add_pnode_constraint', 'drop_pnode_constraint',
             'add_lhs_var', 'drop_lhs_var', 'add_pedge', 'drop_pedge', 'add_rhs_create',
             'drop_rhs_create', 'add_rhs_edge', 'drop_rhs_edge', 'toggle_delete',
             'grow_arity', 'shrink_arity')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text())


def fate_check(fate):
    require(set(fate) == set(FATE), 'fate schema')
    require(all(type(fate[k]) is int and fate[k] >= 0 for k in FATE), 'negative/noninteger fate')
    require(fate['generated'] == sum(fate[k] for k in FATE[1:]), 'candidate fate identity')


def estimate(spec, labels, nodes, edges):
    covered = set()
    cost = 1.0
    for edge in spec['pedges']:
        selectivity = 1.0
        for ref in edge:
            label = spec['var_labels'][ref]
            if label is not None:
                selectivity *= min((labels[label] if label < len(labels) else 0) / max(nodes, 1), 1)
        cost *= max(max(edges, 1) * math.factorial(len(edge)) * selectivity, 1.0)
        covered.update(edge)
    for i in range(spec['n_vars']):
        if i not in covered:
            label = spec['var_labels'][i]
            count = nodes if label is None else labels[label] if label < len(labels) else 0
            cost *= max(count, 1)
    return cost


def gate_check(gate, spec, budget, labels, nodes, edges, cfg):
    grower = len(spec['creates']) > len(spec['deletes']) or len(spec['redges']) > len(spec['pedges'])
    creation = grower and budget <= 0
    node = grower and bool(spec['creates']) and nodes >= cfg['world_ceiling']
    edge = grower and len(spec['redges']) > len(spec['pedges']) and edges >= cfg['edge_ceiling']
    require([gate['creation_blocked'], gate['node_ceiling_blocked'], gate['edge_ceiling_blocked']]
            == [creation, node, edge], 'resource gate reconstruction')
    expected = None if creation or node or edge else estimate(spec, labels, nodes, edges)
    actual = gate['enumeration_estimate']
    require(actual is None if expected is None else actual is not None and
            math.isclose(actual, expected, rel_tol=1e-12, abs_tol=1e-9), 'enumeration estimate reconstruction')
    require(gate['enumeration_skipped'] == (expected is not None and expected > cfg['match_enum_budget']),
            'enumeration censor flag')


def root_order(store):
    roots = []
    for i, slot in enumerate(store['nodes']):
        value = slot['value']
        if value is not None and value['label'] == 1000:
            props = dict(value['properties'])
            roots.append(((slot['version'] << 32) | i, props[204]['Int']))
    require(sorted(m for _, m in roots) == [0, 1, 2], 'initial three original members')
    return [member for _, member in sorted(roots)]


def cycle_check(cycle, cfg, specs):
    horizon = cfg['steps']
    require(cycle['requested_steps'] == horizon and cycle['completed_steps'] == horizon
            and cycle['error'] is None and len(cycle['rows']) == horizon, 'incomplete cycle')
    require(cycle['initial_capital'] == [cfg['e0']] * 3 and cycle['initial_free'] == 500
            and cycle['initial_influx'] == 680, 'initial funding')
    capital, budgets, live = [cfg['e0']] * 3, [cfg['creation_budget_init']] * 3, [True] * 3
    free, dissipated, influx = 500, 0, 680
    nodes, edges = cfg['world_nodes'], cfg['world_edges']
    order = root_order(cycle['initial_rules'])
    for step, row in enumerate(cycle['rows'], 1):
        require(row['step'] == step, 'step sequence')
        influx += cfg['influx']
        dissipated += max(free + cfg['influx'] - cfg['free_cap'], 0)
        free = min(free + cfg['influx'], cfg['free_cap'])
        for m in range(3):
            fate_check(row['fates'][m])
            cost = cfg['rent'] if live[m] and capital[m] >= cfg['rent'] else 0
            require(row['costs'][m] == cost, 'rent reconstruction')
            if live[m] and capital[m] < cfg['rent']:
                free += capital[m]
                capital[m], live[m] = None, False
            elif live[m]:
                capital[m] -= cost
                dissipated += cost
            require(row['live'][m] == live[m], 'starvation/no birth identity')
            require(live[m] or not any(row['fates'][m].values()), 'dead root fate')
        require(sum(row['world_label_counts_before']) == nodes, 'label count total')
        expected_members = {m for m in range(3) if live[m] and specs[m] is not None}
        require({g['member'] for g in row['gates']} == expected_members
                and len(row['gates']) == len(expected_members), 'decoded gate coverage')
        require(row['decode_failures'] == sum(live[m] and specs[m] is None for m in range(3)), 'decode failures')
        for gate in row['gates']:
            m = gate['member']
            gate_check(gate, specs[m], budgets[m], row['world_label_counts_before'], nodes, edges, cfg)
            if gate['enumeration_skipped'] or gate['creation_blocked'] or gate['node_ceiling_blocked'] or gate['edge_ceiling_blocked']:
                require(not any(row['fates'][m].values()), 'gated candidate leakage')
        products = [row['fates'][m]['productive'] for m in range(3)]
        total = sum(products)
        pool = min(cfg['income_pool'], free) if total else 0
        income = [pool * p // total if total else 0 for p in products]
        remainder = pool - sum(income)
        for m in order:
            if products[m] and remainder:
                income[m] += 1
                remainder -= 1
        require(row['income'] == income and remainder == 0, 'proportional income reconstruction')
        free -= sum(income)
        for m in range(3):
            require(type(row['creation_spend'][m]) is int and row['creation_spend'][m] >= 0, 'creation spend')
            if live[m]:
                capital[m] += income[m]
                budgets[m] = min(budgets[m] - row['creation_spend'][m] + cfg['creation_refill'], cfg['creation_budget_max'])
            else:
                require(row['creation_spend'][m] == 0, 'dead root spend')
                budgets[m] = None
        require(row['capital'] == capital and row['creation_remaining'] == budgets, 'capital/creation balances')
        bound = sum(c for c in capital if c is not None)
        require(row['ledger'] == dict(free=free, bound=bound, cumulative_influx=influx,
                                     cumulative_dissipated=dissipated, audit_residual=0), 'flow ledger reconstruction')
        require(influx == free + bound + dissipated, 'cash conservation')
        nodes, edges = row['world_nodes'], row['world_edges']
        require(nodes <= cfg['node_explosion_cap'], 'hard cap incomplete')


def score(cycle, member, executable, changed):
    rows, horizon = cycle['rows'], cycle['requested_steps']
    half = horizon // 2
    total = {k: sum(r['fates'][member][k] for r in rows) for k in FATE}
    late = {k: sum(r['fates'][member][k] for r in rows[half:]) for k in FATE}
    enum = sum(g['enumeration_skipped'] for r in rows for g in r['gates'])
    resource = sum(g['creation_blocked'] or g['node_ceiling_blocked'] or g['edge_ceiling_blocked']
                   for r in rows for g in r['gates'])
    complete = cycle['error'] is None and len(rows) == horizon
    final_live = bool(rows) and all(rows[-1]['live'])
    active = len(rows) == horizon and all(sum(f['productive'] for r in rows[i:i+horizon//8]
                        for f in r['fates']) > 0 for i in range(half, horizon, horizon//8))
    measured = executable and complete and enum == 0
    return dict(executable=executable, changed_executable=changed, complete=complete,
                total_fate=total, late_fate=late, full_horizon_denominator=horizon,
                late_horizon_denominator=horizon-half,
                productive_rate=total['productive']/horizon if measured else None,
                late_productive_rate=late['productive']/(horizon-half) if measured else None,
                enumeration_skips=enum, resource_gate_observations=resource,
                all_members_final_live=final_live, late_cycle_windows_active=active,
                final_capital=rows[-1]['capital'][member] if rows else None,
                focus_death_step=next((r['step'] for r in rows if not r['live'][member]), None),
                bounded_functional_candidate=changed and complete and enum == 0 and final_live
                and late['productive'] > 0 and active)


def isolated_check(observations, parent, cfg, spec):
    require(len(observations) == len(parent), 'isolated opportunity count')
    steps = 0
    for obs, reference in zip(observations, parent):
        for key in ('reference_step', 'source_world_hash', 'before_rng_probe', 'world_nodes_before',
                    'world_edges_before', 'world_label_counts_before'):
            require(obs[key] == reference[key], 'paired reference mismatch')
        if spec is None:
            require(obs['status'] in ('decode_invalid', 'compile_invalid') and
                    obs['completed_steps'] == 0 and obs['fate'] is None, 'invalid isolated measurement')
        else:
            require(obs['completed_steps'] == 1 and len(obs['gates']) == 1, 'isolated execution')
            gate_check(obs['gates'][0], spec, cfg['creation_budget_init'], obs['world_label_counts_before'],
                       obs['world_nodes_before'], obs['world_edges_before'], cfg)
            censored = obs['gates'][0]['enumeration_skipped']
            require(obs['status'] == ('enumeration_unmeasured' if censored else 'measured'), 'isolated censor status')
            if censored:
                require(obs['fate'] is None, 'censored fate must be null')
            else:
                fate_check(obs['fate'])
        steps += obs['completed_steps']
    return steps


def audit(out):
    manifest, summary, status = (load(out / p) for p in ('manifest.json', 'summary.json', 'status.json'))
    seeds, draws, horizon = manifest['world_seeds'], manifest['parameter_seeds'], manifest['horizon']
    preflight = manifest['engineering_preflight']
    require(manifest['protocol'] == 'selfmod-v11-spectrum-v1' and not manifest['fission_enabled'], 'protocol')
    require(seeds == ([7, 42] if preflight else [1, 2, 3, 7, 42, 99, 123, 777]) and
            draws == ([0x5E110001] if preflight else [0x5E110001, 0x5E110003, 0x5E110005, 0x5E110007]) and
            horizon == (32 if preflight else 256) and manifest['operators'] == list(OPERATORS), 'frozen panel')
    provenance = load(out / 'instrument_provenance.json')
    require(sha(out / 'selfmod-spectrum') == provenance['binary_sha256'] and
            sha(out / 'source.tar.gz') == provenance['source_archive_sha256'], 'frozen instrument hashes')
    with tarfile.open(out / 'source.tar.gz') as tar:
        require(set(tar.getnames()) == set(provenance['sources_sha256']), 'source archive inventory')
        for name, expected in provenance['sources_sha256'].items():
            require(hashlib.sha256(tar.extractfile(name).read()).hexdigest() == expected, 'source hash ' + name)
    cfgs = {cfg['seed']: cfg for cfg in manifest['configs']}
    for seed, cfg in cfgs.items():
        require(seed in seeds and cfg['steps'] == horizon and cfg['tie_break'] == 'random' and
                cfg['match_cost_coeff'] == 0 and cfg['copy_number'] == 1 and
                cfg['match_enum_budget'] == 50000 and cfg['match_cap_per_rule'] == 64 and
                cfg['truncation_mode'] == 'canonical' and cfg['economy'] == 'flux' and
                cfg['income_mode'] == 'proportional' and all(cfg[k] == 0 for k in ('mu', 'abio', 'p_dup', 'p_del')),
                'corrected instrument config')
    controls, parent_specs = {}, None
    actual_steps = 0
    baseline = []
    for seed in seeds:
        control = load(out / 'controls' / f'seed_{seed}.json')
        require(control['parent'] == control['clone'] and control['parent_isolated'] == control['clone_isolated'],
                'faithful clone mismatch')
        controls[seed] = control
    mutations = {}
    for op in range(15):
        for m in range(3):
            for d in range(len(draws)):
                id_ = f'op{op:02}_member{m}_draw{d}'
                edit = load(out / 'mutations' / f'{id_}.json')
                require(edit['id'] == id_ and edit['operator'] == op and edit['member'] == m and
                        edit['parameter_seed'] == draws[d] and edit['operator_name'] == OPERATORS[op], 'edit identity')
                require(len(edit['parameters']) == 5 and all(type(p) is int for p in edit['parameters']), 'parameter block')
                require(edit['encoded_changed'] == (edit['before_material'] != edit['after_material']), 'encoded change')
                changed = edit['compiled'] and edit['mutant_class_hash'] != edit['parent_class_hash']
                require(edit['changed_executable'] == changed, 'novel executable classification')
                mutations[id_] = edit
    parent_specs = [mutations[f'op00_member{m}_draw0']['parent_spec'] for m in range(3)]
    for seed, control in controls.items():
        cycle_check(control['parent'], cfgs[seed], parent_specs)
        expected = [score(control['parent'], m, True, False) for m in range(3)]
        require(control['parent_scores'] == expected, 'parent score reconstruction')
        isolated_steps = sum(isolated_check(obs, obs, cfgs[seed], parent_specs[m])
                             for m, obs in enumerate(control['parent_isolated']))
        actual_steps += 2 * (horizon + isolated_steps)
        baseline.append(dict(seed=seed, all_final_live=all(control['parent']['rows'][-1]['live']), scores=expected))
        require([r['cut'] for r in control['reference_worlds']] == manifest['reference_cuts'], 'reference cuts')
        for reference in control['reference_worlds']:
            cut, world = reference['cut'], reference['world']
            require(world['step'] == cut, 'archived reference step')
            labels = [0] * cfgs[seed]['n_labels']
            for slot in world['nodes']:
                if slot['value'] is not None:
                    labels[slot['value']['label']] += 1
            edges = sum(slot['value'] is not None for slot in world['edges'])
            for member in range(3):
                obs = next(o for o in control['parent_isolated'][member] if o['reference_step'] == cut)
                require(obs['world_label_counts_before'] == labels and obs['world_nodes_before'] == sum(labels)
                        and obs['world_edges_before'] == edges and obs['before_rng_probe'] == reference['rng_probe'],
                        'reference stores/counts/stream')
                if cut:
                    require(obs['source_world_hash'] == control['parent']['rows'][cut-1]['world_hash'] and
                            obs['before_rng_probe'] == control['parent']['rows'][cut-1]['rng_probe'], 'reference trajectory')
    engaged = [any(obs['status'] == 'measured' and obs['fate']['productive'] > 0
                   for control in controls.values() for obs in control['parent_isolated'][m]) for m in range(3)]
    require(all(engaged) and summary['parent_roles_engaged'] == engaged, 'parent engagement')
    results, compact, iso_statuses = [], [], collections.Counter()
    for id_, edit in mutations.items():
        member = edit['member']
        specs = parent_specs.copy()
        specs[member] = edit['decoded_spec'] if edit['compiled'] else None
        successes, isolated_positive, paired = [], [], []
        gate_cells, enum_cells = 0, 0
        for seed in seeds:
            cell_id = f'{id_}_seed_{seed}'
            cell = load(out / 'cells' / f'{cell_id}.json')
            require(cell['id'] == cell_id and cell['mutation_id'] == id_ and cell['seed'] == seed
                    and cell['member'] == member, 'cell identity')
            require(cell['cycle']['initial_world'] == controls[seed]['parent']['initial_world'] and
                    cell['cycle']['initial_rules'] == edit['after_material'], 'paired initial conditions')
            cycle_check(cell['cycle'], cfgs[seed], specs)
            reconstructed = score(cell['cycle'], member, edit['compiled'], edit['changed_executable'])
            require(cell['score'] == reconstructed, 'mutation score reconstruction')
            iso_steps = isolated_check(cell['isolated'], controls[seed]['parent_isolated'][member], cfgs[seed], specs[member])
            actual_steps += horizon + iso_steps
            compact.append(dict(id=cell_id, mutation_id=id_, seed=seed, member=member,
                                score=reconstructed, completed_steps=horizon, isolated_steps=iso_steps))
            iso_statuses.update(o['status'] for o in cell['isolated'])
            if any(o['status'] == 'measured' and o['fate']['productive'] > 0 for o in cell['isolated']):
                isolated_positive.append(seed)
            if reconstructed['bounded_functional_candidate']:
                successes.append(seed)
            gate_cells += reconstructed['resource_gate_observations'] > 0
            enum_cells += reconstructed['enumeration_skips'] > 0
            parent_score = controls[seed]['parent_scores'][member]
            delta = None if reconstructed['productive_rate'] is None or parent_score['productive_rate'] is None else reconstructed['productive_rate'] - parent_score['productive_rate']
            paired.append(dict(seed=seed, score=reconstructed, parent_productive_rate=parent_score['productive_rate'],
                               parent_paired_rate_delta=delta,
                               isolated_productive=sum(o['fate']['productive'] for o in cell['isolated'] if o['status'] == 'measured'),
                               parent_isolated_productive=sum(o['fate']['productive'] for o in controls[seed]['parent_isolated'][member] if o['status'] == 'measured'),
                               income=sum(r['income'][member] for r in cell['cycle']['rows']),
                               rent=sum(r['costs'][member] for r in cell['cycle']['rows'])))
        results.append(dict(id=id_, operator=edit['operator_name'], member=member, draw=int(id_.rsplit('draw', 1)[1]),
                            decoded=edit['decoded_spec'] is not None, compiled=edit['compiled'],
                            changed_executable=edit['changed_executable'], encoded_changed=edit['encoded_changed'],
                            operator_reported_change=edit['operator_reported_change'],
                            functional_world_seeds=successes, isolated_positive_world_seeds=isolated_positive,
                            replicated_lead=not preflight and len(successes) >= 6,
                            resource_gated_cells=gate_cells, enumeration_censored_cells=enum_cells, paired=paired))
    require(summary['cells'] == compact and summary['mutation_candidates'] == len(mutations) and
            summary['mutation_cells'] == len(compact) == manifest['mutation_cells_requested'], 'summary/panel counts')
    require(actual_steps == summary['actual_advancing_steps'] == status['actual_advancing_steps'] and
            status['state'] == 'complete' and not summary['training_launched'] and
            not summary['evolution_probe_launched'], 'completion/step accounting')
    require(len(list((out/'cells').glob('*.json'))) == len(compact) and
            len(list((out/'mutations').glob('*.json'))) == len(mutations), 'no duplicate/extra cells')
    operators = []
    for op in OPERATORS:
        group = [r for r in results if r['operator'] == op]
        operators.append(dict(operator=op, candidates=len(group), decoded=sum(r['decoded'] for r in group),
                              compiled=sum(r['compiled'] for r in group), changed=sum(r['changed_executable'] for r in group),
                              functional_cells=sum(len(r['functional_world_seeds']) for r in group),
                              replicated_leads=sum(r['replicated_lead'] for r in group),
                              isolated_positive_candidates=sum(bool(r['isolated_positive_world_seeds']) for r in group),
                              enumeration_censored_cells=sum(r['enumeration_censored_cells'] for r in group)))
    return dict(audit_passed=True, engineering_preflight=preflight, actual_advancing_steps=actual_steps,
                auditor_sha256=sha(pathlib.Path(__file__)),
                mutation_candidates=len(mutations), mutation_cells=len(compact),
                isolated_statuses=dict(iso_statuses), controls=baseline, operators=operators, candidates=results,
                scope='Independent reconstruction of accounting, resource/estimate gates, panel and verdicts. Native matching, decoding, effects and class hashes are not independently reimplemented.',
                evidence_sha256={str(p.relative_to(out)): sha(p) for p in sorted(out.rglob('*'))
                                 if p.is_file() and p.name not in ('audit.json', 'audit.log')})


def adversarial_check(out):
    manifest = load(out / 'manifest.json')
    cfg = manifest['configs'][0]
    control = load(out / 'controls' / f"seed_{cfg['seed']}.json")
    specs = [load(out / 'mutations' / f'op00_member{m}_draw0.json')['parent_spec'] for m in range(3)]
    rejected = []
    for label, alter in (
        ('rent', lambda c: c['rows'][0]['costs'].__setitem__(0, 0)),
        ('fate', lambda c: c['rows'][0]['fates'][0].__setitem__('productive', 999)),
        ('cash', lambda c: c['rows'][0]['ledger'].__setitem__('free', 999)),
        ('creation_balance', lambda c: c['rows'][0]['creation_remaining'].__setitem__(0, 999)),
        ('enumeration', lambda c: c['rows'][0]['gates'][0].__setitem__('enumeration_skipped', True)),
        ('missing_step', lambda c: c['rows'].pop()),
        ('birth', lambda c: c['rows'][0]['live'].__setitem__(0, False)),
    ):
        damaged = copy.deepcopy(control['parent'])
        alter(damaged)
        try:
            cycle_check(damaged, cfg, specs)
        except ValueError:
            rejected.append(label)
        else:
            raise ValueError('auditor accepted adversarial ' + label)
    return rejected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=pathlib.Path)
    args = parser.parse_args()
    report = audit(args.output)
    report['adversarial_corruptions_rejected'] = adversarial_check(args.output)
    with (args.output / 'audit.json').open('x') as file:
        json.dump(report, file, indent=2)
        file.write('\n')
    print(json.dumps({k: report[k] for k in ('audit_passed', 'actual_advancing_steps', 'mutation_candidates',
                                            'mutation_cells', 'isolated_statuses', 'operators',
                                            'adversarial_corruptions_rejected')}, indent=2))


if __name__ == '__main__':
    main()
