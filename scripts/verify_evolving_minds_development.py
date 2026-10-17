#!/usr/bin/env python3
"""Reconstruct the fixed matched pilot, then apply prospectively fixed checks."""
import argparse
import datetime
from fractions import Fraction
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import sys
import time
import tomllib

PROFILE = 'evolving-minds-d0-matched-development-v1'
WORLDS = [101, 103, 107, 109]
ARMS = ['inherited', 'faithful', 'retained', 'immediate', 'coin']
POLICY = 'founder-first-proposal-v1'
FLAGS = dict(development_only=True, scientific_run_launched=False, gate_pass=False,
             training_admitted=False, evolution_enabled=False, reproductive_fairness_accepted=False)


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def data(path):
    return json.loads(Path(path).read_text())


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


def cases():
    rows = []
    for seed in WORLDS:
        for arm in ARMS:
            program = {'immediate': 'Immediate', 'coin': 'Coin'}.get(arm, 'Delayed')
            kernel = 'SourceKernel' if arm in ['inherited', 'retained'] else 'Faithful'
            rows.append(dict(name=f'world{seed}_{arm}', arm=arm, config=dict(seed=seed,
                endowment=24, founders=[program] * 2, kernels=[kernel] * 2, home_offsets=[0, 4],
                rotation=0, reversed=False, horizon=64, scheduler='semantic-lottery-v1')))
    return rows


def denominator(root):
    require(data(root / 'registered_cases.json') == cases(), 'fixed twenty-cell denominator')
    require({p.name for p in root.iterdir() if p.is_dir()} == {x['name'] for x in cases()},
            'complete cell directory denominator')


def ecological_decision(results):
    require(set(results) == {x['name'] for x in cases()}, 'complete decision denominator')
    checks = []
    worlds = []
    immediate_successes = immediate_attempts = coin_successes = coin_attempts = 0
    differences = {'inherited_minus_faithful': [], 'inherited_minus_retained': []}
    for seed in WORLDS:
        row = {'world': seed, 'arms': {}}
        for arm in ARMS:
            name = f'world{seed}_{arm}'
            o = results[name]['opportunity']
            require(o['ticks'] and len(o['ticks']) == 64, 'decision full horizon')
            founders = [m for m in o['lifetimes'] if m['parent'] is None]
            descendants = [m for m in o['lifetimes'] if m['parent'] is not None]
            require({m['id'] for m in founders} == {1, 2} and len(founders) == 2,
                    'decision complete founder denominator')
            require(all(m['attempts'] == m['successes'] + m['failures'] ==
                        m['payable_action_opportunities'] for m in o['lifetimes']),
                    'decision paid attempt accounting')
            exposure = all(m['completion_tick'] == 5 and m['attempts'] >= 4 and
                m['action_funding_blocked_ticks'] == 0 and m['funded_parent_ticks'] >= 1 and
                m['eligible_parent_ticks'] >= 1 and m['requests'] >= 1 for m in founders)
            checks.append(dict(name=name + ':founder_exposure', passed=exposure))
            completed = sum(m['completion_tick'] is not None for m in descendants)
            desc_attempts = sum(m['payable_action_opportunities'] for m in descendants)
            attempts = sum(m['attempts'] for m in founders)
            successes = sum(m['successes'] for m in founders)
            row['arms'][arm] = dict(descendant_yield=o['primary_descendant_harvested'],
                founder_yield=o['founder_harvested'], births=len(descendants),
                descendants_completed=completed, descendants_grammar_failed=o['descendants_grammar_failed'],
                descendants_live_at_64=o['descendants_live_at_64'], requests=o['requests'],
                founder_attempts=attempts, founder_successes=successes,
                founder_success_fraction=float(Fraction(successes, attempts)) if attempts else None,
                descendant_paid_action_opportunities=desc_attempts, founder_exposure_pass=exposure)
            if arm == 'faithful':
                checks.append(dict(name=name + ':paid_descendant_construction', passed=completed >= 1 and desc_attempts >= 1))
            if arm == 'immediate':
                immediate_attempts += attempts
                immediate_successes += successes
                checks.append(dict(name=name + ':positive_control', passed=attempts == successes == 24 and
                                   completed >= 2 and o['primary_descendant_harvested'] >= 64))
            if arm == 'coin':
                coin_attempts += attempts
                coin_successes += successes
                checks.append(dict(name=name + ':chance_control', passed=attempts >= 16 and
                                   Fraction(1, 8) <= Fraction(successes, attempts) <= Fraction(7, 8)))
        for comparator in ['faithful', 'retained']:
            difference = row['arms']['inherited']['descendant_yield'] - row['arms'][comparator]['descendant_yield']
            differences['inherited_minus_' + comparator].append(difference)
        worlds.append(row)
    aggregate_gap = Fraction(immediate_successes, immediate_attempts) - Fraction(coin_successes, coin_attempts) if immediate_attempts and coin_attempts else None
    checks.append(dict(name='aggregate_paid_founder_control_gap', passed=aggregate_gap is not None and aggregate_gap >= Fraction(1, 4)))
    paired = {}
    for name, values in differences.items():
        mean = math.fsum(values) / 4
        passed = mean >= 64 and all(v > 0 for v in values)
        checks.append(dict(name=name + ':development_effect_screen', passed=passed))
        paired[name] = dict(world_order=WORLDS, differences=values, mean=mean,
                            positive_worlds=sum(v > 0 for v in values), passed=passed)
    return dict(worlds=worlds, paired=paired, checks=checks,
        aggregate_founder_success_gap=float(aggregate_gap) if aggregate_gap is not None else None,
        ecological_feasibility_pass=all(x['passed'] for x in checks),
        failed_checks=[x['name'] for x in checks if not x['passed']],
        development_sign_tail_if_all_four_positive='1/16', scientific_sign_test_performed=False,
        power_adequacy_established=False, ready_for_scientific_launch=False)


