use super::*;

#[test]
fn adapter_finalizer_runs_once_before_owned_adapter_resources_drop() {
    struct DropMarker<'a>(&'a RefCell<Vec<&'static str>>);

    impl Drop for DropMarker<'_> {
        fn drop(&mut self) {
            self.0.borrow_mut().push("adapter-drop");
        }
    }

    fn fail_after_adapter_setup(order: &RefCell<Vec<&'static str>>) -> Result<(), ()> {
        let adapter;
        let _finalizer = AdapterFinalizer::new(|| {
            order.borrow_mut().push("finalize");
            DesktopApplicationFinalization::Complete
        });
        adapter = DropMarker(order);
        let _ = &adapter;
        Err(())
    }

    let early_return_order = RefCell::new(Vec::new());
    assert!(fail_after_adapter_setup(&early_return_order).is_err());
    assert_eq!(
        early_return_order.into_inner(),
        ["finalize", "adapter-drop"]
    );

    let explicit_finish_order = RefCell::new(Vec::new());
    {
        let adapter;
        let mut finalizer = AdapterFinalizer::new(|| {
            explicit_finish_order.borrow_mut().push("finalize");
            DesktopApplicationFinalization::Complete
        });
        adapter = DropMarker(&explicit_finish_order);
        finalizer.finish();
        let _ = &adapter;
    }
    assert_eq!(
        explicit_finish_order.into_inner(),
        ["finalize", "adapter-drop"]
    );
}

#[test]
fn adapter_finalizer_retries_without_releasing_adapter_ownership() {
    let attempts = RefCell::new(0_u8);
    let mut finalizer = AdapterFinalizer::new(|| {
        let mut attempts = attempts.borrow_mut();
        *attempts += 1;
        if *attempts < 3 {
            DesktopApplicationFinalization::Retry
        } else {
            DesktopApplicationFinalization::Complete
        }
    });

    finalizer.finish();
    drop(finalizer);
    assert_eq!(attempts.into_inner(), 3);
}

#[test]
fn desktop_capability_helper_is_the_normalizer_source_of_truth() {
    let capabilities = desktop_capability_set().expect("desktop capability set");
    capabilities.validate().expect("valid capability set");
    assert_eq!(
        desktop_capability_set_hash().expect("desktop capability set hash"),
        capabilities.canonical_hash
    );

    let normalizer = lifecycle::DesktopEventNormalizer::new(PersistentId::from_bytes([0x31; 16]))
        .expect("normalizer");
    assert_eq!(
        normalizer.capability_set_hash(),
        capabilities.canonical_hash
    );
}

#[test]
fn suspended_recovery_publishes_one_fresh_host_resume_with_zero_elapsed() {
    assert!(!DesktopRunOptions::default().resume_suspended_application);
    let host_instance_id = PersistentId::from_bytes([0x32; 16]);
    let mut normalizer =
        lifecycle::DesktopEventNormalizer::new(host_instance_id).expect("normalizer");
    let mut stats = DesktopEventStats::default();
    let mut callbacks = Vec::new();
    let mut sink = |events: &[PlatformEventV1], elapsed: Duration| {
        callbacks.push((events.to_vec(), elapsed));
        Ok(())
    };

    assert!(
        !publish_fresh_host_resume_if_requested(false, &mut normalizer, &mut sink, &mut stats,)
            .expect("disabled resume")
    );
    assert!(
        publish_fresh_host_resume_if_requested(true, &mut normalizer, &mut sink, &mut stats,)
            .expect("fresh-host resume")
    );

    assert_eq!(callbacks.len(), 1);
    assert_eq!(callbacks[0].1, Duration::ZERO);
    let events = &callbacks[0].0;
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.host_instance_id, host_instance_id);
    assert_eq!(event.source_sequence, 0);
    assert_eq!(event.platform_sample_tick, 0);
    assert_eq!(event.kind, PlatformEventKindV1::ResumeRequested);
    assert_eq!(
        event.capability_set_hash,
        desktop_capability_set_hash().expect("capability set hash")
    );
    let next_contracts::platform::PlatformEventPayloadV1::Reason { reason } = &event.payload else {
        panic!("fresh-host resume must carry a typed reason");
    };
    assert_eq!(
        reason.as_str(),
        "nextengine.platform.reason.fresh-host-ready"
    );
    assert_eq!(stats.normalized_events, 1);
    assert_eq!(stats.lifecycle_events, 1);
    assert_eq!(stats.last_platform_event_id, Some(event.platform_event_id));
}

