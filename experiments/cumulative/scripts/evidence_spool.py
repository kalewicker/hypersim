"""Single-writer bounded evidence spool. Never executes or retries native work.

The caller must reserve before writing a fresh journal. Only this spool's owned
raw copies can be released, after exact compact verification and durable intent.
A surviving reservation is uncertain work; reopening never dispatches it again.
"""
from contextlib import contextmanager
import fcntl
import json
from pathlib import Path
import re
import uuid
from compact_evidence import (Store, allocation, encode_json, pack, publish,
                              read_bounded, require, sha, source_inventory, sync_dir)

PROFILE = 'cumulative-bounded-evidence-spool-v1'
RAW_LIMIT = 32 * 1024 * 1024
WRITER_LIMIT = 31 * 1024 * 1024
COMPACT_LIMIT = 48 * 1024 * 1024
MAX_FILES = 2050
CONTROL_RESERVE = 4 * 1024 * 1024
RESERVATION = RAW_LIMIT + COMPACT_LIMIT + 2 * MAX_FILES * 8192 + 262144
NAME = re.compile(r'(?:[0-9]{8}\.json|\.staging-[0-9]{8}\.json)\Z')


def usage(root):
    total = 0
    for p in root.rglob('*'):
        require(not p.is_symlink(), 'spool symlink refused')
        if p.is_file():
            total += max(allocation(p.stat().st_size), p.stat().st_blocks * 512) + 4096
        elif p.is_dir():
            total += 4096
    return total


