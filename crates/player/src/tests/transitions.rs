use super::*;

#[test]
fn terminal_control_without_required_modifier_releases_held_action() {
    let context_id = schema("nextengine.input-context.modified");
    let action_id = schema("nextengine.action.modified");
    let modifier = schema("nextengine.input.modifier.shift");
    let action_map = ActionMapManifestV1::new(
        schema("nextengine.action-map.modified"),
        1,
        vec![schema(KEYBOARD_DEVICE_CLASS_ID)],
        vec![digital_action(
            action_id.clone(),
            context_id.clone(),
            "nextengine.binding.modified",
            KEYBOARD_W_CONTROL_PATH_ID,
            vec![modifier.clone()],
        )],
    )
    .expect("modified action map");
    let context_stack = InputContextStackV1::new(
        schema("nextengine.input-context-stack.modified"),
        1,
        vec![
            InputContextV1::new(
                context_id,
                1,
                100,
                InputContextCapturePolicyV1::CaptureAll,
                vec![action_id],
            )
            .expect("modified context"),
        ],
    )
    .expect("modified context stack");
    let mut input = PlayerInputSessionV1::new(
        id(1),
        InputSourceId::from_bytes([2; 16]),
        action_map,
        context_stack,
    )
    .expect("modified session");

    input
        .submit_platform_events(&[control_event_with_modifiers(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
            vec![modifier],
        )])
        .expect("modified press accepted");
    let _ = input.close_frame(1).expect("active modified frame");
    input
        .submit_platform_events(&[control_event(
            2,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Completed,
            vec![0],
        )])
        .expect("release without modifier accepted");
    let completed = resolved(input.close_frame(2).expect("release frame"));
    assert_eq!(
        completed.frame.actions[0].phase,
        PlayerActionPhaseV1::Completed
    );
    assert!(
        input
            .close_frame(3)
            .expect("released follow-up")
            .resolved
            .is_none()
    );
}

