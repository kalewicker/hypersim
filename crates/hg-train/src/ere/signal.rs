//! Fixed complete-profile diagnostic, deliberately separate from training admission.
use crate::optim::{OpenAiEs, PairResult};
use hg_sim_core::ere::signal::{self as backend, PARAMS};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};
type Res<T> = Result<T, String>;
pub const SEED: u64 = 0xE452390001;
pub const PROFILE: &str = "E2-complete-optimizer-signal-v1";
const PHYSICS: [u64; 4] = [101, 211, 307, 401];
fn write(path: &Path, value: &Value) -> Res<()> {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).map_err(|e| e.to_string())
}
fn load(path: &Path) -> Res<Value> {
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn output_bytes(dir: &Path) -> Res<u64> {
    let mut bytes = 0;
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let p = entry.map_err(|e| e.to_string())?.path();
        bytes += if p.is_dir() {
            output_bytes(&p)?
        } else {
            fs::metadata(&p).map_err(|e| e.to_string())?.len()
        };
    }
    Ok(bytes)
}
fn digest(path: &Path) -> Res<String> {
    let r = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if !r.status.success() {
        return Err(format!("hash failed: {}", path.display()));
    }
    Ok(String::from_utf8(r.stdout)
        .map_err(|e| e.to_string())?
        .split_whitespace()
        .next()
        .unwrap()
        .into())
}
fn prerequisite(gate: &Path) -> Res<Value> {
    if digest(&gate.join("output_sha256.json"))?
        != "fa3725f7cf5a03b207635d4c121d47280d38b4a05ae018ec3606e00086a4b60a"
    {
        return Err("unrecognized G-R1 closure seal".into());
    }
    let seal = load(&gate.join("output_sha256.json"))?;
    for name in [
        "manifest.json",
        "classification_summary.json",
        "run_status.json",
        "audit_full.json",
    ] {
        if seal[name].as_str() != Some(&digest(&gate.join(name))?) {
            return Err(format!("G-R1 seal mismatch {name}"));
        }
    }
    let m = load(&gate.join("manifest.json"))?;
    let s = load(&gate.join("run_status.json"))?;
    let a = load(&gate.join("audit_full.json"))?;
    if m["profile"] != "E2-complete-coherent-closure-v1"
        || m["matcher_work_model"] != "repeated-self-pair-feasibility-v1"
        || s["status"] != "complete"
        || s["gate_pass"] != true
        || s["measurement_complete"] != true
        || a["audit_pass"] != true
    {
        return Err("closure prerequisite not accepted".into());
    }
    // This diagnostic cannot silently switch the validated native physics or codec.
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut checked = 0;
    for (name, hash) in seal.as_object().unwrap() {
        if let Some(rel) = name.strip_prefix("source_snapshots/") {
            if [
                "crates/hg-engine/",
                "crates/hg-core/",
                "crates/hg-rules/",
                "crates/hg-meta/",
            ]
            .iter()
            .any(|p| rel.starts_with(p))
                && Path::new(rel)
                    .extension()
                    .is_some_and(|e| e == "rs" || e == "toml")
                || [
                    "crates/hg-sim-core/src/ere/experiments.rs",
                    "crates/hg-sim-core/src/ere/semantic.rs",
                ]
                .contains(&rel)
            {
                if digest(&repo.join(rel))?.as_str() != hash.as_str().unwrap() {
                    return Err(format!("closure runtime drift: {rel}"));
                }
                checked += 1;
            }
        }
    }
    if checked == 0 {
        return Err("no frozen native source comparisons".into());
    }
    Ok(
        json!({"archive":gate.canonicalize().map_err(|e|e.to_string())?,"seal_sha256":digest(&gate.join("output_sha256.json"))?,"runtime_sources_checked":checked,"gate_pass":true,"scope":"diagnostic prerequisite only"}),
    )
}
fn snapshot(repo: &Path, out: &Path) -> Res<Value> {
    fn files(dir: &Path, list: &mut Vec<std::path::PathBuf>) -> Res<()> {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let p = entry.map_err(|e| e.to_string())?.path();
            if p.is_dir() {
                files(&p, list)?;
            } else if p.extension().is_some_and(|e| e == "rs" || e == "toml") {
                list.push(p);
            }
        }
        Ok(())
    }
    let mut paths = vec![
        repo.join("Cargo.toml"),
        repo.join("Cargo.lock"),
        repo.join("docs/ere_complete_optimizer_signal_v1.md"),
        repo.join("scripts/audit_ere_complete_optimizer_signal.py"),
    ];
    for c in [
        "hg-core",
        "hg-engine",
        "hg-rules",
        "hg-meta",
        "hg-sim-core",
        "hg-train",
        "hg-observe",
    ] {
        files(&repo.join("crates").join(c), &mut paths)?;
    }
    paths.sort();
    let mut hashes = BTreeMap::new();
    for p in paths {
        let rel = p.strip_prefix(repo).map_err(|e| e.to_string())?;
        let dst = out.join("source_snapshots").join(rel);
        fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::copy(&p, &dst).map_err(|e| e.to_string())?;
        hashes.insert(rel.to_string_lossy().to_string(), digest(&dst)?);
    }
    let exe = out.join("executed_binary");
    fs::copy(std::env::current_exe().map_err(|e| e.to_string())?, &exe)
        .map_err(|e| e.to_string())?;
    Ok(json!({"sources":hashes,"binary_sha256":digest(&exe)?}))
}
pub fn distribution(theta: &[f32]) -> Res<Vec<Value>> {
    (0..64).map(|i| backend::emit(theta,SEED+1000+i).map(|e|json!({"seed":SEED+1000+i,"signature":e.record["signature"],"status":e.record["status"]}))).collect()
}
pub fn tv(a: &[Value], b: &[Value]) -> f64 {
    let mut counts = BTreeMap::<String, i32>::new();
    for (panel, sign) in [(a, 1), (b, -1)] {
        for r in panel {
            *counts.entry(r["signature"].to_string()).or_default() += sign;
        }
    }
    counts.values().map(|n| n.abs() as f64).sum::<f64>() / (2.0 * a.len() as f64)
}
pub fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}
pub fn sd(xs: &[f64]) -> f64 {
    let m = mean(xs);
    (xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / xs.len() as f64).sqrt()
}
pub fn pair(plus: &[Option<f64>], minus: &[Option<f64>]) -> PairResult {
    let full = |x: &[Option<f64>]| {
        x.iter()
            .copied()
            .collect::<Option<Vec<_>>>()
            .map(|x| mean(&x) as f32)
    };
    match (full(plus), full(minus)) {
        (Some(p), Some(m)) => PairResult {
            plus: Some(p),
            minus: Some(m),
        },
        _ => PairResult {
            plus: None,
            minus: None,
        },
    }
}
pub fn run(out: &Path, gate: &Path) -> Res<()> {
    let prereq = prerequisite(gate)?;
    fs::create_dir(out).map_err(|e| format!("fresh output directory required: {e}"))?;
    let result = (|| {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let sources = snapshot(&repo, out)?;
        let mut manifest = json!({"profile":PROFILE,"status":"running","training_admitted":false,"development_only":true,"prerequisite":prereq,"snapshot":sources,"master_seed":SEED,"scheduled_rollouts":120,"horizon":100,"step_ceiling":12000,"wall_ceiling_seconds":900,"output_ceiling_bytes":536870912,"matcher_work_model":"repeated-self-pair-feasibility-v1","frozen_before_measurement":true});
        write(&out.join("manifest.json"), &manifest)?;
        let start = Instant::now();
        let deadline = start + Duration::from_secs(900);
        let mut stream = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out.join("rollouts.jsonl"))
            .map_err(|e| e.to_string())?;
        let mut rows = vec![];
        let mut rollout =
            |id: String, emission: &backend::Emission, physics: u64| -> Res<Option<f64>> {
                if Instant::now() >= deadline || output_bytes(out)? >= 536870912 {
                    return Err("campaign resource ceiling".into());
                }
                let evaluation = backend::evaluate(emission, physics, deadline)?;
                let score = evaluation["score"].as_f64();
                let row = json!({"id":id,"emission":emission.record,"evaluation":evaluation});
                let wire = serde_json::to_vec(&row).unwrap();
                if output_bytes(out)? + wire.len() as u64 + 1 > 536870912 {
                    return Err("campaign output ceiling".into());
                }
                stream
                    .write_all(&wire)
                    .and_then(|_| stream.write_all(b"\n"))
                    .and_then(|_| stream.flush())
                    .map_err(|e| e.to_string())?;
                rows.push(row);
                Ok(score)
            };
        let mut theta = vec![0.0; PARAMS];
        let initial_panel = distribution(&theta)?;
        write(
            &out.join("initial_distribution.json"),
            &json!(initial_panel),
        )?;
        let mut baselines = vec![];
        for i in 0..8 {
            let e = backend::emit(&theta, SEED + 2000 + i)?;
            let mut scores = vec![];
            for physics in PHYSICS {
                scores.push(rollout(format!("baseline_{i}_{physics}"), &e, physics)?);
            }
            baselines.push(scores);
        }
        let mut controls = vec![];
        let forced = backend::forced(true, SEED + 3000)?;
        for active in [true, false] {
            let e = backend::forced(active, SEED + 3000)?;
            let mut scores = vec![];
            for physics in PHYSICS {
                scores.push(rollout(format!("control_{active}_{physics}"), &e, physics)?);
            }
            controls.push(scores);
        }
        let mut es = OpenAiEs::new(1.0, 0.05, 4, ChaCha8Rng::seed_from_u64(SEED));
        let mut generations = vec![];
        let mut valid_pairs = 0;
        let mut max_tv = 0.0f64;
        for g in 0..5 {
            let before = theta.clone();
            let eps = es.perturbations(PARAMS);
            let mut pairs = vec![];
            for (p, noise) in eps.iter().enumerate() {
                let seed = SEED + 4000 + 4 * g + p as u64;
                let mut arms = vec![];
                for sign in [1.0, -1.0] {
                    let candidate = es.candidate(&theta, noise, sign);
                    let e = backend::emit(&candidate, seed)?;
                    let mut scores = vec![];
                    for physics in PHYSICS[..2].iter().copied() {
                        scores.push(rollout(
                            format!("es_{g}_{p}_{sign}_{physics}"),
                            &e,
                            physics,
                        )?);
                    }
                    arms.push(scores);
                }
                let r = pair(&arms[0], &arms[1]);
                valid_pairs += usize::from(r.plus.is_some());
                pairs.push(r);
            }
            es.update(&mut theta, &eps, &pairs);
            let panel = distribution(&theta)?;
            let distance = tv(&initial_panel, &panel);
            max_tv = max_tv.max(distance);
            let record = json!({"generation":g,"theta_before":before,"eps":eps,"pair_scores":pairs.iter().map(|r|json!([r.plus,r.minus])).collect::<Vec<_>>(),"theta_after":theta,"distribution":panel,"tv_from_initial":distance});
            write(&out.join(format!("generation_{g}.json")), &record)?;
            generations.push(record);
            println!(
                "generation {}: {}/20 valid pairs so far, TV {:.6}",
                g + 1,
                valid_pairs,
                distance
            );
        }
        drop(rollout);
        let baseline = baselines
            .iter()
            .flatten()
            .copied()
            .collect::<Option<Vec<_>>>();
        let control = controls
            .iter()
            .flatten()
            .copied()
            .collect::<Option<Vec<_>>>();
        let baseline_mean = baseline.as_ref().map(|x| mean(x));
        let baseline_sd = baseline.as_ref().map(|x| sd(x));
        let gap = control.as_ref().map(|x| mean(&x[..4]) - mean(&x[4..]));
        let within = baseline
            .as_ref()
            .map(|x| (x.chunks(4).map(|v| sd(v).powi(2)).sum::<f64>() / 8.0).sqrt());
        let between = baseline
            .as_ref()
            .map(|x| sd(&x.chunks(4).map(mean).collect::<Vec<_>>()));
        let p1 = !initial_panel
            .iter()
            .any(|r| r["signature"] == forced.record["signature"]);
        let p2 = max_tv >= 0.05 && valid_pairs == 20;
        let p3 = gap
            .zip(baseline_sd)
            .is_some_and(|(g, s)| g > 2.0 * s && g > 1e-12);
        let p4 = baseline_mean.is_some_and(|m| m > 0.02 && m < 0.98);
        let summary = json!({"profile":PROFILE,"measurement_complete":rows.len()==120,"rollouts":rows.len(),"valid_rollouts":rows.iter().filter(|r|r["evaluation"]["score"].is_number()).count(),"attempted_steps":rows.iter().map(|r|r["evaluation"]["attempted_steps"].as_u64().unwrap()).sum::<u64>(),"advancing_steps":rows.iter().map(|r|r["evaluation"]["advancing_steps"].as_u64().unwrap()).sum::<u64>(),"baseline_mean":baseline_mean,"baseline_population_sd":baseline_sd,"within_genotype_physics_sd":within,"between_genotype_sd":between,"forced_gap":gap,"valid_pairs":valid_pairs,"max_distribution_tv":max_tv,"checks":{"P1":p1,"P2":p2,"P3":p3,"P4":p4},"precheck_pass":p1&&p2&&p3&&p4,"training_admitted":false,"elapsed_seconds":start.elapsed().as_secs_f64()});
        write(&out.join("summary.json"), &summary)?;
        manifest["status"] = json!("complete");
        manifest["measurement_complete"] = json!(true);
        write(&out.join("manifest.json"), &manifest)?;
        println!("{}", serde_json::to_string_pretty(&summary).unwrap());
        Ok(())
    })();
    if let Err(e) = &result {
        write(
            &out.join("run_failure.json"),
            &json!({"status":"incomplete","error":e,"training_admitted":false}),
        )?;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_invalid_arm_withholds_both() {
        let r = pair(&[Some(0.5), None], &[Some(1.0), Some(1.0)]);
        assert!(r.plus.is_none() && r.minus.is_none());
    }
    #[test]
    fn tv_counts_rejected_emissions_in_denominator() {
        assert_eq!(
            tv(
                &[json!({"signature":null}), json!({"signature":"x"})],
                &[json!({"signature":"x"}), json!({"signature":"x"})]
            ),
            0.5
        );
    }
    #[test]
    fn missing_gate_fails_before_creating_output() {
        let d = tempfile::tempdir().unwrap();
        let out = d.path().join("out");
        assert!(run(&out, &d.path().join("missing")).is_err());
        assert!(!out.exists());
    }
}
