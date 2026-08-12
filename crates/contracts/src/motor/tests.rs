use super::*;

fn id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("test identifier")
}

fn reference_profile() -> MotorReferenceTrackingProfileV1 {
    MotorReferenceTrackingProfileV1 {
        schema_version: MOTOR_REFERENCE_TRACKING_PROFILE_V1_SCHEMA_VERSION,
        profile_id: id("nextengine.motor.env.humanoid-reference-tracker.v1"),
        body_schema_hash: content_hash_from_bytes([1; 32]),
        compiled_descriptor_hash: content_hash_from_bytes([2; 32]),
        corpus_profile_hash: content_hash_from_bytes([3; 32]),
        corpus_manifest_hash: content_hash_from_bytes([4; 32]),
        profile_document_sha256: content_hash_from_bytes([5; 32]),
        observation_layout_hash: content_hash_from_bytes([6; 32]),
        action_layout_hash: content_hash_from_bytes([7; 32]),
        reset_profile_hash: content_hash_from_bytes([8; 32]),
        reward_profile_hash: content_hash_from_bytes([9; 32]),
        termination_profile_hash: content_hash_from_bytes([10; 32]),
        rng_derivation_profile_hash: content_hash_from_bytes([11; 32]),
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
    }
}

fn v2_manifest() -> MotorTrainingEnvironmentManifestV2 {
    MotorTrainingEnvironmentManifestV2 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION,
        environment_id: id("nextengine.motor.env.humanoid-reference-tracker.v1"),
        body_schema_hash: content_hash_from_bytes([1; 32]),
        body_instance_projection_hash: content_hash_from_bytes([2; 32]),
        physics_catalog_hash: content_hash_from_bytes([3; 32]),
        observation_layout_hash: content_hash_from_bytes([4; 32]),
        action_layout_hash: content_hash_from_bytes([5; 32]),
        physics_build_profile_hash: content_hash_from_bytes([6; 32]),
        scene_profile_hash: content_hash_from_bytes([7; 32]),
        bridge_abi_hash: content_hash_from_bytes([8; 32]),
        quantization_profile_hash: content_hash_from_bytes([9; 32]),
        translator_version_hash: content_hash_from_bytes([10; 32]),
        command_schedule_profile_hash: content_hash_from_bytes([11; 32]),
        reward_profile_hash: content_hash_from_bytes([12; 32]),
        termination_profile_hash: content_hash_from_bytes([13; 32]),
        rng_derivation_profile_hash: content_hash_from_bytes([14; 32]),
        correspondence_profile_hash: content_hash_from_bytes([15; 32]),
        physics_hz: STAGE0_PHYSICS_HZ,
        motor_hz: STAGE0_MOTOR_HZ,
        maximum_vector_slots: 64,
        maximum_episode_steps: 3_600,
        reward_components: vec![MotorRewardComponentV1 {
            component_id: id("reward.reference-joint-pose"),
            coefficient_q16: 131_072,
            minimum_raw: 0,
            maximum_raw: 65_536,
        }],
    }
}

#[test]
fn action_layout_rejects_implicit_or_reordered_actuators() {
    let mut layout = MotorActionLayoutV1 {
        schema_version: MOTOR_ACTION_LAYOUT_V1_SCHEMA_VERSION,
        layout_id: id("nextengine.motor.action.test"),
        body_schema_hash: content_hash_from_bytes([1; 32]),
        actuator_profile_hash: content_hash_from_bytes([2; 32]),
        channels: vec![
            MotorActionChannelV1 {
                channel_id: id("nextengine.channel.a"),
                semantic: MotorActionSemanticV1::ResidualJointPosition,
                actuator_id: id("nextengine.actuator.a"),
                minimum_raw: -1,
                maximum_raw: 1,
                scale_numerator: 1,
                scale_denominator: 1,
            },
            MotorActionChannelV1 {
                channel_id: id("nextengine.channel.b"),
                semantic: MotorActionSemanticV1::ResidualJointPosition,
                actuator_id: id("nextengine.actuator.b"),
                minimum_raw: -1,
                maximum_raw: 1,
                scale_numerator: 1,
                scale_denominator: 1,
            },
        ],
    };
    assert!(layout.validate().is_ok());
    layout.channels.swap(0, 1);
    assert_eq!(
        layout.validate(),
        Err(MotorContractError::NonCanonicalOrder)
    );
}

