use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_contracts::canonical::sha256;
use next_contracts::ids::content_hash_from_bytes;
use next_project::{CookedProjectV7, cook_project_v7, load_project_authoring_v7};
use serde::{Deserialize, Serialize};

use crate::runtime;
use crate::{
    CreatorDiagnosticV1, CreatorFailure, CreatorProjectIdentityV1, CreatorProjectInput,
    CreatorRuntimeProofV1, map_package_failure, map_runtime_failure, project_identity_from_cooked,
};

mod report;

pub use report::*;

const SCENARIO_VALIDATE_COMMAND: &str = "scenario.validate";
const SCENARIO_RUN_COMMAND: &str = "scenario.run";
const SCENARIO_MINIMIZE_COMMAND: &str = "scenario.minimize";
const UNKNOWN_COMMAND: &str = "unknown";
const MAXIMUM_SCENARIO_BYTES: u64 = 1024 * 1024;
const MAXIMUM_TICK_ACTIONS: usize = 256;
const MAXIMUM_ASSERTIONS: usize = 32;

static STAGING_ORDINAL: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScenarioOperation {
    Validate,
    Run,
    Minimize,
}

impl ScenarioOperation {
    const fn command(self) -> &'static str {
        match self {
            Self::Validate => SCENARIO_VALIDATE_COMMAND,
            Self::Run => SCENARIO_RUN_COMMAND,
            Self::Minimize => SCENARIO_MINIMIZE_COMMAND,
        }
    }
}

#[derive(Debug)]
struct ScenarioCommand {
    operation: ScenarioOperation,
    scenario: PathBuf,
    input: CreatorProjectInput,
    output: Option<PathBuf>,
}

#[derive(Clone, Debug)]
enum ScenarioError {
    Argument,
    Invalid,
    Unsupported,
    SourceInvalid,
    ProjectMismatch,
    Creator(Box<CreatorDiagnosticV1>),
    Assertion(Box<CreatorScenarioFailureIdentityV1>),
    NotReproduced,
    NotMinimizable,
    OutputInvalid,
    Storage,
}

impl ScenarioError {
    fn from_creator(error: CreatorFailure) -> Self {
        Self::Creator(Box::new(error.diagnostic()))
    }

    fn stable_fields(&self) -> (String, String, String) {
        let fields = match self {
            Self::Argument => (
                "CREATOR_CLI_ARGUMENT_INVALID",
                "creator-cli",
                "creator.cli.argument-invalid",
            ),
            Self::Invalid => (
                "CREATOR_SCENARIO_INVALID",
                "creator-scenario",
                "creator.scenario.invalid",
            ),
            Self::Unsupported => (
                "UNSUPPORTED_CREATOR_SCENARIO_FORMAT",
                "creator-scenario",
                "creator.scenario.unsupported-format",
            ),
            Self::SourceInvalid => (
                "CREATOR_SCENARIO_SOURCE_INVALID",
                "creator-scenario",
                "creator.scenario.source-invalid",
            ),
            Self::ProjectMismatch => (
                "CREATOR_SCENARIO_PROJECT_MISMATCH",
                "creator-scenario",
                "creator.scenario.project-mismatch",
            ),
            Self::Assertion(_) => (
                "CREATOR_SCENARIO_ASSERTION_FAILED",
                "creator-scenario",
                "creator.scenario.assertion-failed",
            ),
            Self::NotReproduced => (
                "CREATOR_SCENARIO_NOT_REPRODUCED",
                "creator-scenario",
                "creator.scenario.not-reproduced",
            ),
            Self::NotMinimizable => (
                "CREATOR_SCENARIO_NOT_MINIMIZABLE",
                "creator-scenario",
                "creator.scenario.not-minimizable",
            ),
            Self::OutputInvalid => (
                "CREATOR_SCENARIO_OUTPUT_INVALID",
                "creator-scenario",
                "creator.scenario.output-invalid",
            ),
            Self::Storage => (
                "CREATOR_SCENARIO_STORAGE_FAILED",
                "creator-scenario",
                "creator.scenario.storage-failed",
            ),
            Self::Creator(diagnostic) => {
                return (
                    diagnostic.code.clone(),
                    diagnostic.subsystem.clone(),
                    diagnostic.message_key.clone(),
                );
            }
        };
        (
            fields.0.to_owned(),
            fields.1.to_owned(),
            fields.2.to_owned(),
        )
    }

