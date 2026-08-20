use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::ids::content_hash_from_bytes;
use next_contracts::persistence::{
    ManifestCodecError, ManifestValidationError, ReplayManifestV10, WorldStreamingReplayInputV1,
};
use next_project::{activate_project_package, cook_project_v7, load_project_authoring_v7};

use crate::{
    CreatorDiagnosticV1, CreatorFailure, CreatorProjectIdentityV1, map_package_failure,
    project_identity_from_activated, project_identity_from_cooked,
};

mod report;

pub use report::*;

const REPLAY_VALIDATE_COMMAND: &str = "replay.validate";
const REPLAY_INSPECT_COMMAND: &str = "replay.inspect";
const UNKNOWN_COMMAND: &str = "unknown";
const REPLAY_FORMAT_V10: &str = "nextengine.replay-manifest.v10";
const MAXIMUM_REPLAY_BYTES: u64 = 16 * 1024 * 1024;
const MAXIMUM_REPLAY_TICKS: usize = 4_096;
static TEMPORARY_DIRECTORY_ORDINAL: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
enum ReplayOperation {
    Validate,
    Inspect,
}

impl ReplayOperation {
    const fn command(self) -> &'static str {
        match self {
            Self::Validate => REPLAY_VALIDATE_COMMAND,
            Self::Inspect => REPLAY_INSPECT_COMMAND,
        }
    }
}

#[derive(Clone, Copy)]
enum ReplayDomain {
    Runtime,
    WorldServices,
    Physics,
    Owners,
}

enum ReplayProjectInput {
    Project(PathBuf),
    Package(PathBuf),
    ContentStore(PathBuf),
}

struct ReplayCommand {
    operation: ReplayOperation,
    replay: PathBuf,
    input: ReplayProjectInput,
    tick: Option<u64>,
    domain: Option<ReplayDomain>,
}

#[derive(Debug)]
enum ReplayToolError {
    Argument,
    SourceInvalid,
    Invalid,
    Unsupported,
    ProjectMismatch,
    TickNotFound,
    Creator(CreatorDiagnosticV1),
    Replay(next_application::replay::ReplayError),
    Storage,
}

impl ReplayToolError {
    fn from_creator(error: CreatorFailure) -> Self {
        Self::Creator(error.diagnostic())
    }

    fn stable_fields(&self) -> (String, String, String) {
        let fields = match self {
            Self::Argument => (
                "CREATOR_CLI_ARGUMENT_INVALID",
                "creator-cli",
                "creator.cli.argument-invalid",
            ),
            Self::SourceInvalid => (
                "CREATOR_REPLAY_SOURCE_INVALID",
                "creator-replay",
                "creator.replay.source-invalid",
            ),
            Self::Invalid => (
                "CREATOR_REPLAY_INVALID",
                "creator-replay",
                "creator.replay.invalid",
            ),
            Self::Unsupported => (
                "UNSUPPORTED_REPLAY_MANIFEST_VERSION",
                "creator-replay",
                "creator.replay.unsupported-format",
            ),
            Self::ProjectMismatch => (
                "CREATOR_REPLAY_PROJECT_MISMATCH",
                "creator-replay",
                "creator.replay.project-mismatch",
            ),
            Self::TickNotFound => (
                "CREATOR_REPLAY_TICK_NOT_FOUND",
                "creator-replay",
                "creator.replay.tick-not-found",
            ),
            Self::Storage => (
                "CREATOR_REPLAY_STORAGE_FAILED",
                "creator-replay",
                "creator.replay.storage-failed",
            ),
            Self::Creator(diagnostic) => {
                return (
                    diagnostic.code.clone(),
                    diagnostic.subsystem.clone(),
                    diagnostic.message_key.clone(),
                );
            }
            Self::Replay(error) => {
                let key = if error.stable_code() == "NONDETERMINISTIC_RESULT" {
                    "creator.replay.nondeterministic-result"
                } else {
                    "creator.replay.execution-failed"
                };
                return (
                    error.stable_code().to_owned(),
                    "replay".to_owned(),
                    key.to_owned(),
                );
            }
        };
        (
            fields.0.to_owned(),
            fields.1.to_owned(),
            fields.2.to_owned(),
        )
    }
}

struct LoadedReplay {
    manifest: ReplayManifestV10,
    identity: CreatorReplayIdentityV1,
}

struct PreparedReplayProject {
    package: next_project::ActivatedProjectPackage,
    identity: CreatorProjectIdentityV1,
    source: &'static str,
    _temporary: Option<TemporaryDirectory>,
}

