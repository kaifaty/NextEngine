use super::*;

pub(super) fn build_world_activity_catalog(
    manifest: &ProjectAuthoringManifestV6,
    population: &WorldPopulationCatalogV1,
) -> Result<WorldActivityCatalogV1, ProjectAuthoringError> {
    let authored = &manifest.world_activity_catalog;
    let worker_subject_id = population.courier_subject_id;
    let employer_character_id = persistent_id(&authored.employer_character_id)?;
    let seller_character_id = persistent_id(&authored.seller_character_id)?;
    let topic = SchemaId::new(&authored.work_topic_id)?;
    let claim = || {
        SpeechClaimV1::new(
            worker_subject_id,
            SchemaId::new(&authored.work_claim_predicate_id)?,
            SchemaId::new(&authored.work_claim_value_id)?,
            COGNITION_Q16_ONE.unsigned_abs(),
            vec![content_hash(&authored.work_claim_cited_belief_id)?],
        )
        .map_err(|_| ProjectAuthoringError::InvalidValue)
    };
    let ask = StructuredSpeechActV1::new(
        worker_subject_id,
        employer_character_id,
        SpeechActKindV1::Ask,
        topic.clone(),
        0,
        None,
        Some(SpeechActKindV1::Inform),
        None,
        authored.work_exchange_tick,
        authored.work_exchange_expiry_tick,
    )
    .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    let inform = StructuredSpeechActV1::new(
        employer_character_id,
        worker_subject_id,
        SpeechActKindV1::Inform,
        topic.clone(),
        1,
        Some(claim()?),
        None,
        Some(ask.act_id),
        authored.work_exchange_tick,
        authored.work_exchange_expiry_tick,
    )
    .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    let offer = StructuredSpeechActV1::new(
        employer_character_id,
        worker_subject_id,
        SpeechActKindV1::Offer,
        topic.clone(),
        2,
        Some(claim()?),
        Some(SpeechActKindV1::Accept),
        None,
        authored.work_exchange_tick,
        authored.work_exchange_expiry_tick,
    )
    .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    let accept = StructuredSpeechActV1::new(
        worker_subject_id,
        employer_character_id,
        SpeechActKindV1::Accept,
        topic,
        3,
        None,
        None,
        Some(offer.act_id),
        authored.work_exchange_tick,
        authored.work_exchange_expiry_tick,
    )
    .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    let threat_claim = SpeechClaimV1::new(
        worker_subject_id,
        SchemaId::new(&authored.threat_claim_predicate_id)?,
        SchemaId::new(&authored.threat_claim_value_id)?,
        COGNITION_Q16_ONE.unsigned_abs(),
        vec![content_hash(&authored.threat_claim_cited_belief_id)?],
    )
    .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    let threat_act = StructuredSpeechActV1::new(
        seller_character_id,
        worker_subject_id,
        SpeechActKindV1::Threaten,
        SchemaId::new(&authored.threat_topic_id)?,
        0,
        Some(threat_claim),
        None,
        None,
        authored.threat_tick,
        authored.threat_expiry_tick,
    )
    .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    let value = WorldActivityCatalogV1 {
        schema_version: authored.schema_version,
        catalog_asset_id: asset_id(&authored.catalog_asset_id)?,
        worker_subject_id,
        commitment_id: persistent_id(&authored.commitment_id)?,
        work_id: SchemaId::new(&authored.work_id)?,
        workplace_node_id: SchemaId::new(&authored.workplace_node_id)?,
        work_duration_ticks: authored.work_duration_ticks,
        systemic_work: SystemicWorkProfileV1 {
            employer_character_id,
            seller_character_id,
            worker_inventory_id: persistent_id(&authored.worker_inventory_id)?,
            seller_inventory_id: persistent_id(&authored.seller_inventory_id)?,
            food_item_id: persistent_id(&authored.food_item_id)?,
            currency_resource_id: SchemaId::new(&authored.currency_resource_id)?,
            hunger_resource_id: SchemaId::new(&authored.hunger_resource_id)?,
            satiety_resource_id: SchemaId::new(&authored.satiety_resource_id)?,
            wage_amount: authored.wage_amount,
            food_price: authored.food_price,
            hunger_restore_amount: authored.hunger_restore_amount,
            satiety_gain_amount: authored.satiety_gain_amount,
            listener_trust_q16: authored.listener_trust_q16,
            social_action_id: SchemaId::new(&authored.social_action_id)?,
            await_activity_action_id: SchemaId::new(&authored.await_activity_action_id)?,
            settlement_action_id: SchemaId::new(&authored.settlement_action_id)?,
            social_ready_fact_id: SchemaId::new(&authored.social_ready_fact_id)?,
            activity_ready_fact_id: SchemaId::new(&authored.activity_ready_fact_id)?,
            settlement_ready_fact_id: SchemaId::new(&authored.settlement_ready_fact_id)?,
            work_exchange: StructuredSpeechExchangeV1 {
                acts: vec![ask, inform, offer, accept],
            },
            threat_act,
        },
    };
    value
        .validate()
        .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    Ok(value)
}

