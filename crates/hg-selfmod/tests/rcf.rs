//! G-R0 gate under `cargo test` (see `docs/ere_canonical_form.md`).
//! The `hg-selfmod gate_rcf` CLI runs the same gate with printed detail.

#[test]
fn gr0_rcf_fixtures_hold() {
    assert!(
        hg_selfmod::gates::gate_rcf(),
        "G-R0 RCF fixture set must pass exactly"
    );
}
