use super::*;

#[test]
fn multiregion_routes_workers_and_cache_warmth_are_deterministic() {
    let (project, generation, _root) = fixture_project("multiregion-routes");
    let topology = next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&project)
        .expect("reference topology");
    let canonical_route = topology
        .ordered_multiregion_route()
        .iter()
        .map(|entry| entry.chunk_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(canonical_route.len(), 64);
    assert_eq!(project.world_partition.body.root_region_ids.len(), 4);

    let mut canonical_results = Vec::new();
    for workers in [1, 2, 4] {
        for _warmth in 0..2 {
            let (result_hashes, final_root, snapshot) =
                run_route(&project, &generation, &canonical_route, workers);
            canonical_results.push((result_hashes, final_root));
            assert_eq!(snapshot.generation, 63);
            assert_eq!(snapshot.current_chunk_id, canonical_route[63]);
            assert_eq!(
                snapshot
                    .chunks
                    .iter()
                    .filter(|chunk| chunk.lifecycle == WorldChunkLifecycleV1::Active)
                    .count(),
                1
            );
            assert!(snapshot.chunks.iter().all(|chunk| {
                chunk.chunk_id == canonical_route[63]
                    || chunk.lifecycle == WorldChunkLifecycleV1::Unloaded
            }));
        }
    }
    assert!(canonical_results.windows(2).all(|pair| pair[0] == pair[1]));

    let final_chunk = canonical_route.last().expect("final chunk").clone();
    let mut reversed_middle = canonical_route[1..canonical_route.len() - 1].to_vec();
    reversed_middle.reverse();
    reversed_middle.insert(0, canonical_route[0].clone());
    reversed_middle.push(final_chunk.clone());
    let mut permuted_middle = canonical_route[1..canonical_route.len() - 1]
        .iter()
        .step_by(2)
        .cloned()
        .chain(
            canonical_route[2..canonical_route.len() - 1]
                .iter()
                .step_by(2)
                .cloned(),
        )
        .collect::<Vec<_>>();
    permuted_middle.insert(0, canonical_route[0].clone());
    permuted_middle.push(final_chunk);
    let canonical_root = canonical_results[0].1;
    assert_eq!(
        run_route(&project, &generation, &reversed_middle, 2).1,
        canonical_root
    );
    assert_eq!(
        run_route(&project, &generation, &permuted_middle, 2).1,
        canonical_root
    );
}

#[test]
fn multiregion_mandatory_request_survives_worker_fault_and_retries() {
    let (project, generation, _root) = fixture_project("multiregion-retry");
    let topology = next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&project)
        .expect("reference topology");
    let target = topology
        .ordered_multiregion_route()
        .iter()
        .find(|entry| entry.region_id.as_str().ends_with("high-pass"))
        .expect("filler target")
        .chunk_id
        .clone();
    let mut streamer =
        WorldStreamerV1::activate(project, generation, topology.initial_chunk_id().clone())
            .expect("activate");
    publish_begin(&mut streamer, target.clone(), 70);
    let requested = streamer.snapshot().clone();
    let cache = streamer.active_records().to_vec();
    let request = streamer.pending_request().expect("mandatory request");
    let panic_asset = request.ordered_asset_revisions[0].asset_id;
    assert!(matches!(
        streamer.load_request(request, 4, Some(panic_asset)),
        Err(WorldStreamingError::AssetLoad {
            code: WorldAssetLoadErrorCodeV1::WorkerPanic,
            ..
        })
    ));
    assert_eq!(streamer.snapshot(), &requested);
    assert_eq!(streamer.active_records(), cache);
    let loaded = streamer.load_pending(1).expect("retry mandatory request");
    publish_completion(&mut streamer, loaded, 71);
    assert_eq!(streamer.snapshot().current_chunk_id, target);
    assert_eq!(streamer.snapshot().generation, 1);
}

#[test]
fn save_restore_refetches_requested_transition() {
    let (project, generation, _root) = fixture_project("restore");
    let (start, frontier) = first_two_chunk_ids(&project);
    let mut streamer =
        WorldStreamerV1::activate(project.clone(), generation.clone(), start).expect("activate");
    publish_begin(&mut streamer, frontier.clone(), 31);
    let saved = streamer.snapshot().clone();
    let bytes = saved.canonical_bytes().expect("snapshot bytes");
    let decoded =
        WorldStreamingSnapshotV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("decode");
    let mut restored = WorldStreamerV1::restore(project, generation, decoded).expect("restore");
    let loaded = restored.load_pending(2).expect("refetch");
    publish_completion(&mut restored, loaded, 32);
    assert_eq!(restored.snapshot().current_chunk_id, frontier);
    assert_eq!(restored.snapshot().generation, 1);
}
