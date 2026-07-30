use std::collections::BTreeMap;

use next_assets::{SessionObjectV1, SessionPublicationV1};
use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::ids::{ApplicationSessionId, ContentHash, SchemaId};
use next_contracts::platform::{PlatformContractError, PlatformEventKindV1, PlatformEventV1};
use next_contracts::session::{
    ApplicationLifecycleEventV1, ApplicationLifecycleRequestV1, ApplicationSessionStatusV1,
    CausalInputReferenceV1, CausalInputSourceKindV1, LifecycleReasonKindV1, LifecycleReasonV1,
};
use next_runtime::{
    ArchivedLifecycleRequestV1, SessionStatePublicationPlanV1, SessionTransitionPlanV1,
    SessionTransitionReferencesV1,
};

use crate::ApplicationError;
use crate::durable::{
    DurableApplicationSnapshotV1, DurableCloseOperationV1, DurableLifecycleArchiveEntryV1,
    LIFECYCLE_ARCHIVE_MAX_ENTRIES,
};

use super::ApplicationCoordinator;
use super::identity::{derive_request_id, domain_hash};
use super::run::PreparedRunV1;

pub(super) const PLATFORM_LIFECYCLE_REQUEST_BUDGET: usize = 1_024;

impl ApplicationCoordinator {
    pub fn suspend_from_platform_event(
        &mut self,
        event: &PlatformEventV1,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        if event.kind != PlatformEventKindV1::SuspendRequested {
            return Err(PlatformContractError::KindPayloadMismatch.into());
        }
        self.with_platform_event_admission(
            std::slice::from_ref(event),
            std::slice::from_ref(event),
            |coordinator| coordinator.suspend_from_admitted_platform_event(event),
        )
    }

    pub fn resume_from_platform_event(
        &mut self,
        event: &PlatformEventV1,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        if event.kind != PlatformEventKindV1::ResumeRequested {
            return Err(PlatformContractError::KindPayloadMismatch.into());
        }
        self.with_platform_event_admission(
            std::slice::from_ref(event),
            std::slice::from_ref(event),
            |coordinator| coordinator.resume_from_admitted_platform_event(event),
        )
    }

    pub(crate) fn suspend_from_admitted_platform_event(
        &mut self,
        event: &PlatformEventV1,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        let prepared = self
            .live_run
            .is_some()
            .then(|| self.prepared_run.clone())
            .flatten();
        self.transition_from_platform_event(
            event,
            PlatformEventKindV1::SuspendRequested,
            ApplicationSessionStatusV1::Suspended,
            LifecycleReasonKindV1::SuspendRequested,
            "nextengine.session.suspend-requested",
            prepared.as_ref(),
        )
    }

    pub(crate) fn resume_from_admitted_platform_event(
        &mut self,
        event: &PlatformEventV1,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        self.transition_from_platform_event(
            event,
            PlatformEventKindV1::ResumeRequested,
            ApplicationSessionStatusV1::Active,
            LifecycleReasonKindV1::ResumeRequested,
            "nextengine.session.resume-requested",
            None,
        )
    }

    pub(super) fn suspend_from_admitted_platform_event_with_prepared_run(
        &mut self,
        event: &PlatformEventV1,
        prepared: &PreparedRunV1,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        self.transition_from_platform_event(
            event,
            PlatformEventKindV1::SuspendRequested,
            ApplicationSessionStatusV1::Suspended,
            LifecycleReasonKindV1::SuspendRequested,
            "nextengine.session.suspend-requested",
            Some(prepared),
        )
    }