pub(super) fn execute(arguments: &[OsString]) -> CreatorReplayCommandReportV1 {
    let command = match parse_command(arguments) {
        Ok(command) => command,
        Err((name, error)) => return failure_report(name, error, None, None),
    };
    let command_name = command.operation.command();
    let loaded = match load_replay(&command.replay) {
        Ok(loaded) => loaded,
        Err(error) => return failure_report(command_name, error, None, None),
    };
    let project = match prepare_project(&command.input) {
        Ok(project) => project,
        Err(error) => {
            return failure_report(command_name, error, Some(&loaded.identity), None);
        }
    };
    if let Err(error) = next_application::replay::validate_replay_manifest_v10_for_project(
        &loaded.manifest,
        &project.package.project,
    ) {
        let error = match error {
            next_application::replay::ReplayError::Manifest(
                ManifestValidationError::CompatibilityMismatch,
            ) => ReplayToolError::ProjectMismatch,
            error => ReplayToolError::Replay(error),
        };
        return failure_report(
            command_name,
            error,
            Some(&loaded.identity),
            Some(&project.identity),
        );
    }

    let details = match command.operation {
        ReplayOperation::Validate => CreatorReplayDetailsV1::Validate {
            replay: loaded.identity,
            project: project.identity,
            source: project.source.to_owned(),
            validation_state: "validated-not-run".to_owned(),
        },
        ReplayOperation::Inspect => {
            let tick = command.tick.expect("inspect parser requires tick");
            let domain = command.domain.expect("inspect parser requires domain");
            let Some(index) = loaded
                .manifest
                .ticks
                .iter()
                .position(|candidate| candidate.tick == tick)
            else {
                return failure_report(
                    command_name,
                    ReplayToolError::TickNotFound,
                    Some(&loaded.identity),
                    Some(&project.identity),
                );
            };
            if let Err(error) = next_application::replay::run_replay_manifest_v10(
                &loaded.manifest,
                project.package.clone(),
            ) {
                return failure_report(
                    command_name,
                    ReplayToolError::Replay(error),
                    Some(&loaded.identity),
                    Some(&project.identity),
                );
            }
            let projection = match project_tick(&loaded.manifest, index, domain) {
                Ok(projection) => projection,
                Err(error) => {
                    return failure_report(
                        command_name,
                        error,
                        Some(&loaded.identity),
                        Some(&project.identity),
                    );
                }
            };
            CreatorReplayDetailsV1::Inspect {
                replay: loaded.identity,
                project: project.identity,
                source: project.source.to_owned(),
                verification_state: "replayed-exact".to_owned(),
                projection,
            }
        }
    };
    CreatorReplayCommandReportV1::Pass(Box::new(CreatorReplayCommandPassReportV1 {
        schema_version: CREATOR_REPLAY_REPORT_SCHEMA_VERSION,
        status: "PASS".to_owned(),
        command: command_name.to_owned(),
        details,
    }))
}

fn parse_command(arguments: &[OsString]) -> Result<ReplayCommand, (&'static str, ReplayToolError)> {
    let Some(action) = arguments.first() else {
        return Err((UNKNOWN_COMMAND, ReplayToolError::Argument));
    };
    let operation = if action == "validate" {
        ReplayOperation::Validate
    } else if action == "inspect" {
        ReplayOperation::Inspect
    } else {
        return Err((UNKNOWN_COMMAND, ReplayToolError::Argument));
    };
    let command = operation.command();
    let mut replay = None;
    let mut project = None;
    let mut package = None;
    let mut content_store = None;
    let mut tick = None;
    let mut domain = None;
    let mut fields = arguments[1..].iter();
    while let Some(flag) = fields.next() {
        let Some(value) = fields.next() else {
            return Err((command, ReplayToolError::Argument));
        };
        if value.is_empty() {
            return Err((command, ReplayToolError::Argument));
        }
        if flag == "--replay" && replay.is_none() {
            replay = Some(PathBuf::from(value));
        } else if flag == "--project" && project.is_none() {
            project = Some(PathBuf::from(value));
        } else if flag == "--package" && package.is_none() {
            package = Some(PathBuf::from(value));
        } else if flag == "--content-store" && content_store.is_none() {
            content_store = Some(PathBuf::from(value));
        } else if flag == "--tick" && tick.is_none() {
            tick = value.to_str().and_then(|value| value.parse::<u64>().ok());
            if tick.is_none() {
                return Err((command, ReplayToolError::Argument));
            }
        } else if flag == "--domain" && domain.is_none() {
            domain = match value.to_str() {
                Some("runtime") => Some(ReplayDomain::Runtime),
                Some("world-services") => Some(ReplayDomain::WorldServices),
                Some("physics") => Some(ReplayDomain::Physics),
                Some("owners") => Some(ReplayDomain::Owners),
                _ => return Err((command, ReplayToolError::Argument)),
            };
        } else {
            return Err((command, ReplayToolError::Argument));
        }
    }
    let input = match (project, package, content_store) {
        (Some(path), None, None) => Some(ReplayProjectInput::Project(path)),
        (None, Some(path), None) => Some(ReplayProjectInput::Package(path)),
        (None, None, Some(path)) => Some(ReplayProjectInput::ContentStore(path)),
        _ => None,
    };
    let filters_valid = match operation {
        ReplayOperation::Validate => tick.is_none() && domain.is_none(),
        ReplayOperation::Inspect => tick.is_some() && domain.is_some(),
    };
    replay
        .zip(input)
        .filter(|_| filters_valid)
        .map(|(replay, input)| ReplayCommand {
            operation,
            replay,
            input,
            tick,
            domain,
        })
        .ok_or((command, ReplayToolError::Argument))
}