    const fn minimization_status(&self, command: &str) -> &'static str {
        if !matches!(command.as_bytes(), b"scenario.minimize") {
            return "not-requested";
        }
        match self {
            Self::NotReproduced => "not-reproduced",
            Self::NotMinimizable | Self::Assertion(_) => "not-preserved",
            _ => "not-started",
        }
    }
}

#[derive(Clone, Debug)]
struct LoadedScenario {
    document: CreatorRuntimeScenarioV1,
    identity: CreatorScenarioIdentityV1,
}

enum ScenarioProjectExecution {
    Authoring(Box<CookedProjectV7>),
    Package(PathBuf),
}

struct PreparedScenarioProject {
    identity: CreatorProjectIdentityV1,
    source: &'static str,
    execution: ScenarioProjectExecution,
}

impl PreparedScenarioProject {
    fn run(&self, tick_actions: u32) -> Result<CreatorRuntimeProofV1, ScenarioError> {
        match &self.execution {
            ScenarioProjectExecution::Authoring(cooked) => {
                let publication = cooked
                    .publication()
                    .map_err(CreatorFailure::Cook)
                    .map_err(ScenarioError::from_creator)?;
                runtime::run_publication_scenario(
                    &publication,
                    cooked.project_lock.project_lock_sha256,
                    tick_actions,
                )
                .map_err(map_runtime_failure)
                .map_err(ScenarioError::from_creator)
            }
            ScenarioProjectExecution::Package(root) => runtime::run_store_scenario(
                &root.join("project"),
                content_hash(&self.identity.project_lock_sha256)?,
                tick_actions,
            )
            .map_err(map_runtime_failure)
            .map_err(ScenarioError::from_creator),
        }
    }
}

#[derive(Deserialize)]
struct ScenarioVersionProbe {
    format: String,
}

pub(super) fn execute(arguments: &[OsString]) -> CreatorScenarioCommandReportV1 {
    let command = match parse_command(arguments) {
        Ok(command) => command,
        Err((command, error)) => return failure_report(command, error, None, None),
    };
    let command_name = command.operation.command();
    let loaded = match load_scenario(&command.scenario) {
        Ok(loaded) => loaded,
        Err(error) => return failure_report(command_name, error, None, None),
    };
    let project = match prepare_project(&command.input) {
        Ok(project) => project,
        Err(error) => {
            return failure_report(command_name, error, Some(&loaded.identity), None);
        }
    };
    if loaded.document.project != project.identity {
        return failure_report(
            command_name,
            ScenarioError::ProjectMismatch,
            Some(&loaded.identity),
            Some(&project.identity),
        );
    }

    let result = match command.operation {
        ScenarioOperation::Validate => Ok(CreatorScenarioDetailsV1::Validate {
            scenario: loaded.identity.clone(),
            project: project.identity.clone(),
            source: project.source.to_owned(),
            validation_state: "validated-not-run".to_owned(),
        }),
        ScenarioOperation::Run => {
            run_scenario(&loaded, &project).map(|runtime| CreatorScenarioDetailsV1::Run {
                scenario: loaded.identity.clone(),
                project: project.identity.clone(),
                source: project.source.to_owned(),
                runtime,
                assertions_checked: loaded.identity.assertion_count,
            })
        }
        ScenarioOperation::Minimize => minimize_scenario(
            &loaded,
            &project,
            command.output.as_deref().expect("parser requires output"),
        ),
    };
    match result {
        Ok(details) => pass_report(command_name, details),
        Err(error) => failure_report(
            command_name,
            error,
            Some(&loaded.identity),
            Some(&project.identity),
        ),
    }
}