#[test]
fn stage0_manifest_requires_240_over_60_profile() {
    let mut manifest = MotorTrainingEnvironmentManifestV1 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V1_SCHEMA_VERSION,
        environment_id: id("nextengine.motor.env.test"),
        body_schema_hash: content_hash_from_bytes([1; 32]),
        body_instance_projection_hash: content_hash_from_bytes([2; 32]),
        physics_catalog_hash: content_hash_from_bytes([3; 32]),
        observation_layout_hash: content_hash_from_bytes([4; 32]),
        action_layout_hash: content_hash_from_bytes([5; 32]),
        physics_build_profile_hash: content_hash_from_bytes([6; 32]),
        scene_profile_hash: content_hash_from_bytes([7; 32]),
        bridge_abi_hash: content_hash_from_bytes([8; 32]),
        quantization_profile_hash: content_hash_from_bytes([9; 32]),
        translator_version_hash: content_hash_from_bytes([10; 32]),
        physics_hz: STAGE0_PHYSICS_HZ,
        motor_hz: STAGE0_MOTOR_HZ,
        maximum_vector_slots: 128,
        maximum_episode_steps: 14_400,
        reward_components: vec![MotorRewardComponentV1 {
            component_id: id("nextengine.reward.upright"),
            coefficient_q16: 65_536,
            minimum_raw: 0,
            maximum_raw: 65_536,
        }],
    };
    assert!(manifest.validate().is_ok());
    manifest.physics_hz = 120;
    assert_eq!(
        manifest.validate(),
        Err(MotorContractError::InvalidManifest)
    );
}

#[test]
fn reward_component_order_is_semantic_but_duplicate_ids_are_rejected() {
    let mut manifest = MotorTrainingEnvironmentManifestV1 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V1_SCHEMA_VERSION,
        environment_id: id("nextengine.motor.env.reward-order"),
        body_schema_hash: content_hash_from_bytes([1; 32]),
        body_instance_projection_hash: content_hash_from_bytes([2; 32]),
        physics_catalog_hash: content_hash_from_bytes([3; 32]),
        observation_layout_hash: content_hash_from_bytes([4; 32]),
        action_layout_hash: content_hash_from_bytes([5; 32]),
        physics_build_profile_hash: content_hash_from_bytes([6; 32]),
        scene_profile_hash: content_hash_from_bytes([7; 32]),
        bridge_abi_hash: content_hash_from_bytes([8; 32]),
        quantization_profile_hash: content_hash_from_bytes([9; 32]),
        translator_version_hash: content_hash_from_bytes([10; 32]),
        physics_hz: STAGE0_PHYSICS_HZ,
        motor_hz: STAGE0_MOTOR_HZ,
        maximum_vector_slots: 4_096,
        maximum_episode_steps: 3_600,
        reward_components: ["reward.upright", "reward.action-rate-penalty"]
            .map(|component_id| MotorRewardComponentV1 {
                component_id: id(component_id),
                coefficient_q16: 65_536,
                minimum_raw: i64::MIN,
                maximum_raw: i64::MAX,
            })
            .into(),
    };
    assert!(manifest.validate().is_ok());
    manifest.reward_components[1].component_id = id("reward.upright");
    assert_eq!(
        manifest.validate(),
        Err(MotorContractError::InvalidManifest)
    );
}

#[test]
fn protocol_v2_rejects_v1_manifest_with_typed_version_error() {
    let manifest = MotorTrainingEnvironmentManifestV1 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V1_SCHEMA_VERSION,
        environment_id: id("nextengine.motor.env.legacy"),
        body_schema_hash: content_hash_from_bytes([1; 32]),
        body_instance_projection_hash: content_hash_from_bytes([2; 32]),
        physics_catalog_hash: content_hash_from_bytes([3; 32]),
        observation_layout_hash: content_hash_from_bytes([4; 32]),
        action_layout_hash: content_hash_from_bytes([5; 32]),
        physics_build_profile_hash: content_hash_from_bytes([6; 32]),
        scene_profile_hash: content_hash_from_bytes([7; 32]),
        bridge_abi_hash: content_hash_from_bytes([8; 32]),
        quantization_profile_hash: content_hash_from_bytes([9; 32]),
        translator_version_hash: content_hash_from_bytes([10; 32]),
        physics_hz: STAGE0_PHYSICS_HZ,
        motor_hz: STAGE0_MOTOR_HZ,
        maximum_vector_slots: 1,
        maximum_episode_steps: 1,
        reward_components: vec![MotorRewardComponentV1 {
            component_id: id("reward.test"),
            coefficient_q16: 65_536,
            minimum_raw: 0,
            maximum_raw: 65_536,
        }],
    };
    let error = manifest
        .validate_for_protocol_v2()
        .expect_err("v1 is not a protocol-v2 manifest");
    assert_eq!(
        error.stable_code(),
        "UNSUPPORTED_MOTOR_ENVIRONMENT_MANIFEST_VERSION"
    );
}

