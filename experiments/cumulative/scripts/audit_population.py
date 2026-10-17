"""Shared-economy auditor: independent scheduler, budgets, lineage and full frames.
Native graph transitions reuse the frozen integer/binding auditor; no Rust calls.
ChaCha edit words and slotmap allocation remain outside independent replay scope.
"""
import copy
import json
import sys
from pathlib import Path
from audit_readiness import require, view, trace, compare_scalars, handle, FALSE_FLAGS

PROFILE = "cumulative-shared-population-readiness-v1"
MASK = (1 << 64) - 1


def score(seed, tick, phase, ident):
    z = (seed ^ ((tick*0x9e3779b97f4a7c15)&MASK) ^ ((phase*0xd1b54a32d192ed03)&MASK) ^ ((ident*0x94d049bb133111eb)&MASK))&MASK
    z = (z + 0x9e3779b97f4a7c15)&MASK
    z = ((z ^ (z >> 30))*0xbf58476d1ce4e5b9)&MASK
    z = ((z ^ (z >> 27))*0x94d049bb133111eb)&MASK
    return z ^ (z >> 31)


def members(world):
    result = {m["state"]["cell"]: m for m in world["cells"]}
    require(len(result) == len(world["cells"]), "duplicate population member")
    return result


def checkpoint_view(checkpoint,baselines):
    ms=world_view(checkpoint["state"])
    require(len(checkpoint["cells"])==len(ms),"checkpoint member denominator")
    seen=set()
    for entry in checkpoint["cells"]:
        s=entry["state"];ident=s["cell"]
        require(ident not in seen and ident in ms,"checkpoint member identity");seen.add(ident)
        require(s==ms[ident]["state"] and entry["metadata"]=={k:ms[ident][k] for k in ["site","endowment","grants"]},"checkpoint world binding")
        root=ident
        while ms[root]["state"]["parent"] is not None:root=ms[root]["state"]["parent"]
        require(entry["baseline"]==baselines[root],"checkpoint founder baseline")
        graph=entry["graph"];nodes=[];edges=[];free_nodes=[];free_edges=[]
        for index,slot in enumerate(graph["nodes"]):
            if slot["value"] is None:
                if index:free_nodes.append(index)
                continue
            require(slot["version"]%2==1,"live node generation")
            n=slot["value"];props=[]
            require(n["energy"]==0,"checkpoint energy channel")
            for key,value in n["properties"]:
                require(set(value)=={"Int"} and type(value["Int"]) is int,"checkpoint typed property")
                props.append([key,value["Int"]])
            nodes.append({"id":index|(slot["version"]<<32),"label":n["label"],"born":n["generation"],"properties":sorted(props)})
        for index,slot in enumerate(graph["edges"]):
            if slot["value"] is None:
                if index:free_edges.append(index)
                continue
            require(slot["version"]%2==1,"live edge generation")
            e=slot["value"]
            edges.append({"id":index|(slot["version"]<<32),"label":e["label"],"nodes":[handle(n) for n in e["nodes"]]})
        nodes.sort(key=lambda n:n["id"])
        edges.sort(key=lambda e:e["id"])
        require(nodes==s["nodes"] and edges==s["edges"] and graph["step"]==s["graph_step"],"checkpoint graph projection")
        for actual,expected in zip(s["allocator"],[free_nodes,free_edges]):
            require(len(actual)==len(set(actual)) and sorted(actual)==expected,"allocator free membership")
        for n in nodes:
            slot=graph["node_to_edges"][n["id"]&0xffffffff]
            expected=sorted(e["id"] for e in edges if n["id"] in e["nodes"])
            require(slot["version"]==n["id"]>>32 and sorted(handle(e) for e in slot["value"])==expected,"checkpoint incidence index")
        for label,refs in graph["label_index"].items():
            require(sorted(handle(e) for e in refs)==sorted(e["id"] for e in edges if e["label"]==int(label)),"checkpoint label index")