fn parse_command(arguments: &[OsString]) -> Result<ScenarioCommand, (&'static str, ScenarioError)> {
    let Some(action) = arguments.first() else {
        return Err((UNKNOWN_COMMAND, ScenarioError::Argument));
    };
    let operation = if action == "validate" {
        ScenarioOperation::Validate
    } else if action == "run" {
        ScenarioOperation::Run
    } else if action == "minimize" {
        ScenarioOperation::Minimize
    } else {
        return Err((UNKNOWN_COMMAND, ScenarioError::Argument));
    };
    let command = operation.command();
    let mut scenario = None;
    let mut project = None;
    let mut package = None;
    let mut output = None;
    let mut fields = arguments[1..].iter();
    while let Some(flag) = fields.next() {
        let Some(value) = fields.next() else {
            return Err((command, ScenarioError::Argument));
        };
        if value.is_empty() {
            return Err((command, ScenarioError::Argument));
        }
        if flag == "--scenario" && scenario.is_none() {
            scenario = Some(PathBuf::from(value));
        } else if flag == "--project" && project.is_none() {
            project = Some(PathBuf::from(value));
        } else if flag == "--package" && package.is_none() {
            package = Some(PathBuf::from(value));
        } else if flag == "--output" && output.is_none() {
            output = Some(PathBuf::from(value));
        } else {
            return Err((command, ScenarioError::Argument));
        }
    }
    let input = match (project, package) {
        (Some(project), None) => Some(CreatorProjectInput::Project(project)),
        (None, Some(package)) => Some(CreatorProjectInput::Package(package)),
        _ => None,
    };
    let output_valid = match operation {
        ScenarioOperation::Validate | ScenarioOperation::Run => output.is_none(),
        ScenarioOperation::Minimize => output.is_some(),
    };
    scenario
        .zip(input)
        .filter(|_| output_valid)
        .map(|(scenario, input)| ScenarioCommand {
            operation,
            scenario,
            input,
            output,
        })
        .ok_or((command, ScenarioError::Argument))
}

fn load_scenario(path: &Path) -> Result<LoadedScenario, ScenarioError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ScenarioError::SourceInvalid)?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAXIMUM_SCENARIO_BYTES
    {
        return Err(ScenarioError::SourceInvalid);
    }
    let bytes = fs::read(path).map_err(|_| ScenarioError::Storage)?;
    if u64::try_from(bytes.len()).map_err(|_| ScenarioError::Invalid)? != metadata.len() {
        return Err(ScenarioError::SourceInvalid);
    }
    let probe: ScenarioVersionProbe =
        serde_json::from_slice(&bytes).map_err(|_| ScenarioError::Invalid)?;
    if probe.format != CREATOR_RUNTIME_SCENARIO_FORMAT_V1 {
        return Err(ScenarioError::Unsupported);
    }
    let document: CreatorRuntimeScenarioV1 =
        serde_json::from_slice(&bytes).map_err(|_| ScenarioError::Invalid)?;
    loaded_from_document(document)
}

fn loaded_from_document(
    document: CreatorRuntimeScenarioV1,
) -> Result<LoadedScenario, ScenarioError> {
    validate_document(&document)?;
    let canonical = canonical_json_bytes(&document)?;
    let identity = CreatorScenarioIdentityV1 {
        format: document.format.clone(),
        scenario_id: document.scenario_id.clone(),
        scenario_sha256: hash_bytes(&canonical),
        project_lock_sha256: document.project.project_lock_sha256.clone(),
        action_count: u32::try_from(document.actions.len()).map_err(|_| ScenarioError::Invalid)?,
        assertion_count: u32::try_from(document.assertions.len())
            .map_err(|_| ScenarioError::Invalid)?,
        tick_budget: document.limits.tick_budget,
    };
    Ok(LoadedScenario { document, identity })
}

