"""Independent integer-law and graph/binding auditor. Never invokes Rust.

It reconstructs legal transitions and credit, but does not independently replay
ChaCha8 draws or predict slotmap allocation. Those boundaries are explicit.
"""
import copy
import json
import sys
from pathlib import Path

PROFILE = "cumulative-readiness-v1"
VALUES = [0, 4, 8, 14, 16]
FALSE_FLAGS = ["readiness_gate_complete", "scientific_acceptance", "evolution_demonstrated", "campaign_launch_authorized"]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def handle(n):
    require(set(n) == {"idx", "version"}, "handle fields")
    require(type(n["idx"]) is int and type(n["version"]) is int, "handle types")
    return n["idx"] | (n["version"] << 32)


def view(s, balance=True, transient=False):
    require(s["arm"] in ["endogenous", "external", "nonheritable", "frozen"], "arm")
    for k in ["capital", "environment", "dissipated", "transferred", "initial", "tick", "rewrites", "next_serial", "cell", "generation", "seed"]:
        require(type(s[k]) is int and s[k] >= 0, "state integer " + k)
    require(s["graph_step"] == s["tick"], "graph clock")
    require(s["rewrites"] <= 4000 and len(s["nodes"]) <= 16 and len(s["edges"]) <= 16, "work/store caps")
    require(s["word_pos"].isdigit() and int(s["word_pos"]) < 2**68, "RNG cursor")
    nodes = {n["id"]: copy.deepcopy(n) for n in s["nodes"]}
    require(len(nodes) == len(s["nodes"]), "duplicate node")
    wallet, editor = handle(s["wallet"]), handle(s["editor"])
    require(nodes[wallet]["label"] == 100 and nodes[editor]["label"] == 300, "wallet/editor identity")
    genes, parcels, props = [], [], {}
    for ident, n in nodes.items():
        require(type(ident) is int and 0 < ident < 2**64, "node handle")
        require(type(n["born"]) is int and 0 <= n["born"] <= s["tick"], "birth stamp")
        require(len(dict(n["properties"])) == len(n["properties"]), "property duplicate")
        p = dict(n["properties"])
        require(all(type(k) is int and type(v) is int for k, v in n["properties"]), "typed integer properties")
        props[ident] = p
        if ident == wallet:
            require(set(p) == {1} and p[1] == s["capital"], "wallet accounting")
        elif ident == editor:
            require(set(p) == {4} and p[4] in [0, 1], "editor schema")
        elif n["label"] == 200:
            require(set(p) == {2, 3} and -1 <= p[2] <= 4 and 0 <= p[3] < s["next_serial"] + int(transient), "gene schema")
            genes.append((p[3], ident, p[2]))
        else:
            require(0 <= n["label"] <= 4 and not p, "parcel schema")
            parcels.append(ident)
    genes.sort()
    require(1 <= len(genes) <= 8 and len(set(g[0] for g in genes)) == len(genes), "gene count/serial")
    require(len(parcels) <= 1, "parcel count")
    require(s["genome"] == {"stages": [g[2] for g in genes], "editor_mode": props[editor][4]}, "graph-derived genome")
    edge_ids, owned = set(), set()
    for e in s["edges"]:
        require(e["id"] not in edge_ids, "edge duplicate")
        edge_ids.add(e["id"])
        require(len(e["nodes"]) == 2 and e["nodes"][0] == wallet, "local ownership")
        target = e["nodes"][1]
        require(target in nodes and target != wallet and target not in owned, "ownership target")
        owned.add(target)
        expected = 1001 if target == editor else 1000 if nodes[target]["label"] == 200 else 1002
        require(e["label"] == expected, "ownership role")
    require(owned == set(nodes) - {wallet}, "complete ownership")
    if balance:
        require(s["capital"] + s["environment"] + s["dissipated"] + s["transferred"] + 16 * len(parcels) == s["initial"], "energy conservation")
    return nodes, genes, parcels, wallet, editor


def compare_scalars(a, b, allowed):
    excluded = {"nodes", "edges", "allocator", "genome"} | set(allowed)
    require(set(a) == set(b), "state field set")
    for k in set(a) - excluded:
        require(a[k] == b[k], "unexpected change " + k)


