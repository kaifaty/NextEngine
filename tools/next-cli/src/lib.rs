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
use serde::{Deserialize, Serialize};

pub const CREATOR_COMMAND_REPORT_SCHEMA_VERSION: u32 = 1;

const UNKNOWN_COMMAND: &str = "unknown";
const PROJECT_VALIDATE_COMMAND: &str = "project.validate";
const PROJECT_COOK_COMMAND: &str = "project.cook";
const CONTENT_CURRENT_FILE: &str = "CURRENT";
static PREFLIGHT_ORDINAL: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatorCommandReportV1 {
    Pass(CreatorCommandPassReportV1),
    Fail(CreatorCommandFailureReportV1),
}

impl CreatorCommandReportV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass(_))
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorCommandPassReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: CreatorProjectDetailsV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorCommandFailureReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub diagnostic: CreatorDiagnosticV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorDiagnosticV1 {
    pub code: String,
    pub subsystem: String,
    pub message_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorProjectDetailsV1 {
    pub project_id: String,
    pub project_revision: u64,
    pub authoring_sha256: String,
    pub project_lock_sha256: String,
    pub schema_registry_sha256: String,
    pub content_manifest_sha256: String,
    pub world_partition_sha256: String,
    pub mechanics_lock_sha256: String,
    pub root_asset_count: u32,
    pub content_entry_count: u32,
    pub neutral_record_count: u32,
    pub render_asset_count: u32,
    pub world_chunk_count: u32,
    pub publication_file_count: u32,
    pub publication_state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CreatorCommand {
    Validate { project: PathBuf },
    Cook { project: PathBuf, output: PathBuf },
}

impl CreatorCommand {
    const fn name(&self) -> &'static str {
        match self {
            Self::Validate { .. } => PROJECT_VALIDATE_COMMAND,
            Self::Cook { .. } => PROJECT_COOK_COMMAND,
        }
    }
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
        };
        CreatorDiagnosticV1 {
            code: code.to_owned(),
            subsystem: subsystem.to_owned(),
            message_key: message_key.to_owned(),
        }
    }
}

#[must_use]
pub fn execute(arguments: impl IntoIterator<Item = OsString>) -> CreatorCommandReportV1 {
    let parsed = parse_arguments(arguments);
    let (command_name, result) = match parsed {
        Ok(command) => {
            let command_name = command.name();
            (command_name, execute_command(command))
        }
        Err((command_name, error)) => (command_name, Err(error)),
    };

    match result {
        Ok(details) => CreatorCommandReportV1::Pass(CreatorCommandPassReportV1 {
            schema_version: CREATOR_COMMAND_REPORT_SCHEMA_VERSION,
            status: "PASS".to_owned(),
            command: command_name.to_owned(),
            details,
        }),
        Err(error) => CreatorCommandReportV1::Fail(CreatorCommandFailureReportV1 {
            schema_version: CREATOR_COMMAND_REPORT_SCHEMA_VERSION,
            status: "FAIL".to_owned(),
            command: command_name.to_owned(),
            diagnostic: error.diagnostic(),
        }),
    }
}

fn parse_arguments(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<CreatorCommand, (&'static str, CreatorFailure)> {
    let mut arguments = arguments.into_iter();
    if arguments.next().as_deref() != Some(OsStr::new("project")) {
        return Err((UNKNOWN_COMMAND, CreatorFailure::Argument));
    }
    let Some(action) = arguments.next() else {
        return Err((UNKNOWN_COMMAND, CreatorFailure::Argument));
    };
    let (command_name, needs_output) = if action == OsStr::new("validate") {
        (PROJECT_VALIDATE_COMMAND, false)
    } else if action == OsStr::new("cook") {
        (PROJECT_COOK_COMMAND, true)
    } else {
        return Err((UNKNOWN_COMMAND, CreatorFailure::Argument));
    };

    let mut project = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let Some(value) = arguments.next() else {
            return Err((command_name, CreatorFailure::Argument));
        };
        if value.is_empty() {
            return Err((command_name, CreatorFailure::Argument));
        }
        if flag == OsStr::new("--project") && project.is_none() {
            project = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--output") && needs_output && output.is_none() {
            output = Some(PathBuf::from(value));
        } else {
            return Err((command_name, CreatorFailure::Argument));
        }
    }

    let project = project.ok_or((command_name, CreatorFailure::Argument))?;
    if needs_output {
        Ok(CreatorCommand::Cook {
            project,
            output: output.ok_or((command_name, CreatorFailure::Argument))?,
        })
    } else if output.is_none() {
        Ok(CreatorCommand::Validate { project })
    } else {
        Err((command_name, CreatorFailure::Argument))
    }
}

fn execute_command(command: CreatorCommand) -> Result<CreatorProjectDetailsV1, CreatorFailure> {
    match command {
        CreatorCommand::Validate { project } => validate_project(&project),
        CreatorCommand::Cook { project, output } => cook_project(&project, &output),
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
    fn parser_accepts_only_the_two_bounded_commands() {
        assert!(matches!(
            parse_arguments(args(&["project", "validate", "--project", "sample"])),
            Ok(CreatorCommand::Validate { project })
                if project.as_path() == Path::new("sample")
        ));
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
        assert!(parse_arguments(args(&["project", "diff"])).is_err());
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
