use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::motor::{
    HUMANOID_REFERENCE_HORIZON_OFFSETS, HUMANOID_REFERENCE_TRACKER_ACTIONS,
    HUMANOID_REFERENCE_TRACKER_CHANNELS, MOTOR_REFERENCE_TRACKING_PROFILE_V1_SCHEMA_VERSION,
    MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION,
    MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V3_SCHEMA_VERSION, MotorReferenceTrackingProfileV1,
    MotorRewardComponentV1, MotorTrainingEnvironmentManifestV2, MotorTrainingEnvironmentManifestV3,
    STAGE0_MOTOR_HZ, STAGE0_PHYSICS_HZ,
};

use crate::{CompiledBodySchemaV2, TrainingEnvironmentError};

pub const REFERENCE_TRACKING_ENVIRONMENT_PROFILE_ID: &str =
    "nextengine.motor.env.humanoid-reference-tracker.v1";
pub const REFERENCE_TRACKER_PROFILE_DOCUMENT_SHA256: [u8; 32] = [
    0x50, 0xf5, 0x2b, 0x63, 0xbc, 0xf6, 0xb5, 0xa2, 0xb0, 0xc4, 0x20, 0x1f, 0x17, 0x54, 0xa4, 0xd4,
    0x57, 0xb8, 0x97, 0xb8, 0xd8, 0x94, 0xa5, 0xc5, 0xbf, 0x68, 0x45, 0x8f, 0x97, 0xe7, 0x69, 0xf1,
];

