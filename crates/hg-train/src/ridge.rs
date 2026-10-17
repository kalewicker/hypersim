//! Ridge readout and memory capacity — Stage 1's measurement layer
//! (NN plan §11, Stage 1).
//!
//! In-house against `nalgebra`, which is already in the stack:
//! `w = (XᵀX + λI)⁻¹ Xᵀy`. The quantity reported is
//!
//! ```text
//! MC = Σ_d r²(d)
//! ```
//!
//! the summed coefficient of determination of a linear readout predicting
//! the input `d` symbols ago from the reservoir state now. It is the
//! standard reservoir-computing memory capacity, and it is bounded above by
//! the state dimension.
//!
//! ## Why r² and not correlation
//!
//! `r²` here is the *out-of-sample* coefficient of determination — fit on a
//! training split, scored on a held-out split. Reservoir papers often report
//! the in-sample squared correlation, which on a state vector of dimension
//! comparable to the sample count is close to 1 by construction and says
//! nothing. With `n_windows` in the low hundreds and a 16-bucket state, that
//! failure mode is live rather than theoretical, so the split is not
//! optional.
//!
//! A negative `r²` (worse than predicting the mean) is clamped to 0 for the
//! MC sum, because a delay the readout cannot decode contributes no memory —
//! but the raw value is returned alongside, so "slightly negative" and
//! "exactly zero" stay distinguishable.

use nalgebra::{DMatrix, DVector};

/// Default ridge parameter. Small relative to the state scale: the point is
/// to make `XᵀX` invertible when buckets are collinear, not to regularize
/// for generalization.
pub const DEFAULT_LAMBDA: f64 = 1e-6;

/// Fraction of samples used for fitting; the remainder scores.
pub const TRAIN_FRACTION: f64 = 0.7;

#[derive(Clone, Debug, PartialEq)]
pub struct RidgeFit {
    pub weights: Vec<f64>,
    pub intercept: f64,
    /// Out-of-sample coefficient of determination, unclamped.
    pub r2: f64,
    pub n_train: usize,
    pub n_test: usize,
}