pub(super) fn build_world_navigation_catalog(
    manifest: &ProjectAuthoringManifestV6,
    chunks: &[SourceChunkBindingV1],
) -> Result<WorldNavigationCatalogV1, ProjectAuthoringError> {
    let authored = &manifest.world_navigation_catalog;
    if authored.schema_version != next_contracts::world_population::WORLD_POPULATION_SCHEMA_VERSION
        || authored.topology_revision != manifest.project.project_revision
        || authored.edge_cost == 0
        || chunks.len() < 2
    {
        return Err(ProjectAuthoringError::InvalidValue);
    }
    let mut nodes = chunks
        .iter()
        .map(|chunk| {
            Ok(WorldNavigationNodeV1 {
                node_id: chunk.chunk_id.clone(),
                tile_id: SchemaId::new(format!("{}.navigation-tile", chunk.region_id.as_str()))?,
                chunk_id: chunk.chunk_id.clone(),
            })
        })
        .collect::<Result<Vec<_>, ProjectAuthoringError>>()?;
    nodes.sort();
    let mut region_tiles = BTreeMap::<SchemaId, SchemaId>::new();
    for chunk in chunks {
        region_tiles.insert(
            chunk.region_id.clone(),
            SchemaId::new(format!("{}.navigation-tile", chunk.region_id.as_str()))?,
        );
    }
    let mut tiles = region_tiles
        .into_iter()
        .map(|(region_id, tile_id)| {
            let node_ids = nodes
                .iter()
                .filter(|node| node.tile_id == tile_id)
                .map(|node| &node.node_id)
                .collect::<Vec<_>>();
            Ok(WorldNavigationTileV1 {
                tile_revision: navigation_tile_revision(&tile_id, &region_id, &node_ids)
                    .map_err(|_| ProjectAuthoringError::InvalidValue)?,
                tile_id,
                region_id,
            })
        })
        .collect::<Result<Vec<_>, ProjectAuthoringError>>()?;
    tiles.sort();
    let mut edges = Vec::with_capacity(nodes.len());
    for index in 0..nodes.len() {
        let left = &nodes[index].node_id;
        let right = &nodes[(index + 1) % nodes.len()].node_id;
        let (node_low, node_high) = if left < right {
            (left.clone(), right.clone())
        } else {
            (right.clone(), left.clone())
        };
        edges.push(WorldNavigationEdgeV1 {
            node_low,
            node_high,
            cost: authored.edge_cost,
            capability: NavigationCapabilityV1::AbstractTransfer,
        });
    }
    edges.sort();
    if edges.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ProjectAuthoringError::DuplicateIdentity);
    }
    let value = WorldNavigationCatalogV1 {
        schema_version: authored.schema_version,
        catalog_asset_id: asset_id(&authored.catalog_asset_id)?,
        topology_revision: authored.topology_revision,
        tiles,
        nodes,
        edges,
    };
    value
        .validate()
        .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    Ok(value)
}

