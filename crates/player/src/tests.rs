use next_contracts::ids::{ContentHash, InputSourceId};
use next_contracts::input::{
    ActionAccessibilityV1, ActionBindingTransformV1, ActionBindingV1, ActionConflictPolicyV1,
    ActionDefinitionV1, CORE_CAMERA_ORBIT_ACTION_ID, CORE_GAMEPLAY_CONTEXT_STACK_ID,
    CORE_INTERACT_ACTION_ID, CORE_MOVE_ACTION_ID, INPUT_CONTEXT_SCHEMA_VERSION,
    InputContextCapturePolicyV1, InputContextV1, KEYBOARD_D_CONTROL_PATH_ID,
    KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_E_CONTROL_PATH_ID, KEYBOARD_W_CONTROL_PATH_ID,
    MOUSE_DELTA_CONTROL_PATH_ID, MOUSE_DEVICE_CLASS_ID, PlayerActionValueKindV1,
};

use super::*;

fn id(byte: u8) -> PersistentId {
    PersistentId::from_bytes([byte; 16])
}

fn schema(value: &str) -> SchemaId {
    SchemaId::new(value).expect("test schema ID")
}

fn session() -> PlayerInputSessionV1 {
    PlayerInputSessionV1::new(
        id(1),
        InputSourceId::from_bytes([2; 16]),
        ActionMapManifestV1::core_keyboard_mouse_v1().expect("core action map"),
        InputContextStackV1::gameplay_v1().expect("gameplay context"),
    )
    .expect("valid player input session")
}

fn control_event(
    sequence: u64,
    device_class: &str,
    device_nonce: u8,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
) -> PlatformEventV1 {
    control_event_with_modifiers(
        sequence,
        device_class,
        device_nonce,
        control_path,
        phase,
        value,
        Vec::new(),
    )
}

fn control_event_with_modifiers(
    sequence: u64,
    device_class: &str,
    device_nonce: u8,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
    modifiers: Vec<SchemaId>,
) -> PlatformEventV1 {
    let control = NormalizedControlEventV1::new(
        schema(device_class),
        id(device_nonce),
        schema(control_path),
        phase,
        value,
        modifiers,
        sequence * 10,
        sequence,
    )
    .expect("valid normalized control");
    PlatformEventV1::new(
        id(9),
        schema(if device_class == MOUSE_DEVICE_CLASS_ID {
            "nextengine.platform.source.mouse"
        } else {
            "nextengine.platform.source.keyboard"
        }),
        sequence,
        sequence * 10,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes([8; 32]),
    )
    .expect("valid platform event")
}

fn focus_event(sequence: u64, focused: bool) -> PlatformEventV1 {
    PlatformEventV1::new(
        id(9),
        schema("nextengine.platform.source.window"),
        sequence,
        sequence * 10,
        PlatformEventKindV1::FocusChanged,
        PlatformEventPayloadV1::FocusChanged { focused },
        ContentHash::from_bytes([8; 32]),
    )
    .expect("valid focus event")
}

fn disconnect_event(sequence: u64, device_class: &str, device_nonce: u8) -> PlatformEventV1 {
    PlatformEventV1::new(
        id(9),
        schema("nextengine.platform.source.device"),
        sequence,
        sequence * 10,
        PlatformEventKindV1::DeviceDisconnected,
        PlatformEventPayloadV1::Device {
            device_class: schema(device_class),
            device_instance_nonce: id(device_nonce),
        },
        ContentHash::from_bytes([8; 32]),
    )
    .expect("valid disconnect event")
}

fn resolved(outcome: PlayerInputFrameOutcomeV1) -> ResolvedPlayerInputFrameV1 {
    outcome.resolved.expect("frame has semantic actions")
}