fn load_replay(path: &Path) -> Result<LoadedReplay, ReplayToolError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ReplayToolError::SourceInvalid)?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > MAXIMUM_REPLAY_BYTES
    {
        return Err(ReplayToolError::SourceInvalid);
    }
    let bytes = fs::read(path).map_err(|_| ReplayToolError::Storage)?;
    if u64::try_from(bytes.len()).map_err(|_| ReplayToolError::Invalid)? != metadata.len() {
        return Err(ReplayToolError::SourceInvalid);
    }
    let manifest = ReplayManifestV10::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
        .map_err(map_codec_error)?;
    if manifest.ticks.is_empty() || manifest.ticks.len() > MAXIMUM_REPLAY_TICKS {
        return Err(ReplayToolError::Invalid);
    }
    manifest
        .validate_and_decode(CanonicalDecodeLimits::default())
        .map_err(|error| match error {
            ManifestValidationError::UnsupportedReplayVersion(_) => ReplayToolError::Unsupported,
            _ => ReplayToolError::Invalid,
        })?;
    let tick_count = u32::try_from(manifest.ticks.len()).map_err(|_| ReplayToolError::Invalid)?;
    let initial_owner_count = u32::try_from(manifest.initial_owner_segments.len())
        .map_err(|_| ReplayToolError::Invalid)?;
    let first_tick = manifest.ticks.first().ok_or(ReplayToolError::Invalid)?.tick;
    let last_tick = manifest.ticks.last().ok_or(ReplayToolError::Invalid)?.tick;
    let identity = CreatorReplayIdentityV1 {
        format: REPLAY_FORMAT_V10.to_owned(),
        replay_sha256: content_hash_from_bytes(sha256(&bytes)).to_hex(),
        project_id: manifest.compatibility.project_id.as_str().to_owned(),
        initial_state_root: manifest.initial_state_root.to_hex(),
        first_tick,
        last_tick,
        tick_count,
        initial_owner_count,
    };
    Ok(LoadedReplay { manifest, identity })
}

fn map_codec_error(error: ManifestCodecError) -> ReplayToolError {
    match error {
        ManifestCodecError::Validation(ManifestValidationError::UnsupportedReplayVersion(_)) => {
            ReplayToolError::Unsupported
        }
        _ => ReplayToolError::Invalid,
    }
}