const BODY_SCHEMA_SHA256: [u8; 32] = [
    0xe2, 0x46, 0x0e, 0x7d, 0xc4, 0xaf, 0x93, 0x53, 0x8a, 0xe4, 0xb0, 0xb6, 0x8a, 0x9e, 0x1b, 0xf7,
    0x4b, 0x2b, 0x79, 0x90, 0x16, 0x1e, 0x08, 0xe4, 0x41, 0xd5, 0x86, 0x87, 0xe9, 0x53, 0xc4, 0x3d,
];
const COMPILED_DESCRIPTOR_SHA256: [u8; 32] = [
    0xb6, 0xf8, 0xb1, 0x26, 0x0b, 0x24, 0xad, 0x40, 0x14, 0x53, 0x94, 0x2c, 0x9a, 0x43, 0x03, 0xd9,
    0x9c, 0xe3, 0x78, 0xa8, 0xf7, 0x1c, 0x64, 0x90, 0xdb, 0x79, 0xbb, 0x78, 0xd8, 0x5a, 0x17, 0x82,
];
const CORPUS_PROFILE_SHA256: [u8; 32] = [
    0x2f, 0x35, 0xf8, 0x35, 0xea, 0x58, 0xd9, 0x1c, 0xa3, 0x7c, 0x00, 0x60, 0x01, 0x4f, 0x11, 0x2b,
    0xc2, 0x77, 0x3d, 0xc1, 0x69, 0x54, 0x87, 0x38, 0xed, 0x2f, 0xa5, 0x46, 0x48, 0x37, 0xbd, 0xec,
];
const CORPUS_MANIFEST_SHA256: [u8; 32] = [
    0x0f, 0x1c, 0xe1, 0x47, 0x05, 0x1b, 0x41, 0xc7, 0x38, 0x0b, 0x56, 0x68, 0x44, 0xae, 0xcb, 0x69,
    0x61, 0x2f, 0xd5, 0x01, 0x46, 0x6d, 0x18, 0x48, 0xbd, 0x91, 0x37, 0xd7, 0x55, 0x58, 0x1d, 0xf5,
];
const OBSERVATION_LAYOUT_SHA256: [u8; 32] = [
    0x9c, 0xc3, 0xa9, 0x98, 0xb0, 0xc2, 0x4e, 0x8a, 0xfd, 0x81, 0x40, 0xc3, 0x88, 0x49, 0xec, 0x09,
    0x6e, 0x78, 0x15, 0x58, 0x7e, 0x88, 0xe6, 0xe0, 0xff, 0x48, 0x73, 0xaf, 0x0a, 0x34, 0xec, 0xb9,
];
const ACTION_LAYOUT_SHA256: [u8; 32] = [
    0xd7, 0x0e, 0x3f, 0x93, 0x81, 0x80, 0xd2, 0xb4, 0x88, 0xa6, 0x79, 0xb4, 0xc2, 0xab, 0x38, 0xf5,
    0xe2, 0x10, 0x68, 0x10, 0x36, 0xca, 0xd8, 0x25, 0xb3, 0x72, 0xac, 0x7c, 0xbf, 0x60, 0x9b, 0x4a,
];
const RESET_PROFILE_SHA256: [u8; 32] = [
    0xfc, 0x92, 0x4c, 0x18, 0x6e, 0xbd, 0x77, 0x73, 0x12, 0xa7, 0x39, 0x7b, 0xa6, 0xaf, 0x26, 0x91,
    0x83, 0xbc, 0x78, 0xbc, 0x53, 0x0d, 0x91, 0xb1, 0x0f, 0xe2, 0xf5, 0x12, 0xae, 0xc6, 0xe2, 0x33,
];
const REWARD_PROFILE_SHA256: [u8; 32] = [
    0xd2, 0x8a, 0x43, 0x2b, 0xb3, 0xb9, 0x51, 0x43, 0x39, 0xe7, 0x59, 0xc6, 0xab, 0x35, 0x0c, 0xba,
    0x71, 0x97, 0x8b, 0x2e, 0xf6, 0xf9, 0xb2, 0xd7, 0x2f, 0x19, 0x76, 0x2f, 0xed, 0xb2, 0xf5, 0x3a,
];
const TERMINATION_PROFILE_SHA256: [u8; 32] = [
    0xc6, 0xc5, 0x22, 0xed, 0x26, 0x86, 0xda, 0x1b, 0xeb, 0x6c, 0xd0, 0x18, 0x2e, 0x1e, 0x61, 0x64,
    0xdb, 0x9a, 0xc4, 0x5b, 0xe8, 0x21, 0xfd, 0x23, 0x45, 0x03, 0xbf, 0xd1, 0xa5, 0x45, 0xc0, 0x95,
];
const RNG_PROFILE_SHA256: [u8; 32] = [
    0xc3, 0x8f, 0x72, 0x06, 0x38, 0x96, 0x8f, 0x29, 0x0d, 0x94, 0x09, 0x02, 0x38, 0x9a, 0xa4, 0x6d,
    0xe9, 0xb5, 0x13, 0xc0, 0x0e, 0x9f, 0xbb, 0xe8, 0x29, 0xf3, 0xe2, 0x1d, 0x44, 0x29, 0xae, 0xa9,
];

pub const REFERENCE_REWARD_COMPONENT_IDS: [&str; 13] = [
    "reward.reference-root-orientation",
    "reward.reference-root-height",
    "reward.reference-root-linear-velocity",
    "reward.reference-root-angular-velocity",
    "reward.reference-joint-pose",
    "reward.reference-joint-velocity",
    "reward.reference-center-of-mass",
    "reward.reference-effectors",
    "reward.reference-contacts",
    "reward.contacting-sole-slip-cost",
    "reward.normalized-applied-effort-cost",
    "reward.applied-target-rate-cost",
    "reward.terminal-failure",
];
pub const REFERENCE_REWARD_COEFFICIENTS_Q16: [i64; 13] = [
    65_536, 32_768, 32_768, 16_384, 131_072, 32_768, 32_768, 65_536, 49_152, -6_554, -1_311,
    -3_277, -655_360,
];

