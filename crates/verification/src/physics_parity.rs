use std::error::Error;
use std::fmt::{Display, Formatter};

#[cfg(any(feature = "physx", feature = "physx-mock"))]
use next_contracts::physics::{
    PhysicsGeometryV1, PhysicsMotionKindV1, PhysicsParticipationV1, PhysicsPoseV1,
};
#[cfg(any(feature = "physx", feature = "physx-mock"))]
use next_physics_api::{
    GroundedCapsuleQuery, GroundedCapsuleStaticBox, GroundedCapsuleSweepRequest,
    ReferenceGroundedCapsuleQuery,
};
#[cfg(any(feature = "physx", feature = "physx-mock"))]
use next_physics_physx::PhysXGroundedCapsuleQuery;

#[cfg(any(feature = "physx", feature = "physx-mock"))]
use crate::build_physx_player_fixture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicsBackendParityReport {
    pub compared_substeps: u64,
    pub registration_permutations: u64,
    pub world_lifecycle_cycles: u64,
}

pub fn run_physics_backend_parity_check(
    substeps: u64,
    registration_permutations: u64,
) -> Result<PhysicsBackendParityReport, PhysicsBackendParityError> {
    if substeps == 0 || registration_permutations == 0 {
        return Err(PhysicsBackendParityError::InvalidCount);
    }
    run_enabled_parity_check(substeps, registration_permutations)
}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
fn run_enabled_parity_check(
    substeps: u64,
    registration_permutations: u64,
) -> Result<PhysicsBackendParityReport, PhysicsBackendParityError> {
    let fixture = build_physx_player_fixture("nextengine.physics-backend-parity")?;
    let checkpoint = &fixture.bootstrap.physics_checkpoint;
    let static_boxes = collect_static_boxes(checkpoint)?;
    let mut reference = ReferenceGroundedCapsuleQuery;
    let mut physx = PhysXGroundedCapsuleQuery::new(
        fixture.bootstrap.authoritative_numeric_profile.clone(),
        fixture.bootstrap.physics_quantization_profile.clone(),
    )?;
    for trial in 0..substeps {
        let case = parity_case(trial);
        compare_sweep(&mut reference, &mut physx, &static_boxes, case)?;
    }

    for seed in 0..registration_permutations {
        let mut permuted = PhysXGroundedCapsuleQuery::with_registration_seed(
            fixture.bootstrap.authoritative_numeric_profile.clone(),
            fixture.bootstrap.physics_quantization_profile.clone(),
            seed,
        )?;
        compare_sweep(
            &mut reference,
            &mut permuted,
            &static_boxes,
            parity_case(seed.saturating_add(2)),
        )?;
    }

    Ok(PhysicsBackendParityReport {
        compared_substeps: substeps,
        registration_permutations,
        world_lifecycle_cycles: registration_permutations.saturating_add(1),
    })
}