fn validate_document(document: &CreatorRuntimeScenarioV1) -> Result<(), ScenarioError> {
    if document.format != CREATOR_RUNTIME_SCENARIO_FORMAT_V1
        || !valid_namespaced_id(&document.scenario_id)
        || !valid_namespaced_id(&document.project.project_id)
        || !valid_project_hashes(&document.project)
        || document.actions.is_empty()
        || document.actions.len() > MAXIMUM_TICK_ACTIONS
        || document.limits.tick_budget == 0
        || document.limits.tick_budget as usize > MAXIMUM_TICK_ACTIONS
        || document.actions.len() > document.limits.tick_budget as usize
        || document.assertions.is_empty()
        || document.assertions.len() > MAXIMUM_ASSERTIONS
    {
        return Err(ScenarioError::Invalid);
    }
    let mut action_ids = BTreeSet::new();
    if document
        .actions
        .iter()
        .any(|action| !valid_local_id(&action.action_id) || !action_ids.insert(&action.action_id))
    {
        return Err(ScenarioError::Invalid);
    }
    if document
        .assertions
        .windows(2)
        .any(|pair| pair[0].assertion_id() >= pair[1].assertion_id())
    {
        return Err(ScenarioError::Invalid);
    }
    let mut probes = BTreeSet::new();
    if document.assertions.iter().any(|assertion| {
        !valid_local_id(assertion.assertion_id())
            || !probes.insert(assertion.probe())
            || assertion
                .expected_hash_or_none()
                .is_some_and(|value| !valid_hash(value))
    }) {
        return Err(ScenarioError::Invalid);
    }
    Ok(())
}

fn prepare_project(input: &CreatorProjectInput) -> Result<PreparedScenarioProject, ScenarioError> {
    match input {
        CreatorProjectInput::Project(root) => {
            let source = load_project_authoring_v7(root)
                .map_err(CreatorFailure::Authoring)
                .map_err(ScenarioError::from_creator)?;
            let cooked = cook_project_v7(source)
                .map_err(CreatorFailure::Cook)
                .map_err(ScenarioError::from_creator)?;
            Ok(PreparedScenarioProject {
                identity: project_identity_from_cooked(&cooked),
                source: "authoring",
                execution: ScenarioProjectExecution::Authoring(Box::new(cooked)),
            })
        }
        CreatorProjectInput::Package(root) => {
            let validated = crate::package::validate_and_run(root)
                .map_err(map_package_failure)
                .map_err(ScenarioError::from_creator)?;
            Ok(PreparedScenarioProject {
                identity: validated.project,
                source: "package",
                execution: ScenarioProjectExecution::Package(root.clone()),
            })
        }
    }
}

fn run_scenario(
    scenario: &LoadedScenario,
    project: &PreparedScenarioProject,
) -> Result<CreatorRuntimeProofV1, ScenarioError> {
    let runtime = project.run(scenario.identity.action_count)?;
    for assertion in &scenario.document.assertions {
        let expected = assertion.expected();
        let actual = assertion.actual(&runtime);
        if expected != actual {
            return Err(ScenarioError::Assertion(Box::new(
                CreatorScenarioFailureIdentityV1 {
                    category: "assertion-mismatch".to_owned(),
                    assertion_id: assertion.assertion_id().to_owned(),
                    action_id: scenario
                        .document
                        .actions
                        .last()
                        .expect("validated scenario has actions")
                        .action_id
                        .clone(),
                    tick: runtime.ticks,
                    probe: assertion.probe().to_owned(),
                    expected,
                    actual,
                },
            )));
        }
    }
    Ok(runtime)
}

fn minimize_scenario(
    original: &LoadedScenario,
    project: &PreparedScenarioProject,
    output: &Path,
) -> Result<CreatorScenarioDetailsV1, ScenarioError> {
    let original_failure = match run_scenario(original, project) {
        Ok(_) => return Err(ScenarioError::NotReproduced),
        Err(ScenarioError::Assertion(failure)) => *failure,
        Err(error) => return Err(error),
    };
    let mut minimized = None;
    for prefix_len in 1..=original.document.actions.len() {
        let mut candidate = original.document.clone();
        candidate.actions.truncate(prefix_len);
        let candidate = loaded_from_document(candidate)?;
        if let Err(ScenarioError::Assertion(failure)) = run_scenario(&candidate, project)
            && same_failure(&original_failure, &failure)
        {
            minimized = Some((candidate, *failure));
            break;
        }
    }
    let (minimized, failure) = minimized.ok_or(ScenarioError::NotMinimizable)?;
    let (published, published_failure) =
        publish_minimized(output, &minimized.document, project, &original_failure)?;
    if failure != published_failure {
        return Err(ScenarioError::NotMinimizable);
    }
    Ok(CreatorScenarioDetailsV1::Minimize {
        original_scenario: original.identity.clone(),
        minimized_scenario: published.identity,
        project: project.identity.clone(),
        source: project.source.to_owned(),
        preserved_failure: published_failure,
        minimization_status: "preserved".to_owned(),
    })
}

