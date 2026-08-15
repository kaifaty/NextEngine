use super::*;

#[test]
fn prepared_tick_is_non_mutating_and_commits_the_exact_preview() {
    let mut prepared_fixture = physical_fixture();
    let mut ordinary_fixture = physical_fixture();
    let sample = movement_sample(
        &prepared_fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        None,
    );
    let before_snapshot = prepared_fixture.runtime.snapshot();
    let before_physics = prepared_fixture.runtime.physics_checkpoint().clone();

    let mut preparation = prepared_fixture.runtime.tick_preparation();
    preparation
        .enqueue_input_sample(&prepared_fixture.principal, sample.clone())
        .expect("stage input");
    let prepared = preparation.prepare([]).expect("prepare tick");
    assert!(!prepared.report_is_materialized());
    let preview = prepared.report().clone();
    assert!(prepared.report_is_materialized());
    let preview_checkpoint = prepared
        .world_checkpoint_with_canonical_components()
        .expect("prepared checkpoint")
        .0;

    assert_eq!(prepared_fixture.runtime.snapshot(), before_snapshot);
    assert_eq!(
        prepared_fixture.runtime.physics_checkpoint(),
        &before_physics
    );
    assert!(
        prepared_fixture
            .runtime
            .last_closed_ingress_batch()
            .is_none()
    );

    let validated = prepared_fixture
        .runtime
        .validate_prepared_tick(prepared)
        .expect("validate prepared tick");
    let committed = prepared_fixture.runtime.commit_validated_tick(validated);
    ordinary_fixture
        .runtime
        .enqueue_input_sample(&ordinary_fixture.principal, sample)
        .expect("enqueue ordinary input");
    let ordinary = ordinary_fixture
        .runtime
        .run_tick([])
        .expect("ordinary tick");

    assert_eq!(committed, preview);
    assert_eq!(committed, ordinary);
    assert_eq!(
        prepared_fixture
            .runtime
            .world_checkpoint()
            .expect("committed checkpoint"),
        preview_checkpoint
    );
}

#[test]
fn reportless_commit_matches_the_public_tick_report_path() {
    let mut fast_fixture = physical_fixture();
    let mut ordinary_fixture = physical_fixture();
    let sample = movement_sample(
        &fast_fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        None,
    );

    let mut preparation = fast_fixture.runtime.tick_preparation();
    preparation
        .enqueue_input_sample(&fast_fixture.principal, sample.clone())
        .expect("stage fast input");
    let prepared = preparation.prepare([]).expect("prepare fast tick");
    assert_eq!(prepared.next_tick(), 1);
    let _events = prepared.events();
    let _ = prepared.physics_snapshot();
    assert!(!prepared.report_is_materialized());
    let validated = fast_fixture
        .runtime
        .validate_prepared_tick(prepared)
        .expect("validate fast tick");
    fast_fixture
        .runtime
        .commit_validated_tick_without_report(validated);

    ordinary_fixture
        .runtime
        .enqueue_input_sample(&ordinary_fixture.principal, sample)
        .expect("stage ordinary input");
    ordinary_fixture
        .runtime
        .run_tick([])
        .expect("ordinary tick");

    assert_eq!(
        fast_fixture.runtime.snapshot(),
        ordinary_fixture.runtime.snapshot()
    );
    assert_eq!(
        fast_fixture.runtime.physics_checkpoint(),
        ordinary_fixture.runtime.physics_checkpoint()
    );
    let fast_checkpoint = fast_fixture
        .runtime
        .world_checkpoint_with_canonical_components()
        .expect("fast checkpoint");
    let ordinary_checkpoint = ordinary_fixture
        .runtime
        .world_checkpoint_with_canonical_components()
        .expect("ordinary checkpoint");
    assert_eq!(fast_checkpoint.0, ordinary_checkpoint.0);
    assert_eq!(fast_checkpoint.1, ordinary_checkpoint.1);
}

