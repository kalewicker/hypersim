//! Mask-aware likelihood-ratio adapter for the existing complete grammar.
//! Pure arithmetic: no Engine, RNG, admission, rollout, ranking or training loop.
//! A proposed update is not evidence that its native utility improves.
use super::{complete_admission::Res, feasibility_training};
use hg_sim_core::ere::valid_grammar::{Grammar, PARAMS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const PROFILE: &str = "E2-direct-categorical-adapter-v1";
pub const BASELINE: &str = "leave-one-out-emission-mean-v1";
pub const OBJECTIVE: &str = "mean-constraint-utility-v1";
const PARENTS: usize = 168;
const OPERATORS: usize = 13;
const ROLES: usize = 2;

fn check(ok: bool, message: &str) -> Res<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn index(value: &Value) -> Res<usize> {
    value
        .as_u64()
        .and_then(|x| usize::try_from(x).ok())
        .ok_or("unsigned catalogue index required".into())
}

/// An exact legal production, not just an executable-signature equivalence class.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub parent: usize,
    pub operator: usize,
    /// 0 = Core, 1 = Proposal; singleton legal masks have zero score derivative.
    pub role: usize,
    pub production_index: usize,
}

/// Immutable support/count model. Construction validates all legal masks.
pub struct Model {
    counts: Vec<Vec<[usize; ROLES]>>,
}
impl Model {
    fn new(counts: Vec<Vec<[usize; ROLES]>>) -> Res<Self> {
        check(counts.len() == PARENTS, "168 parents required")?;
        check(
            counts
                .iter()
                .all(|p| p.len() == OPERATORS && p.iter().flatten().any(|&n| n > 0)),
            "13 groups and nonempty support per parent required",
        )?;
        Ok(Self { counts })
    }

    /// Reads an already compiled catalogue; never compiles or samples a grammar.
    /// The caller binds its bytes/hash and validates any native input receipts.
    pub fn from_catalogue(cat: &Value) -> Res<Self> {
        check(
            cat["profile"] == hg_sim_core::ere::valid_grammar::PROFILE
                && index(&cat["params"])? == PARAMS
                && cat["engine_steps"] == 0,
            "complete no-physics catalogue required",
        )?;
        let parents = cat["parents"]
            .as_array()
            .ok_or("catalogue parents required")?;
        check(parents.len() == PARENTS, "168 parents required")?;
        let mut counts = vec![];
        for (p, parent) in parents.iter().enumerate() {
            check(index(&parent["parent"])? == p, "parent catalogue order")?;
            let groups = parent["groups"]
                .as_array()
                .ok_or("operator groups required")?;
            check(groups.len() == OPERATORS, "13 operator groups required")?;
            let mut cp = vec![];
            for (o, group) in groups.iter().enumerate() {
                check(index(&group["operator"])? == o, "operator catalogue order")?;
                let roles = group["roles"].as_array().ok_or("roles required")?;
                check(roles.len() == ROLES, "two role entries required")?;
                let mut cr = [0; ROLES];
                for (r, role) in roles.iter().enumerate() {
                    check(index(&role["role"])? == r, "role catalogue order")?;
                    let productions = role["productions"]
                        .as_array()
                        .ok_or("productions required")?;
                    for (k, leaf) in productions.iter().enumerate() {
                        check(index(&leaf["index"])? == k, "production catalogue order")?;
                    }
                    cr[r] = productions.len();
                }
                let legal: Vec<_> = (0..ROLES).filter(|&r| cr[r] > 0).collect();
                check(
                    group["role_support"] == serde_json::json!(legal),
                    "role mask/count mismatch",
                )?;
                cp.push(cr);
            }
            let legal: Vec<_> = (0..OPERATORS)
                .filter(|&o| cp[o].iter().any(|&n| n > 0))
                .collect();
            check(
                parent["operator_support"] == serde_json::json!(legal),
                "operator mask/count mismatch",
            )?;
            counts.push(cp);
        }
        Self::new(counts)
    }

    /// Support extraction from an existing native Grammar, with no new compilation.
    pub fn from_grammar(grammar: &Grammar) -> Res<Self> {
        check(
            grammar.parent_count() == PARENTS,
            "168 native parents required",
        )?;
        let counts = (0..PARENTS)
            .map(|p| {
                (0..OPERATORS)
                    .map(|o| std::array::from_fn(|r| grammar.production_count(p, o, r)))
                    .collect()
            })
            .collect();
        Self::new(counts)
    }