#[test]
fn reference_profile_closes_corpus_layout_and_reset_semantics() {
    let profile = reference_profile();
    assert!(profile.validate().is_ok());
    let profile_hash = profile.profile_hash().expect("profile hash");

    let mut changed_corpus = profile.clone();
    changed_corpus.corpus_manifest_hash = content_hash_from_bytes([99; 32]);
    assert_ne!(
        changed_corpus.profile_hash().expect("changed profile hash"),
        profile_hash
    );

    let mut invalid_horizon = profile.clone();
    invalid_horizon.horizon_offsets_motor_ticks[3] = 15;
    assert_eq!(
        invalid_horizon.validate(),
        Err(MotorContractError::InvalidManifest)
    );

    let mut recovery_enabled = profile;
    recovery_enabled.reset_mode_weights_basis_points = [6_999, 2_000, 1_000, 1];
    assert_eq!(
        recovery_enabled.validate(),
        Err(MotorContractError::InvalidManifest)
    );
}

#[test]
fn environment_manifest_v3_binds_task_profile_and_provenance() {
    let profile_hash = reference_profile().profile_hash().expect("profile hash");
    let manifest = MotorTrainingEnvironmentManifestV3 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V3_SCHEMA_VERSION,
        base: v2_manifest(),
        task_input_profile_hash: profile_hash,
        input_provenance_root: content_hash_from_bytes([16; 32]),
    };
    assert!(manifest.validate_for_protocol_v2().is_ok());
    let manifest_hash = manifest.manifest_hash().expect("manifest hash");

    let mut different_provenance = manifest.clone();
    different_provenance.input_provenance_root = content_hash_from_bytes([17; 32]);
    assert_ne!(
        different_provenance.manifest_hash().expect("changed hash"),
        manifest_hash
    );

    let mut unclosed = manifest;
    unclosed.task_input_profile_hash = ContentHash::default();
    assert_eq!(
        unclosed.validate(),
        Err(MotorContractError::InvalidManifest)
    );
}

#[test]
fn checkpoint_envelope_round_trips_and_rejects_payload_tamper() {
    let payload = vec![1, 2, 3, 4];
    let envelope = MotorEnvironmentCheckpointEnvelopeV1 {
        schema_version: MOTOR_ENVIRONMENT_CHECKPOINT_ENVELOPE_V1_SCHEMA_VERSION,
        environment_profile_id: id("nextengine.motor.env.test"),
        environment_manifest_hash: content_hash_from_bytes([1; 32]),
        run_root: content_hash_from_bytes([2; 32]),
        episode_ordinal: 7,
        vector_slot: 3,
        motor_tick: 11,
        terminal_disposition: MotorTerminalDispositionV1::Running,
        terminal_reason_id: None,
        current_observation_root: content_hash_from_bytes([3; 32]),
        last_step_root: StateRoot::from_bytes([4; 32]),
        motor_runtime_checkpoint_hash: content_hash_from_bytes(sha256(&payload)),
        motor_runtime_checkpoint_bytes: payload,
    };
    let bytes = envelope.canonical_bytes().expect("encode");
    assert_eq!(
        MotorEnvironmentCheckpointEnvelopeV1::from_canonical_bytes(&bytes).expect("decode"),
        envelope
    );

    let mut tampered = envelope;
    tampered.motor_runtime_checkpoint_bytes[0] ^= 1;
    assert_eq!(tampered.validate(), Err(MotorContractError::InvalidBounds));
}
