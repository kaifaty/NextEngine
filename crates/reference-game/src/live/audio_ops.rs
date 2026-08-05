//! Baseline audio operations of the live reference driver: per-tick audio
//! scene publication, canonical PCM mixing and read-only accessors.
//! Kept out of `live.rs` to respect the 1000-line source-file limit.

use std::sync::Arc;

use next_contracts::command::DomainEventEnvelopeV2;
use next_contracts::presentation::audio_scene::AudioSceneSnapshotV1;
use next_presentation::audio_mix::AudioMixerV1;
use next_presentation::audio_scene::extract_audio_scene;

use crate::ReferenceGameError;

use super::ReferenceGameDriverV1;

impl ReferenceGameDriverV1 {
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

    /// Publishes one immutable audio scene for the current tick and mixes the
    /// exact tick window. Mixer state is presentation-only and never enters
    /// gameplay hashes.
    pub(crate) fn publish_audio(
        &mut self,
        events: &[DomainEventEnvelopeV2],
        simulation_tick: u64,
    ) -> Result<(), ReferenceGameError> {
        let scene = extract_audio_scene(
            self.presentation_extractor.snapshot_epoch(),
            self.next_audio_sequence,
            simulation_tick,
            &self.audio_listener_binding,
            &[],
            &self.audio_cue_bindings,
            events,
            self.runtime.physics_snapshot(),
        )?;
        let pcm = self.audio_mixer.mix_tick(&scene, &self.audio_clips);
        self.audio_scene = scene;
        self.audio_pcm = Arc::from(pcm);
        self.next_audio_sequence = self
            .next_audio_sequence
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        Ok(())
    }
}
