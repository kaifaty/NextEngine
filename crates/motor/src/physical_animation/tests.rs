use std::collections::BTreeMap;

use next_contracts::animation_content::{
    AnimationInterpolationV1, AnimationPropertyV1, AnimationWrapModeV1, NeutralAnimationChannelV1,
    NeutralAnimationKeyV1, NeutralAnimationValueV1, NeutralSkeletonJointV1,
};
use next_contracts::ids::{AssetId, ContentHash, PhysicsWorldId};
use next_contracts::physical_animation::{
    PHYSICAL_ANIMATION_SCHEMA_VERSION, PhysicalAnimationRetargetJointV1,
};
use next_contracts::physics::PhysicsBodyStateV2;

use super::*;

pub(super) struct Fixture {
    pub(super) profile: PhysicalAnimationProfileV1,
    pub(super) skeleton: NeutralSkeletonV1,
    pub(super) idle: NeutralAnimationV1,
    pub(super) locomotion: NeutralAnimationV1,
    pub(super) bindings: Vec<PhysicalAnimationBindingV1>,
    pub(super) physics: PhysicsCanonicalSnapshotV2,
}

pub(super) fn fixture() -> Fixture {
    let root = SchemaId::new("fixture.joint.root").expect("root");
    let hips = SchemaId::new("fixture.joint.hips").expect("hips");
    let foot_l = SchemaId::new("fixture.joint.foot-l").expect("foot L");
    let foot_r = SchemaId::new("fixture.joint.foot-r").expect("foot R");
    let skeleton = NeutralSkeletonV1::new(
        AssetId::from_bytes([1; 16]),
        1,
        SchemaId::new("fixture.coordinate").expect("coordinate"),
        vec![root.clone()],
        vec![
            NeutralSkeletonJointV1 {
                joint_key: root.clone(),
                parent_joint_key: None,
                bind_transform: NeutralTransformV1::translated([0; 3]),
                semantic_roles: Vec::new(),
            },
            NeutralSkeletonJointV1 {
                joint_key: hips.clone(),
                parent_joint_key: Some(root),
                bind_transform: NeutralTransformV1::translated([0, 900_000, 0]),
                semantic_roles: Vec::new(),
            },
            NeutralSkeletonJointV1 {
                joint_key: foot_l.clone(),
                parent_joint_key: Some(hips.clone()),
                bind_transform: NeutralTransformV1::translated([-150_000, -900_000, 0]),
                semantic_roles: Vec::new(),
            },
            NeutralSkeletonJointV1 {
                joint_key: foot_r.clone(),
                parent_joint_key: Some(hips.clone()),
                bind_transform: NeutralTransformV1::translated([150_000, -900_000, 0]),
                semantic_roles: Vec::new(),
            },
        ],
    )
    .expect("skeleton");
    let clip = |asset_byte: u8, clip_id: &str, middle_height: i64| {
        NeutralAnimationV1::new(
            AssetId::from_bytes([asset_byte; 16]),
            1,
            SchemaId::new(clip_id).expect("clip ID"),
            skeleton.asset_revision().expect("skeleton revision"),
            1_000_000,
            AnimationWrapModeV1::Loop,
            vec![NeutralAnimationChannelV1 {
                joint_key: hips.clone(),
                property: AnimationPropertyV1::Translation,
                interpolation: AnimationInterpolationV1::Linear,
                keys: vec![
                    NeutralAnimationKeyV1 {
                        time_microseconds: 0,
                        value: NeutralAnimationValueV1::Translation([0, 900_000, 0]),
                    },
                    NeutralAnimationKeyV1 {
                        time_microseconds: 500_000,
                        value: NeutralAnimationValueV1::Translation([0, middle_height, 0]),
                    },
                    NeutralAnimationKeyV1 {
                        time_microseconds: 1_000_000,
                        value: NeutralAnimationValueV1::Translation([0, 900_000, 0]),
                    },
                ],
            }],
            Vec::new(),
            Vec::new(),
        )
        .expect("clip")
    };
    let idle = clip(2, "fixture.animation.idle", 905_000);
    let locomotion_pose = clip(3, "fixture.animation.locomotion", 930_000);
    let locomotion = NeutralAnimationV1::new(
        locomotion_pose.asset_id,
        2,
        locomotion_pose.clip_id.clone(),
        locomotion_pose.skeleton_revision,
        locomotion_pose.duration_microseconds,
        locomotion_pose.wrap_mode,
        locomotion_pose.channels.clone(),
        locomotion_pose.markers.clone(),
        (0_u64..=30)
            .map(|tick| NeutralAnimationKeyV1 {
                time_microseconds: tick * 1_000_000 / 30,
                value: NeutralAnimationValueV1::Translation([
                    0,
                    0,
                    i64::try_from(tick).expect("tick fits") * 100_000,
                ]),
            })
            .collect(),
    )
    .expect("rooted locomotion");
    let mut retarget_joints = skeleton
        .joints
        .iter()
        .map(|joint| PhysicalAnimationRetargetJointV1 {
            source_joint_key: joint.joint_key.clone(),
            target_joint_key: joint.joint_key.clone(),
        })
        .collect::<Vec<_>>();
    retarget_joints.sort();
    let profile = PhysicalAnimationProfileV1 {
        schema_version: PHYSICAL_ANIMATION_SCHEMA_VERSION,
        profile_id: SchemaId::new("fixture.physical-animation").expect("profile"),
        skeleton_revision: skeleton.asset_revision().expect("skeleton revision"),
        idle_clip_revision: idle.asset_revision().expect("idle revision"),
        locomotion_clip_revision: locomotion.asset_revision().expect("locomotion revision"),
        gameplay_hz: 30,
        locomotion_threshold_micrometres_per_tick: 1,
        presentation_root_offset_micrometres: [0, -900_000, 0],
        presentation_motion_joint_key: hips,
        foot_joint_keys: [foot_l, foot_r],
        max_foot_ik_correction_micrometres: 100_000,
        retarget_joints,
    };
    let bindings = [4_u8, 5]
        .map(|byte| PhysicalAnimationBindingV1 {
            subject_id: PersistentId::from_bytes([byte; 16]),
            body_id: next_contracts::physics::PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([byte; 16]),
                body_slot: 0,
            },
        })
        .to_vec();
    let body_states = bindings
        .iter()
        .map(|binding| {
            (
                binding.body_id,
                PhysicsBodyStateV2 {
                    body_id: binding.body_id,
                    body_revision: 0,
                    pose: PhysicsPoseV1 {
                        translation_micrometres: [0, 900_000, 0],
                        ..PhysicsPoseV1::default()
                    },
                    linear_velocity_micrometres_per_second: [0; 3],
                    angular_velocity_q16: [0; 3],
                    active: true,
                    sleep_counter: 0,
                },
            )
        })
        .collect();
    let physics = PhysicsCanonicalSnapshotV2 {
        schema_version: next_contracts::physics::PHYSICS_SNAPSHOT_SCHEMA_VERSION,
        world_id: PhysicsWorldId::from_bytes([7; 16]),
        world_revision: 0,
        checkpoint_revision: 0,
        physics_tick: 0,
        world_descriptor_hash: ContentHash::from_bytes([1; 32]),
        catalog_hash: ContentHash::from_bytes([2; 32]),
        tick_rate_profile_hash: ContentHash::from_bytes([3; 32]),
        authoritative_numeric_profile_hash: ContentHash::from_bytes([4; 32]),
        physics_quantization_profile_hash: ContentHash::from_bytes([5; 32]),
        physics_limits_profile_hash: ContentHash::from_bytes([6; 32]),
        sorted_body_states: body_states,
        sorted_contact_continuity_states: BTreeMap::new(),
        sorted_solver_continuation_states: BTreeMap::new(),
    };
    Fixture {
        profile,
        skeleton,
        idle,
        locomotion,
        bindings,
        physics,
    }
}

