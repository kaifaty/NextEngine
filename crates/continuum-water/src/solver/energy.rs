#![forbid(unsafe_code)]

use super::*;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct StepEnergyTrace {
    pub(crate) decoded: f64,
    pub(crate) after_divergence: f64,
    pub(crate) after_gravity: f64,
    pub(crate) after_density: f64,
    pub(crate) after_contact: f64,
    pub(crate) after_integration: f64,
    pub(crate) after_publication: f64,
}

pub(super) fn mechanical_energy(
    positions: &[Vec3f],
    velocities: &[Vec3f],
) -> Result<f64, WaterError> {
    let mut total = 0.0_f64;
    for (position, velocity) in positions.iter().zip(velocities) {
        let speed_squared = checked_scalar(velocity.dot(*velocity), "step energy speed squared")?;
        let kinetic = checked_scalar(
            0.5 * crate::profile::UNIFORM_MASS * speed_squared,
            "step kinetic energy",
        )?;
        let potential = checked_scalar(
            crate::profile::UNIFORM_MASS * GRAVITY_MAGNITUDE * position.y,
            "step potential energy",
        )?;
        total = checked_scalar(total + kinetic + potential, "step energy reduction")?;
    }
    Ok(total)
}
