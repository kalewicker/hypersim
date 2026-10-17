#!/usr/bin/env python3
"""Reconstruct v12 money, inheritance, genealogy, gates and establishment."""
import argparse
import collections
import copy
import hashlib
import json
import pathlib
import statistics
import tarfile
from audit_selfmod_spectrum import require, load, sha, fate_check, gate_check

ARMS = ['faithful', 'prune', 'unscreened', 'refund']
SEEDS = [11, 17, 23, 31, 47, 61, 83, 101]


def canonical_spec(spec):
    if spec is None:
        return None
    result = copy.deepcopy(spec)
    for key in ('pedges', 'redges', 'relabels'):
        result[key].sort()
    result['deletes'] = sorted(set(result['deletes']))
    result['guards'].sort(key=lambda g: json.dumps(g, sort_keys=True))
    return result


def rows(path):
    with path.open() as file:
        for line in file:
            yield json.loads(line)


def graph_stocks(snapshot):
    result = {}
    for i, slot in enumerate(snapshot['rules']['nodes']):
        value = slot['value']
        if value is not None and value['label'] == 1000:
            props = dict(value['properties'])
            result[(slot['version'] << 32) | i] = dict(capital=props[200]['Int'], budget=props[202]['Int'],
                                                       member=props[204]['Int'], seq=props[201]['Int'])
    return result


