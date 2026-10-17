"""Independent committed-prefix audit; never executes or retries native work."""
import copy
import hashlib
import json
import sys
from pathlib import Path
from audit_readiness import require,trace,view,FALSE_FLAGS
from audit_regulation import checkpoint as cell_checkpoint
from audit_world import rows,registered,state
from audit_versions import expected_measurement
PROFILE="cumulative-durable-journal-readiness-v1"
sha=lambda b:hashlib.sha256(b).hexdigest()

def checkpoint(cp):
    require(set(cp)=={"state","cells"},"journal checkpoint fields");w=cp["state"];state(w)
    require(len(cp["cells"])==len(w["cells"]),"checkpoint complete member denominator")
    for e,m in zip(cp["cells"],w["cells"]):
        require(e["id"]==m["id"] and e["site"]==m["site"] and e["checkpoint"]["state"]==m["state"],"checkpoint member projection");cell_checkpoint(e["checkpoint"])

def reservation(w,spec):
    alive=sum(m["state"]["alive"] for m in w["cells"]);quota=min(w["parcels_per_tick"],w["reservoir"]//16,alive)
    probes=spec.get("probe_schedule",[[]]*16)[w["tick"]]
    return 5*quota+(len(w["cells"])+len(probes) if w["tick"]%3==0 else len(probes))+(alive if w["arm"]=="external" else 0)

def assay(native,request):
    require(native["profile"]=="cumulative-lineage-intervention-readiness-v1" and len(native["results"])==1,"single preserved native assay")
    for f in FALSE_FLAGS:require(native[f] is False,"durable assay admission")
    result=native["results"][0];local=copy.deepcopy(request);local["id"]=0
    require(result["request"]==local,"global-to-local assay identity mapping")
    initial,final=result["initial"],result["final"]
    require([g[2] for g in view(initial)[1]]==request["stages"] and initial["seed"]==2026100903 and initial["arm"]=="frozen" and initial["genome"]["editor_mode"]==0 and initial["capital"]==40 and initial["environment"]==512 and initial["tick"]==0,"unchanged native assay law")
    n,births=trace(initial,result["receipts"],final)
    require(final["tick"]==4 and len(births)<=1 and all(r["kind"] not in ["selection","endogenous_edit","external_edit"] for r in result["receipts"]),"complete frozen assay horizon")
    require(n==native["native_rewrites"] and result["score"]==final["capital"]+final["transferred"]-40,"assay work and score")
    require(result["status"]==("complete" if births else "unfunded_copy") and (bool(births) or final["capital"]<3+len(request["stages"])+8),"full refused assay route")
    if births:require(births[0]["genome"]==initial["genome"],"assay paid copying")
    return n

def read(directory):
    names=sorted(p.name for p in directory.iterdir());published=[n for n in names if n[:1].isdigit()];staging=[n for n in names if n.startswith(".staging-")]
    require(set(names)==set(published+staging) and published==[f"{i:08}.json" for i in range(len(published))] and 1<=len(published)<=2048,"durable journal exact file set")
    require(sum(p.stat().st_size for p in directory.iterdir())<=33554432,"journal byte cap")
    values=[];head=None
    for i,name in enumerate(published):
        p=directory/name;require(p.is_file() and not p.is_symlink(),"journal regular record")
        raw=p.read_bytes();e=json.loads(raw)
        require(set(e)=={"profile","sequence","previous_sha256","value"} and e["profile"]==PROFILE and type(e["sequence"]) is int and e["sequence"]==i and e["previous_sha256"]==head and raw.endswith(b"\n"),"exact journal hash chain")
        head=sha(raw);values.append(e["value"])
    for name in staging:
        require(name in [f".staging-{len(published):08}.json",f".staging-{len(published)-1:08}.json"] and (directory/name).is_file() and not (directory/name).is_symlink(),"retained publication stage")
        if name==f".staging-{len(published)-1:08}.json":require((directory/name).read_bytes()==(directory/published[-1]).read_bytes(),"linked staging is exactly the committed last record")
    return values,head,staging

def recover(directory,request,request_sha256,source=None):
    values,head,staging=read(directory);header=values[0]
    require(sum(p.stat().st_size for p in directory.iterdir())<=request["journal_byte_allowance"],"per-invocation journal storage allowance")
    require(header=={"kind":"header","request":request,"request_sha256":request_sha256,"carried_work":0 if source is None or request["mode"]=="world_replay" else source["lower"]},"frozen journal header/source")
    mode=request["mode"];specs=request["world_contract"]["cases"];case_map={};assays=[];carried=header["carried_work"];work=carried;pending=None;terminal=None;index=1;replay=mode=="world_replay"
    if source is not None:
        require(not source["unknown"] and not source["staging"] and request["source_head_sha256"]==source["head"],"safe committed source only")
        for k in ["source_identity","world_contract","assay_requests","native_budget"]:require(request[k]==source["request"][k],"source input/budget identity "+k)
        require(source["request"]["mode"]==("assays" if mode=="assay_continue" else "worlds"),"one-hop continuation")
        expected={"kind":"source_prefix","source_head_sha256":source["head"],"cases":source["cases"],"assays":source["assays"],"source_work":source["lower"],"replay":replay}
        require(values[index]==expected,"retained complete ancestry/assay prefix");index+=1
        if replay:require(source["status"]=="complete" and request["replay"]=={"case":11,"start_tick":2},"declared diagnostic replay")
        else:case_map={c["case"]:copy.deepcopy(c) for c in source["cases"]};assays=copy.deepcopy(source["assays"])
    else:require(mode in ["worlds","assays"],"missing source journal")
    for v in values[index:]:
        require(terminal is None,"record after journal terminal");kind=v["kind"]
        if kind=="initial":
            require(pending is None and mode.startswith("world") and set(v)=={"kind","case","checkpoint"},"journal initialization authority")
            case=v["case"];require(type(case) is int and case not in case_map and (case==11 if replay else case==len(case_map)) and (replay or all(c["complete"] for c in case_map.values())),"fixed case order")
            cp=v["checkpoint"];checkpoint(cp)
            if replay:
                reference=source["cases"][11];require(cp["state"]==reference["rows"][1]["after"],"same declared replay checkpoint")
                # Exact wire checkpoint is compared with the source journal result
                # by the package audit, not just its decoded world state.
            else:registered(cp["state"],specs[case],request["world_contract"]["development_seed"])
            case_map[case]={"case":case,"initial":cp["state"],"rows":[],"checkpoint":cp,"complete":False}
        elif kind=="intent":
            require(pending is None and type(v["work_before"]) is int and v["work_before"]==work,"single pre-native intent and cumulative ledger")
            if mode.startswith("world"):
                case=11 if replay else next((i for i,c in sorted(case_map.items()) if not c["complete"]),None);require(case is not None,"world intent has initialized case")
                w=case_map[case]["checkpoint"]["state"];require(w["tick"]<specs[case]["ticks"],"no extended horizon")
                bound=reservation(w,specs[case]);expected={"kind":"intent","unit":"world","case":case,"tick":w["tick"],"work_before":work,"work_bound":bound}
            else:require(len(assays)<len(request["assay_requests"]["assays"]),"all assay request slots fixed");bound=20;expected={"kind":"intent","unit":"assay","index":len(assays),"work_before":work,"work_bound":bound}
            require(v==expected and work+bound<=request["native_budget"] and work-carried+bound<=request["execution_allowance"],"full pre-native work reservation");pending=v
        elif kind=="result":
            require(pending is not None,"result has committed native intent")
            if pending["unit"]=="world":
                require(set(v)=={"kind","unit","case","tick","row","checkpoint","native_rewrites","work_total"} and v["unit"]=="world" and v["case"]==pending["case"] and v["tick"]==pending["tick"],"result action identity")
                c=case_map[v["case"]];cp=v["checkpoint"];checkpoint(cp);require(v["row"]["after"]==cp["state"],"single row/checkpoint commit")
                n,_=rows(c["checkpoint"]["state"],[v["row"]],cp["state"],specs[v["case"]]);c["rows"].append(v["row"]);c["checkpoint"]=cp
            else:
                require(set(v)=={"kind","unit","index","native","native_rewrites","work_total"} and v["unit"]=="assay" and v["index"]==pending["index"],"assay result identity")
                n=assay(v["native"],request["assay_requests"]["assays"][v["index"]]);assays.append(v["native"])
            work+=n;require(type(v["native_rewrites"]) is int and v["native_rewrites"]==n<=pending["work_bound"] and v["work_total"]==work,"actual committed native work")
            pending=None
        elif kind=="case_end":
            require(pending is None and mode.startswith("world") and set(v)=={"kind","case","tick"},"case-end boundary")
            c=case_map[v["case"]];require(not c["complete"] and v["tick"]==c["checkpoint"]["state"]["tick"]==specs[v["case"]]["ticks"],"registered completed case horizon");c["complete"]=True
        elif kind=="budget_stop":
            require(pending is None and v["work_total"]==work,"budget stop before native work")
            if mode.startswith("world"):
                case=11 if replay else next(i for i,c in sorted(case_map.items()) if not c["complete"]);w=case_map[case]["checkpoint"]["state"];bound=reservation(w,specs[case]);expected={"kind":"budget_stop","work_total":work,"reservation":bound,"case":case,"tick":w["tick"]}
            else:bound=20;expected={"kind":"budget_stop","work_total":work,"reservation":bound,"index":len(assays)}
            require(v==expected and (work+bound>request["native_budget"] or work-carried+bound>request["execution_allowance"]),"genuine unchanged cap stop");terminal="budget_stop"
        elif kind=="complete":
            require(pending is None and v=={"kind":"complete","work_total":work,"work_here":work-carried},"complete journal work ledger")
            require((len(case_map)==(1 if replay else 12) and all(c["complete"] for c in case_map.values())) if mode.startswith("world") else len(assays)==len(request["assay_requests"]["assays"]),"no completed-prefix reclassification");terminal="complete"
        else:raise ValueError("unknown journal record "+kind)
    require(not any(n==f".staging-{len(values):08}.json" for n in staging) or pending is not None,"unpublished native result needs pending reservation")
    upper=work+(pending["work_bound"] if pending else 0);require(upper<=request["native_budget"] and upper-carried<=request["execution_allowance"],"unknown work remains charged within caps")
    result={"status":terminal or "partial","head":head,"request":request,"lower":work,"upper":upper,"carried":carried,"here_lower":work-carried,"here_upper":upper-carried,"unknown":pending is not None,"staging":staging,"cases":[case_map[i] for i in sorted(case_map)],"assays":assays,"records":len(values),"pending":pending}
    return result

def summary(r):
    request=r["request"];slots=[]
    if request["mode"].startswith("world"):
        observed={c["case"]:c for c in r["cases"]}
        for i,spec in enumerate(request["world_contract"]["cases"]):
            c=observed.get(i)
            for tick in request["world_contract"]["sampling"]["census_ticks"]:
                status="outside_horizon" if tick>spec["ticks"] else "not_started" if c is None else "committed_census" if tick>=c["initial"]["tick"] and tick<=c["checkpoint"]["state"]["tick"] else "not_reached"
                slots.extend({"case":i,"census":tick,"slot":slot,"status":status} for slot in range(2))
    else:slots=[{"index":i,"status":"committed_assay" if i<len(r["assays"]) else "not_reached"} for i in range(len(request["assay_requests"]["assays"]))]
    return {"status":r["status"],"chain_sha256":r["head"],"records":r["records"],"native_work_lower":r["lower"],"native_work_upper":r["upper"],"native_work_here_lower":r["here_lower"],"native_work_here_upper":r["here_upper"],"carried_work":r["carried"],"native_work_unknown":r["unknown"],"staging_files":r["staging"],"resumable_known_boundary":not r["unknown"] and not r["staging"] and r["status"]=="partial","slots":slots,**{f:False for f in FALSE_FLAGS}}

def audit(package):
    contract=json.loads((package/"contract.json").read_text());limits=contract["limits"];world_contract=json.loads((package/"world_contract.json").read_text());frozen_requests=json.loads((package/"assay_requests.json").read_text());identity=json.loads((package/"identity.json").read_text());routes={};summaries={}
    require(contract["profile"]==PROFILE,"durability profile")
    require(identity=={"binary_sha256":sha((package/"cumulative-readiness").read_bytes()),"source_sha256":sha((package/"source_sha256.json").read_bytes())},"authenticated executable/source identity")
    frozen=json.loads((package/"request_freeze.json").read_text())
    require(frozen["before_all_native_routes"] is True and frozen["outcomes_feed_back"] is False and frozen["world_contract_sha256"]==sha((package/"world_contract.json").read_bytes()) and frozen["assay_requests_sha256"]==sha((package/"assay_requests.json").read_bytes()) and frozen["parent_equivalence_sha256"]==sha((package/"parent_equivalence.json").read_bytes()),"prospectively fixed engineering route inputs")
    for f in FALSE_FLAGS:require(contract[f] is False,"durability admission")
    for route in contract["routes"]:
        name=route["name"];input_path=package/"inputs"/(name+".json");request=json.loads(input_path.read_text());receipt=json.loads((package/"commands"/(name+".json")).read_text())
        require(request["profile"]==PROFILE and request["mode"]==route["mode"] and request["fault"]==route.get("fault") and request["world_contract"]==world_contract and request["assay_requests"]==frozen_requests and request["source_identity"]==identity and request["native_budget"]==route.get("native_budget",6000),"fixed route inputs")
        directory=package/"journals"/name;source=routes.get(route.get("source"))
        require(0<=request["execution_allowance"]<=6000 and 0<=request["journal_byte_allowance"]<=33554432 and request["replay"]==({"case":11,"start_tick":2} if route["mode"]=="world_replay" else None),"bounded route invocation and fixed replay")
        if source is not None:require(request["source_head_sha256"]==source["head"] and Path(request["source_dir"]).name==route["source"],"registered source reference")
        else:require(request["source_dir"] is None and request["source_head_sha256"] is None,"source-free route")
        if route.get("expect_rejection"):
            require(source is not None and source["unknown"] and receipt["exit_code"]!=0 and not directory.exists(),"uncertain continuation rejected before publication/work")
            summaries[name]={"status":"rejected_unsafe_source","native_work_here_lower":0,"native_work_here_upper":0,"native_work_unknown":False,**{f:False for f in FALSE_FLAGS}}
            require(json.loads((package/"recovery"/(name+".json")).read_text())==summaries[name],"retained unsafe-source refusal report");continue
        recovered=recover(directory,request,sha(input_path.read_bytes()),source);routes[name]=recovered;summaries[name]=summary(recovered)
        if recovered["status"]=="complete":require(receipt["exit_code"]==0,"completed native command")
        elif route.get("fault"):require(receipt["exit_code"]!=0,"fixed injected interruption retained")
        require(json.loads((package/"recovery"/(name+".json")).read_text())==summaries[name],"independent recovered route report")
    complete=routes["world_complete"];continued=routes["world_continue"]
    require(complete["status"]==continued["status"]=="complete" and complete["cases"]==continued["cases"] and complete["lower"]==continued["lower"]==151,"exact continued full denominator")
    source_values,_,_=read(package/"journals/world_complete");replay_values,_,_=read(package/"journals/world_replay")
    exact_cp=next(v["checkpoint"] for v in source_values if v["kind"]=="result" and v["case"]==11 and v["tick"]==1)
    require(next(v["checkpoint"] for v in replay_values if v["kind"]=="initial")==exact_cp,"exact allocator checkpoint at declared replay boundary")
    replay=routes["world_replay"];reference=complete["cases"][11];require(replay["status"]=="complete" and replay["cases"][0]["rows"]==reference["rows"][2:] and replay["cases"][0]["checkpoint"]==reference["checkpoint"],"declared replay exact suffix/checkpoint")
    require(routes["assay_complete"]["assays"]==routes["assay_continue"]["assays"] and routes["assay_complete"]["lower"]==routes["assay_continue"]["lower"]==324,"exact continued assay denominator")
    world={"cases":[{"name":s["name"],"initial":c["initial"],"rows":c["rows"],"final":c["checkpoint"]["state"]} for c,s in zip(complete["cases"],world_contract["cases"])]}
    measurement,requests=expected_measurement(world,world_contract);require(requests==frozen_requests and measurement==json.loads((package/"measurement.json").read_text()),"complete version ancestry reconstructed from durable rows")
    reference_hashes=json.loads((package/"parent_equivalence.json").read_text())
    encode=lambda v:json.dumps(v,sort_keys=True,separators=(",",":"),ensure_ascii=False).encode()
    require(reference_hashes["world_cases_sha256"]==sha(encode(world["cases"])) and reference_hashes["measurement_sha256"]==sha(encode(measurement)),"matches authenticated immutable parent cases/lineages")
    global_results=[]
    for native,r in zip(routes["assay_complete"]["assays"],frozen_requests["assays"]):
        result=copy.deepcopy(native["results"][0]);result["request"]=r;global_results.append(result)
    require(reference_hashes["assay_results_sha256"]==sha(encode(global_results)),"matches authenticated immutable parent assays")
    test_work=json.loads((package/"test_work.json").read_text())["native_rewrites"]
    charged=test_work
    for route in contract["routes"]:
        r=json.loads((package/"inputs"/(route["name"]+".json")).read_text());require(r["execution_allowance"]<=limits["native_rewrites"]-charged,"aggregate reservation funded before dispatch")
        charged+=summaries[route["name"]]["native_work_here_upper"]
    lower=test_work+sum(r["here_lower"] for r in routes.values());upper=test_work+sum(r["here_upper"] for r in routes.values())
    require(upper<=limits["native_rewrites"],"aggregate unknown reservations remain funded")
    return {"profile":PROFILE,"engineering_fixture_pass":True,"routes":summaries,"native_rewrites_lower_bound":lower,"native_rewrites_upper_bound":upper,"native_work_unknown":lower!=upper,"test_native_rewrites":test_work,"world_native_rewrites":151,"assay_native_rewrites":324,"samples":96,"assays":33,"independent_prefix_accounting_and_continuation":True,"independent_rng_replay":False,"independent_allocator_replay":False,**{f:False for f in FALSE_FLAGS}}

if __name__=="__main__":
    result=audit(Path(sys.argv[1]))
    if len(sys.argv)==3:Path(sys.argv[2]).write_text(json.dumps(result,indent=2,sort_keys=True)+"\n")
    print(json.dumps({k:v for k,v in result.items() if k!="routes"},indent=2,sort_keys=True))
