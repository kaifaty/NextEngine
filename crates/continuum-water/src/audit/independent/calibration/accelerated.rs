#![forbid(unsafe_code)]

use super::*;

pub(crate) fn compute_support_complete_accelerated_pressure_first_step_probe()
-> Result<AcceleratedPressureFirstStepProbe, WaterError> {
    const STEP: f64 = f64::from_bits(0x3fd0_0000_0000_0000);
    const MAXIMUM_ITERATIONS: u8 = 50;

    let positions = generate_fluid()?;
    let boundary_positions = generate_support_complete_boundary()?;
    let boundary_volumes = filled(boundary_positions.len(), REST_VOLUME)?;
    let rows = reconstruct(&positions, &boundary_positions, &boundary_volumes)?;
    let count = positions.len();
    let gravity_y = finite(DT * -GRAVITY, "independent accelerated gravity velocity")?;
    let velocities = filled(count, F3::new(0.0, gravity_y, 0.0))?;
    let mut right_hand_side = filled(count, 0.0)?;
    let mut scale = filled(count, 0.0)?;
    let mut enabled = filled(count, false)?;
    for index in 0..count {
        let delta = divergence_source(index, &rows, &velocities)?;
        let rho_adv = finite(
            rows[index].rho_ratio + (DT * delta),
            "independent accelerated advected density ratio",
        )?;
        let inverse_diagonal = finite(
            rows[index].alpha * INV_DT2,
            "independent accelerated inverse diagonal",
        )?;
        enabled[index] = inverse_diagonal > 0.0;
        scale[index] = if enabled[index] {
            finite(
                inverse_diagonal.sqrt(),
                "independent accelerated diagonal scale",
            )?
        } else {
            1.0
        };
        right_hand_side[index] = finite(
            scale[index] * (rho_adv - 1.0),
            "independent accelerated scaled right-hand side",
        )?;
    }

    let mut previous = filled(count, 0.0)?;
    let mut iterate = filled(count, 0.0)?;
    let mut previous_gradient = filled(count, 0.0)?;
    for index in 0..count {
        previous_gradient[index] = -right_hand_side[index];
    }
    let mut gradient = previous_gradient.clone();
    let mut error_ppb = accelerated_density_residual_ppb(&gradient, &scale)?;
    let mut kkt_error_ppb = i64::MAX;
    let mut accepted_iteration = 0_u8;
    for iteration in 1..=MAXIMUM_ITERATIONS {
        let momentum = finite(
            f64::from(iteration - 1) / f64::from(iteration + 2),
            "independent accelerated momentum",
        )?;
        let mut next = filled(count, 0.0)?;
        for index in 0..count {
            if !enabled[index] {
                continue;
            }
            let extrapolated = finite(
                iterate[index] + (momentum * (iterate[index] - previous[index])),
                "independent accelerated extrapolation",
            )?;
            let extrapolated_gradient = finite(
                gradient[index] + (momentum * (gradient[index] - previous_gradient[index])),
                "independent accelerated extrapolated gradient",
            )?;
            next[index] = finite(
                extrapolated - (STEP * extrapolated_gradient),
                "independent accelerated candidate",
            )?
            .max(0.0);
        }
        let next_gradient =
            independent_accelerated_gradient(&rows, &scale, &next, &right_hand_side)?;
        error_ppb = accelerated_density_residual_ppb(&next_gradient, &scale)?;
        kkt_error_ppb = accelerated_kkt_residual_ppb(&next, &next_gradient, &scale)?;
        let curvature = independent_accelerated_curvature(
            &previous,
            &iterate,
            &previous_gradient,
            &gradient,
            &next,
            &next_gradient,
            momentum,
        )?;
        if curvature > 1.0 / STEP {
            return Err(WaterError::new(
                DENSITY_NONCONVERGENCE,
                "independent accelerated step failed the majorization guard",
            ));
        }
        previous = iterate;
        iterate = next;
        previous_gradient = gradient;
        gradient = next_gradient;
        if iteration >= DENSITY_MIN_ITERATIONS
            && error_ppb <= DENSITY_THRESHOLD_PPB
            && kkt_error_ppb <= DENSITY_THRESHOLD_PPB
        {
            accepted_iteration = iteration;
            break;
        }
    }
    if accepted_iteration == 0 {
        return Err(WaterError::new(
            DENSITY_NONCONVERGENCE,
            format!(
                "independent accelerated iteration {MAXIMUM_ITERATIONS} ended at density {error_ppb} ppb and KKT {kkt_error_ppb} ppb"
            ),
        ));
    }
    let mut maximum_multiplier = 0.0_f64;
    for index in 0..count {
        let multiplier = finite(
            scale[index] * iterate[index],
            "independent accelerated multiplier",
        )?;
        maximum_multiplier = maximum_multiplier.max(multiplier);
    }
    Ok(AcceleratedPressureFirstStepProbe {
        density_iterations: accepted_iteration,
        density_error_ppb: error_ppb,
        density_kkt_error_ppb: kkt_error_ppb,
        maximum_multiplier_bits: scalar_bits(maximum_multiplier),
    })
}

