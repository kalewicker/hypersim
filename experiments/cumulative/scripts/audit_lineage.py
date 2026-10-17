"""Independently reconstruct sampling, paid ancestry, acquired organization and assays.
Does not import the measurement planner. Existing native law auditors are reused.
"""
import copy
import hashlib
import json
import sys
from pathlib import Path
from audit_readiness import require, trace, view, handle, FALSE_FLAGS
from audit_population import audit as audit_population

PROFILE="cumulative-lineage-intervention-readiness-v1"


def keyed(seed,tick,ident):
    mask=2**64-1
    x=(seed ^ ((tick*0x9e3779b97f4a7c15)&mask) ^ ((2*0xd1b54a32d192ed03)&mask) ^ ((ident*0x94d049bb133111eb)&mask))&mask
    x=(x+0x9e3779b97f4a7c15)&mask
    for shift,mult in [(30,0xbf58476d1ce4e5b9),(27,0x94d049bb133111eb)]:x=((x^(x>>shift))*mult)&mask
    return x^(x>>31)


def graph_stages(state):
    return [g[2] for g in view(state)[1]]


def reconstruct(states,histories,leaf):
    chain=[leaf]
    while states[chain[0]]["parent"] is not None:
        p=states[chain[0]]["parent"]
        require(p not in chain,"ancestry cycle")
        chain.insert(0,p)
    root=chain[0];founder=graph_stages(histories[root]["initial"])
    baseline=[{"id":str(root)+":founder:"+str(i),"stage":s,"origin":"founder"} for i,s in enumerate(founder)]
    genes=copy.deepcopy(baseline);events=[];transfers=[];use=[]
    for n,ident in enumerate(chain):
        h=histories[ident];generation=h["initial"]["generation"]
        require([g["stage"] for g in genes]==graph_stages(h["initial"]),"imported gene identity")
        next_id=chain[n+1] if n+1<len(chain) else None
        records=h["receipts"]
        if next_id is not None:
            cuts=[k for k,r in enumerate(records) if r["kind"] in ["birth","shared_birth"] and r["child"]["cell"]==next_id]
            require(len(cuts)==1,"paid path birth denominator")
            records=records[:cuts[0]+1]
        for k,r in enumerate(records):
            kind=r["kind"]
            if kind in ["endogenous_edit","external_edit"]:
                created=handle(r["created"][0]);nodes={p["id"]:p for p in r["native_after"]["nodes"]}
                props=dict(nodes[created]["properties"]);s=props[2]
                require(props[3]==len(genes),"new gene serial lineage mapping")
                origin="authored" if r["extra"]["probe"] or r["before"]["genome"]["editor_mode"]==1 else "external" if kind=="external_edit" else "endogenous" if r["before"]["arm"]=="endogenous" else "nonheritable"
                token={"id":str(ident)+":edit:"+str(k),"stage":s,"origin":origin}
                before=graph_stages(r["before"])
                require(before==[g["stage"] for g in genes],"pre-edit retained organization")
                event={**token,"cell":ident,"generation":generation,"receipt":k,
                    "novel":s in range(4) and s not in set(founder) and all(g["stage"]!=s for g in genes),
                    "before":before,"after":before+[s],"edit_cost":r["before"]["capital"]-r["native_after"]["capital"]}
                require(event["edit_cost"]==2,"paid acquisition")
                events.append(event);genes.append(token)
            if kind=="metabolic":
                bound_gene=handle(r["bound"][1])
                matched=next(g for g in view(r["before"])[1] if g[1]==bound_gene)
                token=genes[matched[0]]
                require(token["stage"]==matched[2],"active gene lineage identity")
                use.append({"cell":ident,"generation":generation,"token":token["id"],"stage":matched[2]})
            if kind in ["birth","shared_birth"] and r["child"]["cell"]==next_id:
                if r["before"]["arm"]=="nonheritable":genes=copy.deepcopy(baseline)
                require([g["stage"] for g in genes]==graph_stages(r["child"]),"actual transmitted organization")
                cost=r["copy_cost"] if kind=="birth" else r["birth"]["copy_cost"]
                transfers.append({"parent":ident,"child":next_id,"generation":r["child"]["generation"],
                    "copy_cost":cost,"environment":r["child"]["environment"],"tokens":copy.deepcopy(genes)})
    require(graph_stages(states[leaf])==[g["stage"] for g in genes],"sample gene endpoint")
    ids={g["id"] for g in genes};novel=[e for e in events if e["novel"] and e["id"] in ids]
    pairs=[];endpoint=[g["stage"] for g in genes]
    for first in range(len(novel)):
        for second in range(first+1,len(novel)):
            e,l=novel[first],novel[second];a,b=e["stage"],l["stage"]
            keep=[(i,s) for i,s in enumerate(endpoint) if s!=a];removed=[(i,s) for i,s in enumerate(endpoint) if s==a]
            restored=[s for _,s in sorted(keep+removed)]
            single=endpoint.copy();single.pop(next(i for i,s in enumerate(endpoint) if s==a))
            programs={"earlier_before":e["before"],"earlier_after":e["after"],"later_before":l["before"],"later_after":l["after"],
                "intact":endpoint,"remove_earlier":[s for _,s in keep],"remove_later":[s for s in endpoint if s!=b],
                "padded_remove_earlier":[s if s!=a else -1 for s in endpoint],"single_remove_earlier":single,
                "sham":endpoint[:],"restored":restored}
            counts=[];generations=[]
            for event in [e,l]:
                counts.append(sum(event["id"] in {t["id"] for t in edge["tokens"]} for edge in transfers))
                generations.append(sorted(set(u["generation"] for u in use if u["generation"]>event["generation"] and u["stage"]==event["stage"])))
            pairs.append({"earlier":e["id"],"later":l["id"],"origins":[e["origin"],l["origin"]],
                "transmissions":counts,"active_descendant_generations":generations,"programs":programs})
    require(len(pairs)<=28,"complete pair cap")
    return {"chain":chain,"tokens":genes,"events":events,"births":transfers,"activity":use,"pairs":pairs}


