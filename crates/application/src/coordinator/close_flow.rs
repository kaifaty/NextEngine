use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::ids::{ContentHash, SchemaId};
use next_contracts::platform::{PlatformContractError, PlatformEventKindV1, PlatformEventV1};
use next_contracts::session::{
    ApplicationLifecycleEventV1, ApplicationLifecycleRequestV1, ApplicationSessionStatusV1,
    BoundedDeadlineClassV1, CausalInputReferenceV1, CausalInputSourceKindV1,
    CloseSessionOperationJournalV1, CloseSessionProgressResultV1, CloseSessionReceiptV1,
    CloseSessionRequestV1, CloseSessionResultV1, FinalSavePolicyV1, FinalSaveReceiptV1,
    LifecycleReasonKindV1, LifecycleReasonV1, SessionFinalSaveLedgerEntryV1,
    can_close_after_failed_save, close_request_archive_ref,
};
use next_runtime::{SessionTransitionPlanV1, SessionTransitionReferencesV1};

use crate::ApplicationError;
use crate::close::{ApplicationCloseOutcomeV1, CloseExecutionOptionsV1, FinalSaveAttemptFailureV1};
use crate::durable::DurableCloseOperationV1;

use super::ApplicationCoordinator;
use super::identity::{derive_close_request_id, derive_request_id, domain_hash};
use super::recovery::{
    durable_ledger, progress, rebuild_journal, rebuild_retryable_ledger, reservation,
    save_compatibility, save_identity,
};

struct CloseRequestIntentV1<'a> {
    deadline: BoundedDeadlineClassV1,
    reason_kind: LifecycleReasonKindV1,
    reason_code: &'a str,
    causal_source_kind: CausalInputSourceKindV1,
    causal_hash: ContentHash,
}

impl ApplicationCoordinator {
    pub fn close_request(
        &self,
        deadline: BoundedDeadlineClassV1,
    ) -> Result<CloseSessionRequestV1, ApplicationError> {
        if let Some(close) = &self.durable.close {
            return archived_close_request(close);
        }
        self.build_close_request(
            self.machine.state().revision,
            self.machine.state().state,
            CloseRequestIntentV1 {
                deadline,
                reason_kind: LifecycleReasonKindV1::UserCloseRequested,
                reason_code: "nextengine.session.close-requested",
                causal_source_kind: CausalInputSourceKindV1::SystemPolicy,
                causal_hash: domain_hash(
                    b"nextengine.close-request-cause.v1\0",
                    &[
                        self.machine.state().session_id.as_bytes(),
                        &self.machine.state().revision.to_le_bytes(),
                    ],
                ),
            },
        )
    }

    pub fn close_request_from_platform_event(
        &mut self,
        event: &PlatformEventV1,
        deadline: BoundedDeadlineClassV1,
    ) -> Result<CloseSessionRequestV1, ApplicationError> {
        if event.kind != PlatformEventKindV1::CloseRequested {
            return Err(PlatformContractError::KindPayloadMismatch.into());
        }
        self.with_platform_event_admission(
            std::slice::from_ref(event),
            std::slice::from_ref(event),
            |coordinator| coordinator.close_request_from_admitted_platform_event(event, deadline),
        )
    }

    fn close_request_from_admitted_platform_event(
        &self,
        event: &PlatformEventV1,
        deadline: BoundedDeadlineClassV1,
    ) -> Result<CloseSessionRequestV1, ApplicationError> {
        if let Some(close) = &self.durable.close {
            let request = archived_close_request(close)?;
            if request.causal_input_reference.source_kind != CausalInputSourceKindV1::PlatformEvent
                || request.causal_input_reference.canonical_hash != event.platform_event_id
                || request.bounded_deadline_class != deadline
            {
                return Err(ApplicationError::CloseIdentityCollision);
            }
            return Ok(request);
        }
        self.build_close_request(
            self.machine.state().revision,
            self.machine.state().state,
            CloseRequestIntentV1 {
                deadline,
                reason_kind: LifecycleReasonKindV1::HostCloseRequested,
                reason_code: "nextengine.session.host-close-requested",
                causal_source_kind: CausalInputSourceKindV1::PlatformEvent,
                causal_hash: event.platform_event_id,
            },
        )
    }

