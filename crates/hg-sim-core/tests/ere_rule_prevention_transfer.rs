#![cfg(feature = "hg-meta")]
use hg_sim_core::ere::rule_prevention_transfer as p;
use serde_json::json;
use std::time::Instant;
#[test]
fn catalogue_and_deadline_refusals() {
    let mut rows = vec![];
    for arm in p::ARMS {
        for a in 0..4 {
            for b in 0..4 {
                let (raw, compiled) = p::catalogue(arm, [a, b]).unwrap();
                assert_eq!(raw.len(), 9);
                assert!(compiled
                    .iter()
                    .all(|r| r.meta_visible && r.pattern.node_label_ceiling.is_none()));
                rows.push(json!({"arm":arm,"actions":[a,b],"raw":raw,"compiled":compiled}));
            }
        }
    }
    assert!(p::catalogue(p::Arm::Paid, [4, 0]).is_err());
    assert!(p::run(p::Arm::Paid, [10, 11], [1, 1], Instant::now()).is_err());
    assert!(p::run(
        p::Arm::Paid,
        [99, 11],
        [1, 1],
        Instant::now() + std::time::Duration::from_secs(1)
    )
    .is_err());
    assert!(p::run(
        p::Arm::Paid,
        [10, 11],
        [4, 1],
        Instant::now() + std::time::Duration::from_secs(1)
    )
    .is_err());
    if let Some(path) = std::env::var_os("ERE_RULE_PREVENTION_TRANSFER_PREFLIGHT_OUTPUT") {
        std::fs::write(
            std::path::PathBuf::from(path).join("catalogues.json"),
            serde_json::to_vec(&rows).unwrap(),
        )
        .unwrap();
    }
}
