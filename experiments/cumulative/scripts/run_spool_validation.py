"""Freeze bounded spool tests, then execute exactly three funded development units."""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from run_readiness import ROOT, PACKAGE, FALSE, digest, write, source_paths
from run_campaign_adapter import custody
from compact_evidence import Store, source_inventory, sha, publish, encode_json
from evidence_spool import Spool, JournalView, run_development_unit, CONTROL_RESERVE
from audit_campaign_adapter import recover_values
from audit_durability import read


def copied(spool, source):
    identity = sha(encode_json({p.name: digest(p) for p in sorted(source.iterdir())}))
    ticket = spool.reserve(identity)
    shutil.copytree(source, Path(ticket['directory']))
    return ticket['unit']


def expect(fn, phrase):
    try:
        fn()
    except (ValueError, RuntimeError, FileNotFoundError) as e:
        if phrase and phrase not in str(e):
            raise AssertionError('wrong refusal: ' + str(e)) from e
        return str(e)
    raise AssertionError('expected refusal/interruption')


def fixture(out, contract, adapter):
    reports = []
    source = adapter / 'journals/world_boundary'
    def fresh(name):
        return Spool.create(out / 'cases' / name, contract['limits']['spool_cap_bytes'])
    for fault in contract['fault_cases']:
        spool = fresh(fault); unit = copied(spool, source)
        message = expect(lambda: spool.archive(unit, fault), 'declared interruption')
        before = spool.states()[unit]['kind']
        spool = Spool(spool.root)
        spool.recover(unit)
        assert spool.states()[unit]['kind'] == 'released'
        restored = spool.restore(unit)
        assert {p.name: digest(p) for p in Path(restored['directory']).iterdir()} == {p.name: digest(p) for p in source.iterdir()}
        spool.release_restore(unit)
        reports.append({'case': fault, 'unit': unit, 'interruption': message,
                        'recovered_from': before, 'status': 'released', 'capacity': spool.capacity()})
    spool = fresh('stress_full_roundtrip'); unit = copied(spool, adapter / 'journals/stress_0')
    spool.archive(unit); restored = spool.restore(unit)
    assert {p.name: digest(p) for p in Path(restored['directory']).iterdir()} == {p.name: digest(p) for p in (adapter / 'journals/stress_0').iterdir()}
    spool.release_restore(unit)
    reports.append({'case': 'stress_full_roundtrip', 'unit': unit, 'status': 'released', 'capacity': spool.capacity()})
    spool = Spool.create(out / 'cases/cap_before_dispatch', CONTROL_RESERVE + 16384)
    message = expect(lambda: spool.reserve('0' * 64), 'storage admission cap')
    assert not spool.states()
    reports.append({'case': 'cap_before_dispatch', 'error': message, 'native_dispatched': False})
    spool = fresh('reserved_writer_no_retry'); unit = spool.reserve('0' * 64)['unit']
    message = expect(lambda: Spool(spool.root).recover(unit), 'uncertain')
    expect(lambda: Spool(spool.root).reserve('1' * 64), 'single active writer')
    reports.append({'case': 'reserved_writer_no_retry', 'error': message, 'capacity': spool.capacity()})
    for case, directory, phrase in [
        ('pending_native_no_restore', adapter / 'journals/world_unknown', 'uncertain native intent'),
        ('torn_native_no_restore', ROOT / 'output/cumulative_durable_journal_readiness_v3_20261009/journals/world_torn', 'torn journal')]:
        spool = fresh(case); unit = copied(spool, directory); spool.archive(unit)
        message = expect(lambda: spool.restore(unit), phrase)
        assert not any((spool.root / 'restore').iterdir())
        reports.append({'case': case, 'unit': unit, 'error': message, 'capacity': spool.capacity()})
    for case in ['partial_compact_no_overwrite', 'corrupt_compact_no_release', 'foreign_raw_no_release']:
        spool = fresh(case); unit = copied(spool, source)
        fault = {'partial_compact_no_overwrite': 'after_seal', 'corrupt_compact_no_release': 'after_compact', 'foreign_raw_no_release': 'after_verified'}[case]
        expect(lambda: spool.archive(unit, fault), 'declared interruption')
        if case == 'partial_compact_no_overwrite':
            directory = spool.root / 'compact' / unit; directory.mkdir()
            (directory / '.staging-index.json').write_bytes(b'{')
            phrase = ''
        elif case == 'corrupt_compact_no_release':
            next((spool.root / 'compact' / unit / 'objects').iterdir()).write_bytes(b'corrupt')
            phrase = 'stored object size/hash'
        else:
            (spool.root / 'raw' / unit / 'foreign.txt').write_text('not in the owned seal')
            phrase = 'unknown owned files'
        message = expect(lambda: Spool(spool.root).recover(unit), phrase)
        assert all(digest(spool.root / 'raw' / unit / p.name) == digest(p) for p in source.iterdir())
        reports.append({'case': case, 'unit': unit, 'error': message, 'original_raw_preserved': True, 'capacity': spool.capacity()})
    spool = fresh('symlink_raw_no_release'); unit = copied(spool, source)
    target = sorted((spool.root / 'raw' / unit).iterdir())[0]; data = target.read_bytes()
    target.unlink(); target.symlink_to(source / target.name)
    message = expect(lambda: spool.archive(unit), 'regular evidence file')
    target.unlink(); target.write_bytes(data)  # Only remove our test link; original is never written.
    reports.append({'case': 'symlink_raw_no_release', 'unit': unit, 'error': message, 'test_link_removed': True, 'capacity': spool.capacity()})
    spool = fresh('modified_restore_no_release'); unit = copied(spool, source); spool.archive(unit)
    restored = spool.restore(unit); target = sorted(Path(restored['directory']).iterdir())[0]
    target.write_bytes(target.read_bytes() + b' ')
    message = expect(lambda: spool.release_restore(unit), 'restore changed')
    reports.append({'case': 'modified_restore_no_release', 'unit': unit, 'error': message, 'capacity': spool.capacity()})
    return reports


