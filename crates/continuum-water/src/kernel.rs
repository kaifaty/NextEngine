#![forbid(unsafe_code)]

use crate::error::WaterError;
use crate::model::{Vec3f, Vec3i, checked_scalar};
use crate::profile::{KERNEL_K, KERNEL_L, MICROMETRES_PER_METRE, SUPPORT_RADIUS};

#[derive(Clone, Copy, Debug)]
pub(crate) struct KernelSample {
    pub(crate) value: f64,
    pub(crate) gradient: Vec3f,
}

pub(crate) fn sample(displacement_um: Vec3i) -> Result<KernelSample, WaterError> {
    let dx = (displacement_um.x as f64) / MICROMETRES_PER_METRE;
    let dy = (displacement_um.y as f64) / MICROMETRES_PER_METRE;
    let dz = (displacement_um.z as f64) / MICROMETRES_PER_METRE;
    sample_components(dx, dy, dz, displacement_um.squared_length_i128()? == 0)
}

pub(crate) fn sample_metres(displacement: Vec3f) -> Result<KernelSample, WaterError> {
    sample_components(
        displacement.x,
        displacement.y,
        displacement.z,
        displacement.x == 0.0 && displacement.y == 0.0 && displacement.z == 0.0,
    )
}

fn sample_components(
    dx: f64,
    dy: f64,
    dz: f64,
    displacement_is_zero: bool,
) -> Result<KernelSample, WaterError> {
    let r2_xy = (dx * dx) + (dy * dy);
    let r2 = checked_scalar(r2_xy + (dz * dz), "kernel r2")?;
    let r = checked_scalar(r2.sqrt(), "kernel r")?;
    let q = checked_scalar(r / SUPPORT_RADIUS, "kernel q")?;
    if q > 1.0 {
        return Ok(KernelSample {
            value: 0.0,
            gradient: Vec3f::ZERO,
        });
    }

    let value = if q <= 0.5 {
        let q2 = checked_scalar(q * q, "kernel q2")?;
        let q3 = checked_scalar(q2 * q, "kernel q3")?;
        let six_q3 = checked_scalar(6.0 * q3, "kernel six q3")?;
        let six_q2 = checked_scalar(6.0 * q2, "kernel six q2")?;
        let polynomial = checked_scalar((six_q3 - six_q2) + 1.0, "kernel inner polynomial")?;
        checked_scalar(KERNEL_K * polynomial, "kernel value inner")?
    } else {
        let t = checked_scalar(1.0 - q, "kernel t")?;
        let t2 = checked_scalar(t * t, "kernel t2")?;
        let t3 = checked_scalar(t2 * t, "kernel t3")?;
        checked_scalar(KERNEL_K * (2.0 * t3), "kernel value outer")?
    };

    let gradient = if displacement_is_zero {
        Vec3f::ZERO
    } else {
        let grad_q = Vec3f::new(
            checked_scalar((dx / r) / SUPPORT_RADIUS, "kernel gradq x")?,
            checked_scalar((dy / r) / SUPPORT_RADIUS, "kernel gradq y")?,
            checked_scalar((dz / r) / SUPPORT_RADIUS, "kernel gradq z")?,
        )
        .checked("kernel gradq")?;
        let coefficient = if q <= 0.5 {
            let three_q = checked_scalar(3.0 * q, "kernel three q")?;
            let lq = checked_scalar(KERNEL_L * q, "kernel lq")?;
            checked_scalar(lq * (three_q - 2.0), "kernel inner gradient coefficient")?
        } else {
            let t = checked_scalar(1.0 - q, "gradient t")?;
            let t2 = checked_scalar(t * t, "gradient t2")?;
            checked_scalar(-(KERNEL_L * t2), "kernel outer gradient coefficient")?
        };
        grad_q.scale(coefficient).checked("kernel gradient")?
    };
    Ok(KernelSample { value, gradient })
}

pub(crate) fn value_at_zero() -> f64 {
    KERNEL_K
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubic_spline_has_frozen_values_and_gradients() {
        let zero = sample(Vec3i::new(0, 0, 0)).unwrap();
        assert_eq!(zero.value.to_bits(), 0x40a3_e4f5_4b37_0dcf);
        assert_eq!(zero.gradient.x.to_bits(), 0);
        assert_eq!(zero.gradient.y.to_bits(), 0);
        assert_eq!(zero.gradient.z.to_bits(), 0);

        let half = sample(Vec3i::new(50_000, 0, 0)).unwrap();
        assert_eq!(half.value.to_bits(), 0x4083_e4f5_4b37_0dcf);
        assert_eq!(
            half.gradient.x.to_bits(),
            ((-KERNEL_L / 4.0) * 10.0).to_bits()
        );
        assert_eq!(half.gradient.y.to_bits(), (-0.0_f64).to_bits());
        assert_eq!(half.gradient.z.to_bits(), (-0.0_f64).to_bits());

        let edge = sample(Vec3i::new(100_000, 0, 0)).unwrap();
        assert_eq!(edge.value.to_bits(), 0);
        assert_eq!(edge.gradient.x.to_bits(), (-0.0_f64).to_bits());

        let outside = sample(Vec3i::new(100_001, 0, 0)).unwrap();
        assert_eq!(outside.value.to_bits(), 0);
        assert_eq!(outside.gradient.x.to_bits(), 0);
    }
}
