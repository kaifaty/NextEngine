use next_contracts::animation_content::{
    NeutralAnimationContentErrorV1, NeutralAnimationV1, NeutralSkeletonV1,
};
use next_contracts::ids::{AssetId, ContentHash, SchemaId};
use next_contracts::input::{CORE_MOVE_ACTION_ID, PlayerActionPhaseV1, PlayerActionValueV1};
use next_contracts::physical_animation::{
    PHYSICAL_ANIMATION_SCHEMA_VERSION, PhysicalAnimationBindingV1,
    PhysicalAnimationContractErrorV1, PhysicalAnimationProfileV1, PhysicalAnimationRetargetJointV1,
    PhysicalAnimationSnapshotV1, ROOT_MOTION_MOVE_PERFORMED_PHASE_ID,
    ROOT_MOTION_MOVE_STARTED_PHASE_ID,
};
use next_contracts::physics::{PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2};
use next_contracts::project::ActivatedProjectV8;
use next_motor::{PhysicalAnimationOwnerErrorV1, PhysicalAnimationOwnerV1};
use next_player::ResolvedPlayerInputFrameV1;

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

pub(crate) fn reference_root_motion_command(
    fixture: &ReferenceGameSession,
    owner: &PhysicalAnimationOwnerV1,
    resolved: Option<&ResolvedPlayerInputFrameV1>,
    suppress_movement: bool,
    target_tick: u64,
    physics: &PhysicsCanonicalSnapshotV2,
) -> Result<Option<next_contracts::command::WorldCommand>, ReferenceGameError> {
    let Some((sequence, phase, direction_q15)) =
        resolved
            .filter(|_| !suppress_movement)
            .and_then(|resolved| {
                resolved
                    .frame
                    .actions
                    .iter()
                    .find(|action| action.action_id.as_str() == CORE_MOVE_ACTION_ID)
                    .and_then(|action| match action.value {
                        PlayerActionValueV1::Vector2Q15(direction) => {
                            Some((resolved.sample.source_sequence, action.phase, direction))
                        }
                        _ => None,
                    })
            })
    else {
        return Ok(None);
    };
    if direction_q15 != [0, 32_767] {
        return Ok(None);
    }
    let phase_id = match phase {
        PlayerActionPhaseV1::Started => ROOT_MOTION_MOVE_STARTED_PHASE_ID,
        PlayerActionPhaseV1::Performed => ROOT_MOTION_MOVE_PERFORMED_PHASE_ID,
        _ => return Ok(None),
    };
    let intent =
        owner.root_motion_intent(fixture.body_id, sequence, SchemaId::new(phase_id)?, physics)?;
    Ok(Some(next_contracts::command::WorldCommand::root_motion(
        fixture.movement_stream_id,
        fixture.principal.clone(),
        sequence,
        target_tick,
        fixture.body_id,
        intent,
    )?))
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

    let profile = reference_physical_animation_profile(
        &skeleton,
        &idle_clip,
        &locomotion_clip,
        fixture.bootstrap.tick_rate_profile.gameplay_hz,
    )?;
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
    let body_schema_hash = fixture
        .activated_project
        .body_schema_asset
        .body_schema
        .schema_hash()?;
    let body_schema_asset_hash = fixture
        .activated_project
        .body_schema_asset
        .record_sha256()?;
    if fixture.body_projections.body_schema_asset_revision.asset_id
        != fixture.activated_project.body_schema_asset.asset_id
        || fixture
            .body_projections
            .body_schema_asset_revision
            .record_sha256
            != body_schema_asset_hash
        || bindings.iter().any(|binding| {
            fixture
                .body_projections
                .roots_for(binding.subject_id)
                .is_none_or(|roots| roots.body_schema_hash != body_schema_hash)
        })
    {
        return Err(ReferenceGameError::BodyProjectionInvalid);
    }
    Ok(ReferencePhysicalAnimationContentV1 {
        profile,
        skeleton,
        idle_clip,
        locomotion_clip,
        bindings,
    })
}

pub(crate) fn reference_root_motion_source_hashes(
    project: &ActivatedProjectV8,
    gameplay_hz: u32,
) -> Result<(ContentHash, ContentHash), ReferenceGameError> {
    let skeleton = project
        .neutral_skeletons
        .iter()
        .find(|record| record.asset_id == REFERENCE_HUMANOID_SKELETON_ASSET_ID)
        .ok_or_else(content_missing)?;
    let idle_clip = project
        .neutral_animations
        .iter()
        .find(|record| record.asset_id == REFERENCE_HUMANOID_IDLE_CLIP_ASSET_ID)
        .ok_or_else(content_missing)?;
    let locomotion_clip = project
        .neutral_animations
        .iter()
        .find(|record| record.asset_id == REFERENCE_HUMANOID_LOCOMOTION_CLIP_ASSET_ID)
        .ok_or_else(content_missing)?;
    let profile =
        reference_physical_animation_profile(skeleton, idle_clip, locomotion_clip, gameplay_hz)?;
    profile
        .validate_against_content(skeleton, idle_clip, locomotion_clip)
        .map_err(PhysicalAnimationOwnerErrorV1::from)?;
    Ok((
        profile
            .revision()
            .map_err(PhysicalAnimationOwnerErrorV1::from)?,
        profile.locomotion_clip_revision.record_sha256,
    ))
}

fn reference_physical_animation_profile(
    skeleton: &NeutralSkeletonV1,
    idle_clip: &NeutralAnimationV1,
    locomotion_clip: &NeutralAnimationV1,
    gameplay_hz: u32,
) -> Result<PhysicalAnimationProfileV1, ReferenceGameError> {
    let mut retarget_joints = skeleton
        .joints
        .iter()
        .map(|joint| PhysicalAnimationRetargetJointV1 {
            source_joint_key: joint.joint_key.clone(),
            target_joint_key: joint.joint_key.clone(),
        })
        .collect::<Vec<_>>();
    retarget_joints.sort();
    Ok(PhysicalAnimationProfileV1 {
        schema_version: PHYSICAL_ANIMATION_SCHEMA_VERSION,
        profile_id: SchemaId::new(REFERENCE_PHYSICAL_ANIMATION_PROFILE_ID)?,
        skeleton_revision: content_result(skeleton.asset_revision())?,
        idle_clip_revision: content_result(idle_clip.asset_revision())?,
        locomotion_clip_revision: content_result(locomotion_clip.asset_revision())?,
        gameplay_hz,
        locomotion_threshold_micrometres_per_tick: 1,
        presentation_root_offset_micrometres: [0, -900_000, 0],
        presentation_motion_joint_key: SchemaId::new(REFERENCE_HUMANOID_HIPS_JOINT_ID)?,
        foot_joint_keys: [
            SchemaId::new(REFERENCE_HUMANOID_FOOT_L_JOINT_ID)?,
            SchemaId::new(REFERENCE_HUMANOID_FOOT_R_JOINT_ID)?,
        ],
        max_foot_ik_correction_micrometres: 100_000,
        retarget_joints,
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