pub(super) fn build_world_population_catalog(
    manifest: &ProjectAuthoringManifestV6,
    navigation: &WorldNavigationCatalogV1,
) -> Result<WorldPopulationCatalogV1, ProjectAuthoringError> {
    let authored = &manifest.world_population_catalog;
    if authored.schema_version != next_contracts::world_population::WORLD_POPULATION_SCHEMA_VERSION
        || authored.identity_domain.is_empty()
        || usize::try_from(authored.courier_ordinal)
            .ok()
            .is_none_or(|ordinal| ordinal >= WORLD_POPULATION_COUNT_V1)
    {
        return Err(ProjectAuthoringError::InvalidValue);
    }
    let courier_initial_node_id = SchemaId::new(&authored.courier_initial_node_id)?;
    let courier_goal_node_id = SchemaId::new(&authored.courier_goal_node_id)?;
    let node_count = navigation.nodes.len();
    let mut records = Vec::with_capacity(WORLD_POPULATION_COUNT_V1);
    for ordinal in 0..WORLD_POPULATION_COUNT_V1 {
        let subject_id = population_subject_id(&authored.identity_domain, ordinal)?;
        let (cadence_class, initial_tier) = if ordinal < WORLD_POPULATION_ACTIVE_COUNT_V1 {
            (PopulationCadenceClassV1::Active, PopulationTierV1::Active)
        } else if ordinal < WORLD_POPULATION_ACTIVE_COUNT_V1 + WORLD_POPULATION_NEAR_COUNT_V1 {
            (PopulationCadenceClassV1::Near, PopulationTierV1::Simulated)
        } else if ordinal
            == usize::try_from(authored.courier_ordinal)
                .map_err(|_| ProjectAuthoringError::InvalidValue)?
        {
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
        let initial_node_id = if ordinal
            == usize::try_from(authored.courier_ordinal)
                .map_err(|_| ProjectAuthoringError::InvalidValue)?
        {
            courier_initial_node_id.clone()
        } else {
            navigation.nodes[ordinal % node_count].node_id.clone()
        };
        let navigation_goal_node_id = if ordinal
            == usize::try_from(authored.courier_ordinal)
                .map_err(|_| ProjectAuthoringError::InvalidValue)?
        {
            courier_goal_node_id.clone()
        } else {
            navigation.nodes[(ordinal + node_count / 2) % node_count]
                .node_id
                .clone()
        };
        records.push(WorldPopulationDefinitionV1 {
            subject_id,
            home_node_id: initial_node_id.clone(),
            initial_node_id,
            navigation_goal_node_id,
            cadence_class,
            cadence_phase: population_cadence_phase(subject_id, cadence_class),
            initial_tier,
        });
    }
    records.sort();
    let courier_ordinal = usize::try_from(authored.courier_ordinal)
        .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    let courier_subject_id = records
        .iter()
        .find(|record| {
            record.initial_node_id == courier_initial_node_id
                && record.initial_tier == PopulationTierV1::Dormant
        })
        .map(|record| record.subject_id)
        .ok_or(ProjectAuthoringError::InvalidValue)?;
    let _ = courier_ordinal;
    debug_assert_eq!(
        records
            .iter()
            .filter(|record| record.cadence_class == PopulationCadenceClassV1::Background)
            .count(),
        WORLD_POPULATION_BACKGROUND_COUNT_V1
    );
    let value = WorldPopulationCatalogV1 {
        schema_version: authored.schema_version,
        catalog_asset_id: asset_id(&authored.catalog_asset_id)?,
        navigation_catalog_asset_id: navigation.catalog_asset_id,
        navigation_catalog_revision: navigation
            .revision()
            .map_err(|_| ProjectAuthoringError::InvalidValue)?,
        courier_subject_id,
        courier_transition_start_tick: authored.courier_transition_start_tick,
        records,
    };
    value
        .validate_against_navigation(navigation)
        .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    Ok(value)
}

pub(super) fn population_subject_id(
    identity_domain: &str,
    ordinal: usize,
) -> Result<PersistentId, ProjectAuthoringError> {
    let mut preimage = b"nextengine.population-subject.v1\0".to_vec();
    append_string(&mut preimage, identity_domain)?;
    preimage.extend_from_slice(
        &u32::try_from(ordinal)
            .map_err(|_| ProjectAuthoringError::InvalidValue)?
            .to_le_bytes(),
    );
    let digest = next_contracts::canonical::sha256(&preimage);
    let mut subject_bytes = [0_u8; 16];
    subject_bytes.copy_from_slice(&digest[..16]);
    Ok(PersistentId::from_bytes(subject_bytes))
}