#[test]
fn pacing_clock_excludes_cold_first_frame_without_hiding_later_overload() {
    let start = Instant::now();
    let mut clock = InteractivePacingClock::default();

    assert_eq!(clock.elapsed_for_pump(start), Duration::ZERO);
    assert_eq!(
        clock.elapsed_for_pump(start + Duration::from_secs(10)),
        Duration::ZERO,
        "startup pumps before the first submitted frame are not simulation time"
    );

    let first_submission = start + Duration::from_secs(12);
    clock.observe_frame_submission(first_submission);
    assert_eq!(
        clock.elapsed_for_pump(first_submission + Duration::from_millis(16)),
        Duration::from_millis(16)
    );
    assert_eq!(
        clock.elapsed_for_pump(first_submission + Duration::from_secs(5)),
        Duration::from_millis(4_984),
        "post-startup overload remains visible to the bounded scheduler"
    );
}

#[test]
fn pacing_clock_arms_only_once() {
    let start = Instant::now();
    let mut clock = InteractivePacingClock::default();
    clock.observe_frame_submission(start);
    clock.observe_frame_submission(start + Duration::from_secs(10));

    assert_eq!(
        clock.elapsed_for_pump(start + Duration::from_millis(16)),
        Duration::from_millis(16),
        "later submissions must not reset fixed-step pacing"
    );
}

#[test]
fn frame_pacing_sleeps_only_the_unused_part_of_the_sixty_hz_budget() {
    assert_eq!(
        remaining_frame_budget(Duration::from_millis(5)),
        Duration::from_nanos(11_666_667)
    );
    assert_eq!(
        remaining_frame_budget(INTERACTIVE_FRAME_INTERVAL),
        Duration::ZERO
    );
    assert_eq!(
        remaining_frame_budget(Duration::from_millis(40)),
        Duration::ZERO,
        "slow simulation or FIFO presentation must not receive a second delay"
    );
    assert_eq!(
        software_pacing_delay(Duration::from_millis(5), true),
        Duration::ZERO,
        "a successfully submitted FIFO frame must not also be software-paced"
    );
    assert_eq!(
        software_pacing_delay(Duration::from_millis(5), false),
        Duration::from_nanos(11_666_667),
        "a deferred frame keeps bounded anti-spin pacing"
    );
}

#[test]
fn keyboard_controls_use_engine_owned_paths_and_modifiers() {
    assert_eq!(
        keyboard_control_path(Scancode::W),
        Some("nextengine.input.keyboard.w")
    );
    // The core action map binds ui-nav to these exact contract paths; the
    // adapter must emit them or dialogue/menu navigation never resolves.
    assert_eq!(
        keyboard_control_path(Scancode::Up),
        Some("nextengine.input.keyboard.up")
    );
    assert_eq!(
        keyboard_control_path(Scancode::Down),
        Some("nextengine.input.keyboard.down")
    );
    assert_eq!(
        keyboard_control_path(Scancode::Left),
        Some("nextengine.input.keyboard.left")
    );
    assert_eq!(
        keyboard_control_path(Scancode::Right),
        Some("nextengine.input.keyboard.right")
    );
    assert_eq!(
        keyboard_control_path(Scancode::Return),
        Some("nextengine.input.keyboard.return")
    );
    assert_eq!(
        keyboard_control_path(Scancode::Escape),
        Some("nextengine.input.keyboard.escape")
    );
    assert_eq!(keyboard_control_path(Scancode::F1), None);
    assert_eq!(
        normalized_modifiers(Mod::LSHIFTMOD | Mod::RCTRLMOD),
        vec![
            "nextengine.input.modifier.shift",
            "nextengine.input.modifier.control"
        ]
    );
}

#[test]
fn relative_mouse_motion_uses_the_normalized_vector_control() {
    let observation =
        mouse_motion_observation(17, 3, 4.4, -2.6).expect("nonzero finite relative motion");
    assert_eq!(observation.source, lifecycle::DesktopEventSource::Mouse);
    assert_eq!(observation.platform_sample_tick, 17);
    let lifecycle::DesktopObservationKind::Control {
        control_path,
        device_class,
        phase,
        quantized_value,
        ..
    } = observation.kind
    else {
        panic!("mouse motion must be a normalized control");
    };
    assert_eq!(control_path, "nextengine.input.mouse.delta");
    assert_eq!(device_class, "nextengine.input.mouse");
    assert_eq!(phase, NormalizedControlPhaseV1::Changed);
    assert_eq!(quantized_value, vec![4, -3]);
    assert!(mouse_motion_observation(17, 3, 0.0, 0.0).is_none());
    assert!(mouse_motion_observation(17, 3, f32::NAN, 1.0).is_none());
    assert_eq!(quantize_mouse_delta(f32::MAX), Some(i16::MAX));
}

