//! Fixed-parameter, shared-perturbation update reliability; not training.
use super::{
    complete_admission::{self as a, Res},
    confirmation as c, feasibility_training as f, minibatch as b,
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

pub const PROFILE: &str = "E2-complete-direction-reliability-v1";
pub const LATEST: &str = "49a2bca085a7f7619c2bde3b5a25992a8fc16d8418d97165a2db3fb9db698806";
pub const EXTRA: [&str; 4] = [
    "configs/ere_complete_direction_v1.json",
    "docs/ere_complete_direction_v1.md",
    "scripts/audit_ere_complete_direction.py",
    "scripts/test_audit_ere_complete_direction.py",
];
pub const ARMS: [&str; 5] = ["base", "forward_a", "reverse_a", "forward_b", "reverse_b"];
#[derive(Clone, Copy)]
pub struct Layout {
    pub development: bool,
    pub seed: u64,
    pub blocks: usize,
    pub k: usize,
    pub draws: usize,
}
impl Layout {
    pub fn registered(development: bool) -> Self {
        Self {
            development,
            seed: if development {
                0xE658D00001
            } else {
                0xE658000001
            },
            blocks: if development { 1 } else { 8 },
            k: if development { 2 } else { 4 },
            draws: if development { 2 } else { 16 },
        }
    }
    pub fn cells(self) -> usize {
        self.blocks * 2 * (32 * self.k + 20 * self.draws)
    }
    pub fn design(self) -> Value {
        json!({"profile":PROFILE,"mode":if self.development{"development"}else{"scientific"},
        "master_seed":self.seed,"blocks":self.blocks,"contexts":2,"batch_size":self.k,"validation_draws":self.draws,
        "validation_physics":4,"candidates":16,"parameters":183,"sigma":1.0,"alpha":0.05000000074505806,
        "horizon":100,"work":50000,"results":50000,"wall_seconds":if self.development{900}else{3600},
        "cell_wall_seconds":30,"output_cap_bytes":2147483648u64,"maximum_native_steps":self.cells()*100,
        "objective":"mean-constraint-utility-v1","statistical_unit":"paired_estimator_block","zero_direction_alignment":0,
        "parameter_panel":"zero-and-alternating-quarter-v1","validation_arms":ARMS,"optimizer_generations":0,
        "precheck_evaluations":0,"full_training_authorized":false})
    }
    pub fn schedule(self) -> Vec<Value> {
        let mut q = vec![];
        for r in 0..self.blocks {
            for ctx in 0..2 {
                for e in 0..2 {
                    for p in 0..8 {
                        for s in [1, -1] {
                            for draw in 0..self.k {
                                q.push(json!({"id":format!("estimate-r{r}-c{ctx}-e{e}-p{p}-s{s}-b{draw}"),"stage":"estimation","block":r,
                "context":ctx,"estimate":e,"pair":p,"sign":s,"batch_draw":draw,
                "emission_seed":self.seed+10000+1000*r as u64+100*ctx+40*e+self.k as u64*p+draw as u64,
                "physics_seed":self.seed+100000+100*r as u64+10*ctx+4*e+(draw%4) as u64,"horizon":100}));
                            }
                        }
                    }
                }
            }
        }
        for r in 0..self.blocks {
            for ctx in 0..2 {
                for draw in 0..self.draws {
                    for (arm, name) in ARMS.iter().enumerate() {
                        for physics in 0..4 {
                            q.push(json!({"id":format!("validate-r{r}-c{ctx}-d{draw}-{name}-p{physics}"),"stage":"validation","block":r,
                "context":ctx,"draw":draw,"arm":arm,"physics_index":physics,
                "emission_seed":self.seed+20000+1000*r as u64+100*ctx+draw as u64,
                "physics_seed":self.seed+200000+100*r as u64+10*ctx+physics,"horizon":100}));
                        }
                    }
                }
            }
        }
        q
    }
}
pub fn contexts() -> [Vec<f32>; 2] {
    [
        vec![0.; 183],
        (0..183)
            .map(|i| if i % 2 == 0 { 0.25 } else { -0.25 })
            .collect(),
    ]
}
pub fn reverse(theta: &[f32], forward: &[f32]) -> Vec<f32> {
    theta.iter().zip(forward).map(|(t, u)| 2. * t - u).collect()
}
pub fn alignment(theta: &[f32], first: &[f32], second: &[f32]) -> Value {
    assert_eq!(theta.len(), first.len());
    assert_eq!(theta.len(), second.len());
    let x: Vec<_> = first
        .iter()
        .zip(theta)
        .map(|(a, t)| f64::from(*a) - f64::from(*t))
        .collect();
    let y: Vec<_> = second
        .iter()
        .zip(theta)
        .map(|(a, t)| f64::from(*a) - f64::from(*t))
        .collect();
    let nx = x.iter().map(|v| v * v).sum::<f64>().sqrt();
    let ny = y.iter().map(|v| v * v).sum::<f64>().sqrt();
    let degenerate = nx == 0. || ny == 0.;
    let score = if degenerate {
        0.
    } else {
        (x.iter().zip(&y).map(|(a, b)| a * b).sum::<f64>() / (nx * ny)).clamp(-1., 1.)
    };
    json!({"score":score,"degenerate":degenerate,"norm_a":nx,"norm_b":ny})
}
pub fn statistics(
    l: Layout,
    align: &[[Option<Value>; 2]],
    utilities: &[[[Option<f64>; 5]; 2]],
    natives: &[[[Option<f64>; 5]; 2]],
    counts: &[[[usize; 5]; 2]],
    complete: bool,
) -> Value {
    assert_eq!(align.len(), l.blocks);
    assert_eq!(utilities.len(), l.blocks);
    assert_eq!(natives.len(), l.blocks);
    assert_eq!(counts.len(), l.blocks);
    let known = align.iter().flatten().all(Option::is_some)
        && utilities.iter().flatten().flatten().all(Option::is_some);
    let block_align: Vec<_> = align
        .iter()
        .map(|v| {
            v.iter()
                .map(|a| a.as_ref().and_then(|a| a["score"].as_f64()))
                .collect::<Option<Vec<_>>>()
                .map(|v| v.iter().sum::<f64>() / 2.)
        })
        .collect();
    let effects: Vec<[Option<f64>; 2]> = utilities
        .iter()
        .map(|r| {
            std::array::from_fn(|ctx| {
                let u = r[ctx];
                u[1].zip(u[3])
                    .zip(u[0])
                    .map(|((a, b), base)| (a + b) / 2. - base)
            })
        })
        .collect();
    let block_effects: Vec<_> = effects
        .iter()
        .map(|v| v[0].zip(v[1]).map(|(a, b)| (a + b) / 2.))
        .collect();
    let stats = |values: &[Option<f64>]| {
        let v: Vec<_> = values.iter().filter_map(|v| *v).collect();
        let wins = v.iter().filter(|&&x| x > 0.).count();
        let losses = v.iter().filter(|&&x| x < 0.).count();
        json!({"mean":known.then(||v.iter().sum::<f64>()/l.blocks as f64),"sign_p":known.then(||sign_p(wins,losses)),
            "wins":wins,"losses":losses,"ties":v.len()-wins-losses,"complete_blocks":v.len()})
    };
    let reliability = stats(&block_align);
    let usefulness = stats(&block_effects);
    let context_effects: [Option<f64>; 2] = std::array::from_fn(|ctx| {
        known.then(|| effects.iter().map(|v| v[ctx].unwrap()).sum::<f64>() / l.blocks as f64)
    });
    let valid: [usize; 5] = std::array::from_fn(|a| {
        counts
            .iter()
            .map(|r| r.iter().map(|c| c[a]).sum::<usize>())
            .sum()
    });
    let native_known = natives.iter().flatten().flatten().all(Option::is_some);
    let arm_means = |values: &[[[Option<f64>; 5]; 2]], ok: bool| {
        std::array::from_fn::<_, 5, _>(|a| {
            ok.then(|| {
                values
                    .iter()
                    .map(|r| r.iter().map(|c| c[a].unwrap()).sum::<f64>())
                    .sum::<f64>()
                    / (2 * l.blocks) as f64
            })
        })
    };
    let utility_means = arm_means(utilities, known);
    let support = !l.development
        && complete
        && known
        && reliability["mean"].as_f64().unwrap() > 0.
        && reliability["sign_p"].as_f64().unwrap() < 0.05
        && usefulness["mean"].as_f64().unwrap() > 0.
        && usefulness["sign_p"].as_f64().unwrap() < 0.05
        && context_effects.iter().all(|v| v.unwrap() > 0.)
        && valid[1] + valid[3] >= 2 * valid[0];
    json!({"profile":PROFILE,"mode":if l.development{"development"}else{"scientific"},"measurement_complete":complete,
        "statistical_unit":"paired_estimator_block","fixed_denominator":l.blocks,"all_outcomes_known":known,
        "context_alignment":align,"block_alignment":block_align,"reliability":reliability,
        "context_utility_effects":effects,"context_mean_effects":context_effects,"block_utility_effects":block_effects,"usefulness":usefulness,
        "utility_arm_means":utility_means,"native_arm_means":arm_means(natives,native_known),"all_validation_native_valid":native_known,
        "valid_counts":valid,"feasibility_denominator_per_arm":l.blocks*2*l.draws*4,
        "forward_minus_reverse_descriptive":known.then(||(utility_means[1].unwrap()+utility_means[3].unwrap()-utility_means[2].unwrap()-utility_means[4].unwrap())/2.),
        "support_direction":support,"independent_physics_replay":false})
}
fn latest() -> Res<Value> {
    let root = c::repo().join("output/ere_complete_minibatch_trial_20261001");
    a::check(
        a::digest(&root.join("output_sha256.json"))? == LATEST,
        "minibatch lineage seal",
    )?;
    let entries: BTreeMap<String, String> =
        serde_json::from_value(a::load(&root.join("output_sha256.json"))?)
            .map_err(|e| e.to_string())?;
    let mut actual = a::files(&root)?;
    actual.remove("output_sha256.json");
    a::check(
        entries.len() == 10152 && actual == entries.keys().cloned().collect(),
        "minibatch exact coverage",
    )?;
    for ((n, h), got) in entries.iter().zip(a::hashes(
        &entries.keys().map(|n| root.join(n)).collect::<Vec<_>>(),
    )?) {
        a::check(*h == got, &format!("minibatch changed {n}"))?;
    }
    let audit = a::load(&root.join("audit.json"))?;
    a::check(
        audit["audit_pass"] == true
            && audit["scientific_campaign_complete"] == true
            && audit["support_batching"] == false,
        "minibatch negative lineage",
    )?;
    Ok(
        json!({"archive":root,"root_seal":LATEST,"sealed_files_verified":10152,"scope":"diagnostic lineage, not improvement"}),
    )
}
pub fn run(parent: &Path, out: &Path, development: bool) -> Res<()> {
    let start = Instant::now();
    let l = Layout::registered(development);
    a::check(!out.exists(), "fresh direction output required")?;
    let absolute = out
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(out.file_name().ok_or("output name")?);
    a::check(
        !absolute.starts_with(parent.canonicalize().map_err(|e| e.to_string())?)
            && !absolute
                .starts_with(c::repo().join("output/ere_complete_minibatch_trial_20261001")),
        "output inside evidence",
    )?;
    fs::create_dir(out).map_err(|e| e.to_string())?;
    for d in ["cells", "estimates", "directions"] {
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
    let mut runner = f::Runner {
        out,
        plan,
        deadline: start + Duration::from_secs(if development { 900 } else { 3600 }),
        m: json!({"profile":PROFILE,"design":l.design(),"status":"validating","reason":null,"scheduled_cells":l.cells(),
            "schedule_sha256":a::digest(&out.join("schedule.json"))?,"recorded_cells":0,"attempted_steps":0,"advancing_steps":0,
            "unknown_partial_work_cells":0,"inflight_native_work_unknown":false,"grammar_compilations":0,"optimizer_generations":0,
            "direction_starts":0,"direction_estimates":0,"receipt_bytes":0,"training_admitted":false,"full_training_authorized":false}),
    };
    runner.save()?;
    let theta = contexts();
    let mut align = vec![[None, None]; l.blocks];
    let mut us = vec![[[None; 5]; 2]; l.blocks];
    let mut ns = us.clone();
    let mut counts = vec![[[0; 5]; 2]; l.blocks];
    let result = terminal(|| {
        a::check(
            a::load(&c::repo().join(EXTRA[0]))?[if development {
                "development"
            } else {
                "scientific"
            }] == l.design(),
            "registered direction config",
        )?;
        let (proof, pins, _) = c::prepare(parent)?;
        c::write(&out.join("parent_receipt.json"), &proof)?;
        c::write(&out.join("latest_receipt.json"), &latest()?)?;
        let extras: Vec<_> = f::EXTRA
            .iter()
            .chain(b::EXTRA.iter())
            .chain(EXTRA.iter())
            .copied()
            .collect();
        let snapshot = c::freeze_with(out, &pins, &extras)?;
        c::write(&out.join("snapshot.json"), &snapshot)?;
        runner.m["snapshot_sha256"] = json!(a::digest(&out.join("snapshot.json"))?);
        runner.save()?;
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
        c::write(&out.join("contexts.json"), &json!(theta))?;
        runner.m["grammar_compilations"] = json!(1);
        runner.m["status"] = json!("compiling");
        runner.save()?;
        let compile = Instant::now();
        let grammar = valid_grammar::compile()?;
        runner.m["grammar_compile_seconds"] = json!(compile.elapsed().as_secs_f64());
        c::write(&out.join("catalogue.json"), &grammar.catalogue())?;
        a::check(
            a::digest(&out.join("catalogue.json"))? == a::CATALOGUE_SHA,
            "catalogue identity",
        )?;
        c::recheck(&snapshot)?;
        runner.m["training_admitted"] = json!(true);
        runner.m["status"] = json!("estimation");
        runner.save()?;
        let mut forwards = vec![
            [
                [Vec::<f32>::new(), Vec::<f32>::new()],
                [Vec::<f32>::new(), Vec::<f32>::new()]
            ];
            l.blocks
        ];
        for r in 0..l.blocks {
            for ctx in 0..2 {
                c::recheck(&snapshot)?;
                for e in 0..2 {
                    let mut es = OpenAiEs::new(
                        1.,
                        0.05,
                        8,
                        ChaCha8Rng::seed_from_u64(l.seed + 1000000 + 10 * r as u64 + ctx as u64),
                    );
                    let eps = es.perturbations(183);
                    let begin = runner.m["recorded_cells"].as_u64().unwrap() as usize;
                    let ids: Vec<_> = runner.plan[begin..begin + 16 * l.k]
                        .iter()
                        .map(|q| q["id"].clone())
                        .collect();
                    let path = out.join("estimates").join(format!("r{r}-c{ctx}-e{e}.json"));
                    let mut rec = json!({"block":r,"context":ctx,"estimate":e,"theta_before":theta[ctx],"eps":eps,"evaluation_ids":ids,
                    "update_applied":false,"candidate_means":null,"optimizer_pair_scores":null,"theta_after":null});
                    c::write(&path, &rec)?;
                    runner.m["direction_starts"] =
                        json!(runner.m["direction_starts"].as_u64().unwrap() + 1);
                    runner.save()?;
                    let mut scores = vec![];
                    for noise in &eps {
                        for sign in [1., -1.] {
                            let candidate = es.candidate(&theta[ctx], noise, sign);
                            let mut batch = vec![];
                            for _ in 0..l.k {
                                let q = &runner.plan
                                    [runner.m["recorded_cells"].as_u64().unwrap() as usize];
                                let emission = grammar
                                    .emit(&candidate, q["emission_seed"].as_u64().unwrap())?;
                                batch.push(runner.evaluate(&emission, &candidate)?.0);
                            }
                            scores.push(batch);
                        }
                    }
                    let (means, pairs) = b::batch_pairs(&scores, l.k)?;
                    let mut forward = theta[ctx].clone();
                    es.update(&mut forward, &eps, &pairs);
                    rec["update_applied"] = json!(true);
                    rec["candidate_means"] = json!(means);
                    rec["optimizer_pair_scores"] =
                        json!(pairs.iter().map(|p| [p.plus, p.minus]).collect::<Vec<_>>());
                    rec["theta_after"] = json!(forward);
                    c::write(&path, &rec)?;
                    forwards[r][ctx][e] = forward;
                    runner.m["direction_estimates"] =
                        json!(runner.m["direction_estimates"].as_u64().unwrap() + 1);
                    runner.save()?;
                }
                let score = alignment(&theta[ctx], &forwards[r][ctx][0], &forwards[r][ctx][1]);
                align[r][ctx] = Some(score.clone());
                let params = [
                    theta[ctx].clone(),
                    forwards[r][ctx][0].clone(),
                    reverse(&theta[ctx], &forwards[r][ctx][0]),
                    forwards[r][ctx][1].clone(),
                    reverse(&theta[ctx], &forwards[r][ctx][1]),
                ];
                c::write(
                    &out.join("directions").join(format!("r{r}-c{ctx}.json")),
                    &json!({"block":r,"context":ctx,"alignment":score,"parameters":params,"selection":"both_fixed_estimates_no_validation_selection"}),
                )?;
                println!(
                    "direction estimates: block {}/{} context {} complete",
                    r + 1,
                    l.blocks,
                    ctx
                );
            }
        }
        runner.m["status"] = json!("validation");
        runner.save()?;
        for r in 0..l.blocks {
            c::recheck(&snapshot)?;
            for ctx in 0..2 {
                let rec = a::load(&out.join("directions").join(format!("r{r}-c{ctx}.json")))?;
                let params: Vec<Vec<f32>> =
                    serde_json::from_value(rec["parameters"].clone()).map_err(|e| e.to_string())?;
                let mut scores: [Vec<Option<f64>>; 5] = std::array::from_fn(|_| vec![]);
                let mut utilities: [Vec<f64>; 5] = std::array::from_fn(|_| vec![]);
                for _ in 0..l.draws {
                    for arm in 0..5 {
                        for _ in 0..4 {
                            let q =
                                &runner.plan[runner.m["recorded_cells"].as_u64().unwrap() as usize];
                            let emission =
                                grammar.emit(&params[arm], q["emission_seed"].as_u64().unwrap())?;
                            let (s, u) = runner.evaluate(&emission, &params[arm])?;
                            counts[r][ctx][arm] += usize::from(s.is_some());
                            scores[arm].push(s);
                            utilities[arm].push(u);
                        }
                    }
                }
                for arm in 0..5 {
                    us[r][ctx][arm] =
                        Some(utilities[arm].iter().sum::<f64>() / (l.draws * 4) as f64);
                    ns[r][ctx][arm] = scores[arm]
                        .iter()
                        .copied()
                        .collect::<Option<Vec<_>>>()
                        .map(|v| v.iter().sum::<f64>() / (l.draws * 4) as f64);
                }
            }
            println!(
                "direction validation: block {}/{} complete",
                r + 1,
                l.blocks
            );
        }
        c::recheck(&snapshot)?;
        c::prepare(parent)?;
        latest()?;
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
            "final direction resource ceiling",
        )?;
        runner.m["live_sources_rechecked"] = json!(true);
        runner.m["status"] = json!("complete");
        Ok(())
    });
    if let Err(reason) = &result {
        runner.m["status"] = json!(if runner.m["recorded_cells"] == 0 {
            "blocked_before_evaluation"
        } else {
            "incomplete"
        });
        runner.m["reason"] = json!(reason);
    }
    runner.m["elapsed_seconds"] = json!(start.elapsed().as_secs_f64());
    runner.save()?;
    let mut s = statistics(
        l,
        &align,
        &us,
        &ns,
        &counts,
        runner.m["status"] == "complete",
    );
    for k in ["attempted_steps", "advancing_steps"] {
        s[k] = runner.m[k].clone();
    }
    c::write(&out.join("summary.json"), &s)?;
    c::seal(out)?;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_panel_alignment_and_reverse() {
        let t = contexts();
        assert_eq!(t[1][0], 0.25);
        assert_eq!(t[1][1], -0.25);
        assert_eq!(alignment(&t[0], &t[0], &t[0])["score"], 0.);
        assert_eq!(alignment(&t[0], &t[0], &t[0])["degenerate"], true);
        let x = vec![1.; 183];
        let y = reverse(&t[0], &x);
        assert_eq!(alignment(&t[0], &x, &y)["score"], -1.);
        assert_eq!(alignment(&t[0], &x, &x)["score"], 1.);
        assert_eq!(reverse(&t[1], &t[1]), t[1]);
    }
    #[test]
    fn registered_schedule_and_disjoint_panels() {
        for dev in [false, true] {
            let l = Layout::registered(dev);
            let q = l.schedule();
            assert_eq!(q.len(), l.cells());
            assert_eq!(
                a::load(&c::repo().join(EXTRA[0])).unwrap()
                    [if dev { "development" } else { "scientific" }],
                l.design()
            );
            let mut panels = std::collections::BTreeSet::new();
            for r in 0..l.blocks {
                for ctx in 0..2 {
                    for e in 0..2 {
                        let seeds: std::collections::BTreeSet<_> = q
                            .iter()
                            .filter(|v| {
                                v["stage"] == "estimation"
                                    && v["block"] == r
                                    && v["context"] == ctx
                                    && v["estimate"] == e
                            })
                            .map(|v| v["emission_seed"].as_u64().unwrap())
                            .collect();
                        assert_eq!(seeds.len(), 8 * l.k);
                        for seed in seeds {
                            assert!(panels.insert(seed));
                        }
                    }
                }
            }
            let validation: std::collections::BTreeSet<_> = q
                .iter()
                .filter(|v| v["stage"] == "validation")
                .map(|v| v["emission_seed"].as_u64().unwrap())
                .collect();
            assert!(panels.is_disjoint(&validation));
        }
    }
    #[test]
    fn fixed_blocks_missing_and_development_boundary() {
        for dev in [false, true] {
            let l = Layout::registered(dev);
            let mut align = vec![[Some(json!({"score":1.})), Some(json!({"score":1.}))]; l.blocks];
            let u = vec![[[Some(1.), Some(1.2), Some(0.8), Some(1.2), Some(0.8)]; 2]; l.blocks];
            let counts = vec![[[l.draws * 4; 5]; 2]; l.blocks];
            let s = statistics(l, &align, &u, &u, &counts, true);
            assert_eq!(s["fixed_denominator"], l.blocks);
            assert_eq!(s["support_direction"], !dev);
            align[0][0] = None;
            let s = statistics(l, &align, &u, &u, &counts, false);
            assert!(s["usefulness"]["mean"].is_null());
            assert!(s["reliability"]["sign_p"].is_null());
            assert_eq!(s["support_direction"], false);
        }
    }
    #[test]
    fn wrong_parent_zero_work_audit() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = tmp.path().join("bad");
        fs::create_dir(&parent).unwrap();
        c::write(&parent.join("output_sha256.json"), &json!({})).unwrap();
        let out = tmp.path().join("refusal");
        assert!(run(&parent, &out, true).is_err());
        let m = a::load(&out.join("manifest.json")).unwrap();
        assert_eq!(m["attempted_steps"], 0);
        assert_eq!(m["direction_starts"], 0);
        assert_eq!(m["grammar_compilations"], 0);
        let result = std::process::Command::new("python3")
            .arg("-B")
            .arg(c::repo().join(EXTRA[2]))
            .arg(&out)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
