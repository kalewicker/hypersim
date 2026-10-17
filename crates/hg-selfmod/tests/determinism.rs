//! D1/D2 gates under `cargo test` (see `docs/ere_determinism.md`).
//! The `hg-selfmod gate_d1` / `gate_d2` CLIs run the same gates with detail.

#[test]
fn d1_insertion_order_invariance() {
    assert!(
        hg_selfmod::gates::gate_d1(),
        "G-D1 insertion-order invariance must pass exactly"
    );
}

#[test]
fn d2_no_rng_in_compile_path() {
    assert!(
        hg_selfmod::gates::gate_d2(),
        "G-D2 no-RNG compile path must pass exactly"
    );
}