/// Fit `y ~ X` with an intercept, scoring on a held-out tail split.
///
/// Returns `None` when the split leaves too few samples on either side to
/// mean anything — an r² computed from two test points is noise with a
/// number attached.
pub fn ridge_fit(x: &[Vec<f64>], y: &[f64], lambda: f64) -> Option<RidgeFit> {
    if x.len() != y.len() || x.is_empty() {
        return None;
    }
    let dim = x[0].len();
    if dim == 0 || x.iter().any(|r| r.len() != dim) {
        return None;
    }
    let n = x.len();
    let n_train = ((n as f64) * TRAIN_FRACTION).floor() as usize;
    let n_test = n - n_train;
    // Need more training rows than free parameters, or the fit is
    // interpolation and the r² is meaningless.
    if n_train <= dim + 1 || n_test < 3 {
        return None;
    }

    // Design matrix with a leading 1 column for the intercept. The
    // intercept is NOT penalized: shrinking it toward zero would bias every
    // prediction toward the origin rather than toward the mean.
    let build = |rows: &[Vec<f64>]| -> DMatrix<f64> {
        DMatrix::from_fn(rows.len(), dim + 1, |i, j| {
            if j == 0 {
                1.0
            } else {
                rows[i][j - 1]
            }
        })
    };
    let xtr = build(&x[..n_train]);
    let ytr = DVector::from_column_slice(&y[..n_train]);

    let mut penalty = DMatrix::<f64>::identity(dim + 1, dim + 1);
    penalty[(0, 0)] = 0.0;
    let gram = xtr.transpose() * &xtr + lambda * penalty;
    let w = gram.lu().solve(&(xtr.transpose() * ytr))?;

    let xte = build(&x[n_train..]);
    let pred = xte * &w;
    let yte = &y[n_train..];
    let mean = yte.iter().sum::<f64>() / n_test as f64;
    let ss_res: f64 = yte
        .iter()
        .zip(pred.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum();
    let ss_tot: f64 = yte.iter().map(|a| (a - mean).powi(2)).sum();
    // A constant target has no variance to explain. r² is undefined, not 1.
    if ss_tot <= 0.0 {
        return None;
    }

    Some(RidgeFit {
        weights: w.iter().skip(1).cloned().collect(),
        intercept: w[0],
        r2: 1.0 - ss_res / ss_tot,
        n_train,
        n_test,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryCapacity {
    /// `(delay, r²)` for each delay attempted, unclamped.
    pub per_delay: Vec<(usize, f64)>,
    /// `Σ max(0, r²(d))`.
    pub mc: f64,
    /// Largest delay with `r² > 0.1`, the plan's decodability bar.
    pub max_decodable_delay: Option<usize>,
    /// r² at delay **0**: how well the state reflects the symbol being
    /// injected right now. Reported separately and excluded from `mc`
    /// because it is input coupling, not memory — but it is the number that
    /// separates "the substrate forgets within one window" from "the probe
    /// and the injection never worked", and those are very different
    /// findings.
    pub input_coupling_r2: Option<f64>,
    /// r² of a linear readout for XOR of the inputs at delays 1 and 2 —
    /// a task no linear function of the input history can solve, so a
    /// nonzero score means the substrate itself supplied the nonlinearity.
    pub nonlinearity_r2: Option<f64>,
    pub state_dim: usize,
    pub n_samples: usize,
}

/// Memory capacity of a reservoir state sequence against its input tape.
///
/// `states[i]` is the state at window `i`; `inputs[i]` is the symbol
/// injected during window `i`. Delay `d` asks: can a linear readout of
/// `states[i]` recover `inputs[i - d]`?
///
/// **Delays run from 1, not 0.** Delay 0 is "the symbol being injected right
/// now", which is input rather than memory, and including it would add ~1.0
/// to every MC on any substrate that responds to its input at all. A
/// consequence worth stating because it is a factor-of-one ambiguity in the
/// literature: a perfect `k`-tap delay line, whose taps hold delays
/// `0 … k−1`, scores **MC = k − 1** here, not `k`.
pub fn memory_capacity(
    states: &[Vec<f64>],
    inputs: &[f64],
    max_delay: usize,
    lambda: f64,
) -> MemoryCapacity {
    let state_dim = states.first().map(|s| s.len()).unwrap_or(0);
    let mut per_delay = Vec::new();
    let mut mc = 0.0;
    let mut max_decodable = None;

    let input_coupling_r2 = ridge_fit(states, &inputs[..states.len().min(inputs.len())], lambda)
        .map(|f| f.r2);

    for d in 1..=max_delay {
        if states.len() <= d {
            break;
        }
        let x: Vec<Vec<f64>> = states[d..].to_vec();
        let y: Vec<f64> = inputs[..inputs.len().min(states.len()) - d].to_vec();
        let n = x.len().min(y.len());
        let Some(fit) = ridge_fit(&x[..n], &y[..n], lambda) else {
            continue;
        };
        per_delay.push((d, fit.r2));
        mc += fit.r2.max(0.0);
        if fit.r2 > 0.1 {
            max_decodable = Some(d);
        }
    }

    // Nonlinearity: XOR of the symbols at delays 1 and 2.
    let nonlinearity_r2 = (states.len() > 3).then(|| {
        let x: Vec<Vec<f64>> = states[2..].to_vec();
        let y: Vec<f64> = (0..x.len())
            .map(|i| {
                let a = inputs.get(i + 1).copied().unwrap_or(0.0) > 0.5;
                let b = inputs.get(i).copied().unwrap_or(0.0) > 0.5;
                f64::from(a != b)
            })
            .collect();
        ridge_fit(&x, &y, lambda).map(|f| f.r2)
    })
    .flatten();

    MemoryCapacity {
        per_delay,
        mc,
        max_decodable_delay: max_decodable,
        input_coupling_r2,
        nonlinearity_r2,
        state_dim,
        n_samples: states.len(),
    }
}

/// A standard echo-state network, used only as a **baseline of matched
/// state dimension** for Stage 1's MC comparison.
///
/// Not part of the substrate and never coupled to it: it exists so that
/// "the hypergraph has memory capacity 3.2" can be read against a
/// conventional reservoir of the same width on the same tape, rather than
/// against nothing.
pub fn esn_states(inputs: &[f64], dim: usize, spectral_radius: f64, seed: u64) -> Vec<Vec<f64>> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

    let mut w: Vec<Vec<f64>> = (0..dim)
        .map(|_| (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
        .collect();
    // Scale to the requested spectral radius. The power-iteration estimate
    // is enough: the baseline's job is to be a reasonable ESN, not a
    // precisely tuned one, and over-precision here would invite reading the
    // baseline as a result in its own right.
    let mut v = vec![1.0f64; dim];
    for _ in 0..64 {
        let mut nv = vec![0.0; dim];
        for i in 0..dim {
            for j in 0..dim {
                nv[i] += w[i][j] * v[j];
            }
        }
        let norm = nv.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm <= 0.0 {
            break;
        }
        v = nv.iter().map(|x| x / norm).collect();
    }
    let mut wv = vec![0.0; dim];
    for i in 0..dim {
        for j in 0..dim {
            wv[i] += w[i][j] * v[j];
        }
    }
    let rho = wv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
    let scale = spectral_radius / rho;
    for row in w.iter_mut() {
        for x in row.iter_mut() {
            *x *= scale;
        }
    }

    let w_in: Vec<f64> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let mut state = vec![0.0f64; dim];
    let mut out = Vec::with_capacity(inputs.len());
    for &u in inputs {
        let mut next = vec![0.0; dim];
        for i in 0..dim {
            let mut acc = w_in[i] * u;
            for j in 0..dim {
                acc += w[i][j] * state[j];
            }
            next[i] = acc.tanh();
        }
        state = next;
        out.push(state.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};

    fn tape(n: usize, seed: u64) -> Vec<f64> {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        (0..n).map(|_| f64::from(rng.gen_bool(0.5))).collect()
    }

    #[test]
    fn ridge_recovers_a_known_linear_map() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(1);
        let x: Vec<Vec<f64>> = (0..200)
            .map(|_| vec![rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)])
            .collect();
        let y: Vec<f64> = x.iter().map(|r| 3.0 * r[0] - 2.0 * r[1] + 0.5).collect();
        let fit = ridge_fit(&x, &y, 1e-9).expect("fit");
        assert!((fit.weights[0] - 3.0).abs() < 1e-4, "w0 {}", fit.weights[0]);
        assert!((fit.weights[1] + 2.0).abs() < 1e-4, "w1 {}", fit.weights[1]);
        assert!((fit.intercept - 0.5).abs() < 1e-4);
        assert!(fit.r2 > 0.999, "r2 {}", fit.r2);
    }

    /// The failure mode the held-out split exists to prevent: pure noise
    /// with a state dimension comparable to the sample count fits perfectly
    /// in sample and must score ~0 out of sample.
    #[test]
    fn ridge_does_not_reward_fitting_noise() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2);
        let x: Vec<Vec<f64>> = (0..80)
            .map(|_| (0..20).map(|_| rng.gen_range(-1.0..1.0)).collect())
            .collect();
        let y: Vec<f64> = (0..80).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let fit = ridge_fit(&x, &y, DEFAULT_LAMBDA).expect("fit");
        assert!(
            fit.r2 < 0.3,
            "out-of-sample r2 on pure noise was {} — the split is not doing its job",
            fit.r2
        );
    }

    #[test]
    fn ridge_refuses_a_fit_it_cannot_score() {
        // More parameters than training rows.
        let x: Vec<Vec<f64>> = (0..6).map(|_| vec![1.0; 10]).collect();
        let y = vec![1.0; 6];
        assert_eq!(ridge_fit(&x, &y, DEFAULT_LAMBDA), None);
        // Constant target: r² undefined, not 1.
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(3);
        let x: Vec<Vec<f64>> = (0..100)
            .map(|_| vec![rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)])
            .collect();
        assert_eq!(ridge_fit(&x, &vec![7.0; 100], DEFAULT_LAMBDA), None);
    }

    /// A perfect `k`-tap delay line holds delays `0 … k−1`, and MC counts
    /// from delay 1, so its memory capacity is `k − 1` — see the note on
    /// [`memory_capacity`]. Delays past the line decode not at all.
    #[test]
    fn memory_capacity_of_a_delay_line_is_one_less_than_its_tap_count() {
        let inputs = tape(400, 7);
        let k = 5usize;
        let states: Vec<Vec<f64>> = (0..inputs.len())
            .map(|i| {
                (0..k)
                    .map(|d| if i >= d { inputs[i - d] } else { 0.0 })
                    .collect()
            })
            .collect();

        let mc = memory_capacity(&states, &inputs, 10, DEFAULT_LAMBDA);
        let r2_at = |d: usize| mc.per_delay.iter().find(|(dd, _)| *dd == d).unwrap().1;

        for d in 1..k {
            assert!(r2_at(d) > 0.95, "delay {d} should decode exactly, got {}", r2_at(d));
        }
        for d in k..=10 {
            assert!(r2_at(d) < 0.2, "delay {d} is past the line, got {}", r2_at(d));
        }
        assert!(
            (mc.mc - (k - 1) as f64).abs() < 0.6,
            "MC {} should be about {}",
            mc.mc,
            k - 1
        );
        assert_eq!(mc.max_decodable_delay, Some(k - 1));
    }

    /// A memoryless state carries nothing: MC must be ~0, not "small but
    /// positive because r² is noisy".
    #[test]
    fn memory_capacity_of_a_memoryless_state_is_zero() {
        let inputs = tape(400, 9);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(11);
        let states: Vec<Vec<f64>> = (0..inputs.len())
            .map(|_| (0..8).map(|_| rng.gen_range(-1.0..1.0)).collect())
            .collect();
        let mc = memory_capacity(&states, &inputs, 8, DEFAULT_LAMBDA);
        assert!(mc.mc < 0.5, "MC on a memoryless state was {}", mc.mc);
        assert_eq!(mc.max_decodable_delay, None);
    }

    /// A LINEAR delay line cannot solve XOR — that is the point of the
    /// nonlinearity probe. If this ever scored high, the probe would be
    /// measuring leakage rather than substrate nonlinearity.
    #[test]
    fn the_nonlinearity_probe_rejects_a_linear_delay_line() {
        let inputs = tape(400, 13);
        let states: Vec<Vec<f64>> = (0..inputs.len())
            .map(|i| {
                (0..4)
                    .map(|d| if i >= d { inputs[i - d] } else { 0.0 })
                    .collect()
            })
            .collect();
        let mc = memory_capacity(&states, &inputs, 4, DEFAULT_LAMBDA);
        let nl = mc.nonlinearity_r2.expect("nonlinearity score");
        assert!(nl < 0.3, "linear delay line scored {nl} on XOR");
    }

    /// And a state that explicitly contains the XOR is decodable, so the
    /// probe is not simply always-negative.
    #[test]
    fn the_nonlinearity_probe_finds_an_explicit_xor() {
        let inputs = tape(400, 17);
        let states: Vec<Vec<f64>> = (0..inputs.len())
            .map(|i| {
                let a = if i >= 1 { inputs[i - 1] } else { 0.0 };
                let b = if i >= 2 { inputs[i - 2] } else { 0.0 };
                vec![a, b, f64::from((a > 0.5) != (b > 0.5))]
            })
            .collect();
        // states[i] holds xor(inputs[i-1], inputs[i-2]); memory_capacity's
        // probe reads states[2..] against xor(inputs[i+1], inputs[i]),
        // which is the same pairing shifted.
        let mc = memory_capacity(&states, &inputs, 3, DEFAULT_LAMBDA);
        let nl = mc.nonlinearity_r2.expect("nonlinearity score");
        assert!(nl > 0.9, "explicit XOR state scored only {nl}");
    }

    #[test]
    fn esn_baseline_has_memory() {
        let inputs = tape(500, 23);
        let states = esn_states(&inputs, 16, 0.9, 5);
        let mc = memory_capacity(&states, &inputs, 12, DEFAULT_LAMBDA);
        assert!(mc.mc > 1.0, "an ESN at rho=0.9 should remember something, MC {}", mc.mc);
        assert!(
            mc.mc <= 16.0 + 1.0,
            "MC {} exceeded the state dimension, which is impossible",
            mc.mc
        );
    }

    /// A near-zero spectral radius destroys the echo state: no memory.
    #[test]
    fn esn_baseline_loses_memory_when_the_spectral_radius_collapses() {
        let inputs = tape(500, 29);
        let hot = memory_capacity(&esn_states(&inputs, 16, 0.9, 5), &inputs, 12, DEFAULT_LAMBDA);
        let cold = memory_capacity(&esn_states(&inputs, 16, 0.01, 5), &inputs, 12, DEFAULT_LAMBDA);
        assert!(
            cold.mc < hot.mc,
            "rho=0.01 MC {} should be below rho=0.9 MC {}",
            cold.mc,
            hot.mc
        );
    }

    #[test]
    fn esn_is_deterministic_in_its_seed() {
        let inputs = tape(64, 31);
        assert_eq!(esn_states(&inputs, 8, 0.9, 3), esn_states(&inputs, 8, 0.9, 3));
        assert_ne!(esn_states(&inputs, 8, 0.9, 3), esn_states(&inputs, 8, 0.9, 4));
    }
}
