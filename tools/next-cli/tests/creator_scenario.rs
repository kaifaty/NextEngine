use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_cli::{
    CreatorCliReportV1, CreatorRuntimeScenarioV1, CreatorScenarioAssertionV1,
    CreatorScenarioCommandReportV1, CreatorScenarioDetailsV1,
};

static TEST_ORDINAL: AtomicU64 = AtomicU64::new(0);

fn args(values: &[&Path]) -> Vec<OsString> {
    values
        .iter()
        .map(|value| value.as_os_str().to_owned())
        .collect()
}

fn word(value: &str) -> &Path {
    Path::new(value)
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../projects/creator-smoke")
}

fn tracked_scenario_path() -> PathBuf {
    project_root().join("scenarios/smoke.scenario.json")
}

fn tracked_scenario() -> CreatorRuntimeScenarioV1 {
    serde_json::from_slice(
        &std::fs::read(tracked_scenario_path()).expect("read tracked creator scenario"),
    )
    .expect("decode tracked creator scenario")
}

fn write_scenario(root: &Path, name: &str, scenario: &CreatorRuntimeScenarioV1) -> PathBuf {
    let path = root.join(name);
    let mut bytes = serde_json::to_vec_pretty(scenario).expect("serialize scenario");
    bytes.push(b'\n');
    std::fs::write(&path, bytes).expect("write scenario");
    path
}

fn scenario_report(report: CreatorCliReportV1) -> CreatorScenarioCommandReportV1 {
    let CreatorCliReportV1::Scenario(report) = report else {
        panic!("expected scenario report");
    };
    report
}

#[test]
fn validate_and_run_are_repeatable_path_free_public_operations() {
    let project = project_root();
    let scenario = tracked_scenario_path();
    let validate = next_cli::execute(args(&[
        word("scenario"),
        word("validate"),
        word("--scenario"),
        &scenario,
        word("--project"),
        &project,
    ]));
    let CreatorScenarioCommandReportV1::Pass(validate) = scenario_report(validate) else {
        panic!("scenario validate must pass");
    };
    let CreatorScenarioDetailsV1::Validate {
        scenario: identity,
        validation_state,
        ..
    } = &validate.details
    else {
        panic!("validate details");
    };
    assert_eq!(identity.action_count, 3);
    assert_eq!(identity.assertion_count, 9);
    assert_eq!(validation_state, "validated-not-run");

    let run = || {
        next_cli::execute(args(&[
            word("scenario"),
            word("run"),
            word("--scenario"),
            &scenario,
            word("--project"),
            &project,
        ]))
    };
    let first = run();
    let second = run();
    assert_eq!(first, second);
    let json = first.to_json().expect("scenario JSON");
    assert!(!json.contains(project.to_string_lossy().as_ref()));
    assert!(!json.contains(scenario.to_string_lossy().as_ref()));
    let CreatorScenarioCommandReportV1::Pass(run) = scenario_report(first) else {
        panic!("scenario run must pass");
    };
    let CreatorScenarioDetailsV1::Run {
        runtime,
        assertions_checked,
        ..
    } = &run.details
    else {
        panic!("run details");
    };
    assert_eq!(runtime.ticks, 3);
    assert_eq!(runtime.events, 2);
    assert_eq!(*assertions_checked, 9);
}

#[test]
fn authoring_and_package_execute_the_same_scenario_proof() {
    let temporary = TemporaryDirectory::new("package-scenario");
    let project = project_root();
    let package = temporary.path().join("package");
    let package_report = next_cli::execute(args(&[
        word("project"),
        word("package"),
        word("--project"),
        &project,
        word("--output"),
        &package,
    ]));
    assert!(package_report.is_pass());

    let scenario = tracked_scenario_path();
    let authoring = next_cli::execute(args(&[
        word("scenario"),
        word("run"),
        word("--scenario"),
        &scenario,
        word("--project"),
        &project,
    ]));
    let packaged = next_cli::execute(args(&[
        word("scenario"),
        word("run"),
        word("--scenario"),
        &scenario,
        word("--package"),
        &package,
    ]));
    let CreatorScenarioCommandReportV1::Pass(authoring) = scenario_report(authoring) else {
        panic!("authoring scenario run");
    };
    let CreatorScenarioCommandReportV1::Pass(packaged) = scenario_report(packaged) else {
        panic!("package scenario run");
    };
    let CreatorScenarioDetailsV1::Run {
        runtime: authoring_runtime,
        ..
    } = &authoring.details
    else {
        panic!("authoring run details");
    };
    let CreatorScenarioDetailsV1::Run {
        runtime: package_runtime,
        ..
    } = &packaged.details
    else {
        panic!("package run details");
    };
    assert_eq!(authoring_runtime, package_runtime);
}

