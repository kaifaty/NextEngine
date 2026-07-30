use crate::canonical::{
    CANONICAL_TYPE_U16, CanonicalDecodeLimits, DecodedCanonicalSegment, decode_canonical_segment,
    encode_canonical_segment,
};
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
fn lifecycle_request_and_event_round_trip_with_bounded_exact_decoders() {
    let manifest = manifest(CompositionRootV1::Headless, PresentationTargetKindV1::None);
    let request = ApplicationLifecycleRequestV1::new(
        SessionRequestId::from_bytes([0x31; 16]),
        manifest.body.session_id,
        3,
        ApplicationSessionStatusV1::Active,
        ApplicationSessionStatusV1::Suspended,
        reason(LifecycleReasonKindV1::SuspendRequested),
        hash(32),
        causal(CausalInputSourceKindV1::PlatformEvent),
    )
    .expect("lifecycle request");
    let request_bytes = request.canonical_bytes();
    let decoded_request = ApplicationLifecycleRequestV1::from_jcs_bytes(
        &request_bytes,
        CanonicalDecodeLimits::default(),
    )
    .expect("decode lifecycle request");
    assert_eq!(decoded_request, request);

    let event = ApplicationLifecycleEventV1::committed(
        SessionTransitionId::from_bytes([0x33; 16]),
        &request,
        None,
        Some(hash(34)),
        Some(hash(35)),
    )
    .expect("lifecycle event");
    let event_bytes = event.canonical_bytes();
    assert_eq!(
        ApplicationLifecycleEventV1::from_jcs_bytes(
            &event_bytes,
            &request,
            CanonicalDecodeLimits::default(),
        )
        .expect("decode lifecycle event"),
        event
    );

    let limits = CanonicalDecodeLimits {
        max_total_bytes: event_bytes.len() - 1,
        ..CanonicalDecodeLimits::default()
    };
    assert!(ApplicationLifecycleEventV1::from_jcs_bytes(&event_bytes, &request, limits).is_err());
}

#[test]
fn lifecycle_decoders_reject_unknown_schema_fields_and_request_event_mismatch() {
    let manifest = manifest(CompositionRootV1::Headless, PresentationTargetKindV1::None);
    let request = ApplicationLifecycleRequestV1::new(
        SessionRequestId::from_bytes([0x41; 16]),
        manifest.body.session_id,
        3,
        ApplicationSessionStatusV1::Active,
        ApplicationSessionStatusV1::Suspended,
        reason(LifecycleReasonKindV1::SuspendRequested),
        hash(42),
        causal(CausalInputSourceKindV1::PlatformEvent),
    )
    .expect("lifecycle request");
    let bytes = request.canonical_bytes();
    let unknown_schema = String::from_utf8(bytes.clone())
        .expect("request is utf8")
        .replace("\"schema_version\":1", "\"schema_version\":99")
        .into_bytes();
    assert_eq!(
        ApplicationLifecycleRequestV1::from_jcs_bytes(
            &unknown_schema,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::UnsupportedVersion)
    );

    let mut unknown_field = bytes;
    unknown_field.splice(1..1, b"\"ambient_path\":\"forbidden\",".iter().copied());
    assert!(
        ApplicationLifecycleRequestV1::from_jcs_bytes(
            &unknown_field,
            CanonicalDecodeLimits::default(),
        )
        .is_err()
    );

    let event = ApplicationLifecycleEventV1::committed(
        SessionTransitionId::from_bytes([0x43; 16]),
        &request,
        None,
        None,
        None,
    )
    .expect("lifecycle event");
    let collision = ApplicationLifecycleRequestV1::new(
        SessionRequestId::from_bytes([0x44; 16]),
        request.session_id,
        request.expected_revision,
        request.expected_state,
        request.requested_state,
        request.reason.clone(),
        request.policy_hash,
        request.causal_input_reference.clone(),
    )
    .expect("colliding request");
    assert_eq!(
        ApplicationLifecycleEventV1::from_jcs_bytes(
            &event.canonical_bytes(),
            &collision,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::InvalidTransition)
    );
}

#[test]
fn close_request_canonical_round_trip_is_bounded() {
    let request = close_request();
    let bytes = request.canonical_bytes().expect("close request bytes");
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default(),)
            .expect("canonical close request"),
        request,
    );

    let limits = CanonicalDecodeLimits {
        max_total_bytes: bytes.len() - 1,
        ..CanonicalDecodeLimits::default()
    };
    assert!(matches!(
        CloseSessionRequestV1::from_canonical_bytes(&bytes, limits),
        Err(SessionContractError::CanonicalDecode(_)),
    ));
}

