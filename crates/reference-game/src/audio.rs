//! Reference-game baseline audio fixtures: engine-owned synthesized PCM
//! clips and the event/listener binding profiles consumed by the live loop
//! (SPEC-08 baseline audio; clips cook through the A1 `NeutralAudioV1`
//! contract).
//!
//! All clips are deterministic integer synthesis — no external source files,
//! no float waveform evaluation — so the same bytes cook on every host and
//! the existing CC0 generated-content provenance covers them.

use std::collections::BTreeMap;

use next_contracts::audio::{AudioLoudnessMetadataV1, AudioPcmEncodingV1, NeutralAudioV1};
use next_contracts::ids::{AssetId, SchemaId};
use next_contracts::presentation::audio_scene::{
    AudioLoudnessClassV1, AudioPriorityClassV1, AudioSceneSnapshotV1,
};
use next_contracts::project::{ActivatedProjectV2, AssetRevisionRefV1};
use next_contracts::rpg::{
    RPG_EVENT_CHARACTER_RESOURCE_ADJUSTED_SCHEMA_ID, RPG_EVENT_DIALOGUE_ADVANCED_SCHEMA_ID,
    RPG_EVENT_INTERACTIVE_OBJECT_TRANSITIONED_SCHEMA_ID, RPG_EVENT_ITEM_TRANSFERRED_V1_SCHEMA_ID,
};
use next_presentation::audio_mix::AudioMixProfileV1;
use next_presentation::audio_scene::{
    AudioCueEmitterSubjectV1, AudioEventCueBindingV1, AudioListenerBindingV1,
};

use crate::error::ReferenceGameError;
use crate::session::ReferenceGameSession;

pub const REFERENCE_SWITCH_CLIP_ASSET_ID: AssetId = AssetId::from_bytes([0xa1; 16]);
pub const REFERENCE_PICKUP_CLIP_ASSET_ID: AssetId = AssetId::from_bytes([0xa2; 16]);
pub const REFERENCE_MELEE_CLIP_ASSET_ID: AssetId = AssetId::from_bytes([0xa3; 16]);
pub const REFERENCE_DIALOGUE_CLIP_ASSET_ID: AssetId = AssetId::from_bytes([0xa4; 16]);

/// Text ID of the dialogue-accept subtitle (SPEC-08 voice-absent fallback).
pub const REFERENCE_DIALOGUE_SUBTITLE_TEXT_ID: &str = "nextengine.ui.text.subtitle.dialogue-accept";

const CLIP_SAMPLE_RATE: u32 = 48_000;
const CLIP_REVISION: u64 = 1;

/// Builds the four engine-owned baseline clips as neutral audio records.
#[must_use]
pub fn reference_audio_records() -> Vec<NeutralAudioV1> {
    vec![
        build_clip(
            REFERENCE_SWITCH_CLIP_ASSET_ID,
            &synthesize_noise_burst(4_800, 9_000, 0x5eed_1234),
        ),
        build_clip(
            REFERENCE_PICKUP_CLIP_ASSET_ID,
            &synthesize_two_tone(9_600, 55, 36, 8_000),
        ),
        build_clip(
            REFERENCE_MELEE_CLIP_ASSET_ID,
            &synthesize_thud(7_200, 480, 12_000),
        ),
        build_clip(
            REFERENCE_DIALOGUE_CLIP_ASSET_ID,
            &synthesize_two_tone(3_600, 73, 73, 5_000),
        ),
    ]
}

/// Maps cooked audio clips by asset ID for the mixer resolution boundary.
pub fn reference_audio_clip_map(
    activated_project: &ActivatedProjectV2,
) -> BTreeMap<AssetId, NeutralAudioV1> {
    activated_project
        .audio_clips
        .iter()
        .map(|clip| (clip.asset_id, clip.clone()))
        .collect()
}

