#!/usr/bin/env python3
"""Exploratory coverage review of sealed ERE receipts; never invokes native code."""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
import math
from pathlib import Path
import shutil
import subprocess
import sys
import time

import audit_ere_categorical_feasible as pinned

a, cat, ROOT = pinned.a, pinned.cat, pinned.ROOT
mapping = cat.mapping
PROFILE = 'E2-saved-coverage-headroom-review-v1'
# Fixed before this analyzer's first invocation. Completed production-grammar
# campaigns only; substrate/scaling failures and synthetic fixtures are excluded.
INPUTS = [
    ('ere_complete_short_training_continuation_20260930', 'dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132', 1690, 1208),
    ('ere_complete_champion_confirmation_20261001', '822bea3a828ce6e59324dd13e5c133eba4ffb9a75ab0c8a1102c2346fe70c0bb', 6598, 6144),
    ('ere_complete_feasibility_training_20261001', 'f51ba18e5cabdedae4bab811e1cb6a3c86654b21355fbad8d07156f95f853432', 6944, 6464),
    ('ere_complete_minibatch_trial_20261001', '49a2bca085a7f7619c2bde3b5a25992a8fc16d8418d97165a2db3fb9db698806', 10152, 9472),
    ('ere_complete_direction_assay_20261001', 'ba88c696014fa42ca612f4c20baadb1818cc6c5757fd41594c87a1e4bc1a185a', 7686, 7168),
    ('ere_categorical_comparison_scientific_20261003', '387a6e32a152250a081b246e53e5a69adbd273c4f465f726594a256ca3600fc1', 12883, 12288),
    ('ere_categorical_feasible_development_20261003', 'eb6c262b7fdc5299b2133f09a83609354b3d55f05a6825c27434142c55f667b7', 3347, 2176),
    ('ere_categorical_comparison_development_20261002', '3a9d48f7059dd910a8e008a566237cbf03935790629902f1b9305d496ecf3682', 692, 192),
    ('ere_complete_minibatch_development_20261001', 'c626db144036da9a4e305996d289b7997cf38b1a57bbe7edc20f3833bcb88dc3', 557, 88),
    ('ere_complete_direction_development_20261001', 'ee9f28214bf67aebba4410b60f269ae6d3935acd5929e372f55527d51a035361', 684, 208),
]
SUPPLEMENT = ('ere_categorical_comparison_historical_audit_20261003',
              'a33b236aaa4d2ade164f54e067be03f0a1313088cfef537ecfec1d8133f2e3e9', 14)
RUNTIME = ('ere_runtime50k_signal_panel_20260930',
           '0099ecae4b7487e9ac975f73d813e7cbde0de430470dc66198bbafd7df16488c', 460)
TARGET = (4, 11, 1, 0)


def object_hash(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                     allow_nan=False).encode()).hexdigest()


def outcome(row):
    v = row['evaluation']
    score = v['score']
    assert (score is None and v['reason'] == 'resource_cap') or (
        score is not None and 0 <= score <= 1 and v['reason'] is None)
    utility = 0. if score is None else 1. + score
    if 'constraint_utility' in row:
        assert math.isclose(row['constraint_utility'], utility, abs_tol=1e-12)
    return dict(score=score, utility=utility, cap=score is None,
                reason=v['reason'], fires=v['proposal_fires'],
                trace_sha256=object_hash(v['steps']))


def add_observation(units, key, value, provenance):
    """Repeated leaf/physics units cannot be made independent by copying rows."""
    if key in units:
        assert units[key]['outcome'] == value, 'conflicting repeated leaf/physics outcome'
        units[key]['receipts'].append(provenance)
    else:
        units[key] = dict(outcome=value, receipts=[provenance])


def variance_summary(observations):
    """Descriptive variance across distinct physics IDs, not a population CI."""
    by_leaf = defaultdict(list)
    for key, item in observations.items():
        by_leaf[key[:-1]].append(item['outcome'])
    panels = [items for items in by_leaf.values() if len(items) >= 2]
    def varies(items, fields):
        return len({tuple(item[f] for f in fields) for item in items}) > 1
    variances = []
    for items in panels:
        us = [item['utility'] for item in items]
        mean = math.fsum(us)/len(us)
        variances.append(math.fsum((u-mean)**2 for u in us)/(len(us)-1))
    return dict(leaves=len(by_leaf), leaves_with_multiple_physics_seeds=len(panels),
                distinct_leaf_physics_units=len(observations),
                utility_varying_leaves=sum(varies(x, ['utility']) for x in panels),
                cap_varying_leaves=sum(varies(x, ['cap']) for x in panels),
                score_reason_varying_leaves=sum(varies(x, ['score', 'reason']) for x in panels),
                trace_varying_leaves=sum(varies(x, ['trace_sha256']) for x in panels),
                maximum_sample_utility_variance=max(variances, default=None),
                mean_sample_utility_variance=math.fsum(variances)/len(variances) if variances else None)


