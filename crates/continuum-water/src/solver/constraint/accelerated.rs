#![forbid(unsafe_code)]

use super::*;

const PROJECTED_GRADIENT_STEP: f64 = f64::from_bits(0x3fd0_0000_0000_0000);
const MAXIMUM_DIRECTIONAL_CURVATURE: f64 = 4.0;

pub(in crate::solver) fn solve_density_accelerated_projected_gradient(
    reconstruction: &Reconstruction,
    velocities: &mut [Vec3f],
    maximum_iterations: u8,
) -> Result<SolveResult, WaterError> {
    let count = velocities.len();
    let mut right_hand_side = filled_vec(count, 0.0)?;
    let mut scale = filled_vec(count, 0.0)?;
    let mut enabled = filled_vec(count, false)?;
    for index in 0..count {
        let delta = divergence_source(
            index,
            reconstruction.rows[index],
            reconstruction,
            velocities,
        )?;
        let rho_adv = checked_scalar(
            reconstruction.rho_ratio[index] + (DT * delta),
            "accelerated projected-gradient advected density ratio",
        )?;
        let inverse_diagonal = checked_scalar(
            reconstruction.alpha[index] * INV_DT2,
            "accelerated projected-gradient inverse diagonal",
        )?;
        (enabled[index], scale[index]) = coordinate_scale(inverse_diagonal)?;
        right_hand_side[index] = checked_scalar(
            scale[index] * (rho_adv - 1.0),
            "accelerated projected-gradient scaled right-hand side",
        )?;
    }

    let mut previous = filled_vec(count, 0.0)?;
    let mut iterate = filled_vec(count, 0.0)?;
    let mut previous_gradient = filled_vec(count, 0.0)?;
    for index in 0..count {
        previous_gradient[index] = -right_hand_side[index];
    }
    let mut gradient = previous_gradient.clone();
    let mut error_ppb = scaled_density_residual_ppb(&gradient, &scale)?;
    let mut kkt_error_ppb = i64::MAX;
    let mut accepted_iteration = 0_u8;

    for iteration in 1..=maximum_iterations {
        let momentum = checked_scalar(
            f64::from(iteration - 1) / f64::from(iteration + 2),
            "accelerated projected-gradient momentum",
        )?;
        let mut next = filled_vec(count, 0.0)?;
        for index in 0..count {
            if !enabled[index] {
                continue;
            }
            let extrapolated = checked_scalar(
                iterate[index] + (momentum * (iterate[index] - previous[index])),
                "accelerated projected-gradient extrapolation",
            )?;
            let extrapolated_gradient = checked_scalar(
                gradient[index] + (momentum * (gradient[index] - previous_gradient[index])),
                "accelerated projected-gradient extrapolated gradient",
            )?;
            next[index] = checked_scalar(
                extrapolated - (PROJECTED_GRADIENT_STEP * extrapolated_gradient),
                "accelerated projected-gradient candidate",
            )?
            .max(0.0);
        }
        let next_gradient = exact_scaled_gradient(reconstruction, &scale, &next, &right_hand_side)?;
        error_ppb = scaled_density_residual_ppb(&next_gradient, &scale)?;
        kkt_error_ppb = scaled_kkt_residual_ppb(&next, &next_gradient, &scale)?;
        let directional_curvature = directional_curvature(
            &previous,
            &iterate,
            &previous_gradient,
            &gradient,
            &next,
            &next_gradient,
            momentum,
        )?;
        validate_directional_curvature(directional_curvature)?;
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
                "accelerated projected-gradient iteration {maximum_iterations} ended at density {error_ppb} ppb and projected KKT {kkt_error_ppb} ppb"
            ),
        ));
    }
    let mut multiplier = filled_vec(count, 0.0)?;
    for index in 0..count {
        multiplier[index] = checked_scalar(
            scale[index] * iterate[index],
            "accelerated projected-gradient pressure multiplier",
        )?;
    }
    let acceleration = pressure_acceleration(reconstruction, &multiplier)?;
    let boundary_impulse = apply_acceleration(velocities, &acceleration)?;
    let maximum_multiplier_bits = maximum_multiplier_bits(&multiplier)?;
    Ok(SolveResult {
        iterations: accepted_iteration,
        error_ppb,
        kkt_error_ppb: Some(kkt_error_ppb),
        maximum_multiplier_bits,
        boundary_impulse,
    })
}

fn coordinate_scale(inverse_diagonal: f64) -> Result<(bool, f64), WaterError> {
    if inverse_diagonal > 0.0 {
        Ok((
            true,
            checked_scalar(
                inverse_diagonal.sqrt(),
                "accelerated projected-gradient diagonal scale",
            )?,
        ))
    } else if inverse_diagonal == 0.0 {
        Ok((false, 1.0))
    } else {
        Err(WaterError::new(
            DENSITY_NONCONVERGENCE,
            "accelerated projected-gradient inverse diagonal is negative",
        ))
    }
}