#[test]
fn fullscreen_shortcuts_are_shell_requests_not_close_requests() {
    assert!(is_fullscreen_shortcut(Some(Scancode::F11), Mod::NOMOD));
    assert!(is_fullscreen_shortcut(Some(Scancode::Return), Mod::LALTMOD));
    assert!(!is_fullscreen_shortcut(Some(Scancode::Escape), Mod::NOMOD));
}

#[test]
fn graphics_loss_has_stable_recovery_diagnostics() {
    let device = DesktopAdapterError::Graphics(vk::Result::ERROR_DEVICE_LOST);
    let surface = DesktopAdapterError::Graphics(vk::Result::ERROR_SURFACE_LOST_KHR);
    assert!(device.is_recoverable_presentation_loss());
    assert!(surface.is_recoverable_presentation_loss());
    assert_eq!(device.diagnostic_code(), "PRESENTATION_DEVICE_LOST");
    assert_eq!(surface.diagnostic_code(), "PRESENTATION_SURFACE_LOST");

    let exhausted = DesktopAdapterError::DeviceRecoveryLimitExceeded { maximum: 2 };
    assert_eq!(
        exhausted.diagnostic_code(),
        "PRESENTATION_DEVICE_RECOVERY_EXHAUSTED"
    );
}

#[test]
fn graphics_startup_failures_have_distinct_stable_diagnostics() {
    let loader = DesktopAdapterError::Loader("not found".to_owned());
    let version = DesktopAdapterError::LoaderVersionUnsupported {
        required: vk::API_VERSION_1_3,
        actual: vk::API_VERSION_1_2,
    };
    let icd = DesktopAdapterError::IcdUnavailable { error: None };
    let gpu = DesktopAdapterError::GpuUnsupported;

    assert_eq!(
        loader.diagnostic_code(),
        "PLATFORM_GRAPHICS_LOADER_UNAVAILABLE"
    );
    assert_eq!(
        version.diagnostic_code(),
        "PLATFORM_GRAPHICS_LOADER_VERSION_UNSUPPORTED"
    );
    assert_eq!(icd.diagnostic_code(), "PLATFORM_GRAPHICS_ICD_UNAVAILABLE");
    assert_eq!(gpu.diagnostic_code(), "GPU_UNSUPPORTED");
}

#[test]
fn application_callback_errors_preserve_their_stable_diagnostic() {
    let error = DesktopAdapterError::client("SESSION_RUNTIME_FAILED", "tick rejected");
    assert_eq!(error.diagnostic_code(), "SESSION_RUNTIME_FAILED");
    assert_eq!(error.to_string(), "SESSION_RUNTIME_FAILED: tick rejected");
}

#[test]
fn dynamic_snapshot_replacement_rejects_regression_and_unmarked_epoch_reset() {
    let current = test_snapshot(1, 4, 7, 10);
    let next = test_snapshot(1, 5, 8, 10);
    validate_presentation_snapshot_transition(&current, &next).expect("strict same-epoch progress");

    let same_epoch_tick_regression = test_snapshot(1, 5, 6, 10);
    assert!(
        validate_presentation_snapshot_transition(&current, &same_epoch_tick_regression).is_err()
    );

    let stale = test_snapshot(1, 4, 8, 10);
    assert_eq!(
        validate_presentation_snapshot_transition(&current, &stale)
            .expect_err("same-epoch sequence regression")
            .diagnostic_code(),
        "PRESENTATION_SNAPSHOT_TRANSITION_INVALID"
    );

    let reset = test_snapshot(2, 0, 8, 10);
    validate_presentation_snapshot_transition(&current, &reset).expect("explicit cut epoch reset");
    let rollback_reset = test_snapshot(2, 0, 3, 10);
    validate_presentation_snapshot_transition(&current, &rollback_reset)
        .expect("explicit recovery cut may roll simulation back");
    let unmarked_reset = test_snapshot(2, 1, 8, 10);
    assert!(validate_presentation_snapshot_transition(&current, &unmarked_reset).is_err());

    let foreign_project = test_snapshot(1, 5, 8, 11);
    assert!(validate_presentation_snapshot_transition(&current, &foreign_project).is_err());
}