    pub fn close(
        &mut self,
        options: CloseExecutionOptionsV1,
    ) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
        let request = self.close_request(BoundedDeadlineClassV1::Standard)?;
        self.close_with_request(request, options)
    }

    pub fn close_from_platform_event(
        &mut self,
        event: &PlatformEventV1,
        options: CloseExecutionOptionsV1,
    ) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
        let request =
            self.close_request_from_platform_event(event, BoundedDeadlineClassV1::Standard)?;
        self.close_with_request(request, options)
    }

    pub fn close_with_request(
        &mut self,
        request: CloseSessionRequestV1,
        options: CloseExecutionOptionsV1,
    ) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
        request.validate()?;
        self.validate_close_request_preconditions(&request, options.last_safe_generation_hash)?;
        self.flush_reference_game_live_checkpoint()?;
        self.register_or_validate_close(&request, options.last_safe_generation_hash)?;
        if self.machine.state().state == ApplicationSessionStatusV1::Closed {
            return self.closed_outcome();
        }
        self.advance_to_finalizing(&request)?;
        self.execute_final_save_attempt(&request, options)
    }

    fn build_close_request(
        &self,
        starting_revision: u64,
        starting_state: ApplicationSessionStatusV1,
        intent: CloseRequestIntentV1<'_>,
    ) -> Result<CloseSessionRequestV1, ApplicationError> {
        let session_id = self.machine.state().session_id;
        let close_request_id =
            derive_close_request_id(session_id, starting_revision, intent.causal_hash);
        Ok(CloseSessionRequestV1::new(
            close_request_id,
            session_id,
            starting_revision,
            starting_state,
            self.activated_project
                .composition_lock
                .shutdown_policy_sha256,
            FinalSavePolicyV1::Always,
            intent.deadline,
            LifecycleReasonV1 {
                kind: intent.reason_kind,
                reason_code: SchemaId::new(intent.reason_code)?,
            },
            CausalInputReferenceV1 {
                source_kind: intent.causal_source_kind,
                canonical_hash: intent.causal_hash,
            },
        )?)
    }

    pub(super) fn register_or_validate_close(
        &mut self,
        request: &CloseSessionRequestV1,
        last_safe_generation_hash: Option<ContentHash>,
    ) -> Result<(), ApplicationError> {
        self.validate_close_request_preconditions(request, last_safe_generation_hash)?;
        if self.durable.close.is_some() {
            return Ok(());
        }
        let bytes = request.canonical_bytes()?;
        let archive_ref = close_request_archive_ref(&bytes);
        let journal = CloseSessionOperationJournalV1::registered(request, archive_ref)?;
        let close = DurableCloseOperationV1 {
            close_request_id: request.close_request_id,
            canonical_close_request_hash: request.canonical_close_request_hash,
            canonical_close_request_bytes: bytes.clone(),
            close_request_archive_ref: archive_ref,
            starting_session_revision: request.starting_session_revision,
            starting_session_state: request.starting_session_state,
            stage: journal.stage,
            operation_journal_hash: journal.canonical_hash,
            quiesce_event_hash: None,
            finalizing_event_hash: None,
            ledger: None,
            failure_disposition: self
                .activated_project
                .composition_lock
                .shutdown_failure_disposition,
            close_session_receipt_hash: None,
            closed_event_hash: None,
            last_safe_generation_hash,
        };
        self.with_publication_rollback(|coordinator| {
            coordinator.durable.close = Some(close);
            coordinator.record_object(bytes);
            coordinator.record_object(journal.canonical_hash.as_bytes().to_vec());
            coordinator.publish_current(
                Some(coordinator.current_generation),
                Some(coordinator.machine.state().session_id),
                None,
            )
        })
    }

    fn validate_close_request_preconditions(
        &self,
        request: &CloseSessionRequestV1,
        last_safe_generation_hash: Option<ContentHash>,
    ) -> Result<(), ApplicationError> {
        if let Some(existing) = &self.durable.close {
            if existing.close_request_id != request.close_request_id
                || existing.canonical_close_request_hash != request.canonical_close_request_hash
                || existing.canonical_close_request_bytes != request.canonical_bytes()?
                || last_safe_generation_hash
                    .is_some_and(|hash| Some(hash) != existing.last_safe_generation_hash)
            {
                return Err(ApplicationError::CloseIdentityCollision);
            }
            return Ok(());
        }
        if request.session_id != self.machine.state().session_id
            || request.starting_session_revision != self.machine.state().revision
            || request.starting_session_state != self.machine.state().state
            || !matches!(
                request.starting_session_state,
                ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended
            )
            || request.shutdown_policy_hash
                != self
                    .activated_project
                    .composition_lock
                    .shutdown_policy_sha256
            || request.final_save_policy != FinalSavePolicyV1::Always
            || request.close_request_id
                != derive_close_request_id(
                    request.session_id,
                    request.starting_session_revision,
                    request.causal_input_reference.canonical_hash,
                )
        {
            return Err(ApplicationError::CloseStateInvalid);
        }
        if let Some(expected_last_safe) = last_safe_generation_hash {
            let checkpoint = &self
                .prepared_run
                .as_ref()
                .ok_or(ApplicationError::NoRunOutcome)?
                .checkpoint;
            let compatibility = save_compatibility(&self.activated_project, checkpoint)?;
            let loaded = self.save_store.load_latest(&compatibility)?;
            if save_identity(&loaded.image.manifest)?.0 != expected_last_safe {
                return Err(ApplicationError::RecoveryIncompatible);
            }
        }
        Ok(())
    }

    pub(super) fn advance_to_finalizing(
        &mut self,
        close_request: &CloseSessionRequestV1,
    ) -> Result<(), ApplicationError> {
        if matches!(
            self.machine.state().state,
            ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended
        ) {
            let run_revision = self
                .prepared_run
                .as_ref()
                .map(|run| run.summary.authoritative_revision)
                .or(self.machine.state().active_runtime_revision);
            let state = self.machine.state();
            let target = ApplicationSessionStatusV1::Quiescing;
            let request = ApplicationLifecycleRequestV1::new(
                derive_request_id(
                    state.session_id,
                    state.revision,
                    target,
                    close_request.causal_input_reference.canonical_hash,
                ),
                state.session_id,
                state.revision,
                state.state,
                target,
                close_request.reason.clone(),
                close_request.shutdown_policy_hash,
                close_request.causal_input_reference.clone(),
            )?;
            let plan = self.machine.plan_transition(
                request.clone(),
                SessionTransitionReferencesV1 {
                    active_runtime_revision: run_revision,
                    ..SessionTransitionReferencesV1::default()
                },
            )?;
            let event = planned_event(&plan)?;
            let mut close = self
                .durable
                .close
                .clone()
                .ok_or(ApplicationError::CloseJournalInvalid)?;
            let journal = rebuild_journal(close_request, &close)?.with_stage(
                close_request,
                next_contracts::session::CloseSessionOperationStageV1::Quiesced,
                Some(event.canonical_hash),
                None,
                None,
                None,
            )?;
            close.stage = journal.stage;
            close.operation_journal_hash = journal.canonical_hash;
            close.quiesce_event_hash = Some(event.canonical_hash);
            self.publish_planned_transition(plan, close)?;
        }
        if self.machine.state().state == ApplicationSessionStatusV1::Quiescing {
            let mut close = self
                .durable
                .close
                .clone()
                .ok_or(ApplicationError::CloseJournalInvalid)?;
            let reservation = reservation(close_request, &close, &self.activated_project)?;
            let ledger = SessionFinalSaveLedgerEntryV1::reserved(&reservation)?;
            let request = self.lifecycle_request(
                ApplicationSessionStatusV1::Finalizing,
                LifecycleReasonKindV1::FinalSaveReady,
                "nextengine.session.finalizing",
                self.activated_project
                    .composition_lock
                    .shutdown_policy_sha256,
            )?;
            let plan = self
                .machine
                .plan_transition(request, SessionTransitionReferencesV1::default())?;
            let event = planned_event(&plan)?;
            let journal = rebuild_journal(close_request, &close)?.with_stage(
                close_request,
                next_contracts::session::CloseSessionOperationStageV1::Finalizing,
                close.quiesce_event_hash,
                Some(event.canonical_hash),
                Some(ledger.entry_hash),
                None,
            )?;
            close.stage = journal.stage;
            close.operation_journal_hash = journal.canonical_hash;
            close.finalizing_event_hash = Some(event.canonical_hash);
            close.ledger = Some(durable_ledger(&ledger));
            self.publish_planned_transition(plan, close)?;
        }
        if self.machine.state().state != ApplicationSessionStatusV1::Finalizing {
            return Err(ApplicationError::CloseStateInvalid);
        }
        Ok(())
    }

    fn execute_final_save_attempt(
        &mut self,
        close_request: &CloseSessionRequestV1,
        options: CloseExecutionOptionsV1,
    ) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
        let close = self
            .durable
            .close
            .clone()
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        let durable_ledger = close
            .ledger
            .as_ref()
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        match durable_ledger.status {
            next_contracts::session::FinalSaveLedgerStatusV1::Committed => {
                return self.finalize_closed(close_request, CloseSessionResultV1::Saved);
            }
            next_contracts::session::FinalSaveLedgerStatusV1::Failed => {
                if can_close_after_failed_save(
                    close.failure_disposition,
                    close.last_safe_generation_hash,
                ) {
                    return self.finalize_closed(
                        close_request,
                        CloseSessionResultV1::ClosedUsingLastSafeGeneration,
                    );
                }
                return Ok(ApplicationCloseOutcomeV1::Progress(progress(
                    &close,
                    self.machine.state().revision,
                    CloseSessionProgressResultV1::FinalSaveRequiredFailed,
                )?));
            }
            _ => {}
        }

        if let Some(failure) = options.final_save_failure {
            return self.record_save_failure(close_request, failure);
        }
        self.commit_final_save(close_request)?;
        self.finalize_closed(close_request, CloseSessionResultV1::Saved)
    }

    pub(super) fn record_save_failure(
        &mut self,
        close_request: &CloseSessionRequestV1,
        failure: FinalSaveAttemptFailureV1,
    ) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
        let mut close = self
            .durable
            .close
            .clone()
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        let mut ledger = rebuild_retryable_ledger(close_request, &close, &self.activated_project)?;
        let (failure_code, retryable) = match failure {
            FinalSaveAttemptFailureV1::Retryable(code) => (code, true),
            FinalSaveAttemptFailureV1::Terminal(code) => (code, false),
        };
        let next_attempt = ledger
            .attempt_count
            .checked_add(1)
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        ledger = if retryable && next_attempt < ledger.maximum_attempts {
            ledger.retry_pending(failure_code)?
        } else {
            ledger.failed(failure_code)?
        };
        let stage =
            if ledger.status == next_contracts::session::FinalSaveLedgerStatusV1::RetryPending {
                next_contracts::session::CloseSessionOperationStageV1::SaveRetryPending
            } else {
                next_contracts::session::CloseSessionOperationStageV1::SaveTerminal
            };
        let journal = rebuild_journal(close_request, &close)?.with_stage(
            close_request,
            stage,
            close.quiesce_event_hash,
            close.finalizing_event_hash,
            Some(ledger.entry_hash),
            None,
        )?;
        close.stage = stage;
        close.operation_journal_hash = journal.canonical_hash;
        close.ledger = Some(durable_ledger(&ledger));
        self.with_publication_rollback(|coordinator| {
            coordinator.durable.close = Some(close.clone());
            coordinator.record_object(ledger.entry_hash.as_bytes().to_vec());
            coordinator.record_object(journal.canonical_hash.as_bytes().to_vec());
            coordinator.publish_current(
                Some(coordinator.current_generation),
                Some(coordinator.machine.state().session_id),
                None,
            )
        })?;
        if ledger.status == next_contracts::session::FinalSaveLedgerStatusV1::RetryPending {
            return Ok(ApplicationCloseOutcomeV1::Progress(progress(
                &close,
                self.machine.state().revision,
                CloseSessionProgressResultV1::RetryPending,
            )?));
        }
        if can_close_after_failed_save(close.failure_disposition, close.last_safe_generation_hash) {
            self.finalize_closed(
                close_request,
                CloseSessionResultV1::ClosedUsingLastSafeGeneration,
            )
        } else {
            Ok(ApplicationCloseOutcomeV1::Progress(progress(
                &close,
                self.machine.state().revision,
                CloseSessionProgressResultV1::FinalSaveRequiredFailed,
            )?))
        }
    }

    pub(super) fn commit_final_save(
        &mut self,
        close_request: &CloseSessionRequestV1,
    ) -> Result<(), ApplicationError> {
        self.ensure_prepared_run()?;
        let prepared = self
            .prepared_run
            .as_ref()
            .ok_or(ApplicationError::NoRunOutcome)?
            .clone();
        let compatibility = save_compatibility(&self.activated_project, &prepared.checkpoint)?;
        let loaded = match self.save_store.load_latest(&compatibility) {
            Ok(existing)
                if existing.checkpoint == prepared.checkpoint
                    && existing.world_streaming_snapshot.as_ref() == Some(&prepared.streaming) =>
            {
                existing
            }
            _ => {
                self.save_store.commit_world_checkpoint_with_streaming(
                    compatibility.clone(),
                    &prepared.checkpoint,
                    &prepared.streaming,
                )?;
                self.save_store.load_latest(&compatibility)?
            }
        };
        #[cfg(test)]
        if self.pause_after_save_commit {
            return Err(ApplicationError::FinalSaveFailed);
        }
        let (save_generation_hash, save_manifest_hash) = save_identity(&loaded.image.manifest)?;
        let mut close = self
            .durable
            .close
            .clone()
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        let ledger = rebuild_retryable_ledger(close_request, &close, &self.activated_project)?;
        let receipt = FinalSaveReceiptV1::new(
            close_request.session_id,
            close_request.close_request_id,
            close_request.canonical_close_request_hash,
            ledger.reservation_hash,
            ledger.attempt_count + 1,
            self.machine.state().revision,
            prepared.checkpoint.runtime_snapshot.authoritative_revision,
            save_generation_hash,
            save_manifest_hash,
            prepared.summary.authoritative_state_root,
        )?;
        let ledger = ledger.committed(&receipt)?;
        let journal = rebuild_journal(close_request, &close)?.with_stage(
            close_request,
            next_contracts::session::CloseSessionOperationStageV1::SaveTerminal,
            close.quiesce_event_hash,
            close.finalizing_event_hash,
            Some(ledger.entry_hash),
            None,
        )?;
        close.stage = journal.stage;
        close.operation_journal_hash = journal.canonical_hash;
        close.ledger = Some(durable_ledger(&ledger));
        let state_plan = self
            .machine
            .plan_state_publication(None, Some(save_generation_hash))?;
        self.with_publication_rollback(|coordinator| {
            coordinator.durable.close = Some(close);
            coordinator.record_object(receipt.canonical_hash.as_bytes().to_vec());
            coordinator.record_object(ledger.entry_hash.as_bytes().to_vec());
            coordinator.record_object(journal.canonical_hash.as_bytes().to_vec());
            coordinator.publish_state_plan(state_plan)
        })
    }

    fn finalize_closed(
        &mut self,
        close_request: &CloseSessionRequestV1,
        result: CloseSessionResultV1,
    ) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
        let mut close = self
            .durable
            .close
            .clone()
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        let ledger = close
            .ledger
            .as_ref()
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        let (final_save_receipt_hash, last_safe_generation_hash, save_generation_hash) =
            match result {
                CloseSessionResultV1::Saved => (
                    ledger.final_save_receipt_hash,
                    None,
                    ledger.save_generation_hash,
                ),
                CloseSessionResultV1::ClosedUsingLastSafeGeneration => (
                    None,
                    close.last_safe_generation_hash,
                    close.last_safe_generation_hash,
                ),
            };
        let request = self.lifecycle_request(
            ApplicationSessionStatusV1::Closed,
            LifecycleReasonKindV1::FinalSaveReady,
            "nextengine.session.closed",
            self.activated_project
                .composition_lock
                .shutdown_policy_sha256,
        )?;
        let preview = self.machine.plan_transition(
            request.clone(),
            SessionTransitionReferencesV1 {
                save_receipt_hash: final_save_receipt_hash,
                terminal_receipt_hash: Some(ContentHash::from_bytes([0x7f; 32])),
                active_save_generation_hash: save_generation_hash,
                ..SessionTransitionReferencesV1::default()
            },
        )?;
        let closed_event = planned_event(&preview)?;
        let receipt = CloseSessionReceiptV1::new(
            close.close_request_id,
            self.machine.state().session_id,
            close.canonical_close_request_hash,
            close.starting_session_revision,
            close
                .quiesce_event_hash
                .ok_or(ApplicationError::CloseJournalInvalid)?,
            ledger.entry_hash,
            final_save_receipt_hash,
            last_safe_generation_hash,
            close
                .finalizing_event_hash
                .ok_or(ApplicationError::CloseJournalInvalid)?,
            close.operation_journal_hash,
            closed_event.canonical_hash,
            self.machine.state().revision + 1,
            result,
        )?;
        let journal = rebuild_journal(close_request, &close)?.with_stage(
            close_request,
            next_contracts::session::CloseSessionOperationStageV1::Closed,
            close.quiesce_event_hash,
            close.finalizing_event_hash,
            Some(ledger.entry_hash),
            Some(receipt.canonical_hash),
        )?;
        close.stage = journal.stage;
        close.operation_journal_hash = journal.canonical_hash;
        close.close_session_receipt_hash = Some(receipt.canonical_hash);
        close.closed_event_hash = Some(closed_event.canonical_hash);
        let plan = self.machine.plan_transition(
            request,
            SessionTransitionReferencesV1 {
                save_receipt_hash: final_save_receipt_hash,
                terminal_receipt_hash: Some(receipt.canonical_hash),
                active_save_generation_hash: save_generation_hash,
                ..SessionTransitionReferencesV1::default()
            },
        )?;
        if planned_event(&plan)?.canonical_hash != closed_event.canonical_hash {
            return Err(ApplicationError::CloseJournalInvalid);
        }
        self.publish_planned_transition(plan, close)?;
        Ok(ApplicationCloseOutcomeV1::Closed {
            receipt_hash: receipt.canonical_hash,
            result,
            save_generation_hash,
        })
    }

    fn closed_outcome(&self) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
        let close = self
            .durable
            .close
            .as_ref()
            .ok_or(ApplicationError::TerminalReceiptMissing)?;
        let ledger = close
            .ledger
            .as_ref()
            .ok_or(ApplicationError::TerminalReceiptMissing)?;
        let receipt_hash = close
            .close_session_receipt_hash
            .ok_or(ApplicationError::TerminalReceiptMissing)?;
        let (result, save_generation_hash) =
            if ledger.status == next_contracts::session::FinalSaveLedgerStatusV1::Committed {
                (CloseSessionResultV1::Saved, ledger.save_generation_hash)
            } else {
                (
                    CloseSessionResultV1::ClosedUsingLastSafeGeneration,
                    close.last_safe_generation_hash,
                )
            };
        Ok(ApplicationCloseOutcomeV1::Closed {
            receipt_hash,
            result,
            save_generation_hash,
        })
    }

    #[cfg(test)]
    pub(super) fn inject_pause_after_save_commit(&mut self) {
        self.pause_after_save_commit = true;
    }
}

fn planned_event(
    plan: &SessionTransitionPlanV1,
) -> Result<ApplicationLifecycleEventV1, ApplicationError> {
    match plan {
        SessionTransitionPlanV1::Publish { event, .. }
        | SessionTransitionPlanV1::ExactRetry { event, .. } => Ok(event.clone()),
    }
}

fn archived_close_request(
    close: &DurableCloseOperationV1,
) -> Result<CloseSessionRequestV1, ApplicationError> {
    let request = CloseSessionRequestV1::from_canonical_bytes(
        &close.canonical_close_request_bytes,
        CanonicalDecodeLimits::default(),
    )?;
    if request.close_request_id != close.close_request_id
        || request.canonical_close_request_hash != close.canonical_close_request_hash
        || request.starting_session_revision != close.starting_session_revision
        || request.starting_session_state != close.starting_session_state
        || close_request_archive_ref(&close.canonical_close_request_bytes)
            != close.close_request_archive_ref
    {
        return Err(ApplicationError::CloseJournalInvalid);
    }
    Ok(request)
}
