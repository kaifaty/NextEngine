use super::*;

fn root_motion_command(
    fixture: &PhysicalFixture,
    sequence: u64,
    expected_body_revision: u64,
) -> WorldCommand {
    let tick = fixture.runtime.next_tick();
    let intent = RootMotionIntentV1 {
        schema_version: ROOT_MOTION_INTENT_SCHEMA_VERSION,
        subject_id: fixture.physics_body_id.subject_id,
        intent_sequence: sequence,
        source_graph_hash: ContentHash::from_bytes([31; 32]),
        source_clip_hash: ContentHash::from_bytes([32; 32]),
        source_action_or_ability_phase_id: SchemaId::new(ROOT_MOTION_MOVE_PERFORMED_PHASE_ID)
            .expect("phase"),
        source_animation_tick: tick,
        interval_us: ((tick + 1) * 1_000_000 / 30) - (tick * 1_000_000 / 30),
        quantized_local_translation: [0, 0, 100_000],
        quantized_local_yaw: 0,
        locomotion_profile_hash: capsule_root_motion_profile_hash_v1(30).expect("profile"),
        expected_intent_state_revision: tick,
        expected_body_revision,
    };
    WorldCommand::root_motion(
        fixture.command_stream_id,
        fixture.principal.clone(),
        sequence,
        tick,
        fixture.physics_body_id.subject_id,
        intent,
    )
    .expect("root-motion command")
}

#[test]
fn root_motion_command_is_admitted_clipped_or_rejected_without_pose_authority() {
    let mut accepted = physical_fixture();
    let accepted_revision = accepted.runtime.physics_snapshot().sorted_body_states
        [&accepted.physics_body_id]
        .body_revision;
    let command = root_motion_command(&accepted, 0, accepted_revision);
    let bytes = command.canonical_bytes().expect("command bytes");
    assert_eq!(
        WorldCommand::from_canonical_bytes(&bytes, Default::default()).expect("decode command"),
        command,
    );
    let report = accepted
        .runtime
        .run_tick([command])
        .expect("accepted root motion");
    assert_eq!(report.results[0].disposition, CommandDisposition::Committed);
    assert_eq!(report.physics_step_input.accepted_intents.len(), 1);
    assert_eq!(
        report.physics_step_input.accepted_intents[0].direction_q15,
        [0, 32_767],
    );
    assert!(matches!(
        report.command_batches[0].body.envelopes[0].payload,
        CommandPayload::RootMotion(_)
    ));
    assert_eq!(
        accepted.runtime.physics_snapshot().sorted_body_states[&accepted.physics_body_id]
            .pose
            .translation_micrometres[2],
        100_000,
    );

    let mut clipped = clipped_physical_fixture();
    let clipped_revision = clipped.runtime.physics_snapshot().sorted_body_states
        [&clipped.physics_body_id]
        .body_revision;
    let clipped_report = clipped
        .runtime
        .run_tick([root_motion_command(&clipped, 0, clipped_revision)])
        .expect("clipped root motion");
    assert_eq!(
        clipped_report.results[0].disposition,
        CommandDisposition::Committed
    );
    let clipped_z = clipped.runtime.physics_snapshot().sorted_body_states[&clipped.physics_body_id]
        .pose
        .translation_micrometres[2];
    assert!(clipped_z > 0 && clipped_z < 100_000);

    let mut rejected = physical_fixture();
    let before =
        rejected.runtime.physics_snapshot().sorted_body_states[&rejected.physics_body_id].clone();
    let rejected_report = rejected
        .runtime
        .run_tick([root_motion_command(&rejected, 0, before.body_revision + 1)])
        .expect("stale root intent rejects normally");
    assert_eq!(
        rejected_report.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::RootMotionIntentRejected),
    );
    assert!(
        rejected_report
            .physics_step_input
            .accepted_intents
            .is_empty()
    );
    assert!(rejected_report.events.is_empty());
    let after = &rejected.runtime.physics_snapshot().sorted_body_states[&rejected.physics_body_id];
    assert_eq!(after.body_revision, before.body_revision);
    assert_eq!(after.pose, before.pose);

    let mut wrong_interval = physical_fixture();
    let revision = wrong_interval.runtime.physics_snapshot().sorted_body_states
        [&wrong_interval.physics_body_id]
        .body_revision;
    let mut command = root_motion_command(&wrong_interval, 0, revision);
    let CommandPayload::RootMotion(intent) = &mut command.payload else {
        panic!("root-motion fixture command")
    };
    intent.interval_us += 2;
    command
        .refresh_command_id()
        .expect("mutated interval command identity");
    let report = wrong_interval
        .runtime
        .run_tick([command])
        .expect("wrong fixed-tick interval rejects normally");
    assert_eq!(
        report.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::RootMotionIntentRejected),
    );
    assert!(report.physics_step_input.accepted_intents.is_empty());
    assert!(report.events.is_empty());
    assert_eq!(
        wrong_interval.runtime.physics_snapshot().sorted_body_states
            [&wrong_interval.physics_body_id]
            .pose
            .translation_micrometres,
        [0, 900_000, 0],
    );

    let mut wrong_source = physical_fixture();
    let before = wrong_source.runtime.physics_snapshot().sorted_body_states
        [&wrong_source.physics_body_id]
        .clone();
    let mut command = root_motion_command(&wrong_source, 0, before.body_revision);
    let CommandPayload::RootMotion(intent) = &mut command.payload else {
        panic!("root-motion fixture command")
    };
    intent.source_clip_hash = ContentHash::from_bytes([33; 32]);
    command
        .refresh_command_id()
        .expect("mutated source command identity");
    let report = wrong_source
        .runtime
        .run_tick([command])
        .expect("unregistered animation source rejects normally");
    assert_eq!(
        report.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::RootMotionIntentRejected),
    );
    assert!(report.physics_step_input.accepted_intents.is_empty());
    assert!(report.events.is_empty());
    let after =
        &wrong_source.runtime.physics_snapshot().sorted_body_states[&wrong_source.physics_body_id];
    assert_eq!(after.body_revision, before.body_revision);
    assert_eq!(after.pose, before.pose);
}

#[test]
fn root_motion_admission_accepts_the_fractional_tick_interval_quanta() {
    let mut fixture = physical_fixture();
    fixture.runtime.run_tick([]).expect("tick zero");
    fixture.runtime.run_tick([]).expect("tick one");
    let revision = fixture.runtime.physics_snapshot().sorted_body_states[&fixture.physics_body_id]
        .body_revision;
    let command = root_motion_command(&fixture, 0, revision);
    let CommandPayload::RootMotion(intent) = &command.payload else {
        panic!("root-motion fixture command")
    };
    assert_eq!(intent.source_animation_tick, 2);
    assert_eq!(intent.interval_us, 33_334);
    let report = fixture
        .runtime
        .run_tick([command])
        .expect("fractional cadence root motion");
    assert_eq!(report.results[0].disposition, CommandDisposition::Committed);
    assert_eq!(report.physics_step_input.accepted_intents.len(), 1);
}
