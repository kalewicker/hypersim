"""Adversarial checks for the independent lineage and intervention auditor."""
import copy
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from audit_lineage import audit,load_package


class LineageAuditTests(unittest.TestCase):
    def reject(self,mutate):
        data=copy.deepcopy(DATA);mutate(data)
        with self.assertRaises((ValueError,KeyError,IndexError,StopIteration)):audit(*data)

    def test_complete_audit_and_fixed_negative_denominators(self):
        result=audit(*DATA)
        self.assertEqual((result["samples"],result["assays"],result["native_rewrites"]),(75,19,429))
        self.assertEqual((result["dead_samples"],result["empty_slots"],result["outside_horizon_slots"]),(8,7,14))
        self.assertEqual(result["discovery_candidate_passes"],0)

    def test_sampler_dead_empty_and_score_changes(self):
        self.reject(lambda d:d[3]["samples"].pop())
        self.reject(lambda d:d[3]["samples"][0]["ranking"][0].__setitem__(1,0))
        def living_only(d):
            s=next(s for s in d[3]["samples"] if s["status"]=="dead");s["status"]="living"
        self.reject(living_only)

    def test_paid_ancestry_and_origin_cannot_be_laundered(self):
        self.reject(lambda d:d[3]["samples"][72]["lineage"]["events"][0].__setitem__("origin","endogenous"))
        self.reject(lambda d:d[3]["samples"][72]["lineage"]["births"][0].__setitem__("copy_cost",0))
        def external(d):
            event=next(e for s in d[3]["samples"] for e in s.get("lineage",{}).get("events",[]) if e["origin"]=="external")
            event["origin"]="endogenous"
        self.reject(external)

    def test_parent_cutoff_and_seeded_material(self):
        def cutoff(d):
            sample=next(s for s in d[3]["samples"][:72] if len(s.get("lineage",{}).get("chain",[]))>1)
            sample["lineage"]["events"].append(copy.deepcopy(d[3]["samples"][72]["lineage"]["events"][0]))
        self.reject(cutoff)
        self.reject(lambda d:d[3]["samples"][74]["lineage"]["events"].append(copy.deepcopy(d[3]["samples"][72]["lineage"]["events"][0])))

    def test_group_padding_sham_restoration_requests(self):
        for role in ["remove_earlier","padded_remove_earlier","sham","restored"]:
            def change(d,role=role):
                request=next(r for r in d[4]["assays"] if r["role"]==role);request["stages"]=[0]
            self.reject(change)
        self.reject(lambda d:d[4]["assays"].pop())

    def test_activity_and_transmission_counts(self):
        self.reject(lambda d:d[3]["samples"][72]["lineage"]["pairs"][0].__setitem__("transmissions",[99,99]))
        self.reject(lambda d:d[3]["samples"][72]["lineage"]["pairs"][0].__setitem__("active_descendant_generations",[[1],[1]]))
        self.reject(lambda d:d[3]["samples"][72]["lineage"]["activity"][0].__setitem__("token","invented"))

    def test_native_costs_work_and_no_diagnostic_energy_bonus(self):
        self.reject(lambda d:d[5]["results"][0].__setitem__("score",512))
        self.reject(lambda d:d[5].__setitem__("native_rewrites",0))
        self.reject(lambda d:d[5]["results"][0]["initial"].__setitem__("capital",41))
        self.reject(lambda d:d[5]["results"].pop())

    def test_aggregate_and_admission_flags(self):
        self.reject(lambda d:d[6]["rows"][72]["pairs"][0].__setitem__("discovery_candidate_pass",True))
        self.reject(lambda d:d[6].__setitem__("evolution_demonstrated",True))
        self.reject(lambda d:d[7]["assay"].__setitem__("cycles",5))

    def test_embedded_known_source_custody(self):
        with tempfile.TemporaryDirectory() as directory:
            copied=Path(directory)/"package";shutil.copytree(PACKAGE,copied)
            source=copied/"inputs/shared/fixture.json"
            source.write_text(source.read_text()+" ")
            with self.assertRaisesRegex(ValueError,"source input custody"):load_package(copied)


if __name__=="__main__":
    PACKAGE=Path(sys.argv.pop(1)).resolve();DATA=load_package(PACKAGE)
    unittest.main(verbosity=2)