#[test]
fn minimize_publishes_the_shortest_prefix_without_weakening_the_assertion() {
    let temporary = TemporaryDirectory::new("minimize");
    let project = project_root();
    let mut failing = tracked_scenario();
    failing.scenario_id = "org.nextengine.creator-smoke.failing-ticks".to_owned();
    failing.assertions = vec![CreatorScenarioAssertionV1::Ticks {
        assertion_id: "runtime-ticks".to_owned(),
        expected: 99,
    }];
    let source = write_scenario(temporary.path(), "failing.json", &failing);

    let run = next_cli::execute(args(&[
        word("scenario"),
        word("run"),
        word("--scenario"),
        &source,
        word("--project"),
        &project,
    ]));
    let CreatorScenarioCommandReportV1::Fail(run) = scenario_report(run) else {
        panic!("failing scenario must fail");
    };
    let failure = run.diagnostic.failure.expect("assertion failure identity");
    assert_eq!(failure.assertion_id, "runtime-ticks");
    assert_eq!(failure.tick, 3);
    assert_eq!(failure.expected, "99");
    assert_eq!(failure.actual, "3");

    let output = temporary.path().join("minimal.json");
    let minimized = next_cli::execute(args(&[
        word("scenario"),
        word("minimize"),
        word("--scenario"),
        &source,
        word("--project"),
        &project,
        word("--output"),
        &output,
    ]));
    let CreatorScenarioCommandReportV1::Pass(minimized) = scenario_report(minimized) else {
        panic!("scenario minimize must pass");
    };
    let CreatorScenarioDetailsV1::Minimize {
        original_scenario,
        minimized_scenario,
        preserved_failure,
        minimization_status,
        ..
    } = &minimized.details
    else {
        panic!("minimize details");
    };
    assert_eq!(original_scenario.action_count, 3);
    assert_eq!(minimized_scenario.action_count, 1);
    assert_eq!(preserved_failure.assertion_id, "runtime-ticks");
    assert_eq!(preserved_failure.tick, 1);
    assert_eq!(minimization_status, "preserved");
    let published: CreatorRuntimeScenarioV1 =
        serde_json::from_slice(&std::fs::read(output).expect("read minimal scenario"))
            .expect("decode minimal scenario");
    assert_eq!(published.actions.len(), 1);
    assert_eq!(published.assertions, failing.assertions);
}