def trace(initial, records, final):
    current = copy.deepcopy(initial)
    view(current)
    native = 0
    births = []
    for r in records:
        kind = r["kind"]
        if kind in ["boundary", "edit_boundary"]:
            require(r["after"] == current, "accounting boundary")
            view(current)
            continue
        require(r["before"] == current, "receipt continuity " + kind)
        nodes, genes, parcels, wallet, editor = view(current)
        after = r.get("native_after", r.get("after"))
        require(after is not None, "missing after")
        expected_nodes = copy.deepcopy(nodes)
        if kind == "storage":
            amount = 1 + len(genes)
            require(r["amount"] == amount and current["capital"] >= amount, "storage payment")
            compare_scalars(current, after, {"capital", "dissipated"})
            expected_nodes[wallet]["properties"] = [[1, current["capital"] - amount]]
            require(after["capital"] == current["capital"] - amount and after["dissipated"] == current["dissipated"] + amount, "storage ledger")
        elif kind == "supply":
            require(not parcels and current["environment"] >= 16, "finite supply")
            compare_scalars(current, after, {"environment"})
            require(after["environment"] == current["environment"] - 16, "supply debit")
            new = set(n["id"] for n in after["nodes"]) - set(nodes)
            require(len(new) == 1, "supply material")
            ident = new.pop()
            expected_nodes[ident] = {"id": ident, "label": 0, "born": current["tick"], "properties": []}
        elif kind == "selection":
            require(r["opportunities"] == [g[0] for g in genes if 0 <= g[2] < 4] and r["selected_serial"] in r["opportunities"], "selection opportunities")
            require(r["external"] == (current["arm"] == "external"), "selection provenance")
            compare_scalars(current, after, {"word_pos"})
            require(int(after["word_pos"]) > int(current["word_pos"]), "selection draw consumption")
        elif kind in ["metabolic", "settle", "endogenous_edit", "external_edit"]:
            native += 1
            compare_scalars(current, after, {"capital", "rewrites", "word_pos"})
            require(after["rewrites"] == current["rewrites"] + 1, "rewrite count")
            require(int(after["word_pos"]) >= int(current["word_pos"]), "RNG rewind")
            bound = [handle(n) for n in r["bound"]]
            if kind == "metabolic":
                require(len(parcels) == 1 and len(bound) == 3 and bound[0] == wallet and bound[2] == parcels[0], "metabolic binding")
                source = next(g for g in genes if g[1] == bound[1])
                stage = nodes[parcels[0]]["label"]
                opportunities = [g[0] for g in genes if g[2] == stage and stage < 4]
                require(opportunities and source[0] == min(opportunities) and r["extra"] == {"serial": source[0], "stage": stage, "opportunities": opportunities}, "metabolic enumeration")
                require(r["rule"] == f"metabolize-{source[0]}" and r["effects"] == 1 and not r["created"] and not r["deleted"], "reaction event")
                require(current["capital"] >= 1, "reaction funded")
                expected_nodes[parcels[0]]["label"] += 1
                expected_nodes[wallet]["properties"] = [[1, current["capital"] - 1]]
                require(after["capital"] == current["capital"] - 1, "reaction debit")
            elif kind == "settle":
                require(len(parcels) == 1 and bound == [wallet, parcels[0]], "settlement binding")
                stage = nodes[parcels[0]]["label"]
                payout = VALUES[stage]
                require(r["rule"] == "settle" and r["extra"] == {"stage": stage, "payout": payout}, "physical settlement")
                require(r["effects"] == 1 and not r["created"] and [handle(n) for n in r["deleted"]] == parcels, "settlement consumption")
                del expected_nodes[parcels[0]]
                expected_nodes[wallet]["properties"] = [[1, current["capital"] + payout]]
                require(after["capital"] == current["capital"] + payout, "settlement credit")
            else:
                require((kind == "external_edit" and current["arm"] == "external") or (kind == "endogenous_edit" and current["arm"] in ["endogenous", "nonheritable"]), "edit arm")
                require(len(bound) == 3 and bound[:2] == [wallet, editor] and len(genes) < 8 and current["capital"] >= 2, "edit binding/funding")
                source = next(g for g in genes if g[1] == bound[2])
                opportunities = [g[0] for g in genes if 0 <= g[2] < 4]
                require(r["extra"]["opportunities"] == opportunities and r["extra"]["selected_serial"] == source[0], "editor opportunities")
                require(r["rule"] == "duplicate-gene" and r["effects"] == 4 and len(r["created"]) == 1 and not r["deleted"], "native editor event")
                created = handle(r["created"][0])
                require(created not in nodes, "new gene identity")
                newnode = next(n for n in after["nodes"] if n["id"] == created)
                newstage = dict(newnode["properties"])[2]
                delta = newstage - source[2]
                mode = current["genome"]["editor_mode"]
                require(delta == 1 if mode == 1 else delta in [-1, 0, 1], "editor primitive")
                expected_nodes[created] = {"id": created, "label": 200, "born": current["tick"], "properties": [[2, newstage], [3, current["next_serial"]]]}
                expected_nodes[wallet]["properties"] = [[1, current["capital"] - 2]]
                require(after["capital"] == current["capital"] - 2, "edit debit")
                if r["extra"]["probe"]:
                    require(mode == 1, "authored probe classification")
        elif kind == "birth":
            require(not parcels and current["capital"] >= 11 + len(genes), "birth funding")
            cost = 3 + len(genes)
            require(r["copy_cost"] == cost and r["inherited"] == current["genome"], "copy cost/parent genome")
            compare_scalars(current, after, {"capital", "dissipated", "transferred"})
            expected_nodes[wallet]["properties"] = [[1, current["capital"] - cost - 8]]
            require(after["capital"] == current["capital"] - cost - 8 and after["transferred"] == current["transferred"] + 8 and after["dissipated"] == current["dissipated"] + cost, "birth ledger")
            child = r["child"]
            view(child)
            require(child["parent"] == current["cell"] and child["cell"] != current["cell"] and child["generation"] == current["generation"] + 1, "paid ancestry")
            require(child["capital"] == 8 and child["environment"] == 512 and child["initial"] == 520 and child["rewrites"] == child["tick"] == child["dissipated"] == child["transferred"] == 0, "diagnostic child initialization")
            transmitted = {"stages": [0], "editor_mode": 0} if current["arm"] == "nonheritable" else current["genome"]
            require(r["transmitted"] == child["genome"] == transmitted and child["arm"] == current["arm"], "heredity arm")
            births.append(child)
        else:
            raise ValueError("unknown receipt " + kind)
        require({n["id"]: n for n in after["nodes"]} == expected_nodes, "native node changes " + kind)
        view(after, balance=False, transient=kind in ["endogenous_edit", "external_edit"])
        current = copy.deepcopy(after)
        if kind == "metabolic":
            current["dissipated"] += 1
        elif kind == "settle":
            current["dissipated"] += 16 - r["extra"]["payout"]
            current["tick"] += 1
            current["graph_step"] += 1
        elif kind in ["endogenous_edit", "external_edit"]:
            current["dissipated"] += 2
            current["next_serial"] += 1
        view(current)
    require(current == final, "endpoint continuity")
    return native, births


