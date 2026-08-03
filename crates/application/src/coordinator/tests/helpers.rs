use super::*;

pub(super) fn delayed_callback_input_result(
    label: &str,
    callback_milliseconds: &[u64],
    callback_has_input: &[bool],
) -> (
    u64,
    ContentHash,
    ContentHash,
    next_contracts::ids::CommandLedgerHash,
) {
    assert_eq!(callback_milliseconds.len(), callback_has_input.len());
    let root = test_root(label);
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let movement = keyboard_movement_event(&mut game);
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();

    for (&milliseconds, &has_input) in callback_milliseconds.iter().zip(callback_has_input) {
        let events = if has_input {
            std::slice::from_ref(&movement)
        } else {
            &[]
        };
        scheduler
            .advance_reference_game(&mut game, Duration::from_millis(milliseconds), events)
            .expect("fixed-step callback");
    }
    assert_eq!(scheduler.pending_event_count(), 1);
    scheduler
        .advance_reference_game(&mut game, Duration::from_millis(34), &[])
        .expect("next fixed boundary");
    assert_eq!(scheduler.pending_event_count(), 0);

    let run = game.current_live_run().expect("final live result");
    let result = (
        run.ticks,
        run.authoritative_state_root,
        run.command_archive_root,
        run.command_ledger_hash,
    );
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
    result
}

pub(super) fn partitioned_live_result(
    label: &str,
    callback_count: u64,
) -> (
    u64,
    ContentHash,
    ContentHash,
    next_contracts::ids::CommandLedgerHash,
) {
    const INTERVAL_NANOSECONDS: u64 = 500_000_000;

    let root = test_root(label);
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let movement = keyboard_movement_event(&mut game);
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
    for index in 0..callback_count {
        let start = INTERVAL_NANOSECONDS * index / callback_count;
        let end = INTERVAL_NANOSECONDS * (index + 1) / callback_count;
        let events = if index == 0 {
            std::slice::from_ref(&movement)
        } else {
            &[]
        };
        scheduler
            .advance_reference_game(&mut game, Duration::from_nanos(end - start), events)
            .expect("fixed-step pump");
    }
    assert_eq!(scheduler.pending_event_count(), 0);
    let run = game.current_live_run().expect("final live result");
    let result = (
        run.ticks,
        run.authoritative_state_root,
        run.command_archive_root,
        run.command_ledger_hash,
    );
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
    result
}

pub(super) fn keyboard_movement_event(application: &mut ApplicationCoordinator) -> PlatformEventV1 {
    keyboard_movement_event_at(application, 0, NormalizedControlPhaseV1::Started, i16::MAX)
}

pub(super) fn keyboard_movement_event_at(
    application: &mut ApplicationCoordinator,
    source_sequence: u64,
    phase: NormalizedControlPhaseV1,
    value: i16,
) -> PlatformEventV1 {
    let binding = test_platform_host(application);
    keyboard_control_event(
        &binding,
        source_sequence,
        KEYBOARD_W_CONTROL_PATH_ID,
        phase,
        value,
    )
}

pub(super) fn keyboard_escape_event(
    binding: &super::super::platform_host::RegisteredPlatformHostV1,
    source_sequence: u64,
) -> PlatformEventV1 {
    keyboard_control_event(
        binding,
        source_sequence,
        KEYBOARD_ESCAPE_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
        i16::MAX,
    )
}

fn keyboard_control_event(
    binding: &super::super::platform_host::RegisteredPlatformHostV1,
    source_sequence: u64,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: i16,
) -> PlatformEventV1 {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(KEYBOARD_DEVICE_CLASS_ID).expect("device class"),
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(control_path).expect("control path"),
        phase,
        vec![value],
        Vec::new(),
        source_sequence,
        source_sequence,
    )
    .expect("control event");
    PlatformEventV1::new(
        binding.host_instance_id,
        SchemaId::new("nextengine.platform.source.cadence-test").expect("source class"),
        source_sequence,
        source_sequence,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        binding.capability_set_hash,
    )
    .expect("platform event")
}

