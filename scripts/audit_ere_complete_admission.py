#!/usr/bin/env python3
"""Frozen accounting/provenance audit, not independent physics replay."""
import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import shutil
import sys
import tempfile

sys.dont_write_bytecode = True
PROFILE = 'E2-complete-receipt-bound-runner-v1'
CATALOGUE = 'ca02b55e5939ead818cdfce321c74b2f13c1ac7bde3c500061ca3ff3c402c8a6'
SPECS = {
    'deterministic': ('SHA256SUMS', 'b992ca9ab6df4b642b40320377d87a20f331a5bb8cf0b29dcc68262c8c82c5f3', 114),
    'long_gates': ('output_sha256.json', '32bc8eb713b67d71569a94e8dc6ed6000784c3c89afd00ad67a583c6b9edf5c5', 245),
    'closure': ('output_sha256.json', 'fa3725f7cf5a03b207635d4c121d47280d38b4a05ae018ec3606e00086a4b60a', 364),
    'grammar': ('output_sha256.json', 'ccc22798899e426f93cd2801020fef0e93391e8790a23a214d22b625caa58891', 438),
    'runtime': ('output_sha256.json', '0099ecae4b7487e9ac975f73d813e7cbde0de430470dc66198bbafd7df16488c', 460),
}
EXTRA = {
    'crates/hg-train/src/ere.rs', 'crates/hg-train/src/ere/complete_admission.rs',
    'crates/hg-train/src/ere/complete_runner.rs', 'crates/hg-train/src/bin/ere-train.rs',
    'docs/ere_complete_admission_runner_v1.md', 'configs/ere_complete_receipts_v1.json',
    'configs/ere_complete_bounded_request_v1.json', 'scripts/audit_ere_complete_admission.py',
    'scripts/test_audit_ere_complete_admission.py',
}

def load(p):
    def pairs(rows):
        d = {}
        for k, v in rows:
            assert k not in d, 'duplicate JSON key'
            d[k] = v
        return d
    return json.loads(Path(p).read_text(), object_pairs_hook=pairs)

def digest(p):
    h = hashlib.sha256()
    with Path(p).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()

def safe(s):
    return isinstance(s, str) and bool(s) and not any(x in s for x in ('\\', '\n', '\r')) and not s.startswith('/') and all(x not in ('', '.', '..') for x in s.split('/'))

def files(root):
    result = set()
    assert not root.is_symlink()
    for p in root.rglob('*'):
        assert not p.is_symlink(), 'symlink output/evidence'
        if p.is_dir():
            continue
        assert p.is_file(), 'nonregular output/evidence'
        name = p.relative_to(root).as_posix()
        assert safe(name)
        result.add(name)
    return result

def verify(root, seal='output_sha256.json', expected=None, count=None):
    root = Path(root)
    if expected:
        assert digest(root / seal) == expected, 'registered root seal'
    if seal == 'SHA256SUMS':
        entries = {}
        for line in (root / seal).read_text().splitlines():
            h, name = line.split('  ', 1)
            assert name not in entries
            entries[name] = h
    else:
        entries = load(root / seal)
    assert isinstance(entries, dict)
    assert set(entries) == files(root) - {seal}, 'exact seal coverage'
    if count is not None:
        assert len(entries) == count
    for name, h in entries.items():
        assert safe(name) and len(h) == 64 and all(x in '0123456789abcdef' for x in h)
        assert digest(root / name) == h, name
    return len(entries)

def seal(root):
    values = {p: digest(root / p) for p in sorted(files(root) - {'output_sha256.json'})}
    (root / 'output_sha256.json').write_text(json.dumps(values, indent=2))

def request(q):
    exact = dict(profile=PROFILE, purpose='bounded_engineering', protocol='E2-witness-complete-v1',
        grammar_profile='E2-complete-valid-production-grammar-v1', matcher_work_model='repeated-self-pair-feasibility-v1',
        allocation_mode='metadata-isolated-v1', ordinary_world='signal-fixed-world-v1', step_mode='MaxParallel',
        max_pairings=1, match_budget_policy='Pooled', work=50000, results=50000, horizon=100,
        evaluations=1, optimizer_generations=0, held_out_evaluations=0, wall_seconds=900,
        output_cap_bytes=512*1024*1024)
    assert set(q) == set(exact) | {'parameters', 'emission_seed', 'physics_seed'}
    for k, v in exact.items():
        assert type(q[k]) is type(v) and q[k] == v, k
    assert len(q['parameters']) == 183 and all(type(x) in (int, float) and math.isfinite(x) and abs(x) <= 3.4028234663852886e38 for x in q['parameters'])
    for k in ('emission_seed', 'physics_seed'):
        assert type(q[k]) is int and 0 <= q[k] <= 2**64-1

def pinned(name):
    return name.startswith(tuple('crates/'+x+'/' for x in ('hg-core', 'hg-engine', 'hg-meta', 'hg-rules', 'hg-observe', 'hg-sim-core'))) or name in ('Cargo.toml', 'Cargo.lock', 'crates/hg-train/Cargo.toml', 'crates/hg-train/src/optim.rs')