def resource_decision(resources):
    require(set(resources) == {x['name'] for x in cases()}, 'complete resource denominator')
    max_bytes = max(x['total_bytes'] for x in resources.values())
    max_advance = max(x['advance_seconds'] for x in resources.values())
    max_audit = max(x['audit_seconds'] for x in resources.values())
    estimates = dict(campaign_bytes=120 * max_bytes, campaign_advance_seconds=120 * max_advance,
                     campaign_audit_seconds=120 * max_audit)
    passed = estimates['campaign_bytes'] <= 2 * 1024 ** 3 and estimates['campaign_advance_seconds'] <= 7200 and estimates['campaign_audit_seconds'] <= 7200
    return dict(campaign_cells=60, margin=2, maxima=dict(cell_bytes=max_bytes,
        advance_seconds=max_advance, audit_seconds=max_audit), conservative_projections=estimates,
        resource_projection_pass=passed, full_campaign_resource_guarantee=False)


def verify(root):
    start = time.monotonic()
    root = Path(root)
    require(root.is_dir() and not root.is_symlink() and not any(p.is_symlink() for p in root.rglob('*')), 'pilot archive path')
    require(sum(p.stat().st_size for p in root.rglob('*') if p.is_file()) <= 1024 ** 3, 'pilot total bytes cap')
    denominator(root)
    sources = data(root / 'sources.json')
    bindings = [('checker.py', 'scripts/verify_evolving_minds_development.py'),
        ('stream_reference.py', 'scripts/verify_evolving_minds_streaming.py'),
        ('admission_reference.py', 'scripts/verify_evolving_minds_admission.py'),
        ('funding_reference.py', 'scripts/verify_evolving_minds_funding.py'),
        ('opportunity_reference.py', 'scripts/evolving_minds_opportunity.py'),
        ('replay.py', 'scripts/verify_evolving_minds.py'), ('d0_reference.py', 'scripts/verify_evolving_minds_d0.py'),
        ('schedule_reference.py', 'scripts/verify_evolving_minds_choice.py'),
        ('contract.md', 'experiments/evolving_minds/contracts/d0_matched_development_v1.md'),
        ('base_rules.toml', 'rules/evolving_minds/admission_v1.toml')]
    for name, path in bindings:
        require((root / name).read_text() == sources[path], 'pilot frozen source binding')
    v = load(root / 'stream_reference.py', 'pilot_stream')
    a = load(root / 'admission_reference.py', 'pilot_admission')
    c = load(root / 'replay.py', 'pilot_replay')
    d = load(root / 'd0_reference.py', 'pilot_d0')
    s = load(root / 'schedule_reference.py', 'pilot_schedule')
    f = load(root / 'funding_reference.py', 'pilot_funding')
    o = load(root / 'opportunity_reference.py', 'pilot_opportunity')
    modules = (a, c, d, s)
    base = (root / 'base_rules.toml').read_bytes()
    laws = (root / 'b24_base_rules.toml').read_bytes()
    require(hashlib.sha256(base).hexdigest() == a.LAW_SHA and tomllib.loads(laws.decode()) == f.funded_laws(base, 24), 'independent pilot laws')
    results = {}
    resources = {}
    reports = []
    count = 0
    for case in cases():
        require(time.monotonic() - start <= 3600, 'pilot audit time cap')
        cell_start = time.monotonic()
        name = case['name']
        p = root / name
        cfg = case['config']
        policy = POLICY if case['arm'] == 'retained' else None
        profile = 'evolving-minds-d0-retained-streaming-v1' if policy else 'evolving-minds-d0-funded-streaming-v1'
        chain = v.inspect_segment(p, funded_config=cfg, retained_policy=policy)
        require(len(chain) == 1, 'pilot fresh single segment')
        seg = chain[0]
        require([x['marker']['cursor']['tick'] for x in seg['commits']] == list(range(0, 65, 8)), 'pilot all nine publications')
        m = data(p / 'manifest.json')
        status = data(p / 'run_status.json')
        report = data(p / 'segment_report.json')
        v.boundary(status, profile)
        require(m['experiment_name'] == profile and m['seed'] == cfg['seed'] and m['thread_count'] == 4 and
            m['state'] == report['state'] == 'Completed' and m['final_step'] == m['max_steps'] == 2944 and
            report['funded_config'] == cfg and report.get('retained_policy') == policy and
            report['start_tick'] == 0 and report['native_final_tick'] == report['committed_final_tick'] == 64 and
            report['fault'] == seg['header']['fault'] == 'Normal' and status['engineering_execution_complete'], 'actual pilot cell lifecycle/configuration')
        for commit in seg['commits']:
            require(time.monotonic() - start <= 3600, 'pilot audit time cap')
            final, physics = v.physical_audit(chain, commit, modules, laws, funded_config=cfg, retained_policy=policy)
            count += 1
        observer = o.LifetimeObserver(c, d, final, cfg)
        c.apply = observer.apply
        try:
            v.physical_audit(chain, seg['commits'][-1], modules, laws, funded_config=cfg, retained_policy=policy)
            opportunities = observer.finish()
        finally:
            c.apply = observer.orig_apply
        results[name] = dict(opportunity=opportunities, final_row=physics['final_row'])
        if policy:
            results[name]['retained'] = physics['retained']
        resource = report['resources']
        total_bytes = sum(x.stat().st_size for x in p.rglob('*') if x.is_file())
        require(resource['drained_history_empty'] and resource['advance_seconds'] <= 120 and
            resource['payload_bytes'] == (p / 'ticks.jsonl').stat().st_size <= 16 * 1024 ** 2 and
            total_bytes <= 32 * 1024 ** 2 and
            resource['checkpoints'] == [[x['marker']['cursor']['tick'], x['checkpoint_bytes']] for x in seg['commits']] and
            max(x['checkpoint_bytes'] for x in seg['commits']) <= 2 * 1024 ** 2, 'pilot observed resource caps')
        resources[name] = dict(total_bytes=total_bytes, advance_seconds=resource['advance_seconds'],
            audit_seconds=time.monotonic() - cell_start, checkpoint_bytes=max(x['checkpoint_bytes'] for x in seg['commits']))
        reports.append(report)
    for seed in WORLDS:
        for filename in ['schedule.json', 'rules.toml']:
            reference = (root / f'world{seed}_inherited' / filename).read_bytes()
            require(all((root / f'world{seed}_{arm}' / filename).read_bytes() == reference for arm in ARMS), 'matched per-world schedules/physics')
    require(data(root / 'summary.json') == reports, 'pilot report denominator')
    require(data(root / 'execution_counts.json') == dict(worlds=4, arms_per_world=5, cells=20, ticks=1280, native_advances_executed=58880), 'pilot native execution denominator')
    m = data(root / 'manifest.json')
    elapsed = (datetime.datetime.fromisoformat(m['end_time']) - datetime.datetime.fromisoformat(m['start_time'])).total_seconds()
    require(m['state'] == 'Completed' and m['experiment_name'] == PROFILE and m['seed'] == 101 and
        m['thread_count'] == 4 and m['final_step'] == m['max_steps'] == 58880 and elapsed <= 3600 and
        math.fsum(x['advance_seconds'] for x in resources.values()) <= 2400, 'pilot root lifecycle/resource caps')
    status = data(root / 'run_status.json')
    v.boundary(status, PROFILE)
    require(status['engineering_execution_complete'] and count == 180, 'pilot complete execution')
    decision = ecological_decision(results)
    resource = resource_decision(resources)
    decision['development_feasibility_pass'] = decision['ecological_feasibility_pass'] and resource['resource_projection_pass']
    require(time.monotonic() - start <= 3600, 'pilot audit time cap')
    return dict(profile=PROFILE, **FLAGS, audit_pass=True, worlds_verified=4, cells_verified=20,
        published_states_reconstructed=count, native_advances_executed=58880,
        audit_seconds=time.monotonic() - start, native_panel_elapsed_seconds=elapsed,
        results=results, cell_resources=resources, resource_decision=resource, decision=decision,
        power_design=o.power_design(), power_adequacy_established=False, ready_for_scientific_launch=False,
        source_sha256={name: hashlib.sha256(raw.encode()).hexdigest() for name, raw in sources.items()})


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('directory')
    args = parser.parse_args()
    print(json.dumps(verify(args.directory), indent=2))