#[test]
fn shared_frame_source_transfers_the_exact_immutable_projection() {
    let current = Arc::new(test_snapshot(1, 4, 7, 10));
    let next = Arc::new(test_snapshot(1, 5, 8, 10));
    let current_slot = RefCell::new(current);
    let dynamic_surfaces = RefCell::new(DynamicSurfaceState::empty());
    let published = Arc::clone(&next);
    let mut frame_source =
        move |_: &[PlatformEventV1], _: Duration, _: &mut crate::DesktopAudioOutputV1| {
            Ok::<_, DesktopAdapterError>(DesktopFramePublicationV1::snapshot_only(Some(
                Arc::clone(&published),
            )))
        };

    let mut audio = crate::DesktopAudioOutputV1::disabled();
    apply_frame_source_result(
        &current_slot,
        &dynamic_surfaces,
        &mut frame_source,
        &[],
        Duration::ZERO,
        &mut audio,
    )
    .expect("shared projection transition");

    assert!(Arc::ptr_eq(&current_slot.borrow(), &next));
    assert_eq!(dynamic_surfaces.borrow().publications(), 0);
}

#[test]
fn undeclared_dynamic_surface_publication_fails_closed_without_touching_the_snapshot() {
    let current = Arc::new(test_snapshot(1, 4, 7, 10));
    let next = Arc::new(test_snapshot(1, 5, 8, 10));
    let current_slot = RefCell::new(Arc::clone(&current));
    let dynamic_surfaces = RefCell::new(DynamicSurfaceState::empty());
    let update = Arc::new(
        DynamicSurfaceUpdateV1::new(
            AssetRevisionRefV1 {
                asset_id: next_contracts::ids::AssetId::from_bytes([0xd1; 16]),
                record_sha256: ContentHash::from_bytes([0xd2; 32]),
            },
            1,
            vec![[0, 0, 0], [1, 0, 0], [0, 0, 1]],
            vec![[0, i16::MAX, 0]; 3],
            vec![0, 1, 2],
        )
        .expect("valid update"),
    );
    let published = Arc::clone(&next);
    let mut frame_source =
        move |_: &[PlatformEventV1], _: Duration, _: &mut crate::DesktopAudioOutputV1| {
            Ok::<_, DesktopAdapterError>(DesktopFramePublicationV1 {
                snapshot: Some(Arc::clone(&published)),
                dynamic_surface_updates: vec![Arc::clone(&update)],
                particle_surface_update: None,
            })
        };

    let mut audio = crate::DesktopAudioOutputV1::disabled();
    let error = apply_frame_source_result(
        &current_slot,
        &dynamic_surfaces,
        &mut frame_source,
        &[],
        Duration::ZERO,
        &mut audio,
    )
    .expect_err("undeclared surface must be rejected");
    assert_eq!(
        error.diagnostic_code(),
        "PRESENTATION_DYNAMIC_SURFACE_UNDECLARED"
    );
    // The snapshot transition is applied before the surface batch is
    // validated, and the rejected batch leaves the surface state untouched.
    assert!(Arc::ptr_eq(&current_slot.borrow(), &next));
    assert_eq!(dynamic_surfaces.borrow().publications(), 0);
    assert!(dynamic_surfaces.borrow().current().is_empty());
}

#[test]
fn event_loop_iteration_budget_fails_before_exceeding_limit() {
    assert!(matches!(
        advance_event_loop_iteration(0, Some(0)),
        Err(DesktopAdapterError::EventLoopIterationLimitExceeded { maximum: 0 })
    ));
    assert_eq!(
        advance_event_loop_iteration(0, Some(1)).expect("first iteration"),
        1
    );
    let exhausted = advance_event_loop_iteration(1, Some(1))
        .expect_err("second iteration must exceed the budget");
    assert_eq!(
        exhausted.diagnostic_code(),
        "PLATFORM_EVENT_LOOP_ITERATION_LIMIT_EXCEEDED"
    );
    assert_eq!(
        advance_event_loop_iteration(41, None).expect("unbounded iteration"),
        42
    );
}

fn test_snapshot(
    epoch: u8,
    sequence: u64,
    simulation_tick: u64,
    project: u8,
) -> PresentationSnapshotV3 {
    PresentationSnapshotV3::new(
        ContentHash::from_bytes([epoch; 32]),
        sequence,
        simulation_tick,
        ContentHash::from_bytes([project; 32]),
        ContentHash::from_bytes([20; 32]),
        ContentHash::from_bytes([30; 32]),
        Vec::new(),
        1,
        ContentHash::from_bytes([40; 32]),
    )
    .expect("test snapshot")
}