fn independent_accelerated_gradient(
    rows: &[Row],
    scale: &[f64],
    iterate: &[f64],
    right_hand_side: &[f64],
) -> Result<Vec<f64>, WaterError> {
    let mut multiplier = filled(iterate.len(), 0.0)?;
    for index in 0..iterate.len() {
        multiplier[index] = finite(
            scale[index] * iterate[index],
            "independent accelerated operator unscale",
        )?;
    }
    let mut result = pressure_operator(rows, &multiplier)?;
    for index in 0..result.len() {
        result[index] = finite(
            (scale[index] * result[index]) - right_hand_side[index],
            "independent accelerated exact gradient",
        )?;
    }
    Ok(result)
}

fn accelerated_density_residual_ppb(gradient: &[f64], scale: &[f64]) -> Result<i64, WaterError> {
    let mut residual = filled(gradient.len(), 0.0)?;
    for index in 0..gradient.len() {
        residual[index] = finite(
            -gradient[index] / scale[index],
            "independent accelerated density residual",
        )?;
    }
    pressure_density_residual_ppb(&residual)
}

fn accelerated_kkt_residual_ppb(
    iterate: &[f64],
    gradient: &[f64],
    scale: &[f64],
) -> Result<i64, WaterError> {
    let mut sum = 0.0;
    for index in 0..gradient.len() {
        let original_gradient = finite(
            gradient[index] / scale[index],
            "independent accelerated KKT residual",
        )?;
        let projected = if iterate[index] > 0.0 {
            original_gradient.abs()
        } else {
            (-original_gradient).max(0.0)
        };
        sum = finite(sum + projected, "independent accelerated KKT reduction")?;
    }
    quantize_ppb(finite(
        sum / (gradient.len() as f64),
        "independent accelerated mean KKT residual",
    )?)
}

#[allow(clippy::too_many_arguments)]
fn independent_accelerated_curvature(
    previous: &[f64],
    iterate: &[f64],
    previous_gradient: &[f64],
    gradient: &[f64],
    next: &[f64],
    next_gradient: &[f64],
    momentum: f64,
) -> Result<f64, WaterError> {
    let mut displacement_squared = 0.0;
    let mut displacement_action = 0.0;
    for index in 0..iterate.len() {
        let extrapolated = finite(
            iterate[index] + (momentum * (iterate[index] - previous[index])),
            "independent accelerated curvature extrapolation",
        )?;
        let extrapolated_gradient = finite(
            gradient[index] + (momentum * (gradient[index] - previous_gradient[index])),
            "independent accelerated curvature extrapolated gradient",
        )?;
        let displacement = finite(
            next[index] - extrapolated,
            "independent accelerated curvature displacement",
        )?;
        let action = finite(
            next_gradient[index] - extrapolated_gradient,
            "independent accelerated curvature action",
        )?;
        displacement_squared = finite(
            displacement_squared + (displacement * displacement),
            "independent accelerated curvature norm reduction",
        )?;
        displacement_action = finite(
            displacement_action + (displacement * action),
            "independent accelerated curvature action reduction",
        )?;
    }
    if displacement_squared <= SOLVER_EPSILON {
        return Ok(0.0);
    }
    finite(
        displacement_action / displacement_squared,
        "independent accelerated directional curvature",
    )
}
