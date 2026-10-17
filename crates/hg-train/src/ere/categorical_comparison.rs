//! Frozen local direction comparison. Uses the existing guarded native evaluator.
//! Not multi-generation training, admission, or independent physical replay.
use super::{
    categorical as cat,
    complete_admission::{self as a, Res},
    confirmation as c, direction, feasibility_training as f, minibatch,
    short_training::{sign_p, terminal},
};
use crate::optim::OpenAiEs;
use hg_sim_core::ere::valid_grammar;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-categorical-versus-es-direction-v1";
const LIMIT: f64 = 0.100001;
const CONFIG_SHA: &str = "b26d94f1d889018ff4baa79f81f68a82c1d8c838a0375e080e3de99bb81d4328";
pub fn offline_fixture(catalogue: &Path, request: &Path, out: &Path) -> Res<()> {
    use std::io::Write;
    a::check(!out.exists(), "fresh offline result required")?;
    a::check(
        a::digest(catalogue)? == a::CATALOGUE_SHA,
        "pinned offline catalogue",
    )?;
    let req = a::load(request)?;
    a::check(
        req["purpose"] == "engineering_math_only",
        "offline arithmetic purpose",
    )?;
    let raw: Vec<Vec<f32>> =
        serde_json::from_value(req["raw_vectors"].clone()).map_err(|e| e.to_string())?;
    a::check(raw.len() == 4, "four deterministic raw fixtures")?;
    let model = cat::Model::from_catalogue(&a::load(catalogue)?)?;
    let mut contexts = vec![];
    for base in direction::contexts() {
        let matched: Vec<_> = raw
            .iter()
            .map(|r| matched_step(&base, r))
            .collect::<Res<_>>()?;
        let mut vectors = vec![base.clone()];
        for name in ["forward", "reverse"] {
            for m in &matched {
                vectors.push(serde_json::from_value(m[name].clone()).map_err(|e| e.to_string())?);
            }
        }
        contexts.push(json!({"base":base,"matched":matched,"vectors":vectors,"full_support":weights(&model,&base,&vectors)?}));
    }
    let align: Align = vec![vec![vec![Some(0.5), Some(1.)]; 2]; 8];
    let effects: Effects = vec![vec![vec![vec![Some(0.01); 2], vec![Some(0.02); 2]]; 2]; 8];
    let validity: Effects = vec![vec![vec![vec![Some(0.); 2]; 2]; 2]; 8];
    let result = json!({"classification":"engineering_math_only","native_steps_run":0,"grammar_compilations":0,"training_updates":0,"training_admitted":false,"contexts":contexts,"toy_statistics_not_evidence":statistics("scientific",&align,&effects,&validity,true)});
    let bytes = serde_json::to_vec(&result).map_err(|e| e.to_string())?;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)
        .map_err(|e| e.to_string())?
        .write_all(&bytes)
        .map_err(|e| e.to_string())
}
pub fn schedule(development: bool) -> Vec<Value> {
    let (master, n, k, draws) = if development {
        (0xE66100000000_u64, 1, 1, 8)
    } else {
        (0xE66000000000, 8, 4, 128)
    };
    let mut rows = vec![];
    for r in 0..n {
        for ctx in 0..2 {
            for (m, method) in ["es", "categorical"].iter().enumerate() {
                for e in 0..2 {
                    let j = 1000000 * r + 100000 * ctx + 10000 * m + 1000 * e;
                    if *method == "es" {
                        for p in 0..8 {
                            for sign in [1, -1] {
                                for b in 0..k {
                                    let i = 4 * p + b;
                                    rows.push(json!({"id":format!("est-r{r}-c{ctx}-es-e{e}-p{p}-s{sign}-b{b}"),"stage":"estimation","block":r,"context":ctx,"method":method,"estimate":e,"pair":p,"sign":sign,"batch_draw":b,
                "noise_seed":master+10000000+j as u64,"emission_seed":master+20000000+(j+i) as u64,"physics_seed":master+40000000+(j+i) as u64,"horizon":100}));
                                }
                            }
                        }
                    } else {
                        for i in 0..16 * k {
                            rows.push(json!({"id":format!("est-r{r}-c{ctx}-categorical-e{e}-d{i}"),"stage":"estimation","block":r,"context":ctx,"method":method,"estimate":e,"draw":i,
            "emission_seed":master+20000000+(j+i) as u64,"physics_seed":master+40000000+(j+i) as u64,"horizon":100}));
                        }
                    }
                }
            }
        }
    }
    for r in 0..n {
        for ctx in 0..2 {
            for d in 0..draws {
                for p in 0..4 {
                    let j = 1000000 * r + 100000 * ctx;
                    rows.push(json!({"id":format!("val-r{r}-c{ctx}-d{d}-p{p}"),"stage":"validation","block":r,"context":ctx,"draw":d,"physics_index":p,"sampled_arm":"base",
            "emission_seed":master+60000000+(j+d) as u64,"physics_seed":master+80000000+(j+4*d+p) as u64,"horizon":100}));
                }
            }
        }
    }
    rows
}
fn sum(xs: impl IntoIterator<Item = f64>) -> f64 {
    let mut total = 0.;
    for x in xs {
        total += x;
    }
    total
}
fn check_leaf(
    grammar: &valid_grammar::Grammar,
    model: &cat::Model,
    e: &hg_sim_core::ere::signal::Emission,
    runner: &mut f::Runner<'_>,
) -> Res<()> {
    let s = model.selection_from_emission(&e.record)?;
    // No RNG or Engine call: seed is only a receipt label for emit_selected.
    let seed = e.record["emission_seed"].as_u64().ok_or("emission seed")? ^ (1_u64 << 63);
    let rebuilt = grammar.emit_selected(s.parent, s.operator, s.role, s.production_index, seed)?;
    let mut expected = e.record.clone();
    expected["emission_seed"] = json!(seed);
    a::check(
        e.store.canonical_hash() == rebuilt.store.canonical_hash()
            && e.root == rebuilt.root
            && e.program == rebuilt.program
            && expected == rebuilt.record,
        "conditional leaf/store/program/credit reconstruction changed with receipt seed",
    )?;
    runner.m["conditional_leaf_checks"] =
        json!(runner.m["conditional_leaf_checks"].as_u64().unwrap() + 1);
    runner.save()
}
pub fn matched_step(base: &[f32], raw: &[f32]) -> Res<Value> {
    a::check(
        base.len() == 183 && raw.len() == 183 && base.iter().chain(raw).all(|x| x.is_finite()),
        "finite 183-vector",
    )?;
    let delta: Vec<_> = raw
        .iter()
        .zip(base)
        .map(|(r, b)| f64::from(*r) - f64::from(*b))
        .collect();
    let mut centered = delta.clone();
    for (start, end) in [(0, 168), (168, 181), (181, 183)] {
        let mean = sum(delta[start..end].iter().copied()) / (end - start) as f64;
        for x in &mut centered[start..end] {
            *x -= mean;
        }
    }
    let norm = sum(centered.iter().map(|x| x * x)).sqrt();
    let forward: Vec<f32> = if norm == 0. {
        base.to_vec()
    } else {
        base.iter()
            .zip(&centered)
            .map(|(b, x)| (f64::from(*b) + 0.1 * x / norm) as f32)
            .collect()
    };
    let reverse: Vec<f32> = base
        .iter()
        .zip(&forward)
        .map(|(b, f)| 2.0 * b - f)
        .collect();
    for vector in [&forward, &reverse] {
        let d: Vec<_> = vector
            .iter()
            .zip(base)
            .map(|(v, b)| f64::from(*v) - f64::from(*b))
            .collect();
        a::check(
            vector.iter().all(|v| v.is_finite())
                && d.iter().all(|x| x.abs() <= LIMIT)
                && sum(d.iter().map(|x| x * x)).sqrt() <= LIMIT,
            "actual f32 step ceiling",
        )?;
    }
    Ok(
        json!({"raw_delta":delta,"centered_delta":centered,"centered_norm":norm,"degenerate":norm==0.,"forward":forward,"reverse":reverse}),
    )
}
pub fn weight(model: &cat::Model, base: &[f32], proposal: &[f32], s: cat::Selection) -> Res<Value> {
    let b = model.score(base, s)?;
    let p = model.score(proposal, s)?;
    let w = (p.log_probability - b.log_probability).exp();
    a::check(
        w.is_finite() && w >= (-6. * LIMIT).exp() - 1e-12 && w <= (6. * LIMIT).exp() + 1e-12,
        "full leaf ratio ceiling",
    )?;
    Ok(
        json!({"base_log_probability":b.log_probability,"proposal_log_probability":p.log_probability,"weight":w}),
    )
}
fn weights(model: &cat::Model, base: &[f32], vectors: &[Vec<f32>]) -> Res<Value> {
    let mut rows = vec![];
    let mut mass = vec![0.; vectors.len()];
    let mut min = vec![f64::INFINITY; vectors.len()];
    let mut max = vec![0.0_f64; vectors.len()];
    // Uniform production-index law cancels: one group ratio covers every leaf.
    for s in model
        .selections()
        .into_iter()
        .filter(|s| s.production_index == 0)
    {
        let b = model.score(base, s)?;
        let count = b.production_count as f64;
        let mut ratios = vec![];
        for (i, v) in vectors.iter().enumerate() {
            let w = weight(model, base, v, s)?;
            let ratio = w["weight"].as_f64().unwrap();
            mass[i] += b.probability * count * ratio;
            min[i] = min[i].min(ratio);
            max[i] = max[i].max(ratio);
            ratios.push(w);
        }
        rows.push(json!({"selection":s,"production_count":b.production_count,"ratios":ratios}));
    }
    a::check(
        rows.len() == 3232 && mass.iter().all(|x| (*x - 1.).abs() < 1e-12),
        "full support normalization",
    )?;
    Ok(json!({"legal_leaves":18956,"groups":rows,"mass":mass,"minimum":min,"maximum":max}))
}
type Effects = Vec<Vec<Vec<Vec<Option<f64>>>>>;
type Align = Vec<Vec<Vec<Option<f64>>>>;
pub fn statistics(
    mode: &str,
    align: &Align,
    effects: &Effects,
    validity: &Effects,
    complete: bool,
) -> Value {
    let n = if mode == "scientific" { 8 } else { 1 };
    assert_eq!(align.len(), n);
    assert_eq!(effects.len(), n);
    assert_eq!(validity.len(), n);
    let known = complete
        && align.iter().flatten().flatten().all(Option::is_some)
        && effects
            .iter()
            .chain(validity)
            .flatten()
            .flatten()
            .flatten()
            .all(Option::is_some);
    let mut cr = vec![];
    let mut dr = vec![];
    let mut lr = vec![];
    let mut cm = [None, None];
    let mut vf = None;
    if known {
        let context: Vec<Vec<Vec<f64>>> = effects
            .iter()
            .map(|r| {
                r.iter()
                    .map(|ctx| {
                        ctx.iter()
                            .map(|m| sum(m.iter().map(|x| x.unwrap())) / 2.)
                            .collect()
                    })
                    .collect()
            })
            .collect();
        for r in 0..n {
            cr.push(sum(context[r].iter().map(|x| x[1])) / 2.);
            dr.push(sum(context[r].iter().map(|x| x[1] - x[0])) / 2.);
            lr.push(sum(align[r].iter().map(|x| x[1].unwrap())) / 2.);
        }
        for ctx in 0..2 {
            cm[ctx] = Some(sum((0..n).map(|r| context[r][ctx][1])) / n as f64);
        }
        vf = Some(
            sum(validity
                .iter()
                .flat_map(|r| r.iter().flat_map(|ctx| ctx[1].iter().map(|x| x.unwrap()))))
                / (n * 4) as f64,
        );
    }
    let summarize = |xs: &Vec<f64>| {
        let wins = xs.iter().filter(|&&x| x > 1e-12).count();
        let losses = xs.iter().filter(|&&x| x < -1e-12).count();
        json!({"mean":known.then(||sum(xs.iter().copied())/n as f64),"sign_p":known.then(||sign_p(wins,losses)),"wins":wins,"losses":losses,"ties":xs.len()-wins-losses,"fixed_denominator":n})
    };
    let confirm = json!({"categorical_utility":summarize(&cr),"categorical_minus_es":summarize(&dr),"categorical_alignment":summarize(&lr)});
    let support =
        mode == "scientific"
            && known
            && confirm.as_object().unwrap().values().all(|v| {
                v["mean"].as_f64().unwrap() > 1e-12 && v["sign_p"].as_f64().unwrap() < 0.05
            })
            && cm.iter().all(|x| x.unwrap() > 1e-12)
            && vf.unwrap() >= -1e-12;
    let unknown = vec![Value::Null; n];
    json!({"mode":mode,"statistical_unit":"paired_estimator_block","fixed_denominator":n,"all_outcomes_known":known,
        "block_categorical_effects":if known{json!(cr)}else{json!(unknown)},"block_contrasts":if known{json!(dr)}else{json!(unknown)},
        "block_alignment":if known{json!(lr)}else{json!(unknown)},"confirmatory":confirm,"categorical_context_effects":cm,
        "categorical_weighted_validity_effect":vf,"support_categorical_direction":support})
}
fn prepared(out: &Path, root: &str) -> Res<(Value, Value, Vec<Value>, Value)> {
    a::check(
        root.len() == 64 && a::digest(&out.join("prepared_sha256.json"))? == root,
        "prepared root",
    )?;
    let pins: BTreeMap<String, String> =
        serde_json::from_value(a::load(&out.join("prepared_sha256.json"))?)
            .map_err(|e| e.to_string())?;
    a::check(
        pins.keys()
            .all(|n| !n.starts_with('/') && !n.split('/').any(|p| p == ".." || p.is_empty())),
        "safe prepared entries",
    )?;
    for (n, h) in &pins {
        a::check(
            a::digest(&out.join(n))? == *h,
            &format!("prepared input drift {n}"),
        )?;
    }
    let request = a::load(&out.join("request.json"))?;
    a::check(
        request["profile"] == PROFILE
            && request["launch_authorized"] == true
            && request["full_training_authorized"] == false,
        "explicit comparison launch required",
    )?;
    let snapshot = a::load(&out.join("snapshot.json"))?;
    c::recheck(&snapshot)?;
    a::check(
        snapshot["build_features"] == json!(["ere-complete-signal"])
            && snapshot["default_features"] == false,
        "isolated build feature receipt",
    )?;
    let config = a::load(&out.join("config.json"))?;
    a::check(
        a::digest(&out.join("config.json"))? == CONFIG_SHA,
        "frozen numerical contract identity",
    )?;
    let plan: Vec<Value> =
        serde_json::from_value(a::load(&out.join("schedule.json"))?).map_err(|e| e.to_string())?;
    let mode = request["mode"].as_str().ok_or("mode")?;
    a::check(
        ["scientific", "development"].contains(&mode)
            && config["profile"] == PROFILE
            && config[mode]["total_evaluations"] == plan.len(),
        "registered layout",
    )?;
    a::check(
        plan == schedule(mode == "development"),
        "exact registered seed/order schedule",
    )?;
    a::check(
        a::load(&out.join("contexts.json"))? == json!(direction::contexts()),
        "registered contexts",
    )?;
    let checked = std::process::Command::new("python3")
        .arg("-B")
        .arg(out.join("source_snapshots/scripts/audit_ere_categorical_comparison.py"))
        .arg(out)
        .output()
        .map_err(|e| e.to_string())?;
    a::check(
        checked.status.success(),
        &format!(
            "frozen launch preflight: {}",
            String::from_utf8_lossy(&checked.stderr)
        ),
    )?;
    let audit: Value = serde_json::from_slice(&checked.stdout).map_err(|e| e.to_string())?;
    a::check(
        audit["audit_pass"] == true
            && audit["recorded_cells"] == 0
            && audit["grammar_compilations"] == 0
            && audit["campaign_complete"] == false,
        "zero-work launch preflight",
    )?;
    c::recheck(&snapshot)?;
    Ok((config, request, plan, snapshot))
}
pub fn run(out: &Path, root: &str) -> Res<()> {
    let start = Instant::now();
    // Fail before grammar compilation on malformed/unapproved preparation.
    let (config, request, plan, snapshot) = prepared(out, root)?;
    let mode = request["mode"].as_str().unwrap();
    let n = config[mode]["blocks"].as_u64().unwrap() as usize;
    let k = config[mode]["es_emissions_per_candidate"].as_u64().unwrap() as usize;
    let draws = config[mode]["validation_emissions_per_block_context"]
        .as_u64()
        .unwrap() as usize;
    let setup = request["setup_elapsed_seconds"]
        .as_f64()
        .ok_or("setup timing")?;
    let remaining = config[mode]["wall_seconds"].as_f64().unwrap() - setup;
    a::check(
        remaining > 0. && setup >= 0. && remaining.is_finite(),
        "setup wall ceiling",
    )?;
    let m = a::load(&out.join("manifest.json"))?;
    a::check(
        m == a::load(&out.join("initial_manifest.json"))?
            && m["recorded_cells"] == 0
            && m["grammar_compilations"] == 0,
        "fresh prepared manifest",
    )?;
    let mut runner = f::Runner {
        out,
        m,
        plan,
        deadline: start + Duration::from_secs_f64(remaining),
    };
    let theta = direction::contexts();
    let mut align: Align = vec![vec![vec![None; 2]; 2]; n];
    let mut effects: Effects = vec![vec![vec![vec![None; 2]; 2]; 2]; n];
    let mut validity = effects.clone();
    let result = terminal(|| {
        a::check(
            Instant::now() < runner.deadline,
            "preflight/setup wall ceiling",
        )?;
        runner.m["status"] = json!("compiling");
        runner.m["grammar_compilations"] = json!(1);
        runner.save()?;
        let compile = Instant::now();
        let grammar = valid_grammar::compile()?;
        runner.m["grammar_compile_seconds"] = json!(compile.elapsed().as_secs_f64());
        c::write(&out.join("catalogue.json"), &grammar.catalogue())?;
        a::check(
            a::digest(&out.join("catalogue.json"))? == a::CATALOGUE_SHA,
            "catalogue identity",
        )?;
        let model = cat::Model::from_grammar(&grammar)?;
        c::recheck(&snapshot)?;
        runner.m["status"] = json!("estimation");
        runner.save()?;
        for r in 0..n {
            for ctx in 0..2 {
                c::recheck(&snapshot)?;
                let mut vectors = vec![theta[ctx].clone()];
                let mut reverses = vec![];
                for method in ["es", "categorical"] {
                    for e in 0..2 {
                        let begin = runner.m["recorded_cells"].as_u64().unwrap() as usize;
                        let size = 16 * k;
                        let qs = runner.plan[begin..begin + size].to_vec();
                        a::check(
                            qs.iter().all(|q| {
                                q["block"] == r
                                    && q["context"] == ctx
                                    && q["method"] == method
                                    && q["estimate"] == e
                                    && q["stage"] == "estimation"
                            }),
                            "estimate schedule binding",
                        )?;
                        let path = out
                            .join("estimates")
                            .join(format!("r{r}-c{ctx}-{method}-e{e}.json"));
                        let mut rec = json!({"block":r,"context":ctx,"method":method,"estimate":e,"theta_before":theta[ctx],"evaluation_ids":qs.iter().map(|q|q["id"].clone()).collect::<Vec<_>>(),"complete":false,"raw":null,"matched":null});
                        let mut es = OpenAiEs::new(
                            1.,
                            0.05,
                            8,
                            ChaCha8Rng::seed_from_u64(qs[0]["noise_seed"].as_u64().unwrap_or(0)),
                        );
                        let eps = if method == "es" {
                            es.perturbations(183)
                        } else {
                            vec![]
                        };
                        rec["noise_seed"] = qs[0]["noise_seed"].clone();
                        rec["eps"] = json!(eps);
                        c::write(&path, &rec)?;
                        runner.m["direction_starts"] =
                            json!(runner.m["direction_starts"].as_u64().unwrap() + 1);
                        runner.save()?;
                        let raw = if method == "es" {
                            let mut scores = vec![];
                            for noise in &eps {
                                for sign in [1., -1.] {
                                    let candidate = es.candidate(&theta[ctx], noise, sign);
                                    let mut batch = vec![];
                                    for _ in 0..k {
                                        let q = &runner.plan
                                            [runner.m["recorded_cells"].as_u64().unwrap() as usize];
                                        let emission = grammar.emit(
                                            &candidate,
                                            q["emission_seed"].as_u64().unwrap(),
                                        )?;
                                        check_leaf(&grammar, &model, &emission, &mut runner)?;
                                        batch.push(runner.evaluate(&emission, &candidate)?.0);
                                    }
                                    scores.push(batch);
                                }
                            }
                            let (means, pairs) = minibatch::batch_pairs(&scores, k)?;
                            let mut raw = theta[ctx].clone();
                            es.update(&mut raw, &eps, &pairs);
                            rec["raw"] = json!({"candidate_means":means,"optimizer_pair_scores":pairs.iter().map(|p|[p.plus,p.minus]).collect::<Vec<_>>(),"theta_after":raw});
                            raw
                        } else {
                            let mut samples = vec![];
                            for q in &qs {
                                let emission = grammar
                                    .emit(&theta[ctx], q["emission_seed"].as_u64().unwrap())?;
                                let s = model.selection_from_emission(&emission.record)?;
                                check_leaf(&grammar, &model, &emission, &mut runner)?;
                                let (score, _) = runner.evaluate(&emission, &theta[ctx])?;
                                samples.push(cat::Sample {
                                    id: q["id"].as_str().unwrap().into(),
                                    emission_seed: q["emission_seed"].as_u64().unwrap(),
                                    physics_seed: q["physics_seed"].as_u64().unwrap(),
                                    selection: s,
                                    native_score: score,
                                    reason: if score.is_none() {
                                        Some("resource_cap".into())
                                    } else {
                                        None
                                    },
                                });
                            }
                            let est = cat::estimate(&model, &theta[ctx], &samples, size, 0.05)?;
                            let raw = est.theta_after.clone();
                            rec["raw"] = json!(est);
                            raw
                        };
                        let matched = matched_step(&theta[ctx], &raw)?;
                        vectors.push(
                            serde_json::from_value(matched["forward"].clone())
                                .map_err(|e| e.to_string())?,
                        );
                        reverses.push(
                            serde_json::from_value(matched["reverse"].clone())
                                .map_err(|e| e.to_string())?,
                        );
                        rec["matched"] = matched;
                        rec["complete"] = json!(true);
                        c::write(&path, &rec)?;
                        runner.m["direction_estimates"] =
                            json!(runner.m["direction_estimates"].as_u64().unwrap() + 1);
                        runner.save()?;
                    }
                }
                vectors.extend(reverses);
                let alignment: Vec<_> = [(1, 2), (3, 4)]
                    .into_iter()
                    .map(|(a, b)| direction::alignment(&theta[ctx], &vectors[a], &vectors[b]))
                    .collect();
                for m in 0..2 {
                    align[r][ctx][m] = alignment[m]["score"].as_f64();
                }
                let support = weights(&model, &theta[ctx], &vectors)?;
                c::write(
                    &out.join("directions").join(format!("r{r}-c{ctx}.json")),
                    &json!({"block":r,"context":ctx,"vectors":vectors,"alignment":alignment,"full_support":support}),
                )?;
                println!(
                    "categorical comparison estimation block {} context {} complete",
                    r + 1,
                    ctx
                );
            }
        }
        runner.m["status"] = json!("validation");
        runner.save()?;
        for r in 0..n {
            c::recheck(&snapshot)?;
            for ctx in 0..2 {
                let d = a::load(&out.join("directions").join(format!("r{r}-c{ctx}.json")))?;
                let vectors: Vec<Vec<f32>> =
                    serde_json::from_value(d["vectors"].clone()).map_err(|e| e.to_string())?;
                let mut terms = vec![];
                let mut utility_terms = vec![vec![]; 9];
                let mut valid_terms = utility_terms.clone();
                let mut base = vec![];
                let mut native = vec![];
                for draw in 0..draws {
                    let q = &runner.plan[runner.m["recorded_cells"].as_u64().unwrap() as usize];
                    let emission =
                        grammar.emit(&theta[ctx], q["emission_seed"].as_u64().unwrap())?;
                    let selection = model.selection_from_emission(&emission.record)?;
                    check_leaf(&grammar, &model, &emission, &mut runner)?;
                    let mut u = vec![];
                    let mut v = vec![];
                    let mut ids = vec![];
                    for _ in 0..4 {
                        let q = &runner.plan[runner.m["recorded_cells"].as_u64().unwrap() as usize];
                        a::check(
                            q["stage"] == "validation"
                                && q["block"] == r
                                && q["context"] == ctx
                                && q["draw"] == draw,
                            "base validation schedule",
                        )?;
                        ids.push(q["id"].clone());
                        let (s, x) = runner.evaluate(&emission, &theta[ctx])?;
                        u.push(x);
                        v.push(if s.is_some() { 1. } else { 0. });
                        native.push(s);
                    }
                    let utility = sum(u) / 4.;
                    let valid = sum(v) / 4.;
                    base.push(utility);
                    let mut ws = vec![];
                    for (i, vector) in vectors.iter().enumerate() {
                        let w = weight(&model, &theta[ctx], vector, selection)?;
                        let ratio = w["weight"].as_f64().unwrap();
                        utility_terms[i].push((utility - 1.) * (ratio - 1.));
                        valid_terms[i].push((valid - 1.) * (ratio - 1.));
                        ws.push(w);
                    }
                    terms.push(json!({"draw":draw,"selection":selection,"evaluation_ids":ids,"utility":utility,"validity":valid,"weights":ws}));
                }
                let ue: Vec<_> = utility_terms
                    .iter()
                    .map(|v| sum(v.iter().copied()) / draws as f64)
                    .collect();
                let ve: Vec<_> = valid_terms
                    .iter()
                    .map(|v| sum(v.iter().copied()) / draws as f64)
                    .collect();
                for method in 0..2 {
                    for e in 0..2 {
                        effects[r][ctx][method][e] = Some(ue[1 + 2 * method + e]);
                        validity[r][ctx][method][e] = Some(ve[1 + 2 * method + e]);
                    }
                }
                c::write(
                    &out.join("validation").join(format!("r{r}-c{ctx}.json")),
                    &json!({"block":r,"context":ctx,"terms":terms,"utility_effects":ue,"validity_effects":ve,
                "observed_base_mean_utility":sum(base)/draws as f64,"base_native_mean":if native.iter().all(Option::is_some){Some(sum(native.iter().map(|x|x.unwrap()))/(draws*4) as f64)}else{None}}),
                )?;
            }
            println!("categorical comparison validation block {} complete", r + 1);
        }
        c::recheck(&snapshot)?;
        let bytes: u64 = a::files(out)?
            .iter()
            .map(|n| {
                fs::metadata(out.join(n))
                    .map(|m| m.len())
                    .map_err(|e| e.to_string())
            })
            .collect::<Res<Vec<_>>>()?
            .iter()
            .sum();
        a::check(
            bytes < 2147483648 - 16 * 1024 * 1024 && Instant::now() < runner.deadline,
            "final resource ceiling",
        )?;
        runner.m["live_sources_rechecked"] = json!(true);
        runner.m["status"] = json!("complete");
        Ok(())
    });
    if let Err(reason) = &result {
        runner.m["status"] = json!("incomplete");
        runner.m["reason"] = json!(reason);
    }
    runner.m["elapsed_seconds"] = json!(setup + start.elapsed().as_secs_f64());
    runner.save()?;
    c::write(
        &out.join("summary.json"),
        &statistics(mode, &align, &effects, &validity, result.is_ok()),
    )?;
    c::seal(out)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schedules_equal_frozen_plan() {
        let root = c::repo().join("output/ere_categorical_comparison_preregistration_20261001");
        for dev in [false, true] {
            let name = if dev {
                "development_schedule.json"
            } else {
                "scientific_schedule.json"
            };
            assert_eq!(json!(schedule(dev)), a::load(&root.join(name)).unwrap());
        }
    }
    #[test]
    fn centered_radius_reverse_and_zero() {
        for base in direction::contexts() {
            let raw: Vec<_> = base
                .iter()
                .enumerate()
                .map(|(i, b)| (*b as f64 + 0.01 * (i as f64).sin()) as f32)
                .collect();
            let s = matched_step(&base, &raw).unwrap();
            assert_eq!(s["degenerate"], false);
            let f: Vec<f32> = serde_json::from_value(s["forward"].clone()).unwrap();
            let rev: Vec<f32> = serde_json::from_value(s["reverse"].clone()).unwrap();
            assert_eq!(rev, direction::reverse(&base, &f));
            assert_eq!(matched_step(&base, &base).unwrap()["degenerate"], true);
        }
        assert!(matched_step(&vec![f32::NAN; 183], &vec![0.; 183]).is_err());
    }
    #[test]
    fn fixed_statistics_missing_development_and_conjunction() {
        for mode in ["development", "scientific"] {
            let n = if mode == "scientific" { 8 } else { 1 };
            let mut align = vec![vec![vec![Some(1.); 2]; 2]; n];
            let effects = vec![vec![vec![vec![Some(0.01); 2], vec![Some(0.02); 2]]; 2]; n];
            let v = vec![vec![vec![vec![Some(0.); 2]; 2]; 2]; n];
            let s = statistics(mode, &align, &effects, &v, true);
            assert_eq!(s["support_categorical_direction"], mode == "scientific");
            align[0][0][1] = None;
            let s = statistics(mode, &align, &effects, &v, true);
            assert!(s["confirmatory"]["categorical_utility"]["mean"].is_null());
            assert_eq!(s["support_categorical_direction"], false);
        }
    }
    #[test]
    fn unapproved_preparation_runs_zero_work() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(run(tmp.path(), &"0".repeat(64)).is_err());
        assert!(!tmp.path().join("catalogue.json").exists());
    }
}
