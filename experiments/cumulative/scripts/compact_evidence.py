"""Lossless, individually addressable evidence objects; no native execution.

A store is committed only when its index is durably published. Original seals
remain authoritative over decoded bytes. Decoding has a per-object output limit.
The read-only path facade lets unchanged evidence auditors avoid disk expansion.
"""
import fnmatch
import gzip
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
from types import SimpleNamespace
import zlib

PROFILE = 'cumulative-compact-evidence-v1'
CODEC = {'name': 'gzip', 'level': 6, 'mtime': 0}
MAX_RECORD = 32 * 1024 * 1024
MAX_INDEX = 16 * 1024 * 1024
MAX_ENTRIES = 200000
MAX_FIXTURE = 128 * 1024 * 1024
sha = lambda b: hashlib.sha256(b).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def encode_json(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n').encode()


def safe_name(name):
    require(isinstance(name, str) and name and '\\' not in name and '\x00' not in name,
            'unsafe evidence name')
    p = PurePosixPath(name)
    require(not p.is_absolute() and str(p) == name and all(x not in ('.', '..') for x in p.parts),
            'unsafe evidence path')
    return name


def regular(path):
    require(path.is_file() and not path.is_symlink(), 'regular evidence file required')
    for parent in path.parents:
        require(not parent.is_symlink(), 'symlink ancestor refused')


def read_bounded(path, limit):
    regular(path)
    require(path.stat().st_size <= limit, 'encoded file byte cap')
    with path.open('rb') as f:
        data = f.read(limit + 1)
    require(len(data) <= limit, 'encoded read byte cap')
    return data


def sync_dir(path):
    fd = os.open(path, os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def publish(path, raw):
    """Fresh immutable hard-link publication; staging survives interrupted writes."""
    staging = path.with_name('.staging-' + path.name)
    with staging.open('xb') as f:
        f.write(raw)
        f.flush()
        os.fsync(f.fileno())
    os.link(staging, path)
    sync_dir(path.parent)
    staging.unlink()
    sync_dir(path.parent)


def source_inventory(source, expected_root):
    seal_bytes = read_bounded(source / 'output_sha256.json', MAX_INDEX)
    require(sha(seal_bytes) == expected_root, 'known source seal root')
    seal = json.loads(seal_bytes)
    require(isinstance(seal, dict) and len(seal) + 1 <= MAX_ENTRIES, 'source inventory cap')
    expected = dict(seal, **{'output_sha256.json': expected_root})
    require('output_sha256.json' not in seal, 'source seal cannot contain itself')
    actual = set()
    for p in source.rglob('*'):
        require(not p.is_symlink(), 'source symlink refused')
        if p.is_file():
            actual.add(str(p.relative_to(source)))
    require(actual == set(expected), 'complete source file denominator')
    total = 0
    for name, digest in expected.items():
        safe_name(name)
        raw = read_bounded(source / name, MAX_RECORD)
        require(sha(raw) == digest, 'source payload digest')
        total += len(raw)
    require(total <= MAX_FIXTURE, 'development source total cap')
    return expected


def pack(source, destination, expected_root, byte_cap=MAX_FIXTURE):
    inventory = source_inventory(source, expected_root)
    destination.mkdir(exist_ok=False)
    sync_dir(destination.parent)
    (destination / 'objects').mkdir()
    sync_dir(destination)
    entries = []
    stored = 0
    for i, name in enumerate(sorted(inventory)):
        raw = read_bounded(source / name, MAX_RECORD)
        require(sha(raw) == inventory[name], 'source changed while packing')
        encoded = gzip.compress(raw, compresslevel=6, mtime=0)
        # Account for the complete eventual index before each publication.
        object_name = f'objects/{i:08}.gz'
        entry = {'path': name, 'object': object_name, 'raw_bytes': len(raw),
                 'raw_sha256': inventory[name], 'compressed_bytes': len(encoded),
                 'compressed_sha256': sha(encoded), 'mode': (source / name).stat().st_mode & 0o777}
        entries.append(entry)
        index = {'profile': PROFILE, 'codec': CODEC, 'source_seal_sha256': expected_root,
                 'entries': entries}
        index_raw = encode_json(index)
        require(stored + len(encoded) + len(index_raw) <= byte_cap and len(index_raw) <= MAX_INDEX,
                'compact store publication cap')
        publish(destination / object_name, encoded)
        stored += len(encoded)
    publish(destination / 'index.json', index_raw)
    source_inventory(source, expected_root)
    return sha(index_raw)


class Store:
    def __init__(self, directory, expected_index, expected_source):
        self.directory = Path(directory)
        raw = read_bounded(self.directory / 'index.json', MAX_INDEX)
        require(sha(raw) == expected_index, 'known compact index root')
        index = json.loads(raw)
        require(set(index) == {'profile', 'codec', 'source_seal_sha256', 'entries'} and
                index['profile'] == PROFILE and index['codec'] == CODEC and
                index['source_seal_sha256'] == expected_source, 'compact index identity/schema')
        entries = index['entries']
        require(isinstance(entries, list) and 1 <= len(entries) <= MAX_ENTRIES, 'compact index count')
        self.entries = {}
        total = 0
        for i, e in enumerate(entries):
            require(set(e) == {'path', 'object', 'raw_bytes', 'raw_sha256',
                               'compressed_bytes', 'compressed_sha256', 'mode'}, 'entry schema')
            safe_name(e['path'])
            require(e['path'] not in self.entries and e['object'] == f'objects/{i:08}.gz',
                    'unique path and ordered physical object')
            require(type(e['raw_bytes']) is int and 0 <= e['raw_bytes'] <= MAX_RECORD and
                    type(e['compressed_bytes']) is int and 0 < e['compressed_bytes'] <= MAX_RECORD + 65536,
                    'bounded decoded and compressed lengths')
            require(type(e['mode']) is int and 0 <= e['mode'] <= 0o777, 'ordinary permission bits')
            for key in ('raw_sha256', 'compressed_sha256'):
                require(isinstance(e[key], str) and len(e[key]) == 64 and
                        all(c in '0123456789abcdef' for c in e[key]), 'digest schema')
            self.entries[e['path']] = e
            total += e['raw_bytes']
        require(total <= MAX_FIXTURE, 'development decoded total cap')
        require(list(self.entries) == sorted(self.entries), 'canonical inventory order')
        actual = set()
        for p in self.directory.rglob('*'):
            require(not p.is_symlink(), 'compact symlink refused')
            if p.is_file():
                actual.add(str(p.relative_to(self.directory)))
        require(actual == {'index.json'} | {e['object'] for e in entries},
                'committed compact file set, no staging or extras')
        self.expected_source = expected_source
        self.decoded_reads = 0
        self.decoded_bytes = 0
        self.max_decoded_read = 0
        original_seal = self.read('output_sha256.json')
        require(sha(original_seal) == expected_source, 'decoded original seal root')
        original = json.loads(original_seal)
        expected = dict(original, **{'output_sha256.json': expected_source})
        require('output_sha256.json' not in original and set(expected) == set(self.entries) and
                all(e['raw_sha256'] == expected[n] for n, e in self.entries.items()),
                'compact index retains exact original sealed denominator')
        self.directories = {''}
        for name in self.entries:
            p = PurePosixPath(name).parent
            while str(p) != '.':
                self.directories.add(str(p))
                p = p.parent
        self.root = CompactPath(self, '')

    def read(self, name):
        e = self.entries[name]
        encoded = read_bounded(self.directory / e['object'], e['compressed_bytes'])
        require(len(encoded) == e['compressed_bytes'] and sha(encoded) == e['compressed_sha256'],
                'stored object size/hash')
        decoder = zlib.decompressobj(wbits=31)
        try:
            raw = decoder.decompress(encoded, e['raw_bytes'] + 1)
        except zlib.error as exc:
            raise ValueError('invalid gzip payload') from exc
        require(len(raw) == e['raw_bytes'] and decoder.eof and not decoder.unused_data and
                not decoder.unconsumed_tail and sha(raw) == e['raw_sha256'],
                'bounded exact single-member decode and source digest')
        self.decoded_reads += 1
        self.decoded_bytes += len(raw)
        self.max_decoded_read = max(self.max_decoded_read, len(raw))
        return raw

    def verify_all(self):
        for name in self.entries:
            self.read(name)


class CompactPath:
    """Read-only decoded namespace. It never materializes journal payloads on disk."""
    def __init__(self, store, name):
        self.store, self.key = store, name

    def __truediv__(self, name):
        name = str(name)
        safe_name(name)
        return CompactPath(self.store, '/'.join(x for x in (self.key, name) if x))

    def __str__(self):
        return self.key or '.'

    def __eq__(self, other):
        return isinstance(other, CompactPath) and self.store is other.store and self.key == other.key

    def __hash__(self):
        return hash((id(self.store), self.key))

    @property
    def name(self):
        return PurePosixPath(self.key).name

    def is_file(self):
        return self.key in self.store.entries

    def is_dir(self):
        return self.key in self.store.directories

    def is_symlink(self):
        return False  # Physical objects/ancestors are checked on every read.

    def exists(self):
        return self.is_file() or self.is_dir()

    def read_bytes(self):
        return self.store.read(self.key)

    def read_text(self, encoding='utf-8'):
        return self.read_bytes().decode(encoding)

    def stat(self):
        return SimpleNamespace(st_size=self.store.entries[self.key]['raw_bytes'])

    def relative_to(self, other):
        require(self.store is other.store, 'different compact namespace')
        return PurePosixPath(self.key).relative_to(PurePosixPath(other.key))

    def iterdir(self):
        require(self.is_dir(), 'compact directory missing')
        prefix = self.key + '/' if self.key else ''
        children = {n[len(prefix):].split('/')[0] for n in
                    self.store.entries.keys() | self.store.directories if n.startswith(prefix) and n != self.key}
        return iter(self / n for n in sorted(children))

    def rglob(self, pattern):
        require(pattern == '*', 'unsupported compact recursive pattern')
        prefix = self.key + '/' if self.key else ''
        return iter(CompactPath(self.store, n) for n in sorted(self.store.entries.keys() | self.store.directories)
                    if n.startswith(prefix) and n != self.key)

    def glob(self, pattern):
        return (p for p in self.iterdir() if fnmatch.fnmatchcase(p.name, pattern))


def allocation(size, block=4096):
    return ((size + block - 1) // block) * block

if __name__ == '__main__':
    import sys
    if len(sys.argv) != 4:
        raise SystemExit('usage: compact_evidence.py SEALED_SOURCE NEW_STORE SOURCE_SEAL_SHA')
    source, destination = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    digest = pack(source, destination, sys.argv[3])
    store = Store(destination, digest, sys.argv[3])
    store.verify_all()
    files = [p for p in destination.rglob('*') if p.is_file()]
    print(json.dumps({'compact_index_sha256': digest, 'files': len(store.entries),
                      'source_raw_bytes': sum(e['raw_bytes'] for e in store.entries.values()),
                      'compressed_payload_bytes': sum(p.stat().st_size for p in files),
                      'rounded_4k_bytes': sum(allocation(p.stat().st_size) for p in files),
                      'allocated_bytes_on_this_filesystem': sum(p.stat().st_blocks * 512 for p in files),
                      'native_rewrites': 0}, indent=2, sort_keys=True))
