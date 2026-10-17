"""Independent gzip-byte audit plus unchanged frozen semantic auditors.

Only authenticated Python source files are materialized. Evidence and native
binaries remain compressed, and no native command is launched.
"""
import gzip
import hashlib
import importlib
import io
import json
from pathlib import Path
import sys
import tempfile
import time
from compact_evidence import Store


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def check(condition, message):
    if not condition:
        raise ValueError(message)


def independently_decode(directory, expected_index, expected_source):
    start = time.monotonic()
    raw = (directory / 'index.json').read_bytes()
    check(sha(raw) == expected_index, 'independent index root')
    index = json.loads(raw)
    check(index['source_seal_sha256'] == expected_source, 'independent source root')
    digests = {}
    total = 0
    seal = None
    for entry in index['entries']:
        name = entry['path']
        check(name not in digests, 'independent unique original file')
        encoded = (directory / entry['object']).read_bytes()
        check(len(encoded) == entry['compressed_bytes'] and
              sha(encoded) == entry['compressed_sha256'], 'independent compressed hash')
        check(0 <= entry['raw_bytes'] <= 33554432, 'independent decode cap')
        with gzip.GzipFile(fileobj=io.BytesIO(encoded), mode='rb') as decoder:
            decoded = decoder.read(entry['raw_bytes'] + 1)
        check(len(decoded) == entry['raw_bytes'] and sha(decoded) == entry['raw_sha256'],
              'independent gzip exact bytes')
        digests[name] = sha(decoded)
        total += len(decoded)
        if name == 'output_sha256.json':
            check(sha(decoded) == expected_source, 'original seal authenticated after decode')
            seal = json.loads(decoded)
    check(seal is not None and digests == dict(seal, **{'output_sha256.json': expected_source}),
          'independent complete original seal denominator')
    return {'files': len(digests), 'raw_bytes': total, 'seconds': time.monotonic() - start}


def audit(directory, expected_index, expected_source, module_name):
    started = time.monotonic()
    store = Store(directory, expected_index, expected_source)
    byte_audit = independently_decode(directory, expected_index, expected_source)
    store.verify_all()
    # Load precisely the audited archive's original auditor sources, not live code.
    prefix = 'source_snapshots/experiments/cumulative/scripts/'
    with tempfile.TemporaryDirectory(prefix='compact-frozen-auditors-') as scratch:
        scratch = Path(scratch).resolve()
        script_bytes = 0
        for name in store.entries:
            if name.startswith(prefix) and name.endswith('.py'):
                relative = name[len(prefix):]
                target = scratch / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                raw = store.read(name)
                target.write_bytes(raw)
                script_bytes += len(raw)
        check(script_bytes <= 4 * 1024 * 1024, 'bounded Python-only materialization')
        sys.path.insert(0, str(scratch))
        previous = {name: m for name, m in sys.modules.items()
                    if name.startswith(('audit_', 'plan_', 'run_', 'preservation_'))}
        for name in previous:
            sys.modules.pop(name, None)
        try:
            begin = time.monotonic()
            module = importlib.import_module(module_name)
            result = module.audit(store.root, sealed=True) if module_name == 'audit_campaign_adapter' else module.audit(store.root)
            semantic_seconds = time.monotonic() - begin
            original = json.loads(store.read('audit.json'))
            check(result == original, 'unchanged complete frozen semantic audit result')
        finally:
            sys.path.pop(0)
            for name in list(sys.modules):
                m = sys.modules[name]
                location = getattr(m, '__file__', '') or ''
                if str(scratch) in location:
                    sys.modules.pop(name, None)
            sys.modules.update(previous)
    torn_files = [n for n in store.entries if '/.staging-' in n]
    journals = [n for n in store.entries if n.startswith('journals/')]
    return {'passed': True, 'source_seal_sha256': expected_source,
            'compact_index_sha256': expected_index, 'files': len(store.entries),
            'journal_files': len(journals), 'torn_staging_files_retained': torn_files,
            'independent_byte_audit': byte_audit,
            'frozen_semantic_audit_seconds': semantic_seconds,
            'semantic_result_unchanged': True, 'decoded_reads': store.decoded_reads,
            'decoded_bytes_read': store.decoded_bytes, 'maximum_decoded_read_bytes': store.max_decoded_read,
            'python_sources_materialized_bytes': script_bytes,
            'journal_or_native_binary_bytes_materialized': 0,
            'elapsed_seconds': time.monotonic() - started, 'new_native_rewrites': 0}


if __name__ == '__main__':
    if len(sys.argv) != 5:
        raise SystemExit('usage: audit_compact_evidence.py STORE INDEX_SHA SOURCE_SEAL AUDIT_MODULE')
    print(json.dumps(audit(Path(sys.argv[1]), *sys.argv[2:]), indent=2, sort_keys=True))
