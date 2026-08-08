use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::persistence::{SaveCompatibility, TickSettings};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::session::{
    ApplicationSessionStatusV1, CausalInputReferenceV1, CausalInputSourceKindV1,
    CloseSessionJournalStageV2, CloseSessionJournalV2, CloseSessionReceiptV2,
    CloseSessionRequestV2, LifecycleReasonKindV1, LifecycleReasonV1,
};
use next_contracts::snapshot::WorldCheckpointV4;
use next_runtime::SessionTransitionReferencesV1;

use crate::{ApplicationCloseOutcomeV2, ApplicationError};

use super::ApplicationCoordinator;
use super::identity::{derive_close_request_id, domain_hash};

impl ApplicationCoordinator {
    pub fn close_from_platform_event(
        &mut self,
        event: &next_contracts::platform::PlatformEventV1,
    ) -> Result<ApplicationCloseOutcomeV2, ApplicationError> {
        event.validate()?;
        if event.kind != next_contracts::platform::PlatformEventKindV1::CloseRequested {
            return Err(ApplicationError::CloseStateInvalid);
        }
        self.with_platform_event_admission(std::slice::from_ref(event), &[], |coordinator| {
            coordinator.close()
        })
    }

    pub fn save_current_prepared_run(&mut self) -> Result<ContentHash, ApplicationError> {
        if !matches!(
            self.machine.state().state,
            ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended
        ) {
            return Err(ApplicationError::CloseStateInvalid);
        }
        self.flush_reference_game_live_checkpoint()?;
        self.ensure_prepared_run()?;
        let prepared = self
            .prepared_run
            .as_ref()
            .ok_or(ApplicationError::NoRunOutcome)?;
        let compatibility = save_compatibility(&self.activated_project, &prepared.checkpoint)?;
        let receipt = self.save_store.commit_world_checkpoint_with_streaming(
            compatibility,
            &prepared.checkpoint,
            &prepared.streaming,
        )?;
        let loaded = self.save_store.load_latest(&save_compatibility(
            &self.activated_project,
            &prepared.checkpoint,
        )?)?;
        if loaded.image.manifest.generation != receipt.generation {
            return Err(ApplicationError::FinalSaveFailed);
        }
        let generation_hash = save_identity(&loaded.image.manifest)?.0;
        let plan = self.machine.plan_state_publication(
            Some(prepared.summary.authoritative_revision),
            Some(generation_hash),
        )?;
        self.publish_state_plan(plan)?;
        Ok(generation_hash)
    }

    pub fn close(&mut self) -> Result<ApplicationCloseOutcomeV2, ApplicationError> {
        if self.machine.state().state == ApplicationSessionStatusV1::Closed {
            return self.closed_outcome();
        }
        if self.durable.close_journal.is_none() {
            self.prepare_close()?;
        }
        self.publish_prepared_close_save()?;
        self.finish_close()
    }

    fn prepare_close(&mut self) -> Result<(), ApplicationError> {
        if !matches!(
            self.machine.state().state,
            ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended
        ) {
            return Err(ApplicationError::CloseStateInvalid);
        }
        self.flush_reference_game_live_checkpoint()?;
        self.ensure_prepared_run()?;
        let state = self.machine.state();
        let causal_hash = domain_hash(
            b"nextengine.close-cause.v2\0",
            &[state.session_id.as_bytes(), &state.revision.to_le_bytes()],
        );
        let request = CloseSessionRequestV2::new(
            derive_close_request_id(state.session_id, state.revision, causal_hash),
            state.session_id,
            state.revision,
            state.state,
            LifecycleReasonV1 {
                kind: LifecycleReasonKindV1::UserCloseRequested,
                reason_code: SchemaId::new("nextengine.session.close-requested")?,
            },
            CausalInputReferenceV1 {
                source_kind: CausalInputSourceKindV1::System,
                canonical_hash: causal_hash,
            },
        )?;
        self.durable.close_request = Some(request.clone());
        self.publish_current(Some(self.current_generation))?;

        let lifecycle = self.lifecycle_request(
            ApplicationSessionStatusV1::Quiescing,
            LifecycleReasonKindV1::UserCloseRequested,
            "nextengine.session.quiescing",
        )?;
        self.publish_transition(lifecycle, SessionTransitionReferencesV1::default())?;

        let prepared = self
            .prepared_run
            .as_ref()
            .ok_or(ApplicationError::NoRunOutcome)?;
        let image = self.save_store.prepare_world_checkpoint_with_streaming(
            save_compatibility(&self.activated_project, &prepared.checkpoint)?,
            &prepared.checkpoint,
            &prepared.streaming,
        )?;
        let image_hash = image.content_hash()?;
        self.durable.close_journal = Some(CloseSessionJournalV2::prepared(&request, image_hash));
        self.durable.prepared_save_image = Some(image);
        self.publish_current(Some(self.current_generation))
    }

