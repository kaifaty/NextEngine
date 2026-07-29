use std::sync::atomic::{AtomicU64, Ordering};

use next_contracts::ids::{AssetId, ContentHash, SchemaId};
use next_contracts::persistence::{SaveCompatibility, TickSettings};
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::rpg::RpgSnapshotV2;
use next_contracts::snapshot::RuntimeSnapshotV3;
use next_contracts::world::{
    WorldChunkLifecycleV1, WorldChunkResidencyRecordV1, WorldStreamingSnapshotV1,
};

use super::error::SaveLoadError;
use super::generation::{MANIFEST_FILE, SEGMENTS_DIRECTORY, segment_file_name};
use super::store::{CommitBoundary, SaveStore, synthetic_empty_checkpoint};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: std::path::PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-save-test-{}-{sequence}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("test directory is created");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn compatibility(seed: u8) -> SaveCompatibility {
    SaveCompatibility {
        engine_build_hash: ContentHash::from_bytes([seed; 32]),
        game_build_hash: ContentHash::from_bytes([seed.wrapping_add(1); 32]),
        project_id: SchemaId::new("nextengine.save-test").expect("valid project"),
        schema_registry_hash: ContentHash::from_bytes([seed.wrapping_add(2); 32]),
        content_manifest_hash: ContentHash::from_bytes([seed.wrapping_add(3); 32]),
        mechanics_lock_hash: ContentHash::from_bytes([seed.wrapping_add(4); 32]),
        tick_settings: TickSettings {
            gameplay_hz: 30,
            physics_hz: 60,
            motor_hz: 60,
        },
        loaded_chunk_revisions: vec![],
        rng_stream_states: vec![],
        physical_bindings: vec![],
        policy_state_schemas: vec![],
        plugin_script_bindings: vec![],
    }
}

fn snapshot(revision: u64) -> RuntimeSnapshotV3 {
    let bootstrap =
        next_runtime::RuntimeBootstrapV3::neutral_empty().expect("neutral bootstrap is valid");
    let runtime =
        next_runtime::RuntimeState::new(bootstrap, next_runtime::AuthorityRegistry::new())
            .expect("neutral runtime is valid");
    let mut snapshot = runtime.snapshot();
    snapshot.next_tick = revision;
    snapshot.authoritative_revision = revision;
    snapshot.ingress_checkpoint.current_tick = revision;
    snapshot
}

fn world_streaming_snapshot() -> WorldStreamingSnapshotV1 {
    WorldStreamingSnapshotV1 {
        partition_manifest_hash: ContentHash::from_bytes([0x31; 32]),
        content_manifest_hash: ContentHash::from_bytes([0x32; 32]),
        topology_revision: 1,
        generation: 2,
        current_chunk_id: SchemaId::new("nextengine.save-test.chunk").expect("chunk ID"),
        chunks: vec![WorldChunkResidencyRecordV1 {
            chunk_id: SchemaId::new("nextengine.save-test.chunk").expect("chunk ID"),
            chunk_asset: AssetRevisionRefV1 {
                asset_id: AssetId::from_bytes([0x33; 16]),
                record_sha256: ContentHash::from_bytes([0x34; 32]),
            },
            lifecycle: WorldChunkLifecycleV1::Active,
            lifecycle_revision: 3,
            required_asset_ids: vec![AssetId::from_bytes([0x35; 16])],
        }],
        pending_transition: None,
    }
}

#[test]
fn two_generations_commit_and_latest_loads() {
    let directory = TestDirectory::new();
    let store = SaveStore::new(&directory.path);
    let compatibility = compatibility(1);
    let first = store
        .commit_runtime_snapshot(compatibility.clone(), &snapshot(1))
        .expect("first generation commits");
    let second = store
        .commit_runtime_snapshot(compatibility.clone(), &snapshot(2))
        .expect("second generation commits");

    assert_eq!(first.generation, 0);
    assert_eq!(second.generation, 1);
    let loaded = store
        .load_latest(&compatibility)
        .expect("latest generation loads");
    assert_eq!(loaded.snapshot, snapshot(2));
    assert!(loaded.rejected_generations.is_empty());
}

#[test]
fn world_generation_round_trips_rpg_owner_segment() {
    let directory = TestDirectory::new();
    let store = SaveStore::new(&directory.path);
    let compatibility = compatibility(1);
    let rpg_snapshot = RpgSnapshotV2::default();

    store
        .commit_world_snapshot(compatibility.clone(), &snapshot(1), &rpg_snapshot)
        .expect("world generation commits");
    let loaded = store
        .load_latest(&compatibility)
        .expect("world generation loads");

    assert_eq!(loaded.snapshot, snapshot(1));
    assert_eq!(loaded.rpg_snapshot, rpg_snapshot);
}

#[test]
fn world_generation_round_trips_streaming_owner_segment() {
    let directory = TestDirectory::new();
    let store = SaveStore::new(&directory.path);
    let compatibility = compatibility(1);
    let checkpoint =
        synthetic_empty_checkpoint(snapshot(1), RpgSnapshotV2::default()).expect("checkpoint");
    let world = world_streaming_snapshot();

    store
        .commit_world_checkpoint_with_streaming(compatibility.clone(), &checkpoint, &world)
        .expect("streaming generation commits");
    let loaded = store
        .load_latest(&compatibility)
        .expect("streaming generation loads");

    assert_eq!(loaded.world_streaming_snapshot, Some(world));
    assert_eq!(loaded.checkpoint, checkpoint);
    assert_eq!(loaded.image.manifest.segments.len(), 4);
}