#[cfg(not(any(feature = "physx", feature = "physx-mock")))]
fn run_enabled_parity_check(
    _substeps: u64,
    _registration_permutations: u64,
) -> Result<PhysicsBackendParityReport, PhysicsBackendParityError> {
    Err(PhysicsBackendParityError::BackendUnavailable)
}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
#[derive(Clone, Copy)]
struct ParityCase {
    centre_micrometres: [i64; 3],
    axis: u8,
    delta_micrometres: i64,
}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
fn parity_case(trial: u64) -> ParityCase {
    match trial % 3 {
        0 => ParityCase {
            centre_micrometres: [0, 900_000, 0],
            axis: 1,
            delta_micrometres: -10_000,
        },
        1 => ParityCase {
            centre_micrometres: [0, 900_000, 0],
            axis: 0,
            delta_micrometres: 25_000,
        },
        _ => ParityCase {
            centre_micrometres: [0, 900_000, 0],
            axis: 2,
            delta_micrometres: 400_000,
        },
    }
}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
fn compare_sweep(
    reference: &mut ReferenceGroundedCapsuleQuery,
    physx: &mut PhysXGroundedCapsuleQuery,
    static_boxes: &[GroundedCapsuleStaticBox],
    case: ParityCase,
) -> Result<(), PhysicsBackendParityError> {
    let request = || GroundedCapsuleSweepRequest {
        centre_micrometres: case.centre_micrometres,
        axis: case.axis,
        delta_micrometres: case.delta_micrometres,
        capsule_radius_micrometres: 300_000,
        capsule_half_segment_micrometres: 600_000,
        capsule_collision_layer: 0,
        capsule_collision_mask: 1,
        static_boxes,
    };
    let expected = reference.sweep_axis(request())?;
    let actual = physx.sweep_axis(request())?;
    if expected == actual {
        Ok(())
    } else {
        Err(PhysicsBackendParityError::SweepMismatch)
    }
}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
fn collect_static_boxes(
    checkpoint: &next_contracts::physics::PhysicsWorldCheckpointV1,
) -> Result<Vec<GroundedCapsuleStaticBox>, PhysicsBackendParityError> {
    let mut boxes = Vec::new();
    for (body_id, body) in &checkpoint.catalog.bodies {
        if body.motion_kind != PhysicsMotionKindV1::Static || !body.active {
            continue;
        }
        let body_pose = checkpoint
            .snapshot
            .sorted_body_states
            .get(body_id)
            .ok_or(PhysicsBackendParityError::FixtureShape)?
            .pose;
        for shape in body.shapes.values() {
            let PhysicsGeometryV1::Box {
                half_extents_micrometres,
            } = &shape.geometry
            else {
                return Err(PhysicsBackendParityError::FixtureShape);
            };
            if shape.participation != PhysicsParticipationV1::Solid
                || body_pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
                || shape.local_pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
            {
                return Err(PhysicsBackendParityError::FixtureShape);
            }
            let mut centre = [0_i64; 3];
            let mut minimum = [0_i64; 3];
            let mut maximum = [0_i64; 3];
            for axis in 0..3 {
                centre[axis] = body_pose.translation_micrometres[axis]
                    .checked_add(shape.local_pose.translation_micrometres[axis])
                    .ok_or(PhysicsBackendParityError::NumericOverflow)?;
                minimum[axis] = centre[axis]
                    .checked_sub(half_extents_micrometres[axis])
                    .ok_or(PhysicsBackendParityError::NumericOverflow)?;
                maximum[axis] = centre[axis]
                    .checked_add(half_extents_micrometres[axis])
                    .ok_or(PhysicsBackendParityError::NumericOverflow)?;
            }
            boxes.push(GroundedCapsuleStaticBox {
                shape_id: shape.shape_id,
                minimum,
                maximum,
                contact_reporting: shape.contact_reporting,
                collision_layer: shape.collision_layer,
                collision_mask: shape.collision_mask,
            });
        }
    }
    boxes.sort_by_key(|shape| shape.shape_id);
    Ok(boxes)
}

#[derive(Debug)]
pub enum PhysicsBackendParityError {
    InvalidCount,
    BackendUnavailable,
    FixtureShape,
    NumericOverflow,
    SweepMismatch,
    #[cfg(any(feature = "physx", feature = "physx-mock"))]
    Fixture(crate::NeutralFixtureError),
    #[cfg(any(feature = "physx", feature = "physx-mock"))]
    Reference(next_physics_api::ReferencePhysicsError),
    #[cfg(any(feature = "physx", feature = "physx-mock"))]
    Adapter(next_physics_physx::PhysXAdapterError),
}

impl Display for PhysicsBackendParityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCount => formatter.write_str("parity counts must be non-zero"),
            Self::BackendUnavailable => {
                formatter.write_str("PhysX parity requires the physx or physx-mock feature")
            }
            Self::FixtureShape => formatter.write_str("parity fixture has an unsupported shape"),
            Self::NumericOverflow => formatter.write_str("parity fixture numeric overflow"),
            Self::SweepMismatch => {
                formatter.write_str("reference and PhysX sweep results diverged")
            }
            #[cfg(any(feature = "physx", feature = "physx-mock"))]
            Self::Fixture(error) => write!(formatter, "{error}"),
            #[cfg(any(feature = "physx", feature = "physx-mock"))]
            Self::Reference(error) => write!(formatter, "{error}"),
            #[cfg(any(feature = "physx", feature = "physx-mock"))]
            Self::Adapter(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for PhysicsBackendParityError {}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
impl From<crate::NeutralFixtureError> for PhysicsBackendParityError {
    fn from(error: crate::NeutralFixtureError) -> Self {
        Self::Fixture(error)
    }
}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
impl From<next_physics_api::ReferencePhysicsError> for PhysicsBackendParityError {
    fn from(error: next_physics_api::ReferencePhysicsError) -> Self {
        Self::Reference(error)
    }
}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
impl From<next_physics_physx::PhysXAdapterError> for PhysicsBackendParityError {
    fn from(error: next_physics_physx::PhysXAdapterError) -> Self {
        Self::Adapter(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_counts() {
        assert!(matches!(
            run_physics_backend_parity_check(0, 1),
            Err(PhysicsBackendParityError::InvalidCount)
        ));
        assert!(matches!(
            run_physics_backend_parity_check(1, 0),
            Err(PhysicsBackendParityError::InvalidCount)
        ));
    }

    #[test]
    #[cfg(any(feature = "physx", feature = "physx-mock"))]
    fn enabled_backend_matches_short_stress_sample() {
        let report = run_physics_backend_parity_check(30, 10).expect("parity");
        assert_eq!(report.compared_substeps, 30);
        assert_eq!(report.registration_permutations, 10);
        assert_eq!(report.world_lifecycle_cycles, 11);
    }
}
