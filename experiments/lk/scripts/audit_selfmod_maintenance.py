#!/usr/bin/env python3
"""Independent audit of all uninterrupted longitudinal V22 extension receipts."""
import argparse
import copy
import hashlib
import json
import pathlib
import tarfile
from audit_selfmod_birth import load, rows, sha, require, graph_stocks
from audit_selfmod_bounded import BoundedState
from audit_selfmod_financing import validate_allocator
from selfmod_maintenance_common import descendants, reconstruct_extension, maintenance
from run_selfmod_maintenance import CONTRACT, SEEDS, ARMS


def audit(out):
    m = load(out / 'manifest.json')
    pre = m['engineering_preflight']
    seeds = CONTRACT['engineering_seeds'] if pre else SEEDS
    steps = CONTRACT['engineering_steps'] if pre else CONTRACT['extension_steps']
    require(m == dict(protocol=CONTRACT['protocol'], engineering_preflight=pre, contract=CONTRACT,
        seeds=seeds, arms=ARMS, offset=10000, extension_steps=steps,
        requested_trajectories=len(seeds)*2, max_parallel_processes=4,
        independent_replication=False), 'Fixed longitudinal manifest')
    require(load(out / 'contract.json') == CONTRACT, 'Frozen contract')
    prov = load(out / 'instrument_provenance.json')
    for f, h in prov['artifacts_sha256'].items():
        require(sha(out / f) == h, 'Frozen artifact '+f)
    with tarfile.open(out / 'source.tar.gz') as t:
        require(set(t.getnames()) == set(prov['sources_sha256']), 'Source inventory')
        for f, h in prov['sources_sha256'].items():
            require(hashlib.sha256(t.extractfile(f).read()).hexdigest() == h, 'Frozen source '+f)
        for source, dest in [('docs/selfmod_v22_maintenance_contract.json', 'contract.json'),
                             ('docs/selfmod_v22_maintenance_preregistration.md', 'preregistration.md')]:
            require(t.extractfile(source).read() == (out / dest).read_bytes(), 'Frozen preregistration')
    ip = load(out / 'input_provenance.json')
    require(set(ip['inputs_sha256']) == {str(f.relative_to(out)) for f in (out / 'inputs').rglob('*') if f.is_file()}, 'Exact input inventory')
    for f, h in ip['inputs_sha256'].items():
        require(sha(out / f) == h, 'Authenticated input '+f)
    prior = load(out / 'inputs/audit.json')
    done = load(out / 'inputs/completion_verification.json')
    native = load(out / 'inputs/instrument_provenance.json')
    require(prior['audit_passed'] and done['passed'] and load(out / 'inputs/status.json')['state'] == 'complete' and load(out / 'inputs/replay_verification.json')['passed'], 'Validated V21')
    for f in ['audit.json', 'manifest.json', 'summary.json', 'status.json', 'instrument_provenance.json', 'replay_verification.json', 'contract.json']:
        require(sha(out / 'inputs' / f) == done['artifacts_sha256'][f], 'Prior completion anchor '+f)
    require(done['third_family_durable'] == dict(faithful=8, edited=8) and load(out / 'inputs/manifest.json')['seeds'] == SEEDS, 'All original worlds retained')
    require(sha(out / 'selfmod-maintenance') == native['artifacts_sha256']['selfmod-reversible-edit'] and sha(out / 'native_source.tar.gz') == native['artifacts_sha256']['source.tar.gz'], 'Unchanged native binary/source')
    if (out / 'inputs/failed_engineering_raw_inventory.json').exists():
        failed = load(out / 'inputs/failed_engineering_audit_validation_failure.json')
        require(failed['audit_passed'] is False and failed['science_not_launched'], 'Failed engineering not promoted')
        inventory = load(out / 'inputs/failed_engineering_raw_inventory.json')
        for f, h in inventory['raw_sha256'].items():
            require(sha(pathlib.Path(inventory['directory']) / f) == h, 'Preserved failed-engineering raw')
    for f in ['scripts/audit_selfmod_durability.py', 'scripts/audit_selfmod_bounded.py', 'scripts/audit_selfmod_financing.py']:
        require(prov['sources_sha256'][f] == native['sources_sha256'][f], 'Unchanged native audit/criterion '+f)
    if not pre:
        for f in ['audit.json', 'engineering_verification.json', 'source_archive_verification.json', 'missing_case_verification.json', 'test_verification.json']:
            d = load(out / 'inputs' / ('engineering_'+f))
            require(d.get('passed', d.get('audit_passed', False)), 'Engineering gate '+f)
    records = []
    prior_records = {(r['seed'], r['arm']): r for r in prior['records']}
    for seed in seeds:
        for arm in ARMS:
            src = out / f'inputs/seed_{seed}/{arm}'
            dest = out / f'seed_{seed}/{arm}'
            for f in ['final.json', 'final_allocator.json']:
                name = f'seed_{seed}/{arm}/{f}'
                require(sha(src / f) == prior['evidence_sha256'][name] == done['artifacts_sha256'][name], 'Actual audited endpoint '+name)
            birth_name = f'seed_{seed}/third_prefix/{arm}_birth.json'
            require(sha(src / 'birth.json') == prior['evidence_sha256'][birth_name] == done['artifacts_sha256'][birth_name], 'Original paid third founder')
            b = load(src / 'birth.json')
            selected = b['selection']
            old = prior_records[(seed, arm)]
            require(selected['child'] == old['founder'] and old['third_family']['durability']['durable_success'], 'Immutable qualified third founder')
            initial = load(dest / 'initial.json')
            for suffix in ['', '_allocator']:
                require(sha(dest / f'initial{suffix}.json') == sha(src / f'final{suffix}.json'), 'Exact restore with no resets')
            require(initial['cfg']['steps'] == 10000 and initial['cfg'] == load(src / 'final.json')['cfg'], 'Unaltered physical configuration')
            genes = {g['id']: g for g in initial['genes']}
            gene = genes[selected['child']]
            expected = CONTRACT['genotypes'][arm]
            require(gene['class_hash'] == expected['class_hash'] and gene['spec']['var_labels'] == expected['var_labels'] and gene['member'] == 0 and gene['lineage'] and gene['executable'], 'Exact original third genotype')
            ids = descendants(genes, selected['child'])
            require(any(id_ in ids for id_ in graph_stocks(initial)), 'Original family present at extension entry')
            warmup = dict(focal=old['third_family']['windows'][:2], original=old['original_family']['windows'][:2])
            result = reconstruct_extension(dest, selected, steps, selected['child'], warmup)
            final = load(dest / 'final.json')
            require(final['world']['step'] == initial['world']['step']+steps, 'Uninterrupted absolute clock')
            if pre:
                d = dest.parent
                require((dest / 'rows.jsonl').read_bytes() == (d / (arm+'_split_a/rows.jsonl')).read_bytes() + (d / (arm+'_split_b/rows.jsonl')).read_bytes(), 'Exact split rows')
                for suffix in ['', '_allocator']:
                    require(sha(dest / f'checkpoint_11000{suffix}.json') == sha(d / (arm+f'_split_a/final{suffix}.json')), 'Exact split checkpoint')
                    require(sha(dest / f'final{suffix}.json') == sha(d / (arm+f'_split_b/final{suffix}.json')), 'Exact split endpoint')
            records.append(dict(seed=seed, arm=arm, founder=selected['child'], focal=result['focal'], original=result['original']))
            print(f'audited {seed} {arm}', flush=True)
    expected_summary = dict(trajectories_completed=len(records), extension_steps=len(records)*steps,
        scientific_advancing_steps=0 if pre else len(records)*steps,
        engineering_advancing_steps=len(records)*steps*2 if pre else 0,
        fixed_seed_denominator=len(seeds), prior_steps_recounted=0)
    require(load(out / 'summary.json') == expected_summary and load(out / 'status.json') == dict(state='complete', trajectories_completed=len(records)), 'Complete bounded accounting')
    aggregates = {}
    for arm in ARMS:
        g = [r for r in records if r['arm'] == arm]
        n = sum(r['focal']['maintenance']['maintained_success'] for r in g)
        aggregates[arm] = dict(maintained_families=n, qualified=None if pre else n >= 6,
            completed=len(g), missing=len(seeds)-len(g), fixed_seed_denominator=len(seeds),
            family_alive=sum(r['focal']['final_family_copies'] > 0 for r in g),
            culture_alive=sum(all(r['focal']['final_member_copies']) for r in g),
            enumeration_censored=sum(any(w['enumeration_censored'] for w in r['focal']['windows']) for r in g),
            original_family_maintained=sum(r['original']['maintenance']['maintained_success'] for r in g))
    return dict(audit_passed=True, engineering_preflight=pre, counts=expected_summary,
        independent_replication=False, historical_verdicts_changed=False, aggregates=aggregates,
        records=records, auditor_sha256=sha(pathlib.Path(__file__)),
        evidence_sha256={str(f.relative_to(out)): sha(f) for f in sorted(out.rglob('*')) if f.is_file() and f.name not in ['audit.json', 'replay_verification.json']})


