use std::collections::BTreeMap;
use std::fs;

use next_assets::{ContentStore, SaveStore, SessionStore};
use next_contracts::ids::{ApplicationSessionId, ContentHash};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::session::{
    ApplicationSessionManifestBodyV1, ApplicationSessionManifestV1, ApplicationSessionStatusV1,
    PresentationTargetKindV1,
};
use next_project::{activate_project, cook_project_v1};
use next_reference_game::project_source_v2;
use next_runtime::ApplicationSessionMachine;

use crate::close::{ApplicationCloseOutcomeV1, CloseExecutionOptionsV1};
use crate::durable::DurableApplicationSnapshotV1;
use crate::{ApplicationError, LaunchRequestV1, ProjectSelectionV1};

use super::identity::derive_session_id;
use super::publication::restore_lifecycle_archive;
use super::recovery::restore_recovery_evidence_archive;
use super::run::restore_live_run;
use super::{ApplicationCoordinator, PROJECT_DIRECTORY, SAVE_DIRECTORY, SESSION_DIRECTORY};

impl ApplicationCoordinator {
    pub fn launch_or_resume(launch: LaunchRequestV1) -> Result<Self, ApplicationError> {
        match Self::launch(launch.clone()) {
            Ok(application) => Ok(application),
            Err(ApplicationError::SessionAlreadyLive) => {
                let mut resumed = Self::resume(launch.clone())?;
                if resumed.durable.close.is_some()
                    && resumed.state().state != ApplicationSessionStatusV1::Closed
                {
                    let close = resumed.close(CloseExecutionOptionsV1::default())?;
                    if !matches!(close, ApplicationCloseOutcomeV1::Closed { .. }) {
                        return Err(ApplicationError::FinalSaveFailed);
                    }
                    return Self::launch(launch);
                }
                Ok(resumed)
            }
            Err(error) => Err(error),
        }
    }

    pub fn launch(launch: LaunchRequestV1) -> Result<Self, ApplicationError> {
        fs::create_dir_all(&launch.state_root)?;
        let activated_project = activate_selected_project(&launch)?;
        validate_launch(&launch, &activated_project)?;
        let session_store = SessionStore::new(launch.state_root.join(SESSION_DIRECTORY));
        let current = load_session_generation_if_present(&session_store)?;
        if current
            .as_ref()
            .is_some_and(|value| value.live_session_id.is_some())
        {
            return Err(ApplicationError::SessionAlreadyLive);
        }
        let sequence = current.as_ref().map_or(0, |value| value.sequence + 1);
        let session_id = derive_session_id(
            activated_project.composition_lock.composition_lock_sha256,
            launch.composition_root,
            sequence,
        );
        let manifest = session_manifest(&launch, &activated_project, session_id, None)?;
        let machine = ApplicationSessionMachine::new(manifest.clone())?;
        let durable = DurableApplicationSnapshotV1 {
            store_sequence: sequence,
            manifest: manifest.clone(),
            state: machine.state().clone(),
            close: None,
            live_run_recovery_manifest_hash: None,
            live_run_recovery_field_present: true,
            lifecycle_archive: Vec::new(),
            lifecycle_archive_field_present: true,
            recovery_evidence_archive: Vec::new(),
        };
        let mut coordinator = Self {
            save_store: SaveStore::new(launch.state_root.join(SAVE_DIRECTORY)),
            launch,
            activated_project,
            session_store,
            machine,
            durable,
            current_generation: ContentHash::default(),
            objects: BTreeMap::new(),
            prepared_run_objects: BTreeMap::new(),
            prepared_run: None,
            live_run: None,
            platform_host: None,
            #[cfg(test)]
            pause_after_save_commit: false,
            #[cfg(test)]
            fail_next_state_publication: false,
        };
        coordinator.record_object(manifest.to_jcs_bytes());
        coordinator.publish_current(
            current.as_ref().map(|value| value.generation_id),
            Some(session_id),
            None,
        )?;
        coordinator.finish_launch()?;
        Ok(coordinator)
    }