/// Event→clip cue binding profile for the reference fixture. Every binding
/// resolves its exact clip revision from the activated content manifest;
/// a missing clip fails closed before any live tick.
pub fn reference_audio_cue_bindings(
    activated_project: &ActivatedProjectV2,
) -> Result<Vec<AudioEventCueBindingV1>, ReferenceGameError> {
    let revision = |asset_id: AssetId| {
        activated_project
            .content_manifest
            .body
            .asset_entries
            .iter()
            .find(|entry| entry.asset_revision.asset_id == asset_id)
            .map(|entry| entry.asset_revision)
            .ok_or(ReferenceGameError::AudioAssetMissing)
    };
    let binding = |schema_id: &str,
                   clip_revision: AssetRevisionRefV1,
                   loudness: AudioLoudnessClassV1,
                   priority: AudioPriorityClassV1,
                   subject: AudioCueEmitterSubjectV1,
                   subtitle_text_id_or_none: Option<&str>|
     -> Result<AudioEventCueBindingV1, ReferenceGameError> {
        Ok(AudioEventCueBindingV1 {
            event_schema_id: SchemaId::new(schema_id)?,
            clip_revision,
            loudness_class: loudness,
            priority_class: priority,
            occlusion_zone_or_none: None,
            emitter_subject: subject,
            subtitle_text_id_or_none: subtitle_text_id_or_none.map(SchemaId::new).transpose()?,
        })
    };
    Ok(vec![
        binding(
            RPG_EVENT_INTERACTIVE_OBJECT_TRANSITIONED_SCHEMA_ID,
            revision(REFERENCE_SWITCH_CLIP_ASSET_ID)?,
            AudioLoudnessClassV1::Normal,
            AudioPriorityClassV1::Normal,
            AudioCueEmitterSubjectV1::EventPrincipal,
            None,
        )?,
        binding(
            RPG_EVENT_ITEM_TRANSFERRED_V1_SCHEMA_ID,
            revision(REFERENCE_PICKUP_CLIP_ASSET_ID)?,
            AudioLoudnessClassV1::Normal,
            AudioPriorityClassV1::High,
            AudioCueEmitterSubjectV1::EventPrincipal,
            None,
        )?,
        binding(
            RPG_EVENT_CHARACTER_RESOURCE_ADJUSTED_SCHEMA_ID,
            revision(REFERENCE_MELEE_CLIP_ASSET_ID)?,
            AudioLoudnessClassV1::Loud,
            AudioPriorityClassV1::High,
            AudioCueEmitterSubjectV1::EventPrincipal,
            None,
        )?,
        binding(
            RPG_EVENT_DIALOGUE_ADVANCED_SCHEMA_ID,
            revision(REFERENCE_DIALOGUE_CLIP_ASSET_ID)?,
            AudioLoudnessClassV1::Quiet,
            AudioPriorityClassV1::Normal,
            AudioCueEmitterSubjectV1::Listener,
            Some(REFERENCE_DIALOGUE_SUBTITLE_TEXT_ID),
        )?,
    ])
}

/// Listener binding anchored to the exact player capsule body.
#[must_use]
pub fn reference_audio_listener_binding(fixture: &ReferenceGameSession) -> AudioListenerBindingV1 {
    AudioListenerBindingV1 {
        listener_id: fixture.body_id,
        physics_body_id: Some(fixture.physics_body_id),
        fallback_transform: next_contracts::presentation::QuantizedPresentationTransformV1::default(
        ),
    }
}

/// Locked baseline mix profile for the reference game (48 kHz stereo, 30 Hz
/// tick window).
pub fn reference_audio_mix_profile() -> Result<AudioMixProfileV1, ReferenceGameError> {
    Ok(AudioMixProfileV1::stereo_baseline_v1()?)
}

/// How many simulation ticks a speech cue's subtitle stays on the HUD.
pub const REFERENCE_SUBTITLE_WINDOW_TICKS: u64 = 30;

/// Resolves the active subtitle for one published audio scene: the most
/// recent (canonical last) cue whose binding declares speech text. Returns
/// the exact text ID to resolve through the locale fallback chain; never a
/// rendered string (ADR-044/SPEC-18).
#[must_use]
pub fn active_subtitle_text_id(
    scene: &AudioSceneSnapshotV1,
    cue_bindings: &[AudioEventCueBindingV1],
) -> Option<SchemaId> {
    scene.cues.iter().rev().find_map(|cue| {
        cue_bindings
            .iter()
            .find(|binding| binding.event_schema_id == cue.source_event_schema_id)
            .and_then(|binding| binding.subtitle_text_id_or_none.clone())
    })
}

