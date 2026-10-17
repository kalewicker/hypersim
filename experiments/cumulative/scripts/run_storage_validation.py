"""Freeze and validate compact evidence using immutable saved archives only."""
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time
import zlib
from run_readiness import ROOT, PACKAGE, FALSE, digest, write, source_paths
from run_campaign_adapter import custody as prior_custody
from compact_evidence import source_inventory
from storage_projection import project


def run(out):
    started = time.monotonic()
    out.mkdir(parents=True, exist_ok=False)
    for directory in ('commands', 'stores'):
        (out / directory).mkdir()
    contract = json.loads((PACKAGE / 'storage_validation_contract.json').read_text())
    limits = contract['limits']
    manifest = {'profile': contract['profile'], 'status': 'running', 'engineering_storage_pass': False,
                'new_native_rewrites': 0, 'whole_campaign_feasibility_pass': False, **FALSE}
    write(out / 'manifest.json', manifest)

    def command(name, args):
        remaining = limits['workflow_seconds'] - (time.monotonic() - started)
        if remaining <= 0:
            raise ValueError('storage workflow deadline')
        begin = time.monotonic()
        try:
            r = subprocess.run([sys.executable, '-B', *args], cwd=ROOT, capture_output=True, text=True,
                               timeout=remaining, env={**os.environ, 'PYTHONDONTWRITEBYTECODE': '1'})
        except subprocess.TimeoutExpired as e:
            write(out / 'commands' / (name + '.json'), {'timed_out': True, 'stdout': str(e.stdout or ''),
                                                        'stderr': str(e.stderr or '')})
            raise
        receipt = {'python_only_argv': args, 'seconds': time.monotonic() - begin,
                   'exit_code': r.returncode, 'stdout': r.stdout, 'stderr': r.stderr}
        write(out / 'commands' / (name + '.json'), receipt)
        print(json.dumps({'phase': name, 'exit_code': r.returncode}), flush=True)
        if r.returncode:
            raise ValueError(name + ': ' + r.stderr[-2000:])
        return json.loads(r.stdout), receipt

    try:
        prior_custody()
        adapter = ROOT / contract['corpus'][0]['archive']
        parent_pins = json.loads((adapter / 'source_sha256.json').read_text())
        if any(digest(ROOT / rel) != value for rel, value in parent_pins.items()):
            raise ValueError('adapter source changed; storage work must be additive')
        for entry in contract['corpus']:
            source_inventory(ROOT / entry['archive'], entry['seal_sha256'])
        pins = {str(p.relative_to(ROOT)): digest(p) for p in source_paths()}
        write(out / 'source_sha256.json', pins)
        for rel in pins:
            destination = out / 'source_snapshots' / rel
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / rel, destination)
        write(out / 'contract.json', contract)
        write(out / 'runtime.json', {'python': sys.version, 'platform': platform.platform(),
                                     'zlib_compile': zlib.ZLIB_VERSION, 'zlib_runtime': zlib.ZLIB_RUNTIME_VERSION})
        frozen = out / 'source_snapshots/experiments/cumulative/scripts'
        tests, _ = command('codec_corruption_tests', [str(frozen / 'test_compact_evidence.py')])
        indexes, audits, packages, packs = {}, {}, {}, {}
        for entry in contract['corpus']:
            name = entry['name']
            result, receipt = command('pack_' + name, [str(frozen / 'compact_evidence.py'),
                                    str(ROOT / entry['archive']), str(out / 'stores' / name), entry['seal_sha256']])
            packs[name] = {'result': result, 'seconds': receipt['seconds']}
            index_root = result['compact_index_sha256']
            report, _ = command('audit_' + name, [str(frozen / 'audit_compact_evidence.py'),
                                str(out / 'stores' / name), index_root, entry['seal_sha256'], entry['audit_module']])
            audits[name] = report
            packages[name] = {'compact_index_sha256': index_root, 'source_seal_sha256': entry['seal_sha256'],
                              'audit_module': entry['audit_module']}
            indexes[name] = json.loads((out / 'stores' / name / 'index.json').read_text())
        write(out / 'compact_packages.json', packages)
        write(out / 'byte_and_semantic_audits.json', audits)
        write(out / 'pack_metrics.json', packs)
        projection = project(contract, indexes, audits, packs)
        write(out / 'projection.json', projection)
        # Source archives are still byte-identical. No original raw evidence was removed.
        prior_custody()
        for entry in contract['corpus']:
            source_inventory(ROOT / entry['archive'], entry['seal_sha256'])
        if any(digest(ROOT / rel) != value for rel, value in pins.items()):
            raise ValueError('source changed during storage validation')
        write(out / 'preservation.json', {'all_twelve_prior_roots_unchanged': True,
                  'all_previous_source_pins_unchanged': True, 'source_archives_removed_or_rewritten': 0,
                  'native_commands_launched': 0, 'all_native_and_scientific_work': 0})
        manifest.update(status='complete', engineering_storage_pass=True,
                        observed_storage_envelope_pass=projection['observed_storage_envelope_pass'],
                        observed_workflow_projection_fits=projection['observed_workflow_projection_fits'],
                        source_files=len(pins), corpus_archives=len(packages),
                        corpus_files=sum(x['files'] for x in audits.values()),
                        corruption_checks=tests['corruptions_and_publication_failures_rejected'],
                        original_raw_bytes=sum(x['result']['source_raw_bytes'] for x in packs.values()),
                        compact_payload_bytes=sum(x['result']['compressed_payload_bytes'] for x in packs.values()),
                        observed_peak_storage_projection_bytes=projection['observed_peak_storage_projection_bytes'],
                        production_spool_integration_complete=False, elapsed_seconds=time.monotonic() - started)
        write(out / 'manifest.json', manifest)
        size = sum(p.stat().st_size for p in out.rglob('*') if p.is_file())
        if size > limits['output_bytes']:
            raise ValueError('storage validation output cap')
    except Exception as e:
        manifest.update(status='failed', engineering_storage_pass=False, error=str(e),
                        elapsed_seconds=time.monotonic() - started)
        write(out / 'manifest.json', manifest)
        write(out / 'output_sha256.json', {str(p.relative_to(out)): digest(p) for p in sorted(out.rglob('*'))
                                          if p.is_file() and p != out / 'output_sha256.json'})
        raise
    write(out / 'output_sha256.json', {str(p.relative_to(out)): digest(p) for p in sorted(out.rglob('*')) if p.is_file()})
    print(json.dumps({'output': str(out), 'seal_sha256': digest(out / 'output_sha256.json'),
                      'payload_bytes': size, **manifest}, indent=2))


if __name__ == '__main__':
    run(Path(sys.argv[1]).resolve())