fn assert_event_state_unchanged(actual: &PlayerInputSessionV1, expected: &PlayerInputSessionV1) {
    assert_eq!(actual.source_cursors, expected.source_cursors);
    assert_eq!(
        actual.pending_platform_events,
        expected.pending_platform_events
    );
    assert_eq!(
        actual.last_logical_frame_sequence,
        expected.last_logical_frame_sequence
    );
    assert_eq!(actual.held_controls, expected.held_controls);
    assert_eq!(actual.started_controls, expected.started_controls);
    assert_eq!(actual.pending_deltas, expected.pending_deltas);
    assert_eq!(actual.active_actions, expected.active_actions);
    assert_eq!(actual.cancelled_action_ids, expected.cancelled_action_ids);
    assert_eq!(actual.diagnostics, expected.diagnostics);
}

fn digital_action(
    action_id: SchemaId,
    context_id: SchemaId,
    binding_id: &str,
    control_path: &str,
    modifiers: Vec<SchemaId>,
) -> ActionDefinitionV1 {
    ActionDefinitionV1::new(
        action_id,
        PlayerActionValueKindV1::Digital,
        vec![
            PlayerActionPhaseV1::Started,
            PlayerActionPhaseV1::Performed,
            PlayerActionPhaseV1::Completed,
            PlayerActionPhaseV1::Cancelled,
        ],
        vec![
            ActionBindingV1::new(
                schema(binding_id),
                schema(KEYBOARD_DEVICE_CLASS_ID),
                schema(control_path),
                modifiers,
                ActionBindingTransformV1::Digital {
                    pressed_threshold_q15: 1,
                },
            )
            .expect("digital binding"),
        ],
        vec![context_id],
        ActionConflictPolicyV1::Reject,
        ActionAccessibilityV1 {
            semantic_role_id: schema("nextengine.accessibility-role.digital-test"),
            supports_hold: true,
            supports_toggle: false,
        },
    )
    .expect("digital action")
}

#[test]
fn keyboard_movement_emits_started_performed_and_completed() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("W press accepted");

    let started = resolved(input.close_frame(1).expect("first frame"));
    assert_eq!(started.frame.actions.len(), 1);
    assert_eq!(
        started.frame.actions[0].action_id.as_str(),
        CORE_MOVE_ACTION_ID
    );
    assert_eq!(started.frame.actions[0].phase, PlayerActionPhaseV1::Started);
    assert_eq!(
        started.frame.actions[0].value,
        PlayerActionValueV1::Vector2Q15([0, i16::MAX])
    );
    assert_eq!(started.sample.sampled_wall_time, None);
    assert_eq!(started.sample.source_sequence, 1);

    let performed = resolved(input.close_frame(2).expect("held frame"));
    assert_eq!(
        performed.frame.actions[0].phase,
        PlayerActionPhaseV1::Performed
    );

    input
        .submit_platform_events(&[control_event(
            2,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Completed,
            vec![0],
        )])
        .expect("W release accepted");
    let completed = resolved(input.close_frame(3).expect("release frame"));
    assert_eq!(
        completed.frame.actions[0].phase,
        PlayerActionPhaseV1::Completed
    );
    assert_eq!(
        completed.frame.actions[0].value,
        PlayerActionValueV1::Vector2Q15([0, 0])
    );
    assert!(input.close_frame(4).expect("idle frame").resolved.is_none());
}

#[test]
fn committed_input_recovery_continues_same_source_and_held_action_exactly() {
    let mut uninterrupted = session();
    uninterrupted
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("W press accepted");
    uninterrupted.close_frame(1).expect("started frame");

    let bytes = uninterrupted.recovery_bytes().expect("recovery bytes");
    let mut restored =
        PlayerInputSessionV1::restore_from_recovery_bytes(&bytes).expect("restore input");
    assert_eq!(restored.recovery_bytes().expect("round trip"), bytes);

    let changed = control_event(
        2,
        KEYBOARD_DEVICE_CLASS_ID,
        3,
        KEYBOARD_W_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Changed,
        vec![i16::MAX],
    );
    uninterrupted
        .submit_platform_events(std::slice::from_ref(&changed))
        .expect("continued source accepted");
    restored
        .submit_platform_events(std::slice::from_ref(&changed))
        .expect("continued source accepted after restore");
    assert_eq!(
        uninterrupted.close_frame(2).expect("uninterrupted changed"),
        restored.close_frame(2).expect("restored changed")
    );

    let completed = control_event(
        3,
        KEYBOARD_DEVICE_CLASS_ID,
        3,
        KEYBOARD_W_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
        vec![0],
    );
    uninterrupted
        .submit_platform_events(std::slice::from_ref(&completed))
        .expect("release accepted");
    restored
        .submit_platform_events(std::slice::from_ref(&completed))
        .expect("release accepted after restore");
    assert_eq!(
        uninterrupted.close_frame(3).expect("uninterrupted release"),
        restored.close_frame(3).expect("restored release")
    );
    assert_eq!(
        uninterrupted.recovery_bytes().expect("uninterrupted state"),
        restored.recovery_bytes().expect("restored state")
    );
}

