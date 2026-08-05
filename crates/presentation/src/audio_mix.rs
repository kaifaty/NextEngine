//! Deterministic baseline software audio mixer and canonical PCM/WAV sink
//! (SPEC-08 baseline audio, AUDIO-02 displayless check).
//!
//! The mixer consumes the immutable `AudioSceneSnapshotV1` projection and
//! cooked `NeutralAudioV1` clips. Voice admission is bounded by priority
//! class with deterministic preemption; attenuation, panning, resampling and
//! summing are integer/fixed-point only, so the produced PCM is byte-exact
//! for the same scene sequence on every host. Mixer state is presentation
//! only: it is reconstructible from the committed scene history, it never
//! writes to simulation, and gameplay hashes never depend on it.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::audio::{AudioPcmEncodingV1, NeutralAudioV1};
use next_contracts::ids::{AssetId, ContentHash, SchemaId};
use next_contracts::presentation::audio_scene::{
    AudioEmitterKeyV1, AudioLoudnessClassV1, AudioPriorityClassV1, AudioSceneSnapshotV1,
};

const Q16_ONE: i64 = 65_536;
/// Baseline output layout is stereo; mono clips are panned into the stereo
/// field and stereo clips are downmixed first.
pub const AUDIO_MIX_CHANNEL_COUNT: u32 = 2;
pub const AUDIO_MIX_MAX_VOICES_LIMIT: usize = 64;

/// Locked mixing profile: output rate/window, voice bound and the exact
/// distance/pan/occlusion model constants. All fields are engine-owned;
/// device capabilities never select another profile at runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioMixProfileV1 {
    pub sample_rate_hz: u32,
    pub frames_per_tick: u32,
    pub max_voices: usize,
    pub min_distance_micrometres: i64,
    pub max_distance_micrometres: i64,
    pub pan_range_micrometres: i64,
    pub quiet_gain_q16_16: u32,
    pub occlusion_gain_q16_16: u32,
    pub listener_zone_or_none: Option<SchemaId>,
}