#[test]
fn reportless_commit_releases_staged_ledger_ownership() {
    let mut fixture = physical_fixture();
    for sequence in 0..4 {
        // Performance regression guard, not a public contract: a staged
        // generation field that outlives the commit keeps the shared map
        // alive and forces copy-on-write to deep-clone the retained history
        // into a fresh allocation. In-place commits keep the map pointer
        // stable.
        let identity_before =
            std::sync::Arc::as_ptr(&fixture.runtime.command_ledger.identity_index.body.bindings);
        let causal_before = std::sync::Arc::as_ptr(
            &fixture
                .runtime
                .command_ledger
                .causal_identity_registry
                .bindings,
        );
        let sample = movement_sample(
            &fixture,
            sequence,
            PlayerActionPhaseV1::Performed,
            [0, 32_767],
            None,
        );
        let mut preparation = fixture.runtime.tick_preparation();
        preparation
            .enqueue_input_sample(&fixture.principal, sample)
            .expect("stage input");
        let prepared = preparation.prepare([]).expect("prepare tick");
        let validated = fixture
            .runtime
            .validate_prepared_tick(prepared)
            .expect("validate tick");
        fixture
            .runtime
            .commit_validated_tick_without_report(validated);
        assert_eq!(
            identity_before,
            std::sync::Arc::as_ptr(&fixture.runtime.command_ledger.identity_index.body.bindings),
            "reportless commit must mutate the live identity map in place"
        );
        assert_eq!(
            causal_before,
            std::sync::Arc::as_ptr(
                &fixture
                    .runtime
                    .command_ledger
                    .causal_identity_registry
                    .bindings
            ),
            "reportless commit must mutate the causal registry in place"
        );
    }
}

#[test]
fn prepared_tick_rejects_a_stale_runtime_generation() {
    let mut fixture = physical_fixture();
    let prepared = fixture
        .runtime
        .tick_preparation()
        .prepare([])
        .expect("prepare tick");

    fixture.runtime.run_tick([]).expect("advance live runtime");
    let error = fixture
        .runtime
        .validate_prepared_tick(prepared)
        .err()
        .expect("stale preparation must fail");

    assert_eq!(error, RuntimeFatalError::PreparedGenerationStale);
    assert_eq!(error.stable_code(), "RUNTIME_PREPARED_GENERATION_STALE");
    assert_eq!(fixture.runtime.next_tick(), 1);
}

#[test]
fn prepared_tick_generation_includes_the_exact_ingress_checkpoint() {
    let mut fixture = physical_fixture();
    let prepared = fixture
        .runtime
        .tick_preparation()
        .prepare([])
        .expect("prepare tick");
    let sample = movement_sample(
        &fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        None,
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, sample)
        .expect("mutate live ingress only");

    let error = fixture
        .runtime
        .validate_prepared_tick(prepared)
        .err()
        .expect("ingress drift must stale the preparation");
    assert_eq!(error, RuntimeFatalError::PreparedGenerationStale);
    assert_eq!(fixture.runtime.next_tick(), 0);
    assert_eq!(
        fixture
            .runtime
            .snapshot()
            .ingress_checkpoint
            .current_samples
            .len(),
        1
    );
    assert!(fixture.runtime.last_closed_ingress_batch().is_none());
}

#[test]
fn physx_fallback_is_confined_to_world_activation() {
    let bootstrap = RuntimeBootstrapV4::neutral_empty().expect("neutral bootstrap");
    let runtime = RuntimeState::new_with_physics_options(
        bootstrap.clone(),
        AuthorityRegistry::new(),
        PhysicsLaunchOptions::new(PhysicsBackendPolicy::PreferPhysXThenReference),
    )
    .expect("pre-activation fallback");
    assert_eq!(
        runtime.physics_backend_kind(),
        PhysicsBackendKind::Reference
    );

    let required = RuntimeState::new_with_physics_options(
        bootstrap,
        AuthorityRegistry::new(),
        PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX),
    )
    .expect_err("old reference-only profile cannot activate PhysX");
    assert!(matches!(
        required,
        SnapshotRestoreError::PhysicsController(_)
    ));
}

