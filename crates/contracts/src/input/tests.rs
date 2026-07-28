use crate::{
    CanonicalDecodeLimits, CommandId, ContentHash, InputSourceId, PersistentId, SchemaId,
    content_hash_from_bytes,
};

use super::*;

fn hash(byte: u8) -> ContentHash {
    content_hash_from_bytes([byte; 32])
}

#[test]
fn profiles_round_trip_and_bind_hashes() {
    let limits = RuntimeAdmissionLimitsV1::default();
    let bytes = limits.canonical_bytes().expect("limits encode");
    assert_eq!(
        RuntimeAdmissionLimitsV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("limits decode"),
        limits
    );

    let tick = TickRateProfileV1::at_30_hz();
    tick.validate().expect("tick profile valid");
    assert_eq!(tick.physics_hz(), 60);

    let ingress = IngressAssignmentProfileV1::core_v1(&limits).expect("profile");
    ingress.validate().expect("ingress profile valid");
    assert_eq!(
        ingress.admission_limits_hash,
        limits.profile_hash().expect("hash")
    );
}

#[test]
fn core_player_action_map_hashes_bind_their_exact_semantic_action_sets() {
    assert_eq!(
        core_player_action_map_v1_hash().to_hex(),
        "78bcf0dea6dfdf4cfae9420f934d20eb4f04f1cc4d280cba852802dc38cd177c"
    );
    assert_ne!(core_player_action_map_v2_hash(), ContentHash::default());
    assert_ne!(
        core_player_action_map_v2_hash(),
        core_player_action_map_v1_hash()
    );
}

#[test]
fn action_frame_round_trips_and_wall_time_is_not_authoritative() {
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: PersistentId::from_bytes([1; 16]),
        logical_frame_sequence: 7,
        action_map_hash: hash(2),
        action_map_revision: 3,
        context_stack_hash: hash(4),
        context_stack_revision: 5,
        actions: vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("action id"),
            phase: PlayerActionPhaseV1::Performed,
            value: PlayerActionValueV1::Vector2Q15([0, 32_767]),
            semantic_occurrence_ordinal: 0,
        }],
    };
    frame.validate().expect("frame valid");
    let bytes = frame.canonical_bytes().expect("frame encode");
    assert_eq!(
        PlayerActionFrameV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("frame decode"),
        frame
    );

    let mut sample = InputSampleV1 {
        schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source"),
        source_id: InputSourceId::from_bytes([8; 16]),
        source_sequence: 7,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("schema"),
        payload_schema_version: 1,
        payload: bytes,
        sampled_wall_time: Some(123),
    };
    let first = sample.canonical_bytes().expect("sample encode");
    sample.sampled_wall_time = Some(-999);
    assert_eq!(sample.canonical_bytes().expect("sample encode"), first);
}

#[test]
fn closed_ingress_batch_rejects_hash_corruption() {
    let admission = RuntimeAdmissionLimitsV1::default();
    let body = ClosedIngressBatchBodyV1 {
        schema_version: 1,
        queue_generation: 0,
        assigned_tick: 0,
        input_samples: Vec::new(),
        completion_signals: Vec::new(),
        input_assignments: Vec::new(),
        completion_assignments: Vec::new(),
        equivalence_receipts: Vec::new(),
    };
    let mut batch = ClosedIngressBatchV1::from_body(body).expect("batch");
    batch.validate(&admission).expect("valid batch");
    batch.batch_hash = hash(9);
    assert_eq!(
        batch.validate(&admission),
        Err(InputContractError::HashMismatch)
    );
}

#[test]
fn closed_ingress_batch_requires_exact_assignment_and_dedup_closure() {
    let admission = RuntimeAdmissionLimitsV1::default();
    let sample = InputSampleV1 {
        schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
        source_id: InputSourceId::from_bytes([8; 16]),
        source_sequence: 4,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("payload schema"),
        payload_schema_version: 1,
        payload: vec![],
        sampled_wall_time: None,
    };
    let mut body = ClosedIngressBatchBodyV1 {
        schema_version: CLOSED_INGRESS_BATCH_SCHEMA_VERSION,
        queue_generation: 3,
        assigned_tick: 7,
        input_samples: vec![sample.clone()],
        completion_signals: vec![],
        input_assignments: vec![],
        completion_assignments: vec![],
        equivalence_receipts: vec![],
    };
    assert_eq!(
        body.validate(&admission),
        Err(InputContractError::InvalidValue)
    );
    body.input_assignments
        .push(IngressAssignmentV1::from_sample(3, 7, &sample).expect("assignment is canonical"));
    body.validate(&admission)
        .expect("exact assignment closure is valid");

    body.input_samples.push(sample);
    assert_eq!(
        body.validate(&admission),
        Err(InputContractError::NonCanonicalOrder)
    );
}

