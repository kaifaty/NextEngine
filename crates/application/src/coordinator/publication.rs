use next_assets::{SessionObjectV1, SessionPublicationV1};
use next_contracts::ids::{ApplicationSessionId, ContentHash, SchemaId};
use next_contracts::session::{
    ApplicationLifecycleEventV1, ApplicationLifecycleRequestV1, ApplicationSessionStatusV1,
    CausalInputReferenceV1, CausalInputSourceKindV1, LifecycleReasonKindV1, LifecycleReasonV1,
};
use next_runtime::{
    SessionStatePublicationPlanV1, SessionTransitionPlanV1, SessionTransitionReferencesV1,
};

use crate::ApplicationError;
use crate::durable::DurableCloseOperationV1;

use super::ApplicationCoordinator;
use super::identity::{derive_request_id, domain_hash};

impl ApplicationCoordinator {
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

    pub(super) fn publish_planned_transition(
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

    pub(super) fn publish_state_plan(
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

    pub(super) fn publish_current(
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

    pub(super) fn record_object(&mut self, bytes: Vec<u8>) {
        let object = SessionObjectV1::new(bytes);
        self.objects.insert(object.content_hash, object.bytes);
    }
}
