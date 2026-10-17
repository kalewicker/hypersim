#!/usr/bin/env python3
"""Postmeasurement, receipt-only ordered-sum correction; originals stay frozen."""
import argparse
import ast
import copy
import difflib
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
PROFILE = 'E2-categorical-comparison-postmeasurement-ordered-audit-v1'
INPUT_NAME = 'ere_categorical_comparison_scientific_20261003'
INPUT_ROOT = '387a6e32a152250a081b246e53e5a69adbd273c4f465f726594a256ca3600fc1'
FAILED_SHA = 'f234d35b1b7e1a780a66a2bf6a7856c0fa2668b157a8ab34a09f852c42c4e294'
ADAPTER_SHA = '2c5e2d64fcff7c7600b46151c1e7eb857673f155c0f7d84e1fd79c949d47db28'
ROOT = next(p for p in Path(__file__).resolve().parents
            if (p / 'output' / INPUT_NAME / 'output_sha256.json').is_file())
INPUT = ROOT / 'output' / INPUT_NAME
FROZEN = INPUT / 'source_snapshots'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


# Bind every imported research helper to the failed archive, not live scripts.
assert digest(INPUT / 'output_sha256.json') == INPUT_ROOT, 'failed archive root'
pins = json.loads((INPUT / 'output_sha256.json').read_text())
assert digest(INPUT / 'failed_audit.json') == FAILED_SHA
for name, h in pins.items():
    if name.startswith('source_snapshots/scripts/'):
        path = INPUT / name
        assert not path.is_symlink() and digest(path) == h, 'frozen helper drift'
sys.path.insert(0, str(FROZEN / 'scripts'))
import audit_ere_categorical_comparison_bridge as original

a = original.a
cat = original.cat
assert Path(cat.__file__).resolve() == (FROZEN / 'scripts/audit_ere_categorical_adapter.py').resolve()
for module in list(sys.modules.values()):
    file = getattr(module, '__file__', None)
    if file and Path(file).resolve().parent == (FROZEN / 'scripts').resolve():
        name = str(Path(file).resolve().relative_to(INPUT))
        assert name in pins and digest(file) == pins[name]


def ordered_sum(values):
    result = 0.
    for value in values:
        result += value
    return result


def transform(source):
    """Exactly three callee replacements in two pinned functions; no other AST edit."""
    assert hashlib.sha256(source).hexdigest() == ADAPTER_SHA, 'pinned adapter source'
    tree = ast.parse(source)
    nodes = {n.name: n for n in tree.body if isinstance(n, ast.FunctionDef)}
    changes = []
    functions = []
    replacements = []
    lines = source.decode().splitlines(keepends=True)
    for name, count in (('softmax', 1), ('estimate', 2)):
        before = nodes[name]
        after = copy.deepcopy(before)

        class Replace(ast.NodeTransformer):
            def __init__(self):
                self.count = 0

            def visit_Call(self, node):
                if (isinstance(node.func, ast.Attribute)
                        and isinstance(node.func.value, ast.Name)
                        and node.func.value.id == 'math' and node.func.attr == 'fsum'):
                    self.count += 1
                    changes.append(dict(function=name, line=node.lineno,
                                        before='math.fsum', after='_registered_ordered_sum'))
                    node.func = ast.copy_location(ast.Name('_registered_ordered_sum', ast.Load()), node.func)
                return self.generic_visit(node)

        replacer = Replace()
        after = ast.fix_missing_locations(replacer.visit(after))
        assert replacer.count == count, 'unique registered substitutions'
        normalized = copy.deepcopy(after)

        class Restore(ast.NodeTransformer):
            def visit_Name(self, node):
                if node.id == '_registered_ordered_sum':
                    return ast.copy_location(ast.Attribute(ast.Name('math', ast.Load()), 'fsum', ast.Load()), node)
                return node

        normalized = Restore().visit(normalized)
        assert ast.dump(normalized) == ast.dump(before), 'AST change outside three sums'
        functions.append(after)
        fragment = ''.join(lines[before.lineno-1:before.end_lineno])
        assert fragment.count('math.fsum(') == count
        replacements.append((fragment, fragment.replace('math.fsum(', '_registered_ordered_sum(')))
    text = source.decode()
    for before, after in replacements:
        assert text.count(before) == 1
        text = text.replace(before, after, 1)
    assert len(changes) == 3
    proof = dict(profile=PROFILE, postmeasurement=True, original_adapter_sha256=ADAPTER_SHA,
                 substitutions=changes, ast_identical_except_three_sum_callees=True,
                 strict_f32_bits_unchanged=True, tolerances_unchanged=True,
                 native_sources_unchanged=True, scientific_criteria_unchanged=True)
    diff = ''.join(difflib.unified_diff(source.decode().splitlines(True), text.splitlines(True),
                                      'frozen/audit_ere_categorical_adapter.py',
                                      'supplemental/in-memory-ordered-adapter.py'))
    return functions, proof, diff


ORIGINAL_ESTIMATE = cat.estimate
ORIGINAL_SOFTMAX = cat.softmax
functions, PROOF, DIFF = transform(Path(cat.__file__).read_bytes())
cat.__dict__['_registered_ordered_sum'] = ordered_sum
exec(compile(ast.Module(body=functions, type_ignores=[]), '<registered-three-sum-correction>', 'exec'), cat.__dict__)


def verify_input():
    a.verify(INPUT, expected=INPUT_ROOT, count=12883)
    assert a.digest(INPUT / 'failed_audit.json') == FAILED_SHA
    assert not (INPUT / 'audit.json').exists(), 'original audit result must remain failed'
    snapshot = a.load(INPUT / 'snapshot.json')
    assert len(snapshot['sources']) == 481
    assert all(a.digest(ROOT / n) == h for n, h in snapshot['sources'].items()), 'live measured source drift'
    assert a.digest(INPUT / 'executed_binary') == snapshot['binary_sha256']


def audit():
    verify_input()
    result = original.audit(INPUT)
    verify_input()
    return dict(profile=PROFILE, postmeasurement=True,
                original_frozen_audit_pass=False, original_failure_preserved=True,
                input_archive=str(INPUT), input_root=INPUT_ROOT,
                correction=PROOF, corrected_accounting=result,
                summary=a.load(INPUT / 'summary.json'),
                native_evaluations_run=0, native_steps_run=0,
                grammar_compilations_run=0, training_updates_run=0,
                independent_physics_replay=False, independent_rng_replay=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.parse_args()
    print(json.dumps(audit(), indent=2))