def world_view(w):
    require(w["profile"] == PROFILE and 0 <= w["tick"] <= 16 and 1 <= w["sites"] <= 8, "world profile/caps")
    require(0 <= w["reservoir"] <= w["initial_reservoir"] <= 512 and w["reservoir"] % 16 == 0 and 0 <= w["parcels_per_tick"] <= 8, "reservoir domain")
    ms = members(w)
    require(set(ms) == set(range(w["next_id"])) and len(ms) <= 32, "member IDs/archive cap")
    occupied, energy, grants = set(), w["reservoir"], 0
    for ident, m in ms.items():
        c = m["state"]
        _, genes, parcels, _, _ = view(c)
        require(not parcels and c["environment"] == 0 and c["next_serial"] == len(genes), "empty escrow/canonical genome")
        require(c["arm"] == w["arm"] and c["seed"] == score(w["seed"],0,3,ident) and c["genome"]["editor_mode"] == 0, "cell seed/arm/editor")
        require(c["initial"] == m["endowment"] + 16*m["grants"], "cell source budget")
        if m["site"] is not None:
            require(type(m["site"]) is int and 0 <= m["site"] < w["sites"] and m["site"] not in occupied and c["tick"] == w["tick"], "site/live clock")
            occupied.add(m["site"])
        else:
            require(c["capital"] == 0 and c["tick"] <= w["tick"], "dead-cell boundary")
        if c["parent"] is not None:
            require(c["parent"] < ident and c["parent"] in ms and c["generation"] == ms[c["parent"]]["state"]["generation"] + 1 and m["endowment"] == 8, "birth ancestry/endowment")
        else:
            require(c["generation"] == 0 and 0 <= m["endowment"] <= 64, "founder")
        children = sum(n["state"]["parent"] == ident for n in ms.values())
        require(c["transferred"] == 8*children, "parent child transfer")
        energy += c["capital"] + c["dissipated"]
        grants += m["grants"]
    require(energy == w["initial_energy"] and w["initial_reservoir"]-w["reservoir"] == 16*grants, "global conservation/no free resources")
    require(w["initial_energy"] == w["initial_reservoir"] + sum(m["endowment"] for m in ms.values() if m["state"]["parent"] is None), "initial world source")
    require(len(w["births"]) == sum(m["state"]["parent"] is not None for m in ms.values()), "genealogy count")
    return ms


def operations(initial, records, final, baseline):
    current = copy.deepcopy(initial)
    native, grants, births, deaths = 0, 0, [], 0
    index = 0
    specials = {"shared_grant","shared_birth","death","idle_boundary"}
    while index < len(records):
        r = records[index]
        kind = r["kind"]
        if kind not in specials:
            end = index
            while end < len(records) and records[end]["kind"] not in specials:
                end += 1
            endpoint = records[end]["before"] if end < len(records) else final
            n, unexpected = trace(current,records[index:end],endpoint)
            require(not unexpected,"legacy diagnostic births forbidden")
            native += n
            current = copy.deepcopy(endpoint)
            index = end
            continue
        require(r["before"] == current,"population receipt continuity")
        after = r["after"]
        if kind == "shared_grant":
            compare_scalars(current,after,{"initial","environment"})
            require(after["initial"] == current["initial"]+16 and after["environment"] == current["environment"]+16,"escrow transfer")
            require(current["environment"] == 0 and current["nodes"] == after["nodes"] and current["edges"] == after["edges"],"grant material shape")
            grants += 1
        elif kind == "shared_birth":
            cost = 3+len(current["genome"]["stages"])
            require(current["capital"] >= cost+8+24,"reproductive reserve")
            compare_scalars(current,after,{"capital","dissipated","transferred"})
            require(after["capital"] == current["capital"]-cost-8 and after["dissipated"] == current["dissipated"]+cost and after["transferred"] == current["transferred"]+8,"paid copy ledger")
            require(r["birth"]["inherited"] == current["genome"] and r["birth"]["copy_cost"] == cost,"copied genome/cost")
            transmitted = baseline if current["arm"] == "nonheritable" else current["genome"]
            child = r["child"]
            view(child)
            require(r["birth"]["transmitted"] == child["genome"] == transmitted,"transmission control")
            require(child["environment"] == 0 and child["initial"] == child["capital"] == 8 and child["parent"] == current["cell"] and child["generation"] == current["generation"]+1,"no free child resources")
            require(child["tick"] == current["tick"] and child["rewrites"] == child["dissipated"] == child["transferred"] == 0 and child["word_pos"] == "0", "child initial state")
            births.append((r["birth"],child))
            expected_nodes = copy.deepcopy(current["nodes"])
            wallet = view(current)[3]
            for n in expected_nodes:
                if n["id"] == wallet:
                    n["properties"] = [[1,after["capital"]]]
            require(after["nodes"] == expected_nodes and after["edges"] == current["edges"],"copy parent graph")
        elif kind == "death":
            due = 1+len(current["genome"]["stages"])
            require(current["capital"] < due and r["storage_due"] == due and r["amount"] == current["capital"],"unfunded death")
            compare_scalars(current,after,{"capital","dissipated"})
            require(after["capital"] == 0 and after["dissipated"] == current["dissipated"]+current["capital"],"death dissipation")
            expected_nodes=copy.deepcopy(current["nodes"])
            for n in expected_nodes:
                if n["id"] == view(current)[3]:n["properties"]=[[1,0]]
            require(after["nodes"]==expected_nodes and after["edges"]==current["edges"],"death graph")
            deaths += 1
        else:
            compare_scalars(current,after,{"tick","graph_step"})
            require(after["tick"] == current["tick"]+1 and after["graph_step"] == after["tick"] and current["nodes"] == after["nodes"] and current["edges"] == after["edges"],"unfed clock")
        view(after)
        current = copy.deepcopy(after)
        index += 1
    require(current == final,"population cell endpoint")
    return native,grants,births,deaths


