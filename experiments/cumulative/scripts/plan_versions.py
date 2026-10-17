"""Development-only versioned descent; freeze all measurement requests before assays."""
import copy
import json
import sys
from pathlib import Path
from audit_population import score
from audit_readiness import handle
from audit_regulation import project

def histories(case,tick):
    frame=case["initial"] if tick==0 else case["rows"][tick-1]["after"]
    states={m["id"]:m["state"] for m in frame["cells"]}
    hs={m["id"]:{"initial":m["state"],"frames":[]} for m in case["initial"]["cells"]}
    for row in case["rows"][:tick]:
        for op in row["operations"]:
            if op["phase"]=="newborn":hs[op["cell"]]={"initial":op["initial"],"frames":[]}
            hs[op["cell"]]["frames"]+=op["frames"]
    return frame,states,hs

def descent(states,hs,leaf):
    chain=[];i=leaf
    while i is not None:
        if i in chain:raise ValueError("version ancestry cycle")
        chain.append(i);i=states[i]["core"]["parent"]
    chain.reverse();root=chain[0];base=hs[root]["initial"]["program"]
    founder={serial:{"serial":serial,"entity":f"{root}:founder:{serial}","version":f"{root}:founder:{serial}","stage":stage,"acquisition":None,"origin":"founder","generation":0} for serial,stage in base["genes"]}
    founder_policy={"version":f"{root}:founder:editor","enabled":base["enabled"],"mask":base["mask"],"origin":"founder","generation":0}
    genes=copy.deepcopy(founder);policy=copy.deepcopy(founder_policy);seen={g[1] for g in base["genes"]};events=[];tombstones=[];births=[];activity=[]
    for position,ident in enumerate(chain):
        h=hs[ident];generation=h["initial"]["core"]["generation"];next_child=chain[position+1] if position+1<len(chain) else None;cut=False
        if [[s,g["stage"]] for s,g in sorted(genes.items())]!=h["initial"]["program"]["genes"]:raise ValueError("version import")
        for index,f in enumerate(h["frames"]):
            if f["kind"]=="native_edit":
                r=f["receipts"][0];family,target=r["extra"]["family"],r["extra"]["target"];event_id=f"{ident}:edit:{index}"
                origin="authored" if r["extra"]["probe"] or r["extra"]["diagnostic_restore"] else "external" if r["extra"]["external"] else "endogenous" if r["before"]["arm"]=="endogenous" else "nonheritable"
                before={"genes":[[s,g["stage"]] for s,g in sorted(genes.items())],"enabled":policy["enabled"],"mask":policy["mask"]}
                after=f["after"]["program"];novel=False;stage=None;prior=None
                if family in [0,1]:
                    old=genes[target];serial=r["before"]["next_serial"] if family==0 else target;stage=dict(after["genes"])[serial];prior=old["version"]
                    token=copy.deepcopy(old);token.update(serial=serial,version=event_id,stage=stage)
                    if family==0:token["entity"]=f"{ident}:gene:{serial}"
                    if stage!=old["stage"]:
                        token.update(acquisition=event_id,origin=origin,generation=generation);novel=0<=stage<4 and stage not in seen
                    seen.add(stage);genes[serial]=token
                elif family==2:
                    old=genes.pop(target);prior=old["version"];tombstones.append({**old,"deleted_by":event_id,"cell":ident,"generation":generation})
                else:
                    prior=policy["version"];policy={"version":event_id,"enabled":after["enabled"],"mask":after["mask"],"origin":origin,"generation":generation}
                events.append({"id":event_id,"cell":ident,"generation":generation,"frame":index,"family":family,"target":target,"origin":origin,"previous_version":prior,"stage":stage,"novel":novel,"before":before,"after":{"genes":after["genes"],"enabled":after["enabled"],"mask":after["mask"]},"edit_cost":2})
            elif f["kind"]=="resource_tick":
                for receipt_index,r in enumerate(f["receipts"]):
                    if r["kind"]=="metabolic":
                        gid=handle(r["bound"][1]);props=dict(next(n["properties"] for n in r["before"]["nodes"] if n["id"]==gid));g=genes[props[3]]
                        activity.append({"cell":ident,"generation":generation,"frame":index,"receipt":receipt_index,"serial":g["serial"],"entity":g["entity"],"version":g["version"],"acquisition":g["acquisition"],"stage":g["stage"]})
            elif f["kind"]=="shared_birth" and f["child"]["core"]["cell"]==next_child:
                if states[ident]["core"]["arm"]=="nonheritable":genes=copy.deepcopy(founder);policy=copy.deepcopy(founder_policy)
                births.append({"parent":ident,"child":next_child,"frame":index,"generation":f["child"]["core"]["generation"],"copy_cost":f["birth"]["copy_cost"],"tokens":[genes[s] for s in sorted(genes)],"policy":copy.deepcopy(policy)})
                cut=True;break
        if next_child is not None and not cut:raise ValueError("version missing paid cutoff")
    endpoint=states[leaf]["program"]
    if [[s,g["stage"]] for s,g in sorted(genes.items())]!=endpoint["genes"] or (policy["enabled"],policy["mask"])!=(endpoint["enabled"],endpoint["mask"]):raise ValueError("version endpoint")
    retained={g["acquisition"] for g in genes.values()};novel=[e for e in events if e["novel"] and e["id"] in retained];stages=[g["stage"] for _,g in sorted(genes.items())];pairs=[]
    for i,e in enumerate(novel):
        for later in novel[i+1:]:
            a,b=e["stage"],later["stage"];single=stages.copy();single.remove(a)
            programs={"earlier_before":[g[1] for g in e["before"]["genes"]],"earlier_after":[g[1] for g in e["after"]["genes"]],"later_before":[g[1] for g in later["before"]["genes"]],"later_after":[g[1] for g in later["after"]["genes"]],"intact":stages,"remove_earlier":[s for s in stages if s!=a],"remove_later":[s for s in stages if s!=b],"padded_remove_earlier":[-1 if s==a else s for s in stages],"single_remove_earlier":single,"sham":stages.copy(),"restored":stages.copy()}
            pairs.append({"earlier":e["id"],"later":later["id"],"origins":[e["origin"],later["origin"]],"transmissions":[sum(any(t["acquisition"]==ev["id"] for t in edge["tokens"]) for edge in births) for ev in [e,later]],"active_descendant_generations":[sorted({u["generation"] for u in activity if u["generation"]>ev["generation"] and u["acquisition"]==ev["id"]}) for ev in [e,later]],"programs":programs})
    return {"chain":chain,"tokens":[genes[s] for s in sorted(genes)],"policy":policy,"events":events,"tombstones":tombstones,"births":births,"activity":activity,"pairs":pairs}