#[test]
fn action_frames_move_capsule_exactly_and_wall_time_is_nonauthoritative() {
    let mut fixture = physical_fixture();
    for sequence in 0..3 {
        let sample = movement_sample(
            &fixture,
            sequence,
            PlayerActionPhaseV1::Performed,
            [0, 32_767],
            Some(i64::try_from(sequence).expect("small") * 999),
        );
        fixture
            .runtime
            .enqueue_input_sample(&fixture.principal, sample)
            .expect("enqueue");
        let report = fixture.runtime.run_tick([]).expect("movement tick");
        assert_eq!(report.results[0].disposition, CommandDisposition::Committed);
        assert_eq!(report.events.len(), 1);
        assert_eq!(
            report.mapping_receipts[0].frame_code,
            InputMappingCodeV1::Accepted
        );
    }
    let body = &fixture.runtime.physics_snapshot().sorted_body_states[&fixture.physics_body_id];
    assert_eq!(body.pose.translation_micrometres, [0, 900_000, 300_000]);
    assert_eq!(fixture.runtime.physics_snapshot().physics_tick, 6);
}

#[test]
fn invalid_and_colliding_input_never_reaches_command_ledger() {
    let mut fixture = physical_fixture();
    let diagonal = movement_sample(
        &fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [32_767, 32_767],
        None,
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, diagonal)
        .expect("enqueue diagonal");
    let invalid = fixture.runtime.run_tick([]).expect("mapping rejection");
    assert!(invalid.results.is_empty());
    assert!(invalid.events.is_empty());
    assert_eq!(
        invalid.mapping_receipts[0].frame_code,
        InputMappingCodeV1::ValueOutOfProfile
    );
    assert_eq!(
        invalid
            .snapshot
            .command_ledger
            .streams
            .values()
            .next()
            .expect("stream")
            .finalized_receipt_count,
        0
    );

    let first = movement_sample(
        &fixture,
        1,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        None,
    );
    let second = movement_sample(
        &fixture,
        1,
        PlayerActionPhaseV1::Performed,
        [0, -32_767],
        None,
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, first)
        .expect("first");
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, second)
        .expect("second");
    let collision = fixture.runtime.run_tick([]).expect("input collision");
    assert!(collision.results.is_empty());
    assert_eq!(
        collision
            .closed_ingress_batch
            .body
            .equivalence_receipts
            .len(),
        1
    );
    assert_eq!(
        collision
            .snapshot
            .command_ledger
            .streams
            .values()
            .next()
            .expect("stream")
            .finalized_receipt_count,
        0
    );
}

#[test]
fn ingress_rejects_noncanonical_action_order_against_the_exact_registry() {
    let mut fixture = physical_fixture();
    let sample = exact_player_sample(
        &fixture,
        0,
        vec![
            PlayerActionV1 {
                action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("movement action"),
                phase: PlayerActionPhaseV1::Performed,
                value: PlayerActionValueV1::Vector2Q15([0, 32_767]),
                semantic_occurrence_ordinal: 0,
            },
            PlayerActionV1 {
                action_id: SchemaId::new(CORE_CAMERA_ORBIT_ACTION_ID).expect("camera action"),
                phase: PlayerActionPhaseV1::Performed,
                value: PlayerActionValueV1::Vector2Q15([12, -9]),
                semantic_occurrence_ordinal: 1,
            },
        ],
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, sample)
        .expect("sample is structurally admissible");

    let report = fixture.runtime.run_tick([]).expect("mapping close");
    assert!(report.results.is_empty());
    assert!(report.events.is_empty());
    assert_eq!(
        report.mapping_receipts[0].frame_code,
        InputMappingCodeV1::FrameInvalid
    );
}

