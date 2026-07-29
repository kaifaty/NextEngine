use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{
    ApplicationSessionId, CloseRequestId, ContentHash, SchemaId, SessionRequestId,
    SessionTransitionId,
};
use crate::platform::PresentationTargetKindV1;

use super::*;

#[test]
fn manifest_round_trip_is_canonical_and_rejects_unknown_fields() {
    let manifest = manifest(CompositionRootV1::Headless, PresentationTargetKindV1::None);
    let bytes = manifest.to_jcs_bytes();
    assert_eq!(
        ApplicationSessionManifestV1::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("canonical manifest"),
        manifest
    );

    let mut with_unknown = bytes;
    with_unknown.splice(1..1, b"\"ambient_path\":\"forbidden\",".iter().copied());
    assert!(
        ApplicationSessionManifestV1::from_jcs_bytes(
            &with_unknown,
            CanonicalDecodeLimits::default()
        )
        .is_err()
    );

    let mut tampered = manifest;
    tampered.body.launch_profile_hash = hash(99);
    assert_eq!(tampered.validate(), Err(SessionContractError::HashMismatch));
}

#[test]
fn root_target_matrix_is_closed() {
    for (root, target) in [
        (
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        ),
        (CompositionRootV1::Game, PresentationTargetKindV1::None),
        (CompositionRootV1::Headless, PresentationTargetKindV1::None),
        (CompositionRootV1::Tools, PresentationTargetKindV1::None),
        (
            CompositionRootV1::Tools,
            PresentationTargetKindV1::Interactive,
        ),
        (
            CompositionRootV1::CaptureWorker,
            PresentationTargetKindV1::DisplaylessOffscreen,
        ),
    ] {
        validate_root_target(root, target).expect("allowed root/target");
    }
    for (root, target) in [
        (
            CompositionRootV1::Headless,
            PresentationTargetKindV1::Interactive,
        ),
        (
            CompositionRootV1::Headless,
            PresentationTargetKindV1::DisplaylessOffscreen,
        ),
        (
            CompositionRootV1::CaptureWorker,
            PresentationTargetKindV1::Interactive,
        ),
    ] {
        assert_eq!(
            validate_root_target(root, target),
            Err(SessionContractError::InvalidRootTarget)
        );
    }
}

#[test]
fn lifecycle_accepts_exactly_the_nine_spec_edges() {
    use ApplicationSessionStatusV1 as S;
    let states = [
        S::Created,
        S::CompositionStaged,
        S::RuntimeStaged,
        S::Active,
        S::Suspended,
        S::Quiescing,
        S::Finalizing,
        S::Closed,
    ];
    let legal = [
        (S::Created, S::CompositionStaged),
        (S::CompositionStaged, S::RuntimeStaged),
        (S::RuntimeStaged, S::Active),
        (S::Active, S::Suspended),
        (S::Suspended, S::Active),
        (S::Active, S::Quiescing),
        (S::Suspended, S::Quiescing),
        (S::Quiescing, S::Finalizing),
        (S::Finalizing, S::Closed),
    ];
    for from in states {
        for to in states {
            assert_eq!(
                from.can_transition_to(to),
                legal.contains(&(from, to)),
                "unexpected edge {from:?} -> {to:?}"
            );
        }
    }
}

#[test]
fn lifecycle_event_advances_one_revision_and_hash_tamper_fails() {
    let manifest = manifest(CompositionRootV1::Headless, PresentationTargetKindV1::None);
    let state = ApplicationSessionStateV1::created(&manifest);
    let request = ApplicationLifecycleRequestV1::new(
        SessionRequestId::from_bytes([1; 16]),
        manifest.body.session_id,
        0,
        ApplicationSessionStatusV1::Created,
        ApplicationSessionStatusV1::CompositionStaged,
        reason(LifecycleReasonKindV1::CompositionReady),
        hash(20),
        causal(CausalInputSourceKindV1::SystemPolicy),
    )
    .expect("legal request");
    let event = ApplicationLifecycleEventV1::committed(
        SessionTransitionId::from_bytes([2; 16]),
        &request,
        Some(hash(21)),
        None,
        None,
    )
    .expect("event");
    let next = event.next_state(&state, None).expect("next state");
    assert_eq!(next.revision, 1);
    assert_eq!(next.state, ApplicationSessionStatusV1::CompositionStaged);
    assert_eq!(
        ApplicationLifecycleRequestV1::new(
            SessionRequestId::from_bytes([3; 16]),
            manifest.body.session_id,
            1,
            ApplicationSessionStatusV1::CompositionStaged,
            ApplicationSessionStatusV1::Closed,
            reason(LifecycleReasonKindV1::HostCloseRequested),
            hash(20),
            causal(CausalInputSourceKindV1::PlatformEvent),
        ),
        Err(SessionContractError::InvalidTransition)
    );
}

