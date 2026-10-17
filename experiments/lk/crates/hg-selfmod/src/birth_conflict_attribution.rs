//! Default-off observation of the archived random/canonical operating point.
//! No scheduler inputs, acceptance, store or main RNG are changed.
use super::*;
use crate::decode::DecodedRule;
use crate::resolve::{resolve_conflicts_tiebreak, TieBreak};
use crate::runner::TruncationMode;
use hg_observe::Binding;
use slotmap::Key;

fn binding_value(b: &Binding) -> Value {
    json!({"rid":b.rule_id,"priority":b.priority,
        "nodes":b.bound_nodes.iter().map(|n|n.data().as_ffi()).collect::<Vec<_>>(),
        "edges":b.matched_edges.iter().map(|e|e.data().as_ffi()).collect::<Vec<_>>()})
}

fn trace_pool(pool: &[Binding], key: u64) -> Value {
    // This stream is separate from the world's RNG, just as in resolve.rs.
    let mut local = ChaCha8Rng::seed_from_u64(key ^ 0x7138_4C2D_9B05_A1F7);
    let words: Vec<u64> = (0..pool.len()).map(|_| local.gen::<u64>()).collect();
    let draws: Vec<f64> = words.iter().map(|w| *w as f64).collect();
    let mut order: Vec<usize> = (0..pool.len()).collect();
    order.sort_by(|&i, &j| {
        pool[j]
            .priority
            .cmp(&pool[i].priority)
            .then_with(|| draws[j].total_cmp(&draws[i]))
            .then_with(|| pool[i].node_ids_sorted().cmp(&pool[j].node_ids_sorted()))
    });
    let mut owners = HashMap::<u64, usize>::new();
    let mut accepted = Vec::new();
    let mut decisions = Vec::new();
    for (rank, i) in order.into_iter().enumerate() {
        let b = &pool[i];
        let blockers: Vec<Value> = b
            .bound_nodes
            .iter()
            .filter_map(|n| {
                let node = n.data().as_ffi();
                owners
                    .get(&node)
                    .map(|winner| json!({"node":node,"winner_pool_index":winner}))
            })
            .collect();
        let wins = blockers.is_empty();
        if wins {
            for n in &b.bound_nodes {
                owners.insert(n.data().as_ffi(), i);
            }
            accepted.push(b.clone());
        }
        decisions.push(json!({"rank":rank,"pool_index":i,"draw_word":words[i],"draw":draws[i],"accepted":wins,"blockers":blockers}));
    }
    // Cross-check actual production acceptance including variable and edge order.
    let native = resolve_conflicts_tiebreak(pool, TieBreak::Random, &[], key, 0.0);
    assert_eq!(
        native.iter().map(binding_value).collect::<Vec<_>>(),
        accepted.iter().map(binding_value).collect::<Vec<_>>()
    );
    json!({"key":key,"decisions":decisions,"native_acceptance_crosscheck":true})
}

