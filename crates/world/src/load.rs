use super::*;

pub(super) fn load_one_record(
    generation: &PinnedContentGeneration,
    entry: &ContentAssetEntryV1,
    known_assets: &BTreeSet<AssetId>,
    declared_dependencies: &BTreeMap<AssetId, BTreeSet<AssetId>>,
) -> Result<NeutralRecordV1, WorldAssetLoadErrorCodeV1> {
    let bytes = generation
        .read_content_blob(entry.neutral_record_blob_sha256, WORLD_CHUNK_MAX_BLOB_BYTES)
        .map_err(|_| WorldAssetLoadErrorCodeV1::ContentSource)?;
    let record = NeutralRecordV1::from_canonical_bytes(
        &bytes,
        CanonicalDecodeLimits {
            max_total_bytes: WORLD_CHUNK_MAX_BLOB_BYTES,
            ..CanonicalDecodeLimits::default()
        },
    )
    .map_err(|_| WorldAssetLoadErrorCodeV1::Decode)?;
    validate_record(entry, &record, known_assets, declared_dependencies)?;
    Ok(record)
}

pub(super) fn validate_record(
    entry: &ContentAssetEntryV1,
    record: &NeutralRecordV1,
    known_assets: &BTreeSet<AssetId>,
    declared_dependencies: &BTreeMap<AssetId, BTreeSet<AssetId>>,
) -> Result<(), WorldAssetLoadErrorCodeV1> {
    if entry.semantic_class != ContentSemanticClassV1::DomainRelevant {
        return Err(WorldAssetLoadErrorCodeV1::SemanticClass);
    }
    if record.schema_ref != entry.schema_ref {
        return Err(WorldAssetLoadErrorCodeV1::Schema);
    }
    if record.asset_id != entry.asset_revision.asset_id {
        return Err(WorldAssetLoadErrorCodeV1::AssetIdentity);
    }
    if record
        .record_sha256()
        .map_err(|_| WorldAssetLoadErrorCodeV1::Decode)?
        != entry.asset_revision.record_sha256
        || entry.neutral_record_blob_sha256 != entry.asset_revision.record_sha256
    {
        return Err(WorldAssetLoadErrorCodeV1::RecordRevision);
    }
    if record
        .asset_dependencies
        .iter()
        .any(|dependency| !known_assets.contains(dependency))
    {
        return Err(WorldAssetLoadErrorCodeV1::MissingDependency);
    }
    let actual_dependencies = record
        .asset_dependencies
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if declared_dependencies
        .get(&record.asset_id)
        .is_none_or(|expected| expected != &actual_dependencies)
    {
        return Err(WorldAssetLoadErrorCodeV1::MissingDependency);
    }
    Ok(())
}

pub(super) fn map_store_error(
    asset_id: AssetId,
    _error: &ContentStoreError,
) -> WorldStreamingError {
    WorldStreamingError::AssetLoad {
        asset_id,
        code: WorldAssetLoadErrorCodeV1::ContentSource,
    }
}

pub(super) fn content_entries(
    project: &ActivatedProjectV6,
) -> BTreeMap<AssetId, &ContentAssetEntryV1> {
    project
        .content_manifest
        .body
        .asset_entries
        .iter()
        .map(|entry| (entry.asset_revision.asset_id, entry))
        .collect()
}

pub(super) fn declared_dependencies(
    project: &ActivatedProjectV6,
) -> BTreeMap<AssetId, BTreeSet<AssetId>> {
    let mut dependencies = project
        .content_manifest
        .body
        .asset_entries
        .iter()
        .map(|entry| (entry.asset_revision.asset_id, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &project.content_manifest.body.dependency_edges {
        if edge.required
            && let Some(entry_dependencies) = dependencies.get_mut(&edge.source_asset_id)
        {
            entry_dependencies.insert(edge.target_asset_id);
        }
    }
    dependencies
}

pub(super) fn eager_chunk_records(
    project: &ActivatedProjectV6,
    chunk_id: &SchemaId,
) -> Result<Vec<NeutralRecordV1>, WorldStreamingError> {
    let binding = project
        .world_partition
        .body
        .chunk_bindings
        .binary_search_by(|binding| binding.chunk_id.cmp(chunk_id))
        .map(|index| &project.world_partition.body.chunk_bindings[index])
        .map_err(|_| WorldStreamingError::UnknownChunk)?;
    let ids = binding_asset_ids(binding);
    let records = project
        .neutral_records
        .iter()
        .map(|record| (record.asset_id, record))
        .collect::<BTreeMap<_, _>>();
    ids.into_iter()
        .map(|asset_id| {
            records
                .get(&asset_id)
                .map(|record| (*record).clone())
                .ok_or(WorldStreamingError::RequiredAssetUnavailable)
        })
        .collect()
}

pub(super) fn binding_asset_ids(binding: &WorldChunkBindingV1) -> Vec<AssetId> {
    let mut ids = binding.required_asset_ids.clone();
    ids.push(binding.chunk_asset.asset_id);
    ids.sort_unstable();
    ids.dedup();
    ids
}

pub(super) fn chunk_definition(
    chunk: &WorldChunkResidencyRecordV1,
) -> (&SchemaId, AssetRevisionRefV1, &[AssetId]) {
    (
        &chunk.chunk_id,
        chunk.chunk_asset,
        &chunk.required_asset_ids,
    )
}

pub(super) fn extend_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), WorldStreamingError> {
    bytes.extend_from_slice(
        &u32::try_from(count)
            .map_err(|_| WorldStreamingError::Overflow)?
            .to_le_bytes(),
    );
    Ok(())
}

pub(super) fn extend_text(bytes: &mut Vec<u8>, text: &str) -> Result<(), WorldStreamingError> {
    extend_count(bytes, text.len())?;
    bytes.extend_from_slice(text.as_bytes());
    Ok(())
}