#[test]
fn invalid_sources_and_outputs_fail_without_partial_publication() {
    let temporary = TemporaryDirectory::new("negative");
    let project = project_root();
    let mut retired = tracked_scenario();
    retired.format = "nextengine.creator-runtime-scenario.v0".to_owned();
    let retired = write_scenario(temporary.path(), "retired.json", &retired);
    let report = next_cli::execute(args(&[
        word("scenario"),
        word("validate"),
        word("--scenario"),
        &retired,
        word("--project"),
        &project,
    ]));
    let CreatorScenarioCommandReportV1::Fail(report) = scenario_report(report) else {
        panic!("retired scenario must fail");
    };
    assert_eq!(
        report.diagnostic.code,
        "UNSUPPORTED_CREATOR_SCENARIO_FORMAT"
    );

    let mut mismatch = tracked_scenario();
    mismatch.project.project_lock_sha256 = "00".repeat(32);
    let mismatch = write_scenario(temporary.path(), "project-mismatch.json", &mismatch);
    let report = next_cli::execute(args(&[
        word("scenario"),
        word("validate"),
        word("--scenario"),
        &mismatch,
        word("--project"),
        &project,
    ]));
    let CreatorScenarioCommandReportV1::Fail(report) = scenario_report(report) else {
        panic!("scenario bound to another project closure must fail");
    };
    assert_eq!(report.diagnostic.code, "CREATOR_SCENARIO_PROJECT_MISMATCH");

    let mut over_budget = tracked_scenario();
    over_budget.limits.tick_budget = 2;
    let over_budget = write_scenario(temporary.path(), "over-budget.json", &over_budget);
    let report = next_cli::execute(args(&[
        word("scenario"),
        word("validate"),
        word("--scenario"),
        &over_budget,
        word("--project"),
        &project,
    ]));
    let CreatorScenarioCommandReportV1::Fail(report) = scenario_report(report) else {
        panic!("scenario exceeding its declared tick budget must fail");
    };
    assert_eq!(report.diagnostic.code, "CREATOR_SCENARIO_INVALID");

    let mut unsorted = tracked_scenario();
    unsorted.assertions.swap(0, 1);
    let unsorted = write_scenario(temporary.path(), "unsorted.json", &unsorted);
    let report = next_cli::execute(args(&[
        word("scenario"),
        word("validate"),
        word("--scenario"),
        &unsorted,
        word("--project"),
        &project,
    ]));
    let CreatorScenarioCommandReportV1::Fail(report) = scenario_report(report) else {
        panic!("unsorted assertions must fail");
    };
    assert_eq!(report.diagnostic.code, "CREATOR_SCENARIO_INVALID");

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let linked_source = temporary.path().join("linked-source.json");
        symlink(tracked_scenario_path(), &linked_source).expect("link scenario source");
        let report = next_cli::execute(args(&[
            word("scenario"),
            word("validate"),
            word("--scenario"),
            &linked_source,
            word("--project"),
            &project,
        ]));
        let CreatorScenarioCommandReportV1::Fail(report) = scenario_report(report) else {
            panic!("linked scenario source must fail");
        };
        assert_eq!(report.diagnostic.code, "CREATOR_SCENARIO_SOURCE_INVALID");
    }

    let passing = tracked_scenario_path();
    let absent = temporary.path().join("must-remain-absent.json");
    let report = next_cli::execute(args(&[
        word("scenario"),
        word("minimize"),
        word("--scenario"),
        &passing,
        word("--project"),
        &project,
        word("--output"),
        &absent,
    ]));
    let CreatorScenarioCommandReportV1::Fail(report) = scenario_report(report) else {
        panic!("passing scenario cannot be minimized");
    };
    assert_eq!(report.diagnostic.code, "CREATOR_SCENARIO_NOT_REPRODUCED");
    assert!(!absent.exists());

    let mut failing = tracked_scenario();
    failing.assertions = vec![CreatorScenarioAssertionV1::Ticks {
        assertion_id: "runtime-ticks".to_owned(),
        expected: 99,
    }];
    let failing = write_scenario(temporary.path(), "occupied-source.json", &failing);
    let occupied = temporary.path().join("occupied-output.json");
    std::fs::write(&occupied, b"caller bytes").expect("write occupied output");
    let report = next_cli::execute(args(&[
        word("scenario"),
        word("minimize"),
        word("--scenario"),
        &failing,
        word("--project"),
        &project,
        word("--output"),
        &occupied,
    ]));
    let CreatorScenarioCommandReportV1::Fail(report) = scenario_report(report) else {
        panic!("occupied output must fail");
    };
    assert_eq!(report.diagnostic.code, "CREATOR_SCENARIO_OUTPUT_INVALID");
    assert_eq!(
        std::fs::read(&occupied).expect("read caller output"),
        b"caller bytes"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let target = temporary.path().join("linked-output-target.json");
        std::fs::write(&target, b"target bytes").expect("write output target");
        let linked_output = temporary.path().join("linked-output.json");
        symlink(&target, &linked_output).expect("link scenario output");
        let report = next_cli::execute(args(&[
            word("scenario"),
            word("minimize"),
            word("--scenario"),
            &failing,
            word("--project"),
            &project,
            word("--output"),
            &linked_output,
        ]));
        let CreatorScenarioCommandReportV1::Fail(report) = scenario_report(report) else {
            panic!("linked scenario output must fail");
        };
        assert_eq!(report.diagnostic.code, "CREATOR_SCENARIO_OUTPUT_INVALID");
        assert_eq!(
            std::fs::read(&target).expect("read linked output target"),
            b"target bytes"
        );
    }
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "next-cli-creator-scenario-{label}-{}-{}",
            std::process::id(),
            TEST_ORDINAL.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir(&path).expect("create temporary directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
