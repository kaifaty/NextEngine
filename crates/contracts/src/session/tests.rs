use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{
    ApplicationSessionId, CloseRequestId, ContentHash, SchemaId, SessionRequestId,
    SessionTransitionId,
};

use super::*;

fn hash(byte: u8) -> ContentHash {
    ContentHash::from_bytes([byte; 32])
}

fn manifest() -> ApplicationSessionManifestV2 {
    ApplicationSessionManifestV2::new(ApplicationSessionManifestBodyV2 {
        session_id: ApplicationSessionId::from_bytes([1; 16]),
        composition_root: CompositionRootV1::Headless,
        project_composition_lock_hash: hash(2),
        launch_profile_hash: hash(3),
        platform_capability_set_hash: None,
        runtime_determinism_profile_hash: hash(4),
        schema_registry_hash: hash(5),
        content_manifest_hash: hash(6),
        presentation_target_kind: PresentationTargetKindV1::None,
    })
    .expect("manifest")
}

fn cause() -> CausalInputReferenceV1 {
    CausalInputReferenceV1 {
        source_kind: CausalInputSourceKindV1::System,
        canonical_hash: hash(9),
    }
}

fn reason(kind: LifecycleReasonKindV1) -> LifecycleReasonV1 {
    LifecycleReasonV1 {
        kind,
        reason_code: SchemaId::new("nextengine.test.reason").expect("id"),
    }
}

#[test]
fn manifest_v2_round_trips_and_v1_is_unsupported() {
    let value = manifest();
    let bytes = value.to_jcs_bytes();
    assert_eq!(
        ApplicationSessionManifestV2::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("decode"),
        value
    );
    let legacy = String::from_utf8(bytes).expect("utf8").replace(
        "nextengine.application-session-manifest.v2",
        "nextengine.application-session-manifest.v1",
    );
    assert!(matches!(
        ApplicationSessionManifestV2::from_jcs_bytes(
            legacy.as_bytes(),
            CanonicalDecodeLimits::default()
        ),
        Err(SessionContractError::UnsupportedVersion)
    ));
}

#[test]
fn lifecycle_keeps_exact_eight_state_graph() {
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
    assert_eq!(states.len(), 8);
    assert!(S::Created.can_transition_to(S::CompositionStaged));
    assert!(S::Active.can_transition_to(S::Suspended));
    assert!(S::Suspended.can_transition_to(S::Active));
    assert!(S::Finalizing.can_transition_to(S::Closed));
    assert!(!S::Suspended.can_transition_to(S::Closed));
}

#[test]
fn request_event_and_state_round_trip_without_policy_hash() {
    let manifest = manifest();
    let state = ApplicationSessionStateV2::created(&manifest);
    let request = ApplicationLifecycleRequestV2::new(
        SessionRequestId::from_bytes([7; 16]),
        state.session_id,
        0,
        ApplicationSessionStatusV1::Created,
        ApplicationSessionStatusV1::CompositionStaged,
        reason(LifecycleReasonKindV1::CompositionReady),
        cause(),
    )
    .expect("request");
    let decoded = ApplicationLifecycleRequestV2::from_jcs_bytes(
        &request.canonical_bytes(),
        CanonicalDecodeLimits::default(),
    )
    .expect("request decode");
    assert_eq!(decoded, request);
    let event = ApplicationLifecycleEventV2::committed(
        SessionTransitionId::from_bytes([8; 16]),
        &request,
        None,
        None,
        None,
    )
    .expect("event");
    let next = event.next_state(&state, None).expect("next");
    assert_eq!(next.state, ApplicationSessionStatusV1::CompositionStaged);
    assert_eq!(
        ApplicationSessionStateV2::from_jcs_bytes(
            &next.to_jcs_bytes(),
            CanonicalDecodeLimits::default()
        )
        .expect("state decode"),
        next
    );
}

#[test]
fn close_journal_has_only_prepared_and_save_published() {
    let request = CloseSessionRequestV2::new(
        CloseRequestId::from_bytes([10; 16]),
        manifest().body.session_id,
        3,
        ApplicationSessionStatusV1::Active,
        reason(LifecycleReasonKindV1::UserCloseRequested),
        cause(),
    )
    .expect("close request");
    let prepared = CloseSessionJournalV2::prepared(&request, hash(11));
    assert_eq!(prepared.stage, CloseSessionJournalStageV2::Prepared);
    assert!(prepared.save_generation_hash.is_none());
    let published = prepared.save_published(hash(12)).expect("published");
    assert_eq!(published.stage, CloseSessionJournalStageV2::SavePublished);
    assert_eq!(
        CloseSessionJournalV2::from_canonical_bytes(
            &published.canonical_bytes(),
            CanonicalDecodeLimits::default()
        )
        .expect("decode"),
        published
    );
    let receipt =
        CloseSessionReceiptV2::new(request.close_request_id, request.session_id, hash(12));
    assert_eq!(
        CloseSessionReceiptV2::from_canonical_bytes(
            &receipt.canonical_bytes(),
            CanonicalDecodeLimits::default()
        )
        .expect("decode"),
        receipt
    );
}