pub fn biomechanics_reference_tracking_profile_v1(
    compiled: &CompiledBodySchemaV2,
) -> Result<MotorReferenceTrackingProfileV1, TrainingEnvironmentError> {
    if compiled.body_schema_hash != hash(BODY_SCHEMA_SHA256)
        || compiled.compiled_descriptor_hash != hash(COMPILED_DESCRIPTOR_SHA256)
        || compiled.actuator_definitions.len() != HUMANOID_REFERENCE_TRACKER_ACTIONS as usize
    {
        return Err(TrainingEnvironmentError::UnsupportedProfile);
    }
    let value = MotorReferenceTrackingProfileV1 {
        schema_version: MOTOR_REFERENCE_TRACKING_PROFILE_V1_SCHEMA_VERSION,
        profile_id: id(REFERENCE_TRACKING_ENVIRONMENT_PROFILE_ID),
        body_schema_hash: compiled.body_schema_hash,
        compiled_descriptor_hash: compiled.compiled_descriptor_hash,
        corpus_profile_hash: hash(CORPUS_PROFILE_SHA256),
        corpus_manifest_hash: hash(CORPUS_MANIFEST_SHA256),
        profile_document_sha256: hash(REFERENCE_TRACKER_PROFILE_DOCUMENT_SHA256),
        observation_layout_hash: hash(OBSERVATION_LAYOUT_SHA256),
        action_layout_hash: hash(ACTION_LAYOUT_SHA256),
        reset_profile_hash: hash(RESET_PROFILE_SHA256),
        reward_profile_hash: hash(REWARD_PROFILE_SHA256),
        termination_profile_hash: hash(TERMINATION_PROFILE_SHA256),
        rng_derivation_profile_hash: hash(RNG_PROFILE_SHA256),
        eligible_partition_id: id("locomotion"),
        clip_selection_stream_id: id("randomization.reference-clip"),
        phase_selection_stream_id: id("randomization.reference-phase"),
        reset_mode_stream_id: id("randomization.reset-mode"),
        physics_hz: STAGE0_PHYSICS_HZ,
        motor_hz: STAGE0_MOTOR_HZ,
        reference_hz: STAGE0_MOTOR_HZ,
        horizon_offsets_motor_ticks: HUMANOID_REFERENCE_HORIZON_OFFSETS,
        reset_mode_weights_basis_points: [7_000, 2_000, 1_000, 0],
        actor_channel_count: HUMANOID_REFERENCE_TRACKER_CHANNELS,
        critic_channel_count: HUMANOID_REFERENCE_TRACKER_CHANNELS,
        action_channel_count: HUMANOID_REFERENCE_TRACKER_ACTIONS,
        success_reason_id: id("terminal.reference-complete"),
        failure_reason_ids: [
            "terminal.forbidden-locomotion-contact",
            "terminal.hard-impact",
            "terminal.hard-rom",
            "terminal.joint-safety",
            "terminal.non-finite",
            "terminal.reference-tracking-lost",
        ]
        .map(id)
        .into(),
    };
    value.validate()?;
    Ok(value)
}

