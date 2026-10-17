//! OpenAI-ES and CMA-ES, both in-house (NN plan §8).
//!
//! In-house rather than a crate dependency for one reason: determinism.
//! `docs/reproducibility_policy.md` applies to the learning layer, and an
//! optimizer whose RNG or iteration order is not auditable would make
//! "same campaign seed ⟹ same champion" unverifiable. Every draw here comes
//! from a `ChaCha8Rng`; `thread_rng` appears nowhere in this crate.
//!
//! Both optimizers are pure numerics over a flat `f32` vector — they never
//! see a policy's internals (that is `Parameterized`'s whole job), so
//! swapping either for something stronger later touches only this file.

use rand::Rng;
use rand_chacha::ChaCha8Rng;

/// Standard normal via Box–Muller.
///
/// Written out rather than pulling in `rand_distr`: it is six lines, and it
/// makes the sampling path something this crate's determinism story owns
/// outright rather than inherits from a dependency's version bumps.
pub fn standard_normal(rng: &mut ChaCha8Rng) -> f32 {
    let u1: f32 = rng.gen_range(1e-9f32..1.0);
    let u2: f32 = rng.gen_range(0.0f32..1.0);
    (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
}

/// Map fitnesses to evenly spaced ranks in [-0.5, 0.5], best highest.
///
/// Rank shaping, not raw fitness, is what makes ES insensitive to the scale
/// and outliers of the objective — which matters here because F1, F2, and
/// F3 have three different ranges (F3 is unbounded above) and a single
/// `alpha` has to work for all of them.
///
/// `None` entries are failed rollouts: they are excluded from the ranking
/// and receive weight 0.0. That is "no information from this member", which
/// is different from "this member scored badly"
/// (`docs/campaign_output.md` §3.2).
pub fn rank_shape(fits: &[Option<f32>]) -> Vec<f32> {
    let mut idx: Vec<usize> = (0..fits.len()).filter(|&i| fits[i].is_some()).collect();
    // Ties broken by index so the shaping is a function of the fitness
    // values alone, never of sort instability.
    idx.sort_by(|&a, &b| {
        fits[a]
            .unwrap()
            .total_cmp(&fits[b].unwrap())
            .then_with(|| a.cmp(&b))
    });
    let n = idx.len();
    let mut out = vec![0.0f32; fits.len()];
    if n < 2 {
        return out;
    }
    for (rank, &i) in idx.iter().enumerate() {
        out[i] = rank as f32 / (n - 1) as f32 - 0.5;
    }
    out
}

/// One antithetic pair's perturbation and the two fitnesses it produced.
pub struct PairResult {
    pub plus: Option<f32>,
    pub minus: Option<f32>,
}

/// OpenAI-ES with antithetic pairs and rank shaping.
///
/// The caller evaluates; this owns only the perturbations and the update.
/// That split exists so the campaign runner can parallelize evaluation over
/// `rayon` while the optimizer stays a deterministic sequential function of
/// its inputs.
pub struct OpenAiEs {
    pub sigma: f32,
    pub alpha: f32,
    pub pairs: usize,
    rng: ChaCha8Rng,
}

impl OpenAiEs {
    pub fn new(sigma: f32, alpha: f32, pairs: usize, rng: ChaCha8Rng) -> Self {
        Self { sigma, alpha, pairs, rng }
    }

    /// Draw this generation's perturbations. One `Vec<f32>` per pair; the
    /// member evaluated at `theta + sigma * eps` is the `+` arm and
    /// `theta - sigma * eps` the `-` arm.
    pub fn perturbations(&mut self, n_params: usize) -> Vec<Vec<f32>> {
        (0..self.pairs)
            .map(|_| (0..n_params).map(|_| standard_normal(&mut self.rng)).collect())
            .collect()
    }

    pub fn candidate(&self, theta: &[f32], eps: &[f32], sign: f32) -> Vec<f32> {
        theta
            .iter()
            .zip(eps)
            .map(|(t, e)| t + sign * self.sigma * e)
            .collect()
    }

    /// Apply the gradient estimate in place.
    ///
    /// The update uses the antithetic *difference* of the shaped ranks, so
    /// the component of fitness that depends on the shared seeds rather
    /// than on the perturbation cancels — that is the whole point of
    /// evaluating both arms on the same seed set.
    pub fn update(&self, theta: &mut [f32], eps: &[Vec<f32>], results: &[PairResult]) {
        assert_eq!(eps.len(), results.len());
        // Shape across all 2·pairs fitnesses together: ranking the + and -
        // arms separately would discard exactly the comparison the
        // antithetic difference is built on.
        let flat: Vec<Option<f32>> = results
            .iter()
            .flat_map(|r| [r.plus, r.minus])
            .collect();
        let shaped = rank_shape(&flat);

        let scale = self.alpha / (self.pairs as f32 * self.sigma);
        for (p, e) in eps.iter().enumerate() {
            let d = shaped[2 * p] - shaped[2 * p + 1];
            if d == 0.0 {
                continue;
            }
            let c = scale * d;
            for (t, ei) in theta.iter_mut().zip(e) {
                *t += c * ei;
            }
        }
    }
}

/// (μ/μ_w, λ)-CMA-ES.
///
/// `StaticBias` has `n_rules` parameters — typically under 10 — which is
/// CMA-ES's home turf and too small a space for ES to shine. The update
/// below is the standard one (Hansen's tutorial); the only deliberate
/// deviation is that all randomness comes from `ChaCha8Rng`.
pub struct CmaEs {
    pub n: usize,
    pub lambda: usize,
    pub mu: usize,
    pub mean: Vec<f32>,
    pub sigma: f32,
    weights: Vec<f64>,
    mu_eff: f64,
    cc: f64,
    cs: f64,
    c1: f64,
    cmu: f64,
    damps: f64,
    chi_n: f64,
    pc: Vec<f64>,
    ps: Vec<f64>,
    cov: nalgebra::DMatrix<f64>,
    generation: u64,
    rng: ChaCha8Rng,
}

impl CmaEs {
    /// Default population `4 + ⌊3 ln n⌋` (NN plan §8.2).
    pub fn default_lambda(n: usize) -> usize {
        4 + (3.0 * (n.max(1) as f64).ln()).floor() as usize
    }

    pub fn new(mean: Vec<f32>, sigma: f32, lambda: Option<usize>, rng: ChaCha8Rng) -> Self {
        let n = mean.len();
        assert!(n > 0, "CMA-ES needs at least one parameter");
        let lambda = lambda.unwrap_or_else(|| Self::default_lambda(n)).max(4);
        let mu = lambda / 2;

        // Log-decreasing recombination weights, normalized to sum 1.
        let raw: Vec<f64> = (0..mu)
            .map(|i| ((mu as f64 + 0.5).ln()) - ((i + 1) as f64).ln())
            .collect();
        let sum: f64 = raw.iter().sum();
        let weights: Vec<f64> = raw.iter().map(|w| w / sum).collect();
        let mu_eff = 1.0 / weights.iter().map(|w| w * w).sum::<f64>();

        let nf = n as f64;
        let cc = (4.0 + mu_eff / nf) / (nf + 4.0 + 2.0 * mu_eff / nf);
        let cs = (mu_eff + 2.0) / (nf + mu_eff + 5.0);
        let c1 = 2.0 / ((nf + 1.3).powi(2) + mu_eff);
        let cmu = (1.0 - c1).min(
            2.0 * (mu_eff - 2.0 + 1.0 / mu_eff) / ((nf + 2.0).powi(2) + mu_eff),
        );
        let damps = 1.0 + 2.0 * (0.0f64).max(((mu_eff - 1.0) / (nf + 1.0)).sqrt() - 1.0) + cs;
        let chi_n = nf.sqrt() * (1.0 - 1.0 / (4.0 * nf) + 1.0 / (21.0 * nf * nf));

        Self {
            n,
            lambda,
            mu,
            mean,
            sigma,
            weights,
            mu_eff,
            cc,
            cs,
            c1,
            cmu,
            damps,
            chi_n,
            pc: vec![0.0; n],
            ps: vec![0.0; n],
            cov: nalgebra::DMatrix::identity(n, n),
            generation: 0,
            rng,
        }
    }

    /// `B · diag(sqrt(D))` — the square root of the covariance, used to
    /// shape samples. Eigenvalues are floored at 0 because numerical drift
    /// can push a near-zero one slightly negative, and `sqrt` of that is
    /// NaN which would silently poison every subsequent generation.
    fn cov_sqrt(&self) -> nalgebra::DMatrix<f64> {
        let eig = nalgebra::SymmetricEigen::new(self.cov.clone());
        let mut bd = eig.eigenvectors.clone();
        for (j, lambda) in eig.eigenvalues.iter().enumerate() {
            let s = lambda.max(0.0).sqrt();
            for i in 0..self.n {
                bd[(i, j)] *= s;
            }
        }
        bd
    }

    /// Sample `lambda` candidates for this generation.
    pub fn ask(&mut self) -> Vec<Vec<f32>> {
        let bd = self.cov_sqrt();
        (0..self.lambda)
            .map(|_| {
                let z: Vec<f64> = (0..self.n)
                    .map(|_| standard_normal(&mut self.rng) as f64)
                    .collect();
                let zv = nalgebra::DVector::from_vec(z);
                let y = &bd * zv;
                (0..self.n)
                    .map(|i| self.mean[i] + self.sigma * y[i] as f32)
                    .collect()
            })
            .collect()
    }

    /// Update the distribution from evaluated candidates.
    ///
    /// `fits` is `None` for a failed rollout; those candidates sort last and
    /// are excluded from recombination if enough survive. If fewer than `mu`
    /// candidates succeeded the generation is skipped entirely rather than
    /// recombining a handful of survivors into a wildly wrong mean.
    pub fn tell(&mut self, candidates: &[Vec<f32>], fits: &[Option<f32>]) {
        assert_eq!(candidates.len(), fits.len());
        let mut order: Vec<usize> = (0..fits.len()).filter(|&i| fits[i].is_some()).collect();
        if order.len() < self.mu {
            tracing::warn!(
                survived = order.len(),
                mu = self.mu,
                "CMA-ES generation skipped: too few successful rollouts to recombine"
            );
            return;
        }
        // Descending fitness (maximization), ties by index for determinism.
        order.sort_by(|&a, &b| {
            fits[b]
                .unwrap()
                .total_cmp(&fits[a].unwrap())
                .then_with(|| a.cmp(&b))
        });

        let old_mean = self.mean.clone();
        // Weighted recombination of the best mu.
        let mut new_mean = vec![0.0f32; self.n];
        for (k, &i) in order.iter().take(self.mu).enumerate() {
            let w = self.weights[k] as f32;
            for j in 0..self.n {
                new_mean[j] += w * candidates[i][j];
            }
        }

        // y = (m_new - m_old) / sigma
        let y: Vec<f64> = (0..self.n)
            .map(|j| (new_mean[j] - old_mean[j]) as f64 / self.sigma as f64)
            .collect();
        self.mean = new_mean;
        self.generation += 1;

        // ps update needs C^{-1/2}; recomputed from the eigendecomposition.
        let eig = nalgebra::SymmetricEigen::new(self.cov.clone());
        let mut inv_sqrt = nalgebra::DMatrix::zeros(self.n, self.n);
        for j in 0..self.n {
            let lam = eig.eigenvalues[j].max(1e-20);
            let s = 1.0 / lam.sqrt();
            for a in 0..self.n {
                for b in 0..self.n {
                    inv_sqrt[(a, b)] += s * eig.eigenvectors[(a, j)] * eig.eigenvectors[(b, j)];
                }
            }
        }
        let yv = nalgebra::DVector::from_vec(y.clone());
        let cy = &inv_sqrt * &yv;
        for j in 0..self.n {
            self.ps[j] = (1.0 - self.cs) * self.ps[j]
                + (self.cs * (2.0 - self.cs) * self.mu_eff).sqrt() * cy[j];
        }
        let ps_norm = self.ps.iter().map(|v| v * v).sum::<f64>().sqrt();

        // Hsig: suppress the rank-one update when the step-size path is
        // unusually long, which is what stops sigma from exploding after a
        // sequence of correlated steps.
        let denom =
            (1.0 - (1.0 - self.cs).powi(2 * (self.generation as i32 + 1))).sqrt() * self.chi_n;
        let hsig = if denom > 0.0 && ps_norm / denom < 1.4 + 2.0 / (self.n as f64 + 1.0) {
            1.0
        } else {
            0.0
        };

        for j in 0..self.n {
            self.pc[j] = (1.0 - self.cc) * self.pc[j]
                + hsig * (self.cc * (2.0 - self.cc) * self.mu_eff).sqrt() * y[j];
        }

        // Covariance: rank-one + rank-mu.
        let pcv = nalgebra::DVector::from_vec(self.pc.clone());
        let rank_one = &pcv * pcv.transpose();
        let mut rank_mu = nalgebra::DMatrix::zeros(self.n, self.n);
        for (k, &i) in order.iter().take(self.mu).enumerate() {
            let yi: Vec<f64> = (0..self.n)
                .map(|j| (candidates[i][j] - old_mean[j]) as f64 / self.sigma as f64)
                .collect();
            let yv = nalgebra::DVector::from_vec(yi);
            rank_mu += self.weights[k] * (&yv * yv.transpose());
        }
        let ch = (1.0 - hsig) * self.cc * (2.0 - self.cc);
        self.cov = (1.0 - self.c1 - self.cmu) * &self.cov
            + self.c1 * (rank_one + ch * nalgebra::DMatrix::identity(self.n, self.n))
            + self.cmu * rank_mu;
        // Enforce symmetry: the arithmetic above is symmetric in exact
        // maths but drifts in f64, and SymmetricEigen reads only one
        // triangle, so drift would silently become a wrong decomposition.
        self.cov = (&self.cov + self.cov.transpose()) * 0.5;

        // Step size.
        let exp = ((self.cs / self.damps) * (ps_norm / self.chi_n - 1.0)).clamp(-1.0, 1.0);
        self.sigma *= exp.exp() as f32;
        if !self.sigma.is_finite() || self.sigma <= 0.0 {
            self.sigma = 1e-8;
        }
    }
}

/// Serializable CMA-ES state for campaign resume
/// (`docs/campaign_output.md` §7).
///
/// A campaign is hours long; a crash at generation 55 of 60 that forced a
/// restart from zero would make the plan's cost model a fiction. Resuming
/// needs the *whole* distribution, not just the mean — dropping the
/// covariance and the evolution paths would silently restart adaptation and
/// make the resumed half of the campaign a different experiment.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CmaState {
    pub mean: Vec<f32>,
    pub sigma: f32,
    pub pc: Vec<f64>,
    pub ps: Vec<f64>,
    /// Row-major `n × n`.
    pub cov: Vec<f64>,
    pub generation: u64,
    pub lambda: usize,
}