#[test]
fn actual_capsule_displacement_selects_graph_per_shared_archetype() {
    let fixture = fixture();
    let mut owner = PhysicalAnimationOwnerV1::activate(
        fixture.profile,
        fixture.skeleton,
        fixture.idle,
        fixture.locomotion,
        fixture.bindings.clone(),
        &fixture.physics,
    )
    .expect("activate");
    let mut next = fixture.physics.clone();
    next.physics_tick = 2;
    next.sorted_body_states
        .get_mut(&fixture.bindings[0].body_id)
        .expect("player")
        .pose
        .translation_micrometres[0] = 10;
    owner.advance(&fixture.physics, &next, 1).expect("advance");
    assert_eq!(
        owner.snapshot().records[0].graph_state,
        PhysicalAnimationGraphStateV1::Locomotion,
    );
    assert_eq!(owner.snapshot().records[0].phase_ticks, 0);
    assert_eq!(
        owner.snapshot().records[1].graph_state,
        PhysicalAnimationGraphStateV1::Idle,
    );
    assert_eq!(owner.snapshot().records[1].phase_ticks, 1);

    let restored = PhysicalAnimationOwnerV1::restore(
        owner.profile().clone(),
        owner.skeleton.clone(),
        owner.idle_clip.clone(),
        owner.locomotion_clip.clone(),
        fixture.bindings,
        owner.snapshot().clone(),
        &next,
        1,
    )
    .expect("restore");
    assert_eq!(restored.snapshot(), owner.snapshot());
}

