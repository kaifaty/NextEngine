use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::SchemaId;
use next_contracts::session::{
    ApplicationSessionStatusV1, BoundedDeadlineClassV1, CloseSessionRequestV1,
    CloseSessionResultV1, CompositionRootV1, FailureDispositionV1, PresentationTargetKindV1,
    ShutdownPolicyV1,
};
use next_project::cook_project_v1;
use next_reference_game::project_source_v2;

use crate::{
    ApplicationCloseOutcomeV1, CloseExecutionOptionsV1, FinalSaveAttemptFailureV1, LaunchRequestV1,
    ProjectSelectionV1,
};

use super::ApplicationCoordinator;

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn production_run_closes_once_and_restart_returns_the_same_receipt() {
    let root = test_root("close-restart");
    let launch = headless_launch(&root);
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    let run = application
        .run_reference_game(true)
        .expect("reference game");
    assert_eq!(run.ticks, 16);
    assert_eq!(run.events, 17);
    let closed = application
        .close(CloseExecutionOptionsV1::default())
        .expect("close");
    let receipt_hash = closed.receipt_hash().expect("terminal receipt");
    assert_eq!(
        application.state().state,
        ApplicationSessionStatusV1::Closed
    );

    drop(application);
    let mut resumed = ApplicationCoordinator::resume(launch).expect("resume closed");
    let retried = resumed
        .close(CloseExecutionOptionsV1::default())
        .expect("exact terminal retry");
    assert_eq!(retried.receipt_hash(), Some(receipt_hash));
    cleanup(root);
}

#[test]
fn same_close_id_with_different_canonical_bytes_is_rejected() {
    let root = test_root("close-collision");
    let mut application = ApplicationCoordinator::launch(headless_launch(&root)).expect("launch");
    application
        .run_reference_game(true)
        .expect("reference game");
    let request = application
        .close_request(BoundedDeadlineClassV1::Standard)
        .expect("request");
    let collision = CloseSessionRequestV1::new(
        request.close_request_id,
        request.session_id,
        request.starting_session_revision,
        request.starting_session_state,
        request.shutdown_policy_hash,
        request.final_save_policy,
        BoundedDeadlineClassV1::Immediate,
        request.reason.clone(),
        request.causal_input_reference.clone(),
    )
    .expect("colliding request");
    application
        .close_with_request(
            request,
            CloseExecutionOptionsV1::with_failure(
                FinalSaveAttemptFailureV1::Retryable(
                    SchemaId::new("nextengine.test.retry").expect("code"),
                ),
                None,
            ),
        )
        .expect("first attempt");
    let error = application
        .close_with_request(collision, CloseExecutionOptionsV1::default())
        .expect_err("collision rejected");
    assert_eq!(
        error.diagnostic_code(),
        "SESSION_FINAL_SAVE_IDENTITY_COLLISION"
    );
    cleanup(root);
}

#[test]
fn required_save_exhaustion_stays_finalizing_and_can_recover_from_last_safe() {
    let root = test_root("required-recovery");
    let launch = headless_launch(&root);
    let mut prior = ApplicationCoordinator::launch(launch.clone()).expect("prior launch");
    prior.run_reference_game(true).expect("prior run");
    prior
        .close(CloseExecutionOptionsV1::default())
        .expect("prior close");
    drop(prior);

    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    application.run_reference_game(true).expect("run");
    for attempt in 0..3 {
        let outcome = application
            .close(CloseExecutionOptionsV1::with_failure(
                FinalSaveAttemptFailureV1::Retryable(
                    SchemaId::new("nextengine.test.save-unavailable").expect("code"),
                ),
                None,
            ))
            .expect("failure is durable progress");
        assert!(matches!(outcome, ApplicationCloseOutcomeV1::Progress(_)));
        assert_eq!(
            application.state().state,
            ApplicationSessionStatusV1::Finalizing
        );
        if attempt < 2 {
            assert!(application.state().terminal_receipt_hash.is_none());
        }
    }
    let failed_session_id = application.state().session_id;
    drop(application);

    let resumed = ApplicationCoordinator::resume(launch).expect("resume failed close");
    assert_eq!(
        resumed.state().state,
        ApplicationSessionStatusV1::Finalizing
    );
    let recovered = resumed.recover_required_save_failure().expect("recovery");
    assert_eq!(recovered.state().state, ApplicationSessionStatusV1::Active);
    assert_ne!(recovered.state().session_id, failed_session_id);
    assert!(recovered.state().active_save_generation_hash.is_some());
    cleanup(root);
}

