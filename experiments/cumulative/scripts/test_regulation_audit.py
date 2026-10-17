"""Adversarial tests of the independent regulation measurement boundary."""
import copy
import json
import sys
import unittest
from pathlib import Path
from audit_regulation import audit

class RegulationAuditTests(unittest.TestCase):
    def reject(self,mutate):
        data=copy.deepcopy(DATA);mutate(data)
        with self.assertRaises((ValueError,KeyError,IndexError,StopIteration)):audit(data,CONTRACT)

    def test_full_denominator_and_native_families(self):
        result=audit(DATA,CONTRACT)
        self.assertEqual(result["native_rewrites"],108)
        self.assertEqual(result["native_families"],list(range(5)))
        self.assertEqual(len(result["cases"]),17)
        self.assertFalse(result["evolution_demonstrated"])

    def test_policy_and_full_opportunity_denominator(self):
        self.reject(lambda d:d["cases"][0]["frames"][1]["opportunities"].pop())
        self.reject(lambda d:d["cases"][0]["frames"][1].__setitem__("binding_count",1))
        self.reject(lambda d:d["cases"][0]["frames"][1].__setitem__("selected",[1,0]))
        self.reject(lambda d:d["cases"][6]["initial"]["program"].__setitem__("enabled",1))

    def test_native_binding_effect_and_ownership(self):
        self.reject(lambda d:d["cases"][0]["frames"][2]["receipts"][0].__setitem__("effects",3))
        self.reject(lambda d:d["cases"][0]["frames"][2]["receipts"][0]["bound"].reverse())
        self.reject(lambda d:d["cases"][0]["frames"][2]["receipts"][0]["native_after"]["edges"][0].__setitem__("label",1001))
        self.reject(lambda d:d["cases"][0]["frames"][2]["receipts"][0]["native_after"]["nodes"][0]["properties"][0].__setitem__(1,40))

    def test_refusal_death_and_horizon_cannot_disappear(self):
        self.reject(lambda d:d["cases"][9]["frames"].pop())
        self.reject(lambda d:d["cases"][9]["final"].__setitem__("alive",True))
        self.reject(lambda d:d["cases"][7]["frames"][1].__setitem__("reason","disabled"))
        self.reject(lambda d:d["cases"][16]["frames"][1]["restrictions"].pop())

    def test_stable_serials_and_complete_policy_transmission(self):
        self.reject(lambda d:d["continuation"]["child"]["program"].__setitem__("next_serial",3))
        self.reject(lambda d:d["continuation"]["original"][0]["transmitted"]["genes"][1].__setitem__(0,1))
        self.reject(lambda d:d["cases"][12]["child"]["initial"]["program"].__setitem__("enabled",0))
        self.reject(lambda d:d["cases"][13]["child"]["frames"][0].__setitem__("reason","arm"))

    def test_birth_cost_reserve_and_resource_creation(self):
        self.reject(lambda d:d["cases"][11]["frames"][-1].__setitem__("copy_cost",0))
        self.reject(lambda d:d["cases"][11]["child"]["initial"].__setitem__("reservoir",16))
        self.reject(lambda d:d["cases"][11]["child"]["initial"]["core"].__setitem__("environment",16))
        self.reject(lambda d:d["cases"][14]["frames"][-1].__setitem__("reason","dead"))

    def test_authored_restoration_and_external_provenance(self):
        self.reject(lambda d:d["causal"]["restored"]["frames"][0].__setitem__("diagnostic",False))
        self.reject(lambda d:d["causal"]["sham"]["frames"][0].__setitem__("probe",[3,0]))
        self.reject(lambda d:d["cases"][11]["child"]["frames"][0].__setitem__("external",False))
        self.reject(lambda d:d["causal"]["disabled"]["frames"][-1]["after"]["core"].__setitem__("word_pos","3"))

    def test_checkpoint_indices_allocator_and_order(self):
        self.reject(lambda d:d["continuation"]["checkpoint"]["graph"]["label_index"].__setitem__("1001",[]))
        self.reject(lambda d:d["continuation"]["checkpoint"]["state"]["core"]["allocator"][0].append(1))
        self.reject(lambda d:d["continuation"]["restored"].pop())
        self.reject(lambda d:d["order"]["reverse"].pop())
        self.reject(lambda d:d["offset"]["final"]["program"].__setitem__("mask",0))

    def test_work_contract_and_admission(self):
        self.reject(lambda d:d["cases"].pop())
        self.reject(lambda d:d.__setitem__("native_rewrites",0))
        self.reject(lambda d:d.__setitem__("scientific_acceptance",True))
        self.reject(lambda d:d["contract"]["law"].__setitem__("edit_cost",0))

if __name__=="__main__":
    DATA=json.loads(Path(sys.argv.pop(1)).read_text());CONTRACT=json.loads(Path(sys.argv.pop(1)).read_text())
    unittest.main(verbosity=2)