    pub fn resume(launch: LaunchRequestV1) -> Result<Self, ApplicationError> {
        fs::create_dir_all(&launch.state_root)?;
        let activated_project = activate_selected_project(&launch)?;
        validate_launch(&launch, &activated_project)?;
        let session_store = SessionStore::new(launch.state_root.join(SESSION_DIRECTORY));
        let published = session_store.load_current()?;
        let mut durable = DurableApplicationSnapshotV1::from_canonical_bytes(&published.snapshot)?;
        if durable.manifest.body.project_composition_lock_hash
            != activated_project.composition_lock.composition_lock_sha256
            || durable.manifest.body.composition_root != launch.composition_root
            || durable.manifest.body.presentation_target_kind != launch.presentation_target
            || durable.manifest.body.platform_capability_set_hash
                != launch
                    .platform_capability_set
                    .as_ref()
                    .map(|capabilities| capabilities.canonical_hash)
            || published.project_composition_lock_hash
                != durable.manifest.body.project_composition_lock_hash
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        if (durable.state.state == ApplicationSessionStatusV1::Closed)
            != published.live_session_id.is_none()
            || published
                .live_session_id
                .is_some_and(|id| id != durable.state.session_id)
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        let archived_requests = restore_lifecycle_archive(&durable, &published.objects)?;
        restore_recovery_evidence_archive(&durable, &published.objects)?;
        let machine = ApplicationSessionMachine::restore(
            durable.manifest.clone(),
            durable.state.clone(),
            archived_requests,
        )?;
        let (objects, prepared_run_objects, prepared_run, live_run) =
            if let Some(manifest_hash) = durable.live_run_recovery_manifest_hash {
                let recovered = restore_live_run(
                    published.objects,
                    manifest_hash,
                    durable.state.session_id,
                    activated_project.clone(),
                    durable
                        .state
                        .active_runtime_revision
                        .ok_or(ApplicationError::RecoveryIncompatible)?,
                    launch.presentation_target,
                )?;
                if durable.state.active_runtime_revision
                    != Some(recovered.prepared_run.summary.authoritative_revision)
                {
                    return Err(ApplicationError::RecoveryIncompatible);
                }
                durable.live_run_recovery_manifest_hash =
                    Some(recovered.live_run_recovery_manifest_hash);
                let live_run = matches!(
                    durable.state.state,
                    ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended
                )
                .then_some(recovered.live_run);
                (
                    recovered.lifecycle_objects,
                    recovered.prepared_run_objects,
                    Some(recovered.prepared_run),
                    live_run,
                )
            } else {
                if !durable.live_run_recovery_field_present
                    && durable.state.active_runtime_revision.is_some()
                {
                    return Err(ApplicationError::RecoveryIncompatible);
                }
                (published.objects, BTreeMap::new(), None, None)
            };
        let mut coordinator = Self {
            save_store: SaveStore::new(launch.state_root.join(SAVE_DIRECTORY)),
            launch,
            activated_project,
            session_store,
            machine,
            durable,
            current_generation: published.generation_id,
            objects,
            prepared_run_objects,
            prepared_run,
            live_run,
            platform_host: None,
            #[cfg(test)]
            pause_after_save_commit: false,
            #[cfg(test)]
            fail_next_state_publication: false,
        };
        coordinator.finish_launch()?;
        Ok(coordinator)
    }
}

pub(super) fn session_manifest(
    launch: &LaunchRequestV1,
    project: &ActivatedProjectV2,
    session_id: ApplicationSessionId,
    recovery_session_link_hash: Option<ContentHash>,
) -> Result<ApplicationSessionManifestV1, ApplicationError> {
    let lock = &project.composition_lock;
    Ok(ApplicationSessionManifestV1::new(
        ApplicationSessionManifestBodyV1 {
            session_id,
            composition_root: launch.composition_root,
            project_composition_lock_hash: lock.composition_lock_sha256,
            launch_profile_hash: lock.launch_profiles_sha256,
            platform_capability_set_hash: launch
                .platform_capability_set
                .as_ref()
                .map(|capabilities| capabilities.canonical_hash),
            runtime_determinism_profile_hash: lock.runtime_determinism_profile_sha256,
            schema_registry_hash: lock.schema_registry_manifest_sha256,
            content_manifest_hash: lock.content_manifest_sha256,
            recovery_policy_hash: lock.recovery_policy_sha256,
            shutdown_policy_hash: lock.shutdown_policy_sha256,
            recovery_session_link_hash,
            presentation_target_kind: launch.presentation_target,
        },
    )?)
}

