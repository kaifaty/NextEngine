#![forbid(unsafe_code)]

use crate::audit::vector_bits;
use crate::calibration::{ContactProjectionCase, ContactProjectionProbe};
use crate::model::Vec3f;

use super::*;

const INVERSE_DT: f64 = f64::from_bits(0x406e_0000_0000_0000);
const PARTICLE_RADIUS: f64 = f64::from_bits(0x3f99_9999_9999_999a);
const MASS: f64 = f64::from_bits(0x3fc0_0000_0000_0000);

pub(crate) fn compute() -> Result<ContactProjectionProbe, WaterError> {
    let inputs = [
        (
            "lower-face-inward",
            I3::new(25_000, 500_000, 500_000),
            I3::new(-200_000, 0, 0),
        ),
        (
            "lower-face-separating",
            I3::new(25_000, 500_000, 500_000),
            I3::new(200_000, 0, 0),
        ),
        (
            "upper-face-inward",
            I3::new(500_000, 975_000, 500_000),
            I3::new(0, 300_000, 0),
        ),
        (
            "lower-edge-inward",
            I3::new(25_000, 25_000, 500_000),
            I3::new(-100_000, -200_000, 50_000),
        ),
        (
            "mixed-corner-inward",
            I3::new(975_000, 25_000, 975_000),
            I3::new(100_000, -200_000, 300_000),
        ),
        (
            "admitted-shallow-position",
            I3::new(24_000, 500_000, 500_000),
            I3::new(0, 0, 0),
        ),
        (
            "interior-no-hit",
            I3::new(30_000, 500_000, 500_000),
            I3::new(-1_000_000, 0, 0),
        ),
        (
            "interior-predicted-hit",
            I3::new(26_000, 500_000, 500_000),
            I3::new(-1_000_000, 0, 0),
        ),
    ];
    let mut cases = reserved(inputs.len())?;
    for (id, position_um, velocity_um_s) in inputs {
        let position = F3::new(
            (position_um.x as f64) / SCALE,
            (position_um.y as f64) / SCALE,
            (position_um.z as f64) / SCALE,
        );
        let before = F3::new(
            (velocity_um_s.x as f64) / SCALE,
            (velocity_um_s.y as f64) / SCALE,
            (velocity_um_s.z as f64) / SCALE,
        );
        let after = F3::new(
            project_axis(position.x, before.x, PARTICLE_RADIUS, 1.0 - PARTICLE_RADIUS)?,
            project_axis(position.y, before.y, PARTICLE_RADIUS, 1.0 - PARTICLE_RADIUS)?,
            project_axis(position.z, before.z, PARTICLE_RADIUS, 1.0 - PARTICLE_RADIUS)?,
        );
        let delta = after.sub(before);
        let impulse = F3::new(delta.x * MASS, delta.y * MASS, delta.z * MASS);
        let active_components = [delta.x, delta.y, delta.z]
            .into_iter()
            .filter(|component| *component != 0.0)
            .count();
        cases.push(ContactProjectionCase {
            id,
            position_um: position_um.common(),
            velocity_before_bits: vector_bits(Vec3f::new(before.x, before.y, before.z)),
            velocity_after_bits: vector_bits(Vec3f::new(after.x, after.y, after.z)),
            fluid_impulse_bits: vector_bits(Vec3f::new(impulse.x, impulse.y, impulse.z)),
            active_components,
        });
    }
    Ok(ContactProjectionProbe { cases })
}

fn project_axis(position: f64, velocity: f64, lower: f64, upper: f64) -> Result<f64, WaterError> {
    let minimum_velocity = finite(
        (lower - position) * INVERSE_DT,
        "independent contact minimum velocity",
    )?;
    let maximum_velocity = finite(
        (upper - position) * INVERSE_DT,
        "independent contact maximum velocity",
    )?;
    Ok(velocity.clamp(minimum_velocity, maximum_velocity))
}