#[test]
fn recovered_held_controls_can_be_cancelled_once_through_the_action_path() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("W press accepted");
    input.close_frame(1).expect("started frame");

    let mut restored = PlayerInputSessionV1::restore_from_recovery_bytes(
        &input.recovery_bytes().expect("recovery bytes"),
    )
    .expect("restore input");
    restored
        .cancel_recovered_controls()
        .expect("schedule recovered cancellation");
    assert!(restored.source_cursors.is_empty());
    let pending_cancellation = restored
        .recovery_bytes()
        .expect("pending cancellation recovery v2");
    let mut restarted = PlayerInputSessionV1::restore_from_recovery_bytes(&pending_cancellation)
        .expect("restore pending cancellation");
    assert_eq!(
        restarted
            .recovery_bytes()
            .expect("pending cancellation round trip"),
        pending_cancellation
    );
    let cancelled = resolved(restarted.close_frame(2).expect("cancellation frame"));
    assert_eq!(cancelled.frame.actions.len(), 1);
    assert_eq!(
        cancelled.frame.actions[0].phase,
        PlayerActionPhaseV1::Cancelled
    );
    assert!(
        restarted
            .close_frame(3)
            .expect("cancellation is one-shot")
            .resolved
            .is_none()
    );
}

#[test]
fn v1_recovery_remains_readable_when_no_pending_cancellation_exists() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("W press accepted");
    input.close_frame(1).expect("started frame");

    let legacy = input
        .recovery_bytes_v1_for_test()
        .expect("compatible v1 recovery");
    let restored =
        PlayerInputSessionV1::restore_from_recovery_bytes(&legacy).expect("read v1 recovery");
    assert!(restored.cancelled_action_ids.is_empty());
    assert_eq!(restored.active_actions, input.active_actions);
    assert_ne!(
        restored.recovery_bytes().expect("v2 upgrade on next write"),
        legacy
    );
}

#[test]
fn input_recovery_rejects_pending_and_malformed_state() {
    let mut pending = session();
    pending
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("pending event");
    assert!(matches!(
        pending.recovery_bytes(),
        Err(PlayerInputError::RecoveryInvalid)
    ));

    let mut bytes = session().recovery_bytes().expect("empty recovery");
    bytes[0] ^= 0xff;
    assert!(matches!(
        PlayerInputSessionV1::restore_from_recovery_bytes(&bytes),
        Err(PlayerInputError::RecoveryInvalid)
    ));
}

#[test]
fn callback_permutation_and_composite_keys_resolve_to_canonical_axial_action() {
    let events = vec![
        control_event(
            2,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_D_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        ),
        control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        ),
    ];
    let mut left = session();
    left.submit_platform_events(&events)
        .expect("ordered events accepted");
    let left = resolved(left.close_frame(1).expect("left close"));

    let mut right = session();
    let mut reversed = events;
    reversed.reverse();
    right
        .submit_platform_events(&reversed)
        .expect("permuted events accepted");
    let right = resolved(right.close_frame(1).expect("right close"));

    assert_eq!(
        left.frame.canonical_bytes().expect("left bytes"),
        right.frame.canonical_bytes().expect("right bytes")
    );
    assert_eq!(
        left.frame.actions[0].value,
        PlayerActionValueV1::Vector2Q15([i16::MAX, 0])
    );
}

