//! Stateless procedural safety route for the active `CapsuleAnimation` tier.
//!
//! The controller emits only a validated capsule locomotion proposal. It
//! never writes a transform and owns no state: recovery is derived from the
//! committed Physics snapshot, so save/load and replay need no second owner.

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::body::{BodyInstanceProjectionV1, BodyProjectionRootsV1, BodySchemaV1};
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, content_hash_from_bytes};
use next_contracts::physical_animation::capsule_root_motion_profile_hash_v1;
use next_contracts::physics::{
    PhysicalCommandV1, PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2, PhysicsPoseV1,
};

use crate::{CompiledBodySchemaV1, body_projection_compiler_profile_hash_v1};

pub const CAPSULE_PROCEDURAL_MOTOR_PROFILE_VERSION_V1: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CapsuleMotorRouteV1 {
    Idle = 1,
    ProceduralTracking = 2,
    ProceduralRecovery = 3,
}

/// Reconstructible decision evidence for one logical gameplay tick.
///
/// `clamp_mask` uses bit 0 for local-right and bit 1 for local-forward. The
/// current recovery route clamps both requested horizontal channels to zero
/// while Physics reports non-zero vertical velocity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapsuleMotorDecisionV1 {
    pub source_physics_tick: u64,
    pub source_body_revision: u64,
    pub route: CapsuleMotorRouteV1,
    pub requested_direction_q15: [i16; 2],
    pub applied_direction_q15: [i16; 2],
    pub clamp_mask: u8,
    pub controller_profile_hash: ContentHash,
    pub body_projection_root: ContentHash,
    pub actuator_safety_root: ContentHash,
    pub decision_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapsuleProceduralMotorControllerV1 {
    subject_id: PersistentId,
    body_id: PhysicsBodyIdV1,
    body_projection_root: ContentHash,
    action_layout_hash: ContentHash,
    actuator_safety_root: ContentHash,
    locomotion_profile_hash: ContentHash,
    controller_profile_hash: ContentHash,
}

impl CapsuleProceduralMotorControllerV1 {
    pub fn activate(
        schema: &BodySchemaV1,
        projection: &BodyInstanceProjectionV1,
        expected_roots: &BodyProjectionRootsV1,
        body_id: PhysicsBodyIdV1,
        gameplay_hz: u32,
    ) -> Result<Self, CapsuleProceduralMotorError> {
        projection
            .validate_against(schema)
            .map_err(|_| CapsuleProceduralMotorError::ProjectionMismatch)?;
        expected_roots
            .validate()
            .map_err(|_| CapsuleProceduralMotorError::ProjectionMismatch)?;
        let compiled = CompiledBodySchemaV1::compile_projection(schema, projection)
            .map_err(|_| CapsuleProceduralMotorError::ProjectionMismatch)?;
        if compiled.projection_roots != *expected_roots {
            return Err(CapsuleProceduralMotorError::ProjectionMismatch);
        }
        let body_projection_root = expected_roots
            .projection_root()
            .map_err(|_| CapsuleProceduralMotorError::ProjectionMismatch)?;
        let locomotion_profile_hash = capsule_root_motion_profile_hash_v1(gameplay_hz)
            .map_err(|_| CapsuleProceduralMotorError::LocomotionProfileMismatch)?;
        if projection.subject_id != body_id.subject_id
            || expected_roots.body_schema_hash != projection.body_schema_hash
            || expected_roots.body_instance_projection_hash
                != compiled.body_instance_projection_hash
            || expected_roots.compiler_profile_hash != body_projection_compiler_profile_hash_v1()
        {
            return Err(CapsuleProceduralMotorError::ProjectionMismatch);
        }

        let controller_profile_hash = controller_profile_hash(
            projection.subject_id,
            body_id,
            body_projection_root,
            expected_roots.action_layout_hash,
            expected_roots.actuator_safety_root,
            locomotion_profile_hash,
        );
        Ok(Self {
            subject_id: projection.subject_id,
            body_id,
            body_projection_root,
            action_layout_hash: expected_roots.action_layout_hash,
            actuator_safety_root: expected_roots.actuator_safety_root,
            locomotion_profile_hash,
            controller_profile_hash,
        })
    }

    #[must_use]
    pub const fn subject_id(&self) -> PersistentId {
        self.subject_id
    }

    #[must_use]
    pub const fn body_id(&self) -> PhysicsBodyIdV1 {
        self.body_id
    }

    #[must_use]
    pub const fn body_projection_root(&self) -> ContentHash {
        self.body_projection_root
    }

    #[must_use]
    pub const fn action_layout_hash(&self) -> ContentHash {
        self.action_layout_hash
    }

    #[must_use]
    pub const fn actuator_safety_root(&self) -> ContentHash {
        self.actuator_safety_root
    }

    #[must_use]
    pub const fn locomotion_profile_hash(&self) -> ContentHash {
        self.locomotion_profile_hash
    }

    #[must_use]
    pub const fn controller_profile_hash(&self) -> ContentHash {
        self.controller_profile_hash
    }