#[test]
fn corrupt_latest_generation_falls_back_and_preserves_original_bytes() {
    let directory = TestDirectory::new();
    let store = SaveStore::new(&directory.path);
    let compatibility = compatibility(1);
    store
        .commit_runtime_snapshot(compatibility.clone(), &snapshot(1))
        .expect("first generation commits");
    store
        .commit_runtime_snapshot(compatibility.clone(), &snapshot(2))
        .expect("second generation commits");
    let corrupt_path = store
        .slot_path(1)
        .join(SEGMENTS_DIRECTORY)
        .join(segment_file_name(0));
    let mut corrupt_bytes = std::fs::read(&corrupt_path).expect("segment exists");
    corrupt_bytes[0] ^= 1;
    std::fs::write(&corrupt_path, &corrupt_bytes).expect("test corrupts latest segment");

    let loaded = store
        .load_latest(&compatibility)
        .expect("prior valid generation loads");
    assert_eq!(loaded.snapshot, snapshot(1));
    assert_eq!(loaded.rejected_generations.len(), 1);
    assert!(
        loaded.rejected_generations[0]
            .original_files
            .iter()
            .any(|file| file.bytes == corrupt_bytes)
    );
    assert_eq!(
        std::fs::read(&corrupt_path).expect("load does not rewrite corrupt source"),
        corrupt_bytes
    );
}

#[test]
fn every_commit_boundary_retains_a_valid_generation() {
    let boundaries = [
        CommitBoundary::StagingCreated,
        CommitBoundary::SegmentsSynced,
        CommitBoundary::ManifestSynced,
        CommitBoundary::StagingValidated,
        CommitBoundary::InactiveSlotRemoved,
        CommitBoundary::GenerationPublished,
        CommitBoundary::PointerStaged,
        CommitBoundary::PriorPointerRemoved,
        CommitBoundary::PointerPublished,
    ];
    for boundary in boundaries {
        let directory = TestDirectory::new();
        let store = SaveStore::new(&directory.path);
        let compatibility = compatibility(1);
        store
            .commit_runtime_snapshot(compatibility.clone(), &snapshot(1))
            .expect("baseline generation commits");
        let result = store.commit_runtime_snapshot_inner(
            compatibility.clone(),
            &snapshot(2),
            Some(boundary),
        );
        assert!(result.is_err(), "boundary {boundary:?} must inject a fault");
        let loaded = store
            .load_latest(&compatibility)
            .expect("a valid generation must remain");
        assert!(
            loaded.snapshot == snapshot(1) || loaded.snapshot == snapshot(2),
            "boundary {boundary:?} loaded an unexpected state"
        );
    }
}

#[test]
fn incompatible_save_fails_closed_with_original_files() {
    let directory = TestDirectory::new();
    let store = SaveStore::new(&directory.path);
    let written = compatibility(1);
    store
        .commit_runtime_snapshot(written, &snapshot(1))
        .expect("generation commits");

    let error = store
        .load_latest(&compatibility(9))
        .expect_err("incompatible build must not load");
    let SaveLoadError::NoValidGeneration { rejected } = error;
    assert_eq!(rejected.len(), 1);
    assert!(!rejected[0].original_files.is_empty());
    assert_eq!(rejected[0].stable_code, "SAVE_COMPATIBILITY_MISMATCH");
}

#[test]
fn v1_save_is_rejected_before_snapshot_activation_and_source_is_preserved() {
    let directory = TestDirectory::new();
    let store = SaveStore::new(&directory.path);
    let compatibility = compatibility(1);
    store
        .commit_runtime_snapshot(compatibility.clone(), &snapshot(1))
        .expect("V2 generation commits");
    let manifest_path = store.slot_path(0).join(MANIFEST_FILE);
    let manifest_bytes = std::fs::read(&manifest_path).expect("manifest exists");
    let manifest_text = std::str::from_utf8(&manifest_bytes).expect("manifest is UTF-8");
    let v1_text = manifest_text.replace("\"schema_version\":2", "\"schema_version\":1");
    assert_ne!(v1_text.as_bytes(), manifest_bytes);
    std::fs::write(&manifest_path, v1_text.as_bytes()).expect("test writes V1 manifest");

    let error = store
        .load_latest(&compatibility)
        .expect_err("V1 save must not activate");
    let SaveLoadError::NoValidGeneration { rejected } = error;
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].stable_code, "UNSUPPORTED_SAVE_MANIFEST_VERSION");
    assert!(
        rejected[0]
            .original_files
            .iter()
            .any(|file| file.relative_path == MANIFEST_FILE && file.bytes == v1_text.as_bytes())
    );
    assert_eq!(
        std::fs::read(&manifest_path).expect("loader leaves V1 source untouched"),
        v1_text.as_bytes()
    );
}
