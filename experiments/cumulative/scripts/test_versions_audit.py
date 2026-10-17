"""Corruption tests plus explicitly synthetic version-decoder edge cases.
Synthetic traces test identity semantics only and never count as native evidence.
"""
import copy
import json
import sys
import unittest
from pathlib import Path
from audit_versions import audit,load,reconstruction
from plan_versions import descent
from audit_readiness import handle

class IntegrationAuditTests(unittest.TestCase):
    def reject(self,mutation):
        d=copy.deepcopy(DATA);mutation(d)
        with self.assertRaises((ValueError,KeyError,IndexError,StopIteration)):audit(*d)

    def test_full_denominator_and_unchanged_frozen_assays(self):
        r=audit(*DATA)
        self.assertEqual((r["samples"],r["assays"],r["native_rewrites"]),(96,33,529))
        self.assertEqual((r["living_samples"],r["dead_samples"],r["empty_slots"],r["outside_horizon_slots"]),(58,5,11,22))
        self.assertEqual(r["retained_pairs"],0);self.assertEqual(r["discovery_candidate_passes"],0)

    def test_complete_shared_scheduler_and_site_refusals(self):
        self.reject(lambda d:d[0]["cases"][0]["rows"][0]["resource_ranking"].pop())
        self.reject(lambda d:d[0]["cases"][0]["rows"][0]["resource_winners"].clear())
        self.reject(lambda d:d[0]["cases"][9]["rows"][0]["birth_refusals"].pop())
        self.reject(lambda d:d[0]["cases"][10]["rows"][0]["resource_winners"].append(1))

    def test_no_free_resource_and_full_birth_policy(self):
        def allocation(d):
            f=next(f for op in d[0]["cases"][0]["rows"][0]["operations"] for f in op["frames"] if f["kind"]=="shared_allocation");f["amount"]=32
        self.reject(allocation)
        def birth(d):
            f=next(f for op in d[0]["cases"][11]["rows"][0]["operations"] for f in op["frames"] if f["kind"]=="shared_birth");f["child"]["reservoir"]=16
        self.reject(birth)
        self.reject(lambda d:d[0]["cases"][11]["final"]["births"][0]["transmitted"].__setitem__("mask",0))
        self.reject(lambda d:d[0]["cases"][2]["rows"][0]["after"]["cells"][1]["state"]["program"].__setitem__("enabled",0))

    def test_death_edit_cadence_and_external_origin(self):
        self.reject(lambda d:d[0]["cases"][8]["rows"].pop())
        self.reject(lambda d:d[0]["cases"][8]["final"]["cells"][0].__setitem__("site",0))
        def extra(d):
            op=next(op for op in d[0]["cases"][0]["rows"][1]["operations"] if op["phase"]=="edit");op["frames"].append(copy.deepcopy(d[0]["cases"][0]["rows"][0]["operations"][2]["frames"][0]))
        self.reject(extra)
        def origin(d):
            op=next(op for row in d[0]["cases"][1]["rows"] for op in row["operations"] if op["phase"]=="newborn");op["frames"][0]["external"]=False
        self.reject(origin)

    def test_deleted_genes_stable_serials_and_policy_versions(self):
        sample=next(i for i,s in enumerate(DATA[1]["samples"]) if s.get("lineage",{}).get("tombstones"))
        self.reject(lambda d:d[1]["samples"][sample]["lineage"]["tombstones"].pop())
        self.reject(lambda d:d[1]["samples"][sample]["lineage"]["tokens"][0].__setitem__("version","invented"))
        self.reject(lambda d:d[1]["samples"][sample]["lineage"]["policy"].__setitem__("origin","invented"))
        self.reject(lambda d:d[1]["samples"][sample]["lineage"]["tokens"][0].__setitem__("serial",99))

    def test_parent_cutoff_and_exact_acquisition_activity(self):
        child=next(i for i,s in enumerate(DATA[1]["samples"]) if s["source"]=="version_witness" and s.get("cell")==1 and s["census"]==2)
        parent=next(s for s in DATA[1]["samples"] if s["source"]=="version_witness" and s.get("cell")==0 and s["census"]==2)
        self.assertEqual(len(DATA[1]["samples"][child]["lineage"]["events"]),3);self.assertEqual(len(parent["lineage"]["events"]),6)
        self.reject(lambda d:d[1]["samples"][child]["lineage"]["events"].append(copy.deepcopy(parent["lineage"]["events"][-2])))
        used=next(i for i,s in enumerate(DATA[1]["samples"]) if s.get("lineage",{}).get("activity"))
        self.reject(lambda d:d[1]["samples"][used]["lineage"]["activity"][0].__setitem__("acquisition","invented"))
        self.reject(lambda d:d[1]["samples"][child]["lineage"]["births"][0]["policy"].__setitem__("mask",0))

    def test_fixed_sampling_requests_and_admission(self):
        self.reject(lambda d:d[1]["samples"].pop())
        self.reject(lambda d:d[2]["assays"].pop())
        self.reject(lambda d:d[3]["results"][0].__setitem__("score",512))
        self.reject(lambda d:d[4].__setitem__("evolution_demonstrated",True))
        self.reject(lambda d:d[5]["sampling"].__setitem__("slots",1))

    def test_checkpoint_graph_membership_and_order(self):
        self.reject(lambda d:d[0]["continuation"]["checkpoint"]["cells"][0]["checkpoint"]["graph"]["label_index"].__setitem__("1001",[]))
        self.reject(lambda d:d[0]["continuation"]["checkpoint"]["cells"][0]["checkpoint"]["state"]["core"]["allocator"][0].append(1))
        self.reject(lambda d:d[0]["order"]["reverse"].pop())
        self.reject(lambda d:d[0]["order"]["offset_final"]["cells"][0]["state"]["program"].__setitem__("mask",31))

    def test_synthetic_neutral_versions_redundancy_and_birth_cutoff(self):
        states,hs=synthetic()
        planned=descent(states,hs,2);checked=reconstruction(states,hs,2)
        self.assertEqual(planned,checked);self.assertEqual(len(checked["pairs"]),1)
        pair=checked["pairs"][0];self.assertEqual(pair["transmissions"],[2,2])
        self.assertEqual(pair["active_descendant_generations"],[[2],[1,2]])
        first=checked["events"][0]["id"];gene=next(t for t in checked["tokens"] if t["serial"]==1)
        self.assertEqual(gene["acquisition"],first);self.assertNotEqual(gene["version"],first)
        self.assertTrue(checked["tombstones"]);self.assertFalse(any(e["target"]==1 and e["stage"]==0 for e in checked["events"]))

    def test_synthetic_reacquisition_is_not_a_new_innovation(self):
        states,hs=synthetic(reacquire=True)
        planned=descent(states,hs,2);checked=reconstruction(states,hs,2)
        self.assertEqual(planned,checked)
        later=[e for e in checked["events"] if e["cell"]==2 and e["family"]==1]
        self.assertEqual([e["novel"] for e in later],[False,False]);self.assertEqual(checked["pairs"],[])

