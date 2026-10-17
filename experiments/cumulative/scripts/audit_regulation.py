"""Independent policy, full-opportunity, graph-edit, inheritance and budget audit.
Native chemistry is checked by the preserved integer-law auditor after a checked
projection of invariant editor-policy properties. No native code is invoked.
"""
import copy
import json
import sys
from pathlib import Path
from audit_readiness import require,view,handle,compare_scalars,FALSE_FLAGS
from audit_population import operations

PROFILE="cumulative-editor-regulation-readiness-v1"


def project(core):
    c=copy.deepcopy(core);editor=handle(c["editor"])
    for n in c["nodes"]:
        if n["id"]==editor:n["properties"]=[p for p in n["properties"] if p[0] not in [5,6]]
    return c


def program(p):
    require(set(p)=={"genes","next_serial","enabled","mask"},"program field set")
    require(all(type(p[k]) is int for k in ["next_serial","enabled","mask"]),"program integer policy")
    require(0<=p["enabled"]<=1 and 0<=p["mask"]<=31 and 1<=p["next_serial"]<=32 and 1<=len(p["genes"])<=8,"program domain")
    require(all(len(g)==2 and all(type(x) is int for x in g) and 0<=g[0]<p["next_serial"] and -1<=g[1]<=4 for g in p["genes"]),"stable gene domain")
    require(p["genes"]==sorted(p["genes"]) and len({g[0] for g in p["genes"]})==len(p["genes"]),"stable serial ordering")


def state(s):
    require(set(s)=={"profile","core","program","baseline","reservoir","initial_reservoir","endowment","grants","alive","attempts"} and s["profile"]==PROFILE,"regulated state fields")
    c=s["core"];_,genes,parcels,_,editor=view(project(c))
    require(not parcels and c["environment"]==0 and c["genome"]["editor_mode"]==0,"regulated boundary core")
    props=dict(next(n["properties"] for n in c["nodes"] if n["id"]==editor))
    require(set(props)=={4,5,6} and all(type(v) is int for v in props.values()),"policy property schema")
    p={"genes":[[g[0],g[2]] for g in genes],"next_serial":c["next_serial"],"enabled":props[5],"mask":props[6]}
    program(p);program(s["baseline"]);require(s["program"]==p,"graph-derived full program")
    for k in ["reservoir","initial_reservoir","endowment","grants","attempts"]:require(type(s[k]) is int and s[k]>=0,"metadata integer "+k)
    require(type(s["alive"]) is bool and (s["alive"] or c["capital"]==0),"life/energy")
    require(c["tick"]<=16 and c["next_serial"]<=32 and s["attempts"]<=32 and s["endowment"]<=64 and s["grants"]<=16,"regulated caps")
    require(s["reservoir"]<=s["initial_reservoir"]<=512 and s["reservoir"]%16==s["initial_reservoir"]%16==0,"finite reservoir")
    require(s["initial_reservoir"]-s["reservoir"]==16*s["grants"] and c["initial"]==s["endowment"]+16*s["grants"],"source ledger")
    require(s["reservoir"]+c["capital"]+c["dissipated"]+c["transferred"]==s["initial_reservoir"]+s["endowment"],"compartment conservation")
    return p


def catalog(s,external,diagnostic):
    p=s["program"];admitted=[];restrictions=[]
    for family in range(5):
        reason=None
        if not external and not diagnostic and not p["mask"]&(1<<family):reason="masked"
        elif family==0 and len(p["genes"])==8:reason="gene_cap"
        elif family==0 and p["next_serial"]==32:reason="serial_cap"
        elif family==2 and len(p["genes"])==1:reason="last_gene"
        targets=[i for i,stage in p["genes"] if 0<=stage<4] if family in [0,1] else [i for i,_ in p["genes"]] if family==2 else [1-p["enabled"]] if family==3 else list(range(32))
        if reason:restrictions.append([family,reason])
        elif not targets:restrictions.append([family,"no_executable_gene"])
        else:admitted.append([family,targets])
    return admitted,restrictions


