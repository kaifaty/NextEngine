//! Plan `continuum-water/34`: water heard. The stage's edge records become
//! looped flow emitters, the floating boxes' threshold crossings become
//! splash cues, and a listener under the level hears the mix through a
//! low-pass. Everything here is presentation-only: it reads the committed
//! checkpoint's records and writes the audio scene of the tick; nothing
//! flows back into gameplay, saves or roots.

use std::collections::BTreeMap;

use next_contracts::ids::{EventId, PersistentId, SchemaId};
use next_contracts::physics::{PhysicsBodyIdV1, WaterSubmersionClassV1, WaterVolumeSetV1};
use next_contracts::presentation::QuantizedPresentationTransformV1;
use next_contracts::presentation::audio_scene::{
    AcousticFactV1, AudioCueV1, AudioEmitterKeyV1, AudioLoudnessClassV1, AudioPriorityClassV1,
    AudioSceneSnapshotV1,
};
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::project::domain_hash;
use next_presentation::audio_scene::AudioEmitterBindingV1;

use crate::ReferenceGameError;
use crate::water_presentation::{
    WATER_SPLASH_SPEED_THRESHOLD_MICROMETRES_PER_SECOND, WaterEdgePresentationV1,
    WaterFloatingBoxV1,
};

/// Flux bands of the flow loop's loudness class, in cubic millimetres per
/// published tick (about `0.24 L/s` and `1.2 L/s` at 30 Hz).
pub const WATER_AUDIO_FLOW_NORMAL_FLUX_CUBIC_MILLIMETRES: i64 = 8_000;
pub const WATER_AUDIO_FLOW_LOUD_FLUX_CUBIC_MILLIMETRES: i64 = 40_000;
/// A splash is `Loud` from this vertical speed, `Normal` from the stage's
/// splash threshold.
pub const WATER_AUDIO_SPLASH_LOUD_SPEED_MICROMETRES_PER_SECOND: i64 = 1_000_000;
/// The splash cue's event domain and schema.
pub const WATER_AUDIO_SPLASH_EVENT_DOMAIN: &str = "nextengine.water-audio.splash.v1";
pub const WATER_AUDIO_SPLASH_SCHEMA_ID: &str = "nextengine.water.splash";
/// One-pole low-pass for a listener under the level: `800 Hz` at `48 kHz`
/// as a Q16 coefficient (`dt / (RC + dt)`).
pub const WATER_AUDIO_UNDERWATER_LOW_PASS_ALPHA_Q16: i32 = 6_226;

/// The two water clips, resolved once from the activated content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterAudioClipsV1 {
    pub flow: AssetRevisionRefV1,
    pub splash: AssetRevisionRefV1,
}

/// Presentation-only memory between ticks: the boxes' vertical speeds of
/// the previous tick (the splash trigger is an upward crossing) and the
/// low-pass state per channel.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WaterAudioStateV1 {
    pub previous_speeds: BTreeMap<PhysicsBodyIdV1, i64>,
    pub low_pass: [i32; 2],
}

/// The records of one tick read from the committed checkpoint.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WaterAudioInputsV1 {
    pub edges: Vec<WaterEdgePresentationV1>,
    pub boxes: Vec<(PhysicsBodyIdV1, WaterFloatingBoxV1)>,
}

/// The loudness class of a flow record by its absolute flux.
#[must_use]
pub const fn flow_loudness_class(flux_cubic_millimetres: i64) -> AudioLoudnessClassV1 {
    let flux = flux_cubic_millimetres.unsigned_abs();
    if flux < WATER_AUDIO_FLOW_NORMAL_FLUX_CUBIC_MILLIMETRES.unsigned_abs() {
        AudioLoudnessClassV1::Quiet
    } else if flux < WATER_AUDIO_FLOW_LOUD_FLUX_CUBIC_MILLIMETRES.unsigned_abs() {
        AudioLoudnessClassV1::Normal
    } else {
        AudioLoudnessClassV1::Loud
    }
}

