"""Independent shared-world scheduler/accounting audit over the frozen cell law."""
import copy
from audit_readiness import require,FALSE_FLAGS,handle
from audit_population import score
from audit_regulation import state as cell_state,trajectory,checkpoint as cell_checkpoint,program
PROFILE="cumulative-regulated-world-lineage-readiness-v1"

def members(w):return {m["id"]:m for m in w["cells"]}

def state(w):
    require(set(w)=={"profile","seed","arm","tick","reservoir","initial_reservoir","sites","parcels_per_tick","initial_energy","next_id","births","cells"} and w["profile"]==PROFILE,"world fields/profile")
    require(all(type(w[k]) is int for k in ["seed","tick","reservoir","initial_reservoir","sites","parcels_per_tick","initial_energy","next_id"]),"world typed scalars")
    require(0<=w["seed"]<2**64 and 0<=w["tick"]<=16 and 1<=w["sites"]<=8 and 0<=w["parcels_per_tick"]<=8 and 0<=w["reservoir"]<=w["initial_reservoir"]<=512 and w["reservoir"]%16==w["initial_reservoir"]%16==0,"world bounds")
    require([m["id"] for m in w["cells"]]==list(range(w["next_id"])) and 1<=len(w["cells"])<=32,"retained identity denominator")
    ms=members(w);sites=[];energy=w["reservoir"];grants=0;founders=0
    for i,m in ms.items():
        require(set(m)=={"id","site","state"},"member fields");s=m["state"];cell_state(s);c=s["core"]
        require(c["cell"]==i and c["seed"]==score(w["seed"],0,3,i) and c["arm"]==w["arm"] and c["tick"]==w["tick"] and s["reservoir"]==0 and s["initial_reservoir"]==16*s["grants"],"shared cell identity/clock/source")
        require(s["alive"]==(m["site"] is not None),"life/site")
        if m["site"] is not None:require(type(m["site"]) is int and 0<=m["site"]<w["sites"],"site bounds");sites.append(m["site"])
        parent=c["parent"]
        if parent is None:require(c["generation"]==0,"founder generation");founders+=s["endowment"]
        else:require(parent in ms and parent<i and c["generation"]==ms[parent]["state"]["core"]["generation"]+1 and s["endowment"]==8 and s["baseline"]==ms[parent]["state"]["baseline"],"shared ancestry/baseline")
        require(c["transferred"]==8*sum(n["state"]["core"]["parent"]==i for n in ms.values()),"global copy transfers")
        energy+=c["capital"]+c["dissipated"];grants+=s["grants"]
    require(len(sites)==len(set(sites)) and energy==w["initial_energy"]==w["initial_reservoir"]+founders and w["initial_reservoir"]-w["reservoir"]==16*grants,"shared energy/sites")
    require([b["child"] for b in w["births"]]==[i for i,m in ms.items() if m["state"]["core"]["parent"] is not None],"birth archive denominator")
    for b in w["births"]:
        program(b["inherited"]);program(b["transmitted"])
        require(ms[b["child"]]["state"]["core"]["parent"]==b["parent"] and 1<=b["tick"]<=w["tick"] and b["copy_cost"]==3+len(b["inherited"]["genes"]),"birth log")

def birth(parent,frame,world,site,ident):
    require(frame["kind"]=="shared_birth" and frame["before"]==parent,"shared paid birth frame")
    c=parent["core"];p=parent["program"];cost=3+len(p["genes"])
    wanted=parent["baseline"] if world["arm"]=="nonheritable" else p
    record={"parent":c["cell"],"child":ident,"tick":world["tick"]+1,"site":site,"copy_cost":cost,"inherited":p,"transmitted":wanted}
    require(frame["birth"]==record and parent["alive"] and c["capital"]>=cost+8+24,"complete funded birth/reserve")
    expected=copy.deepcopy(parent);d=expected["core"];d["capital"]-=cost+8;d["dissipated"]+=cost;d["transferred"]+=8
    next(n for n in d["nodes"] if n["id"]==handle(d["wallet"]))["properties"]=[[1,d["capital"]]]
    require(frame["after"]==expected,"exact parent copying debit")
    child=frame["child"];cell_state(child);ch=child["core"]
    require(child["program"]==wanted and child["baseline"]==parent["baseline"] and child["alive"] and child["endowment"]==8 and child["initial_reservoir"]==child["reservoir"]==child["grants"]==child["attempts"]==0,"no birth resource or program reset leak")
    require(ch["cell"]==ident and ch["parent"]==c["cell"] and ch["generation"]==c["generation"]+1 and ch["seed"]==score(world["seed"],0,3,ident) and ch["arm"]==world["arm"] and ch["tick"]==world["tick"]+1,"paid immutable birth identity")
    require(ch["capital"]==ch["initial"]==8 and ch["environment"]==ch["dissipated"]==ch["transferred"]==ch["rewrites"]==0 and ch["word_pos"]=="0" and all(n["born"]==ch["tick"] for n in ch["nodes"]),"newborn cost/clock")
    return expected,child,record