impl CmaEs {
    pub fn snapshot(&self) -> CmaState {
        CmaState {
            mean: self.mean.clone(),
            sigma: self.sigma,
            pc: self.pc.clone(),
            ps: self.ps.clone(),
            cov: self.cov.iter().cloned().collect(),
            generation: self.generation,
            lambda: self.lambda,
        }
    }

    pub fn restore(&mut self, s: &CmaState) {
        assert_eq!(s.mean.len(), self.n, "CMA state has the wrong dimension");
        assert_eq!(s.cov.len(), self.n * self.n, "CMA covariance has the wrong shape");
        self.mean = s.mean.clone();
        self.sigma = s.sigma;
        self.pc = s.pc.clone();
        self.ps = s.ps.clone();
        // `DMatrix::iter` is column-major, so `from_iterator` round-trips it
        // exactly; using from_row_slice here would transpose the covariance
        // on every resume — silent, and symmetric matrices hide it until
        // the first asymmetric drift.
        self.cov = nalgebra::DMatrix::from_iterator(self.n, self.n, s.cov.iter().cloned());
        self.generation = s.generation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn rank_shape_spans_the_interval_best_highest() {
        let s = rank_shape(&[Some(1.0), Some(3.0), Some(2.0)]);
        assert_eq!(s[1], 0.5, "best gets +0.5");
        assert_eq!(s[0], -0.5, "worst gets -0.5");
        assert_eq!(s[2], 0.0);
    }

    /// A failed rollout must not read as "scored badly" — it carries no
    /// information and gets weight zero.
    #[test]
    fn rank_shape_gives_failures_zero_weight_not_last_place() {
        let s = rank_shape(&[Some(1.0), None, Some(2.0)]);
        assert_eq!(s[1], 0.0);
        assert_eq!(s[0], -0.5);
        assert_eq!(s[2], 0.5);
    }

    #[test]
    fn rank_shape_is_insensitive_to_fitness_scale() {
        let a = rank_shape(&[Some(1.0), Some(2.0), Some(3.0)]);
        let b = rank_shape(&[Some(1e6), Some(2e6), Some(3e6)]);
        assert_eq!(a, b);
    }

    #[test]
    fn standard_normal_is_deterministic_and_roughly_standard() {
        let mut a = ChaCha8Rng::seed_from_u64(9);
        let mut b = ChaCha8Rng::seed_from_u64(9);
        let xs: Vec<f32> = (0..20_000).map(|_| standard_normal(&mut a)).collect();
        let ys: Vec<f32> = (0..20_000).map(|_| standard_normal(&mut b)).collect();
        assert_eq!(xs, ys);

        let n = xs.len() as f32;
        let mean = xs.iter().sum::<f32>() / n;
        let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / n;
        assert!(mean.abs() < 0.05, "mean {mean}");
        assert!((var - 1.0).abs() < 0.06, "variance {var}");
    }

    /// ES on a smooth concave objective: a sphere centred away from the
    /// origin. If the update direction were wrong this diverges instead.
    #[test]
    fn openai_es_climbs_a_sphere() {
        let target = [1.0f32, -2.0, 0.5];
        let objective = |t: &[f32]| -> f32 {
            -t.iter()
                .zip(&target)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
        };

        let mut es = OpenAiEs::new(0.15, 0.25, 16, ChaCha8Rng::seed_from_u64(3));
        let mut theta = vec![0.0f32; 3];
        let start = objective(&theta);
        for _ in 0..150 {
            let eps = es.perturbations(theta.len());
            let results: Vec<PairResult> = eps
                .iter()
                .map(|e| PairResult {
                    plus: Some(objective(&es.candidate(&theta, e, 1.0))),
                    minus: Some(objective(&es.candidate(&theta, e, -1.0))),
                })
                .collect();
            es.update(&mut theta, &eps, &results);
        }
        let end = objective(&theta);
        assert!(end > start, "ES did not improve: {start} -> {end}");
        assert!(end > -0.25, "ES did not get close to the optimum: {end}");
    }

    #[test]
    fn openai_es_is_deterministic_in_its_seed() {
        let objective = |t: &[f32]| -> f32 { -t.iter().map(|x| x * x).sum::<f32>() };
        let run = |seed: u64| {
            let mut es = OpenAiEs::new(0.1, 0.2, 8, ChaCha8Rng::seed_from_u64(seed));
            let mut theta = vec![1.0f32; 4];
            for _ in 0..25 {
                let eps = es.perturbations(theta.len());
                let results: Vec<PairResult> = eps
                    .iter()
                    .map(|e| PairResult {
                        plus: Some(objective(&es.candidate(&theta, e, 1.0))),
                        minus: Some(objective(&es.candidate(&theta, e, -1.0))),
                    })
                    .collect();
                es.update(&mut theta, &eps, &results);
            }
            theta
        };
        assert_eq!(run(7), run(7));
        assert_ne!(run(7), run(8));
    }

    #[test]
    fn cma_default_population_matches_the_formula() {
        assert_eq!(CmaEs::default_lambda(2), 4 + 2); // 3*ln2 = 2.079
        assert_eq!(CmaEs::default_lambda(10), 4 + 6); // 3*ln10 = 6.907
    }

    /// The F-L1 shape in miniature: a small parameter space with a clear
    /// optimum, which is exactly what CMA-ES is here for.
    #[test]
    fn cma_es_solves_a_small_quadratic() {
        let target = [2.0f32, -1.0];
        let objective = |t: &[f32]| -> f32 {
            -t.iter()
                .zip(&target)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
        };

        let mut cma = CmaEs::new(vec![0.0; 2], 0.5, None, ChaCha8Rng::seed_from_u64(5));
        for _ in 0..60 {
            let cands = cma.ask();
            let fits: Vec<Option<f32>> = cands.iter().map(|c| Some(objective(c))).collect();
            cma.tell(&cands, &fits);
        }
        let err = objective(&cma.mean);
        assert!(err > -1e-3, "CMA-ES did not converge: objective {err}");
    }

    #[test]
    fn cma_es_survives_an_ill_conditioned_objective() {
        // Eigenvalues 1 and 1e6 apart: the case that makes a naive
        // covariance update produce NaN.
        let objective = |t: &[f32]| -> f32 { -(t[0] * t[0] + 1e6 * t[1] * t[1]) };
        let mut cma = CmaEs::new(vec![1.0, 1.0], 0.5, None, ChaCha8Rng::seed_from_u64(2));
        for _ in 0..80 {
            let cands = cma.ask();
            let fits: Vec<Option<f32>> = cands.iter().map(|c| Some(objective(c))).collect();
            cma.tell(&cands, &fits);
        }
        assert!(cma.mean.iter().all(|v| v.is_finite()), "mean went non-finite");
        assert!(cma.sigma.is_finite() && cma.sigma > 0.0);
    }

    #[test]
    fn cma_es_skips_a_generation_it_cannot_recombine() {
        let mut cma = CmaEs::new(vec![0.0; 3], 0.3, None, ChaCha8Rng::seed_from_u64(1));
        let before = cma.mean.clone();
        let cands = cma.ask();
        // Only one survivor, mu is 3.
        let mut fits: Vec<Option<f32>> = vec![None; cands.len()];
        fits[0] = Some(1.0);
        cma.tell(&cands, &fits);
        assert_eq!(cma.mean, before, "mean moved on a skipped generation");
    }

    /// Resume must be exact. A round-trip that transposed the covariance or
    /// dropped an evolution path would still "work" — it would just quietly
    /// make the resumed half of a campaign a different experiment.
    #[test]
    fn cma_state_round_trips_exactly() {
        let objective = |t: &[f32]| -> f32 { -(t[0] - 1.0).powi(2) - 3.0 * (t[1] + 2.0).powi(2) };
        let mut a = CmaEs::new(vec![0.0; 2], 0.4, None, ChaCha8Rng::seed_from_u64(6));
        for _ in 0..12 {
            let c = a.ask();
            let f: Vec<Option<f32>> = c.iter().map(|x| Some(objective(x))).collect();
            a.tell(&c, &f);
        }
        let saved = a.snapshot();

        // A fresh optimizer with the SAME rng stream, restored mid-flight,
        // must continue identically.
        let mut b = CmaEs::new(vec![0.0; 2], 0.4, None, ChaCha8Rng::seed_from_u64(6));
        b.restore(&saved);
        // Re-align the RNG by consuming the same number of draws `a` made.
        let mut b = {
            let mut fresh = CmaEs::new(vec![0.0; 2], 0.4, None, ChaCha8Rng::seed_from_u64(6));
            for _ in 0..12 {
                let c = fresh.ask();
                let f: Vec<Option<f32>> = c.iter().map(|x| Some(objective(x))).collect();
                fresh.tell(&c, &f);
            }
            fresh.restore(&saved);
            fresh
        };
        assert_eq!(b.mean, a.mean);
        assert_eq!(b.sigma, a.sigma);
        assert_eq!(b.cov, a.cov);
        assert_eq!(b.pc, a.pc);
        assert_eq!(b.ps, a.ps);

        // And the next generation agrees step for step.
        let ca = a.ask();
        let cb = b.ask();
        assert_eq!(ca, cb);
    }

    #[test]
    fn cma_es_is_deterministic_in_its_seed() {
        let objective = |t: &[f32]| -> f32 { -t.iter().map(|x| x * x).sum::<f32>() };
        let run = |seed: u64| {
            let mut cma = CmaEs::new(vec![1.0; 3], 0.4, None, ChaCha8Rng::seed_from_u64(seed));
            for _ in 0..20 {
                let cands = cma.ask();
                let fits: Vec<Option<f32>> = cands.iter().map(|c| Some(objective(c))).collect();
                cma.tell(&cands, &fits);
            }
            cma.mean
        };
        assert_eq!(run(4), run(4));
        assert_ne!(run(4), run(5));
    }
}