def helper(runtime):
    path = runtime / 'source_snapshots/scripts/audit_ere_runtime50k_signal.py'
    spec = importlib.util.spec_from_file_location('sealed_runtime_accounting', path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod

def evaluation(row, catalogue, q, h):
    assert row['engineering_only'] is True
    e, v = row['emission'], row['evaluation']
    h.check_membership(catalogue, e, full=True)
    assert e['kernel']['training_admitted'] is False and e['kernel']['advancing_engine_steps'] == 0
    assert e['status'] == e['kernel']['status'] and e['admitted'] is True
    assert not e['kernel']['codec_reencoded'] and not e['kernel']['metadata_recounted']
    assert v['physics_seed'] == q['physics_seed'] and e['emission_seed'] == q['emission_seed']
    assert v['runtime_match_budget'] == dict(policy='Pooled', work=50000, results=50000)
    c = e['classes']
    ids = [i for i, b in enumerate(v['candidate_execution']) if c['active'] and c['core'] != c['proposal'] and b == c['proposal_execution']]
    assert ids == v['credited_rule_ids']
    steps = v['steps']
    assert len(steps) <= 100 and v['attempted_steps'] == len(steps)
    assert v['advancing_steps'] == sum(s['advanced'] for s in steps)
    fires, failed = 0, False
    for i, s in enumerate(steps, 1):
        assert s['step'] == i and len(s['rule_ids']) <= 1
        assert all(type(n) is int and 0 <= n < len(v['candidate_execution']) for n in s['rule_ids'])
        h.check_allowances(s['usage'], 50000, 50000)
        fires += sum(n in ids for n in s['rule_ids'])
        if s['cap'] or not s['valid_family'] or s['advanced'] != 1:
            assert i == len(steps)
            failed = True
    assert fires == v['proposal_fires']
    full = len(steps) == 100 and not failed
    assert (v['score'] is not None) == full and (v['reason'] is None) == full
    h.close(v['score'], math.log1p(max(fires-1, 0))/math.log(100) if full else None)
    return len(steps), v['advancing_steps'], full

def audit(out, integrity=True):
    out = Path(out)
    if integrity:
        verify(out)
    m, s = load(out/'manifest.json'), load(out/'status.json')
    for value in (m, s):
        assert value['profile'] == PROFILE and value['training_admitted'] is False
        assert value['full_training_authorized'] is False and value['engineering_only'] is True
    assert s['status'] == m['status'] and s['counts'] == m['counts'] and s['reason'] == m['reason']
    counts = m['counts']
    assert set(counts) == {'grammar_compilations', 'optimizer_evaluation_steps', 'precheck_evaluation_steps', 'held_out_evaluation_steps', 'bounded_evaluation_steps', 'bounded_attempted_steps'}
    assert all(counts[k] == 0 for k in ('optimizer_evaluation_steps', 'precheck_evaluation_steps', 'held_out_evaluation_steps'))
    if m['status'] == 'blocked_before_evaluation':
        assert all(x == 0 for x in counts.values()) and isinstance(m['reason'], str) and m['reason']
        assert not (out/'evaluation.json').exists() and not (out/'grammar_catalogue.json').exists()
        return dict(audit_pass=True, classification='sealed_zero_work_refusal', training_admitted=False)
    request(m['request'])
    a = load(out/'admission.json')
    assert a['profile'] == PROFILE and a['bounded_execution_admitted'] is True
    assert a['historical_gate_evidence_verified'] is True and a['historical_scopes_preserved'] is True
    assert a['training_admitted'] is False and a['full_training_authorized'] is False
    assert a['admitted_horizon'] == 100 and a['admitted_evaluations'] == 1
    assert a['implementation_sources_verified'] == 230
    assert a['runtime_match_budget'] == dict(policy='Pooled', work=50000, results=50000)
    assert a['prior_original_frozen_checker_pass'] is False and a['prior_corrected_accounting_pass'] is True
    assert set(a['receipts']) == set(SPECS) == set(m['receipt_paths'])
    for name, (rootseal, sha, count) in SPECS.items():
        path = Path(m['receipt_paths'][name])
        proof = a['receipts'][name]
        assert Path(proof['archive']) == path and proof['seal_sha256'] == sha
        assert proof['seal_file'] == rootseal and proof['sealed_files_verified'] == count and proof['exact_file_coverage'] is True
        verify(path, rootseal, sha, count)
    runtime = Path(m['receipt_paths']['runtime'])
    rm = load(runtime/'manifest.json')
    pins = {k: v for k, v in rm['snapshot']['sources'].items() if pinned(k)}
    assert len(pins) == 230
    snap = m['snapshot']
    assert snap['frozen_before_evaluation'] is True and set(snap['sources']) == set(pins) | EXTRA
    assert len(snap['sources']) == 239 and digest(out/'executed_binary') == snap['binary_sha256']
    for name, sha in snap['sources'].items():
        assert safe(name) and digest(out/'source_snapshots'/name) == sha
        if name in pins:
            assert sha == pins[name]
    if m['run_mode'] == 'preflight':
        assert m['status'] == 'ready_for_bounded_execution' and all(x == 0 for x in counts.values())
        assert not (out/'evaluation.json').exists() and not (out/'grammar_catalogue.json').exists()
        return dict(audit_pass=True, classification='verified_zero_work_preflight', implementation_sources=230, frozen_sources=239, training_admitted=False)
    assert m['run_mode'] == 'bounded_engineering' and m['status'] in ('complete', 'complete_with_null')
    assert counts['grammar_compilations'] == 1 and m['execution_boundary_rechecked'] is True
    assert 0 <= m['grammar_compile_seconds'] <= m['elapsed_seconds'] < 900
    assert digest(out/'grammar_catalogue.compact.json') == CATALOGUE
    catalogue = load(out/'grammar_catalogue.json')
    assert catalogue == load(out/'grammar_catalogue.compact.json')
    attempts, advances, full = evaluation(load(out/'evaluation.json'), catalogue, m['request'], helper(runtime))
    assert counts['bounded_attempted_steps'] == attempts and counts['bounded_evaluation_steps'] == advances
    assert m['bounded_evaluation_complete'] == full and (m['status'] == 'complete') == full
    assert sum((out/p).stat().st_size for p in files(out)) < 512*1024*1024
    return dict(audit_pass=True, classification='bounded_engineering_accounting', implementation_sources=230, frozen_sources=239,
                attempted_steps=attempts, advancing_steps=advances, complete=full, training_admitted=False, independent_physics_replay=False)

def tamper_tests(out):
    out = Path(out)
    # All mutations are on copies, including sources/binary; originals are never modified.
    cases = ['horizon', 'work', 'matcher', 'parameters', 'unknown', 'training', 'counts', 'source', 'binary', 'coverage', 'readiness', 'receipt']
    if (out/'evaluation.json').exists():
        cases += ['score', 'fires', 'seed', 'credit', 'step', 'allowance', 'membership', 'catalogue', 'evaluation_budget', 'execution_boundary']
    for case in cases:
        with tempfile.TemporaryDirectory(prefix='ere-admission-audit-') as tmp:
            dst = Path(tmp)/'receipt'
            shutil.copytree(out, dst)
            m = load(dst/'manifest.json')
            if case == 'horizon': m['request']['horizon'] = 10000
            elif case == 'work': m['request']['work'] = 400000
            elif case == 'matcher': m['request']['matcher_work_model'] = 'legacy'
            elif case == 'parameters': m['request']['parameters'].pop()
            elif case == 'unknown': m['request']['admit_training'] = True
            elif case == 'training': m['training_admitted'] = True
            elif case == 'counts': m['counts']['optimizer_evaluation_steps'] = 1
            elif case == 'source': (dst/'source_snapshots/crates/hg-train/src/optim.rs').write_text('stale')
            elif case == 'binary': (dst/'executed_binary').write_bytes(b'stale')
            elif case == 'coverage': (dst/'extra.txt').write_text('unsealed')
            elif case == 'readiness':
                a = load(dst/'admission.json'); a['admitted_horizon'] = 10000
                (dst/'admission.json').write_text(json.dumps(a))
            elif case == 'receipt': m['receipt_paths']['runtime'] = m['receipt_paths']['closure']
            elif case == 'catalogue': (dst/'grammar_catalogue.compact.json').write_text('{}')
            elif case == 'execution_boundary': m['execution_boundary_rechecked'] = False
            else:
                row = load(dst/'evaluation.json'); e, v = row['emission'], row['evaluation']
                if case == 'score': v['score'] = .1234
                elif case == 'fires': v['proposal_fires'] += 1
                elif case == 'seed': v['physics_seed'] += 1
                elif case == 'credit': v['credited_rule_ids'] = [-1]
                elif case == 'step': v['steps'][0]['step'] = 2
                elif case == 'allowance': v['steps'][0]['usage']['rules'][0]['work_allowance'] += 1
                elif case == 'membership': e['production_count'] += 1
                elif case == 'evaluation_budget': v['runtime_match_budget']['work'] = 400000
                (dst/'evaluation.json').write_text(json.dumps(row))
            (dst/'manifest.json').write_text(json.dumps(m))
            if case != 'coverage':
                seal(dst)  # Rehashed lies must also fail semantics/provenance.
            try:
                audit(dst)
            except (AssertionError, ValueError, KeyError, OSError, TypeError):
                continue
            raise AssertionError('tamper accepted: '+case)
    return len(cases)

def main():
    assert __debug__, 'run without -O; assertions are required'
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('output', type=Path)
    p.add_argument('--tamper-tests', action='store_true')
    p.add_argument('--seal', action='store_true')
    args = p.parse_args()
    result = audit(args.output)
    result['tamper_rejections'] = tamper_tests(args.output) if args.tamper_tests else 0
    if args.seal:
        (args.output/'audit.json').write_text(json.dumps(result, indent=2))
        seal(args.output)
        audit(args.output)
    print(json.dumps(result))

if __name__ == '__main__':
    main()