#[test]
fn ingress_rejects_a_phase_not_declared_by_the_exact_action_map() {
    let mut fixture = physical_fixture();
    let sample = exact_player_sample(
        &fixture,
        0,
        vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_CAMERA_ORBIT_ACTION_ID).expect("camera action"),
            phase: PlayerActionPhaseV1::Started,
            value: PlayerActionValueV1::Vector2Q15([12, -9]),
            semantic_occurrence_ordinal: 0,
        }],
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, sample)
        .expect("sample is structurally admissible");

    let report = fixture.runtime.run_tick([]).expect("mapping close");
    assert!(report.results.is_empty());
    assert!(report.events.is_empty());
    assert_eq!(
        report.mapping_receipts[0].frame_code,
        InputMappingCodeV1::FrameInvalid
    );
}

#[test]
fn input_configuration_activation_is_atomic_revision_bound_and_queue_safe() {
    let mut fixture = physical_fixture();
    let original = fixture
        .runtime
        .player_controller_registry
        .bindings
        .get(&fixture.source_id)
        .expect("controller binding")
        .clone();

    fixture
        .runtime
        .activate_player_input_configuration(
            fixture.source_id,
            original.action_map.clone(),
            original.context_stack.clone(),
        )
        .expect("same exact registration is idempotent");

    let mut collision_actions = original.action_map.actions.clone();
    let movement = collision_actions
        .iter_mut()
        .find(|action| action.action_id.as_str() == CORE_MOVE_ACTION_ID)
        .expect("movement action");
    let north = movement
        .binding_slots
        .iter_mut()
        .find(|binding| {
            matches!(
                binding.transform,
                ActionBindingTransformV1::Vector2ContributionQ15 {
                    contribution_q15: [0, 32_767]
                }
            )
        })
        .expect("north binding");
    north.transform = ActionBindingTransformV1::Vector2ContributionQ15 {
        contribution_q15: [0, 32_766],
    };
    let collision_map = ActionMapManifestV1::new(
        original.action_map.action_map_id.clone(),
        original.action_map.revision,
        original.action_map.supported_device_classes.clone(),
        collision_actions,
    )
    .expect("same-revision collision map");
    assert_eq!(
        fixture.runtime.activate_player_input_configuration(
            fixture.source_id,
            collision_map,
            original.context_stack.clone(),
        ),
        Err(InputContractError::RegistryCollision)
    );
    assert_eq!(
        fixture
            .runtime
            .player_controller_registry
            .bindings
            .get(&fixture.source_id)
            .expect("binding after collision"),
        &original
    );

    let queued = movement_sample(
        &fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        None,
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, queued)
        .expect("old revision queued");
    let next_map = ActionMapManifestV1::new(
        original.action_map.action_map_id.clone(),
        original.action_map.revision + 1,
        original.action_map.supported_device_classes.clone(),
        original.action_map.actions.clone(),
    )
    .expect("next action map");
    let next_context = InputContextStackV1::new(
        original.context_stack.stack_id.clone(),
        original.context_stack.revision + 1,
        original.context_stack.entries.clone(),
    )
    .expect("next context stack");
    assert_eq!(
        fixture.runtime.activate_player_input_configuration(
            fixture.source_id,
            next_map.clone(),
            next_context.clone(),
        ),
        Err(InputContractError::InvalidProfile)
    );
    assert_eq!(
        fixture
            .runtime
            .run_tick([])
            .expect("old queued revision remains valid")
            .mapping_receipts[0]
            .frame_code,
        InputMappingCodeV1::Accepted
    );

    fixture
        .runtime
        .activate_player_input_configuration(
            fixture.source_id,
            next_map.clone(),
            next_context.clone(),
        )
        .expect("next revision activates at an empty ingress boundary");
    let activated = fixture
        .runtime
        .player_controller_registry
        .bindings
        .get(&fixture.source_id)
        .expect("activated binding");
    assert_eq!(activated.action_map_hash, next_map.content_hash);
    assert_eq!(activated.action_map_revision, next_map.revision);
    assert_eq!(activated.context_stack_hash, next_context.content_hash);
    assert_eq!(activated.context_stack_revision, next_context.revision);

    let sample = exact_player_sample(
        &fixture,
        1,
        vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("movement action"),
            phase: PlayerActionPhaseV1::Performed,
            value: PlayerActionValueV1::Vector2Q15([0, 32_767]),
            semantic_occurrence_ordinal: 0,
        }],
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, sample)
        .expect("new revision queued");
    assert_eq!(
        fixture
            .runtime
            .run_tick([])
            .expect("new revision tick")
            .mapping_receipts[0]
            .frame_code,
        InputMappingCodeV1::Accepted
    );
}