#[test]
fn allow_last_safe_policy_closes_after_terminal_save_failure() {
    let root = test_root("last-safe");
    let project_root = root.join("published-project");
    let mut source = project_source_v2().expect("source");
    source.shutdown_policy =
        ShutdownPolicyV1::new(1, FailureDispositionV1::AllowLastSafeGeneration).expect("policy");
    let cooked = cook_project_v1(source).expect("cook");
    ContentStore::new(&project_root)
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let launch = LaunchRequestV1 {
        project: ProjectSelectionV1::PublishedStateRoot(project_root),
        expected_project_lock: Some(cooked.composition_lock.composition_lock_sha256),
        state_root: root.clone(),
        composition_root: CompositionRootV1::Headless,
        presentation_target: PresentationTargetKindV1::None,
    };
    let mut prior = ApplicationCoordinator::launch(launch.clone()).expect("prior launch");
    prior.run_reference_game(true).expect("prior run");
    let prior_close = prior
        .close(CloseExecutionOptionsV1::default())
        .expect("prior close");
    let last_safe = match prior_close {
        ApplicationCloseOutcomeV1::Closed {
            save_generation_hash: Some(hash),
            ..
        } => hash,
        other => panic!("unexpected prior close: {other:?}"),
    };
    drop(prior);

    let mut application = ApplicationCoordinator::launch(launch).expect("launch");
    application.run_reference_game(true).expect("run");
    let outcome = application
        .close(CloseExecutionOptionsV1::with_failure(
            FinalSaveAttemptFailureV1::Terminal(
                SchemaId::new("nextengine.test.disk-terminal").expect("code"),
            ),
            Some(last_safe),
        ))
        .expect("last-safe close");
    assert!(matches!(
        outcome,
        ApplicationCloseOutcomeV1::Closed {
            result: CloseSessionResultV1::ClosedUsingLastSafeGeneration,
            save_generation_hash: Some(hash),
            ..
        } if hash == last_safe
    ));
    cleanup(root);
}

#[test]
fn crash_after_save_publication_does_not_create_a_second_save_generation() {
    let root = test_root("save-publication-crash");
    let launch = headless_launch(&root);
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    application.run_reference_game(true).expect("run");
    application.inject_pause_after_save_commit();
    assert!(
        application
            .close(CloseExecutionOptionsV1::default())
            .is_err()
    );
    drop(application);

    let mut resumed = ApplicationCoordinator::resume(launch.clone()).expect("resume");
    let closed = resumed
        .close(CloseExecutionOptionsV1::default())
        .expect("resume close");
    let first_hash = match closed {
        ApplicationCloseOutcomeV1::Closed {
            save_generation_hash: Some(hash),
            ..
        } => hash,
        other => panic!("unexpected close: {other:?}"),
    };
    drop(resumed);
    let mut terminal = ApplicationCoordinator::resume(launch).expect("terminal resume");
    let retried = terminal
        .close(CloseExecutionOptionsV1::default())
        .expect("terminal retry");
    assert!(matches!(
        retried,
        ApplicationCloseOutcomeV1::Closed {
            save_generation_hash: Some(hash),
            ..
        } if hash == first_hash
    ));
    cleanup(root);
}

#[test]
fn game_and_headless_share_authoritative_hashes_and_headless_has_no_presentation() {
    let root = test_root("root-parity");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        root.join("game"),
        CompositionRootV1::Game,
        PresentationTargetKindV1::None,
    ))
    .expect("game launch");
    let game_run = game.run_reference_game(true).expect("game run");
    game.close(CloseExecutionOptionsV1::default())
        .expect("game close");

    let mut headless = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        root.join("headless"),
        CompositionRootV1::Headless,
        PresentationTargetKindV1::None,
    ))
    .expect("headless launch");
    let headless_run = headless.run_reference_game(true).expect("headless run");
    headless
        .close(CloseExecutionOptionsV1::default())
        .expect("headless close");

    assert_eq!(
        game_run.authoritative_state_root,
        headless_run.authoritative_state_root
    );
    assert_eq!(
        game_run.command_archive_root,
        headless_run.command_archive_root
    );
    assert_eq!(
        game_run.command_identity_index_root,
        headless_run.command_identity_index_root
    );
    assert!(headless_run.presentation_snapshot.is_none());
    cleanup(root);
}

fn headless_launch(root: &std::path::Path) -> LaunchRequestV1 {
    LaunchRequestV1::reference(
        root,
        CompositionRootV1::Headless,
        PresentationTargetKindV1::None,
    )
}

fn test_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "nextengine-application-{label}-{}-{}",
        std::process::id(),
        TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}

fn cleanup(root: std::path::PathBuf) {
    if root.exists() {
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
