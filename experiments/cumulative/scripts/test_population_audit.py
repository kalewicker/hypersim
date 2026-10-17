import copy
import json
import sys
import unittest
from pathlib import Path
from audit_population import audit


class PopulationAuditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data=json.loads(Path(sys.argv[1]).read_text());cls.contract=json.loads(Path(sys.argv[2]).read_text())

    def reject(self,mutate):
        d=copy.deepcopy(self.data);mutate(d)
        with self.assertRaises((ValueError,KeyError,StopIteration)):audit(d,self.contract)

    def test_complete_audit(self):
        result=audit(self.data,self.contract)
        self.assertTrue(result["scheduler_reconstructed"]);self.assertEqual(result["native_rewrites"],159)

    def test_scheduler_scores_and_winner_membership(self):
        self.reject(lambda d:d["cases"][0]["rows"][0]["resource_ranking"][0].__setitem__(1,0))
        self.reject(lambda d:d["cases"][0]["rows"][0].update(resource_winners=[]))
        self.reject(lambda d:d["cases"][0]["rows"][0]["birth_ranking"].reverse())

    def test_no_free_birth_resources(self):
        def mutate(d):
            r=next(r for o in d["cases"][0]["rows"][0]["operations"] for r in o["receipts"] if r["kind"]=="shared_birth")
            r["child"]["environment"]=512
        self.reject(mutate)
        self.reject(lambda d:d["cases"][0]["rows"][0]["after"].update(reservoir=128))

    def test_genealogy_and_nonheritable_control(self):
        def mutate(d):
            r=next(r for o in d["cases"][4]["rows"][0]["operations"] for r in o["receipts"] if r["kind"]=="shared_birth")
            r["birth"]["transmitted"]["stages"]=[0,0]
        self.reject(mutate)
        self.reject(lambda d:d["cases"][0]["rows"][0]["after"]["births"][0].update(parent=999))

    def test_death_and_unfed_storage(self):
        self.reject(lambda d:d["cases"][5]["rows"][1]["operations"].clear())
        self.reject(lambda d:d["cases"][0]["rows"][0]["operations"][0]["receipts"][0].update(amount=0))

    def test_denominators_refusals_and_work(self):
        self.reject(lambda d:d["cases"].pop())
        self.reject(lambda d:d["cases"][7]["rows"][0]["birth_refusals"].clear())
        self.reject(lambda d:d.update(native_rewrites=158))

    def test_checkpoint_wire_indices_and_parity(self):
        self.reject(lambda d:d["continuation"]["checkpoint"]["cells"][0]["graph"]["nodes"][1]["value"]["properties"][0][1].update(Int=0))
        self.reject(lambda d:d["continuation"]["checkpoint"]["cells"][0]["graph"]["label_index"]["1000"].clear())
        self.reject(lambda d:d["continuation"]["restored"].pop())
        self.reject(lambda d:d["order"]["reverse"].pop())

    def test_admission_and_contract(self):
        self.reject(lambda d:d.update(evolution_demonstrated=True))
        self.reject(lambda d:d["contract"]["law"].update(child_environment_energy=512))


if __name__=="__main__":unittest.main(argv=[sys.argv[0]],verbosity=2)