def plan(world,contract):
    samples=[];requests=[];registry={}
    def register(role,stages):
        if not stages:return None
        key=(role,tuple(stages))
        if key not in registry:registry[key]=len(requests);requests.append({"id":len(requests),"role":role,"stages":stages})
        return registry[key]
    for case in world["cases"]:
        for tick in contract["sampling"]["census_ticks"]:
            if tick>case["final"]["tick"]:
                samples.extend({"source":case["name"],"census":tick,"slot":slot,"status":"outside_horizon","ranking":[]} for slot in range(contract["sampling"]["slots"]));continue
            frame,states,hs=histories(case,tick);rank=sorted([[i,score(frame["seed"],tick,2,i)] for i in states],key=lambda r:(r[1],r[0]))
            for slot in range(contract["sampling"]["slots"]):
                sample={"source":case["name"],"census":tick,"slot":slot,"ranking":rank}
                if slot>=len(rank):sample["status"]="empty"
                else:
                    leaf=rank[slot][0];d=descent(states,hs,leaf);stages=[g[1] for g in states[leaf]["program"]["genes"]]
                    sample.update(cell=leaf,status="living" if states[leaf]["alive"] else "dead",lineage=d,endpoint_assays={role:register(role,stages) for role in contract["assay"]["endpoint_roles"]})
                    if len(d["pairs"])>contract["limits"]["pairs_per_sample"]:raise ValueError("version pair cap")
                    for pair in d["pairs"]:
                        pair["assays"]={role:register(role,pair["programs"][role]) for role in contract["assay"]["pair_roles"]};pair["inadmissible_empty_ablation"]=any(x is None for x in pair["assays"].values());del pair["programs"]
                samples.append(sample)
    if len(samples)!=contract["sampling"]["samples"] or len(requests)>contract["limits"]["assays"]:raise ValueError("complete integration sample/request cap")
    return {"profile":contract["profile"],"samples":samples},{"profile":contract["assay"]["native_profile"],"assays":requests}

def evaluate(measurement,results):
    lookup={r["request"]["id"]:r for r in results["results"]};rows=[]
    for index,sample in enumerate(measurement["samples"]):
        if "lineage" not in sample:rows.append({"sample":index,"status":sample["status"],"pairs":[]});continue
        endpoint={role:{"score":lookup[i]["score"],"status":lookup[i]["status"]} for role,i in sample["endpoint_assays"].items()};pairs=[]
        for pair in sample["lineage"]["pairs"]:
            values={role:None if i is None else lookup[i]["score"] for role,i in pair["assays"].items()};funded=all(i is not None and lookup[i]["status"]=="complete" for i in pair["assays"].values())
            marginal=[values["earlier_after"]-values["earlier_before"]-2,values["later_after"]-values["later_before"]-2]
            useful=funded and all(m>0 for m in marginal) and values["intact"]>values["remove_earlier"] and values["intact"]>values["remove_later"] and values["intact"]>values["padded_remove_earlier"] and values["intact"]==values["sham"]==values["restored"]
            durable=all(n>=2 for n in pair["transmissions"]) and all(len(g)>=2 for g in pair["active_descendant_generations"])
            pairs.append({"earlier":pair["earlier"],"later":pair["later"],"scores":values,"marginal_paid_benefits":marginal,"all_comparisons_funded":funded,"causally_useful":useful,"durable":durable,"autonomous_origin":pair["origins"]==["endogenous","endogenous"],"discovery_candidate_pass":useful and durable and pair["origins"]==["endogenous","endogenous"]})
        rows.append({"sample":index,"status":sample["status"],"endpoint":endpoint,"pairs":pairs})
    return {"profile":measurement["profile"],"rows":rows,**{k:False for k in ["readiness_gate_complete","scientific_acceptance","evolution_demonstrated","campaign_launch_authorized"]}}

if __name__=="__main__":
    if sys.argv[1]=="aggregate":
        values=[json.loads(Path(p).read_text()) for p in sys.argv[2:4]]
        with Path(sys.argv[4]).open("x") as f:json.dump(evaluate(*values),f,indent=2);f.write("\n")
    else:
        world,contract=[json.loads(Path(p).read_text()) for p in sys.argv[1:3]]
        for path,value in zip(sys.argv[3:5],plan(world,contract)):
            with Path(path).open("x") as f:json.dump(value,f,indent=2);f.write("\n")