pub(crate) fn probe(
    world: &HypergraphStore,
    rules: &[DecodedRule],
    cfg: &RunConfig,
    budgets: &HashMap<NodeId, i64>,
) -> Value {
    assert_eq!(
        cfg.tie_break,
        TieBreak::Random,
        "Observer supports archived random tie-break only"
    );
    assert!(
        matches!(cfg.truncation_mode, TruncationMode::Canonical),
        "Observer supports canonical truncation only"
    );
    let n = world.node_count();
    let e = world.edge_count();
    let mut labels = vec![0; cfg.n_labels as usize];
    for (_, d) in world.nodes() {
        labels[d.label.0 as usize] += 1;
    }
    let snap = world.snapshot();
    let cap = if cfg.match_cap_per_rule == 0 {
        usize::MAX
    } else {
        cfg.match_cap_per_rule
    };
    let mut generated = Vec::new();
    let mut pool = Vec::new();
    let mut table = Vec::new();
    for (rid, rule) in rules.iter().enumerate() {
        let grower = is_grower(&rule.spec);
        let blocked = grower
            && (budgets.get(&rule.root).copied().unwrap_or(0) <= 0
                || (n >= cfg.world_ceiling && !rule.spec.creates.is_empty())
                || (e >= cfg.edge_ceiling && rule.spec.redges.len() > rule.spec.pedges.len()));
        let skipped = !blocked && enum_cost(&rule.spec, &labels, n, e) > cfg.match_enum_budget;
        table.push(json!({"rid":rid,"root":key_u64(rule.root),"class_hash":rule.class_hash,"blocked":blocked,"enumeration_skipped":skipped}));
        if blocked || skipped {
            continue;
        }
        let mut bindings = hg_engine::find_bindings(&snap, &rule.compiled, rid);
        bindings.sort_by_key(|b| b.canonical_key());
        for (rank, b) in bindings.into_iter().enumerate() {
            let submitted = rank < cap;
            generated.push(json!({"binding":binding_value(&b),"canonical_rank":rank,"pool_index":if submitted {Some(pool.len())} else {None}}));
            if submitted {
                pool.push(b);
            }
        }
    }
    let key = cfg.seed ^ (world.step() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    json!({"world_step_before":world.step(),"world_witness":{"nodes":world.nodes().map(|(id,n)|json!({"id":key_u64(id),"label":n.label.0})).collect::<Vec<_>>(),"edges":world.edges().map(|(id,e)|json!({"id":id.data().as_ffi(),"label":e.label.0,"nodes":e.nodes.iter().map(|n|key_u64(*n)).collect::<Vec<_>>()})).collect::<Vec<_>>()},"tie_break":"random","truncation_mode":"canonical","cap":cfg.match_cap_per_rule,"rules":table,"generated":generated,"resolution":trace_pool(&pool,key)})
}

fn allocator_path(path: &Path) -> std::path::PathBuf {
    path.with_file_name(format!(
        "{}_allocator.json",
        path.file_stem().unwrap().to_string_lossy()
    ))
}

pub fn extend(input: &Path, offset: usize, steps: usize, out: &Path) -> io::Result<Value> {
    fs::create_dir(out)?;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let initial = read(input)?;
        let mut state = restore(initial.clone())?;
        financing::restore_allocator(&mut state, read(&allocator_path(input))?)?;
        assert_eq!(snapshot(&state), initial);
        assert!(stocks(&state).len() <= LIMIT);
        save(&out.join("initial.json"), &initial)?;
        save(
            &out.join("initial_allocator.json"),
            &financing::allocator(&state),
        )?;
        status(
            out,
            json!({"state":"running","offset":offset,"steps_completed":0}),
        )?;
        let mut receipts = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join("rows.jsonl"))?,
        );
        let mut traces = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join("conflicts.jsonl"))?,
        );
        for i in 1..=steps {
            let mut trace = Value::Null;
            let mut row = object_observed(&mut state, Some(&mut trace));
            row.continuation_step = Some(offset + i);
            let receipt = finish_phase(&mut state, row, Policy::Replace8);
            serde_json::to_writer(&mut receipts, &receipt)?;
            receipts.write_all(b"\n")?;
            trace["continuation_step"] = json!(offset + i);
            serde_json::to_writer(&mut traces, &trace)?;
            traces.write_all(b"\n")?;
            if state.world.node_count() > state.cfg.node_explosion_cap {
                return Err(io::Error::other("hard node stop; raw retained"));
            }
            if i % 1000 == 0 {
                receipts.flush()?;
                traces.flush()?;
                save(
                    &out.join(format!("checkpoint_{}.json", offset + i)),
                    &snapshot(&state),
                )?;
                save(
                    &out.join(format!("checkpoint_{}_allocator.json", offset + i)),
                    &financing::allocator(&state),
                )?;
                status(
                    out,
                    json!({"state":"running","offset":offset,"steps_completed":i}),
                )?;
            }
        }
        receipts.flush()?;
        traces.flush()?;
        assert_eq!(snapshot(&state)["cfg"], initial["cfg"]);
        save(&out.join("final.json"), &snapshot(&state))?;
        save(
            &out.join("final_allocator.json"),
            &financing::allocator(&state),
        )?;
        let result = json!({"state":"complete","offset":offset,"steps_completed":steps});
        status(out, result.clone())?;
        Ok(result)
    }))
    .unwrap_or_else(|_| Err(io::Error::other("observer/native panic; evidence retained")));
    if let Err(e) = &result {
        status(out, json!({"state":"failed","error":e.to_string()}))?;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn b(rid: usize, nodes: &[u64]) -> Binding {
        Binding::new(
            rid,
            0,
            nodes
                .iter()
                .map(|n| NodeId::from(slotmap::KeyData::from_ffi(*n)))
                .collect(),
        )
    }
    #[test]
    fn keyed_order_blockers_and_empty_patterns_match_native() {
        let pool = vec![
            b(0, &[1, 2]),
            b(1, &[2, 3]),
            b(2, &[3, 4]),
            b(3, &[]),
            b(4, &[1, 4]),
        ];
        let mut multiple = false;
        for key in 0..64 {
            let trace = trace_pool(&pool, key);
            for d in trace["decisions"].as_array().unwrap() {
                assert_eq!(
                    d["accepted"].as_bool().unwrap(),
                    d["blockers"].as_array().unwrap().is_empty()
                );
                multiple |= d["blockers"].as_array().unwrap().len() > 1;
            }
            assert_eq!(trace, trace_pool(&pool, key));
        }
        assert!(multiple);
    }
    #[test]
    fn observer_preserves_native_lifecycle_128_steps() {
        let mut plain = initialize(7, 10000);
        let snap = snapshot(&plain);
        let mut observed = restore(snap).unwrap();
        financing::restore_allocator(&mut observed, financing::allocator(&plain)).unwrap();
        for i in 1..=128 {
            let mut a = object(&mut plain);
            let mut trace = Value::Null;
            let mut b = object_observed(&mut observed, Some(&mut trace));
            a.continuation_step = Some(i);
            b.continuation_step = Some(i);
            let a = finish_phase(&mut plain, a, Policy::Replace8);
            let b = finish_phase(&mut observed, b, Policy::Replace8);
            assert_eq!(a, b);
            assert_eq!(snapshot(&plain), snapshot(&observed));
            assert_eq!(
                financing::allocator(&plain),
                financing::allocator(&observed)
            );
        }
    }
}
