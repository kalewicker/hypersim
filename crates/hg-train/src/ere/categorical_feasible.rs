//! Separate risk-panel, estimation-only finite proposal. No rollout or admission.
//! Observed-cap protection is not a certificate for unseen leaves or native worlds.
use super::{categorical as c, complete_admission::Res};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const PROFILE: &str = "E2-categorical-feasibility-proposal-v2";
const N: usize = 183;
const RADIUS: f64 = 0.1;
const MARGIN: f64 = 0.1;
const MIN_GRADIENT_NORM: f64 = 1e-12;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub stage: String,
    pub expected_draws: usize,
    pub samples: Vec<c::Sample>,
}

fn check(ok: bool, message: &str) -> Res<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).fold(0., |total, (x, y)| total + x * y)
}
fn norm(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}
fn center(a: &[f64]) -> Vec<f64> {
    let mut result = a.to_vec();
    for (start, end) in [(0, 168), (168, 181), (181, 183)] {
        let mean = a[start..end].iter().sum::<f64>() / (end - start) as f64;
        for value in &mut result[start..end] {
            *value -= mean;
        }
    }
    result
}

/// Unit direction maximizing the estimated utility linear objective in a cone.
/// The cone requires at least MARGIN cosine with the validity gradient. A
/// conflicting/zero utility objective holds base rather than forcing damage.
fn cone(utility: &[f64], validity: &[f64]) -> Option<Vec<f64>> {
    let un = norm(utility);
    let vn = norm(validity);
    if un <= MIN_GRADIENT_NORM || vn <= MIN_GRADIENT_NORM {
        return None;
    }
    let u: Vec<_> = utility.iter().map(|x| x / un).collect();
    let v: Vec<_> = validity.iter().map(|x| x / vn).collect();
    let alignment = dot(&u, &v);
    let candidate = if alignment >= MARGIN {
        u
    } else {
        let perpendicular: Vec<_> = u.iter().zip(&v).map(|(x, y)| x - alignment * y).collect();
        let pn = norm(&perpendicular);
        if pn <= MIN_GRADIENT_NORM {
            return None;
        }
        let tangent = (1. - MARGIN * MARGIN).sqrt();
        perpendicular
            .iter()
            .zip(&v)
            .map(|(x, y)| tangent * x / pn + MARGIN * y)
            .collect()
    };
    if dot(utility, &candidate) <= 0. {
        None
    } else {
        Some(candidate)
    }
}