def validate_claims(out, report):
    """Reject false archived result claims without replaying native simulations."""
    m = load(out / 'manifest.json')
    prior = {(r['seed'], r['arm']): r for r in load(out / 'inputs/audit.json')['records']}
    require([(r['seed'], r['arm']) for r in report['records']] ==
            [(s, a) for s in m['seeds'] for a in ARMS], 'Exact result coverage')
    require(report['counts']['fixed_seed_denominator'] == len(m['seeds']), 'Result denominator')
    for r in report['records']:
        selected = load(out / f"inputs/seed_{r['seed']}/{r['arm']}/birth.json")['selection']
        require(r['founder'] == selected['child'], 'Claimed immutable founder')
        old = prior[(r['seed'], r['arm'])]
        for key, old_key in [('focal', 'third_family'), ('original', 'original_family')]:
            expected = maintenance(selected, old[old_key]['windows'][:2], r[key]['windows'], True)
            require(r[key]['maintenance'] == expected, 'Native-window maintenance claim')
    for arm in ARMS:
        n = sum(r['focal']['maintenance']['maintained_success'] for r in report['records'] if r['arm'] == arm)
        g = report['aggregates'][arm]
        require(g['maintained_families'] == n and g['fixed_seed_denominator'] == len(m['seeds']) and
                g['qualified'] == (None if m['engineering_preflight'] else n >= 6), 'Fixed aggregate claims')


