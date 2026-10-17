use hg_sim_core::evolving_minds::{
    resume_admission, resume_choice, resume_d0, resume_family, resume_lifecycle, resume_mutation,
    resume_stream_segment, resume_to, write_admission_panel, write_arbitration_panel,
    write_choice_panel, write_d0_panel, write_descendant_panel, write_development_panel,
    write_family_panel, write_funded_admission_panel, write_funded_stream_segment,
    write_funding_panel, write_handoff_panel, write_interventions, write_lifecycle_panel,
    write_mutation_panel, write_neighborhood_panel, write_retained_panel,
    write_retained_stream_segment, write_smoke, write_stream_panel, Config, FundedAdmissionConfig,
    StreamFault, TransferGate0Config, write_transfer_gate0, write_transfer_gate1, transfer_gate1_registration, transfer_gate1_evaluation_registration,
};
use std::path::Path;
use hg_sim_core::evolving_minds::{mutation_stress_registration,mutation_stress_probe_registration,write_mutation_stress_panel};
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result=match args.as_slice(){
        [cmd] if cmd=="mutation-stress-register"=>mutation_stress_registration().map(|x|println!("{}",serde_json::to_string_pretty(&x).unwrap())),
        [cmd] if cmd=="mutation-stress-probe-register"=>mutation_stress_probe_registration().map(|x|println!("{}",serde_json::to_string_pretty(&x).unwrap())),
        [cmd,out] if cmd=="mutation-stress-development"=>write_mutation_stress_panel(Path::new(out),false),
        [cmd,out] if cmd=="mutation-stress-probe"=>write_mutation_stress_panel(Path::new(out),true),
        [cmd] if cmd=="transfer-gate1-register"=>transfer_gate1_registration().map(|x| println!("{}",serde_json::to_string_pretty(&x).unwrap())),
        [cmd] if cmd=="transfer-gate1-evaluation-register"=>transfer_gate1_evaluation_registration().map(|x| println!("{}",serde_json::to_string_pretty(&x).unwrap())),
        [cmd,prior,out] if cmd=="transfer-gate1-development"=>write_transfer_gate1(Path::new(out),Path::new(prior)),
        [cmd] if cmd=="transfer-register"=>TransferGate0Config::default().registration().map(|x| println!("{}",serde_json::to_string_pretty(&x).unwrap())),
        [cmd,out] if cmd=="transfer-development"=>write_transfer_gate0(Path::new(out)),
        [cmd,out] if cmd=="descendant-development"=>write_descendant_panel(Path::new(out)),
        [cmd,source,out] if cmd=="descendant-resume"=>resume_stream_segment(Path::new(source),Path::new(out)).map(|_|()),
        [cmd,out] if cmd=="handoff-development"=>write_handoff_panel(Path::new(out)),
        [cmd,source,out] if cmd=="handoff-resume"=>resume_stream_segment(Path::new(source),Path::new(out)).map(|_|()),
        [cmd,out] if cmd=="matched-development"=>write_development_panel(Path::new(out)),
        [cmd,out] if cmd=="retained"=>write_retained_panel(Path::new(out)),
        [cmd,out,flag,config] if cmd=="retained-run" && flag=="--config"=>std::fs::read_to_string(config).map_err(|e|e.to_string()).and_then(|x|serde_json::from_str::<FundedAdmissionConfig>(&x).map_err(|e|e.to_string())).and_then(|c|write_retained_stream_segment(Path::new(out),c,StreamFault::Normal).map(|_|())),
        [cmd,source,out] if cmd=="retained-resume"=>resume_stream_segment(Path::new(source),Path::new(out)).map(|_|()),
        [cmd,out] if cmd=="funded-admission"=>write_funded_admission_panel(Path::new(out)),
        [cmd,out,flag,config] if cmd=="funded-run" && flag=="--config"=>std::fs::read_to_string(config).map_err(|e|e.to_string()).and_then(|x|serde_json::from_str::<FundedAdmissionConfig>(&x).map_err(|e|e.to_string())).and_then(|c|write_funded_stream_segment(Path::new(out),c,StreamFault::Normal).map(|_|())),
        [cmd,source,out] if cmd=="funded-resume"=>resume_stream_segment(Path::new(source),Path::new(out)).map(|_|()),
        [cmd,out] if cmd=="funding"=>write_funding_panel(Path::new(out)),
        [cmd,out] if cmd=="neighborhood"=>write_neighborhood_panel(Path::new(out)),
        [cmd,out] if cmd=="stream"=>write_stream_panel(Path::new(out)),
        [cmd,source,out] if cmd=="stream-resume"=>resume_stream_segment(Path::new(source),Path::new(out)).map(|_|()),
        [cmd,out] if cmd=="admission"=>write_admission_panel(Path::new(out)),
        [cmd,cp,out] if cmd=="admission-resume"=>resume_admission(Path::new(out),Path::new(cp)),
        [cmd,out] if cmd=="lifecycle"=>write_lifecycle_panel(Path::new(out)),
        [cmd,cp,out] if cmd=="lifecycle-resume"=>resume_lifecycle(Path::new(out),Path::new(cp)),
        [cmd,out] if cmd=="arbitration"=>write_arbitration_panel(Path::new(out)),
        [cmd,out] if cmd=="mutation"=>write_mutation_panel(Path::new(out)),
        [cmd,checkpoint,out] if cmd=="mutation-resume"=>resume_mutation(Path::new(out),Path::new(checkpoint)),
        [cmd,out] if cmd=="family"=>write_family_panel(Path::new(out)),
        [cmd,checkpoint,out] if cmd=="family-resume"=>resume_family(Path::new(out),Path::new(checkpoint)),
        [cmd,out] if cmd=="d0"=>write_d0_panel(Path::new(out)),
        [cmd,checkpoint,out] if cmd=="d0-resume"=>resume_d0(Path::new(out),Path::new(checkpoint)),
        [cmd,out] if cmd=="choice"=>write_choice_panel(Path::new(out)),
        [cmd,checkpoint,out] if cmd=="choice-resume"=>resume_choice(Path::new(out),Path::new(checkpoint)),
        [cmd,out] if cmd=="interventions"=>write_interventions(Path::new(out)),
        [cmd,out] if cmd=="smoke"=>write_smoke(Path::new(out),Config::default()),
        [cmd,out,flag,config] if cmd=="smoke" && flag=="--config"=>
            std::fs::read_to_string(config).map_err(|e|e.to_string())
                .and_then(|s|toml::from_str::<Config>(&s).map_err(|e|e.to_string()))
                .and_then(|c|write_smoke(Path::new(out),c)),
        [cmd,checkpoint,out] if cmd=="resume"=>resume_to(Path::new(out),Path::new(checkpoint)),
        _=>Err("usage: evolving-minds transfer-register | transfer-development NEW_DIRECTORY | descendant-development NEW_DIRECTORY | descendant-resume SOURCE_SEGMENT NEW_DIRECTORY | handoff-development NEW_DIRECTORY | handoff-resume SOURCE_SEGMENT NEW_DIRECTORY | matched-development NEW_DIRECTORY | retained NEW_DIRECTORY | retained-run NEW_DIRECTORY --config CONFIG_JSON | retained-resume SOURCE_SEGMENT NEW_DIRECTORY | funded-admission NEW_DIRECTORY | funded-run NEW_DIRECTORY --config CONFIG_JSON | funded-resume SOURCE_SEGMENT NEW_DIRECTORY | funding NEW_DIRECTORY | neighborhood NEW_DIRECTORY | stream NEW_DIRECTORY | stream-resume SOURCE_SEGMENT NEW_DIRECTORY | admission NEW_DIRECTORY | admission-resume CHECKPOINT NEW_DIRECTORY | lifecycle NEW_DIRECTORY | lifecycle-resume CHECKPOINT NEW_DIRECTORY | arbitration NEW_DIRECTORY | mutation NEW_DIRECTORY | mutation-resume CHECKPOINT NEW_DIRECTORY | family NEW_DIRECTORY | family-resume CHECKPOINT NEW_DIRECTORY | d0 NEW_DIRECTORY | d0-resume CHECKPOINT NEW_DIRECTORY | choice NEW_DIRECTORY | choice-resume CHECKPOINT NEW_DIRECTORY | interventions NEW_DIRECTORY | smoke NEW_DIRECTORY [--config FILE] | resume CHECKPOINT NEW_DIRECTORY".into()),
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