#[test]
fn focus_lifecycle_and_control_partitioning_produce_one_closed_batch_result() {
    let control = control_event(
        1,
        KEYBOARD_DEVICE_CLASS_ID,
        3,
        KEYBOARD_W_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
        vec![i16::MAX],
    );
    let focus = focus_event(1, false);

    let mut unified = session();
    unified
        .submit_platform_events(&[focus.clone(), control.clone()])
        .expect("unified callbacks accepted");
    let expected = unified.close_frame(1).expect("unified close");

    for partitions in [
        vec![vec![focus.clone()], vec![control.clone()]],
        vec![vec![control.clone()], vec![focus.clone()]],
        vec![vec![control.clone(), focus.clone()]],
    ] {
        let mut partitioned = session();
        for partition in partitions {
            partitioned
                .submit_platform_events(&partition)
                .expect("partition accepted");
        }
        assert_eq!(
            partitioned.close_frame(1).expect("partitioned close"),
            expected
        );
    }
    assert!(expected.resolved.is_none());
}

#[test]
fn disconnect_and_control_partitioning_discards_the_disconnected_device() {
    let control = control_event(
        1,
        MOUSE_DEVICE_CLASS_ID,
        4,
        MOUSE_DELTA_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Changed,
        vec![100, 50],
    );
    let disconnect = disconnect_event(1, MOUSE_DEVICE_CLASS_ID, 4);

    let mut unified = session();
    unified
        .submit_platform_events(&[control.clone(), disconnect.clone()])
        .expect("unified callbacks accepted");
    let expected = unified.close_frame(1).expect("unified close");

    for partitions in [
        vec![vec![disconnect.clone()], vec![control.clone()]],
        vec![vec![control.clone()], vec![disconnect.clone()]],
        vec![vec![disconnect.clone(), control.clone()]],
    ] {
        let mut partitioned = session();
        for partition in partitions {
            partitioned
                .submit_platform_events(&partition)
                .expect("partition accepted");
        }
        assert_eq!(
            partitioned.close_frame(1).expect("partitioned close"),
            expected
        );
    }
    assert!(expected.resolved.is_none());
    assert_eq!(
        expected.diagnostics,
        vec![PlayerInputDiagnosticCodeV1::InputDeviceLost]
    );
}

#[test]
fn quick_interaction_pulse_is_not_lost_inside_one_frame() {
    let mut input = session();
    input
        .submit_platform_events(&[
            control_event(
                1,
                KEYBOARD_DEVICE_CLASS_ID,
                3,
                KEYBOARD_E_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
            ),
            control_event(
                2,
                KEYBOARD_DEVICE_CLASS_ID,
                3,
                KEYBOARD_E_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Completed,
                vec![0],
            ),
        ])
        .expect("tap accepted");
    let frame = resolved(input.close_frame(1).expect("tap frame"));
    assert_eq!(frame.frame.actions.len(), 1);
    assert_eq!(
        frame.frame.actions[0].action_id.as_str(),
        CORE_INTERACT_ACTION_ID
    );
    assert_eq!(frame.frame.actions[0].phase, PlayerActionPhaseV1::Started);
    assert_eq!(
        frame.frame.actions[0].value,
        PlayerActionValueV1::Digital(true)
    );
    assert!(
        input
            .close_frame(2)
            .expect("post-tap frame")
            .resolved
            .is_none()
    );
}

#[test]
fn mouse_delta_is_presentation_action_and_does_not_remain_held() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            1,
            MOUSE_DEVICE_CLASS_ID,
            4,
            MOUSE_DELTA_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Changed,
            vec![120, -80],
        )])
        .expect("mouse delta accepted");
    let frame = resolved(input.close_frame(1).expect("mouse frame"));
    assert_eq!(
        frame.frame.actions[0].action_id.as_str(),
        CORE_CAMERA_ORBIT_ACTION_ID
    );
    assert_eq!(frame.frame.actions[0].phase, PlayerActionPhaseV1::Performed);
    assert_eq!(
        frame.frame.actions[0].value,
        PlayerActionValueV1::Vector2Q15([120, -80])
    );
    assert!(
        input
            .close_frame(2)
            .expect("delta was consumed")
            .resolved
            .is_none()
    );
}

