"""Build a bounded measurement request before seeing any native assay outcomes."""
import copy
import json
import sys
from pathlib import Path
from audit_population import score


def descent(states, histories, leaf):
    chain=[];ident=leaf
    while ident is not None:
        if ident in chain:raise ValueError("ancestry cycle")
        chain.append(ident);ident=states[ident]["parent"]
    chain.reverse()
    founder=histories[chain[0]]["initial"]["genome"]["stages"]
    baseline=[{"id":f"{chain[0]}:founder:{i}","stage":s,"origin":"founder"} for i,s in enumerate(founder)]
    tokens=copy.deepcopy(baseline);events=[];births=[];activity=[]
    for position,ident in enumerate(chain):
        h=histories[ident]
        assert [t["stage"] for t in tokens]==h["initial"]["genome"]["stages"]
        next_child=chain[position+1] if position+1<len(chain) else None
        reached=False
        for index,r in enumerate(h["receipts"]):
            kind=r["kind"]
            if kind in ["endogenous_edit","external_edit"]:
                stages=r["native_after"]["genome"]["stages"];s=stages[-1]
                origin="authored" if r["extra"]["probe"] or r["before"]["genome"]["editor_mode"]==1 else "external" if kind=="external_edit" else "endogenous" if r["before"]["arm"]=="endogenous" else "nonheritable"
                token={"id":f"{ident}:edit:{index}","stage":s,"origin":origin}
                events.append({**token,"cell":ident,"generation":h["initial"]["generation"],"receipt":index,
                    "novel":0<=s<4 and s not in founder and s not in [t["stage"] for t in tokens],
                    "before":r["before"]["genome"]["stages"],"after":stages,"edit_cost":2})
                tokens.append(token)
            elif kind=="metabolic":
                t=tokens[r["extra"]["serial"]]
                activity.append({"cell":ident,"generation":h["initial"]["generation"],"token":t["id"],"stage":t["stage"]})
            elif kind in ["birth","shared_birth"] and r["child"]["cell"]==next_child:
                if r["before"]["arm"]=="nonheritable":tokens=copy.deepcopy(baseline)
                births.append({"parent":ident,"child":next_child,"generation":r["child"]["generation"],
                    "copy_cost":r.get("copy_cost",r.get("birth",{}).get("copy_cost")),
                    "environment":r["child"]["environment"],"tokens":copy.deepcopy(tokens)})
                reached=True;break
        if next_child is not None and not reached:raise ValueError("missing paid path edge")
    if [t["stage"] for t in tokens]!=states[leaf]["genome"]["stages"]:raise ValueError("endpoint gene projection")
    retained={t["id"] for t in tokens}
    novel=[e for e in events if e["novel"] and e["id"] in retained]
    pairs=[]
    for i,earlier in enumerate(novel):
        for later in novel[i+1:]:
            stages=[t["stage"] for t in tokens];a,b=earlier["stage"],later["stage"]
            erased=[s for s in stages if s!=a]
            roles={"earlier_before":earlier["before"],"earlier_after":earlier["after"],
                "later_before":later["before"],"later_after":later["after"],"intact":stages,
                "remove_earlier":erased,"remove_later":[s for s in stages if s!=b],
                "padded_remove_earlier":[-1 if s==a else s for s in stages],
                "single_remove_earlier":stages[:stages.index(a)]+stages[stages.index(a)+1:],
                "sham":stages.copy(),"restored":[s for s in stages]}
            transmissions=[sum(any(t["id"]==e["id"] for t in birth["tokens"]) for birth in births) for e in [earlier,later]]
            active=[sorted({r["generation"] for r in activity if r["stage"]==e["stage"] and r["generation"]>e["generation"]}) for e in [earlier,later]]
            pairs.append({"earlier":earlier["id"],"later":later["id"],"origins":[earlier["origin"],later["origin"]],
                "transmissions":transmissions,"active_descendant_generations":active,"programs":roles})
    if len(pairs)>28:raise ValueError("complete pair cap exceeded")
    return {"chain":chain,"tokens":tokens,"events":events,"births":births,"activity":activity,"pairs":pairs}


