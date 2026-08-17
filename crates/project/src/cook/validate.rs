use super::*;

pub(super) fn validate_source(source: &NeutralProjectSourceV7) -> Result<(), ProjectCookError> {
    if source.project_revision == 0 {
        return Err(ProjectCookError::InvalidRevision);
    }
    if source.allowed_presentation_targets.is_empty() {
        return Err(ProjectCookError::InvalidValue);
    }
    ensure_unique(
        source
            .records
            .iter()
            .map(|record| record.asset_id)
            .chain(
                source
                    .render_records
                    .iter()
                    .map(NeutralRenderRecordV1::asset_id),
            )
            .chain(
                source
                    .text_catalogs
                    .iter()
                    .map(|catalog| catalog.catalog_asset_id),
            )
            .chain(source.audio_records.iter().map(|record| record.asset_id))
            .chain(source.skeletons.iter().map(|record| record.asset_id))
            .chain(source.animations.iter().map(|record| record.asset_id))
            .chain([source.body_schema_asset.asset_id])
            .chain(
                source
                    .world_routine_catalog_or_none
                    .iter()
                    .map(|catalog| catalog.catalog_asset_id),
            )
            .chain([source.world_navigation_catalog.catalog_asset_id])
            .chain([source.world_population_catalog.catalog_asset_id])
            .chain([source.agent_cognition_catalog.catalog_asset_id])
            .chain([source.world_activity_catalog.catalog_asset_id]),
    )?;
    ensure_unique(source.records.iter().map(|record| record.record_id))?;
    if source
        .records
        .iter()
        .filter(|record| record.kind == NeutralRecordKindV1::CharacterDefinition)
        .filter(|record| {
            record
                .asset_dependencies
                .contains(&source.body_schema_asset.asset_id)
        })
        .count()
        != 1
    {
        return Err(ProjectCookError::MissingReference);
    }
    for animation in &source.animations {
        let skeleton = source
            .skeletons
            .iter()
            .find(|skeleton| skeleton.asset_id == animation.skeleton_revision.asset_id)
            .ok_or(ProjectCookError::MissingReference)?;
        if skeleton.asset_revision()? != animation.skeleton_revision
            || animation.channels.iter().any(|channel| {
                !skeleton
                    .joints
                    .iter()
                    .any(|joint| joint.joint_key == channel.joint_key)
            })
        {
            return Err(ProjectCookError::MissingReference);
        }
    }
    ensure_unique(source.root_asset_ids.iter().copied())?;
    ensure_unique(source.chunks.iter().map(|chunk| chunk.chunk_id.as_str()))?;
    ensure_unique(source.chunks.iter().map(|chunk| chunk.chunk_asset_id))?;
    let assets: BTreeSet<_> = source
        .records
        .iter()
        .map(|record| record.asset_id)
        .chain(
            source
                .render_records
                .iter()
                .map(NeutralRenderRecordV1::asset_id),
        )
        .chain(
            source
                .text_catalogs
                .iter()
                .map(|record| record.catalog_asset_id),
        )
        .chain(source.audio_records.iter().map(|record| record.asset_id))
        .chain(source.skeletons.iter().map(|record| record.asset_id))
        .chain(source.animations.iter().map(|record| record.asset_id))
        .chain([source.body_schema_asset.asset_id])
        .chain(
            source
                .world_routine_catalog_or_none
                .iter()
                .map(|catalog| catalog.catalog_asset_id),
        )
        .chain([source.world_navigation_catalog.catalog_asset_id])
        .chain([source.world_population_catalog.catalog_asset_id])
        .chain([source.agent_cognition_catalog.catalog_asset_id])
        .chain([source.world_activity_catalog.catalog_asset_id])
        .collect();
    let mut revisions = BTreeMap::new();
    for record in &source.records {
        revisions.insert(record.asset_id, asset_revision(record)?);
    }
    for record in &source.render_records {
        revisions.insert(record.asset_id(), record.asset_revision()?);
    }
    for record in &source.skeletons {
        revisions.insert(record.asset_id, record.asset_revision()?);
    }
    for record in &source.animations {
        revisions.insert(record.asset_id, record.asset_revision()?);
    }
    source
        .body_schema_asset
        .validate()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    revisions.insert(
        source.body_schema_asset.asset_id,
        AssetRevisionRefV1 {
            asset_id: source.body_schema_asset.asset_id,
            record_sha256: source
                .body_schema_asset
                .record_sha256()
                .map_err(|_| ProjectCookError::InvalidValue)?,
        },
    );
    source
        .world_navigation_catalog
        .validate()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    source
        .world_population_catalog
        .validate_against_navigation(&source.world_navigation_catalog)
        .map_err(|_| ProjectCookError::InvalidValue)?;
    source
        .agent_cognition_catalog
        .validate()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    source
        .world_activity_catalog
        .validate()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    if source.world_navigation_catalog.topology_revision != source.project_revision
        || source.world_population_catalog.navigation_catalog_asset_id
            != source.world_navigation_catalog.catalog_asset_id
        || source
            .world_population_catalog
            .definition(source.agent_cognition_catalog.subject_id)
            .is_none()
        || source.world_activity_catalog.worker_subject_id
            != source.agent_cognition_catalog.subject_id
        || source
            .world_population_catalog
            .definition(source.world_activity_catalog.worker_subject_id)
            .is_none_or(|definition| {
                definition.navigation_goal_node_id
                    != source.world_activity_catalog.workplace_node_id
            })
    {
        return Err(ProjectCookError::InvalidValue);
    }
    revisions.insert(
        source.world_navigation_catalog.catalog_asset_id,
        AssetRevisionRefV1 {
            asset_id: source.world_navigation_catalog.catalog_asset_id,
            record_sha256: source
                .world_navigation_catalog
                .revision()
                .map_err(|_| ProjectCookError::InvalidValue)?,
        },
    );
    revisions.insert(
        source.world_population_catalog.catalog_asset_id,
        AssetRevisionRefV1 {
            asset_id: source.world_population_catalog.catalog_asset_id,
            record_sha256: source
                .world_population_catalog
                .revision(&source.world_navigation_catalog)
                .map_err(|_| ProjectCookError::InvalidValue)?,
        },
    );
    revisions.insert(
        source.agent_cognition_catalog.catalog_asset_id,
        AssetRevisionRefV1 {
            asset_id: source.agent_cognition_catalog.catalog_asset_id,
            record_sha256: source
                .agent_cognition_catalog
                .revision()
                .map_err(|_| ProjectCookError::InvalidValue)?,
        },
    );
    revisions.insert(
        source.world_activity_catalog.catalog_asset_id,
        AssetRevisionRefV1 {
            asset_id: source.world_activity_catalog.catalog_asset_id,
            record_sha256: source
                .world_activity_catalog
                .revision()
                .map_err(|_| ProjectCookError::InvalidValue)?,
        },
    );
    match (
        source.world_routine_catalog_or_none.as_ref(),
        source.world_routine_interaction_binding_or_none.as_ref(),
    ) {
        (Some(catalog), Some(binding)) => {
            catalog
                .validate()
                .map_err(|_| ProjectCookError::InvalidValue)?;
            binding
                .validate_against(catalog)
                .map_err(|_| ProjectCookError::InvalidValue)?;
            revisions.insert(
                catalog.catalog_asset_id,
                AssetRevisionRefV1 {
                    asset_id: catalog.catalog_asset_id,
                    record_sha256: catalog
                        .revision()
                        .map_err(|_| ProjectCookError::InvalidValue)?,
                },
            );
        }
        (None, None) => {}
        _ => return Err(ProjectCookError::InvalidValue),
    }
    let records: BTreeSet<_> = source
        .records
        .iter()
        .map(|record| record.record_id)
        .collect();
    for record in &source.records {
        NeutralRecordV1::from_canonical_bytes(
            &record.canonical_bytes()?,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )?;
        if record
            .asset_dependencies
            .iter()
            .any(|dependency| !assets.contains(dependency))
            || record
                .persistent_references
                .iter()
                .any(|reference| !records.contains(reference))
        {
            return Err(ProjectCookError::MissingReference);
        }
    }
    for record in &source.render_records {
        let bytes = record.canonical_bytes()?;
        if NeutralRenderRecordV1::from_canonical_bytes(
            &bytes,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )? != record.clone()
        {
            return Err(ProjectCookError::Render(
                RenderContentContractError::NonCanonical,
            ));
        }
        for dependency in record.dependencies() {
            if revisions.get(&dependency.asset_id) != Some(&dependency) {
                return Err(ProjectCookError::MissingReference);
            }
        }
    }
    compile_render_content_catalog_v1(&source.render_records)?;
    for catalog in &source.text_catalogs {
        TextCatalogV1::from_canonical_bytes(
            &catalog.canonical_bytes()?,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )?;
    }
    validate_text_catalog_closure(&source.text_catalogs)?;
    for record in &source.audio_records {
        NeutralAudioV1::from_canonical_bytes(
            &record.canonical_bytes()?,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )?;
    }
    for record in &source.skeletons {
        NeutralSkeletonV1::from_canonical_bytes(
            &record.canonical_bytes()?,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )?;
    }
    for record in &source.animations {
        NeutralAnimationV1::from_canonical_bytes(
            &record.canonical_bytes()?,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )?;
    }
    let body_schema_bytes = source
        .body_schema_asset
        .canonical_bytes()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    if BodySchemaAssetV1::from_canonical_bytes(
        &body_schema_bytes,
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )
    .map_err(|_| ProjectCookError::InvalidValue)?
        != source.body_schema_asset
    {
        return Err(ProjectCookError::InvalidValue);
    }
    let activity_bytes = source
        .world_activity_catalog
        .canonical_bytes()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    if next_contracts::world_activity::WorldActivityCatalogV1::from_canonical_bytes(
        &activity_bytes,
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )
    .map_err(|_| ProjectCookError::InvalidValue)?
        != source.world_activity_catalog
    {
        return Err(ProjectCookError::InvalidValue);
    }
    if !source
        .root_asset_ids
        .contains(&source.body_schema_asset.asset_id)
        || source
            .root_asset_ids
            .iter()
            .any(|asset_id| !assets.contains(asset_id))
    {
        return Err(ProjectCookError::MissingReference);
    }
    for chunk in &source.chunks {
        ensure_unique(chunk.required_asset_ids.iter().copied())?;
        if !assets.contains(&chunk.chunk_asset_id)
            || chunk
                .required_asset_ids
                .iter()
                .any(|asset_id| !assets.contains(asset_id))
        {
            return Err(ProjectCookError::MissingReference);
        }
        let chunk_record = source
            .records
            .iter()
            .find(|record| record.asset_id == chunk.chunk_asset_id)
            .ok_or(ProjectCookError::InvalidValue)?;
        if chunk_record.kind != NeutralRecordKindV1::WorldChunk
            || chunk_record
                .asset_dependencies
                .iter()
                .any(|dependency| !chunk.required_asset_ids.contains(dependency))
        {
            return Err(ProjectCookError::InvalidValue);
        }
    }
    let navigation_nodes = source
        .world_navigation_catalog
        .nodes
        .iter()
        .map(|node| node.chunk_id.as_str())
        .collect::<BTreeSet<_>>();
    let source_chunks = source
        .chunks
        .iter()
        .map(|chunk| chunk.chunk_id.as_str())
        .collect::<BTreeSet<_>>();
    if navigation_nodes != source_chunks
        || source.world_navigation_catalog.nodes.iter().any(|node| {
            source
                .chunks
                .iter()
                .find(|chunk| chunk.chunk_id == node.chunk_id)
                .is_none_or(|chunk| {
                    source
                        .world_navigation_catalog
                        .region_for_node(&node.node_id)
                        != Some(&chunk.region_id)
                })
        })
    {
        return Err(ProjectCookError::InvalidValue);
    }
    Ok(())
}
