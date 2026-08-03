use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{
    CommandId, ContentHash, InputSourceId, PersistentId, SchemaId, content_hash_from_bytes,
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

fn mapping_receipt_v2_fixture() -> InputMappingReceiptV2 {
    InputMappingReceiptV2 {
        schema_version: INPUT_MAPPING_RECEIPT_SCHEMA_VERSION,
        assigned_tick: 9,
        source_id: InputSourceId::from_bytes([7; 16]),
        source_sequence: 4,
        payload_hash: hash(6),
        frame_code: InputMappingCodeV1::Accepted,
        action_results: vec![
            InputActionMappingResultV2 {
                frame_action_ordinal: 0,
                semantic_occurrence_ordinal: 1,
                action_id: SchemaId::new(CORE_CAMERA_ORBIT_ACTION_ID).expect("action id"),
                mapping_code: InputMappingCodeV1::Accepted,
                first_command_ordinal: 0,
                command_count: 0,
            },
            InputActionMappingResultV2 {
                frame_action_ordinal: 1,
                semantic_occurrence_ordinal: 0,
                action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("action id"),
                mapping_code: InputMappingCodeV1::Accepted,
                first_command_ordinal: 0,
                command_count: 2,
            },
        ],
        derived_commands: vec![
            InputDerivedCommandRefV2 {
                command_ordinal: 0,
                source_action_ordinal: 1,
                mapper_command_slot: 0,
                command_id: CommandId::from_bytes([5; 16]),
            },
            InputDerivedCommandRefV2 {
                command_ordinal: 1,
                source_action_ordinal: 1,
                mapper_command_slot: 1,
                command_id: CommandId::from_bytes([6; 16]),
            },
        ],
    }
}

#[test]
fn mapping_receipt_v2_round_trips_without_changing_the_v1_envelope() {
    let receipt = mapping_receipt_v2_fixture();
    receipt.validate().expect("receipt valid");
    let bytes = receipt.canonical_bytes().expect("receipt encode");
    assert_eq!(
        InputMappingReceiptV2::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("receipt decode"),
        receipt
    );
    assert!(matches!(
        InputMappingReceiptV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default()),
        Err(InputContractError::WrongEnvelope)
    ));

    let legacy = InputMappingReceiptV1 {
        assigned_tick: 9,
        source_id: InputSourceId::from_bytes([7; 16]),
        source_sequence: 4,
        payload_hash: hash(6),
        code: InputMappingCodeV1::Accepted,
        derived_command_id: Some(CommandId::from_bytes([5; 16])),
    };
    assert!(matches!(
        InputMappingReceiptV2::from_canonical_bytes(
            &legacy.canonical_bytes().expect("legacy receipt"),
            CanonicalDecodeLimits::default(),
        ),
        Err(InputContractError::WrongEnvelope)
    ));
}

#[test]
fn mapping_receipt_v2_rejects_noncanonical_action_and_command_provenance() {
    let mut receipt = mapping_receipt_v2_fixture();
    receipt.action_results[1].frame_action_ordinal = 2;
    assert_eq!(
        receipt.validate(),
        Err(InputContractError::NonCanonicalOrder)
    );

    let mut receipt = mapping_receipt_v2_fixture();
    receipt.action_results[1].semantic_occurrence_ordinal = 1;
    assert_eq!(
        receipt.validate(),
        Err(InputContractError::NonCanonicalOrder)
    );

    let mut receipt = mapping_receipt_v2_fixture();
    receipt.action_results[1].first_command_ordinal = 1;
    assert_eq!(
        receipt.validate(),
        Err(InputContractError::NonCanonicalOrder)
    );

    let mut receipt = mapping_receipt_v2_fixture();
    receipt.derived_commands[1].command_ordinal = 2;
    assert_eq!(
        receipt.validate(),
        Err(InputContractError::NonCanonicalOrder)
    );

    let mut receipt = mapping_receipt_v2_fixture();
    receipt.derived_commands[1].mapper_command_slot = 2;
    assert_eq!(
        receipt.validate(),
        Err(InputContractError::NonCanonicalOrder)
    );

    let mut receipt = mapping_receipt_v2_fixture();
    receipt.derived_commands[1].source_action_ordinal = 0;
    assert_eq!(
        receipt.validate(),
        Err(InputContractError::NonCanonicalOrder)
    );

    let mut receipt = mapping_receipt_v2_fixture();
    receipt.derived_commands[1].command_id = receipt.derived_commands[0].command_id;
    assert_eq!(receipt.validate(), Err(InputContractError::InvalidValue));
}

