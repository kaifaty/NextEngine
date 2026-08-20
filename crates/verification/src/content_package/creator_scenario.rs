use std::ffi::OsString;
use std::path::Path;

use next_cli::{
    CreatorCliReportV1, CreatorRuntimeScenarioV1, CreatorScenarioAssertionV1,
    CreatorScenarioCommandReportV1, CreatorScenarioDetailsV1,
};

use super::{ContentPackageCheckError, ScratchContext};

pub(super) fn verify(scratch: &ScratchContext) -> Result<(), ContentPackageCheckError> {
    let directory = scratch
        .create_directory("content-package-creator-scenario")
        .map_err(ContentPackageCheckError::Cleanup)?;
    let result = (|| {
        let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../projects/creator-smoke");
        let scenario = project.join("scenarios/smoke.scenario.json");
        let authoring_validate =
            execute_scenario("validate", &scenario, "--project", &project, None);
        let CreatorScenarioCommandReportV1::Pass(authoring_validate) = authoring_validate else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let CreatorScenarioDetailsV1::Validate {
            scenario: validated_identity,
            ..
        } = &authoring_validate.details
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        if validated_identity.action_count != 3 || validated_identity.assertion_count != 9 {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }

        let authoring_run = execute_scenario("run", &scenario, "--project", &project, None);
        let CreatorScenarioCommandReportV1::Pass(authoring_run) = authoring_run else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let CreatorScenarioDetailsV1::Run {
            runtime: authoring_runtime,
            assertions_checked,
            ..
        } = &authoring_run.details
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        if authoring_runtime.ticks != 3 || authoring_runtime.events != 2 || *assertions_checked != 9
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }

        let package = directory.path().join("package");
        let packaged = next_cli::execute([
            OsString::from("project"),
            OsString::from("package"),
            OsString::from("--project"),
            project.as_os_str().to_owned(),
            OsString::from("--output"),
            package.as_os_str().to_owned(),
        ]);
        if !packaged.is_pass() {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        let package_run = execute_scenario("run", &scenario, "--package", &package, None);
        let CreatorScenarioCommandReportV1::Pass(package_run) = package_run else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let CreatorScenarioDetailsV1::Run {
            runtime: package_runtime,
            ..
        } = &package_run.details
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        if authoring_runtime != package_runtime {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }

        let mut failing: CreatorRuntimeScenarioV1 = serde_json::from_slice(
            &std::fs::read(&scenario).map_err(ContentPackageCheckError::Cleanup)?,
        )
        .map_err(|_| ContentPackageCheckError::FixtureClosureMismatch)?;
        failing.scenario_id = "org.nextengine.creator-smoke.governing-failure".to_owned();
        failing.assertions = vec![CreatorScenarioAssertionV1::Ticks {
            assertion_id: "runtime-ticks".to_owned(),
            expected: 99,
        }];
        let failing_path = directory.path().join("failing.scenario.json");
        let mut bytes = serde_json::to_vec_pretty(&failing)
            .map_err(|_| ContentPackageCheckError::FixtureClosureMismatch)?;
        bytes.push(b'\n');
        std::fs::write(&failing_path, bytes).map_err(ContentPackageCheckError::Cleanup)?;
        let minimized_path = directory.path().join("minimal.scenario.json");
        let minimized = execute_scenario(
            "minimize",
            &failing_path,
            "--project",
            &project,
            Some(&minimized_path),
        );
        let CreatorScenarioCommandReportV1::Pass(minimized) = minimized else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let CreatorScenarioDetailsV1::Minimize {
            original_scenario,
            minimized_scenario,
            preserved_failure,
            minimization_status,
            ..
        } = &minimized.details
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        if original_scenario.action_count != 3
            || minimized_scenario.action_count != 1
            || preserved_failure.assertion_id != "runtime-ticks"
            || preserved_failure.tick != 1
            || minimization_status != "preserved"
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        Ok(())
    })();
    directory.finish(result, ContentPackageCheckError::Cleanup)
}

fn execute_scenario(
    action: &str,
    scenario: &Path,
    input_flag: &str,
    input: &Path,
    output: Option<&Path>,
) -> CreatorScenarioCommandReportV1 {
    let mut arguments = vec![
        OsString::from("scenario"),
        OsString::from(action),
        OsString::from("--scenario"),
        scenario.as_os_str().to_owned(),
        OsString::from(input_flag),
        input.as_os_str().to_owned(),
    ];
    if let Some(output) = output {
        arguments.push(OsString::from("--output"));
        arguments.push(output.as_os_str().to_owned());
    }
    let CreatorCliReportV1::Scenario(report) = next_cli::execute(arguments) else {
        unreachable!("scenario command always selects the scenario report family");
    };
    report
}