def refusal(s,external,diagnostic,probe,ops):
    if not s["alive"]:return "dead"
    arm=s["core"]["arm"]
    if not diagnostic and ((external and arm!="external") or (not external and arm in ["external","frozen"])):return "arm"
    if not external and not diagnostic and s["program"]["enabled"]==0:return "disabled"
    if s["core"]["capital"]<2:return "unfunded"
    if not ops:return "no_opportunities"
    if probe is not None and not any(f==probe[0] and probe[1] in targets for f,targets in ops):return "probe_unavailable"
    return None


def edit_native(current,selection,frame):
    records=frame["receipts"];require(len(records)==1,"single native edit receipt")
    r=records[0];c=current["core"];p=current["program"];family,target=selection["selected"]
    require(r["kind"]=="regulated_edit" and r["before"]==c and r["rule"]==f"regulated-{family}-{target}","selected native rule identity")
    require(r["extra"]=={"family":family,"target":target,"external":selection["external"],"probe":selection["probe"] is not None,"diagnostic_restore":selection["diagnostic"]},"native invocation provenance")
    nodes,genes,_,wallet,editor=view(project(c));expected=copy.deepcopy(nodes)
    bound=[handle(n) for n in r["bound"]];require(bound[:2]==[wallet,editor],"local editor authority")
    after=r["native_after"];require(c["capital"]>=2,"native edit funded")
    expected[wallet]["properties"]=[[1,c["capital"]-2]]
    actual_nodes={n["id"]:n for n in after["nodes"]};editor_props=dict(next(n["properties"] for n in c["nodes"] if n["id"]==editor))
    expected[editor]["properties"]=sorted(editor_props.items());expected[editor]["properties"]=[list(x) for x in expected[editor]["properties"]]
    created=[handle(n) for n in r["created"]];deleted=[handle(n) for n in r["deleted"]]
    # The engine removes matched edges and recreates the RHS. Preserve identity
    # only for unmatched edges; independently require the complete RHS semantics.
    matched_endpoints=[[wallet,editor]]+([[wallet,bound[2]]] if family<3 else [])
    matched={e["id"] for e in c["edges"] if e["nodes"] in matched_endpoints}
    edge_map={e["id"]:e for e in c["edges"] if e["id"] not in matched}
    replacements=[(1001,[wallet,editor])]
    if family<3:
        require(len(bound)==3,"gene edit binding")
        serial,gid,old=next(g for g in genes if g[0]==target)
        require(bound[2]==gid,"semantic target binding")
        if family in [0,1]:require(0<=old<4,"executable mutation source")
        if family==0:
            require(len(created)==1 and not deleted and created[0] not in nodes and len(genes)<8 and p["next_serial"]<32 and r["effects"]==4,"bounded native duplication")
            newstage=dict(actual_nodes[created[0]]["properties"])[2];require(newstage-old in [-1,0,1],"blind duplicate jitter")
            expected[created[0]]={"id":created[0],"label":200,"born":c["tick"],"properties":[[2,newstage],[3,p["next_serial"]]]}
            replacements.extend([(1000,[wallet,gid]),(1000,[wallet,created[0]])])
        elif family==1:
            require(not created and not deleted and r["effects"]==2,"native retune event")
            newstage=dict(actual_nodes[gid]["properties"])[2];require(newstage-old in [-1,0,1],"blind retune jitter")
            expected[gid]["properties"]=[[2,newstage],[3,serial]]
            replacements.append((1000,[wallet,gid]))
        else:
            require(not created and deleted==[gid] and len(genes)>1 and r["effects"]==1,"bounded native deletion")
            del expected[gid]
    else:
        require(bound==[wallet,editor] and not created and not deleted and r["effects"]==2,"policy edit event")
        if family==3:require(target==1-p["enabled"],"toggle target");editor_props[5]=target
        else:require(0<=target<=31,"mask target");editor_props[6]=target
        expected[editor]["properties"]=[list(x) for x in sorted(editor_props.items())]
    # Restore policy properties on every expected node, which legacy view projects away.
    require(actual_nodes==expected,"exact regulated native node changes")
    actual_edges={e["id"]:e for e in after["edges"]}
    require(not matched.intersection(actual_edges) and all(actual_edges.get(i)==e for i,e in edge_map.items()),"matched replacement and unmatched edge identity")
    rhs=[e for i,e in actual_edges.items() if i not in edge_map]
    require(sorted((e["label"],e["nodes"]) for e in rhs)==sorted(replacements),"exact regulated RHS ownership")
    compare_scalars(c,after,{"capital","rewrites","word_pos"})
    require(after["capital"]==c["capital"]-2 and after["rewrites"]==c["rewrites"]+1 and int(after["word_pos"])>=int(c["word_pos"]),"native edit payment/work/cursor")
    view(project(after),balance=False,transient=family==0)
    final=copy.deepcopy(after);final["dissipated"]+=2;final["next_serial"]+=int(family==0)
    outer=copy.deepcopy(current);outer["core"]=final
    gs=view(project(final))[1];properties=dict(expected[editor]["properties"])
    outer["program"]={"genes":[[g[0],g[2]] for g in gs],"next_serial":final["next_serial"],"enabled":properties[5],"mask":properties[6]}
    require(frame["after"]==outer,"regulated edit boundary");state(outer)
    return outer,family


