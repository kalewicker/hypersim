//! Separately frozen native integration fixture. No scientific or training gate.
use super::{
    categorical as cat,
    categorical_comparison_bridge::weight,
    categorical_feasible::{self as proposal, Panel},
    complete_admission::{self as a, Res},
    confirmation as c, direction, feasibility_training as f,
    short_training::terminal,
};
use hg_sim_core::ere::{signal::Emission, valid_grammar};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-categorical-feasibility-development-v1";
const CONFIG_SHA: &str = "0150569bc1dbbb9922c47faa53073203c7b4a1206be945e5660e80fd22f3454f";

pub fn schedule() -> Vec<Value> {
    let master = 0xE66200000000_u64;
    let mut rows = vec![];
    for ctx in 0..2 {
        for est in 0..2 {
            for (stage, n, offset) in [
                ("utility_estimation", 16, 0),
                ("risk_estimation", 512, 1000),
            ] {
                for draw in 0..n {
                    let j = ctx * 100000 + est * 10000 + offset + draw;
                    rows.push(json!({"id":format!("est-c{ctx}-e{est}-{stage}-d{draw}"),"stage":stage,
                        "block":0,"context":ctx,"estimate":est,"draw":draw,
                        "emission_seed":master+20000000+j as u64,"physics_seed":master+40000000+j as u64,"horizon":100}));
                }
            }
        }
    }
    for ctx in 0..2 {
        for draw in 0..8 {
            for physics in 0..4 {
                let j = ctx * 100000;
                rows.push(json!({"id":format!("val-c{ctx}-d{draw}-p{physics}"),"stage":"validation","block":0,
                    "context":ctx,"draw":draw,"physics_index":physics,"sampled_arm":"base",
                    "emission_seed":master+60000000+(j+draw) as u64,
                    "physics_seed":master+80000000+(j+4*draw+physics) as u64,"horizon":100}));
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
    g: &valid_grammar::Grammar,
    model: &cat::Model,
    e: &Emission,
    r: &mut f::Runner<'_>,
) -> Res<()> {
    let s = model.selection_from_emission(&e.record)?;
    let seed = e.record["emission_seed"].as_u64().ok_or("emission seed")? ^ (1_u64 << 63);
    let rebuilt = g.emit_selected(s.parent, s.operator, s.role, s.production_index, seed)?;
    let mut expected = e.record.clone();
    expected["emission_seed"] = json!(seed);
    a::check(
        e.store.canonical_hash() == rebuilt.store.canonical_hash()
            && e.root == rebuilt.root
            && e.program == rebuilt.program
            && expected == rebuilt.record,
        "conditional leaf reconstruction",
    )?;
    r.m["conditional_leaf_checks"] = json!(r.m["conditional_leaf_checks"].as_u64().unwrap() + 1);
    r.save()
}

fn support(model: &cat::Model, base: &[f32], vectors: &[Vec<f32>]) -> Res<Value> {
    let mut rows = vec![];
    let mut mass = vec![0.; vectors.len()];
    let mut minimum = vec![f64::INFINITY; vectors.len()];
    let mut maximum = vec![0.0_f64; vectors.len()];
    for s in model
        .selections()
        .into_iter()
        .filter(|s| s.production_index == 0)
    {
        let b = model.score(base, s)?;
        let mut ratios = vec![];
        for (i, vector) in vectors.iter().enumerate() {
            let w = weight(model, base, vector, s)?;
            let ratio = w["weight"].as_f64().unwrap();
            mass[i] += b.probability * b.production_count as f64 * ratio;
            minimum[i] = minimum[i].min(ratio);
            maximum[i] = maximum[i].max(ratio);
            ratios.push(w);
        }
        rows.push(json!({"selection":s,"production_count":b.production_count,"ratios":ratios}));
    }
    a::check(
        rows.len() == 3232 && mass.iter().all(|x| (*x - 1.).abs() < 1e-12),
        "full support normalization",
    )?;
    Ok(json!({"legal_leaves":18956,"groups":rows,"mass":mass,"minimum":minimum,"maximum":maximum}))
}

fn summary(complete: bool, points: &Value) -> Value {
    json!({"profile":PROFILE,"mode":"development","measurement_complete":complete,
        "fixed_proposals":4,"fixed_validation_leaves_per_context":8,
        "context_points":if complete {points.clone()} else {Value::Null},
        "scientific_support":false,"training_admitted":false,"training_updates":0,
        "unseen_cap_feasibility_certificate":false,"independent_physics_replay":false,"independent_rng_replay":false})
}

fn prepared(out: &Path, root: &str, launch: bool) -> Res<(Value, Vec<Value>, Value)> {
    a::check(
        root.len() == 64 && a::digest(&out.join("prepared_sha256.json"))? == root,
        "prepared root",
    )?;
    let pins: BTreeMap<String, String> =
        serde_json::from_value(a::load(&out.join("prepared_sha256.json"))?)
            .map_err(|e| e.to_string())?;
    for (n, h) in &pins {
        a::check(
            !n.starts_with('/') && !n.split('/').any(|p| p == ".." || p.is_empty()),
            "safe prepared path",
        )?;
        a::check(
            a::digest(&out.join(n))? == *h,
            &format!("prepared input drift {n}"),
        )?;
    }
    let req = a::load(&out.join("request.json"))?;
    a::check(
        req["profile"] == PROFILE
            && req["mode"] == "development"
            && req["launch_authorized"].is_boolean()
            && req["full_training_authorized"] == false,
        "isolated development request",
    )?;
    if launch {
        a::check(
            req["launch_authorized"] == true,
            "explicit native development approval required",
        )?;
    }
    a::check(
        a::digest(&out.join("config.json"))? == CONFIG_SHA,
        "frozen numerical config",
    )?;
    let plan: Vec<Value> =
        serde_json::from_value(a::load(&out.join("schedule.json"))?).map_err(|e| e.to_string())?;
    a::check(
        plan == schedule() && plan.len() == 2176,
        "exact fixed schedule",
    )?;
    a::check(
        a::load(&out.join("contexts.json"))? == json!(direction::contexts()),
        "fixed contexts",
    )?;
    let snap = a::load(&out.join("snapshot.json"))?;
    c::recheck(&snap)?;
    let m = a::load(&out.join("manifest.json"))?;
    a::check(
        m == a::load(&out.join("initial_manifest.json"))?
            && m["status"] == "zero_work_preflight"
            && m["recorded_cells"] == 0
            && m["grammar_compilations"] == 0,
        "fresh zero-work preparation",
    )?;
    let process = std::process::Command::new("python3")
        .arg("-B")
        .arg(out.join("source_snapshots/scripts/audit_ere_categorical_feasible_development.py"))
        .arg(out)
        .output()
        .map_err(|e| e.to_string())?;
    a::check(
        process.status.success(),
        &format!(
            "frozen preflight: {}",
            String::from_utf8_lossy(&process.stderr)
        ),
    )?;
    let audit: Value = serde_json::from_slice(&process.stdout).map_err(|e| e.to_string())?;
    a::check(
        audit["audit_pass"] == true
            && audit["recorded_cells"] == 0
            && audit["grammar_compilations"] == 0
            && audit["campaign_complete"] == false
            && audit["source_registration_verified"] == true,
        "independent zero-work preflight",
    )?;
    c::recheck(&snap)?;
    Ok((req, plan, snap))
}

/// Read-only path: no Runner, grammar, RNG or Engine construction.
pub fn check_prepared(out: &Path, root: &str) -> Res<Value> {
    let _ = prepared(out, root, false)?;
    Ok(
        json!({"prepared_checked":true,"native_evaluations_run":0,"native_steps_run":0,"grammar_compilations_run":0,"training_updates":0}),
    )
}

pub fn run(out: &Path, root: &str) -> Res<()> {
    let start = Instant::now();
    let (req, plan, snapshot) = prepared(out, root, true)?;
    let setup = req["setup_elapsed_seconds"].as_f64().ok_or("setup wall")?;
    a::check(
        setup >= 0. && setup.is_finite() && setup < 900.,
        "setup wall ceiling",
    )?;
    let mut r = f::Runner {
        out,
        m: a::load(&out.join("manifest.json"))?,
        plan,
        deadline: start + Duration::from_secs_f64(900. - setup),
    };
    let theta = direction::contexts();
    let mut points = vec![];
    let result = terminal(|| {
        a::check(Instant::now() < r.deadline, "preflight wall ceiling")?;
        r.m["status"] = json!("compiling");
        r.m["grammar_compilations"] = json!(1);
        r.save()?;
        let compile = Instant::now();
        let grammar = valid_grammar::compile()?;
        r.m["grammar_compile_seconds"] = json!(compile.elapsed().as_secs_f64());
        c::write(&out.join("catalogue.json"), &grammar.catalogue())?;
        a::check(
            a::digest(&out.join("catalogue.json"))? == a::CATALOGUE_SHA,
            "catalogue identity",
        )?;
        let model = cat::Model::from_grammar(&grammar)?;
        c::recheck(&snapshot)?;
        r.m["status"] = json!("estimation");
        r.save()?;
        for ctx in 0..2 {
            let mut vectors = vec![theta[ctx].clone()];
            for est in 0..2 {
                c::recheck(&snapshot)?;
                let begin = r.m["recorded_cells"].as_u64().unwrap() as usize;
                let qs = r.plan[begin..begin + 528].to_vec();
                let path = out.join("estimates").join(format!("c{ctx}-e{est}.json"));
                let mut rec = json!({"context":ctx,"estimate":est,"theta_before":theta[ctx],
                    "evaluation_ids":qs.iter().map(|q|q["id"].clone()).collect::<Vec<_>>(),
                    "complete":false,"request":null,"proposal":null});
                c::write(&path, &rec)?;
                r.m["direction_starts"] = json!(r.m["direction_starts"].as_u64().unwrap() + 1);
                r.save()?;
                let mut panels = vec![];
                for (stage, size) in [("utility_estimation", 16), ("risk_estimation", 512)] {
                    let mut samples = vec![];
                    for _ in 0..size {
                        let q = r.plan[r.m["recorded_cells"].as_u64().unwrap() as usize].clone();
                        a::check(
                            q["stage"] == stage && q["context"] == ctx && q["estimate"] == est,
                            "panel stage provenance",
                        )?;
                        let emission =
                            grammar.emit(&theta[ctx], q["emission_seed"].as_u64().unwrap())?;
                        let selection = model.selection_from_emission(&emission.record)?;
                        check_leaf(&grammar, &model, &emission, &mut r)?;
                        let (score, _) = r.evaluate(&emission, &theta[ctx])?;
                        samples.push(cat::Sample {
                            id: q["id"].as_str().unwrap().into(),
                            emission_seed: q["emission_seed"].as_u64().unwrap(),
                            physics_seed: q["physics_seed"].as_u64().unwrap(),
                            selection,
                            native_score: score,
                            reason: if score.is_none() {
                                Some("resource_cap".into())
                            } else {
                                None
                            },
                        });
                    }
                    panels.push(Panel {
                        stage: stage.into(),
                        expected_draws: size,
                        samples,
                    });
                }
                let proposed = proposal::propose(&model, &theta[ctx], &panels[0], &panels[1])?;
                vectors.push(
                    serde_json::from_value(proposed["theta_after"].clone())
                        .map_err(|e| e.to_string())?,
                );
                let panel_json = |p: &Panel| json!({"stage":p.stage,"expected_draws":p.expected_draws,"samples":p.samples});
                rec["request"] = json!({"theta":theta[ctx],"utility":panel_json(&panels[0]),"risk":panel_json(&panels[1])});
                rec["proposal"] = proposed;
                rec["complete"] = json!(true);
                c::write(&path, &rec)?;
                r.m["direction_estimates"] =
                    json!(r.m["direction_estimates"].as_u64().unwrap() + 1);
                r.save()?;
            }
            c::write(
                &out.join("directions").join(format!("c{ctx}.json")),
                &json!({"context":ctx,"vectors":vectors,
                "alignment":direction::alignment(&theta[ctx],&vectors[1],&vectors[2]),"full_support":support(&model,&theta[ctx],&vectors)?}),
            )?;
            println!("feasibility development context {ctx}: proposals fixed");
        }
        a::check(
            r.m["direction_estimates"] == 4,
            "all four proposals before validation",
        )?;
        r.m["status"] = json!("validation");
        r.save()?;
        for ctx in 0..2 {
            c::recheck(&snapshot)?;
            let d = a::load(&out.join("directions").join(format!("c{ctx}.json")))?;
            let vectors: Vec<Vec<f32>> =
                serde_json::from_value(d["vectors"].clone()).map_err(|e| e.to_string())?;
            let mut terms = vec![];
            let mut ue = vec![0.; 3];
            let mut ve = vec![0.; 3];
            let mut base = vec![];
            let mut native = vec![];
            for draw in 0..8 {
                let seed = r.plan[r.m["recorded_cells"].as_u64().unwrap() as usize]
                    ["emission_seed"]
                    .as_u64()
                    .unwrap();
                let emission = grammar.emit(&theta[ctx], seed)?;
                let selection = model.selection_from_emission(&emission.record)?;
                check_leaf(&grammar, &model, &emission, &mut r)?;
                let mut u = vec![];
                let mut v = vec![];
                let mut ids = vec![];
                for p in 0..4 {
                    let q = &r.plan[r.m["recorded_cells"].as_u64().unwrap() as usize];
                    a::check(
                        q["stage"] == "validation"
                            && q["context"] == ctx
                            && q["draw"] == draw
                            && q["physics_index"] == p,
                        "base-only validation schedule",
                    )?;
                    ids.push(q["id"].clone());
                    let (s, x) = r.evaluate(&emission, &theta[ctx])?;
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
                    ue[i] += (utility - 1.) * (ratio - 1.);
                    ve[i] += (valid - 1.) * (ratio - 1.);
                    ws.push(w);
                }
                terms.push(json!({"draw":draw,"selection":selection,"evaluation_ids":ids,"utility":utility,"validity":valid,"weights":ws}));
            }
            for x in ue.iter_mut().chain(ve.iter_mut()) {
                *x /= 8.;
            }
            let rec = json!({"block":0,"context":ctx,"terms":terms,"utility_effects":ue,"validity_effects":ve,
                "observed_base_mean_utility":sum(base)/8.,"base_native_mean":if native.iter().all(Option::is_some){Some(sum(native.iter().map(|x|x.unwrap()))/32.)}else{None}});
            c::write(&out.join("validation").join(format!("c{ctx}.json")), &rec)?;
            points.push(json!({"context":ctx,"utility_effects":ue,"validity_effects":ve}));
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
            bytes < 2147483648 - 16 * 1024 * 1024 && Instant::now() < r.deadline,
            "final resource ceiling",
        )?;
        a::check(
            r.m["recorded_cells"] == 2176 && r.m["conditional_leaf_checks"] == 2128,
            "fixed completion counts",
        )?;
        r.m["live_sources_rechecked"] = json!(true);
        r.m["status"] = json!("complete");
        Ok(())
    });
    if let Err(reason) = &result {
        r.m["status"] = json!("incomplete");
        r.m["reason"] = json!(reason);
    }
    r.m["elapsed_seconds"] = json!(setup + start.elapsed().as_secs_f64());
    r.save()?;
    c::write(
        &out.join("summary.json"),
        &summary(result.is_ok(), &json!(points)),
    )?;
    c::seal(out)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_python_schedule() {
        let p=std::process::Command::new("python3").args(["-B","-c","import sys,json;sys.path.insert(0,'scripts');import ere_categorical_feasible_development as d;print(json.dumps(d.schedule()))"])
            .current_dir(c::repo()).output().unwrap();
        assert!(p.status.success(), "{}", String::from_utf8_lossy(&p.stderr));
        assert_eq!(
            json!(schedule()),
            serde_json::from_slice::<Value>(&p.stdout).unwrap()
        );
    }
    #[test]
    fn no_missing_preparation_work() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(check_prepared(tmp.path(), &"0".repeat(64)).is_err());
        assert!(run(tmp.path(), &"0".repeat(64)).is_err());
        assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), 0);
    }
    #[test]
    fn incomplete_points_not_promoted() {
        let s = summary(false, &json!([{"utility_effects":[1.,1.,1.]}]));
        assert!(s["context_points"].is_null());
        assert_eq!(s["scientific_support"], false);
        assert_eq!(summary(true, &json!([]))["training_admitted"], false);
    }
    #[test]
    fn full_support_including_base_holds_without_grammar() {
        let file = c::repo()
            .join("output/ere_categorical_feasible_engineering_v2_20261003/catalogue.json");
        let model = cat::Model::from_catalogue(&a::load(&file).unwrap()).unwrap();
        for base in direction::contexts() {
            let mut moved = base.clone();
            moved[0] += 0.05;
            let s = support(&model, &base, &[base.clone(), base.clone(), moved]).unwrap();
            assert_eq!(s["groups"].as_array().unwrap().len(), 3232);
            assert_eq!(s["minimum"][0], 1.);
            assert_eq!(s["maximum"][1], 1.);
            assert!(s["maximum"][2].as_f64().unwrap() > 1.);
        }
    }
}