fn prepare_project(input: &ReplayProjectInput) -> Result<PreparedReplayProject, ReplayToolError> {
    match input {
        ReplayProjectInput::Project(root) => {
            let source = load_project_authoring_v7(root)
                .map_err(CreatorFailure::Authoring)
                .map_err(ReplayToolError::from_creator)?;
            let cooked = cook_project_v7(source)
                .map_err(CreatorFailure::Cook)
                .map_err(ReplayToolError::from_creator)?;
            let publication = cooked
                .publication()
                .map_err(CreatorFailure::Cook)
                .map_err(ReplayToolError::from_creator)?;
            let temporary = TemporaryDirectory::new()?;
            ContentStore::new(temporary.path())
                .publish(&publication)
                .map_err(|_| ReplayToolError::Storage)?;
            let package = activate_project_package(&ContentStore::new(temporary.path())).map_err(
                |error| ReplayToolError::from_creator(CreatorFailure::Activation(error)),
            )?;
            Ok(PreparedReplayProject {
                identity: project_identity_from_cooked(&cooked),
                package,
                source: "authoring",
                _temporary: Some(temporary),
            })
        }
        ReplayProjectInput::Package(root) => {
            crate::package::validate_and_run(root)
                .map_err(map_package_failure)
                .map_err(ReplayToolError::from_creator)?;
            let package = activate_project_package(&ContentStore::new(root.join("project")))
                .map_err(|error| {
                    ReplayToolError::from_creator(CreatorFailure::Activation(error))
                })?;
            Ok(PreparedReplayProject {
                identity: project_identity_from_activated(&package.project),
                package,
                source: "package",
                _temporary: None,
            })
        }
        ReplayProjectInput::ContentStore(root) => {
            let metadata =
                fs::symlink_metadata(root).map_err(|_| ReplayToolError::SourceInvalid)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(ReplayToolError::SourceInvalid);
            }
            let package = activate_project_package(&ContentStore::new(root)).map_err(|error| {
                ReplayToolError::from_creator(CreatorFailure::Activation(error))
            })?;
            Ok(PreparedReplayProject {
                identity: project_identity_from_activated(&package.project),
                package,
                source: "content-store",
                _temporary: None,
            })
        }
    }
}

fn project_tick(
    manifest: &ReplayManifestV10,
    index: usize,
    domain: ReplayDomain,
) -> Result<CreatorReplayDomainProjectionV1, ReplayToolError> {
    let tick = manifest
        .ticks
        .get(index)
        .ok_or(ReplayToolError::TickNotFound)?;
    let point = manifest
        .compare_points
        .get(index)
        .ok_or(ReplayToolError::Invalid)?;
    let count = |length: usize| u32::try_from(length).map_err(|_| ReplayToolError::Invalid);
    match domain {
        ReplayDomain::Runtime => Ok(CreatorReplayDomainProjectionV1::Runtime {
            tick: tick.tick,
            state_root: point.state_root.to_hex(),
            command_ledger_hash: point.command_ledger_hash.to_hex(),
            closed_ingress_batch_hash: point.closed_ingress_batch_hash.to_hex(),
            ingress_command_batch_hash: point.ingress_command_batch_hash.to_hex(),
            outcome_command_batch_hash: point.outcome_command_batch_hash.to_hex(),
            direct_command_count: count(tick.direct_external_commands.len())?,
            command_result_count: count(tick.expected_command_results.len())?,
            event_count: count(tick.expected_events.len())?,
        }),
        ReplayDomain::WorldServices => {
            let (streaming_operation, target_chunk_id) = match &tick.world_streaming_input {
                WorldStreamingReplayInputV1::None => ("none", None),
                WorldStreamingReplayInputV1::BeginTransition {
                    target_chunk_id, ..
                } => (
                    "begin-transition",
                    Some(target_chunk_id.as_str().to_owned()),
                ),
                WorldStreamingReplayInputV1::CompletePendingTransition {
                    target_chunk_id, ..
                } => (
                    "complete-pending-transition",
                    Some(target_chunk_id.as_str().to_owned()),
                ),
            };
            Ok(CreatorReplayDomainProjectionV1::WorldServices {
                tick: tick.tick,
                streaming_operation: streaming_operation.to_owned(),
                target_chunk_id,
                mapping_receipt_count: count(tick.expected_mapping_receipts.len())?,
                interaction_count: count(tick.expected_interaction_availability.len())?,
                interaction_availability_hash: point.interaction_availability_hash.to_hex(),
                targeting_query_count: count(tick.expected_authoritative_targeting_queries.len())?,
                targeting_query_trace_hash: point.targeting_query_trace_hash.to_hex(),
            })
        }
        ReplayDomain::Physics => Ok(CreatorReplayDomainProjectionV1::Physics {
            tick: tick.tick,
            physics_substeps: tick.expected_physics_step_input.physics_substeps,
            accepted_intent_count: count(tick.expected_physics_step_input.accepted_intents.len())?,
            contact_count: count(tick.expected_contact_batch.events.len())?,
            query_count: count(tick.expected_physics_query_batch.requests.len())?,
            query_result_count: count(tick.expected_physics_query_results.len())?,
            physics_step_input_hash: point.physics_step_input_hash.to_hex(),
            contact_batch_hash: point.contact_batch_hash.to_hex(),
            physics_query_batch_hash: point.physics_query_batch_hash.to_hex(),
            physics_query_results_hash: point.physics_query_results_hash.to_hex(),
        }),
        ReplayDomain::Owners => Ok(CreatorReplayDomainProjectionV1::Owners {
            tick: tick.tick,
            owners: point
                .owner_segments
                .iter()
                .map(|owner| CreatorReplayOwnerProjectionV1 {
                    owner_id: owner.owner_id.as_str().to_owned(),
                    schema_id: owner.schema_id.as_str().to_owned(),
                    segment_id: owner.segment_id.as_str().to_owned(),
                    schema_version: owner.schema_version,
                    byte_length: owner.byte_length,
                    content_hash: owner.content_hash.to_hex(),
                })
                .collect(),
        }),
    }
}