    fn transition_from_platform_event(
        &mut self,
        event: &PlatformEventV1,
        expected_kind: PlatformEventKindV1,
        target: ApplicationSessionStatusV1,
        reason_kind: LifecycleReasonKindV1,
        reason_code: &str,
        prepared: Option<&PreparedRunV1>,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        event.validate()?;
        if event.kind != expected_kind {
            return Err(PlatformContractError::KindPayloadMismatch.into());
        }
        let request = self.platform_lifecycle_request(event, target, reason_kind, reason_code)?;
        let active_runtime_revision = prepared
            .map(|run| run.summary.authoritative_revision)
            .or(self.machine.state().active_runtime_revision);
        self.publish_transition_optional_prepared(
            request,
            SessionTransitionReferencesV1 {
                active_runtime_revision,
                ..SessionTransitionReferencesV1::default()
            },
            self.durable.close.clone(),
            prepared,
        )
    }

    fn platform_lifecycle_request(
        &self,
        event: &PlatformEventV1,
        target: ApplicationSessionStatusV1,
        reason_kind: LifecycleReasonKindV1,
        reason_code: &str,
    ) -> Result<ApplicationLifecycleRequestV1, ApplicationError> {
        let session_id = self.machine.state().session_id;
        let archived_requests = self.machine.archived_requests();
        let prior_request = archived_requests
            .iter()
            .find(|archived| {
                archived.event.to_state == target
                    && derive_request_id(
                        session_id,
                        archived.event.before_revision,
                        target,
                        event.platform_event_id,
                    ) == archived.request_id
            })
            .cloned();
        if prior_request.is_none() {
            let mut platform_request_count = 0_usize;
            for archived in &archived_requests {
                let request = ApplicationLifecycleRequestV1::from_jcs_bytes(
                    &archived.canonical_request_bytes,
                    CanonicalDecodeLimits::default(),
                )?;
                if request.causal_input_reference.source_kind
                    == CausalInputSourceKindV1::PlatformEvent
                {
                    platform_request_count = platform_request_count
                        .checked_add(1)
                        .ok_or(ApplicationError::LifecycleArchiveBudgetExceeded)?;
                }
            }
            ensure_platform_lifecycle_budget(platform_request_count)?;
        }
        let (expected_revision, expected_state) = prior_request.map_or_else(
            || (self.machine.state().revision, self.machine.state().state),
            |archived| (archived.event.before_revision, archived.event.from_state),
        );
        let request_id = derive_request_id(
            session_id,
            expected_revision,
            target,
            event.platform_event_id,
        );
        Ok(ApplicationLifecycleRequestV1::new(
            request_id,
            session_id,
            expected_revision,
            expected_state,
            target,
            LifecycleReasonV1 {
                kind: reason_kind,
                reason_code: SchemaId::new(reason_code)?,
            },
            self.activated_project
                .composition_lock
                .recovery_policy_sha256,
            CausalInputReferenceV1 {
                source_kind: CausalInputSourceKindV1::PlatformEvent,
                canonical_hash: event.platform_event_id,
            },
        )?)
    }

