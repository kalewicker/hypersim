"""Independent versioned gene/policy descent and frozen native assay audit.
Does not import the measurement planner. Shared law and cell law are checked first.
"""
import copy
import json
import sys
from pathlib import Path
from audit_readiness import require,trace,view,handle,FALSE_FLAGS
from audit_population import score
from audit_regulation import project
from audit_world import audit_world,PROFILE

def reconstruction(states,records,leaf):
    chain=[leaf]
    while states[chain[0]]["core"]["parent"] is not None:
        p=states[chain[0]]["core"]["parent"];require(p not in chain,"version ancestry cycle");chain.insert(0,p)
    root=chain[0];first=records[root]["initial"]["program"]
    baseline={}
    for serial,stage in first["genes"]:
        name=str(root)+":founder:"+str(serial)
        baseline[serial]={"serial":serial,"entity":name,"version":name,"stage":stage,"acquisition":None,"origin":"founder","generation":0}
    original_policy={"version":str(root)+":founder:editor","enabled":first["enabled"],"mask":first["mask"],"origin":"founder","generation":0}
    tokens=copy.deepcopy(baseline);policy=copy.deepcopy(original_policy);ever=set(stage for _,stage in first["genes"]);events=[];deleted=[];edges=[];uses=[]
    for position,ident in enumerate(chain):
        h=records[ident];generation=h["initial"]["core"]["generation"]
        require([[i,t["stage"]] for i,t in sorted(tokens.items())]==h["initial"]["program"]["genes"] and [policy["enabled"],policy["mask"]]==[h["initial"]["program"]["enabled"],h["initial"]["program"]["mask"]],"complete gene and policy import")
        child=chain[position+1] if position+1<len(chain) else None;frames=h["frames"]
        if child is not None:
            cuts=[i for i,f in enumerate(frames) if f["kind"]=="shared_birth" and f["child"]["core"]["cell"]==child]
            require(len(cuts)==1,"actual paid version cutoff");frames=frames[:cuts[0]+1]
        for index,f in enumerate(frames):
            if f["kind"]=="native_edit":
                r=f["receipts"][0];family,target=r["extra"]["family"],r["extra"]["target"];name=str(ident)+":edit:"+str(index)
                origin="authored" if r["extra"]["probe"] or r["extra"]["diagnostic_restore"] else "external" if r["extra"]["external"] else "endogenous" if r["before"]["arm"]=="endogenous" else "nonheritable"
                source=view(project(r["before"]))[1];dest=view(project(r["native_after"]),balance=False,transient=family==0)[1]
                before={"genes":[[s,g["stage"]] for s,g in sorted(tokens.items())],"enabled":policy["enabled"],"mask":policy["mask"]}
                require(before["genes"]==[[g[0],g[2]] for g in source],"gene versions before native edit")
                editor=handle(r["native_after"]["editor"]);props=dict(next(n["properties"] for n in r["native_after"]["nodes"] if n["id"]==editor));after={"genes":[[g[0],g[2]] for g in dest],"enabled":props[5],"mask":props[6]}
                novel=False;stage=None;previous=None
                if family==2:
                    require(target in tokens,"deleted version exists");old=tokens.pop(target);previous=old["version"];deleted.append({**old,"deleted_by":name,"cell":ident,"generation":generation})
                elif family in [3,4]:
                    previous=policy["version"];policy={"version":name,"enabled":after["enabled"],"mask":after["mask"],"origin":origin,"generation":generation}
                else:
                    original=tokens[target];serial=r["before"]["next_serial"] if family==0 else target;stage=next(g[2] for g in dest if g[0]==serial);previous=original["version"]
                    token={**original,"serial":serial,"version":name,"stage":stage}
                    if family==0:token["entity"]=str(ident)+":gene:"+str(serial)
                    if stage!=original["stage"]:
                        token["acquisition"]=name;token["origin"]=origin;token["generation"]=generation
                        novel=stage in range(4) and stage not in ever
                    ever.add(stage);tokens[serial]=token
                paid=r["before"]["capital"]-r["native_after"]["capital"];require(paid==2,"version acquisition payment")
                events.append({"id":name,"cell":ident,"generation":generation,"frame":index,"family":family,"target":target,"origin":origin,"previous_version":previous,"stage":stage,"novel":novel,"before":before,"after":after,"edit_cost":paid})
                require([[i,t["stage"]] for i,t in sorted(tokens.items())]==after["genes"] and [policy["enabled"],policy["mask"]]==[props[5],props[6]],"exact native version endpoint")
            elif f["kind"]=="resource_tick":
                for k,r in enumerate(f["receipts"]):
                    if r["kind"]!="metabolic":continue
                    target=handle(r["bound"][1]);serial,_,stage=next(g for g in view(project(r["before"]))[1] if g[1]==target);t=tokens[serial]
                    require(t["stage"]==stage,"exact metabolic version activity")
                    uses.append({"cell":ident,"generation":generation,"frame":index,"receipt":k,"serial":serial,"entity":t["entity"],"version":t["version"],"acquisition":t["acquisition"],"stage":stage})
            elif f["kind"]=="shared_birth" and f["child"]["core"]["cell"]==child:
                if f["before"]["core"]["arm"]=="nonheritable":tokens=copy.deepcopy(baseline);policy=copy.deepcopy(original_policy)
                require([[i,t["stage"]] for i,t in sorted(tokens.items())]==f["child"]["program"]["genes"] and [policy["enabled"],policy["mask"]]==[f["child"]["program"]["enabled"],f["child"]["program"]["mask"]],"actual paid full version transmission")
                edges.append({"parent":ident,"child":child,"frame":index,"generation":f["child"]["core"]["generation"],"copy_cost":f["birth"]["copy_cost"],"tokens":[tokens[s] for s in sorted(tokens)],"policy":copy.deepcopy(policy)})
    endpoint=states[leaf]["program"];require([[i,t["stage"]] for i,t in sorted(tokens.items())]==endpoint["genes"] and [policy["enabled"],policy["mask"]]==[endpoint["enabled"],endpoint["mask"]],"sampled full version endpoint")
    alive_ids={t["acquisition"] for t in tokens.values()};retained=[e for e in events if e["novel"] and e["id"] in alive_ids];pairs=[];stages=[tokens[s]["stage"] for s in sorted(tokens)]
    for a_index in range(len(retained)):
        for b_index in range(a_index+1,len(retained)):
            a,b=retained[a_index],retained[b_index];sa,sb=a["stage"],b["stage"];positions=[i for i,s in enumerate(stages) if s==sa];single=[s for i,s in enumerate(stages) if i!=positions[0]]
            programs={"earlier_before":[s for _,s in a["before"]["genes"]],"earlier_after":[s for _,s in a["after"]["genes"]],"later_before":[s for _,s in b["before"]["genes"]],"later_after":[s for _,s in b["after"]["genes"]],"intact":stages[:],"remove_earlier":[s for s in stages if s!=sa],"remove_later":[s for s in stages if s!=sb],"padded_remove_earlier":[s if s!=sa else -1 for s in stages],"single_remove_earlier":single,"sham":stages[:],"restored":stages[:]}
            pairs.append({"earlier":a["id"],"later":b["id"],"origins":[a["origin"],b["origin"]],"transmissions":[sum(e["id"] in {t["acquisition"] for t in edge["tokens"]} for edge in edges) for e in [a,b]],"active_descendant_generations":[sorted({u["generation"] for u in uses if u["acquisition"]==e["id"] and u["generation"]>e["generation"]}) for e in [a,b]],"programs":programs})
    return {"chain":chain,"tokens":[tokens[s] for s in sorted(tokens)],"policy":policy,"events":events,"tombstones":deleted,"births":edges,"activity":uses,"pairs":pairs}

