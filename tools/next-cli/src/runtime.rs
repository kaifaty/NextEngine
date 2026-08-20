use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_application::{
    ApplicationCoordinator, ApplicationError, LaunchRequestV1, ProjectSelectionV1, RunReportV1,
};
use next_assets::ContentStore;
use next_contracts::ids::ContentHash;
use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};

use crate::CreatorRuntimeProofV1;

static TEMPORARY_DIRECTORY_ORDINAL: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(super) enum RuntimeExecutionError {
    Application(ApplicationError),
    Report,
    Storage,
}

pub(super) fn run_publication(
    publication: &next_assets::ContentPublicationV1,
    expected_project_lock: ContentHash,
) -> Result<CreatorRuntimeProofV1, RuntimeExecutionError> {
    run_publication_with_kind(publication, expected_project_lock, RunKind::Project)
}

pub(super) fn run_publication_scenario(
    publication: &next_assets::ContentPublicationV1,
    expected_project_lock: ContentHash,
    tick_actions: u32,
) -> Result<CreatorRuntimeProofV1, RuntimeExecutionError> {
    run_publication_with_kind(
        publication,
        expected_project_lock,
        RunKind::Scenario { tick_actions },
    )
}

fn run_publication_with_kind(
    publication: &next_assets::ContentPublicationV1,
    expected_project_lock: ContentHash,
    kind: RunKind,
) -> Result<CreatorRuntimeProofV1, RuntimeExecutionError> {
    let temporary = TemporaryDirectory::new("publication")?;
    let result = (|| {
        let project_store_root = temporary.path().join("project");
        ContentStore::new(&project_store_root)
            .publish(publication)
            .map_err(|_| RuntimeExecutionError::Storage)?;
        run_store_in_state_root(
            &project_store_root,
            expected_project_lock,
            &temporary.path().join("state"),
            kind,
        )
    })();
    temporary.finish(result)
}

pub(super) fn run_store(
    project_store_root: &Path,
    expected_project_lock: ContentHash,
) -> Result<CreatorRuntimeProofV1, RuntimeExecutionError> {
    let temporary = TemporaryDirectory::new("state")?;
    let result = run_store_in_state_root(
        project_store_root,
        expected_project_lock,
        &temporary.path().join("state"),
        RunKind::Project,
    );
    temporary.finish(result)
}

pub(super) fn run_store_scenario(
    project_store_root: &Path,
    expected_project_lock: ContentHash,
    tick_actions: u32,
) -> Result<CreatorRuntimeProofV1, RuntimeExecutionError> {
    let temporary = TemporaryDirectory::new("scenario-state")?;
    let result = run_store_in_state_root(
        project_store_root,
        expected_project_lock,
        &temporary.path().join("state"),
        RunKind::Scenario { tick_actions },
    );
    temporary.finish(result)
}

#[derive(Clone, Copy)]
enum RunKind {
    Project,
    Scenario { tick_actions: u32 },
}

fn run_store_in_state_root(
    project_store_root: &Path,
    expected_project_lock: ContentHash,
    state_root: &Path,
    kind: RunKind,
) -> Result<CreatorRuntimeProofV1, RuntimeExecutionError> {
    let mut application = ApplicationCoordinator::launch(LaunchRequestV1 {
        project: ProjectSelectionV1::PublishedStateRoot(project_store_root.to_path_buf()),
        expected_project_lock: Some(expected_project_lock),
        state_root: state_root.to_path_buf(),
        composition_root: CompositionRootV1::Headless,
        presentation_target: PresentationTargetKindV1::None,
        platform_capability_set: None,
    })
    .map_err(RuntimeExecutionError::Application)?;
    let (run, expected_ticks) = match kind {
        RunKind::Project => (
            application
                .run_project_headless()
                .map_err(RuntimeExecutionError::Application)?,
            1,
        ),
        RunKind::Scenario { tick_actions } => (
            application
                .run_project_headless_scenario(tick_actions)
                .map_err(RuntimeExecutionError::Application)?,
            u64::from(tick_actions),
        ),
    };
    let close = application
        .close()
        .map_err(RuntimeExecutionError::Application)?;
    let report = RunReportV1::new(CompositionRootV1::Headless, &run, &close, 0)
        .ok_or(RuntimeExecutionError::Report)?;
    if report.project_composition_lock_hash != expected_project_lock.to_hex()
        || report.presentation.is_some()
        || report.ticks != expected_ticks
        || report.status != "PASS"
    {
        return Err(RuntimeExecutionError::Report);
    }
    Ok(CreatorRuntimeProofV1 {
        status: report.status,
        composition_root: report.composition_root,
        session_id: report.session_id,
        close_receipt_hash: report.close_receipt_hash,
        final_save_generation_hash: report
            .final_save_generation_hash
            .ok_or(RuntimeExecutionError::Report)?,
        ticks: report.ticks,
        events: report.events,
        rpg_events: report.rpg_events,
        authoritative_revision: report.authoritative_revision,
        authoritative_state_root: report.authoritative_state_root,
        command_archive_root: report.command_archive_root,
        command_identity_index_root: report.command_identity_index_root,
        command_ledger_hash: report.command_ledger_hash,
        project_composition_lock_hash: report.project_composition_lock_hash,
    })
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new(label: &str) -> Result<Self, RuntimeExecutionError> {
        for _ in 0..1_024 {
            let ordinal = TEMPORARY_DIRECTORY_ORDINAL.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                ".nextengine-creator-{label}-{}-{ordinal}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(RuntimeExecutionError::Storage),
            }
        }
        Err(RuntimeExecutionError::Storage)
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn finish<T>(
        self,
        result: Result<T, RuntimeExecutionError>,
    ) -> Result<T, RuntimeExecutionError> {
        let cleanup = fs::remove_dir_all(&self.path).map_err(|_| RuntimeExecutionError::Storage);
        std::mem::forget(self);
        match (result, cleanup) {
            (Err(error), _) => Err(error),
            (Ok(value), Ok(())) => Ok(value),
            (Ok(_), Err(error)) => Err(error),
        }
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