#[test]
fn focus_loss_cancels_active_action_once() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("W accepted");
    let _ = input.close_frame(1).expect("active frame");

    input
        .submit_platform_events(&[focus_event(1, false)])
        .expect("focus loss accepted");
    let cancelled = resolved(input.close_frame(2).expect("cancel frame"));
    assert_eq!(
        cancelled.frame.actions[0].phase,
        PlayerActionPhaseV1::Cancelled
    );

    input
        .submit_platform_events(&[focus_event(2, false)])
        .expect("repeated focus loss accepted");
    assert!(
        input
            .close_frame(3)
            .expect("second close")
            .resolved
            .is_none()
    );
}

#[test]
fn device_disconnect_cancels_once_and_reports_stable_diagnostic() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("W accepted");
    let _ = input.close_frame(1).expect("active frame");

    input
        .submit_platform_events(&[disconnect_event(1, KEYBOARD_DEVICE_CLASS_ID, 3)])
        .expect("disconnect accepted");
    let outcome = input.close_frame(2).expect("disconnect close");
    assert_eq!(
        outcome.diagnostics,
        vec![PlayerInputDiagnosticCodeV1::InputDeviceLost]
    );
    assert_eq!(outcome.diagnostics[0].stable_code(), "INPUT_DEVICE_LOST");
    assert_eq!(
        outcome.resolved.expect("cancellation frame").frame.actions[0].phase,
        PlayerActionPhaseV1::Cancelled
    );
}

#[test]
fn device_disconnect_discards_unpublished_mouse_delta() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            1,
            MOUSE_DEVICE_CLASS_ID,
            4,
            MOUSE_DELTA_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Changed,
            vec![100, 50],
        )])
        .expect("mouse delta accepted");
    input
        .submit_platform_events(&[disconnect_event(1, MOUSE_DEVICE_CLASS_ID, 4)])
        .expect("mouse disconnect accepted");
    let outcome = input.close_frame(1).expect("disconnect close");
    assert!(outcome.resolved.is_none());
    assert_eq!(
        outcome.diagnostics,
        vec![PlayerInputDiagnosticCodeV1::InputDeviceLost]
    );
}

#[test]
fn queued_context_transition_applies_after_current_frame_boundary() {
    let mut input = session();
    let old_hash = input.context_stack().content_hash;
    let old_revision = input.context_stack().revision;
    let mut entry = input.context_stack().entries[0].clone();
    entry.revision = 2;
    assert_eq!(entry.schema_version, INPUT_CONTEXT_SCHEMA_VERSION);
    let next = InputContextStackV1::new(schema(CORE_GAMEPLAY_CONTEXT_STACK_ID), 2, vec![entry])
        .expect("next stack");
    let next_hash = next.content_hash;
    input
        .queue_context_stack(next)
        .expect("context transition queued");
    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("W accepted");

    let current = input.close_frame(1).expect("current close");
    assert!(current.configuration_changed);
    let current = resolved(current);
    assert_eq!(current.frame.context_stack_hash, old_hash);
    assert_eq!(current.frame.context_stack_revision, old_revision);

    let following = resolved(input.close_frame(2).expect("following close"));
    assert_eq!(following.frame.context_stack_hash, next_hash);
    assert_eq!(following.frame.context_stack_revision, 2);
}