def expected_measurement(world,contract):
    samples=[];registry={};requests=[]
    def request(role,stages):
        if len(stages)==0:return None
        key=(role,tuple(stages))
        if key not in registry:registry[key]=len(requests);requests.append({"id":len(requests),"role":role,"stages":stages})
        return registry[key]
    for case in world["cases"]:
        for tick in contract["sampling"]["census_ticks"]:
            if tick>case["final"]["tick"]:
                for slot in range(2):samples.append({"source":case["name"],"census":tick,"slot":slot,"status":"outside_horizon","ranking":[]})
                continue
            snapshot=case["initial"] if tick==0 else next(r["after"] for r in case["rows"] if r["after"]["tick"]==tick)
            states={m["id"]:m["state"] for m in snapshot["cells"]};records={m["id"]:{"initial":m["state"],"frames":[]} for m in case["initial"]["cells"]}
            for row in case["rows"]:
                if row["after"]["tick"]>tick:break
                for op in row["operations"]:
                    if op["phase"]=="newborn":records[op["cell"]]={"initial":op["initial"],"frames":[]}
                    records[op["cell"]]["frames"].extend(op["frames"])
            ranks=sorted([[i,score(snapshot["seed"],tick,2,i)] for i in states],key=lambda r:(r[1],r[0]))
            for slot in range(2):
                entry={"source":case["name"],"census":tick,"slot":slot,"ranking":ranks}
                if slot>=len(ranks):entry["status"]="empty"
                else:
                    i=ranks[slot][0];lineage=reconstruction(states,records,i);stages=[s for _,s in states[i]["program"]["genes"]]
                    entry.update(cell=i,status="living" if states[i]["alive"] else "dead",lineage=lineage,endpoint_assays={role:request(role,stages) for role in contract["assay"]["endpoint_roles"]})
                    require(len(lineage["pairs"])<=28,"full version pair denominator")
                    for p in lineage["pairs"]:
                        indices={role:request(role,p["programs"][role]) for role in contract["assay"]["pair_roles"]};del p["programs"];p.update(assays=indices,inadmissible_empty_ablation=any(i is None for i in indices.values()))
                samples.append(entry)
    return {"profile":PROFILE,"samples":samples},{"profile":contract["assay"]["native_profile"],"assays":requests}