/// One looped emitter per edge record, keyed by the edge id at the crest.
#[must_use]
pub fn flow_emitters(
    edges: &[WaterEdgePresentationV1],
    clips: &WaterAudioClipsV1,
) -> Vec<AudioEmitterBindingV1> {
    edges
        .iter()
        .map(|edge| AudioEmitterBindingV1 {
            emitter_key: AudioEmitterKeyV1 {
                subject_id: edge.edge_id,
                incarnation: 0,
            },
            clip_revision: clips.flow,
            loudness_class: flow_loudness_class(edge.flux_cubic_millimetres),
            priority_class: AudioPriorityClassV1::Normal,
            occlusion_zone_or_none: None,
            looped: true,
            physics_body_id: None,
            fallback_transform: QuantizedPresentationTransformV1 {
                translation_micrometres: edge.crest_micrometres,
                ..QuantizedPresentationTransformV1::default()
            },
        })
        .collect()
}

/// The splashes of one tick: the non-looped emitters carrying the
/// positions, the cues and their acoustic facts.
#[derive(Clone, Debug, Default)]
pub struct WaterSplashRecordsV1 {
    pub emitters: Vec<AudioEmitterBindingV1>,
    pub cues: Vec<AudioCueV1>,
    pub facts: Vec<AcousticFactV1>,
}

/// A splash: the box's vertical speed crosses the stage's splash threshold
/// upward from the previous tick and the box straddles its volume's level.
/// Returns the emitter (non-looped, the position) and the cue; updates the
/// previous speeds for every box.
pub fn splash_records(
    boxes: &[(PhysicsBodyIdV1, WaterFloatingBoxV1)],
    volumes: &WaterVolumeSetV1,
    tick: u64,
    listener_id: PersistentId,
    clips: &WaterAudioClipsV1,
    state: &mut WaterAudioStateV1,
) -> Result<WaterSplashRecordsV1, ReferenceGameError> {
    let mut emitters = Vec::new();
    let mut cues = Vec::new();
    let mut facts = Vec::new();
    let mut next_speeds = BTreeMap::new();
    for (body_id, floating) in boxes {
        let speed = floating
            .vertical_velocity_micrometres_per_second
            .unsigned_abs();
        let previous = state
            .previous_speeds
            .get(body_id)
            .map_or(0, |value| value.unsigned_abs());
        next_speeds.insert(*body_id, floating.vertical_velocity_micrometres_per_second);
        let threshold = WATER_SPLASH_SPEED_THRESHOLD_MICROMETRES_PER_SECOND.unsigned_abs();
        if speed < threshold || previous >= threshold {
            continue;
        }
        let centre = [
            (floating.minimum_micrometres[0] + floating.maximum_micrometres[0]) / 2,
            (floating.minimum_micrometres[1] + floating.maximum_micrometres[1]) / 2,
            (floating.minimum_micrometres[2] + floating.maximum_micrometres[2]) / 2,
        ];
        let level = volumes
            .submersion_at(
                [centre[0], floating.minimum_micrometres[1], centre[2]],
                tick,
            )
            .level_micrometres;
        let Some(level) = level else {
            continue;
        };
        if level < floating.minimum_micrometres[1] || level > floating.maximum_micrometres[1] {
            continue;
        }
        let emitter_key = AudioEmitterKeyV1 {
            subject_id: body_id.subject_id,
            incarnation: 0,
        };
        let loudness =
            if speed >= WATER_AUDIO_SPLASH_LOUD_SPEED_MICROMETRES_PER_SECOND.unsigned_abs() {
                AudioLoudnessClassV1::Loud
            } else {
                AudioLoudnessClassV1::Normal
            };
        let mut preimage = Vec::with_capacity(16 + 4 + 8);
        preimage.extend_from_slice(body_id.subject_id.as_bytes());
        preimage.extend_from_slice(&body_id.body_slot.to_le_bytes());
        preimage.extend_from_slice(&tick.to_le_bytes());
        let hash = domain_hash(WATER_AUDIO_SPLASH_EVENT_DOMAIN, &preimage);
        let mut event_bytes = [0_u8; 16];
        event_bytes.copy_from_slice(&hash.as_bytes()[..16]);
        let cue = AudioCueV1::new(
            EventId::from_bytes(event_bytes),
            SchemaId::new(WATER_AUDIO_SPLASH_SCHEMA_ID)?,
            0,
            tick,
            emitter_key,
            clips.splash,
            loudness,
            AudioPriorityClassV1::High,
            None,
        )?;
        facts.push(AcousticFactV1 {
            tick,
            source: emitter_key,
            listener_id,
            loudness_class: loudness,
            occlusion_zone_or_none: None,
        });
        cues.push(cue);
        emitters.push(AudioEmitterBindingV1 {
            emitter_key,
            clip_revision: clips.splash,
            loudness_class: loudness,
            priority_class: AudioPriorityClassV1::High,
            occlusion_zone_or_none: None,
            looped: false,
            physics_body_id: None,
            fallback_transform: QuantizedPresentationTransformV1 {
                translation_micrometres: centre,
                ..QuantizedPresentationTransformV1::default()
            },
        });
    }
    state.previous_speeds = next_speeds;
    Ok(WaterSplashRecordsV1 {
        emitters,
        cues,
        facts,
    })
}

