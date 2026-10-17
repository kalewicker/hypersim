"""Zero-native corruption and publication boundary tests on owned tiny fixtures."""
import copy
import gzip
import json
from pathlib import Path
import shutil
import tempfile
from compact_evidence import Store, encode_json, pack, sha


def run():
    results = []
    with tempfile.TemporaryDirectory(prefix='compact-evidence-tests-') as temporary:
        root = Path(temporary).resolve()
        source = root / 'source'
        source.mkdir()
        payload = b'fixed source payload\n'
        (source / 'receipt.json').write_bytes(payload)
        seal = encode_json({'receipt.json': sha(payload)})
        (source / 'output_sha256.json').write_bytes(seal)
        source_root = sha(seal)
        good = root / 'good'
        index_root = pack(source, good, source_root)
        assert pack(source, root / 'deterministic_copy', source_root) == index_root
        store = Store(good, index_root, source_root)
        store.verify_all()
        assert store.read('receipt.json') == payload
        index = json.loads((good / 'index.json').read_bytes())
        payload_index = next(i for i, e in enumerate(index['entries']) if e['path'] == 'receipt.json')

        def rejected(name, mutate, resign=False, source_hash=source_root):
            target = root / name
            shutil.copytree(good, target)
            data = copy.deepcopy(index)
            mutate(target, data)
            expected = index_root
            if resign:
                encoded = encode_json(data)
                (target / 'index.json').write_bytes(encoded)
                expected = sha(encoded)
            try:
                Store(target, expected, source_hash).verify_all()
            except (ValueError, FileNotFoundError, KeyError, OSError):
                results.append(name)
            else:
                raise AssertionError('accepted corruption: ' + name)

        def object_bytes(target, data, content):
            e = data['entries'][payload_index]
            (target / e['object']).write_bytes(content)
            e['compressed_bytes'] = len(content)
            e['compressed_sha256'] = sha(content)

        rejected('index_root', lambda p, d: (p / 'index.json').write_bytes(b'{}\n'))
        rejected('bit_flip', lambda p, d: (p / d['entries'][payload_index]['object']).write_bytes(b'bad'))
        rejected('truncated_gzip', lambda p, d: object_bytes(p, d, (p / d['entries'][payload_index]['object']).read_bytes()[:-3]), True)
        rejected('concatenated_gzip', lambda p, d: object_bytes(p, d, (p / d['entries'][payload_index]['object']).read_bytes() + gzip.compress(b'', mtime=0)), True)
        rejected('trailing_bytes', lambda p, d: object_bytes(p, d, (p / d['entries'][payload_index]['object']).read_bytes() + b'junk'), True)
        rejected('expansion_bomb', lambda p, d: object_bytes(p, d, gzip.compress(b'x' * 100000, mtime=0)), True)
        rejected('oversized_declaration', lambda p, d: d['entries'][payload_index].update(raw_bytes=33554433), True)
        rejected('path_escape', lambda p, d: d['entries'][payload_index].update(path='../escape'), True)
        rejected('duplicate_path', lambda p, d: d['entries'][1].update(path=d['entries'][0]['path']), True)
        rejected('missing_object', lambda p, d: (p / d['entries'][payload_index]['object']).unlink())
        rejected('extra_object', lambda p, d: (p / 'objects/extra.gz').write_bytes(b'extra'))
        rejected('incomplete_index', lambda p, d: (p / 'index.json').unlink())
        rejected('pending_publication', lambda p, d: (p / '.staging-index.json').write_bytes(b'{'))
        rejected('changed_decoded_digest', lambda p, d: d['entries'][payload_index].update(raw_sha256='0' * 64), True)
        rejected('wrong_original_seal', lambda p, d: None, source_hash='0' * 64)

        def symlink(p, data):
            target = p / data['entries'][payload_index]['object']
            target.unlink()
            target.symlink_to(source / 'receipt.json')
        rejected('symlink_object', symlink)
        capped = root / 'capped'
        try:
            pack(source, capped, source_root, byte_cap=1)
        except ValueError:
            assert not (capped / 'index.json').exists()
            results.append('cap_before_publication')
        else:
            raise AssertionError('publication cap bypass')
        try:
            pack(source, good, source_root)
        except FileExistsError:
            assert sha((good / 'index.json').read_bytes()) == index_root
            results.append('no_overwrite')
        else:
            raise AssertionError('existing store overwritten')
    return {'passed': True, 'checks': results, 'corruptions_and_publication_failures_rejected': len(results),
            'deterministic_republication': True, 'native_rewrites': 0}


if __name__ == '__main__':
    print(json.dumps(run(), indent=2))
