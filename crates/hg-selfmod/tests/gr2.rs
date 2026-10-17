//! G-R2 envelope under `cargo test` (see `docs/ere_gr2_protocol.md`).
//! Short-trajectory pins on expressible rules plus the headline limit case.
//! The full 1000-step gate is `hg-selfmod gate_gr2`.

use hg_selfmod::gr2::{
    compare_rule, gr2_caps, load_shipped_rule, rule_to_spec, BehaviorVerdict, Inexpressible,
    GR2_SWEEP_WORLDS,
};

fn load(file: &str, name: &str) -> hg_rules::Rule {
    load_shipped_rule(file, name)
}

#[test]
fn gr2_expressible_rules_identical_short() {
    let caps = gr2_caps();
    for (file, name) in [
        ("crystal_battery.toml", "crystal_grow"),
        ("wolfram_analog.toml", "wolfram_like"),
        // E1d proof: the classic fixture blocked pre-extension now converts
        // and matches (see ere_gr2_findings.md re-run section).
        ("triangle_subdivide.toml", "triangle_subdivide"),
        // E2 proof: the energy-metered rule blocked pre-extension now
        // converts and matches.
        ("lk_metered_subdivide.toml", "lk_metered_subdivide"),
    ] {
        let rule = load(file, name);
        let spec = rule_to_spec(&rule, &caps).expect("envelope rule must convert");
        let beh = compare_rule(&rule, &spec, 60);
        assert_eq!(
            beh.verdict,
            BehaviorVerdict::Identical,
            "{file}::{name} must match"
        );
        assert_eq!(beh.traj_steps, 60);
        assert_eq!(beh.sweep_engaged, GR2_SWEEP_WORLDS);
        assert!(beh.engaged, "{file}::{name} must engage its fixture");
    }
}

#[test]
fn gr2_headline_limit_pinned() {
    // Post-E2, the binding constraint is creation schedules (SB-R2): seeding
    // ex nihilo stays out of envelope — pinned so lifting it is deliberate.
    let caps = gr2_caps();
    let rule = load("conservation_soak.toml", "seed_cycle");
    assert_eq!(rule_to_spec(&rule, &caps), Err(Inexpressible::Schedule));
}
