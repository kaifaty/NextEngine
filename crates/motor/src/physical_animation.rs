//! Deterministic R5a animation owner over committed physics projections.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::animation_content::{
    AnimationInterpolationV1, AnimationWrapModeV1, NeutralAnimationChannelV1,
    NeutralAnimationKeyV1, NeutralAnimationV1, NeutralAnimationValueV1, NeutralSkeletonV1,
    NeutralTransformV1,
};
use next_contracts::ids::{PersistentId, SchemaId};
use next_contracts::physical_animation::{
    PhysicalAnimationBindingV1, PhysicalAnimationContractErrorV1, PhysicalAnimationGraphStateV1,
    PhysicalAnimationProfileV1, PhysicalAnimationSnapshotV1, ROOT_MOTION_INTENT_SCHEMA_VERSION,
    RootMotionIntentV1, capsule_root_motion_profile_hash_v1,
    capsule_root_motion_step_micrometres_v1,
};
use next_contracts::physics::{PhysicsCanonicalSnapshotV2, PhysicsPoseV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationPresentationAvailabilityV1 {
    pub clip_sampling_available: bool,
    pub foot_ik_available: bool,
}

impl PhysicalAnimationPresentationAvailabilityV1 {
    pub const FULL: Self = Self {
        clip_sampling_available: true,
        foot_ik_available: true,
    };
    pub const BIND_POSE_FALLBACK: Self = Self {
        clip_sampling_available: false,
        foot_ik_available: false,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PhysicalAnimationProjectionModeV1 {
    SampledWithFootIk = 1,
    SampledWithoutFootIk = 2,
    BindPoseFallback = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationJointPoseV1 {
    pub joint_key: SchemaId,
    pub local_transform: NeutralTransformV1,
}

/// Reconstructible presentation projection. Neither root pose is accepted as
/// a physics update: both are derived from the committed body pose.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationPoseV1 {
    pub subject_id: PersistentId,
    pub graph_state: PhysicalAnimationGraphStateV1,
    pub clip_time_microseconds: u64,
    pub projection_mode: PhysicalAnimationProjectionModeV1,
    pub rigid_root_pose: PhysicsPoseV1,
    pub skeleton_root_pose: PhysicsPoseV1,
    pub joint_poses: Vec<PhysicalAnimationJointPoseV1>,
    pub foot_ik_corrections_micrometres: [i64; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalAnimationOwnerV1 {
    profile: PhysicalAnimationProfileV1,
    skeleton: NeutralSkeletonV1,
    idle_clip: NeutralAnimationV1,
    locomotion_clip: NeutralAnimationV1,
    bindings: Vec<PhysicalAnimationBindingV1>,
    snapshot: PhysicalAnimationSnapshotV1,
}

impl PhysicalAnimationOwnerV1 {
    pub fn activate(
        profile: PhysicalAnimationProfileV1,
        skeleton: NeutralSkeletonV1,
        idle_clip: NeutralAnimationV1,
        locomotion_clip: NeutralAnimationV1,
        bindings: Vec<PhysicalAnimationBindingV1>,
        physics: &PhysicsCanonicalSnapshotV2,
    ) -> Result<Self, PhysicalAnimationOwnerErrorV1> {
        profile.validate_against_content(&skeleton, &idle_clip, &locomotion_clip)?;
        let snapshot = PhysicalAnimationSnapshotV1::initial(&profile, &bindings)?;
        let value = Self {
            profile,
            skeleton,
            idle_clip,
            locomotion_clip,
            bindings,
            snapshot,
        };
        value.validate(physics, 0)?;
        Ok(value)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "restore names the exact immutable content, bindings, owner bytes and physics projection"
    )]
    pub fn restore(
        profile: PhysicalAnimationProfileV1,
        skeleton: NeutralSkeletonV1,
        idle_clip: NeutralAnimationV1,
        locomotion_clip: NeutralAnimationV1,
        bindings: Vec<PhysicalAnimationBindingV1>,
        snapshot: PhysicalAnimationSnapshotV1,
        physics: &PhysicsCanonicalSnapshotV2,
        next_simulation_tick: u64,
    ) -> Result<Self, PhysicalAnimationOwnerErrorV1> {
        profile.validate_against_content(&skeleton, &idle_clip, &locomotion_clip)?;
        let value = Self {
            profile,
            skeleton,
            idle_clip,
            locomotion_clip,
            bindings,
            snapshot,
        };
        value.validate(physics, next_simulation_tick)?;
        Ok(value)
    }

    #[must_use]
    pub const fn profile(&self) -> &PhysicalAnimationProfileV1 {
        &self.profile
    }

    #[must_use]
    pub fn bindings(&self) -> &[PhysicalAnimationBindingV1] {
        &self.bindings
    }

    #[must_use]
    pub const fn snapshot(&self) -> &PhysicalAnimationSnapshotV1 {
        &self.snapshot
    }

    /// Samples one future locomotion interval without mutating graph state.
    /// The returned value is only a proposal; Runtime must still validate it
    /// and capsule physics remains the only transform writer.
    pub fn root_motion_intent(
        &self,
        subject_id: PersistentId,
        intent_sequence: u64,
        source_action_or_ability_phase_id: SchemaId,
        physics: &PhysicsCanonicalSnapshotV2,
    ) -> Result<RootMotionIntentV1, PhysicalAnimationOwnerErrorV1> {
        self.validate(physics, self.snapshot.next_simulation_tick)?;
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
        let phase_ticks = match record.graph_state {
            PhysicalAnimationGraphStateV1::Idle => 0,
            PhysicalAnimationGraphStateV1::Locomotion => record.phase_ticks,
        };
        let next_phase_ticks = phase_ticks
            .checked_add(1)
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        let elapsed_us = |ticks: u64| {
            u128::from(ticks)
                .checked_mul(1_000_000)
                .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)
                .map(|value| value / u128::from(self.profile.gameplay_hz))
                .and_then(|value| {
                    u64::try_from(value)
                        .map_err(|_| PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)
                })
        };
        let start_us = elapsed_us(phase_ticks)?;
        let end_us = elapsed_us(next_phase_ticks)?;
        let start = sample_root_translation_unwrapped(&self.locomotion_clip, start_us)?;
        let end = sample_root_translation_unwrapped(&self.locomotion_clip, end_us)?;
        let translation: [Option<i64>; 3] =
            std::array::from_fn(|axis| end[axis].checked_sub(start[axis]));
        let translation = translation
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .and_then(|values| values.try_into().ok())
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        if translation
            != [
                0,
                0,
                capsule_root_motion_step_micrometres_v1(self.profile.gameplay_hz)?,
            ]
        {
            return Err(PhysicalAnimationOwnerErrorV1::RootMotionRejected);
        }
        let intent = RootMotionIntentV1 {
            schema_version: ROOT_MOTION_INTENT_SCHEMA_VERSION,
            subject_id,
            intent_sequence,
            source_graph_hash: self.profile.revision()?,
            source_clip_hash: self
                .locomotion_clip
                .record_sha256()
                .map_err(PhysicalAnimationContractErrorV1::from)?,
            source_action_or_ability_phase_id,
            source_animation_tick: self.snapshot.next_simulation_tick,
            interval_us: end_us
                .checked_sub(start_us)
                .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?,
            quantized_local_translation: translation,
            quantized_local_yaw: 0,
            locomotion_profile_hash: capsule_root_motion_profile_hash_v1(self.profile.gameplay_hz)?,
            expected_intent_state_revision: self.snapshot.next_simulation_tick,
            expected_body_revision: body.body_revision,
        };
        intent.validate()?;
        Ok(intent)
    }

    pub fn validate(
        &self,
        physics: &PhysicsCanonicalSnapshotV2,
        next_simulation_tick: u64,
    ) -> Result<(), PhysicalAnimationOwnerErrorV1> {
        self.profile.validate_against_content(
            &self.skeleton,
            &self.idle_clip,
            &self.locomotion_clip,
        )?;
        self.snapshot
            .validate_against(&self.profile, &self.bindings)?;
        if self.snapshot.next_simulation_tick != next_simulation_tick {
            return Err(PhysicalAnimationOwnerErrorV1::TickSequenceInvalid);
        }
        validate_bound_physics(&self.bindings, physics)
    }

    /// Advances graph state from actual committed body displacement. The
    /// candidate is built completely before replacing the current snapshot.
    pub fn advance(
        &mut self,
        previous_physics: &PhysicsCanonicalSnapshotV2,
        next_physics: &PhysicsCanonicalSnapshotV2,
        next_simulation_tick: u64,
    ) -> Result<(), PhysicalAnimationOwnerErrorV1> {
        self.validate(previous_physics, self.snapshot.next_simulation_tick)?;
        validate_bound_physics(&self.bindings, next_physics)?;
        if next_simulation_tick
            != self
                .snapshot
                .next_simulation_tick
                .checked_add(1)
                .ok_or(PhysicalAnimationOwnerErrorV1::TickSequenceInvalid)?
            || previous_physics.world_id != next_physics.world_id
            || previous_physics.catalog_hash != next_physics.catalog_hash
            || previous_physics.physics_tick >= next_physics.physics_tick
        {
            return Err(PhysicalAnimationOwnerErrorV1::TickSequenceInvalid);
        }
        let mut records = Vec::with_capacity(self.snapshot.records.len());
        for record in &self.snapshot.records {
            let previous = previous_physics
                .sorted_body_states
                .get(&record.body_id)
                .ok_or(PhysicalAnimationOwnerErrorV1::PhysicsProjectionInvalid)?;
            let next = next_physics
                .sorted_body_states
                .get(&record.body_id)
                .ok_or(PhysicalAnimationOwnerErrorV1::PhysicsProjectionInvalid)?;
            let moved = [0_usize, 2].into_iter().try_fold(false, |moved, axis| {
                let delta = next.pose.translation_micrometres[axis]
                    .checked_sub(previous.pose.translation_micrometres[axis])
                    .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
                Ok::<_, PhysicalAnimationOwnerErrorV1>(
                    moved
                        || delta.unsigned_abs()
                            >= self.profile.locomotion_threshold_micrometres_per_tick,
                )
            })?;
            let graph_state = if moved {
                PhysicalAnimationGraphStateV1::Locomotion
            } else {
                PhysicalAnimationGraphStateV1::Idle
            };
            let phase_ticks = if graph_state == record.graph_state {
                record
                    .phase_ticks
                    .checked_add(1)
                    .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?
            } else {
                0
            };
            records.push(
                next_contracts::physical_animation::PhysicalAnimationRecordV1 {
                    subject_id: record.subject_id,
                    body_id: record.body_id,
                    graph_state,
                    phase_ticks,
                },
            );
        }
        let candidate = PhysicalAnimationSnapshotV1 {
            schema_version: next_contracts::physical_animation::PHYSICAL_ANIMATION_SCHEMA_VERSION,
            profile_revision: self.profile.revision()?,
            next_simulation_tick,
            records,
        };
        candidate.validate_against(&self.profile, &self.bindings)?;
        self.snapshot = candidate;
        Ok(())
    }

    /// Reconstructs one pose. Optional presentation capability failures select
    /// a deterministic projection fallback without changing owner state.
    pub fn pose(
        &self,
        subject_id: PersistentId,
        physics: &PhysicsCanonicalSnapshotV2,
        ground_height_micrometres_or_none: Option<i64>,
        availability: PhysicalAnimationPresentationAvailabilityV1,
    ) -> Result<PhysicalAnimationPoseV1, PhysicalAnimationOwnerErrorV1> {
        self.validate(physics, self.snapshot.next_simulation_tick)?;
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
        let clip = match record.graph_state {
            PhysicalAnimationGraphStateV1::Idle => &self.idle_clip,
            PhysicalAnimationGraphStateV1::Locomotion => &self.locomotion_clip,
        };
        let clip_time_microseconds = clip_time_microseconds(
            record.phase_ticks,
            self.profile.gameplay_hz,
            clip.duration_microseconds,
        )?;
        let sampled = availability
            .clip_sampling_available
            .then(|| sample_clip(&self.skeleton, clip, clip_time_microseconds))
            .transpose()
            .ok()
            .flatten();
        let (mut local_poses, mut projection_mode) = match sampled {
            Some(poses) => (
                poses,
                PhysicalAnimationProjectionModeV1::SampledWithoutFootIk,
            ),
            None => (
                bind_pose(&self.skeleton),
                PhysicalAnimationProjectionModeV1::BindPoseFallback,
            ),
        };
        let mut foot_ik_corrections_micrometres = [0_i64; 2];
        if projection_mode != PhysicalAnimationProjectionModeV1::BindPoseFallback
            && availability.foot_ik_available
            && let Some(ground_height) = ground_height_micrometres_or_none
            && let Some(corrections) = foot_ik_corrections(
                &self.profile,
                &self.skeleton,
                &local_poses,
                body.pose,
                ground_height,
            )?
        {
            for (foot, correction) in self.profile.foot_joint_keys.iter().zip(corrections) {
                let pose = local_poses
                    .get_mut(foot)
                    .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?;
                pose.translation_micrometres[1] = pose.translation_micrometres[1]
                    .checked_add(correction)
                    .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
            }
            foot_ik_corrections_micrometres = corrections;
            projection_mode = PhysicalAnimationProjectionModeV1::SampledWithFootIk;
        }

        let motion = local_poses
            .get(&self.profile.presentation_motion_joint_key)
            .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?;
        let bind_motion = self
            .skeleton
            .joints
            .iter()
            .find(|joint| joint.joint_key == self.profile.presentation_motion_joint_key)
            .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?
            .bind_transform;
        let motion_delta: [Option<i64>; 3] = std::array::from_fn(|axis| {
            motion.translation_micrometres[axis]
                .checked_sub(bind_motion.translation_micrometres[axis])
        });
        let motion_delta = motion_delta
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .and_then(|values| values.try_into().ok())
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        let rigid_root_pose = PhysicsPoseV1 {
            translation_micrometres: checked_vec3_add(
                body.pose.translation_micrometres,
                motion_delta,
            )?,
            rotation_q1_30: body.pose.rotation_q1_30,
        };
        let skeleton_root_pose = PhysicsPoseV1 {
            translation_micrometres: checked_vec3_add(
                body.pose.translation_micrometres,
                self.profile.presentation_root_offset_micrometres,
            )?,
            rotation_q1_30: body.pose.rotation_q1_30,
        };
        let joint_poses = self
            .profile
            .retarget_joints
            .iter()
            .map(|retarget| {
                local_poses
                    .get(&retarget.source_joint_key)
                    .copied()
                    .map(|local_transform| PhysicalAnimationJointPoseV1 {
                        joint_key: retarget.target_joint_key.clone(),
                        local_transform,
                    })
                    .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PhysicalAnimationPoseV1 {
            subject_id,
            graph_state: record.graph_state,
            clip_time_microseconds,
            projection_mode,
            rigid_root_pose,
            skeleton_root_pose,
            joint_poses,
            foot_ik_corrections_micrometres,
        })
    }
}

fn validate_bound_physics(
    bindings: &[PhysicalAnimationBindingV1],
    physics: &PhysicsCanonicalSnapshotV2,
) -> Result<(), PhysicalAnimationOwnerErrorV1> {
    if bindings.iter().any(|binding| {
        physics
            .sorted_body_states
            .get(&binding.body_id)
            .is_none_or(|body| !body.active)
    }) {
        return Err(PhysicalAnimationOwnerErrorV1::PhysicsProjectionInvalid);
    }
    Ok(())
}

fn clip_time_microseconds(
    phase_ticks: u64,
    gameplay_hz: u32,
    duration_microseconds: u64,
) -> Result<u64, PhysicalAnimationOwnerErrorV1> {
    let elapsed = u128::from(phase_ticks)
        .checked_mul(1_000_000)
        .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?
        / u128::from(gameplay_hz);
    u64::try_from(elapsed % u128::from(duration_microseconds))
        .map_err(|_| PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)
}

fn bind_pose(skeleton: &NeutralSkeletonV1) -> BTreeMap<SchemaId, NeutralTransformV1> {
    skeleton
        .joints
        .iter()
        .map(|joint| (joint.joint_key.clone(), joint.bind_transform))
        .collect()
}

fn sample_clip(
    skeleton: &NeutralSkeletonV1,
    clip: &NeutralAnimationV1,
    time_microseconds: u64,
) -> Result<BTreeMap<SchemaId, NeutralTransformV1>, PhysicalAnimationOwnerErrorV1> {
    let mut poses = bind_pose(skeleton);
    for channel in &clip.channels {
        let translation = sample_translation_channel(channel, time_microseconds)?;
        poses
            .get_mut(&channel.joint_key)
            .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?
            .translation_micrometres = translation;
    }
    Ok(poses)
}

fn sample_translation_channel(
    channel: &NeutralAnimationChannelV1,
    time_microseconds: u64,
) -> Result<[i64; 3], PhysicalAnimationOwnerErrorV1> {
    let upper = channel
        .keys
        .partition_point(|key| key.time_microseconds <= time_microseconds);
    let left_index = upper.saturating_sub(1);
    let right_index = upper.min(channel.keys.len().saturating_sub(1));
    let left = channel
        .keys
        .get(left_index)
        .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?;
    let right = channel
        .keys
        .get(right_index)
        .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?;
    let translation = |value| match value {
        NeutralAnimationValueV1::Translation(value) => Ok(value),
        _ => Err(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid),
    };
    let left_value = translation(left.value)?;
    if channel.interpolation == AnimationInterpolationV1::Step
        || left.time_microseconds == right.time_microseconds
    {
        return Ok(left_value);
    }
    if channel.interpolation != AnimationInterpolationV1::Linear {
        return Err(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid);
    }
    let right_value = translation(right.value)?;
    let numerator = time_microseconds
        .checked_sub(left.time_microseconds)
        .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
    let denominator = right
        .time_microseconds
        .checked_sub(left.time_microseconds)
        .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
    let mut result = [0_i64; 3];
    for axis in 0..3 {
        let delta = i128::from(right_value[axis]) - i128::from(left_value[axis]);
        let scaled = delta
            .checked_mul(i128::from(numerator))
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        let interpolated = i128::from(left_value[axis])
            .checked_add(crate::safety_control::round_div_ties_even(
                scaled,
                i128::from(denominator),
            ))
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        result[axis] = i64::try_from(interpolated)
            .map_err(|_| PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
    }
    Ok(result)
}

fn sample_root_translation_unwrapped(
    clip: &NeutralAnimationV1,
    elapsed_us: u64,
) -> Result<[i64; 3], PhysicalAnimationOwnerErrorV1> {
    if clip.wrap_mode != AnimationWrapModeV1::Loop || clip.root_motion_intent.is_empty() {
        return Err(PhysicalAnimationOwnerErrorV1::RootMotionRejected);
    }
    let duration = clip.duration_microseconds;
    let cycles = elapsed_us / duration;
    let local_time = elapsed_us % duration;
    let first = root_translation(
        clip.root_motion_intent
            .first()
            .ok_or(PhysicalAnimationOwnerErrorV1::RootMotionRejected)?,
    )?;
    let last = root_translation(
        clip.root_motion_intent
            .last()
            .ok_or(PhysicalAnimationOwnerErrorV1::RootMotionRejected)?,
    )?;
    let local = sample_root_translation(&clip.root_motion_intent, local_time)?;
    let mut result = [0_i64; 3];
    for axis in 0..3 {
        let cycle_delta = i128::from(last[axis]) - i128::from(first[axis]);
        let value = i128::from(local[axis])
            .checked_add(
                cycle_delta
                    .checked_mul(i128::from(cycles))
                    .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?,
            )
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        result[axis] =
            i64::try_from(value).map_err(|_| PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
    }
    Ok(result)
}

fn sample_root_translation(
    keys: &[NeutralAnimationKeyV1],
    time_microseconds: u64,
) -> Result<[i64; 3], PhysicalAnimationOwnerErrorV1> {
    let upper = keys.partition_point(|key| key.time_microseconds <= time_microseconds);
    let left = keys
        .get(upper.saturating_sub(1))
        .ok_or(PhysicalAnimationOwnerErrorV1::RootMotionRejected)?;
    let right = keys
        .get(upper.min(keys.len().saturating_sub(1)))
        .ok_or(PhysicalAnimationOwnerErrorV1::RootMotionRejected)?;
    let left_value = root_translation(left)?;
    if left.time_microseconds == right.time_microseconds {
        return Ok(left_value);
    }
    let right_value = root_translation(right)?;
    let numerator = time_microseconds
        .checked_sub(left.time_microseconds)
        .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
    let denominator = right
        .time_microseconds
        .checked_sub(left.time_microseconds)
        .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
    let mut result = [0_i64; 3];
    for axis in 0..3 {
        let delta = i128::from(right_value[axis]) - i128::from(left_value[axis]);
        let scaled = delta
            .checked_mul(i128::from(numerator))
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        let value = i128::from(left_value[axis])
            .checked_add(crate::safety_control::round_div_ties_even(
                scaled,
                i128::from(denominator),
            ))
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        result[axis] =
            i64::try_from(value).map_err(|_| PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
    }
    Ok(result)
}

fn root_translation(
    key: &NeutralAnimationKeyV1,
) -> Result<[i64; 3], PhysicalAnimationOwnerErrorV1> {
    match key.value {
        NeutralAnimationValueV1::Translation(value) => Ok(value),
        _ => Err(PhysicalAnimationOwnerErrorV1::RootMotionRejected),
    }
}

fn foot_ik_corrections(
    profile: &PhysicalAnimationProfileV1,
    skeleton: &NeutralSkeletonV1,
    local_poses: &BTreeMap<SchemaId, NeutralTransformV1>,
    body_pose: PhysicsPoseV1,
    ground_height: i64,
) -> Result<Option<[i64; 2]>, PhysicalAnimationOwnerErrorV1> {
    let mut global_y = BTreeMap::<SchemaId, i64>::new();
    for joint in &skeleton.joints {
        let parent_y = joint.parent_joint_key.as_ref().map_or(Ok(0), |parent| {
            global_y
                .get(parent)
                .copied()
                .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)
        })?;
        let local_y = local_poses
            .get(&joint.joint_key)
            .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?
            .translation_micrometres[1];
        global_y.insert(
            joint.joint_key.clone(),
            parent_y
                .checked_add(local_y)
                .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?,
        );
    }
    let skeleton_root_y = body_pose.translation_micrometres[1]
        .checked_add(profile.presentation_root_offset_micrometres[1])
        .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
    let mut corrections = [0_i64; 2];
    for (index, foot) in profile.foot_joint_keys.iter().enumerate() {
        let foot_y = skeleton_root_y
            .checked_add(
                global_y
                    .get(foot)
                    .copied()
                    .ok_or(PhysicalAnimationOwnerErrorV1::ContentClosureInvalid)?,
            )
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?;
        let correction = ground_height
            .checked_sub(foot_y)
            .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)?
            .max(0);
        if correction.unsigned_abs() > profile.max_foot_ik_correction_micrometres {
            return Ok(None);
        }
        corrections[index] = correction;
    }
    Ok(Some(corrections))
}

fn checked_vec3_add(
    left: [i64; 3],
    right: [i64; 3],
) -> Result<[i64; 3], PhysicalAnimationOwnerErrorV1> {
    let values: [Option<i64>; 3] = std::array::from_fn(|axis| left[axis].checked_add(right[axis]));
    values
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .and_then(|values| values.try_into().ok())
        .ok_or(PhysicalAnimationOwnerErrorV1::ArithmeticOverflow)
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysicalAnimationOwnerErrorV1 {
    Contract(PhysicalAnimationContractErrorV1),
    TickSequenceInvalid,
    PhysicsProjectionInvalid,
    BindingMissing,
    ContentClosureInvalid,
    RootMotionRejected,
    ArithmeticOverflow,
}

impl PhysicalAnimationOwnerErrorV1 {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.diagnostic_code(),
            Self::TickSequenceInvalid => "PHYSICAL_ANIMATION_TICK_SEQUENCE_INVALID",
            Self::PhysicsProjectionInvalid => "PHYSICAL_ANIMATION_PHYSICS_PROJECTION_INVALID",
            Self::BindingMissing => "PHYSICAL_ANIMATION_BINDING_INVALID",
            Self::ContentClosureInvalid => "PHYSICAL_ANIMATION_CONTENT_CLOSURE_INVALID",
            Self::RootMotionRejected => "ANIM_ROOT_MOTION_REJECTED",
            Self::ArithmeticOverflow => "PHYSICAL_ANIMATION_ARITHMETIC_OVERFLOW",
        }
    }
}

impl Display for PhysicalAnimationOwnerErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for PhysicalAnimationOwnerErrorV1 {}

impl From<PhysicalAnimationContractErrorV1> for PhysicalAnimationOwnerErrorV1 {
    fn from(value: PhysicalAnimationContractErrorV1) -> Self {
        Self::Contract(value)
    }
}

#[cfg(test)]
mod tests;
