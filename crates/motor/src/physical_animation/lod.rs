//! Reconstructible animation-work selection for the bounded R5 humanoid.

use next_contracts::ids::{ContentHash, PersistentId, PhysicsWorldId};
use next_contracts::physics::{PhysicsCanonicalSnapshotV2, PhysicsPoseV1};
use next_contracts::project::domain_hash;

use super::{
    PhysicalAnimationOwnerErrorV1, PhysicalAnimationOwnerV1, PhysicalAnimationPoseV1,
    PhysicalAnimationPresentationAvailabilityV1, checked_vec3_add,
};

pub const PHYSICAL_ANIMATION_REDUCED_POSE_CADENCE_TICKS_V1: u64 = 2;
pub const PHYSICAL_ANIMATION_MAX_HELD_POSE_AGE_TICKS_V1: u64 = 4;
const MAX_LOD_CADENCE_TICKS_V1: u64 = 16;
const MAX_HELD_POSE_AGE_TICKS_V1: u64 = 16;

/// Presentation work requested for one logical animation tick. These levels
/// never alter the physical-animation cursor or committed Physics state.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicalAnimationLodLevelV1 {
    IntentOnly = 1,
    ReducedPose = 2,
    FullPose = 3,
    HeldPresentationPose = 4,
    CulledPresentation = 5,
}

impl PhysicalAnimationLodLevelV1 {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::IntentOnly => "IntentOnly",
            Self::ReducedPose => "ReducedPose",
            Self::FullPose => "FullPose",
            Self::HeldPresentationPose => "HeldPresentationPose",
            Self::CulledPresentation => "CulledPresentation",
        }
    }
}

/// Fixed, reconstructible scheduling profile for the current R5 consumer.
/// A creator-authored durable LOD descriptor remains outside this bounded cut.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationLodProfileV1 {
    reduced_pose_cadence_ticks: u64,
    max_held_pose_age_ticks: u64,
}

impl PhysicalAnimationLodProfileV1 {
    pub const REFERENCE_R5: Self = Self {
        reduced_pose_cadence_ticks: PHYSICAL_ANIMATION_REDUCED_POSE_CADENCE_TICKS_V1,
        max_held_pose_age_ticks: PHYSICAL_ANIMATION_MAX_HELD_POSE_AGE_TICKS_V1,
    };

    pub fn new(
        reduced_pose_cadence_ticks: u64,
        max_held_pose_age_ticks: u64,
    ) -> Result<Self, PhysicalAnimationOwnerErrorV1> {
        if reduced_pose_cadence_ticks == 0
            || reduced_pose_cadence_ticks > MAX_LOD_CADENCE_TICKS_V1
            || max_held_pose_age_ticks < reduced_pose_cadence_ticks
            || max_held_pose_age_ticks > MAX_HELD_POSE_AGE_TICKS_V1
        {
            return Err(PhysicalAnimationOwnerErrorV1::LodProfileInvalid);
        }
        Ok(Self {
            reduced_pose_cadence_ticks,
            max_held_pose_age_ticks,
        })
    }

    #[must_use]
    pub const fn reduced_pose_cadence_ticks(self) -> u64 {
        self.reduced_pose_cadence_ticks
    }

    #[must_use]
    pub const fn max_held_pose_age_ticks(self) -> u64 {
        self.max_held_pose_age_ticks
    }

    #[must_use]
    pub fn revision(self) -> ContentHash {
        let mut bytes = b"nextengine.physical-animation-lod-profile.v1\0".to_vec();
        bytes.extend_from_slice(&self.reduced_pose_cadence_ticks.to_le_bytes());
        bytes.extend_from_slice(&self.max_held_pose_age_ticks.to_le_bytes());
        for level in [
            PhysicalAnimationLodLevelV1::IntentOnly,
            PhysicalAnimationLodLevelV1::ReducedPose,
            PhysicalAnimationLodLevelV1::FullPose,
            PhysicalAnimationLodLevelV1::HeldPresentationPose,
            PhysicalAnimationLodLevelV1::CulledPresentation,
        ] {
            bytes.push(level as u8);
            bytes.extend_from_slice(level.token().as_bytes());
            bytes.push(0);
        }
        domain_hash("nextengine.physical-animation-lod-profile.v1", &bytes)
    }

    const fn reduced_sample_is_due(self, logical_animation_tick: u64) -> bool {
        logical_animation_tick.is_multiple_of(self.reduced_pose_cadence_ticks)
    }
}