def leaf_means(units):
    by_leaf = defaultdict(list)
    for key, item in units.items():
        by_leaf[key[:-1]].append(item['outcome'])
    return {key: dict(physics_seeds=len(items),
                      utility=math.fsum(x['utility'] for x in items)/len(items),
                      cap_rate=math.fsum(x['cap'] for x in items)/len(items),
                      positive_scores=sum(x['score'] is not None and x['score'] > 0 for x in items),
                      minimum_utility=min(x['utility'] for x in items),
                      maximum_utility=max(x['utility'] for x in items))
            for key, items in by_leaf.items()}


def coverage_bounds(masses, means, leaves):
    """Missing-support bounds CONDITIONAL on recorded means, not confidence bounds.

    Unobserved leaves retain their full mass. Structurally ineligible leaves
    cannot earn reuse, but their resource-cap outcome is still unknown.
    """
    assert math.isclose(math.fsum(masses.values()), 1., abs_tol=1e-12)
    known = set(means) & set(masses)
    seen = math.fsum(masses[key] for key in known)
    lower = math.fsum(masses[key]*means[key]['utility'] for key in known)
    risk = math.fsum(masses[key]*means[key]['cap_rate'] for key in known)
    unknown_upper = math.fsum(masses[key]*(2. if leaves[key]['eligible'] else 1.)
                              for key in masses if key not in known)
    return dict(observed_probability_mass=seen, missing_probability_mass=1.-seen,
                conditional_utility_lower=lower, conditional_utility_upper=lower+unknown_upper,
                conditional_cap_lower=risk, conditional_cap_upper=risk+1.-seen,
                population_confidence_bound=False, observed_support_renormalized=False)


def group_summary(means, groups):
    by_group = defaultdict(list)
    for key, value in means.items():
        by_group[key[:3]].append((key, value))
    records = []
    for key, group in sorted(groups.items()):
        seen = by_group[key]
        us = [v['utility'] for _, v in seen]
        caps = [v['cap_rate'] for _, v in seen]
        complete = len(seen) == group['count']
        records.append(dict(group=list(key), production_count=group['count'], observed_productions=len(seen),
            complete=complete, observed_utility_range=max(us)-min(us) if us else None,
            observed_cap_range=max(caps)-min(caps) if caps else None,
            # Only full production coverage yields the group's unrenormalized
            # empirical average; means can have different physics seed panels.
            complete_empirical_uniform_utility=math.fsum(us)/group['count'] if complete else None,
            observed_best_utility=max(us) if us else None,
            uniform_production_penalty=max(us)-math.fsum(us)/len(us) if complete else None))
    return records


def policies():
    # Analytic selectors, not fitted candidates or fresh measured policies.
    zero = [0.]*183
    alternating = [.25 if i % 2 == 0 else -.25 for i in range(183)]
    concentrated = [-8.]*183
    for i in (4, 168+11, 181+1):
        concentrated[i] = 8.
    return [('zero', zero), ('alternating_025', alternating),
            ('positive_control_selector_pm8', concentrated)]


def seal_inputs():
    for name, root, count, _ in INPUTS:
        a.verify(ROOT/'output'/name, expected=root, count=count)
    for name, root, count in (SUPPLEMENT, RUNTIME):
        a.verify(ROOT/'output'/name, expected=root, count=count)


