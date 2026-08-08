use std::fs;

use next_assets::{ContentStore, SaveStore, SessionStore};
use next_contracts::ids::{ApplicationSessionId, ContentHash};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::session::{
    ApplicationSessionManifestBodyV2, ApplicationSessionManifestV2, ApplicationSessionStatusV1,
    PresentationTargetKindV1,
};
use next_project::{activate_project, cook_project_v1};
use next_reference_game::project_source_v2;
use next_runtime::ApplicationSessionMachine;

use crate::durable::DurableApplicationSnapshotV4;
use crate::{ApplicationError, LaunchRequestV1, ProjectSelectionV1};

use super::identity::derive_session_id;
use super::{ApplicationCoordinator, PROJECT_DIRECTORY, SAVE_DIRECTORY, SESSION_DIRECTORY};

impl ApplicationCoordinator {
    pub fn launch_or_resume(launch: LaunchRequestV1) -> Result<Self, ApplicationError> {
        match Self::launch(launch.clone()) {
            Ok(application) => Ok(application),
            Err(ApplicationError::SessionAlreadyLive) => {
                let mut resumed = Self::resume(launch.clone())?;
                if resumed.durable.close_journal.is_some()
                    && resumed.state().state != ApplicationSessionStatusV1::Closed
                {
                    resumed.close()?;
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
        if let Some(current) = current.as_ref() {
            let snapshot = DurableApplicationSnapshotV4::from_canonical_bytes(&current.snapshot)?;
            if snapshot.state.state != ApplicationSessionStatusV1::Closed {
                return Err(ApplicationError::SessionAlreadyLive);
            }
        }
        let prior_sequence = current.as_ref().map(|value| value.sequence);
        let session_sequence = prior_sequence.map_or(0, |value| value.saturating_add(1));
        let session_id = derive_session_id(
            activated_project.composition_lock.composition_lock_sha256,
            launch.composition_root,
            session_sequence,
        );
        let manifest = session_manifest(&launch, &activated_project, session_id)?;
        let machine = ApplicationSessionMachine::new(manifest.clone())?;
        let durable = DurableApplicationSnapshotV4 {
            store_sequence: prior_sequence.unwrap_or(0),
            manifest: manifest.clone(),
            state: machine.state().clone(),
            last_lifecycle: None,
            close_request: None,
            close_journal: None,
            close_receipt: None,
            prepared_save_image: None,
        };
        let mut coordinator = Self {
            save_store: SaveStore::new(launch.state_root.join(SAVE_DIRECTORY)),
            launch,
            activated_project,
            session_store,
            machine,
            durable,
            current_generation: ContentHash::default(),
            prepared_run: None,
            live_run: None,
            platform_host: None,
        };
        coordinator.publish_current(current.as_ref().map(|value| value.generation_id))?;
        coordinator.finish_launch()?;
        Ok(coordinator)
    }

    pub fn resume(launch: LaunchRequestV1) -> Result<Self, ApplicationError> {
        fs::create_dir_all(&launch.state_root)?;
        let activated_project = activate_selected_project(&launch)?;
        validate_launch(&launch, &activated_project)?;
        let session_store = SessionStore::new(launch.state_root.join(SESSION_DIRECTORY));
        let published = session_store.load_current()?;
        let durable = DurableApplicationSnapshotV4::from_canonical_bytes(&published.snapshot)?;
        if durable.store_sequence != published.sequence
            || durable.manifest.body.project_composition_lock_hash
                != activated_project.composition_lock.composition_lock_sha256
            || durable.manifest.body.composition_root != launch.composition_root
            || durable.manifest.body.presentation_target_kind != launch.presentation_target
            || durable.manifest.body.platform_capability_set_hash
                != launch
                    .platform_capability_set
                    .as_ref()
                    .map(|value| value.canonical_hash)
            || durable.state.state == ApplicationSessionStatusV1::Closed
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        let machine = ApplicationSessionMachine::restore(
            durable.manifest.clone(),
            durable.state.clone(),
            durable.last_lifecycle.clone(),
        )?;
        let mut coordinator = Self {
            save_store: SaveStore::new(launch.state_root.join(SAVE_DIRECTORY)),
            launch,
            activated_project,
            session_store,
            machine,
            durable,
            current_generation: published.generation_id,
            prepared_run: None,
            live_run: None,
            platform_host: None,
        };
        coordinator.restore_after_crash()?;
        Ok(coordinator)
    }
}

pub(super) fn session_manifest(
    launch: &LaunchRequestV1,
    project: &ActivatedProjectV2,
    session_id: ApplicationSessionId,
) -> Result<ApplicationSessionManifestV2, ApplicationError> {
    let lock = &project.composition_lock;
    Ok(ApplicationSessionManifestV2::new(
        ApplicationSessionManifestBodyV2 {
            session_id,
            composition_root: launch.composition_root,
            project_composition_lock_hash: lock.composition_lock_sha256,
            launch_profile_hash: lock.launch_profiles_sha256,
            platform_capability_set_hash: launch
                .platform_capability_set
                .as_ref()
                .map(|value| value.canonical_hash),
            runtime_determinism_profile_hash: lock.runtime_determinism_profile_sha256,
            schema_registry_hash: lock.schema_registry_manifest_sha256,
            content_manifest_hash: lock.content_manifest_sha256,
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
    validate_platform_capabilities(launch)
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
        let has = |expected: &str| {
            capabilities
                .input_classes
                .iter()
                .any(|value| value.as_str() == expected)
        };
        if !has("nextengine.input.keyboard") || !has("nextengine.input.mouse") {
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
) -> Result<Option<next_assets::PublishedSessionGenerationV2>, ApplicationError> {
    if store.current_path().exists() {
        Ok(Some(store.load_current()?))
    } else {
        Ok(None)
    }
}