class Spool:
    def __init__(self, root):
        self.root = Path(root)
        require(self.root.is_dir() and not self.root.is_symlink(), 'owned spool root')
        self.owner = json.loads(read_bounded(self.root / 'owner.json', 4096))
        require(set(self.owner) == {'profile', 'owner', 'cap_bytes'} and self.owner['profile'] == PROFILE,
                'spool ownership profile')
        require(type(self.owner['cap_bytes']) is int and CONTROL_RESERVE <= self.owner['cap_bytes'] <= 8 * 1024**3,
                'spool fixed cap')

    @classmethod
    def create(cls, root, cap_bytes):
        require(type(cap_bytes) is int and CONTROL_RESERVE <= cap_bytes <= 8 * 1024**3, 'cap domain')
        root = Path(root)
        root.mkdir(exist_ok=False)
        sync_dir(root.parent)
        for name in ('events', 'raw', 'compact', 'restore'):
            (root / name).mkdir()
        (root / 'lock').touch(exist_ok=False)
        publish(root / 'owner.json', encode_json({'profile': PROFILE, 'owner': uuid.uuid4().hex,
                                                'cap_bytes': cap_bytes}))
        return cls(root)

    @contextmanager
    def locked(self):
        # All API mutations, including recovery, serialize on this owned lock.
        read_bounded(self.root / 'lock', 0)
        with (self.root / 'lock').open('rb') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            try:
                yield
            finally:
                fcntl.flock(lock, fcntl.LOCK_UN)

    def events(self):
        paths = sorted((self.root / 'events').iterdir())
        require(len(paths) <= 10000, 'event count cap')
        records, head = [], None
        for i, p in enumerate(paths):
            require(p.name == f'{i:08}.json', 'incomplete event publication; manual recovery required')
            raw = read_bounded(p, 1048576)
            e = json.loads(raw)
            require(set(e) == {'owner', 'sequence', 'previous', 'value'} and
                    e['owner'] == self.owner['owner'] and e['sequence'] == i and e['previous'] == head,
                    'spool event chain')
            records.append(e['value'])
            head = sha(raw)
        return records, head

    def states(self):
        states = {}
        transitions = {'reserved': 'sealed', 'sealed': 'verified', 'verified': 'release_intent',
                       'release_intent': 'released'}
        records, _ = self.events()
        for e in records:
            unit, kind = e['unit'], e['kind']
            if kind == 'reserved':
                require(unit == f'u{len(states):06}' and unit not in states and
                        e['reservation'] == RESERVATION and e['writer_byte_allowance'] == WRITER_LIMIT,
                        'fresh reservation order')
                states[unit] = dict(e)
            elif kind in ('restore_intent', 'restored', 'restore_release_intent', 'restore_released'):
                require(unit in states and states[unit]['kind'] == 'released', 'restore requires archived source')
                expected = {None: 'restore_intent', 'restore_intent': 'restored',
                            'restored': 'restore_release_intent', 'restore_release_intent': 'restore_released'}
                require(expected.get(states[unit].get('restore_state')) == kind, 'restore transition')
                states[unit]['restore_state'] = kind
                if 'restore_reservation' in e:
                    states[unit]['restore_reservation'] = e['restore_reservation']
            else:
                require(unit in states and transitions.get(states[unit]['kind']) == kind, 'archive transition')
                states[unit].update(e)
        return states

    def append(self, event):
        records, head = self.events()
        require(len(records) < 10000, 'event count admission')
        value = {'owner': self.owner['owner'], 'sequence': len(records), 'previous': head, 'value': event}
        publish(self.root / 'events' / f'{len(records):08}.json', encode_json(value))

    def capacity(self, extra=0):
        states = self.states()
        live = [s for s in states.values() if s['kind'] != 'released']
        # Pending reservations remain fully charged, even after the writer dies.
        retained = usage(self.root)
        pending = sum(max(0, RESERVATION - usage(self.root / 'raw' / s['unit']) -
                          usage(self.root / 'compact' / s['unit'])) for s in live)
        pending += sum(max(0, s.get('restore_reservation', 0) - usage(self.root / 'restore' / s['unit']))
                       for s in states.values() if s.get('restore_state') not in (None, 'restore_released'))
        charged = retained + pending + CONTROL_RESERVE + extra
        require(charged <= self.owner['cap_bytes'], 'storage admission cap; work must not begin')
        return {'retained_bytes': retained, 'pending_reservation_bytes': pending,
                'control_reserve_bytes': CONTROL_RESERVE, 'charged_bytes': charged,
                'cap_bytes': self.owner['cap_bytes']}

    def reserve(self, request_identity):
        require(isinstance(request_identity, str) and len(request_identity) == 64 and
                all(c in '0123456789abcdef' for c in request_identity), 'request hash')
        with self.locked():
            states = self.states()
            require(not any(s['kind'] != 'released' for s in states.values()), 'single active writer; no uncertain retry')
            require(not any(s.get('restore_state') in ('restore_intent', 'restore_release_intent') for s in states.values()), 'unfinished restore lease')
            admission = self.capacity(RESERVATION + 1048576)
            unit = f'u{len(states):06}'
            self.append({'kind': 'reserved', 'unit': unit, 'request_sha256': request_identity,
                         'reservation': RESERVATION, 'writer_byte_allowance': WRITER_LIMIT, 'admission': admission})
            # Journal directory is created by the native writer, not by reserve().
            return {'unit': unit, 'directory': str(self.root / 'raw' / unit),
                    'journal_byte_allowance': WRITER_LIMIT, 'request_sha256': request_identity}

    def _store(self, unit, state):
        store = Store(self.root / 'compact' / unit, state['index_sha256'], state['seal_sha256'])
        store.verify_all()
        return store

    def _release(self, unit, state, fault=None):
        store = self._store(unit, state)
        raw = self.root / 'raw' / unit
        expected = {n: e['raw_sha256'] for n, e in store.entries.items()}
        actual = {p.name for p in raw.iterdir()} if raw.exists() else set()
        require(actual <= set(expected), 'release refuses unknown owned files')
        for i, name in enumerate(sorted(actual)):
            p = raw / name
            require(sha(read_bounded(p, RAW_LIMIT)) == expected[name], 'release rechecks each exact raw file')
            p.unlink()
            sync_dir(raw)
            if fault == 'during_release' and i == 0:
                raise RuntimeError('declared interruption during raw release')
        if raw.exists():
            raw.rmdir()
            sync_dir(raw.parent)
        self.append({'kind': 'released', 'unit': unit})

    def archive(self, unit, fault=None):
        require(fault in (None, 'after_seal', 'after_compact', 'after_verified', 'during_release'), 'fault domain')
        with self.locked():
            states = self.states()
            require(unit in states and states[unit]['kind'] == 'reserved', 'archive fresh owned reservation only')
            raw = self.root / 'raw' / unit
            require(raw.is_dir() and not raw.is_symlink(), 'closed owned journal exists')
            files = sorted(raw.iterdir())
            require(1 <= len(files) <= 2048 and all(NAME.fullmatch(p.name) for p in files), 'flat closed journal file domain')
            require(sum(len(read_bounded(p, RAW_LIMIT)) for p in files) <= WRITER_LIMIT, 'writer allowance exceeded')
            seal = {p.name: sha(read_bounded(p, RAW_LIMIT)) for p in files}
            seal_raw = encode_json(seal)
            publish(raw / 'output_sha256.json', seal_raw)
            root = sha(seal_raw)
            self.append({'kind': 'sealed', 'unit': unit, 'seal_sha256': root})
            if fault == 'after_seal':
                raise RuntimeError('declared interruption after seal')
            self._compact_and_release(unit, self.states()[unit], fault)

    def _compact_and_release(self, unit, state, fault=None):
        raw, compact = self.root / 'raw' / unit, self.root / 'compact' / unit
        if state['kind'] == 'sealed':
            # A complete compact index can be adopted after a crash; partial stores
            # remain charged and are never overwritten or silently retried.
            if compact.exists():
                index = read_bounded(compact / 'index.json', 16 * 1024**2)
                index_hash = sha(index)
            else:
                index_hash = pack(raw, compact, state['seal_sha256'], byte_cap=COMPACT_LIMIT)
            store = Store(compact, index_hash, state['seal_sha256'])
            store.verify_all()
            source_inventory(raw, state['seal_sha256'])
            if fault == 'after_compact':
                raise RuntimeError('declared interruption after compact publication')
            self.append({'kind': 'verified', 'unit': unit, 'index_sha256': index_hash})
            state = self.states()[unit]
        if state['kind'] == 'verified':
            self._store(unit, state)
            if fault == 'after_verified':
                raise RuntimeError('declared interruption after verified receipt')
            self.append({'kind': 'release_intent', 'unit': unit})
            state = self.states()[unit]
        if state['kind'] == 'release_intent':
            self._release(unit, state, fault)
        self.capacity()

    def recover(self, unit):
        with self.locked():
            state = self.states()[unit]
            require(state['kind'] in ('sealed', 'verified', 'release_intent'),
                    'reserved writer is uncertain; no redispatch or automatic retry')
            self._compact_and_release(unit, state)

    def restore(self, unit):
        with self.locked():
            state = self.states()[unit]
            require(state['kind'] == 'released' and state.get('restore_state') is None, 'one fresh bounded restore')
            require(not any(s.get('restore_state') not in (None, 'restore_released') for s in self.states().values()),
                    'single restore lease')
            store = self._store(unit, state)
            # Require the exact durable journal parser to reject torn staging and
            # unpaired actions before issuing a continuation source to a runner.
            from audit_durability import read
            values, head, staging = read(JournalView(store))
            require(not staging, 'torn journal cannot resume')
            pending = False
            for value in values:
                if value.get('kind') == 'intent':
                    require(not pending, 'nested intent'); pending = True
                elif value.get('kind') == 'result':
                    require(pending, 'orphan result'); pending = False
            require(not pending, 'uncertain native intent cannot resume')
            size = sum(allocation(e['raw_bytes']) + 4096 for n, e in store.entries.items()
                       if n != 'output_sha256.json')
            self.capacity(size + 262144)
            self.append({'kind': 'restore_intent', 'unit': unit, 'restore_reservation': size + 262144})
            destination = self.root / 'restore' / unit
            destination.mkdir(exist_ok=False)
            sync_dir(destination.parent)
            for name in sorted(store.entries):
                if name != 'output_sha256.json':
                    publish(destination / name, store.read(name))
            self.append({'kind': 'restored', 'unit': unit})
            return {'directory': str(destination), 'head': head, 'unit': unit}

    def release_restore(self, unit):
        with self.locked():
            state = self.states()[unit]
            require(state.get('restore_state') in ('restore_intent', 'restored', 'restore_release_intent'),
                    'owned restore lease required')
            store = self._store(unit, state)
            if state['restore_state'] == 'restore_intent':
                # Interrupted extraction is retained for operator inspection.
                raise ValueError('partial restore is not a completed continuation source')
            if state['restore_state'] == 'restored':
                self.append({'kind': 'restore_release_intent', 'unit': unit})
            directory = self.root / 'restore' / unit
            if directory.exists():
                require({p.name for p in directory.iterdir()} <= set(store.entries) - {'output_sha256.json'}, 'restore foreign file')
                for p in sorted(directory.iterdir()):
                    require(sha(read_bounded(p, RAW_LIMIT)) == store.entries[p.name]['raw_sha256'], 'restore changed')
                    p.unlink(); sync_dir(directory)
                directory.rmdir(); sync_dir(directory.parent)
            self.append({'kind': 'restore_released', 'unit': unit})
            self.capacity()