#[test]
fn close_request_decoder_rejects_malformed_envelope_and_field_set() {
    let mut truncated = close_request()
        .canonical_bytes()
        .expect("close request bytes");
    truncated.pop();
    assert!(matches!(
        CloseSessionRequestV1::from_canonical_bytes(&truncated, CanonicalDecodeLimits::default(),),
        Err(SessionContractError::CanonicalDecode(_)),
    ));

    let wrong_owner = mutate_close_request_bytes(|segment| {
        segment.owner_id = "nextengine.tools".to_owned();
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(&wrong_owner, CanonicalDecodeLimits::default(),),
        Err(SessionContractError::InvalidStateFields),
    );

    let unknown_schema = mutate_close_request_bytes(|segment| {
        segment.schema_id = "nextengine.close-session-request.v2".to_owned();
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(
            &unknown_schema,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::UnsupportedVersion),
    );

    let wrong_segment_id = mutate_close_request_bytes(|segment| {
        segment.segment_id = "00000000000000000000000000000000".to_owned();
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(
            &wrong_segment_id,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::IdentityMismatch),
    );

    let unknown_field = mutate_close_request_bytes(|segment| {
        field_mut(segment, 11).field_id = 12;
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(
            &unknown_field,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::InvalidStateFields),
    );

    let missing_field = mutate_close_request_bytes(|segment| {
        segment.fields.retain(|field| field.field_id != 11);
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(
            &missing_field,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::InvalidStateFields),
    );

    let wrong_type = mutate_close_request_bytes(|segment| {
        field_mut(segment, 7).type_tag = CANONICAL_TYPE_U16;
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(&wrong_type, CanonicalDecodeLimits::default(),),
        Err(SessionContractError::InvalidStateFields),
    );

    let wrong_length = mutate_close_request_bytes(|segment| {
        field_mut(segment, 2).payload.pop();
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(
            &wrong_length,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::InvalidStateFields),
    );
}

#[test]
fn close_request_decoder_rejects_unknown_closed_values_and_invalid_payloads() {
    for (field_id, unknown_value) in [(5, 0_u8), (7, 2), (8, 4)] {
        let bytes = mutate_close_request_bytes(|segment| {
            field_mut(segment, field_id).payload[0] = unknown_value;
        });
        assert_eq!(
            CloseSessionRequestV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default(),),
            Err(SessionContractError::UnknownClosedValue),
        );
    }

    let invalid_close_state = mutate_close_request_bytes(|segment| {
        field_mut(segment, 5).payload[0] = ApplicationSessionStatusV1::Created as u8;
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(
            &invalid_close_state,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::InvalidCloseState),
    );

    let unknown_reason = mutate_close_request_bytes(|segment| {
        field_mut(segment, 9).payload[0] = 11;
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(
            &unknown_reason,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::UnknownClosedValue),
    );

    let invalid_reason_utf8 = mutate_close_request_bytes(|segment| {
        field_mut(segment, 9).payload = vec![LifecycleReasonKindV1::UserCloseRequested as u8, 0xff];
    });
    assert!(matches!(
        CloseSessionRequestV1::from_canonical_bytes(
            &invalid_reason_utf8,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::CanonicalDecode(_)),
    ));

    let empty_reason_code = mutate_close_request_bytes(|segment| {
        field_mut(segment, 9).payload = vec![LifecycleReasonKindV1::UserCloseRequested as u8];
    });
    assert!(matches!(
        CloseSessionRequestV1::from_canonical_bytes(
            &empty_reason_code,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::Identifier(_)),
    ));

    let unknown_cause = mutate_close_request_bytes(|segment| {
        field_mut(segment, 10).payload[0] = 6;
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(
            &unknown_cause,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::UnknownClosedValue),
    );

    let short_cause = mutate_close_request_bytes(|segment| {
        field_mut(segment, 10).payload.pop();
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(&short_cause, CanonicalDecodeLimits::default(),),
        Err(SessionContractError::InvalidStateFields),
    );
}

#[test]
fn close_request_decoder_rejects_version_and_embedded_hash_tamper() {
    let unsupported_version = mutate_close_request_bytes(|segment| {
        field_mut(segment, 1).payload = 2_u32.to_le_bytes().to_vec();
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(
            &unsupported_version,
            CanonicalDecodeLimits::default(),
        ),
        Err(SessionContractError::UnsupportedVersion),
    );

    let hash_tamper = mutate_close_request_bytes(|segment| {
        field_mut(segment, 11).payload[0] ^= 0xff;
    });
    assert_eq!(
        CloseSessionRequestV1::from_canonical_bytes(&hash_tamper, CanonicalDecodeLimits::default(),),
        Err(SessionContractError::HashMismatch),
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
    let bytes = link.to_jcs_bytes();
    assert_eq!(
        RecoverySessionLinkV1::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("recovery link round trip"),
        link
    );
    let limits = CanonicalDecodeLimits {
        max_total_bytes: bytes.len() - 1,
        ..CanonicalDecodeLimits::default()
    };
    assert!(RecoverySessionLinkV1::from_jcs_bytes(&bytes, limits).is_err());
    let unknown_schema = String::from_utf8(bytes)
        .expect("link is utf8")
        .replace("\"schema_version\":1", "\"schema_version\":99")
        .into_bytes();
    assert_eq!(
        RecoverySessionLinkV1::from_jcs_bytes(&unknown_schema, CanonicalDecodeLimits::default()),
        Err(SessionContractError::UnsupportedVersion)
    );
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

fn mutate_close_request_bytes(mutate: impl FnOnce(&mut DecodedCanonicalSegment)) -> Vec<u8> {
    let bytes = close_request()
        .canonical_bytes()
        .expect("close request bytes");
    let mut segment =
        decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("segment");
    mutate(&mut segment);
    encode_canonical_segment(
        &segment.owner_id,
        &segment.schema_id,
        &segment.segment_id,
        segment.fields,
    )
    .expect("mutated canonical segment")
}

fn field_mut(
    segment: &mut DecodedCanonicalSegment,
    field_id: u32,
) -> &mut crate::canonical::CanonicalField {
    segment
        .fields
        .iter_mut()
        .find(|field| field.field_id == field_id)
        .expect("fixture field")
}
