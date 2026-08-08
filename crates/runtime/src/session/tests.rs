use next_contracts::ids::{ApplicationSessionId, ContentHash, SchemaId, SessionRequestId};
use next_contracts::platform::PresentationTargetKindV1;
use next_contracts::session::{
    ApplicationLifecycleRequestV1, ApplicationSessionManifestBodyV1, ApplicationSessionManifestV1,
    ApplicationSessionStatusV1, CausalInputReferenceV1, CausalInputSourceKindV1, CompositionRootV1,
    LifecycleReasonKindV1, LifecycleReasonV1,
};

use super::*;

#[test]
fn exact_retry_returns_event_before_historical_revision_validation() {
    let mut machine = ApplicationSessionMachine::new(manifest()).expect("machine");
    let request = request(
        1,
        0,
        ApplicationSessionStatusV1::Created,
        ApplicationSessionStatusV1::CompositionStaged,
    );
    let plan = machine
        .plan_transition(request.clone(), SessionTransitionReferencesV1::default())
        .expect("plan");
    let event = machine.commit(plan);
    let retry = machine
        .plan_transition(request, SessionTransitionReferencesV1::default())
        .expect("retry");
    assert!(matches!(
        retry,
        SessionTransitionPlanV1::ExactRetry {
            event: retried,
            current_state
        } if retried == event && current_state.revision == 1
    ));
}

#[test]
fn same_id_with_different_bytes_collides_without_mutation() {
    let mut machine = ApplicationSessionMachine::new(manifest()).expect("machine");
    let first = request(
        1,
        0,
        ApplicationSessionStatusV1::Created,
        ApplicationSessionStatusV1::CompositionStaged,
    );
    let plan = machine
        .plan_transition(first.clone(), SessionTransitionReferencesV1::default())
        .expect("plan");
    let event = machine.commit(plan);
    let mut restored = ApplicationSessionMachine::restore(
        machine.manifest().clone(),
        machine.state().clone(),
        vec![ArchivedLifecycleRequestV1 {
            request_id: first.request_id,
            canonical_request_hash: first.canonical_hash,
            canonical_request_bytes: first.canonical_bytes(),
            event,
        }],
    )
    .expect("restored");
    let collision = request(
        1,
        0,
        ApplicationSessionStatusV1::Created,
        ApplicationSessionStatusV1::CompositionStaged,
    );
    let mut collision = collision;
    collision.policy_hash = hash(99);
    collision.canonical_hash = ApplicationLifecycleRequestV1::new(
        collision.request_id,
        collision.session_id,
        collision.expected_revision,
        collision.expected_state,
        collision.requested_state,
        collision.reason.clone(),
        collision.policy_hash,
        collision.causal_input_reference.clone(),
    )
    .expect("rehashed")
    .canonical_hash;
    assert_eq!(
        restored.plan_transition(collision, SessionTransitionReferencesV1::default()),
        Err(SessionMachineError::RequestIdentityCollision)
    );
    assert_eq!(restored.state().revision, 1);
    let _ = &mut restored;
}

#[test]
fn stale_and_illegal_requests_do_not_change_state() {
    let machine = ApplicationSessionMachine::new(manifest()).expect("machine");
    let stale = request(
        2,
        7,
        ApplicationSessionStatusV1::Created,
        ApplicationSessionStatusV1::CompositionStaged,
    );
    assert!(matches!(
        machine.plan_transition(stale, SessionTransitionReferencesV1::default()),
        Err(SessionMachineError::ExpectedRevisionStale { .. })
    ));
    assert_eq!(machine.state().revision, 0);
}

#[test]
fn runtime_observation_is_published_before_in_memory_commit_and_cannot_regress() {
    let mut machine = ApplicationSessionMachine::new(manifest()).expect("machine");
    let plan = machine
        .plan_state_publication(Some(12), Some(hash(44)))
        .expect("observation plan");
    assert_eq!(machine.state().active_runtime_revision, None);
    assert_eq!(plan.next_state.active_runtime_revision, Some(12));
    machine.commit_state_publication(plan);
    assert_eq!(machine.state().active_runtime_revision, Some(12));
    assert_eq!(machine.state().active_save_generation_hash, Some(hash(44)));
    assert_eq!(
        machine.plan_state_publication(Some(11), None),
        Err(SessionMachineError::ObservationRegression)
    );
}

#[test]
fn verified_save_load_can_replace_an_observation_only_while_suspended() {
    let mut machine = ApplicationSessionMachine::new(manifest()).expect("machine");
    for (id, from, to) in [
        (
            1,
            ApplicationSessionStatusV1::Created,
            ApplicationSessionStatusV1::CompositionStaged,
        ),
        (
            2,
            ApplicationSessionStatusV1::CompositionStaged,
            ApplicationSessionStatusV1::RuntimeStaged,
        ),
        (
            3,
            ApplicationSessionStatusV1::RuntimeStaged,
            ApplicationSessionStatusV1::Active,
        ),
    ] {
        let revision = machine.state().revision;
        advance(&mut machine, request(id, revision, from, to));
    }
    let observed = machine
        .plan_state_publication(Some(12), None)
        .expect("later observation");
    machine.commit_state_publication(observed);
    assert_eq!(
        machine.plan_save_load_publication(3, hash(55)),
        Err(SessionMachineError::ObservationRegression)
    );

    let revision = machine.state().revision;
    advance(
        &mut machine,
        request(
            4,
            revision,
            ApplicationSessionStatusV1::Active,
            ApplicationSessionStatusV1::Suspended,
        ),
    );
    let lifecycle_revision = machine.state().revision;
    let loaded = machine
        .plan_save_load_publication(3, hash(55))
        .expect("verified suspended save load");
    assert_eq!(loaded.next_state.active_runtime_revision, Some(3));
    assert_eq!(
        loaded.next_state.active_save_generation_hash,
        Some(hash(55))
    );
    assert_eq!(loaded.next_state.revision, lifecycle_revision);
    machine.commit_state_publication(loaded);
    assert_eq!(machine.state().active_runtime_revision, Some(3));
    assert_eq!(machine.state().active_save_generation_hash, Some(hash(55)));
}