#[test]
fn context_transition_cancels_a_now_blocked_active_action_once() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("W accepted");
    let _ = input.close_frame(1).expect("active frame");

    let gameplay = input.context_stack().entries[0].clone();
    let modal = InputContextV1::new(
        schema("nextengine.input-context.modal"),
        1,
        200,
        InputContextCapturePolicyV1::CaptureAll,
        Vec::new(),
    )
    .expect("modal context");
    let next = InputContextStackV1::new(
        schema(CORE_GAMEPLAY_CONTEXT_STACK_ID),
        2,
        vec![gameplay, modal],
    )
    .expect("modal stack");
    input
        .queue_context_stack(next)
        .expect("modal transition queued");

    let boundary = input.close_frame(2).expect("transition boundary");
    assert!(boundary.configuration_changed);
    let boundary = resolved(boundary);
    assert_eq!(
        boundary
            .frame
            .actions
            .iter()
            .filter(|action| action.phase == PlayerActionPhaseV1::Cancelled)
            .count(),
        1
    );
    assert_eq!(
        boundary
            .frame
            .actions
            .last()
            .expect("terminal action")
            .phase,
        PlayerActionPhaseV1::Cancelled
    );
    assert!(
        input
            .close_frame(3)
            .expect("blocked follow-up")
            .resolved
            .is_none()
    );
}

#[test]
fn source_sequence_rejection_is_atomic() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            2,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("sequence 2 buffered");
    let frame = resolved(input.close_frame(1).expect("sequence 2 accepted"));
    assert_eq!(
        frame.frame.actions[0].value,
        PlayerActionValueV1::Vector2Q15([0, i16::MAX])
    );
    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_D_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("older sequence is structurally valid");
    let before_close = input.clone();
    let error = input
        .close_frame(2)
        .expect_err("older sequence rejected at the close barrier");
    assert_eq!(error.stable_code(), "INPUT_SEQUENCE_NON_MONOTONIC");
    assert_event_state_unchanged(&input, &before_close);
}

#[test]
fn exact_platform_event_duplicates_are_idempotent() {
    let mut input = session();
    let event = control_event(
        1,
        KEYBOARD_DEVICE_CLASS_ID,
        3,
        KEYBOARD_W_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
        vec![i16::MAX],
    );
    input
        .submit_platform_events(&[event.clone(), event.clone()])
        .expect("same-batch duplicate collapses");
    input
        .submit_platform_events(&[event])
        .expect("head retry collapses");

    let frame = resolved(input.close_frame(1).expect("deduplicated frame"));
    assert_eq!(frame.frame.actions.len(), 1);
    assert_eq!(frame.frame.actions[0].phase, PlayerActionPhaseV1::Started);
}

#[test]
fn platform_event_identity_collision_is_atomic() {
    let mut input = session();
    input
        .submit_platform_events(&[
            control_event(
                1,
                KEYBOARD_DEVICE_CLASS_ID,
                3,
                KEYBOARD_W_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
            ),
            control_event(
                1,
                KEYBOARD_DEVICE_CLASS_ID,
                3,
                KEYBOARD_D_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
            ),
        ])
        .expect("colliding callbacks are buffered until close");
    let before_close = input.clone();
    let error = input
        .close_frame(1)
        .expect_err("same source identity with different bytes collides");
    assert_eq!(error.stable_code(), "INPUT_EVENT_IDENTITY_COLLISION");
    assert_event_state_unchanged(&input, &before_close);
}

#[test]
fn source_sequence_gap_is_atomic() {
    let mut input = session();
    input
        .submit_platform_events(&[
            control_event(
                1,
                KEYBOARD_DEVICE_CLASS_ID,
                3,
                KEYBOARD_W_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
            ),
            control_event(
                3,
                KEYBOARD_DEVICE_CLASS_ID,
                3,
                KEYBOARD_D_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
            ),
        ])
        .expect("gapped callbacks are buffered until close");
    let before_close = input.clone();
    let error = input
        .close_frame(1)
        .expect_err("sequence gap rejected at the close barrier");
    assert_eq!(error.stable_code(), "INPUT_SEQUENCE_GAP");
    assert_event_state_unchanged(&input, &before_close);
}

#[test]
fn malformed_known_control_is_rejected_without_partial_state() {
    let mut input = session();
    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![1, 2],
        )])
        .expect("well-formed platform event buffered");
    let before_close = input.clone();
    let error = input
        .close_frame(1)
        .expect_err("wrong keyboard value shape rejected at close");
    assert_eq!(error.stable_code(), "INPUT_EVENT_INVALID");
    assert_event_state_unchanged(&input, &before_close);
}

mod transitions;
