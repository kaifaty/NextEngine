use super::*;
use crate::animation_content::{
    AnimationWrapModeV1, NeutralAnimationChannelV1, NeutralAnimationKeyV1, NeutralAnimationValueV1,
    NeutralSkeletonJointV1, NeutralTransformV1,
};
use crate::ids::AssetId;

fn fixture() -> (
    PhysicalAnimationProfileV1,
    NeutralSkeletonV1,
    NeutralAnimationV1,
    NeutralAnimationV1,
    Vec<PhysicalAnimationBindingV1>,
) {
    let root = SchemaId::new("fixture.joint.root").expect("root");
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
                joint_key: foot_l.clone(),
                parent_joint_key: Some(root.clone()),
                bind_transform: NeutralTransformV1::translated([-1, -2, 0]),
                semantic_roles: Vec::new(),
            },
            NeutralSkeletonJointV1 {
                joint_key: foot_r.clone(),
                parent_joint_key: Some(root.clone()),
                bind_transform: NeutralTransformV1::translated([1, -2, 0]),
                semantic_roles: Vec::new(),
            },
        ],
    )
    .expect("skeleton");
    let clip = |asset_byte, clip_id: &str, height| {
        NeutralAnimationV1::new(
            AssetId::from_bytes([asset_byte; 16]),
            1,
            SchemaId::new(clip_id).expect("clip ID"),
            skeleton.asset_revision().expect("skeleton revision"),
            1_000_000,
            AnimationWrapModeV1::Loop,
            vec![NeutralAnimationChannelV1 {
                joint_key: root.clone(),
                property: AnimationPropertyV1::Translation,
                interpolation: AnimationInterpolationV1::Linear,
                keys: vec![
                    NeutralAnimationKeyV1 {
                        time_microseconds: 0,
                        value: NeutralAnimationValueV1::Translation([0, height, 0]),
                    },
                    NeutralAnimationKeyV1 {
                        time_microseconds: 1_000_000,
                        value: NeutralAnimationValueV1::Translation([0, height, 0]),
                    },
                ],
            }],
            Vec::new(),
            Vec::new(),
        )
        .expect("clip")
    };
    let idle = clip(2, "fixture.animation.idle", 0);
    let locomotion = clip(3, "fixture.animation.locomotion", 1);
    let profile = PhysicalAnimationProfileV1 {
        schema_version: PHYSICAL_ANIMATION_SCHEMA_VERSION,
        profile_id: SchemaId::new("fixture.physical-animation").expect("profile ID"),
        skeleton_revision: skeleton.asset_revision().expect("skeleton revision"),
        idle_clip_revision: idle.asset_revision().expect("idle revision"),
        locomotion_clip_revision: locomotion.asset_revision().expect("locomotion revision"),
        gameplay_hz: 30,
        locomotion_threshold_micrometres_per_tick: 1,
        presentation_root_offset_micrometres: [0, -900_000, 0],
        presentation_motion_joint_key: root,
        foot_joint_keys: [foot_l, foot_r],
        max_foot_ik_correction_micrometres: 100_000,
        retarget_joints: skeleton
            .joints
            .iter()
            .map(|joint| PhysicalAnimationRetargetJointV1 {
                source_joint_key: joint.joint_key.clone(),
                target_joint_key: joint.joint_key.clone(),
            })
            .collect::<Vec<_>>(),
    };
    let mut profile = profile;
    profile
        .retarget_joints
        .sort_by(|left, right| left.source_joint_key.cmp(&right.source_joint_key));
    let bindings = vec![
        PhysicalAnimationBindingV1 {
            subject_id: PersistentId::from_bytes([4; 16]),
            body_id: PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([4; 16]),
                body_slot: 0,
            },
        },
        PhysicalAnimationBindingV1 {
            subject_id: PersistentId::from_bytes([5; 16]),
            body_id: PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([5; 16]),
                body_slot: 0,
            },
        },
    ];
    (profile, skeleton, idle, locomotion, bindings)
}

#[test]
fn profile_and_snapshot_round_trip_canonically() {
    let (profile, skeleton, idle, locomotion, bindings) = fixture();
    profile
        .validate_against_content(&skeleton, &idle, &locomotion)
        .expect("content closure");
    let profile_bytes = profile.canonical_bytes().expect("profile bytes");
    assert_eq!(
        PhysicalAnimationProfileV1::from_canonical_bytes(
            &profile_bytes,
            CanonicalDecodeLimits::default(),
        )
        .expect("decode profile"),
        profile,
    );
    let snapshot = PhysicalAnimationSnapshotV1::initial(&profile, &bindings).expect("snapshot");
    let bytes = snapshot.canonical_bytes().expect("snapshot bytes");
    let decoded =
        PhysicalAnimationSnapshotV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("decode snapshot");
    assert_eq!(decoded, snapshot);
    decoded
        .validate_against(&profile, &bindings)
        .expect("snapshot closure");
}

