use std::sync::atomic::{AtomicU64, Ordering};

use next_contracts::session::{
    ApplicationSessionStatusV1, CompositionRootV1, LifecycleReasonKindV1, PresentationTargetKindV1,
};
use next_runtime::SessionTransitionReferencesV1;

use crate::{ApplicationCloseOutcomeV2, LaunchRequestV1};

use super::ApplicationCoordinator;

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn state_root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "next-application-{name}-{}-{}",
        std::process::id(),
        TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}

fn launch(root: &std::path::Path) -> ApplicationCoordinator {
    ApplicationCoordinator::launch(LaunchRequestV1::reference(
        root.to_path_buf(),
        CompositionRootV1::Headless,
        PresentationTargetKindV1::None,
    ))
    .expect("launch")
}

#[test]
fn manual_save_load_and_close_keep_the_r2_world() {
    let root = state_root("save-load-close");
    let _ = std::fs::remove_dir_all(&root);
    let mut game = launch(&root);
    let before = game.begin_reference_game_live(true).expect("run");
    let save_hash = game.save_current_prepared_run().expect("save");
    let suspend = game
        .lifecycle_request(
            ApplicationSessionStatusV1::Suspended,
            LifecycleReasonKindV1::SuspendRequested,
            "nextengine.test.suspend",
        )
        .expect("request");
    game.publish_transition(suspend, SessionTransitionReferencesV1::default())
        .expect("suspend");
    let loaded = game.load_latest_save_into_live_run().expect("load");
    assert_eq!(
        loaded.authoritative_state_root,
        before.authoritative_state_root
    );
    assert_eq!(game.state().active_save_generation_hash, Some(save_hash));
    let closed = game.close().expect("close");
    assert!(matches!(closed, ApplicationCloseOutcomeV2::Closed { .. }));
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Closed);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn crash_resumes_suspended_from_latest_save() {
    let root = state_root("crash-resume");
    let _ = std::fs::remove_dir_all(&root);
    let mut game = launch(&root);
    let saved = game.run_reference_game(true).expect("run");
    game.save_current_prepared_run().expect("save");
    drop(game);
    let resumed = ApplicationCoordinator::resume(LaunchRequestV1::reference(
        root.clone(),
        CompositionRootV1::Headless,
        PresentationTargetKindV1::None,
    ))
    .expect("resume");
    assert_eq!(resumed.state().state, ApplicationSessionStatusV1::Suspended);
    assert_eq!(
        resumed
            .current_live_run()
            .expect("live")
            .authoritative_state_root,
        saved.authoritative_state_root
    );
    let _ = std::fs::remove_dir_all(root);
}