    fn publish_prepared_close_save(&mut self) -> Result<(), ApplicationError> {
        let journal = self
            .durable
            .close_journal
            .clone()
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        if journal.stage == CloseSessionJournalStageV2::SavePublished {
            return Ok(());
        }
        let image = self
            .durable
            .prepared_save_image
            .clone()
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        let receipt = self.save_store.commit_prepared_image(&image)?;
        if receipt.generation != image.manifest.generation {
            return Err(ApplicationError::FinalSaveFailed);
        }
        let save_generation_hash = save_identity(&image.manifest)?.0;
        self.durable.close_journal = Some(journal.save_published(save_generation_hash)?);
        self.durable.prepared_save_image = None;
        self.publish_current(Some(self.current_generation))
    }

    fn finish_close(&mut self) -> Result<ApplicationCloseOutcomeV2, ApplicationError> {
        let request = self
            .durable
            .close_request
            .clone()
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        let save_generation_hash = self
            .durable
            .close_journal
            .as_ref()
            .and_then(|journal| journal.save_generation_hash)
            .ok_or(ApplicationError::CloseJournalInvalid)?;
        if self.machine.state().state == ApplicationSessionStatusV1::Quiescing {
            let lifecycle = self.lifecycle_request(
                ApplicationSessionStatusV1::Finalizing,
                LifecycleReasonKindV1::FinalSaveReady,
                "nextengine.session.finalizing",
            )?;
            self.publish_transition(
                lifecycle,
                SessionTransitionReferencesV1 {
                    save_receipt_hash: Some(save_generation_hash),
                    active_save_generation_hash: Some(save_generation_hash),
                    ..SessionTransitionReferencesV1::default()
                },
            )?;
        }
        if self.machine.state().state == ApplicationSessionStatusV1::Finalizing {
            let receipt = CloseSessionReceiptV2::new(
                request.close_request_id,
                request.session_id,
                save_generation_hash,
            );
            self.durable.close_receipt = Some(receipt.clone());
            self.publish_current(Some(self.current_generation))?;
            let lifecycle = self.lifecycle_request(
                ApplicationSessionStatusV1::Closed,
                LifecycleReasonKindV1::FinalSaveReady,
                "nextengine.session.closed",
            )?;
            self.publish_transition(
                lifecycle,
                SessionTransitionReferencesV1 {
                    terminal_receipt_hash: Some(receipt.canonical_hash),
                    active_save_generation_hash: Some(save_generation_hash),
                    ..SessionTransitionReferencesV1::default()
                },
            )?;
        }
        self.closed_outcome()
    }

    fn closed_outcome(&self) -> Result<ApplicationCloseOutcomeV2, ApplicationError> {
        let receipt = self
            .durable
            .close_receipt
            .as_ref()
            .ok_or(ApplicationError::TerminalReceiptMissing)?;
        Ok(ApplicationCloseOutcomeV2::Closed {
            receipt_hash: receipt.canonical_hash,
            save_generation_hash: receipt.save_generation_hash,
        })
    }
}

pub(crate) fn save_compatibility(
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

pub(crate) fn save_identity(
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
