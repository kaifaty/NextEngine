use std::collections::{BTreeMap, BTreeSet};

use next_assets::SessionObjectV1;
use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::ids::{ApplicationSessionId, ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::persistence::{SaveCompatibility, TickSettings};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::session::{
    ApplicationSessionStatusV1, CloseSessionOperationJournalV1, CloseSessionProgressResultV1,
    CloseSessionProgressV1, CloseSessionRequestV1, FailureDispositionV1,
    FinalSaveReservationBodyV1, RecoverySessionLinkV1, SessionFinalSaveLedgerEntryV1,
    close_request_archive_ref,
};
use next_contracts::snapshot::WorldCheckpointV4;
use next_runtime::ApplicationSessionMachine;

use crate::ApplicationError;
use crate::durable::{
    DurableApplicationSnapshotV1, DurableCloseOperationV1, DurableLedgerV1,
    DurableRecoveryEvidenceEntryV1, RECOVERY_EVIDENCE_ARCHIVE_MAX_ENTRIES,
    RECOVERY_EVIDENCE_ARCHIVE_MAX_OBJECT_REFERENCES, RECOVERY_EVIDENCE_OBJECT_BUDGET,
};

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
        if self.durable.recovery_evidence_archive.len() >= RECOVERY_EVIDENCE_ARCHIVE_MAX_ENTRIES {
            return Err(ApplicationError::RecoveryEvidenceBudgetExceeded);
        }
        let existing_reference_count = self
            .durable
            .recovery_evidence_archive
            .iter()
            .try_fold(0_usize, |count, entry| {
                if entry.evidence_object_hashes.len() > RECOVERY_EVIDENCE_OBJECT_BUDGET {
                    None
                } else {
                    count.checked_add(entry.evidence_object_hashes.len())
                }
            })
            .ok_or(ApplicationError::RecoveryEvidenceBudgetExceeded)?;
        if existing_reference_count > RECOVERY_EVIDENCE_ARCHIVE_MAX_OBJECT_REFERENCES {
            return Err(ApplicationError::RecoveryEvidenceBudgetExceeded);
        }
        let prior_publication = self.session_store.load_current()?;
        let next_reference_count = existing_reference_count
            .checked_add(prior_publication.objects.len())
            .ok_or(ApplicationError::RecoveryEvidenceBudgetExceeded)?;
        let next_evidence_object_count = prior_publication
            .objects
            .len()
            .checked_add(2)
            .ok_or(ApplicationError::RecoveryEvidenceBudgetExceeded)?;
        if next_reference_count > RECOVERY_EVIDENCE_ARCHIVE_MAX_OBJECT_REFERENCES
            || next_evidence_object_count > RECOVERY_EVIDENCE_OBJECT_BUDGET
        {
            return Err(ApplicationError::RecoveryEvidenceBudgetExceeded);
        }
        restore_recovery_evidence_archive(&self.durable, &self.objects)?;
        if prior_publication.generation_id != self.current_generation
            || prior_publication.sequence != self.durable.store_sequence
            || prior_publication.snapshot != self.durable.canonical_bytes()?
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
        let recovery_link_bytes = recovery_link.to_jcs_bytes();
        let recovery_link_object = SessionObjectV1::new(recovery_link_bytes.clone());
        let prior_snapshot_object = SessionObjectV1::new(prior_publication.snapshot);
        let evidence_object_hashes = prior_publication
            .objects
            .keys()
            .copied()
            .collect::<Vec<_>>();
        let mut recovery_evidence_archive = self.durable.recovery_evidence_archive.clone();
        recovery_evidence_archive.push(DurableRecoveryEvidenceEntryV1 {
            recovery_link_hash: recovery_link.canonical_hash,
            recovery_link_object_hash: recovery_link_object.content_hash,
            prior_snapshot_object_hash: prior_snapshot_object.content_hash,
            evidence_object_hashes,
        });
        let mut recovery_evidence_objects = prior_publication.objects;
        insert_evidence_object(&mut recovery_evidence_objects, prior_snapshot_object)?;
        insert_evidence_object(&mut recovery_evidence_objects, recovery_link_object)?;
        let manifest = session_manifest(
            &self.launch,
            &self.activated_project,
            new_session_id,
            Some(recovery_link.canonical_hash),
        )?;
        let next_machine = ApplicationSessionMachine::new(manifest.clone())?;
        let next_durable = DurableApplicationSnapshotV1 {
            store_sequence: next_sequence,
            manifest: manifest.clone(),
            state: next_machine.state().clone(),
            close: None,
            live_run_recovery_manifest_hash: None,
            live_run_recovery_field_present: true,
            lifecycle_archive: Vec::new(),
            lifecycle_archive_field_present: true,
            recovery_evidence_archive,
        };
        let prior_generation = self.current_generation;
        let prior_session_id = recovery_link.prior_session_id;
        self.with_publication_rollback(|coordinator| {
            coordinator.machine = next_machine;
            coordinator.durable = next_durable;
            coordinator.platform_host = None;
            coordinator.objects.clear();
            coordinator.prepared_run_objects.clear();
            coordinator.record_object(manifest.to_jcs_bytes());
            for bytes in recovery_evidence_objects.into_values() {
                coordinator.record_object(bytes);
            }
            coordinator.publish_current(
                Some(prior_generation),
                Some(new_session_id),
                Some(prior_session_id),
            )
        })?;
        self.prepared_run = None;
        self.live_run = None;
        self.finish_launch()?;
        let plan = self
            .machine
            .plan_state_publication(None, Some(last_safe_generation_hash))?;
        self.publish_state_plan(plan)?;
        Ok(self)
    }
}