#[test]
fn restore_requires_empty_history_exactly_for_created_revision_zero() {
    let created = ApplicationSessionMachine::new(manifest()).expect("created machine");
    ApplicationSessionMachine::restore(
        created.manifest().clone(),
        created.state().clone(),
        Vec::new(),
    )
    .expect("created revision zero has an empty complete history");

    let mut advanced = ApplicationSessionMachine::new(manifest()).expect("advanced machine");
    advance(
        &mut advanced,
        request(
            1,
            0,
            ApplicationSessionStatusV1::Created,
            ApplicationSessionStatusV1::CompositionStaged,
        ),
    );
    assert_eq!(
        ApplicationSessionMachine::restore(
            advanced.manifest().clone(),
            advanced.state().clone(),
            Vec::new(),
        )
        .expect_err("nonzero revision cannot restore without its history"),
        SessionMachineError::ArchiveInvalid
    );
}

#[test]
fn restore_rejects_a_valid_suffix_instead_of_a_complete_lifecycle_chain() {
    let mut machine = ApplicationSessionMachine::new(manifest()).expect("machine");
    advance(
        &mut machine,
        request(
            1,
            0,
            ApplicationSessionStatusV1::Created,
            ApplicationSessionStatusV1::CompositionStaged,
        ),
    );
    advance(
        &mut machine,
        request(
            2,
            1,
            ApplicationSessionStatusV1::CompositionStaged,
            ApplicationSessionStatusV1::RuntimeStaged,
        ),
    );
    let mut suffix = machine.archived_requests();
    suffix.retain(|archived| archived.event.before_revision != 0);
    assert_eq!(
        ApplicationSessionMachine::restore(
            machine.manifest().clone(),
            machine.state().clone(),
            suffix,
        )
        .expect_err("a valid suffix is not the complete chain"),
        SessionMachineError::ArchiveInvalid
    );
}

#[test]
fn restore_rejects_duplicate_revision_events_even_when_archive_count_matches() {
    let mut machine = ApplicationSessionMachine::new(manifest()).expect("machine");
    advance(
        &mut machine,
        request(
            1,
            0,
            ApplicationSessionStatusV1::Created,
            ApplicationSessionStatusV1::CompositionStaged,
        ),
    );
    advance(
        &mut machine,
        request(
            2,
            1,
            ApplicationSessionStatusV1::CompositionStaged,
            ApplicationSessionStatusV1::RuntimeStaged,
        ),
    );

    let mut alternate = ApplicationSessionMachine::new(manifest()).expect("alternate machine");
    advance(
        &mut alternate,
        request(
            3,
            0,
            ApplicationSessionStatusV1::Created,
            ApplicationSessionStatusV1::CompositionStaged,
        ),
    );
    let alternate_first = alternate
        .archived_requests()
        .into_iter()
        .next()
        .expect("alternate first event");
    let mut duplicate_revision = machine.archived_requests();
    duplicate_revision[1] = alternate_first;

    assert_eq!(
        ApplicationSessionMachine::restore(
            machine.manifest().clone(),
            machine.state().clone(),
            duplicate_revision,
        )
        .expect_err("two revision-zero events do not form a chain"),
        SessionMachineError::ArchiveInvalid
    );
}

fn advance(
    machine: &mut ApplicationSessionMachine,
    request: ApplicationLifecycleRequestV1,
) -> ApplicationLifecycleEventV1 {
    let plan = machine
        .plan_transition(request, SessionTransitionReferencesV1::default())
        .expect("transition plan");
    machine.commit(plan)
}

fn manifest() -> ApplicationSessionManifestV1 {
    ApplicationSessionManifestV1::new(ApplicationSessionManifestBodyV1 {
        session_id: ApplicationSessionId::from_bytes([1; 16]),
        composition_root: CompositionRootV1::Headless,
        project_composition_lock_hash: hash(1),
        launch_profile_hash: hash(2),
        platform_capability_set_hash: None,
        runtime_determinism_profile_hash: hash(3),
        schema_registry_hash: hash(4),
        content_manifest_hash: hash(5),
        recovery_policy_hash: hash(6),
        shutdown_policy_hash: hash(7),
        recovery_session_link_hash: None,
        presentation_target_kind: PresentationTargetKindV1::None,
    })
    .expect("manifest")
}

fn request(
    id: u8,
    revision: u64,
    from: ApplicationSessionStatusV1,
    to: ApplicationSessionStatusV1,
) -> ApplicationLifecycleRequestV1 {
    ApplicationLifecycleRequestV1::new(
        SessionRequestId::from_bytes([id; 16]),
        ApplicationSessionId::from_bytes([1; 16]),
        revision,
        from,
        to,
        LifecycleReasonV1 {
            kind: LifecycleReasonKindV1::CompositionReady,
            reason_code: SchemaId::new("nextengine.session.test").expect("reason"),
        },
        hash(8),
        CausalInputReferenceV1 {
            source_kind: CausalInputSourceKindV1::SystemPolicy,
            canonical_hash: hash(9),
        },
    )
    .expect("request")
}

fn hash(byte: u8) -> ContentHash {
    ContentHash::from_bytes([byte; 32])
}
