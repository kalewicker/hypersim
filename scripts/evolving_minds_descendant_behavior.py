#!/usr/bin/env python3
"""EM-16: finite behavior and inherited-class audit of sealed descendant evidence."""
import argparse
import copy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sys
import time
sys.dont_write_bytecode = True
import evolving_minds_opportunity as common
import evolving_minds_descendant_routes as routes

PROFILE = 'evolving-minds-d0-descendant-behavior-propagation-v1'
NATIVE_SEAL = 'b4e5bb279744e43eec686bf477444a9d60a1c1d6ea8fcb12ae965edbb5154e5b'
AUDIT_SEAL = '5925366f3e60cf9a73fb582b64e2041fae34a23a0c214fd27536999d3a842199'
FLAGS = dict(common.FLAGS, scientific_acceptance=False, sustained_claim=False, power_adequacy_established=False,
             ready_for_scientific_launch=False)
require = common.require
SOURCE_PATHS = ['scripts/evolving_minds_descendant_behavior.py',
 'scripts/test_evolving_minds_descendant_behavior.py',
 'experiments/evolving_minds/contracts/d0_descendant_behavior_propagation_v1.md',
 'scripts/evolving_minds_descendant_routes.py', 'scripts/evolving_minds_failure_audit.py',
 'scripts/evolving_minds_opportunity.py', 'scripts/evolving_minds_shared_replay.py']


def put(path, value):
    with Path(path).open('x') as out:
        json.dump(value, out, indent=2, sort_keys=True)
        out.write('\n')