def samples(shared,witness,contract):
    output=[]
    for case in shared["cases"]:
        for tick in contract["sampling"]["census_ticks"]:
            if tick>case["final"]["tick"]:
                output.extend({"source":case["name"],"census":tick,"slot":slot,"status":"outside_horizon","ranking":[]} for slot in range(2));continue
            frame=case["initial"] if tick==0 else next(r["after"] for r in case["rows"] if r["after"]["tick"]==tick)
            states={m["state"]["cell"]:m["state"] for m in frame["cells"]}
            histories={i:{"initial":s,"receipts":[]} for i,s in ((m["state"]["cell"],m["state"]) for m in case["initial"]["cells"])}
            for row in case["rows"]:
                if row["after"]["tick"]>tick:break
                records=[(op["cell"],r) for op in row["operations"] for r in op["receipts"]]
                for _,r in records:
                    if r["kind"]=="shared_birth":histories[r["child"]["cell"]]={"initial":r["child"],"receipts":[]}
                for ident,r in records:histories[ident]["receipts"].append(r)
            ranks=sorted([[i,keyed(frame["seed"],tick,i)] for i in states],key=lambda row:(row[1],row[0]))
            for slot in range(2):
                entry={"source":case["name"],"census":tick,"slot":slot,"ranking":ranks}
                if slot<len(ranks):
                    ident=ranks[slot][0];site=next(m["site"] for m in frame["cells"] if m["state"]["cell"]==ident)
                    entry.update(status="dead" if site is None else "living",cell=ident,lineage=reconstruct(states,histories,ident))
                else:entry["status"]="empty"
                output.append(entry)
    for control in witness["controls"]:
        histories={g["initial"]["cell"]:g for g in control["generations"]}
        output.append({"source":control["name"],"census":3,"slot":0,"status":"authored_control","cell":3,"ranking":[],
            "lineage":reconstruct({i:g["final"] for i,g in histories.items()},histories,3)})
    return output