/// Optional presentation capabilities visible to the LOD planner. Required
/// physical-intent work is deliberately absent from this resource set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationLodResourcesV1 {
    pub clip_sampling_available: bool,
    pub foot_ik_available: bool,
    pub bind_pose_available: bool,
}

impl PhysicalAnimationLodResourcesV1 {
    pub const FULL: Self = Self {
        clip_sampling_available: true,
        foot_ik_available: true,
        bind_pose_available: true,
    };
    pub const CLIP_UNAVAILABLE: Self = Self {
        clip_sampling_available: false,
        foot_ik_available: false,
        bind_pose_available: true,
    };
    pub const NO_POSE: Self = Self {
        clip_sampling_available: false,
        foot_ik_available: false,
        bind_pose_available: false,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationLodRequestV1 {
    pub requested_lod: PhysicalAnimationLodLevelV1,
    pub resources: PhysicalAnimationLodResourcesV1,
}

impl PhysicalAnimationLodRequestV1 {
    #[must_use]
    pub const fn full_pose() -> Self {
        Self {
            requested_lod: PhysicalAnimationLodLevelV1::FullPose,
            resources: PhysicalAnimationLodResourcesV1::FULL,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicalAnimationLodPublicationModeV1 {
    NoPose = 1,
    Sampled = 2,
    HeldPresentationPose = 3,
    BindPoseFallback = 4,
}

/// Exact path selected by the fixed planner. This is diagnostic evidence, not
/// authoritative animation state.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicalAnimationLodDecisionV1 {
    Sampled = 1,
    ReducedCadenceHeld = 2,
    RequestedHeld = 3,
    ClipUnavailableHeld = 4,
    ReducedCadenceBind = 5,
    HeldUnavailableBind = 6,
    ClipUnavailableBind = 7,
    IntentOnly = 8,
    RequestedCull = 9,
    ReducedCadenceCull = 10,
    HeldUnavailableCull = 11,
    ClipUnavailableCull = 12,
}

/// One complete reconstructible projection or an explicit no-pose result.
/// Held results refresh their roots from current Physics while preserving the
/// bounded prior local pose and its source animation tick.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationLodProjectionV1 {
    subject_id: PersistentId,
    logical_animation_tick: u64,
    source_physics_tick: u64,
    source_physics_world_id: PhysicsWorldId,
    source_physics_catalog_hash: ContentHash,
    physical_animation_profile_revision: ContentHash,
    lod_profile_revision: ContentHash,
    requested_lod: PhysicalAnimationLodLevelV1,
    decision: PhysicalAnimationLodDecisionV1,
    publication_mode: PhysicalAnimationLodPublicationModeV1,
    pose_source_animation_tick: Option<u64>,
    source_body_pose: PhysicsPoseV1,
    pose: Option<PhysicalAnimationPoseV1>,
}

impl PhysicalAnimationLodProjectionV1 {
    #[must_use]
    pub const fn subject_id(&self) -> PersistentId {
        self.subject_id
    }

    #[must_use]
    pub const fn logical_animation_tick(&self) -> u64 {
        self.logical_animation_tick
    }

    #[must_use]
    pub const fn source_physics_tick(&self) -> u64 {
        self.source_physics_tick
    }

    #[must_use]
    pub const fn source_physics_world_id(&self) -> PhysicsWorldId {
        self.source_physics_world_id
    }

    #[must_use]
    pub const fn source_physics_catalog_hash(&self) -> ContentHash {
        self.source_physics_catalog_hash
    }

    #[must_use]
    pub const fn physical_animation_profile_revision(&self) -> ContentHash {
        self.physical_animation_profile_revision
    }

    #[must_use]
    pub const fn lod_profile_revision(&self) -> ContentHash {
        self.lod_profile_revision
    }

    #[must_use]
    pub const fn requested_lod(&self) -> PhysicalAnimationLodLevelV1 {
        self.requested_lod
    }

    #[must_use]
    pub const fn decision(&self) -> PhysicalAnimationLodDecisionV1 {
        self.decision
    }

    #[must_use]
    pub const fn publication_mode(&self) -> PhysicalAnimationLodPublicationModeV1 {
        self.publication_mode
    }

    #[must_use]
    pub const fn pose_source_animation_tick(&self) -> Option<u64> {
        self.pose_source_animation_tick
    }

    #[must_use]
    pub const fn source_body_pose(&self) -> PhysicsPoseV1 {
        self.source_body_pose
    }

    #[must_use]
    pub const fn pose(&self) -> Option<&PhysicalAnimationPoseV1> {
        self.pose.as_ref()
    }
}

impl PhysicalAnimationOwnerV1 {
    /// Resolves one animation-work request against committed owner/Physics
    /// snapshots. The operation is read-only and returns a complete pose or an
    /// explicit no-pose result.
    pub fn project_pose_lod(
        &self,
        subject_id: PersistentId,
        physics: &PhysicsCanonicalSnapshotV2,
        ground_height_micrometres_or_none: Option<i64>,
        lod_profile: PhysicalAnimationLodProfileV1,
        request: PhysicalAnimationLodRequestV1,
        previous_projection_or_none: Option<&PhysicalAnimationLodProjectionV1>,
    ) -> Result<PhysicalAnimationLodProjectionV1, PhysicalAnimationOwnerErrorV1> {
        self.validate(physics, self.snapshot.next_simulation_tick)?;
        let logical_animation_tick = self.snapshot.next_simulation_tick;
        let record = self
            .snapshot
            .records
            .iter()
            .find(|record| record.subject_id == subject_id)
            .ok_or(PhysicalAnimationOwnerErrorV1::BindingMissing)?;
        let body = physics
            .sorted_body_states
            .get(&record.body_id)
            .ok_or(PhysicalAnimationOwnerErrorV1::PhysicsProjectionInvalid)?;
        let compatible_held = previous_projection_or_none.filter(|previous| {
            self.held_projection_is_compatible(previous, subject_id, physics, lod_profile)
        });

        match request.requested_lod {
            PhysicalAnimationLodLevelV1::IntentOnly => self.no_pose_projection(
                subject_id,
                physics,
                body.pose,
                lod_profile,
                request.requested_lod,
                PhysicalAnimationLodDecisionV1::IntentOnly,
            ),
            PhysicalAnimationLodLevelV1::CulledPresentation => self.no_pose_projection(
                subject_id,
                physics,
                body.pose,
                lod_profile,
                request.requested_lod,
                PhysicalAnimationLodDecisionV1::RequestedCull,
            ),
            PhysicalAnimationLodLevelV1::HeldPresentationPose => {
                if let Some(previous) = compatible_held {
                    self.held_projection(
                        subject_id,
                        physics,
                        body.pose,
                        lod_profile,
                        request.requested_lod,
                        PhysicalAnimationLodDecisionV1::RequestedHeld,
                        previous,
                    )
                } else if request.resources.bind_pose_available {
                    self.bind_projection(
                        subject_id,
                        physics,
                        ground_height_micrometres_or_none,
                        lod_profile,
                        request.requested_lod,
                        PhysicalAnimationLodDecisionV1::HeldUnavailableBind,
                    )
                } else {
                    self.no_pose_projection(
                        subject_id,
                        physics,
                        body.pose,
                        lod_profile,
                        request.requested_lod,
                        PhysicalAnimationLodDecisionV1::HeldUnavailableCull,
                    )
                }
            }
            PhysicalAnimationLodLevelV1::ReducedPose
                if !lod_profile.reduced_sample_is_due(logical_animation_tick) =>
            {
                if let Some(previous) = compatible_held {
                    self.held_projection(
                        subject_id,
                        physics,
                        body.pose,
                        lod_profile,
                        request.requested_lod,
                        PhysicalAnimationLodDecisionV1::ReducedCadenceHeld,
                        previous,
                    )
                } else if request.resources.bind_pose_available {
                    self.bind_projection(
                        subject_id,
                        physics,
                        ground_height_micrometres_or_none,
                        lod_profile,
                        request.requested_lod,
                        PhysicalAnimationLodDecisionV1::ReducedCadenceBind,
                    )
                } else {
                    self.no_pose_projection(
                        subject_id,
                        physics,
                        body.pose,
                        lod_profile,
                        request.requested_lod,
                        PhysicalAnimationLodDecisionV1::ReducedCadenceCull,
                    )
                }
            }
            PhysicalAnimationLodLevelV1::FullPose | PhysicalAnimationLodLevelV1::ReducedPose => {
                if request.resources.clip_sampling_available {
                    self.sampled_projection(
                        subject_id,
                        physics,
                        ground_height_micrometres_or_none,
                        lod_profile,
                        request,
                    )
                } else if let Some(previous) = compatible_held {
                    self.held_projection(
                        subject_id,
                        physics,
                        body.pose,
                        lod_profile,
                        request.requested_lod,
                        PhysicalAnimationLodDecisionV1::ClipUnavailableHeld,
                        previous,
                    )
                } else if request.resources.bind_pose_available {
                    self.bind_projection(
                        subject_id,
                        physics,
                        ground_height_micrometres_or_none,
                        lod_profile,
                        request.requested_lod,
                        PhysicalAnimationLodDecisionV1::ClipUnavailableBind,
                    )
                } else {
                    self.no_pose_projection(
                        subject_id,
                        physics,
                        body.pose,
                        lod_profile,
                        request.requested_lod,
                        PhysicalAnimationLodDecisionV1::ClipUnavailableCull,
                    )
                }
            }
        }
    }

    fn sampled_projection(
        &self,
        subject_id: PersistentId,
        physics: &PhysicsCanonicalSnapshotV2,
        ground_height_micrometres_or_none: Option<i64>,
        lod_profile: PhysicalAnimationLodProfileV1,
        request: PhysicalAnimationLodRequestV1,
    ) -> Result<PhysicalAnimationLodProjectionV1, PhysicalAnimationOwnerErrorV1> {
        let body_pose = self.bound_body_pose(subject_id, physics)?;
        let pose = self.pose(
            subject_id,
            physics,
            ground_height_micrometres_or_none,
            PhysicalAnimationPresentationAvailabilityV1 {
                clip_sampling_available: true,
                foot_ik_available: request.requested_lod == PhysicalAnimationLodLevelV1::FullPose
                    && request.resources.foot_ik_available,
            },
        )?;
        self.complete_projection(
            subject_id,
            physics.physics_tick,
            physics.world_id,
            physics.catalog_hash,
            body_pose,
            lod_profile,
            request.requested_lod,
            PhysicalAnimationLodDecisionV1::Sampled,
            PhysicalAnimationLodPublicationModeV1::Sampled,
            self.snapshot.next_simulation_tick,
            pose,
        )
    }

    fn bind_projection(
        &self,
        subject_id: PersistentId,
        physics: &PhysicsCanonicalSnapshotV2,
        ground_height_micrometres_or_none: Option<i64>,
        lod_profile: PhysicalAnimationLodProfileV1,
        requested_lod: PhysicalAnimationLodLevelV1,
        decision: PhysicalAnimationLodDecisionV1,
    ) -> Result<PhysicalAnimationLodProjectionV1, PhysicalAnimationOwnerErrorV1> {
        let body_pose = self.bound_body_pose(subject_id, physics)?;
        let pose = self.pose(
            subject_id,
            physics,
            ground_height_micrometres_or_none,
            PhysicalAnimationPresentationAvailabilityV1::BIND_POSE_FALLBACK,
        )?;
        self.complete_projection(
            subject_id,
            physics.physics_tick,
            physics.world_id,
            physics.catalog_hash,
            body_pose,
            lod_profile,
            requested_lod,
            decision,
            PhysicalAnimationLodPublicationModeV1::BindPoseFallback,
            self.snapshot.next_simulation_tick,
            pose,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "a held projection binds its complete current and prior immutable evidence"
    )]
    fn held_projection(
        &self,
        subject_id: PersistentId,
        physics: &PhysicsCanonicalSnapshotV2,
        current_body_pose: PhysicsPoseV1,
        lod_profile: PhysicalAnimationLodProfileV1,
        requested_lod: PhysicalAnimationLodLevelV1,
        decision: PhysicalAnimationLodDecisionV1,
        previous: &PhysicalAnimationLodProjectionV1,
    ) -> Result<PhysicalAnimationLodProjectionV1, PhysicalAnimationOwnerErrorV1> {
        let previous_pose = previous
            .pose
            .as_ref()
            .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?;
        let motion_delta: [Option<i64>; 3] = std::array::from_fn(|axis| {
            previous_pose.rigid_root_pose.translation_micrometres[axis]
                .checked_sub(previous.source_body_pose.translation_micrometres[axis])
        });
        let motion_delta = motion_delta
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .and_then(|values| values.try_into().ok())
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        let mut pose = previous_pose.clone();
        pose.rigid_root_pose = PhysicsPoseV1 {
            translation_micrometres: checked_vec3_add(
                current_body_pose.translation_micrometres,
                motion_delta,
            )?,
            rotation_q1_30: current_body_pose.rotation_q1_30,
        };
        pose.skeleton_root_pose = PhysicsPoseV1 {
            translation_micrometres: checked_vec3_add(
                current_body_pose.translation_micrometres,
                self.profile.presentation_root_offset_micrometres,
            )?,
            rotation_q1_30: current_body_pose.rotation_q1_30,
        };
        self.complete_projection(
            subject_id,
            physics.physics_tick,
            physics.world_id,
            physics.catalog_hash,
            current_body_pose,
            lod_profile,
            requested_lod,
            decision,
            PhysicalAnimationLodPublicationModeV1::HeldPresentationPose,
            previous
                .pose_source_animation_tick
                .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?,
            pose,
        )
    }

    fn held_projection_is_compatible(
        &self,
        previous: &PhysicalAnimationLodProjectionV1,
        subject_id: PersistentId,
        physics: &PhysicsCanonicalSnapshotV2,
        lod_profile: PhysicalAnimationLodProfileV1,
    ) -> bool {
        let current_tick = self.snapshot.next_simulation_tick;
        let Some(source_tick) = previous.pose_source_animation_tick else {
            return false;
        };
        let Some(age) = current_tick.checked_sub(source_tick) else {
            return false;
        };
        let Some(pose) = previous.pose.as_ref() else {
            return false;
        };
        previous.subject_id == subject_id
            && previous.physical_animation_profile_revision == self.snapshot.profile_revision
            && previous.lod_profile_revision == lod_profile.revision()
            && previous.logical_animation_tick < current_tick
            && previous.source_physics_tick <= physics.physics_tick
            && previous.source_physics_world_id == physics.world_id
            && previous.source_physics_catalog_hash == physics.catalog_hash
            && age <= lod_profile.max_held_pose_age_ticks
            && pose.subject_id == subject_id
            && pose.joint_poses.len() == self.profile.retarget_joints.len()
            && pose
                .joint_poses
                .iter()
                .zip(&self.profile.retarget_joints)
                .all(|(pose, mapping)| pose.joint_key == mapping.target_joint_key)
    }

    fn bound_body_pose(
        &self,
        subject_id: PersistentId,
        physics: &PhysicsCanonicalSnapshotV2,
    ) -> Result<PhysicsPoseV1, PhysicalAnimationOwnerErrorV1> {
        let body_id = self
            .snapshot
            .records
            .iter()
            .find(|record| record.subject_id == subject_id)
            .map(|record| record.body_id)
            .ok_or(PhysicalAnimationOwnerErrorV1::BindingMissing)?;
        physics
            .sorted_body_states
            .get(&body_id)
            .map(|body| body.pose)
            .ok_or(PhysicalAnimationOwnerErrorV1::PhysicsProjectionInvalid)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the projection binds every immutable selection and source field explicitly"
    )]
    fn complete_projection(
        &self,
        subject_id: PersistentId,
        source_physics_tick: u64,
        source_physics_world_id: PhysicsWorldId,
        source_physics_catalog_hash: ContentHash,
        source_body_pose: PhysicsPoseV1,
        lod_profile: PhysicalAnimationLodProfileV1,
        requested_lod: PhysicalAnimationLodLevelV1,
        decision: PhysicalAnimationLodDecisionV1,
        publication_mode: PhysicalAnimationLodPublicationModeV1,
        pose_source_animation_tick: u64,
        pose: PhysicalAnimationPoseV1,
    ) -> Result<PhysicalAnimationLodProjectionV1, PhysicalAnimationOwnerErrorV1> {
        Ok(PhysicalAnimationLodProjectionV1 {
            subject_id,
            logical_animation_tick: self.snapshot.next_simulation_tick,
            source_physics_tick,
            source_physics_world_id,
            source_physics_catalog_hash,
            physical_animation_profile_revision: self.profile.revision()?,
            lod_profile_revision: lod_profile.revision(),
            requested_lod,
            decision,
            publication_mode,
            pose_source_animation_tick: Some(pose_source_animation_tick),
            source_body_pose,
            pose: Some(pose),
        })
    }

    fn no_pose_projection(
        &self,
        subject_id: PersistentId,
        physics: &PhysicsCanonicalSnapshotV2,
        source_body_pose: PhysicsPoseV1,
        lod_profile: PhysicalAnimationLodProfileV1,
        requested_lod: PhysicalAnimationLodLevelV1,
        decision: PhysicalAnimationLodDecisionV1,
    ) -> Result<PhysicalAnimationLodProjectionV1, PhysicalAnimationOwnerErrorV1> {
        Ok(PhysicalAnimationLodProjectionV1 {
            subject_id,
            logical_animation_tick: self.snapshot.next_simulation_tick,
            source_physics_tick: physics.physics_tick,
            source_physics_world_id: physics.world_id,
            source_physics_catalog_hash: physics.catalog_hash,
            physical_animation_profile_revision: self.profile.revision()?,
            lod_profile_revision: lod_profile.revision(),
            requested_lod,
            decision,
            publication_mode: PhysicalAnimationLodPublicationModeV1::NoPose,
            pose_source_animation_tick: None,
            source_body_pose,
            pose: None,
        })
    }
}