fn validate_directional_curvature(curvature: f64) -> Result<(), WaterError> {
    if curvature <= MAXIMUM_DIRECTIONAL_CURVATURE {
        Ok(())
    } else {
        Err(WaterError::new(
            DENSITY_NONCONVERGENCE,
            format!(
                "accelerated projected-gradient step is not a local quadratic majorizer: curvature bits 0x{:016x}",
                curvature.to_bits(),
            ),
        ))
    }
}

fn exact_scaled_gradient(
    reconstruction: &Reconstruction,
    scale: &[f64],
    iterate: &[f64],
    right_hand_side: &[f64],
) -> Result<Vec<f64>, WaterError> {
    let mut result = scaled_pressure_operator(reconstruction, scale, iterate)?;
    for index in 0..result.len() {
        result[index] = checked_scalar(
            result[index] - right_hand_side[index],
            "accelerated projected-gradient exact gradient",
        )?;
    }
    Ok(result)
}

fn scaled_pressure_operator(
    reconstruction: &Reconstruction,
    scale: &[f64],
    vector: &[f64],
) -> Result<Vec<f64>, WaterError> {
    let mut unscaled = filled_vec(vector.len(), 0.0)?;
    for index in 0..vector.len() {
        unscaled[index] = checked_scalar(
            scale[index] * vector[index],
            "accelerated projected-gradient unscale",
        )?;
    }
    let mut result = density_pressure_operator(reconstruction, &unscaled)?;
    for index in 0..result.len() {
        result[index] = checked_scalar(
            scale[index] * result[index],
            "accelerated projected-gradient scale",
        )?;
    }
    Ok(result)
}

fn scaled_density_residual_ppb(gradient: &[f64], scale: &[f64]) -> Result<i64, WaterError> {
    let mut residual = filled_vec(gradient.len(), 0.0)?;
    for index in 0..gradient.len() {
        residual[index] = checked_scalar(
            -gradient[index] / scale[index],
            "accelerated projected-gradient residual",
        )?;
    }
    density_residual_ppb(&residual)
}

fn scaled_kkt_residual_ppb(
    iterate: &[f64],
    gradient: &[f64],
    scale: &[f64],
) -> Result<i64, WaterError> {
    let mut sum = 0.0;
    for index in 0..gradient.len() {
        let original_gradient = checked_scalar(
            gradient[index] / scale[index],
            "accelerated projected-gradient KKT residual",
        )?;
        let projected = if iterate[index] > 0.0 {
            original_gradient.abs()
        } else {
            (-original_gradient).max(0.0)
        };
        sum = checked_scalar(
            sum + projected,
            "accelerated projected-gradient KKT residual reduction",
        )?;
    }
    quantize_ppb(checked_scalar(
        sum / (gradient.len() as f64),
        "accelerated projected-gradient mean KKT residual",
    )?)
}

#[allow(clippy::too_many_arguments)]
fn directional_curvature(
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
        let extrapolated = checked_scalar(
            iterate[index] + (momentum * (iterate[index] - previous[index])),
            "accelerated projected-gradient curvature extrapolation",
        )?;
        let extrapolated_gradient = checked_scalar(
            gradient[index] + (momentum * (gradient[index] - previous_gradient[index])),
            "accelerated projected-gradient curvature extrapolated gradient",
        )?;
        let displacement = checked_scalar(
            next[index] - extrapolated,
            "accelerated projected-gradient curvature displacement",
        )?;
        let action = checked_scalar(
            next_gradient[index] - extrapolated_gradient,
            "accelerated projected-gradient curvature action",
        )?;
        displacement_squared = checked_scalar(
            displacement_squared + (displacement * displacement),
            "accelerated projected-gradient curvature norm reduction",
        )?;
        displacement_action = checked_scalar(
            displacement_action + (displacement * action),
            "accelerated projected-gradient curvature action reduction",
        )?;
    }
    if displacement_squared <= SOLVER_EPSILON {
        return Ok(0.0);
    }
    checked_scalar(
        displacement_action / displacement_squared,
        "accelerated projected-gradient directional curvature",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_inverse_diagonal_freezes_multiplier_but_retains_residuals() {
        assert_eq!(coordinate_scale(0.0).unwrap(), (false, 1.0));
        assert_eq!(
            coordinate_scale(-1.0).unwrap_err().code(),
            DENSITY_NONCONVERGENCE
        );
        let gradient = [-0.25];
        let scale = [1.0];
        let iterate = [0.0];
        assert_eq!(
            scaled_density_residual_ppb(&gradient, &scale).unwrap(),
            250_000_000
        );
        assert_eq!(
            scaled_kkt_residual_ppb(&iterate, &gradient, &scale).unwrap(),
            250_000_000
        );
    }

    #[test]
    fn step_and_majorization_limit_are_exact_profile_values() {
        assert_eq!(PROJECTED_GRADIENT_STEP.to_bits(), 0x3fd0_0000_0000_0000);
        assert!(validate_directional_curvature(4.0).is_ok());
        assert_eq!(
            validate_directional_curvature(f64::from_bits(4.0_f64.to_bits() + 1))
                .unwrap_err()
                .code(),
            DENSITY_NONCONVERGENCE
        );
    }
}