def audit(witness,shared,shared_contract,measurement,requests,results,report,contract):
    require(contract["profile"]==PROFILE and witness["profile"]==measurement["profile"]==requests["profile"]==results["profile"]==report["profile"]==PROFILE,"lineage profiles")
    require(witness["contract"]==contract,"witness contract")
    for data in [witness,results,report,contract]:
        for flag in FALSE_FLAGS:require(data[flag] is False,"lineage admission "+flag)
    require(witness["discovery_evidence"] is False and witness["shared_economy_claim"] is False,"authored diagnostic scope")
    prior=audit_population(shared,shared_contract)
    work=0
    require([c["name"] for c in witness["controls"]]==[c["name"] for c in contract["controls"]],"control denominator")
    for control,spec in zip(witness["controls"],contract["controls"]):
        require(len(control["generations"])==4,"diagnostic generation denominator")
        child=None
        for gen,g in enumerate(control["generations"]):
            if child is not None:require(g["initial"]==child,"paid diagnostic continuity")
            else:
                require(graph_stages(g["initial"])==spec["founder"] and g["initial"]["capital"]==40 and g["initial"]["genome"]["editor_mode"]==spec["editor_mode"] and g["initial"]["seed"]==contract["development_seed"],"authored founder input")
            n,births=trace(g["initial"],g["receipts"],g["final"]);work+=n
            require(g["initial"]["cell"]==g["initial"]["generation"]==gen and g["initial"]["tick"]==0 and g["final"]["tick"]==4,"control full clocks")
            edits=[r for r in g["receipts"] if r["kind"] in ["endogenous_edit","external_edit"]]
            require([r["extra"]["selected_serial"] for r in edits]==spec["edits_by_generation"][gen],"declared authored edit targets")
            require(all(r["extra"]["probe"] is True and r["before"]["tick"]==1 for r in edits),"resource-before-authored-edit phase")
            require(len(births)==int(gen<3),"paid control transmissions")
            child=births[0] if births else None
            if child:require(child["cell"]==gen+1,"control child identity")
    require(work==witness["native_rewrites"],"witness work")
    expected=samples(shared,witness,contract);registry={};inputs=[]
    for sample in expected:
        for pair in sample.get("lineage",{}).get("pairs",[]):
            indices={}
            for role in contract["assay"]["roles"]:
                stages=pair["programs"][role];key=(role,tuple(stages))
                if key not in registry:
                    registry[key]=len(inputs);inputs.append({"id":len(inputs),"role":role,"stages":stages})
                indices[role]=registry[key]
            del pair["programs"];pair["assays"]=indices
    require(measurement=={"profile":PROFILE,"samples":expected},"complete sampled lineage reconstruction")
    require(requests=={"profile":PROFILE,"assays":inputs} and len(inputs)<=128 and len(expected)==75,"prospective request denominator")
    require([r["request"] for r in results["results"]]==inputs,"native assay denominator")
    assay_work=0;scores=[];statuses=[]
    for result,r in zip(results["results"],inputs):
        initial,final=result["initial"],result["final"]
        require(graph_stages(initial)==r["stages"] and initial["genome"]["editor_mode"]==0 and initial["arm"]=="frozen" and initial["capital"]==40 and initial["environment"]==512 and initial["seed"]==contract["development_seed"] and initial["tick"]==0,"standardized intervention context")
        n,births=trace(initial,result["receipts"],final);assay_work+=n
        require(final["tick"]==4 and len(births)<=1 and all(r["kind"] not in ["endogenous_edit","external_edit","selection"] for r in result["receipts"]),"assay horizon/mutation disabled")
        cost=3+len(r["stages"])
        if births:
            require(result["status"]=="complete" and births[0]["genome"]==initial["genome"],"paid standardized copy")
        else:require(result["status"]=="unfunded_copy" and final["capital"]<cost+8,"retained assay copy refusal")
        score=final["capital"]+final["transferred"]-initial["capital"]
        require(score==result["score"],"native score excludes diagnostic environment")
        scores.append(score);statuses.append(result["status"])
    require(assay_work==results["native_rewrites"],"complete assay work")
    expected_rows=[]
    for index,sample in enumerate(expected):
        evaluations=[]
        for pair in sample.get("lineage",{}).get("pairs",[]):
            v={k:scores[i] for k,i in pair["assays"].items()}
            marginal=[v["earlier_after"]-v["earlier_before"]-2,v["later_after"]-v["later_before"]-2]
            funded=all(statuses[i]=="complete" for i in pair["assays"].values())
            useful=funded and min(marginal)>0 and all(v["intact"]>v[k] for k in ["remove_earlier","remove_later","padded_remove_earlier"]) and v["intact"]==v["sham"]==v["restored"]
            durable=min(pair["transmissions"])>=2 and all(len(g)>=2 for g in pair["active_descendant_generations"])
            origin=all(o=="endogenous" for o in pair["origins"])
            evaluations.append({"earlier":pair["earlier"],"later":pair["later"],"scores":v,"marginal_paid_benefits":marginal,
                "all_comparisons_funded":funded,"causally_useful":useful,"durable":durable,"functional_cumulative_witness":useful and durable,
                "autonomous_origin":origin,"discovery_candidate_pass":useful and durable and origin})
        expected_rows.append({"sample":index,"pairs":evaluations})
    require(report=={"profile":PROFILE,"rows":expected_rows,**{f:False for f in FALSE_FLAGS}},"independent criterion aggregation")
    for index in [72,73]:
        pairs=expected_rows[index]["pairs"]
        require(len(pairs)==1 and pairs[0]["functional_cumulative_witness"] and not pairs[0]["discovery_candidate_pass"],"authored positive control without discovery")
    redundant=expected_rows[73]["pairs"][0]["scores"]
    require(redundant["single_remove_earlier"]>=redundant["intact"]>redundant["remove_earlier"],"redundant single vs group necessity")
    require(not expected_rows[74]["pairs"],"seeded founder is not postfounder innovation")
    require(work+assay_work<=contract["limits"]["native_rewrites"],"lineage fixture work cap")
    return {"profile":PROFILE,"engineering_fixture_pass":True,"samples":len(expected),"assays":len(inputs),
        "candidate_pairs":sum(len(r["pairs"]) for r in expected_rows),"dead_samples":sum(s["status"]=="dead" for s in expected),
        "empty_slots":sum(s["status"]=="empty" for s in expected),"outside_horizon_slots":sum(s["status"]=="outside_horizon" for s in expected),
        "authored_functional_controls":2,"discovery_candidate_passes":sum(p["discovery_candidate_pass"] for r in expected_rows for p in r["pairs"]),
        "witness_native_rewrites":work,"assay_native_rewrites":assay_work,"native_rewrites":work+assay_work,
        "prior_recorded_native_rewrites":prior["native_rewrites"],"prior_native_rewrites_executed":0,
        "independent_lineage_and_sampling":True,"independent_rng_replay":False,"independent_allocator_replay":False,**{f:False for f in FALSE_FLAGS}}