#[test]
fn mapping_receipt_round_trips_with_derived_command_identity() {
    let receipt = InputMappingReceiptV1 {
        assigned_tick: 9,
        source_id: InputSourceId::from_bytes([7; 16]),
        source_sequence: 4,
        payload_hash: hash(6),
        code: InputMappingCodeV1::Accepted,
        derived_command_id: Some(CommandId::from_bytes([5; 16])),
    };
    let bytes = receipt.canonical_bytes().expect("receipt");
    assert_eq!(
        InputMappingReceiptV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("receipt decode"),
        receipt
    );
}

#[test]
fn action_and_payload_limits_accept_boundary_and_reject_overflow() {
    let action = |ordinal| PlayerActionV1 {
        action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("action id"),
        phase: PlayerActionPhaseV1::Performed,
        value: PlayerActionValueV1::Vector2Q15([0, 32_767]),
        semantic_occurrence_ordinal: ordinal,
    };
    for count in [
        MAX_PLAYER_ACTIONS_PER_FRAME - 1,
        MAX_PLAYER_ACTIONS_PER_FRAME,
    ] {
        let frame = PlayerActionFrameV1 {
            schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
            controller_id: PersistentId::from_bytes([1; 16]),
            logical_frame_sequence: 0,
            action_map_hash: hash(2),
            action_map_revision: 0,
            context_stack_hash: hash(3),
            context_stack_revision: 0,
            actions: (0..u32::try_from(count).expect("count fits"))
                .map(action)
                .collect(),
        };
        frame.validate().expect("boundary action count is valid");
    }
    let overflow_frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: PersistentId::from_bytes([1; 16]),
        logical_frame_sequence: 0,
        action_map_hash: hash(2),
        action_map_revision: 0,
        context_stack_hash: hash(3),
        context_stack_revision: 0,
        actions: (0..=u32::try_from(MAX_PLAYER_ACTIONS_PER_FRAME).expect("count fits"))
            .map(action)
            .collect(),
    };
    assert_eq!(
        overflow_frame.validate(),
        Err(InputContractError::ResourceLimit)
    );

    let limits = RuntimeAdmissionLimitsV1::default();
    let sample = |payload_size| InputSampleV1 {
        schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
        source_id: InputSourceId::from_bytes([4; 16]),
        source_sequence: 0,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("payload schema"),
        payload_schema_version: 1,
        payload: vec![0; payload_size],
        sampled_wall_time: None,
    };
    let maximum = usize::try_from(limits.max_input_payload_bytes).expect("limit fits");
    sample(maximum - 1)
        .validate(&limits)
        .expect("N-1 payload is valid");
    sample(maximum)
        .validate(&limits)
        .expect("N payload is valid");
    assert_eq!(
        sample(maximum + 1).validate(&limits),
        Err(InputContractError::ResourceLimit)
    );
}

#[test]
fn occurrence_ordinals_are_global_while_action_storage_is_key_sorted() {
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: PersistentId::from_bytes([1; 16]),
        logical_frame_sequence: 0,
        action_map_hash: hash(2),
        action_map_revision: 0,
        context_stack_hash: hash(3),
        context_stack_revision: 0,
        actions: vec![
            PlayerActionV1 {
                action_id: SchemaId::new("nextengine.action.alpha").expect("action id"),
                phase: PlayerActionPhaseV1::Performed,
                value: PlayerActionValueV1::Digital(true),
                semantic_occurrence_ordinal: 1,
            },
            PlayerActionV1 {
                action_id: SchemaId::new("nextengine.action.beta").expect("action id"),
                phase: PlayerActionPhaseV1::Performed,
                value: PlayerActionValueV1::Digital(true),
                semantic_occurrence_ordinal: 0,
            },
        ],
    };
    frame
        .validate()
        .expect("key order and global continuous ordinals are independent");
}
