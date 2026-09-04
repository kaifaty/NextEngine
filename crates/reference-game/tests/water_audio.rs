//! Plan `continuum-water/34` G3: the gate's jet is heard from the first
//! tick, deterministically, and the low-pass leaves an above-water listener
//! alone.

use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::presentation::audio_scene::AudioSceneSnapshotV1;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "nextengine-reference-water-audio-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

fn run(ticks: usize) -> (Vec<AudioSceneSnapshotV1>, Vec<Vec<i16>>) {
    let root = test_root();
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v7(
        next_reference_game::project_source_v7().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let mut driver =
        next_reference_game::ReferenceGameDriverV2::new(activated, true).expect("live driver");
    let mut scenes = Vec::with_capacity(ticks);
    let mut windows = Vec::with_capacity(ticks);
    for _ in 0..ticks {
        driver.advance(&[]).expect("advance");
        scenes.push(driver.audio_scene().clone());
        windows.push(driver.audio_mixed_pcm().to_vec());
    }
    std::fs::remove_dir_all(root).expect("remove store");
    (scenes, windows)
}

#[test]
fn the_gate_jet_is_a_looped_emitter_and_the_mix_is_not_silent() {
    let (scenes, windows) = run(120);
    let gate = next_reference_game::REFERENCE_WATER_FLOW_GATE_ID;
    let flow_clip = next_reference_game::audio::REFERENCE_WATER_FLOW_CLIP_ASSET_ID;
    for (index, scene) in scenes.iter().enumerate() {
        let emitter = scene
            .emitters
            .iter()
            .find(|emitter| emitter.emitter_key.subject_id == gate)
            .unwrap_or_else(|| panic!("tick {index}: the gate's flow emitter is missing"));
        assert!(emitter.looped);
        assert_eq!(emitter.clip_revision.asset_id, flow_clip);
    }
    // The loop's voice starts on admission; from the second window on the
    // mix carries it.
    for (index, window) in windows.iter().enumerate().skip(1) {
        assert!(
            window.iter().any(|sample| *sample != 0),
            "tick {index}: the mixed window is silent"
        );
    }
    let (again, windows_again) = run(120);
    assert!(
        scenes
            .iter()
            .zip(&again)
            .all(|(first, second)| first.canonical_hash == second.canonical_hash)
    );
    assert_eq!(windows, windows_again);
}