def plan(shared,witness,contract):
    samples=[]
    for case in shared["cases"]:
        initial=case["initial"]
        for tick in contract["sampling"]["census_ticks"]:
            if tick>case["final"]["tick"]:
                for slot in range(2):samples.append({"source":case["name"],"census":tick,"slot":slot,"status":"outside_horizon","ranking":[]})
                continue
            frame=initial if tick==0 else case["rows"][tick-1]["after"]
            states={m["state"]["cell"]:m["state"] for m in frame["cells"]}
            metadata={m["state"]["cell"]:m for m in frame["cells"]}
            histories={m["state"]["cell"]:{"initial":m["state"],"receipts":[]} for m in initial["cells"]}
            for row in case["rows"][:tick]:
                for op in row["operations"]:
                    for r in op["receipts"]:
                        if r["kind"]=="shared_birth":histories[r["child"]["cell"]]={"initial":r["child"],"receipts":[]}
                    histories[op["cell"]]["receipts"]+=op["receipts"]
            ranking=sorted([[i,score(initial["seed"],tick,2,i)] for i in states],key=lambda p:(p[1],p[0]))
            for slot in range(2):
                sample={"source":case["name"],"census":tick,"slot":slot,"ranking":ranking}
                if slot>=len(ranking):sample["status"]="empty"
                else:
                    leaf=ranking[slot][0];sample.update(cell=leaf,status="dead" if metadata[leaf]["site"] is None else "living",
                        lineage=descent(states,histories,leaf))
                samples.append(sample)
    for control in witness["controls"]:
        histories={g["initial"]["cell"]:g for g in control["generations"]}
        states={i:g["final"] for i,g in histories.items()}
        samples.append({"source":control["name"],"census":3,"slot":0,"status":"authored_control","cell":3,
            "ranking":[],"lineage":descent(states,histories,3)})
    registry={};requests=[]
    for sample in samples:
        for pair in sample.get("lineage",{}).get("pairs",[]):
            pair["assays"]={}
            for role in contract["assay"]["roles"]:
                stages=pair["programs"][role];key=(role,tuple(stages))
                if key not in registry:
                    registry[key]=len(requests);requests.append({"id":len(requests),"role":role,"stages":stages})
                pair["assays"][role]=registry[key]
            del pair["programs"]
    if len(samples)!=75 or len(requests)>contract["limits"]["assays"]:raise ValueError("sample/assay cap")
    return {"profile":contract["profile"],"samples":samples}, {"profile":contract["profile"],"assays":requests}


def evaluate(plan,results):
    scores={r["request"]["id"]:r["score"] for r in results["results"]};statuses={r["request"]["id"]:r["status"] for r in results["results"]};rows=[]
    for index,sample in enumerate(plan["samples"]):
        pairs=[]
        for p in sample.get("lineage",{}).get("pairs",[]):
            v={role:scores[i] for role,i in p["assays"].items()}
            marginal=[v["earlier_after"]-v["earlier_before"]-2,v["later_after"]-v["later_before"]-2]
            funded=all(statuses[i]=="complete" for i in p["assays"].values())
            useful=funded and all(x>0 for x in marginal) and v["intact"]>v["remove_earlier"] and v["intact"]>v["remove_later"] and v["intact"]>v["padded_remove_earlier"] and v["intact"]==v["sham"]==v["restored"]
            durable=all(n>=2 for n in p["transmissions"]) and all(len(g)>=2 for g in p["active_descendant_generations"])
            pairs.append({"earlier":p["earlier"],"later":p["later"],"scores":v,"marginal_paid_benefits":marginal,
                "all_comparisons_funded":funded,"causally_useful":useful,"durable":durable,"functional_cumulative_witness":useful and durable,
                "autonomous_origin":p["origins"]==["endogenous","endogenous"],
                "discovery_candidate_pass":useful and durable and p["origins"]==["endogenous","endogenous"]})
        rows.append({"sample":index,"pairs":pairs})
    return {"profile":plan["profile"],"rows":rows,**{k:False for k in ["readiness_gate_complete","scientific_acceptance","evolution_demonstrated","campaign_launch_authorized"]}}


if __name__=="__main__":
    if sys.argv[1]=="aggregate":
        measurement,results=[json.loads(Path(p).read_text()) for p in sys.argv[2:4]]
        with Path(sys.argv[4]).open("x") as f:json.dump(evaluate(measurement,results),f,indent=2);f.write("\n")
        raise SystemExit(0)
    shared,witness,contract=[json.loads(Path(p).read_text()) for p in sys.argv[1:4]]
    measurement,requests=plan(shared,witness,contract)
    for path,value in zip(sys.argv[4:6],[measurement,requests]):
        with Path(path).open("x") as f:json.dump(value,f,indent=2);f.write("\n")