def rows(initial,rows,final,spec):
    current=copy.deepcopy(initial);state(current);work=0;families=[]
    for row in rows:
        require(set(row)=={"before","after","resource_ranking","resource_winners","birth_ranking","birth_refusals","operations"},"world row field set")
        require(row["before"]==current,"world row continuity");tick=current["tick"];ms=members(current);ids=list(ms)
        requestors=[i for i,m in ms.items() if m["state"]["alive"] and m["state"]["core"]["capital"]>=2+len(m["state"]["program"]["genes"])]
        rank=sorted([[i,score(current["seed"],tick,0,i)] for i in requestors],key=lambda r:(r[1],r[0]))
        winners=[r[0] for r in rank[:min(current["parcels_per_tick"],current["reservoir"]//16)]]
        require(row["resource_ranking"]==rank and row["resource_winners"]==winners,"complete shared keyed resource arbitration")
        operations=row["operations"];index=0
        def take(phase,ident):
            nonlocal index
            op=operations[index];index+=1
            require(set(op)=={"phase","cell","initial","frames","final"},"world operation field set")
            require(op["phase"]==phase and op["cell"]==ident and op["initial"]==ms[ident]["state"],"complete world operation order")
            return op
        for ident in ids:
            op=take("resource",ident);start=op["initial"];frames=op["frames"]
            if ident in winners:
                f=frames[0];expected=copy.deepcopy(start);expected["reservoir"]+=16;expected["initial_reservoir"]+=16
                require(f=={"kind":"shared_allocation","before":start,"after":expected,"amount":16},"exact finite-bank allocation")
                start=expected;frames=frames[1:];current["reservoir"]-=16
            require(len(frames)==1 and frames[0]["kind"]=="resource_tick" and set(frames[0])=={"kind","before","after","receipts"},"one mandatory resource/storage phase")
            n,f,b=trajectory(start,frames,op["final"]);work+=n;require(not f and not b,"resource phase authority")
            require(op["final"]["core"]["tick"]==tick+1 and op["final"]["reservoir"]==0,"resource boundary clock/escrow")
            ms[ident]["state"]=op["final"]
            if not op["final"]["alive"]:ms[ident]["site"]=None
        for ident in ids:
            op=take("edit",ident);probes=spec.get("probe_schedule",[[]]*16)[tick] if ident==0 else []
            n,f,b=trajectory(op["initial"],op["frames"],op["final"],probes=bool(probes));work+=n;families+=f;require(not b,"edit birth separation")
            attempts=[f for f in op["frames"] if f["kind"] in ["edit_selection","edit_refusal"]]
            require(len(attempts)==(len(probes) if probes else int(tick%3==0)),"fixed edit opportunity cadence")
            require([f["probe"] for f in attempts]==(probes if probes else [None]*len(attempts)) and all(f["external"] is False and f["diagnostic"] is False for f in attempts),"declared authored vs blind origin")
            require(len(op["frames"])==sum(2 if f["kind"]=="edit_selection" else 1 for f in attempts),"complete edit phase")
            ms[ident]["state"]=op["final"]
        parents=[i for i in ids if ms[i]["state"]["alive"]];rank=sorted([[i,score(current["seed"],tick,1,i)] for i in parents],key=lambda r:(r[1],r[0]))
        require(row["birth_ranking"]==rank,"complete keyed birth arbitration");refusals=[]
        for ident,_ in rank:
            p=ms[ident]["state"];cost=3+len(p["program"]["genes"]);occupied={m["site"] for m in ms.values() if m["site"] is not None};free=[s for s in range(current["sites"]) if s not in occupied]
            reason="unfunded" if p["core"]["capital"]<cost+8+24 else "no_site" if not free else "cell_cap" if len(ms)==32 else None
            if reason:refusals.append({"cell":ident,"reason":reason});continue
            op=take("birth",ident);require(len(op["frames"])==1,"one paid birth per parent tick")
            next_id=len(ms);parent,child,record=birth(p,op["frames"][0],row["before"],free[0],next_id)
            require(op["final"]==parent,"birth operation endpoint");ms[ident]["state"]=parent;ms[next_id]={"id":next_id,"site":free[0],"state":child};current["births"].append(record)
            op=take("newborn",next_id);n,f,b=trajectory(child,op["frames"],op["final"]);work+=n;families+=f;require(not b,"no newborn reproduction")
            attempts=[f for f in op["frames"] if f["kind"] in ["edit_selection","edit_refusal"]]
            require(len(attempts)==int(current["arm"]=="external") and all(f["external"] is True and f["probe"] is None and f["diagnostic"] is False for f in attempts),"single external birth mutation")
            require(len(op["frames"])==sum(2 if f["kind"]=="edit_selection" else 1 for f in attempts),"newborn phase complete");ms[next_id]["state"]=op["final"]
        require(refusals==row["birth_refusals"] and index==len(operations),"all site/funding/archive refusals and operations")
        current.update(tick=tick+1,next_id=len(ms),cells=[ms[i] for i in sorted(ms)])
        require(current==row["after"],"exact shared world reconstruction");state(current)
    require(current==final,"complete world endpoint");return work,families

def registered(initial,spec,seed):
    state(initial)
    require(initial["seed"]==seed and initial["arm"]==spec["arm"] and initial["tick"]==0 and initial["reservoir"]==initial["initial_reservoir"]==spec["reservoir"] and initial["sites"]==spec["sites"] and initial["parcels_per_tick"]==spec["parcels_per_tick"] and not initial["births"] and len(initial["cells"])==len(spec["programs"]),"registered world input")
    for m,p in zip(initial["cells"],spec["programs"]):
        s=m["state"];wanted={"genes":[[i,x] for i,x in enumerate(p["stages"])],"next_serial":len(p["stages"]),"enabled":p["enabled"],"mask":p["mask"]}
        require(s["program"]==s["baseline"]==wanted and s["endowment"]==s["core"]["capital"]==spec["capital"] and s["core"]["parent"] is None and s["core"]["word_pos"]=="0" and s["core"]["rewrites"]==s["attempts"]==s["grants"]==0 and m["site"]==m["id"],"registered founder program and budget")

def audit_world(data,contract):
    require(data["profile"]==contract["profile"]==PROFILE and data["contract"]==contract,"integration contract")
    for d in [data,contract]:
        for f in FALSE_FLAGS:require(d[f] is False,"integration admission "+f)
    require([c["name"] for c in data["cases"]]==[c["name"] for c in contract["cases"]],"world case denominator");work=0;families=[]
    for case,spec in zip(data["cases"],contract["cases"]):
        registered(case["initial"],spec,contract["development_seed"]);n,f=rows(case["initial"],case["rows"],case["final"],spec);work+=n;families+=f
        require(len(case["rows"])==spec["ticks"]==case["final"]["tick"],"registered full world horizon")
    spec=next(c for c in contract["cases"] if c["name"]==contract["diagnostics"]["order_case"]);order=data["order"]
    registered(order["initial"],spec,contract["development_seed"]);registered(order["offset_initial"],spec,contract["development_seed"])
    require(order["original"]==order["reverse"],"world enumeration order parity")
    for branch,initial,final in [("original",order["initial"],order["final"]),("reverse",order["initial"],order["final"]),("offset",order["offset_initial"],order["offset_final"])]:
        n,f=rows(initial,order[branch],final,spec);work+=n;families+=f;require(len(order[branch])==2,"order horizon")
    require(order["final"]["births"]==order["offset_final"]["births"],"offset ancestry parity")
    for a,b in zip(order["final"]["cells"],order["offset_final"]["cells"]):
        require(a["site"]==b["site"] and a["state"]["program"]==b["state"]["program"],"allocation policy/serial/site parity")
        for k in ["capital","dissipated","transferred","word_pos","tick","rewrites"]:require(a["state"]["core"][k]==b["state"]["core"][k],"allocation energy/cursor parity")
    cp=data["continuation"]["checkpoint"];require(cp["state"]==order["final"],"world checkpoint phase")
    require([e["id"] for e in cp["cells"]]==[m["id"] for m in cp["state"]["cells"]],"checkpoint retained membership")
    for e,m in zip(cp["cells"],cp["state"]["cells"]):require(e["site"]==m["site"] and e["checkpoint"]["state"]==m["state"],"checkpoint metadata");cell_checkpoint(e["checkpoint"])
    cont=data["continuation"];require(cont["original"]==cont["restored"],"exact world restored continuation")
    for branch in ["original","restored"]:
        n,f=rows(cp["state"],cont[branch],cont["final"],spec);work+=n;families+=f;require(len(cont[branch])==2 and cont["final"]["tick"]==4,"continuation horizon")
    require(work==data["native_rewrites"]<=contract["limits"]["native_rewrites"] and set(families)==set(range(5)),"all world native work and families")
    return {"native_rewrites":work,"cases":len(data["cases"]),"native_families":sorted(set(families)),"independent_shared_world_accounting_and_scheduler":True}