#[test]
fn mapping_receipt_v2_is_fail_closed_but_allows_accepted_zero_command_actions() {
    let mut rejected_frame = mapping_receipt_v2_fixture();
    rejected_frame.frame_code = InputMappingCodeV1::FrameInvalid;
    assert_eq!(
        rejected_frame.validate(),
        Err(InputContractError::InvalidValue)
    );

    let mut rejected_action = mapping_receipt_v2_fixture();
    rejected_action.action_results[1].mapping_code = InputMappingCodeV1::ActionUnmapped;
    assert_eq!(
        rejected_action.validate(),
        Err(InputContractError::InvalidValue)
    );

    let mut presentation_only = mapping_receipt_v2_fixture();
    presentation_only.action_results.truncate(1);
    presentation_only.action_results[0].semantic_occurrence_ordinal = 0;
    presentation_only.derived_commands.clear();
    presentation_only
        .validate()
        .expect("an accepted presentation action may derive no command");
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

#[test]
fn core_action_map_and_context_are_hash_bound_canonical_contracts() {
    let action_map = ActionMapManifestV1::core_keyboard_mouse_v1().expect("core action map");
    action_map.validate().expect("action map valid");
    assert_eq!(action_map.revision, 1);
    for action_id in [
        CORE_CAMERA_ORBIT_ACTION_ID,
        CORE_EQUIP_USE_ACTION_ID,
        CORE_INTERACT_ACTION_ID,
        CORE_MELEE_ACTION_ID,
        CORE_MOVE_ACTION_ID,
        CORE_PICKUP_ACTION_ID,
    ] {
        assert!(
            action_map
                .action(&SchemaId::new(action_id).expect("core action"))
                .is_some(),
            "default manifest is missing {action_id}"
        );
    }
    let action_bytes = action_map.canonical_bytes().expect("action map encode");
    assert_eq!(
        ActionMapManifestV1::from_canonical_bytes(&action_bytes, CanonicalDecodeLimits::default())
            .expect("action map decode"),
        action_map
    );

    let context = InputContextStackV1::gameplay_v1().expect("gameplay context");
    context.validate().expect("context valid");
    let context_bytes = context.canonical_bytes().expect("context encode");
    assert_eq!(
        InputContextStackV1::from_canonical_bytes(&context_bytes, CanonicalDecodeLimits::default())
            .expect("context decode"),
        context
    );
}

#[test]
fn action_map_rejects_hash_corruption_and_ambiguous_control_bindings() {
    let mut corrupt = ActionMapManifestV1::core_keyboard_mouse_v1().expect("core action map");
    corrupt.content_hash = hash(99);
    assert_eq!(corrupt.validate(), Err(InputContractError::HashMismatch));

    let mut ambiguous = ActionMapManifestV1::core_keyboard_mouse_v1().expect("core action map");
    let first = ambiguous.actions[0].binding_slots[0].clone();
    ambiguous.actions[1].binding_slots[0].device_class = first.device_class;
    ambiguous.actions[1].binding_slots[0].control_path_id = first.control_path_id;
    assert_eq!(ambiguous.validate(), Err(InputContractError::DuplicateKey));
}

#[test]
fn stateful_action_requires_complete_terminal_phase_policy() {
    let action_map = ActionMapManifestV1::core_keyboard_mouse_v1().expect("core action map");
    let mut movement = action_map
        .action(&SchemaId::new(CORE_MOVE_ACTION_ID).expect("movement action"))
        .expect("movement definition")
        .clone();
    movement
        .allowed_phases
        .retain(|phase| *phase != PlayerActionPhaseV1::Cancelled);
    assert_eq!(movement.validate(), Err(InputContractError::InvalidProfile));

    let mut movement = action_map
        .action(&SchemaId::new(CORE_MOVE_ACTION_ID).expect("movement action"))
        .expect("movement definition")
        .clone();
    movement
        .allowed_phases
        .retain(|phase| *phase != PlayerActionPhaseV1::Completed);
    assert_eq!(movement.validate(), Err(InputContractError::InvalidProfile));
}

#[test]
fn higher_capture_all_context_blocks_lower_gameplay_context() {
    let gameplay = InputContextStackV1::gameplay_v1().expect("gameplay context");
    let overlay = InputContextV1::new(
        SchemaId::new("nextengine.input-context.modal-overlay").expect("overlay ID"),
        1,
        200,
        InputContextCapturePolicyV1::CaptureAll,
        Vec::new(),
    )
    .expect("overlay context");
    let stack = InputContextStackV1::new(
        SchemaId::new("nextengine.input-context-stack.modal").expect("stack ID"),
        2,
        vec![gameplay.entries[0].clone(), overlay],
    )
    .expect("modal stack");
    assert!(!stack.allows_action(&SchemaId::new(CORE_MOVE_ACTION_ID).expect("movement action")));
}

#[test]
fn action_frame_order_is_validated_against_effective_context_priority() {
    let high_context_id =
        SchemaId::new("nextengine.input-context.priority-high").expect("high context");
    let low_context_id =
        SchemaId::new("nextengine.input-context.priority-low").expect("low context");
    let high_action_id = SchemaId::new("nextengine.action.zeta").expect("high action");
    let low_action_id = SchemaId::new("nextengine.action.alpha").expect("low action");
    let action =
        |action_id: SchemaId, context_id: SchemaId, binding_id: &str, control_path_id: &str| {
            ActionDefinitionV1::new(
                action_id,
                PlayerActionValueKindV1::Digital,
                vec![
                    PlayerActionPhaseV1::Started,
                    PlayerActionPhaseV1::Completed,
                    PlayerActionPhaseV1::Cancelled,
                ],
                vec![
                    ActionBindingV1::new(
                        SchemaId::new(binding_id).expect("binding ID"),
                        SchemaId::new(KEYBOARD_DEVICE_CLASS_ID).expect("device class"),
                        SchemaId::new(control_path_id).expect("control path"),
                        Vec::new(),
                        ActionBindingTransformV1::Digital {
                            pressed_threshold_q15: 1,
                        },
                    )
                    .expect("binding"),
                ],
                vec![context_id],
                ActionConflictPolicyV1::Reject,
                ActionAccessibilityV1 {
                    semantic_role_id: SchemaId::new("nextengine.accessibility-role.priority-test")
                        .expect("accessibility role"),
                    supports_hold: true,
                    supports_toggle: false,
                },
            )
            .expect("action")
        };
    let action_map = ActionMapManifestV1::new(
        SchemaId::new("nextengine.action-map.priority-test").expect("map ID"),
        1,
        vec![SchemaId::new(KEYBOARD_DEVICE_CLASS_ID).expect("device class")],
        vec![
            action(
                high_action_id.clone(),
                high_context_id.clone(),
                "nextengine.binding.priority-high",
                KEYBOARD_W_CONTROL_PATH_ID,
            ),
            action(
                low_action_id.clone(),
                low_context_id.clone(),
                "nextengine.binding.priority-low",
                KEYBOARD_D_CONTROL_PATH_ID,
            ),
        ],
    )
    .expect("action map");
    let context_stack = InputContextStackV1::new(
        SchemaId::new("nextengine.input-context-stack.priority-test").expect("stack ID"),
        1,
        vec![
            InputContextV1::new(
                high_context_id,
                1,
                200,
                InputContextCapturePolicyV1::Passthrough,
                vec![high_action_id.clone()],
            )
            .expect("high context"),
            InputContextV1::new(
                low_context_id,
                1,
                100,
                InputContextCapturePolicyV1::CaptureAll,
                vec![low_action_id.clone()],
            )
            .expect("low context"),
        ],
    )
    .expect("context stack");
    let player_action = |action_id, ordinal| PlayerActionV1 {
        action_id,
        phase: PlayerActionPhaseV1::Started,
        value: PlayerActionValueV1::Digital(true),
        semantic_occurrence_ordinal: ordinal,
    };
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: PersistentId::from_bytes([1; 16]),
        logical_frame_sequence: 1,
        action_map_hash: action_map.content_hash,
        action_map_revision: action_map.revision,
        context_stack_hash: context_stack.content_hash,
        context_stack_revision: context_stack.revision,
        actions: vec![
            player_action(high_action_id.clone(), 1),
            player_action(low_action_id.clone(), 0),
        ],
    };
    frame
        .validate_against(&action_map, &context_stack)
        .expect("higher priority action precedes lexical action order");

    let mut wrong_order = frame;
    wrong_order.actions.reverse();
    assert_eq!(
        wrong_order.validate_against(&action_map, &context_stack),
        Err(InputContractError::NonCanonicalOrder)
    );
}

