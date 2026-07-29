use std::collections::BTreeMap;
use std::fs;

use next_assets::{ContentStore, SaveStore, SessionObjectV1, SessionPublicationV1, SessionStore};
use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::ids::{
    ApplicationSessionId, CommandLedgerHash, ContentHash, SchemaId, content_hash_from_bytes,
};
use next_contracts::persistence::{SaveCompatibility, TickSettings};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::session::{
    ApplicationLifecycleEventV1, ApplicationLifecycleRequestV1, ApplicationSessionManifestBodyV1,
    ApplicationSessionManifestV1, ApplicationSessionStatusV1, BoundedDeadlineClassV1,
    CausalInputReferenceV1, CausalInputSourceKindV1, CloseSessionOperationJournalV1,
    CloseSessionProgressResultV1, CloseSessionProgressV1, CloseSessionReceiptV1,
    CloseSessionRequestV1, CloseSessionResultV1, FailureDispositionV1, FinalSavePolicyV1,
    FinalSaveReceiptV1, FinalSaveReservationBodyV1, LifecycleReasonKindV1, LifecycleReasonV1,
    PresentationTargetKindV1, RecoverySessionLinkV1, SessionFinalSaveLedgerEntryV1,
    can_close_after_failed_save, close_request_archive_ref,
};
use next_contracts::snapshot::WorldCheckpointV4;
use next_contracts::world::WorldStreamingSnapshotV1;
use next_project::{activate_project, cook_project_v1};
use next_reference_game::{ReferenceRunOutcomeV1, project_source_v2, run_reference_game};
use next_runtime::{
    ApplicationSessionMachine, SessionStatePublicationPlanV1, SessionTransitionPlanV1,
    SessionTransitionReferencesV1,
};

use crate::close::{ApplicationCloseOutcomeV1, CloseExecutionOptionsV1, FinalSaveAttemptFailureV1};
use crate::durable::{DurableApplicationSnapshotV1, DurableCloseOperationV1, DurableLedgerV1};
use crate::{ApplicationError, LaunchRequestV1, ProjectSelectionV1};

mod close_flow;
mod identity;

use identity::*;

const PROJECT_DIRECTORY: &str = "project";
const SESSION_DIRECTORY: &str = "sessions";
const SAVE_DIRECTORY: &str = "saves";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationRunOutcomeV1 {
    pub session_id: ApplicationSessionId,
    pub project_composition_lock_hash: ContentHash,
    pub ticks: u64,
    pub events: u64,
    pub rpg_events: u64,
    pub authoritative_revision: u64,
    pub authoritative_state_root: ContentHash,
    pub command_archive_root: ContentHash,
    pub command_identity_index_root: ContentHash,
    pub command_ledger_hash: CommandLedgerHash,
    pub presentation_input_count: u64,
    pub presentation_snapshot: Option<next_contracts::presentation::PresentationSnapshotV2>,
}

#[derive(Clone)]
struct PreparedRunV1 {
    checkpoint: WorldCheckpointV4,
    streaming: WorldStreamingSnapshotV1,
    summary: ApplicationRunOutcomeV1,
}

pub struct ApplicationCoordinator {
    launch: LaunchRequestV1,
    activated_project: ActivatedProjectV2,
    session_store: SessionStore,
    save_store: SaveStore,
    machine: ApplicationSessionMachine,
    durable: DurableApplicationSnapshotV1,
    current_generation: ContentHash,
    objects: BTreeMap<ContentHash, Vec<u8>>,
    prepared_run: Option<PreparedRunV1>,
    #[cfg(test)]
    pause_after_save_commit: bool,
}