    pub fn selections(&self) -> Vec<Selection> {
        let mut values = vec![];
        for p in 0..PARENTS {
            for o in 0..OPERATORS {
                for r in 0..ROLES {
                    for k in 0..self.counts[p][o][r] {
                        values.push(Selection {
                            parent: p,
                            operator: o,
                            role: r,
                            production_index: k,
                        });
                    }
                }
            }
        }
        values
    }

    /// Computes log P and its derivative at the actual finite f32 parameter vector.
    pub fn score(&self, theta: &[f32], selection: Selection) -> Res<ScoreReceipt> {
        check(
            theta.len() == PARAMS && theta.iter().all(|x| x.is_finite()),
            "183 finite f32 logits required",
        )?;
        let Selection {
            parent: p,
            operator: o,
            role: r,
            production_index: k,
        } = selection;
        check(
            p < PARENTS && o < OPERATORS && r < ROLES,
            "selection index outside model",
        )?;
        let count = self.counts[p][o][r];
        check(k < count, "selection outside legal production support")?;
        let os: Vec<_> = (0..OPERATORS)
            .filter(|&j| self.counts[p][j].iter().any(|&n| n > 0))
            .collect();
        let rs: Vec<_> = (0..ROLES).filter(|&j| self.counts[p][o][j] > 0).collect();
        let pp = softmax(&theta[..PARENTS], &(0..PARENTS).collect::<Vec<_>>())?;
        let po = softmax(&theta[PARENTS..PARENTS + OPERATORS], &os)?;
        let pr = softmax(&theta[PARENTS + OPERATORS..], &rs)?;
        check(
            pp[p] > 0. && po[o] > 0. && pr[r] > 0.,
            "selected probability underflows; unsupported receipt",
        )?;
        let log_probability = pp[p].ln() + po[o].ln() + pr[r].ln() - (count as f64).ln();
        let mut gradient: Vec<_> = pp.iter().chain(&po).chain(&pr).map(|&v| -v).collect();
        gradient[p] += 1.;
        gradient[PARENTS + o] += 1.;
        gradient[PARENTS + OPERATORS + r] += 1.;
        Ok(ScoreReceipt {
            profile: PROFILE.into(),
            theta: theta.to_vec(),
            selection,
            operator_support: os,
            role_support: rs,
            production_count: count,
            parent_probabilities: pp,
            operator_probabilities: po,
            role_probabilities: pr,
            log_probability,
            probability: log_probability.exp(),
            gradient,
        })
    }

    /// Checks the native emission's support/selection envelope only, NOT physics,
    /// kernel semantics, RNG replay or admission. Those remain the caller's audit.
    pub fn selection_from_emission(&self, emission: &Value) -> Res<Selection> {
        check(
            emission["grammar_profile"] == hg_sim_core::ere::valid_grammar::PROFILE
                && emission["status"] == "Applied"
                && emission["admitted"] == true
                && emission["resampled"] == false
                && emission["parent_fallback"] == false
                && emission["kernel_attempts"] == 1,
            "unsupported emission envelope",
        )?;
        let role = match emission["role"].as_str() {
            Some("Core") => 0,
            Some("Proposal") => 1,
            _ => return Err("unknown role".into()),
        };
        let s = Selection {
            parent: index(&emission["parent"])?,
            operator: index(&emission["operator"])?,
            role,
            production_index: index(&emission["production_index"])?,
        };
        let receipt = self.score(&vec![0.; PARAMS], s)?;
        check(
            emission["operator_support"] == serde_json::json!(receipt.operator_support)
                && emission["role_support"] == serde_json::json!(receipt.role_support)
                && index(&emission["production_count"])? == receipt.production_count,
            "emission support/count mismatch",
        )?;
        Ok(s)
    }

    /// Full-support engineering oracle. Values must cover every production once.
    /// This is not a native reward oracle unless every conditional outcome is known.
    pub fn expectation(&self, theta: &[f32], values: &[(Selection, f64)]) -> Res<Expectation> {
        let expected: BTreeSet<_> = self.selections().into_iter().collect();
        check(
            values.len() == expected.len()
                && values.iter().map(|v| v.0).collect::<BTreeSet<_>>() == expected,
            "full unique oracle support required",
        )?;
        let mut result = Expectation {
            value: 0.,
            gradient: vec![0.; PARAMS],
            mass: 0.,
        };
        for &(selection, value) in values {
            check(value.is_finite(), "finite oracle value required")?;
            let r = self.score(theta, selection)?;
            result.mass += r.probability;
            result.value += r.probability * value;
            for (g, s) in result.gradient.iter_mut().zip(r.gradient) {
                *g += r.probability * value * s;
            }
        }
        Ok(result)
    }
}