/// Rebuilds the extracted scene with the water cues and facts appended
/// (the emitters were bindings of the extraction already).
pub fn scene_with_water(
    scene: AudioSceneSnapshotV1,
    cues: Vec<AudioCueV1>,
    facts: Vec<AcousticFactV1>,
) -> Result<AudioSceneSnapshotV1, ReferenceGameError> {
    if cues.is_empty() && facts.is_empty() {
        return Ok(scene);
    }
    let mut all_cues = scene.cues.clone();
    all_cues.extend(cues);
    let mut all_facts = scene.acoustic_facts.clone();
    all_facts.extend(facts);
    Ok(AudioSceneSnapshotV1::new(
        scene.snapshot_epoch,
        scene.snapshot_sequence,
        scene.simulation_tick,
        scene.listener,
        scene.emitters.clone(),
        all_cues,
        all_facts,
    )?)
}

/// The listener is under the level when the camera point's exact
/// submersion is not `Dry`.
#[must_use]
pub fn listener_submerged(
    volumes: &WaterVolumeSetV1,
    camera_translation_micrometres: [i64; 3],
    tick: u64,
) -> bool {
    !volumes.is_empty()
        && volumes
            .submersion_at(camera_translation_micrometres, tick)
            .class
            != WaterSubmersionClassV1::Dry
}