def analyze():
    seal_inputs()
    snapshots = []
    for name, _, _, _ in INPUTS:
        p = ROOT/'output'/name
        m = a.load(p/'manifest.json')
        assert m['status'] == 'complete'
        snapshots.append(a.load(p/'snapshot.json') if (p/'snapshot.json').exists() else m['snapshot'])
    # This coarse source grouping is only an exploratory reporting partition.
    # Retain every full source map and binary identity. No replay/cache admission.
    common = set.intersection(*(set(s['sources']) for s in snapshots))
    paths = sorted(n for n in common if (n.startswith('crates/') and '/src/' in n
                   and n.endswith('.rs') and not n.startswith('crates/hg-train/'))
                   or n in ('Cargo.toml', 'Cargo.lock') or (
                   n.startswith('crates/') and n.endswith('/Cargo.toml')
                   and not n.startswith('crates/hg-train/')))
    assert 'crates/hg-sim-core/src/ere/signal.rs' in paths
    catalogue = a.load(ROOT/'output'/INPUTS[0][0]/'catalogue.json')
    leaves, groups, counts = mapping.catalogue_model(catalogue)
    policy_masses = {}
    for label, theta in policies():
        prob = mapping.probabilities(catalogue, theta, groups)
        policy_masses[label] = {k: prob['group_mass'][k[:3]]/groups[k[:3]]['count'] for k in leaves}
    native, helper = cat.d.c.helpers(ROOT/'output'/INPUTS[0][0])
    all_units = {}; identities = {}; archives = []; source_partitions = {}; cohort_units = defaultdict(dict)
    attempted = advancing = 0
    for (name, root, entries, cell_count), snap in zip(INPUTS, snapshots):
        p = ROOT/'output'/name
        assert a.digest(p/'catalogue.json') == a.CATALOGUE
        pinned.history.verify_snapshot_sources(p/'source_snapshots', snap['sources'])
        if (p/'executed_binary').exists():
            assert a.digest(p/'executed_binary') == snap['binary_sha256']
        cohort = object_hash({n: snap['sources'][n] for n in paths})
        source_partitions.setdefault(cohort, dict(paths={n: snap['sources'][n] for n in paths}, archives=[]))['archives'].append(name)
        if name == INPUTS[5][0]:
            receipt = a.load(ROOT/'output'/SUPPLEMENT[0]/'audit.json')
            assert receipt['archive_accounting_pass'] and not receipt['original_frozen_audit_pass']
            audit_scope = 'separate corrected historical accounting; original failure preserved'
        else:
            receipt = a.load(p/'audit.json')
            assert receipt['audit_pass']
            audit_scope = 'saved accounting audit_pass'
        manifest = a.load(p/'manifest.json')
        schedule = a.load(p/'schedule.json') if (p/'schedule.json').exists() else manifest['schedule']
        assert len(schedule) == cell_count and len({q['id'] for q in schedule}) == cell_count
        assert {f.stem for f in (p/'cells').glob('*.json')} == {q['id'] for q in schedule}
        units = {}; stages = defaultdict(Counter); observed = set(); controls = 0
        for q in schedule:
            row = a.load(p/'cells'/(q['id']+'.json'))
            assert row['schedule'] == q and q['horizon'] == 100
            at, ad, _ = native.native(row, catalogue, helper)
            attempted += at; advancing += ad
            assert row['status'] in ('Complete', 'Null'), 'complete archive contains unknown work'
            e = row['emission']; key = cat.key(cat.selection(e)); assert key in leaves
            assert e['signature'] == leaves[key]['signature']
            assert bool(row['evaluation']['credited_rule_ids']) == leaves[key]['eligible']
            identity = object_hash({k: e[k] for k in ('classes', 'signature', 'initial_world_hash', 'root')})
            if key in identities:
                assert identities[key] == identity, 'same leaf has different emitted semantics'
            identities[key] = identity
            value = outcome(row); unit = key+(q['physics_seed'],)
            provenance = dict(archive=name, id=q['id'], stage=q['stage'],
                              emission_seed=q['emission_seed'], physics_seed=q['physics_seed'])
            add_observation(units, unit, value, provenance)
            add_observation(cohort_units[cohort], unit, value, provenance)
            add_observation(all_units, (cohort,)+unit, value, provenance)
            observed.add(key); controls += q['id'].startswith('control-')
            st = stages[q['stage']]; st['rows'] += 1; st['caps'] += value['cap']
            st['positive_scores'] += value['score'] is not None and value['score'] > 0
            st['utility_sum'] += value['utility']
        archives.append(dict(name=name, root_sha256=root, seal_entries=entries, cells=cell_count,
            source_partition=cohort, audit_scope=audit_scope, distinct_leaves=len(observed),
            full_source_map=snap['sources'], binary_sha256=snap['binary_sha256'],
            distinct_groups=len({k[:3] for k in observed}), duplicate_leaf_physics_rows=cell_count-len(units),
            forced_control_rows=controls, variance=variance_summary(units),
            stages={k: dict(v) for k, v in stages.items()},
            catalogue_coverage=[dict(policy=label, **coverage_bounds(masses, leaf_means(units), leaves))
                                for label, masses in policy_masses.items()]))
        print(f'Checked {name}: {cell_count} rows, {len(observed)} leaves, {len(units)} distinct leaf/physics units', flush=True)
    cohorts = []; leaf_records = []; group_records = []
    for cohort, units in sorted(cohort_units.items()):
        means = leaf_means(units)
        summaries = group_summary(means, groups)
        group_records.extend(dict(source_partition=cohort, **x) for x in summaries)
        leaf_records.extend(dict(source_partition=cohort, leaf=list(k), **v) for k, v in sorted(means.items()))
        target = means.get(TARGET)
        cohorts.append(dict(source_partition=cohort, archives=source_partitions[cohort]['archives'],
            variance=variance_summary(units), groups_observed=sum(x['observed_productions'] > 0 for x in summaries),
            groups_complete=sum(x['complete'] for x in summaries),
            groups_observed_utility_heterogeneous=sum((x['observed_utility_range'] or 0) > 1e-12 for x in summaries),
            complete_groups_utility_heterogeneous=sum(x['complete'] and (x['observed_utility_range'] or 0) > 1e-12 for x in summaries),
            complete_groups_cap_heterogeneous=sum(x['complete'] and (x['observed_cap_range'] or 0) > 1e-12 for x in summaries),
            maximum_complete_uniform_production_penalty=max((x['uniform_production_penalty'] for x in summaries if x['complete']), default=None),
            positive_control_leaf=target,
            policy_bounds=[dict(policy=label, **coverage_bounds(masses, means, leaves),
                target_leaf_probability=masses[TARGET]) for label, masses in policy_masses.items()]))
    union = {key[1:-1] for key in all_units}
    union_groups = {key[:3] for key in union}
    # No pooled reward estimate across source partitions; union is coverage only.
    union_mass = {label: math.fsum(masses[k] for k in union) for label, masses in policy_masses.items()}
    rows = sum(x[3] for x in INPUTS)
    result = dict(profile=PROFILE, classification='exploratory_exposed_historical_receipts',
        scientific_acceptance=False, training_admitted=False, native_evaluations_run=0,
        native_steps_run=0, grammar_compilations_run=0, training_updates_run=0,
        independent_physics_replay=False, reward_cache_admitted=False,
        catalogue=counts, raw_rows=rows, historical_attempted_steps_checked=attempted,
        historical_advancing_steps_checked=advancing, distinct_source_leaf_physics_units=len(all_units),
        duplicate_source_leaf_physics_rows=rows-len(all_units), source_partitions=len(cohorts),
        union_coverage=dict(leaves=len(union), groups=len(union_groups), parents=len({k[0] for k in union}),
            missing_leaves=len(leaves)-len(union), observed_policy_mass=union_mass),
        archives=archives, cohorts=cohorts,
        positive_control=dict(leaf=list(TARGET), group_production_count=groups[TARGET[:3]]['count'],
            policy='positive_control_selector_pm8', finite_logits=[-8., 8.],
            native_confirmation_run=False, emergence_claim=False),
        limits=['Missing-support bounds condition on observed empirical means, not true population outcomes.',
                'Physics seed panels differ by leaf; this is not a complete fixed-panel reward table.',
                'Source partitions bind common native source paths and manifests, not whole binary equivalence.',
                'Execution signatures are not merged; credit-bearing emission identity is checked per leaf.',
                'Historical adaptive/control/development rows are descriptive, not independent held-out tests.'])
    seal_inputs()
    return result, leaf_records, group_records, source_partitions


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False)+'\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    assert out.is_relative_to(ROOT/'output') and not out.exists(), 'requires fresh output under project output/'
    out.mkdir()
    sources = ['scripts/analyze_ere_coverage_headroom.py', 'scripts/test_analyze_ere_coverage_headroom.py',
               'scripts/audit_ere_categorical_feasible.py', 'scripts/audit_ere_categorical_comparison_archive.py']
    source_hashes = {n: a.digest(ROOT/n) for n in sources}
    for name in sources:
        dest = out/'source_snapshots'/name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT/name, dest)
    start = time.time()
    manifest = dict(profile=PROFILE, status='running', classification='exploratory_offline_review',
                    inputs=[dict(name=n, root_sha256=r, seal_entries=c, cells=k) for n, r, c, k in INPUTS],
                    source_hashes=source_hashes, native_work_run=0, training_admitted=False)
    write_json(out/'manifest.json', manifest)
    try:
        tests = subprocess.run([sys.executable, '-B', '-m', 'unittest', 'discover', '-s', 'scripts',
                                '-p', 'test_analyze_ere_coverage_headroom.py', '-v'],
                               cwd=ROOT, capture_output=True, text=True)
        write_json(out/'test_receipt.json', dict(returncode=tests.returncode,
                   stdout=tests.stdout, stderr=tests.stderr, native_work_run=0))
        assert tests.returncode == 0, 'coverage review tests failed'
        result, leaves, groups, cohorts = analyze()
        assert source_hashes == {n: a.digest(ROOT/n) for n in sources}, 'analysis source drift'
        write_json(out/'analysis.json', result)
        write_json(out/'leaves.json', leaves)
        write_json(out/'groups.json', groups)
        write_json(out/'source_partitions.json', cohorts)
        manifest.update(status='complete', elapsed_seconds=time.time()-start,
                        receipt_accounting_checked=True, independent_physics_replay=False)
        write_json(out/'manifest.json', manifest)
    except Exception as error:
        manifest.update(status='failed', error=repr(error), elapsed_seconds=time.time()-start)
        write_json(out/'manifest.json', manifest)
        a.seal(out)
        raise
    a.seal(out)
    print(json.dumps(dict(output=str(out), raw_rows=result['raw_rows'],
        union_coverage=result['union_coverage'], source_partitions=result['source_partitions']), indent=2))


if __name__ == '__main__':
    main()