fn softmax(logits: &[f32], support: &[usize]) -> Res<Vec<f64>> {
    check(
        !support.is_empty()
            && support.iter().all(|&i| i < logits.len())
            && logits.iter().all(|x| x.is_finite()),
        "finite supported softmax required",
    )?;
    let max = support
        .iter()
        .map(|&i| f64::from(logits[i]))
        .fold(f64::NEG_INFINITY, f64::max);
    let mut probabilities = vec![0.; logits.len()];
    for &i in support {
        probabilities[i] = (f64::from(logits[i]) - max).exp();
    }
    let sum = probabilities.iter().sum::<f64>();
    for v in &mut probabilities {
        *v /= sum;
    }
    Ok(probabilities)
}

#[derive(Clone, Debug, Serialize)]
pub struct ScoreReceipt {
    pub profile: String,
    pub theta: Vec<f32>,
    pub selection: Selection,
    pub operator_support: Vec<usize>,
    pub role_support: Vec<usize>,
    pub production_count: usize,
    pub parent_probabilities: Vec<f64>,
    pub operator_probabilities: Vec<f64>,
    pub role_probabilities: Vec<f64>,
    pub log_probability: f64,
    pub probability: f64,
    pub gradient: Vec<f64>,
}
#[derive(Debug, Serialize)]
pub struct Expectation {
    pub value: f64,
    pub gradient: Vec<f64>,
    pub mass: f64,
}

/// One independently sampled emission with one already audited native outcome.
/// Physics repeats of one emission must not be passed as separate LOO samples.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub id: String,
    pub emission_seed: u64,
    pub physics_seed: u64,
    pub selection: Selection,
    pub native_score: Option<f64>,
    pub reason: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Contribution {
    pub sample: Sample,
    pub score: ScoreReceipt,
    pub utility: f64,
    pub baseline: f64,
    pub advantage: f64,
}
#[derive(Debug, Serialize)]
pub struct Estimate {
    pub profile: String,
    pub objective: String,
    pub baseline: String,
    pub theta_before: Vec<f32>,
    pub alpha: f32,
    pub expected_draws: usize,
    pub utility_mean: f64,
    pub contributions: Vec<Contribution>,
    pub gradient: Vec<f64>,
    /// Arithmetic proposal only; the caller chooses whether/where to apply it.
    pub theta_after: Vec<f32>,
}

