//! Syntax-only neighborhood; no reachability, behavior or fitness filtering.
use super::{mutation::raw_program_bytes, *};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub fn grammar_source_candidates(p: &D0Program) -> Result<Vec<(u32, u8, D0Program)>, String> {
    p.validate()?;
    let mut ids: Vec<_> = p.instructions.iter().map(|g| g.id).collect();
    ids.sort();
    let mut candidates = vec![];
    for id in ids {
        let old = p.instructions.iter().find(|g| g.id == id).unwrap().source;
        for source in 0..8 {
            if source == old {
                continue;
            }
            let mut q = p.clone();
            q.instructions
                .iter_mut()
                .find(|g| g.id == id)
                .unwrap()
                .source = source;
            if q.validate().is_ok() {
                candidates.push((id, source, q));
            }
        }
    }
    Ok(candidates)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrammarCertificate {
    pub request_id: i64,
    pub candidate_count: usize,
    pub candidate_sha256: String,
    pub mutation_gate_selected: bool,
    pub selected_index: Option<usize>,
    pub identity_probability_numerator: u64,
    pub identity_probability_denominator: u64,
    pub each_candidate_probability_numerator: u64,
    pub each_candidate_probability_denominator: u64,
}
pub fn grammar_certificate(r: &MutationRecord) -> Result<GrammarCertificate, String> {
    if r.operator != "grammar_source" || r.raw_draws.is_empty() {
        return Err("grammar certificate operator/draw".into());
    }
    let candidates = grammar_source_candidates(&r.parent_program)?;
    let mut hash = Sha256::new();
    hash.update(b"HG-TA-GRAMMAR-CANDIDATES-1");
    for (id, source, p) in &candidates {
        hash.update(id.to_le_bytes());
        hash.update([*source]);
        hash.update(raw_program_bytes(p));
    }
    let gate = r.raw_draws[0] % 4 != 0;
    let selected = r.target_id.zip(r.new_source).and_then(|(id, src)| {
        candidates
            .iter()
            .position(|(i, s, _)| *i == id && *s == src)
    });
    if (gate && !candidates.is_empty()) != selected.is_some() {
        return Err("grammar certificate selected alternative".into());
    }
    let n = candidates.len();
    Ok(GrammarCertificate {
        request_id: r.request_id,
        candidate_count: n,
        candidate_sha256: format!("{:x}", hash.finalize()),
        mutation_gate_selected: gate,
        selected_index: selected,
        identity_probability_numerator: 1,
        identity_probability_denominator: if n == 0 { 1 } else { 4 },
        each_candidate_probability_numerator: if n == 0 { 0 } else { 3 },
        each_candidate_probability_denominator: if n == 0 { 1 } else { 4 * n as u64 },
    })
}
