//! Baseline audio operations of the live reference driver: per-tick audio
//! scene publication, canonical PCM mixing and read-only accessors.

use std::sync::Arc;

use next_contracts::command::DomainEventEnvelopeV2;
use next_contracts::ids::SchemaId;
use next_contracts::presentation::audio_scene::AudioSceneSnapshotV1;
use next_presentation::audio_mix::AudioMixerV1;
use next_presentation::audio_scene::extract_audio_scene;

use crate::ReferenceGameError;

use super::ReferenceGameDriverV2;

impl ReferenceGameDriverV2 {
    /// Latest published immutable audio scene for this driver.
    #[must_use]
    pub fn audio_scene(&self) -> &AudioSceneSnapshotV1 {
        &self.audio_scene
    }

    /// Canonical stereo PCM window mixed for the latest tick
    /// (`AudioMixProfileV1::frames_per_tick` stereo frames, S16LE order).
    #[must_use]
    pub fn audio_mixed_pcm(&self) -> &[i16] {
        &self.audio_pcm
    }

    /// Shared ownership of the latest canonical PCM window for handoff to a
    /// presentation-only device adapter.
    #[must_use]
    pub fn audio_pcm_shared(&self) -> Arc<[i16]> {
        Arc::clone(&self.audio_pcm)
    }

    /// Presentation-only mixer diagnostics (voice counts, drops, clipping).
    #[must_use]
    pub const fn audio_mixer(&self) -> &AudioMixerV1 {
        &self.audio_mixer
    }

    /// Subtitle text ID active at `tick`, if a recent speech cue published
    /// one within the subtitle window (A5 voice-absent fallback).
    #[must_use]
    pub fn current_audio_subtitle(&self, tick: u64) -> Option<SchemaId> {
        self.audio_subtitle_or_none
            .as_ref()
            .and_then(|(text_id, until)| (tick <= *until).then(|| text_id.clone()))
    }

    /// Publishes one immutable audio scene for the current tick and mixes the
    /// exact tick window. Mixer state is presentation-only and never enters
    /// gameplay hashes.
    pub(crate) fn publish_audio(
        &mut self,
        events: &[DomainEventEnvelopeV2],
        simulation_tick: u64,
    ) -> Result<(), ReferenceGameError> {
        // Plan 34: the water records of the committed checkpoint keep the
        // flow loops alive across a republication; no splash trigger here
        // (the speeds do not change without a tick).
        let water_checkpoint = self.runtime.physics_checkpoint();
        let water_edges = crate::water_presentation::water_audio_edge_records(
            &water_checkpoint.water_volumes,
            &water_checkpoint.water_flow,
            simulation_tick,
        );
        let water_emitters =
            crate::water_audio::flow_emitters(&water_edges, &self.water_audio_clips);
        let scene = extract_audio_scene(
            self.presentation_extractor.snapshot_epoch(),
            self.next_audio_sequence,
            simulation_tick,
            &self.audio_listener_binding,
            &water_emitters,
            &self.audio_cue_bindings,
            events,
            self.runtime.physics_snapshot(),
        )?;
        let mut pcm = self.audio_mixer.mix_tick(&scene, &self.audio_clips);
        let camera_translation = self
            .camera_binding()?
            .current_result_sample
            .pose
            .translation_micrometres;
        let submerged = crate::water_audio::listener_submerged(
            &water_checkpoint.water_volumes,
            camera_translation,
            simulation_tick,
        );
        crate::water_audio::low_pass_in_place(&mut pcm, submerged, &mut self.water_audio);
        #[cfg(feature = "physical-sound-lab")]
        let pcm = {
            let physical_pcm = if self.physical_sound_lab_enabled {
                self.physical_sound_lab.mix_tick(&[])
            } else {
                vec![0; pcm.len()]
            };
            let mut mixed = pcm;
            next_presentation::physical_sound_lab::mix_physical_sound_in_place(
                &mut mixed,
                &physical_pcm,
            );
            mixed
        };
        if let Some(text_id) =
            crate::audio::active_subtitle_text_id(&scene, &self.audio_cue_bindings)
        {
            self.audio_subtitle_or_none = Some((
                text_id,
                simulation_tick.saturating_add(crate::audio::REFERENCE_SUBTITLE_WINDOW_TICKS),
            ));
        }
        self.audio_scene = scene;
        self.audio_pcm = Arc::from(pcm);
        self.next_audio_sequence = self
            .next_audio_sequence
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        Ok(())
    }
}