#[test]
fn final_save_attempts_are_bounded_and_committed_is_terminal() {
    let request = close_request();
    let archive = close_request_archive_ref(&request.canonical_bytes().expect("archive bytes"));
    let reservation = FinalSaveReservationBodyV1::new(
        request.session_id,
        request.close_request_id,
        request.canonical_close_request_hash,
        archive,
        request.starting_session_revision,
        FinalSavePolicyV1::Always,
        request.shutdown_policy_hash,
        3,
    )
    .expect("reservation");
    let reserved = SessionFinalSaveLedgerEntryV1::reserved(&reservation).expect("ledger entry");
    let retry = reserved
        .retry_pending(SchemaId::new("SESSION_STORAGE_RETRYABLE").expect("code"))
        .expect("retry");
    assert_eq!(retry.attempt_count, 1);
    let receipt = FinalSaveReceiptV1::new(
        request.session_id,
        request.close_request_id,
        request.canonical_close_request_hash,
        reservation.reservation_hash(),
        2,
        7,
        44,
        hash(30),
        hash(31),
        hash(32),
    )
    .expect("receipt");
    let committed = retry.committed(&receipt).expect("committed");
    committed.validate().expect("valid committed entry");
    assert!(
        committed
            .failed(SchemaId::new("SESSION_STORAGE_FATAL").expect("code"))
            .is_err()
    );
}

#[test]
fn shutdown_policies_cover_required_and_last_safe_dispositions() {
    let required = ShutdownPolicyV1::reference_game_default();
    assert_eq!(required.maximum_attempts, 3);
    assert_eq!(
        required.failure_disposition,
        FailureDispositionV1::RequireFinalSave
    );
    required.validate().expect("default policy");

    let fallback = ShutdownPolicyV1::new(2, FailureDispositionV1::AllowLastSafeGeneration)
        .expect("fallback policy");
    fallback.validate().expect("valid fallback");
    assert!(can_close_after_failed_save(
        fallback.failure_disposition,
        Some(hash(40))
    ));
    assert!(!can_close_after_failed_save(
        required.failure_disposition,
        Some(hash(40))
    ));
}

#[test]
fn recovery_link_binds_new_identity_and_rejects_tamper() {
    let mut link = RecoverySessionLinkV1::new(
        ApplicationSessionId::from_bytes([1; 16]),
        hash(1),
        9,
        CloseRequestId::from_bytes([2; 16]),
        hash(2),
        hash(3),
        hash(4),
        hash(5),
        hash(6),
        hash(7),
        ApplicationSessionId::from_bytes([8; 16]),
        SchemaId::new("nextengine.session.recovery.required-save").expect("reason"),
    )
    .expect("link");
    link.validate().expect("valid link");
    link.last_safe_save_manifest_hash = hash(99);
    assert_eq!(link.validate(), Err(SessionContractError::HashMismatch));
}

fn manifest(
    root: CompositionRootV1,
    target: PresentationTargetKindV1,
) -> ApplicationSessionManifestV1 {
    ApplicationSessionManifestV1::new(ApplicationSessionManifestBodyV1 {
        session_id: ApplicationSessionId::from_bytes([7; 16]),
        composition_root: root,
        project_composition_lock_hash: hash(1),
        launch_profile_hash: hash(2),
        platform_capability_set_hash: (target == PresentationTargetKindV1::Interactive)
            .then_some(hash(3)),
        runtime_determinism_profile_hash: hash(4),
        schema_registry_hash: hash(5),
        content_manifest_hash: hash(6),
        recovery_policy_hash: hash(7),
        shutdown_policy_hash: hash(8),
        recovery_session_link_hash: None,
        presentation_target_kind: target,
    })
    .expect("manifest")
}

fn close_request() -> CloseSessionRequestV1 {
    CloseSessionRequestV1::new(
        CloseRequestId::from_bytes([9; 16]),
        ApplicationSessionId::from_bytes([8; 16]),
        5,
        ApplicationSessionStatusV1::Active,
        ShutdownPolicyV1::reference_game_default().canonical_hash,
        FinalSavePolicyV1::Always,
        BoundedDeadlineClassV1::Standard,
        reason(LifecycleReasonKindV1::UserCloseRequested),
        causal(CausalInputSourceKindV1::PlatformEvent),
    )
    .expect("close request")
}

fn reason(kind: LifecycleReasonKindV1) -> LifecycleReasonV1 {
    LifecycleReasonV1 {
        kind,
        reason_code: SchemaId::new("nextengine.session.test").expect("reason"),
    }
}

fn causal(source_kind: CausalInputSourceKindV1) -> CausalInputReferenceV1 {
    CausalInputReferenceV1 {
        source_kind,
        canonical_hash: hash(11),
    }
}

fn hash(byte: u8) -> ContentHash {
    ContentHash::from_bytes([byte; 32])
}
