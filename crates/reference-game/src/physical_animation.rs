use next_contracts::animation_content::{
    NeutralAnimationContentErrorV1, NeutralAnimationV1, NeutralSkeletonV1,
};
use next_contracts::ids::{AssetId, SchemaId};
use next_contracts::physical_animation::{
    PHYSICAL_ANIMATION_SCHEMA_VERSION, PhysicalAnimationBindingV1,
    PhysicalAnimationContractErrorV1, PhysicalAnimationProfileV1, PhysicalAnimationRetargetJointV1,
    PhysicalAnimationSnapshotV1,
};
use next_contracts::physics::{PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2};
use next_motor::{PhysicalAnimationOwnerErrorV1, PhysicalAnimationOwnerV1};

use crate::{ReferenceGameError, ReferenceGameSession};

const REFERENCE_HUMANOID_SKELETON_ASSET_ID: AssetId = AssetId::from_bytes([0xb1; 16]);
const REFERENCE_HUMANOID_IDLE_CLIP_ASSET_ID: AssetId = AssetId::from_bytes([0xb2; 16]);
const REFERENCE_HUMANOID_LOCOMOTION_CLIP_ASSET_ID: AssetId = AssetId::from_bytes([0xb9; 16]);
const REFERENCE_PHYSICAL_ANIMATION_PROFILE_ID: &str =
    "nextengine.reference-alpha.physical-animation.humanoid-v1";
const REFERENCE_HUMANOID_HIPS_JOINT_ID: &str =
    "nextengine.reference-alpha.humanoid.cc0.v1.joint.hips";
const REFERENCE_HUMANOID_FOOT_L_JOINT_ID: &str =
    "nextengine.reference-alpha.humanoid.cc0.v1.joint.foot-l";
const REFERENCE_HUMANOID_FOOT_R_JOINT_ID: &str =
    "nextengine.reference-alpha.humanoid.cc0.v1.joint.foot-r";

struct ReferencePhysicalAnimationContentV1 {
    profile: PhysicalAnimationProfileV1,
    skeleton: NeutralSkeletonV1,
    idle_clip: NeutralAnimationV1,
    locomotion_clip: NeutralAnimationV1,
    bindings: Vec<PhysicalAnimationBindingV1>,
}

pub fn reference_physical_animation_owner(
    fixture: &ReferenceGameSession,
    physics: &PhysicsCanonicalSnapshotV2,
) -> Result<PhysicalAnimationOwnerV1, ReferenceGameError> {
    let content = reference_physical_animation_content(fixture)?;
    Ok(PhysicalAnimationOwnerV1::activate(
        content.profile,
        content.skeleton,
        content.idle_clip,
        content.locomotion_clip,
        content.bindings,
        physics,
    )?)
}

pub fn restore_reference_physical_animation_owner(
    fixture: &ReferenceGameSession,
    snapshot: PhysicalAnimationSnapshotV1,
    physics: &PhysicsCanonicalSnapshotV2,
    next_simulation_tick: u64,
) -> Result<PhysicalAnimationOwnerV1, ReferenceGameError> {
    let content = reference_physical_animation_content(fixture)?;
    Ok(PhysicalAnimationOwnerV1::restore(
        content.profile,
        content.skeleton,
        content.idle_clip,
        content.locomotion_clip,
        content.bindings,
        snapshot,
        physics,
        next_simulation_tick,
    )?)
}

fn reference_physical_animation_content(
    fixture: &ReferenceGameSession,
) -> Result<ReferencePhysicalAnimationContentV1, ReferenceGameError> {
    let skeleton = fixture
        .activated_project
        .neutral_skeletons
        .iter()
        .find(|record| record.asset_id == REFERENCE_HUMANOID_SKELETON_ASSET_ID)
        .cloned()
        .ok_or_else(content_missing)?;
    let idle_clip = fixture
        .activated_project
        .neutral_animations
        .iter()
        .find(|record| record.asset_id == REFERENCE_HUMANOID_IDLE_CLIP_ASSET_ID)
        .cloned()
        .ok_or_else(content_missing)?;
    let locomotion_clip = fixture
        .activated_project
        .neutral_animations
        .iter()
        .find(|record| record.asset_id == REFERENCE_HUMANOID_LOCOMOTION_CLIP_ASSET_ID)
        .cloned()
        .ok_or_else(content_missing)?;

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
        profile_id: SchemaId::new(REFERENCE_PHYSICAL_ANIMATION_PROFILE_ID)?,
        skeleton_revision: content_result(skeleton.asset_revision())?,
        idle_clip_revision: content_result(idle_clip.asset_revision())?,
        locomotion_clip_revision: content_result(locomotion_clip.asset_revision())?,
        gameplay_hz: fixture.bootstrap.tick_rate_profile.gameplay_hz,
        locomotion_threshold_micrometres_per_tick: 1,
        presentation_root_offset_micrometres: [0, -900_000, 0],
        presentation_motion_joint_key: SchemaId::new(REFERENCE_HUMANOID_HIPS_JOINT_ID)?,
        foot_joint_keys: [
            SchemaId::new(REFERENCE_HUMANOID_FOOT_L_JOINT_ID)?,
            SchemaId::new(REFERENCE_HUMANOID_FOOT_R_JOINT_ID)?,
        ],
        max_foot_ik_correction_micrometres: 100_000,
        retarget_joints,
    };
    let mut bindings = vec![
        PhysicalAnimationBindingV1 {
            subject_id: fixture.body_id,
            body_id: fixture.physics_body_id,
        },
        PhysicalAnimationBindingV1 {
            subject_id: fixture.npc_character_id,
            body_id: PhysicsBodyIdV1 {
                subject_id: fixture.npc_character_id,
                body_slot: 0,
            },
        },
    ];
    bindings.sort();
    Ok(ReferencePhysicalAnimationContentV1 {
        profile,
        skeleton,
        idle_clip,
        locomotion_clip,
        bindings,
    })
}

fn content_result<T>(
    result: Result<T, NeutralAnimationContentErrorV1>,
) -> Result<T, ReferenceGameError> {
    result
        .map_err(PhysicalAnimationContractErrorV1::from)
        .map_err(PhysicalAnimationOwnerErrorV1::from)
        .map_err(ReferenceGameError::from)
}

fn content_missing() -> ReferenceGameError {
    PhysicalAnimationOwnerErrorV1::ContentClosureInvalid.into()
}