impl AudioMixProfileV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the locked mix profile keeps every model constant explicit"
    )]
    pub fn new(
        sample_rate_hz: u32,
        frames_per_tick: u32,
        max_voices: usize,
        min_distance_micrometres: i64,
        max_distance_micrometres: i64,
        pan_range_micrometres: i64,
        quiet_gain_q16_16: u32,
        occlusion_gain_q16_16: u32,
        listener_zone_or_none: Option<SchemaId>,
    ) -> Result<Self, AudioMixErrorV1> {
        if !(8_000..=192_000).contains(&sample_rate_hz)
            || frames_per_tick == 0
            || frames_per_tick > sample_rate_hz
            || max_voices == 0
            || max_voices > AUDIO_MIX_MAX_VOICES_LIMIT
            || min_distance_micrometres <= 0
            || max_distance_micrometres <= min_distance_micrometres
            || pan_range_micrometres <= 0
            || quiet_gain_q16_16 > 65_536
            || occlusion_gain_q16_16 > 65_536
        {
            return Err(AudioMixErrorV1::InvalidProfile);
        }
        Ok(Self {
            sample_rate_hz,
            frames_per_tick,
            max_voices,
            min_distance_micrometres,
            max_distance_micrometres,
            pan_range_micrometres,
            quiet_gain_q16_16,
            occlusion_gain_q16_16,
            listener_zone_or_none,
        })
    }

    /// Reference baseline: 48 kHz stereo, 1 600 frames per 30 Hz tick,
    /// 16 voices, full gain inside 1 m, silence past 20 m, full pan at 5 m.
    pub fn stereo_baseline_v1() -> Result<Self, AudioMixErrorV1> {
        Self::new(
            48_000, 1_600, 16, 1_000_000, 20_000_000, 5_000_000, 32_768, 32_768, None,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum AudioVoiceKeyV1 {
    Cue(ContentHash),
    Emitter(AudioEmitterKeyV1),
}

#[derive(Clone, Debug)]
struct AudioVoiceV1 {
    key: AudioVoiceKeyV1,
    emitter_key: AudioEmitterKeyV1,
    clip_asset_id: AssetId,
    position_q16_16: u64,
    step_q16_16: u64,
    priority_class: AudioPriorityClassV1,
    loudness_class: AudioLoudnessClassV1,
    looped: bool,
    admission_ordinal: u64,
}

/// Bounded deterministic voice admission and summing state. Reconstructible
/// by replaying the committed scene sequence; never checkpointed as gameplay
/// state and never read back into simulation.
#[derive(Clone, Debug)]
pub struct AudioMixerV1 {
    profile: AudioMixProfileV1,
    voices: Vec<AudioVoiceV1>,
    next_admission_ordinal: u64,
    mixed_ticks: u64,
    preempted_voices: u64,
    dropped_cues: u64,
    clipped_samples: u64,
}

impl AudioMixerV1 {
    pub fn new(profile: AudioMixProfileV1) -> Self {
        Self {
            profile,
            voices: Vec::new(),
            next_admission_ordinal: 0,
            mixed_ticks: 0,
            preempted_voices: 0,
            dropped_cues: 0,
            clipped_samples: 0,
        }
    }

    #[must_use]
    pub const fn profile(&self) -> &AudioMixProfileV1 {
        &self.profile
    }

    #[must_use]
    pub const fn mixed_ticks(&self) -> u64 {
        self.mixed_ticks
    }

    #[must_use]
    pub const fn active_voice_count(&self) -> usize {
        self.voices.len()
    }

    #[must_use]
    pub const fn preempted_voices(&self) -> u64 {
        self.preempted_voices
    }

    #[must_use]
    pub const fn dropped_cues(&self) -> u64 {
        self.dropped_cues
    }

    #[must_use]
    pub const fn clipped_samples(&self) -> u64 {
        self.clipped_samples
    }

    /// Admits the scene's new one-shot cues and continuous emitters, applies
    /// bounded priority admission and sums exactly `frames_per_tick` stereo
    /// frames. `clips` resolves exact clip revisions from cooked content; an
    /// unresolvable or hash-mismatched clip skips its voice with a bounded
    /// diagnostic instead of failing presentation.
    pub fn mix_tick(
        &mut self,
        scene: &AudioSceneSnapshotV1,
        clips: &BTreeMap<AssetId, NeutralAudioV1>,
    ) -> Vec<i16> {
        self.admit_scene(scene, clips);
        self.retire_absent_emitters(scene);
        let frame_count = usize::try_from(self.profile.frames_per_tick).unwrap_or(0);
        let mut left = vec![0_i64; frame_count];
        let mut right = vec![0_i64; frame_count];
        let mut finished = Vec::new();
        let profile = &self.profile;
        for (voice_index, voice) in self.voices.iter_mut().enumerate() {
            let Some(clip) = clips.get(&voice.clip_asset_id) else {
                finished.push(voice_index);
                continue;
            };
            let (gain_q16, pan_q16) = voice_gain_and_pan(profile, voice, scene);
            let left_pan_q16 = (Q16_ONE - pan_q16) / 2;
            let right_pan_q16 = (Q16_ONE + pan_q16) / 2;
            let mut voice_finished = gain_q16 == 0;
            for frame in 0..frame_count {
                if voice_finished {
                    break;
                }
                if let Some(sample) = next_sample(clip, voice) {
                    let gained = (sample * gain_q16) >> 16;
                    left[frame] += (gained * left_pan_q16) >> 16;
                    right[frame] += (gained * right_pan_q16) >> 16;
                    voice.position_q16_16 += voice.step_q16_16;
                } else {
                    voice_finished = true;
                }
            }
            if voice_finished && !voice.looped {
                finished.push(voice_index);
            }
            if !voice.looped && !voice_finished && (voice.position_q16_16 >> 16) >= clip.frame_count
            {
                finished.push(voice_index);
            }
        }
        for index in finished.into_iter().rev() {
            self.voices.remove(index);
        }
        let mut output = Vec::with_capacity(frame_count * 2);
        for frame in 0..frame_count {
            for value in [left[frame], right[frame]] {
                let clamped = value.clamp(i64::from(i16::MIN), i64::from(i16::MAX));
                if clamped != value {
                    self.clipped_samples += 1;
                }
                output.push(clamped as i16);
            }
        }
        self.mixed_ticks += 1;
        output
    }

    fn admit_scene(
        &mut self,
        scene: &AudioSceneSnapshotV1,
        clips: &BTreeMap<AssetId, NeutralAudioV1>,
    ) {
        let mut candidates: Vec<AudioVoiceV1> = Vec::new();
        for cue in &scene.cues {
            let key = AudioVoiceKeyV1::Cue(cue.cue_id);
            if self.voices.iter().any(|voice| voice.key == key) {
                continue;
            }
            let Some(step) =
                resolve_clip_step(clips, cue.clip_revision, self.profile.sample_rate_hz)
            else {
                self.dropped_cues += 1;
                continue;
            };
            candidates.push(self.new_voice(
                key,
                cue.emitter_key,
                cue.clip_revision,
                step,
                cue.priority_class,
                cue.loudness_class,
                false,
            ));
        }
        for emitter in &scene.emitters {
            if !emitter.looped {
                continue;
            }
            let key = AudioVoiceKeyV1::Emitter(emitter.emitter_key);
            if self.voices.iter().any(|voice| voice.key == key) {
                continue;
            }
            let Some(step) =
                resolve_clip_step(clips, emitter.clip_revision, self.profile.sample_rate_hz)
            else {
                self.dropped_cues += 1;
                continue;
            };
            candidates.push(self.new_voice(
                key,
                emitter.emitter_key,
                emitter.clip_revision,
                step,
                emitter.priority_class,
                emitter.loudness_class,
                true,
            ));
        }
        if candidates.is_empty() {
            return;
        }
        let overflow = self.voices.len() + candidates.len() > self.profile.max_voices;
        if overflow {
            // Deterministic bounded admission: keep the highest priorities;
            // ties keep the earlier admission, then the canonical key. New
            // candidates always lose ties against active voices.
            let mut pool: Vec<(bool, AudioVoiceV1)> =
                self.voices.drain(..).map(|voice| (true, voice)).collect();
            pool.extend(candidates.into_iter().map(|voice| (false, voice)));
            pool.sort_by(|(left_active, left), (right_active, right)| {
                right
                    .priority_class
                    .cmp(&left.priority_class)
                    .then_with(|| right_active.cmp(left_active))
                    .then_with(|| left.admission_ordinal.cmp(&right.admission_ordinal))
                    .then_with(|| left.key.cmp(&right.key))
            });
            let kept = pool.split_off(pool.len().min(self.profile.max_voices));
            for (was_active, _) in kept {
                if was_active {
                    self.preempted_voices += 1;
                } else {
                    self.dropped_cues += 1;
                }
            }
            pool.sort_by_key(|(_, voice)| voice.admission_ordinal);
            self.voices = pool.into_iter().map(|(_, voice)| voice).collect();
        } else {
            self.voices.extend(candidates);
            self.voices.sort_by_key(|voice| voice.admission_ordinal);
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "voice admission keeps every canonical identity field explicit"
    )]
    fn new_voice(
        &mut self,
        key: AudioVoiceKeyV1,
        emitter_key: AudioEmitterKeyV1,
        clip_revision: next_contracts::project::AssetRevisionRefV1,
        step_q16_16: u64,
        priority_class: AudioPriorityClassV1,
        loudness_class: AudioLoudnessClassV1,
        looped: bool,
    ) -> AudioVoiceV1 {
        let ordinal = self.next_admission_ordinal;
        self.next_admission_ordinal += 1;
        AudioVoiceV1 {
            key,
            emitter_key,
            clip_asset_id: clip_revision.asset_id,
            position_q16_16: 0,
            step_q16_16,
            priority_class,
            loudness_class,
            looped,
            admission_ordinal: ordinal,
        }
    }

    fn retire_absent_emitters(&mut self, scene: &AudioSceneSnapshotV1) {
        self.voices.retain(|voice| {
            !voice.looped
                || scene
                    .emitters
                    .iter()
                    .any(|emitter| emitter.looped && emitter.emitter_key == voice.emitter_key)
        });
    }
}

fn voice_gain_and_pan(
    profile: &AudioMixProfileV1,
    voice: &AudioVoiceV1,
    scene: &AudioSceneSnapshotV1,
) -> (i64, i64) {
    let class_gain = match voice.loudness_class {
        AudioLoudnessClassV1::Quiet => i64::from(profile.quiet_gain_q16_16),
        AudioLoudnessClassV1::Normal | AudioLoudnessClassV1::Loud => Q16_ONE,
    };
    let Some(emitter) = scene
        .emitters
        .iter()
        .find(|emitter| emitter.emitter_key == voice.emitter_key)
    else {
        // Non-spatial voice: full class gain, centered.
        return (class_gain, 0);
    };
    let listener = scene.listener.transform.translation_micrometres;
    let emitter_position = emitter.transform.translation_micrometres;
    let relative = [
        emitter_position[0] - listener[0],
        emitter_position[1] - listener[1],
        emitter_position[2] - listener[2],
    ];
    let squared = relative.iter().fold(0_i128, |sum, value| {
        sum + i128::from(*value) * i128::from(*value)
    });
    let distance = i64::try_from(isqrt_u128(squared as u128)).unwrap_or(i64::MAX);
    let attenuation = if distance <= profile.min_distance_micrometres {
        Q16_ONE
    } else if distance >= profile.max_distance_micrometres {
        0
    } else {
        ((profile.max_distance_micrometres - distance) * Q16_ONE)
            / (profile.max_distance_micrometres - profile.min_distance_micrometres)
    };
    let occlusion = if emitter.occlusion_zone_or_none.is_some()
        && emitter.occlusion_zone_or_none != profile.listener_zone_or_none
    {
        i64::from(profile.occlusion_gain_q16_16)
    } else {
        Q16_ONE
    };
    let gain = (((class_gain * attenuation) >> 16) * occlusion) >> 16;
    let rotated = rotate_by_inverse_orientation(relative, scene.listener.transform.orientation_q30);
    let pan = (i128::from(rotated[0]) * i128::from(Q16_ONE)
        / i128::from(profile.pan_range_micrometres))
    .clamp(-i128::from(Q16_ONE), i128::from(Q16_ONE)) as i64;
    (gain, pan)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AudioMixErrorV1 {
    InvalidProfile,
}

impl Display for AudioMixErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProfile => formatter.write_str("audio mix profile is invalid"),
        }
    }
}

