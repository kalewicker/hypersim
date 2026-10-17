//! G-R1 harness under `cargo test` (see `docs/ere_gr1_protocol.md`).
//! Small-scale structural run: the full 10^4 gate is `hg-selfmod gate_gr1`.

use hg_selfmod::gr1::{gr1_seed_set, run_gr1};
use std::collections::HashSet;

#[test]
fn gr1_seed_set_well_formed() {
    let seeds = gr1_seed_set();
    assert!(seeds.len() >= 20, "need ≥20 seeds, have {}", seeds.len());
    let uniq: HashSet<Vec<u8>> = seeds.iter().map(|(_, s)| s.rcf_bytes()).collect();
    assert_eq!(
        uniq.len(),
        seeds.len(),
        "seeds must be pairwise RCF-distinct"
    );
}

#[test]
fn gr1_small_scale_structural() {
    let rep = run_gr1(300, 30);
    assert_eq!(rep.n, 300);
    let class_sum: usize = rep.counts.iter().sum();
    assert_eq!(class_sum, 300, "classes must partition samples");
    let op_sum: usize = rep.per_op.iter().map(|r| r.iter().sum::<usize>()).sum();
    assert_eq!(op_sum, 300, "per-op rows must partition samples");
    let v_sum: usize = rep.per_validator.iter().sum();
    assert_eq!(v_sum, rep.counts[hg_selfmod::gr1::C_QUARANTINED]);
    assert_eq!(rep.cap_total + rep.off_total, 300);
    assert_eq!(
        rep.cap_valid + rep.off_valid,
        300 - rep.counts[hg_selfmod::gr1::C_QUARANTINED],
        "valid = all non-quarantined (incl. catastrophic)"
    );
}