pub(super) fn restore_recovery_evidence_archive(
    durable: &DurableApplicationSnapshotV1,
    objects: &BTreeMap<ContentHash, Vec<u8>>,
) -> Result<Vec<RecoverySessionLinkV1>, ApplicationError> {
    if durable.recovery_evidence_archive.is_empty() {
        return if durable.manifest.body.recovery_session_link_hash.is_none() {
            Ok(Vec::new())
        } else {
            Err(ApplicationError::RecoveryIncompatible)
        };
    }
    if !durable.lifecycle_archive_field_present {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    let mut links = Vec::with_capacity(durable.recovery_evidence_archive.len());
    for (index, entry) in durable.recovery_evidence_archive.iter().enumerate() {
        let bytes = objects
            .get(&entry.recovery_link_object_hash)
            .ok_or(ApplicationError::RecoveryIncompatible)?;
        if SessionObjectV1::new(bytes.clone()).content_hash != entry.recovery_link_object_hash {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        let link = RecoverySessionLinkV1::from_jcs_bytes(bytes, CanonicalDecodeLimits::default())?;
        if link.canonical_hash != entry.recovery_link_hash
            || link.project_composition_lock_hash
                != durable.manifest.body.project_composition_lock_hash
            || links.last().is_some_and(|prior: &RecoverySessionLinkV1| {
                prior.new_session_id != link.prior_session_id
            })
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }

        let prior_snapshot_bytes = objects
            .get(&entry.prior_snapshot_object_hash)
            .ok_or(ApplicationError::RecoveryIncompatible)?;
        if SessionObjectV1::new(prior_snapshot_bytes.clone()).content_hash
            != entry.prior_snapshot_object_hash
        {
            return Err(ApplicationError::RecoveryIncompatible);
        }
        let prior_durable =
            DurableApplicationSnapshotV1::from_canonical_bytes(prior_snapshot_bytes)?;
        if prior_durable.recovery_evidence_archive != durable.recovery_evidence_archive[..index] {
            return Err(ApplicationError::RecoveryIncompatible);
        }

        let evidence_objects = entry
            .evidence_object_hashes
            .iter()
            .map(|hash| {
                let bytes = objects
                    .get(hash)
                    .cloned()
                    .ok_or(ApplicationError::RecoveryIncompatible)?;
                if SessionObjectV1::new(bytes.clone()).content_hash != *hash {
                    return Err(ApplicationError::RecoveryIncompatible);
                }
                Ok((*hash, bytes))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        validate_prior_recovery_evidence(&prior_durable, &evidence_objects, &link)?;
        links.push(link);
    }
    let last = links.last().ok_or(ApplicationError::RecoveryIncompatible)?;
    if last.new_session_id != durable.state.session_id
        || durable.manifest.body.recovery_session_link_hash != Some(last.canonical_hash)
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    Ok(links)
}

fn validate_prior_recovery_evidence(
    prior: &DurableApplicationSnapshotV1,
    objects: &BTreeMap<ContentHash, Vec<u8>>,
    link: &RecoverySessionLinkV1,
) -> Result<(), ApplicationError> {
    if prior.state.session_id != link.prior_session_id
        || prior.state.canonical_hash != link.prior_session_state_hash
        || prior.state.revision != link.prior_session_revision
        || prior.manifest.canonical_hash != link.prior_application_session_manifest_hash
        || prior.manifest.body.project_composition_lock_hash != link.project_composition_lock_hash
        || prior.state.state != ApplicationSessionStatusV1::Finalizing
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }

    require_evidence_bytes(objects, &prior.manifest.to_jcs_bytes())?;
    let archived_lifecycle = super::publication::restore_lifecycle_archive(prior, objects)?;

    if let Some(live_manifest_hash) = prior.live_run_recovery_manifest_hash {
        super::run::validate_live_run_evidence_closure(
            objects,
            live_manifest_hash,
            prior.state.session_id,
            prior.manifest.body.project_composition_lock_hash,
            prior.manifest.body.content_manifest_hash,
            prior
                .state
                .active_runtime_revision
                .ok_or(ApplicationError::RecoveryIncompatible)?,
            prior.manifest.body.presentation_target_kind,
        )?;
    }

    let close = prior
        .close
        .as_ref()
        .ok_or(ApplicationError::RecoveryIncompatible)?;
    let request = CloseSessionRequestV1::from_canonical_bytes(
        &close.canonical_close_request_bytes,
        CanonicalDecodeLimits::default(),
    )?;
    if request.session_id != prior.state.session_id
        || request.close_request_id != close.close_request_id
        || request.close_request_id != link.prior_close_request_id
        || request.canonical_close_request_hash != close.canonical_close_request_hash
        || request.canonical_close_request_hash != link.canonical_close_request_hash
        || request.starting_session_revision != close.starting_session_revision
        || request.starting_session_state != close.starting_session_state
        || close_request_archive_ref(&close.canonical_close_request_bytes)
            != close.close_request_archive_ref
        || close.failure_disposition != FailureDispositionV1::RequireFinalSave
        || close.close_session_receipt_hash.is_some()
        || close.closed_event_hash.is_some()
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    require_evidence_bytes(objects, &close.canonical_close_request_bytes)?;

    let journal = rebuild_journal(&request, close)?;
    require_evidence_bytes(objects, journal.canonical_hash.as_bytes())?;

    let durable_ledger = close
        .ledger
        .as_ref()
        .ok_or(ApplicationError::RecoveryIncompatible)?;
    let ledger = rebuild_failed_ledger(&request, close, durable_ledger)?;
    if ledger.entry_hash != link.failed_final_save_ledger_entry_hash {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    require_evidence_bytes(objects, ledger.entry_hash.as_bytes())?;

    let lifecycle_event_hashes = archived_lifecycle
        .into_iter()
        .map(|archived| archived.event.canonical_hash)
        .collect::<BTreeSet<_>>();
    if close
        .quiesce_event_hash
        .is_some_and(|hash| !lifecycle_event_hashes.contains(&hash))
        || close
            .finalizing_event_hash
            .is_some_and(|hash| !lifecycle_event_hashes.contains(&hash))
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    Ok(())
}

fn rebuild_failed_ledger(
    request: &CloseSessionRequestV1,
    close: &DurableCloseOperationV1,
    durable: &DurableLedgerV1,
) -> Result<SessionFinalSaveLedgerEntryV1, ApplicationError> {
    if durable.status != next_contracts::session::FinalSaveLedgerStatusV1::Failed
        || durable.attempt_count == 0
        || durable.last_failure_code.is_none()
        || durable.save_generation_hash.is_some()
        || durable.final_save_receipt_hash.is_some()
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    let reservation = FinalSaveReservationBodyV1::new(
        request.session_id,
        request.close_request_id,
        request.canonical_close_request_hash,
        close.close_request_archive_ref,
        request.starting_session_revision,
        request.final_save_policy,
        request.shutdown_policy_hash,
        durable.maximum_attempts,
    )?;
    let failure = durable
        .last_failure_code
        .clone()
        .ok_or(ApplicationError::RecoveryIncompatible)?;
    let mut ledger = SessionFinalSaveLedgerEntryV1::reserved(&reservation)?;
    for _ in 1..durable.attempt_count {
        ledger = ledger.retry_pending(failure.clone())?;
    }
    ledger = ledger.failed(failure)?;
    if durable_ledger(&ledger) != *durable {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    Ok(ledger)
}

fn require_evidence_bytes(
    objects: &BTreeMap<ContentHash, Vec<u8>>,
    bytes: &[u8],
) -> Result<(), ApplicationError> {
    let object = SessionObjectV1::new(bytes.to_vec());
    if objects.get(&object.content_hash) != Some(&object.bytes) {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    Ok(())
}

fn insert_evidence_object(
    objects: &mut BTreeMap<ContentHash, Vec<u8>>,
    object: SessionObjectV1,
) -> Result<(), ApplicationError> {
    if objects
        .get(&object.content_hash)
        .is_some_and(|bytes| bytes != &object.bytes)
    {
        return Err(ApplicationError::RecoveryIncompatible);
    }
    objects.insert(object.content_hash, object.bytes);
    Ok(())
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