#[test]
fn exact_duplicate_input_closes_and_moves_once() {
    let mut fixture = physical_fixture();
    let sample = movement_sample(
        &fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        Some(1),
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, sample.clone())
        .expect("first duplicate");
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, sample)
        .expect("second duplicate");
    let report = fixture.runtime.run_tick([]).expect("tick");
    assert_eq!(report.closed_ingress_batch.body.input_samples.len(), 1);
    assert_eq!(
        report.stage_trace[0],
        StageTraceEntry {
            stage: TransactionStage::IngressClose,
            received: 1,
            accepted: 1,
            rejected: 0,
            committed: 1,
            deduplicated: 1,
        }
    );
    assert_eq!(
        report.physics_snapshot.sorted_body_states[&fixture.physics_body_id]
            .pose
            .translation_micrometres,
        [0, 900_000, 100_000]
    );
}

#[test]
fn player_mapping_binds_the_assignment_with_the_exact_source_class() {
    let mut fixture = physical_fixture();
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: 0,
        action_map_hash: fixture.action_map_hash,
        action_map_revision: 1,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: 1,
        actions: vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_INTERACT_ACTION_ID).expect("action"),
            phase: PlayerActionPhaseV1::Started,
            value: PlayerActionValueV1::Digital(true),
            semantic_occurrence_ordinal: 0,
        }],
    };
    let player = InputSampleV1 {
        schema_version: 1,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
        source_id: fixture.source_id,
        source_sequence: 0,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("schema"),
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes().expect("frame"),
        sampled_wall_time: None,
    };
    let foreign = InputSampleV1 {
        source_class: SchemaId::new("aaa.foreign-input").expect("foreign source class"),
        ..player.clone()
    };
    fixture.runtime.ingress_checkpoint.current_samples = vec![foreign, player];
    let closed = super::ingress::close_ingress(
        0,
        1,
        &fixture.runtime.admission_limits,
        &fixture.runtime.player_controller_registry,
        true,
        &mut fixture.runtime.ingress_checkpoint,
    )
    .expect("closed ingress");

    assert_eq!(closed.pending_interactions.len(), 1);
    assert_eq!(
        closed.pending_interactions[0]
            .assignment
            .source_class
            .as_str(),
        PLAYER_ACTION_SOURCE_CLASS
    );
}

#[test]
fn replay_driver_rejects_query_batch_divergence_before_state_commit() {
    let mut recorded = fixture();
    let direct = command(&recorded, 0, 0);
    let initial_checkpoint = recorded
        .runtime
        .world_checkpoint()
        .expect("initial checkpoint");
    let authority = recorded.runtime.authority.clone();
    let report = recorded
        .runtime
        .run_tick([direct.clone()])
        .expect("recorded tick");
    let mut wrong_query_batch = report.physics_query_batch.clone();
    wrong_query_batch.snapshot_selector.physics_snapshot_hash = ContentHash::from_bytes([0xee; 32]);

    let mut replay =
        RuntimeReplayDriver::new(initial_checkpoint.clone(), authority).expect("replay driver");
    let error = replay
        .replay_tick_with_query_facts(
            report.closed_ingress_batch.clone(),
            vec![direct],
            &report.command_batches[0],
            &report.physics_step_input,
            &report.contact_batch,
            &report.targeting_intents,
            &report.authoritative_targeting_queries,
            &wrong_query_batch,
            &report.physics_query_results,
            &report.command_batches[1],
        )
        .expect_err("query batch divergence");

    assert!(matches!(
        error,
        RuntimeReplayError::PhysicsQueryBatchMismatch { tick: 0 }
    ));
    assert_eq!(
        replay.world_checkpoint().expect("checkpoint after failure"),
        initial_checkpoint
    );
}

