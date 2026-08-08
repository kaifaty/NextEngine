use next_assets::SessionPublicationV2;
use next_contracts::ids::{ContentHash, SchemaId};
use next_contracts::platform::{PlatformContractError, PlatformEventKindV1, PlatformEventV1};
use next_contracts::session::{
    ApplicationLifecycleEventV2, ApplicationLifecycleRequestV2, ApplicationSessionStatusV1,
    CausalInputReferenceV1, CausalInputSourceKindV1, LifecycleReasonKindV1, LifecycleReasonV1,
};
use next_runtime::{
    LastLifecycleRecordV2, SessionStatePublicationPlanV1, SessionTransitionPlanV1,
    SessionTransitionReferencesV1,
};

use crate::ApplicationError;

use super::identity::{derive_request_id, domain_hash};
use super::{ApplicationCoordinator, PreparedRunV1};

impl ApplicationCoordinator {
    pub fn suspend_from_platform_event(
        &mut self,
        event: &PlatformEventV1,
    ) -> Result<ApplicationLifecycleEventV2, ApplicationError> {
        self.with_platform_event_admission(std::slice::from_ref(event), &[], |coordinator| {
            coordinator.suspend_from_admitted_platform_event(event)
        })
    }

    pub fn resume_from_platform_event(
        &mut self,
        event: &PlatformEventV1,
    ) -> Result<ApplicationLifecycleEventV2, ApplicationError> {
        self.with_platform_event_admission(std::slice::from_ref(event), &[], |coordinator| {
            coordinator.resume_from_admitted_platform_event(event)
        })
    }

    pub(crate) fn suspend_from_admitted_platform_event(
        &mut self,
        event: &PlatformEventV1,
    ) -> Result<ApplicationLifecycleEventV2, ApplicationError> {
        self.transition_from_platform_event(
            event,
            PlatformEventKindV1::SuspendRequested,
            ApplicationSessionStatusV1::Suspended,
            LifecycleReasonKindV1::SuspendRequested,
            "nextengine.session.suspend-requested",
        )
    }

    pub(crate) fn resume_from_admitted_platform_event(
        &mut self,
        event: &PlatformEventV1,
    ) -> Result<ApplicationLifecycleEventV2, ApplicationError> {
        self.transition_from_platform_event(
            event,
            PlatformEventKindV1::ResumeRequested,
            ApplicationSessionStatusV1::Active,
            LifecycleReasonKindV1::ResumeRequested,
            "nextengine.session.resume-requested",
        )
    }

    pub(super) fn suspend_from_admitted_platform_event_with_prepared_run(
        &mut self,
        event: &PlatformEventV1,
        _prepared: &PreparedRunV1,
    ) -> Result<ApplicationLifecycleEventV2, ApplicationError> {
        self.suspend_from_admitted_platform_event(event)
    }

    pub(super) fn suspend_from_committed_ui_action_with_prepared_run(
        &mut self,
        causal_hash: ContentHash,
        _prepared: &PreparedRunV1,
    ) -> Result<ApplicationLifecycleEventV2, ApplicationError> {
        let state = self.machine.state();
        let target = ApplicationSessionStatusV1::Suspended;
        let request = ApplicationLifecycleRequestV2::new(
            derive_request_id(state.session_id, state.revision, target, causal_hash),
            state.session_id,
            state.revision,
            state.state,
            target,
            LifecycleReasonV1 {
                kind: LifecycleReasonKindV1::SuspendRequested,
                reason_code: SchemaId::new("nextengine.session.ui-pause-requested")?,
            },
            CausalInputReferenceV1 {
                source_kind: CausalInputSourceKindV1::PlayerAction,
                canonical_hash: causal_hash,
            },
        )?;
        self.publish_transition(request, SessionTransitionReferencesV1::default())
    }

    fn transition_from_platform_event(
        &mut self,
        event: &PlatformEventV1,
        expected_kind: PlatformEventKindV1,
        target: ApplicationSessionStatusV1,
        reason_kind: LifecycleReasonKindV1,
        reason_code: &str,
    ) -> Result<ApplicationLifecycleEventV2, ApplicationError> {
        event.validate()?;
        if event.kind != expected_kind {
            return Err(PlatformContractError::KindPayloadMismatch.into());
        }
        let state = self.machine.state();
        let request = ApplicationLifecycleRequestV2::new(
            derive_request_id(
                state.session_id,
                state.revision,
                target,
                event.platform_event_id,
            ),
            state.session_id,
            state.revision,
            state.state,
            target,
            LifecycleReasonV1 {
                kind: reason_kind,
                reason_code: SchemaId::new(reason_code)?,
            },
            CausalInputReferenceV1 {
                source_kind: CausalInputSourceKindV1::PlatformEvent,
                canonical_hash: event.platform_event_id,
            },
        )?;
        self.publish_transition(request, SessionTransitionReferencesV1::default())
    }