class State:
    def __init__(self, snapshot):
        self.cfg = snapshot['cfg']
        self.genes = {g['id']: copy.deepcopy(g) for g in snapshot['genes']}
        self.balances = {id_: dict(capital=s['capital'], budget=s['budget']) for id_, s in graph_stocks(snapshot).items()}
        self.free = snapshot['ledger']['free']
        self.influx = snapshot['ledger']['cumulative_influx']
        self.dissipated = snapshot['ledger']['cumulative_dissipated']
        self.seq = snapshot['seq']
        self.step = snapshot['world']['step']
        self.nodes = sum(s['value'] is not None for s in snapshot['world']['nodes'])
        self.edges = sum(s['value'] is not None for s in snapshot['world']['edges'])
        self.pos = snapshot['rng_word_pos']
        self.probe = snapshot['rng_probe']

    def cash(self):
        bound = sum(v['capital'] for v in self.balances.values())
        require(self.influx == bound + self.free + self.dissipated, 'token conservation')
        return dict(free=self.free, bound=bound, cumulative_influx=self.influx,
                    cumulative_dissipated=self.dissipated, audit_residual=0)

    def match_snapshot(self, snapshot, full_rng=True):
        actual = {id_: dict(capital=s['capital'], budget=s['budget']) for id_, s in graph_stocks(snapshot).items()}
        require(actual == self.balances, 'snapshot capital/budget stocks')
        require({g['id']: g for g in snapshot['genes']} == self.genes, 'snapshot genealogy')
        require(snapshot['ledger'] == self.cash() and snapshot['seq'] == self.seq and
                snapshot['world']['step'] == self.step, 'snapshot flow/sequence/step')
        require(sum(s['value'] is not None for s in snapshot['world']['nodes']) == self.nodes and
                sum(s['value'] is not None for s in snapshot['world']['edges']) == self.edges, 'snapshot world counts')
        if full_rng:
            require(snapshot['rng_word_pos'] == self.pos and snapshot['rng_probe'] == self.probe, 'snapshot RNG')
        for id_, stock in graph_stocks(snapshot).items():
            require(stock['member'] == self.genes[id_]['member'], 'genome/member attribution')

    def object(self, row):
        cfg = self.cfg
        require(row['absolute_step'] == self.step + 1 and row['world_nodes_before'] == self.nodes
                and row['world_edges_before'] == self.edges, 'world step/count pairing')
        require([r['id'] for r in row['roots']] == sorted(self.balances), 'root coverage/order')
        require(sum(row['label_counts_before']) == self.nodes, 'label total')
        self.influx += cfg['influx']
        self.dissipated += max(self.free + cfg['influx'] - cfg['free_cap'], 0)
        self.free = min(self.free + cfg['influx'], cfg['free_cap'])
        for r in row['roots']:
            id_ = r['id']
            stock, gene = self.balances[id_], self.genes[id_]
            require(r['capital_before'] == stock['capital'] and r['budget_before'] == stock['budget'], 'prior stock')
            require(r['member'] == gene['member'] and r['lineage'] == gene['lineage']
                    and r['executable'] == gene['executable'], 'fate genealogy')
            active = stock['capital'] >= cfg['rent']
            require(r['active'] == active and r['cost'] == (cfg['rent'] if active else 0), 'rent/starvation')
            fate_check(r['fate'])
            require(r['fate']['truncated'] == max(r['fate']['generated'] - cfg['match_cap_per_rule'], 0), 'canonical truncation')
            require(type(r['spend']) is int and r['spend'] >= 0, 'creation spend')
            if not active:
                self.free += stock['capital']
                del self.balances[id_]
                require(r['capital_after_income'] is None and r['gate'] is None and not any(r['fate'].values())
                        and r['income'] == r['spend'] == 0, 'dead-root leakage')
                continue
            stock['capital'] -= r['cost']
            self.dissipated += r['cost']
            if gene['executable']:
                require(r['gate'] is not None, 'missing executable gate')
                gate_check(r['gate'], gene['spec'], stock['budget'], row['label_counts_before'], self.nodes, self.edges, cfg)
                gate = r['gate']
                if gate['enumeration_skipped'] or gate['creation_blocked'] or gate['node_ceiling_blocked'] or gate['edge_ceiling_blocked']:
                    require(not any(r['fate'].values()) and r['spend'] == 0, 'gated leakage')
            else:
                require(r['gate'] is None and not any(r['fate'].values()) and r['spend'] == 0, 'invalid-root firing')
            stock['budget'] -= r['spend']
        products = {r['id']: r['fate']['productive'] for r in row['roots'] if r['fate']['productive'] > 0}
        total = sum(products.values())
        pool = min(cfg['income_pool'], self.free) if total else 0
        incomes = {id_: pool * p // total for id_, p in products.items()}
        remainder = pool - sum(incomes.values())
        for id_ in sorted(products):
            if remainder:
                incomes[id_] += 1
                remainder -= 1
        require(remainder == 0, 'income remainder')
        self.free -= sum(incomes.values())
        for r in row['roots']:
            require(r['income'] == incomes.get(r['id'], 0), 'proportional income')
            if r['active']:
                self.balances[r['id']]['capital'] += r['income']
                require(r['capital_after_income'] == self.balances[r['id']]['capital'], 'post-income stock')
        self.nodes, self.edges, self.step = row['world_nodes'], row['world_edges'], row['absolute_step']
        require(self.nodes <= cfg['node_explosion_cap'], 'hard stop/incomplete')

    def finish(self, row, selection=None, evidence=None, *, fission_enabled=True):
        cfg = self.cfg
        eligible = [id_ for id_ in sorted(self.balances) if self.balances[id_]['capital'] >= cfg['repro_threshold']]
        eligible = eligible[:max(cfg['rule_cap'] - len(self.balances), 0)]
        if not fission_enabled:
            eligible = []
        require([b['parent'] for b in row['births']] == eligible, 'native eligibility/order/cap')
        if selection:
            target = next((b for b in row['births'] if self.genes[b['parent']]['member'] == 0), None)
            require(target is not None and target['parent'] == selection['parent'] and target['child'] == selection['child'], 'first natural R1 birth')
        for b in row['births']:
            parent, child = b['parent'], b['child']
            stock, pg = self.balances[parent], self.genes[parent]
            require(child not in self.genes and child not in self.balances, 'duplicate permanent root identity')
            require(b['parent_before'] == stock['capital'] and b['parent_after_native'] == stock['capital'] - 50
                    and b['child_tokens'] == 50 and b['inherited_budget'] == stock['budget'], 'native transfer/inheritance')
            require(0 <= b['coin'] < 1 and 0 <= b['operator'] < 15 and len(b['parameters']) == 5, 'ordinary birth draw block')
            chosen = selection is not None and child == selection['child']
            gene = b['gene']
            require(gene['id'] == child and gene['parent'] == parent and gene['member'] == pg['member']
                    and gene['born'] == self.step and gene['lineage'] == (chosen or pg['lineage']), 'birth genealogy')
            if not chosen or selection['arm'] in ('faithful', 'refund'):
                require(canonical_spec(gene['spec']) == canonical_spec(pg['spec']) and gene['class_hash'] == pg['class_hash']
                        and gene['executable'] == pg['executable'], 'faithful future genotype')
            if chosen and selection['arm'] == 'prune':
                require(gene['spec'] == evidence['decoded_spec'] and gene['class_hash'] == evidence['mutant_class_hash']
                        and gene['executable'] and selection['changed_class'], 'named variant genotype')
            cancelled = chosen and selection['arm'] == 'refund'
            require(b['cancelled'] == cancelled, 'refund target')
            stock['capital'] -= 50
            if cancelled:
                stock['capital'] += 50
            else:
                self.balances[child] = dict(capital=50, budget=stock['budget'])
            self.genes[child] = gene
            self.seq += 1
            if chosen:
                require(selection['cancelled'] == cancelled and selection['executable'] == gene['executable']
                        and selection['changed_class'] == (gene['class_hash'] != pg['class_hash']), 'selection attributes')
        for stock in self.balances.values():
            stock['budget'] = min(stock['budget'] + cfg['creation_refill'], cfg['creation_budget_max'])
        require(row['stocks'] == [dict(id=id_, **s) for id_, s in sorted(self.balances.items())], 'post-fission/refill stocks')
        require(row['ledger'] == self.cash(), 'row flux ledger')
        require(row['rule_cap_binding'] == (len(self.balances) >= cfg['rule_cap']), 'rule-cap flag')
        require(row['rng_word_pos'] == self.pos + 14 * len(row['births']), 'native RNG draw count')
        if not row['births']:
            require(row['rng_probe'] == self.probe, 'RNG drift without births')
        self.pos, self.probe = row['rng_word_pos'], row['rng_probe']

    def step_row(self, row):
        self.object(row)
        self.finish(row)


def new_totals():
    return dict(productive=[0]*3, member_copy_steps=[0]*3, lineage_productive=0, lineage_copy_steps=0,
                lineage_income=0, lineage_rent=0, late_lineage_income=0, late_lineage_rent=0,
                parent_productive=0, parent_income=0, parent_rent=0, parent_death_step=None,
                child_death_step=None, first_lineage_productive=None, late_member_windows=[[0]*3 for _ in range(4)],
                late_lineage_windows=[0]*4, enumeration_skips=0, resource_gates=0, rule_cap_steps=0)


def add_totals(t, row, selected, horizon):
    step = row['continuation_step']
    late = step > horizon//2
    window = (step - horizon//2 - 1)//(horizon//8) if late else None
    for r in row['roots']:
        m, p = r['member'], r['fate']['productive']
        t['productive'][m] += p
        t['member_copy_steps'][m] += r['active']
        if r['gate']:
            t['enumeration_skips'] += r['gate']['enumeration_skipped']
            t['resource_gates'] += r['gate']['creation_blocked'] or r['gate']['node_ceiling_blocked'] or r['gate']['edge_ceiling_blocked']
        if r['lineage']:
            t['lineage_productive'] += p
            t['lineage_copy_steps'] += r['active']
            t['lineage_income'] += r['income']
            t['lineage_rent'] += r['cost']
            if late:
                t['late_lineage_income'] += r['income']
                t['late_lineage_rent'] += r['cost']
                t['late_lineage_windows'][window] += p
            if p and t['first_lineage_productive'] is None:
                t['first_lineage_productive'] = step
        if r['id'] == selected['parent']:
            t['parent_productive'] += p
            t['parent_income'] += r['income']
            t['parent_rent'] += r['cost']
        if late:
            t['late_member_windows'][window][m] += p
    ids = {s['id'] for s in row['stocks']}
    if selected['parent'] not in ids and t['parent_death_step'] is None:
        t['parent_death_step'] = step
    if not selected['cancelled'] and selected['child'] not in ids and t['child_death_step'] is None:
        t['child_death_step'] = step
    t['rule_cap_steps'] += row['rule_cap_binding']


def score(state, selected, totals, completed):
    horizon = state.cfg['steps']
    final_members = [0]*3
    lineage_copies = lineage_capital = 0
    for id_, stock in state.balances.items():
        gene = state.genes[id_]
        final_members[gene['member']] += 1
        if gene['lineage']:
            lineage_copies += 1
            lineage_capital += stock['capital']
    complete = completed == horizon
    cycle = complete and all(final_members) and all(all(w) for w in totals['late_member_windows'])
    measured = complete and not totals['enumeration_skips']
    established = selected['executable'] and measured and cycle and lineage_copies > 0 and \
        all(totals['late_lineage_windows']) and totals['late_lineage_income'] >= totals['late_lineage_rent']
    return dict(complete=complete, completed_steps=completed, requested_steps=horizon, error=None,
                whole_cycle_persistent=cycle, whole_cycle_uncensored=cycle and measured,
                bounded_establishment=None if selected['cancelled'] else established,
                selected_child_executable=None if selected['cancelled'] else selected['executable'],
                final_member_copies=final_members, final_lineage_copies=lineage_copies, final_lineage_capital=lineage_capital,
                parent_final_capital=state.balances.get(selected['parent'], {}).get('capital'),
                lineage_productive_rate=totals['lineage_productive']/horizon if measured and selected['executable'] and not selected['cancelled'] else None,
                lineage_per_copy_rate=totals['lineage_productive']/totals['lineage_copy_steps'] if measured and selected['executable'] and not selected['cancelled'] and totals['lineage_copy_steps'] else None,
                parent_productive_rate=totals['parent_productive']/horizon if measured else None, totals=totals)


def audit(out):
    manifest, summary, status, provenance = (load(out/p) for p in ('manifest.json', 'summary.json', 'status.json', 'instrument_provenance.json'))
    preflight = manifest['engineering_preflight']
    seeds = [7, 42] if preflight else SEEDS
    horizon = 64 if preflight else 2000
    require(manifest['protocol'] == 'selfmod-v12-birth-v1' and manifest['seeds'] == seeds and manifest['arms'] == ARMS
            and manifest['horizon'] == horizon and manifest['prefix_limit'] == 256 and manifest['fission_enabled']
            and not manifest['future_mutation'], 'frozen protocol/panel')
    evidence = load(out/'v11_selected_mutation.json')
    require(manifest['prune_parameters'] == evidence['parameters'] and evidence['id'] == 'op11_member0_draw1', 'selected v11 provenance')
    require(sha(out/'selfmod-birth') == provenance['binary_sha256'] and sha(out/'source.tar.gz') == provenance['source_archive_sha256']
            and sha(out/'v11_selected_mutation.json') == provenance['selected_v11_evidence_sha256'], 'instrument/evidence hashes')
    with tarfile.open(out/'source.tar.gz') as archive:
        require(set(archive.getnames()) == set(provenance['sources_sha256']), 'source archive inventory')
        for name, digest in provenance['sources_sha256'].items():
            require(hashlib.sha256(archive.extractfile(name).read()).hexdigest() == digest, 'source bytes ' + name)
    configs = {c['seed']: c for c in manifest['configs']}
    for cfg in configs.values():
        require(cfg['steps'] == horizon and cfg['seed_pop'] == 'churn3' and cfg['copy_number'] == 1 and cfg['e0'] == 60
                and cfg['fission_mode'] == 'full' and cfg['birth_cost'] == 50 and cfg['repro_threshold'] == 100
                and cfg['tie_break'] == 'random' and cfg['match_cost_coeff'] == 0 and cfg['truncation_mode'] == 'canonical'
                and cfg['income_mode'] == 'proportional' and cfg['income_pool'] == 12 and cfg['rent'] == 1
                and cfg['influx'] == 20 and cfg['free_cap'] == 500 and cfg['match_enum_budget'] == 50000
                and cfg['match_cap_per_rule'] == 64 and cfg['creation_budget_init'] == cfg['creation_budget_max'] == 200
                and cfg['creation_refill'] == 1 and all(cfg[k] == 0 for k in ('mu', 'abio', 'p_dup', 'p_del')), 'native operating point')
    compact, prefixes, forks, paired = [], 0, [], []
    for seed in seeds:
        directory = out/f'seed_{seed}'
        initial = load(directory/'initial.json')
        require(initial['cfg'] == configs[seed], 'config pairing')
        state = State(initial)
        require(len(state.balances) == 3 and all(s == dict(capital=60, budget=200) for s in state.balances.values())
                and state.cash() == dict(free=500, bound=180, cumulative_influx=680, cumulative_dissipated=0, audit_residual=0), 'initial funding')
        for row in rows(directory/'prefix.jsonl'):
            require(row['continuation_step'] is None and not any(b['gene']['member'] == 0 for b in row['births']), 'first R1 trigger not skipped')
            state.step_row(row)
            prefixes += 1
        if (directory/'untriggered.json').exists():
            require(state.step == 256 and load(directory/'untriggered.json')['prefix_steps'] == 256, 'untriggered limit')
            continue
        fork = load(directory/'fork.json')
        state.object(fork['pending'])
        prefixes += 1
        state.match_snapshot(fork['checkpoint'])
        require(state.step <= 256, 'trigger deadline')
        faithful_birth = fork['faithful_birth']
        forks.append(dict(seed=seed, absolute_birth_step=state.step,
                          parent_tokens=next(b['parent_before'] for b in faithful_birth['births'] if b['child'] == fork['faithful_selection']['child']),
                          inherited_budget=next(b['inherited_budget'] for b in faithful_birth['births'] if b['child'] == fork['faithful_selection']['child']),
                          unscreened_operator=next(b['operator'] for b in faithful_birth['births'] if b['child'] == fork['faithful_selection']['child'])))
        seed_scores = {}
        for arm in ARMS:
            branch_dir = directory/arm
            birth = load(branch_dir/'birth.json')
            selected, row = birth['selection'], birth['row']
            require(selected['arm'] == arm and selected['parent'] == fork['faithful_selection']['parent']
                    and selected['child'] == fork['faithful_selection']['child'], 'selected root pairing')
            for key in ('absolute_step', 'roots', 'label_counts_before', 'world_nodes_before', 'world_edges_before', 'world_nodes',
                        'world_edges', 'world_hash', 'rng_word_pos', 'rng_probe', 'ledger'):
                require(row[key] == faithful_birth[key], 'common birth-phase pairing ' + key)
            require(len(row['births']) == len(faithful_birth['births']), 'paired sibling count')
            for b, baseline in zip(row['births'], faithful_birth['births']):
                for key in ('parent', 'child', 'parent_before', 'parent_after_native', 'child_tokens', 'inherited_budget', 'coin', 'operator', 'parameters'):
                    require(b[key] == baseline[key], 'paired native birth ' + key)
            branch = copy.deepcopy(state)
            branch.finish(row, selected, evidence)
            branch.match_snapshot(birth['initial'])
            if arm == 'faithful':
                require(row == faithful_birth and selected == fork['faithful_selection'], 'faithful fork exactness')
            totals = new_totals()
            completed = 0
            for row in rows(branch_dir/'rows.jsonl'):
                completed += 1
                require(row['continuation_step'] == completed, 'continuation denominator/sequence')
                branch.step_row(row)
                add_totals(totals, row, selected, horizon)
            require(completed == horizon, 'incomplete continuation')
            reconstructed = score(branch, selected, totals, completed)
            require(reconstructed == load(branch_dir/'score.json'), 'establishment verdict reconstruction')
            branch.match_snapshot(load(branch_dir/'final.json'))
            compact.append(dict(seed=seed, arm=arm, score=reconstructed))
            seed_scores[arm] = reconstructed
        paid = load(directory/'faithful/birth.json')
        refund = load(directory/'refund/birth.json')
        parent = paid['selection']['parent']
        paid_cap = next(s['capital'] for s in paid['row']['stocks'] if s['id'] == parent)
        refund_cap = next(s['capital'] for s in refund['row']['stocks'] if s['id'] == parent)
        require(refund_cap - paid_cap == 50, 'exact cancelled-birth parent refund')
        paired.append(dict(seed=seed, scores=seed_scores,
                           prune_minus_faithful_lineage_rate=None if seed_scores['prune']['lineage_productive_rate'] is None or
                           seed_scores['faithful']['lineage_productive_rate'] is None else
                           seed_scores['prune']['lineage_productive_rate']-seed_scores['faithful']['lineage_productive_rate'],
                           paid_parent_initial_capital=paid_cap, refunded_parent_initial_capital=refund_cap))
    actual = prefixes + len(compact)*horizon
    require(summary['results'] == compact and summary['shared_prefix_steps'] == prefixes and
            summary['continuations_completed'] == len(compact) and summary['triggered_seeds'] == len(forks)
            and summary['continuation_steps'] == len(compact)*horizon and summary['scientific_advancing_steps'] == actual
            and status['state'] == 'complete' and status['scientific_advancing_steps'] == actual
            and not summary['probation_launched'] and not summary['mutation_sweep_launched'], 'summary and scientific accounting')
    aggregates = {}
    for arm in ARMS:
        group = [r['score'] for r in compact if r['arm'] == arm]
        rates = [r['lineage_productive_rate'] for r in group if r['lineage_productive_rate'] is not None]
        established = None if arm == 'refund' else sum(r['bounded_establishment'] for r in group)
        aggregates[arm] = dict(continuations=len(group), established=established,
                               replicated_establishment=None if preflight or arm == 'refund' else established >= 6,
                               whole_cycle_persistent=sum(r['whole_cycle_persistent'] for r in group),
                               whole_cycle_uncensored=sum(r['whole_cycle_uncensored'] for r in group),
                               final_lineage_alive=sum(r['final_lineage_copies'] > 0 for r in group),
                               parent_root_alive=sum(r['parent_final_capital'] is not None for r in group),
                               enumeration_censored=sum(r['totals']['enumeration_skips'] > 0 for r in group),
                               resource_gated=sum(r['totals']['resource_gates'] > 0 for r in group),
                               mean_lineage_productive_rate=statistics.mean(rates) if rates else None)
    return dict(audit_passed=True, engineering_preflight=preflight, scientific_advancing_steps=actual,
                shared_prefix_steps=prefixes, continuations_completed=len(compact), forks=forks, aggregates=aggregates, paired=paired,
                auditor_sha256=sha(pathlib.Path(__file__)),
                scope='Independent money, genealogy, allowance, eligibility, gate and verdict reconstruction. Native mutator, decoder, matcher/effect classification, class hash and ChaCha values are checked by native tests/replay, not reimplemented.',
                evidence_sha256={str(p.relative_to(out)): sha(p) for p in sorted(out.rglob('*'))
                                 if p.is_file() and p.name not in ('audit.json', 'replay_verification.json')})


def adversarial(out):
    seed = load(out/'manifest.json')['seeds'][0]
    directory = out/f'seed_{seed}'
    birth = load(directory/'faithful/birth.json')
    row = next(rows(directory/'faithful/rows.jsonl'))
    rejected = []
    for label, alter in (
        ('rent', lambda r: r['roots'][0].__setitem__('cost', 999)),
        ('fate', lambda r: r['roots'][0]['fate'].__setitem__('productive', 999)),
        ('cash', lambda r: r['ledger'].__setitem__('free', 999)),
        ('creation_stock', lambda r: r['stocks'][0].__setitem__('budget', 999)),
        ('genealogy', lambda r: r['roots'][0].__setitem__('lineage', not r['roots'][0]['lineage'])),
        ('rng_draws', lambda r: r.__setitem__('rng_word_pos', r['rng_word_pos']+2)),
        ('missing_root', lambda r: r['roots'].pop()),
    ):
        damaged = copy.deepcopy(row)
        alter(damaged)
        try:
            State(birth['initial']).step_row(damaged)
        except ValueError:
            rejected.append(label)
        else:
            raise ValueError('accepted corrupted ' + label)
    fork = load(directory/'fork.json')
    for label, alter in (
        ('birth_endowment', lambda r: r['births'][0].__setitem__('child_tokens', 999)),
        ('inherited_budget', lambda r: r['births'][0].__setitem__('inherited_budget', 999)),
        ('birth_parent', lambda r: r['births'][0].__setitem__('parent', -1)),
    ):
        damaged = copy.deepcopy(birth['row'])
        alter(damaged)
        try:
            State(fork['checkpoint']).finish(damaged, birth['selection'], load(out/'v11_selected_mutation.json'))
        except (ValueError, KeyError):
            rejected.append(label)
        else:
            raise ValueError('accepted corrupted ' + label)
    return rejected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=pathlib.Path)
    args = parser.parse_args()
    report = audit(args.output)
    report['adversarial_corruptions_rejected'] = adversarial(args.output)
    with (args.output/'audit.json').open('x') as file:
        json.dump(report, file, indent=2)
        file.write('\n')
    print(json.dumps({k: report[k] for k in ('audit_passed', 'scientific_advancing_steps', 'continuations_completed',
                                            'forks', 'aggregates', 'adversarial_corruptions_rejected')}, indent=2))


if __name__ == '__main__':
    main()
