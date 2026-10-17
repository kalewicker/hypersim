//! Fixed-champion confirmation with a prospectively frozen, independent panel.
use super::{
    complete_admission::{self as a, Res},
    short_training::{sign_p, terminal},
};
use crate::optim::standard_normal;
use hg_sim_core::ere::{signal, valid_grammar};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
pub const PROFILE: &str = "E2-complete-fixed-champion-confirmation-v1";
pub const SEED: u64 = 0xE655000001;
pub const PARENT_SEAL: &str = "dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132";
const CHAMPION_SHA: &str = "14cf0b13540167b3a3b20082db71ddd9bbd761a5f5b0d81083dd6d719e24988f";
pub const ARMS: [&str; 3] = ["champion", "untrained", "random"];
pub const EXTRA: [&str; 6] = [
    "configs/ere_complete_champion_confirmation_v1.json",
    "docs/ere_complete_champion_confirmation_v1.md",
    "scripts/audit_ere_complete_confirmation.py",
    "scripts/test_audit_ere_complete_confirmation.py",
    "scripts/audit_ere_complete_admission.py",
    "scripts/audit_ere_complete_scaling.py",
];
const OUTPUT_CAP: u64 = 2 * 1024 * 1024 * 1024;
pub fn design() -> Value {
    json!({"profile":PROFILE,"master_seed":SEED,"draws":512,"physics_seeds":[SEED+100000,SEED+100001,SEED+100002,SEED+100003],
        "arms":ARMS,"horizon":100,"parameters":183,"work":50000,"results":50000,
        "wall_seconds":3600,"cell_wall_seconds":30,"output_cap_bytes":OUTPUT_CAP,
        "optimizer_generations":0,"maximum_native_steps":614400,"champion_selection":"sealed_final_generation_parameters"})
}
pub fn schedule() -> Vec<Value> {
    let mut cells = vec![];
    for i in 0..512 {
        for (arm, name) in ARMS.iter().enumerate() {
            for p in 0..4 {
                cells.push(json!({"id":format!("d{i}-{name}-p{p}"),"stage":"held_out","draw":i,"arm":arm,
            "emission_seed":SEED+2000+i,"random_parameter_seed":SEED+3000+i,"physics_seed":SEED+100000+p,"horizon":100}));
            }
        }
    }
    cells
}
pub(super) fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
pub(super) fn write(path: &Path, v: &Value) -> Res<()> {
    let pending = path.with_extension("pending.json");
    fs::write(&pending, serde_json::to_vec(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    fs::rename(pending, path).map_err(|e| e.to_string())
}
pub(super) fn seal(out: &Path) -> Res<()> {
    let mut names = a::files(out)?;
    names.remove("output_sha256.json");
    let files: Vec<_> = names.iter().map(|n| out.join(n)).collect();
    let entries: BTreeMap<_, _> = names.into_iter().zip(a::hashes(&files)?).collect();
    write(&out.join("output_sha256.json"), &json!(entries))
}
fn pinned(p: &str) -> bool {
    [
        "hg-core",
        "hg-engine",
        "hg-meta",
        "hg-rules",
        "hg-observe",
        "hg-sim-core",
    ]
    .iter()
    .any(|c| p.starts_with(&format!("crates/{c}/")))
        || [
            "Cargo.toml",
            "Cargo.lock",
            "crates/hg-train/Cargo.toml",
            "crates/hg-train/src/optim.rs",
        ]
        .contains(&p)
}
pub(super) fn prepare(parent: &Path) -> Res<(Value, BTreeMap<String, String>, Vec<f32>)> {
    a::check(
        a::digest(&parent.join("output_sha256.json"))? == PARENT_SEAL,
        "fixed training receipt identity",
    )?;
    let entries: BTreeMap<String, String> =
        serde_json::from_value(a::load(&parent.join("output_sha256.json"))?)
            .map_err(|e| e.to_string())?;
    let mut actual = a::files(parent)?;
    actual.remove("output_sha256.json");
    a::check(
        entries.len() == 1690 && actual == entries.keys().cloned().collect(),
        "training receipt exact coverage",
    )?;
    let files: Vec<_> = entries.keys().map(|n| parent.join(n)).collect();
    for ((n, h), v) in entries.iter().zip(a::hashes(&files)?) {
        a::check(*h == v, &format!("training receipt changed {n}"))?;
    }
    let m = a::load(&parent.join("manifest.json"))?;
    let audit = a::load(&parent.join("audit.json"))?;
    a::check(
        m["status"] == "complete"
            && m["training_admitted"] == true
            && audit["audit_pass"] == true
            && audit["training_complete"] == true,
        "training receipt did not complete audited training",
    )?;
    a::check(
        a::digest(&parent.join("champion.json"))? == CHAMPION_SHA,
        "fixed final champion identity",
    )?;
    let theta: Vec<f32> =
        serde_json::from_value(a::load(&parent.join("champion.json"))?["parameters"].clone())
            .map_err(|e| e.to_string())?;
    a::check(
        theta.len() == 183 && theta.iter().all(|x| x.is_finite()),
        "champion parameters",
    )?;
    let all: BTreeMap<String, String> =
        serde_json::from_value(m["snapshot"]["sources"].clone()).map_err(|e| e.to_string())?;
    let pins: BTreeMap<_, _> = all.into_iter().filter(|(p, _)| pinned(p)).collect();
    let mut coverage: BTreeSet<String> = [
        "Cargo.toml",
        "Cargo.lock",
        "crates/hg-train/Cargo.toml",
        "crates/hg-train/src/optim.rs",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    for c in [
        "hg-core",
        "hg-engine",
        "hg-meta",
        "hg-rules",
        "hg-observe",
        "hg-sim-core",
    ] {
        for p in a::files(&repo().join("crates").join(c))? {
            if Path::new(&p)
                .extension()
                .is_some_and(|e| e == "rs" || e == "toml")
            {
                coverage.insert(format!("crates/{c}/{p}"));
            }
        }
    }
    a::check(
        pins.len() == 230 && coverage == pins.keys().cloned().collect(),
        "native source coverage",
    )?;
    let files: Vec<_> = pins.keys().map(|n| repo().join(n)).collect();
    for ((n, h), v) in pins.iter().zip(a::hashes(&files)?) {
        a::check(*h == v, &format!("native drift {n}"))?;
    }
    Ok((
        json!({"parent_archive":parent.canonicalize().map_err(|e|e.to_string())?,"parent_seal":PARENT_SEAL,"sealed_files_verified":1690,
        "champion_sha256":CHAMPION_SHA,"native_sources_verified":230,"prior_improvement_supported":false,"prior_sign_p":0.109375}),
        pins,
        theta,
    ))
}
fn freeze(out: &Path, pins: &BTreeMap<String, String>) -> Res<Value> {
    freeze_with(out, pins, &[])
}
pub(super) fn freeze_with(
    out: &Path,
    pins: &BTreeMap<String, String>,
    extras: &[&str],
) -> Res<Value> {
    let mut names: BTreeSet<_> = pins.keys().cloned().collect();
    for n in a::files(&repo().join("crates/hg-train"))? {
        if Path::new(&n)
            .extension()
            .is_some_and(|e| e == "rs" || e == "toml")
        {
            names.insert(format!("crates/hg-train/{n}"));
        }
    }
    names.extend(EXTRA.into_iter().map(String::from));
    names.extend(extras.iter().map(|s| s.to_string()));
    let mut files = vec![];
    for n in &names {
        let dst = out.join("source_snapshots").join(n);
        fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::copy(repo().join(n), &dst).map_err(|e| e.to_string())?;
        files.push(dst);
    }
    let sources: BTreeMap<_, _> = names.into_iter().zip(a::hashes(&files)?).collect();
    fs::copy(
        std::env::current_exe().map_err(|e| e.to_string())?,
        out.join("executed_binary"),
    )
    .map_err(|e| e.to_string())?;
    Ok(
        json!({"sources":sources,"binary_sha256":a::digest(&out.join("executed_binary"))?,"frozen_before_measurement":true}),
    )
}
pub(super) fn recheck(snapshot: &Value) -> Res<()> {
    let sources: BTreeMap<String, String> =
        serde_json::from_value(snapshot["sources"].clone()).map_err(|e| e.to_string())?;
    let files: Vec<_> = sources.keys().map(|n| repo().join(n)).collect();
    for ((n, h), v) in sources.iter().zip(a::hashes(&files)?) {
        a::check(*h == v, &format!("frozen source changed {n}"))?;
    }
    a::check(
        a::digest(&std::env::current_exe().map_err(|e| e.to_string())?)?
            == snapshot["binary_sha256"]
                .as_str()
                .ok_or("binary identity")?,
        "binary changed",
    )
}
pub fn statistics(scores: &[[[Option<f64>; 4]; 3]]) -> Value {
    let means: Vec<[Option<f64>; 3]> = scores
        .iter()
        .map(|d| {
            std::array::from_fn(|a| {
                d[a].iter()
                    .copied()
                    .collect::<Option<Vec<_>>>()
                    .map(|s| s.iter().sum::<f64>() / 4.0)
            })
        })
        .collect();
    let all = means.iter().all(|d| d.iter().all(Option::is_some));
    let differences: Vec<_> = means
        .iter()
        .filter_map(|d| d[0].zip(d[1]).map(|(a, b)| a - b))
        .collect();
    let random: Vec<_> = means
        .iter()
        .filter_map(|d| d[0].zip(d[2]).map(|(a, b)| a - b))
        .collect();
    let wins = differences.iter().filter(|&&d| d > 0.0).count();
    let losses = differences.iter().filter(|&&d| d < 0.0).count();
    let primary = if all {
        Some(differences.iter().sum::<f64>() / 512.0)
    } else {
        None
    };
    let secondary = if all {
        Some(random.iter().sum::<f64>() / 512.0)
    } else {
        None
    };
    let p = if all {
        Some(sign_p(wins, losses))
    } else {
        None
    };
    json!({"fixed_denominator":512,"all_valid":all,"complete_primary_pairs":differences.len(),"complete_secondary_pairs":random.len(),
        "arm_means":ARMS.iter().enumerate().map(|(a,name)|(*name,if all{Some(means.iter().map(|d|d[a].unwrap()).sum::<f64>()/512.0)}else{None})).collect::<BTreeMap<_,_>>(),
        "champion_minus_untrained":primary,"champion_minus_random":secondary,"wins":wins,"losses":losses,"ties":differences.len()-wins-losses,
        "one_sided_paired_sign_p":p,"improvement_supported":primary.zip(p).is_some_and(|(g,p)|g>0.0 && p<0.05),"draw_means":means})
}
pub fn run(parent: &Path, out: &Path) -> Res<()> {
    a::check(!out.exists(), "fresh confirmation output required")?;
    let absolute = out
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(out.file_name().ok_or("output name")?);
    a::check(
        !absolute.starts_with(parent.canonicalize().map_err(|e| e.to_string())?),
        "confirmation output inside training archive",
    )?;
    fs::create_dir(out).map_err(|e| e.to_string())?;
    fs::create_dir(out.join("cells")).map_err(|e| e.to_string())?;
    let plan = schedule();
    write(&out.join("schedule.json"), &json!(plan))?;
    for q in &plan {
        write(
            &out.join("cells")
                .join(format!("{}.json", q["id"].as_str().unwrap())),
            &json!({"schedule":q,"status":"NotRun","reason":"not_started","evaluation":null}),
        )?;
    }
    let start = Instant::now();
    let deadline = start + Duration::from_secs(3600);
    let mut scores = vec![[[None; 4]; 3]; 512];
    let mut m = json!({"profile":PROFILE,"design":design(),"status":"validating","reason":null,"scheduled_cells":6144,
        "schedule_sha256":a::digest(&out.join("schedule.json"))?,"recorded_cells":0,"attempted_steps":0,"advancing_steps":0,
        "unknown_partial_work_cells":0,"inflight_native_work_unknown":false,"grammar_compilations":0,"optimizer_generations":0,"receipt_bytes":0});
    write(&out.join("manifest.json"), &m)?;
    let result = terminal(|| {
        a::check(
            a::load(&repo().join("configs/ere_complete_champion_confirmation_v1.json"))?
                == design(),
            "confirmation config differs from registration",
        )?;
        let (proof, pins, theta) = prepare(parent)?;
        write(&out.join("parent_receipt.json"), &proof)?;
        fs::copy(parent.join("champion.json"), out.join("champion.json"))
            .map_err(|e| e.to_string())?;
        let snapshot = freeze(out, &pins)?;
        write(&out.join("snapshot.json"), &snapshot)?;
        m["snapshot_sha256"] = json!(a::digest(&out.join("snapshot.json"))?);
        // Read-only audit of the original sealed campaign, including its prefix lineage.
        let checked = std::process::Command::new("python3")
            .arg("-B")
            .arg(parent.join("source_snapshots/scripts/audit_ere_complete_short_training.py"))
            .arg(parent)
            .output()
            .map_err(|e| e.to_string())?;
        a::check(
            checked.status.success(),
            &format!(
                "parent frozen audit failed: {}",
                String::from_utf8_lossy(&checked.stderr)
            ),
        )?;
        let audit: Value = serde_json::from_slice(&checked.stdout).map_err(|e| e.to_string())?;
        a::check(
            audit["audit_pass"] == true && audit["training_complete"] == true,
            "parent audit refusal",
        )?;
        write(&out.join("parent_audit.json"), &audit)?;
        m["status"] = json!("compiling");
        m["grammar_compilations"] = json!(1);
        write(&out.join("manifest.json"), &m)?;
        let compile = Instant::now();
        let grammar = valid_grammar::compile()?;
        m["grammar_compile_seconds"] = json!(compile.elapsed().as_secs_f64());
        fs::write(
            out.join("catalogue.json"),
            serde_json::to_vec(&grammar.catalogue()).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        a::check(
            a::digest(&out.join("catalogue.json"))? == a::CATALOGUE_SHA,
            "catalogue identity",
        )?;
        recheck(&snapshot)?;
        m["status"] = json!("evaluating");
        write(&out.join("manifest.json"), &m)?;
        for i in 0..512 {
            if i % 64 == 0 {
                recheck(&snapshot)?;
            }
            let mut rng = ChaCha8Rng::seed_from_u64(SEED + 3000 + i as u64);
            let random: Vec<f32> = (0..183).map(|_| standard_normal(&mut rng)).collect();
            let zero = vec![0.0; 183];
            for (arm, t) in [&theta, &zero, &random].into_iter().enumerate() {
                let e = grammar.emit(t, SEED + 2000 + i as u64)?;
                for p in 0..4 {
                    a::check(Instant::now() < deadline, "confirmation wall ceiling")?;
                    let q = &plan[i * 12 + arm * 4 + p];
                    let id = q["id"].as_str().unwrap();
                    m["current_cell"] = q.clone();
                    m["inflight_native_work_unknown"] = json!(true);
                    write(&out.join("manifest.json"), &m)?;
                    let cell_start = Instant::now();
                    let evaluated = terminal(|| {
                        signal::evaluate_with_horizon(
                            &e,
                            q["physics_seed"].as_u64().unwrap(),
                            deadline.min(cell_start + Duration::from_secs(30)),
                            50000,
                            50000,
                            100,
                        )
                    });
                    let mut row = json!({"schedule":q,"emission":e.record,"parameters":t,"elapsed_seconds":cell_start.elapsed().as_secs_f64()});
                    let error;
                    match evaluated {
                        Ok(mut v) => {
                            v["runtime_match_budget"] =
                                json!({"policy":"Pooled","work":50000,"results":50000});
                            row["status"] = json!(if v["score"].is_number() {
                                "Complete"
                            } else {
                                "Null"
                            });
                            row["evaluation"] = v;
                            error = None;
                        }
                        Err(reason) => {
                            row["status"] = json!("Incomplete");
                            row["reason"] = json!(reason);
                            row["evaluation"] = Value::Null;
                            row["partial_steps_unknown"] = json!(true);
                            error = Some(reason);
                        }
                    }
                    let size = serde_json::to_vec(&row).map_err(|e| e.to_string())?.len() as u64;
                    a::check(
                        m["receipt_bytes"].as_u64().unwrap() + size
                            < OUTPUT_CAP - 128 * 1024 * 1024,
                        "confirmation receipt output ceiling",
                    )?;
                    write(&out.join("cells").join(format!("{id}.json")), &row)?;
                    if let Some(v) = row["evaluation"].as_object() {
                        for field in ["attempted_steps", "advancing_steps"] {
                            m[field] = json!(
                                m[field].as_u64().unwrap()
                                    + v[field].as_u64().ok_or("native count")?
                            );
                        }
                    } else {
                        m["unknown_partial_work_cells"] =
                            json!(m["unknown_partial_work_cells"].as_u64().unwrap() + 1);
                    }
                    scores[i][arm][p] = row["evaluation"]["score"].as_f64();
                    m["recorded_cells"] = json!(i * 12 + arm * 4 + p + 1);
                    m["inflight_native_work_unknown"] = json!(false);
                    m["receipt_bytes"] = json!(m["receipt_bytes"].as_u64().unwrap() + size);
                    write(&out.join("manifest.json"), &m)?;
                    if let Some(reason) = error {
                        return Err(reason);
                    }
                }
            }
            if (i + 1) % 64 == 0 {
                println!("confirmation: {}/512 draws completed", i + 1);
            }
        }
        recheck(&snapshot)?;
        prepare(parent)?;
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
            total < OUTPUT_CAP - 16 * 1024 * 1024 && Instant::now() < deadline,
            "final confirmation resource ceiling",
        )?;
        m["status"] = json!("complete");
        m["live_sources_rechecked"] = json!(true);
        Ok(())
    });
    if let Err(reason) = &result {
        m["status"] = json!(if m["recorded_cells"] == 0 {
            "blocked_before_evaluation"
        } else {
            "incomplete"
        });
        m["reason"] = json!(reason);
    }
    m["elapsed_seconds"] = json!(start.elapsed().as_secs_f64());
    write(&out.join("manifest.json"), &m)?;
    let mut summary = statistics(&scores);
    summary["improvement_supported"] =
        json!(summary["improvement_supported"] == true && m["status"] == "complete");
    summary["profile"] = json!(PROFILE);
    summary["measurement_complete"] = json!(m["status"] == "complete");
    summary["attempted_steps"] = m["attempted_steps"].clone();
    summary["advancing_steps"] = m["advancing_steps"].clone();
    write(&out.join("summary.json"), &summary)?;
    seal(out)?;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_config_schedule_and_denominators() {
        assert_eq!(schedule().len(), 6144);
        assert_eq!(
            schedule()
                .iter()
                .map(|q| q["horizon"].as_u64().unwrap())
                .sum::<u64>(),
            614400
        );
        assert_eq!(
            a::load(&repo().join("configs/ere_complete_champion_confirmation_v1.json")).unwrap(),
            design()
        );
        let mut scores = vec![[[Some(0.0); 4]; 3]; 512];
        for s in &mut scores {
            s[0] = [Some(1.0); 4];
        }
        let v = statistics(&scores);
        assert_eq!(v["champion_minus_untrained"], 1.0);
        assert_eq!(v["improvement_supported"], true);
        scores[0][0][0] = None;
        let v = statistics(&scores);
        assert!(v["champion_minus_untrained"].is_null());
        assert_eq!(v["fixed_denominator"], 512);
        assert_eq!(v["complete_primary_pairs"], 511);
        assert_eq!(v["improvement_supported"], false);
    }
    #[test]
    fn real_parent_and_native_identity_are_readonly() {
        let p = repo().join("output/ere_complete_short_training_continuation_20260930");
        let (proof, pins, t) = prepare(&p).unwrap();
        assert_eq!(proof["native_sources_verified"], 230);
        assert_eq!(pins.len(), 230);
        assert_eq!(t.len(), 183);
        assert!(prepare(&repo().join("output/nonexistent-confirmation-parent")).is_err());
        let tmp = tempfile::tempdir().unwrap();
        assert!(run(&p, &p).is_err());
        assert!(!tmp.path().join("missing").exists());
    }
}