fn build_clip(asset_id: AssetId, samples: &[i16]) -> NeutralAudioV1 {
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    let mut peak = 0_u32;
    let mut energy = 0_u64;
    for sample in samples {
        pcm.extend_from_slice(&sample.to_le_bytes());
        let magnitude = u32::from(sample.unsigned_abs());
        peak = peak.max(magnitude);
        energy += u64::from(magnitude);
    }
    // Q16.16 fixed-point metadata: peak relative to full scale; integrated
    // loudness approximated by mean magnitude (authored baseline value).
    let peak_q16_16 = peak * 65_536 / 32_767;
    let sample_count = u64::try_from(samples.len()).expect("clip length fits");
    let mean = energy / sample_count.max(1);
    let integrated_q16_16 = i32::try_from(mean * 65_536 / 32_767).unwrap_or(i32::MAX) - 65_536;
    NeutralAudioV1::new(
        asset_id,
        CLIP_REVISION,
        CLIP_SAMPLE_RATE,
        1,
        AudioPcmEncodingV1::PcmS16Le,
        u64::try_from(samples.len()).expect("clip length fits"),
        None,
        Vec::new(),
        AudioLoudnessMetadataV1::new(integrated_q16_16, peak_q16_16).expect("loudness"),
        pcm,
    )
    .expect("reference clip is valid")
}

/// Linear decay envelope helper: amplitude scales from `amplitude` to zero
/// over the clip with truncating integer math.
fn envelope(amplitude: i32, index: usize, total: usize) -> i32 {
    let total = i32::try_from(total).unwrap_or(i32::MAX).max(1);
    let index = i32::try_from(index).unwrap_or(i32::MAX);
    amplitude * (total - index) / total
}

/// Deterministic xorshift noise burst with linear decay (switch/interact).
fn synthesize_noise_burst(frames: usize, amplitude: i32, seed: u32) -> Vec<i16> {
    let mut state = seed.max(1);
    (0..frames)
        .map(|index| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let noise = i32::from((state & 0xFFFF) as u16) - 32_767;
            let sample = noise * envelope(amplitude, index, frames) / 32_767;
            sample.clamp(-32_767, 32_767) as i16
        })
        .collect()
}

/// Two sequential square-wave tones with linear decay (pickup/dialogue).
fn synthesize_two_tone(
    frames: usize,
    first_period: usize,
    second_period: usize,
    amplitude: i32,
) -> Vec<i16> {
    (0..frames)
        .map(|index| {
            let period = if index * 2 < frames {
                first_period.max(2)
            } else {
                second_period.max(2)
            };
            let phase = (index % period) * 2 < period;
            let wave = if phase { 1_i32 } else { -1_i32 };
            let sample = wave * envelope(amplitude, index, frames);
            sample.clamp(-32_767, 32_767) as i16
        })
        .collect()
}

/// Low-frequency square thud with fast quadratic decay (melee hit).
fn synthesize_thud(frames: usize, period: usize, amplitude: i32) -> Vec<i16> {
    (0..frames)
        .map(|index| {
            let phase = (index % period.max(2)) * 2 < period.max(2);
            let wave = if phase { 1_i32 } else { -1_i32 };
            let linear = envelope(amplitude, index, frames);
            let sample = wave * linear * linear / amplitude.max(1);
            sample.clamp(-32_767, 32_767) as i16
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_are_deterministic_and_within_bounds() {
        let first = reference_audio_records();
        let second = reference_audio_records();
        assert_eq!(first, second);
        assert_eq!(first.len(), 4);
        for clip in &first {
            assert_eq!(clip.sample_rate_hz, CLIP_SAMPLE_RATE);
            assert_eq!(clip.channel_count, 1);
            assert!(clip.frame_count > 0);
            assert_eq!(
                clip.pcm_bytes().len() as u64,
                clip.frame_count * 2,
                "mono S16 PCM length matches frame count"
            );
            let bytes = clip.canonical_bytes().expect("encode");
            assert_eq!(
                NeutralAudioV1::from_canonical_bytes(
                    &bytes,
                    next_contracts::canonical::CanonicalDecodeLimits::default(),
                )
                .expect("decode"),
                *clip
            );
        }
    }

    #[test]
    fn synthesis_is_non_silent_and_distinct() {
        let noise = synthesize_noise_burst(4_800, 9_000, 0x5eed_1234);
        let tone = synthesize_two_tone(9_600, 55, 36, 8_000);
        let thud = synthesize_thud(7_200, 480, 12_000);
        for samples in [&noise, &tone, &thud] {
            assert!(samples.iter().any(|sample| *sample != 0));
            assert!(
                samples
                    .iter()
                    .all(|sample| sample.saturating_abs() <= 12_000)
            );
        }
        assert_ne!(noise[..100], tone[..100]);
        assert_ne!(tone[..100], thud[..100]);
        // Envelopes decay to silence at the end of each clip.
        assert_eq!(noise[4_799], 0);
        assert_eq!(thud[7_199], 0);
    }
}
