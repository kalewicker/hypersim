//! The policy zoo (NN plan §5), ordered by parameter count.
//!
//! Every policy here obeys the creation-rule exemption
//! (`docs/policy_config.md` §3): creation bindings pass through
//! unconditionally, and only pattern-match bindings are scored and
//! truncated. `StimulusDriver` is the single sanctioned exception, and it
//! is never a learned component.

mod heuristics;
mod learned;
mod simple;
mod stack;

pub use heuristics::{DegreeGreedy, RuleBias};
pub use learned::{pool_standardize, MlpScorer, ScoreNorm, StaticBias, SCORE_NORM_EPS};
pub use simple::{PassThrough, RandomSubset};
pub use stack::{PolicyStack, StimulusDriver};

/// Numerically stable softmax over the finite entries of `logits`, writing
/// into `out`. Entries whose mask is `false` receive 0.0 and are excluded
/// from the normalization.
pub(crate) fn masked_softmax(logits: &[f32], mask: &[bool], out: &mut Vec<f32>) {
    out.clear();
    out.resize(logits.len(), 0.0);
    let max = logits
        .iter()
        .zip(mask)
        .filter(|(_, m)| **m)
        .map(|(l, _)| *l)
        .fold(f32::NEG_INFINITY, f32::max);
    if !max.is_finite() {
        return;
    }
    let mut sum = 0.0f32;
    for (i, (l, m)) in logits.iter().zip(mask).enumerate() {
        if *m {
            let e = (l - max).exp();
            out[i] = e;
            sum += e;
        }
    }
    if sum > 0.0 {
        for v in out.iter_mut() {
            *v /= sum;
        }
    }
}

/// Draw an index from a probability vector using one uniform sample.
/// Returns `None` if the vector sums to zero.
pub(crate) fn categorical(probs: &[f32], u: f32) -> Option<usize> {
    let total: f32 = probs.iter().sum();
    if total <= 0.0 {
        return None;
    }
    let mut acc = 0.0f32;
    let target = u * total;
    for (i, p) in probs.iter().enumerate() {
        acc += p;
        if acc >= target {
            return Some(i);
        }
    }
    // Float drift only: fall back to the last non-zero entry.
    probs.iter().rposition(|p| *p > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masked_softmax_normalizes_over_the_mask_only() {
        let mut out = Vec::new();
        masked_softmax(&[1.0, 2.0, 3.0], &[true, false, true], &mut out);
        assert_eq!(out[1], 0.0);
        assert!(((out[0] + out[2]) - 1.0).abs() < 1e-6);
        assert!(out[2] > out[0]);
    }

    #[test]
    fn masked_softmax_survives_large_logits() {
        let mut out = Vec::new();
        masked_softmax(&[1e30, 1e30 + 1.0], &[true, true], &mut out);
        assert!(out.iter().all(|v| v.is_finite()));
        assert!((out.iter().sum::<f32>() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn masked_softmax_all_masked_out_is_all_zero() {
        let mut out = Vec::new();
        masked_softmax(&[1.0, 2.0], &[false, false], &mut out);
        assert_eq!(out, vec![0.0, 0.0]);
    }

    #[test]
    fn categorical_covers_the_whole_unit_interval() {
        let probs = [0.25, 0.5, 0.25];
        assert_eq!(categorical(&probs, 0.0), Some(0));
        assert_eq!(categorical(&probs, 0.5), Some(1));
        assert_eq!(categorical(&probs, 1.0), Some(2));
        assert_eq!(categorical(&[0.0, 0.0], 0.5), None);
    }
}
