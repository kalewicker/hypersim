#!/usr/bin/env python3
"""Sealed-pilot failure decomposition and measured shared replay audit."""
import argparse
from collections import Counter
import copy
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import time
import evolving_minds_shared_replay as shared
import evolving_minds_opportunity as common

PROFILE = 'evolving-minds-d0-failure-audit-optimization-v1'
INPUT_SEAL = '7ff33c1aee9e748efac59ee204e31893d356cedc84585e8f51ed53aa23543340'
FLAGS = dict(common.FLAGS)
SOURCE_PATHS = ['scripts/evolving_minds_failure_audit.py', 'scripts/evolving_minds_shared_replay.py',
    'scripts/test_evolving_minds_failure_audit.py', 'experiments/evolving_minds/contracts/d0_failure_audit_optimization_v1.md']


def put(path, value):
    with path.open('x') as handle:
        json.dump(value, handle, indent=2, sort_keys=True)
        handle.write('\n')


def canonical(raw_bytes):
    return hashlib.sha256(bytes(raw_bytes)).hexdigest()


def decompose(records, opportunity, immediate_program):
    require = common.require
    members = {m['id']: m for m in opportunity['lifetimes']}
    require(len(members) == len(opportunity['lifetimes']), 'complete unique lifetime records')
    require(len(records) == opportunity['requests'] and len({r['request_id'] for r in records}) == len(records), 'complete request denominator')
    first = {}
    for record in sorted(records, key=lambda r: (r['tick'], r['request_id'])):
        first.setdefault(record['parent'], record['request_id'])
    routes = []
    for record in sorted(records, key=lambda r: (r['tick'], r['request_id'])):
        parent = members[record['parent']]
        child = members.get(record['child_id'])
        require(record['settled'] and record['request_charge'] == 1, 'paid settled request')
        category = ('syntax_rejected' if not record['syntax_valid'] else 'paid_unreachable' if not record['material_reachable']
                    else 'reachable_unchanged' if record['proposed_bytes'] == record['parent_bytes'] else 'reachable_edited')
        if category == 'syntax_rejected':
            require(child is None and record['child_id'] is None and record['copying_charge'] == 0 and record['refunded'] == 29, 'invalid route settlement')
        else:
            require(child is not None and child['parent'] == record['parent'] and child['born_tick'] == record['tick'] and
                child['program'] == record['proposed_program'] and record['copying_charge'] == 5 and record['refunded'] == 0,
                'paid child route identity')
        stage = 'descendant' if parent['parent'] is not None else 'founder_first' if first[parent['id']] == record['request_id'] else 'founder_later'
        routes.append(dict(request_id=record['request_id'], tick=record['tick'], parent=record['parent'], site=record['site'],
            stage=stage, category=category, operator=record['operator'], stream=record['stream'],
            raw_draws=record['raw_draws'], target_id=record['target_id'], old_source=record['old_source'], new_source=record['new_source'],
            parent_program_sha256=canonical(record['parent_bytes']), proposed_program_sha256=canonical(record['proposed_bytes']),
            matches_authored_immediate=record['proposed_program'] == immediate_program,
            request_charge=record['request_charge'], copying_charge=record['copying_charge'], refunded=record['refunded'],
            child_id=record['child_id'], child_endowment_transferred=24 if child else 0,
            child=None if child is None else {k: child[k] for k in ['completion_tick','development_failures','death_tick','death_cause']} | dict(paid_action_opportunities=child['payable_action_opportunities'],
                harvested=child['harvested'], funded_parent_ticks=child['funded_parent_ticks'], eligible_parent_ticks=child['eligible_parent_ticks'],
                reproduction_funding_blocked_ticks=child['reproduction_funding_blocked_ticks'], site_blocked_funded_ticks=child['site_blocked_funded_ticks'],
                requests=child['requests'], committed_children=child['committed_children'])))
    require({r['child_id'] for r in records if r['child_id'] is not None} == {m['id'] for m in members.values() if m['parent'] is not None}, 'all child routes retained')
    totals = {key: math.fsum(r[key] for r in routes) for key in ['request_charge','copying_charge','refunded','child_endowment_transferred']}
    require(totals == {key: opportunity['totals'][key if key != 'refunded' else 'refunds'] for key in totals}, 'full route payment reconciliation')
    require(math.fsum(m['harvested'] for m in members.values() if m['parent'] is not None) == opportunity['primary_descendant_harvested'], 'route descendant endpoint')
    founders = [m for m in members.values() if m['parent'] is None]
    descendants = [m for m in members.values() if m['parent'] is not None]
    return dict(requests=len(routes), categories=dict(Counter(r['category'] for r in routes)), stages=dict(Counter(r['stage'] for r in routes)),
        sampled_requests=sum(r['operator'] == 'source' for r in routes), retained_reuses=sum(r['operator'] == 'retained_founder' for r in routes),
        authored_immediate_proposals=sum(r['matches_authored_immediate'] for r in routes),
        payments=totals, descendant_count=len(descendants), descendant_requests=sum(m['requests'] for m in descendants),
        descendant_funded_parent_ticks=sum(m['funded_parent_ticks'] for m in descendants),
        descendant_reproduction_funding_blocked_ticks=sum(m['reproduction_funding_blocked_ticks'] for m in descendants),
        descendant_site_blocked_funded_ticks=sum(m['site_blocked_funded_ticks'] for m in descendants),
        descendant_paid_action_opportunities=sum(m['payable_action_opportunities'] for m in descendants),
        descendant_yield=opportunity['primary_descendant_harvested'],
        founders=[{k:m[k] for k in ['id','death_tick','death_cause','attempts','successes','requests',
            'action_funding_blocked_ticks','funded_parent_ticks','eligible_parent_ticks','reproduction_funding_blocked_ticks',
            'site_blocked_funded_ticks','development_charge','attempt_charge','maintenance_charge','request_charge',
            'copying_charge','child_endowment_transferred','refunds','harvested','death_return']} for m in founders], routes=routes)