def trajectory(initial,frames,final,probes=False,restore=False):
    current=copy.deepcopy(initial);state(current);work=0;families=[];births=[];pending=None
    for frame in frames:
        kind=frame["kind"]
        if kind=="native_edit":
            require(pending is not None,"orphan native edit")
            current,family=edit_native(current,pending,frame);pending=None;work+=1;families.append(family);continue
        require(pending is None and frame["before"]==current,"regulated frame continuity")
        after=frame["after"];state(after)
        if kind in ["edit_selection","edit_refusal"]:
            external,diagnostic,probe=frame["external"],frame["diagnostic"],frame["probe"]
            require(type(external) is bool and type(diagnostic) is bool and (probe is None or probes),"fixture-only authored probe")
            require(not diagnostic or (restore and probe==[3,1] and current["program"]["enabled"]==0),"fixture-only restoration authority")
            ops,restrictions=catalog(current,external,diagnostic)
            require(frame["opportunities"]==ops and frame["restrictions"]==restrictions,"complete permission and target enumeration")
            reason=refusal(current,external,diagnostic,probe,ops)
            expected=copy.deepcopy(current);expected["attempts"]+=1
            if kind=="edit_refusal":
                require(reason==frame["reason"] and reason is not None and after==expected,"unchanged paid/RNG refusal")
            else:
                require(reason is None,"native editing eligibility")
                f,t=frame["selected"];require(any(f==k and t in ts for k,ts in ops),"selected family/target membership")
                require(frame["binding_count"]==sum(len(ts) for _,ts in ops)<=64,"complete binding denominator")
                if probe is not None:require(probe==[f,t] and after==expected,"authored selection provenance")
                else:
                    expected["core"]["word_pos"]=after["core"]["word_pos"]
                    require(int(after["core"]["word_pos"])>=int(current["core"]["word_pos"]) and after==expected,"blind selection cursor-only change")
                pending=frame
        elif kind=="resource_tick":
            c=current["core"];paid=current["alive"] and c["capital"]>=1+len(current["program"]["genes"])
            grant=paid and current["reservoir"]>=16 and c["capital"]-1-len(current["program"]["genes"])>=1
            records=copy.deepcopy(frame["receipts"])
            if current["alive"]:require(records[0]["kind"]==("storage" if paid else "death"),"mandatory storage/death")
            else:require(len(records)==1 and records[0]["kind"]=="idle_boundary","retained dead tick")
            for r in records:
                for key in ["before","after","native_after"]:
                    if key in r:
                        raw=r[key];ed=handle(raw["editor"]);props=dict(next(n["properties"] for n in raw["nodes"] if n["id"]==ed))
                        require(props[5]==current["program"]["enabled"] and props[6]==current["program"]["mask"],"chemistry cannot edit policy")
                        r[key]=project(raw)
            n,g,b,d=operations(project(c),records,project(after["core"]),{"stages":[],"editor_mode":0});work+=n
            require(g==int(grant) and not b and d==int(current["alive"] and not paid),"resource opportunity and death denominator")
            require(after["core"]["tick"]==c["tick"]+1 and after["core"]["tick"]==after["core"]["graph_step"],"resource clock")
            for key in ["profile","program","baseline","initial_reservoir","endowment","attempts"]:require(after[key]==current[key],"immutable resource metadata "+key)
            require(after["alive"]==paid and after["reservoir"]==current["reservoir"]-16*int(grant) and after["grants"]==current["grants"]+int(grant),"finite resource/life metadata")
        elif kind in ["birth","birth_refusal"]:
            cost=3+len(current["program"]["genes"]);eligible=current["alive"] and current["core"]["capital"]>=cost+8+24
            if kind=="birth_refusal":require(not eligible and after==current and frame["reason"]==("unfunded" if current["alive"] else "dead"),"unchanged birth refusal")
            else:
                require(eligible and current["core"]["transferred"]==0 and frame["copy_cost"]==cost and frame["inherited"]==current["program"],"funded copying/reserve")
                wanted=current["baseline"] if current["core"]["arm"]=="nonheritable" else current["program"]
                require(frame["transmitted"]==wanted,"full regulated transmission semantics")
                child=frame["child"];state(child);ch=child["core"];c=current["core"]
                require(child["program"]==wanted and child["baseline"]==current["baseline"] and child["alive"] and child["reservoir"]==child["initial_reservoir"]==child["grants"]==child["attempts"]==0 and child["endowment"]==8,"no free birth resource/policy reset")
                require(ch["parent"]==c["cell"] and ch["cell"]==c["cell"]+1 and ch["generation"]==c["generation"]+1 and ch["seed"]==(c["seed"]+1)%2**64 and ch["arm"]==c["arm"] and ch["tick"]==c["tick"],"regulated paid ancestry")
                require(ch["capital"]==ch["initial"]==8 and ch["environment"]==ch["dissipated"]==ch["transferred"]==ch["rewrites"]==0 and ch["word_pos"]=="0" and all(n["born"]==ch["tick"] for n in ch["nodes"]),"child initial budget/clock")
                expected=copy.deepcopy(current);expected["core"]["capital"]-=cost+8;expected["core"]["dissipated"]+=cost;expected["core"]["transferred"]+=8
                w=handle(c["wallet"])
                next(n for n in expected["core"]["nodes"] if n["id"]==w)["properties"]=[[1,expected["core"]["capital"]]]
                require(after==expected,"exact regulated birth debit");births.append(child)
        else:raise ValueError("unknown regulated frame "+kind)
        current=after
    require(pending is None and current==final,"complete regulated endpoint")
    return work,families,births


