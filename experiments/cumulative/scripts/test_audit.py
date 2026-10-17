import copy
import json
import sys
import unittest
from pathlib import Path
from audit_readiness import audit


class AuditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data = json.loads(Path(sys.argv[1]).read_text())
        cls.contract = json.loads(Path(sys.argv[2]).read_text())

    def rejected(self, mutate):
        d = copy.deepcopy(self.data)
        mutate(d)
        with self.assertRaises((ValueError, KeyError, StopIteration)):
            audit(d, self.contract)

    def test_valid_package(self):
        result = audit(self.data, self.contract)
        self.assertEqual(result["native_rewrites"], 501)

    def test_denominators_and_work(self):
        self.rejected(lambda d: d["cases"].pop())
        self.rejected(lambda d: d["transmissions"].pop())
        self.rejected(lambda d: d.update(native_rewrites=500))

    def test_credit_and_native_outcome(self):
        def mutate(d):
            r = next(r for r in d["cases"][0]["receipts"] if r["kind"] == "settle")
            r["extra"]["payout"] += 1
        self.rejected(mutate)
        self.rejected(lambda d: d["cases"][0]["final"].update(capital=99))

    def test_edit_funding_identity_and_opportunities(self):
        def mutate(d):
            r = d["cases"][8]["receipts"][0]
            r["extra"]["opportunities"] = []
        self.rejected(mutate)
        self.rejected(lambda d: d["cases"][8]["receipts"][0]["bound"][0].update(idx=2))
        self.rejected(lambda d: d["cases"][8]["receipts"][0]["native_after"].update(capital=40))

    def test_ancestry_and_heredity(self):
        def mutate(d):
            r = next(r for r in d["transmissions"][0]["receipts"] if r["kind"] == "birth")
            r["child"]["parent"] = 999
        self.rejected(mutate)
        def mutate_genome(d):
            r = next(r for r in d["cases"][11]["receipts"] if r["kind"] == "birth")
            r["transmitted"]["stages"] = [0, 1]
        self.rejected(mutate_genome)

    def test_native_resume_and_order_parity(self):
        self.rejected(lambda d: d["continuation"]["restored_final"].update(word_pos="0"))
        self.rejected(lambda d: d["order"]["reverse_receipts"].pop())

    def test_accounting_and_local_ownership(self):
        self.rejected(lambda d: d["cases"][0]["initial"].update(environment=513))
        self.rejected(lambda d: d["cases"][0]["initial"]["edges"][0]["nodes"].reverse())

    def test_admission_and_contract(self):
        self.rejected(lambda d: d.update(evolution_demonstrated=True))
        self.rejected(lambda d: d["contract"]["law"].update(edit_cost=0))


if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0]], verbosity=2)