    pub fn evaluate(
        &self,
        physics: &PhysicsCanonicalSnapshotV2,
        requested_direction_q15: [i16; 2],
    ) -> Result<CapsuleMotorDecisionV1, CapsuleProceduralMotorError> {
        physics
            .validate()
            .map_err(|_| CapsuleProceduralMotorError::PhysicsSnapshotInvalid)?;
        PhysicalCommandV1::SetCapsuleLocomotionIntent {
            direction_q15: requested_direction_q15,
        }
        .validate()
        .map_err(|_| CapsuleProceduralMotorError::CandidateInvalid)?;
        let body = physics
            .sorted_body_states
            .get(&self.body_id)
            .ok_or(CapsuleProceduralMotorError::BodyMissing)?;
        if body.body_id.subject_id != self.subject_id || !body.active {
            return Err(CapsuleProceduralMotorError::BodyUnavailable);
        }
        if body.pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
            || body.angular_velocity_q16 != [0; 3]
        {
            return Err(CapsuleProceduralMotorError::BodyStateInvalid);
        }

        let recovery_required = body.linear_velocity_micrometres_per_second[1] != 0;
        let (route, applied_direction_q15) = if recovery_required {
            (CapsuleMotorRouteV1::ProceduralRecovery, [0, 0])
        } else if requested_direction_q15 == [0, 0] {
            (CapsuleMotorRouteV1::Idle, [0, 0])
        } else {
            (
                CapsuleMotorRouteV1::ProceduralTracking,
                requested_direction_q15,
            )
        };
        PhysicalCommandV1::SetCapsuleLocomotionIntent {
            direction_q15: applied_direction_q15,
        }
        .validate()
        .map_err(|_| CapsuleProceduralMotorError::CandidateInvalid)?;
        let clamp_mask = u8::from(requested_direction_q15[0] != applied_direction_q15[0])
            | (u8::from(requested_direction_q15[1] != applied_direction_q15[1]) << 1);
        let decision_hash = decision_hash(
            physics.physics_tick,
            body.body_revision,
            route,
            requested_direction_q15,
            applied_direction_q15,
            clamp_mask,
            self.controller_profile_hash,
            self.body_projection_root,
            self.actuator_safety_root,
        );
        Ok(CapsuleMotorDecisionV1 {
            source_physics_tick: physics.physics_tick,
            source_body_revision: body.body_revision,
            route,
            requested_direction_q15,
            applied_direction_q15,
            clamp_mask,
            controller_profile_hash: self.controller_profile_hash,
            body_projection_root: self.body_projection_root,
            actuator_safety_root: self.actuator_safety_root,
            decision_hash,
        })
    }
}

fn controller_profile_hash(
    subject_id: PersistentId,
    body_id: PhysicsBodyIdV1,
    body_projection_root: ContentHash,
    action_layout_hash: ContentHash,
    actuator_safety_root: ContentHash,
    locomotion_profile_hash: ContentHash,
) -> ContentHash {
    let mut bytes = b"nextengine.capsule-procedural-motor-profile.v1\0".to_vec();
    bytes.extend_from_slice(&CAPSULE_PROCEDURAL_MOTOR_PROFILE_VERSION_V1.to_le_bytes());
    bytes.extend_from_slice(subject_id.as_bytes());
    bytes.extend_from_slice(body_id.subject_id.as_bytes());
    bytes.extend_from_slice(&body_id.body_slot.to_le_bytes());
    for hash in [
        body_projection_root,
        action_layout_hash,
        actuator_safety_root,
        locomotion_profile_hash,
    ] {
        bytes.extend_from_slice(hash.as_bytes());
    }
    content_hash_from_bytes(sha256(&bytes))
}

#[allow(
    clippy::too_many_arguments,
    reason = "the decision hash covers the complete immutable route evidence"
)]
fn decision_hash(
    source_physics_tick: u64,
    source_body_revision: u64,
    route: CapsuleMotorRouteV1,
    requested_direction_q15: [i16; 2],
    applied_direction_q15: [i16; 2],
    clamp_mask: u8,
    controller_profile_hash: ContentHash,
    body_projection_root: ContentHash,
    actuator_safety_root: ContentHash,
) -> ContentHash {
    let mut bytes = b"nextengine.capsule-procedural-motor-decision.v1\0".to_vec();
    bytes.extend_from_slice(&source_physics_tick.to_le_bytes());
    bytes.extend_from_slice(&source_body_revision.to_le_bytes());
    bytes.push(route as u8);
    for value in requested_direction_q15
        .into_iter()
        .chain(applied_direction_q15)
    {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.push(clamp_mask);
    for hash in [
        controller_profile_hash,
        body_projection_root,
        actuator_safety_root,
    ] {
        bytes.extend_from_slice(hash.as_bytes());
    }
    content_hash_from_bytes(sha256(&bytes))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapsuleProceduralMotorError {
    ProjectionMismatch,
    LocomotionProfileMismatch,
    PhysicsSnapshotInvalid,
    CandidateInvalid,
    BodyMissing,
    BodyUnavailable,
    BodyStateInvalid,
}

impl CapsuleProceduralMotorError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::ProjectionMismatch => "MOTOR_CAPSULE_PROJECTION_MISMATCH",
            Self::LocomotionProfileMismatch => "MOTOR_CAPSULE_LOCOMOTION_PROFILE_MISMATCH",
            Self::PhysicsSnapshotInvalid => "MOTOR_CAPSULE_PHYSICS_SNAPSHOT_INVALID",
            Self::CandidateInvalid => "MOTOR_CAPSULE_CANDIDATE_INVALID",
            Self::BodyMissing => "MOTOR_CAPSULE_BODY_MISSING",
            Self::BodyUnavailable => "MOTOR_CAPSULE_BODY_UNAVAILABLE",
            Self::BodyStateInvalid => "MOTOR_CAPSULE_BODY_STATE_INVALID",
        }
    }
}

impl Display for CapsuleProceduralMotorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for CapsuleProceduralMotorError {}