class JournalView:
    """Exclude the bundle seal from the unchanged native journal namespace."""
    def __init__(self, store):
        self.store = store
    def iterdir(self):
        return (p for p in self.store.root.iterdir() if p.name != 'output_sha256.json')
    def __truediv__(self, name):
        return self.store.root / name


def run_development_unit(spool, request, binary, timeout_seconds):
    """Storage admission wrapper for the existing development-only native binary.

    Native-work funding and semantic audit remain the caller's responsibility.
    A timeout/nonzero exit never causes redispatch. Closed partial evidence is
    archived like any other journal, and the child exit is retained verbatim.
    """
    import copy
    import subprocess
    require(request['purpose'] == 'development' and request['seed'] == 2026101001 and
            request['profile'] == 'cumulative-campaign-adapter-readiness-v1', 'development route only')
    require(0 < timeout_seconds <= 300, 'bounded child deadline')
    require(sha(read_bounded(Path(binary), RAW_LIMIT)) == request['source_identity']['binary_sha256'],
            'native executable identity')
    r = copy.deepcopy(request)
    r['journal_byte_allowance'] = min(r['journal_byte_allowance'], WRITER_LIMIT)
    raw = encode_json(r)
    require(len(raw) <= 65536, 'bounded native request')
    ticket = spool.reserve(sha(raw))
    requests = spool.root / 'requests'
    if not requests.exists():
        requests.mkdir(); sync_dir(spool.root)
    path = requests / (ticket['unit'] + '.json')
    publish(path, raw)
    try:
        result = subprocess.run([str(binary), str(path), ticket['directory']],
                                capture_output=True, text=True, timeout=timeout_seconds)
        receipt = {'returncode': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr,
                   'timed_out': False}
    except subprocess.TimeoutExpired as error:
        receipt = {'returncode': None, 'stdout': str(error.stdout or ''), 'stderr': str(error.stderr or ''),
                   'timed_out': True}
    publish(requests / (ticket['unit'] + '.receipt.json'), encode_json(receipt))
    if Path(ticket['directory']).is_dir():
        spool.archive(ticket['unit'])
    else:
        raise RuntimeError('native child left no closed journal; reservation retained, no retry')
    return ticket, r, receipt