#[test]
fn root_curve_proposes_exact_future_intent_without_mutation() {
    let fixture = fixture();
    let owner = PhysicalAnimationOwnerV1::activate(
        fixture.profile,
        fixture.skeleton,
        fixture.idle,
        fixture.locomotion,
        fixture.bindings.clone(),
        &fixture.physics,
    )
    .expect("activate");
    let snapshot = owner.snapshot().clone();
    let intent = owner
        .root_motion_intent(
            fixture.bindings[0].subject_id,
            17,
            SchemaId::new(next_contracts::physical_animation::ROOT_MOTION_MOVE_STARTED_PHASE_ID)
                .expect("phase"),
            &fixture.physics,
        )
        .expect("root intent");
    assert_eq!(intent.intent_sequence, 17);
    assert_eq!(intent.source_animation_tick, 0);
    assert_eq!(intent.expected_intent_state_revision, 0);
    assert_eq!(intent.expected_body_revision, 0);
    assert_eq!(intent.quantized_local_translation, [0, 0, 100_000]);
    assert_eq!(intent.quantized_local_yaw, 0);
    assert_eq!(owner.snapshot(), &snapshot);
    assert_eq!(
        fixture.physics.sorted_body_states[&fixture.bindings[0].body_id]
            .pose
            .translation_micrometres[2],
        0
    );
}

#[test]
fn sampling_and_ik_fallbacks_do_not_mutate_owner_or_physics() {
    let fixture = fixture();
    let owner = PhysicalAnimationOwnerV1::activate(
        fixture.profile,
        fixture.skeleton,
        fixture.idle,
        fixture.locomotion,
        fixture.bindings.clone(),
        &fixture.physics,
    )
    .expect("activate");
    let snapshot = owner.snapshot().clone();
    let physics = fixture.physics.clone();
    let primary = owner
        .pose(
            fixture.bindings[0].subject_id,
            &physics,
            Some(0),
            PhysicalAnimationPresentationAvailabilityV1::FULL,
        )
        .expect("primary pose");
    assert_eq!(
        primary.projection_mode,
        PhysicalAnimationProjectionModeV1::SampledWithFootIk,
    );
    let fallback = owner
        .pose(
            fixture.bindings[0].subject_id,
            &physics,
            Some(0),
            PhysicalAnimationPresentationAvailabilityV1::BIND_POSE_FALLBACK,
        )
        .expect("fallback pose");
    assert_eq!(
        fallback.projection_mode,
        PhysicalAnimationProjectionModeV1::BindPoseFallback,
    );
    let ik_failure = owner
        .pose(
            fixture.bindings[0].subject_id,
            &physics,
            Some(2_000_000),
            PhysicalAnimationPresentationAvailabilityV1::FULL,
        )
        .expect("bounded IK fallback");
    assert_eq!(
        ik_failure.projection_mode,
        PhysicalAnimationProjectionModeV1::SampledWithoutFootIk,
    );
    assert_eq!(owner.snapshot(), &snapshot);
    assert_eq!(physics, fixture.physics);
}

#[test]
fn interpolation_uses_sign_symmetric_ties_to_even() {
    assert_eq!(crate::safety_control::round_div_ties_even(5, 2), 2);
    assert_eq!(crate::safety_control::round_div_ties_even(7, 2), 4);
    assert_eq!(crate::safety_control::round_div_ties_even(-5, 2), -2);
    assert_eq!(crate::safety_control::round_div_ties_even(-7, 2), -4);
}