def adversarial(out, report):
    d = out / f"seed_{report['records'][0]['seed']}/faithful"
    initial = load(d / 'initial.json')
    first = next(rows(d / 'rows.jsonl'))
    rejected = []
    for name, edit in [
        ('rent', lambda r: r['roots'][0].__setitem__('cost', 999)),
        ('rng', lambda r: r.__setitem__('rng_word_pos', r['rng_word_pos']+2)),
        ('cap', lambda r: r['bounded'].__setitem__('limit', 9)),
        ('lineage', lambda r: r['roots'][0].__setitem__('lineage', not r['roots'][0]['lineage'])),
        ('absolute_clock', lambda r: r.__setitem__('absolute_step', r['absolute_step']+1))]:
        bad = copy.deepcopy(first)
        edit(bad)
        try:
            BoundedState(initial, 'replace8').step_row(bad)
        except (ValueError, KeyError):
            rejected.append(name)
        else:
            raise ValueError('Accepted corruption '+name)
    state = BoundedState(initial, 'replace8')
    for r in rows(d / 'rows.jsonl'):
        if r['births']:
            for name, edit in [
                ('payment', lambda x: x['births'][0].__setitem__('child_tokens', 999)),
                ('retirement', lambda x: x['bounded']['admissions'][0].__setitem__('parent', -1)),
                ('future_genotype', lambda x: x['births'][0]['gene'].__setitem__('class_hash', 0))]:
                bad = copy.deepcopy(r)
                edit(bad)
                try:
                    copy.deepcopy(state).step_row(bad)
                except (ValueError, KeyError):
                    rejected.append(name)
                else:
                    raise ValueError('Accepted corruption '+name)
            break
        state.step_row(r)
    return rejected


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('output', type=pathlib.Path)
    out = p.parse_args().output.resolve()
    report = audit(out)
    report['adversarial_corruptions_rejected'] = adversarial(out, report)
    validate_claims(out, report)
    with (out / 'audit.json').open('x') as f:
        json.dump(report, f, indent=2)
        f.write('\n')
    print(json.dumps({k: report[k] for k in ['audit_passed', 'counts', 'aggregates']}, indent=2))


if __name__ == '__main__':
    main()
