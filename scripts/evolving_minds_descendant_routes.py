#!/usr/bin/env python3
"""Reconcile paid descendant proposals, root lifetimes and genotype classes."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import time
import evolving_minds_opportunity as common
import evolving_minds_failure_audit as base
import evolving_minds_shared_replay as shared

PROFILE = 'evolving-minds-d0-descendant-route-diagnosis-v1'
NATIVE_SEAL = '289591b6e0fcdf864a0618c2ac3477a04eeb83c233822ac15ee4f4bcdaff5981'
AUDIT_SEAL = '7f23352df4f56ddb08aef797e08e4a6d4d8f5f6e93c71237f00dae4acff54809'
FLAGS = dict(common.FLAGS, scientific_acceptance=False, sustained_claim=False,
             power_adequacy_established=False, ready_for_scientific_launch=False)
WORLDS = [401, 403, 407, 409]
CATEGORIES = ['syntax_rejected', 'paid_unreachable', 'reachable_unchanged', 'reachable_edited']
SOURCE_PATHS = ['scripts/evolving_minds_descendant_routes.py',
    'scripts/test_evolving_minds_descendant_routes.py',
    'experiments/evolving_minds/contracts/d0_descendant_route_diagnosis_v1.md',
    'scripts/evolving_minds_failure_audit.py', 'scripts/evolving_minds_opportunity.py',
    'scripts/evolving_minds_shared_replay.py']
require = common.require


def put(path, value):
    with path.open('x') as handle:
        json.dump(value, handle, indent=2, sort_keys=True)
        handle.write('\n')


def program_hash(program, d):
    return hashlib.sha256(d.canonical(program)).hexdigest()


def boundary_intervals(members):
    """Post-cleanup presence, including births and excluding a death boundary."""
    present = [t for t in range(65) if any(m['born_tick'] <= t and
               (m['death_tick'] is None or m['death_tick'] > t) for m in members)]
    intervals = []
    for tick in present:
        if intervals and intervals[-1]['last_observed_tick'] + 1 == tick:
            intervals[-1]['last_observed_tick'] = tick
        else:
            intervals.append(dict(first_observed_tick=tick, last_observed_tick=tick))
    return intervals


def decompose_cell(result, a, d):
    records, opportunity = result['records'], result['opportunity']
    require(len(opportunity['ticks']) == 64 and
            [t['tick'] for t in opportunity['ticks']] == list(range(1, 65)), 'full64tick horizon')
    members = {m['id']: m for m in opportunity['lifetimes']}
    require(len(members) == len(opportunity['lifetimes']) and
            {m['id'] for m in members.values() if m['parent'] is None} == {1, 2}, 'complete lifetime/founder denominator')
    require(all(r['request_id'] == r['child_id'] for r in records if r['child_id'] is not None), 'native child/request identity')
    incoming = {r['child_id']: r for r in records if r['child_id'] is not None}
    require(len(incoming) == sum(r['child_id'] is not None for r in records), 'unique incoming child requests')
    depth, founder, ancestry = {}, {}, {}
    for m in sorted(members.values(), key=lambda m: (m['born_tick'], m['id'])):
        parent = m['parent']
        require(0 <= m['born_tick'] <= 64 and
                (m['death_tick'] is None or m['born_tick'] <= m['death_tick'] <= 64), 'root lifetime bounds')
        if parent is None:
            depth[m['id']], founder[m['id']], ancestry[m['id']] = 0, m['id'], []
            require(m['born_tick'] == 0, 'founder birth boundary')
        else:
            require(parent in depth and members[parent]['born_tick'] < m['born_tick'], 'chronological ancestry')
            depth[m['id']] = depth[parent] + 1
            founder[m['id']] = founder[parent]
            ancestry[m['id']] = ancestry[parent] + [parent]
        require(m['founder'] == founder[m['id']] and
                m['right_censored_at_64'] == (m['death_tick'] is None) and
                m['live_start_ticks'] == (64 if m['death_tick'] is None else m['death_tick']) - m['born_tick'], 'root ancestry/censoring/clock')
        require(m['attempts'] == m['successes'] + m['failures'] == m['payable_action_opportunities'], 'paid action counters')
        if m['completion_tick'] is not None:
            require(m['born_tick'] < m['completion_tick'] <= (m['death_tick'] or 64) and
                    m['cursor'] == m['development_paid'] == len(m['program']['instructions']) and
                    m['development_failures'] == 0, 'completed native construction')
        own = [r for r in records if r['parent'] == m['id']]
        require(m['requests'] == len(own) and m['committed_children'] == sum(r['child_id'] is not None for r in own) and
                m['invalid_requests'] == sum(not r['syntax_valid'] for r in own), 'lifetime request counters')
        require(m['request_charge'] == math.fsum(r['request_charge'] for r in own) and
                m['copying_charge'] == math.fsum(r['copying_charge'] for r in own) and
                m['refunds'] == math.fsum(r['refunded'] for r in own) and
                m['child_endowment_transferred'] == 24 * sum(r['child_id'] is not None for r in own), 'lifetime request payments')
    for r in records:
        require(r['parent'] in members, 'request parent present')
        parent = members[r['parent']]
        require(parent['born_tick'] < r['tick'] <= (parent['death_tick'] or 64) and
                r['parent_program'] == parent['program'], 'request parent lifetime/genotype')
        require(r['parent_bytes'] == list(d.canonical(r['parent_program'])) and
                r['proposed_bytes'] == list(d.canonical(r['proposed_program'])), 'canonical proposal byte provenance')
        q = r['proposed_program']
        require(r['syntax_valid'] == a.typed_valid(q) and
                r['invalid_reason'] == (None if r['syntax_valid'] else 'D0 typed parameter/order grammar') and
                r['material_reachable'] == (r['syntax_valid'] and a.reachable(q)), 'proposal grammar/material status')
        p = r['parent_program']
        if r['target_id'] is None:
            require(r['old_source'] is None and r['new_source'] is None and p == q, 'unchanged proposal metadata')
        else:
            target = [g for g in p['instructions'] if g['id'] == r['target_id']]
            require(len(target) == 1 and target[0]['source'] == r['old_source'] and r['old_source'] != r['new_source'], 'edit target/old/new provenance')
            expected = json.loads(json.dumps(p))
            next(g for g in expected['instructions'] if g['id'] == r['target_id'])['source'] = r['new_source']
            require(q == expected, 'single declared source edit')
    original = base.decompose(records, opportunity, a.funded_program('Immediate', d))
    require(original['descendant_count'] == opportunity['descendant_denominator'] and
            result['final_row']['births'] == original['descendant_count'] and
            result['final_row']['max_ancestry_depth'] == max(depth.values()) and
            result['final_row']['harvested'] == opportunity['total_harvested'], 'complete lineage/final endpoint')
    require(opportunity['descendants_completed'] == sum(m['completion_tick'] is not None for m in members.values() if m['parent'] is not None) and
            opportunity['descendants_grammar_failed'] == sum(m['development_failures'] > 0 for m in members.values() if m['parent'] is not None) and
            opportunity['descendants_live_at_64'] == sum(m['death_tick'] is None for m in members.values() if m['parent'] is not None), 'complete descendant outcome counters')
    classes = {}
    for m in members.values():
        sha = program_hash(m['program'], d)
        classes.setdefault(sha, dict(program=m['program'], roots=[]))['roots'].append(m)
    catalogs = {}
    for sha, row in sorted(classes.items()):
        roots = sorted(row['roots'], key=lambda m: (m['born_tick'], m['id']))
        first = min(m['born_tick'] for m in roots)
        catalogs[sha] = dict(program=row['program'], first_install_tick=first,
            first_installation_cohort=[m['id'] for m in roots if m['born_tick'] == first],
            root_ids=[m['id'] for m in roots], completed_root_ids=[m['id'] for m in roots if m['completion_tick'] is not None],
            paid_reproducing_root_ids=[m['id'] for m in roots if m['requests'] > 0],
            surviving_root_ids=[m['id'] for m in roots if m['death_tick'] is None],
            class_present_at_64=any(m['death_tick'] is None for m in roots),
            post_cleanup_boundary_intervals=boundary_intervals(roots),
            harvested=math.fsum(m['harvested'] for m in roots), requests=sum(m['requests'] for m in roots))
    routes = []
    for route in original['routes']:
        r = next(r for r in records if r['request_id'] == route['request_id'])
        child = members.get(r['child_id'])
        cohort = None if child is None else catalogs[program_hash(child['program'], d)]
        subtree = [] if child is None else [m for m in members.values() if m['id'] == child['id'] or child['id'] in ancestry[m['id']]]
        downstream = [q for q in records if any(q['parent'] == m['id'] for m in subtree)]
        route.update(parent_depth=depth[r['parent']], founder=founder[r['parent']],
            proposed_bytes_changed=r['parent_bytes'] != r['proposed_bytes'],
            class_installed=child is not None,
            class_first_install_tick=None if cohort is None else cohort['first_install_tick'],
            first_installation_cohort=False if child is None else child['born_tick'] == cohort['first_install_tick'],
            class_present_at_64=False if cohort is None else cohort['class_present_at_64'],
            class_boundary_intervals=[] if cohort is None else cohort['post_cleanup_boundary_intervals'],
            root_right_censored_at_64=False if child is None else child['right_censored_at_64'],
            child_depth=None if child is None else depth[child['id']],
            child_request_ids=[] if child is None else sorted(q['request_id'] for q in records if q['parent'] == child['id']),
            child_committed_child_ids=[] if child is None else sorted(q['child_id'] for q in records if q['parent'] == child['id'] and q['child_id'] is not None),
            subtree=dict(nonadditive=True,root_ids=sorted(m['id'] for m in subtree),
                requests=len(downstream), harvested=math.fsum(m['harvested'] for m in subtree),
                living_root_ids_at_64=sorted(m['id'] for m in subtree if m['death_tick'] is None),
                subsequent_byte_edit_request_ids=sorted(q['request_id'] for q in downstream if q['parent_bytes'] != q['proposed_bytes'])))
        routes.append(route)
    partitions = {}
    for category in CATEGORIES:
        selected = [r for r in routes if r['category'] == category]
        children = [members[r['child_id']] for r in selected if r['child_id'] is not None]
        partitions[category] = dict(requests=len(selected),child_roots=len(children),
            completed_roots=sum(m['completion_tick'] is not None for m in children),
            reproducing_roots=sum(m['requests'] > 0 for m in children),
            surviving_roots_at_64=sum(m['death_tick'] is None for m in children),
            direct_child_harvested=math.fsum(m['harvested'] for m in children))
    require(math.fsum(x['direct_child_harvested'] for x in partitions.values()) == opportunity['primary_descendant_harvested'], 'additive incoming-route yield partition')
    original.update(routes=routes, classes=catalogs, incoming_route_partition=partitions,
                    genotype_classes_installed=len(catalogs), full_lifetimes_verified=len(members), subtree_totals_nonadditive=True)
    return original


def summarize(cells):
    expected = {f'world{w}_{arm}' for w in WORLDS for arm in ['faithful', 'source']} | {
        'world17_unchanged', 'world17_invalid', 'world17_unreachable', 'world17_recovery'}
    require(set(cells) == expected, 'complete twelve-case diagnosis denominator')
    focus = [dict(cell=name, **r) for name in sorted(cells) if name.endswith('_source')
             for r in cells[name]['routes'] if r['stage'] == 'descendant']
    require(len(focus) == 31 and all(r['operator'] == 'source' for r in focus), 'complete31 sampled descendant requests')
    counts = Counter(r['category'] for r in focus)
    require(counts == dict(zip(CATEGORIES, [12, 5, 10, 4])), 'fixed source route category denominator')
    groups = {}
    for category in CATEGORIES:
        selected = [r for r in focus if r['category'] == category]
        children = [r for r in selected if r['child_id'] is not None]
        groups[category] = dict(requests=len(selected),installed_roots=len(children),
            completed_roots=sum(r['child']['completion_tick'] is not None for r in children),
            reproducing_roots=sum(r['child']['requests'] > 0 for r in children),
            surviving_roots_at_64=sum(r['root_right_censored_at_64'] for r in children),
            direct_child_harvested=math.fsum(r['child']['harvested'] for r in children),
            request_charge=math.fsum(r['request_charge'] for r in selected),
            copying_charge=math.fsum(r['copying_charge'] for r in selected),
            refunds=math.fsum(r['refunded'] for r in selected),
            endowment_transferred=math.fsum(r['child_endowment_transferred'] for r in selected))
    worlds = []
    for w in WORLDS:
        faithful, source = cells[f'world{w}_faithful'], cells[f'world{w}_source']
        worlds.append(dict(world=w, faithful_descendant_harvested=faithful['descendant_yield'],
            source_descendant_harvested=source['descendant_yield'], difference=source['descendant_yield'] - faithful['descendant_yield'],
            source_categories=dict(Counter(r['category'] for r in source['routes'] if r['stage'] == 'descendant')),
            faithful_descendant_requests=faithful['descendant_requests'], source_descendant_requests=source['descendant_requests']))
    edited = [r for r in focus if r['category'] == 'reachable_edited']
    return dict(profile=PROFILE, **FLAGS, native_advances=0, full_cases=12,
        full_requests=sum(x['requests'] for x in cells.values()),
        full_organism_records=sum(x['full_lifetimes_verified'] for x in cells.values()),
        focused_descendant_requests=31, categories=groups, worlds=worlds,
        reachable_edited_routes=edited, causal_effect_of_individual_edit_identified=False,
        mean_paired_difference=math.fsum(w['difference'] for w in worlds) / 4)


def analyze(native, audited, destination, preservation_snapshot):
    native, audited, destination = Path(native).resolve(), Path(audited).resolve(), Path(destination).absolute()
    start = time.monotonic()
    native_before, native_size = common.sealed_input(native, NATIVE_SEAL)
    audit_before, audit_size = common.sealed_input(audited, AUDIT_SEAL)
    destination.mkdir()
    manifest = dict(run_id='paid_handoff_descendant_routes_v1',experiment_name=PROFILE,seed=0,
        thread_count=1,step_mode='archive-only; zero native advances',state='Running',
        start_time=datetime.now(timezone.utc).isoformat(),end_time=None,max_steps=0,final_step=0)
    (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    try:
        if (Path(__file__).parent/'analysis_sources.json').exists():
            sources = common.data(Path(__file__).parent/'analysis_sources.json')
        else:
            workspace = Path(__file__).resolve().parents[1]
            sources = {p: (workspace/p).read_text() for p in SOURCE_PATHS}
        require(Path(__file__).read_text() == sources[SOURCE_PATHS[0]] and
                Path(base.__file__).read_text() == sources[SOURCE_PATHS[3]] and
                Path(common.__file__).read_text() == sources[SOURCE_PATHS[4]] and
                Path(shared.__file__).read_text() == sources[SOURCE_PATHS[5]], 'executed analysis/dependency source identity')
        require(Path(common.__file__).read_bytes() == (native/'opportunity_reference.py').read_bytes(), 'frozen physical/helper dependency')
        put(destination/'analysis_sources.json', sources)
        for filename, key in [('evolving_minds_descendant_routes.py',SOURCE_PATHS[0]),('tests.py',SOURCE_PATHS[1]),
            ('contract.md',SOURCE_PATHS[2]),('evolving_minds_failure_audit.py',SOURCE_PATHS[3]),
            ('evolving_minds_opportunity.py',SOURCE_PATHS[4]),('evolving_minds_shared_replay.py',SOURCE_PATHS[5])]:
            with (destination/filename).open('x') as handle: handle.write(sources[key])
        put(destination/'input_binding.json', dict(profile=PROFILE,**FLAGS,native_directory=str(native),
            native_seal=NATIVE_SEAL,native_files=len(native_before),native_bytes=native_size,native_sha256=native_before,
            audited_directory=str(audited),audit_seal=AUDIT_SEAL,audit_files=len(audit_before),audit_bytes=audit_size,audit_sha256=audit_before))
        preservation = common.data(Path(preservation_snapshot))
        require(len(preservation) == 46, '46 prior sealed archives')
        put(destination/'prior_archives.json', preservation)
        repair = common.load(audited/'audit_continuation.py', 'route_frozen_repair')
        checker = common.load(native/'checker.py', 'route_frozen_checker')
        verifier, derivation = repair.repaired_function(checker, (native/'checker.py').read_text())
        require(derivation == common.data(audited/'derivation.json'), 'exact sealed repair AST')
        fresh = verifier(native)
        historical = common.data(audited/'independent_replay.json')
        require(fresh['decision'] == historical['decision'] and fresh['source_sha256'] == historical['source_sha256'] and
                fresh['derivations'] == historical['derivations'] and fresh['resources'] == historical['resources'], 'unchanged decisions/sources/AST/resources')
        require(set(fresh['results']) == set(historical['results']) and fresh['published_states_reconstructed'] == 109, 'all109 reconstructed states')
        for name, result in fresh['results'].items():
            require({k:v for k,v in result.items() if k != 'audit_seconds'} ==
                    {k:v for k,v in historical['results'][name].items() if k != 'audit_seconds'}, 'exact full route/lifetime replay equality')
        put(destination/'independent_replay.json', fresh)
        put(destination/'replay_comparison.json', dict(profile=PROFILE,**FLAGS,exact_full_results_equal=True,
            published_states=109,excluded_equality_fields=['audit_seconds'],decisions_unchanged=True,
            metadata_repair_ast=derivation,native_advances=0))
        a = common.load(native/'admission_reference.py', 'route_admission')
        d = common.load(native/'d0_reference.py', 'route_d0')
        cells = {name: decompose_cell(result, a, d) for name,result in fresh['results'].items()}
        summary = summarize(cells)
        require(summary['full_organism_records'] == 113, 'all113 lifetime records retained')
        put(destination/'routes.json', dict(profile=PROFILE,**FLAGS,cells=cells,native_advances=0))
        put(destination/'summary.json', summary)
        require(common.inventory(native)[0] == native_before and common.inventory(audited)[0] == audit_before, 'read-only source inputs')
        for directory, receipt in preservation.items():
            p = Path(directory)
            files, _ = common.inventory(p)
            require(files.pop('output_sha256.json') == receipt['seal_sha256'] and files == receipt['files_sha256'], 'all prior archives preserved')
        put(destination/'preservation_receipt.json', dict(profile=PROFILE,**FLAGS,prior_archives_verified=46,
            prior_files_verified=sum(len(x['files_sha256'])+1 for x in preservation.values()),exact_inventory_and_hashes_preserved=True,
            source_inputs_read_only=True,native_advances=0))
        require(time.monotonic()-start <= 3600 and common.inventory(destination)[1] <= 256*1024**2, 'diagnosis resource caps')
        put(destination/'run_status.json',dict(profile=PROFILE,**FLAGS,engineering_execution_complete=True,
            analysis_pass=True,native_advances=0))
        manifest.update(state='Completed',end_time=datetime.now(timezone.utc).isoformat())
        (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        print(json.dumps(dict(profile=PROFILE,analysis_pass=True,native_advances=0,
            focused_requests=31,categories=summary['categories'],edited_routes=summary['reachable_edited_routes'],
            analysis_seconds=time.monotonic()-start),indent=2))
    except Exception as error:
        manifest.update(state='Interrupted',end_time=datetime.now(timezone.utc).isoformat())
        (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        (destination/'failure.txt').write_text(repr(error)+'\n')
        if not (destination/'run_status.json').exists():
            put(destination/'run_status.json',dict(profile=PROFILE,**FLAGS,engineering_execution_complete=False,native_advances=0))
        raise


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('native');parser.add_argument('audited');parser.add_argument('destination')
    parser.add_argument('--preservation-snapshot',required=True)
    args=parser.parse_args()
    analyze(args.native,args.audited,args.destination,args.preservation_snapshot)