def checkpoint(cp):
    s=cp["state"];state(s);c=s["core"];graph=cp["graph"];nodes=[];edges=[];free=[]
    for field in ["nodes","edges"]:
        gaps=[]
        for i,slot in enumerate(graph[field]):
            if slot["value"] is None:
                if i:gaps.append(i)
                continue
            require(slot["version"]%2==1,"live checkpoint generation")
            value=slot["value"];ident=i|(slot["version"]<<32)
            if field=="nodes":
                require(value["energy"]==0 and all(set(v)=={"Int"} and type(v["Int"]) is int for _,v in value["properties"]),"checkpoint typed node")
                nodes.append({"id":ident,"label":value["label"],"born":value["generation"],"properties":sorted([[k,v["Int"]] for k,v in value["properties"]])})
            else:edges.append({"id":ident,"label":value["label"],"nodes":[handle(n) for n in value["nodes"]]})
        free.append(gaps)
    require(sorted(nodes,key=lambda n:n["id"])==c["nodes"] and sorted(edges,key=lambda e:e["id"])==c["edges"] and graph["step"]==c["graph_step"],"checkpoint graph projection")
    require(all(len(actual)==len(set(actual)) and sorted(actual)==expected for actual,expected in zip(c["allocator"],free)),"checkpoint allocator membership")
    for n in nodes:
        slot=graph["node_to_edges"][n["id"]&0xffffffff]
        require(slot["version"]==n["id"]>>32 and sorted(handle(e) for e in slot["value"])==sorted(e["id"] for e in edges if n["id"] in e["nodes"]),"checkpoint incidence")
    require({str(e["label"]) for e in edges}<=set(graph["label_index"]),"checkpoint missing label index")
    for label,refs in graph["label_index"].items():require(sorted(handle(e) for e in refs)==sorted(e["id"] for e in edges if e["label"]==int(label)),"checkpoint label index")


