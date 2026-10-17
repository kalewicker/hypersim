import builtins
from pathlib import Path
import symtable
import unittest


def undefined_globals(source):
    tree = symtable.symtable(source, '<verifier>', 'exec')
    known = {s.get_name() for s in tree.get_symbols() if s.is_assigned() or s.is_imported()}
    known |= set(dir(builtins)) | {'__file__', '__name__', '__package__', '__doc__'}
    missing = set()

    def visit(table):
        for symbol in table.get_symbols():
            if symbol.is_global() and symbol.is_referenced() and symbol.get_name() not in known:
                missing.add(symbol.get_name())
        for child in table.get_children():
            visit(child)
    visit(tree)
    return missing


class VerifierSourceTests(unittest.TestCase):
    def test_complete_verifier_imports(self):
        source = Path(__file__).with_name('verify_ere_categorical_comparison_bounded_v4.py').read_text()
        self.assertEqual(undefined_globals(source), set())

    def test_guard_detects_prior_missing_import_pattern(self):
        self.assertEqual(undefined_globals('def f(x):\n math.isfinite(x)\n sys.executable\n json.dumps(x)\n'), {'math', 'sys', 'json'})


if __name__ == '__main__':
    unittest.main()