fn publish_minimized(
    output: &Path,
    document: &CreatorRuntimeScenarioV1,
    project: &PreparedScenarioProject,
    original_failure: &CreatorScenarioFailureIdentityV1,
) -> Result<(LoadedScenario, CreatorScenarioFailureIdentityV1), ScenarioError> {
    let mut bytes = serde_json::to_vec_pretty(document).map_err(|_| ScenarioError::Invalid)?;
    bytes.push(b'\n');
    let staging = StagingFile::new(output)?;
    staging.write(&bytes)?;
    let loaded = load_scenario(staging.path())?;
    let failure = match run_scenario(&loaded, project) {
        Err(ScenarioError::Assertion(failure)) if same_failure(original_failure, &failure) => {
            *failure
        }
        _ => return Err(ScenarioError::NotMinimizable),
    };
    staging.publish()?;
    Ok((loaded, failure))
}

fn same_failure(
    left: &CreatorScenarioFailureIdentityV1,
    right: &CreatorScenarioFailureIdentityV1,
) -> bool {
    left.category == right.category
        && left.assertion_id == right.assertion_id
        && left.probe == right.probe
}

fn pass_report(command: &str, details: CreatorScenarioDetailsV1) -> CreatorScenarioCommandReportV1 {
    CreatorScenarioCommandReportV1::Pass(Box::new(CreatorScenarioCommandPassReportV1 {
        schema_version: CREATOR_SCENARIO_REPORT_SCHEMA_VERSION,
        status: "PASS".to_owned(),
        command: command.to_owned(),
        details,
    }))
}

fn failure_report(
    command: &str,
    error: ScenarioError,
    scenario: Option<&CreatorScenarioIdentityV1>,
    project: Option<&CreatorProjectIdentityV1>,
) -> CreatorScenarioCommandReportV1 {
    let (code, subsystem, message_key) = error.stable_fields();
    let failure = match &error {
        ScenarioError::Assertion(failure) => Some(failure.as_ref().clone()),
        _ => None,
    };
    CreatorScenarioCommandReportV1::Fail(Box::new(CreatorScenarioCommandFailureReportV1 {
        schema_version: CREATOR_SCENARIO_REPORT_SCHEMA_VERSION,
        status: "FAIL".to_owned(),
        command: command.to_owned(),
        diagnostic: CreatorScenarioDiagnosticV1 {
            code,
            subsystem,
            message_key,
            scenario: scenario.cloned(),
            project: project.cloned(),
            failure,
            minimization_status: error.minimization_status(command).to_owned(),
        },
    }))
}

fn valid_project_hashes(project: &CreatorProjectIdentityV1) -> bool {
    [
        &project.authoring_sha256,
        &project.project_lock_sha256,
        &project.schema_registry_sha256,
        &project.content_manifest_sha256,
        &project.world_partition_sha256,
        &project.mechanics_lock_sha256,
    ]
    .into_iter()
    .all(|value| valid_hash(value))
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_namespaced_id(value: &str) -> bool {
    valid_local_id(value)
        && value.contains('.')
        && value.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && segment
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
        })
}

fn valid_local_id(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte))
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
}