/// Raw-utility likelihood-ratio gradient, with independent-emission LOO baseline.
/// No clipping, rank shaping, normalization, projection or outcome-dependent drop.
pub fn estimate(
    model: &Model,
    theta: &[f32],
    samples: &[Sample],
    expected_draws: usize,
    alpha: f32,
) -> Res<Estimate> {
    check(
        expected_draws >= 2 && samples.len() == expected_draws,
        "complete fixed sample denominator >=2 required",
    )?;
    check(
        alpha.is_finite() && alpha >= 0.,
        "finite nonnegative f32 alpha required",
    )?;
    check(
        samples.iter().all(|s| !s.id.trim().is_empty())
            && samples.iter().map(|s| &s.id).collect::<BTreeSet<_>>().len() == expected_draws,
        "unique nonempty receipt IDs required",
    )?;
    check(
        samples
            .iter()
            .map(|s| s.emission_seed)
            .collect::<BTreeSet<_>>()
            .len()
            == expected_draws,
        "one sample per distinct emission seed required",
    )?;
    let utilities = samples
        .iter()
        .map(|s| feasibility_training::utility(s.native_score, s.reason.as_deref()))
        .collect::<Res<Vec<_>>>()?;
    let mut gradient = vec![0.; PARAMS];
    let mut contributions = vec![];
    for (i, sample) in samples.iter().enumerate() {
        let score = model.score(theta, sample.selection)?;
        let baseline = utilities
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, u)| u)
            .sum::<f64>()
            / (expected_draws - 1) as f64;
        let advantage = utilities[i] - baseline;
        for (g, s) in gradient.iter_mut().zip(&score.gradient) {
            *g += advantage * s / expected_draws as f64;
        }
        contributions.push(Contribution {
            sample: sample.clone(),
            score,
            utility: utilities[i],
            baseline,
            advantage,
        });
    }
    let theta_after = theta
        .iter()
        .zip(&gradient)
        .map(|(&t, &g)| {
            let next = f64::from(t) + f64::from(alpha) * g;
            let cast = next as f32;
            if next.is_finite() && cast.is_finite() {
                Ok(cast)
            } else {
                Err("proposed f32 update overflows".into())
            }
        })
        .collect::<Res<Vec<_>>>()?;
    Ok(Estimate {
        profile: PROFILE.into(),
        objective: OBJECTIVE.into(),
        baseline: BASELINE.into(),
        theta_before: theta.to_vec(),
        alpha,
        expected_draws,
        utility_mean: utilities.iter().sum::<f64>() / expected_draws as f64,
        contributions,
        gradient,
        theta_after,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn toy() -> Model {
        let mut counts = vec![vec![[0, 0]; OPERATORS]; PARENTS];
        for (p, c) in counts.iter_mut().enumerate() {
            c[0] = [2, 1];
            c[1] = if p % 2 == 0 { [0, 1] } else { [1, 0] };
        }
        Model::new(counts).unwrap()
    }
    fn selection(p: usize, o: usize, r: usize) -> Selection {
        Selection {
            parent: p,
            operator: o,
            role: r,
            production_index: 0,
        }
    }
    fn sample(i: usize, score: Option<f64>, reason: Option<&str>) -> Sample {
        Sample {
            id: format!("receipt-{i}"),
            emission_seed: i as u64,
            physics_seed: 42,
            selection: selection(i, 0, 0),
            native_score: score,
            reason: reason.map(String::from),
        }
    }
    fn close(a: f64, b: f64, tolerance: f64) {
        assert!((a - b).abs() <= tolerance, "{a} != {b}");
    }

    #[test]
    fn masks_uniform_productions_singletons_and_normalization() {
        let model = toy();
        let t = vec![0.; PARAMS];
        let s = model.score(&t, selection(0, 0, 0)).unwrap();
        close(s.probability, 1. / 168. / 2. / 2. / 2., 1e-15);
        assert_eq!(s.operator_support, vec![0, 1]);
        assert_eq!(s.role_support, vec![0, 1]);
        assert!(s.gradient[170..181].iter().all(|&x| x == 0.));
        let s = model.score(&t, selection(0, 1, 1)).unwrap();
        assert_eq!(s.role_probabilities, vec![0., 1.]);
        assert_eq!(&s.gradient[181..], &[0., 0.]);
        let values: Vec<_> = model.selections().into_iter().map(|s| (s, 1.)).collect();
        let e = model.expectation(&t, &values).unwrap();
        close(e.mass, 1., 1e-12);
        close(e.value, 1., 1e-12);
        assert!(e.gradient.iter().all(|v| v.abs() < 1e-12));
    }

    #[test]
    fn score_all_coordinates_match_log_probability_finite_differences() {
        let m = toy();
        let t: Vec<_> = (0..PARAMS)
            .map(|i| if i % 2 == 0 { 0.25 } else { -0.25 })
            .collect();
        let s = selection(3, 0, 1);
        let receipt = m.score(&t, s).unwrap();
        for i in 0..PARAMS {
            let mut plus = t.clone();
            let mut minus = t.clone();
            plus[i] += 0.001;
            minus[i] -= 0.001;
            let fd = (m.score(&plus, s).unwrap().log_probability
                - m.score(&minus, s).unwrap().log_probability)
                / (f64::from(plus[i]) - f64::from(minus[i]));
            close(fd, receipt.gradient[i], 1e-7);
        }
    }

    #[test]
    fn full_oracle_gradient_matches_finite_differences_and_gauge() {
        let m = toy();
        let t = vec![0.; PARAMS];
        let values: Vec<_> = m
            .selections()
            .into_iter()
            .map(|s| {
                (
                    s,
                    if s.parent % 2 == 1 && s.role == 1 {
                        1.
                    } else {
                        0.
                    },
                )
            })
            .collect();
        let e = m.expectation(&t, &values).unwrap();
        for i in [0, 1, 167, 168, 169, 174, 181, 182] {
            let mut plus = t.clone();
            let mut minus = t.clone();
            plus[i] = 0.001;
            minus[i] = -0.001;
            let fd = (m.expectation(&plus, &values).unwrap().value
                - m.expectation(&minus, &values).unwrap().value)
                / (f64::from(plus[i]) - f64::from(minus[i]));
            close(fd, e.gradient[i], 1e-7);
        }
        let mut shifted = t.clone();
        for x in &mut shifted[..168] {
            *x += 2.;
        }
        for x in &mut shifted[168..181] {
            *x -= 3.;
        }
        for x in &mut shifted[181..] {
            *x += 4.;
        }
        let s = selection(0, 0, 1);
        let a = m.score(&t, s).unwrap();
        let b = m.score(&shifted, s).unwrap();
        close(a.probability, b.probability, 1e-15);
        for (x, y) in a.gradient.iter().zip(b.gradient) {
            close(*x, y, 1e-15);
        }
        for (start, end) in [(0, 168), (168, 181), (181, 183)] {
            close(e.gradient[start..end].iter().sum(), 0., 1e-12);
        }
    }

    #[test]
    fn loo_exact_pair_expectation_is_unbiased_not_self_centered() {
        // Two equiprobable parent reward classes. Enumerate the four iid pair
        // cases: LOO expectation equals the exact gradient (self-mean is half).
        let m = toy();
        let t = vec![0.; PARAMS];
        let mut expected = vec![0.; PARAMS];
        for a in 0..168 {
            for b in 0..168 {
                let mut sa = sample(0, Some(if a % 2 == 1 { 1. } else { 0. }), None);
                sa.selection.parent = a;
                let mut sb = sample(1, Some(if b % 2 == 1 { 1. } else { 0. }), None);
                sb.selection.parent = b;
                let e = estimate(&m, &t, &[sa, sb], 2, 0.).unwrap();
                for (g, x) in expected.iter_mut().zip(e.gradient) {
                    *g += x / (168. * 168.);
                }
            }
        }
        for (i, &g) in expected.iter().take(168).enumerate() {
            close(g, if i % 2 == 1 { 0.5 / 168. } else { -0.5 / 168. }, 1e-12);
        }
        assert!(expected[168..].iter().all(|x| x.abs() < 1e-12));
    }

    #[test]
    fn native_caps_kept_sign_and_exact_single_cast_update() {
        let m = toy();
        let t = vec![0.; PARAMS];
        let samples = [
            sample(0, None, Some("resource_cap")),
            sample(1, Some(1.), None),
        ];
        let e = estimate(&m, &t, &samples, 2, 0.05).unwrap();
        assert_eq!(e.utility_mean, 1.);
        assert_eq!(e.contributions[0].sample.native_score, None);
        assert_eq!(e.contributions[0].utility, 0.);
        assert_eq!(e.contributions[0].baseline, 2.);
        assert_eq!(e.contributions[1].baseline, 0.);
        assert!(e.gradient[0] < 0. && e.gradient[1] > 0.);
        for i in 0..PARAMS {
            assert_eq!(
                e.theta_after[i],
                (f64::from(t[i]) + f64::from(0.05f32) * e.gradient[i]) as f32
            );
        }
        let equal = [
            sample(0, None, Some("resource_cap")),
            sample(1, None, Some("resource_cap")),
        ];
        let z = estimate(&m, &t, &equal, 2, 0.05).unwrap();
        assert!(z.gradient.iter().all(|&v| v == 0.));
        assert_eq!(z.theta_after, t);
        let equal = [sample(0, Some(0.), None), sample(1, Some(0.), None)];
        assert_eq!(
            estimate(&m, &t, &equal, 2, 0.05).unwrap().gradient,
            vec![0.; PARAMS]
        );
    }

    #[test]
    fn refuse_missing_duplicate_correlated_and_unsupported_outcomes() {
        let m = toy();
        let t = vec![0.; PARAMS];
        let good = vec![sample(0, Some(0.), None), sample(1, Some(1.), None)];
        assert!(estimate(&m, &t, &good, 3, 0.05).is_err());
        assert!(estimate(&m, &t, &good[..1], 1, 0.05).is_err());
        for (score, reason) in [
            (None, None),
            (None, Some("crash")),
            (Some(1.), Some("resource_cap")),
            (Some(f64::NAN), None),
            (Some(1.1), None),
        ] {
            let mut bad = good.clone();
            bad[0] = sample(0, score, reason);
            assert!(estimate(&m, &t, &bad, 2, 0.05).is_err());
        }
        let mut bad = good.clone();
        bad[1].id = bad[0].id.clone();
        assert!(estimate(&m, &t, &bad, 2, 0.05).is_err());
        let mut bad = good.clone();
        bad[1].emission_seed = bad[0].emission_seed;
        assert!(estimate(&m, &t, &bad, 2, 0.05).is_err());
        assert!(estimate(&m, &t, &good, 2, -0.1).is_err());
        assert!(estimate(&m, &t, &good, 2, f32::INFINITY).is_err());
    }

    #[test]
    fn invalid_logits_support_extremes_and_oracle_coverage_refuse() {
        let m = toy();
        let s = selection(0, 0, 0);
        assert!(m.score(&[0.; 3], s).is_err());
        let mut t = vec![0.; PARAMS];
        t[0] = f32::NAN;
        assert!(m.score(&t, s).is_err());
        assert!(m.score(&vec![0.; PARAMS], selection(0, 2, 0)).is_err());
        assert!(m.score(&vec![0.; PARAMS], selection(0, 1, 0)).is_err());
        t[0] = f32::MAX;
        let r = m.score(&t, s).unwrap();
        assert!(r.log_probability.is_finite());
        assert!(m.score(&t, selection(1, 0, 0)).is_err());
        let mut values: Vec<_> = m.selections().into_iter().map(|s| (s, 1.)).collect();
        values.pop();
        assert!(m.expectation(&vec![0.; PARAMS], &values).is_err());
        values.push(values[0]);
        assert!(m.expectation(&vec![0.; PARAMS], &values).is_err());
    }

    #[test]
    fn catalogue_and_emission_envelopes_refuse_inconsistent_support() {
        let m = toy();
        let parents: Vec<_> = m.counts.iter().enumerate().map(|(p, counts)| {
            let groups: Vec<_> = counts.iter().enumerate().map(|(o, counts)| {
                let roles: Vec<_> = counts.iter().enumerate().map(|(r, &n)| {
                    serde_json::json!({"role":r,"productions":(0..n).map(|k|serde_json::json!({"index":k})).collect::<Vec<_>>()})
                }).collect();
                serde_json::json!({"operator":o,"roles":roles,"role_support":(0..2).filter(|&r|counts[r]>0).collect::<Vec<_>>()})
            }).collect();
            serde_json::json!({"parent":p,"groups":groups,"operator_support":[0,1]})
        }).collect();
        let cat = serde_json::json!({"profile":hg_sim_core::ere::valid_grammar::PROFILE,"params":183,"engine_steps":0,"parents":parents});
        assert_eq!(
            Model::from_catalogue(&cat).unwrap().selections(),
            m.selections()
        );
        let mut bad = cat.clone();
        bad["parents"][0]["groups"][0]["role_support"] = serde_json::json!([1, 0]);
        assert!(Model::from_catalogue(&bad).is_err());
        let mut bad = cat.clone();
        bad["parents"][0]["groups"][0]["roles"][0]["productions"][1]["index"] =
            serde_json::json!(0);
        assert!(Model::from_catalogue(&bad).is_err());
        let e = serde_json::json!({"grammar_profile":hg_sim_core::ere::valid_grammar::PROFILE,"status":"Applied","admitted":true,
            "resampled":false,"parent_fallback":false,"kernel_attempts":1,"role":"Core","parent":0,"operator":0,
            "production_index":1,"production_count":2,"operator_support":[0,1],"role_support":[0,1]});
        assert_eq!(m.selection_from_emission(&e).unwrap().production_index, 1);
        let mut bad = e.clone();
        bad["role_support"] = serde_json::json!([0]);
        assert!(m.selection_from_emission(&bad).is_err());
        let mut bad = e.clone();
        bad["production_count"] = serde_json::json!(3);
        assert!(m.selection_from_emission(&bad).is_err());
        let mut bad = e.clone();
        bad["resampled"] = serde_json::json!(true);
        assert!(m.selection_from_emission(&bad).is_err());
    }

    #[test]
    fn overflowing_update_refuses_and_production_has_no_score_coordinate() {
        let m = toy();
        let t = vec![f32::MAX; PARAMS];
        let samples = [
            sample(0, None, Some("resource_cap")),
            sample(1, Some(1.), None),
        ];
        assert!(estimate(&m, &t, &samples, 2, f32::MAX).is_err());
        let t = vec![0.1; PARAMS];
        let first = selection(0, 0, 0);
        let mut second = first;
        second.production_index = 1;
        let a = m.score(&t, first).unwrap();
        let b = m.score(&t, second).unwrap();
        assert_eq!(a.gradient, b.gradient);
        assert_eq!(a.probability, b.probability);
    }
}