def run(out):
    started = time.monotonic(); out.mkdir(exist_ok=False, parents=True)
    for d in ['cases', 'commands', 'native_inputs']: (out / d).mkdir()
    contract = json.loads((PACKAGE / 'spool_integration_contract.json').read_text())
    adapter = ROOT / contract['adapter_parent']['archive']
    manifest = {'profile': contract['profile'], 'status': 'running', 'engineering_spool_pass': False,
                'native_work_lower': 0, 'native_work_upper': 0, **FALSE}
    write(out / 'manifest.json', manifest)
    charged = 0; lower = 0; native_seconds = 0
    try:
        custody()
        parent = ROOT / contract['parent']['archive']
        source_inventory(parent, contract['parent']['seal_sha256'])
        source_inventory(adapter, contract['adapter_parent']['seal_sha256'])
        previous = json.loads((parent / 'source_sha256.json').read_text())
        if any(digest(ROOT / n) != h for n, h in previous.items()): raise ValueError('storage parent source modified')
        pins = {str(p.relative_to(ROOT)): digest(p) for p in source_paths()}
        write(out / 'source_sha256.json', pins)
        for rel in pins:
            dest = out / 'source_snapshots' / rel; dest.parent.mkdir(parents=True, exist_ok=True); shutil.copy2(ROOT / rel, dest)
        write(out / 'contract.json', contract)
        fixture_reports = fixture(out, contract, adapter)
        write(out / 'fault_tests.json', fixture_reports)
        print(json.dumps({'phase': 'saved_evidence_faults', 'checks': len(fixture_reports), 'native_rewrites': 0}), flush=True)
        spool = Spool.create(out / 'native_spool', contract['limits']['spool_cap_bytes'])
        binary = out / 'cumulative-campaign'; shutil.copy2(adapter / 'cumulative-campaign', binary)
        base = json.loads((adapter / 'inputs/world_endogenous.json').read_text())
        base.update(native_budget=contract['limits']['native_rewrites'], journal_byte_allowance=31*1024**2)
        base['source_identity'] = {'binary_sha256': digest(binary), 'source_sha256': digest(out / 'source_sha256.json')}
        adapter_contract = json.loads((adapter / 'contract.json').read_text()); write(out / 'adapter_contract.json', adapter_contract)
        reports = {}; restored = None
        for route in contract['native_routes']:
            name = route['name']; request = copy.deepcopy(base)
            request.update(execution_allowance=contract['limits']['native_rewrites']-charged, fault=route['fault'], source=None)
            if route.get('source'):
                restored = spool.restore(reports[route['source']]['unit'])
                request['source'] = {'directory': restored['directory'], 'head': restored['head']}
            begin = time.monotonic()
            ticket, frozen_request, child = run_development_unit(spool, request, binary,
                                            contract['limits']['native_seconds']-native_seconds)
            elapsed = time.monotonic()-begin; native_seconds += elapsed
            write(out / 'commands' / (name + '.json'), {'seconds': elapsed, **child})
            write(out / 'native_inputs' / (name + '.json'), frozen_request)
            state = spool.states()[ticket['unit']]; store = Store(spool.root / 'compact' / ticket['unit'], state['index_sha256'], state['seal_sha256'])
            values, head, staging = read(JournalView(store))
            source_result = reports.get(route.get('source'), {}).get('recovered')
            recovered = recover_values(values, head, staging, frozen_request, ticket['request_sha256'], adapter_contract, [], source_result)
            charged += recovered['here_upper']; lower += recovered['here_lower']
            if child['returncode'] != (1 if route['fault'] else 0): raise ValueError('unexpected native exit')
            reports[name] = {'unit': ticket['unit'], 'recovered': recovered, 'capacity': spool.capacity()}
            if restored:
                spool.release_restore(restored['unit']); restored = None
            print(json.dumps({'phase': name, 'native_work_here': recovered['here_upper'], 'charged': charged}), flush=True)
        if reports['complete']['recovered']['rows'] != reports['continue']['recovered']['rows'] or reports['complete']['recovered']['checkpoint'] != reports['continue']['recovered']['checkpoint']:
            raise ValueError('spooled native continuation changed results')
        write(out / 'native_routes.json', {n: {'unit': r['unit'], 'lower': r['recovered']['here_lower'], 'upper': r['recovered']['here_upper'], 'head': r['recovered']['head'], 'capacity': r['capacity']} for n,r in reports.items()})
        frozen = out / 'source_snapshots/experiments/cumulative/scripts'
        r = subprocess.run([sys.executable, '-B', str(frozen / 'audit_spool.py'), str(out)], capture_output=True, text=True,
                           timeout=max(1, contract['limits']['workflow_seconds']-(time.monotonic()-started)))
        write(out / 'commands/independent_audit.json', {'exit_code': r.returncode, 'stdout': r.stdout, 'stderr': r.stderr})
        if r.returncode: raise ValueError(r.stderr)
        audit = json.loads(r.stdout); write(out / 'audit.json', audit)
        custody(); source_inventory(parent, contract['parent']['seal_sha256']); source_inventory(adapter, contract['adapter_parent']['seal_sha256'])
        if any(digest(ROOT / n) != h for n,h in pins.items()): raise ValueError('source changed during spool fixture')
        write(out / 'preservation.json', {'all_thirteen_previous_roots_unchanged': True, 'all_previous_source_pins_unchanged': True, 'historical_raw_files_removed': 0})
        if charged>250 or native_seconds>60: raise ValueError('native funding bound')
        manifest.update(audit, status='complete', native_work_lower=lower, native_work_upper=charged,
                        native_seconds=native_seconds, elapsed_seconds=time.monotonic()-started, source_files=len(pins))
        write(out / 'manifest.json', manifest)
        if time.monotonic()-started>1200 or sum(p.stat().st_size for p in out.rglob('*') if p.is_file())>134217728:
            raise ValueError('workflow/output cap')
    except Exception as e:
        manifest.update(status='failed', error=str(e), engineering_spool_pass=False,
                        native_work_lower=lower, native_work_upper=250, native_work_unknown=True)
        write(out / 'manifest.json', manifest)
        write(out / 'output_sha256.json', {str(p.relative_to(out)): digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p!=out/'output_sha256.json'})
        raise
    write(out / 'output_sha256.json', {str(p.relative_to(out)): digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    print(json.dumps({'output': str(out), 'seal_sha256': digest(out/'output_sha256.json'), **manifest}, indent=2))


if __name__ == '__main__': run(Path(sys.argv[1]).resolve())
