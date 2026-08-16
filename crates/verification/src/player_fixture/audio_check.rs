//! AUDIO-02-style displayless audio check (SPEC-08): the production live
//! loop (`ReferenceGameDriverV2`) runs a scripted interactive session twice;
//! per-tick audio scenes, acoustic facts, canonical PCM and the gameplay
//! state root must be byte-exact across the two runs, cue bindings must
//! produce the authored activations, and the canonical WAV sink must encode
//! the concatenated stream deterministically.

use next_contracts::canonical::sha256;
use next_contracts::ids::{
    ContentHash, PersistentId, SchemaId, StateRoot, content_hash_from_bytes,
};
use next_contracts::input::{
    KEYBOARD_D_CONTROL_PATH_ID, KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_E_CONTROL_PATH_ID,
    KEYBOARD_F_CONTROL_PATH_ID, KEYBOARD_Q_CONTROL_PATH_ID, KEYBOARD_R_CONTROL_PATH_ID,
    KEYBOARD_RETURN_CONTROL_PATH_ID, KEYBOARD_W_CONTROL_PATH_ID,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_contracts::presentation::audio_scene::AudioSceneSnapshotV1;
use next_presentation::audio_mix::encode_canonical_wav;
use next_reference_game::ReferenceGameDriverV2;

use super::error::PlayCheckError;
use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioSceneCheckReportV1 {
    pub ticks: u64,
    pub cue_count: u64,
    pub acoustic_fact_count: u64,
    pub non_silent_windows: u64,
    pub pcm_frames: u64,
    pub pcm_digest: ContentHash,
    pub canonical_wav_digest: ContentHash,
    pub final_state_root: StateRoot,
    pub repeated_run_identical: bool,
}

pub fn run_audio_scene_check() -> Result<AudioSceneCheckReportV1, PlayCheckError> {
    run_audio_scene_check_in(&std::env::temp_dir())
}

pub fn run_audio_scene_check_in(
    scratch_root: &std::path::Path,
) -> Result<AudioSceneCheckReportV1, PlayCheckError> {
    let scratch = ScratchContext::new(scratch_root).map_err(scratch_error)?;
    run_audio_scene_check_with_scratch(&scratch)
}

pub(crate) fn run_audio_scene_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<AudioSceneCheckReportV1, PlayCheckError> {
    let directory = scratch
        .create_directory("audio-scene")
        .map_err(scratch_error)?;
    let result = (|| {
        let first = run_scripted_audio_session(scratch)?;
        let second = run_scripted_audio_session(scratch)?;
        if first.scenes != second.scenes
            || first.pcm_bytes != second.pcm_bytes
            || first.final_state_root != second.final_state_root
        {
            return Err(PlayCheckError::AcceptanceMismatch(
                "audio scene/PCM/gameplay roots diverged between identical scripted runs"
                    .to_owned(),
            ));
        }
        let cue_count = first
            .scenes
            .iter()
            .map(|scene| scene.cues.len() as u64)
            .sum::<u64>();
        let acoustic_fact_count = first
            .scenes
            .iter()
            .map(|scene| scene.acoustic_facts.len() as u64)
            .sum::<u64>();
        let clip_ids: std::collections::BTreeSet<_> = first
            .scenes
            .iter()
            .flat_map(|scene| scene.cues.iter().map(|cue| cue.clip_revision.asset_id))
            .collect();
        let expected_clips = [
            next_reference_game::audio::REFERENCE_SWITCH_CLIP_ASSET_ID,
            next_reference_game::audio::REFERENCE_PICKUP_CLIP_ASSET_ID,
            next_reference_game::audio::REFERENCE_MELEE_CLIP_ASSET_ID,
        ];
        if cue_count < 4
            || acoustic_fact_count != cue_count
            || !expected_clips
                .iter()
                .all(|clip_id| clip_ids.contains(clip_id))
            || clip_ids.contains(&next_reference_game::audio::REFERENCE_DIALOGUE_CLIP_ASSET_ID)
        {
            return Err(PlayCheckError::AcceptanceMismatch(format!(
                "audio cue acceptance failed: cues={cue_count} facts={acoustic_fact_count} \
                 clips={clip_ids:?}"
            )));
        }
        // The scripted live attempt occurs after Duty -> Rest, so the rejected
        // interaction must not synthesize a dialogue cue or subtitle.
        if first.subtitle_after_session.is_some()
            || first.subtitle_after_session != second.subtitle_after_session
        {
            return Err(PlayCheckError::AcceptanceMismatch(format!(
                "audio subtitle acceptance failed: {:?}",
                first.subtitle_after_session
            )));
        }
        let frames_per_window = 1_600_u64 * 2;
        let windows: Vec<&[i16]> = first
            .pcm_samples
            .chunks(frames_per_window as usize)
            .collect();
        let non_silent_windows = windows
            .iter()
            .filter(|window| window.iter().any(|sample| *sample != 0))
            .count() as u64;
        if non_silent_windows < 3 {
            return Err(PlayCheckError::AcceptanceMismatch(format!(
                "audio PCM acceptance failed: only {non_silent_windows} non-silent windows"
            )));
        }
        let profile = next_presentation::audio_mix::AudioMixProfileV1::stereo_baseline_v1()
            .expect("baseline mix profile is valid");
        let wav = encode_canonical_wav(&profile, &first.pcm_samples);
        Ok(AudioSceneCheckReportV1 {
            ticks: first.scenes.len() as u64,
            cue_count,
            acoustic_fact_count,
            non_silent_windows,
            pcm_frames: first.pcm_samples.len() as u64 / 2,
            pcm_digest: content_hash_from_bytes(sha256(&first.pcm_bytes)),
            canonical_wav_digest: content_hash_from_bytes(sha256(&wav)),
            final_state_root: first.final_state_root,
            repeated_run_identical: true,
        })
    })();
    directory.finish(result, |error| {
        PlayCheckError::Fixture(crate::NeutralFixtureError::Cleanup(error))
    })
}

struct ScriptedAudioOutcomeV1 {
    scenes: Vec<AudioSceneSnapshotV1>,
    pcm_samples: Vec<i16>,
    pcm_bytes: Vec<u8>,
    final_state_root: StateRoot,
    subtitle_after_session: Option<String>,
}

fn run_scripted_audio_session(
    scratch: &ScratchContext,
) -> Result<ScriptedAudioOutcomeV1, PlayCheckError> {
    let prepared = prepare_fixture_project_package_with_scratch(
        scratch,
        next_reference_game::REFERENCE_GAME_PROJECT_ID,
    )?;
    let result = run_scripted_audio_session_with_package(prepared.package.clone());
    prepared.finish(result, scratch_error)
}

fn run_scripted_audio_session_with_package(
    package: next_project::ActivatedProjectPackage,
) -> Result<ScriptedAudioOutcomeV1, PlayCheckError> {
    let mut driver = ReferenceGameDriverV2::new(package, true)?;
    let mut scenes = Vec::new();
    let mut pcm_samples = Vec::new();
    for frame_events in audio_script() {
        driver.advance(&frame_events)?;
        scenes.push(driver.audio_scene().clone());
        pcm_samples.extend_from_slice(driver.audio_mixed_pcm());
    }
    let pcm_bytes: Vec<u8> = pcm_samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect();
    let state = driver.state()?;
    let final_state_root = next_contracts::snapshot::
        world_checkpoint_with_physical_animation_and_systemic_cognition_v1_state_root_from_canonical_components(
            &state.checkpoint_canonical_components,
            &state.world_streaming_snapshot,
            state.world_routine_snapshot_or_none.as_ref(),
            &state.world_population_snapshot,
            &state.world_activity_snapshot,
            &state.agent_cognition_snapshot,
            &state.agent_memory_snapshot,
            &state.physical_animation_snapshot,
        )?;
    let subtitle_after_session = driver
        .current_audio_subtitle(driver.next_tick())
        .map(|text_id| text_id.as_str().to_owned());
    Ok(ScriptedAudioOutcomeV1 {
        scenes,
        pcm_samples,
        pcm_bytes,
        final_state_root,
        subtitle_after_session,
    })
}

/// Scripted interactive session mirroring the reference play scenario's
/// interactive portion: move forward, pickup, equip, interact (switch),
/// approach the NPC, melee, then attempt the now Rest-gated dialogue.
fn audio_script() -> Vec<Vec<PlatformEventV1>> {
    use NormalizedControlPhaseV1::{Completed, Started};
    let mut sequence = 0_u64;
    let mut frame = |events: Vec<(&str, NormalizedControlPhaseV1)>| {
        events
            .into_iter()
            .map(|(path, phase)| {
                sequence += 1;
                control_event(path, phase, sequence)
            })
            .collect::<Vec<_>>()
    };
    vec![
        frame(vec![(KEYBOARD_W_CONTROL_PATH_ID, Started)]),
        frame(vec![]),
        frame(vec![]),
        frame(vec![]),
        frame(vec![(KEYBOARD_Q_CONTROL_PATH_ID, Started)]),
        frame(vec![(KEYBOARD_Q_CONTROL_PATH_ID, Completed)]),
        frame(vec![(KEYBOARD_R_CONTROL_PATH_ID, Started)]),
        frame(vec![(KEYBOARD_R_CONTROL_PATH_ID, Completed)]),
        frame(vec![(KEYBOARD_E_CONTROL_PATH_ID, Started)]),
        frame(vec![(KEYBOARD_E_CONTROL_PATH_ID, Completed)]),
        frame(vec![
            (KEYBOARD_W_CONTROL_PATH_ID, Completed),
            (KEYBOARD_D_CONTROL_PATH_ID, Started),
        ]),
        frame(vec![]),
        frame(vec![]),
        frame(vec![
            (KEYBOARD_D_CONTROL_PATH_ID, Completed),
            (KEYBOARD_F_CONTROL_PATH_ID, Started),
        ]),
        frame(vec![(KEYBOARD_F_CONTROL_PATH_ID, Completed)]),
        frame(vec![(KEYBOARD_E_CONTROL_PATH_ID, Started)]),
        frame(vec![(KEYBOARD_E_CONTROL_PATH_ID, Completed)]),
        frame(vec![(KEYBOARD_RETURN_CONTROL_PATH_ID, Started)]),
        frame(vec![(KEYBOARD_RETURN_CONTROL_PATH_ID, Completed)]),
        frame(vec![]),
    ]
}

fn control_event(
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    source_sequence: u64,
) -> PlatformEventV1 {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(KEYBOARD_DEVICE_CLASS_ID).expect("device class"),
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(control_path).expect("control path"),
        phase,
        vec![if phase == NormalizedControlPhaseV1::Started {
            i16::MAX
        } else {
            0
        }],
        Vec::new(),
        source_sequence,
        source_sequence,
    )
    .expect("control event");
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.audio-check").expect("source class"),
        source_sequence,
        source_sequence,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(sha256(b"nextengine.platform.audio-check-capabilities.v1")),
    )
    .expect("platform event")
}

fn scratch_error(error: std::io::Error) -> PlayCheckError {
    PlayCheckError::Fixture(crate::NeutralFixtureError::Cleanup(error))
}