def observer_type(base, collected):
    """Only observation overrides; all existing physical/lifetime checks still run."""
    class BehaviorObserver(base):
        def __init__(self, *args):
            self.behavior = {}
            super().__init__(*args)
            collected.append(self)

        def add(self, bid, reserve):
            super().add(bid, reserve)
            self.behavior.setdefault(str(bid), [])

        def begin(self, index, nodes):
            super().begin(index, nodes)
            phase, tick = index % 46, index // 46 + 1
            if phase not in (13, 16):
                return
            c, d = self.c, self.d
            for bid, root in self.roots(nodes).items():
                if phase == 13:
                    site_id = self.evidence['lineage'][str(bid)]['site']
                    site = next(n for n in nodes.values() if n['label'] == c.SITE and
                                c.value(n, 241009, 'Int') == site_id)
                    slots = {str(c.value(n, d.DEST, 'Int')): c.value(n, c.SIGNAL, 'Bool')
                             for n in nodes.values() if n['label'] in (c.SENSOR, d.STATE, c.ACTUATOR)
                             and c.value(n, c.OWNER, 'Int') == bid}
                    row = dict(tick=tick, site=site_id, cue=c.value(site, d.CUE, 'Bool'),
                        coin=c.value(site, d.COIN, 'Bool'), resource=c.value(site, d.RESOURCE, 'Bool'),
                        reserve=c.value(root, c.ENERGY, 'Float'), age=c.value(root, c.AGE, 'Int'),
                        cursor=c.value(root, d.CURSOR, 'Int'), status=c.value(root, d.STATUS, 'Int'),
                        has_actuator=c.value(root, d.HAS_ACT, 'Bool'),
                        actuator_signal=slots.get('3'), pending=c.value(root, d.ATTEMPT, 'Bool'),
                        previous_choice=c.value(root, d.CHOICE, 'Bool'), internal_slots=slots,
                        attempt_choice=None, settlement=None, harvested=0.0, maintenance=0.0,
                        request_charge=0.0, request_debit=0.0, death_cause=None, death_return=0.0)
                    rows = self.behavior.setdefault(str(bid), [])
                    require(not rows or rows[-1]['tick'] < tick, 'unique behavior tick')
                    rows.append(row)
                else:
                    row = self.behavior[str(bid)][-1]
                    require(row['tick'] == tick, 'funding observation tick')
                    reserve = c.value(root, c.ENERGY, 'Float')
                    ready = c.value(root, d.STATUS, 'Int') == 2 and c.value(root, c.AGE, 'Int') >= 2
                    funded = ready and reserve >= 2 + self.cfg['endowment'] + (11 + len(self.members[bid]['program']['instructions'])) / 4
                    offers = [x['target'] for x in self.evidence['admission']['rounds'][tick-1]['offers'] if x['parent'] == bid]
                    row.update(pre_request_reserve=reserve, age_ready=ready, funded=funded,
                               offered_targets=sorted(offers))

        def apply(self, nodes, edges, event, rule, index):
            # Advance first so a tick row exists even at the first accepted event.
            self.advance(index, nodes)
            bound = dict(zip(rule['lhs']['nodes'], map(self.c.handle, event['bound'])))
            root = nodes.get(bound.get('r'))
            bid = self.c.value(root, self.c.BID, 'Int') if root and root['label'] == self.c.ROOT else None
            before = self.c.value(root, self.c.ENERGY, 'Float') if bid is not None else None
            binding = super().apply(nodes, edges, event, rule, index)
            phase = index % 46
            if bid is not None and phase in (13, 14, 15, 16, 19):
                row = self.behavior[str(bid)][-1]
                require(row['tick'] == index // 46 + 1, 'event behavior tick')
                after = self.c.value(nodes[binding['r']], self.c.ENERGY, 'Float')
                debit = before - after
                if phase == 13:
                    require(row['attempt_choice'] is None, 'unique behavior attempt')
                    row['attempt_choice'] = self.c.value(nodes[binding['r']], self.d.CHOICE, 'Bool')
                elif phase == 14:
                    require(row['settlement'] is None, 'unique behavior settlement')
                    row.update(settlement=event['rule'], harvested=-debit)
                elif phase == 15:
                    row['maintenance'] += debit
                elif phase == 16:
                    row.update(request_charge=1.0, request_debit=debit)
                else:
                    row.update(death_cause=event['rule'], death_return=debit)
            return binding
    return BehaviorObserver


def validate_trace(result, traces):
    members = result['opportunity']['lifetimes']
    require(set(traces) == {str(m['id']) for m in members}, 'full behavior root denominator')
    for m in members:
        rows = traces[str(m['id'])]
        end = m['death_tick'] if m['death_tick'] is not None else 64
        require([r['tick'] for r in rows] == list(range(m['born_tick'] + 1, end + 1)), 'full behavior lifetime coverage')
        require(all((r['attempt_choice'] is None) == (r['settlement'] is None) for r in rows), 'behavior attempt/settlement coverage')
        require(sum(r['attempt_choice'] is not None for r in rows) == m['attempts'] and
                sum(r['settlement'] == 'settle_success' for r in rows) == m['successes'] and
                sum(r['settlement'] == 'settle_failure' for r in rows) == m['failures'] and
                sum(r['harvested'] for r in rows) == m['harvested'], 'behavior action counters')
        require(sum(r['maintenance'] for r in rows) == m['maintenance_charge'] and
                sum(r['request_charge'] for r in rows) == m['request_charge'] and
                sum(r['request_debit'] for r in rows) == m['request_debits'] and
                sum(r['death_return'] for r in rows) == m['death_return'], 'behavior paid counters')
        require(sum(r['funded'] for r in rows) == m['funded_parent_ticks'] and
                sum(bool(r['offered_targets']) for r in rows) == m['eligible_parent_ticks'] and
                sum(len(r['offered_targets']) for r in rows) == m['offered_targets'], 'behavior funding opportunities')


def compare_roots(target, control, target_trace, control_trace):
    if target is None:
        return dict(status='no_installed_child', finite_behavior_equal=False, universal_equivalence=False)
    require(control is not None, 'unchanged control child')
    a = [{k:v for k,v in r.items() if k != 'internal_slots'} for r in target_trace]
    b = [{k:v for k,v in r.items() if k != 'internal_slots'} for r in control_trace]
    boundary = {k:target[k] for k in ('born_tick','death_tick','completion_tick','death_cause','right_censored_at_64')}
    reference = {k:control[k] for k in boundary}
    require(len({r['tick'] for r in a}) == len(a) and len({r['tick'] for r in b}) == len(b), 'unique comparison ticks')
    aa, bb = {r['tick']:r for r in a}, {r['tick']:r for r in b}
    differences = [dict(tick=t, fields=sorted(k for k in set(aa.get(t,{})) | set(bb.get(t,{}))
                       if aa.get(t,{}).get(k) != bb.get(t,{}).get(k)), missing_side=
                       'target' if t not in aa else 'control' if t not in bb else None)
                   for t in sorted(set(aa)|set(bb)) if aa.get(t) != bb.get(t)]
    matched_birth = target['born_tick'] == control['born_tick']
    equal = boundary == reference and a == b
    return dict(status='finite_equal' if equal else 'finite_different' if matched_birth else 'unmatched_birth_exposure',
                finite_behavior_equal=equal, universal_equivalence=False,
                lifetime_boundaries_equal=boundary == reference, target_boundaries=boundary,
                control_boundaries=reference, target_observed_ticks=len(a), control_observed_ticks=len(b),
                first_divergence=None if not differences else differences[0], divergent_ticks=len(differences),
                internal_state_different_ticks=sum(x.get('internal_slots') != y.get('internal_slots')
                    for x,y in zip(target_trace,control_trace)) if [x['tick'] for x in a] == [x['tick'] for x in b] else None)


def class_propagation(result, target_id, d):
    members = {m['id']:m for m in result['opportunity']['lifetimes']}
    require(len(members) == len(result['opportunity']['lifetimes']), 'unique class roots')
    for m in members.values():
        if m['parent'] is not None:
            require(m['parent'] in members and members[m['parent']]['born_tick'] < m['born_tick'], 'class chronological parent edge')
    if target_id is None:
        return dict(installed=False, target_id=None, inherited_edges=[], same_class_descendant_ids=[])
    require(target_id in members, 'target class root')
    target = members[target_id]
    canon = d.canonical(target['program'])
    cls = [m for m in members.values() if d.canonical(m['program']) == canon]
    levels = {target_id:0}
    edges = []
    for m in sorted(members.values(), key=lambda x:(x['born_tick'],x['id'])):
        parent = m['parent']
        if m['id'] != target_id and parent in levels and d.canonical(m['program']) == canon:
            require(d.canonical(members[parent]['program']) == canon, 'same class transmission')
            levels[m['id']] = levels[parent] + 1
            edges.append(dict(parent=parent, child=m['id'], tick=m['born_tick'], inherited_generation=levels[m['id']]))
    origins = [m['id'] for m in cls if m['parent'] is None or d.canonical(members[m['parent']]['program']) != canon]
    inherited = [members[i] for i in levels]
    return dict(installed=True, target_id=target_id, canonical_bytes=list(canon), class_sha256=hashlib.sha256(canon).hexdigest(),
        class_origin_ids=sorted(origins), class_root_ids=sorted(m['id'] for m in cls),
        independent_class_roots_outside_target_lineage=sorted(m['id'] for m in cls if m['id'] not in levels),
        inherited_edges=edges, same_class_descendant_ids=sorted(i for i in levels if i != target_id),
        max_inherited_generation=max(levels.values()),
        completed_same_class_descendants=sum(m['id'] != target_id and m['completion_tick'] is not None for m in inherited),
        reproducing_same_class_descendants=sum(m['id'] != target_id and m['requests'] > 0 for m in inherited),
        target_root_death_tick=target['death_tick'], target_root_censored=target['death_tick'] is None,
        inherited_lineage_boundary_intervals=routes.boundary_intervals(inherited),
        whole_class_boundary_intervals=routes.boundary_intervals(cls),
        inherited_lineage_present_at64=any(m['death_tick'] is None for m in inherited),
        class_present_at64=any(m['death_tick'] is None for m in cls),
        same_class_descendants_outlive_target=target['death_tick'] is not None and any(m['id'] != target_id and
             (m['death_tick'] is None or m['death_tick'] > target['death_tick']) for m in inherited), nonadditive=True)


def analyze_results(fresh, traces, checker, a, d):
    require(set(fresh['results']) == {x['name'] for x in checker.cases()} == set(traces), 'complete case/trace denominator')
    decision = checker.decision(fresh['results'])
    require(decision == fresh['decision'], 'unchanged fixed target decision')
    cells = {}
    for name, result in fresh['results'].items():
        validate_trace(result, traces[name])
        cells[name] = routes.decompose_cell(result, a, d)
    targets = []
    for row in decision['targets']:
        name = f"world{row['world']}_{row['arm']}"
        result = fresh['results'][name]
        control_name = f"world{row['world']}_unchanged"
        control = next(r for r in decision['targets'] if r['world']==row['world'] and r['arm']=='unchanged' and r['founder']==row['founder'])
        target, baseline = row['target_child'], control['target_child']
        if target is not None:
            record = row['request']; baseline_record = control['request']
            require(record['tick'] == baseline_record['tick'] and record['site'] == baseline_record['site'], 'matched target birth/site')
        comparison = compare_roots(target, baseline, [] if target is None else traces[name][str(target['id'])],
                                   traces[control_name][str(baseline['id'])])
        targets.append(dict(world=row['world'], arm=row['arm'], founder=row['founder'], category=row['category'],
            target_id=None if target is None else target['id'], control_id=baseline['id'],
            behavior=comparison, propagation=class_propagation(result,None if target is None else target['id'],d)))
    require(len(targets)==144 and len({(t['world'],t['arm'],t['founder']) for t in targets})==144, '144 unique targets')
    substitutions=[t for t in targets if t['arm']!='unchanged']
    summary=dict(profile=PROFILE,**FLAGS,native_advances=0,cases=73,main_cells=72,targets=144,substitutions=140,
        all_root_records=sum(len(r['opportunity']['lifetimes']) for r in fresh['results'].values()),
        all_behavior_observations=sum(len(v) for cell in traces.values() for v in cell.values()),
        category_counts=decision['category_counts'],
        installed_substitution_roots=sum(t['propagation']['installed'] for t in substitutions),
        finite_equal_substitution_roots=sum(t['behavior']['finite_behavior_equal'] for t in substitutions),
        substitution_roots_with_same_class_descendants=sum(bool(t['propagation']['same_class_descendant_ids']) for t in substitutions),
        universal_equivalence_established=False, adaptive_improvement_established=False,
        reachable_edited_targets=[t for t in substitutions if t['category']=='reachable_edited'])
    return cells, targets, summary


def analyze(native, audited, destination):
    native, audited, destination = Path(native).resolve(), Path(audited).resolve(), Path(destination).absolute()
    start=time.monotonic()
    nb, ns=common.sealed_input(native,NATIVE_SEAL); ab, az=common.sealed_input(audited,AUDIT_SEAL)
    require(ns<=512*1024**2 and az<=512*1024**2, 'input caps')
    preservation={}
    for seal in sorted(native.parent.glob('evolving_minds_*/output_sha256.json')):
        files,size=common.inventory(seal.parent); raw=common.data(seal)
        require({k:v for k,v in files.items() if k!='output_sha256.json'}==raw['files_sha256'], 'prior archive inventory')
        preservation[str(seal.parent)]=dict(seal_sha256=files.pop('output_sha256.json'),files_sha256=files)
    require(len(preservation)==49 and sum(len(x['files_sha256'])+1 for x in preservation.values())==11883, 'fixed49 archive denominator')
    destination.mkdir()
    manifest=dict(experiment_name=PROFILE,run_id=destination.name,seed=0,thread_count=1,
        step_mode='archive-only; zero native advances',state='Running',start_time=datetime.now(timezone.utc).isoformat(),end_time=None,max_steps=0,final_step=0)
    (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    try:
        bundle=Path(__file__).parent/'analysis_sources.json'
        workspace=Path(__file__).resolve().parents[1]
        sources=common.data(bundle) if bundle.exists() else {p:(workspace/p).read_text() for p in SOURCE_PATHS}
        for module,key in [(sys.modules[__name__],SOURCE_PATHS[0]),(routes,SOURCE_PATHS[3]),(routes.base,SOURCE_PATHS[4]),(common,SOURCE_PATHS[5]),(routes.shared,SOURCE_PATHS[6])]:
            require(Path(module.__file__).read_text()==sources[key], 'executed source identity')
        require(Path(common.__file__).read_bytes()==(native/'opportunity_reference.py').read_bytes(), 'frozen observer helper binding')
        put(destination/'analysis_sources.json',sources)
        for p,raw in sources.items():
            name='tests.py' if p==SOURCE_PATHS[1] else 'contract.md' if p==SOURCE_PATHS[2] else Path(p).name
            (destination/name).write_text(raw)
        put(destination/'input_binding.json',dict(native_directory=str(native),audited_directory=str(audited),native_seal=NATIVE_SEAL,audit_seal=AUDIT_SEAL,native_files=len(nb),audit_files=len(ab),native_bytes=ns,audit_bytes=az))
        put(destination/'prior_archives.json',preservation)
        checker=common.load(native/'checker.py','behavior_frozen_checker')
        collected=[]; original=checker.load
        def instrument(path,name):
            module=original(path,name)
            if Path(path).name=='opportunity_reference.py':
                module.LifetimeObserver=observer_type(module.LifetimeObserver,collected)
            return module
        checker.load=instrument
        fresh=checker.verify(native)
        old=common.data(audited/'independent_replay.json')
        for k in set(fresh)|set(old):
            if k in ('audit_seconds','results'):continue
            require(fresh[k]==old[k], 'exact prior replay '+k)
        for name,result in fresh['results'].items():
            require({k:v for k,v in result.items() if k!='audit_seconds'}=={k:v for k,v in old['results'][name].items() if k!='audit_seconds'}, 'exact full result '+name)
        require(len(collected)==73 and fresh['published_states_reconstructed']==658, '73 observers/658 states')
        names=[x['name'] for x in checker.cases()]
        traces={name:obs.behavior for name,obs in zip(names,collected)}
        a=common.load(native/'admission_reference.py','behavior_admission');d=common.load(native/'d0_reference.py','behavior_d0')
        cells,targets,summary=analyze_results(fresh,traces,checker,a,d)
        put(destination/'independent_replay.json',fresh)
        put(destination/'behavior_traces.json',traces)
        put(destination/'classes_and_routes.json',cells)
        put(destination/'target_comparisons.json',targets)
        put(destination/'summary.json',summary)
        put(destination/'replay_comparison.json',dict(profile=PROFILE,**FLAGS,exact_complete_results_equal=True,published_states=658,excluded_equality_fields=['audit_seconds'],native_advances=0,observer_changes_physical_state=False))
        require(common.inventory(native)[0]==nb and common.inventory(audited)[0]==ab, 'read-only inputs')
        for directory,receipt in preservation.items():
            files,_=common.inventory(Path(directory))
            require(files.pop('output_sha256.json')==receipt['seal_sha256'] and files==receipt['files_sha256'], 'prior archives preserved')
        put(destination/'preservation_receipt.json',dict(profile=PROFILE,**FLAGS,prior_archives_verified=49,prior_files_verified=11883,exact_inventory_and_hashes_preserved=True,native_advances=0))
        require(time.monotonic()-start<=3600 and common.inventory(destination)[1]<=256*1024**2,'analysis caps')
        put(destination/'run_status.json',dict(profile=PROFILE,**FLAGS,engineering_execution_complete=True,analysis_pass=True,native_advances=0,analysis_seconds=time.monotonic()-start))
        manifest.update(state='Completed',end_time=datetime.now(timezone.utc).isoformat())
        (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        print(json.dumps({k:v for k,v in summary.items() if k!='reachable_edited_targets'},indent=2))
    except Exception as error:
        manifest.update(state='Interrupted',end_time=datetime.now(timezone.utc).isoformat())
        (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        (destination/'failure.txt').write_text(repr(error)+'\n')
        put(destination/'run_status.json',dict(profile=PROFILE,**FLAGS,engineering_execution_complete=False,native_advances=0))
        raise


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for arg in ['native','audited','destination']:parser.add_argument(arg)
    args=parser.parse_args();analyze(args.native,args.audited,args.destination)