pub(super) fn assert_run_roots_equal(
    left: &crate::ApplicationRunOutcomeV1,
    right: &crate::ApplicationRunOutcomeV1,
) {
    assert_eq!(left.ticks, right.ticks);
    assert_eq!(left.events, right.events);
    assert_eq!(left.rpg_events, right.rpg_events);
    assert_eq!(left.authoritative_revision, right.authoritative_revision);
    assert_eq!(
        left.authoritative_state_root,
        right.authoritative_state_root
    );
    assert_eq!(left.command_archive_root, right.command_archive_root);
    assert_eq!(
        left.command_identity_index_root,
        right.command_identity_index_root
    );
    assert_eq!(left.command_ledger_hash, right.command_ledger_hash);
}

pub(super) fn durable_v1_bytes(bytes: &[u8]) -> Vec<u8> {
    let decoded = next_contracts::canonical::decode_canonical_segment(
        bytes,
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )
    .expect("decode durable v3");
    let mut fields = decoded.fields;
    fields[0].payload = 1_u32.to_le_bytes().to_vec();
    fields.truncate(5);
    next_contracts::canonical::encode_canonical_segment(
        &decoded.owner_id,
        &decoded.schema_id,
        &decoded.segment_id,
        fields,
    )
    .expect("encode durable v1")
}

pub(super) fn durable_v2_bytes(bytes: &[u8]) -> Vec<u8> {
    let decoded = next_contracts::canonical::decode_canonical_segment(
        bytes,
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )
    .expect("decode durable v3");
    let mut fields = decoded.fields;
    fields[0].payload = 2_u32.to_le_bytes().to_vec();
    fields.truncate(6);
    next_contracts::canonical::encode_canonical_segment(
        &decoded.owner_id,
        &decoded.schema_id,
        &decoded.segment_id,
        fields,
    )
    .expect("encode durable v2")
}

pub(super) fn platform_reason_event(
    application: &mut ApplicationCoordinator,
    kind: PlatformEventKindV1,
    source_sequence: u64,
    reason: &str,
) -> PlatformEventV1 {
    let binding = test_platform_host(application);
    platform_reason_event_with_identity(
        binding.host_instance_id,
        binding.capability_set_hash,
        kind,
        source_sequence,
        reason,
    )
}

pub(super) fn platform_reason_event_with_identity(
    host_instance_id: PersistentId,
    capability_set_hash: ContentHash,
    kind: PlatformEventKindV1,
    source_sequence: u64,
    reason: &str,
) -> PlatformEventV1 {
    PlatformEventV1::new(
        host_instance_id,
        SchemaId::new("nextengine.platform.source.lifecycle-test").expect("source class"),
        source_sequence,
        source_sequence,
        kind,
        PlatformEventPayloadV1::Reason {
            reason: SchemaId::new(reason).expect("platform reason"),
        },
        capability_set_hash,
    )
    .expect("platform lifecycle event")
}

pub(super) fn test_platform_host(
    application: &mut ApplicationCoordinator,
) -> super::super::platform_host::RegisteredPlatformHostV1 {
    if let Some(binding) = application.platform_host.clone() {
        return binding;
    }
    let capabilities = application
        .launch
        .platform_capability_set
        .clone()
        .expect("interactive test launch has capabilities");
    let host_instance_id = application
        .register_platform_host(&capabilities)
        .expect("register test platform host");
    let binding = application
        .platform_host
        .clone()
        .expect("registered test platform host");
    assert_eq!(binding.host_instance_id, host_instance_id);
    binding
}

pub(super) fn interactive_launch(root: &std::path::Path) -> LaunchRequestV1 {
    LaunchRequestV1::reference(
        root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    )
}

pub(super) fn headless_launch(root: &std::path::Path) -> LaunchRequestV1 {
    LaunchRequestV1::reference(
        root,
        CompositionRootV1::Headless,
        PresentationTargetKindV1::None,
    )
}

pub(super) fn test_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "nextengine-application-{label}-{}-{}",
        std::process::id(),
        TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}

pub(super) fn cleanup(root: std::path::PathBuf) {
    if root.exists() {
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