pub fn biomechanics_reference_environment_manifest_v3(
    compiled: &CompiledBodySchemaV2,
    input_provenance_root: ContentHash,
) -> Result<MotorTrainingEnvironmentManifestV3, TrainingEnvironmentError> {
    let profile = biomechanics_reference_tracking_profile_v1(compiled)?;
    let reward_components = REFERENCE_REWARD_COMPONENT_IDS
        .into_iter()
        .zip(REFERENCE_REWARD_COEFFICIENTS_Q16)
        .map(|(component_id, coefficient_q16)| MotorRewardComponentV1 {
            component_id: id(component_id),
            coefficient_q16,
            minimum_raw: 0,
            maximum_raw: 65_536,
        })
        .collect();
    let base = MotorTrainingEnvironmentManifestV2 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION,
        environment_id: id(REFERENCE_TRACKING_ENVIRONMENT_PROFILE_ID),
        body_schema_hash: compiled.body_schema_hash,
        body_instance_projection_hash: domain_hash(
            "nextengine.body-instance.biomechanics-fixed.v1",
            compiled.body_schema_hash,
        ),
        physics_catalog_hash: compiled.compiled_descriptor_hash,
        observation_layout_hash: profile.observation_layout_hash,
        action_layout_hash: profile.action_layout_hash,
        physics_build_profile_hash: domain_hash(
            "nextengine.physx.build-profile.locked.v1",
            compiled.body_schema_hash,
        ),
        scene_profile_hash: domain_hash(
            "nextengine.physx.scene.deterministic-humanoid.v1",
            compiled.body_schema_hash,
        ),
        bridge_abi_hash: domain_hash("nextengine.physx.bridge-abi.v1", compiled.body_schema_hash),
        quantization_profile_hash: domain_hash(
            "nextengine.physics.quantization.humanoid.v1",
            compiled.body_schema_hash,
        ),
        translator_version_hash: domain_hash(
            "nextengine.isaac.biomechanics-mirror.v1",
            compiled.body_schema_hash,
        ),
        command_schedule_profile_hash: domain_hash(
            "nextengine.motor.command.zero-reference-tracker.v1",
            compiled.body_schema_hash,
        ),
        reward_profile_hash: profile.reward_profile_hash,
        termination_profile_hash: profile.termination_profile_hash,
        rng_derivation_profile_hash: profile.rng_derivation_profile_hash,
        correspondence_profile_hash: domain_hash(
            "nextengine.motor.reference-correspondence.v1",
            compiled.body_schema_hash,
        ),
        physics_hz: STAGE0_PHYSICS_HZ,
        motor_hz: STAGE0_MOTOR_HZ,
        maximum_vector_slots: crate::training::MAX_CPU_VECTOR_SLOTS,
        maximum_episode_steps: 3_600,
        reward_components,
    };
    let value = MotorTrainingEnvironmentManifestV3 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V3_SCHEMA_VERSION,
        base,
        task_input_profile_hash: profile.profile_hash()?,
        input_provenance_root,
    };
    value.validate()?;
    Ok(value)
}

fn id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("reference tracker IDs are compile-time validated")
}

const fn hash(value: [u8; 32]) -> ContentHash {
    content_hash_from_bytes(value)
}

fn domain_hash(domain: &str, body_schema_hash: ContentHash) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(body_schema_hash.as_bytes());
    content_hash_from_bytes(sha256(&preimage))
}

#[cfg(test)]
mod tests {
    use next_contracts::ids::{PersistentId, content_hash_from_bytes};

    use super::*;
    use crate::biomechanics_humanoid_body_schema_v2;

    #[test]
    fn profile_and_v3_manifest_bind_exact_biomechanics_generation() {
        let compiled = CompiledBodySchemaV2::compile(
            &biomechanics_humanoid_body_schema_v2(),
            PersistentId::from_bytes([77; 16]),
        )
        .expect("compile biomechanics profile");
        let profile =
            biomechanics_reference_tracking_profile_v1(&compiled).expect("reference profile");
        assert_eq!(
            profile.profile_document_sha256,
            hash(REFERENCE_TRACKER_PROFILE_DOCUMENT_SHA256)
        );
        assert_eq!(profile.actor_channel_count, 435);
        assert_eq!(profile.action_channel_count, 23);

        let manifest = biomechanics_reference_environment_manifest_v3(
            &compiled,
            content_hash_from_bytes([88; 32]),
        )
        .expect("V3 manifest");
        assert_eq!(manifest.base.environment_id, profile.profile_id);
        assert_eq!(
            manifest.task_input_profile_hash,
            profile.profile_hash().unwrap()
        );
        assert_ne!(manifest.manifest_hash().unwrap(), ContentHash::default());
    }

    #[test]
    fn v3_manifest_rejects_unresolved_provenance() {
        let compiled = CompiledBodySchemaV2::compile(
            &biomechanics_humanoid_body_schema_v2(),
            PersistentId::from_bytes([78; 16]),
        )
        .expect("compile biomechanics profile");
        let error =
            biomechanics_reference_environment_manifest_v3(&compiled, ContentHash::default())
                .expect_err("missing provenance must reject");
        assert_eq!(error.stable_code(), "MOTOR_ENVIRONMENT_MANIFEST_INVALID");
    }
}