fn activate_selected_project(
    launch: &LaunchRequestV1,
) -> Result<ActivatedProjectV2, ApplicationError> {
    let project_root = match &launch.project {
        ProjectSelectionV1::Reference => launch.state_root.join(PROJECT_DIRECTORY),
        ProjectSelectionV1::PublishedStateRoot(root) => root.clone(),
    };
    let store = ContentStore::new(&project_root);
    if matches!(launch.project, ProjectSelectionV1::Reference) {
        // The embedded reference project is an exact engine-owned fixture.
        // Publishing its current generation on every activation atomically
        // advances legacy local packages after a schema/content-profile
        // change, while `ContentStore` still rejects corruption if the exact
        // generation already exists.
        let cooked = cook_project_v1(project_source_v2()?)?;
        store.publish(&cooked.publication()?)?;
    }
    Ok(activate_project(&store)?)
}

fn validate_launch(
    launch: &LaunchRequestV1,
    project: &ActivatedProjectV2,
) -> Result<(), ApplicationError> {
    project
        .validate()
        .map_err(next_project::ProjectActivationError::from)?;
    if launch
        .expected_project_lock
        .is_some_and(|expected| expected != project.composition_lock.composition_lock_sha256)
    {
        return Err(ApplicationError::ProjectLockMismatch);
    }
    if !project
        .composition_lock
        .allowed_presentation_targets
        .contains(&launch.presentation_target)
    {
        return Err(ApplicationError::ProjectTargetForbidden);
    }
    next_contracts::session::validate_root_target(
        launch.composition_root,
        launch.presentation_target,
    )?;
    validate_platform_capabilities(launch)?;
    Ok(())
}

fn validate_platform_capabilities(launch: &LaunchRequestV1) -> Result<(), ApplicationError> {
    let Some(capabilities) = launch.platform_capability_set.as_ref() else {
        return if launch.presentation_target == PresentationTargetKindV1::None {
            Ok(())
        } else {
            Err(ApplicationError::PlatformCapabilityRequired)
        };
    };
    capabilities.validate()?;
    if launch.presentation_target == PresentationTargetKindV1::None
        || !capabilities.required_capability_failures.is_empty()
        || !capabilities
            .presentation_target_kinds
            .contains(&launch.presentation_target)
    {
        return Err(ApplicationError::PlatformCapabilityRequired);
    }
    if launch.presentation_target == PresentationTargetKindV1::Interactive {
        let has_input_class = |expected: &str| {
            capabilities
                .input_classes
                .iter()
                .any(|input_class| input_class.as_str() == expected)
        };
        if !has_input_class("nextengine.input.keyboard")
            || !has_input_class("nextengine.input.mouse")
        {
            return Err(ApplicationError::PlatformCapabilityRequired);
        }
    }
    if launch.presentation_target == PresentationTargetKindV1::DisplaylessOffscreen
        && capabilities
            .presentation_target_kinds
            .contains(&PresentationTargetKindV1::Interactive)
    {
        return Err(ApplicationError::ProjectTargetForbidden);
    }
    Ok(())
}

fn load_session_generation_if_present(
    store: &SessionStore,
) -> Result<Option<next_assets::PublishedSessionGenerationV1>, ApplicationError> {
    if store
        .root()
        .join(next_assets::CONTENT_CURRENT_FILE)
        .exists()
    {
        Ok(Some(store.load_current()?))
    } else {
        Ok(None)
    }
}