/// One-pole low-pass over an interleaved stereo window, or a reset of the
/// state when the listener is above the level.
pub fn low_pass_in_place(pcm: &mut [i16], submerged: bool, state: &mut WaterAudioStateV1) {
    if !submerged {
        state.low_pass = [0; 2];
        return;
    }
    for (index, sample) in pcm.iter_mut().enumerate() {
        let channel = index % 2;
        let input = i32::from(*sample);
        let output = state.low_pass[channel]
            + (((input - state.low_pass[channel]) * WATER_AUDIO_UNDERWATER_LOW_PASS_ALPHA_Q16)
                >> 16);
        state.low_pass[channel] = output;
        *sample = output.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::ids::ContentHash;
    use next_contracts::physics::WaterVolumeDefinitionV1;

    fn clips() -> WaterAudioClipsV1 {
        let revision = |byte: u8| AssetRevisionRefV1 {
            asset_id: next_contracts::ids::AssetId::from_bytes([byte; 16]),
            record_sha256: ContentHash::from_bytes([byte; 32]),
        };
        WaterAudioClipsV1 {
            flow: revision(0xa5),
            splash: revision(0xa6),
        }
    }

    fn edge(flux: i64) -> WaterEdgePresentationV1 {
        WaterEdgePresentationV1 {
            edge_id: PersistentId::from_bytes([0x80; 16]),
            kind: crate::water_presentation::WaterEdgePresentationKindV1::Jet,
            crest_micrometres: [1_000_000, 2_000_000, 3_000_000],
            direction_q15: [32_767, 0],
            source_level_micrometres: 2_000_000,
            sink_level_micrometres: 1_000_000,
            flux_cubic_millimetres: flux,
        }
    }

    #[test]
    fn flow_records_become_looped_emitters_at_the_crest_by_flux_band() {
        let clips = clips();
        let emitters = flow_emitters(&[edge(1_000), edge(-20_000), edge(60_000)], &clips);
        assert_eq!(emitters.len(), 3);
        assert!(emitters.iter().all(|emitter| emitter.looped));
        assert_eq!(
            emitters[0].fallback_transform.translation_micrometres,
            [1_000_000, 2_000_000, 3_000_000]
        );
        assert_eq!(emitters[0].loudness_class, AudioLoudnessClassV1::Quiet);
        assert_eq!(emitters[1].loudness_class, AudioLoudnessClassV1::Normal);
        assert_eq!(emitters[2].loudness_class, AudioLoudnessClassV1::Loud);
        assert_eq!(emitters[0].clip_revision, clips.flow);
    }

    fn volumes() -> WaterVolumeSetV1 {
        WaterVolumeSetV1::from_definitions([WaterVolumeDefinitionV1 {
            volume_id: PersistentId::from_bytes([0x7a; 16]),
            minimum_micrometres: [0, 0, 0],
            maximum_micrometres: [4_000_000, 2_000_000, 4_000_000],
            initial_level_micrometres: 500_000,
            swimming_depth_micrometres: 1_200_000,
            level_ramp: None,
            profile_revision: 1,
        }])
        .expect("volume")
    }

    fn floating(bottom: i64, speed: i64) -> (PhysicsBodyIdV1, WaterFloatingBoxV1) {
        (
            PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([0x87; 16]),
                body_slot: 0,
            },
            WaterFloatingBoxV1 {
                minimum_micrometres: [1_000_000, bottom, 1_000_000],
                maximum_micrometres: [1_500_000, bottom + 500_000, 1_500_000],
                vertical_velocity_micrometres_per_second: speed,
            },
        )
    }

    #[test]
    fn a_box_crossing_the_threshold_at_the_level_splashes_once() {
        let clips = clips();
        let volumes = volumes();
        let listener = PersistentId::from_bytes([0x01; 16]);
        let mut state = WaterAudioStateV1::default();
        // Slow: nothing, the speed is remembered.
        let WaterSplashRecordsV1 { cues, .. } = splash_records(
            &[floating(300_000, 100_000)],
            &volumes,
            1,
            listener,
            &clips,
            &mut state,
        )
        .expect("records");
        assert!(cues.is_empty());
        // Fast and straddling the level (0.5 m): one cue, Normal.
        let WaterSplashRecordsV1 {
            emitters,
            cues,
            facts,
        } = splash_records(
            &[floating(300_000, -600_000)],
            &volumes,
            2,
            listener,
            &clips,
            &mut state,
        )
        .expect("records");
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].loudness_class, AudioLoudnessClassV1::Normal);
        assert_eq!(cues[0].clip_revision, clips.splash);
        assert_eq!(cues[0].activation_simulation_tick, 2);
        assert_eq!(emitters.len(), 1);
        assert!(!emitters[0].looped);
        assert_eq!(facts.len(), 1);
        // Still fast next tick: no second cue.
        let WaterSplashRecordsV1 { cues, .. } = splash_records(
            &[floating(300_000, -700_000)],
            &volumes,
            3,
            listener,
            &clips,
            &mut state,
        )
        .expect("records");
        assert!(cues.is_empty());
        // Slow then very fast but wholly under the level: none.
        splash_records(
            &[floating(-100_000, 0)],
            &volumes,
            4,
            listener,
            &clips,
            &mut state,
        )
        .expect("records");
        let WaterSplashRecordsV1 { cues, .. } = splash_records(
            &[floating(-1_000_000, 1_500_000)],
            &volumes,
            5,
            listener,
            &clips,
            &mut state,
        )
        .expect("records");
        assert!(cues.is_empty());
        // Slow then very fast at the level: Loud.
        splash_records(
            &[floating(300_000, 0)],
            &volumes,
            6,
            listener,
            &clips,
            &mut state,
        )
        .expect("records");
        let WaterSplashRecordsV1 { cues, .. } = splash_records(
            &[floating(300_000, 1_500_000)],
            &volumes,
            7,
            listener,
            &clips,
            &mut state,
        )
        .expect("records");
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].loudness_class, AudioLoudnessClassV1::Loud);
    }

    /// Plan 34 G5 timing probe: the records-to-scene step on the reference
    /// checkpoint; run with `--ignored --nocapture` in release.
    #[test]
    #[ignore = "timing probe"]
    fn bench_records_to_scene() {
        let volumes = crate::water::reference_water_volumes().expect("volumes");
        let mut network = crate::water::reference_water_flow(&volumes).expect("network");
        let mut volumes = volumes;
        for _ in 0..30 {
            network.step_in_place(&mut volumes).expect("step");
        }
        let checkpoint_boxes = vec![floating(300_000, 400_000)];
        let clips = clips();
        let listener = PersistentId::from_bytes([0x01; 16]);
        let mut state = WaterAudioStateV1::default();
        let mut pcm = vec![1_000_i16; 3_200];
        let started = std::time::Instant::now();
        let iterations = 2_000;
        let mut emitted = 0;
        for tick in 0..iterations {
            let edges =
                crate::water_presentation::water_audio_edge_records(&volumes, &network, tick);
            let emitters = flow_emitters(&edges, &clips);
            let splashes = splash_records(
                &checkpoint_boxes,
                &volumes,
                tick,
                listener,
                &clips,
                &mut state,
            )
            .expect("records");
            low_pass_in_place(&mut pcm, true, &mut state);
            emitted += emitters.len() + splashes.emitters.len();
        }
        let mean = started.elapsed().as_micros() / u128::from(iterations);
        eprintln!("records_to_scene mean {mean} us over {iterations} ticks ({emitted} emitters)");
    }

    #[test]
    fn listener_under_the_level_is_submerged_and_dry_ground_is_not() {
        let volumes = volumes();
        assert!(listener_submerged(
            &volumes,
            [2_000_000, 200_000, 2_000_000],
            0
        ));
        assert!(!listener_submerged(
            &volumes,
            [2_000_000, 900_000, 2_000_000],
            0
        ));
        assert!(!listener_submerged(
            &volumes,
            [9_000_000, 200_000, 2_000_000],
            0
        ));
    }

    #[test]
    fn low_pass_damps_a_nyquist_alternation_and_passes_a_constant() {
        let mut state = WaterAudioStateV1::default();
        let mut alternating: Vec<i16> = (0..2_000)
            .map(|index| {
                if (index / 2) % 2 == 0 {
                    10_000
                } else {
                    -10_000
                }
            })
            .collect();
        low_pass_in_place(&mut alternating, true, &mut state);
        let peak = alternating[1_000..]
            .iter()
            .map(|s| s.unsigned_abs())
            .max()
            .unwrap();
        assert!(peak < 1_000, "alternation peak {peak}");
        let mut constant = vec![8_000_i16; 4_000];
        let mut state = WaterAudioStateV1::default();
        low_pass_in_place(&mut constant, true, &mut state);
        assert!(
            constant[3_999] > 7_900,
            "constant settles to {}",
            constant[3_999]
        );
        let mut untouched = vec![8_000_i16; 4];
        low_pass_in_place(&mut untouched, false, &mut state);
        assert_eq!(untouched, vec![8_000; 4]);
        assert_eq!(state.low_pass, [0; 2]);
    }
}
