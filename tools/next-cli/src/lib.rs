#![forbid(unsafe_code)]

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::{CONTENT_GENERATIONS_DIRECTORY, ContentStore};
use next_project::{
    CookedProjectV7, ProjectActivationError, ProjectAuthoringError, ProjectCookError,
    activate_project, cook_project_v7, load_project_authoring_v7,
};
use serde::Serialize;

mod inspection;
mod package;
mod report;
mod runtime;

pub use inspection::*;
pub use report::*;

pub const CREATOR_COMMAND_REPORT_SCHEMA_VERSION: u32 = 1;
pub const CREATOR_RUN_REPORT_SCHEMA_VERSION: u32 = 1;
pub const CREATOR_PACKAGE_REPORT_SCHEMA_VERSION: u32 = 1;
pub const CREATOR_INSPECT_REPORT_SCHEMA_VERSION: u32 = 1;
pub const CREATOR_DIFF_REPORT_SCHEMA_VERSION: u32 = 1;

const UNKNOWN_COMMAND: &str = "unknown";
const PROJECT_VALIDATE_COMMAND: &str = "project.validate";
const PROJECT_COOK_COMMAND: &str = "project.cook";
const PROJECT_RUN_COMMAND: &str = "project.run";
const PROJECT_PACKAGE_COMMAND: &str = "project.package";
const PROJECT_INSPECT_COMMAND: &str = "project.inspect";
const PROJECT_DIFF_COMMAND: &str = "project.diff";
const CONTENT_CURRENT_FILE: &str = "CURRENT";
static PREFLIGHT_ORDINAL: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CreatorCliReportV1 {
    Project(CreatorCommandReportV1),
    Run(CreatorRunCommandReportV1),
    Package(CreatorPackageCommandReportV1),
    Inspect(CreatorInspectCommandReportV1),
    Diff(CreatorDiffCommandReportV1),
}

