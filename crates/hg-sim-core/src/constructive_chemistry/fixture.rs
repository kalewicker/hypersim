//! Frozen-contract harness. Diagnostics deliberately remain outside world authority.
use super::*;
use rayon::prelude::*;
use std::time::Instant;

fn number(v: &Value) -> i64 {
    v.as_i64()
        .unwrap_or_else(|| v.as_str().unwrap().parse().unwrap())
}
fn unsigned(v: &Value) -> u64 {
    v.as_u64()
        .unwrap_or_else(|| v.as_str().unwrap().parse().unwrap())
}
fn resolved(contract: &Value, name: &str) -> Value {
    let f = &contract["fixtures"][name];
    let mut out = if let Some(base) = f["base"].as_str() {
        resolved(contract, base)
    } else {
        json!({})
    };
    for (k, v) in f.as_object().unwrap() {
        if k != "base" {
            out[k] = v.clone();
        }
    }
    if let Some(omitted) = out["omit_beads"].as_array().cloned() {
        out["beads"]
            .as_array_mut()
            .unwrap()
            .retain(|b| !omitted.contains(&b["id"]));
    }
    for (key, field) in [("set_activation", "activated"), ("set_site", "site")] {
        if let Some(changes) = out[key].as_object().cloned() {
            for b in out["beads"].as_array_mut().unwrap() {
                if let Some(change) = changes.get(&number(&b["id"]).to_string()) {
                    b[field] = change.clone();
                }
            }
        }
    }
    out
}
fn add_relation(g: &mut HypergraphStore, label: Label, a: NodeId, b: NodeId) {
    g.add_edge(HyperEdge::new(label, [a, b].into_iter().collect()));
}
fn raw(contract: &Value, name: &str) -> (HypergraphStore, u64, i64, [i64; 2], i64) {
    use PropertyValue::{Bool, Int};
    let f = resolved(contract, name);
    let mut g = HypergraphStore::new();
    let mut ids = BTreeMap::new();
    for s in 0..3 {
        ids.insert(
            s + 1,
            g.add_node(data(SITE, s + 1, 0, &[(SITE_INDEX, Int(s))])),
        );
    }
    add_relation(&mut g, ADJACENT, ids[&1], ids[&2]);
    add_relation(&mut g, ADJACENT, ids[&2], ids[&3]);
    for b in f["beads"].as_array().unwrap() {
        let id = number(&b["id"]);
        let species = ["A", "B", "C", "D", "F"]
            .iter()
            .position(|s| Some(*s) == b["species"].as_str())
            .unwrap() as i64;
        let h = g.add_node(data(
            BEAD,
            id,
            0,
            &[
                (SPECIES, Int(species)),
                (ACTIVE, Bool(b["activated"].as_bool().unwrap())),
            ],
        ));
        ids.insert(id, h);
        add_relation(&mut g, PLACED, h, ids[&(number(&b["site"]) + 1)]);
    }
    if let Some(bonds) = f["initial_bonds"].as_array() {
        for (i, b) in bonds.iter().enumerate() {
            let id = number(&f["initial_bond_ids"][i]);
            let h = g.add_node(data(
                BOND,
                id,
                0,
                &[(KIND, Int(0)), (PORT_A, Int(0)), (PORT_B, Int(0))],
            ));
            ids.insert(id, h);
            add_relation(&mut g, BOND_A, h, ids[&number(&b[0])]);
            add_relation(&mut g, BOND_B, h, ids[&number(&b[1])]);
        }
    }
    let revision = f.get("initial_revision").map(unsigned).unwrap_or(0);
    g.set_step(f.get("initial_graph_step").map(unsigned).unwrap_or(0));
    let frontier = f
        .get("semantic_id_frontier")
        .map(number)
        .unwrap_or(ids.keys().last().unwrap() + 1);
    let reservoir = [
        f.get("reservoir_active_fuel").map(number).unwrap_or(0),
        f.get("reservoir_inactive_fuel").map(number).unwrap_or(0),
    ];
    (
        g,
        revision,
        frontier,
        reservoir,
        f.get("initial_heat").map(number).unwrap_or(0),
    )
}
fn absent(w: &ChemistryWorld, id: i64) -> ChemicalHandle {
    w.handle(id).unwrap_or(ChemicalHandle {
        node: NodeId::default(),
        semantic_id: id,
    })
}
fn bond(w: &ChemistryWorld, b: &Value) -> ChemicalAction {
    ChemicalAction::FormBond {
        a: absent(w, number(&b[0])),
        b: absent(w, number(&b[1])),
        fuel: absent(w, number(&b[2])),
    }
}
fn record_error(
    log: &mut Vec<Value>,
    phase: &str,
    before: &Value,
    after: &Value,
    action: Value,
    error: impl std::fmt::Debug,
) {
    log.push(json!({"phase":phase,"action":action,"decision":"rejected","reason":format!("{error:?}"),"before":before,"after":after}));
}
fn run_action(
    w: &mut ChemistryWorld,
    a: ChemicalAction,
    log: &mut Vec<Value>,
) -> Result<ChemicalEvent, ChemicalRejection> {
    let before = w.observation();
    let result = prepare(w, &a).and_then(|p| commit(w, p));
    match &result {
        Ok(event)=>log.push(json!({"phase":"prepare_commit","action":a,"decision":"accepted","event":event,"before":before,"after":w.observation()})),
        Err(error)=>record_error(log,"prepare_commit",&before,&w.observation(),json!(a),error),
    }
    result
}
fn try_prepared(
    w: &mut ChemistryWorld,
    p: PreparedChemicalCommit,
    a: &ChemicalAction,
    log: &mut Vec<Value>,
) {
    let before = w.observation();
    match commit(w,p) {
        Ok(event)=>log.push(json!({"phase":"commit","action":a,"decision":"accepted","event":event,"before":before,"after":w.observation()})),
        Err(error)=>record_error(log,"commit",&before,&w.observation(),json!(a),error),
    }
}
fn fault_point(s: &str) -> FaultPoint {
    match s {
        "after_fuel_update" => FaultPoint::AfterFuelUpdate,
        "after_bond_node" => FaultPoint::AfterBondNode,
        "after_first_incidence" => FaultPoint::AfterFirstIncidence,
        "after_ledger_update" => FaultPoint::AfterLedgerUpdate,
        "before_publication" => FaultPoint::BeforePublication,
        _ => panic!("unknown frozen fault"),
    }
}
fn mutate_initial(
    g: &mut HypergraphStore,
    reservoir: &mut [i64; 2],
    frontier: &mut i64,
    mutation: &str,
) {
    use PropertyValue::{Bool, Int};
    let h = g.nodes().find(|(_, n)| integer(n, ID) == Ok(30)).unwrap().0;
    match mutation {
        "missing_property" => g
            .get_node_mut(h)
            .unwrap()
            .properties
            .retain(|(k, _)| *k != ACTIVE),
        "wrong_property_type" => g.get_node_mut(h).unwrap().set_property(ACTIVE, Int(1)),
        "duplicate_property" => g
            .get_node_mut(h)
            .unwrap()
            .properties
            .push((ACTIVE, Bool(true))),
        "dangling_endpoint" => add_relation(g, PLACED, h, NodeId::default()),
        "extra_untyped_edge" => add_relation(g, Label(7), h, h),
        "overoccupied_site" => {
            let site = g.nodes().find(|(_, n)| integer(n, ID) == Ok(1)).unwrap().0;
            for _ in 0..6 {
                let n = g.add_node(data(
                    BEAD,
                    *frontier,
                    0,
                    &[(SPECIES, Int(4)), (ACTIVE, Bool(false))],
                ));
                *frontier += 1;
                add_relation(g, PLACED, n, site);
            }
        }
        "negative_reservoir" => reservoir[0] = -1,
        _ => panic!("unknown frozen mutation"),
    }
}
fn invariant_mutant(w: &ChemistryWorld, mutation: &str) -> State {
    use PropertyValue::{Bool, Int};
    let mut s = match mutation {
        "missing_heat" | "double_fuel_credit" => {
            prepare(w, &bond(w, &json!([10, 20, 30]))).unwrap().next
        }
        _ => w.state.clone(),
    };
    match mutation {
        "extra_unpaid_bead" => {
            // A schema-valid extra inactive F, without paying its reservoir material.
            let site = w.handle(1).unwrap().node;
            let id = s.semantic_id_frontier;
            s.semantic_id_frontier += 1;
            let h = s.graph.add_node(data(
                BEAD,
                id,
                s.revision,
                &[(SPECIES, Int(4)), (ACTIVE, Bool(false))],
            ));
            add_relation(&mut s.graph, PLACED, h, site);
        }
        "missing_reservoir_material_debit" => {
            // Produce a real funded transfer on an unpublished copy, then omit
            // exactly its reservoir debit. This differs from an unpaid creation.
            let mut copy = ChemistryWorld {
                state: w.state.clone(),
                authority: w.authority,
            };
            let out = prepare(
                &copy,
                &ChemicalAction::TransferFuelOut {
                    fuel: w.handle(30).unwrap(),
                },
            )
            .unwrap();
            commit(&mut copy, out).unwrap();
            s = prepare(&copy, &ChemicalAction::TransferFuelIn { activated: true })
                .unwrap()
                .next;
            s.reservoir = copy.state.reservoir;
        }
        "missing_heat" => s.heat = 0,
        "double_fuel_credit" => s
            .graph
            .get_node_mut(w.handle(30).unwrap().node)
            .unwrap()
            .set_property(ACTIVE, Bool(true)),
        "negative_heat" => s.heat = -1,
        _ => panic!("unknown frozen invariant"),
    }
    s
}
pub fn run_case(contract: &Value, c: &Value, workers: usize) -> Value {
    let (mut graph, revision, mut frontier, mut reservoir, heat) =
        raw(contract, c["fixture"].as_str().unwrap());
    let id = c["id"].as_str().unwrap();
    if c["layer"] == "initial_admission" {
        mutate_initial(
            &mut graph,
            &mut reservoir,
            &mut frontier,
            c["mutation"].as_str().unwrap(),
        );
        let raw = json!({"graph":graph,"allocator":graph.allocation_state(),"revision":revision,"frontier":frontier,"reservoir":reservoir,"heat":heat});
        let result =
            ChemistryWorld::admit(graph, id.to_owned(), revision, frontier, reservoir, heat);
        return json!({"id":id,"workers":workers,"layer":c["layer"],"raw_initial":raw,"accepted":0,"rejected":[format!("{:?}",result.err().expect("malformed must reject"))],"attempts":[]});
    }
    let mut w =
        ChemistryWorld::admit(graph, id.to_owned(), revision, frontier, reservoir, heat).unwrap();
    let initial = w.observation();
    let mut log = vec![];
    let mut diagnostic = Value::Null;
    let operation = c["operation"].as_str().unwrap();
    match operation {
        "prepare_both_commit_first_then_second_then_reprepare_second"
        | "prepare_both_commit_first_then_reprepare_second" => {
            let actions: Vec<_> = c["order"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| bond(&w, b))
                .collect();
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .unwrap();
            let staged: Vec<_> = pool.install(|| {
                actions
                    .par_iter()
                    .map(|a| prepare(&w, a).unwrap())
                    .collect()
            });
            let mut staged = staged.into_iter();
            for a in &actions {
                log.push(json!({"phase":"prepare","action":a,"decision":"prepared","before":w.observation(),"after":w.observation()}));
            }
            try_prepared(&mut w, staged.next().unwrap(), &actions[0], &mut log);
            try_prepared(&mut w, staged.next().unwrap(), &actions[1], &mut log);
            let _ = run_action(&mut w, actions[1].clone(), &mut log);
        }
        "form_bond" | "form_bond_with_missing_handle" => {
            let a = bond(&w, &c["binding"]);
            if c["layer"] == "commit_fault" {
                let before = w.observation();
                let error =
                    prepare_with_fault(&w, &a, Some(fault_point(c["fault"].as_str().unwrap())))
                        .err()
                        .expect("injected fault");
                record_error(
                    &mut log,
                    "fault_staging",
                    &before,
                    &w.observation(),
                    json!(a),
                    error,
                );
            } else {
                let _ = run_action(&mut w, a, &mut log);
            }
        }
        "transfer_in_then_out" => {
            let event = run_action(
                &mut w,
                ChemicalAction::TransferFuelIn {
                    activated: c["activated"].as_bool().unwrap(),
                },
                &mut log,
            )
            .unwrap();
            let a = ChemicalAction::TransferFuelOut {
                fuel: w.handle(event.allocated_semantic_id.unwrap()).unwrap(),
            };
            let _ = run_action(&mut w, a, &mut log);
        }
        "transfer_fuel_in" => {
            let _ = run_action(
                &mut w,
                ChemicalAction::TransferFuelIn {
                    activated: c["activated"].as_bool().unwrap(),
                },
                &mut log,
            );
        }
        "transfer_bead_out" | "transfer_fuel_out" => {
            let a = ChemicalAction::TransferFuelOut {
                fuel: absent(&w, number(&c["semantic_id"])),
            };
            let _ = run_action(&mut w, a, &mut log);
        }
        "prepare_then_intervening_bond_then_commit_and_reprepare" => {
            let a = bond(&w, &c["binding"]);
            let p = prepare(&w, &a).unwrap();
            log.push(json!({"phase":"prepare","action":a,"decision":"prepared","before":w.observation(),"after":w.observation()}));
            let intervening = bond(&w, &c["intervening"]);
            run_action(&mut w, intervening, &mut log).unwrap();
            try_prepared(&mut w, p, &a, &mut log);
            let _ = run_action(&mut w, a, &mut log);
        }
        "prepare_then_transfer_out_and_in_then_try_old_handle" => {
            let a = bond(&w, &c["binding"]);
            let p = prepare(&w, &a).unwrap();
            let old = w.handle(30).unwrap();
            log.push(json!({"phase":"prepare","action":a,"decision":"prepared","before":w.observation(),"after":w.observation()}));
            run_action(
                &mut w,
                ChemicalAction::TransferFuelOut { fuel: old },
                &mut log,
            )
            .unwrap();
            let event = run_action(
                &mut w,
                ChemicalAction::TransferFuelIn { activated: true },
                &mut log,
            )
            .unwrap();
            diagnostic = json!({"old_handle":old,"new_handle":w.handle(event.allocated_semantic_id.unwrap()).unwrap()});
            try_prepared(&mut w, p, &a, &mut log);
            let _ = run_action(&mut w, a, &mut log);
        }
        "commit_prepared_in_distinct_world" => {
            let a = bond(&w, &json!([10, 20, 30]));
            let p = prepare(&w, &a).unwrap();
            let (g, r, f, res, h) = raw(contract, c["fixture"].as_str().unwrap());
            // Same textual identity too: an independent admission has distinct authority.
            let mut other = ChemistryWorld::admit(g, id.to_owned(), r, f, res, h).unwrap();
            let before = other.observation();
            let error = commit(&mut other, p).err().unwrap();
            diagnostic = json!({"other_before":before,"other_after":other.observation()});
            record_error(
                &mut log,
                "cross_world_commit",
                &w.observation(),
                &w.observation(),
                json!(a),
                error,
            );
        }
        "remove_node_without_full_incident_footprint" | "replace_activation_bool_with_int" => {
            let edit = if operation == "remove_node_without_full_incident_footprint" {
                CheckedEdit::RemoveNode(w.handle(number(&c["semantic_id"])).unwrap().node)
            } else {
                CheckedEdit::ReplaceProperty {
                    node: w.handle(30).unwrap().node,
                    key: ACTIVE,
                    expected: PropertyValue::Bool(true),
                    value: PropertyValue::Int(1),
                }
            };
            let error = stage_checked(w.graph(), &patch(vec![edit])).err().unwrap();
            record_error(
                &mut log,
                "generic_stage",
                &w.observation(),
                &w.observation(),
                json!({"operation":operation}),
                error,
            );
        }
        "validate_mutant_poststate_without_publication" => {
            let mutant = invariant_mutant(&w, c["mutation"].as_str().unwrap());
            let error = validate(&mutant).err().unwrap();
            diagnostic = json!({"mutant":observation(&mutant),"validation":format!("{error:?}")});
            record_error(
                &mut log,
                "invariant_check",
                &w.observation(),
                &w.observation(),
                json!({"mutation":c["mutation"]}),
                error,
            );
        }
        _ => panic!("unknown frozen case"),
    }
    let accepted = log.iter().filter(|a| a["decision"] == "accepted").count();
    let rejected: Vec<_> = log
        .iter()
        .filter(|a| a["decision"] == "rejected")
        .map(|a| a["reason"].clone())
        .collect();
    assert!(log.len() <= 32, "offer cap");
    json!({"id":id,"workers":workers,"layer":c["layer"],"initial":initial,"final":w.observation(),"accepted":accepted,"rejected":rejected,"attempts":log,"diagnostic":diagnostic})
}
pub fn run_contract(contract: &Value) -> Value {
    let frozen: Value = serde_json::from_str(include_str!(
        "../../../../experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json"
    ))
    .unwrap();
    assert_eq!(
        contract, &frozen,
        "CC-1 requires the unchanged frozen contract"
    );
    let start = Instant::now();
    assert_eq!(contract["law_version"], LAW);
    assert_eq!(contract["profile"], PROFILE);
    let cases = contract["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 41);
    assert_eq!(contract["denominator"]["worker_counts"], json!([1, 4]));
    let mut variants = vec![];
    let mut bytes = 0;
    let mut cap_failure: Option<String> = None;
    'cases: for case in cases {
        for workers in [1, 4] {
            if start.elapsed().as_secs() >= 300 {
                cap_failure = Some(format!("native_time:{}:{workers}", case["id"]));
                break 'cases;
            }
            let result = run_case(contract, case, workers);
            let size = serde_json::to_vec(&result).unwrap().len();
            if size > 16_777_216 || bytes + size > 134_217_728 {
                cap_failure = Some(format!("artifact_bytes:{}:{workers}", case["id"]));
                break 'cases;
            }
            bytes += size;
            variants.push(result);
        }
    }
    json!({"status":if cap_failure.is_some(){"incomplete_cap_binding"}else{"executed_pending_independent_audit"},"cap_failure":cap_failure,"schema":1,"profile":PROFILE,"law_version":LAW,"cases":41,"variants":variants,"native_seconds":start.elapsed().as_secs_f64(),
        "training_admitted":false,"scientific_acceptance":false,"sustained_claim":false,"evolution_enabled":false,"scientific_run_launched":false,
        "ready_for_scientific_launch":false,"reproductive_fairness_accepted":false,"power_adequacy_established":false,"independent_scientific_worlds":0,
        "rng":null,"physical_time":null,"seeds":[]})
}