#[test]
fn profile_admits_forward_locomotion_root_but_keeps_retarget_bounded() {
    let (mut profile, skeleton, idle, locomotion, _) = fixture();
    let rooted_locomotion = NeutralAnimationV1::new(
        locomotion.asset_id,
        2,
        locomotion.clip_id.clone(),
        locomotion.skeleton_revision,
        locomotion.duration_microseconds,
        locomotion.wrap_mode,
        locomotion.channels.clone(),
        locomotion.markers.clone(),
        vec![
            NeutralAnimationKeyV1 {
                time_microseconds: 0,
                value: NeutralAnimationValueV1::Translation([0, 0, 0]),
            },
            NeutralAnimationKeyV1 {
                time_microseconds: locomotion.duration_microseconds,
                value: NeutralAnimationValueV1::Translation([0, 0, 3_000_000]),
            },
        ],
    )
    .expect("rooted locomotion");
    profile.locomotion_clip_revision = rooted_locomotion.asset_revision().expect("rooted revision");
    profile
        .validate_against_content(&skeleton, &idle, &rooted_locomotion)
        .expect("bounded forward root motion is admitted");
    profile.retarget_joints[0].target_joint_key =
        SchemaId::new("fixture.joint.changed").expect("changed joint");
    assert_eq!(
        profile
            .validate()
            .expect_err("non-identity retarget is outside R5a")
            .diagnostic_code(),
        "PHYSICAL_ANIMATION_PROFILE_INVALID",
    );
}

#[test]
fn root_motion_intent_round_trips_and_rejects_pose_authority() {
    let intent = RootMotionIntentV1 {
        schema_version: ROOT_MOTION_INTENT_SCHEMA_VERSION,
        subject_id: PersistentId::from_bytes([8; 16]),
        intent_sequence: 7,
        source_graph_hash: ContentHash::from_bytes([1; 32]),
        source_clip_hash: ContentHash::from_bytes([2; 32]),
        source_action_or_ability_phase_id: SchemaId::new(ROOT_MOTION_MOVE_PERFORMED_PHASE_ID)
            .expect("phase"),
        source_animation_tick: 9,
        interval_us: 33_333,
        quantized_local_translation: [0, 0, 100_000],
        quantized_local_yaw: 0,
        locomotion_profile_hash: capsule_root_motion_profile_hash_v1(30).expect("profile"),
        expected_intent_state_revision: 9,
        expected_body_revision: 11,
    };
    let bytes = intent.canonical_payload_bytes().expect("intent bytes");
    assert_eq!(
        RootMotionIntentV1::from_canonical_payload_bytes(&bytes, CanonicalDecodeLimits::default(),)
            .expect("decode intent"),
        intent,
    );

    let mut invalid = intent;
    invalid.quantized_local_yaw = 1;
    assert_eq!(
        invalid
            .validate()
            .expect_err("bounded R5c has no yaw authority")
            .diagnostic_code(),
        "ANIM_ROOT_MOTION_REJECTED",
    );
}

#[test]
fn snapshot_rejects_reorder_and_profile_drift() {
    let (profile, _, _, _, bindings) = fixture();
    let mut snapshot = PhysicalAnimationSnapshotV1::initial(&profile, &bindings).expect("snapshot");
    snapshot.records.swap(0, 1);
    assert_eq!(
        snapshot
            .validate_against(&profile, &bindings)
            .expect_err("records must be sorted")
            .diagnostic_code(),
        "PHYSICAL_ANIMATION_SNAPSHOT_CLOSURE_INVALID",
    );
    let mut snapshot = PhysicalAnimationSnapshotV1::initial(&profile, &bindings).expect("snapshot");
    snapshot.profile_revision = ContentHash::from_bytes([9; 32]);
    assert_eq!(
        snapshot
            .validate_against(&profile, &bindings)
            .expect_err("profile drift")
            .diagnostic_code(),
        "PHYSICAL_ANIMATION_SNAPSHOT_CLOSURE_INVALID",
    );
}