pub fn propose(model: &c::Model, theta: &[f32], utility: &Panel, risk: &Panel) -> Res<Value> {
    check(
        theta.len() == N && theta.iter().all(|x| x.is_finite() && x.abs() <= 8.),
        "183 finite base logits within +/-8 required; never clipped",
    )?;
    check(
        utility.stage == "utility_estimation" && risk.stage == "risk_estimation",
        "separate estimation roles required; validation is forbidden",
    )?;
    let mut ids = BTreeSet::new();
    let mut seeds = BTreeSet::new();
    for panel in [utility, risk] {
        for sample in &panel.samples {
            check(ids.insert(&sample.id), "duplicate/cross-panel receipt ID")?;
            check(
                seeds.insert(sample.emission_seed) && seeds.insert(sample.physics_seed),
                "distinct joint seed domains within/across panels required",
            )?;
        }
    }
    // Existing utility/null validation and score calculus are not changed.
    let ur = c::estimate(model, theta, &utility.samples, utility.expected_draws, 0.05)?;
    let rr = c::estimate(model, theta, &risk.samples, risk.expected_draws, 0.)?;
    let values: Vec<f64> = risk
        .samples
        .iter()
        .map(|s| if s.native_score.is_some() { 1. } else { 0. })
        .collect();
    let caps = values.iter().filter(|&&v| v == 0.).count();
    let mut vg = vec![0.; N];
    let mut baselines = vec![];
    for (i, contribution) in rr.contributions.iter().enumerate() {
        // Fixed-one control variate exactly matches the derivative of the
        // ordinary cap-weight guard: g_V = -mean_cap(score), not a noisy
        // sample-centered direction inconsistent with that finite estimand.
        let baseline = 1.;
        baselines.push(baseline);
        for (g, score) in vg.iter_mut().zip(&contribution.score.gradient) {
            *g += (values[i] - baseline) * score / risk.expected_draws as f64;
        }
    }
    let ug = center(&ur.gradient);
    let vg = center(&vg);
    check(
        ug.iter().chain(&vg).all(|x| x.is_finite()),
        "finite centered gradients required",
    )?;
    let (direction, hold_reason) = if caps == 0 {
        (None, Some("no_observed_caps"))
    } else if caps == risk.expected_draws || norm(&vg) <= MIN_GRADIENT_NORM {
        (None, Some("no_informative_validity_gradient"))
    } else if norm(&ug) <= MIN_GRADIENT_NORM {
        (None, Some("zero_utility_gradient"))
    } else {
        let d = cone(&ug, &vg);
        let reason = if d.is_none() {
            Some("utility_validity_conflict")
        } else {
            None
        };
        (d, reason)
    };
    let mut candidates = vec![];
    let mut selected: Option<usize> = None;
    let mut after = theta.to_vec();
    if let Some(d) = &direction {
        // Fixed engineering ladder; evaluate all nine, never fit on validation.
        for index in 0..9 {
            let radius = RADIUS / (1u64 << index) as f64;
            let proposal: Vec<f32> = theta
                .iter()
                .zip(d)
                .map(|(&t, &x)| (f64::from(t) + radius * x) as f32)
                .collect();
            check(
                proposal.iter().all(|x| x.is_finite()),
                "f32 proposal overflow",
            )?;
            let delta: Vec<f64> = proposal
                .iter()
                .zip(theta)
                .map(|(&x, &t)| f64::from(x) - f64::from(t))
                .collect();
            let actual_norm = norm(&delta);
            let maximum = delta.iter().map(|x| x.abs()).fold(0., f64::max);
            let utility_gain = dot(&ug, &delta);
            let validity_gain = dot(&vg, &delta);
            let mut weights = vec![];
            let mut cap_effect = 0.;
            for (sample, contribution) in risk.samples.iter().zip(&rr.contributions) {
                let logp = model.score(&proposal, sample.selection)?.log_probability;
                let weight = (logp - contribution.score.log_probability).exp();
                check(
                    weight.is_finite() && weight > 0.,
                    "finite positive risk weight required",
                )?;
                weights.push(weight);
                if sample.native_score.is_none() {
                    cap_effect += (weight - 1.) / risk.expected_draws as f64;
                }
            }
            check(
                cap_effect.is_finite(),
                "finite observed cap-risk effect required",
            )?;
            let passes = actual_norm > 0.
                && actual_norm <= 0.100001
                && maximum <= 0.100001
                && utility_gain > 0.
                && validity_gain >= MARGIN * norm(&vg) * actual_norm - 1e-12
                && cap_effect <= 0.;
            if passes && selected.is_none() {
                selected = Some(index);
                after = proposal.clone();
            }
            candidates.push(
                json!({"index":index,"requested_radius":radius,"theta_after":proposal,
                "actual_norm":actual_norm,"maximum_coordinate_step":maximum,
                "utility_linear_gain":utility_gain,"validity_linear_gain":validity_gain,
                "observed_cap_risk_effect":cap_effect,"risk_weights":weights,"passes":passes}),
            );
        }
    }
    Ok(
        json!({"profile":PROFILE,"classification":"engineering_arithmetic_only",
        "utility_input":ur,"risk_input_validation":rr,"risk_validity_values":values,"risk_fixed_baselines":baselines,
        "centered_utility_gradient":ug,"centered_validity_gradient":vg,"risk_cap_count":caps,
        "direction":direction,"cone_margin":MARGIN,"minimum_gradient_norm":MIN_GRADIENT_NORM,"maximum_radius":RADIUS,"candidates":candidates,
        "selected_candidate":selected,"theta_after":after,
        "hold_reason":if selected.is_some() {None} else {hold_reason.or(Some("no_finite_candidate_passes"))},
        "observed_cap_guard_only":true,"unseen_cap_feasibility_certificate":false,
        "validation_used":false,"native_steps_run":0,"grammar_compilations":0,"training_updates":0,"training_admitted":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cone_keeps_safe_direction() {
        assert_eq!(cone(&[1., 0.], &[1., 0.]).unwrap(), vec![1., 0.]);
    }
    #[test]
    fn cone_moves_boundary_inward() {
        let d = cone(&[0., 1.], &[1., 0.]).unwrap();
        assert!((norm(&d) - 1.).abs() < 1e-14 && (d[0] - MARGIN).abs() < 1e-14 && d[1] > 0.);
    }
    #[test]
    fn cone_refuses_pure_conflict_and_degeneracy() {
        assert!(cone(&[-1., 0.], &[1., 0.]).is_none());
        assert!(cone(&[0., 0.], &[1., 0.]).is_none());
        assert!(cone(&[1., 0.], &[0., 0.]).is_none());
    }
    #[test]
    fn center_preserves_only_group_gauges() {
        let d = center(&vec![2.; N]);
        assert!(d.iter().all(|x| *x == 0.));
    }
    #[test]
    fn centering_dot_and_radius_operations_are_ordered() {
        assert_eq!(dot(&[1e16, 1., -1e16], &[1., 1., 1.]), 0.);
        for i in 0..9 {
            assert!(RADIUS / (1u64 << i) as f64 <= RADIUS);
        }
    }
}