impl ApplicationCoordinator {
    pub fn launch_or_resume(launch: LaunchRequestV1) -> Result<Self, ApplicationError> {
        match Self::launch(launch.clone()) {
            Ok(application) => Ok(application),
            Err(ApplicationError::SessionAlreadyLive) => {
                let mut resumed = Self::resume(launch.clone())?;
                if matches!(
                    resumed.state().state,
                    ApplicationSessionStatusV1::Quiescing | ApplicationSessionStatusV1::Finalizing
                ) {
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
            prepared_run: None,
            #[cfg(test)]
            pause_after_save_commit: false,
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
        let durable = DurableApplicationSnapshotV1::from_canonical_bytes(&published.snapshot)?;
        if durable.manifest.body.project_composition_lock_hash
            != activated_project.composition_lock.composition_lock_sha256
            || durable.manifest.body.composition_root != launch.composition_root
            || durable.manifest.body.presentation_target_kind != launch.presentation_target
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
        let machine = ApplicationSessionMachine::restore(
            durable.manifest.clone(),
            durable.state.clone(),
            Vec::new(),
        )?;
        let mut coordinator = Self {
            save_store: SaveStore::new(launch.state_root.join(SAVE_DIRECTORY)),
            launch,
            activated_project,
            session_store,
            machine,
            durable,
            current_generation: published.generation_id,
            objects: published.objects,
            prepared_run: None,
            #[cfg(test)]
            pause_after_save_commit: false,
        };
        coordinator.finish_launch()?;
        Ok(coordinator)
    }

    #[must_use]
    pub fn state(&self) -> &next_contracts::session::ApplicationSessionStateV1 {
        self.machine.state()
    }

    #[must_use]
    pub fn manifest(&self) -> &ApplicationSessionManifestV1 {
        self.machine.manifest()
    }

    #[must_use]
    pub fn activated_project(&self) -> &ActivatedProjectV2 {
        &self.activated_project
    }

    #[must_use]
    pub const fn current_session_generation(&self) -> ContentHash {
        self.current_generation
    }

    pub fn run_reference_game(
        &mut self,
        include_interaction: bool,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        if self.machine.state().state != ApplicationSessionStatusV1::Active {
            return Err(ApplicationError::CloseStateInvalid);
        }
        let prepared = prepare_reference_run(
            self.machine.state().session_id,
            self.activated_project.clone(),
            include_interaction,
            self.launch.presentation_target,
        )?;
        self.record_prepared_run(&prepared)?;
        let plan = self
            .machine
            .plan_state_publication(Some(prepared.summary.authoritative_revision), None)?;
        self.publish_state_plan(plan)?;
        let summary = prepared.summary.clone();
        self.prepared_run = Some(prepared);
        Ok(summary)
    }

    pub fn close_request(
        &self,
        deadline: BoundedDeadlineClassV1,
    ) -> Result<CloseSessionRequestV1, ApplicationError> {
        self.build_close_request(
            self.machine.state().revision,
            self.machine.state().state,
            deadline,
        )
    }

    pub fn close(
        &mut self,
        options: CloseExecutionOptionsV1,
    ) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
        let request = if let Some(close) = &self.durable.close {
            let request = self.build_close_request(
                close.starting_session_revision,
                close.starting_session_state,
                BoundedDeadlineClassV1::Standard,
            )?;
            if request.canonical_close_request_hash != close.canonical_close_request_hash
                || request.canonical_bytes()? != close.canonical_close_request_bytes
            {
                return Err(ApplicationError::CloseJournalInvalid);
            }
            request
        } else {
            self.close_request(BoundedDeadlineClassV1::Standard)?
        };
        self.close_with_request(request, options)
    }

    pub fn close_with_request(
        &mut self,
        request: CloseSessionRequestV1,
        options: CloseExecutionOptionsV1,
    ) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
        request.validate()?;
        self.register_or_validate_close(&request, options.last_safe_generation_hash)?;
        if self.machine.state().state == ApplicationSessionStatusV1::Closed {
            return self.closed_outcome();
        }
        self.advance_to_finalizing(&request)?;
        self.execute_final_save_attempt(&request, options)
    }

    pub fn recover_required_save_failure(mut self) -> Result<Self, ApplicationError> {
        let prior_close = self
            .durable
            .close
            .clone()
            .ok_or(ApplicationError::RecoveryIncompatible)?;
        let ledger = prior_close
            .ledger
            .as_ref()
            .ok_or(ApplicationError::RecoveryIncompatible)?;
        let lock = self.activated_project.composition_lock.clone();
        if self.machine.state().state != ApplicationSessionStatusV1::Finalizing
            || ledger.status != next_contracts::session::FinalSaveLedgerStatusV1::Failed
            || prior_close.failure_disposition != FailureDispositionV1::RequireFinalSave
            || !lock.recovery_permit_required_save
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        self.ensure_prepared_run()?;
        let checkpoint = &self
            .prepared_run
            .as_ref()
            .ok_or(ApplicationError::NoRunOutcome)?
            .checkpoint;
        let compatibility = save_compatibility(&self.activated_project, checkpoint)?;
        let last_safe = self.save_store.load_latest(&compatibility)?;
        let (last_safe_generation_hash, last_safe_manifest_hash) =
            save_identity(&last_safe.image.manifest)?;
        let next_sequence = self
            .durable
            .store_sequence
            .checked_add(1)
            .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        let new_session_id = derive_session_id(
            lock.composition_lock_sha256,
            self.launch.composition_root,
            next_sequence,
        );
        let recovery_link = RecoverySessionLinkV1::new(
            self.machine.state().session_id,
            self.machine.state().canonical_hash,
            self.machine.state().revision,
            prior_close.close_request_id,
            prior_close.canonical_close_request_hash,
            ledger.entry_hash,
            lock.composition_lock_sha256,
            self.machine.manifest().canonical_hash,
            last_safe_generation_hash,
            last_safe_manifest_hash,
            new_session_id,
            SchemaId::new("nextengine.session.recovery.required-save")?,
        )?;
        recovery_link.validate()?;
        let manifest = session_manifest(
            &self.launch,
            &self.activated_project,
            new_session_id,
            Some(recovery_link.canonical_hash),
        )?;
        self.machine = ApplicationSessionMachine::new(manifest.clone())?;
        self.durable = DurableApplicationSnapshotV1 {
            store_sequence: next_sequence,
            manifest: manifest.clone(),
            state: self.machine.state().clone(),
            close: None,
        };
        self.objects.clear();
        self.record_object(manifest.to_jcs_bytes());
        self.record_object(recovery_link.canonical_hash.as_bytes().to_vec());
        let prior_generation = self.current_generation;
        self.publish_current(
            Some(prior_generation),
            Some(new_session_id),
            Some(recovery_link.prior_session_id),
        )?;
        self.prepared_run = None;
        self.finish_launch()?;
        let plan = self
            .machine
            .plan_state_publication(None, Some(last_safe_generation_hash))?;
        self.publish_state_plan(plan)?;
        Ok(self)
    }

    fn finish_launch(&mut self) -> Result<(), ApplicationError> {
        loop {
            let (target, reason, code, policy) = match self.machine.state().state {
                ApplicationSessionStatusV1::Created => (
                    ApplicationSessionStatusV1::CompositionStaged,
                    LifecycleReasonKindV1::CompositionReady,
                    "nextengine.session.composition-ready",
                    self.activated_project
                        .composition_lock
                        .launch_profiles_sha256,
                ),
                ApplicationSessionStatusV1::CompositionStaged => (
                    ApplicationSessionStatusV1::RuntimeStaged,
                    LifecycleReasonKindV1::RuntimeReady,
                    "nextengine.session.runtime-staged",
                    self.activated_project
                        .composition_lock
                        .runtime_determinism_profile_sha256,
                ),
                ApplicationSessionStatusV1::RuntimeStaged => (
                    ApplicationSessionStatusV1::Active,
                    LifecycleReasonKindV1::RuntimeReady,
                    "nextengine.session.active",
                    self.activated_project
                        .composition_lock
                        .runtime_determinism_profile_sha256,
                ),
                _ => break,
            };
            let request = self.lifecycle_request(target, reason, code, policy)?;
            let references = SessionTransitionReferencesV1 {
                activation_receipt_hash: (target == ApplicationSessionStatusV1::Active).then_some(
                    self.activated_project
                        .composition_lock
                        .composition_lock_sha256,
                ),
                active_runtime_revision: (target == ApplicationSessionStatusV1::Active)
                    .then_some(0),
                ..SessionTransitionReferencesV1::default()
            };
            self.publish_transition(request, references, self.durable.close.clone())?;
        }
        Ok(())
    }

    fn lifecycle_request(
        &self,
        target: ApplicationSessionStatusV1,
        reason_kind: LifecycleReasonKindV1,
        reason_code: &str,
        policy_hash: ContentHash,
    ) -> Result<ApplicationLifecycleRequestV1, ApplicationError> {
        let state = self.machine.state();
        let causal_hash = domain_hash(
            b"nextengine.session-lifecycle-cause.v1\0",
            &[
                state.session_id.as_bytes(),
                &state.revision.to_le_bytes(),
                &[target as u8],
            ],
        );
        let request_id = derive_request_id(state.session_id, state.revision, target, causal_hash);
        Ok(ApplicationLifecycleRequestV1::new(
            request_id,
            state.session_id,
            state.revision,
            state.state,
            target,
            LifecycleReasonV1 {
                kind: reason_kind,
                reason_code: SchemaId::new(reason_code)?,
            },
            policy_hash,
            CausalInputReferenceV1 {
                source_kind: CausalInputSourceKindV1::SystemPolicy,
                canonical_hash: causal_hash,
            },
        )?)
    }

    fn build_close_request(
        &self,
        starting_revision: u64,
        starting_state: ApplicationSessionStatusV1,
        deadline: BoundedDeadlineClassV1,
    ) -> Result<CloseSessionRequestV1, ApplicationError> {
        let session_id = self.machine.state().session_id;
        let causal_hash = domain_hash(
            b"nextengine.close-request-cause.v1\0",
            &[session_id.as_bytes(), &starting_revision.to_le_bytes()],
        );
        let close_request_id = derive_close_request_id(session_id, starting_revision, causal_hash);
        Ok(CloseSessionRequestV1::new(
            close_request_id,
            session_id,
            starting_revision,
            starting_state,
            self.activated_project
                .composition_lock
                .shutdown_policy_sha256,
            FinalSavePolicyV1::Always,
            deadline,
            LifecycleReasonV1 {
                kind: LifecycleReasonKindV1::UserCloseRequested,
                reason_code: SchemaId::new("nextengine.session.close-requested")?,
            },
            CausalInputReferenceV1 {
                source_kind: CausalInputSourceKindV1::SystemPolicy,
                canonical_hash: causal_hash,
            },
        )?)
    }

    fn publish_transition(
        &mut self,
        request: ApplicationLifecycleRequestV1,
        references: SessionTransitionReferencesV1,
        close: Option<DurableCloseOperationV1>,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        let plan = self.machine.plan_transition(request, references)?;
        match plan {
            SessionTransitionPlanV1::ExactRetry { event, .. } => Ok(event),
            plan @ SessionTransitionPlanV1::Publish { .. } => {
                let close = close.ok_or_else(|| {
                    if self.durable.close.is_none() {
                        ApplicationError::CloseJournalInvalid
                    } else {
                        ApplicationError::DurableSnapshotInvalid
                    }
                });
                if self.durable.close.is_none() {
                    self.publish_planned_transition_optional(plan, None)
                } else {
                    self.publish_planned_transition(plan, close?)
                }
            }
        }
    }

    fn publish_planned_transition(
        &mut self,
        plan: SessionTransitionPlanV1,
        close: DurableCloseOperationV1,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        self.publish_planned_transition_optional(plan, Some(close))
    }

    fn publish_planned_transition_optional(
        &mut self,
        plan: SessionTransitionPlanV1,
        close: Option<DurableCloseOperationV1>,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        let SessionTransitionPlanV1::Publish {
            request,
            event,
            next_state,
            ..
        } = &plan
        else {
            return Err(ApplicationError::DurableSnapshotInvalid);
        };
        let request_bytes = request.canonical_bytes();
        let expected_event = event.clone();
        let next_state = next_state.as_ref().clone();
        self.record_object(request_bytes);
        self.record_object(expected_event.canonical_hash.as_bytes().to_vec());
        self.durable.state = next_state.clone();
        self.durable.close = close;
        let live = (next_state.state != ApplicationSessionStatusV1::Closed)
            .then_some(next_state.session_id);
        self.publish_current(Some(self.current_generation), live, None)?;
        let committed = self.machine.commit(plan);
        debug_assert_eq!(committed, expected_event);
        Ok(committed)
    }

    fn publish_state_plan(
        &mut self,
        plan: SessionStatePublicationPlanV1,
    ) -> Result<(), ApplicationError> {
        self.durable.state = plan.next_state.clone();
        let live = (plan.next_state.state != ApplicationSessionStatusV1::Closed)
            .then_some(plan.next_state.session_id);
        self.publish_current(Some(self.current_generation), live, None)?;
        self.machine.commit_state_publication(plan);
        Ok(())
    }

    fn publish_current(
        &mut self,
        expected_previous: Option<ContentHash>,
        live_session_id: Option<ApplicationSessionId>,
        superseded_session_id: Option<ApplicationSessionId>,
    ) -> Result<(), ApplicationError> {
        if expected_previous.is_some() {
            self.durable.store_sequence = self
                .durable
                .store_sequence
                .checked_add(1)
                .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        }
        let snapshot = self.durable.canonical_bytes()?;
        let objects = self
            .objects
            .values()
            .cloned()
            .map(SessionObjectV1::new)
            .collect();
        let publication = SessionPublicationV1::new(
            self.durable.store_sequence,
            self.durable.manifest.body.project_composition_lock_hash,
            live_session_id,
            expected_previous,
            superseded_session_id,
            snapshot,
            objects,
        )?;
        self.current_generation = self.session_store.publish(&publication)?;
        Ok(())
    }

    fn record_object(&mut self, bytes: Vec<u8>) {
        let object = SessionObjectV1::new(bytes);
        self.objects.insert(object.content_hash, object.bytes);
    }

    fn record_prepared_run(&mut self, prepared: &PreparedRunV1) -> Result<(), ApplicationError> {
        self.record_object(prepared.checkpoint.runtime_snapshot.canonical_bytes()?);
        self.record_object(prepared.checkpoint.rpg_snapshot.canonical_bytes()?);
        self.record_object(prepared.checkpoint.physics_checkpoint.canonical_bytes()?);
        self.record_object(prepared.streaming.canonical_bytes()?);
        Ok(())
    }

    fn ensure_prepared_run(&mut self) -> Result<(), ApplicationError> {
        if self.prepared_run.is_some() {
            return Ok(());
        }
        let prepared = prepare_reference_run(
            self.machine.state().session_id,
            self.activated_project.clone(),
            true,
            self.launch.presentation_target,
        )?;
        if self
            .machine
            .state()
            .active_runtime_revision
            .is_some_and(|revision| revision != prepared.summary.authoritative_revision)
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        self.record_prepared_run(&prepared)?;
        self.prepared_run = Some(prepared);
        Ok(())
    }

    #[cfg(test)]
    fn inject_pause_after_save_commit(&mut self) {
        self.pause_after_save_commit = true;
    }
}

fn activate_selected_project(
    launch: &LaunchRequestV1,
) -> Result<ActivatedProjectV2, ApplicationError> {
    let project_root = match &launch.project {
        ProjectSelectionV1::Reference => launch.state_root.join(PROJECT_DIRECTORY),
        ProjectSelectionV1::PublishedStateRoot(root) => root.clone(),
    };
    let store = ContentStore::new(&project_root);
    if matches!(launch.project, ProjectSelectionV1::Reference)
        && !project_root
            .join(next_assets::CONTENT_CURRENT_FILE)
            .exists()
    {
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
    Ok(())
}

fn session_manifest(
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
            platform_capability_set_hash: (launch.presentation_target
                == PresentationTargetKindV1::Interactive)
                .then_some(lock.platform_capability_profile_sha256),
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

fn prepare_reference_run(
    session_id: ApplicationSessionId,
    project: ActivatedProjectV2,
    include_interaction: bool,
    presentation_target: PresentationTargetKindV1,
) -> Result<PreparedRunV1, ApplicationError> {
    let run: ReferenceRunOutcomeV1 = run_reference_game(project, include_interaction)?;
    let checkpoint = run.runtime.world_checkpoint()?;
    let presentation_snapshot = if presentation_target == PresentationTargetKindV1::None {
        None
    } else {
        let mut extractor = next_presentation::PresentationExtractorV1::new(
            run.project_composition_lock_hash,
            next_contracts::project::domain_hash(
                "nextengine.presentation-profile.b0.v1",
                b"sdr-reference-no-optional-features",
            ),
            8,
        )?;
        Some(
            extractor
                .extract(
                    run.ticks,
                    run.project_composition_lock_hash,
                    run.content_manifest_hash,
                    run.runtime.physics_snapshot(),
                    &run.presentation_bindings,
                )?
                .clone(),
        )
    };
    let presentation_input_count = u64::try_from(run.presentation_bindings.len())
        .map_err(|_| ApplicationError::DurableSnapshotInvalid)?;
    let authoritative_state_root =
        next_contracts::snapshot::world_checkpoint_with_streaming_v1_state_root(
            &checkpoint.runtime_snapshot,
            &checkpoint.rpg_snapshot,
            &checkpoint.physics_checkpoint,
            &run.world_streaming_snapshot,
        )?;
    let summary = ApplicationRunOutcomeV1 {
        session_id,
        project_composition_lock_hash: run.project_composition_lock_hash,
        ticks: run.ticks,
        events: run.events,
        rpg_events: run.rpg_events,
        authoritative_revision: checkpoint.runtime_snapshot.authoritative_revision,
        authoritative_state_root: ContentHash::from_bytes(*authoritative_state_root.as_bytes()),
        command_archive_root: checkpoint
            .runtime_snapshot
            .command_ledger
            .body_archive
            .archive_root,
        command_identity_index_root: checkpoint
            .runtime_snapshot
            .command_ledger
            .identity_index
            .index_root,
        command_ledger_hash: checkpoint.runtime_snapshot.command_ledger_hash()?,
        presentation_input_count,
        presentation_snapshot,
    };
    Ok(PreparedRunV1 {
        checkpoint,
        streaming: run.world_streaming_snapshot,
        summary,
    })
}

fn reservation(
    request: &CloseSessionRequestV1,
    close: &DurableCloseOperationV1,
    project: &ActivatedProjectV2,
) -> Result<FinalSaveReservationBodyV1, ApplicationError> {
    Ok(FinalSaveReservationBodyV1::new(
        request.session_id,
        request.close_request_id,
        request.canonical_close_request_hash,
        close.close_request_archive_ref,
        request.starting_session_revision,
        request.final_save_policy,
        request.shutdown_policy_hash,
        project.composition_lock.shutdown_maximum_attempts,
    )?)
}

fn rebuild_retryable_ledger(
    request: &CloseSessionRequestV1,
    close: &DurableCloseOperationV1,
    project: &ActivatedProjectV2,
) -> Result<SessionFinalSaveLedgerEntryV1, ApplicationError> {
    let durable = close
        .ledger
        .as_ref()
        .ok_or(ApplicationError::CloseJournalInvalid)?;
    if matches!(
        durable.status,
        next_contracts::session::FinalSaveLedgerStatusV1::Committed
            | next_contracts::session::FinalSaveLedgerStatusV1::Failed
    ) {
        return Err(ApplicationError::CloseStateInvalid);
    }
    let mut ledger =
        SessionFinalSaveLedgerEntryV1::reserved(&reservation(request, close, project)?)?;
    let failure = durable
        .last_failure_code
        .clone()
        .unwrap_or(SchemaId::new("nextengine.session.final-save-retry")?);
    for _ in 0..durable.attempt_count {
        ledger = ledger.retry_pending(failure.clone())?;
    }
    if ledger.entry_hash != durable.entry_hash
        || ledger.reservation_hash != durable.reservation_hash
    {
        return Err(ApplicationError::CloseJournalInvalid);
    }
    Ok(ledger)
}

fn rebuild_journal(
    request: &CloseSessionRequestV1,
    close: &DurableCloseOperationV1,
) -> Result<CloseSessionOperationJournalV1, ApplicationError> {
    let registered =
        CloseSessionOperationJournalV1::registered(request, close.close_request_archive_ref)?;
    let journal =
        if close.stage == next_contracts::session::CloseSessionOperationStageV1::Registered {
            registered
        } else {
            registered.with_stage(
                request,
                close.stage,
                close.quiesce_event_hash,
                close.finalizing_event_hash,
                close.ledger.as_ref().map(|ledger| ledger.entry_hash),
                close.close_session_receipt_hash,
            )?
        };
    if journal.canonical_hash != close.operation_journal_hash {
        return Err(ApplicationError::CloseJournalInvalid);
    }
    Ok(journal)
}

fn durable_ledger(ledger: &SessionFinalSaveLedgerEntryV1) -> DurableLedgerV1 {
    DurableLedgerV1 {
        status: ledger.status,
        attempt_count: ledger.attempt_count,
        maximum_attempts: ledger.maximum_attempts,
        last_failure_code: ledger.last_failure_code.clone(),
        reservation_hash: ledger.reservation_hash,
        entry_hash: ledger.entry_hash,
        save_generation_hash: ledger.save_generation_hash,
        final_save_receipt_hash: ledger.final_save_receipt_hash,
    }
}

fn progress(
    close: &DurableCloseOperationV1,
    revision: u64,
    result: CloseSessionProgressResultV1,
) -> Result<CloseSessionProgressV1, ApplicationError> {
    let ledger = close
        .ledger
        .as_ref()
        .ok_or(ApplicationError::CloseJournalInvalid)?;
    Ok(CloseSessionProgressV1::new(
        close_request_session_id(close)?,
        close.close_request_id,
        close.canonical_close_request_hash,
        close.operation_journal_hash,
        revision,
        ledger.entry_hash,
        ledger.attempt_count,
        result,
    )?)
}

fn close_request_session_id(
    close: &DurableCloseOperationV1,
) -> Result<ApplicationSessionId, ApplicationError> {
    let decoded = next_contracts::canonical::decode_canonical_segment(
        &close.canonical_close_request_bytes,
        CanonicalDecodeLimits::default(),
    )?;
    let field = decoded
        .field(3)
        .ok_or(ApplicationError::CloseJournalInvalid)?;
    Ok(ApplicationSessionId::from_bytes(
        field
            .payload
            .as_slice()
            .try_into()
            .map_err(|_| ApplicationError::CloseJournalInvalid)?,
    ))
}

fn save_compatibility(
    project: &ActivatedProjectV2,
    checkpoint: &WorldCheckpointV4,
) -> Result<SaveCompatibility, ApplicationError> {
    let profile = checkpoint.runtime_snapshot.tick_rate_profile;
    Ok(SaveCompatibility {
        engine_build_hash: project.composition_lock.runtime_determinism_profile_sha256,
        game_build_hash: project.composition_lock.project_manifest_sha256,
        project_id: SchemaId::new(project.composition_lock.project_id.as_str())?,
        schema_registry_hash: project.composition_lock.schema_registry_manifest_sha256,
        content_manifest_hash: project.composition_lock.content_manifest_sha256,
        mechanics_lock_hash: project.composition_lock.mechanics_lock_sha256,
        tick_settings: TickSettings {
            gameplay_hz: profile.gameplay_hz,
            physics_hz: profile.physics_hz(),
            motor_hz: profile.physics_hz() / profile.motor_period_physics_substeps,
        },
        loaded_chunk_revisions: Vec::new(),
        rng_stream_states: Vec::new(),
        physical_bindings: Vec::new(),
        policy_state_schemas: Vec::new(),
        plugin_script_bindings: Vec::new(),
    })
}

fn save_identity(
    manifest: &next_contracts::persistence::SaveManifestV2,
) -> Result<(ContentHash, ContentHash), ApplicationError> {
    let bytes = manifest.to_jcs_bytes()?;
    let manifest_hash = content_hash_from_bytes(sha256(&bytes));
    let generation_hash = domain_hash(
        b"nextengine.save-generation.v1\0",
        &[&manifest.generation.to_le_bytes(), manifest_hash.as_bytes()],
    );
    Ok((generation_hash, manifest_hash))
}

fn planned_event(
    plan: &SessionTransitionPlanV1,
) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
    match plan {
        SessionTransitionPlanV1::Publish { event, .. }
        | SessionTransitionPlanV1::ExactRetry { event, .. } => Ok(event.clone()),
    }
}

#[cfg(test)]
mod tests;
