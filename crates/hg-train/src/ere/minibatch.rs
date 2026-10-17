//! Paired comparison of compute allocation to updates versus emission batches.
use super::{
    complete_admission::{self as a, Res},
    confirmation as c, feasibility_training as f,
    short_training::{sign_p, terminal},
};
use crate::optim::{OpenAiEs, PairResult};
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

pub const PROFILE: &str = "E2-complete-budget-matched-minibatch-v1";
pub const SEED: u64 = 0xE657000001;
pub const DEV_SEED: u64 = 0xE657D00001;
pub const LATEST_SEAL: &str = "f51ba18e5cabdedae4bab811e1cb6a3c86654b21355fbad8d07156f95f853432";
pub const EXTRA: [&str; 4] = [
    "configs/ere_complete_minibatch_v1.json",
    "docs/ere_complete_minibatch_v1.md",
    "scripts/audit_ere_complete_minibatch.py",
    "scripts/test_audit_ere_complete_minibatch.py",
];
pub const ARMS: [&str; 2] = ["single", "batch"];
#[derive(Clone, Copy)]
pub struct Layout {
    pub seed: u64,
    pub replicates: usize,
    pub single: usize,
    pub batch: usize,
    pub size: usize,
    pub draws: usize,
    pub development: bool,
}
impl Layout {
    pub fn registered(development: bool) -> Self {
        if development {
            Self {
                seed: DEV_SEED,
                replicates: 1,
                single: 2,
                batch: 1,
                size: 2,
                draws: 2,
                development,
            }
        } else {
            Self {
                seed: SEED,
                replicates: 8,
                single: 20,
                batch: 5,
                size: 4,
                draws: 64,
                development,
            }
        }
    }
    pub fn cells(self) -> usize {
        self.replicates * 2 * self.single * 16 + self.draws * 4 * (1 + 2 * self.replicates)
    }
    pub fn design(self) -> Value {
        json!({"profile":PROFILE,"mode":if self.development{"development"}else{"scientific"},"master_seed":self.seed,
            "replicates":self.replicates,"single_generations":self.single,"batch_generations":self.batch,"batch_size":self.size,
            "candidates":16,"parameters":183,"sigma":1.0,"alpha":0.05000000074505806,"horizon":100,
            "held_out_draws":self.draws,"held_out_physics":[self.seed+200000,self.seed+200001,self.seed+200002,self.seed+200003],
            "objective":"mean-constraint-utility-v1","statistical_unit":"paired_training_replicate",
            "maximum_native_steps":self.cells()*100,"wall_seconds":if self.development{900}else{3600},
            "cell_wall_seconds":30,"output_cap_bytes":2147483648u64,"work":50000,"results":50000,
            "precheck_evaluations":0,"champion_selection":"final_generation_parameters","full_training_authorized":false})
    }
    pub fn schedule(self) -> Vec<Value> {
        let mut q = vec![];
        for r in 0..self.replicates {
            for (arm, name) in ARMS.iter().enumerate() {
                let k = if arm == 0 { 1 } else { self.size };
                let gs = if arm == 0 { self.single } else { self.batch };
                for g in 0..gs {
                    for p in 0..8 {
                        for sign in [1, -1] {
                            for b in 0..k {
                                let pool = (8 * g + p) * k + b;
                                q.push(json!({"id":format!("train-r{r}-{name}-g{g}-p{p}-s{sign}-b{b}"),"stage":"training","replicate":r,
                    "arm":arm,"generation":g,"pair":p,"sign":sign,"batch_draw":b,"pool_index":pool,
                    "emission_seed":self.seed+10000+1000*r as u64+pool as u64,
                    "physics_seed":self.seed+100000+100*r as u64+(pool%4) as u64,"horizon":100}));
                            }
                        }
                    }
                }
            }
        }
        for i in 0..self.draws {
            for p in 0..4 {
                q.push(json!({"id":format!("hold-untrained-d{i}-p{p}"),"stage":"held_out","replicate":null,"arm":2,"draw":i,
                "emission_seed":self.seed+20000+i as u64,"physics_seed":self.seed+200000+p,"horizon":100}));
            }
        }
        for r in 0..self.replicates {
            for i in 0..self.draws {
                for (arm, name) in ARMS.iter().enumerate() {
                    for p in 0..4 {
                        q.push(json!({"id":format!("hold-r{r}-{name}-d{i}-p{p}"),"stage":"held_out","replicate":r,"arm":arm,"draw":i,
                "emission_seed":self.seed+20000+i as u64,"physics_seed":self.seed+200000+p,"horizon":100}));
                    }
                }
            }
        }
        q
    }
}
/// Mean utility across fixed draws, then ordinal rank input. Native null stays null.
pub fn batch_pairs(scores: &[Vec<Option<f64>>], k: usize) -> Res<(Vec<f64>, Vec<PairResult>)> {
    a::check(
        scores.len() == 16 && k > 0 && scores.iter().all(|s| s.len() == k),
        "fixed complete candidate batches",
    )?;
    let means: Vec<f64> = scores
        .iter()
        .map(|v| {
            v.iter()
                .map(|s| match s {
                    Some(x) if x.is_finite() && (0.0..=1.0).contains(x) => {
                        Ok(1.0 + f64::from(*x as f32))
                    }
                    None => Ok(0.0),
                    _ => Err("invalid batch optimizer score".into()),
                })
                .collect::<Res<Vec<_>>>()
                .map(|u| u.iter().sum::<f64>() / k as f64)
        })
        .collect::<Res<_>>()?;
    let mut levels = means.clone();
    levels.sort_by(f64::total_cmp);
    levels.dedup();
    let codes: Vec<f32> = means
        .iter()
        .map(|v| levels.iter().position(|x| x == v).unwrap() as f32)
        .collect();
    let pairs = codes
        .chunks_exact(2)
        .map(|v| PairResult {
            plus: Some(v[0]),
            minus: Some(v[1]),
        })
        .collect();
    Ok((means, pairs))
}
pub fn statistics(
    l: Layout,
    utility: &[Vec<[Option<f64>; 2]>],
    native: &[Vec<[Option<f64>; 2]>],
    baseline: &[Option<f64>],
    counts: &[[usize; 2]],
    complete: bool,
) -> Value {
    assert_eq!(utility.len(), l.replicates);
    assert_eq!(native.len(), l.replicates);
    assert_eq!(counts.len(), l.replicates);
    assert_eq!(baseline.len(), l.draws);
    assert!(utility.iter().chain(native).all(|v| v.len() == l.draws));
    let aggregate = |v: &[Vec<[Option<f64>; 2]>]| -> Vec<[Option<f64>; 2]> {
        v.iter()
            .map(|draws| {
                std::array::from_fn(|a| {
                    draws
                        .iter()
                        .map(|d| d[a])
                        .collect::<Option<Vec<_>>>()
                        .map(|s| s.iter().sum::<f64>() / l.draws as f64)
                })
            })
            .collect()
    };
    let us = aggregate(utility);
    let ns = aggregate(native);
    let known =
        us.iter().all(|d| d.iter().all(Option::is_some)) && baseline.iter().all(Option::is_some);
    let differences: Vec<_> = us
        .iter()
        .filter_map(|d| d[1].zip(d[0]).map(|(b, s)| b - s))
        .collect();
    let wins = differences.iter().filter(|&&v| v > 0.0).count();
    let losses = differences.iter().filter(|&&v| v < 0.0).count();
    let mean = known.then(|| differences.iter().sum::<f64>() / l.replicates as f64);
    let p = known.then(|| sign_p(wins, losses));
    let totals: [usize; 2] = std::array::from_fn(|a| counts.iter().map(|r| r[a]).sum());
    let native_known = ns.iter().all(|r| r.iter().all(Option::is_some));
    let native_diffs: Vec<_> = ns
        .iter()
        .filter_map(|d| d[1].zip(d[0]).map(|(b, s)| b - s))
        .collect();
    let native_p = native_known.then(|| {
        sign_p(
            native_diffs.iter().filter(|&&d| d > 0.0).count(),
            native_diffs.iter().filter(|&&d| d < 0.0).count(),
        )
    });
    json!({"profile":PROFILE,"mode":if l.development{"development"}else{"scientific"},"measurement_complete":complete,
        "statistical_unit":"paired_training_replicate","fixed_denominator":l.replicates,"draws_per_replicate":l.draws,
        "all_utility_outcomes_known":known,"replicate_utility_means":us,"replicate_native_means":ns,
        "batch_minus_single_utility":mean,"utility_sign_p":p,"wins":wins,"losses":losses,"ties":differences.len()-wins-losses,"complete_pairs":differences.len(),
        "method_utility_means":std::array::from_fn::<_,2,_>(|a|known.then(||us.iter().map(|r|r[a].unwrap()).sum::<f64>()/l.replicates as f64)),
        "untrained_utility_mean":baseline.iter().copied().collect::<Option<Vec<_>>>().map(|v|v.iter().sum::<f64>()/l.draws as f64),
        "feasible_counts":totals,"feasibility_denominator_per_method":l.replicates*l.draws*4,
        "batch_minus_single_native":native_known.then(||ns.iter().map(|r|r[1].unwrap()-r[0].unwrap()).sum::<f64>()/l.replicates as f64),
        "native_sign_p":native_p,"all_comparison_native_valid":native_known,
        "support_batching":!l.development && complete && known && totals[1]>=totals[0] && mean.zip(p).is_some_and(|(d,p)|d>0.0 && p<0.05),
        "independent_physics_replay":false})
}
fn verify_latest() -> Res<Value> {
    let root = c::repo().join("output/ere_complete_feasibility_training_20261001");
    a::check(
        a::digest(&root.join("output_sha256.json"))? == LATEST_SEAL,
        "latest experiment seal",
    )?;
    let entries: BTreeMap<String, String> =
        serde_json::from_value(a::load(&root.join("output_sha256.json"))?)
            .map_err(|e| e.to_string())?;
    let mut actual = a::files(&root)?;
    actual.remove("output_sha256.json");
    a::check(
        entries.len() == 6944 && actual == entries.keys().cloned().collect(),
        "latest exact coverage",
    )?;
    let paths: Vec<_> = entries.keys().map(|n| root.join(n)).collect();
    for ((n, h), v) in entries.iter().zip(a::hashes(&paths)?) {
        a::check(*h == v, &format!("latest changed {n}"))?;
    }
    let audit = a::load(&root.join("audit.json"))?;
    a::check(
        audit["audit_pass"] == true
            && audit["training_complete"] == true
            && audit["constraint_utility"]["improvement_supported"] == false,
        "latest diagnostic lineage",
    )?;
    Ok(
        json!({"archive":root,"root_seal":LATEST_SEAL,"sealed_files_verified":6944,"scope":"diagnostic lineage, not improvement"}),
    )
}
pub fn run(parent: &Path, out: &Path, development: bool) -> Res<()> {
    let l = Layout::registered(development);
    a::check(!out.exists(), "fresh minibatch output required")?;
    let absolute = out
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(out.file_name().ok_or("output name")?);
    a::check(
        !absolute.starts_with(parent.canonicalize().map_err(|e| e.to_string())?)
            && !absolute
                .starts_with(c::repo().join("output/ere_complete_feasibility_training_20261001")),
        "output inside evidence",
    )?;
    fs::create_dir(out).map_err(|e| e.to_string())?;
    for d in ["cells", "generations", "champions"] {
        fs::create_dir(out.join(d)).map_err(|e| e.to_string())?;
    }
    let plan = l.schedule();
    c::write(&out.join("schedule.json"), &json!(plan))?;
    for q in &plan {
        c::write(
            &out.join("cells")
                .join(format!("{}.json", q["id"].as_str().unwrap())),
            &json!({"schedule":q,"status":"NotRun","reason":"not_started","evaluation":null}),
        )?;
    }
    let start = Instant::now();
    let mut r = f::Runner {
        out,
        plan,
        deadline: start + Duration::from_secs(if development { 900 } else { 3600 }),
        m: json!({"profile":PROFILE,"design":l.design(),"status":"validating","reason":null,"scheduled_cells":l.cells(),"schedule_sha256":a::digest(&out.join("schedule.json"))?,
            "recorded_cells":0,"attempted_steps":0,"advancing_steps":0,"unknown_partial_work_cells":0,"inflight_native_work_unknown":false,
            "grammar_compilations":0,"optimizer_generations":0,"started_generations":0,"receipt_bytes":0,"training_admitted":false,"full_training_authorized":false}),
    };
    r.save()?;
    let mut utility = vec![vec![[None; 2]; l.draws]; l.replicates];
    let mut native = utility.clone();
    let mut baseline = vec![None; l.draws];
    let mut counts = vec![[0; 2]; l.replicates];
    let result = terminal(|| {
        let cfg = a::load(&c::repo().join(EXTRA[0]))?;
        a::check(
            cfg[if development {
                "development"
            } else {
                "scientific"
            }] == l.design(),
            "registered minibatch config",
        )?;
        let (proof, pins, _) = c::prepare(parent)?;
        c::write(&out.join("parent_receipt.json"), &proof)?;
        c::write(&out.join("latest_receipt.json"), &verify_latest()?)?;
        let extras: Vec<_> = f::EXTRA.iter().chain(EXTRA.iter()).copied().collect();
        let snapshot = c::freeze_with(out, &pins, &extras)?;
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
            "parent admission",
        )?;
        c::write(&out.join("parent_audit.json"), &audit)?;
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
        c::recheck(&snapshot)?;
        r.m["training_admitted"] = json!(true);
        r.m["status"] = json!("training");
        r.save()?;
        let mut champions = vec![vec![vec![0.0; 183]; 2]; l.replicates];
        for (rep, rep_champions) in champions.iter_mut().enumerate() {
            for (arm, theta) in rep_champions.iter_mut().enumerate() {
                let k = if arm == 0 { 1 } else { l.size };
                let gs = if arm == 0 { l.single } else { l.batch };
                let mut es = OpenAiEs::new(
                    1.0,
                    0.05,
                    8,
                    ChaCha8Rng::seed_from_u64(l.seed + 1000000 + rep as u64),
                );
                for g in 0..gs {
                    c::recheck(&snapshot)?;
                    let eps = es.perturbations(183);
                    let begin = r.m["recorded_cells"].as_u64().unwrap() as usize;
                    let qs = &r.plan[begin..begin + 16 * k];
                    let ids: Vec<_> = qs.iter().map(|q| q["id"].clone()).collect();
                    let mut rec = json!({"replicate":rep,"arm":arm,"generation":g,"theta_before":theta,"eps":eps,"evaluation_ids":ids,
                    "update_applied":false,"candidate_means":null,"optimizer_pair_scores":null,"theta_after":null});
                    let path = out
                        .join("generations")
                        .join(format!("r{rep}-{}-g{g}.json", ARMS[arm]));
                    c::write(&path, &rec)?;
                    r.m["started_generations"] =
                        json!(r.m["started_generations"].as_u64().unwrap() + 1);
                    r.save()?;
                    let mut values = vec![];
                    for noise in &eps {
                        for sign in [1, -1] {
                            let candidate = es.candidate(theta, noise, sign as f32);
                            let mut batch = vec![];
                            for _ in 0..k {
                                let q = &r.plan[r.m["recorded_cells"].as_u64().unwrap() as usize];
                                let e = grammar
                                    .emit(&candidate, q["emission_seed"].as_u64().unwrap())?;
                                batch.push(r.evaluate(&e, &candidate)?.0);
                            }
                            values.push(batch);
                        }
                    }
                    let (means, pairs) = batch_pairs(&values, k)?;
                    es.update(theta, &eps, &pairs);
                    rec["update_applied"] = json!(true);
                    rec["candidate_means"] = json!(means);
                    rec["optimizer_pair_scores"] =
                        json!(pairs.iter().map(|p| [p.plus, p.minus]).collect::<Vec<_>>());
                    rec["theta_after"] = json!(theta);
                    c::write(&path, &rec)?;
                    r.m["optimizer_generations"] =
                        json!(r.m["optimizer_generations"].as_u64().unwrap() + 1);
                    r.save()?;
                }
                c::write(
                    &out.join("champions")
                        .join(format!("r{rep}-{}.json", ARMS[arm])),
                    &json!({"parameters":theta,"replicate":rep,"arm":arm,"generation":gs,"selection":"final_generation_parameters"}),
                )?;
                println!(
                    "minibatch {}: replicate {}/{} {} training complete",
                    if development {
                        "development"
                    } else {
                        "scientific"
                    },
                    rep + 1,
                    l.replicates,
                    ARMS[arm]
                );
            }
        }
        r.m["status"] = json!("held_out");
        r.save()?;
        let zero = vec![0.0; 183];
        for value in &mut baseline {
            let mut us = vec![];
            for _ in 0..4 {
                let q = &r.plan[r.m["recorded_cells"].as_u64().unwrap() as usize];
                let e = grammar.emit(&zero, q["emission_seed"].as_u64().unwrap())?;
                us.push(r.evaluate(&e, &zero)?.1);
            }
            *value = Some(us.iter().sum::<f64>() / 4.0);
        }
        for rep in 0..l.replicates {
            c::recheck(&snapshot)?;
            for i in 0..l.draws {
                for arm in 0..2 {
                    let theta = &champions[rep][arm];
                    let e = grammar.emit(theta, l.seed + 20000 + i as u64)?;
                    let mut scores = vec![];
                    let mut us = vec![];
                    for _ in 0..4 {
                        let (s, u) = r.evaluate(&e, theta)?;
                        counts[rep][arm] += usize::from(s.is_some());
                        scores.push(s);
                        us.push(u);
                    }
                    native[rep][i][arm] = scores
                        .iter()
                        .copied()
                        .collect::<Option<Vec<_>>>()
                        .map(|s| s.iter().sum::<f64>() / 4.0);
                    utility[rep][i][arm] = Some(us.iter().sum::<f64>() / 4.0);
                }
            }
            println!(
                "minibatch held-out: replicate {}/{} complete",
                rep + 1,
                l.replicates
            );
        }
        c::recheck(&snapshot)?;
        c::prepare(parent)?;
        verify_latest()?;
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
            "final minibatch resource ceiling",
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
    let mut summary = statistics(
        l,
        &utility,
        &native,
        &baseline,
        &counts,
        r.m["status"] == "complete",
    );
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
    fn wrong_receipt_refuses_with_audited_zero_native_work() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = tmp.path().join("bad-parent");
        let out = tmp.path().join("refusal");
        fs::create_dir(&parent).unwrap();
        c::write(&parent.join("output_sha256.json"), &json!({})).unwrap();
        assert!(run(&parent, &out, true).is_err());
        let m = a::load(&out.join("manifest.json")).unwrap();
        assert_eq!(m["status"], "blocked_before_evaluation");
        for k in [
            "recorded_cells",
            "attempted_steps",
            "advancing_steps",
            "grammar_compilations",
            "optimizer_generations",
        ] {
            assert_eq!(m[k], 0);
        }
        assert_eq!(m["training_admitted"], false);
        let checked = std::process::Command::new("python3")
            .arg("-B")
            .arg(c::repo().join("scripts/audit_ere_complete_minibatch.py"))
            .arg(&out)
            .output()
            .unwrap();
        assert!(
            checked.status.success(),
            "{}",
            String::from_utf8_lossy(&checked.stderr)
        );
        let audit: Value = serde_json::from_slice(&checked.stdout).unwrap();
        assert_eq!(audit["classification"], "zero_work_refusal");
    }
    #[test]
    fn matched_draw_pools_and_independent_replicates() {
        for development in [false, true] {
            let l = Layout::registered(development);
            assert_eq!(l.single, l.batch * l.size);
            let q = l.schedule();
            assert_eq!(q.len(), l.cells());
            let cfg = a::load(&c::repo().join(EXTRA[0])).unwrap();
            assert_eq!(
                cfg[if development {
                    "development"
                } else {
                    "scientific"
                }],
                l.design()
            );
            for r in 0..l.replicates {
                let mut pools: [Vec<_>; 2] = std::array::from_fn(|arm| {
                    q.iter()
                        .filter(|q| {
                            q["stage"] == "training" && q["replicate"] == r && q["arm"] == arm
                        })
                        .map(|q| {
                            (
                                q["emission_seed"].as_u64().unwrap(),
                                q["physics_seed"].as_u64().unwrap(),
                            )
                        })
                        .collect()
                });
                pools[0].sort();
                pools[1].sort();
                assert_eq!(pools[0], pools[1]);
                assert_eq!(pools[0].len(), l.single * 16);
            }
        }
    }
    #[test]
    fn batch_one_compatibility_caps_and_fixed_batch_denominator() {
        let scores: Vec<_> = (0..16)
            .map(|i| {
                if i == 0 {
                    None
                } else {
                    Some((i % 5) as f64 / 5.0)
                }
            })
            .collect();
        let old = f::optimizer_pairs(&scores).unwrap();
        let (_, new) =
            batch_pairs(&scores.iter().map(|x| vec![*x]).collect::<Vec<_>>(), 1).unwrap();
        let flatten =
            |p: Vec<PairResult>| p.iter().flat_map(|p| [p.plus, p.minus]).collect::<Vec<_>>();
        assert_eq!(rank_shape(&flatten(old)), rank_shape(&flatten(new)));
        let mut batches = vec![vec![Some(0.0); 4]; 16];
        batches[0][0] = None;
        let (means, pairs) = batch_pairs(&batches, 4).unwrap();
        assert_eq!(means[0], 0.75);
        assert!(pairs[0].plus.unwrap() < pairs[0].minus.unwrap());
        assert!(batch_pairs(&batches, 3).is_err());
        let (means, p) = batch_pairs(&vec![vec![None; 4]; 16], 4).unwrap();
        assert_eq!(means, vec![0.0; 16]);
        assert!(p.iter().all(|p| p.plus == p.minus));
    }
    #[test]
    fn replicate_not_rollout_is_statistical_unit() {
        let l = Layout::registered(false);
        let u = vec![vec![[Some(1.0), Some(2.0)]; 64]; 8];
        let n = vec![vec![[Some(0.0), Some(1.0)]; 64]; 8];
        let s = statistics(l, &u, &n, &vec![Some(1.0); 64], &vec![[256, 256]; 8], true);
        assert_eq!(s["fixed_denominator"], 8);
        assert_eq!(s["utility_sign_p"], 1.0 / 256.0);
        assert_eq!(s["support_batching"], true);
        let mut u = u;
        u[0][0][1] = None;
        let s = statistics(l, &u, &n, &vec![Some(1.0); 64], &vec![[256, 256]; 8], false);
        assert!(s["batch_minus_single_utility"].is_null());
        assert!(s["utility_sign_p"].is_null());
        assert_eq!(s["fixed_denominator"], 8);
    }
}