def audit(data,contract):
    require(data["profile"]==contract["profile"]==PROFILE and data["contract"]==contract,"regulated contract")
    for d in [data,contract]:
        for flag in FALSE_FLAGS:require(d[flag] is False,"regulated admission "+flag)
    require([c["name"] for c in data["cases"]]==[c["name"] for c in contract["cases"]],"regulated case denominator")
    work=0;families=[];summary=[]
    def attempt_end(frames,i):
        require(frames[i]["kind"] in ["edit_selection","edit_refusal"],"one edit opportunity per declared phase")
        return i+(2 if frames[i]["kind"]=="edit_selection" else 1)
    for case,spec in zip(data["cases"],contract["cases"]):
        s=case["initial"];state(s);p=s["program"]
        require(s["core"]["arm"]==spec["arm"] and s["core"]["seed"]==contract["development_seed"] and s["endowment"]==s["core"]["capital"]==spec["capital"] and s["reservoir"]==s["initial_reservoir"]==spec["reservoir"] and s["core"]["tick"]==s["grants"]==s["attempts"]==0,"registered regulated input")
        require(p==s["baseline"]=={"genes":[[i,stage] for i,stage in enumerate(spec["stages"])],"next_serial":spec.get("next_serial",len(spec["stages"])),"enabled":spec["enabled"],"mask":spec["mask"]},"registered initial program")
        n,f,b=trajectory(s,case["frames"],case["final"]);work+=n;families+=f
        frames=case["frames"];index=attempt_end(frames,0) if spec["pre_edit"] else 0
        for _ in range(spec["ticks"]):
            require(frames[index]["kind"]=="resource_tick","registered tick phase");index=attempt_end(frames,index+1)
        if spec["birth"]:
            require(frames[index]["kind"] in ["birth","birth_refusal"],"registered birth phase");index+=1
        require(index==len(frames) and case["final"]["core"]["tick"]==spec["ticks"],"all phases/full horizon")
        if case["child"] is None:require(not b,"missing child route")
        else:
            child=case["child"];require(len(b)==1 and child["initial"]==b[0],"child phase continuity")
            n,f,later=trajectory(child["initial"],child["frames"],child["final"]);work+=n;families+=f
            require(not later and attempt_end(child["frames"],0)==len(child["frames"]),"one child edit opportunity")
            require(child["frames"][0]["external"]==(spec["arm"]=="external"),"child external mutation timing")
            require(case["final"]["reservoir"]+case["final"]["core"]["capital"]+case["final"]["core"]["dissipated"]+child["final"]["core"]["capital"]+child["final"]["core"]["dissipated"]==s["endowment"]+s["initial_reservoir"],"parent-child global conservation")
        summary.append({"name":spec["name"],"alive":case["final"]["alive"],"program":case["final"]["program"],"edit_refusals":sum(f["kind"]=="edit_refusal" for f in frames),"births":len(b)})
    causal=data["causal"]
    require(causal["initial"]["program"]=={"genes":[[0,0]],"next_serial":1,"enabled":1,"mask":31} and causal["initial"]["endowment"]==40 and causal["initial"]["reservoir"]==0,"declared causal setup")
    for branch in ["disabled","sham"]:
        n,f,b=trajectory(causal["initial"],causal[branch]["frames"],causal[branch]["final"],probes=True);work+=n;families+=f;require(not b,"causal birth separation")
    disabled=causal["disabled"];sham=causal["sham"];restored=causal["restored"]
    require([len(causal[k]["frames"]) for k in ["disabled","sham","restored"]]==[3,4,4],"fixed causal opportunity denominator")
    require(sham["frames"][2]["probe"]==restored["frames"][2]["probe"]==disabled["frames"][2]["probe"]==[0,0],"declared causal duplication target")
    require(disabled["frames"][0]["probe"]==[3,0] and disabled["frames"][-1]["kind"]=="edit_refusal" and disabled["frames"][-1]["reason"]=="disabled","native disable prevents duplication")
    require(sham["frames"][0]["probe"]==[4,31] and len(sham["final"]["program"]["genes"])==2,"paid neutral sham allows duplication")
    require(restored["initial"]==disabled["frames"][1]["after"],"restoration fork at disabled state")
    n,f,b=trajectory(restored["initial"],restored["frames"],restored["final"],probes=True,restore=True);work+=n;families+=f
    require(not b and restored["frames"][0]["diagnostic"] is True and restored["final"]["program"]["enabled"]==1 and len(restored["final"]["program"]["genes"])==2,"paid authored restoration restores permission")
    continuation=data["continuation"];setup=continuation["setup"]
    n,f,b=trajectory(setup["initial"],setup["frames"],setup["final"],probes=True);work+=n;families+=f;require(not b,"setup birth separation")
    require([f["probe"] for f in setup["frames"] if f["kind"]=="edit_selection"]==[[2,1],[4,17],[0,0]],"declared hole setup")
    require(len(setup["frames"])==7 and setup["frames"][0]["kind"]=="resource_tick" and setup["initial"]["core"]["tick"]==0 and setup["initial"]["program"]=={"genes":[[0,0],[1,1],[2,2]],"next_serial":3,"enabled":1,"mask":20} and setup["initial"]["endowment"]==40 and setup["initial"]["initial_reservoir"]==16,"registered continuation setup")
    checkpoint(continuation["checkpoint"]);require(continuation["checkpoint"]["state"]==setup["final"],"checkpoint setup endpoint")
    require(continuation["original"]==continuation["restored"] and continuation["final"]==continuation["restored_final"],"exact regulated continuation")
    for branch in ["original","restored"]:
        n,f,b=trajectory(setup["final"],continuation[branch],continuation["final"]);work+=n;families+=f
        frames=continuation[branch];require(frames[0]["kind"]=="birth","continuation paid birth phase");index=1
        for _ in range(2):
            require(frames[index]["kind"]=="resource_tick","declared continuation tick");index=attempt_end(frames,index+1)
        require(index==len(frames) and continuation["final"]["core"]["tick"]==3,"complete continuation horizon")
        require(len(b)==1 and b[0]==continuation["child"] and [g[0] for g in b[0]["program"]["genes"]]==[0,2,3] and b[0]["program"]["next_serial"]==4,"serial holes copied without reuse")
    order=data["order"];require(order["normal"]==order["reverse"],"regulated candidate order parity")
    require(order["initial"]==by_case_initial(data,"mask"),"order uses registered mask case")
    for branch in ["normal","reverse"]:
        n,f,b=trajectory(order["initial"],order[branch],order["final"]);work+=n;families+=f;require(not b,"order birth separation")
        require(sum(f["kind"]=="resource_tick" for f in order[branch])==2 and order["final"]["core"]["tick"]==2,"order full horizon")
    offset=data["offset"];n,f,b=trajectory(offset["initial"],offset["frames"],offset["final"]);work+=n;families+=f;require(not b,"offset birth separation")
    require(offset["initial"]["program"]==order["initial"]["program"] and offset["initial"]["core"]["tick"]==0 and offset["initial"]["endowment"]==40 and offset["initial"]["reservoir"]==32 and offset["initial"]["core"]["arm"]=="endogenous" and offset["initial"]["core"]["seed"]==contract["development_seed"],"registered allocation input")
    require(sum(f["kind"]=="resource_tick" for f in offset["frames"])==2,"allocation full horizon")
    for field in ["capital","dissipated","transferred","tick","word_pos","rewrites","next_serial"]:require(offset["final"]["core"][field]==order["final"]["core"][field],"allocation projection parity")
    require(offset["final"]["program"]==order["final"]["program"],"allocation policy/program parity")
    by_name={c["name"]:c for c in data["cases"]}
    require(by_name["heritable"]["child"]["frames"][0]["reason"]=="disabled" and by_name["nonheritable"]["child"]["frames"][0]["kind"]=="edit_selection","inherited vs reset editor effect")
    require(set(families)==set(range(5)) and work==data["native_rewrites"]<=contract["limits"]["native_rewrites"],"all native families/work")
    return {"profile":PROFILE,"engineering_fixture_pass":True,"cases":summary,"native_rewrites":work,"native_families":sorted(set(families)),
        "independent_policy_binding_and_budget_audit":True,"independent_rng_replay":False,"independent_allocator_replay":False,**{f:False for f in FALSE_FLAGS}}


def by_case_initial(data,name):
    return next(c["initial"] for c in data["cases"] if c["name"]==name)


if __name__=="__main__":
    print(json.dumps(audit(json.loads(Path(sys.argv[1]).read_text()),json.loads(Path(sys.argv[2]).read_text())),indent=2,sort_keys=True))
