#!/usr/bin/env python3
"""Read-only, ancestry-based endpoint/lifetime instrumentation of sealed native traces."""
import argparse
from collections import Counter
import copy
from datetime import datetime, timezone
from fractions import Fraction
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import sys
import time

PROFILE = 'evolving-minds-d0-opportunity-accounting-v1'
INPUT_SEAL = 'b7a3de57b05974acecbb29f1e89aa5d6f200ee930ed92471144e56076e8d84f4'
FLAGS = dict(development_only=True, scientific_run_launched=False, gate_pass=False,
             training_admitted=False, evolution_enabled=False, reproductive_fairness_accepted=False)
CAP = 512 * 1024 * 1024


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    old = sys.dont_write_bytecode
    try:
        sys.dont_write_bytecode = True
        spec.loader.exec_module(module)
    finally:
        sys.dont_write_bytecode = old
    return module


def data(path):
    return json.loads(Path(path).read_text())


def inventory(root):
    require(root.is_dir() and not root.is_symlink(), 'archive directory')
    files = {}
    size = 0
    for p in sorted(root.rglob('*')):
        require(not p.is_symlink(), 'archive symlink')
        if p.is_dir():
            continue
        require(p.is_file(), 'archive special file')
        size += p.stat().st_size
        require(size <= CAP, 'archive input ceiling')
        files[str(p.relative_to(root))] = digest(p.read_bytes())
    return files, size


def sealed_input(root, expected=INPUT_SEAL):
    files, size = inventory(root)
    require(files.get('output_sha256.json') == expected, 'pinned input seal')
    seal = data(root / 'output_sha256.json')
    require(all(seal.get(k) is v for k, v in FLAGS.items()), 'sealed research boundary')
    payload = dict(files)
    payload.pop('output_sha256.json')
    require(payload == seal['files_sha256'], 'sealed complete inventory/hash')
    return files, size


def sign_tail(n, positives):
    require(type(n) is int and type(positives) is int and n > 0 and 0 <= positives <= n,
            'sign-tail domain')
    return Fraction(sum(math.comb(n, k) for k in range(positives, n + 1)), 2 ** n)


def power_design():
    n, alpha = 8, Fraction(1, 40)
    critical = next(k for k in range(n + 1) if sign_tail(n, k) <= alpha)
    scenarios = []
    for p in [Fraction(1, 2), Fraction(3, 5), Fraction(7, 10), Fraction(4, 5),
              Fraction(9, 10), Fraction(19, 20), Fraction(99, 100), Fraction(1)]:
        marginal = sum(Fraction(math.comb(n, k)) * p ** k * (1 - p) ** (n - k)
                       for k in range(critical, n + 1))
        scenarios.append(dict(hypothetical_positive_probability=float(p),
                              one_comparison_sign_count_probability=float(marginal),
                              two_comparison_joint_lower_bound=float(max(0, 2 * marginal - 1)),
                              two_comparison_joint_upper_bound=float(marginal)))
    return dict(worlds=n, per_comparison_alpha=str(alpha), required_positives=critical,
                seven_positive_tail=str(sign_tail(8, 7)), eight_positive_tail=str(sign_tail(8, 8)),
                scenarios=scenarios, empirical_ecological_power_estimated=False,
                effect_floor_and_feasibility_not_modeled=True,
                homogeneous_independent_worlds_assumed_in_sensitivity_only=True,
                comparison_independence_assumed=False,
                positive_probability_for_80_percent_marginal_sign_power=0.8 ** (1 / n))


