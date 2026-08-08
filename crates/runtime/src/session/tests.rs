use next_contracts::ids::{ApplicationSessionId, ContentHash, SchemaId, SessionRequestId};
use next_contracts::session::{
    ApplicationLifecycleRequestV2, ApplicationSessionManifestBodyV2, ApplicationSessionManifestV2,
    ApplicationSessionStatusV1, CausalInputReferenceV1, CausalInputSourceKindV1, CompositionRootV1,
    LifecycleReasonKindV1, LifecycleReasonV1, PresentationTargetKindV1,
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

fn request(
    machine: &ApplicationSessionMachine,
    target: ApplicationSessionStatusV1,
) -> ApplicationLifecycleRequestV2 {
    ApplicationLifecycleRequestV2::new(
        SessionRequestId::from_bytes([machine.state().revision as u8 + 10; 16]),
        machine.state().session_id,
        machine.state().revision,
        machine.state().state,
        target,
        LifecycleReasonV1 {
            kind: LifecycleReasonKindV1::RuntimeReady,
            reason_code: SchemaId::new("nextengine.test.transition").expect("id"),
        },
        CausalInputReferenceV1 {
            source_kind: CausalInputSourceKindV1::System,
            canonical_hash: hash(20),
        },
    )
    .expect("request")
}

#[test]
fn machine_retains_only_last_lifecycle_record() {
    let mut machine = ApplicationSessionMachine::new(manifest()).expect("machine");
    for target in [
        ApplicationSessionStatusV1::CompositionStaged,
        ApplicationSessionStatusV1::RuntimeStaged,
        ApplicationSessionStatusV1::Active,
    ] {
        let plan = machine
            .plan_transition(
                request(&machine, target),
                SessionTransitionReferencesV1::default(),
            )
            .expect("plan");
        machine.commit(plan);
    }
    let last = machine.last_transition().cloned().expect("last");
    assert_eq!(last.event.to_state, ApplicationSessionStatusV1::Active);
    let restored = ApplicationSessionMachine::restore(
        machine.manifest().clone(),
        machine.state().clone(),
        Some(last),
    )
    .expect("restore");
    assert_eq!(restored.state(), machine.state());
}

#[test]
fn exact_retry_is_limited_to_last_request() {
    let machine = ApplicationSessionMachine::new(manifest()).expect("machine");
    let request = request(&machine, ApplicationSessionStatusV1::CompositionStaged);
    let plan = machine
        .plan_transition(request.clone(), SessionTransitionReferencesV1::default())
        .expect("plan");
    let mut committed = machine;
    let expected = committed.commit(plan);
    assert!(
        matches!(committed.plan_transition(request, SessionTransitionReferencesV1::default()).expect("retry"), SessionTransitionPlanV1::ExactRetry { event, .. } if event == expected)
    );
}
