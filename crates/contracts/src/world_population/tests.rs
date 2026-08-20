use super::*;

fn fixture() -> (WorldNavigationCatalogV1, WorldPopulationCatalogV1) {
    let region_a = SchemaId::new("nextengine.test.region.a").expect("region");
    let region_b = SchemaId::new("nextengine.test.region.b").expect("region");
    let node_a = SchemaId::new("nextengine.test.chunk.a").expect("node");
    let node_b = SchemaId::new("nextengine.test.chunk.b").expect("node");
    let tile_a = SchemaId::new("nextengine.test.tile.a").expect("tile");
    let tile_b = SchemaId::new("nextengine.test.tile.b").expect("tile");
    let mut navigation = WorldNavigationCatalogV1 {
        schema_version: WORLD_POPULATION_SCHEMA_VERSION,
        catalog_asset_id: AssetId::from_bytes([0x96; 16]),
        topology_revision: 1,
        tiles: vec![
            WorldNavigationTileV1 {
                tile_id: tile_a.clone(),
                region_id: region_a,
                tile_revision: navigation_tile_revision(
                    &tile_a,
                    &SchemaId::new("nextengine.test.region.a").expect("region"),
                    &[&node_a],
                )
                .expect("revision"),
            },
            WorldNavigationTileV1 {
                tile_id: tile_b.clone(),
                region_id: region_b,
                tile_revision: navigation_tile_revision(
                    &tile_b,
                    &SchemaId::new("nextengine.test.region.b").expect("region"),
                    &[&node_b],
                )
                .expect("revision"),
            },
        ],
        nodes: vec![
            WorldNavigationNodeV1 {
                node_id: node_a.clone(),
                tile_id: tile_a,
                chunk_id: node_a.clone(),
            },
            WorldNavigationNodeV1 {
                node_id: node_b.clone(),
                tile_id: tile_b,
                chunk_id: node_b.clone(),
            },
        ],
        edges: vec![WorldNavigationEdgeV1 {
            node_low: node_a.clone(),
            node_high: node_b.clone(),
            cost: 1_000,
            capability: NavigationCapabilityV1::AbstractTransfer,
        }],
    };
    navigation.tiles.sort();
    navigation.nodes.sort();
    navigation.edges.sort();
    let navigation_revision = navigation.revision().expect("navigation revision");
    let mut records = (0_u8..100)
        .map(|ordinal| {
            let subject_id = PersistentId::from_bytes([ordinal; 16]);
            let (cadence_class, initial_tier) = if ordinal < 16 {
                (PopulationCadenceClassV1::Active, PopulationTierV1::Active)
            } else if ordinal < 48 {
                (PopulationCadenceClassV1::Near, PopulationTierV1::Simulated)
            } else if ordinal == 99 {
                (
                    PopulationCadenceClassV1::Background,
                    PopulationTierV1::Dormant,
                )
            } else {
                (
                    PopulationCadenceClassV1::Background,
                    PopulationTierV1::Abstract,
                )
            };
            WorldPopulationDefinitionV1 {
                subject_id,
                home_node_id: node_a.clone(),
                initial_node_id: node_a.clone(),
                navigation_goal_node_id: node_b.clone(),
                cadence_class,
                cadence_phase: population_cadence_phase(subject_id, cadence_class),
                initial_tier,
            }
        })
        .collect::<Vec<_>>();
    records.sort();
    let population = WorldPopulationCatalogV1 {
        schema_version: WORLD_POPULATION_SCHEMA_VERSION,
        catalog_asset_id: AssetId::from_bytes([0x95; 16]),
        navigation_catalog_asset_id: navigation.catalog_asset_id,
        navigation_catalog_revision: navigation_revision,
        courier_subject_id: PersistentId::from_bytes([99; 16]),
        courier_transition_start_tick: 1,
        records,
    };
    (navigation, population)
}

#[test]
fn catalogs_and_snapshot_are_current_canonical_closure() {
    let (navigation, population) = fixture();
    population
        .validate_against_navigation(&navigation)
        .expect("population closure");
    let navigation_bytes = navigation.canonical_bytes().expect("navigation bytes");
    assert_eq!(
        WorldNavigationCatalogV1::from_canonical_bytes(
            &navigation_bytes,
            CanonicalDecodeLimits::default(),
        )
        .expect("navigation decode"),
        navigation
    );
    let population_bytes = population.canonical_bytes().expect("population bytes");
    assert_eq!(
        WorldPopulationCatalogV1::from_canonical_bytes(
            &population_bytes,
            CanonicalDecodeLimits::default(),
        )
        .expect("population decode"),
        population
    );
    let snapshot =
        WorldPopulationSnapshotV1::initial(&population, &navigation).expect("initial snapshot");
    let snapshot_bytes = snapshot.canonical_bytes().expect("snapshot bytes");
    let decoded = WorldPopulationSnapshotV1::from_canonical_bytes(
        &snapshot_bytes,
        CanonicalDecodeLimits::default(),
    )
    .expect("snapshot decode");
    decoded
        .validate_against(&population, &navigation, 0)
        .expect("snapshot closure");
}

#[test]
fn cadence_distribution_and_active_transfer_boundary_are_exact() {
    let (navigation, population) = fixture();
    let mut due = [0_u64; 3];
    for tick in 0_u64..60 {
        for record in &population.records {
            if tick % record.cadence_class.period_ticks() == record.cadence_phase {
                due[usize::from(record.cadence_class as u8 - 1)] += 1;
            }
        }
    }
    assert_eq!(due, [320, 128, 52]);

    let snapshot =
        WorldPopulationSnapshotV1::initial(&population, &navigation).expect("initial snapshot");
    let courier = snapshot
        .record(population.courier_subject_id)
        .expect("courier");
    assert_eq!(courier.tier, PopulationTierV1::Dormant);
    assert_eq!(
        WorldPopulationContractError::PhysicalTraversalRequired.diagnostic_code(),
        "PHYSICAL_TRAVERSAL_REQUIRED"
    );
}

#[test]
fn prevalidated_snapshot_path_rejects_revision_mismatch() {
    let (navigation, population) = fixture();
    let snapshot =
        WorldPopulationSnapshotV1::initial(&population, &navigation).expect("initial snapshot");
    let population_revision = population
        .revision(&navigation)
        .expect("population revision");
    let navigation_revision = navigation.revision().expect("navigation revision");

    assert_eq!(
        snapshot.validate_against_prevalidated_revisions(
            &population,
            &navigation,
            0,
            ContentHash::from_bytes([0; 32]),
            navigation_revision,
        ),
        Err(WorldPopulationContractError::SnapshotClosureInvalid)
    );
    assert_eq!(
        snapshot.validate_against_prevalidated_revisions(
            &population,
            &navigation,
            0,
            population_revision,
            ContentHash::from_bytes([0; 32]),
        ),
        Err(WorldPopulationContractError::SnapshotClosureInvalid)
    );
}
