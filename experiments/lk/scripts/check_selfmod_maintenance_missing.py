#!/usr/bin/env python3
"""Read-only mocked input/coverage/result rejection; no science or source mutation."""
import argparse
import contextlib
import copy
import io
import json
import pathlib
import unittest.mock
import audit_selfmod_maintenance as auditor
from audit_selfmod_birth import sha


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('engineering', type=pathlib.Path)
    p.add_argument('receipt', type=pathlib.Path)
    args = p.parse_args()
    out = args.engineering.resolve()
    original = auditor.load
    rejected = []
    for name in ['offset', 'input_hash', 'configuration', 'founder']:
        def damaged(path):
            d = original(path)
            if name == 'offset' and path == out/'manifest.json':
                d = copy.deepcopy(d); d['offset'] = 0
            elif name == 'input_hash' and path == out/'input_provenance.json':
                d = copy.deepcopy(d); d['inputs_sha256']['inputs/audit.json'] = '0'*64
            elif name == 'configuration' and path == out/'seed_457/faithful/initial.json':
                d = copy.deepcopy(d); d['cfg']['rent'] += 1
            elif name == 'founder' and path == out/'inputs/seed_457/faithful/birth.json':
                d = copy.deepcopy(d); d['selection']['child'] = d['selection']['parent']
            return d
        try:
            with unittest.mock.patch.object(auditor, 'load', damaged), contextlib.redirect_stdout(io.StringIO()):
                auditor.audit(out)
        except (ValueError, FileNotFoundError):
            rejected.append(name)
        else:
            raise AssertionError('Accepted '+name)
    native_rows = auditor.reconstruct_extension
    def missing(directory, *args, **kwargs):
        if directory == out/'seed_457/edited':
            raise FileNotFoundError('Mocked omitted branch')
        return native_rows(directory, *args, **kwargs)
    try:
        with unittest.mock.patch.object(auditor, 'reconstruct_extension', missing), contextlib.redirect_stdout(io.StringIO()):
            auditor.audit(out)
    except (ValueError, FileNotFoundError):
        rejected.append('omitted_branch')
    else:
        raise AssertionError('Accepted omitted branch')
    source = original(out/'audit.json')
    for name in ['denominator', 'false_maintenance', 'false_aggregate', 'wrong_claimed_founder']:
        d = copy.deepcopy(source)
        if name == 'denominator': d['counts']['fixed_seed_denominator'] = 1
        elif name == 'false_maintenance': d['records'][0]['focal']['maintenance']['maintained_success'] = True
        elif name == 'false_aggregate': d['aggregates']['faithful']['maintained_families'] = 2
        else: d['records'][0]['founder'] = -1
        try:
            auditor.validate_claims(out, d)
        except (ValueError, KeyError):
            rejected.append(name)
        else:
            raise AssertionError('Accepted '+name)
    auditor.validate_claims(out, source)
    report = dict(passed=True, rejected=rejected, untampered_claims_passed=True,
                  script_sha256=sha(pathlib.Path(__file__)), engineering_only=True)
    with args.receipt.open('x') as f:
        json.dump(report, f, indent=2); f.write('\n')
    args.receipt.with_suffix('.py').write_bytes(pathlib.Path(__file__).read_bytes())
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