#[test]
fn core_action_map_declares_universal_ui_actions_for_modal_contexts() {
    let action_map = ActionMapManifestV1::core_keyboard_mouse_v1().expect("core action map");
    assert_eq!(action_map.actions.len(), 11);

    let ui_menu = SchemaId::new(CORE_UI_MENU_CONTEXT_ID).expect("ui menu context id");
    let ui_dialogue = SchemaId::new(CORE_UI_DIALOGUE_CONTEXT_ID).expect("ui dialogue context id");
    let gameplay = SchemaId::new(CORE_GAMEPLAY_CONTEXT_ID).expect("gameplay context id");

    let navigate = action_map
        .action(&SchemaId::new(CORE_UI_NAVIGATE_ACTION_ID).expect("ui nav id"))
        .expect("ui navigate action declared");
    assert_eq!(navigate.value_kind, PlayerActionValueKindV1::Vector2Q15);
    assert_eq!(navigate.binding_slots.len(), 4);
    assert_eq!(
        navigate.allowed_context_ids,
        vec![ui_dialogue.clone(), ui_menu.clone()]
    );

    for (action_id, control_path) in [
        (CORE_UI_CONFIRM_ACTION_ID, KEYBOARD_RETURN_CONTROL_PATH_ID),
        (CORE_UI_BACK_ACTION_ID, KEYBOARD_ESCAPE_CONTROL_PATH_ID),
        (CORE_UI_INVENTORY_ACTION_ID, KEYBOARD_I_CONTROL_PATH_ID),
        (CORE_UI_JOURNAL_ACTION_ID, KEYBOARD_J_CONTROL_PATH_ID),
    ] {
        let action = action_map
            .action(&SchemaId::new(action_id).expect("ui action id"))
            .expect("ui action declared");
        assert_eq!(action.value_kind, PlayerActionValueKindV1::Digital);
        assert_eq!(action.binding_slots.len(), 1);
        assert_eq!(
            action.binding_slots[0].control_path_id,
            SchemaId::new(control_path).expect("control path id")
        );
    }
    let back = action_map
        .action(&SchemaId::new(CORE_UI_BACK_ACTION_ID).expect("ui back id"))
        .expect("ui back action declared");
    assert_eq!(
        back.allowed_context_ids,
        vec![gameplay, ui_dialogue, ui_menu]
    );
}