def audit(data, contract):
    require(data["profile"] == PROFILE and data["contract"] == contract and contract["profile"] == PROFILE, "profile/contract")
    for flag in FALSE_FLAGS:
        require(data[flag] is False, "admission flag " + flag)
    expected_names = [c["name"] for c in contract["capacity_cases"]] + ["paid_accessibility", "arm-endogenous", "arm-external", "arm-nonheritable", "arm-frozen"]
    require([c["name"] for c in data["cases"]] == expected_names, "fixed case denominator")
    total = 0
    gains = {}
    for index, case in enumerate(data["cases"]):
        n, births = trace(case["initial"], case["receipts"], case["final"])
        total += n
        if index < 8:
            require(case["initial"]["genome"]["stages"] == contract["capacity_cases"][index]["stages"] and case["initial"]["capital"] == 40 and case["initial"]["tick"] == 0, "fixed capacity input")
            require(case["final"]["tick"] == 12 and not births, "full capacity horizon")
            gains[case["name"]] = case["final"]["capital"] - case["initial"]["capital"]
        if "child_receipts" in case:
            require(len(births) == 1, "arm birth count")
            require(case["child_initial"] == births[0], "child phase start")
            n, later = trace(births[0], case["child_receipts"], case["child_final"])
            require(not later and case["final"]["tick"] == case["child_final"]["tick"] == 4, "arm horizon")
            total += n
    require(gains == {"founder":12,"first":36,"second":84,"earlier_ablation":0,"later_ablation":36,"sham":84,"restoration":84,"redundant_bloat":72}, "headroom and causal controls")
    require(len(data["transmissions"]) == 3, "transmission denominator")
    previous_child = None
    for generation, t in enumerate(data["transmissions"], 1):
        if previous_child is not None:
            require(t["initial"] == previous_child, "lineage continuity")
        n, births = trace(t["initial"], t["receipts"], t["final"])
        require(len(births) == 1 and births[0]["generation"] == generation and births[0]["genome"]["stages"] == [0,1,2], "capacity transmission")
        previous_child = births[0]
        total += n
    cont = data["continuation"]
    require(cont["receipts"] == cont["restored_receipts"] and cont["final"] == cont["restored_final"], "exact continuation")
    n, _ = trace(cont["receipts"][0]["before"], cont["receipts"], cont["final"])
    total += cont["checkpoint"]["rewrites"] + 2*n
    order = data["order"]
    require(order["receipts"] == order["reverse_receipts"] and order["final"] == order["reverse_final"], "candidate-order parity")
    n, _ = trace(order["receipts"][0]["before"], order["receipts"], order["final"])
    total += 2*n
    offset = data["offset"]
    n, _ = trace(offset["receipts"][0]["before"], offset["receipts"], offset["final"])
    total += n
    for field in ["genome", "capital", "dissipated", "environment", "transferred", "rewrites", "tick"]:
        require(offset["final"][field] == order["final"][field], "allocation-offset parity")
    require(total == data["native_rewrites"] and total <= contract["limits"]["native_rewrites"], "complete native work")
    return {"profile":PROFILE,"engineering_fixture_pass":True,"capacity_net_gains":gains,
            "native_rewrites":total,"cases":len(expected_names),"paid_transmissions":3,
            "independent_rng_replay":False,"independent_allocator_replay":False,
            **{flag:False for flag in FALSE_FLAGS}}


if __name__ == "__main__":
    result = audit(json.loads(Path(sys.argv[1]).read_text()), json.loads(Path(sys.argv[2]).read_text()))
    print(json.dumps(result, indent=2, sort_keys=True))
