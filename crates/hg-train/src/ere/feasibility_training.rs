//! Constraint-aware ES adapter; native cap fitness remains null.
use super::{
    complete_admission::{self as a, Res},
    confirmation as c,
    short_training::{sign_p, terminal},
};
use crate::optim::{standard_normal, OpenAiEs, PairResult};
use hg_sim_core::ere::{
    signal,
    valid_grammar::{self, Grammar},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-complete-feasibility-training-v1";
pub const SEED: u64 = 0xE656000001;
pub const EXTRA: [&str; 4] = [
    "configs/ere_complete_feasibility_training_v1.json",
    "docs/ere_complete_feasibility_training_v1.md",
    "scripts/audit_ere_complete_feasibility_training.py",
    "scripts/test_audit_ere_complete_feasibility_training.py",
];
pub fn design() -> Value {
    json!({"profile":PROFILE,"master_seed":SEED,"horizon":100,"parameters":183,"initial_parameter_value":0.0,
        "generations":20,"candidates":16,"sigma":1.0,"alpha":0.05000000074505806,"held_out_draws":512,
        "training_physics":[SEED+100000,SEED+100001,SEED+100002,SEED+100003],
        "held_out_physics":[SEED+200000,SEED+200001,SEED+200002,SEED+200003],
        "work":50000,"results":50000,"wall_seconds":3600,"cell_wall_seconds":30,"output_cap_bytes":2147483648u64,
        "maximum_native_steps":646400,"objective":"feasible-first-reuse-v1","champion_selection":"final_generation_parameters",
        "precheck_evaluations":0,"full_training_authorized":false})
}
pub fn schedule() -> Vec<Value> {
    let mut q = vec![];
    for g in 0..20 {
        for p in 0..8 {
            for sign in [1, -1] {
                q.push(json!({"id":format!("train-{g}-{p}-{sign}"),"stage":"training","generation":g,"pair":p,"sign":sign,
            "emission_seed":SEED+5000+8*g+p,"physics_seed":SEED+100000+(g+p)%4,"horizon":100}));
            }
        }
    }
    for i in 0..512 {
        for (arm, name) in c::ARMS.iter().enumerate() {
            for p in 0..4 {
                q.push(json!({"id":format!("d{i}-{name}-p{p}"),"stage":"held_out","draw":i,"arm":arm,
            "emission_seed":SEED+2000+i,"random_parameter_seed":SEED+3000+i,"physics_seed":SEED+200000+p,"horizon":100}));
            }
        }
    }
    q
}
/// This utility is NOT native fitness. Only a measured resource cap maps to zero.
pub fn utility(score: Option<f64>, reason: Option<&str>) -> Res<f64> {
    match (score, reason) {
        (Some(x), None) if x.is_finite() && (0.0..=1.0).contains(&x) => Ok(1.0 + x),
        (None, Some("resource_cap")) => Ok(0.0),
        _ => Err("unsupported outcome for feasibility objective".into()),
    }
}
/// Dense ordinal scores preserve native f32 rank ordering without shift rounding.
/// All pairs participate; two capped arms receive identical scores and no update.
pub fn optimizer_pairs(scores: &[Option<f64>]) -> Res<Vec<PairResult>> {
    a::check(scores.len() == 16, "fixed sixteen candidate outcomes")?;
    let keys: Vec<f64> = scores
        .iter()
        .map(|s| match s {
            Some(x) if x.is_finite() && (0.0..=1.0).contains(x) => Ok(1.0 + f64::from(*x as f32)),
            None => Ok(0.0),
            _ => Err("invalid optimizer score".into()),
        })
        .collect::<Res<_>>()?;
    let mut levels = keys.clone();
    levels.sort_by(f64::total_cmp);
    levels.dedup();
    let fits: Vec<f32> = keys
        .iter()
        .map(|k| levels.iter().position(|v| v == k).unwrap() as f32)
        .collect();
    Ok(fits
        .chunks_exact(2)
        .map(|v| PairResult {
            plus: Some(v[0]),
            minus: Some(v[1]),
        })
        .collect())
}
pub fn statistics(
    scores: &[[[Option<f64>; 4]; 3]],
    utilities: &[[[Option<f64>; 4]; 3]],
    complete: bool,
) -> Value {
    assert_eq!(scores.len(), 512);
    assert_eq!(utilities.len(), 512);
    let mut s = c::statistics(scores);
    let means: Vec<[Option<f64>; 3]> = utilities
        .iter()
        .map(|d| {
            std::array::from_fn(|arm| {
                d[arm]
                    .iter()
                    .copied()
                    .collect::<Option<Vec<_>>>()
                    .map(|v| v.iter().sum::<f64>() / 4.0)
            })
        })
        .collect();
    let known = means.iter().all(|d| d.iter().all(Option::is_some));
    let counts: [usize; 3] = std::array::from_fn(|arm| {
        scores
            .iter()
            .flat_map(|d| d[arm])
            .filter(Option::is_some)
            .count()
    });
    let diffs: Vec<f64> = means
        .iter()
        .filter_map(|d| d[0].zip(d[1]).map(|(a, b)| a - b))
        .collect();
    let wins = diffs.iter().filter(|&&x| x > 0.0).count();
    let losses = diffs.iter().filter(|&&x| x < 0.0).count();
    let primary = known.then(|| diffs.iter().sum::<f64>() / 512.0);
    let p = known.then(|| sign_p(wins, losses));
    s["profile"] = json!(PROFILE);
    s["measurement_complete"] = json!(complete);
    s["improvement_supported"] = json!(s["improvement_supported"] == true && complete);
    s["constraint_utility"] = json!({"all_outcomes_known":known,"fixed_denominator":512,
        "arm_means":c::ARMS.iter().enumerate().map(|(a,n)|(*n,known.then(||means.iter().map(|d|d[a].unwrap()).sum::<f64>()/512.0))).collect::<BTreeMap<_,_>>(),
        "champion_minus_untrained":primary,"champion_minus_random":known.then(||means.iter().map(|d|d[0].unwrap()-d[2].unwrap()).sum::<f64>()/512.0),
        "wins":wins,"losses":losses,"ties":diffs.len()-wins-losses,"complete_pairs":diffs.len(),"one_sided_paired_sign_p":p,
        "improvement_supported":complete && known && counts[0]>=counts[1] && primary.zip(p).is_some_and(|(g,p)|g>0.0 && p<0.05),
        "draw_means":means});
    s["feasibility"] = json!({"denominator_per_arm":2048,"valid_counts":counts,
        "rates":known.then(||counts.map(|n|n as f64/2048.0)),"champion_minus_untrained":known.then(||(counts[0] as f64-counts[1] as f64)/2048.0)});
    s
}
pub(super) struct Runner<'a> {
    pub(super) out: &'a Path,
    pub(super) m: Value,
    pub(super) plan: Vec<Value>,
    pub(super) deadline: Instant,
}
impl Runner<'_> {
    pub(super) fn save(&self) -> Res<()> {
        c::write(&self.out.join("manifest.json"), &self.m)
    }
    pub(super) fn evaluate(&mut self, e: &signal::Emission, theta: &[f32]) -> Res<(Option<f64>, f64)> {
        let i = self.m["recorded_cells"].as_u64().unwrap() as usize;
        let q = self.plan[i].clone();
        a::check(
            q["emission_seed"] == e.record["emission_seed"],
            "emission schedule",
        )?;
        a::check(Instant::now() < self.deadline, "campaign wall ceiling")?;
        self.m["current_cell"] = q.clone();
        self.m["inflight_native_work_unknown"] = json!(true);
        self.save()?;
        let start = Instant::now();
        let evaluated = terminal(|| {
            signal::evaluate_with_horizon(
                e,
                q["physics_seed"].as_u64().unwrap(),
                self.deadline.min(start + Duration::from_secs(30)),
                50000,
                50000,
                100,
            )
        });
        let mut row = json!({"schedule":q,"emission":e.record,"parameters":theta,"elapsed_seconds":start.elapsed().as_secs_f64()});
        let mut error = None;
        let mut score = None;
        let mut u = None;
        match evaluated {
            Ok(mut v) => {
                v["runtime_match_budget"] = json!({"policy":"Pooled","work":50000,"results":50000});
                score = v["score"].as_f64();
                row["status"] = json!(if score.is_some() { "Complete" } else { "Null" });
                match utility(score, v["reason"].as_str()) {
                    Ok(x) => u = Some(x),
                    Err(e) => error = Some(e),
                }
                row["evaluation"] = v;
            }
            Err(e) => {
                row["status"] = json!("Incomplete");
                row["reason"] = json!(e);
                row["evaluation"] = Value::Null;
                row["partial_steps_unknown"] = json!(true);
                error = Some(e);
            }
        }
        row["constraint_utility"] = json!(u);
        let size = serde_json::to_vec(&row).map_err(|e| e.to_string())?.len() as u64;
        a::check(
            self.m["receipt_bytes"].as_u64().unwrap() + size < 2147483648 - 128 * 1024 * 1024,
            "receipt output ceiling",
        )?;
        c::write(
            &self
                .out
                .join("cells")
                .join(format!("{}.json", q["id"].as_str().unwrap())),
            &row,
        )?;
        if row["evaluation"].is_object() {
            for k in ["attempted_steps", "advancing_steps"] {
                self.m[k] = json!(
                    self.m[k].as_u64().unwrap()
                        + row["evaluation"][k].as_u64().ok_or("native counter")?
                );
            }
        } else {
            self.m["unknown_partial_work_cells"] =
                json!(self.m["unknown_partial_work_cells"].as_u64().unwrap() + 1);
        }
        self.m["recorded_cells"] = json!(i + 1);
        self.m["inflight_native_work_unknown"] = json!(false);
        self.m["receipt_bytes"] = json!(self.m["receipt_bytes"].as_u64().unwrap() + size);
        self.save()?;
        if let Some(e) = error {
            return Err(e);
        }
        Ok((score, u.unwrap()))
    }
}
fn distribution(g: &Grammar, t: &[f32]) -> Res<Vec<Value>> {
    (0..64).map(|i|g.emit(t,SEED+1000+i).map(|e|json!({"seed":SEED+1000+i,"signature":e.record["signature"],"selection":e.record}))).collect()
}
pub fn run(parent: &Path, out: &Path) -> Res<()> {
    a::check(!out.exists(), "fresh feasibility training output required")?;
    let absolute = out
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(out.file_name().ok_or("output name")?);
    a::check(
        !absolute.starts_with(parent.canonicalize().map_err(|e| e.to_string())?),
        "output inside parent archive",
    )?;
    fs::create_dir(out).map_err(|e| e.to_string())?;
    for d in ["cells", "generations"] {
        fs::create_dir(out.join(d)).map_err(|e| e.to_string())?;
    }
    let plan = schedule();
    c::write(&out.join("schedule.json"), &json!(plan))?;
    for q in &plan {
        c::write(
            &out.join("cells")
                .join(format!("{}.json", q["id"].as_str().unwrap())),
            &json!({"schedule":q,"status":"NotRun","reason":"not_started","evaluation":null}),
        )?;
    }
    let start = Instant::now();
    let mut r = Runner {
        out,
        plan,
        deadline: start + Duration::from_secs(3600),
        m: json!({"profile":PROFILE,"design":design(),"status":"validating","reason":null,"scheduled_cells":6464,
        "schedule_sha256":a::digest(&out.join("schedule.json"))?,"recorded_cells":0,"attempted_steps":0,"advancing_steps":0,
        "unknown_partial_work_cells":0,"inflight_native_work_unknown":false,"grammar_compilations":0,"optimizer_generations":0,
        "receipt_bytes":0,"training_admitted":false,"full_training_authorized":false}),
    };
    r.save()?;
    let mut scores = vec![[[None; 4]; 3]; 512];
    let mut utilities = scores.clone();
    let result = terminal(|| {
        a::check(
            a::load(&c::repo().join(EXTRA[0]))? == design(),
            "registered design drift",
        )?;
        let (proof, pins, _) = c::prepare(parent)?;
        c::write(&out.join("parent_receipt.json"), &proof)?;
        let snapshot = c::freeze_with(out, &pins, &EXTRA)?;
        c::write(&out.join("snapshot.json"), &snapshot)?;
        r.m["snapshot_sha256"] = json!(a::digest(&out.join("snapshot.json"))?);
        r.save()?;
        let checked = std::process::Command::new("python3")
            .arg("-B")
            .arg(parent.join("source_snapshots/scripts/audit_ere_complete_short_training.py"))
            .arg(parent)
            .output()
            .map_err(|e| e.to_string())?;
        a::check(
            checked.status.success(),
            &format!(
                "parent frozen audit: {}",
                String::from_utf8_lossy(&checked.stderr)
            ),
        )?;
        let audit: Value = serde_json::from_slice(&checked.stdout).map_err(|e| e.to_string())?;
        a::check(
            audit["audit_pass"] == true
                && audit["training_complete"] == true
                && audit["precheck_pass"] == true,
            "parent admission refusal",
        )?;
        c::write(&out.join("parent_audit.json"), &audit)?;
        r.m["status"] = json!("compiling");
        r.m["grammar_compilations"] = json!(1);
        r.save()?;
        let compilation = Instant::now();
        let grammar = valid_grammar::compile()?;
        r.m["grammar_compile_seconds"] = json!(compilation.elapsed().as_secs_f64());
        c::write(&out.join("catalogue.json"), &grammar.catalogue())?;
        a::check(
            a::digest(&out.join("catalogue.json"))? == a::CATALOGUE_SHA,
            "catalogue identity",
        )?;
        c::recheck(&snapshot)?;
        r.m["training_admitted"] = json!(true);
        r.m["status"] = json!("training");
        r.save()?;
        let mut theta = vec![0.0; 183];
        let initial = distribution(&grammar, &theta)?;
        c::write(&out.join("initial_distribution.json"), &json!(initial))?;
        let mut optimizer = OpenAiEs::new(1.0, 0.05, 8, ChaCha8Rng::seed_from_u64(SEED + 10000));
        for generation in 0..20 {
            c::recheck(&snapshot)?;
            let before = theta.clone();
            let eps = optimizer.perturbations(183);
            let mut values = vec![];
            let mut ids = vec![];
            for noise in &eps {
                for sign in [1, -1] {
                    let t = optimizer.candidate(&theta, noise, sign as f32);
                    let q = r.plan[r.m["recorded_cells"].as_u64().unwrap() as usize].clone();
                    let e = grammar.emit(&t, q["emission_seed"].as_u64().unwrap())?;
                    values.push(r.evaluate(&e, &t)?.0);
                    ids.push(q["id"].clone());
                }
            }
            let pairs = optimizer_pairs(&values)?;
            optimizer.update(&mut theta, &eps, &pairs);
            let panel = distribution(&grammar, &theta)?;
            c::write(
                &out.join("generations")
                    .join(format!("train-{generation}.json")),
                &json!({"generation":generation,"theta_before":before,"eps":eps,"evaluation_ids":ids,"native_scores":values,
                    "optimizer_pair_scores":pairs.iter().map(|p|[p.plus,p.minus]).collect::<Vec<_>>(),
                    "update_applied":true,"theta_after":theta,"distribution":panel,"tv_from_initial":super::runtime_signal::tv(&initial,&panel)}),
            )?;
            r.m["optimizer_generations"] = json!(generation + 1);
            r.save()?;
            println!(
                "feasibility training generation {}: {}/16 native-valid candidates",
                generation + 1,
                values.iter().flatten().count()
            );
        }
        c::write(
            &out.join("champion.json"),
            &json!({"parameters":theta,"selection":"final_generation_parameters","generation":20}),
        )?;
        r.m["status"] = json!("held_out");
        r.save()?;
        for i in 0..512 {
            if i % 64 == 0 {
                c::recheck(&snapshot)?;
            }
            let mut rng = ChaCha8Rng::seed_from_u64(SEED + 3000 + i as u64);
            let random: Vec<f32> = (0..183).map(|_| standard_normal(&mut rng)).collect();
            let zero = vec![0.0; 183];
            for (arm, t) in [&theta, &zero, &random].into_iter().enumerate() {
                let e = grammar.emit(t, SEED + 2000 + i as u64)?;
                for p in 0..4 {
                    let (score, u) = r.evaluate(&e, t)?;
                    scores[i][arm][p] = score;
                    utilities[i][arm][p] = Some(u);
                }
            }
            if (i + 1) % 64 == 0 {
                println!("feasibility held-out: {}/512 draws", i + 1);
            }
        }
        c::recheck(&snapshot)?;
        c::prepare(parent)?;
        let total: u64 = a::files(out)?
            .iter()
            .map(|p| {
                fs::metadata(out.join(p))
                    .map(|m| m.len())
                    .map_err(|e| e.to_string())
            })
            .collect::<Res<Vec<_>>>()?
            .iter()
            .sum();
        a::check(
            total < 2147483648 - 16 * 1024 * 1024 && Instant::now() < r.deadline,
            "final campaign resource ceiling",
        )?;
        r.m["live_sources_rechecked"] = json!(true);
        r.m["status"] = json!("complete");
        Ok(())
    });
    if let Err(reason) = &result {
        r.m["status"] = json!(if r.m["recorded_cells"] == 0 {
            "blocked_before_evaluation"
        } else {
            "incomplete"
        });
        r.m["reason"] = json!(reason);
    }
    r.m["elapsed_seconds"] = json!(start.elapsed().as_secs_f64());
    r.save()?;
    let mut summary = statistics(&scores, &utilities, r.m["status"] == "complete");
    for k in ["attempted_steps", "advancing_steps"] {
        summary[k] = r.m[k].clone();
    }
    c::write(&out.join("summary.json"), &summary)?;
    c::seal(out)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optim::rank_shape;
    #[test]
    fn capped_pair_direction_and_ties() {
        let mut scores = vec![Some(0.0); 16];
        scores[0] = None;
        let pairs = optimizer_pairs(&scores).unwrap();
        assert!(pairs[0].plus.unwrap() < pairs[0].minus.unwrap());
        let es = OpenAiEs::new(1.0, 0.05, 8, ChaCha8Rng::seed_from_u64(1));
        let mut eps = vec![vec![0.0; 183]; 8];
        eps[0][0] = 1.0;
        let mut theta = vec![0.0; 183];
        es.update(&mut theta, &eps, &pairs);
        assert!(theta[0] < 0.0, "update favors the feasible minus arm");
        scores[1] = None;
        let pairs = optimizer_pairs(&scores).unwrap();
        assert_eq!(pairs[0].plus, pairs[0].minus);
        let before = theta.clone();
        es.update(&mut theta, &eps, &pairs);
        assert_eq!(theta, before, "both-capped pair has zero contribution");
        assert_eq!(utility(None, Some("resource_cap")).unwrap(), 0.0);
        assert_eq!(utility(Some(0.0), None).unwrap(), 1.0);
        assert!(utility(None, Some("invalid_family")).is_err());
        assert!(utility(Some(f64::NAN), None).is_err());
        let p = optimizer_pairs(&vec![None; 16]).unwrap();
        assert!(p.iter().all(|p| p.plus == p.minus));
        es.update(&mut theta, &eps, &p);
        assert_eq!(theta, before, "all-capped generation has zero update");
    }
    #[test]
    fn all_feasible_rank_and_update_compatibility() {
        let scores: Vec<_> = (0..16)
            .map(|i| {
                Some(if i < 4 {
                    0.0
                } else {
                    f64::from(f32::from_bits(0x3f000000 + i))
                })
            })
            .collect();
        let old: Vec<_> = scores.iter().map(|s| s.map(|x| x as f32)).collect();
        let pairs = optimizer_pairs(&scores).unwrap();
        let mapped: Vec<_> = pairs.iter().flat_map(|p| [p.plus, p.minus]).collect();
        assert_eq!(rank_shape(&old), rank_shape(&mapped));
        let es = OpenAiEs::new(1.0, 0.05, 8, ChaCha8Rng::seed_from_u64(1));
        let eps: Vec<_> = (0..8).map(|i| vec![i as f32; 183]).collect();
        let legacy: Vec<_> = old
            .chunks_exact(2)
            .map(|p| PairResult {
                plus: p[0],
                minus: p[1],
            })
            .collect();
        let mut a = vec![0.0; 183];
        let mut b = a.clone();
        es.update(&mut a, &eps, &legacy);
        es.update(&mut b, &eps, &pairs);
        assert_eq!(a, b);
    }
    #[test]
    fn fixed_design_and_null_native_statistics() {
        assert_eq!(a::load(&c::repo().join(EXTRA[0])).unwrap(), design());
        assert_eq!(schedule().len(), 6464);
        let scores = vec![[[None; 4]; 3]; 512];
        let u = vec![[[Some(0.0); 4]; 3]; 512];
        let s = statistics(&scores, &u, true);
        assert!(s["champion_minus_untrained"].is_null());
        assert_eq!(s["constraint_utility"]["one_sided_paired_sign_p"], 1.0);
        assert_eq!(s["constraint_utility"]["all_outcomes_known"], true);
        assert_eq!(s["feasibility"]["valid_counts"], json!([0, 0, 0]));
        let s = statistics(&scores, &scores, false);
        assert!(s["constraint_utility"]["champion_minus_untrained"].is_null());
    }
}