def trajectory(initial,rows,final,baselines):
    world_view(initial)
    current=initial
    work=0
    deaths=0
    for row in rows:
        require(row["before"] == current,"world continuity")
        before=world_view(current);after=world_view(row["after"])
        tick=current["tick"]
        live=[i for i,m in before.items() if m["site"] is not None]
        survivors=[i for i in live if before[i]["state"]["capital"] >= 1+len(before[i]["state"]["genome"]["stages"])]
        requestors=[i for i in survivors if before[i]["state"]["capital"]-(1+len(before[i]["state"]["genome"]["stages"])) >= 1]
        rank=lambda ids,phase:sorted([[i,score(current["seed"],tick,phase,i)] for i in ids],key=lambda p:(p[1],p[0]))
        ranking=rank(requestors,0)
        count=min(len(ranking),current["parcels_per_tick"],current["reservoir"]//16)
        winners=[p[0] for p in ranking[:count]]
        require(row["resource_ranking"]==ranking and row["resource_winners"]==winners,"resource scheduler full enumeration")
        birth_ranking=rank(survivors,1)
        require(row["birth_ranking"]==birth_ranking,"birth scheduler full enumeration")
        ops={o["cell"]:o["receipts"] for o in row["operations"]}
        require(len(ops)==len(row["operations"]) and set(ops)==set(live)|{i for i in after if i not in before and ops.get(i)},"complete operation denominator")
        child_initials={}
        new_births=[]
        parent_birth_states={}
        for i in sorted(after):
            if i in before:
                start=before[i]["state"]
                root=i
                while after[root]["state"]["parent"] is not None:root=after[root]["state"]["parent"]
                baseline=baselines[root]
            else:
                require(i in child_initials,"child without preceding paid parent")
                start=child_initials[i]
                root=start["parent"]
                while after[root]["state"]["parent"] is not None:root=after[root]["state"]["parent"]
                baseline=baselines[root]
            n,g,bs,d=operations(start,ops.get(i,[]),after[i]["state"],baseline)
            work+=n;deaths+=d
            old_grants=before[i]["grants"] if i in before else 0
            require(after[i]["grants"]==old_grants+g and g==int(i in winners),"funded grant membership")
            if i in before:
                require(after[i]["endowment"]==before[i]["endowment"],"endowment immutable")
                require(after[i]["site"]==(before[i]["site"] if i in survivors else None),"site death/retention")
            for b,child in bs:
                require(child["cell"] not in child_initials,"duplicate birth")
                child_initials[child["cell"]]=child;new_births.append(b)
                parent_birth_states[i]=next(r["before"] for r in ops[i] if r["kind"]=="shared_birth")
            # Editing is required by the prospective clock and funding, not optional by outcome.
            records=ops.get(i,[])
            edits=[r for r in records if r["kind"] in ["endogenous_edit","external_edit"]]
            if i not in before:
                require(len(edits)==int(current["arm"]=="external"),"external birth edit count")
            elif i in survivors:
                funded=before[i]["state"]["capital"]-1-len(before[i]["state"]["genome"]["stages"])
                settles=[r for r in records if r["kind"]=="settle"]
                if settles:funded=settles[0]["native_after"]["capital"]
                expected=int(current["arm"] in ["endogenous","nonheritable"] and tick%3==0 and funded>=2)
                require(len(edits)==expected,"endogenous edit clock/funding")
        occupied={before[i]["site"] for i in survivors};refusals=[];accepted=[]
        for i,_ in birth_ranking:
            state=parent_birth_states.get(i,after[i]["state"])
            cost=3+len(state["genome"]["stages"])
            if state["capital"]<cost+8+24:refusals.append({"cell":i,"reason":"unfunded"});continue
            free=[s for s in range(current["sites"]) if s not in occupied]
            if not free:refusals.append({"cell":i,"reason":"no_site"});continue
            child=current["next_id"]+len(accepted)
            b=next((b for b in new_births if b["parent"]==i),None)
            require(b is not None and b["child"]==child and b["site"]==free[0] and b["tick"]==tick+1,"birth admission/identity/site")
            require(after[child]["site"]==free[0] and after[child]["endowment"]==8,"child physical site")
            accepted.append(b);occupied.add(free[0])
        require(len(accepted)==len(new_births) and row["birth_refusals"]==refusals,"copy refusal denominator")
        expected_refusals=[]
        if current["arm"] in ["endogenous","nonheritable"] and tick%3==0:
            for i in survivors:
                rec=ops[i];settles=[r for r in rec if r["kind"]=="settle"]
                funding=settles[0]["native_after"]["capital"] if settles else before[i]["state"]["capital"]-1-len(before[i]["state"]["genome"]["stages"])
                if funding<2:expected_refusals.append({"cell":i,"reason":"unfunded"})
        require(row["edit_refusals"]==expected_refusals,"edit refusal denominator")
        require(row["after"]["births"]==current["births"]+accepted and row["after"]["reservoir"]==current["reservoir"]-16*len(winners),"world bank/genealogy")
        for key in ["profile","arm","seed","initial_reservoir","parcels_per_tick","sites","initial_energy"]:
            require(row["after"][key]==current[key],"immutable world law "+key)
        require(row["after"]["tick"]==tick+1,"world clock")
        current=row["after"]
    require(current==final,"full world endpoint")
    return work,deaths


def audit(data,contract):
    require(data["profile"]==contract["profile"]==PROFILE and data["contract"]==contract,"population contract")
    for flag in FALSE_FLAGS:require(data[flag] is False,"population admission "+flag)
    require([c["name"] for c in data["cases"]]==[c["name"] for c in contract["cases"]],"case denominator")
    work=0;summary=[]
    for case,spec in zip(data["cases"],contract["cases"]):
        initial=case["initial"]
        require(initial["tick"]==0 and initial["reservoir"]==spec["reservoir"] and initial["arm"]==spec["arm"] and initial["seed"]==contract["development_seed"] and initial["sites"]==spec["sites"] and initial["parcels_per_tick"]==spec["parcels_per_tick"],"registered world input")
        require([m["state"]["genome"]["stages"] for m in initial["cells"]]==spec["genomes"] and all(m["endowment"]==spec["capital"] for m in initial["cells"]),"registered founders")
        baselines={i:{"stages":stages,"editor_mode":0} for i,stages in enumerate(spec["genomes"])}
        n,d=trajectory(initial,case["rows"],case["final"],baselines);work+=n
        require(len(case["rows"])==case["final"]["tick"]==spec["ticks"],"full horizon")
        summary.append({"name":case["name"],"births":len(case["final"]["births"]),"deaths":d,
            "alive":sum(m["site"] is not None for m in case["final"]["cells"]),"reservoir":case["final"]["reservoir"]})
    baselines={0:{"stages":[0],"editor_mode":0},1:{"stages":[0],"editor_mode":0}}
    order=data["order"]
    require(order["original"]==order["reverse"],"world order parity")
    for key in ["original","reverse","offset"]:
        rows=order[key];n,_=trajectory(rows[0]["before"],rows,rows[-1]["after"],baselines);work+=n
    orig=members(order["original"][-1]["after"]);off=members(order["offset"][-1]["after"])
    require(set(orig)==set(off),"allocation member parity")
    for i in orig:
        for field in ["capital","genome","word_pos","dissipated","transferred","tick","rewrites"]:
            require(orig[i]["state"][field]==off[i]["state"][field],"allocation outcome parity")
        require(orig[i]["site"]==off[i]["site"],"allocation site parity")
    cont=data["continuation"]
    checkpoint_view(cont["checkpoint"],baselines)
    require(cont["original"]==cont["restored"] and cont["checkpoint"]["state"]==cont["original"][0]["before"],"exact population continuation")
    for key in ["original","restored"]:
        rows=cont[key];n,_=trajectory(rows[0]["before"],rows,rows[-1]["after"],baselines);work+=n
    require(work==data["native_rewrites"] and work<=contract["limits"]["native_rewrites"],"population native work")
    require(summary[5]["alive"]==0 and summary[5]["births"]==summary[6]["births"]==summary[7]["births"]==0 and summary[8]["reservoir"]==0,"negative ecology controls")
    return {"profile":PROFILE,"engineering_fixture_pass":True,"cases":summary,"native_rewrites":work,
            "scheduler_reconstructed":True,"checkpoint_wire_and_indices_checked":True,"independent_rng_replay":False,"independent_allocator_replay":False,**{flag:False for flag in FALSE_FLAGS}}


if __name__=="__main__":
    print(json.dumps(audit(json.loads(Path(sys.argv[1]).read_text()),json.loads(Path(sys.argv[2]).read_text())),indent=2,sort_keys=True))