    pub(super) fn finish_launch(&mut self) -> Result<(), ApplicationError> {
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

    pub(super) fn lifecycle_request(
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

    fn publish_transition(
        &mut self,
        request: ApplicationLifecycleRequestV1,
        references: SessionTransitionReferencesV1,
        close: Option<DurableCloseOperationV1>,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        self.publish_transition_optional_prepared(request, references, close, None)
    }

    fn publish_transition_optional_prepared(
        &mut self,
        request: ApplicationLifecycleRequestV1,
        references: SessionTransitionReferencesV1,
        close: Option<DurableCloseOperationV1>,
        prepared: Option<&PreparedRunV1>,
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
                    self.publish_planned_transition_optional(plan, None, prepared)
                } else {
                    self.publish_planned_transition_optional(plan, Some(close?), prepared)
                }
            }
        }
    }

    pub(super) fn publish_planned_transition(
        &mut self,
        plan: SessionTransitionPlanV1,
        close: DurableCloseOperationV1,
    ) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
        self.publish_planned_transition_optional(plan, Some(close), None)
    }

    fn publish_planned_transition_optional(
        &mut self,
        plan: SessionTransitionPlanV1,
        close: Option<DurableCloseOperationV1>,
        prepared: Option<&PreparedRunV1>,
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
        let event_bytes = event.canonical_bytes();
        let expected_event = event.clone();
        let request_id = request.request_id;
        let next_state = next_state.as_ref().clone();
        let live = (next_state.state != ApplicationSessionStatusV1::Closed)
            .then_some(next_state.session_id);
        self.with_publication_rollback(|coordinator| {
            if let Some(prepared) = prepared {
                coordinator.record_prepared_run(prepared)?;
            }
            coordinator.record_lifecycle_archive_entry(request_id, request_bytes, event_bytes)?;
            coordinator.durable.state = next_state;
            coordinator.durable.close = close;
            coordinator.publish_current(Some(coordinator.current_generation), live, None)
        })?;
        let committed = self.machine.commit(plan);
        debug_assert_eq!(committed, expected_event);
        Ok(committed)
    }

    pub(super) fn publish_state_plan(
        &mut self,
        plan: SessionStatePublicationPlanV1,
    ) -> Result<(), ApplicationError> {
        let prior_durable = self.durable.clone();
        let prior_generation = self.current_generation;
        self.durable.state = plan.next_state.clone();
        let live = (plan.next_state.state != ApplicationSessionStatusV1::Closed)
            .then_some(plan.next_state.session_id);
        if let Err(error) = self.publish_current(Some(prior_generation), live, None) {
            self.durable = prior_durable;
            self.current_generation = prior_generation;
            return Err(error);
        }
        self.machine.commit_state_publication(plan);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn inject_fail_next_state_publication(&mut self) {
        self.inject_fail_next_publication();
    }

    #[cfg(test)]
    pub(crate) fn inject_fail_next_publication(&mut self) {
        self.fail_next_state_publication = true;
    }

    pub(super) fn with_publication_rollback<T>(
        &mut self,
        publication: impl FnOnce(&mut Self) -> Result<T, ApplicationError>,
    ) -> Result<T, ApplicationError> {
        let prior_machine = self.machine.clone();
        let prior_durable = self.durable.clone();
        let prior_generation = self.current_generation;
        let prior_objects = self.objects.clone();
        let prior_prepared_run_objects = self.prepared_run_objects.clone();
        let prior_platform_host = self.platform_host.clone();

        match publication(self) {
            Ok(value) => Ok(value),
            Err(error) => {
                self.machine = prior_machine;
                self.durable = prior_durable;
                self.current_generation = prior_generation;
                self.objects = prior_objects;
                self.prepared_run_objects = prior_prepared_run_objects;
                self.platform_host = prior_platform_host;
                Err(error)
            }
        }
    }

    pub(super) fn publish_current(
        &mut self,
        expected_previous: Option<ContentHash>,
        live_session_id: Option<ApplicationSessionId>,
        superseded_session_id: Option<ApplicationSessionId>,
    ) -> Result<(), ApplicationError> {
        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_state_publication) {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        if expected_previous.is_some() {
            self.durable.store_sequence = self
                .durable
                .store_sequence
                .checked_add(1)
                .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        }
        let snapshot = self.durable.canonical_bytes()?;
        let mut published_object_bytes = self.objects.clone();
        for (content_hash, bytes) in &self.prepared_run_objects {
            if published_object_bytes
                .get(content_hash)
                .is_some_and(|existing| existing != bytes)
            {
                return Err(ApplicationError::DurableSnapshotInvalid);
            }
            published_object_bytes.insert(*content_hash, bytes.clone());
        }
        let objects = published_object_bytes
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

    pub(super) fn record_object(&mut self, bytes: Vec<u8>) {
        let object = SessionObjectV1::new(bytes);
        self.objects.insert(object.content_hash, object.bytes);
    }

    pub(super) fn record_lifecycle_archive_entry(
        &mut self,
        request_id: next_contracts::ids::SessionRequestId,
        request_bytes: Vec<u8>,
        event_bytes: Vec<u8>,
    ) -> Result<(), ApplicationError> {
        if self.durable.lifecycle_archive.len() >= LIFECYCLE_ARCHIVE_MAX_ENTRIES
            || self
                .durable
                .lifecycle_archive
                .iter()
                .any(|entry| entry.request_id == request_id)
        {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        let request_object = SessionObjectV1::new(request_bytes);
        let event_object = SessionObjectV1::new(event_bytes);
        for object in [&request_object, &event_object] {
            if self
                .objects
                .get(&object.content_hash)
                .is_some_and(|bytes| bytes != &object.bytes)
            {
                return Err(ApplicationError::DurableSnapshotInvalid);
            }
        }
        self.objects
            .insert(request_object.content_hash, request_object.bytes);
        self.objects
            .insert(event_object.content_hash, event_object.bytes);
        self.durable
            .lifecycle_archive
            .push(DurableLifecycleArchiveEntryV1 {
                request_id,
                request_object_hash: request_object.content_hash,
                event_object_hash: event_object.content_hash,
            });
        self.durable
            .lifecycle_archive
            .sort_by_key(|entry| entry.request_id);
        Ok(())
    }
}

pub(super) fn ensure_platform_lifecycle_budget(
    archived_platform_request_count: usize,
) -> Result<(), ApplicationError> {
    if archived_platform_request_count >= PLATFORM_LIFECYCLE_REQUEST_BUDGET {
        Err(ApplicationError::LifecycleArchiveBudgetExceeded)
    } else {
        Ok(())
    }
}

pub(super) fn restore_lifecycle_archive(
    durable: &DurableApplicationSnapshotV1,
    objects: &BTreeMap<ContentHash, Vec<u8>>,
) -> Result<Vec<ArchivedLifecycleRequestV1>, ApplicationError> {
    if !durable.lifecycle_archive_field_present {
        return if durable.state.revision == 0
            && durable.state.state == ApplicationSessionStatusV1::Created
            && durable.state.last_transition_id.is_none()
        {
            Ok(Vec::new())
        } else {
            Err(ApplicationError::RecoveryIncompatible)
        };
    }
    let expected_count = usize::try_from(durable.state.revision)
        .map_err(|_| ApplicationError::RecoveryIncompatible)?;
    if durable.lifecycle_archive.len() != expected_count
        || (expected_count == 0
            && (durable.state.state != ApplicationSessionStatusV1::Created
                || durable.state.last_transition_id.is_some()))
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    let mut archived: Vec<_> = durable
        .lifecycle_archive
        .iter()
        .map(|entry| {
            let request_bytes = objects
                .get(&entry.request_object_hash)
                .ok_or(ApplicationError::RecoveryIncompatible)?
                .clone();
            let event_bytes = objects
                .get(&entry.event_object_hash)
                .ok_or(ApplicationError::RecoveryIncompatible)?;
            let request = ApplicationLifecycleRequestV1::from_jcs_bytes(
                &request_bytes,
                CanonicalDecodeLimits::default(),
            )?;
            let event = ApplicationLifecycleEventV1::from_jcs_bytes(
                event_bytes,
                &request,
                CanonicalDecodeLimits::default(),
            )?;
            if request.request_id != entry.request_id {
                return Err(ApplicationError::RecoveryIncompatible);
            }
            Ok(ArchivedLifecycleRequestV1 {
                request_id: entry.request_id,
                canonical_request_hash: request.canonical_hash,
                canonical_request_bytes: request_bytes,
                event,
            })
        })
        .collect::<Result<_, _>>()?;
    archived.sort_by_key(|entry| entry.event.before_revision);
    for (revision, entry) in archived.iter().enumerate() {
        let revision =
            u64::try_from(revision).map_err(|_| ApplicationError::RecoveryIncompatible)?;
        if entry.event.before_revision != revision
            || entry.event.after_revision != revision + 1
            || (revision == 0 && entry.event.from_state != ApplicationSessionStatusV1::Created)
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }
    }
    Ok(archived)
}