def audit(world,measurement,requests,results,report,contract):
    prior=audit_world(world,contract)
    require(measurement["profile"]==report["profile"]==PROFILE and results["profile"]==requests["profile"]==contract["assay"]["native_profile"],"version measurement profiles")
    for d in [results,report]:
        for f in FALSE_FLAGS:require(d[f] is False,"version assay admission "+f)
    expected,inputs=expected_measurement(world,contract)
    require(measurement==expected and requests==inputs and len(expected["samples"])==96 and len(inputs["assays"])<=128,"independent complete version sampling and requests")
    require([r["request"] for r in results["results"]]==requests["assays"],"all native endpoint/pair assays")
    work=0;lookup={}
    for result,r in zip(results["results"],requests["assays"]):
        initial,final=result["initial"],result["final"]
        require([g[2] for g in view(initial)[1]]==r["stages"] and initial["arm"]=="frozen" and initial["genome"]["editor_mode"]==0 and initial["capital"]==40 and initial["environment"]==512 and initial["tick"]==0 and initial["seed"]==2026100903,"unchanged standardized frozen assay context")
        n,births=trace(initial,result["receipts"],final);work+=n
        require(final["tick"]==4 and len(births)<=1 and all(r["kind"] not in ["endogenous_edit","external_edit","selection"] for r in result["receipts"]),"full assay horizon without mutation")
        cost=3+len(r["stages"])
        require(result["status"]==("complete" if births else "unfunded_copy") and (bool(births) or final["capital"]<cost+8),"retained copy status")
        if births:require(births[0]["genome"]==initial["genome"],"paid diagnostic assay transmission")
        require(result["score"]==final["capital"]+final["transferred"]-40,"no assay environment/complexity score bonus");lookup[r["id"]]=result
    require(work==results["native_rewrites"],"assay native work")
    rows=[]
    for index,s in enumerate(expected["samples"]):
        if "lineage" not in s:rows.append({"sample":index,"status":s["status"],"pairs":[]});continue
        endpoint={role:{"score":lookup[i]["score"],"status":lookup[i]["status"]} for role,i in s["endpoint_assays"].items()};pairs=[]
        require(endpoint["intact"]==endpoint["sham"]==endpoint["restored"],"endpoint sham/restoration parity")
        for p in s["lineage"]["pairs"]:
            scores={role:lookup[i]["score"] if i is not None else None for role,i in p["assays"].items()};funded=all(i is not None and lookup[i]["status"]=="complete" for i in p["assays"].values())
            marginal=[scores["earlier_after"]-scores["earlier_before"]-2,scores["later_after"]-scores["later_before"]-2]
            useful=funded and min(marginal)>0 and all(scores["intact"]>scores[role] for role in ["remove_earlier","remove_later","padded_remove_earlier"]) and scores["intact"]==scores["sham"]==scores["restored"]
            durable=min(p["transmissions"])>=2 and all(len(g)>=2 for g in p["active_descendant_generations"]);origin=p["origins"]==["endogenous","endogenous"]
            pairs.append({"earlier":p["earlier"],"later":p["later"],"scores":scores,"marginal_paid_benefits":marginal,"all_comparisons_funded":funded,"causally_useful":useful,"durable":durable,"autonomous_origin":origin,"discovery_candidate_pass":useful and durable and origin})
        rows.append({"sample":index,"status":s["status"],"endpoint":endpoint,"pairs":pairs})
    require(report=={"profile":PROFILE,"rows":rows,**{f:False for f in FALSE_FLAGS}},"versioned assay aggregate")
    total=prior["native_rewrites"]+work;require(total<=contract["limits"]["native_rewrites"],"integration fixture and assays work cap")
    return {"profile":PROFILE,"engineering_fixture_pass":True,"world_native_rewrites":prior["native_rewrites"],"assay_native_rewrites":work,"native_rewrites":total,"worlds":len(world["cases"]),"samples":len(expected["samples"]),"assays":len(requests["assays"]),"living_samples":sum(s["status"]=="living" for s in expected["samples"]),"dead_samples":sum(s["status"]=="dead" for s in expected["samples"]),"empty_slots":sum(s["status"]=="empty" for s in expected["samples"]),"outside_horizon_slots":sum(s["status"]=="outside_horizon" for s in expected["samples"]),"retained_pairs":sum(len(s.get("lineage",{}).get("pairs",[])) for s in expected["samples"]),"discovery_candidate_passes":sum(p["discovery_candidate_pass"] for r in rows for p in r["pairs"]),"independent_shared_world_and_versioned_lineage":True,"independent_rng_replay":False,"independent_allocator_replay":False,**{f:False for f in FALSE_FLAGS}}

def load(package):return [json.loads((package/name).read_text()) for name in ["fixture.json","measurement.json","requests.json","assays.json","report.json","contract.json"]]
if __name__=="__main__":print(json.dumps(audit(*load(Path(sys.argv[1]))),indent=2,sort_keys=True))