#[test]
fn ui_modal_context_stacks_validate_against_core_action_map() {
    let action_map = ActionMapManifestV1::core_keyboard_mouse_v1().expect("core action map");
    let gameplay = InputContextStackV1::gameplay_v1().expect("gameplay stack");
    gameplay
        .validate_against_action_map(&action_map)
        .expect("gameplay stack validates");
    assert!(gameplay.allows_action(&SchemaId::new(CORE_UI_BACK_ACTION_ID).expect("ui back id")));
    for action_id in [CORE_UI_INVENTORY_ACTION_ID, CORE_UI_JOURNAL_ACTION_ID] {
        assert!(
            gameplay.allows_action(&SchemaId::new(action_id).expect("ui screen action id")),
            "gameplay context keeps the read-only screen toggles live"
        );
    }
    assert!(
        !gameplay.allows_action(&SchemaId::new(CORE_UI_CONFIRM_ACTION_ID).expect("ui confirm id"))
    );

    for stack in [
        InputContextStackV1::ui_menu_v1().expect("ui menu stack"),
        InputContextStackV1::ui_dialogue_v1().expect("ui dialogue stack"),
    ] {
        stack
            .validate_against_action_map(&action_map)
            .expect("ui modal stack validates against core map");
        for action_id in [
            CORE_UI_BACK_ACTION_ID,
            CORE_UI_CONFIRM_ACTION_ID,
            CORE_UI_NAVIGATE_ACTION_ID,
        ] {
            assert!(stack.allows_action(&SchemaId::new(action_id).expect("ui action id")));
        }
        assert!(
            !stack.allows_action(&SchemaId::new(CORE_MOVE_ACTION_ID).expect("move id")),
            "modal capture-all layer blocks gameplay actions"
        );
    }
}
