use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::ids::{ApplicationSessionId, ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::persistence::{SaveCompatibility, TickSettings};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::session::{
    ApplicationSessionStatusV1, CloseSessionOperationJournalV1, CloseSessionProgressResultV1,
    CloseSessionProgressV1, CloseSessionRequestV1, FailureDispositionV1,
    FinalSaveReservationBodyV1, RecoverySessionLinkV1, SessionFinalSaveLedgerEntryV1,
};
use next_contracts::snapshot::WorldCheckpointV4;
use next_runtime::ApplicationSessionMachine;

use crate::ApplicationError;
use crate::durable::{DurableApplicationSnapshotV1, DurableCloseOperationV1, DurableLedgerV1};

use super::ApplicationCoordinator;
use super::activation::session_manifest;
use super::identity::{derive_session_id, domain_hash};

impl ApplicationCoordinator {
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
}

pub(super) fn reservation(
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

pub(super) fn rebuild_retryable_ledger(
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

pub(super) fn rebuild_journal(
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

pub(super) fn durable_ledger(ledger: &SessionFinalSaveLedgerEntryV1) -> DurableLedgerV1 {
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

pub(super) fn progress(
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

pub(super) fn save_compatibility(
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

pub(super) fn save_identity(
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