fn failure_report(
    command: &str,
    error: ReplayToolError,
    replay: Option<&CreatorReplayIdentityV1>,
    project: Option<&CreatorProjectIdentityV1>,
) -> CreatorReplayCommandReportV1 {
    let divergence = match &error {
        ReplayToolError::Replay(error) => {
            error
                .first_divergence()
                .map(|divergence| CreatorReplayDivergenceV1 {
                    first_divergent_tick: divergence.first_divergent_tick,
                    stage: divergence.stage.to_owned(),
                    owner: divergence.owner.to_owned(),
                    expected: divergence.expected,
                    actual: divergence.actual,
                })
        }
        _ => None,
    };
    let (code, subsystem, message_key) = error.stable_fields();
    CreatorReplayCommandReportV1::Fail(Box::new(CreatorReplayCommandFailureReportV1 {
        schema_version: CREATOR_REPLAY_REPORT_SCHEMA_VERSION,
        status: "FAIL".to_owned(),
        command: command.to_owned(),
        diagnostic: CreatorReplayDiagnosticV1 {
            code,
            subsystem,
            message_key,
            replay: replay.cloned(),
            project: project.cloned(),
            divergence,
        },
    }))
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new() -> Result<Self, ReplayToolError> {
        for _ in 0..1_024 {
            let ordinal = TEMPORARY_DIRECTORY_ORDINAL.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                ".nextengine-replay-project-{}-{ordinal}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(ReplayToolError::Storage),
            }
        }
        Err(ReplayToolError::Storage)
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure_code(report: CreatorReplayCommandReportV1) -> String {
        let CreatorReplayCommandReportV1::Fail(report) = report else {
            panic!("expected replay failure report");
        };
        report.diagnostic.code
    }

    #[test]
    fn replay_parser_requires_exact_inspection_filters() {
        let report = execute(&[
            "inspect".into(),
            "--replay".into(),
            "replay.jcs".into(),
            "--content-store".into(),
            "store".into(),
            "--tick".into(),
            "0".into(),
        ]);
        assert_eq!(failure_code(report), "CREATOR_CLI_ARGUMENT_INVALID");
    }

    #[test]
    fn retired_replay_version_fails_before_project_access() {
        let directory = TemporaryDirectory::new().expect("temporary directory");
        let replay = directory.path().join("retired.jcs");
        fs::write(&replay, br#"{"schema_version":9}"#).expect("write retired replay");
        let report = execute(&[
            "validate".into(),
            "--replay".into(),
            replay.into_os_string(),
            "--content-store".into(),
            directory.path().join("absent-store").into_os_string(),
        ]);
        assert_eq!(failure_code(report), "UNSUPPORTED_REPLAY_MANIFEST_VERSION");
    }

    #[cfg(unix)]
    #[test]
    fn replay_source_link_is_rejected_without_following_it() {
        use std::os::unix::fs::symlink;

        let directory = TemporaryDirectory::new().expect("temporary directory");
        let target = directory.path().join("target.jcs");
        let link = directory.path().join("link.jcs");
        fs::write(&target, br#"{"schema_version":9}"#).expect("write target replay");
        symlink(&target, &link).expect("link replay");
        let report = execute(&[
            "validate".into(),
            "--replay".into(),
            link.into_os_string(),
            "--content-store".into(),
            directory.path().join("absent-store").into_os_string(),
        ]);
        assert_eq!(failure_code(report), "CREATOR_REPLAY_SOURCE_INVALID");
        assert_eq!(
            fs::read(target).expect("target remains"),
            br#"{"schema_version":9}"#
        );
    }
}