fn content_hash(value: &str) -> Result<next_contracts::ids::ContentHash, ScenarioError> {
    if !valid_hash(value) {
        return Err(ScenarioError::Invalid);
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = hex_nibble(pair[0]).ok_or(ScenarioError::Invalid)?;
        let low = hex_nibble(pair[1]).ok_or(ScenarioError::Invalid)?;
        bytes[index] = high << 4 | low;
    }
    Ok(content_hash_from_bytes(bytes))
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, ScenarioError> {
    let value = serde_json::to_value(value).map_err(|_| ScenarioError::Invalid)?;
    let mut bytes = Vec::new();
    write_canonical_value(&value, &mut bytes)?;
    Ok(bytes)
}

fn write_canonical_value(
    value: &serde_json::Value,
    output: &mut Vec<u8>,
) -> Result<(), ScenarioError> {
    match value {
        serde_json::Value::Null => output.extend_from_slice(b"null"),
        serde_json::Value::Bool(value) => {
            output.extend_from_slice(if *value { b"true" } else { b"false" });
        }
        serde_json::Value::Number(value) => output.extend_from_slice(value.to_string().as_bytes()),
        serde_json::Value::String(value) => {
            serde_json::to_writer(output, value).map_err(|_| ScenarioError::Invalid)?;
        }
        serde_json::Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_value(value, output)?;
            }
            output.push(b']');
        }
        serde_json::Value::Object(values) => {
            output.push(b'{');
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(key, _)| *key);
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key).map_err(|_| ScenarioError::Invalid)?;
                output.push(b':');
                write_canonical_value(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> String {
    content_hash_from_bytes(sha256(bytes)).to_hex()
}

struct StagingFile {
    output: PathBuf,
    path: PathBuf,
}

impl StagingFile {
    fn new(requested_output: &Path) -> Result<Self, ScenarioError> {
        let absolute = if requested_output.is_absolute() {
            requested_output.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|_| ScenarioError::Storage)?
                .join(requested_output)
        };
        if !matches!(
            fs::symlink_metadata(&absolute),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        ) {
            return Err(ScenarioError::OutputInvalid);
        }
        let parent = absolute.parent().ok_or(ScenarioError::OutputInvalid)?;
        let resolved_parent = fs::canonicalize(parent).map_err(|_| ScenarioError::OutputInvalid)?;
        let metadata =
            fs::symlink_metadata(&resolved_parent).map_err(|_| ScenarioError::OutputInvalid)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(ScenarioError::OutputInvalid);
        }
        let name = absolute
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(ScenarioError::OutputInvalid)?;
        let output = resolved_parent.join(name);
        for _ in 0..1_024 {
            let ordinal = STAGING_ORDINAL.fetch_add(1, Ordering::Relaxed);
            let path = resolved_parent.join(format!(
                ".{name}.scenario-staging-{}-{ordinal}",
                std::process::id()
            ));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Self { output, path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(ScenarioError::Storage),
            }
        }
        Err(ScenarioError::Storage)
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, bytes: &[u8]) -> Result<(), ScenarioError> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|_| ScenarioError::Storage)?;
        file.write_all(bytes).map_err(|_| ScenarioError::Storage)?;
        file.sync_all().map_err(|_| ScenarioError::Storage)
    }

    fn publish(self) -> Result<(), ScenarioError> {
        match fs::hard_link(&self.path, &self.output) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(ScenarioError::OutputInvalid);
            }
            Err(_) => return Err(ScenarioError::Storage),
        }
        // Publication has already succeeded atomically. Staging cleanup is
        // best-effort so a cleanup fault cannot turn a valid published result
        // into a reported failure with a newly materialized output.
        let _ = fs::remove_file(&self.path);
        Ok(())
    }
}

impl Drop for StagingFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_parser_requires_exact_input_and_minimize_output() {
        let args = |values: &[&str]| values.iter().map(OsString::from).collect::<Vec<_>>();
        assert!(
            parse_command(&args(&[
                "run",
                "--scenario",
                "smoke.json",
                "--project",
                "project",
            ]))
            .is_ok()
        );
        assert!(
            parse_command(&args(&[
                "minimize",
                "--scenario",
                "failure.json",
                "--package",
                "package",
                "--output",
                "minimal.json",
            ]))
            .is_ok()
        );
        assert!(
            parse_command(&args(&[
                "validate",
                "--scenario",
                "smoke.json",
                "--project",
                "project",
                "--package",
                "package",
            ]))
            .is_err()
        );
        assert!(
            parse_command(&args(&[
                "minimize",
                "--scenario",
                "failure.json",
                "--project",
                "project",
            ]))
            .is_err()
        );
    }
}