def analyze(source, destination):
    source, destination = Path(source).resolve(), Path(destination).absolute()
    destination.mkdir()  # Exclusive; preserve every failed analysis destination.
    manifest = dict(run_id='matched_failure_shared_audit_v1', experiment_name=PROFILE, seed=0,
        thread_count=1, step_mode='archive-only analysis; zero native advances', state='Running',
        start_time=datetime.now(timezone.utc).isoformat(), end_time=None, max_steps=0, final_step=0)
    (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    start = time.monotonic()
    try:
        common.require(Path(common.__file__).read_text() == (source/'opportunity_reference.py').read_text(), 'sealed inventory/helper dependency binding')
        before, size = common.sealed_input(source, INPUT_SEAL)
        workspace = Path(__file__).resolve().parents[1]
        # The frozen analyzer is also runnable in-place: read its bundled sources rather than repository paths.
        frozen_sources = destination/'analysis_sources.json'
        if (Path(__file__).parent/'analysis_sources.json').exists():
            analysis_sources = common.data(Path(__file__).parent/'analysis_sources.json')
        else:
            analysis_sources = {p: (workspace/p).read_text() for p in SOURCE_PATHS}
        common.require(Path(__file__).read_text()==analysis_sources[SOURCE_PATHS[0]] and
            Path(shared.__file__).read_text()==analysis_sources[SOURCE_PATHS[1]],'executed analysis source identity')
        put(frozen_sources, analysis_sources)
        for name, key in [('evolving_minds_failure_audit.py',SOURCE_PATHS[0]),('evolving_minds_shared_replay.py',SOURCE_PATHS[1]),
                          ('tests.py',SOURCE_PATHS[2]),('contract.md',SOURCE_PATHS[3])]:
            with (destination/name).open('x') as handle: handle.write(analysis_sources[key])
        # Bind the shared helper and seal routines used by this invocation.
        with (destination/'evolving_minds_opportunity.py').open('x') as handle:
            handle.write((source/'opportunity_reference.py').read_text())
        put(destination/'input_binding.json',dict(profile=PROFILE,**FLAGS,input_directory=str(source),
            input_seal=INPUT_SEAL,input_files=len(before),input_bytes=size,input_sha256=before))
        pilot = common.load(source/'checker.py', 'failure_frozen_pilot')
        native_sources = common.data(source/'sources.json')
        common.require((source/'checker.py').read_text() == native_sources['scripts/verify_evolving_minds_development.py'], 'frozen original pilot')
        derivations = []
        def cell(*args): return shared.optimized_cell(*args, derivations=derivations)
        verify, derivation = shared.pilot_function(pilot, (source/'checker.py').read_text(), cell)
        optimized = verify(source)
        historical = common.data(source/'independent_replay.json')
        common.require(optimized['results'] == historical['results'], 'exact complete original/optimized results')
        old_ecology = copy.deepcopy(historical['decision']); new_ecology = copy.deepcopy(optimized['decision'])
        old_ecology.pop('development_feasibility_pass'); new_ecology.pop('development_feasibility_pass')
        common.require(old_ecology == new_ecology and optimized['power_design'] == historical['power_design'], 'unchanged ecological and power conclusions')
        common.require(len(derivations)==20 and all(r==derivations[0] for r in derivations), 'all cells use the same preserved admission AST')
        put(destination/'derivation.json',dict(pilot=derivation, admission=derivations[0], cells=20, published_boundaries=180,
            original_reference_passes=323840, optimized_reference_passes=58880, native_advances=0))
        put(destination/'optimized_replay.json', optimized)
        put(destination/'comparison.json',dict(profile=PROFILE,**FLAGS,exact_full_result_equality=True,
            ecological_decision_unchanged=True,power_design_unchanged=True,published_states_verified=180,
            original_audit_seconds=historical['audit_seconds'],optimized_audit_seconds=optimized['audit_seconds'],
            measured_total_speedup=historical['audit_seconds']/optimized['audit_seconds'],
            historical_resource_projection=historical['resource_decision'],optimized_resource_projection=optimized['resource_decision'],
            historical_development_feasibility_pass=historical['decision']['development_feasibility_pass'],
            optimized_development_feasibility_pass=optimized['decision']['development_feasibility_pass'],
            power_adequacy_established=False,ready_for_scientific_launch=False))
        a=common.load(source/'admission_reference.py','failure_diagnostic_admission')
        d=common.load(source/'d0_reference.py','failure_diagnostic_d0')
        immediate=a.funded_program('Immediate',d)
        diagnostics={}
        for case in pilot.cases():
            records=[r for line in (source/case['name']/'ticks.jsonl').read_text().splitlines() for r in json.loads(line)['records']]
            diagnostics[case['name']]=decompose(records,optimized['results'][case['name']]['opportunity'],immediate)
        common.require(len(diagnostics)==20,'complete diagnostic denominator')
        put(destination/'diagnostics.json',dict(profile=PROFILE,**FLAGS,cells=diagnostics,
            native_advances=0,causal_effect_identified=False,power_adequacy_established=False,ready_for_scientific_launch=False))
        after,_=common.inventory(source)
        common.require(before==after,'read-only input preservation')
        put(destination/'read_only_audit.json',dict(profile=PROFILE,**FLAGS,input_files=len(before),exact_inventory_and_hashes_preserved=True,
            input_sha256=before,elapsed_seconds=time.monotonic()-start))
        common.require(time.monotonic()-start<=3600,'analysis wall cap')
        common.require(sum(p.stat().st_size for p in destination.rglob('*') if p.is_file())<=256*1024**2,'analysis output cap')
        put(destination/'run_status.json',dict(profile=PROFILE,**FLAGS,engineering_execution_complete=True,
            power_adequacy_established=False,ready_for_scientific_launch=False,native_advances=0))
        manifest.update(state='Completed',end_time=datetime.now(timezone.utc).isoformat())
        (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        print(json.dumps(dict(profile=PROFILE,analysis_pass=True,cells=20,checkpoints=180,
            speedup=historical['audit_seconds']/optimized['audit_seconds'],resource_projection_pass=optimized['resource_decision']['resource_projection_pass'],
            ecological_feasibility_pass=optimized['decision']['ecological_feasibility_pass'],native_advances=0),indent=2))
    except Exception as error:
        manifest.update(state='Interrupted',end_time=datetime.now(timezone.utc).isoformat())
        (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        (destination/'failure.txt').write_text(repr(error)+'\n')
        if not (destination/'run_status.json').exists():
            put(destination/'run_status.json',dict(profile=PROFILE,**FLAGS,engineering_execution_complete=False,native_advances=0))
        raise


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('source');parser.add_argument('destination')
    args=parser.parse_args();analyze(args.source,args.destination)
