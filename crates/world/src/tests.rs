use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::{ContentStore, PinnedContentGeneration};
use next_project::{activate_project_package, cook_project_v4};

use super::*;

mod partition;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn worker_counts_and_result_permutations_are_deterministic() {
    let (project, generation, _root) = fixture_project("permutations");
    let (initial, target) = first_two_chunk_ids(&project);
    let mut roots = Vec::new();
    let mut result_hashes = Vec::new();
    for workers in [1, 2, 4] {
        let mut streamer =
            WorldStreamerV1::activate(project.clone(), generation.clone(), initial.clone())
                .expect("activate");
        publish_begin(&mut streamer, target.clone(), 7);
        let loaded = streamer.load_pending(workers).expect("packaged load");
        assert_eq!(
            loaded.metrics(),
            WorldStreamingLoadMetricsV1 {
                asset_count: 9,
                encoded_bytes: 6_714,
                required_staging_bytes: 6_714,
            }
        );
        result_hashes.push(loaded.result_hash());
        publish_completion(&mut streamer, loaded, 8);
        roots.push(streamer.snapshot().state_hash().expect("state hash"));
    }
    assert!(result_hashes.windows(2).all(|pair| pair[0] == pair[1]));
    assert!(roots.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn transition_round_trip_preserves_generation_and_unloads_previous_chunk() {
    let (project, generation, _root) = fixture_project("round-trip");
    let (start, frontier) = first_two_chunk_ids(&project);
    let mut streamer =
        WorldStreamerV1::activate(project, generation, start.clone()).expect("activate");
    execute_transition(&mut streamer, frontier, 10);
    execute_transition(&mut streamer, start.clone(), 20);
    assert_eq!(streamer.snapshot().generation, 2);
    assert_eq!(streamer.snapshot().current_chunk_id, start);
    assert!(streamer.snapshot().pending_transition.is_none());
    assert_eq!(
        streamer
            .snapshot()
            .chunks
            .iter()
            .filter(|chunk| chunk.lifecycle == WorldChunkLifecycleV1::Active)
            .count(),
        1
    );
}

#[test]
fn load_fault_and_stale_publication_preserve_requested_root_and_cache() {
    let (project, generation, _root) = fixture_project("faults");
    let (start, frontier) = first_two_chunk_ids(&project);
    let mut streamer = WorldStreamerV1::activate(project, generation, start).expect("activate");
    publish_begin(&mut streamer, frontier, 42);
    let requested = streamer.snapshot().clone();
    let cache = streamer.active_records().to_vec();
    let request = streamer.pending_request().expect("request");
    let panic_asset = request.ordered_asset_revisions[0].asset_id;
    assert!(matches!(
        streamer.load_request(request, 2, Some(panic_asset)),
        Err(WorldStreamingError::AssetLoad {
            code: WorldAssetLoadErrorCodeV1::WorkerPanic,
            ..
        })
    ));
    assert_eq!(streamer.snapshot(), &requested);
    assert_eq!(streamer.active_records(), cache);

    let loaded = streamer.load_pending(2).expect("load");
    let prepared = streamer
        .prepare_loaded_commit(loaded, 43)
        .expect("prepare completion");
    let stale_tick = prepared.expected_gameplay_tick() + 1;
    assert_eq!(
        streamer.validate_prepared_publication(prepared, stale_tick),
        Err(WorldStreamingError::PublicationStale)
    );
    assert_eq!(streamer.snapshot(), &requested);
    assert_eq!(streamer.active_records(), cache);
}

#[test]
fn semantic_completion_fault_matrix_never_publishes_partial_state() {
    let (project, generation, _root) = fixture_project("semantic-faults");
    let (start, frontier) = first_two_chunk_ids(&project);
    let mut streamer = WorldStreamerV1::activate(project, generation, start).expect("activate");
    publish_begin(&mut streamer, frontier, 51);
    let requested = streamer.snapshot().clone();
    let cache = streamer.active_records().to_vec();
    let baseline = streamer.load_pending(2).expect("baseline load");

    let mut corrupt_hash = baseline.clone();
    corrupt_hash.result_hash = ContentHash::from_bytes([0xee; 32]);
    assert_eq!(
        streamer.prepare_loaded_commit(corrupt_hash, 52),
        Err(WorldStreamingError::ResultCorrupt)
    );

    let mut wrong_schema = baseline.clone();
    wrong_schema.records[0].schema_ref.schema_version += 1;
    wrong_schema.result_hash = wrong_schema.computed_hash().expect("schema result hash");
    assert!(matches!(
        streamer.prepare_loaded_commit(wrong_schema, 52),
        Err(WorldStreamingError::AssetLoad {
            code: WorldAssetLoadErrorCodeV1::Schema,
            ..
        })
    ));

    let mut wrong_asset = baseline.clone();
    wrong_asset.records[0].asset_id = AssetId::from_bytes([0xfd; 16]);
    wrong_asset.result_hash = wrong_asset.computed_hash().expect("asset result hash");
    assert!(matches!(
        streamer.prepare_loaded_commit(wrong_asset, 52),
        Err(WorldStreamingError::AssetLoad {
            code: WorldAssetLoadErrorCodeV1::AssetIdentity,
            ..
        })
    ));

    let mut wrong_revision = baseline.clone();
    wrong_revision.records[0].record_id = PersistentId::from_bytes([0xfc; 16]);
    wrong_revision.ordered_record_ids = wrong_revision
        .records
        .iter()
        .map(|record| record.record_id)
        .collect();
    wrong_revision.ordered_record_ids.sort_unstable();
    wrong_revision.result_hash = wrong_revision
        .computed_hash()
        .expect("revision result hash");
    assert!(matches!(
        streamer.prepare_loaded_commit(wrong_revision, 52),
        Err(WorldStreamingError::AssetLoad {
            code: WorldAssetLoadErrorCodeV1::RecordRevision,
            ..
        })
    ));

    let mut duplicate_id = baseline.clone();
    duplicate_id.records[1].record_id = duplicate_id.records[0].record_id;
    duplicate_id.ordered_record_ids = duplicate_id
        .records
        .iter()
        .map(|record| record.record_id)
        .collect();
    duplicate_id.ordered_record_ids.sort_unstable();
    duplicate_id.result_hash = duplicate_id.computed_hash().expect("duplicate result hash");
    assert_eq!(
        streamer.prepare_loaded_commit(duplicate_id, 52),
        Err(WorldStreamingError::ObjectIdCollision)
    );

    let loaded_ids = baseline
        .records
        .iter()
        .map(|record| record.asset_id)
        .collect::<BTreeSet<_>>();
    let external_dependency = baseline
        .records
        .iter()
        .flat_map(|record| record.asset_dependencies.iter().copied())
        .find(|dependency| !loaded_ids.contains(dependency))
        .expect("project-global dependency");
    Arc::make_mut(&mut streamer.project)
        .content_manifest
        .body
        .asset_entries
        .retain(|entry| entry.asset_revision.asset_id != external_dependency);
    assert!(matches!(
        streamer.prepare_loaded_commit(baseline, 52),
        Err(WorldStreamingError::AssetLoad {
            code: WorldAssetLoadErrorCodeV1::MissingDependency,
            ..
        })
    ));

    assert_eq!(streamer.snapshot(), &requested);
    assert_eq!(streamer.active_records(), cache);
}

fn execute_transition(streamer: &mut WorldStreamerV1, target: SchemaId, tick: u64) {
    publish_begin(streamer, target, tick);
    let loaded = streamer.load_pending(2).expect("load");
    publish_completion(streamer, loaded, tick + 1);
}

fn run_route(
    project: &ActivatedProjectV5,
    generation: &PinnedContentGeneration,
    route: &[SchemaId],
    workers: usize,
) -> (Vec<ContentHash>, ContentHash, WorldStreamingSnapshotV1) {
    let mut streamer =
        WorldStreamerV1::activate(project.clone(), generation.clone(), route[0].clone())
            .expect("activate route");
    let mut result_hashes = Vec::with_capacity(route.len() - 1);
    let mut tick = 1_u64;
    for target in &route[1..] {
        publish_begin(&mut streamer, target.clone(), tick);
        let loaded = streamer.load_pending(workers).expect("route load");
        result_hashes.push(loaded.result_hash());
        publish_completion(&mut streamer, loaded, tick + 1);
        tick += 2;
    }
    let snapshot = streamer.snapshot().clone();
    let root = snapshot.state_hash().expect("route root");
    (result_hashes, root, snapshot)
}

fn publish_begin(streamer: &mut WorldStreamerV1, target: SchemaId, tick: u64) {
    let prepared = streamer
        .prepare_begin_transition(target, tick)
        .expect("prepare begin");
    let validated = streamer
        .validate_prepared_publication(prepared, tick)
        .expect("validate begin");
    assert!(streamer.commit_validated_publication(validated).is_none());
}

fn publish_completion(streamer: &mut WorldStreamerV1, loaded: PreparedWorldChunkLoadV1, tick: u64) {
    let prepared = streamer
        .prepare_loaded_commit(loaded, tick)
        .expect("prepare completion");
    let validated = streamer
        .validate_prepared_publication(prepared, tick)
        .expect("validate completion");
    streamer
        .commit_validated_publication(validated)
        .expect("transition receipt");
}

fn fixture_project(
    label: &str,
) -> (
    ActivatedProjectV5,
    PinnedContentGeneration,
    std::path::PathBuf,
) {
    let cooked = cook_project_v4(next_reference_game::project_source_v4().expect("fixture source"))
        .expect("cook fixture");
    let root = std::env::temp_dir().join(format!(
        "nextengine-world-{label}-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let store = ContentStore::new(&root);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let package = activate_project_package(&store).expect("activate package");
    (package.project, package.content_generation, root)
}

fn first_two_chunk_ids(project: &ActivatedProjectV5) -> (SchemaId, SchemaId) {
    let topology = next_reference_game::ReferenceWorldTopologyV1::from_activated_project(project)
        .expect("reference topology");
    (
        topology.initial_chunk_id().clone(),
        topology.gameplay_target_chunk_id().clone(),
    )
}
