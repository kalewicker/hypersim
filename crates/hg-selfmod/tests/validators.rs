//! G-R4 gate under `cargo test` (see `docs/ere_validators.md`).
//! The `hg-selfmod gate_gr4` CLI runs the same gate with printed detail.

#[test]
fn gr4_quarantine_engagement() {
    assert!(
        hg_selfmod::gates::gate_gr4(),
        "G-R4 quarantine engagement must pass exactly"
    );
}
