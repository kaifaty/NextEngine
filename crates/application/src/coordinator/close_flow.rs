use super::*;

impl ApplicationCoordinator {
    pub(super) fn register_or_validate_close(
        &mut self,
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
        {
            return Err(ApplicationError::CloseStateInvalid);
        }
        if let Some(expected_last_safe) = last_safe_generation_hash {
            self.ensure_prepared_run()?;
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
        self.durable.close = Some(close);
        self.record_object(bytes);
        self.record_object(journal.canonical_hash.as_bytes().to_vec());
        self.publish_current(
            Some(self.current_generation),
            Some(self.machine.state().session_id),
            None,
        )
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
            let request = self.lifecycle_request(
                ApplicationSessionStatusV1::Quiescing,
                LifecycleReasonKindV1::UserCloseRequested,
                "nextengine.session.quiescing",
                self.activated_project
                    .composition_lock
                    .shutdown_policy_sha256,
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

    pub(super) fn execute_final_save_attempt(
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

    fn record_save_failure(
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
        self.durable.close = Some(close.clone());
        self.record_object(ledger.entry_hash.as_bytes().to_vec());
        self.record_object(journal.canonical_hash.as_bytes().to_vec());
        self.publish_current(
            Some(self.current_generation),
            Some(self.machine.state().session_id),
            None,
        )?;
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

    fn commit_final_save(
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
        self.durable.close = Some(close);
        self.record_object(receipt.canonical_hash.as_bytes().to_vec());
        self.record_object(ledger.entry_hash.as_bytes().to_vec());
        self.record_object(journal.canonical_hash.as_bytes().to_vec());
        let state_plan = self
            .machine
            .plan_state_publication(None, Some(save_generation_hash))?;
        self.publish_state_plan(state_plan)
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

    pub(super) fn closed_outcome(&self) -> Result<ApplicationCloseOutcomeV1, ApplicationError> {
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
}