def load_package(package):
    contract=json.loads((package/"contract.json").read_text());directory=package/"inputs/shared"
    seal_bytes=(directory/"output_sha256.json").read_bytes()
    require(hashlib.sha256(seal_bytes).hexdigest()==contract["shared_input"]["seal_sha256"],"known source input root")
    seal=json.loads(seal_bytes)
    for name in ["fixture.json","contract.json","audit.json"]:
        require(hashlib.sha256((directory/name).read_bytes()).hexdigest()==seal[name],"source input custody "+name)
    fixed=json.loads((package/"request_freeze.json").read_text())
    require(fixed["before_native_assays"] is True and fixed["outcomes_feed_back"] is False and fixed["scientific_prospective_sampling"] is False,"measurement request boundary")
    require(set(fixed["sha256"])=={"witness.json","measurement.json","requests.json","contract.json"},"fixed request hash denominator")
    for name,h in fixed["sha256"].items():
        require(hashlib.sha256((package/name).read_bytes()).hexdigest()==h,"fixed request custody "+name)
    failed=package/"retained_development_failure"
    require(hashlib.sha256((failed/"output_sha256.json").read_bytes()).hexdigest()==contract["retained_failed_development_seal_sha256"],"retained failure known root")
    failure_seal=json.loads((failed/"output_sha256.json").read_text())
    require({str(p.relative_to(failed)) for p in failed.rglob("*") if p.is_file() and p!=failed/"output_sha256.json"}==set(failure_seal),"retained failure file set")
    for rel,h in failure_seal.items():
        p=Path(rel)
        require(not p.is_absolute() and ".." not in p.parts and hashlib.sha256((failed/p).read_bytes()).hexdigest()==h,"retained failure custody")
    require(json.loads((failed/"manifest.json").read_text())["status"]=="failed","retained failure classification")
    names=[package/"witness.json",directory/"fixture.json",directory/"contract.json",package/"measurement.json",package/"requests.json",package/"assays.json",package/"report.json",package/"contract.json"]
    return [json.loads(p.read_text()) for p in names]


if __name__=="__main__":
    print(json.dumps(audit(*load_package(Path(sys.argv[1]))),indent=2,sort_keys=True))