impl Error for AudioMixErrorV1 {}

/// Encodes one mixed PCM window as a canonical RIFF/WAVE PCM S16LE byte
/// stream: fixed 44-byte header, no optional chunks, little-endian fields.
/// This is the bounded canonical sink used by the displayless audio check.
#[must_use]
pub fn encode_canonical_wav(profile: &AudioMixProfileV1, samples: &[i16]) -> Vec<u8> {
    let data_bytes = u32::try_from(samples.len() * 2).unwrap_or(u32::MAX);
    let mut bytes = Vec::with_capacity(44 + samples.len() * 2);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&data_bytes.wrapping_add(36).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&(AUDIO_MIX_CHANNEL_COUNT as u16).to_le_bytes());
    bytes.extend_from_slice(&profile.sample_rate_hz.to_le_bytes());
    bytes.extend_from_slice(&(profile.sample_rate_hz * AUDIO_MIX_CHANNEL_COUNT * 2).to_le_bytes());
    bytes.extend_from_slice(&(AUDIO_MIX_CHANNEL_COUNT as u16 * 2).to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

fn resolve_clip_step(
    clips: &BTreeMap<AssetId, NeutralAudioV1>,
    revision: next_contracts::project::AssetRevisionRefV1,
    mix_rate: u32,
) -> Option<u64> {
    let clip = clips.get(&revision.asset_id)?;
    let record_hash = clip.record_sha256().ok()?;
    if record_hash != revision.record_sha256 {
        return None;
    }
    Some(u64::from(clip.sample_rate_hz) * 65_536 / u64::from(mix_rate))
}

/// Fetches the current resampled, downmixed S16 sample and wraps looped
/// voices into their loop region. Returns `None` when a one-shot voice is
/// finished.
fn next_sample(clip: &NeutralAudioV1, voice: &mut AudioVoiceV1) -> Option<i64> {
    let frame_total = clip.frame_count;
    let loop_end = clip
        .loop_region_or_none
        .map_or(frame_total, |region| region.end_frame);
    let loop_start = clip
        .loop_region_or_none
        .map_or(0, |region| region.start_frame);
    loop {
        let frame_index = voice.position_q16_16 >> 16;
        if frame_index >= loop_end {
            if voice.looped && loop_end > loop_start {
                voice.position_q16_16 -= (loop_end - loop_start) << 16;
                continue;
            }
            return None;
        }
        let fraction = (voice.position_q16_16 & 0xFFFF) as i64;
        let first = i64::from(frame_s16(clip, frame_index));
        let second_index = if frame_index + 1 >= loop_end {
            if voice.looped {
                loop_start
            } else {
                frame_index
            }
        } else {
            frame_index + 1
        };
        let second = i64::from(frame_s16(clip, second_index));
        return Some(first + (((second - first) * fraction) >> 16));
    }
}

/// Downmixes one clip frame to a single S16 sample. Multi-channel clips are
/// averaged with truncating integer division; S24 takes the most significant
/// bytes; canonical F32 maps `[-1, 1]` onto the full S16 range.
fn frame_s16(clip: &NeutralAudioV1, frame_index: u64) -> i16 {
    let channels = usize::try_from(clip.channel_count).unwrap_or(1).max(1);
    let frame = usize::try_from(frame_index).unwrap_or(0);
    let pcm = clip.pcm_bytes();
    let sum: i32 = match clip.pcm_encoding {
        AudioPcmEncodingV1::PcmS16Le => (0..channels)
            .map(|channel| {
                let base = (frame * channels + channel) * 2;
                i32::from(i16::from_le_bytes([pcm[base], pcm[base + 1]]))
            })
            .sum(),
        AudioPcmEncodingV1::PcmS24LePacked => (0..channels)
            .map(|channel| {
                let base = (frame * channels + channel) * 3;
                let mut value = i32::from(pcm[base])
                    | i32::from(pcm[base + 1]) << 8
                    | i32::from(pcm[base + 2]) << 16;
                if value & 0x80_0000 != 0 {
                    value |= !0xFF_FFFF;
                }
                value >> 8
            })
            .sum(),
        AudioPcmEncodingV1::PcmF32LeCanonical => (0..channels)
            .map(|channel| {
                let base = (frame * channels + channel) * 4;
                let value =
                    f32::from_le_bytes([pcm[base], pcm[base + 1], pcm[base + 2], pcm[base + 3]]);
                (value * 32_767.0).round() as i32
            })
            .sum(),
    };
    (sum / channels as i32).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

fn isqrt_u128(value: u128) -> u128 {
    let mut remaining = value;
    let mut root = 0_u128;
    let mut bit = 1_u128 << 126;
    while bit > remaining {
        bit >>= 2;
    }
    while bit != 0 {
        if remaining >= root + bit {
            remaining -= root + bit;
            root = (root >> 1) + bit;
        } else {
            root >>= 1;
        }
        bit >>= 2;
    }
    root
}

/// Rotates a micrometre vector by the inverse of a unit Q1.30 quaternion.
/// All math is integer with truncating shifts, so the result is exact and
/// identical on every host.
fn rotate_by_inverse_orientation(vector: [i64; 3], orientation: [i32; 4]) -> [i64; 3] {
    let q = [
        -i128::from(orientation[0]),
        -i128::from(orientation[1]),
        -i128::from(orientation[2]),
        i128::from(orientation[3]),
    ];
    let v = [
        i128::from(vector[0]),
        i128::from(vector[1]),
        i128::from(vector[2]),
    ];
    // t = 2 * q_vec × v (Q1.30 scale carried in q components).
    let cross = |a: [i128; 3], b: [i128; 3]| -> [i128; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let t = cross([q[0], q[1], q[2]], v).map(|value| value >> 29);
    let second = cross([q[0], q[1], q[2]], t).map(|value| value >> 30);
    let mut output = [0_i64; 3];
    for index in 0..3 {
        let rotated = v[index] + ((q[3] * t[index]) >> 30) + second[index];
        output[index] = i64::try_from(rotated).unwrap_or(i64::MAX);
    }
    output
}

#[cfg(test)]
mod tests {
    use next_contracts::audio::{AudioLoudnessMetadataV1, AudioPcmEncodingV1, NeutralAudioV1};
    use next_contracts::ids::{EventId, PersistentId};
    use next_contracts::presentation::QuantizedPresentationTransformV1;
    use next_contracts::presentation::audio_scene::{
        AudioCueV1, AudioEmitterKeyV1, AudioEmitterRecordV1, AudioListenerRecordV1,
        AudioLoudnessClassV1, AudioPriorityClassV1,
    };
    use next_contracts::project::{AssetRevisionRefV1, domain_hash};

    use super::*;

    fn clip(seed: u8, rate: u32, frames: u64, pcm: Vec<u8>) -> NeutralAudioV1 {
        NeutralAudioV1::new(
            AssetId::from_bytes([seed; 16]),
            1,
            rate,
            1,
            AudioPcmEncodingV1::PcmS16Le,
            frames,
            None,
            Vec::new(),
            AudioLoudnessMetadataV1::new(0, 65_536).expect("loudness"),
            pcm,
        )
        .expect("clip")
    }

    fn constant_clip(seed: u8, frames: u64, value: i16) -> NeutralAudioV1 {
        let mut pcm = Vec::new();
        for _ in 0..frames {
            pcm.extend_from_slice(&value.to_le_bytes());
        }
        clip(seed, 48_000, frames, pcm)
    }

    fn clips_map(clips: Vec<NeutralAudioV1>) -> BTreeMap<AssetId, NeutralAudioV1> {
        clips
            .into_iter()
            .map(|clip| (clip.asset_id, clip))
            .collect()
    }

    fn revision_of(clip: &NeutralAudioV1) -> AssetRevisionRefV1 {
        AssetRevisionRefV1 {
            asset_id: clip.asset_id,
            record_sha256: clip.record_sha256().expect("hash"),
        }
    }

    fn listener_at(x: i64) -> AudioListenerRecordV1 {
        AudioListenerRecordV1::new(
            PersistentId::from_bytes([0x99; 16]),
            QuantizedPresentationTransformV1 {
                translation_micrometres: [x, 0, 0],
                ..QuantizedPresentationTransformV1::default()
            },
        )
        .expect("listener")
    }

    fn scene(
        listener: AudioListenerRecordV1,
        emitters: Vec<AudioEmitterRecordV1>,
        cues: Vec<AudioCueV1>,
    ) -> AudioSceneSnapshotV1 {
        AudioSceneSnapshotV1::new(
            domain_hash("test.mix.epoch", b"epoch"),
            0,
            0,
            listener,
            emitters,
            cues,
            Vec::new(),
        )
        .expect("scene")
    }

    fn cue(seed: u8, clip: &NeutralAudioV1, priority: AudioPriorityClassV1) -> AudioCueV1 {
        AudioCueV1::new(
            EventId::from_bytes([seed; 16]),
            next_contracts::ids::SchemaId::new("nextengine.test.event.schema").expect("schema"),
            0,
            0,
            AudioEmitterKeyV1 {
                subject_id: PersistentId::from_bytes([seed; 16]),
                incarnation: 0,
            },
            revision_of(clip),
            AudioLoudnessClassV1::Normal,
            priority,
            None,
        )
        .expect("cue")
    }

    fn positional_emitter(
        seed: u8,
        clip: &NeutralAudioV1,
        x: i64,
        looped: bool,
    ) -> AudioEmitterRecordV1 {
        AudioEmitterRecordV1::new(
            AudioEmitterKeyV1 {
                subject_id: PersistentId::from_bytes([seed; 16]),
                incarnation: 0,
            },
            revision_of(clip),
            QuantizedPresentationTransformV1 {
                translation_micrometres: [x, 0, 0],
                ..QuantizedPresentationTransformV1::default()
            },
            AudioLoudnessClassV1::Normal,
            AudioPriorityClassV1::Normal,
            None,
            looped,
        )
        .expect("emitter")
    }

    #[test]
    fn profile_validation_is_fail_closed() {
        assert!(AudioMixProfileV1::stereo_baseline_v1().is_ok());
        for profile in [
            AudioMixProfileV1::new(7_999, 1_600, 16, 1, 2, 1, 0, 0, None),
            AudioMixProfileV1::new(48_000, 0, 16, 1, 2, 1, 0, 0, None),
            AudioMixProfileV1::new(48_000, 1_600, 0, 1, 2, 1, 0, 0, None),
            AudioMixProfileV1::new(48_000, 1_600, 16, 2, 2, 1, 0, 0, None),
            AudioMixProfileV1::new(48_000, 1_600, 16, 1, 2, 0, 0, 0, None),
            AudioMixProfileV1::new(48_000, 1_600, 16, 1, 2, 1, 65_537, 0, None),
        ] {
            assert_eq!(profile, Err(AudioMixErrorV1::InvalidProfile));
        }
    }

    #[test]
    fn non_spatial_cue_mixes_exactly_and_completes() {
        let clip = constant_clip(1, 3_200, 10_000);
        let clips = clips_map(vec![clip.clone()]);
        let mut mixer = AudioMixerV1::new(
            AudioMixProfileV1::new(
                48_000, 1_600, 16, 1_000_000, 20_000_000, 5_000_000, 32_768, 32_768, None,
            )
            .expect("profile"),
        );
        let first_scene = scene(
            listener_at(0),
            Vec::new(),
            vec![cue(1, &clip, AudioPriorityClassV1::Normal)],
        );
        let first = mixer.mix_tick(&first_scene, &clips);
        assert_eq!(first.len(), 3_200);
        // Centered non-spatial voice: both channels carry the constant sample
        // at half pan gain (linear pan law): 10 000 * 0.5.
        assert!(first[..100].iter().all(|sample| *sample == 5_000));
        assert_eq!(mixer.active_voice_count(), 1);
        let empty_scene = scene(listener_at(0), Vec::new(), Vec::new());
        let second = mixer.mix_tick(&empty_scene, &clips);
        assert!(second[..100].iter().all(|sample| *sample == 5_000));
        // The one-shot clip (3 200 frames) is finished after two windows.
        assert_eq!(mixer.active_voice_count(), 0);
        let third = mixer.mix_tick(&empty_scene, &clips);
        assert!(third.iter().all(|sample| *sample == 0));
    }

    #[test]
    fn priority_admission_preempts_lower_voices_deterministically() {
        let low_clip = constant_clip(1, 3_200, 5_000);
        let high_clip = constant_clip(2, 3_200, 8_000);
        let clips = clips_map(vec![low_clip.clone(), high_clip.clone()]);
        let mut mixer = AudioMixerV1::new(
            AudioMixProfileV1::new(
                48_000, 1_600, 1, 1_000_000, 20_000_000, 5_000_000, 32_768, 32_768, None,
            )
            .expect("profile"),
        );
        let first_scene = scene(
            listener_at(0),
            Vec::new(),
            vec![cue(1, &low_clip, AudioPriorityClassV1::Low)],
        );
        mixer.mix_tick(&first_scene, &clips);
        let second_scene = scene(
            listener_at(0),
            Vec::new(),
            vec![cue(2, &high_clip, AudioPriorityClassV1::Critical)],
        );
        let window = mixer.mix_tick(&second_scene, &clips);
        assert_eq!(mixer.preempted_voices(), 1);
        assert_eq!(mixer.active_voice_count(), 1);
        // Only the Critical voice (8 000) remains: half pan gain => 4 000.
        assert!(window[..100].iter().all(|sample| *sample == 4_000));
    }

    #[test]
    fn distance_attenuation_and_pan_are_deterministic() {
        let clip = constant_clip(1, 3_200, 16_000);
        let clips = clips_map(vec![clip.clone()]);
        let profile = AudioMixProfileV1::stereo_baseline_v1().expect("profile");
        // Emitter at 1 m: full gain, centered ahead on +X gives full right pan.
        let near_scene = scene(
            listener_at(0),
            vec![positional_emitter(1, &clip, 1_000_000, true)],
            Vec::new(),
        );
        let mut near_mixer = AudioMixerV1::new(profile.clone());
        let near = near_mixer.mix_tick(&near_scene, &clips);
        assert!(near[1] > near[0] && near[0] > 0, "right-panned voice");
        // Emitter at 25 m: past max distance, full silence.
        let far_scene = scene(
            listener_at(0),
            vec![positional_emitter(1, &clip, 25_000_000, true)],
            Vec::new(),
        );
        let mut far_mixer = AudioMixerV1::new(profile.clone());
        let far = far_mixer.mix_tick(&far_scene, &clips);
        assert!(far.iter().all(|sample| *sample == 0));
        // Same scene on two mixers produces byte-exact PCM.
        let mut repeat_mixer = AudioMixerV1::new(profile);
        let repeat = repeat_mixer.mix_tick(&near_scene, &clips);
        assert_eq!(near, repeat);
    }

    #[test]
    fn looped_emitter_wraps_into_loop_region() {
        let mut pcm = Vec::new();
        for value in [1_000_i16, 2_000, 3_000, 4_000] {
            pcm.extend_from_slice(&value.to_le_bytes());
        }
        let mut loop_clip = clip(7, 48_000, 4, pcm);
        loop_clip.loop_region_or_none =
            Some(next_contracts::audio::AudioLoopRegionV1::new(1, 4).expect("loop"));
        let clips = clips_map(vec![loop_clip.clone()]);
        let mut mixer = AudioMixerV1::new(
            AudioMixProfileV1::new(
                48_000, 8, 16, 1_000_000, 20_000_000, 5_000_000, 32_768, 32_768, None,
            )
            .expect("profile"),
        );
        let emitter_scene = scene(
            listener_at(0),
            vec![positional_emitter(7, &loop_clip, 500_000, true)],
            Vec::new(),
        );
        let window = mixer.mix_tick(&emitter_scene, &clips);
        // Non-spatial? No: emitter is positional at 0.5 m (< min 1 m → full
        // gain), centered? position x=+0.5 m pans right partially. Just check
        // the window is non-zero across the loop wrap (8 frames > 4-frame clip).
        assert!(window.iter().any(|sample| *sample != 0));
        assert_eq!(mixer.active_voice_count(), 1);
        // Voice survives subsequent windows as long as the emitter persists.
        let second = mixer.mix_tick(&emitter_scene, &clips);
        assert!(second.iter().any(|sample| *sample != 0));
        // Emitter leaving the scene retires the voice.
        let gone = scene(listener_at(0), Vec::new(), Vec::new());
        mixer.mix_tick(&gone, &clips);
        assert_eq!(mixer.active_voice_count(), 0);
    }

    #[test]
    fn wav_sink_writes_canonical_header_and_samples() {
        let profile = AudioMixProfileV1::stereo_baseline_v1().expect("profile");
        let wav = encode_canonical_wav(&profile, &[0_i16, 1, -1, 32_767, -32_768, 0]);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(
            u32::from_le_bytes(wav[24..28].try_into().expect("rate")),
            48_000
        );
        assert_eq!(u16::from_le_bytes(wav[22..24].try_into().expect("ch")), 2);
        assert_eq!(
            u32::from_le_bytes(wav[40..44].try_into().expect("size")),
            12
        );
        assert_eq!(wav.len(), 44 + 12);
        assert_eq!(&wav[44..48], &[0, 0, 1, 0]);
    }

    #[test]
    fn zone_mismatch_applies_occlusion_gain() {
        let clip = constant_clip(1, 3_200, 16_000);
        let clips = clips_map(vec![clip.clone()]);
        let profile = AudioMixProfileV1::stereo_baseline_v1().expect("profile");
        let open = scene(
            listener_at(0),
            vec![positional_emitter(1, &clip, 500_000, true)],
            Vec::new(),
        );
        let mut open_mixer = AudioMixerV1::new(profile.clone());
        let open_window = open_mixer.mix_tick(&open, &clips);
        let mut occluded_emitter = positional_emitter(1, &clip, 500_000, true);
        occluded_emitter.occlusion_zone_or_none =
            Some(next_contracts::ids::SchemaId::new("nextengine.test.zone.cave").expect("zone"));
        let occluded = scene(listener_at(0), vec![occluded_emitter], Vec::new());
        let mut occluded_mixer = AudioMixerV1::new(profile);
        let occluded_window = occluded_mixer.mix_tick(&occluded, &clips);
        assert!(open_window[1] > occluded_window[1]);
        // Two truncating shift stages: exact halving within one unit.
        assert!((open_window[1] - occluded_window[1] * 2).abs() <= 2);
    }

    #[test]
    fn resampling_half_rate_clip_is_exact_length() {
        let half_rate = clip(3, 24_000, 1_600, {
            let mut pcm = Vec::new();
            for _ in 0..1_600 {
                pcm.extend_from_slice(&6_000_i16.to_le_bytes());
            }
            pcm
        });
        let clips = clips_map(vec![half_rate.clone()]);
        let mut mixer =
            AudioMixerV1::new(AudioMixProfileV1::stereo_baseline_v1().expect("profile"));
        let first_scene = scene(
            listener_at(0),
            Vec::new(),
            vec![cue(3, &half_rate, AudioPriorityClassV1::Normal)],
        );
        let window = mixer.mix_tick(&first_scene, &clips);
        assert_eq!(window.len(), 3_200);
        assert!(window[..100].iter().all(|sample| *sample == 3_000));
        // 1 600 clip frames at half rate stretch over two 48 kHz windows.
        assert_eq!(mixer.active_voice_count(), 1);
        let second = mixer.mix_tick(&scene(listener_at(0), Vec::new(), Vec::new()), &clips);
        assert!(second[..100].iter().all(|sample| *sample == 3_000));
        assert_eq!(mixer.active_voice_count(), 0);
        let third = mixer.mix_tick(&scene(listener_at(0), Vec::new(), Vec::new()), &clips);
        assert!(third.iter().all(|sample| *sample == 0));
    }
}