class LifetimeObserver:
    """Observe graph replay without altering rules, events, state or the reference model."""
    def __init__(self, c, d, evidence, cfg):
        self.c, self.d, self.evidence, self.cfg = c, d, evidence, cfg
        self.entries = evidence['journal']
        require(len(self.entries) == 64 * 46, 'full opportunity horizon')
        require(all(e['step'] == i + 1 and e['tick'] == i // 46 + 1 and e['pass'] == i % 46
                    for i, e in enumerate(self.entries)), 'opportunity pass coverage')
        self.members = {}
        self.index = -1
        self.event_count = 0
        self.nodes = None
        self.request_events = {}
        self.orig_apply = c.apply
        self.ticks = []
        self.before_opportunities = set()
        self.actual_attempts = set()
        for key, birth in sorted(evidence['lineage'].items(), key=lambda kv: (kv[1]['born_tick'], int(kv[0]))):
            if birth['parent'] is None:
                require(birth['born_tick'] == 0, 'initial founder ancestry')
                self.add(int(key), 64.0)
        require(len(self.members) == 2, 'two initial founders')

    def add(self, bid, reserve):
        require(bid not in self.members, 'duplicate admitted organism')
        birth = self.evidence['lineage'][str(bid)]
        parent = birth['parent']
        require(parent is None or parent in self.members, 'observed birth ancestry')
        self.members[bid] = dict(id=bid, parent=parent,
            founder=bid if parent is None else self.members[parent]['founder'],
            program=copy.deepcopy(birth['program']), born_tick=birth['born_tick'], death_tick=None,
            death_cause=None, completion_tick=None, cursor=0, initial_reserve=reserve, final_reserve=reserve,
            live_start_ticks=0, unfinished_development_ticks=0, development_energy_wait_ticks=0,
            halted_actuator_ticks=0, action_funding_blocked_ticks=0, payable_action_opportunities=0,
            halted_age_ready_ticks=0, funded_parent_ticks=0, reproduction_funding_blocked_ticks=0,
            site_blocked_funded_ticks=0, eligible_parent_ticks=0, offered_targets=0,
            development_paid=0, development_failures=0, attempts=0, successes=0, failures=0,
            harvested=0.0, development_charge=0.0, attempt_charge=0.0, maintenance_charge=0.0,
            request_debits=0.0, request_charge=0.0, copying_charge=0.0, child_endowment_transferred=0.0,
            refunds=0.0, death_return=0.0, requests=0, committed_children=0, invalid_requests=0)

    def roots(self, nodes):
        return {self.c.value(n, self.c.BID, 'Int'): n for n in nodes.values()
                if n['label'] == self.c.ROOT}

    def begin(self, index, nodes):
        self.index, self.event_count = index, 0
        phase, tick = index % 46, index // 46 + 1
        roots = self.roots(nodes)
        require(set(roots) <= set(self.members), 'unobserved native organism')
        if phase == 13:
            self.before_opportunities, self.actual_attempts = set(), set()
        offers = []
        occupied = {self.evidence['lineage'][str(b)]['site'] for b in roots}
        for bid, n in roots.items():
            m = self.members[bid]
            reserve = self.c.value(n, self.c.ENERGY, 'Float')
            status = self.c.value(n, self.d.STATUS, 'Int')
            if phase == 0:
                m['live_start_ticks'] += 1
            elif phase == 12 and status <= 1:
                m['unfinished_development_ticks'] += 1
                m['development_energy_wait_ticks'] += reserve < 1
            elif phase == 13 and status == 2 and self.c.value(n, self.d.HAS_ACT, 'Bool'):
                m['halted_actuator_ticks'] += 1
                if reserve < 1:
                    m['action_funding_blocked_ticks'] += 1
                elif not self.c.value(n, self.d.ATTEMPT, 'Bool'):
                    m['payable_action_opportunities'] += 1
                    self.before_opportunities.add(bid)
            elif phase == 16 and status == 2 and self.c.value(n, self.c.AGE, 'Int') >= 2:
                m['halted_age_ready_ticks'] += 1
                length = len(m['program']['instructions'])
                fee = 1 + (11 + length) / 4
                if reserve < 1 + self.cfg['endowment'] + fee:
                    m['reproduction_funding_blocked_ticks'] += 1
                    continue
                m['funded_parent_ticks'] += 1
                home = self.evidence['lineage'][str(bid)]['site']
                targets = [t for t in [(home - 1) % 8, (home + 1) % 8] if t not in occupied]
                m['site_blocked_funded_ticks'] += not targets
                m['eligible_parent_ticks'] += bool(targets)
                m['offered_targets'] += len(targets)
                offers.extend(dict(parent=bid, home=(home - self.cfg['rotation']) % 8,
                                   target=(t - self.cfg['rotation']) % 8, length=length) for t in targets)
        if phase == 16:
            require(sorted(offers, key=lambda o: (o['parent'], o['home'], o['target'], o['length'])) ==
                    self.evidence['admission']['rounds'][tick - 1]['offers'],
                    'graph-derived reproductive offers')

    def end(self, nodes):
        entry = self.entries[self.index]
        require(self.event_count == len(entry['events']), 'observer accepted-event coverage')
        if self.index % 46 == 13:
            require(self.actual_attempts == self.before_opportunities, 'paid opportunity/attempt identity')
        if self.index % 46 == 45:
            self.reconcile_tick(nodes, self.index // 46 + 1)

    def advance(self, index, nodes):
        require(self.index <= index < len(self.entries), 'observer monotone pass')
        while self.index < index:
            if self.index >= 0:
                self.end(nodes)
            self.begin(self.index + 1, nodes)
            if self.index < index:
                require(not self.entries[self.index]['events'], 'skipped nonempty pass')

    def apply(self, nodes, edges, event, rule, index):
        self.nodes = nodes
        self.advance(index, nodes)
        expected = self.entries[index]['events']
        require(self.event_count < len(expected) and event == expected[self.event_count], 'observer event identity')
        bound = dict(zip(rule['lhs']['nodes'], map(self.c.handle, event['bound'])))
        root = nodes.get(bound.get('r'))
        bid = self.c.value(root, self.c.BID, 'Int') if root and root['label'] == self.c.ROOT else None
        before = self.c.value(root, self.c.ENERGY, 'Float') if bid is not None else 0.0
        escrow_before = self.c.value(nodes[bound['escrow']], self.c.ENERGY, 'Float') if 'escrow' in bound else None
        rid = self.c.value(nodes[bound['escrow']], 242014, 'Int') if escrow_before is not None else None
        binding = self.orig_apply(nodes, edges, event, rule, index)
        self.event_count += 1
        tick, phase = index // 46 + 1, index % 46
        name = event['rule']
        if bid is not None:
            m = self.members[bid]
            after = self.c.value(nodes[binding['r']], self.c.ENERGY, 'Float')
            debit = before - after
            m['final_reserve'] = after
            if phase == 12:
                require(name.startswith('development_') and debit in [0.0, 1.0], 'instruction payment')
                m['development_charge'] += debit
                m['development_paid'] += int(debit)
                m['cursor'] = self.c.value(nodes[binding['r']], self.d.CURSOR, 'Int')
                if name == 'development_halt':
                    require(m['completion_tick'] is None, 'duplicate completion')
                    m['completion_tick'] = tick
                if name == 'development_fail':
                    m['development_failures'] += 1
            elif phase == 13:
                require(name == 'attempt_commit' and debit == 1.0 and bid not in self.actual_attempts, 'attempt payment')
                self.actual_attempts.add(bid)
                m['attempts'] += 1
                m['attempt_charge'] += debit
            elif phase == 14:
                require(name in ['settle_success', 'settle_failure'] and debit == (-8.0 if name == 'settle_success' else 0.0), 'reward transfer')
                m['successes'] += name == 'settle_success'
                m['failures'] += name == 'settle_failure'
                m['harvested'] -= debit
            elif phase == 15:
                require(name in ['maintenance_full', 'maintenance_exhausted'] and debit >= 0, 'upkeep payment')
                m['maintenance_charge'] += debit
            elif phase == 16:
                escrow = nodes[binding['escrow']]
                request = self.c.value(escrow, 242014, 'Int')
                deposit = self.c.value(escrow, self.c.ENERGY, 'Float')
                require(request not in self.request_events and debit == 1 + deposit, 'request escrow payment')
                self.request_events[request] = dict(parent=bid, request_charge=debit - deposit,
                                                   deposit=deposit, copying_charge=None, refunded=None, child_id=None)
                m['request_debits'] += debit
            elif phase == 17:
                require(rid in self.request_events and self.request_events[rid]['parent'] == bid, 'settlement parent')
                r = self.request_events[rid]
                require(self.c.value(nodes[binding['escrow']], self.c.ENERGY, 'Float') == 0.0, 'settled escrow')
                if 'child' in binding:
                    require(debit == 0.0, 'copy debited parent twice')
                    child = nodes[binding['child']]
                    child_id = self.c.value(child, self.c.BID, 'Int')
                    reserve = self.c.value(child, self.c.ENERGY, 'Float')
                    require(reserve == self.cfg['endowment'] and self.evidence['lineage'][str(child_id)]['parent'] == bid,
                            'parent-funded child identity')
                    self.add(child_id, reserve)
                    r.update(child_id=child_id, copying_charge=escrow_before - reserve, refunded=0.0)
                else:
                    require(-debit == escrow_before, 'complete reject refund')
                    r.update(copying_charge=0.0, refunded=-debit)
                    m['refunds'] -= debit
            elif phase == 19:
                require(name in ['death_age', 'death_empty'] and after == 0.0 and m['death_tick'] is None, 'death settlement')
                m.update(death_tick=tick, death_cause=name)
                m['death_return'] += debit
            else:
                require(debit == 0.0, 'unclassified organism energy effect')
        return binding

    def reconcile_tick(self, nodes, tick):
        row = self.evidence['rows'][tick - 1]
        members = list(self.members.values())
        for field in ['development_paid', 'development_failures', 'attempts', 'successes', 'failures', 'harvested']:
            require(math.fsum(m[field] for m in members) == row[field], 'per-tick lifetime ' + field)
        require(sum(m['parent'] is not None for m in members) == row['births'] and
                sum(m['death_tick'] is not None for m in members) == row['deaths'], 'per-tick birth/death denominator')
        charges = math.fsum(m[k] for m in members for k in ['development_charge', 'attempt_charge', 'maintenance_charge'])
        paid = [r for r in self.request_events.values() if r['copying_charge'] is not None]
        require(charges + math.fsum(r['request_charge'] + r['copying_charge'] for r in paid) == row['dissipated'],
                'per-tick independent dissipation')
        for bid, m in self.members.items():
            balance = (m['initial_reserve'] + m['harvested'] + m['refunds'] - m['development_charge'] -
                       m['attempt_charge'] - m['maintenance_charge'] - m['request_debits'] - m['death_return'])
            require(balance == m['final_reserve'], 'organism root energy balance')
        founder = math.fsum(m['harvested'] for m in members if m['parent'] is None)
        descendant = math.fsum(m['harvested'] for m in members if m['parent'] is not None)
        require(founder + descendant == row['harvested'], 'founder/descendant endpoint partition')
        self.ticks.append(dict(tick=tick, founder_harvested=founder, descendant_harvested=descendant,
                               total_harvested=row['harvested']))

    def finish(self):
        require(self.nodes is not None, 'missing replay')
        self.advance(len(self.entries) - 1, self.nodes)
        self.end(self.nodes)
        require(len(self.ticks) == 64 and set(self.members) == {int(k) for k in self.evidence['lineage']}, 'full lifetime denominator')
        records = self.evidence['admission']['records']
        require(set(self.request_events) == {r['request_id'] for r in records}, 'native request denominator')
        for r in records:
            measured = self.request_events[r['request_id']]
            require(all(measured[k] == r[k] for k in ['parent', 'request_charge', 'copying_charge', 'refunded', 'child_id']),
                    'graph-derived request charge/copy/refund')
            m = self.members[r['parent']]
            m['requests'] += 1
            m['committed_children'] += r['child_id'] is not None
            m['invalid_requests'] += not r['syntax_valid']
            m['request_charge'] += measured['request_charge']
            m['copying_charge'] += measured['copying_charge']
            m['child_endowment_transferred'] += self.cfg['endowment'] if r['child_id'] is not None else 0
        for bid, m in self.members.items():
            b = self.evidence['lineage'][str(bid)]
            require(m['death_tick'] == b['death_tick'] and m['live_start_ticks'] == (b['death_tick'] or 64) - b['born_tick'],
                    'native lifetime timing')
            require(m['attempts'] == m['successes'] + m['failures'] == m['payable_action_opportunities'], 'lifetime action accounting')
            if m['completion_tick'] is not None:
                require(m['cursor'] == m['development_paid'] == len(m['program']['instructions']) and m['development_failures'] == 0,
                        'paid full instruction completion')
            m['right_censored_at_64'] = m['death_tick'] is None
            m['died_incomplete_without_grammar_failure'] = (m['death_tick'] is not None and m['completion_tick'] is None and m['development_failures'] == 0)
        totals = {k: math.fsum(m[k] for m in self.members.values()) for k in
                  ['live_start_ticks', 'payable_action_opportunities', 'action_funding_blocked_ticks',
                   'eligible_parent_ticks', 'offered_targets', 'funded_parent_ticks', 'site_blocked_funded_ticks',
                   'reproduction_funding_blocked_ticks', 'development_charge', 'attempt_charge',
                   'maintenance_charge', 'request_charge', 'copying_charge', 'child_endowment_transferred', 'refunds']}
        descendants = [m for m in self.members.values() if m['parent'] is not None]
        return dict(primary_descendant_harvested=self.ticks[-1]['descendant_harvested'],
                    founder_harvested=self.ticks[-1]['founder_harvested'], total_harvested=self.ticks[-1]['total_harvested'],
                    descendant_denominator=len(descendants), descendants_completed=sum(m['completion_tick'] is not None for m in descendants),
                    descendants_grammar_failed=sum(m['development_failures'] > 0 for m in descendants),
                    descendants_died_incomplete=sum(m['died_incomplete_without_grammar_failure'] for m in descendants),
                    descendants_live_at_64=sum(m['right_censored_at_64'] for m in descendants),
                    requests=len(records), totals=totals, ticks=self.ticks,
                    lifetimes=[self.members[k] for k in sorted(self.members)])


def analyze_cell(root, cfg, directory, modules, v):
    a, c, d, s = modules
    chain = v.inspect_segment(directory, funded_config=cfg)
    require(len(chain) == 1 and chain[0]['header']['fault'] == 'Normal' and chain[0]['header']['start_tick'] == 0,
            'ordinary full baseline only')
    commit = chain[-1]['commits'][-1]
    require(commit['marker']['cursor']['tick'] == 64, 'uncensored endpoint')
    manifest, status = data(directory / 'manifest.json'), data(directory / 'run_status.json')
    v.boundary(status, 'evolving-minds-d0-funded-streaming-v1')
    require(manifest['state'] == 'Completed' and manifest['seed'] == cfg['seed'] and
            manifest['final_step'] == manifest['max_steps'] == 2944, 'actual baseline manifest')
    evidence = v.flatten(chain, commit)
    observer = LifetimeObserver(c, d, evidence, cfg)
    c.apply = observer.apply
    started = time.monotonic()
    try:
        audited, physics = v.physical_audit(chain, commit, modules,
            (root / ('base_rules.toml' if cfg['endowment'] == 12 else 'b24_base_rules.toml')).read_bytes(), funded_config=cfg)
        require(audited == evidence, 'observer evidence identity')
        result = observer.finish()
    finally:
        c.apply = observer.orig_apply
    return dict(config=cfg, independent_physics=physics, opportunity=result, audit_seconds=time.monotonic() - started)


def put(path, content):
    with path.open('xb') as f:
        f.write(content)


def write_json(path, obj):
    put(path, (json.dumps(obj, indent=2, sort_keys=True, allow_nan=False) + '\n').encode())


def run(source, destination):
    source, destination = source.resolve(), destination.absolute()
    require(source not in destination.parents and destination != source, 'separate analysis destination')
    before, size = sealed_input(source)
    script_root = Path(__file__).resolve().parent
    repo = script_root.parent
    # Exclusive publication: a failed analysis is never reused or overwritten.
    destination.mkdir()
    started = time.monotonic()
    manifest = dict(profile=PROFILE, **FLAGS, state='Running', started_at=datetime.now(timezone.utc).isoformat(),
                    analysis_kind='read-only endpoint and opportunity accounting', native_advances_executed=0,
                    max_steps=10, final_step=0, step_mode='independently audited baseline cells')
    (destination / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    try:
        tests = script_root / ('tests.py' if (script_root / 'contract.md').exists() else 'test_evolving_minds_opportunity.py')
        contract = script_root / 'contract.md' if (script_root / 'contract.md').exists() else repo / 'experiments/evolving_minds/contracts/d0_opportunity_accounting_v1.md'
        for name, path in [('analyzer.py', Path(__file__)), ('tests.py', tests), ('contract.md', contract)]:
            put(destination / name, path.read_bytes())
        write_json(destination / 'input_binding.json', dict(directory=str(source), seal_sha256=INPUT_SEAL,
            files_sha256=before, bytes=size, original_independent_receipt_sha256=before['independent_replay.json']))
        f = load(source / 'checker.py', 'opportunity_funded_denominator')
        v = load(source / 'stream_reference.py', 'opportunity_storage')
        modules = tuple(load(source / n, 'opportunity_' + name) for n, name in
            [('admission_reference.py', 'admission'), ('replay.py', 'replay'), ('d0_reference.py', 'd0'), ('schedule_reference.py', 'schedule')])
        cases = f.cases()
        f.denominator(source, cases)
        write_json(destination / 'registered_cases.json', cases)
        results = {}
        for case in cases:
            require(time.monotonic() - started < 1800, 'analysis wall-time ceiling')
            name = 'baseline_' + case['name']
            results[name] = analyze_cell(source, case['config'], source / name, modules, v)
            write_json(destination / (name + '.json'), results[name])
            manifest['final_step'] += 1
            (destination / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
        require(list(results) == ['baseline_' + case['name'] for case in cases], 'complete analysis denominator')
        write_json(destination / 'power_design.json', power_design())
        write_json(destination / 'summary.json', dict(profile=PROFILE, **FLAGS, analysis_pass=True,
            baseline_cells_verified=10, full_ticks_verified=640, native_passes_reconstructed=29440,
            native_advances_executed=0, new_worlds_exposed=[], ue_comparator_implemented=False,
            results={name: {k: val for k, val in r['opportunity'].items() if k not in ['lifetimes', 'ticks']} for name, r in results.items()}))
        after, _ = inventory(source)
        require(after == before, 'analysis modified original evidence')
        elapsed = time.monotonic() - started
        require(elapsed < 1800 and sum(p.stat().st_size for p in destination.iterdir()) <= 16 * 1024 * 1024,
                'analysis time/output caps')
        write_json(destination / 'read_only_audit.json', dict(read_only=True, inputs_verified=len(before),
            input_seal_sha256=INPUT_SEAL, audit_seconds=elapsed, input_bytes=size))
        manifest.update(state='Completed', ended_at=datetime.now(timezone.utc).isoformat(), elapsed_seconds=elapsed)
        (destination / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
        return data(destination / 'summary.json')
    except Exception as error:
        manifest.update(state='Interrupted', ended_at=datetime.now(timezone.utc).isoformat(), failure=str(error))
        (destination / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('destination', type=Path)
    args = parser.parse_args()
    summary = run(args.source, args.destination)
    print(json.dumps({k: v for k, v in summary.items() if k != 'results'}, indent=2))