#[test]
fn presentation_only_camera_action_is_admitted_without_a_world_command() {
    let mut fixture = physical_fixture();
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: 0,
        action_map_hash: fixture.action_map_hash,
        action_map_revision: 1,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: 1,
        actions: vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_CAMERA_ORBIT_ACTION_ID).expect("camera action"),
            phase: PlayerActionPhaseV1::Performed,
            value: PlayerActionValueV1::Vector2Q15([12, -9]),
            semantic_occurrence_ordinal: 0,
        }],
    };
    let sample = InputSampleV1 {
        schema_version: 1,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
        source_id: fixture.source_id,
        source_sequence: 0,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("schema"),
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes().expect("frame"),
        sampled_wall_time: None,
    };
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, sample)
        .expect("enqueue camera frame");
    let report = fixture.runtime.run_tick([]).expect("camera-only tick");
    assert_eq!(
        report.mapping_receipts[0].frame_code,
        InputMappingCodeV1::Accepted
    );
    assert!(report.command_batches[0].body.envelopes.is_empty());
    assert!(report.results.is_empty());
    assert!(report.events.is_empty());
}

#[test]
fn explicitly_late_input_is_persisted_for_the_following_tick() {
    let mut fixture = physical_fixture();
    let sample = movement_sample(
        &fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        Some(777),
    );
    fixture
        .runtime
        .enqueue_input_sample_for_next_tick(&fixture.principal, sample)
        .expect("enqueue after current close barrier");

    let current = fixture.runtime.run_tick([]).expect("current tick");
    assert!(current.closed_ingress_batch.body.input_samples.is_empty());
    assert!(current.mapping_receipts.is_empty());
    assert_eq!(
        current.physics_snapshot.sorted_body_states[&fixture.physics_body_id]
            .pose
            .translation_micrometres,
        [0, 900_000, 0]
    );

    let following = fixture.runtime.run_tick([]).expect("following tick");
    assert_eq!(following.closed_ingress_batch.body.input_samples.len(), 1);
    assert_eq!(
        following.mapping_receipts[0].frame_code,
        InputMappingCodeV1::Accepted
    );
    assert_eq!(
        following.physics_snapshot.sorted_body_states[&fixture.physics_body_id]
            .pose
            .translation_micrometres,
        [0, 900_000, 100_000]
    );
}

#[test]
fn input_arrival_permutations_close_to_identical_batches_and_state() {
    let mut left = physical_fixture();
    let mut right = physical_fixture();
    let first = movement_sample(
        &left,
        0,
        PlayerActionPhaseV1::Performed,
        [32_767, 0],
        Some(1),
    );
    let second = movement_sample(
        &left,
        1,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        Some(2),
    );
    for sample in [first.clone(), second.clone()] {
        left.runtime
            .enqueue_input_sample(&left.principal, sample)
            .expect("left enqueue");
    }
    for sample in [second, first] {
        right
            .runtime
            .enqueue_input_sample(&right.principal, sample)
            .expect("right enqueue");
    }
    let left = left.runtime.run_tick([]).expect("left tick");
    let right = right.runtime.run_tick([]).expect("right tick");
    assert_eq!(left.closed_ingress_batch, right.closed_ingress_batch);
    assert_eq!(left.command_batches, right.command_batches);
    assert_eq!(left.results, right.results);
    assert_eq!(left.events, right.events);
    assert_eq!(left.physics_snapshot, right.physics_snapshot);
    assert_eq!(left.snapshot.command_ledger, right.snapshot.command_ledger);
    assert!(
        left.mapping_receipts
            .iter()
            .all(|receipt| receipt.frame_code == InputMappingCodeV1::FrameInvalid)
    );
    assert!(
        left.mapping_receipts
            .iter()
            .all(|receipt| receipt.frame_code == InputMappingCodeV1::FrameInvalid)
    );
}