impl CreatorCliReportV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        match self {
            Self::Project(report) => report.is_pass(),
            Self::Run(report) => report.is_pass(),
            Self::Package(report) => report.is_pass(),
            Self::Inspect(report) => report.is_pass(),
            Self::Diff(report) => report.is_pass(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CreatorCommand {
    Validate {
        project: PathBuf,
    },
    Cook {
        project: PathBuf,
        output: PathBuf,
    },
    RunProject {
        project: PathBuf,
    },
    RunPackage {
        package: PathBuf,
    },
    Package {
        project: PathBuf,
        output: PathBuf,
    },
    Inspect {
        input: CreatorProjectInput,
    },
    Diff {
        base: CreatorProjectInput,
        candidate: CreatorProjectInput,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CreatorProjectInput {
    Project(PathBuf),
    Package(PathBuf),
}

impl CreatorCommand {
    const fn name(&self) -> &'static str {
        match self {
            Self::Validate { .. } => PROJECT_VALIDATE_COMMAND,
            Self::Cook { .. } => PROJECT_COOK_COMMAND,
            Self::RunProject { .. } | Self::RunPackage { .. } => PROJECT_RUN_COMMAND,
            Self::Package { .. } => PROJECT_PACKAGE_COMMAND,
            Self::Inspect { .. } => PROJECT_INSPECT_COMMAND,
            Self::Diff { .. } => PROJECT_DIFF_COMMAND,
        }
    }

    const fn report_kind(&self) -> CreatorReportKind {
        match self {
            Self::Validate { .. } | Self::Cook { .. } => CreatorReportKind::Project,
            Self::RunProject { .. } | Self::RunPackage { .. } => CreatorReportKind::Run,
            Self::Package { .. } => CreatorReportKind::Package,
            Self::Inspect { .. } => CreatorReportKind::Inspect,
            Self::Diff { .. } => CreatorReportKind::Diff,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CreatorReportKind {
    Project,
    Run,
    Package,
    Inspect,
    Diff,
}

#[derive(Debug)]
enum CreatorFailure {
    Argument,
    Authoring(ProjectAuthoringError),
    Cook(ProjectCookError),
    OutputInvalid,
    Publication,
    Activation(ProjectActivationError),
    ActivationMismatch,
    ReportInvalid,
    Application(next_application::ApplicationError),
    RuntimeStorage,
    PackageOutputInvalid,
    PackageInvalid,
    PackageNoticeInvalid,
    PackageUnsupported,
}

impl CreatorFailure {
    fn diagnostic(&self) -> CreatorDiagnosticV1 {
        let (code, subsystem, message_key) = match self {
            Self::Argument => (
                "CREATOR_CLI_ARGUMENT_INVALID",
                "creator-cli",
                "creator.cli.argument-invalid",
            ),
            Self::Authoring(error) => (
                error.diagnostic_code(),
                "project-authoring",
                "creator.project.authoring-invalid",
            ),
            Self::Cook(error) => (
                error.diagnostic_code(),
                "project-cooker",
                "creator.project.cook-invalid",
            ),
            Self::OutputInvalid => (
                "CREATOR_OUTPUT_INVALID",
                "content-store",
                "creator.output.invalid",
            ),
            Self::Publication => (
                "CONTENT_PUBLICATION_FAILED",
                "content-store",
                "creator.project.publication-failed",
            ),
            Self::Activation(error) => (
                error.diagnostic_code(),
                "project-activation",
                "creator.project.activation-invalid",
            ),
            Self::ActivationMismatch => (
                "PROJECT_HASH_MISMATCH",
                "project-activation",
                "creator.project.activation-invalid",
            ),
            Self::ReportInvalid => (
                "CREATOR_REPORT_INVALID",
                "creator-cli",
                "creator.report.invalid",
            ),
            Self::Application(error) => (
                error.diagnostic_code(),
                "application-session",
                "creator.project.run-failed",
            ),
            Self::RuntimeStorage => (
                "CREATOR_RUNTIME_STORAGE_FAILED",
                "application-session",
                "creator.project.runtime-storage-failed",
            ),
            Self::PackageOutputInvalid => (
                "CREATOR_PACKAGE_OUTPUT_INVALID",
                "creator-package",
                "creator.package.output-invalid",
            ),
            Self::PackageInvalid => (
                "CREATOR_PACKAGE_INVALID",
                "creator-package",
                "creator.package.invalid",
            ),
            Self::PackageNoticeInvalid => (
                "CREATOR_PACKAGE_NOTICE_INVALID",
                "creator-package",
                "creator.package.notice-invalid",
            ),
            Self::PackageUnsupported => (
                "UNSUPPORTED_CREATOR_PACKAGE_FORMAT",
                "creator-package",
                "creator.package.unsupported-format",
            ),
        };
        CreatorDiagnosticV1 {
            code: code.to_owned(),
            subsystem: subsystem.to_owned(),
            message_key: message_key.to_owned(),
        }
    }
}

#[must_use]
pub fn execute(arguments: impl IntoIterator<Item = OsString>) -> CreatorCliReportV1 {
    let parsed = parse_arguments(arguments);
    let (command_name, report_kind, result) = match parsed {
        Ok(command) => {
            let command_name = command.name();
            let report_kind = command.report_kind();
            (command_name, report_kind, execute_command(command))
        }
        Err((command_name, report_kind, error)) => (command_name, report_kind, Err(error)),
    };

    match result {
        Ok(CreatorSuccess::Project(details)) => {
            CreatorCliReportV1::Project(CreatorCommandReportV1::Pass(CreatorCommandPassReportV1 {
                schema_version: CREATOR_COMMAND_REPORT_SCHEMA_VERSION,
                status: "PASS".to_owned(),
                command: command_name.to_owned(),
                details,
            }))
        }
        Ok(CreatorSuccess::Run(details)) => CreatorCliReportV1::Run(
            CreatorRunCommandReportV1::Pass(Box::new(CreatorRunCommandPassReportV1 {
                schema_version: CREATOR_RUN_REPORT_SCHEMA_VERSION,
                status: "PASS".to_owned(),
                command: command_name.to_owned(),
                details,
            })),
        ),
        Ok(CreatorSuccess::Package(details)) => CreatorCliReportV1::Package(
            CreatorPackageCommandReportV1::Pass(Box::new(CreatorPackageCommandPassReportV1 {
                schema_version: CREATOR_PACKAGE_REPORT_SCHEMA_VERSION,
                status: "PASS".to_owned(),
                command: command_name.to_owned(),
                details,
            })),
        ),
        Ok(CreatorSuccess::Inspect(details)) => CreatorCliReportV1::Inspect(
            CreatorInspectCommandReportV1::Pass(Box::new(CreatorInspectCommandPassReportV1 {
                schema_version: CREATOR_INSPECT_REPORT_SCHEMA_VERSION,
                status: "PASS".to_owned(),
                command: command_name.to_owned(),
                details,
            })),
        ),
        Ok(CreatorSuccess::Diff(details)) => CreatorCliReportV1::Diff(
            CreatorDiffCommandReportV1::Pass(Box::new(CreatorDiffCommandPassReportV1 {
                schema_version: CREATOR_DIFF_REPORT_SCHEMA_VERSION,
                status: "PASS".to_owned(),
                command: command_name.to_owned(),
                details,
            })),
        ),
        Err(error) => failure_report(report_kind, command_name, error),
    }
}

fn failure_report(
    kind: CreatorReportKind,
    command_name: &str,
    error: CreatorFailure,
) -> CreatorCliReportV1 {
    let failure = |schema_version| CreatorCommandFailureReportV1 {
        schema_version,
        status: "FAIL".to_owned(),
        command: command_name.to_owned(),
        diagnostic: error.diagnostic(),
    };
    match kind {
        CreatorReportKind::Project => CreatorCliReportV1::Project(CreatorCommandReportV1::Fail(
            failure(CREATOR_COMMAND_REPORT_SCHEMA_VERSION),
        )),
        CreatorReportKind::Run => CreatorCliReportV1::Run(CreatorRunCommandReportV1::Fail(
            failure(CREATOR_RUN_REPORT_SCHEMA_VERSION),
        )),
        CreatorReportKind::Package => CreatorCliReportV1::Package(
            CreatorPackageCommandReportV1::Fail(failure(CREATOR_PACKAGE_REPORT_SCHEMA_VERSION)),
        ),
        CreatorReportKind::Inspect => CreatorCliReportV1::Inspect(
            CreatorInspectCommandReportV1::Fail(failure(CREATOR_INSPECT_REPORT_SCHEMA_VERSION)),
        ),
        CreatorReportKind::Diff => CreatorCliReportV1::Diff(CreatorDiffCommandReportV1::Fail(
            failure(CREATOR_DIFF_REPORT_SCHEMA_VERSION),
        )),
    }
}

enum CreatorSuccess {
    Project(CreatorProjectDetailsV1),
    Run(CreatorRunDetailsV1),
    Package(CreatorPackageDetailsV1),
    Inspect(CreatorInspectDetailsV1),
    Diff(CreatorProjectDiffV1),
}

#[derive(Clone, Copy)]
enum CreatorAction {
    Validate,
    Cook,
    Run,
    Package,
    Inspect,
    Diff,
}

fn parse_arguments(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<CreatorCommand, (&'static str, CreatorReportKind, CreatorFailure)> {
    let mut arguments = arguments.into_iter();
    if arguments.next().as_deref() != Some(OsStr::new("project")) {
        return Err((
            UNKNOWN_COMMAND,
            CreatorReportKind::Project,
            CreatorFailure::Argument,
        ));
    }
    let Some(action) = arguments.next() else {
        return Err((
            UNKNOWN_COMMAND,
            CreatorReportKind::Project,
            CreatorFailure::Argument,
        ));
    };
    let (action, command_name, report_kind) = if action == OsStr::new("validate") {
        (
            CreatorAction::Validate,
            PROJECT_VALIDATE_COMMAND,
            CreatorReportKind::Project,
        )
    } else if action == OsStr::new("cook") {
        (
            CreatorAction::Cook,
            PROJECT_COOK_COMMAND,
            CreatorReportKind::Project,
        )
    } else if action == OsStr::new("run") {
        (
            CreatorAction::Run,
            PROJECT_RUN_COMMAND,
            CreatorReportKind::Run,
        )
    } else if action == OsStr::new("package") {
        (
            CreatorAction::Package,
            PROJECT_PACKAGE_COMMAND,
            CreatorReportKind::Package,
        )
    } else if action == OsStr::new("inspect") {
        (
            CreatorAction::Inspect,
            PROJECT_INSPECT_COMMAND,
            CreatorReportKind::Inspect,
        )
    } else if action == OsStr::new("diff") {
        (
            CreatorAction::Diff,
            PROJECT_DIFF_COMMAND,
            CreatorReportKind::Diff,
        )
    } else {
        return Err((
            UNKNOWN_COMMAND,
            CreatorReportKind::Project,
            CreatorFailure::Argument,
        ));
    };

    let mut project = None;
    let mut package = None;
    let mut output = None;
    let mut base_project = None;
    let mut base_package = None;
    let mut candidate_project = None;
    let mut candidate_package = None;
    while let Some(flag) = arguments.next() {
        let Some(value) = arguments.next() else {
            return Err((command_name, report_kind, CreatorFailure::Argument));
        };
        if value.is_empty() {
            return Err((command_name, report_kind, CreatorFailure::Argument));
        }
        if flag == OsStr::new("--project") && project.is_none() {
            project = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--package") && package.is_none() {
            package = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--output") && output.is_none() {
            output = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--base-project") && base_project.is_none() {
            base_project = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--base-package") && base_package.is_none() {
            base_package = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--candidate-project") && candidate_project.is_none() {
            candidate_project = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--candidate-package") && candidate_package.is_none() {
            candidate_package = Some(PathBuf::from(value));
        } else {
            return Err((command_name, report_kind, CreatorFailure::Argument));
        }
    }

    let no_diff_inputs = base_project.is_none()
        && base_package.is_none()
        && candidate_project.is_none()
        && candidate_package.is_none();
    let result = match action {
        CreatorAction::Validate if package.is_none() && output.is_none() && no_diff_inputs => {
            project.map(|project| CreatorCommand::Validate { project })
        }
        CreatorAction::Cook if package.is_none() && no_diff_inputs => project
            .zip(output)
            .map(|(project, output)| CreatorCommand::Cook { project, output }),
        CreatorAction::Run if output.is_none() && no_diff_inputs => match (project, package) {
            (Some(project), None) => Some(CreatorCommand::RunProject { project }),
            (None, Some(package)) => Some(CreatorCommand::RunPackage { package }),
            _ => None,
        },
        CreatorAction::Package if package.is_none() && no_diff_inputs => project
            .zip(output)
            .map(|(project, output)| CreatorCommand::Package { project, output }),
        CreatorAction::Inspect if output.is_none() && no_diff_inputs => {
            exclusive_project_input(project, package).map(|input| CreatorCommand::Inspect { input })
        }
        CreatorAction::Diff if project.is_none() && package.is_none() && output.is_none() => {
            exclusive_project_input(base_project, base_package)
                .zip(exclusive_project_input(
                    candidate_project,
                    candidate_package,
                ))
                .map(|(base, candidate)| CreatorCommand::Diff { base, candidate })
        }
        _ => None,
    };
    result.ok_or((command_name, report_kind, CreatorFailure::Argument))
}

fn exclusive_project_input(
    project: Option<PathBuf>,
    package: Option<PathBuf>,
) -> Option<CreatorProjectInput> {
    match (project, package) {
        (Some(project), None) => Some(CreatorProjectInput::Project(project)),
        (None, Some(package)) => Some(CreatorProjectInput::Package(package)),
        _ => None,
    }
}

fn execute_command(command: CreatorCommand) -> Result<CreatorSuccess, CreatorFailure> {
    match command {
        CreatorCommand::Validate { project } => {
            validate_project(&project).map(CreatorSuccess::Project)
        }
        CreatorCommand::Cook { project, output } => {
            cook_project(&project, &output).map(CreatorSuccess::Project)
        }
        CreatorCommand::RunProject { project } => run_project(&project).map(CreatorSuccess::Run),
        CreatorCommand::RunPackage { package } => run_package(&package).map(CreatorSuccess::Run),
        CreatorCommand::Package { project, output } => {
            package_project(&project, &output).map(CreatorSuccess::Package)
        }
        CreatorCommand::Inspect { input } => inspect_input(&input).map(CreatorSuccess::Inspect),
        CreatorCommand::Diff { base, candidate } => {
            let base = inspect_input(&base)?;
            let candidate = inspect_input(&candidate)?;
            inspection::diff_projects(&base, &candidate)
                .map(CreatorSuccess::Diff)
                .map_err(|_| CreatorFailure::ReportInvalid)
        }
    }
}

fn validate_project(project: &Path) -> Result<CreatorProjectDetailsV1, CreatorFailure> {
    let source = load_project_authoring_v7(project).map_err(CreatorFailure::Authoring)?;
    let neutral_record_count = source.records.len();
    let cooked = cook_project_v7(source).map_err(CreatorFailure::Cook)?;
    let publication = cooked.publication().map_err(CreatorFailure::Cook)?;
    project_details(
        &cooked,
        neutral_record_count,
        publication.files.len(),
        "validated-not-written",
    )
}

fn cook_project(project: &Path, output: &Path) -> Result<CreatorProjectDetailsV1, CreatorFailure> {
    let source = load_project_authoring_v7(project).map_err(CreatorFailure::Authoring)?;
    let neutral_record_count = source.records.len();
    let cooked = cook_project_v7(source).map_err(CreatorFailure::Cook)?;
    let publication = cooked.publication().map_err(CreatorFailure::Cook)?;
    ensure_managed_output_root(output)?;
    preflight_publication(&publication)?;
    let store = ContentStore::new(output);
    store
        .publish(&publication)
        .map_err(|_| CreatorFailure::Publication)?;
    let activated = activate_project(&store).map_err(CreatorFailure::Activation)?;
    if activated.project_lock.project_lock_sha256 != cooked.project_lock.project_lock_sha256
        || activated.content_manifest.content_manifest_sha256
            != cooked.content_manifest.content_manifest_sha256
    {
        return Err(CreatorFailure::ActivationMismatch);
    }
    project_details(
        &cooked,
        neutral_record_count,
        publication.files.len(),
        "published-and-activated",
    )
}

fn run_project(project: &Path) -> Result<CreatorRunDetailsV1, CreatorFailure> {
    let source = load_project_authoring_v7(project).map_err(CreatorFailure::Authoring)?;
    let cooked = cook_project_v7(source).map_err(CreatorFailure::Cook)?;
    let publication = cooked.publication().map_err(CreatorFailure::Cook)?;
    let runtime = runtime::run_publication(&publication, cooked.project_lock.project_lock_sha256)
        .map_err(map_runtime_failure)?;
    Ok(CreatorRunDetailsV1 {
        project: project_identity_from_cooked(&cooked),
        runtime,
        source: "authoring".to_owned(),
    })
}

fn run_package(package_root: &Path) -> Result<CreatorRunDetailsV1, CreatorFailure> {
    let validated = package::validate_and_run(package_root).map_err(map_package_failure)?;
    Ok(CreatorRunDetailsV1 {
        project: validated.project,
        runtime: validated.runtime,
        source: "package".to_owned(),
    })
}

fn package_project(
    project: &Path,
    output: &Path,
) -> Result<CreatorPackageDetailsV1, CreatorFailure> {
    let source = load_project_authoring_v7(project).map_err(CreatorFailure::Authoring)?;
    let cooked = cook_project_v7(source).map_err(CreatorFailure::Cook)?;
    package::build_project_package(project, &cooked, output).map_err(map_package_failure)
}

fn inspect_input(input: &CreatorProjectInput) -> Result<CreatorInspectDetailsV1, CreatorFailure> {
    match input {
        CreatorProjectInput::Project(project) => inspect_project(project),
        CreatorProjectInput::Package(package) => inspect_package(package),
    }
}

fn inspect_project(project: &Path) -> Result<CreatorInspectDetailsV1, CreatorFailure> {
    let source = load_project_authoring_v7(project).map_err(CreatorFailure::Authoring)?;
    let neutral_records = source.records.clone();
    let cooked = cook_project_v7(source).map_err(CreatorFailure::Cook)?;
    inspection::inspect_cooked(&cooked, &neutral_records).map_err(|_| CreatorFailure::ReportInvalid)
}

fn inspect_package(package_root: &Path) -> Result<CreatorInspectDetailsV1, CreatorFailure> {
    let validated = package::validate_and_run(package_root).map_err(map_package_failure)?;
    inspection::inspect_activated(&validated.activated).map_err(|_| CreatorFailure::ReportInvalid)
}

fn map_runtime_failure(error: runtime::RuntimeExecutionError) -> CreatorFailure {
    match error {
        runtime::RuntimeExecutionError::Application(error) => CreatorFailure::Application(error),
        runtime::RuntimeExecutionError::Report => CreatorFailure::ReportInvalid,
        runtime::RuntimeExecutionError::Storage => CreatorFailure::RuntimeStorage,
    }
}

fn map_package_failure(error: package::CreatorPackageError) -> CreatorFailure {
    match error {
        package::CreatorPackageError::OutputInvalid => CreatorFailure::PackageOutputInvalid,
        package::CreatorPackageError::NoticeInvalid => CreatorFailure::PackageNoticeInvalid,
        package::CreatorPackageError::Unsupported => CreatorFailure::PackageUnsupported,
        package::CreatorPackageError::Runtime(error) => map_runtime_failure(error),
        package::CreatorPackageError::Activation
        | package::CreatorPackageError::Invalid
        | package::CreatorPackageError::Storage => CreatorFailure::PackageInvalid,
    }
}

fn preflight_publication(
    publication: &next_assets::ContentPublicationV1,
) -> Result<(), CreatorFailure> {
    let temporary_root = create_preflight_directory()?;
    let result = (|| {
        let store = ContentStore::new(&temporary_root);
        store
            .publish(publication)
            .map_err(|_| CreatorFailure::Publication)?;
        activate_project(&store).map_err(CreatorFailure::Activation)?;
        Ok(())
    })();
    let cleanup = fs::remove_dir_all(&temporary_root).map_err(|_| CreatorFailure::Publication);
    result.and(cleanup)
}

fn create_preflight_directory() -> Result<PathBuf, CreatorFailure> {
    for _ in 0..1_024 {
        let ordinal = PREFLIGHT_ORDINAL.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            ".nextengine-creator-preflight-{}-{ordinal}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(_) => return Err(CreatorFailure::Publication),
        }
    }
    Err(CreatorFailure::Publication)
}

fn ensure_managed_output_root(output: &Path) -> Result<(), CreatorFailure> {
    let metadata = match fs::symlink_metadata(output) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(CreatorFailure::Publication),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(CreatorFailure::OutputInvalid);
    }

    let mut has_generations = false;
    let mut has_current = false;
    for entry in fs::read_dir(output).map_err(|_| CreatorFailure::Publication)? {
        let entry = entry.map_err(|_| CreatorFailure::Publication)?;
        let name = entry.file_name();
        if name == OsStr::new(CONTENT_GENERATIONS_DIRECTORY) {
            let metadata =
                fs::symlink_metadata(entry.path()).map_err(|_| CreatorFailure::Publication)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(CreatorFailure::OutputInvalid);
            }
            has_generations = true;
        } else if name == OsStr::new(CONTENT_CURRENT_FILE) {
            let metadata =
                fs::symlink_metadata(entry.path()).map_err(|_| CreatorFailure::Publication)?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(CreatorFailure::OutputInvalid);
            }
            has_current = true;
        } else {
            return Err(CreatorFailure::OutputInvalid);
        }
    }
    if has_current && !has_generations {
        return Err(CreatorFailure::OutputInvalid);
    }
    if has_current && activate_project(&ContentStore::new(output)).is_err() {
        return Err(CreatorFailure::OutputInvalid);
    }
    Ok(())
}

pub(crate) fn project_identity_from_cooked(cooked: &CookedProjectV7) -> CreatorProjectIdentityV1 {
    CreatorProjectIdentityV1 {
        project_id: cooked.project_lock.project_id.as_str().to_owned(),
        project_revision: cooked.project_lock.project_revision,
        authoring_sha256: cooked.project_lock.authoring_sha256.to_hex(),
        project_lock_sha256: cooked.project_lock.project_lock_sha256.to_hex(),
        schema_registry_sha256: cooked
            .schema_registry
            .schema_registry_manifest_sha256
            .to_hex(),
        content_manifest_sha256: cooked.content_manifest.content_manifest_sha256.to_hex(),
        world_partition_sha256: cooked
            .world_partition
            .world_partition_manifest_sha256
            .to_hex(),
        mechanics_lock_sha256: cooked
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256
            .to_hex(),
    }
}

pub(crate) fn project_identity_from_activated(
    activated: &next_contracts::project::ActivatedProjectV8,
) -> CreatorProjectIdentityV1 {
    CreatorProjectIdentityV1 {
        project_id: activated.project_lock.project_id.as_str().to_owned(),
        project_revision: activated.project_lock.project_revision,
        authoring_sha256: activated.project_lock.authoring_sha256.to_hex(),
        project_lock_sha256: activated.project_lock.project_lock_sha256.to_hex(),
        schema_registry_sha256: activated
            .schema_registry
            .schema_registry_manifest_sha256
            .to_hex(),
        content_manifest_sha256: activated.content_manifest.content_manifest_sha256.to_hex(),
        world_partition_sha256: activated
            .world_partition
            .world_partition_manifest_sha256
            .to_hex(),
        mechanics_lock_sha256: activated
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256
            .to_hex(),
    }
}

fn project_details(
    cooked: &CookedProjectV7,
    neutral_record_count: usize,
    publication_file_count: usize,
    publication_state: &str,
) -> Result<CreatorProjectDetailsV1, CreatorFailure> {
    let render = &cooked.render_content_catalog;
    let render_asset_count = 1_usize
        .checked_add(render.meshes().len())
        .and_then(|count| count.checked_add(render.materials().len()))
        .and_then(|count| count.checked_add(render.textures().len()))
        .and_then(|count| count.checked_add(render.base_skinning_profiles().len()))
        .ok_or(CreatorFailure::ReportInvalid)?;
    Ok(CreatorProjectDetailsV1 {
        project_id: cooked.project_lock.project_id.as_str().to_owned(),
        project_revision: cooked.project_lock.project_revision,
        authoring_sha256: cooked.project_lock.authoring_sha256.to_hex(),
        project_lock_sha256: cooked.project_lock.project_lock_sha256.to_hex(),
        schema_registry_sha256: cooked
            .schema_registry
            .schema_registry_manifest_sha256
            .to_hex(),
        content_manifest_sha256: cooked.content_manifest.content_manifest_sha256.to_hex(),
        world_partition_sha256: cooked
            .world_partition
            .world_partition_manifest_sha256
            .to_hex(),
        mechanics_lock_sha256: cooked
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256
            .to_hex(),
        root_asset_count: report_count(cooked.content_manifest.body.root_assets.len())?,
        content_entry_count: report_count(cooked.content_manifest.body.asset_entries.len())?,
        neutral_record_count: report_count(neutral_record_count)?,
        render_asset_count: report_count(render_asset_count)?,
        world_chunk_count: report_count(cooked.world_partition.body.chunk_bindings.len())?,
        publication_file_count: report_count(publication_file_count)?,
        publication_state: publication_state.to_owned(),
    })
}

fn report_count(value: usize) -> Result<u32, CreatorFailure> {
    u32::try_from(value).map_err(|_| CreatorFailure::ReportInvalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parser_accepts_only_the_six_bounded_operations() {
        assert!(matches!(
            parse_arguments(args(&["project", "validate", "--project", "sample"])),
            Ok(CreatorCommand::Validate { project })
                if project.as_path() == Path::new("sample")
        ));
        assert!(matches!(
            parse_arguments(args(&["project", "run", "--project", "sample"])),
            Ok(CreatorCommand::RunProject { project })
                if project.as_path() == Path::new("sample")
        ));
        assert!(matches!(
            parse_arguments(args(&["project", "run", "--package", "bundle"])),
            Ok(CreatorCommand::RunPackage { package })
                if package.as_path() == Path::new("bundle")
        ));
        assert!(matches!(
            parse_arguments(args(&[
                "project",
                "package",
                "--project",
                "sample",
                "--output",
                "bundle",
            ])),
            Ok(CreatorCommand::Package { project, output })
                if project.as_path() == Path::new("sample")
                    && output.as_path() == Path::new("bundle")
        ));
        assert!(
            parse_arguments(args(&[
                "project",
                "run",
                "--project",
                "sample",
                "--package",
                "bundle",
            ]))
            .is_err()
        );
        assert!(matches!(
            parse_arguments(args(&[
                "project",
                "cook",
                "--output",
                "out",
                "--project",
                "sample",
            ])),
            Ok(CreatorCommand::Cook { project, output })
                if project.as_path() == Path::new("sample")
                    && output.as_path() == Path::new("out")
        ));
        assert!(matches!(
            parse_arguments(args(&[
                "project",
                "inspect",
                "--package",
                "bundle",
            ])),
            Ok(CreatorCommand::Inspect {
                input: CreatorProjectInput::Package(package),
            }) if package.as_path() == Path::new("bundle")
        ));
        assert!(matches!(
            parse_arguments(args(&[
                "project",
                "diff",
                "--base-project",
                "before",
                "--candidate-package",
                "after",
            ])),
            Ok(CreatorCommand::Diff {
                base: CreatorProjectInput::Project(base),
                candidate: CreatorProjectInput::Package(candidate),
            }) if base.as_path() == Path::new("before")
                && candidate.as_path() == Path::new("after")
        ));
        assert!(parse_arguments(args(&["project", "diff"])).is_err());
        assert!(
            parse_arguments(args(&[
                "project",
                "inspect",
                "--project",
                "sample",
                "--package",
                "bundle",
            ]))
            .is_err()
        );
        assert!(
            parse_arguments(args(&[
                "project",
                "diff",
                "--base-project",
                "before",
                "--base-package",
                "before-package",
                "--candidate-project",
                "after",
            ]))
            .is_err()
        );
        assert!(
            parse_arguments(args(&[
                "project",
                "validate",
                "--project",
                "one",
                "--project",
                "two",
            ]))
            .is_err()
        );
    }

    #[test]
    fn argument_failure_is_one_stable_path_free_json_object() {
        let report = execute(args(&["project", "cook", "--project", "sample"]));
        assert!(!report.is_pass());
        assert_eq!(
            report.to_json().expect("report JSON"),
            concat!(
                "{\"schema_version\":1,\"status\":\"FAIL\",",
                "\"command\":\"project.cook\",\"diagnostic\":{",
                "\"code\":\"CREATOR_CLI_ARGUMENT_INVALID\",",
                "\"subsystem\":\"creator-cli\",",
                "\"message_key\":\"creator.cli.argument-invalid\"}}"
            )
        );
    }
}
