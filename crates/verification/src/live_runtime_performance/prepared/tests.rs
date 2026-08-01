use super::{
    LiveRuntimePerformanceError, ScratchContext, finish_failed_preparation,
    prepare_live_runtime_long_session_performance_check_in,
    prepare_live_runtime_performance_check_in,
};

#[test]
fn explicit_preparation_matches_the_convenience_path() {
    let scratch = std::env::temp_dir();
    let convenience = super::super::run_live_runtime_performance_check_in(&scratch)
        .expect("convenience live runtime workload");
    let mut prepared =
        prepare_live_runtime_performance_check_in(&scratch).expect("prepared workload");
    assert_eq!(prepared.ticks(), 900);
    let measurement = prepared.run_measured();
    let explicit = prepared
        .finish(measurement)
        .expect("finish prepared workload");
    assert_eq!(explicit.ticks, convenience.ticks);
    assert_eq!(explicit.command_body_count, convenience.command_body_count);
    assert_eq!(
        explicit.driver_prepare_microseconds.len(),
        convenience.driver_prepare_microseconds.len()
    );
    assert_eq!(
        explicit
            .driver_checkpoint_materialization_microseconds
            .len(),
        convenience
            .driver_checkpoint_materialization_microseconds
            .len()
    );
    assert_eq!(
        explicit.final_command_archive_root,
        convenience.final_command_archive_root
    );
    assert_eq!(
        explicit.final_command_identity_index_root,
        convenience.final_command_identity_index_root
    );
    assert_eq!(explicit.final_state_root, convenience.final_state_root);
}

#[test]
fn prepared_workload_rejects_a_second_run_and_cleans_up() {
    let mut prepared = prepare_live_runtime_performance_check_in(&std::env::temp_dir())
        .expect("prepared workload");
    let path = prepared.driver_directory.path().to_path_buf();
    let measurement = prepared.run_measured();
    let second_error = match prepared.run_measured() {
        Err(error) => error,
        Ok(_) => panic!("prepared workload must be single-use"),
    };
    assert!(second_error.to_string().contains("can run only once"));
    prepared.finish(measurement).expect("finish first run");
    assert!(!path.exists());
}

#[test]
fn finish_preserves_a_workload_error_and_cleans_unrun_scratch() {
    let prepared = prepare_live_runtime_performance_check_in(&std::env::temp_dir())
        .expect("prepared workload");
    let path = prepared.driver_directory.path().to_path_buf();
    let result = prepared.finish(injected_failure());
    assert_injected_failure(result);
    assert!(!path.exists());
}

#[test]
fn long_session_error_cleanup_removes_driver_and_application_scratch() {
    let prepared = prepare_live_runtime_long_session_performance_check_in(&std::env::temp_dir())
        .expect("prepared long-session workload");
    let driver_path = prepared.driver_directory.path().to_path_buf();
    let application_path = prepared
        .application_directory
        .as_ref()
        .expect("long-session application directory")
        .path()
        .to_path_buf();
    let result = prepared.finish(injected_failure());
    assert_injected_failure(result);
    assert!(!driver_path.exists());
    assert!(!application_path.exists());
}

#[test]
fn failed_preparation_preserves_primary_and_cleanup_diagnostics() {
    let scratch = ScratchContext::new(&std::env::temp_dir()).expect("scratch context");
    let directory = scratch
        .create_directory("live-runtime-failed-preparation")
        .expect("scratch directory");
    let path = directory.path().to_path_buf();
    std::fs::remove_dir(&path).expect("replace scratch directory");
    std::fs::write(&path, b"not a directory").expect("replacement file");

    let error = finish_failed_preparation(
        directory,
        LiveRuntimePerformanceError::new("prepare application", "injected primary failure"),
        "remove performance fixture",
    );
    let diagnostic = error.to_string();
    std::fs::remove_file(path).expect("remove replacement file");

    assert!(diagnostic.contains("injected primary failure"));
    assert!(diagnostic.contains("changed file type"));
}

fn injected_failure()
-> Result<super::LiveRuntimePerformanceMeasurement, LiveRuntimePerformanceError> {
    Err(LiveRuntimePerformanceError::new(
        "measured workload",
        "injected caller failure",
    ))
}

fn assert_injected_failure(
    result: Result<super::LiveRuntimePerformanceReport, LiveRuntimePerformanceError>,
) {
    assert!(
        result
            .expect_err("workload failure")
            .to_string()
            .contains("injected caller failure")
    );
}