    pub(super) fn finish_launch(&mut self) -> Result<(), ApplicationError> {
        loop {
            let (target, reason, code) = match self.machine.state().state {
                ApplicationSessionStatusV1::Created => (
                    ApplicationSessionStatusV1::CompositionStaged,
                    LifecycleReasonKindV1::CompositionReady,
                    "nextengine.session.composition-ready",
                ),
                ApplicationSessionStatusV1::CompositionStaged => (
                    ApplicationSessionStatusV1::RuntimeStaged,
                    LifecycleReasonKindV1::RuntimeReady,
                    "nextengine.session.runtime-staged",
                ),
                ApplicationSessionStatusV1::RuntimeStaged => (
                    ApplicationSessionStatusV1::Active,
                    LifecycleReasonKindV1::RuntimeReady,
                    "nextengine.session.active",
                ),
                _ => break,
            };
            let request = self.lifecycle_request(target, reason, code)?;
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
            self.publish_transition(request, references)?;
        }
        Ok(())
    }

    pub(super) fn lifecycle_request(
        &self,
        target: ApplicationSessionStatusV1,
        reason_kind: LifecycleReasonKindV1,
        reason_code: &str,
    ) -> Result<ApplicationLifecycleRequestV2, ApplicationError> {
        let state = self.machine.state();
        let causal_hash = domain_hash(
            b"nextengine.session-lifecycle-cause.v2\0",
            &[
                state.session_id.as_bytes(),
                &state.revision.to_le_bytes(),
                &[target as u8],
            ],
        );
        Ok(ApplicationLifecycleRequestV2::new(
            derive_request_id(state.session_id, state.revision, target, causal_hash),
            state.session_id,
            state.revision,
            state.state,
            target,
            LifecycleReasonV1 {
                kind: reason_kind,
                reason_code: SchemaId::new(reason_code)?,
            },
            CausalInputReferenceV1 {
                source_kind: CausalInputSourceKindV1::System,
                canonical_hash: causal_hash,
            },
        )?)
    }

    pub(super) fn publish_transition(
        &mut self,
        request: ApplicationLifecycleRequestV2,
        references: SessionTransitionReferencesV1,
    ) -> Result<ApplicationLifecycleEventV2, ApplicationError> {
        let plan = self.machine.plan_transition(request, references)?;
        match plan {
            SessionTransitionPlanV1::ExactRetry { event, .. } => Ok(event),
            plan @ SessionTransitionPlanV1::Publish { .. } => self.publish_planned_transition(plan),
        }
    }

    pub(super) fn publish_planned_transition(
        &mut self,
        plan: SessionTransitionPlanV1,
    ) -> Result<ApplicationLifecycleEventV2, ApplicationError> {
        let SessionTransitionPlanV1::Publish {
            request,
            event,
            next_state,
            ..
        } = &plan
        else {
            return Err(ApplicationError::DurableSnapshotInvalid);
        };
        let expected_event = event.clone();
        let next_state = next_state.as_ref().clone();
        let last = LastLifecycleRecordV2 {
            request_id: request.request_id,
            canonical_request_hash: request.canonical_hash,
            canonical_request_bytes: request.canonical_bytes(),
            event: event.clone(),
        };
        self.with_publication_rollback(|coordinator| {
            coordinator.durable.state = next_state;
            coordinator.durable.last_lifecycle = Some(last);
            coordinator.publish_current(Some(coordinator.current_generation))
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
        if let Err(error) = self.publish_current(Some(prior_generation)) {
            self.durable = prior_durable;
            self.current_generation = prior_generation;
            return Err(error);
        }
        self.machine.commit_state_publication(plan);
        Ok(())
    }

    pub(super) fn with_publication_rollback<T>(
        &mut self,
        publication: impl FnOnce(&mut Self) -> Result<T, ApplicationError>,
    ) -> Result<T, ApplicationError> {
        let prior_machine = self.machine.clone();
        let prior_durable = self.durable.clone();
        let prior_generation = self.current_generation;
        let prior_platform_host = self.platform_host.clone();
        match publication(self) {
            Ok(value) => Ok(value),
            Err(error) => {
                self.machine = prior_machine;
                self.durable = prior_durable;
                self.current_generation = prior_generation;
                self.platform_host = prior_platform_host;
                Err(error)
            }
        }
    }

    pub(super) fn publish_current(
        &mut self,
        expected_previous: Option<ContentHash>,
    ) -> Result<(), ApplicationError> {
        if expected_previous.is_some() {
            self.durable.store_sequence = self
                .durable
                .store_sequence
                .checked_add(1)
                .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        }
        let publication = SessionPublicationV2::new(
            self.durable.store_sequence,
            expected_previous,
            self.durable.canonical_bytes()?,
        )?;
        self.current_generation = self.session_store.publish(&publication)?;
        Ok(())
    }
}