#[test]
fn release_after_rebind_does_not_reactivate_when_old_binding_returns() {
    let mut input = session();
    let original = input.action_map().clone();
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

    let mut rebound_actions = original.actions.clone();
    let movement = rebound_actions
        .iter_mut()
        .find(|action| action.action_id.as_str() == CORE_MOVE_ACTION_ID)
        .expect("movement action");
    movement
        .binding_slots
        .iter_mut()
        .find(|binding| binding.control_path_id.as_str() == KEYBOARD_W_CONTROL_PATH_ID)
        .expect("W binding")
        .control_path_id = schema("nextengine.input.keyboard.rebound-up");
    let rebound = ActionMapManifestV1::new(
        original.action_map_id.clone(),
        2,
        original.supported_device_classes.clone(),
        rebound_actions,
    )
    .expect("rebound map");
    input.queue_action_map(rebound).expect("rebind queued");
    let boundary = resolved(input.close_frame(2).expect("rebind boundary"));
    assert!(
        boundary
            .frame
            .actions
            .iter()
            .any(|action| action.action_id.as_str() == CORE_MOVE_ACTION_ID
                && action.phase == PlayerActionPhaseV1::Cancelled),
        "active action must close under the old map revision"
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
        .expect("old binding release accepted");
    assert!(
        input
            .close_frame(3)
            .expect("release under rebound map")
            .resolved
            .is_none(),
        "the old binding is no longer active after revision cancellation"
    );

    let restored = ActionMapManifestV1::new(
        original.action_map_id,
        3,
        original.supported_device_classes,
        original.actions,
    )
    .expect("restored map");
    input
        .queue_action_map(restored)
        .expect("restore binding queued");
    assert!(
        input
            .close_frame(4)
            .expect("restore boundary")
            .resolved
            .is_none()
    );
    assert!(
        input
            .close_frame(5)
            .expect("no ghost activation")
            .resolved
            .is_none()
    );
}

#[test]
fn incompatible_context_and_non_monotonic_frame_have_stable_errors() {
    let mut input = session();
    let stale = InputContextStackV1::new(
        schema("nextengine.input-context-stack.stale"),
        2,
        vec![
            InputContextV1::new(
                schema("nextengine.input-context.stale"),
                1,
                100,
                InputContextCapturePolicyV1::CaptureAll,
                vec![schema(CORE_MOVE_ACTION_ID)],
            )
            .expect("well-formed but incompatible context"),
        ],
    )
    .expect("well-formed stack");
    let error = input
        .queue_context_stack(stale)
        .expect_err("incompatible context rejected");
    assert_eq!(error.stable_code(), "INPUT_CONTEXT_STALE");

    assert!(
        input
            .close_frame(5)
            .expect("first close")
            .resolved
            .is_none()
    );
    let error = input
        .close_frame(5)
        .expect_err("duplicate logical frame rejected");
    assert_eq!(error.stable_code(), "INPUT_SEQUENCE_NON_MONOTONIC");
}

#[test]
fn platform_event_batch_bound_is_checked_before_state_mutation() {
    let mut input = session();
    let events = vec![focus_event(1, true); MAX_PLATFORM_EVENTS_PER_INPUT_FRAME + 1];
    let error = input
        .submit_platform_events(&events)
        .expect_err("oversized event batch rejected");
    assert_eq!(error.stable_code(), "INPUT_RESOURCE_LIMIT");
    assert!(
        input
            .close_frame(1)
            .expect("oversized batch changed no state")
            .resolved
            .is_none()
    );
}

#[test]
fn platform_event_bound_is_aggregate_across_submissions_until_frame_close() {
    let mut input = session();
    let admitted = vec![focus_event(1, true); MAX_PLATFORM_EVENTS_PER_INPUT_FRAME - 1];
    input
        .submit_platform_events(&admitted)
        .expect("N - 1 exact retry occurrences fit");

    let error = input
        .submit_platform_events(&[focus_event(2, true), focus_event(2, true)])
        .expect_err("aggregate N + 1 must reject before processing");
    assert_eq!(error.stable_code(), "INPUT_RESOURCE_LIMIT");

    input.close_frame(1).expect("frame close resets aggregate");
    input
        .submit_platform_events(&[focus_event(2, true)])
        .expect("rejected aggregate did not advance the source cursor");
}

#[test]
fn action_map_and_context_transitions_require_same_identity_and_newer_revision() {
    let mut input = session();
    let same_map = input.action_map().clone();
    assert_eq!(
        input
            .queue_action_map(same_map)
            .expect_err("same action-map revision is stale")
            .stable_code(),
        "INPUT_CONTEXT_STALE"
    );
    let current_map = input.action_map().clone();
    let foreign_map = ActionMapManifestV1::new(
        schema("nextengine.action-map.foreign"),
        current_map.revision + 1,
        current_map.supported_device_classes,
        current_map.actions,
    )
    .expect("well-formed foreign map");
    assert_eq!(
        input
            .queue_action_map(foreign_map)
            .expect_err("action-map identity cannot be replaced")
            .stable_code(),
        "INPUT_CONTEXT_STALE"
    );

    let same_context = input.context_stack().clone();
    assert_eq!(
        input
            .queue_context_stack(same_context)
            .expect_err("same context revision is stale")
            .stable_code(),
        "INPUT_CONTEXT_STALE"
    );
}

#[test]
fn higher_context_priority_precedes_lexically_lower_action_id() {
    let high_context_id = schema("nextengine.input-context.priority-high");
    let low_context_id = schema("nextengine.input-context.priority-low");
    let high_action_id = schema("nextengine.action.zeta");
    let low_action_id = schema("nextengine.action.alpha");
    let action_map = ActionMapManifestV1::new(
        schema("nextengine.action-map.priority-test"),
        1,
        vec![schema(KEYBOARD_DEVICE_CLASS_ID)],
        vec![
            digital_action(
                high_action_id.clone(),
                high_context_id.clone(),
                "nextengine.binding.priority-high",
                KEYBOARD_W_CONTROL_PATH_ID,
                Vec::new(),
            ),
            digital_action(
                low_action_id.clone(),
                low_context_id.clone(),
                "nextengine.binding.priority-low",
                KEYBOARD_D_CONTROL_PATH_ID,
                Vec::new(),
            ),
        ],
    )
    .expect("priority action map");
    let context_stack = InputContextStackV1::new(
        schema("nextengine.input-context-stack.priority-test"),
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
    .expect("priority context stack");
    let mut input = PlayerInputSessionV1::new(
        id(1),
        InputSourceId::from_bytes([2; 16]),
        action_map,
        context_stack,
    )
    .expect("priority session");
    input
        .submit_platform_events(&[
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
        ])
        .expect("priority controls accepted");
    let frame = resolved(input.close_frame(1).expect("priority frame"));
    assert_eq!(
        frame
            .frame
            .actions
            .iter()
            .map(|action| action.action_id.clone())
            .collect::<Vec<_>>(),
        vec![high_action_id, low_action_id]
    );
}

#[test]
fn scalar_q15_binding_uses_the_same_canonical_frame_path() {
    let context_id = schema("nextengine.input-context.analog-test");
    let action_id = schema("nextengine.action.analog-test");
    let action_map = ActionMapManifestV1::new(
        schema("nextengine.action-map.analog-test"),
        1,
        vec![schema(KEYBOARD_DEVICE_CLASS_ID)],
        vec![
            ActionDefinitionV1::new(
                action_id.clone(),
                PlayerActionValueKindV1::ScalarQ15,
                vec![
                    PlayerActionPhaseV1::Started,
                    PlayerActionPhaseV1::Performed,
                    PlayerActionPhaseV1::Completed,
                    PlayerActionPhaseV1::Cancelled,
                ],
                vec![
                    ActionBindingV1::new(
                        schema("nextengine.binding.analog-test"),
                        schema(KEYBOARD_DEVICE_CLASS_ID),
                        schema("nextengine.input.keyboard.analog-test"),
                        Vec::new(),
                        ActionBindingTransformV1::ScalarPassthroughQ15 {
                            scale_q15: i16::MAX,
                        },
                    )
                    .expect("scalar binding"),
                ],
                vec![context_id.clone()],
                ActionConflictPolicyV1::PreferCanonicalBinding,
                ActionAccessibilityV1 {
                    semantic_role_id: schema("nextengine.accessibility-role.analog-test"),
                    supports_hold: true,
                    supports_toggle: false,
                },
            )
            .expect("scalar action"),
        ],
    )
    .expect("scalar action map");
    let context_stack = InputContextStackV1::new(
        schema("nextengine.input-context-stack.analog-test"),
        1,
        vec![
            InputContextV1::new(
                context_id,
                1,
                100,
                InputContextCapturePolicyV1::CaptureAll,
                vec![action_id.clone()],
            )
            .expect("scalar context"),
        ],
    )
    .expect("scalar context stack");
    let mut input = PlayerInputSessionV1::new(
        id(1),
        InputSourceId::from_bytes([2; 16]),
        action_map,
        context_stack,
    )
    .expect("scalar session");

    input
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            "nextengine.input.keyboard.analog-test",
            NormalizedControlPhaseV1::Changed,
            vec![-1_234],
        )])
        .expect("scalar input accepted");
    let frame = resolved(input.close_frame(1).expect("scalar close"));
    assert_eq!(frame.frame.actions[0].action_id, action_id);
    assert_eq!(
        frame.frame.actions[0].value,
        PlayerActionValueV1::ScalarQ15(-1_234)
    );
}