#[test]
fn ui_actions_are_admitted_as_replayable_evidence_without_world_commands() {
    let mut fixture = physical_fixture();
    let pause_press = exact_player_sample(
        &fixture,
        0,
        vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_UI_BACK_ACTION_ID).expect("ui back action"),
            phase: PlayerActionPhaseV1::Started,
            value: PlayerActionValueV1::Digital(true),
            semantic_occurrence_ordinal: 0,
        }],
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, pause_press)
        .expect("enqueue ui back press");
    let report = fixture.runtime.run_tick([]).expect("ui back tick");
    assert_eq!(
        report.mapping_receipts[0].frame_code,
        InputMappingCodeV1::Accepted
    );
    assert!(report.command_batches[0].body.envelopes.is_empty());
    assert!(report.events.is_empty());

    let invalid_press = exact_player_sample(
        &fixture,
        1,
        vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_UI_BACK_ACTION_ID).expect("ui back action"),
            phase: PlayerActionPhaseV1::Started,
            value: PlayerActionValueV1::Digital(false),
            semantic_occurrence_ordinal: 0,
        }],
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, invalid_press)
        .expect("enqueue invalid ui back press");
    assert_eq!(
        fixture
            .runtime
            .run_tick([])
            .expect("invalid ui back tick")
            .mapping_receipts[0]
            .frame_code,
        InputMappingCodeV1::ValueOutOfProfile
    );

    // Modal transitions are revisioned reconfigurations of the bound stack
    // identity: the ui-menu layer activates as revision 2 of the gameplay
    // stack at an empty ingress boundary.
    let menu_stack = InputContextStackV1::new(
        SchemaId::new(CORE_GAMEPLAY_CONTEXT_STACK_ID).expect("gameplay stack id"),
        2,
        vec![
            InputContextV1::new(
                SchemaId::new(CORE_UI_MENU_CONTEXT_ID).expect("ui menu context id"),
                1,
                200,
                InputContextCapturePolicyV1::CaptureAll,
                [
                    CORE_UI_BACK_ACTION_ID,
                    CORE_UI_CONFIRM_ACTION_ID,
                    CORE_UI_NAVIGATE_ACTION_ID,
                ]
                .into_iter()
                .map(SchemaId::new)
                .collect::<Result<Vec<_>, _>>()
                .expect("ui action ids"),
            )
            .expect("ui menu context"),
        ],
    )
    .expect("ui menu stack revision");
    fixture
        .runtime
        .activate_player_input_configuration(
            fixture.source_id,
            ActionMapManifestV1::core_keyboard_mouse_v1().expect("core action map"),
            menu_stack,
        )
        .expect("ui menu configuration activates at an empty ingress boundary");

    let menu_frame = exact_player_sample(
        &fixture,
        2,
        vec![
            PlayerActionV1 {
                action_id: SchemaId::new(CORE_UI_CONFIRM_ACTION_ID).expect("ui confirm action"),
                phase: PlayerActionPhaseV1::Started,
                value: PlayerActionValueV1::Digital(true),
                semantic_occurrence_ordinal: 0,
            },
            PlayerActionV1 {
                action_id: SchemaId::new(CORE_UI_NAVIGATE_ACTION_ID).expect("ui nav action"),
                phase: PlayerActionPhaseV1::Performed,
                value: PlayerActionValueV1::Vector2Q15([0, 32_767]),
                semantic_occurrence_ordinal: 1,
            },
        ],
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, menu_frame)
        .expect("enqueue menu frame");
    let menu_report = fixture.runtime.run_tick([]).expect("menu tick");
    assert_eq!(
        menu_report.mapping_receipts[0].frame_code,
        InputMappingCodeV1::Accepted
    );
    assert_eq!(
        menu_report.mapping_receipts[0]
            .action_results
            .iter()
            .map(|action| action.mapping_code)
            .collect::<Vec<_>>(),
        vec![InputMappingCodeV1::Accepted, InputMappingCodeV1::Accepted]
    );
    assert!(menu_report.mapping_receipts[0].derived_commands.is_empty());
    assert!(menu_report.command_batches[0].body.envelopes.is_empty());
    assert!(menu_report.events.is_empty());
}
