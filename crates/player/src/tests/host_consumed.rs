use super::*;

#[test]
fn host_consumed_events_admit_cursors_without_actions_or_controls() {
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
        .expect("real press buffered");
    let outcome = input.close_frame(1).expect("press frame closes");
    resolved(outcome);

    // The host consumed sequences 2 and 3 outside the game input stream
    // (interactive pause-menu keys while suspended).
    input
        .queue_host_consumed_platform_events(&[
            control_event(
                2,
                KEYBOARD_DEVICE_CLASS_ID,
                3,
                KEYBOARD_RETURN_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
            ),
            control_event(
                3,
                KEYBOARD_DEVICE_CLASS_ID,
                3,
                KEYBOARD_RETURN_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Completed,
                vec![0],
            ),
        ])
        .expect("host-consumed menu keys queued");
    // A queued host-consumed queue is transient and never recoverable.
    assert!(input.recovery_bytes().is_err());

    // Sequence 4 continues gaplessly across the consumed range; consumed
    // keys produced no held controls and no actions.
    input
        .submit_platform_events(&[control_event(
            4,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Completed,
            vec![0],
        )])
        .expect("real release buffered");
    let outcome = input.close_frame(2).expect("merged frame closes");
    let frame = resolved(outcome);
    assert_eq!(frame.frame.actions.len(), 1);
    assert!(frame.frame.actions[0].action_id.as_str().contains("move"));
    assert!(input.held_controls.is_empty());
}

#[test]
fn host_consumed_continuity_violations_are_atomic() {
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
        .expect("real press buffered");
    input.close_frame(1).expect("press frame closes");

    // A host-consumed event that skips a sequence is still a gap.
    input
        .queue_host_consumed_platform_events(&[control_event(
            3,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_RETURN_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("gapped host-consumed key queued");
    let before_close = input.clone();
    let error = input
        .close_frame(2)
        .expect_err("host-consumed gap rejected at the close barrier");
    assert_eq!(error.stable_code(), "INPUT_SEQUENCE_GAP");
    assert_event_state_unchanged(&input, &before_close);

    // Without the queued range a later real event gaps the same way.
    let mut unqueued = session();
    unqueued
        .submit_platform_events(&[control_event(
            1,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("real press buffered");
    unqueued.close_frame(1).expect("press frame closes");
    unqueued
        .submit_platform_events(&[control_event(
            4,
            KEYBOARD_DEVICE_CLASS_ID,
            3,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Completed,
            vec![0],
        )])
        .expect("real release buffered");
    let error = unqueued
        .close_frame(2)
        .expect_err("missing host-consumed range still gaps");
    assert_eq!(error.stable_code(), "INPUT_SEQUENCE_GAP");
}