def synthetic(reacquire=False):
    """Hypothetical, typed decoder traces; never submitted to native-law admission."""
    template=copy.deepcopy(DATA[0]["cases"][0]["initial"]["cells"][0]["state"])
    def build(genes,ident=0,parent=None,generation=0):
        s=copy.deepcopy(template);c=s["core"];c.update(cell=ident,parent=parent,generation=generation,capital=64,initial=64,dissipated=0,transferred=0,environment=0,rewrites=0,next_serial=max(dict(genes))+1)
        wallet=handle(c["wallet"]);editor=handle(c["editor"])
        c["nodes"]=[{"id":wallet,"label":100,"born":0,"properties":[[1,64]]},{"id":editor,"label":300,"born":0,"properties":[[4,0],[5,1],[6,31]]}]+[{"id":(1<<32)|(serial+3),"label":200,"born":0,"properties":[[2,stage],[3,serial]]} for serial,stage in genes]
        c["edges"]=[{"id":(1<<32)|1,"label":1001,"nodes":[wallet,editor]}]+[{"id":(1<<32)|(serial+2),"label":1000,"nodes":[wallet,(1<<32)|(serial+3)]} for serial,_ in genes]
        c["genome"]={"stages":[x[1] for x in genes],"editor_mode":0};s["program"]={"genes":copy.deepcopy(genes),"next_serial":c["next_serial"],"enabled":1,"mask":31};return s
    current=build([[0,0]]);hs={0:{"initial":copy.deepcopy(current),"frames":[]}};states={}
    def edit(ident,family,target,stage=None):
        nonlocal current
        before=copy.deepcopy(current);genes=copy.deepcopy(before["program"]["genes"])
        if family==0:genes.append([before["program"]["next_serial"],stage])
        elif family==1:next(g for g in genes if g[0]==target)[1]=stage
        else:genes=[g for g in genes if g[0]!=target]
        after=build(genes,ident,before["core"]["parent"],before["core"]["generation"]);after["program"]["next_serial"]=after["core"]["next_serial"]=max(before["core"]["next_serial"],after["core"]["next_serial"])
        native=copy.deepcopy(after["core"]);native["capital"]=62;native["nodes"][0]["properties"]=[[1,62]]
        hs[ident]["frames"].append({"kind":"native_edit","receipts":[{"before":before["core"],"native_after":native,"extra":{"family":family,"target":target,"probe":True,"external":False,"diagnostic_restore":False}}],"after":after});current=after
    def use(ident,serial):
        hs[ident]["frames"].append({"kind":"resource_tick","receipts":[{"kind":"metabolic","before":copy.deepcopy(current["core"]),"bound":[current["core"]["wallet"],{"idx":serial+3,"version":1}]}]})
    def birth(ident,child):
        nonlocal current
        parent=copy.deepcopy(current);next_state=build(current["program"]["genes"],child,ident,current["core"]["generation"]+1)
        hs[ident]["frames"].append({"kind":"shared_birth","before":parent,"child":next_state,"birth":{"copy_cost":3+len(parent["program"]["genes"])}});states[ident]=parent;hs[child]={"initial":copy.deepcopy(next_state),"frames":[]};current=next_state
    edit(0,0,0,1);edit(0,0,1,2);edit(0,1,1,1);edit(0,0,1,1);edit(0,2,3);edit(0,0,0,1)
    birth(0,1);child_state=copy.deepcopy(current);current=states[0];edit(0,1,1,0);states[0]=copy.deepcopy(current);current=child_state
    use(1,4);use(1,2);birth(1,2);use(2,1);use(2,2)
    if reacquire:edit(2,1,1,0);edit(2,1,1,1)
    states[2]=copy.deepcopy(current);return states,hs

if __name__=="__main__":DATA=load(Path(sys.argv.pop(1)));unittest.main(verbosity=2)
