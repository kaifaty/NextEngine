//! Explicit save-load cut for the interactive reference driver.

use super::*;

pub(super) fn initial_chunk_id(fixture: &ReferenceGameSession) -> SchemaId {
    fixture.world_topology().initial_chunk_id().clone()
}

impl ReferenceGameDriverV2 {
    /// Builds a fully validated candidate driver from a published save world.
    /// The current driver is unchanged until the application coordinator has
    /// atomically published the candidate recovery closure.
    pub fn load_saved_world_candidate(
        &self,
        checkpoint: WorldCheckpointV4,
        world_streaming_snapshot: WorldStreamingSnapshotV1,
        world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
        world_population_snapshot: WorldPopulationSnapshotV1,
        world_activity_snapshot: WorldActivitySnapshotV1,
        agent_cognition_snapshot: AgentCognitionSnapshotV1,
        agent_memory_snapshot: AgentMemorySnapshotV1,
    ) -> Result<Self, ReferenceGameError> {
        checkpoint.validate()?;
        world_streaming_snapshot.validate()?;
        let loaded_state_root =
            next_contracts::snapshot::world_checkpoint_with_systemic_cognition_v1_state_root(
                &checkpoint.runtime_snapshot,
                &checkpoint.rpg_snapshot,
                &checkpoint.physics_checkpoint,
                &world_streaming_snapshot,
                world_routine_snapshot_or_none.as_ref(),
                &world_population_snapshot,
                &world_activity_snapshot,
                &agent_cognition_snapshot,
                &agent_memory_snapshot,
            )?;
        let next_logical_frame_sequence = checkpoint.runtime_snapshot.next_tick;
        let events = checkpoint.runtime_snapshot.committed_event_count;
        let fixture = self.fixture.clone();
        // The input configuration is authoritative save state. Dialogue and
        // other modal transitions can advance the gameplay context revision,
        // so rebasing to the project's initial fixture would make the first
        // post-load frame stale against the restored runtime registry even
        // when both stacks represent gameplay. Preserve the exact loaded
        // binding while replacing only run-local host/control state.
        let loaded_player_binding = checkpoint
            .runtime_snapshot
            .player_controller_registry
            .bindings
            .get(&fixture.source_id)
            .cloned()
            .ok_or(ReferenceGameError::RecoveryInvalid)?;
        if loaded_player_binding.controller_id != self.input.controller_id()
            || loaded_player_binding.source_id != self.input.source_id()
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        let runtime = RuntimeState::restore_world_checkpoint_with_definitions(
            checkpoint,
            fixture.authority.clone(),
            fixture.bootstrap.rpg_definitions.clone(),
        )?;
        let world_streamer = WorldStreamerV1::restore(
            fixture.activated_project.clone(),
            self.content_generation.clone(),
            world_streaming_snapshot,
        )?;
        let world_routine = WorldRoutineOwnerV1::restore(
            fixture.activated_project.world_routine_catalog_or_none,
            world_routine_snapshot_or_none,
            runtime.next_tick(),
        )?;
        let world_population = WorldPopulationOwnerV1::restore(
            fixture.activated_project.world_population_catalog.clone(),
            fixture.activated_project.world_navigation_catalog.clone(),
            world_population_snapshot,
            runtime.next_tick(),
        )?;
        let cognition = next_agent::cognition::StrategicAgentOwnersV1::restore(
            fixture.activated_project.agent_cognition_catalog.clone(),
            agent_cognition_snapshot,
            agent_memory_snapshot,
        )?;
        let world_activity = WorldActivityOwnerV1::restore(
            fixture.activated_project.world_activity_catalog.clone(),
            world_activity_snapshot,
            runtime.next_tick(),
        )?;
        runtime.validate_world_routine_ledger_closure(&world_routine)?;
        runtime.validate_world_population_ledger_closure(&world_population)?;
        let presentation_bindings =
            fixture_presentation_bindings(&fixture, &runtime.rpg_snapshot())?;

        let mut input = self.input.clone();
        input.rebase_for_loaded_world(
            loaded_player_binding.action_map,
            loaded_player_binding.context_stack,
            next_logical_frame_sequence,
        )?;

        let prior_presentation = self.presentation_snapshot()?;
        let mut epoch_preimage = Vec::with_capacity(72);
        epoch_preimage.extend_from_slice(prior_presentation.snapshot_epoch.as_bytes());
        epoch_preimage.extend_from_slice(&prior_presentation.snapshot_sequence.to_le_bytes());
        epoch_preimage.extend_from_slice(loaded_state_root.as_bytes());
        let snapshot_epoch = next_contracts::project::domain_hash(
            "nextengine.presentation-save-load-epoch.v1",
            &epoch_preimage,
        );
        if snapshot_epoch == prior_presentation.snapshot_epoch {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        let presentation_extractor =
            PresentationExtractorV1::new_with_snapshot_epoch_and_ui_batch_limits(
                snapshot_epoch,
                reference_b0_presentation_profile_hash(),
                8,
                1,
                SEMANTIC_UI_RECORDS_PER_BATCH,
            )?;
        let audio_listener_binding = self.audio_listener_binding;
        let audio_mixer = AudioMixerV1::new(crate::audio::reference_audio_mix_profile()?);
        let audio_scene = extract_audio_scene(
            snapshot_epoch,
            0,
            runtime.next_tick(),
            &audio_listener_binding,
            &[],
            &self.audio_cue_bindings,
            &[],
            runtime.physics_snapshot(),
        )?;
        let mut driver = Self {
            fixture,
            content_generation: self.content_generation.clone(),
            runtime,
            world_routine,
            world_population,
            world_activity,
            cognition,
            world_streamer,
            input,
            presentation_bindings,
            presentation_extractor,
            audio_clips: Arc::clone(&self.audio_clips),
            audio_cue_bindings: Arc::clone(&self.audio_cue_bindings),
            audio_listener_binding,
            audio_mixer,
            audio_scene,
            audio_pcm: Arc::from(Vec::new()),
            next_audio_sequence: 0,
            audio_subtitle_or_none: None,
            next_logical_frame_sequence,
            events,
            // This is a run-local report counter, not authoritative save
            // state. Historical event payloads are deliberately absent from
            // SaveManifestV2, so counting restarts at the explicit load cut.
            rpg_events: 0,
            camera_yaw_millidegrees: self.camera_yaw_millidegrees,
            camera_pitch_millidegrees: self.camera_pitch_millidegrees,
            camera_cut: true,
            ui_screen: ReferenceUiScreenV1::None,
            dialogue: ReferenceDialogueUiV1::Closed,
            dialogue_entry_node_id: self.dialogue_entry_node_id.clone(),
        };
        driver.publish_presentation()?;
        let loaded = driver.presentation_snapshot()?;
        if loaded.snapshot_epoch != snapshot_epoch
            || loaded.snapshot_sequence != 0
            || loaded.simulation_tick != next_logical_frame_sequence
            || loaded.camera_records().any(|camera| !camera.cut)
        {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        Ok(driver)
    }
}
